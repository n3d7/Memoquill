use super::*;
use crate::workspace::{ErrorCode, RequestContext};

#[test]
fn catalogue_adopts_legacy_bytes_without_opening_or_rewriting_them() {
    let temp = tempfile::tempdir().unwrap();
    let old = temp.path().join("workspace");
    std::fs::create_dir(&old).unwrap();
    std::fs::write(
        old.join("workspace.sqlite3"),
        b"unread unknown legacy fixture",
    )
    .unwrap();
    std::fs::write(old.join("original.bin"), b"original bytes").unwrap();
    let manager = CourseManager::open(temp.path().to_owned()).unwrap();
    let catalogue = manager.list().unwrap();
    assert_eq!(catalogue.courses.len(), 1);
    assert_eq!(catalogue.courses[0].title, "Imported workspace");
    let id = catalogue.selected_course_id;
    assert_eq!(
        std::fs::read(old.join("workspace.sqlite3")).unwrap(),
        b"unread unknown legacy fixture"
    );
    assert_eq!(
        std::fs::read(old.join("original.bin")).unwrap(),
        b"original bytes"
    );
    drop(manager);
    let reopened = CourseManager::open(temp.path().to_owned()).unwrap();
    assert_eq!(reopened.list().unwrap().selected_course_id, id);
    assert_eq!(
        std::fs::read(old.join("workspace.sqlite3")).unwrap(),
        b"unread unknown legacy fixture"
    );
}

#[test]
fn exact_creation_retry_selection_and_catalogue_versions_survive_restart() {
    let temp = tempfile::tempdir().unwrap();
    let manager = CourseManager::open(temp.path().to_owned()).unwrap();
    let id = uuid::Uuid::new_v4();
    let created = manager.create(id, "Physics".into()).unwrap();
    assert_eq!(created.courses.len(), 2);
    assert_eq!(manager.create(id, "Physics".into()).unwrap(), created);
    assert_eq!(
        manager.create(id, "Different".into()).unwrap_err().code,
        ErrorCode::IdempotencyConflict
    );
    assert_eq!(
        manager.select(uuid::Uuid::new_v4()).unwrap_err().code,
        ErrorCode::InvalidReference
    );
    assert_eq!(
        manager
            .create(uuid::Uuid::new_v4(), "  ".into())
            .unwrap_err()
            .code,
        ErrorCode::InvalidRequest
    );
    manager.select(id).unwrap();
    drop(manager);
    let manager = CourseManager::open(temp.path().to_owned()).unwrap();
    assert_eq!(manager.list().unwrap().selected_course_id, id);
    drop(manager);
    let database = temp.path().join("courses/catalogue.sqlite3");
    let connection = rusqlite::Connection::open(&database).unwrap();
    connection.pragma_update(None, "user_version", 99).unwrap();
    drop(connection);
    let before = std::fs::read(&database).unwrap();
    assert_eq!(
        CourseManager::open(temp.path().to_owned())
            .err()
            .unwrap()
            .code,
        ErrorCode::UnsupportedSchema
    );
    assert_eq!(std::fs::read(database).unwrap(), before);
}

#[test]
fn unknown_catalogue_data_is_preserved_and_never_replaced_by_an_empty_course() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("courses");
    std::fs::create_dir(&root).unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
    let database = root.join("catalogue.sqlite3");
    std::fs::write(&database, b"not a sqlite database").unwrap();
    std::fs::set_permissions(&database, std::fs::Permissions::from_mode(0o600)).unwrap();
    assert!(CourseManager::open(temp.path().to_owned()).is_err());
    assert_eq!(std::fs::read(database).unwrap(), b"not a sqlite database");
    assert!(!temp.path().join("workspace").exists());
}

#[test]
fn catalogue_with_lost_legacy_mapping_requires_recovery_instead_of_empty_replacement() {
    let temp = tempfile::tempdir().unwrap();
    drop(CourseManager::open(temp.path().to_owned()).unwrap());
    let path = temp.path().join("courses/catalogue.sqlite3");
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute("UPDATE courses SET legacy=0", [])
        .unwrap();
    drop(connection);
    let before = std::fs::read(&path).unwrap();
    assert_eq!(
        CourseManager::open(temp.path().to_owned())
            .err()
            .unwrap()
            .code,
        ErrorCode::RecoveryRequired
    );
    assert_eq!(std::fs::read(path).unwrap(), before);
}

#[test]
fn bounded_catalogue_rejects_more_courses_without_losing_existing_entries() {
    let temp = tempfile::tempdir().unwrap();
    let manager = CourseManager::open(temp.path().to_owned()).unwrap();
    for number in 1..32 {
        manager
            .create(uuid::Uuid::new_v4(), format!("Course {number}"))
            .unwrap();
    }
    let before = manager.list().unwrap();
    assert_eq!(
        manager
            .create(uuid::Uuid::new_v4(), "Excess".into())
            .unwrap_err()
            .code,
        ErrorCode::BudgetExceeded
    );
    assert_eq!(manager.list().unwrap(), before);
    assert_eq!(
        CourseManager::open(temp.path().to_owned())
            .err()
            .unwrap()
            .code,
        ErrorCode::AlreadyOpen
    );
}

#[test]
fn two_course_databases_and_receipts_remain_separate_after_drain_and_restart() {
    tauri::async_runtime::block_on(async {
        let temp = tempfile::tempdir().unwrap();
        let manager = CourseManager::open(temp.path().to_owned()).unwrap();
        let first = manager.list().unwrap().selected_course_id;
        let second = uuid::Uuid::new_v4();
        manager.create(second, "Second".into()).unwrap();
        let a = manager.service(first).unwrap();
        let b = manager.service(second).unwrap();
        let one = a.library(RequestContext::local_ui()).await.unwrap();
        let two = b.library(RequestContext::local_ui()).await.unwrap();
        assert_ne!(one.workspace_id, two.workspace_id);
        let resource = uuid::Uuid::new_v4().to_string();
        let operation = uuid::Uuid::new_v4().to_string();
        let mutation = serde_json::json!({"contractVersion":1,"workspaceId":one.workspace_id,"operationId":operation,"expectedRevision":one.revision,"command":{"type":"createResource","id":resource,"title":"Only first course"}});
        let request = serde_json::from_value(mutation.clone()).unwrap();
        a.apply_content(RequestContext::local_ui(), request)
            .await
            .unwrap();
        assert_eq!(
            b.apply_content(
                RequestContext::local_ui(),
                serde_json::from_value(mutation).unwrap()
            )
            .await
            .unwrap_err()
            .code,
            ErrorCode::PermissionDenied
        );
        assert_eq!(
            a.search_resources(RequestContext::local_ui(), "Only first".into(), 0)
                .await
                .unwrap()
                .items
                .len(),
            1
        );
        assert!(
            b.search_resources(RequestContext::local_ui(), "Only first".into(), 0)
                .await
                .unwrap()
                .items
                .is_empty()
        );
        assert!(
            b.read_document(
                RequestContext::local_ui(),
                serde_json::from_value(serde_json::json!(resource)).unwrap(),
                None
            )
            .await
            .is_err()
        );
        assert!(
            a.operation_receipt(
                RequestContext::local_ui(),
                operation.clone().try_into().unwrap()
            )
            .await
            .unwrap()
            .is_some()
        );
        assert!(
            b.operation_receipt(RequestContext::local_ui(), operation.try_into().unwrap())
                .await
                .unwrap()
                .is_none()
        );
        assert!(manager.service(uuid::Uuid::new_v4()).is_err());
        manager.shutdown().await.unwrap();
        assert_eq!(
            manager.service(first).err().unwrap().code,
            ErrorCode::OwnerUnavailable
        );
        drop(manager);
        let manager = CourseManager::open(temp.path().to_owned()).unwrap();
        assert_eq!(
            manager
                .service(first)
                .unwrap()
                .library(RequestContext::local_ui())
                .await
                .unwrap()
                .workspace_id,
            one.workspace_id
        );
        assert_eq!(
            manager
                .service(second)
                .unwrap()
                .library(RequestContext::local_ui())
                .await
                .unwrap()
                .workspace_id,
            two.workspace_id
        );
        assert_eq!(
            manager
                .service(first)
                .unwrap()
                .search_resources(RequestContext::local_ui(), "Only first".into(), 0)
                .await
                .unwrap()
                .items
                .len(),
            1
        );
        assert!(
            manager
                .service(second)
                .unwrap()
                .search_resources(RequestContext::local_ui(), "Only first".into(), 0)
                .await
                .unwrap()
                .items
                .is_empty()
        );
        manager.shutdown().await.unwrap();
    });
}
