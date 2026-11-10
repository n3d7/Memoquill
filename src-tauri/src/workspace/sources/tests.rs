use super::*;
use crate::organization::Command;
use std::io::Cursor;

#[test]
fn reviewed_omissions_and_empty_records_survive_restart_and_require_explicit_reopening() {
    let (temp, mut db, mut request) = fixture();
    let asset = db
        .register_asset(
            RequestContext::local_ui(),
            &RegisterAsset {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
            },
            &mut Cursor::new(b"key,value\n,\n"),
            MAX_SOURCE_BYTES,
        )
        .unwrap();
    request.source.filename = "rows.csv".into();
    request.source.digest = asset.digest;
    let source = db
        .save_source(RequestContext::local_ui(), &request)
        .unwrap();
    let job = db
        .create_ingestion_job(
            RequestContext::local_ui(),
            &CreateIngestionJob {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                source_id: source.id,
                source_revision: source.revision,
            },
        )
        .unwrap();
    let mapping = IngestionMapping {
        key_column: 0,
        title_column: 0,
        body_column: 1,
        json_properties: None,
    };
    let review = ReviewIngestionJob {
        contract_version: 1,
        workspace_id: db.id.clone(),
        operation_id: OperationId::generate(),
        job_id: job.id,
        expected_revision: 0,
        previous_job_id: None,
        expected_previous_revision: None,
        mapping: mapping.clone(),
        decisions: vec![
            IngestionDecision {
                index: Some(0),
                key: None,
                disposition: IngestionDisposition::Omitted,
            },
            IngestionDecision {
                index: Some(1),
                key: None,
                disposition: IngestionDisposition::Empty,
            },
        ],
    };
    let reviewed = db
        .review_ingestion_job(RequestContext::local_ui(), &review)
        .unwrap();
    let summaries = db
        .list_ingestion_jobs(RequestContext::local_ui(), source.resource_id, 0)
        .unwrap();
    assert!(
        !summaries
            .iter()
            .find(|item| item.id == reviewed.id)
            .unwrap()
            .eligible_baseline
    );
    drop(db);
    let mut db = Database::open(&temp.path().join("workspace")).unwrap();
    assert_eq!(
        db.review_ingestion_job(RequestContext::local_ui(), &review)
            .unwrap(),
        reviewed
    );
    assert_eq!(
        db.ingestion_job(RequestContext::local_ui(), job.id)
            .unwrap()
            .decisions
            .len(),
        2
    );
    let apply = ApplyIngestionJob {
        contract_version: 1,
        workspace_id: db.id.clone(),
        operation_id: OperationId::generate(),
        job_id: job.id,
        expected_revision: reviewed.revision,
        expected_library_revision: db.library(RequestContext::local_ui()).unwrap().revision,
        mapping: mapping.clone(),
        selected: vec![0],
    };
    assert_eq!(
        db.apply_ingestion_job(RequestContext::local_ui(), &apply)
            .unwrap_err()
            .code,
        ErrorCode::InvalidRequest
    );
    let reopen = ReviewIngestionJob {
        operation_id: OperationId::generate(),
        expected_revision: reviewed.revision,
        decisions: vec![IngestionDecision {
            index: Some(0),
            key: None,
            disposition: IngestionDisposition::Pending,
        }],
        ..review
    };
    let reopened = db
        .review_ingestion_job(RequestContext::local_ui(), &reopen)
        .unwrap();
    let apply = ApplyIngestionJob {
        expected_revision: reopened.revision,
        ..apply
    };
    assert_eq!(
        db.apply_ingestion_job(RequestContext::local_ui(), &apply)
            .unwrap()
            .outcomes
            .len(),
        1
    );
}

#[test]
fn reviewed_removed_key_remains_in_lineage_and_never_deletes_live_material() {
    let (temp, mut db, mut source_request) = fixture();
    let mapping = IngestionMapping {
        key_column: 0,
        title_column: 0,
        body_column: 1,
        json_properties: None,
    };
    let mut make_job = |db: &mut Database, bytes: &[u8], revision: u64| {
        let asset = db
            .register_asset(
                RequestContext::local_ui(),
                &RegisterAsset {
                    contract_version: 1,
                    workspace_id: db.id.clone(),
                    operation_id: OperationId::generate(),
                },
                &mut Cursor::new(bytes),
                MAX_SOURCE_BYTES,
            )
            .unwrap();
        source_request.source.id = Uuid::new_v4();
        source_request.source.filename = "rows.csv".into();
        source_request.source.digest = asset.digest;
        source_request.operation_id = OperationId::generate();
        source_request.expected_revision = revision;
        let source = db
            .save_source(RequestContext::local_ui(), &source_request)
            .unwrap();
        db.create_ingestion_job(
            RequestContext::local_ui(),
            &CreateIngestionJob {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                source_id: source.id,
                source_revision: source.revision,
            },
        )
        .unwrap()
    };
    let original = make_job(&mut db, b"a,one\nb,two\n", 0);
    let original = db
        .apply_ingestion_job(
            RequestContext::local_ui(),
            &ApplyIngestionJob {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                job_id: original.id,
                expected_revision: 0,
                expected_library_revision: db.library(RequestContext::local_ui()).unwrap().revision,
                mapping: mapping.clone(),
                selected: vec![0, 1],
            },
        )
        .unwrap();
    let second = make_job(&mut db, b"a,one\n", 1);
    let preview = db
        .preview_ingestion_reimport(RequestContext::local_ui(), second.id, original.id, &mapping)
        .unwrap();
    assert!(
        preview
            .items
            .iter()
            .any(|item| item.key == "b" && item.status == ReimportStatus::Removed)
    );
    let review = ReviewIngestionJob {
        contract_version: 1,
        workspace_id: db.id.clone(),
        operation_id: OperationId::generate(),
        job_id: second.id,
        expected_revision: 0,
        previous_job_id: Some(original.id),
        expected_previous_revision: Some(original.revision),
        mapping: mapping.clone(),
        decisions: vec![IngestionDecision {
            index: None,
            key: Some("b".into()),
            disposition: IngestionDisposition::RemovedRetained,
        }],
    };
    let reviewed = db
        .review_ingestion_job(RequestContext::local_ui(), &review)
        .unwrap();
    let summaries = db
        .list_ingestion_jobs(RequestContext::local_ui(), original.source.resource_id, 0)
        .unwrap();
    assert!(
        !summaries
            .iter()
            .find(|item| item.id == original.id)
            .unwrap()
            .eligible_baseline
    );
    assert!(
        summaries
            .iter()
            .find(|item| item.id == reviewed.id)
            .unwrap()
            .eligible_baseline
    );
    let id = original.outcomes[1].resource_id;
    drop(db);
    let mut db = Database::open(&temp.path().join("workspace")).unwrap();
    assert_eq!(
        db.review_ingestion_job(RequestContext::local_ui(), &review)
            .unwrap(),
        reviewed
    );
    assert!(!db.library(RequestContext::local_ui()).unwrap().resources[&id].archived);
    let third = make_job(&mut db, b"a,one\nb,two\n", 2);
    let comparison = db
        .preview_ingestion_reimport(RequestContext::local_ui(), third.id, reviewed.id, &mapping)
        .unwrap();
    let restored = comparison
        .items
        .iter()
        .find(|item| item.key == "b")
        .unwrap();
    assert_eq!(
        (restored.status, restored.resource_id),
        (ReimportStatus::Unchanged, Some(id))
    );
    db.cancel_ingestion_job(
        RequestContext::local_ui(),
        &CancelIngestionJob {
            contract_version: 1,
            workspace_id: db.id.clone(),
            operation_id: OperationId::generate(),
            job_id: reviewed.id,
            expected_revision: reviewed.revision,
        },
    )
    .unwrap();
    let summaries = db
        .list_ingestion_jobs(RequestContext::local_ui(), original.source.resource_id, 0)
        .unwrap();
    assert!(
        summaries
            .iter()
            .find(|item| item.id == original.id)
            .unwrap()
            .eligible_baseline
    );
    assert!(
        !summaries
            .iter()
            .find(|item| item.id == reviewed.id)
            .unwrap()
            .eligible_baseline
    );
    let other = make_job(&mut db, b"a,one\nb,two\n", 3);
    let applied = db
        .apply_ingestion_reimport(
            RequestContext::local_ui(),
            &ApplyReimportJob {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                job_id: other.id,
                previous_job_id: original.id,
                expected_revision: 0,
                expected_previous_revision: original.revision,
                expected_library_revision: db.library(RequestContext::local_ui()).unwrap().revision,
                mapping,
                selected: vec![1],
                resolutions: vec![],
            },
        )
        .unwrap();
    assert_eq!(applied.outcomes[0].resource_id, id);
}

#[test]
fn partial_reimport_preserves_unselected_lineage_in_next_generation() {
    let (_temp, mut db, mut source_request) = fixture();
    let mapping = IngestionMapping {
        key_column: 0,
        title_column: 0,
        body_column: 1,
        json_properties: None,
    };
    let mut make_job = |db: &mut Database, bytes: &[u8], source_revision: u64| {
        let asset = db
            .register_asset(
                RequestContext::local_ui(),
                &RegisterAsset {
                    contract_version: 1,
                    workspace_id: db.id.clone(),
                    operation_id: OperationId::generate(),
                },
                &mut Cursor::new(bytes),
                MAX_SOURCE_BYTES,
            )
            .unwrap();
        source_request.source.id = Uuid::new_v4();
        source_request.source.filename = "rows.csv".into();
        source_request.source.digest = asset.digest;
        source_request.operation_id = OperationId::generate();
        source_request.expected_revision = source_revision;
        let source = db
            .save_source(RequestContext::local_ui(), &source_request)
            .unwrap();
        db.create_ingestion_job(
            RequestContext::local_ui(),
            &CreateIngestionJob {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                source_id: source.id,
                source_revision: source.revision,
            },
        )
        .unwrap()
    };
    let first = make_job(&mut db, b"a,one\nb,two\n", 0);
    let first = db
        .apply_ingestion_job(
            RequestContext::local_ui(),
            &ApplyIngestionJob {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                job_id: first.id,
                expected_revision: 0,
                expected_library_revision: db.library(RequestContext::local_ui()).unwrap().revision,
                mapping: mapping.clone(),
                selected: vec![0, 1],
            },
        )
        .unwrap();
    let second = make_job(&mut db, b"a,one\nb,two\n", 1);
    let second = db
        .apply_ingestion_reimport(
            RequestContext::local_ui(),
            &ApplyReimportJob {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                job_id: second.id,
                previous_job_id: first.id,
                expected_revision: 0,
                expected_previous_revision: first.revision,
                expected_library_revision: db.library(RequestContext::local_ui()).unwrap().revision,
                mapping: mapping.clone(),
                selected: vec![0],
                resolutions: vec![],
            },
        )
        .unwrap();
    let third = make_job(&mut db, b"a,one\nb,two\n", 2);
    let preview = db
        .preview_ingestion_reimport(RequestContext::local_ui(), third.id, second.id, &mapping)
        .unwrap();
    let b = preview.items.iter().find(|item| item.key == "b").unwrap();
    assert_eq!(b.status, ReimportStatus::Unchanged);
    assert_eq!(b.resource_id, Some(first.outcomes[1].resource_id));
    let third = db
        .apply_ingestion_reimport(
            RequestContext::local_ui(),
            &ApplyReimportJob {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                job_id: third.id,
                previous_job_id: second.id,
                expected_revision: 0,
                expected_previous_revision: second.revision,
                expected_library_revision: db.library(RequestContext::local_ui()).unwrap().revision,
                mapping: mapping.clone(),
                selected: vec![1],
                resolutions: vec![],
            },
        )
        .unwrap();
    assert_eq!(third.outcomes[0].resource_id, first.outcomes[1].resource_id);
    let late_parent = ApplyReimportJob {
        contract_version: 1,
        workspace_id: db.id.clone(),
        operation_id: OperationId::generate(),
        job_id: second.id,
        previous_job_id: first.id,
        expected_revision: second.revision,
        expected_previous_revision: first.revision,
        expected_library_revision: db.library(RequestContext::local_ui()).unwrap().revision,
        mapping: mapping.clone(),
        selected: vec![1],
        resolutions: vec![],
    };
    assert_eq!(
        db.apply_ingestion_reimport(RequestContext::local_ui(), &late_parent)
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        db.cancel_ingestion_job(
            RequestContext::local_ui(),
            &CancelIngestionJob {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                job_id: second.id,
                expected_revision: second.revision
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::RevisionConflict
    );
    let resource_id = first.outcomes[1].resource_id;
    let saved = db
        .read_document(RequestContext::local_ui(), resource_id, None)
        .unwrap()
        .unwrap();
    let doc = crate::document::Document::parse(serde_json::to_string(&serde_json::json!({"schemaVersion":1,"blocks":[{"id":Uuid::new_v4(),"content":{"type":"paragraph","inlines":[{"text":"human","marks":[]}]}}]})).unwrap().as_bytes()).unwrap();
    db.save_document(
        RequestContext::local_ui(),
        &SaveDocument {
            contract_version: 1,
            workspace_id: db.id.clone(),
            operation_id: OperationId::generate(),
            resource_id,
            expected_revision: saved.revision,
            document: doc,
        },
    )
    .unwrap();
    let fourth = make_job(&mut db, b"a,one\nb,changed\n", 3);
    let comparison = db
        .preview_ingestion_reimport(RequestContext::local_ui(), fourth.id, third.id, &mapping)
        .unwrap();
    assert_eq!(
        comparison
            .items
            .iter()
            .find(|item| item.key == "b")
            .unwrap()
            .status,
        ReimportStatus::Conflict
    );
    db.undo_ingestion_job(
        RequestContext::local_ui(),
        &UndoIngestionJob {
            contract_version: 1,
            workspace_id: db.id.clone(),
            operation_id: OperationId::generate(),
            job_id: third.id,
            expected_revision: third.revision,
            expected_library_revision: db.library(RequestContext::local_ui()).unwrap().revision,
        },
    )
    .unwrap();
    assert_eq!(
        db.read_document(RequestContext::local_ui(), resource_id, None)
            .unwrap()
            .unwrap()
            .document
            .text(),
        "human"
    );
    assert!(!db.library(RequestContext::local_ui()).unwrap().resources[&resource_id].archived);
}

#[test]
fn undone_import_cannot_be_a_baseline_but_allows_a_new_visible_import() {
    let (_temp, mut db, mut source_request) = fixture();
    let asset = db
        .register_asset(
            RequestContext::local_ui(),
            &RegisterAsset {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
            },
            &mut Cursor::new(b"a,one\n"),
            MAX_SOURCE_BYTES,
        )
        .unwrap();
    source_request.source.filename = "rows.csv".into();
    source_request.source.digest = asset.digest;
    let source = db
        .save_source(RequestContext::local_ui(), &source_request)
        .unwrap();
    let create = |db: &mut Database| {
        db.create_ingestion_job(
            RequestContext::local_ui(),
            &CreateIngestionJob {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                source_id: source.id,
                source_revision: source.revision,
            },
        )
        .unwrap()
    };
    let first = create(&mut db);
    let mapping = IngestionMapping {
        key_column: 0,
        title_column: 0,
        body_column: 1,
        json_properties: None,
    };
    let first = db
        .apply_ingestion_job(
            RequestContext::local_ui(),
            &ApplyIngestionJob {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                job_id: first.id,
                expected_revision: 0,
                expected_library_revision: db.library(RequestContext::local_ui()).unwrap().revision,
                mapping: mapping.clone(),
                selected: vec![0],
            },
        )
        .unwrap();
    let first = db
        .undo_ingestion_job(
            RequestContext::local_ui(),
            &UndoIngestionJob {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                job_id: first.id,
                expected_revision: first.revision,
                expected_library_revision: db.library(RequestContext::local_ui()).unwrap().revision,
            },
        )
        .unwrap();
    let next = create(&mut db);
    assert_eq!(
        db.preview_ingestion_reimport(RequestContext::local_ui(), next.id, first.id, &mapping)
            .unwrap_err()
            .code,
        ErrorCode::InvalidRequest
    );
    let imported = db
        .apply_ingestion_job(
            RequestContext::local_ui(),
            &ApplyIngestionJob {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                job_id: next.id,
                expected_revision: 0,
                expected_library_revision: db.library(RequestContext::local_ui()).unwrap().revision,
                mapping,
                selected: vec![0],
            },
        )
        .unwrap();
    let library = db.library(RequestContext::local_ui()).unwrap();
    assert!(!library.resources[&imported.outcomes[0].resource_id].archived);
    assert_ne!(
        first.outcomes[0].resource_id,
        imported.outcomes[0].resource_id
    );
}

#[test]
fn independently_archived_material_is_not_reported_as_an_unchanged_success() {
    let (_temp, mut db, mut request) = fixture();
    let asset = db
        .register_asset(
            RequestContext::local_ui(),
            &RegisterAsset {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
            },
            &mut Cursor::new(b"key,value\n"),
            MAX_SOURCE_BYTES,
        )
        .unwrap();
    request.source.filename = "rows.csv".into();
    request.source.digest = asset.digest;
    let source = db
        .save_source(RequestContext::local_ui(), &request)
        .unwrap();
    let create = |db: &mut Database| {
        db.create_ingestion_job(
            RequestContext::local_ui(),
            &CreateIngestionJob {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                source_id: source.id,
                source_revision: source.revision,
            },
        )
        .unwrap()
    };
    let job = create(&mut db);
    let mapping = IngestionMapping {
        key_column: 0,
        title_column: 0,
        body_column: 1,
        json_properties: None,
    };
    let job = db
        .apply_ingestion_job(
            RequestContext::local_ui(),
            &ApplyIngestionJob {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                job_id: job.id,
                expected_revision: 0,
                expected_library_revision: db.library(RequestContext::local_ui()).unwrap().revision,
                mapping: mapping.clone(),
                selected: vec![0],
            },
        )
        .unwrap();
    db.apply_content(
        RequestContext::local_ui(),
        &ApplyContent {
            contract_version: 1,
            workspace_id: db.id.clone(),
            operation_id: OperationId::generate(),
            expected_revision: db.library(RequestContext::local_ui()).unwrap().revision,
            command: Command::Archive {
                target: crate::organization::ArchiveTarget::Resource(job.outcomes[0].resource_id),
                archived: true,
            },
        },
    )
    .unwrap();
    let next = create(&mut db);
    let preview = db
        .preview_ingestion_reimport(RequestContext::local_ui(), next.id, job.id, &mapping)
        .unwrap();
    assert_eq!(preview.items[0].status, ReimportStatus::Unresolved);
}

#[test]
fn canonical_search_survives_missing_or_corrupt_originals_after_restart() {
    for damage in ["missing", "corrupt"] {
        let (temp, mut db, mut request) = fixture();
        let asset = db
            .register_asset(
                RequestContext::local_ui(),
                &RegisterAsset {
                    contract_version: 1,
                    workspace_id: db.id.clone(),
                    operation_id: OperationId::generate(),
                },
                &mut Cursor::new(b"unique-key,canonical needle\n"),
                MAX_SOURCE_BYTES,
            )
            .unwrap();
        request.source.filename = "rows.csv".into();
        request.source.digest = asset.digest.clone();
        let source = db
            .save_source(RequestContext::local_ui(), &request)
            .unwrap();
        let job = db
            .create_ingestion_job(
                RequestContext::local_ui(),
                &CreateIngestionJob {
                    contract_version: 1,
                    workspace_id: db.id.clone(),
                    operation_id: OperationId::generate(),
                    source_id: source.id,
                    source_revision: source.revision,
                },
            )
            .unwrap();
        let applied = db
            .apply_ingestion_job(
                RequestContext::local_ui(),
                &ApplyIngestionJob {
                    contract_version: 1,
                    workspace_id: db.id.clone(),
                    operation_id: OperationId::generate(),
                    job_id: job.id,
                    expected_revision: 0,
                    expected_library_revision: db
                        .library(RequestContext::local_ui())
                        .unwrap()
                        .revision,
                    mapping: IngestionMapping {
                        key_column: 0,
                        title_column: 0,
                        body_column: 1,
                        json_properties: None,
                    },
                    selected: vec![0],
                },
            )
            .unwrap();
        let path = assets::asset_path(db.owner.root(), &asset.digest);
        drop(db);
        if damage == "missing" {
            std::fs::remove_file(&path).unwrap();
        } else {
            std::fs::write(&path, b"corrupted original").unwrap();
        }
        let db = Database::open(&temp.path().join("workspace")).unwrap();
        let page = db
            .search_resources(RequestContext::local_ui(), "canonical needle", 0)
            .unwrap();
        assert_eq!(page.items[0].resource_id, applied.outcomes[0].resource_id);
        assert!(!page.items[0].references[0].original_available);
        assert!(db.recovery_report().missing_assets + db.recovery_report().corrupt_assets > 0);
    }
}

fn fixture() -> (tempfile::TempDir, Database, SaveSource) {
    let temp = tempfile::tempdir().unwrap();
    let mut db = Database::open(&temp.path().join("workspace")).unwrap();
    let resource_id = ResourceId::generate();
    db.apply_content(
        RequestContext::local_ui(),
        &ApplyContent {
            contract_version: 1,
            workspace_id: db.id.clone(),
            operation_id: OperationId::generate(),
            expected_revision: 0,
            command: Command::CreateResource {
                id: resource_id,
                title: "Resource".into(),
            },
        },
    )
    .unwrap();
    let asset = db
        .register_asset(
            RequestContext::local_ui(),
            &RegisterAsset {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
            },
            &mut Cursor::new(b"<script>passive text only</script>"),
            MAX_SOURCE_BYTES,
        )
        .unwrap();
    let request = SaveSource {
        contract_version: 1,
        workspace_id: db.id.clone(),
        operation_id: OperationId::generate(),
        expected_revision: 0,
        source: NewSource {
            id: Uuid::new_v4(),
            resource_id,
            digest: asset.digest,
            filename: "Original.html".into(),
            media_type: "text/html".into(),
            attribution: Some("Author".into()),
            license: Some("CC BY 4.0".into()),
            origin_url: Some("https://example.test/original".into()),
        },
    };
    (temp, db, request)
}

#[test]
fn ingestion_job_and_extracted_records_survive_restart_and_exact_retry() {
    let (temp, mut db, mut source_request) = fixture();
    let bytes = b"first,one\nsecond,two\n";
    let asset = db
        .register_asset(
            RequestContext::local_ui(),
            &RegisterAsset {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
            },
            &mut Cursor::new(bytes),
            MAX_SOURCE_BYTES,
        )
        .unwrap();
    source_request.source.filename = "rows.csv".into();
    source_request.source.digest = asset.digest;
    let source = db
        .save_source(RequestContext::local_ui(), &source_request)
        .unwrap();
    let request = CreateIngestionJob {
        contract_version: 1,
        workspace_id: db.id.clone(),
        operation_id: OperationId::generate(),
        source_id: source.id,
        source_revision: source.revision,
    };
    let mut reused = request.clone();
    reused.operation_id = source_request.operation_id.clone();
    assert_eq!(
        db.create_ingestion_job(RequestContext::local_ui(), &reused)
            .unwrap_err()
            .code,
        ErrorCode::IdempotencyConflict
    );
    let job = db
        .create_ingestion_job(RequestContext::local_ui(), &request)
        .unwrap();
    assert_eq!(job.extraction.records.len(), 2);
    drop(db);
    let mut db = Database::open(&temp.path().join("workspace")).unwrap();
    assert_eq!(
        db.create_ingestion_job(RequestContext::local_ui(), &request)
            .unwrap(),
        job
    );
    assert_eq!(
        db.ingestion_job(RequestContext::local_ui(), job.id)
            .unwrap(),
        job
    );
    assert_eq!(
        db.list_ingestion_jobs(RequestContext::local_ui(), source.resource_id, 0)
            .unwrap()
            .iter()
            .map(|item| item.id)
            .collect::<Vec<_>>(),
        vec![job.id]
    );
    drop(db);
    tauri::async_runtime::block_on(async {
        let service = ApplicationService::start(temp.path().join("workspace")).unwrap();
        assert_eq!(
            service
                .ingestion_job(RequestContext::local_ui(), job.id)
                .await
                .unwrap(),
            job
        );
        assert_eq!(
            service
                .create_ingestion_job(RequestContext::local_ui(), request)
                .await
                .unwrap(),
            job
        );
        service.shutdown().await.unwrap();
    });
}

#[test]
fn cancelling_a_staged_ingestion_is_durable_and_does_not_change_the_library() {
    let (temp, mut db, mut request) = fixture();
    request.source.filename = "rows.txt".into();
    let source = db
        .save_source(RequestContext::local_ui(), &request)
        .unwrap();
    let create = CreateIngestionJob {
        contract_version: 1,
        workspace_id: db.id.clone(),
        operation_id: OperationId::generate(),
        source_id: source.id,
        source_revision: source.revision,
    };
    let job = db
        .create_ingestion_job(RequestContext::local_ui(), &create)
        .unwrap();
    let before = db.library(RequestContext::local_ui()).unwrap();
    let cancel = CancelIngestionJob {
        contract_version: 1,
        workspace_id: db.id.clone(),
        operation_id: OperationId::generate(),
        job_id: job.id,
        expected_revision: 0,
    };
    let cancelled = db
        .cancel_ingestion_job(RequestContext::local_ui(), &cancel)
        .unwrap();
    assert_eq!(cancelled.status, IngestionStatus::Cancelled);
    assert_eq!(cancelled.revision, 1);
    assert_eq!(db.library(RequestContext::local_ui()).unwrap(), before);
    drop(db);
    let mut db = Database::open(&temp.path().join("workspace")).unwrap();
    assert_eq!(
        db.cancel_ingestion_job(RequestContext::local_ui(), &cancel)
            .unwrap(),
        cancelled
    );
    assert_eq!(
        db.ingestion_job(RequestContext::local_ui(), job.id)
            .unwrap(),
        cancelled
    );
    drop(db);
    tauri::async_runtime::block_on(async {
        let service = ApplicationService::start(temp.path().join("workspace")).unwrap();
        assert_eq!(
            service
                .cancel_ingestion_job(RequestContext::local_ui(), cancel)
                .await
                .unwrap(),
            cancelled
        );
        service.shutdown().await.unwrap();
    });
}

#[test]
fn reviewed_subset_is_atomic_and_exact_retry_never_duplicates_resources() {
    let (temp, mut db, mut request) = fixture();
    let bytes = b"alpha,first\nbeta,second\n";
    let asset = db
        .register_asset(
            RequestContext::local_ui(),
            &RegisterAsset {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
            },
            &mut Cursor::new(bytes),
            MAX_SOURCE_BYTES,
        )
        .unwrap();
    request.source.filename = "rows.csv".into();
    request.source.digest = asset.digest;
    let source = db
        .save_source(RequestContext::local_ui(), &request)
        .unwrap();
    let job = db
        .create_ingestion_job(
            RequestContext::local_ui(),
            &CreateIngestionJob {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                source_id: source.id,
                source_revision: source.revision,
            },
        )
        .unwrap();
    let before = db.library(RequestContext::local_ui()).unwrap();
    let apply = ApplyIngestionJob {
        contract_version: 1,
        workspace_id: db.id.clone(),
        operation_id: OperationId::generate(),
        job_id: job.id,
        expected_revision: 0,
        expected_library_revision: before.revision,
        mapping: IngestionMapping {
            key_column: 0,
            title_column: 0,
            body_column: 1,
            json_properties: None,
        },
        selected: vec![1],
    };
    let applied = db
        .apply_ingestion_job(RequestContext::local_ui(), &apply)
        .unwrap();
    assert_eq!(applied.outcomes.len(), 1);
    let outcome = &applied.outcomes[0];
    assert_eq!(outcome.key, "beta");
    assert_eq!(
        db.read_document(RequestContext::local_ui(), outcome.resource_id, None)
            .unwrap()
            .unwrap()
            .document
            .text(),
        "second"
    );
    let after = db.library(RequestContext::local_ui()).unwrap();
    assert_eq!(after.resources.len(), before.resources.len() + 1);
    let hit = db
        .search_resources(RequestContext::local_ui(), "second", 0)
        .unwrap()
        .items
        .remove(0);
    assert_eq!(hit.references[0].source_id, source.id);
    assert_eq!(hit.references[0].locator.value, "2");
    drop(db);
    let mut db = Database::open(&temp.path().join("workspace")).unwrap();
    assert_eq!(
        db.apply_ingestion_job(RequestContext::local_ui(), &apply)
            .unwrap(),
        applied
    );
    assert_eq!(db.library(RequestContext::local_ui()).unwrap(), after);
    let another = db
        .create_ingestion_job(
            RequestContext::local_ui(),
            &CreateIngestionJob {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                source_id: source.id,
                source_revision: source.revision,
            },
        )
        .unwrap();
    let mut duplicate = apply.clone();
    duplicate.operation_id = OperationId::generate();
    duplicate.job_id = another.id;
    duplicate.expected_library_revision = after.revision;
    assert!(
        db.apply_ingestion_job(RequestContext::local_ui(), &duplicate)
            .is_err()
    );
    assert_eq!(db.library(RequestContext::local_ui()).unwrap(), after);
    drop(db);
    tauri::async_runtime::block_on(async {
        let service = ApplicationService::start(temp.path().join("workspace")).unwrap();
        assert_eq!(
            service
                .apply_ingestion_job(RequestContext::local_ui(), apply)
                .await
                .unwrap(),
            applied
        );
        service.shutdown().await.unwrap();
    });
}

#[test]
fn invalid_later_selected_record_rolls_back_the_whole_batch() {
    let (_temp, mut db, mut request) = fixture();
    let bytes = b"good,first\n,second\n";
    let asset = db
        .register_asset(
            RequestContext::local_ui(),
            &RegisterAsset {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
            },
            &mut Cursor::new(bytes),
            MAX_SOURCE_BYTES,
        )
        .unwrap();
    request.source.filename = "rows.csv".into();
    request.source.digest = asset.digest;
    let source = db
        .save_source(RequestContext::local_ui(), &request)
        .unwrap();
    let job = db
        .create_ingestion_job(
            RequestContext::local_ui(),
            &CreateIngestionJob {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                source_id: source.id,
                source_revision: source.revision,
            },
        )
        .unwrap();
    let before = db.library(RequestContext::local_ui()).unwrap();
    let apply = ApplyIngestionJob {
        contract_version: 1,
        workspace_id: db.id.clone(),
        operation_id: OperationId::generate(),
        job_id: job.id,
        expected_revision: 0,
        expected_library_revision: before.revision,
        mapping: IngestionMapping {
            key_column: 1,
            title_column: 0,
            body_column: 1,
            json_properties: None,
        },
        selected: vec![0, 1],
    };
    assert!(
        db.apply_ingestion_job(RequestContext::local_ui(), &apply)
            .is_err()
    );
    assert_eq!(db.library(RequestContext::local_ui()).unwrap(), before);
    assert_eq!(
        db.ingestion_job(RequestContext::local_ui(), job.id)
            .unwrap(),
        job
    );
    assert!(
        db.operation_receipt(RequestContext::local_ui(), &apply.operation_id)
            .unwrap()
            .is_none()
    );
}

#[test]
fn undo_archives_only_unchanged_imports_and_refuses_later_human_edits() {
    let (_temp, mut db, mut request) = fixture();
    let asset = db
        .register_asset(
            RequestContext::local_ui(),
            &RegisterAsset {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
            },
            &mut Cursor::new(b"one,body\n"),
            MAX_SOURCE_BYTES,
        )
        .unwrap();
    request.source.filename = "data.csv".into();
    request.source.digest = asset.digest;
    let source = db
        .save_source(RequestContext::local_ui(), &request)
        .unwrap();
    let job = db
        .create_ingestion_job(
            RequestContext::local_ui(),
            &CreateIngestionJob {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                source_id: source.id,
                source_revision: source.revision,
            },
        )
        .unwrap();
    let applied = db
        .apply_ingestion_job(
            RequestContext::local_ui(),
            &ApplyIngestionJob {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                job_id: job.id,
                expected_revision: 0,
                expected_library_revision: db.library(RequestContext::local_ui()).unwrap().revision,
                mapping: IngestionMapping {
                    key_column: 0,
                    title_column: 0,
                    body_column: 1,
                    json_properties: None,
                },
                selected: vec![0],
            },
        )
        .unwrap();
    let preview = db
        .preview_ingestion_undo(RequestContext::local_ui(), applied.id)
        .unwrap();
    assert!(preview.can_undo);
    let current = db.library(RequestContext::local_ui()).unwrap();
    db.apply_content(
        RequestContext::local_ui(),
        &ApplyContent {
            contract_version: 1,
            workspace_id: db.id.clone(),
            operation_id: OperationId::generate(),
            expected_revision: current.revision,
            command: Command::RenameResource {
                id: applied.outcomes[0].resource_id,
                title: "human".into(),
            },
        },
    )
    .unwrap();
    assert!(
        !db.preview_ingestion_undo(RequestContext::local_ui(), applied.id)
            .unwrap()
            .can_undo
    );
    let undo = UndoIngestionJob {
        contract_version: 1,
        workspace_id: db.id.clone(),
        operation_id: OperationId::generate(),
        job_id: applied.id,
        expected_revision: applied.revision,
        expected_library_revision: db.library(RequestContext::local_ui()).unwrap().revision,
    };
    assert_eq!(
        db.undo_ingestion_job(RequestContext::local_ui(), &undo)
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert!(
        !db.library(RequestContext::local_ui()).unwrap().resources
            [&applied.outcomes[0].resource_id]
            .archived
    );
    let current = db.library(RequestContext::local_ui()).unwrap();
    db.apply_content(
        RequestContext::local_ui(),
        &ApplyContent {
            contract_version: 1,
            workspace_id: db.id.clone(),
            operation_id: OperationId::generate(),
            expected_revision: current.revision,
            command: Command::RenameResource {
                id: applied.outcomes[0].resource_id,
                title: "one".into(),
            },
        },
    )
    .unwrap();
    let mut valid = undo;
    valid.expected_library_revision = db.library(RequestContext::local_ui()).unwrap().revision;
    let undone = db
        .undo_ingestion_job(RequestContext::local_ui(), &valid)
        .unwrap();
    assert_eq!(undone.status, IngestionStatus::Undone);
    assert!(
        db.library(RequestContext::local_ui()).unwrap().resources[&applied.outcomes[0].resource_id]
            .archived
    );
    assert_eq!(
        db.undo_ingestion_job(RequestContext::local_ui(), &valid)
            .unwrap(),
        undone
    );
}

#[test]
fn reimport_preview_detects_changed_source_and_manual_document_edits() {
    let (_temp, mut db, mut request) = fixture();
    let asset = db
        .register_asset(
            RequestContext::local_ui(),
            &RegisterAsset {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
            },
            &mut Cursor::new(b"key,old\n"),
            MAX_SOURCE_BYTES,
        )
        .unwrap();
    request.source.filename = "data.csv".into();
    request.source.digest = asset.digest;
    let original = db
        .save_source(RequestContext::local_ui(), &request)
        .unwrap();
    let old = db
        .create_ingestion_job(
            RequestContext::local_ui(),
            &CreateIngestionJob {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                source_id: original.id,
                source_revision: original.revision,
            },
        )
        .unwrap();
    let mapping = IngestionMapping {
        key_column: 0,
        title_column: 0,
        body_column: 1,
        json_properties: None,
    };
    let applied = db
        .apply_ingestion_job(
            RequestContext::local_ui(),
            &ApplyIngestionJob {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                job_id: old.id,
                expected_revision: 0,
                expected_library_revision: db.library(RequestContext::local_ui()).unwrap().revision,
                mapping: mapping.clone(),
                selected: vec![0],
            },
        )
        .unwrap();
    let asset = db
        .register_asset(
            RequestContext::local_ui(),
            &RegisterAsset {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
            },
            &mut Cursor::new(b"key,new\n"),
            MAX_SOURCE_BYTES,
        )
        .unwrap();
    request.source.id = Uuid::new_v4();
    request.source.digest = asset.digest;
    request.operation_id = OperationId::generate();
    request.expected_revision = 1;
    let updated = db
        .save_source(RequestContext::local_ui(), &request)
        .unwrap();
    let new = db
        .create_ingestion_job(
            RequestContext::local_ui(),
            &CreateIngestionJob {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                source_id: updated.id,
                source_revision: updated.revision,
            },
        )
        .unwrap();
    let preview = db
        .preview_ingestion_reimport(RequestContext::local_ui(), new.id, old.id, &mapping)
        .unwrap();
    assert_eq!(preview.items[0].status, ReimportStatus::Changed);
    let reimport = ApplyReimportJob {
        contract_version: 1,
        workspace_id: db.id.clone(),
        operation_id: OperationId::generate(),
        job_id: new.id,
        previous_job_id: old.id,
        expected_revision: 0,
        expected_previous_revision: applied.revision,
        expected_library_revision: db.library(RequestContext::local_ui()).unwrap().revision,
        mapping: mapping.clone(),
        selected: vec![0],
        resolutions: vec![],
    };
    let updated_job = db
        .apply_ingestion_reimport(RequestContext::local_ui(), &reimport)
        .unwrap();
    assert_eq!(
        updated_job.outcomes[0].resource_id,
        applied.outcomes[0].resource_id
    );
    assert_eq!(
        db.read_document(
            RequestContext::local_ui(),
            applied.outcomes[0].resource_id,
            None
        )
        .unwrap()
        .unwrap()
        .document
        .text(),
        "new"
    );
    assert_eq!(
        db.apply_ingestion_reimport(RequestContext::local_ui(), &reimport)
            .unwrap(),
        updated_job
    );
    assert!(
        db.preview_ingestion_undo(RequestContext::local_ui(), updated_job.id)
            .unwrap()
            .can_undo
    );
    let undo = UndoIngestionJob {
        contract_version: 1,
        workspace_id: db.id.clone(),
        operation_id: OperationId::generate(),
        job_id: updated_job.id,
        expected_revision: updated_job.revision,
        expected_library_revision: db.library(RequestContext::local_ui()).unwrap().revision,
    };
    db.undo_ingestion_job(RequestContext::local_ui(), &undo)
        .unwrap();
    assert_eq!(
        db.read_document(
            RequestContext::local_ui(),
            applied.outcomes[0].resource_id,
            None
        )
        .unwrap()
        .unwrap()
        .document
        .text(),
        "old"
    );
    let saved = db
        .read_document(
            RequestContext::local_ui(),
            applied.outcomes[0].resource_id,
            None,
        )
        .unwrap()
        .unwrap();
    let doc = crate::document::Document::parse(serde_json::to_string(&serde_json::json!({"schemaVersion":1,"blocks":[{"id":Uuid::new_v4(),"content":{"type":"paragraph","inlines":[{"text":"human","marks":[]}]}}]})).unwrap().as_bytes()).unwrap();
    db.save_document(
        RequestContext::local_ui(),
        &SaveDocument {
            contract_version: 1,
            workspace_id: db.id.clone(),
            operation_id: OperationId::generate(),
            resource_id: saved.resource_id,
            expected_revision: saved.revision,
            document: doc,
        },
    )
    .unwrap();
    db.apply_content(
        RequestContext::local_ui(),
        &ApplyContent {
            contract_version: 1,
            workspace_id: db.id.clone(),
            operation_id: OperationId::generate(),
            expected_revision: db.library(RequestContext::local_ui()).unwrap().revision,
            command: Command::RenameResource {
                id: saved.resource_id,
                title: "Human title".into(),
            },
        },
    )
    .unwrap();
    let human = db
        .read_document(RequestContext::local_ui(), saved.resource_id, None)
        .unwrap()
        .unwrap();
    let preview = db
        .preview_ingestion_reimport(RequestContext::local_ui(), new.id, old.id, &mapping)
        .unwrap();
    assert_eq!(preview.items[0].status, ReimportStatus::Conflict);
    let values = serde_json::to_value(&preview.items[0]).unwrap();
    assert_eq!(values["prior"]["body"], "old");
    assert_eq!(values["prior"]["locator"]["value"], "1");
    assert_eq!(values["currentBody"], "human");
    assert_eq!(values["incoming"]["body"], "new");
    assert_eq!(values["incoming"]["locator"]["value"], "1");

    let retry_job = db
        .create_ingestion_job(
            RequestContext::local_ui(),
            &CreateIngestionJob {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                source_id: updated.id,
                source_revision: updated.revision,
            },
        )
        .unwrap();
    let preview = db
        .preview_ingestion_reimport(RequestContext::local_ui(), retry_job.id, old.id, &mapping)
        .unwrap();
    let mut conflict_request = ApplyReimportJob {
        contract_version: 1,
        workspace_id: db.id.clone(),
        operation_id: OperationId::generate(),
        job_id: retry_job.id,
        previous_job_id: old.id,
        expected_revision: 0,
        expected_previous_revision: applied.revision,
        expected_library_revision: db.library(RequestContext::local_ui()).unwrap().revision,
        mapping: mapping.clone(),
        selected: vec![0],
        resolutions: vec![],
    };
    assert_eq!(
        db.apply_ingestion_reimport(RequestContext::local_ui(), &conflict_request)
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    conflict_request.operation_id = OperationId::generate();
    conflict_request.resolutions.push(ReimportResolution {
        index: 0,
        expected_document_revision_id: preview.items[0].current_revision_id.unwrap(),
        expected_title: preview.items[0].current_title.clone().unwrap(),
    });
    let resolved = db
        .apply_ingestion_reimport(RequestContext::local_ui(), &conflict_request)
        .unwrap();
    assert_eq!(
        db.read_document(
            RequestContext::local_ui(),
            applied.outcomes[0].resource_id,
            None
        )
        .unwrap()
        .unwrap()
        .document
        .text(),
        "new"
    );
    let reimported = db
        .read_document(RequestContext::local_ui(), saved.resource_id, None)
        .unwrap()
        .unwrap();
    let undo = UndoIngestionJob {
        contract_version: 1,
        workspace_id: db.id.clone(),
        operation_id: OperationId::generate(),
        job_id: resolved.id,
        expected_revision: resolved.revision,
        expected_library_revision: db.library(RequestContext::local_ui()).unwrap().revision,
    };
    db.undo_ingestion_job(RequestContext::local_ui(), &undo)
        .unwrap();
    assert_eq!(
        db.read_document(
            RequestContext::local_ui(),
            applied.outcomes[0].resource_id,
            None
        )
        .unwrap()
        .unwrap()
        .document
        .text(),
        "human"
    );
    assert_eq!(
        db.library(RequestContext::local_ui()).unwrap().resources[&saved.resource_id].title,
        "Human title"
    );
    assert_eq!(
        db.read_document(
            RequestContext::local_ui(),
            saved.resource_id,
            Some(human.id)
        )
        .unwrap()
        .unwrap(),
        human
    );
    assert_eq!(
        db.read_document(
            RequestContext::local_ui(),
            saved.resource_id,
            Some(reimported.id)
        )
        .unwrap()
        .unwrap(),
        reimported
    );
}

#[test]
fn extraction_preview_survives_restart_without_changing_originals_or_receipts() {
    let (temp, mut db, mut request) = fixture();
    let bytes = b"word,meaning\r\nhello,world\r\n";
    let info = db
        .register_asset(
            RequestContext::local_ui(),
            &RegisterAsset {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
            },
            &mut Cursor::new(bytes),
            MAX_SOURCE_BYTES,
        )
        .unwrap();
    request.source.filename = "data.csv".into();
    request.source.digest = info.digest.clone();
    let source = db
        .save_source(RequestContext::local_ui(), &request)
        .unwrap();
    let before = db
        .sources(RequestContext::local_ui(), source.resource_id)
        .unwrap();
    let receipt = db
        .operation_receipt(RequestContext::local_ui(), &request.operation_id)
        .unwrap();
    let preview = db
        .source_preview(RequestContext::local_ui(), source.id)
        .unwrap();
    let extraction = preview.extraction.as_ref().unwrap();
    assert_eq!(extraction.status, ExtractionStatus::Complete);
    assert_eq!(extraction.format, ExtractionFormat::Csv);
    assert_eq!(extraction.records[1].fields, ["hello", "world"]);
    assert_eq!(extraction.records[1].locator.value, "2");
    assert_eq!(
        preview.text.as_deref(),
        Some("word,meaning\r\nhello,world\r\n")
    );
    drop(db);
    let db = Database::open(&temp.path().join("workspace")).unwrap();
    assert_eq!(
        db.source_preview(RequestContext::local_ui(), source.id)
            .unwrap(),
        preview
    );
    assert_eq!(
        db.sources(RequestContext::local_ui(), source.resource_id)
            .unwrap(),
        before
    );
    assert_eq!(
        db.operation_receipt(RequestContext::local_ui(), &request.operation_id)
            .unwrap(),
        receipt
    );
    assert_eq!(
        std::fs::read(assets::asset_path(
            &temp.path().join("workspace"),
            &info.digest
        ))
        .unwrap(),
        bytes
    );
    assert_eq!(
        db.source_preview(RequestContext::unverified_external(), source.id)
            .unwrap_err()
            .code,
        ErrorCode::PermissionDenied
    );
}

#[test]
fn distinct_provenance_and_exact_receipts_survive_restart() {
    let (temp, mut db, request) = fixture();
    let first = db
        .save_source(RequestContext::local_ui(), &request)
        .unwrap();
    let mut second = request.clone();
    second.source.id = Uuid::new_v4();
    second.source.filename = "Other name.html".into();
    second.source.license = Some("All rights reserved".into());
    second.operation_id = OperationId::generate();
    second.expected_revision = 1;
    let next = db.save_source(RequestContext::local_ui(), &second).unwrap();
    assert_eq!((first.revision, next.revision), (1, 2));
    assert_eq!(first.digest, next.digest);
    assert_ne!(first.id, next.id);
    drop(db);
    let mut db = Database::open(&temp.path().join("workspace")).unwrap();
    assert_eq!(
        db.save_source(RequestContext::local_ui(), &request)
            .unwrap(),
        first
    );
    let list = db
        .sources(RequestContext::local_ui(), request.source.resource_id)
        .unwrap();
    assert_eq!(
        list,
        SourceList {
            revision: 2,
            sources: vec![first, next]
        }
    );
    let mut changed = request.clone();
    changed.source.attribution = Some("Different".into());
    assert_eq!(
        db.save_source(RequestContext::local_ui(), &changed)
            .unwrap_err()
            .code,
        ErrorCode::IdempotencyConflict
    );
    changed.operation_id = OperationId::generate();
    assert_eq!(
        db.save_source(RequestContext::local_ui(), &changed)
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    changed.expected_revision = 2;
    assert_eq!(
        db.save_source(RequestContext::local_ui(), &changed)
            .unwrap_err()
            .code,
        ErrorCode::InvalidRequest
    );
    assert_eq!(
        db.sources(RequestContext::local_ui(), request.source.resource_id)
            .unwrap(),
        list
    );
}

#[test]
fn unauthorized_missing_or_invalid_sources_never_write_receipts() {
    let (_temp, mut db, request) = fixture();
    assert_eq!(
        db.save_source(RequestContext::unverified_external(), &request)
            .unwrap_err()
            .code,
        ErrorCode::PermissionDenied
    );
    assert_eq!(
        db.sources(
            RequestContext::unverified_external(),
            request.source.resource_id
        )
        .unwrap_err()
        .code,
        ErrorCode::PermissionDenied
    );
    assert_eq!(
        db.source_preview(RequestContext::unverified_external(), request.source.id)
            .unwrap_err()
            .code,
        ErrorCode::PermissionDenied
    );
    let mut variants = vec![];
    let mut changed = request.clone();
    changed.workspace_id = WorkspaceId::generate();
    variants.push((changed, ErrorCode::PermissionDenied));
    let mut changed = request.clone();
    changed.source.resource_id = ResourceId::generate();
    variants.push((changed, ErrorCode::InvalidRequest));
    let mut changed = request.clone();
    changed.source.digest = AssetDigest::try_from("a".repeat(64)).unwrap();
    variants.push((changed, ErrorCode::InvalidRequest));
    let mut changed = request.clone();
    changed.source.id = Uuid::nil();
    variants.push((changed, ErrorCode::InvalidRequest));
    let mut changed = request.clone();
    changed.expected_revision = 9_007_199_254_740_991;
    variants.push((changed, ErrorCode::InvalidRequest));
    let mut changed = request.clone();
    changed.contract_version = 2;
    variants.push((changed, ErrorCode::InvalidRequest));
    for (changed, code) in variants {
        assert_eq!(
            db.save_source(RequestContext::local_ui(), &changed)
                .unwrap_err()
                .code,
            code
        );
        assert!(
            db.operation_receipt(RequestContext::local_ui(), &changed.operation_id)
                .unwrap()
                .is_none()
        );
    }
    assert_eq!(
        db.sources(RequestContext::local_ui(), request.source.resource_id)
            .unwrap(),
        SourceList {
            revision: 0,
            sources: vec![]
        }
    );
}

#[test]
fn malformed_metadata_is_rejected_without_losing_original_fields() {
    let (_temp, mut db, request) = fixture();
    let mut variants = vec![];
    for filename in [
        "".into(),
        "a/b".into(),
        "a\\b".into(),
        "a\0b".into(),
        "a\nb".into(),
        "x".repeat(257),
    ] {
        let mut s = request.source.clone();
        s.filename = filename;
        variants.push(s);
    }
    for media_type in [
        "".into(),
        "text/π".into(),
        "x".repeat(129),
        "text/plain\n".into(),
    ] {
        let mut s = request.source.clone();
        s.media_type = media_type;
        variants.push(s);
    }
    let mut s = request.source.clone();
    s.attribution = Some("x".repeat(4097));
    variants.push(s);
    let mut s = request.source.clone();
    s.license = Some("x".repeat(4097));
    variants.push(s);
    for url in [
        "file:///tmp/local".into(),
        "javascript:alert(1)".into(),
        "https://".into(),
        "https://example.test/".to_owned() + &"x".repeat(2048),
    ] {
        let mut s = request.source.clone();
        s.origin_url = Some(url);
        variants.push(s);
    }
    for source in variants {
        let mut changed = request.clone();
        changed.source = source;
        assert_eq!(
            db.save_source(RequestContext::local_ui(), &changed)
                .unwrap_err()
                .code,
            ErrorCode::InvalidRequest
        );
    }
    let source = db
        .save_source(RequestContext::local_ui(), &request)
        .unwrap();
    assert_eq!(source.filename, request.source.filename);
    assert_eq!(source.attribution, request.source.attribution);
    assert_eq!(source.license, request.source.license);
    assert_eq!(source.origin_url, request.source.origin_url);
}

#[test]
fn previews_are_passive_and_missing_or_corrupt_bytes_preserve_records() {
    let (temp, mut db, request) = fixture();
    let source = db
        .save_source(RequestContext::local_ui(), &request)
        .unwrap();
    let preview = db
        .source_preview(RequestContext::local_ui(), source.id)
        .unwrap();
    assert_eq!(
        preview.text.as_deref(),
        Some("<script>passive text only</script>")
    );
    assert_eq!(preview.hex, None);
    assert!(!preview.truncated);
    let asset = db
        .register_asset(
            RequestContext::local_ui(),
            &RegisterAsset {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
            },
            &mut Cursor::new(vec![0xff, 0x00, 0x80]),
            MAX_SOURCE_BYTES,
        )
        .unwrap();
    let mut binary = request.clone();
    binary.source.id = Uuid::new_v4();
    binary.source.digest = asset.digest;
    binary.source.filename = "binary.bin".into();
    binary.operation_id = OperationId::generate();
    binary.expected_revision = 1;
    let binary = db.save_source(RequestContext::local_ui(), &binary).unwrap();
    let preview = db
        .source_preview(RequestContext::local_ui(), binary.id)
        .unwrap();
    assert_eq!(preview.text, None);
    assert_eq!(preview.hex.as_deref(), Some("ff0080"));
    let list = db
        .sources(RequestContext::local_ui(), source.resource_id)
        .unwrap();
    let path = assets::asset_path(&temp.path().join("workspace"), &source.digest);
    std::fs::write(&path, b"corrupt").unwrap();
    let mut unavailable = request.clone();
    unavailable.source.id = Uuid::new_v4();
    unavailable.operation_id = OperationId::generate();
    unavailable.expected_revision = 2;
    assert_eq!(
        db.save_source(RequestContext::local_ui(), &unavailable)
            .unwrap_err()
            .code,
        ErrorCode::RecoveryRequired
    );
    assert!(
        db.operation_receipt(RequestContext::local_ui(), &unavailable.operation_id)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        db.source_preview(RequestContext::local_ui(), source.id)
            .unwrap_err()
            .code,
        ErrorCode::RecoveryRequired
    );
    std::fs::remove_file(path).unwrap();
    assert_eq!(
        db.source_preview(RequestContext::local_ui(), source.id)
            .unwrap_err()
            .code,
        ErrorCode::RecoveryRequired
    );
    assert_eq!(
        db.sources(RequestContext::local_ui(), source.resource_id)
            .unwrap(),
        list
    );
    drop(db);
    let db = Database::open(&temp.path().join("workspace")).unwrap();
    assert_eq!(
        db.sources(RequestContext::local_ui(), source.resource_id)
            .unwrap(),
        list
    );
    assert_eq!(
        db.source_preview(RequestContext::local_ui(), Uuid::new_v4())
            .unwrap_err()
            .code,
        ErrorCode::InvalidRequest
    );
}

#[test]
fn malformed_persisted_source_metadata_is_rejected() {
    let (_temp, mut db, request) = fixture();
    let source = db
        .save_source(RequestContext::local_ui(), &request)
        .unwrap();
    db.connection
        .execute(
            "UPDATE original_sources SET filename='../unsafe' WHERE id=?1",
            [source.id.to_string()],
        )
        .unwrap();
    assert_eq!(
        db.sources(RequestContext::local_ui(), source.resource_id)
            .unwrap_err()
            .code,
        ErrorCode::RecoveryRequired
    );
    assert_eq!(
        db.source_preview(RequestContext::local_ui(), source.id)
            .unwrap_err()
            .code,
        ErrorCode::RecoveryRequired
    );
}

#[test]
fn bounded_previews_handle_utf8_boundaries_and_binary_truncation() {
    let (temp, mut db, request) = fixture();
    let mut text = vec![b'x'; 65_535];
    text.extend_from_slice("😀end".as_bytes());
    let info = db
        .register_asset(
            RequestContext::local_ui(),
            &RegisterAsset {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
            },
            &mut Cursor::new(text),
            MAX_SOURCE_BYTES,
        )
        .unwrap();
    let mut request = request;
    request.source.digest = info.digest;
    let source = db
        .save_source(RequestContext::local_ui(), &request)
        .unwrap();
    let preview = db
        .source_preview(RequestContext::local_ui(), source.id)
        .unwrap();
    assert!(preview.truncated);
    assert_eq!(preview.text.unwrap(), "x".repeat(65_535));
    assert_eq!(preview.hex, None);
    let info = db
        .register_asset(
            RequestContext::local_ui(),
            &RegisterAsset {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
            },
            &mut Cursor::new(vec![0xff; 300]),
            MAX_SOURCE_BYTES,
        )
        .unwrap();
    request.source.id = Uuid::new_v4();
    request.source.digest = info.digest;
    request.operation_id = OperationId::generate();
    request.expected_revision = 1;
    let source = db
        .save_source(RequestContext::local_ui(), &request)
        .unwrap();
    let preview = db
        .source_preview(RequestContext::local_ui(), source.id)
        .unwrap();
    assert!(preview.truncated);
    assert_eq!(preview.text, None);
    assert_eq!(preview.hex.unwrap(), "ff".repeat(256));
    let root = temp.path().join("workspace");
    let path = assets::asset_path(&root, &source.digest);
    let retained = path.with_extension("original");
    std::fs::rename(&path, &retained).unwrap();
    std::os::unix::fs::symlink(&retained, &path).unwrap();
    assert_eq!(
        db.source_preview(RequestContext::local_ui(), source.id)
            .unwrap_err()
            .code,
        ErrorCode::RecoveryRequired
    );
    assert_eq!(
        db.sources(RequestContext::local_ui(), source.resource_id)
            .unwrap()
            .sources
            .len(),
        2
    );
}

#[test]
fn oversized_registered_asset_and_malformed_wire_are_rejected() {
    let (_temp, mut db, request) = fixture();
    let mut wire = serde_json::to_value(&request).unwrap();
    wire["callerPath"] = serde_json::json!("/tmp/not-accepted");
    assert!(serde_json::from_value::<SaveSource>(wire).is_err());
    let mut wire = serde_json::to_value(&request.source).unwrap();
    wire["unexpected"] = serde_json::json!(true);
    assert!(serde_json::from_value::<NewSource>(wire).is_err());
    db.connection
        .execute(
            "UPDATE assets SET byte_length=?1 WHERE digest=?2",
            params![MAX_SOURCE_BYTES as i64 + 1, request.source.digest.as_str()],
        )
        .unwrap();
    assert_eq!(
        db.save_source(RequestContext::local_ui(), &request)
            .unwrap_err()
            .code,
        ErrorCode::InvalidRequest
    );
    assert_eq!(
        db.sources(RequestContext::local_ui(), request.source.resource_id)
            .unwrap()
            .revision,
        0
    );
}

#[test]
fn typed_service_lists_saves_and_previews_without_caller_paths() {
    tauri::async_runtime::block_on(async {
        let (temp, db, request) = fixture();
        drop(db);
        let service = ApplicationService::start(temp.path().join("workspace")).unwrap();
        let original = service
            .save_source(RequestContext::local_ui(), request.clone())
            .await
            .unwrap();
        assert_eq!(
            service
                .sources(RequestContext::local_ui(), original.resource_id)
                .await
                .unwrap()
                .sources,
            vec![original.clone()]
        );
        assert_eq!(
            service
                .source_preview(RequestContext::local_ui(), original.id)
                .await
                .unwrap()
                .source,
            original
        );
        assert_eq!(
            service
                .save_source(RequestContext::local_ui(), request)
                .await
                .unwrap(),
            original
        );
        assert_eq!(
            service
                .sources(RequestContext::unverified_external(), original.resource_id)
                .await
                .unwrap_err()
                .code,
            ErrorCode::PermissionDenied
        );
        service.shutdown().await.unwrap();
    });
}

// These tests catch positional JSON mapping, missing-property fallback, duplicate
// collapse, and loss of durable keyed identity across changed source layouts.
#[test]
fn structured_json_real_database_restart_reimport_and_ambiguous_records() {
    let (temp, mut db, mut request) = fixture();
    fn stage(db: &mut Database, request: &mut SaveSource, bytes: &[u8]) -> IngestionJob {
        let asset = db
            .register_asset(
                RequestContext::local_ui(),
                &RegisterAsset {
                    contract_version: 1,
                    workspace_id: db.id.clone(),
                    operation_id: OperationId::generate(),
                },
                &mut Cursor::new(bytes),
                MAX_SOURCE_BYTES,
            )
            .unwrap();
        request.source.id = Uuid::new_v4();
        request.source.filename = "rows.json".into();
        request.source.digest = asset.digest;
        request.operation_id = OperationId::generate();
        let source = db.save_source(RequestContext::local_ui(), request).unwrap();
        request.expected_revision += 1;
        db.create_ingestion_job(
            RequestContext::local_ui(),
            &CreateIngestionJob {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                source_id: source.id,
                source_revision: source.revision,
            },
        )
        .unwrap()
    }
    let old = stage(&mut db, &mut request, r#"[{"id":"κ","title":"Title","body":{"x":1}}, {"id":"missing","body":"no title"}, {"id":"a","id":"b","title":"bad","body":"bad"}]"#.as_bytes());
    assert_eq!(old.handler_version, 2);
    let mapping: IngestionMapping = serde_json::from_value(serde_json::json!({"keyColumn":0,"titleColumn":0,"bodyColumn":0,"jsonProperties":{"key":"id","title":"title","body":"body"}})).unwrap();
    let mut apply = ApplyIngestionJob {
        contract_version: 1,
        workspace_id: db.id.clone(),
        operation_id: OperationId::generate(),
        job_id: old.id,
        expected_revision: 0,
        expected_library_revision: db.library(RequestContext::local_ui()).unwrap().revision,
        mapping: mapping.clone(),
        selected: vec![1],
    };
    for index in [1, 2] {
        apply.selected = vec![index];
        assert_eq!(
            db.apply_ingestion_job(RequestContext::local_ui(), &apply)
                .unwrap_err()
                .code,
            ErrorCode::InvalidRequest
        );
        assert!(
            db.ingestion_job(RequestContext::local_ui(), old.id)
                .unwrap()
                .outcomes
                .is_empty()
        );
    }
    let reviewed = db
        .review_ingestion_job(
            RequestContext::local_ui(),
            &ReviewIngestionJob {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                job_id: old.id,
                expected_revision: 0,
                previous_job_id: None,
                expected_previous_revision: None,
                mapping: mapping.clone(),
                decisions: vec![
                    IngestionDecision {
                        index: Some(1),
                        key: None,
                        disposition: IngestionDisposition::Unresolved,
                    },
                    IngestionDecision {
                        index: Some(2),
                        key: None,
                        disposition: IngestionDisposition::Unresolved,
                    },
                ],
            },
        )
        .unwrap();
    apply.expected_revision = reviewed.revision;
    apply.selected = vec![0];
    let applied = db
        .apply_ingestion_job(RequestContext::local_ui(), &apply)
        .unwrap();
    let resource = applied.outcomes[0].resource_id;
    assert_eq!(applied.outcomes[0].key, "κ");
    assert_eq!(
        db.read_document(RequestContext::local_ui(), resource, None)
            .unwrap()
            .unwrap()
            .document
            .text(),
        r#"{"x":1}"#
    );
    let new = stage(
        &mut db,
        &mut request,
        r#"[{"extra":"new schema","body":[ 1,2 ],"title":"Title 2","id":"κ"}]"#.as_bytes(),
    );
    drop(db);
    let mut db = Database::open(&temp.path().join("workspace")).unwrap();
    assert_eq!(
        db.apply_ingestion_job(RequestContext::local_ui(), &apply)
            .unwrap(),
        applied
    );
    let loaded = db
        .ingestion_job(RequestContext::local_ui(), new.id)
        .unwrap();
    assert_eq!(loaded, new);
    let preview = db
        .preview_ingestion_reimport(RequestContext::local_ui(), new.id, old.id, &mapping)
        .unwrap();
    assert_eq!(preview.items[0].status, ReimportStatus::Changed);
    let reimport = ApplyReimportJob {
        contract_version: 1,
        workspace_id: db.id.clone(),
        operation_id: OperationId::generate(),
        job_id: new.id,
        previous_job_id: old.id,
        expected_revision: 0,
        expected_previous_revision: applied.revision,
        expected_library_revision: db.library(RequestContext::local_ui()).unwrap().revision,
        mapping,
        selected: vec![0],
        resolutions: vec![],
    };
    let updated = db
        .apply_ingestion_reimport(RequestContext::local_ui(), &reimport)
        .unwrap();
    assert_eq!(updated.outcomes[0].resource_id, resource);
    assert_eq!(
        db.read_document(RequestContext::local_ui(), resource, None)
            .unwrap()
            .unwrap()
            .document
            .text(),
        "[ 1,2 ]"
    );
    assert_eq!(
        db.apply_ingestion_reimport(RequestContext::local_ui(), &reimport)
            .unwrap(),
        updated
    );
}

#[test]
fn legacy_json_persisted_job_and_receipts_keep_raw_mapping_and_fingerprints() {
    let (temp, mut db, mut source_request) = fixture();
    let raw = r#"{"id":"first","id":"second"}"#;
    let asset = db
        .register_asset(
            RequestContext::local_ui(),
            &RegisterAsset {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
            },
            &mut Cursor::new(raw.as_bytes()),
            MAX_SOURCE_BYTES,
        )
        .unwrap();
    source_request.source.filename = "legacy.json".into();
    source_request.source.digest = asset.digest;
    let source = db
        .save_source(RequestContext::local_ui(), &source_request)
        .unwrap();
    let create = CreateIngestionJob {
        contract_version: 1,
        workspace_id: db.id.clone(),
        operation_id: OperationId::generate(),
        source_id: source.id,
        source_revision: source.revision,
    };
    let job = db
        .create_ingestion_job(RequestContext::local_ui(), &create)
        .unwrap();
    // Literal historical DTO without any version-2 fields; persist the actual
    // job and receipt, then reopen through ordinary database verification.
    let legacy_wire = serde_json::json!({
        "id":job.id, "handlerVersion":1, "revision":0, "status":"staged", "previousJobId":null,
        "source":source, "mapping":null, "outcomes":[], "decisions":[],
        "extraction":{"contractVersion":1,"format":"json","status":"complete","diagnostic":null,
        "records":[{"locator":{"kind":"jsonPointer","value":""},"fields":[raw]}]},
    });
    let legacy: IngestionJob = serde_json::from_value(legacy_wire.clone()).unwrap();
    db.connection
        .execute(
            "UPDATE ingestion_jobs SET data=?1 WHERE id=?2",
            params![legacy_wire.to_string(), job.id.to_string()],
        )
        .unwrap();
    let receipt = operations::applied(
        create.operation_id.clone(),
        OperationOutcome::IngestionJob(Box::new(legacy.clone())),
    );
    db.connection
        .execute(
            "UPDATE operations SET result=?1 WHERE id=?2",
            params![
                serde_json::to_string(&receipt).unwrap(),
                create.operation_id.as_str()
            ],
        )
        .unwrap();
    drop(db);
    let mut db = Database::open(&temp.path().join("workspace")).unwrap();
    assert_eq!(
        db.ingestion_job(RequestContext::local_ui(), job.id)
            .unwrap(),
        legacy
    );
    assert_eq!(
        db.create_ingestion_job(RequestContext::local_ui(), &create)
            .unwrap(),
        legacy
    );
    let mapping: IngestionMapping =
        serde_json::from_str(r#"{"keyColumn":0,"titleColumn":0,"bodyColumn":0}"#).unwrap();
    let apply = ApplyIngestionJob {
        contract_version: 1,
        workspace_id: db.id.clone(),
        operation_id: OperationId::generate(),
        job_id: job.id,
        expected_revision: 0,
        expected_library_revision: db.library(RequestContext::local_ui()).unwrap().revision,
        mapping,
        selected: vec![0],
    };
    let legacy_request = format!(
        r#"[1,"ingestion-apply",{{"contractVersion":1,"workspaceId":"{}","operationId":"{}","jobId":"{}","expectedRevision":0,"expectedLibraryRevision":{},"mapping":{{"keyColumn":0,"titleColumn":0,"bodyColumn":0}},"selected":[0]}}]"#,
        db.id.as_str(),
        apply.operation_id.as_str(),
        job.id,
        apply.expected_library_revision
    );
    assert_eq!(
        operations::request_hash("ingestion-apply", &apply).unwrap(),
        migrations::checksum(legacy_request.as_bytes())
    );
    let applied = db
        .apply_ingestion_job(RequestContext::local_ui(), &apply)
        .unwrap();
    assert_eq!(applied.outcomes[0].key, raw);
    let id = applied.outcomes[0].resource_id;
    assert_eq!(
        db.read_document(RequestContext::local_ui(), id, None)
            .unwrap()
            .unwrap()
            .document
            .text(),
        raw
    );
    drop(db);
    let mut db = Database::open(&temp.path().join("workspace")).unwrap();
    assert_eq!(
        db.apply_ingestion_job(RequestContext::local_ui(), &apply)
            .unwrap(),
        applied
    );
    assert_eq!(
        db.ingestion_job(RequestContext::local_ui(), job.id)
            .unwrap(),
        applied
    );
}

#[test]
fn json_property_null_is_explicit_raw_and_missing_selector_is_rejected() {
    assert!(serde_json::from_value::<IngestionMapping>(serde_json::json!({"keyColumn":0,"titleColumn":0,"bodyColumn":0,"jsonProperties":{"key":null,"title":null}})).is_err());
    let raw: IngestionMapping = serde_json::from_value(serde_json::json!({"keyColumn":0,"titleColumn":0,"bodyColumn":0,"jsonProperties":{"key":null,"title":null,"body":null}})).unwrap();
    let missing: IngestionMapping = serde_json::from_value(serde_json::json!({"keyColumn":0,"titleColumn":0,"bodyColumn":0,"jsonProperties":{"key":"missing","title":null,"body":null}})).unwrap();
    let extracted = extraction::extract_for_ingestion("rows.json", br#"{"id":"one"}"#);
    // Different explicit selectors must produce different idempotency hashes.
    assert_ne!(
        operations::request_hash("mapping", &raw).unwrap(),
        operations::request_hash("mapping", &missing).unwrap()
    );
    assert_eq!(extracted.records[0].fields[0], r#"{"id":"one"}"#);
}

#[test]
fn structured_json_explicit_raw_selection_and_missing_property_have_distinct_durable_receipts() {
    let (temp, mut db, mut request) = fixture();
    let raw = r#"{"id":"κ"}"#;
    let asset = db
        .register_asset(
            RequestContext::local_ui(),
            &RegisterAsset {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
            },
            &mut Cursor::new(raw.as_bytes()),
            MAX_SOURCE_BYTES,
        )
        .unwrap();
    request.source.filename = "raw.json".into();
    request.source.digest = asset.digest;
    let source = db
        .save_source(RequestContext::local_ui(), &request)
        .unwrap();
    let job = db
        .create_ingestion_job(
            RequestContext::local_ui(),
            &CreateIngestionJob {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                source_id: source.id,
                source_revision: source.revision,
            },
        )
        .unwrap();
    let mapping: IngestionMapping = serde_json::from_value(serde_json::json!({"keyColumn":0,"titleColumn":0,"bodyColumn":0,"jsonProperties":{"key":null,"title":null,"body":null}})).unwrap();
    let apply = ApplyIngestionJob {
        contract_version: 1,
        workspace_id: db.id.clone(),
        operation_id: OperationId::generate(),
        job_id: job.id,
        expected_revision: 0,
        expected_library_revision: db.library(RequestContext::local_ui()).unwrap().revision,
        mapping,
        selected: vec![0],
    };
    let mut missing = apply.clone();
    missing.mapping.json_properties.as_mut().unwrap().body = Some("absent".into());
    assert_eq!(
        db.apply_ingestion_job(RequestContext::local_ui(), &missing)
            .unwrap_err()
            .code,
        ErrorCode::InvalidRequest
    );
    let applied = db
        .apply_ingestion_job(RequestContext::local_ui(), &apply)
        .unwrap();
    assert_eq!(applied.outcomes[0].key, raw);
    assert_eq!(
        db.read_document(
            RequestContext::local_ui(),
            applied.outcomes[0].resource_id,
            None
        )
        .unwrap()
        .unwrap()
        .document
        .text(),
        raw
    );
    drop(db);
    let mut db = Database::open(&temp.path().join("workspace")).unwrap();
    assert_eq!(
        db.apply_ingestion_job(RequestContext::local_ui(), &apply)
            .unwrap(),
        applied
    );
    assert_eq!(
        db.apply_ingestion_job(RequestContext::local_ui(), &missing)
            .unwrap_err()
            .code,
        ErrorCode::IdempotencyConflict
    );
    // Historical v1 jobs must also remain usable as raw lineage baselines for
    // new handler-2 jobs, without treating their bytes as named properties.
    let mut legacy = applied.clone();
    legacy.handler_version = 1;
    legacy.mapping = Some(IngestionMapping {
        key_column: 0,
        title_column: 0,
        body_column: 0,
        json_properties: None,
    });
    legacy.extraction = serde_json::from_value(serde_json::json!({"contractVersion":1,"format":"json","status":"complete","diagnostic":null,"records":[{"locator":{"kind":"jsonPointer","value":""},"fields":[raw]}]})).unwrap();
    db.connection
        .execute(
            "UPDATE ingestion_jobs SET data=?1 WHERE id=?2",
            params![
                serde_json::to_string(&legacy).unwrap(),
                legacy.id.to_string()
            ],
        )
        .unwrap();
    let next = db
        .create_ingestion_job(
            RequestContext::local_ui(),
            &CreateIngestionJob {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                source_id: source.id,
                source_revision: source.revision,
            },
        )
        .unwrap();
    assert_eq!(next.handler_version, 2);
    let legacy_mapping = legacy.mapping.clone().unwrap();
    let preview = db
        .preview_ingestion_reimport(
            RequestContext::local_ui(),
            next.id,
            legacy.id,
            &legacy_mapping,
        )
        .unwrap();
    assert_eq!(preview.items[0].status, ReimportStatus::Unchanged);
    let library = db.library(RequestContext::local_ui()).unwrap();
    let updated = db
        .apply_ingestion_reimport(
            RequestContext::local_ui(),
            &ApplyReimportJob {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                job_id: next.id,
                previous_job_id: legacy.id,
                expected_revision: 0,
                expected_previous_revision: legacy.revision,
                expected_library_revision: library.revision,
                mapping: legacy_mapping,
                selected: vec![0],
                resolutions: vec![],
            },
        )
        .unwrap();
    assert_eq!(
        updated.outcomes[0].resource_id,
        applied.outcomes[0].resource_id
    );
    drop(db);
    let db = Database::open(&temp.path().join("workspace")).unwrap();
    assert_eq!(
        db.ingestion_job(RequestContext::local_ui(), updated.id)
            .unwrap(),
        updated
    );
}

#[test]
fn undo_guard_preserves_saved_draft_pins_through_real_service_reopen() {
    use crate::workspace::lessons::{
        ApplyLessonDraft, Binding, CreateLessonDraft, EditLessonDraft, LessonDraftScope,
        LessonOperation,
    };
    let (temp, mut db, mut request) = fixture();
    request.source.filename = "rows.txt".into();
    let source = db
        .save_source(RequestContext::local_ui(), &request)
        .unwrap();
    let job = db
        .create_ingestion_job(
            RequestContext::local_ui(),
            &CreateIngestionJob {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                source_id: source.id,
                source_revision: source.revision,
            },
        )
        .unwrap();
    let applied = db
        .apply_ingestion_job(
            RequestContext::local_ui(),
            &ApplyIngestionJob {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                job_id: job.id,
                expected_revision: job.revision,
                expected_library_revision: db.library(RequestContext::local_ui()).unwrap().revision,
                mapping: IngestionMapping {
                    key_column: 0,
                    title_column: 0,
                    body_column: 0,
                    json_properties: None,
                },
                selected: vec![0],
            },
        )
        .unwrap();
    let imported = &applied.outcomes[0];
    let unit = Uuid::new_v4().to_string();
    db.connection
        .execute("INSERT INTO units VALUES(?1,'Lesson','goal',0)", [&unit])
        .unwrap();
    let draft = db
        .lesson_create(
            RequestContext::local_ui(),
            &CreateLessonDraft {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                unit_id: unit.clone(),
                expected_head_revision: None,
            },
        )
        .unwrap();
    let draft = db
        .lesson_edit(
            RequestContext::local_ui(),
            &EditLessonDraft {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                unit_id: unit.clone(),
                draft_id: draft.id.clone(),
                expected_draft_revision: draft.revision,
                operations: vec![LessonOperation::BindReference {
                    binding: Binding {
                        binding_id: "import".into(),
                        target_kind: "resource".into(),
                        target_id: imported.resource_id.to_string(),
                        revision_id: imported.document_revision_id.to_string(),
                        locator: None,
                    },
                }],
            },
        )
        .unwrap();
    let snapshot = db.library(RequestContext::local_ui()).unwrap();
    let head = db
        .read_document(RequestContext::local_ui(), imported.resource_id, None)
        .unwrap()
        .unwrap();
    let undo = UndoIngestionJob {
        contract_version: 1,
        workspace_id: db.id.clone(),
        operation_id: OperationId::generate(),
        job_id: applied.id,
        expected_revision: applied.revision,
        expected_library_revision: snapshot.revision,
    };
    drop(db);
    tauri::async_runtime::block_on(async {
        let service = ApplicationService::start(temp.path().join("workspace")).unwrap();
        let preview = service
            .preview_ingestion_undo(RequestContext::local_ui(), applied.id)
            .await
            .unwrap();
        assert!(
            !preview.can_undo,
            "a durable draft pin must prevent archival"
        );
        let diagnostics = serde_json::to_value(&preview).unwrap();
        assert!(
            diagnostics["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v.as_str().unwrap().contains("draft"))
        );
        assert_eq!(
            service
                .undo_ingestion_job(RequestContext::local_ui(), undo)
                .await
                .unwrap_err()
                .code,
            ErrorCode::RevisionConflict
        );
        assert_eq!(
            service.library(RequestContext::local_ui()).await.unwrap(),
            snapshot
        );
        assert_eq!(
            service
                .read_document(RequestContext::local_ui(), imported.resource_id, None)
                .await
                .unwrap()
                .unwrap(),
            head
        );
        assert_eq!(
            service
                .lesson_list_drafts(RequestContext::local_ui(), unit.clone())
                .await
                .unwrap(),
            vec![draft.clone()]
        );
        let preview = service
            .lesson_preview(
                RequestContext::local_ui(),
                LessonDraftScope {
                    contract_version: 1,
                    workspace_id: WorkspaceId::try_from(snapshot.workspace_id.to_string()).unwrap(),
                    unit_id: unit.clone(),
                    draft_id: draft.id.clone(),
                    expected_draft_revision: draft.revision,
                },
            )
            .await
            .unwrap();
        service
            .lesson_apply(
                RequestContext::local_ui(),
                ApplyLessonDraft {
                    contract_version: 1,
                    workspace_id: WorkspaceId::try_from(snapshot.workspace_id.to_string()).unwrap(),
                    operation_id: OperationId::generate(),
                    unit_id: unit.clone(),
                    draft_id: draft.id,
                    expected_draft_revision: draft.revision,
                    expected_head_revision: draft.base_head_revision,
                    preview_hash: preview.hash,
                },
            )
            .await
            .unwrap();
        assert!(
            !service
                .preview_ingestion_undo(RequestContext::local_ui(), applied.id)
                .await
                .unwrap()
                .can_undo,
            "published pins must also prevent archival"
        );
        assert_eq!(
            service
                .lesson_resource(
                    RequestContext::local_ui(),
                    unit,
                    imported.document_revision_id.to_string()
                )
                .await
                .unwrap()
                .unwrap()
                .document,
            head.document
        );
        service.shutdown().await.unwrap();
    });
}
