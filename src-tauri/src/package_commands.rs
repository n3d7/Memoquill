//! Course-bound native picker capabilities. No command accepts a filesystem path.
use crate::{
    native_files,
    workspace::{
        AppError, CourseManager, ErrorCode, NativeCapture, PortableCapture, RequestContext,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::Seek,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
use tauri::{Manager, State};
use tauri_plugin_dialog::DialogExt;
use uuid::Uuid;
type Result<T> = std::result::Result<T, AppError>;
fn invalid() -> AppError {
    AppError::new(ErrorCode::InvalidRequest)
}
#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PackageKind {
    NativeFull,
    PortableContent,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportRequest {
    kind: PackageKind,
    include_originals: bool,
    #[serde(default)]
    include_history: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConfirmRequest {
    review_id: Uuid,
    operation_id: Uuid,
    title: String,
    allow_incomplete: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageSummary {
    kind: PackageKind,
    manifest: serde_json::Value,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageReview {
    review_id: Uuid,
    #[serde(flatten)]
    summary: PackageSummary,
}
#[derive(Serialize)]
pub struct ImportResult {
    kind: PackageKind,
    report: serde_json::Value,
}
struct LargePackagePermit(Arc<AtomicBool>);

impl LargePackagePermit {
    fn acquire(reservation: &Arc<AtomicBool>) -> Result<Self> {
        reservation
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| AppError::new(ErrorCode::Busy))?;
        Ok(Self(reservation.clone()))
    }
}

impl Drop for LargePackagePermit {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

enum Capture {
    Native {
        capture: Box<NativeCapture>,
        _permit: Option<LargePackagePermit>,
    },
    Portable(Box<PortableCapture>),
}
impl Capture {
    fn native(capture: NativeCapture, permit: Option<LargePackagePermit>) -> Self {
        Self::Native {
            capture: Box::new(capture),
            _permit: permit,
        }
    }

    fn summary(&self) -> Result<PackageSummary> {
        match self {
            Self::Native { capture: c, .. } => Ok(PackageSummary {
                kind: PackageKind::NativeFull,
                manifest: serde_json::to_value(c.manifest()).map_err(|_| invalid())?,
            }),
            Self::Portable(c) => Ok(PackageSummary {
                kind: PackageKind::PortableContent,
                manifest: serde_json::to_value(c.manifest()).map_err(|_| invalid())?,
            }),
        }
    }
    fn write(&self, file: &mut std::fs::File) -> Result<()> {
        match self {
            Self::Native { capture: c, .. } => c.write_selected_file(file),
            Self::Portable(c) => c.write_to(file),
        }
    }
}
struct Review {
    course: Uuid,
    capture: Arc<Capture>,
}
#[derive(Default)]
pub struct PackageReviews(Mutex<HashMap<Uuid, Review>>, Arc<AtomicBool>);
#[tauri::command]
pub async fn save_original<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    manager: State<'_, CourseManager>,
    course_id: Uuid,
    source_id: Uuid,
) -> Result<Option<bool>> {
    let service = manager.service(course_id)?;
    let managed = manager.managed_root().to_owned();
    let operation = Uuid::new_v4();
    // Verified immutable bytes and identity are captured before the picker awaits.
    let (name, bytes) = service
        .original_copy(RequestContext::local_ui(), source_id)
        .await?;
    let name = native_files::suggested_name(&name);
    tauri::async_runtime::spawn_blocking(move || {
        let path = app
            .dialog()
            .file()
            .set_file_name(name)
            .blocking_save_file()
            .map(|v| v.into_path().map_err(|_| invalid()))
            .transpose()?;
        native_files::publish_selection(&managed, path, operation, |file| {
            use std::io::Write;
            file.write_all(&bytes)?;
            Ok(())
        })
    })
    .await
    .map_err(|_| AppError::new(ErrorCode::OwnerUnavailable))?
}
#[tauri::command]
pub async fn package_export<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    manager: State<'_, CourseManager>,
    reviews: State<'_, PackageReviews>,
    course_id: Uuid,
    request: ExportRequest,
) -> Result<Option<PackageSummary>> {
    let service = manager.service(course_id)?;
    let managed = manager.managed_root().to_owned();
    let operation = Uuid::new_v4();
    let capture = match request.kind {
        PackageKind::NativeFull => {
            let context = RequestContext::local_ui();
            let (capture, permit) = match service.capture_native(context, course_id).await {
                Ok(capture) => (capture, None),
                Err(error) if error.code == ErrorCode::BudgetExceeded => {
                    let permit = LargePackagePermit::acquire(&reviews.1)?;
                    let capture = service
                        .capture_native_stream(
                            context,
                            course_id,
                            CourseManager::package_staging_root(&managed),
                        )
                        .await?;
                    (capture, Some(permit))
                }
                Err(error) => return Err(error),
            };
            Capture::native(capture, permit)
        }
        PackageKind::PortableContent => Capture::Portable(Box::new(
            service
                .capture_portable_with_history(
                    RequestContext::local_ui(),
                    course_id,
                    request.include_originals,
                    request.include_history,
                )
                .await?,
        )),
    };
    let summary = capture.summary()?;
    tauri::async_runtime::spawn_blocking(move || {
        let name = match request.kind {
            PackageKind::NativeFull if summary.manifest["formatVersion"] == 2 => {
                "course-recovery-v2.memoquill"
            }
            PackageKind::NativeFull => "course-recovery.memoquill",
            PackageKind::PortableContent => "course-content.memoquill",
        };
        let path = app
            .dialog()
            .file()
            .set_file_name(name)
            .blocking_save_file()
            .map(|v| v.into_path().map_err(|_| invalid()))
            .transpose()?;
        let max_bytes = if summary.manifest["formatVersion"] == 2
            && matches!(request.kind, PackageKind::NativeFull)
        {
            NativeCapture::MAX_STREAM_FILE_BYTES
        } else {
            NativeCapture::MAX_LEGACY_FILE_BYTES
        };
        match native_files::publish_selection_with_limit(
            &managed,
            path,
            operation,
            max_bytes,
            |file| capture.write(file),
        )? {
            Some(_) => Ok(Some(summary)),
            None => Ok(None),
        }
    })
    .await
    .map_err(|_| AppError::new(ErrorCode::OwnerUnavailable))?
}
fn read_capture(
    path: &std::path::Path,
    managed: &std::path::Path,
    reservation: &Arc<AtomicBool>,
) -> Result<Capture> {
    let mut file = native_files::open_package(path)?;
    let large = file.metadata()?.len() > NativeCapture::MAX_LEGACY_FILE_BYTES;
    let (permit, wire_limit) = if large {
        (
            Some(LargePackagePermit::acquire(reservation)?),
            NativeCapture::MAX_STREAM_FILE_BYTES,
        )
    } else {
        (None, NativeCapture::MAX_LEGACY_FILE_BYTES)
    };
    if let Some(capture) = NativeCapture::read_stream_in(
        &mut file,
        &CourseManager::package_staging_root(managed),
        wire_limit,
    )? {
        return Ok(Capture::native(capture, permit));
    }
    if large {
        return Err(invalid());
    }
    // Dispatch only through the bounded backend readers. Do not let ZipArchive
    // allocate an unbounded central directory merely to probe the manifest kind.
    if let Ok(capture) = NativeCapture::read_zip_from(&mut file) {
        return Ok(Capture::native(capture, None));
    }
    file.rewind()?;
    PortableCapture::read_from(file).map(|capture| Capture::Portable(Box::new(capture)))
}
#[tauri::command]
pub async fn package_inspect<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    manager: State<'_, CourseManager>,
    reviews: State<'_, PackageReviews>,
    course_id: Uuid,
) -> Result<Option<PackageReview>> {
    let _captured = manager.service(course_id)?;
    let managed = manager.managed_root().to_owned();
    let reservation = reviews.1.clone();
    let capture = tauri::async_runtime::spawn_blocking(move || {
        let Some(selected) = app.dialog().file().blocking_pick_file() else {
            return Ok(None);
        };
        let path = selected.into_path().map_err(|_| invalid())?;
        read_capture(&path, &managed, &reservation).map(Some)
    })
    .await
    .map_err(|_| AppError::new(ErrorCode::OwnerUnavailable))??;
    let Some(capture) = capture else {
        return Ok(None);
    };
    let summary = capture.summary()?;
    let review_id = Uuid::new_v4();
    let mut reviews = reviews.0.lock().map_err(|_| invalid())?;
    if reviews.len() >= 8 {
        return Err(AppError::new(ErrorCode::BudgetExceeded));
    }
    reviews.insert(
        review_id,
        Review {
            course: course_id,
            capture: Arc::new(capture),
        },
    );
    Ok(Some(PackageReview { review_id, summary }))
}
fn reviewed(reviews: &PackageReviews, course: Uuid, id: Uuid) -> Result<Arc<Capture>> {
    let reviews = reviews.0.lock().map_err(|_| invalid())?;
    let review = reviews.get(&id).ok_or_else(invalid)?;
    if review.course != course {
        return Err(AppError::new(ErrorCode::PermissionDenied));
    }
    Ok(review.capture.clone())
}
#[tauri::command]
pub async fn package_confirm<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    manager: State<'_, CourseManager>,
    reviews: State<'_, PackageReviews>,
    course_id: Uuid,
    request: ConfirmRequest,
) -> Result<ImportResult> {
    let _captured = manager.service(course_id)?;
    let capture = reviewed(&reviews, course_id, request.review_id)?;
    // Keep review for exact retries after an uncertain response. Durable backend binds UUID/title/hash.
    tauri::async_runtime::spawn_blocking(move || {
        let manager = app.state::<CourseManager>();
        match capture.as_ref() {
            Capture::Native { capture: c, .. } => Ok(ImportResult {
                kind: PackageKind::NativeFull,
                report: serde_json::to_value(manager.restore_native(
                    request.operation_id,
                    request.title,
                    c,
                    request.allow_incomplete,
                )?)
                .map_err(|_| invalid())?,
            }),
            Capture::Portable(c) => Ok(ImportResult {
                kind: PackageKind::PortableContent,
                report: serde_json::to_value(manager.import_portable(
                    request.operation_id,
                    request.title,
                    c,
                )?)
                .map_err(|_| invalid())?,
            }),
        }
    })
    .await
    .map_err(|_| AppError::new(ErrorCode::OwnerUnavailable))?
}
#[tauri::command]
pub fn package_discard(
    reviews: State<'_, PackageReviews>,
    course_id: Uuid,
    review_id: Uuid,
) -> Result<()> {
    let mut reviews = reviews.0.lock().map_err(|_| invalid())?;
    if let Some(review) = reviews.get(&review_id)
        && review.course != course_id
    {
        return Err(AppError::new(ErrorCode::PermissionDenied));
    }
    reviews.remove(&review_id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder};
    use tauri::{
        Manager,
        ipc::{CallbackFn, InvokeBody},
        webview::InvokeRequest,
    };
    #[test]
    fn real_package_ipc_admits_new_course_and_rejects_foreign_surfaces_and_review_rebinding() {
        let temp = tempfile::tempdir().unwrap();
        let manager = CourseManager::open(temp.path().to_owned()).unwrap();
        let course = manager.list().unwrap().selected_course_id;
        let service = manager.service(course).unwrap();
        let capture = tauri::async_runtime::block_on(
            service.capture_native(RequestContext::local_ui(), course),
        )
        .unwrap();
        let review_id = Uuid::new_v4();
        let reviews = PackageReviews::default();
        reviews.0.lock().unwrap().insert(
            review_id,
            Review {
                course,
                capture: Arc::new(Capture::native(capture, None)),
            },
        );
        let other_course = Uuid::new_v4();
        manager.create(other_course, "Other".into()).unwrap();
        let app = mock_builder()
            .manage(manager)
            .manage(reviews)
            .invoke_handler(tauri::generate_handler![
                package_confirm,
                package_discard,
                package_export,
                package_inspect,
                save_original
            ])
            .build(tauri::generate_context!())
            .unwrap();
        let main = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        let other = tauri::WebviewWindowBuilder::new(&app, "foreign", Default::default())
            .build()
            .unwrap();
        let call = |window: &tauri::WebviewWindow<tauri::test::MockRuntime>,
                    cmd: &str,
                    url: &str,
                    body: serde_json::Value| {
            get_ipc_response(
                window,
                InvokeRequest {
                    cmd: cmd.into(),
                    callback: CallbackFn(0),
                    error: CallbackFn(1),
                    url: url.parse().unwrap(),
                    body: InvokeBody::Json(body),
                    headers: Default::default(),
                    invoke_key: INVOKE_KEY.into(),
                },
            )
            .map(|v| v.deserialize::<serde_json::Value>().unwrap())
        };
        let operation = Uuid::new_v4();
        let body = json!({"courseId":course,"request":{"reviewId":review_id,"operationId":operation,"title":"Restored via IPC","allowIncomplete":false}});
        assert!(call(&other, "package_confirm", "tauri://localhost", body.clone()).is_err());
        assert!(
            call(
                &main,
                "package_confirm",
                "https://untrusted.example",
                body.clone()
            )
            .is_err()
        );
        let mut forged = body.clone();
        forged["courseId"] = json!(other_course);
        assert_eq!(
            call(&main, "package_confirm", "tauri://localhost", forged).unwrap_err()["code"],
            "permission_denied"
        );
        let result = call(&main, "package_confirm", "tauri://localhost", body.clone()).unwrap();
        assert_eq!(result["report"]["courseId"], operation.to_string());
        assert_eq!(result["report"]["sourceCourseId"], course.to_string());
        assert_eq!(
            call(&main, "package_confirm", "tauri://localhost", body).unwrap(),
            result
        );
        for (command, request) in [
            (
                "package_export",
                json!({"request":{"kind":"nativeFull","includeOriginals":false}}),
            ),
            ("package_inspect", json!({})),
            ("save_original", json!({"sourceId":Uuid::new_v4()})),
        ] {
            let mut body = request;
            body["courseId"] = json!(Uuid::new_v4());
            let error = call(&main, command, "tauri://localhost", body.clone()).unwrap_err();
            assert!(
                error.get("code").is_some(),
                "{command} must reach its typed handler through real ACL: {error}"
            );
            assert!(call(&other, command, "tauri://localhost", body.clone()).is_err());
            assert!(call(&main, command, "https://untrusted.example", body).is_err());
        }
        let discard = json!({"courseId":course,"reviewId":review_id});
        assert!(
            call(
                &other,
                "package_discard",
                "tauri://localhost",
                discard.clone()
            )
            .is_err()
        );
        call(&main, "package_discard", "tauri://localhost", discard).unwrap();
        assert_eq!(
            app.state::<CourseManager>().list().unwrap().courses.len(),
            3
        );
        tauri::async_runtime::block_on(app.state::<CourseManager>().shutdown()).unwrap();
    }
    #[test]
    fn captured_native_and_portable_packages_publish_and_inspect_without_path_authority() {
        let temp = tempfile::tempdir().unwrap();
        let managed = temp.path().join("managed");
        let manager = CourseManager::open(managed.clone()).unwrap();
        let course = manager.list().unwrap().selected_course_id;
        let service = manager.service(course).unwrap();
        for capture in [
            Capture::native(
                tauri::async_runtime::block_on(
                    service.capture_native(RequestContext::local_ui(), course),
                )
                .unwrap(),
                None,
            ),
            Capture::Portable(Box::new(
                tauri::async_runtime::block_on(service.capture_portable(
                    RequestContext::local_ui(),
                    course,
                    false,
                ))
                .unwrap(),
            )),
        ] {
            manager
                .create(Uuid::new_v4(), "Switched while picker was open".into())
                .unwrap();
            assert_ne!(manager.list().unwrap().selected_course_id, course);
            let out = temp.path().join("selected.memoquill");
            native_files::publish(&managed, &out, |f| capture.write(f)).unwrap();
            assert_eq!(
                read_capture(&out, &managed, &Arc::default())
                    .unwrap()
                    .summary()
                    .unwrap()
                    .manifest,
                capture.summary().unwrap().manifest
            );
            assert!(
                native_files::publish(&managed, &managed.join("courses/catalogue.sqlite3"), |f| {
                    capture.write(f)
                })
                .is_err()
            );
        }
        tauri::async_runtime::block_on(manager.shutdown()).unwrap();
    }

    #[test]
    fn streaming_package_inspection_uses_managed_staging_and_preserves_version_errors() {
        let temp = tempfile::tempdir().unwrap();
        let managed = temp.path().join("managed");
        let manager = CourseManager::open(managed.clone()).unwrap();
        let course = manager.list().unwrap().selected_course_id;
        let service = manager.service(course).unwrap();
        let snapshot = tauri::async_runtime::block_on(
            service.capture_native(RequestContext::local_ui(), course),
        )
        .unwrap();
        let out = temp.path().join("selected-stream.memoquill");
        native_files::publish(&managed, &out, |file| snapshot.write_stream_to(file)).unwrap();
        let staged_count = || {
            std::fs::read_dir(CourseManager::package_staging_root(&managed))
                .unwrap()
                .filter(|entry| {
                    entry
                        .as_ref()
                        .unwrap()
                        .file_name()
                        .as_encoded_bytes()
                        .starts_with(b"memoquill-stream-")
                })
                .count()
        };
        let inspected = read_capture(&out, &managed, &Arc::default()).unwrap();
        assert_eq!(inspected.summary().unwrap().manifest["formatVersion"], 2);
        assert_eq!(staged_count(), 1);
        drop(inspected);
        assert_eq!(staged_count(), 0);

        let mut corrupt = std::fs::read(&out).unwrap();
        corrupt[8..12].copy_from_slice(&3u32.to_le_bytes());
        std::fs::write(&out, corrupt).unwrap();
        assert_eq!(
            read_capture(&out, &managed, &Arc::default())
                .err()
                .unwrap()
                .code,
            ErrorCode::UnsupportedVersion
        );
        assert_eq!(staged_count(), 0);
    }

    #[test]
    fn large_package_selection_reserves_staging_and_rejects_non_streams() {
        let temp = tempfile::tempdir().unwrap();
        let managed = temp.path().join("managed");
        let _manager = CourseManager::open(managed.clone()).unwrap();
        let path = temp.path().join("oversized-zip.memoquill");
        let file = std::fs::File::create(&path).unwrap();
        file.set_len(NativeCapture::MAX_LEGACY_FILE_BYTES + 1)
            .unwrap();
        drop(file);
        let reservation = Arc::default();
        let permit = LargePackagePermit::acquire(&reservation).unwrap();
        assert_eq!(
            read_capture(&path, &managed, &reservation)
                .err()
                .unwrap()
                .code,
            ErrorCode::Busy
        );
        drop(permit);
        assert_eq!(
            read_capture(&path, &managed, &reservation)
                .err()
                .unwrap()
                .code,
            ErrorCode::InvalidRequest
        );
        assert!(LargePackagePermit::acquire(&reservation).is_ok());
    }
}
