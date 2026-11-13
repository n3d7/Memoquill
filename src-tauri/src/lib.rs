mod attachments;
mod authoring;
mod course_commands;
pub mod document;
pub mod draft;
mod ingestion_commands;
mod lesson_commands;
mod native_files;
pub mod organization;
mod package_commands;
pub mod workspace;

use std::sync::atomic::{AtomicU8, Ordering};
use tauri::Manager;
use workspace::{
    AppError, CourseManager, OperationId, OperationReceipt, PreferencesSnapshot, RequestContext,
    SavePreferences, WorkspaceStatus,
};

#[tauri::command]
async fn workspace_status(
    manager: tauri::State<'_, CourseManager>,
    course_id: uuid::Uuid,
) -> Result<WorkspaceStatus, AppError> {
    let service = manager.service(course_id)?;
    service.status(RequestContext::local_ui()).await
}
#[tauri::command]
async fn save_reading_preferences(
    manager: tauri::State<'_, CourseManager>,
    course_id: uuid::Uuid,
    request: SavePreferences,
) -> Result<PreferencesSnapshot, AppError> {
    let service = manager.service(course_id)?;
    service
        .save_preferences(RequestContext::local_ui(), request)
        .await
}
#[tauri::command]
async fn operation_receipt(
    manager: tauri::State<'_, CourseManager>,
    course_id: uuid::Uuid,
    operation_id: OperationId,
) -> Result<Option<OperationReceipt>, AppError> {
    let service = manager.service(course_id)?;
    service
        .operation_receipt(RequestContext::local_ui(), operation_id)
        .await
}
#[tauri::command]
async fn reopen_workspace(
    manager: tauri::State<'_, CourseManager>,
    course_id: uuid::Uuid,
) -> Result<WorkspaceStatus, AppError> {
    let service = manager.service(course_id)?;
    service.reopen(RequestContext::local_ui()).await
}

#[tauri::command]
fn check_draft(draft: draft::Draft) -> Result<draft::DraftSummary, String> {
    draft::check_draft(draft)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(package_commands::PackageReviews::default())
        .setup(initialize_workspace)
        .invoke_handler(tauri::generate_handler![
            course_commands::courses_list,
            course_commands::course_create,
            course_commands::course_select,
            lesson_commands::lesson_discover,
            lesson_commands::lesson_read,
            lesson_commands::lesson_read_source,
            lesson_commands::lesson_list_drafts,
            lesson_commands::lesson_read_activity,
            lesson_commands::lesson_read_resource,
            lesson_commands::lesson_create_draft,
            lesson_commands::lesson_edit_draft,
            lesson_commands::lesson_preview_draft,
            lesson_commands::lesson_validate_draft,
            lesson_commands::lesson_apply_draft,
            lesson_commands::lesson_discard_draft,
            lesson_commands::lesson_get_receipt,
            check_draft,
            workspace_status,
            save_reading_preferences,
            operation_receipt,
            reopen_workspace,
            authoring::library,
            authoring::apply_content,
            authoring::save_document,
            authoring::read_document,
            authoring::save_annotation,
            authoring::annotations,
            authoring::save_reading_position,
            authoring::reading_position,
            attachments::pick_source,
            attachments::sources,
            attachments::save_source,
            attachments::source_preview,
            package_commands::save_original,
            package_commands::package_export,
            package_commands::package_inspect,
            package_commands::package_confirm,
            package_commands::package_discard,
            ingestion_commands::ingestion_create,
            ingestion_commands::ingestion_read,
            ingestion_commands::ingestion_list,
            ingestion_commands::ingestion_cancel,
            ingestion_commands::ingestion_apply,
            ingestion_commands::ingestion_review,
            ingestion_commands::ingestion_reimport_preview,
            ingestion_commands::ingestion_reimport_apply,
            ingestion_commands::ingestion_undo_preview,
            ingestion_commands::ingestion_undo,
            authoring::search_resources,
            authoring::rebuild_search_index
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
fn start_workspace(app: &tauri::App) -> Result<CourseManager, Box<dyn std::error::Error>> {
    let root = app.path().app_local_data_dir()?;
    CourseManager::open(root).map_err(Into::into)
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
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = app.state::<CourseManager>().shutdown().await;
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

    #[test]
    fn ingestion_main_ipc_runs_durable_workflow_and_denies_foreign_surfaces() {
        use serde_json::{Value, json};
        use workspace::*;
        let temp = tempfile::tempdir().unwrap();
        let manager = CourseManager::open(temp.path().to_owned()).unwrap();
        let course = manager.list().unwrap().selected_course_id;
        let service = manager.service(course).unwrap();
        let resource = crate::organization::ResourceId::generate();
        let (workspace, source) = tauri::async_runtime::block_on(async {
            let workspace = service
                .status(RequestContext::local_ui())
                .await
                .unwrap()
                .preferences
                .workspace_id;
            service
                .apply_content(
                    RequestContext::local_ui(),
                    ApplyContent {
                        contract_version: 1,
                        workspace_id: workspace.clone(),
                        operation_id: OperationId::generate(),
                        expected_revision: 0,
                        command: crate::organization::Command::CreateResource {
                            id: resource,
                            title: "Original".into(),
                        },
                    },
                )
                .await
                .unwrap();
            let asset = service
                .register_asset(
                    RequestContext::local_ui(),
                    RegisterAsset {
                        contract_version: 1,
                        workspace_id: workspace.clone(),
                        operation_id: OperationId::generate(),
                    },
                    Box::new(std::io::Cursor::new(b"one,body\n")),
                    1024,
                )
                .await
                .unwrap();
            let source = service
                .save_source(
                    RequestContext::local_ui(),
                    SaveSource {
                        contract_version: 1,
                        workspace_id: workspace.clone(),
                        operation_id: OperationId::generate(),
                        expected_revision: 0,
                        source: NewSource {
                            id: uuid::Uuid::new_v4(),
                            resource_id: resource,
                            digest: asset.digest,
                            filename: "rows.csv".into(),
                            media_type: "text/csv".into(),
                            attribution: None,
                            license: None,
                            origin_url: None,
                        },
                    },
                )
                .await
                .unwrap();
            (workspace, source)
        });
        let other_course = uuid::Uuid::new_v4();
        manager.create(other_course, "Other".into()).unwrap();
        let app = mock_builder()
            .manage(manager)
            .invoke_handler(tauri::generate_handler![
                ingestion_commands::ingestion_create,
                ingestion_commands::ingestion_read,
                ingestion_commands::ingestion_list,
                ingestion_commands::ingestion_cancel,
                ingestion_commands::ingestion_apply,
                ingestion_commands::ingestion_review,
                ingestion_commands::ingestion_reimport_preview,
                ingestion_commands::ingestion_reimport_apply,
                ingestion_commands::ingestion_undo_preview,
                ingestion_commands::ingestion_undo,
                authoring::library,
                authoring::rebuild_search_index,
            ])
            .build(tauri::generate_context!())
            .unwrap();
        let main = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        let other = tauri::WebviewWindowBuilder::new(&app, "other", Default::default())
            .build()
            .unwrap();
        for command in [
            "ingestion_create",
            "ingestion_read",
            "ingestion_list",
            "ingestion_cancel",
            "ingestion_apply",
            "ingestion_review",
            "ingestion_reimport_preview",
            "ingestion_reimport_apply",
            "ingestion_undo_preview",
            "ingestion_undo",
            "rebuild_search_index",
        ] {
            for (view, origin) in [
                (&other, "tauri://localhost"),
                (&main, "https://example.invalid"),
            ] {
                let error = raw_ipc(view, command, origin, json!({"courseId":course})).unwrap_err();
                assert!(
                    error.to_string().contains("not allowed"),
                    "{command}: {error}"
                );
            }
        }
        let call = |command: &str, mut body: Value| -> Value {
            body["courseId"] = json!(course);
            raw_ipc(&main, command, "tauri://localhost", body)
                .unwrap_or_else(|e| panic!("{command}: {e}"))
                .deserialize()
                .unwrap()
        };
        let create = || json!({"contractVersion":1,"workspaceId":workspace,"operationId":uuid::Uuid::new_v4(),"sourceId":source.id,"sourceRevision":source.revision});
        let create_request = create();
        let job = call("ingestion_create", json!({"request":create_request}));
        assert_eq!(
            call("ingestion_create", json!({"request":create_request})),
            job
        );
        assert_eq!(call("ingestion_read", json!({"jobId":job["id"]})), job);
        assert_eq!(
            call("ingestion_list", json!({"resourceId":resource,"offset":0}))
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert!(
            raw_ipc(
                &main,
                "ingestion_read",
                "tauri://localhost",
                json!({"courseId":other_course,"jobId":job["id"]})
            )
            .is_err()
        );
        let mapping = json!({"keyColumn":0,"titleColumn":0,"bodyColumn":1});
        let revision = || call("library", json!({}))["revision"].clone();
        let applied = call(
            "ingestion_apply",
            json!({"request":{"contractVersion":1,"workspaceId":workspace,"operationId":uuid::Uuid::new_v4(),"jobId":job["id"],"expectedRevision":0,"expectedLibraryRevision":revision(),"mapping":mapping,"selected":[0]}}),
        );
        let next = call("ingestion_create", json!({"request":create()}));
        let preview = call(
            "ingestion_reimport_preview",
            json!({"jobId":next["id"],"previousJobId":job["id"],"mapping":mapping}),
        );
        assert_eq!(preview["items"][0]["status"], "unchanged");
        assert_eq!(preview["items"][0]["prior"]["body"], "body");
        let next = call(
            "ingestion_reimport_apply",
            json!({"request":{"contractVersion":1,"workspaceId":workspace,"operationId":uuid::Uuid::new_v4(),"jobId":next["id"],"previousJobId":job["id"],"expectedRevision":0,"expectedPreviousRevision":applied["revision"],"expectedLibraryRevision":revision(),"mapping":mapping,"selected":[0],"resolutions":[]}}),
        );
        assert_eq!(
            call("ingestion_undo_preview", json!({"jobId":next["id"]}))["canUndo"],
            true
        );
        let undone = call(
            "ingestion_undo",
            json!({"request":{"contractVersion":1,"workspaceId":workspace,"operationId":uuid::Uuid::new_v4(),"jobId":next["id"],"expectedRevision":next["revision"],"expectedLibraryRevision":revision()}}),
        );
        assert_eq!(undone["status"], "undone");
        let review = call("ingestion_create", json!({"request":create()}));
        let reviewed = call(
            "ingestion_review",
            json!({"request":{"contractVersion":1,"workspaceId":workspace,"operationId":uuid::Uuid::new_v4(),"jobId":review["id"],"expectedRevision":0,"previousJobId":null,"expectedPreviousRevision":null,"mapping":mapping,"decisions":[{"index":0,"key":null,"disposition":"omitted"}]}}),
        );
        let cancelled = call(
            "ingestion_cancel",
            json!({"request":{"contractVersion":1,"workspaceId":workspace,"operationId":uuid::Uuid::new_v4(),"jobId":review["id"],"expectedRevision":reviewed["revision"]}}),
        );
        assert_eq!(cancelled["status"], "cancelled");
        call("rebuild_search_index", json!({}));
        tauri::async_runtime::block_on(app.state::<CourseManager>().shutdown()).unwrap();
    }

    #[test]
    fn course_routes_reject_missing_and_unknown_course_ids() {
        let temp = tempfile::tempdir().unwrap();
        let manager = CourseManager::open(temp.path().to_owned()).unwrap();
        let service = manager
            .service(manager.list().unwrap().selected_course_id)
            .unwrap();
        let app = mock_builder()
            .manage(manager)
            .invoke_handler(tauri::generate_handler![authoring::library])
            .build(tauri::generate_context!())
            .unwrap();
        let main = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        assert!(raw_ipc(&main, "library", "tauri://localhost", serde_json::json!({})).is_err());
        assert!(
            raw_ipc(
                &main,
                "library",
                "tauri://localhost",
                serde_json::json!({"courseId":uuid::Uuid::new_v4()})
            )
            .is_err()
        );
        tauri::async_runtime::block_on(service.shutdown()).unwrap();
    }

    #[test]
    fn real_two_course_ipc_isolates_content_search_preferences_drafts_and_receipts() {
        use serde_json::{Value, json};
        let temp = tempfile::tempdir().unwrap();
        let manager = CourseManager::open(temp.path().to_owned()).unwrap();
        let a = manager.list().unwrap().selected_course_id;
        let b = uuid::Uuid::new_v4();
        manager.create(b, "Second".into()).unwrap();
        let app = mock_builder()
            .manage(manager)
            .invoke_handler(tauri::generate_handler![
                course_commands::courses_list,
                course_commands::course_create,
                course_commands::course_select,
                workspace_status,
                save_reading_preferences,
                operation_receipt,
                authoring::library,
                authoring::apply_content,
                authoring::read_document,
                authoring::search_resources,
                lesson_commands::lesson_create_draft,
                lesson_commands::lesson_list_drafts,
                lesson_commands::lesson_preview_draft,
                lesson_commands::lesson_get_receipt
            ])
            .build(tauri::generate_context!())
            .unwrap();
        let main = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        let other = tauri::WebviewWindowBuilder::new(&app, "other", Default::default())
            .build()
            .unwrap();
        let call = |course: uuid::Uuid, command: &str, mut body: Value| {
            body["courseId"] = json!(course);
            raw_ipc(&main, command, "tauri://localhost", body)
                .map(|v| v.deserialize::<Value>().unwrap())
        };
        let one = call(a, "library", json!({})).unwrap();
        let two = call(b, "library", json!({})).unwrap();
        assert_ne!(one["workspaceId"], two["workspaceId"]);
        let resource = uuid::Uuid::new_v4();
        let unit = uuid::Uuid::new_v4();
        let operation = uuid::Uuid::new_v4();
        let request = json!({"contractVersion":1,"workspaceId":one["workspaceId"],"operationId":operation,"expectedRevision":0,"command":{"type":"createResource","id":resource,"title":"A secret needle"}});
        call(a, "apply_content", json!({"request":request})).unwrap();
        assert_eq!(
            call(b, "apply_content", json!({"request":request})).unwrap_err()["code"],
            "permission_denied"
        );
        assert_eq!(
            call(a, "search_resources", json!({"query":"needle","offset":0})).unwrap()["items"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert!(
            call(b, "search_resources", json!({"query":"needle","offset":0})).unwrap()["items"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert!(
            call(
                b,
                "read_document",
                json!({"resourceId":resource,"revisionId":null})
            )
            .is_err()
        );
        assert!(
            !call(a, "operation_receipt", json!({"operationId":operation}))
                .unwrap()
                .is_null()
        );
        assert!(
            call(b, "operation_receipt", json!({"operationId":operation}))
                .unwrap()
                .is_null()
        );
        let preference_operation = uuid::Uuid::new_v4();
        call(a, "save_reading_preferences", json!({"request":{"contractVersion":1,"workspaceId":one["workspaceId"],"operationId":preference_operation,"expectedRevision":0,"preferences":{"theme":"dark","textScale":150}}})).unwrap();
        assert_eq!(
            call(a, "workspace_status", json!({})).unwrap()["preferences"]["preferences"]["theme"],
            "dark"
        );
        assert_eq!(
            call(b, "workspace_status", json!({})).unwrap()["preferences"]["preferences"]["theme"],
            "system"
        );
        // The same Unit ID exists in both databases: a foreign draft still cannot resolve.
        for (course, workspace, revision) in
            [(a, &one["workspaceId"], 1), (b, &two["workspaceId"], 0)]
        {
            call(course, "apply_content", json!({"request":{"contractVersion":1,"workspaceId":workspace,"operationId":uuid::Uuid::new_v4(),"expectedRevision":revision,"command":{"type":"createUnit","id":unit,"title":"Lesson","objective":"Explain","goalIds":[],"resourceIds":[]}}})).unwrap();
        }
        let draft_operation = uuid::Uuid::new_v4();
        let draft = call(a, "lesson_create_draft", json!({"request":{"contractVersion":1,"workspaceId":one["workspaceId"],"unitId":unit,"operationId":draft_operation,"expectedHeadRevision":null}})).unwrap();
        assert_eq!(
            call(a, "lesson_list_drafts", json!({"unitId":unit}))
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert!(
            call(b, "lesson_list_drafts", json!({"unitId":unit}))
                .unwrap()
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert!(call(b, "lesson_preview_draft", json!({"request":{"contractVersion":1,"workspaceId":two["workspaceId"],"unitId":unit,"draftId":draft["id"],"expectedDraftRevision":draft["revision"]}})).is_err());
        assert!(
            !call(
                a,
                "operation_receipt",
                json!({"operationId":draft_operation})
            )
            .unwrap()
            .is_null()
        );
        assert!(
            call(
                b,
                "lesson_get_receipt",
                json!({"operationId":draft_operation})
            )
            .unwrap()
            .is_null()
        );
        call(a, "course_select", json!({})).unwrap();
        // A changed global selection cannot redirect an explicit B request.
        assert!(
            call(b, "library", json!({})).unwrap()["resources"]
                .as_object()
                .unwrap()
                .is_empty()
        );
        for command in [
            "courses_list",
            "course_create",
            "course_select",
            "library",
            "lesson_list_drafts",
        ] {
            assert!(raw_ipc(&other, command, "tauri://localhost", json!({"courseId":a})).is_err());
            assert!(
                raw_ipc(
                    &main,
                    command,
                    "https://untrusted.example",
                    json!({"courseId":a})
                )
                .is_err()
            );
        }
        assert!(raw_ipc(&main, "course_create", "tauri://localhost", json!({"request":{"operationId":uuid::Uuid::new_v4(),"title":"Forged","root":"/tmp/foreign"}})).is_err());
        assert!(
            call(
                a,
                "lesson_create_draft",
                json!({"trustedUi":true,"request":{}})
            )
            .is_err()
        );
        tauri::async_runtime::block_on(app.state::<CourseManager>().shutdown()).unwrap();
        drop(other);
        drop(main);
        drop(app);
        let reopened = CourseManager::open(temp.path().to_owned()).unwrap();
        assert_eq!(reopened.list().unwrap().selected_course_id, a);
        let restored = tauri::async_runtime::block_on(
            reopened
                .service(a)
                .unwrap()
                .lesson_list_drafts(RequestContext::local_ui(), unit.to_string()),
        )
        .unwrap();
        assert_eq!(restored.len(), 1);
        assert_eq!(serde_json::to_value(&restored[0]).unwrap(), draft);
        tauri::async_runtime::block_on(reopened.shutdown()).unwrap();
    }

    #[test]
    fn lesson_raw_decoder_rejects_budgets_and_returns_structured_errors_through_ipc() {
        let temp = tempfile::tempdir().unwrap();
        let manager = CourseManager::open(temp.path().to_owned()).unwrap();
        let service = manager
            .service(manager.list().unwrap().selected_course_id)
            .unwrap();
        let app = mock_builder()
            .manage(manager)
            .invoke_handler(tauri::generate_handler![
                lesson_commands::lesson_edit_draft,
                lesson_commands::lesson_discover
            ])
            .build(tauri::generate_context!())
            .unwrap();
        let main = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        let too_large = InvokeBody::Raw(
            format!("{{\"request\":\"{}\"}}", "x".repeat(2 * 1024 * 1024)).into_bytes(),
        );
        let error = ipc(&main, "lesson_edit_draft", "tauri://localhost", too_large).unwrap_err();
        assert_eq!(error["code"], "budget_exceeded");
        assert_eq!(error["diagnostics"][0]["field"], "request");
        assert_eq!(error["diagnostics"][0]["repair"], "split_request");
        let registry = ipc(
            &main,
            "lesson_discover",
            "tauri://localhost",
            InvokeBody::Raw(b"{}".to_vec()),
        )
        .unwrap()
        .deserialize::<Vec<workspace::RegistryDescriptor>>()
        .unwrap();
        assert_eq!(registry.len(), 8);
        tauri::async_runtime::block_on(service.shutdown()).unwrap();
    }
    #[test]
    fn lesson_registry_and_mutations_respect_native_capabilities_and_closed_context() {
        let temp = tempfile::tempdir().unwrap();
        let manager = CourseManager::open(temp.path().to_owned()).unwrap();
        let service = manager
            .service(manager.list().unwrap().selected_course_id)
            .unwrap();
        let app = mock_builder()
            .manage(manager)
            .invoke_handler(tauri::generate_handler![
                lesson_commands::lesson_discover,
                lesson_commands::lesson_create_draft
            ])
            .build(tauri::generate_context!())
            .unwrap();
        let main = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        let other = tauri::WebviewWindowBuilder::new(&app, "secondary", Default::default())
            .build()
            .unwrap();
        let registry = ipc(
            &main,
            "lesson_discover",
            "tauri://localhost",
            serde_json::json!({}),
        )
        .unwrap()
        .deserialize::<Vec<workspace::RegistryDescriptor>>()
        .unwrap();
        assert_eq!(registry.len(), 8);
        assert!(
            ipc(
                &other,
                "lesson_discover",
                "tauri://localhost",
                serde_json::json!({})
            )
            .is_err()
        );
        assert!(
            ipc(
                &main,
                "lesson_discover",
                "https://untrusted.example",
                serde_json::json!({})
            )
            .is_err()
        );
        let status =
            tauri::async_runtime::block_on(service.status(RequestContext::local_ui())).unwrap();
        let forged = serde_json::json!({"request":{"contractVersion":1,"workspaceId":status.preferences.workspace_id,"unitId":uuid::Uuid::new_v4(),"operationId":OperationId::generate(),"expectedHeadRevision":null,"trustedUi":true}});
        assert!(ipc(&main, "lesson_create_draft", "tauri://localhost", forged).is_err());
        tauri::async_runtime::block_on(service.shutdown()).unwrap();
    }
    #[test]
    fn authoring_read_is_scoped_to_the_trusted_main_webview() {
        let temp = tempfile::tempdir().unwrap();
        let manager = CourseManager::open(temp.path().to_owned()).unwrap();
        let service = manager
            .service(manager.list().unwrap().selected_course_id)
            .unwrap();
        let app = mock_builder()
            .manage(manager)
            .invoke_handler(tauri::generate_handler![
                authoring::library,
                authoring::search_resources
            ])
            .build(tauri::generate_context!())
            .unwrap();
        let main = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        let other = tauri::WebviewWindowBuilder::new(&app, "secondary", Default::default())
            .build()
            .unwrap();
        let snapshot = ipc(&main, "library", "tauri://localhost", serde_json::json!({}))
            .unwrap()
            .deserialize::<crate::organization::Snapshot>()
            .unwrap();
        assert!(snapshot.subjects.is_empty());
        let search = serde_json::json!({"query":"", "offset":0});
        let page = ipc(
            &main,
            "search_resources",
            "tauri://localhost",
            search.clone(),
        )
        .unwrap()
        .deserialize::<workspace::SearchPage>()
        .unwrap();
        assert!(page.items.is_empty());
        assert!(
            ipc(
                &other,
                "search_resources",
                "tauri://localhost",
                search.clone()
            )
            .is_err()
        );
        assert!(
            ipc(
                &main,
                "search_resources",
                "https://untrusted.example",
                search
            )
            .is_err()
        );
        assert!(
            ipc(
                &main,
                "search_resources",
                "tauri://localhost",
                serde_json::json!({"query":"", "offset":-1})
            )
            .is_err()
        );
        assert_eq!(
            ipc(
                &main,
                "search_resources",
                "tauri://localhost",
                serde_json::json!({"query":"\u{0}", "offset":0})
            )
            .unwrap_err(),
            serde_json::json!({"code":"invalid_request"})
        );
        assert!(
            ipc(
                &other,
                "library",
                "tauri://localhost",
                serde_json::json!({})
            )
            .is_err()
        );
        assert!(
            ipc(
                &main,
                "library",
                "https://untrusted.example",
                serde_json::json!({})
            )
            .is_err()
        );
        tauri::async_runtime::block_on(service.shutdown()).unwrap();
    }

    #[test]
    fn native_selection_rejects_other_surfaces_and_caller_path_authority() {
        let temp = tempfile::tempdir().unwrap();
        let manager = CourseManager::open(temp.path().to_owned()).unwrap();
        let service = manager
            .service(manager.list().unwrap().selected_course_id)
            .unwrap();
        let status =
            tauri::async_runtime::block_on(service.status(RequestContext::local_ui())).unwrap();
        let app = mock_builder()
            .manage(manager)
            .invoke_handler(tauri::generate_handler![attachments::pick_source])
            .build(tauri::generate_context!())
            .unwrap();
        let main = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        let other = tauri::WebviewWindowBuilder::new(&app, "other", Default::default())
            .build()
            .unwrap();
        let body = serde_json::json!({"request": {"contractVersion":1,
            "workspaceId":status.preferences.workspace_id, "operationId":uuid::Uuid::new_v4()}});
        assert!(ipc(&other, "pick_source", "tauri://localhost", body.clone()).is_err());
        assert!(
            ipc(
                &main,
                "pick_source",
                "https://untrusted.example",
                body.clone()
            )
            .is_err()
        );
        let mut forged = body.clone();
        forged["request"]["path"] = serde_json::json!("/etc/passwd");
        assert!(ipc(&main, "pick_source", "tauri://localhost", forged).is_err());
        let mut foreign = body;
        foreign["request"]["workspaceId"] = serde_json::json!(uuid::Uuid::new_v4());
        let error = ipc(&main, "pick_source", "tauri://localhost", foreign).unwrap_err();
        assert_eq!(
            error,
            serde_json::to_value(AppError::new(workspace::ErrorCode::PermissionDenied)).unwrap()
        );
        tauri::async_runtime::block_on(service.shutdown()).unwrap();
    }

    #[test]
    fn authoring_mutation_rejects_foreign_surfaces_and_forged_principals() {
        let temp = tempfile::tempdir().unwrap();
        let manager = CourseManager::open(temp.path().to_owned()).unwrap();
        let service = manager
            .service(manager.list().unwrap().selected_course_id)
            .unwrap();
        let initial =
            tauri::async_runtime::block_on(service.library(RequestContext::local_ui())).unwrap();
        let app = mock_builder()
            .manage(manager)
            .invoke_handler(tauri::generate_handler![authoring::apply_content])
            .build(tauri::generate_context!())
            .unwrap();
        let main = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        let other = tauri::WebviewWindowBuilder::new(&app, "other", Default::default())
            .build()
            .unwrap();
        let request = serde_json::json!({"contractVersion":1,"workspaceId":initial.workspace_id,
            "operationId":uuid::Uuid::new_v4(),"expectedRevision":0,
            "command":{"type":"createSubject","id":uuid::Uuid::new_v4(),"title":"Quantum physics","priorKnowledge":null}});
        let body = serde_json::json!({"request":request});
        assert!(ipc(&other, "apply_content", "tauri://localhost", body.clone()).is_err());
        assert!(
            ipc(
                &main,
                "apply_content",
                "https://untrusted.example",
                body.clone()
            )
            .is_err()
        );
        let mut forged = body.clone();
        forged["request"]["trustedUi"] = serde_json::json!(true);
        assert!(ipc(&main, "apply_content", "tauri://localhost", forged).is_err());
        let result = ipc(&main, "apply_content", "tauri://localhost", body)
            .unwrap()
            .deserialize::<crate::organization::Snapshot>()
            .unwrap();
        assert_eq!(result.revision, 1);
        assert_eq!(result.subjects.len(), 1);
        tauri::async_runtime::block_on(service.shutdown()).unwrap();
    }

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
        let manager = CourseManager::open(temp.path().to_owned()).unwrap();
        let service = manager
            .service(manager.list().unwrap().selected_course_id)
            .unwrap();
        let app = mock_builder()
            .manage(manager)
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
        body: impl Into<InvokeBody>,
    ) -> Result<tauri::ipc::InvokeResponseBody, serde_json::Value> {
        let course_id = webview
            .state::<CourseManager>()
            .list()
            .unwrap()
            .selected_course_id;
        let body = match body.into() {
            InvokeBody::Json(mut value) => {
                value["courseId"] = serde_json::json!(course_id);
                InvokeBody::Json(value)
            }
            InvokeBody::Raw(bytes) => {
                // Oversized preflight fixtures stay byte-identical.
                if bytes.len() > 1024 * 1024 {
                    InvokeBody::Raw(bytes)
                } else {
                    let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                    value["courseId"] = serde_json::json!(course_id);
                    InvokeBody::Raw(serde_json::to_vec(&value).unwrap())
                }
            }
        };
        raw_ipc(webview, command, origin, body)
    }

    fn raw_ipc(
        webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
        command: &str,
        origin: &str,
        body: impl Into<InvokeBody>,
    ) -> Result<tauri::ipc::InvokeResponseBody, serde_json::Value> {
        get_ipc_response(
            webview,
            InvokeRequest {
                cmd: command.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: origin.parse().unwrap(),
                body: body.into(),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.into(),
            },
        )
    }
    #[test]
    fn native_preference_save_and_receipt_round_trip_preserves_authority() {
        let temp = tempfile::tempdir().unwrap();
        let manager = CourseManager::open(temp.path().to_owned()).unwrap();
        let service = manager
            .service(manager.list().unwrap().selected_course_id)
            .unwrap();
        let initial =
            tauri::async_runtime::block_on(service.status(RequestContext::local_ui())).unwrap();
        let app = mock_builder()
            .manage(manager)
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
