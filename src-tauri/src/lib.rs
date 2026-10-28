pub mod draft;

#[tauri::command]
fn check_draft(draft: draft::Draft) -> Result<draft::DraftSummary, String> {
    draft::check_draft(draft)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![check_draft])
        .run(tauri::generate_context!())
        .expect("Memoquill could not start");
}

#[cfg(test)]
mod tests {
    use super::check_draft;
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
        let app = mock_builder()
            .invoke_handler(tauri::generate_handler![check_draft, forbidden_probe])
            .build(tauri::generate_context!())
            .unwrap();
        let webview = tauri::WebviewWindowBuilder::new(&app, label, Default::default())
            .build()
            .unwrap();
        get_ipc_response(
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
        )
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
    fn glib_string_iteration_preserves_values_under_release_optimization() {
        let expected = ["first", "世界", "last"];
        let value = expected.to_variant();
        let mut iterator = value.array_iter_str().unwrap();
        assert_eq!(iterator.next(), Some("first"));
        assert_eq!(iterator.next_back(), Some("last"));
        assert_eq!(iterator.last(), Some("世界"));
    }
}
