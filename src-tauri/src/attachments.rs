//! User-selected originals. No command accepts an operating-system path.
use crate::workspace::{
    AppError, ApplicationService, AssetDigest, CourseManager, ErrorCode, OperationId,
    RegisterAsset, RequestContext, WorkspaceId,
};
use serde::{Deserialize, Serialize};
use std::{fs::File, path::Path};
use tauri::State;
use tauri_plugin_dialog::DialogExt;

const MAX_SOURCE_BYTES: u64 = 64 * 1024 * 1024;

#[tauri::command]
pub async fn sources(
    manager: State<'_, CourseManager>,
    course_id: uuid::Uuid,
    resource_id: crate::organization::ResourceId,
) -> Result<crate::workspace::SourceList, AppError> {
    let service = manager.service(course_id)?;
    service
        .sources(RequestContext::local_ui(), resource_id)
        .await
}

#[tauri::command]
pub async fn save_source(
    manager: State<'_, CourseManager>,
    course_id: uuid::Uuid,
    request: crate::workspace::SaveSource,
) -> Result<crate::workspace::OriginalSource, AppError> {
    let service = manager.service(course_id)?;
    service
        .save_source(RequestContext::local_ui(), request)
        .await
}

#[tauri::command]
pub async fn source_preview(
    manager: State<'_, CourseManager>,
    course_id: uuid::Uuid,
    source_id: uuid::Uuid,
) -> Result<crate::workspace::SourcePreview, AppError> {
    let service = manager.service(course_id)?;
    service
        .source_preview(RequestContext::local_ui(), source_id)
        .await
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SelectSource {
    contract_version: u32,
    workspace_id: WorkspaceId,
    operation_id: OperationId,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedSource {
    digest: AssetDigest,
    byte_length: u64,
    filename: String,
}

fn invalid() -> AppError {
    AppError::new(ErrorCode::InvalidRequest)
}

#[tauri::command]
pub async fn pick_source<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    manager: State<'_, CourseManager>,
    course_id: uuid::Uuid,
    request: SelectSource,
) -> Result<Option<SelectedSource>, AppError> {
    let service = manager.service(course_id)?;
    validate_selection(&service, &request).await?;
    let selected =
        tauri::async_runtime::spawn_blocking(move || app.dialog().file().blocking_pick_file())
            .await
            .map_err(|_| AppError::new(ErrorCode::OwnerUnavailable))?;
    let Some(selected) = selected else {
        return Ok(None);
    };
    let path = selected.into_path().map_err(|_| invalid())?;
    preserve_selection(&service, request, &path).await.map(Some)
}

async fn validate_selection(
    service: &ApplicationService,
    request: &SelectSource,
) -> Result<(), AppError> {
    if request.contract_version != 1 {
        return Err(invalid());
    }
    let status = service.status(RequestContext::local_ui()).await?;
    if status.preferences.workspace_id != request.workspace_id {
        return Err(AppError::new(ErrorCode::PermissionDenied));
    }
    Ok(())
}

fn open_selected(path: &Path) -> Result<(File, String), AppError> {
    use rustix::fs::{CWD, Mode, OFlags, ResolveFlags, openat2};
    if !path.is_absolute() {
        return Err(invalid());
    }
    let filename = selection_name(path)?;
    // Resolve all components without following links; NONBLOCK prevents a FIFO
    // selection from blocking before the regular-file metadata check.
    let fd = openat2(
        CWD,
        path,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK,
        Mode::empty(),
        ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS,
    )
    .map_err(|_| invalid())?;
    let file = File::from(fd);
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > MAX_SOURCE_BYTES {
        return Err(invalid());
    }
    Ok((file, filename))
}

async fn preserve_selection(
    service: &ApplicationService,
    request: SelectSource,
    path: &Path,
) -> Result<SelectedSource, AppError> {
    let (file, filename) = open_selected(path)?;
    let info = service
        .register_asset(
            RequestContext::local_ui(),
            RegisterAsset {
                contract_version: 1,
                workspace_id: request.workspace_id,
                operation_id: request.operation_id,
            },
            Box::new(file),
            MAX_SOURCE_BYTES,
        )
        .await?;
    Ok(SelectedSource {
        digest: info.digest,
        byte_length: info.byte_length,
        filename,
    })
}

fn selection_name(path: &Path) -> Result<String, AppError> {
    let name = path
        .file_name()
        .and_then(|v| v.to_str())
        .ok_or_else(invalid)?;
    if name.is_empty()
        || name.chars().count() > 256
        || name
            .chars()
            .any(|c| c.is_control() || c == '/' || c == '\\')
    {
        return Err(invalid());
    }
    Ok(name.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Read, os::unix::fs::symlink};

    #[test]
    fn selected_copy_is_durable_and_retry_never_replaces_the_original() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        let path = temp.path().join("original.txt");
        std::fs::write(&path, "original").unwrap();
        tauri::async_runtime::block_on(async {
            let service = ApplicationService::start(root.clone()).unwrap();
            let workspace = service
                .status(RequestContext::local_ui())
                .await
                .unwrap()
                .preferences
                .workspace_id;
            let operation = OperationId::generate();
            let request = || SelectSource {
                contract_version: 1,
                workspace_id: workspace.clone(),
                operation_id: operation.clone(),
            };
            let original = preserve_selection(&service, request(), &path)
                .await
                .unwrap();
            assert_eq!(original.filename, "original.txt");
            assert_eq!(original.byte_length, 8);
            assert_eq!(
                preserve_selection(&service, request(), &path)
                    .await
                    .unwrap()
                    .digest,
                original.digest
            );
            std::fs::write(&path, "changed").unwrap();
            assert_eq!(
                preserve_selection(&service, request(), &path)
                    .await
                    .err()
                    .unwrap()
                    .code,
                ErrorCode::IdempotencyConflict
            );
            service.shutdown().await.unwrap();
            let reopened = ApplicationService::start(root).unwrap();
            let receipt = reopened
                .operation_receipt(RequestContext::local_ui(), operation)
                .await
                .unwrap()
                .unwrap();
            let crate::workspace::ReceiptResult::Applied(
                crate::workspace::OperationOutcome::Asset(info),
            ) = receipt.result
            else {
                panic!("original asset receipt required")
            };
            assert_eq!(info.digest, original.digest);
            assert_eq!(info.byte_length, 8);
            reopened.shutdown().await.unwrap();
        });
    }

    #[test]
    fn selected_regular_file_retains_exact_original_bytes_and_basename() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("original.html");
        let original = "<script>throw new Error('inert original')</script>\nλ";
        std::fs::write(&path, original).unwrap();
        let (mut file, name) = open_selected(&path).unwrap();
        let mut bytes = String::new();
        file.read_to_string(&mut bytes).unwrap();
        assert_eq!(bytes, original);
        assert_eq!(name, "original.html");
    }

    #[test]
    fn source_selection_rejects_symlinks_in_every_component_and_special_files() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("original");
        std::fs::write(&path, "data").unwrap();
        let link = temp.path().join("file-link");
        symlink(&path, &link).unwrap();
        assert!(open_selected(&link).is_err());
        let parent = temp.path().join("parent-link");
        symlink(temp.path(), &parent).unwrap();
        assert!(open_selected(&parent.join("original")).is_err());
        assert!(open_selected(temp.path()).is_err());
        let fifo = temp.path().join("fifo");
        rustix::fs::mkfifoat(
            rustix::fs::CWD,
            &fifo,
            rustix::fs::Mode::RUSR | rustix::fs::Mode::WUSR,
        )
        .unwrap();
        assert!(open_selected(&fifo).is_err());
        assert!(open_selected(Path::new("/dev/null")).is_err());
    }

    #[test]
    fn source_selection_rejects_oversized_files_and_unsafe_original_names() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("large");
        File::create(&path)
            .unwrap()
            .set_len(MAX_SOURCE_BYTES + 1)
            .unwrap();
        assert!(open_selected(&path).is_err());
        File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_len(MAX_SOURCE_BYTES)
            .unwrap();
        assert!(open_selected(&path).is_ok());
        for name in ["line\nbreak", "path\\spoof"] {
            let path = temp.path().join(name);
            std::fs::write(&path, "data").unwrap();
            assert!(open_selected(&path).is_err());
        }
    }
}
