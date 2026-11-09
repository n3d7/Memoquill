use super::*;
use crate::organization::Command;
use std::io::Cursor;

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
