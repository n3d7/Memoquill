use super::*;
use crate::workspace::{ErrorCode, RequestContext};

#[test]
fn catalogue_adopts_legacy_bytes_without_opening_or_rewriting_them() {
    let temp = tempfile::tempdir().unwrap();
    let old = temp.path().join("workspace");
    std::fs::create_dir(&old).unwrap();
    std::fs::write(
        old.join("memoquill.sqlite3"),
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
        std::fs::read(old.join("memoquill.sqlite3")).unwrap(),
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
        std::fs::read(old.join("memoquill.sqlite3")).unwrap(),
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

#[test]
fn previously_opened_course_never_recreates_a_missing_database() {
    tauri::async_runtime::block_on(async {
        let temp = tempfile::tempdir().unwrap();
        let manager = CourseManager::open(temp.path().to_owned()).unwrap();
        let id = Uuid::new_v4();
        manager.create(id, "Keep identity".into()).unwrap();
        manager
            .service(id)
            .unwrap()
            .status(RequestContext::local_ui())
            .await
            .unwrap();
        manager.shutdown().await.unwrap();
        drop(manager);
        let root = temp.path().join("courses/data").join(id.to_string());
        let database = root.join("memoquill.sqlite3");
        std::fs::rename(&database, root.join("preserved.sqlite3")).unwrap();
        let manager = CourseManager::open(temp.path().to_owned()).unwrap();
        assert_eq!(
            manager.service(id).err().unwrap().code,
            ErrorCode::RecoveryRequired
        );
        assert!(!database.exists());
    });
}

#[test]
fn real_legacy_workspace_links_preferences_and_draft_survive_catalogue_adoption() {
    tauri::async_runtime::block_on(async {
        use serde_json::json;
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        let service = ApplicationService::start(root.clone()).unwrap();
        let before = service.library(RequestContext::local_ui()).await.unwrap();
        let resource = Uuid::new_v4();
        let unit = Uuid::new_v4();
        for (revision, command) in [
            (
                0,
                json!({"type":"createResource","id":resource,"title":"Legacy resource"}),
            ),
            (
                1,
                json!({"type":"createUnit","id":unit,"title":"Legacy lesson","objective":"Preserve links","goalIds":[],"resourceIds":[resource]}),
            ),
        ] {
            service.apply_content(RequestContext::local_ui(), serde_json::from_value(json!({"contractVersion":1,"workspaceId":before.workspace_id,"operationId":Uuid::new_v4(),"expectedRevision":revision,"command":command})).unwrap()).await.unwrap();
        }
        let preferences = service.save_preferences(RequestContext::local_ui(), serde_json::from_value(json!({"contractVersion":1,"workspaceId":before.workspace_id,"operationId":Uuid::new_v4(),"expectedRevision":0,"preferences":{"theme":"dark","textScale":150}})).unwrap()).await.unwrap();
        let snapshot = service.library(RequestContext::local_ui()).await.unwrap();
        let draft = service.lesson_create(RequestContext::local_ui(), serde_json::from_value(json!({"contractVersion":1,"workspaceId":before.workspace_id,"unitId":unit,"operationId":Uuid::new_v4(),"expectedHeadRevision":null})).unwrap()).await.unwrap();
        service.shutdown().await.unwrap();
        drop(service);
        let database = root.join("memoquill.sqlite3");
        let bytes = std::fs::read(&database).unwrap();
        let manager = CourseManager::open(temp.path().to_owned()).unwrap();
        let catalogue = manager.list().unwrap();
        assert_eq!(catalogue.courses[0].title, "Imported workspace");
        assert_eq!(std::fs::read(&database).unwrap(), bytes);
        drop(manager);
        let manager = CourseManager::open(temp.path().to_owned()).unwrap();
        assert_eq!(std::fs::read(&database).unwrap(), bytes);
        let adopted = manager.service(catalogue.selected_course_id).unwrap();
        assert_eq!(
            serde_json::to_value(adopted.library(RequestContext::local_ui()).await.unwrap())
                .unwrap(),
            serde_json::to_value(snapshot).unwrap()
        );
        assert_eq!(
            serde_json::to_value(
                adopted
                    .status(RequestContext::local_ui())
                    .await
                    .unwrap()
                    .preferences
            )
            .unwrap(),
            serde_json::to_value(preferences).unwrap()
        );
        let drafts = adopted
            .lesson_list_drafts(RequestContext::local_ui(), unit.to_string())
            .await
            .unwrap();
        assert_eq!(drafts.len(), 1);
        assert_eq!(
            serde_json::to_value(&drafts[0]).unwrap(),
            serde_json::to_value(draft).unwrap()
        );
        manager.shutdown().await.unwrap();
        drop(manager);
        let before_failed_bootstrap = std::fs::read(&database).unwrap();
        std::fs::write(
            temp.path().join("courses/catalogue.sqlite3"),
            b"corrupt catalogue",
        )
        .unwrap();
        assert!(CourseManager::open(temp.path().to_owned()).is_err());
        assert_eq!(std::fs::read(&database).unwrap(), before_failed_bootstrap);
    });
}

#[test]
fn drained_manager_releases_ownership_while_stopped_state_remains_managed() {
    tauri::async_runtime::block_on(async {
        let temp = tempfile::tempdir().unwrap();
        let manager = CourseManager::open(temp.path().to_owned()).unwrap();
        let id = manager.list().unwrap().selected_course_id;
        let status = manager
            .service(id)
            .unwrap()
            .status(RequestContext::local_ui())
            .await
            .unwrap();
        manager.shutdown().await.unwrap();
        let reopened = CourseManager::open(temp.path().to_owned()).unwrap();
        assert_eq!(
            reopened
                .service(id)
                .unwrap()
                .status(RequestContext::local_ui())
                .await
                .unwrap()
                .preferences
                .workspace_id,
            status.preferences.workspace_id
        );
        assert_eq!(
            manager.list().unwrap_err().code,
            ErrorCode::OwnerUnavailable
        );
        assert_eq!(
            manager
                .create(Uuid::new_v4(), "Closed".into())
                .unwrap_err()
                .code,
            ErrorCode::OwnerUnavailable
        );
        assert_eq!(
            manager.select(id).unwrap_err().code,
            ErrorCode::OwnerUnavailable
        );
        assert_eq!(
            manager.service(id).err().unwrap().code,
            ErrorCode::OwnerUnavailable
        );
        reopened.shutdown().await.unwrap();
    });
}

#[test]
fn failed_drain_keeps_exclusive_catalogue_ownership() {
    tauri::async_runtime::block_on(async {
        let temp = tempfile::tempdir().unwrap();
        let manager = CourseManager::open(temp.path().to_owned()).unwrap();
        let id = manager.list().unwrap().selected_course_id;
        manager.service(id).unwrap().shutdown().await.unwrap();
        assert_eq!(
            manager.shutdown().await.unwrap_err().code,
            ErrorCode::OwnerUnavailable
        );
        assert_eq!(
            CourseManager::open(temp.path().to_owned())
                .err()
                .unwrap()
                .code,
            ErrorCode::AlreadyOpen
        );
    });
}

#[test]
fn uninitialized_course_does_not_adopt_an_abandoned_managed_root() {
    let temp = tempfile::tempdir().unwrap();
    let manager = CourseManager::open(temp.path().to_owned()).unwrap();
    let id = Uuid::new_v4();
    manager.create(id, "New course".into()).unwrap();
    let root = temp.path().join("courses/data").join(id.to_string());
    std::fs::create_dir(&root).unwrap();
    let path = root.join("preserved.txt");
    std::fs::write(&path, b"abandoned data").unwrap();
    assert_eq!(
        manager.service(id).err().unwrap().code,
        ErrorCode::RecoveryRequired
    );
    assert_eq!(std::fs::read(&path).unwrap(), b"abandoned data");
    assert!(!root.join("memoquill.sqlite3").exists());
}

fn file_snapshot(root: &std::path::Path) -> std::collections::BTreeMap<PathBuf, Vec<u8>> {
    let mut files = std::collections::BTreeMap::new();
    for entry in std::fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files.extend(file_snapshot(&path));
        } else {
            files.insert(path.clone(), std::fs::read(path).unwrap());
        }
    }
    files
}

fn lost_catalogue_preserves_used_courses(empty_file: bool) {
    tauri::async_runtime::block_on(async {
        let temp = tempfile::tempdir().unwrap();
        let manager = CourseManager::open(temp.path().to_owned()).unwrap();
        let first = manager.list().unwrap().selected_course_id;
        let second = Uuid::new_v4();
        manager.create(second, "Second real course".into()).unwrap();
        for id in [first, second] {
            manager
                .service(id)
                .unwrap()
                .status(RequestContext::local_ui())
                .await
                .unwrap();
        }
        manager.shutdown().await.unwrap();
        drop(manager);
        let catalogue = temp.path().join("courses/catalogue.sqlite3");
        if empty_file {
            std::fs::write(&catalogue, []).unwrap();
        } else {
            std::fs::remove_file(&catalogue).unwrap();
        }
        let before = file_snapshot(temp.path());
        assert_eq!(
            CourseManager::open(temp.path().to_owned())
                .err()
                .unwrap()
                .code,
            ErrorCode::RecoveryRequired
        );
        assert_eq!(file_snapshot(temp.path()), before);
        assert_eq!(catalogue.exists(), empty_file);
    });
}

#[test]
fn missing_catalogue_with_real_managed_courses_requires_recovery_without_creating_files() {
    lost_catalogue_preserves_used_courses(false);
}

#[test]
fn empty_catalogue_with_real_managed_courses_requires_recovery_without_rewriting_files() {
    lost_catalogue_preserves_used_courses(true);
}

#[test]
fn valid_catalogue_preserves_unrelated_abandoned_roots_and_remains_readable() {
    let temp = tempfile::tempdir().unwrap();
    let manager = CourseManager::open(temp.path().to_owned()).unwrap();
    let expected = manager.list().unwrap();
    drop(manager);
    let orphan = temp
        .path()
        .join("courses/data")
        .join(Uuid::new_v4().to_string());
    std::fs::create_dir(&orphan).unwrap();
    std::fs::write(
        orphan.join("memoquill.sqlite3"),
        b"unrelated abandoned bytes",
    )
    .unwrap();
    let before = file_snapshot(temp.path());
    let reopened = CourseManager::open(temp.path().to_owned()).unwrap();
    assert_eq!(reopened.list().unwrap(), expected);
    assert_eq!(file_snapshot(temp.path()), before);
}
