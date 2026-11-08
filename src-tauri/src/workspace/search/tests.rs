use super::*;
use crate::document::Document;
use uuid::Uuid;

fn resource(db: &Database, title: &str) -> ResourceId {
    let id = ResourceId::generate();
    db.connection
        .execute(
            "INSERT INTO resources(id,title,archived) VALUES(?1,?2,0)",
            params![id.as_uuid().to_string(), title],
        )
        .unwrap();
    id
}
fn document(db: &Database, resource: ResourceId, revision: u64, text: &str) {
    let doc = serde_json::json!({"schemaVersion":1,"blocks":[{"id":Uuid::new_v4(),"content":{"type":"paragraph","inlines":[{"text":text,"marks":[]}]}}]});
    let doc = Document::parse(&serde_json::to_vec(&doc).unwrap()).unwrap();
    let id = Uuid::new_v4().to_string();
    db.connection.execute("INSERT INTO resource_revisions(id,resource_id,revision,document,digest) VALUES(?1,?2,?3,?4,?5)", params![id, resource.as_uuid().to_string(), revision as i64, String::from_utf8(doc.canonical_bytes().unwrap()).unwrap(), doc.digest().unwrap()]).unwrap();
    db.connection.execute("INSERT INTO resource_heads(resource_id,revision_id) VALUES(?1,?2) ON CONFLICT(resource_id) DO UPDATE SET revision_id=excluded.revision_id", params![resource.as_uuid().to_string(), id]).unwrap();
}

#[test]
fn discovers_unicode_literal_titles_tags_and_current_canonical_text_without_archived_or_json_keys()
{
    let temp = tempfile::tempdir().unwrap();
    let db = Database::open(&temp.path().join("workspace")).unwrap();
    let id = resource(&db, "ÉNERGIE %_.*");
    document(&db, id, 1, "Квантовая λ漢字 and original");
    let tag = Uuid::new_v4().to_string();
    db.connection
        .execute("INSERT INTO tags(id,title) VALUES(?1,'ΒΕΤΑ')", [&tag])
        .unwrap();
    db.connection
        .execute(
            "INSERT INTO tag_resources(tag_id,resource_id,position) VALUES(?1,?2,0)",
            params![tag, id.as_uuid().to_string()],
        )
        .unwrap();
    let hidden = resource(&db, "ÉNERGIE archived");
    document(&db, hidden, 1, "Квантовая λ漢字");
    db.connection
        .execute(
            "UPDATE resources SET archived=1 WHERE id=?1",
            [hidden.as_uuid().to_string()],
        )
        .unwrap();
    db.connection
        .execute(
            "INSERT INTO library_meta(workspace_id,revision) VALUES(?1,7)",
            [db.id.as_str()],
        )
        .unwrap();
    for query in ["énergie", "%_.*", "βετα", "квантовая", "λ漢字"] {
        let page = db
            .search_resources(RequestContext::local_ui(), query, 0)
            .unwrap();
        assert_eq!(page.revision, 7);
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].resource_id, id);
        assert_eq!(page.items[0].snippet, "Квантовая λ漢字 and original");
    }
    for query in ["schemaVersion", "missing|ÉNERGIE"] {
        assert!(
            db.search_resources(RequestContext::local_ui(), query, 0)
                .unwrap()
                .items
                .is_empty()
        );
    }
    document(&db, id, 2, "Revised cooking source");
    assert!(
        db.search_resources(RequestContext::local_ui(), "original", 0)
            .unwrap()
            .items
            .is_empty()
    );
    assert_eq!(
        db.search_resources(RequestContext::local_ui(), "revised", 0)
            .unwrap()
            .items[0]
            .resource_id,
        id
    );
}

#[test]
fn pages_twenty_hits_and_returns_a_continuation_after_an_empty_two_hundred_row_scan() {
    let temp = tempfile::tempdir().unwrap();
    let db = Database::open(&temp.path().join("workspace")).unwrap();
    let mut ids = (0..201).map(|_| resource(&db, "match")).collect::<Vec<_>>();
    ids.sort_by_key(|id| id.as_uuid().to_string());
    db.connection
        .execute(
            "UPDATE resources SET title='last needle' WHERE id=?1",
            [ids[200].as_uuid().to_string()],
        )
        .unwrap();
    let empty = db
        .search_resources(RequestContext::local_ui(), "needle", 0)
        .unwrap();
    assert!(empty.items.is_empty());
    assert_eq!(empty.next_offset, Some(200));
    let last = db
        .search_resources(RequestContext::local_ui(), "needle", 200)
        .unwrap();
    assert_eq!(last.items[0].resource_id, ids[200]);
    assert_eq!(last.next_offset, None);
    let mut offset = Some(0);
    let mut found = Vec::new();
    for _ in 0..12 {
        let Some(cursor) = offset else {
            break;
        };
        let page = db
            .search_resources(RequestContext::local_ui(), "", cursor)
            .unwrap();
        assert!(page.items.len() <= 20);
        found.extend(page.items.into_iter().map(|hit| hit.resource_id));
        offset = page.next_offset;
    }
    assert_eq!(offset, None);
    assert_eq!(found, ids);
}

#[test]
fn validates_scalar_bounds_context_and_corrupt_documents_before_returning_results() {
    let temp = tempfile::tempdir().unwrap();
    let db = Database::open(&temp.path().join("workspace")).unwrap();
    assert_eq!(
        db.search_resources(RequestContext::unverified_external(), "", 0)
            .unwrap_err()
            .code,
        ErrorCode::PermissionDenied
    );
    for query in ["🧠".repeat(257), "a\0b".into()] {
        assert_eq!(
            db.search_resources(RequestContext::local_ui(), &query, 0)
                .unwrap_err()
                .code,
            ErrorCode::InvalidRequest
        );
    }
    assert!(
        db.search_resources(RequestContext::local_ui(), &"🧠".repeat(256), 0)
            .is_ok()
    );
    assert!(
        db.search_resources(
            RequestContext::local_ui(),
            "",
            super::super::types::MAX_REVISION as u64
        )
        .unwrap()
        .items
        .is_empty()
    );
    assert_eq!(
        db.search_resources(
            RequestContext::local_ui(),
            "",
            super::super::types::MAX_REVISION as u64 + 1
        )
        .unwrap_err()
        .code,
        ErrorCode::InvalidRequest
    );
    let id = resource(&db, "Notes");
    document(&db, id, 1, "safe");
    db.connection
        .execute("UPDATE resource_revisions SET document='{}'", [])
        .unwrap();
    assert_eq!(
        db.search_resources(RequestContext::local_ui(), "Notes", 0)
            .unwrap_err()
            .code,
        ErrorCode::RecoveryRequired
    );
    assert_eq!(
        db.connection
            .query_row("SELECT count(*) FROM resource_revisions", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
}

#[test]
fn typed_service_discovery_survives_restart_and_denies_untrusted_contexts() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("workspace");
    tauri::async_runtime::block_on(async {
        let service = ApplicationService::start(root.clone()).unwrap();
        let snapshot = service.library(RequestContext::local_ui()).await.unwrap();
        let id = ResourceId::generate();
        service
            .apply_content(
                RequestContext::local_ui(),
                ApplyContent {
                    contract_version: 1,
                    workspace_id: WorkspaceId::try_from(snapshot.workspace_id.to_string()).unwrap(),
                    operation_id: OperationId::generate(),
                    expected_revision: 0,
                    command: crate::organization::Command::CreateResource {
                        id,
                        title: "Quantum physics".into(),
                    },
                },
            )
            .await
            .unwrap();
        assert_eq!(
            service
                .search_resources(RequestContext::unverified_external(), "Quantum".into(), 0)
                .await
                .unwrap_err()
                .code,
            ErrorCode::PermissionDenied
        );
        service.shutdown().await.unwrap();
        let reopened = ApplicationService::start(root).unwrap();
        let page = reopened
            .search_resources(RequestContext::local_ui(), "quantum".into(), 0)
            .await
            .unwrap();
        assert_eq!(page.revision, 1);
        assert_eq!(page.items[0].resource_id, id);
        reopened.shutdown().await.unwrap();
    });
}
