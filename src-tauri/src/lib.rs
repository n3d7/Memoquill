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
        .setup(initialize_workspace)
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

fn initialize_workspace(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let service = start_workspace(app)?;
    app.manage(service);
    app.manage(Shutdown(AtomicU8::new(0)));
    Ok(())
}
fn start_workspace(app: &tauri::App) -> Result<ApplicationService, Box<dyn std::error::Error>> {
    let root = app.path().app_local_data_dir()?.join("workspace");
    ApplicationService::start(root).map_err(Into::into)
}
struct Shutdown(AtomicU8);
fn handle_event(app: &tauri::AppHandle, event: tauri::RunEvent) {
    if let tauri::RunEvent::ExitRequested { api, .. } = event {
        prevent_until_drained(app, api);
    }
}
fn prevent_until_drained(app: &tauri::AppHandle, api: tauri::ExitRequestApi) {
    if app.state::<Shutdown>().0.load(Ordering::Acquire) == 2 {
        return;
    }
    api.prevent_exit();
    start_draining(app);
}
fn start_draining(app: &tauri::AppHandle) {
    if app
        .state::<Shutdown>()
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
        let response = ipc(
            &webview,
            command,
            origin,
            serde_json::json!({"draft":serde_json::from_str::<serde_json::Value>(include_str!("../../tests/fixtures/draft.json")).unwrap()}),
        );
        tauri::async_runtime::block_on(service.shutdown()).unwrap();
        response
    }

    fn ipc(
        webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
        command: &str,
        origin: &str,
        body: serde_json::Value,
    ) -> Result<tauri::ipc::InvokeResponseBody, serde_json::Value> {
        get_ipc_response(
            webview,
            InvokeRequest {
                cmd: command.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: origin.parse().unwrap(),
                body: InvokeBody::Json(body),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.into(),
            },
        )
    }
    #[test]
    fn native_preference_save_and_receipt_round_trip_preserves_authority() {
        let temp = tempfile::tempdir().unwrap();
        let service = ApplicationService::start(temp.path().join("workspace")).unwrap();
        let initial =
            tauri::async_runtime::block_on(service.status(RequestContext::local_ui())).unwrap();
        let app = mock_builder()
            .manage(service.clone())
            .invoke_handler(tauri::generate_handler![
                save_reading_preferences,
                operation_receipt
            ])
            .build(tauri::generate_context!())
            .unwrap();
        let main = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        let secondary = tauri::WebviewWindowBuilder::new(&app, "secondary", Default::default())
            .build()
            .unwrap();
        let request = SavePreferences {
            contract_version: 1,
            workspace_id: initial.preferences.workspace_id,
            operation_id: OperationId::generate(),
            expected_revision: 0,
            preferences: workspace::ReadingPreferences {
                theme: workspace::Theme::Dark,
                text_scale: workspace::TextScale::try_from(150).unwrap(),
            },
        };
        let body = serde_json::json!({"request":request});
        let result = ipc(
            &main,
            "save_reading_preferences",
            "tauri://localhost",
            body.clone(),
        )
        .unwrap()
        .deserialize::<PreferencesSnapshot>()
        .unwrap();
        assert_eq!(result.revision, 1);
        assert_eq!(result.preferences, request.preferences);
        let receipt_body = serde_json::json!({"operationId":request.operation_id});
        let receipt = ipc(
            &main,
            "operation_receipt",
            "tauri://localhost",
            receipt_body.clone(),
        )
        .unwrap()
        .deserialize::<Option<OperationReceipt>>()
        .unwrap()
        .unwrap();
        assert_eq!(
            receipt.result,
            workspace::ReceiptResult::Applied(workspace::OperationOutcome::Preferences(result))
        );
        assert!(
            ipc(
                &secondary,
                "save_reading_preferences",
                "tauri://localhost",
                body.clone()
            )
            .is_err()
        );
        assert!(
            ipc(
                &main,
                "save_reading_preferences",
                "https://example.com",
                body.clone()
            )
            .is_err()
        );
        assert!(
            ipc(
                &secondary,
                "operation_receipt",
                "tauri://localhost",
                receipt_body
            )
            .is_err()
        );
        let mut forged = body;
        forged["request"]["principal"] = serde_json::json!("local-ui");
        assert!(
            ipc(
                &main,
                "save_reading_preferences",
                "tauri://localhost",
                forged
            )
            .is_err()
        );
        assert_eq!(
            tauri::async_runtime::block_on(service.status(RequestContext::local_ui()))
                .unwrap()
                .preferences
                .revision,
            1
        );
        tauri::async_runtime::block_on(service.shutdown()).unwrap();
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
