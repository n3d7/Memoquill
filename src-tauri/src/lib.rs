pub mod draft;
pub mod workspace;

use std::sync::atomic::{AtomicU8, Ordering};
use tauri::Manager;
use workspace::{
    AppError, ApplicationService, OperationId, OperationReceipt, PreferencesSnapshot,
    RequestContext, SavePreferences, WorkspaceStatus,
};

#[tauri::command]
async fn workspace_status(
    service: tauri::State<'_, ApplicationService>,
) -> Result<WorkspaceStatus, AppError> {
    service.status(RequestContext::local_ui()).await
}
#[tauri::command]
async fn save_reading_preferences(
    service: tauri::State<'_, ApplicationService>,
    request: SavePreferences,
) -> Result<PreferencesSnapshot, AppError> {
    service
        .save_preferences(RequestContext::local_ui(), request)
        .await
}
#[tauri::command]
async fn operation_receipt(
    service: tauri::State<'_, ApplicationService>,
    operation_id: OperationId,
) -> Result<Option<OperationReceipt>, AppError> {
    service
        .operation_receipt(RequestContext::local_ui(), operation_id)
        .await
}
#[tauri::command]
async fn reopen_workspace(
    service: tauri::State<'_, ApplicationService>,
) -> Result<WorkspaceStatus, AppError> {
    service.reopen(RequestContext::local_ui()).await
}

#[tauri::command]
fn check_draft(draft: draft::Draft) -> Result<draft::DraftSummary, String> {
    draft::check_draft(draft)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let root = app.path().app_local_data_dir()?.join("workspace");
            app.manage(ApplicationService::start(root)?);
            app.manage(Shutdown(AtomicU8::new(0)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            check_draft,
            workspace_status,
            save_reading_preferences,
            operation_receipt,
            reopen_workspace
        ])
        .build(tauri::generate_context!())
        .expect("Memoquill could not start")
        .run(handle_event);
}

struct Shutdown(AtomicU8);
fn handle_event(app: &tauri::AppHandle, event: tauri::RunEvent) {
    let tauri::RunEvent::ExitRequested { api, .. } = event else {
        return;
    };
    let shutdown = app.state::<Shutdown>();
    if shutdown.0.load(Ordering::Acquire) == 2 {
        return;
    }
    api.prevent_exit();
    if shutdown
        .0
        .compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }
    let service = app.state::<ApplicationService>().inner().clone();
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = service.shutdown().await;
        app.state::<Shutdown>().0.store(2, Ordering::Release);
        app.exit(0);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use glib::variant::ToVariant;
    use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder};
    use tauri::{
        ipc::{CallbackFn, InvokeBody},
        webview::InvokeRequest,
    };

    #[tauri::command]
    fn forbidden_probe() -> bool {
        true
    }

    fn invoke(
        command: &str,
        label: &str,
        origin: &str,
    ) -> Result<tauri::ipc::InvokeResponseBody, serde_json::Value> {
        let temp = tempfile::tempdir().unwrap();
        let service = ApplicationService::start(temp.path().join("workspace")).unwrap();
        let app = mock_builder()
            .manage(service.clone())
            .invoke_handler(tauri::generate_handler![
                check_draft,
                workspace_status,
                save_reading_preferences,
                operation_receipt,
                reopen_workspace,
                forbidden_probe
            ])
            .build(tauri::generate_context!())
            .unwrap();
        let webview = tauri::WebviewWindowBuilder::new(&app, label, Default::default())
            .build()
            .unwrap();
        let response = get_ipc_response(
            &webview,
            InvokeRequest {
                cmd: command.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: origin.parse().unwrap(),
                body: InvokeBody::Json(
                    serde_json::json!({"draft": serde_json::from_str::<serde_json::Value>(include_str!("../../tests/fixtures/draft.json")).unwrap()}),
                ),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.into(),
            },
        );
        tauri::async_runtime::block_on(service.shutdown()).unwrap();
        response
    }

    #[test]
    fn main_local_webview_can_check_draft() {
        let result = invoke("check_draft", "main", "tauri://localhost").unwrap();
        let value = result.deserialize::<serde_json::Value>().unwrap();
        assert_eq!(
            value,
            serde_json::json!({"title":"Quantum cooking", "proseCharacters":14, "codeLines":2})
        );
    }

    #[test]
    fn unlisted_registered_command_is_denied() {
        assert!(invoke("forbidden_probe", "main", "tauri://localhost").is_err());
    }

    #[test]
    fn other_windows_and_remote_origins_have_no_draft_authority() {
        assert!(invoke("check_draft", "secondary", "tauri://localhost").is_err());
        assert!(invoke("check_draft", "main", "https://example.com").is_err());
    }

    #[test]
    fn only_main_local_webview_can_read_the_managed_workspace() {
        let result = invoke("workspace_status", "main", "tauri://localhost").unwrap();
        let status = result.deserialize::<workspace::WorkspaceStatus>().unwrap();
        assert_eq!(status.preferences.revision, 0);
        assert_eq!(status.contract_version, 1);
        for command in [
            "workspace_status",
            "save_reading_preferences",
            "operation_receipt",
            "reopen_workspace",
        ] {
            assert!(invoke(command, "secondary", "tauri://localhost").is_err());
            assert!(invoke(command, "main", "https://example.com").is_err());
        }
    }

    #[test]
    fn glib_string_iteration_preserves_values_under_release_optimization() {
        let expected = ["first", "世界", "last"];
        let value = expected.to_variant();
        let mut iterator = value.array_iter_str().unwrap();
        assert_eq!(iterator.next(), Some("first"));
        assert_eq!(iterator.next_back(), Some("last"));
        assert_eq!(iterator.last(), Some("世界"));
    }
}
