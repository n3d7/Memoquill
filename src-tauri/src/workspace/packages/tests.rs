use super::*;
use crate::workspace::{CourseManager, RequestContext, database::Database};
use std::io::Cursor;
use uuid::Uuid;

fn capture() -> (tempfile::TempDir, Database, NativeCapture) {
    let temp = tempfile::tempdir().unwrap();
    let mut db = Database::open(&temp.path().join("source")).unwrap();
    let capture = db
        .capture_native(RequestContext::local_ui(), Uuid::new_v4())
        .unwrap();
    (temp, db, capture)
}

fn roundtrip(capture: &NativeCapture) -> NativeCapture {
    let mut bytes = Cursor::new(Vec::new());
    capture.write_to(&mut bytes).unwrap();
    NativeCapture::read_from(Cursor::new(bytes.into_inner())).unwrap()
}

#[test]
fn captured_assets_survive_unlink_and_missing_originals_require_explicit_confirmation() {
    let temp = tempfile::tempdir().unwrap();
    let mut db = Database::open(&temp.path().join("source")).unwrap();
    let info = db
        .register_asset(
            RequestContext::local_ui(),
            &RegisterAsset {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
            },
            &mut Cursor::new(b"original"),
            64,
        )
        .unwrap();
    let captured = db
        .capture_native(RequestContext::local_ui(), Uuid::new_v4())
        .unwrap();
    std::fs::remove_file(assets::asset_path(db.owner.root(), &info.digest)).unwrap();
    assert!(roundtrip(&captured).manifest.complete);
    let missing = db
        .capture_native(RequestContext::local_ui(), Uuid::new_v4())
        .unwrap();
    assert!(!missing.manifest.complete);
    assert_eq!(missing.manifest.missing_assets, vec![info.digest.clone()]);
    let verified = roundtrip(&missing);
    let manager = CourseManager::open(temp.path().join("target")).unwrap();
    let before = manager.list().unwrap();
    let operation = Uuid::new_v4();
    assert_eq!(
        manager
            .restore_native(operation, "Incomplete".into(), &verified, false)
            .unwrap_err()
            .code,
        ErrorCode::RecoveryRequired
    );
    assert_eq!(before, manager.list().unwrap());
    let restored = manager
        .restore_native(operation, "Incomplete".into(), &verified, true)
        .unwrap();
    assert!(!restored.complete);
    let restored_db = Database::open(
        &temp
            .path()
            .join("target/courses/data")
            .join(operation.to_string()),
    )
    .unwrap();
    assert_eq!(restored_db.recovery.missing_assets, 1);
}

#[test]
fn root_publication_interruption_and_committed_lost_reply_retry_exactly() {
    for phase in [faults::Phase::Publish, faults::Phase::AfterCommit] {
        let (temp, _db, package) = capture();
        let target = temp.path().join("target");
        let manager = CourseManager::open(target.clone()).unwrap();
        let before = manager.list().unwrap();
        let operation = Uuid::new_v4();
        let result = faults::using(
            move |p| {
                if p == phase {
                    Err(AppError::new(ErrorCode::OutcomeUnconfirmed))
                } else {
                    Ok(())
                }
            },
            || manager.restore_native(operation, "Restore".into(), &package, false),
        );
        assert_eq!(result.unwrap_err().code, ErrorCode::OutcomeUnconfirmed);
        if phase == faults::Phase::Publish {
            assert_eq!(manager.list().unwrap(), before);
        }
        drop(manager);
        let reopened = CourseManager::open(target).unwrap();
        assert_eq!(
            reopened
                .restore_native(operation, "changed".into(), &package, false)
                .unwrap_err()
                .code,
            ErrorCode::IdempotencyConflict
        );
        let report = reopened
            .restore_native(operation, "Restore".into(), &package, false)
            .unwrap();
        assert_eq!(report.course_id, operation);
        assert_eq!(
            reopened.list().unwrap().courses.len(),
            before.courses.len() + 1
        );
    }
}

#[test]
fn pending_operation_never_adopts_matching_but_unowned_root() {
    let (temp, _db, package) = capture();
    let manager = CourseManager::open(temp.path().join("target")).unwrap();
    let operation = Uuid::new_v4();
    let _ = faults::using(
        |phase| {
            if phase == faults::Phase::Publish {
                Err(AppError::new(ErrorCode::OutcomeUnconfirmed))
            } else {
                Ok(())
            }
        },
        || manager.restore_native(operation, "Restore".into(), &package, false),
    );
    let root = temp
        .path()
        .join("target/courses/data")
        .join(operation.to_string());
    std::fs::remove_dir_all(&root).unwrap();
    owner::create_private_tree(&root).unwrap();
    package.materialize(&root).unwrap();
    assert_eq!(
        manager
            .restore_native(operation, "Restore".into(), &package, false)
            .unwrap_err()
            .code,
        ErrorCode::RecoveryRequired
    );
}

#[test]
fn unknown_roots_and_course_limit_preserve_existing_data() {
    let (temp, _db, package) = capture();
    let manager = CourseManager::open(temp.path().join("target")).unwrap();
    let before = manager.list().unwrap();
    let operation = Uuid::new_v4();
    let root = temp
        .path()
        .join("target/courses/data")
        .join(operation.to_string());
    owner::create_private_tree(&root).unwrap();
    std::fs::write(root.join("unknown"), b"untouched").unwrap();
    assert_eq!(
        manager
            .restore_native(operation, "Restore".into(), &package, false)
            .unwrap_err()
            .code,
        ErrorCode::RecoveryRequired
    );
    assert_eq!(std::fs::read(root.join("unknown")).unwrap(), b"untouched");
    assert_eq!(before, manager.list().unwrap());
    for _ in 1..32 {
        manager.create(Uuid::new_v4(), "Course".into()).unwrap();
    }
    assert_eq!(
        manager
            .restore_native(Uuid::new_v4(), "Restore".into(), &package, false)
            .unwrap_err()
            .code,
        ErrorCode::BudgetExceeded
    );
}

#[test]
fn recomputed_hashes_cannot_admit_extra_sqlite_ddl_or_broken_domain_references() {
    for sql in [
        "CREATE VIEW hostile AS SELECT 1",
        "CREATE TRIGGER hostile AFTER INSERT ON preferences BEGIN SELECT 1; END",
        "CREATE INDEX hostile ON preferences(revision)",
        "PRAGMA foreign_keys=OFF; DELETE FROM workspace_meta",
        "PRAGMA user_version=999",
        "INSERT INTO resources VALUES('00000000-0000-4000-8000-000000000001','Invalid',0); INSERT INTO resource_revisions VALUES('00000000-0000-4000-8000-000000000002','00000000-0000-4000-8000-000000000001',1,'{}','0000000000000000000000000000000000000000000000000000000000000000')",
    ] {
        let (_temp, _db, mut package) = capture();
        let path = package.staging.path().join("database.sqlite3");
        let connection = Connection::open(&path).unwrap();
        connection.execute_batch(sql).unwrap();
        package.manifest.table_counts = validation::table_counts(&connection).unwrap();
        drop(connection);
        let (hash, size) = hash_file(&path, MAX_DATABASE).unwrap();
        package.manifest.payloads[0].sha256 = hash;
        package.manifest.payloads[0].byte_length = size;
        let mut bytes = Cursor::new(Vec::new());
        package.write_to(&mut bytes).unwrap();
        assert!(
            NativeCapture::read_from(Cursor::new(bytes.into_inner())).is_err(),
            "accepted {sql}"
        );
    }
}

#[test]
fn archive_hostility_is_rejected_before_admission() {
    use zip::{ZipWriter, write::SimpleFileOptions};
    let (_temp, _db, package) = capture();
    let mut good = Cursor::new(Vec::new());
    package.write_to(&mut good).unwrap();
    let good = good.into_inner();
    for mutate in [0, 1, 2, 3, 4] {
        let mut bytes = good.clone();
        let end = bytes.len() - 22;
        match mutate {
            0 => bytes[end + 10..end + 12].copy_from_slice(&65535u16.to_le_bytes()),
            1 => {
                bytes[end + 12..end + 16].copy_from_slice(&(MAX_MANIFEST as u32 + 1).to_le_bytes())
            }
            2 => bytes[end + 4] = 1,
            3 => {
                bytes.insert(0, b'!');
            }
            _ => bytes.truncate(bytes.len() - 5),
        }
        assert!(NativeCapture::read_from(Cursor::new(bytes)).is_err());
    }
    for name in [
        "../escape",
        "/absolute",
        "assets/../escape",
        "database.sqlite3/",
        "C:\\escape",
    ] {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .start_file(name, SimpleFileOptions::default())
            .unwrap();
        writer.write_all(b"unsafe").unwrap();
        assert!(NativeCapture::read_from(writer.finish().unwrap()).is_err());
    }
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    writer
        .add_symlink(
            "database.sqlite3",
            "/etc/passwd",
            SimpleFileOptions::default(),
        )
        .unwrap();
    assert!(NativeCapture::read_from(writer.finish().unwrap()).is_err());
    // Two central directory records pointing at one local header (overlap/duplicate).
    let mut bytes = good.clone();
    let end = bytes.len() - 22;
    let offset = u32::from_le_bytes(bytes[end + 16..end + 20].try_into().unwrap()) as usize;
    let n = 46
        + u16::from_le_bytes(bytes[offset + 28..offset + 30].try_into().unwrap()) as usize
        + u16::from_le_bytes(bytes[offset + 30..offset + 32].try_into().unwrap()) as usize;
    let duplicate = bytes[offset..offset + n].to_vec();
    bytes.splice(end..end, duplicate);
    let new_end = bytes.len() - 22;
    let count = u16::from_le_bytes(bytes[new_end + 10..new_end + 12].try_into().unwrap()) + 1;
    bytes[new_end + 8..new_end + 10].copy_from_slice(&count.to_le_bytes());
    bytes[new_end + 10..new_end + 12].copy_from_slice(&count.to_le_bytes());
    let size = (new_end - offset) as u32;
    bytes[new_end + 12..new_end + 16].copy_from_slice(&size.to_le_bytes());
    assert!(NativeCapture::read_from(Cursor::new(bytes)).is_err());
}

#[test]
fn future_manifest_and_expansion_bomb_are_rejected() {
    let (_temp, _db, mut package) = capture();
    package.manifest.database_section_version = 999;
    let mut writer = Cursor::new(Vec::new());
    package.write_to(&mut writer).unwrap();
    assert!(NativeCapture::read_from(Cursor::new(writer.into_inner())).is_err());
    let (_temp, _db, package) = capture();
    let mut bytes = Cursor::new(Vec::new());
    package.write_to(&mut bytes).unwrap();
    let mut bytes = bytes.into_inner();
    let end = bytes.len() - 22;
    let offset = u32::from_le_bytes(bytes[end + 16..end + 20].try_into().unwrap()) as usize;
    bytes[offset + 24..offset + 28].copy_from_slice(&(MAX_MANIFEST as u32 + 1).to_le_bytes());
    assert!(NativeCapture::read_from(Cursor::new(bytes)).is_err());
}

#[test]
fn declared_table_counts_must_match_the_verified_snapshot() {
    let (_temp, _db, mut package) = capture();
    assert_eq!(package.manifest.table_counts.get("preferences"), Some(&1));
    package
        .manifest
        .table_counts
        .insert("preferences".into(), 2);
    let mut bytes = Cursor::new(Vec::new());
    package.write_to(&mut bytes).unwrap();
    assert!(NativeCapture::read_from(Cursor::new(bytes.into_inner())).is_err());
}

#[test]
fn archive_bounds_encryption_overlap_and_corrupt_payload_are_rejected() {
    let (_temp, _db, package) = capture();
    let mut good = Cursor::new(Vec::new());
    package.write_to(&mut good).unwrap();
    let good = good.into_inner();
    let end = good.len() - 22;
    let central = u32::from_le_bytes(good[end + 16..end + 20].try_into().unwrap()) as usize;
    let local_data = 30
        + u16::from_le_bytes(good[26..28].try_into().unwrap()) as usize
        + u16::from_le_bytes(good[28..30].try_into().unwrap()) as usize;
    for scenario in 0..3 {
        let mut bytes = good.clone();
        match scenario {
            0 => {
                bytes[6] |= 1;
                bytes[central + 8] |= 1;
            }
            1 => {
                let size =
                    u32::from_le_bytes(bytes[central + 20..central + 24].try_into().unwrap()) + 40;
                bytes[central + 20..central + 24].copy_from_slice(&size.to_le_bytes());
            }
            _ => bytes[local_data + 2] ^= 0x7f,
        }
        assert!(NativeCapture::read_from(Cursor::new(bytes)).is_err());
    }
    let sparse = tempfile::tempfile().unwrap();
    sparse.set_len(MAX_ARCHIVE + 1).unwrap();
    assert_eq!(
        NativeCapture::read_from(sparse).err().unwrap().code,
        ErrorCode::BudgetExceeded
    );
}

#[test]
fn exact_operation_rejects_a_different_package_even_with_the_same_title() {
    let (temp, _db, mut package) = capture();
    let manager = CourseManager::open(temp.path().join("target")).unwrap();
    let op = Uuid::new_v4();
    manager
        .restore_native(op, "Restore".into(), &package, false)
        .unwrap();
    let before = manager.list().unwrap();
    package.manifest.source_course_id = Uuid::new_v4();
    assert_eq!(
        manager
            .restore_native(op, "Restore".into(), &package, false)
            .unwrap_err()
            .code,
        ErrorCode::IdempotencyConflict
    );
    assert_eq!(manager.list().unwrap(), before);
}

pub(super) fn populated() -> (tempfile::TempDir, Database) {
    let temp = tempfile::tempdir().unwrap();
    let mut db = Database::open(&temp.path().join("source")).unwrap();
    let resource = crate::organization::ResourceId::generate();
    db.apply_content(
        RequestContext::local_ui(),
        &ApplyContent {
            contract_version: 1,
            workspace_id: db.id.clone(),
            operation_id: OperationId::generate(),
            expected_revision: 0,
            command: crate::organization::Command::CreateResource {
                id: resource,
                title: "Material".into(),
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
            &mut Cursor::new(br#"[{"key":"a","title":"Title","body":"Meaning"}]"#),
            MAX_ASSET,
        )
        .unwrap();
    let source = db
        .save_source(
            RequestContext::local_ui(),
            &SaveSource {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                expected_revision: 0,
                source: NewSource {
                    id: Uuid::new_v4(),
                    resource_id: resource,
                    digest: asset.digest,
                    filename: "records.json".into(),
                    media_type: "application/json".into(),
                    attribution: Some("Author".into()),
                    license: Some("CC0".into()),
                    origin_url: None,
                },
            },
        )
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
    let mapping=serde_json::from_value(serde_json::json!({"keyColumn":0,"titleColumn":0,"bodyColumn":0,"jsonProperties":{"key":"key","title":"title","body":"body"}})).unwrap();
    db.apply_ingestion_job(
        RequestContext::local_ui(),
        &ApplyIngestionJob {
            contract_version: 1,
            workspace_id: db.id.clone(),
            operation_id: OperationId::generate(),
            job_id: job.id,
            expected_revision: 0,
            expected_library_revision: db.library(RequestContext::local_ui()).unwrap().revision,
            mapping,
            selected: vec![0],
        },
    )
    .unwrap();
    let unit = Uuid::new_v4().to_string();
    db.connection
        .execute(
            "INSERT INTO units VALUES(?1,'Lesson','Objective',0)",
            [&unit],
        )
        .unwrap();
    let draft = db
        .lesson_create(
            RequestContext::local_ui(),
            &CreateLessonDraft {
                contract_version: 1,
                workspace_id: db.id.clone(),
                unit_id: unit.clone(),
                operation_id: OperationId::generate(),
                expected_head_revision: None,
            },
        )
        .unwrap();
    let draft=db.lesson_edit(RequestContext::local_ui(),&EditLessonDraft {contract_version:1,workspace_id:db.id.clone(),unit_id:unit.clone(),draft_id:draft.id,expected_draft_revision:draft.revision,operation_id:OperationId::generate(),operations:vec![serde_json::from_value(serde_json::json!({"kind":"createSourceBindingRevision","sourceId":source.id,"bindingId":"source","locator":null,"declaredUsedRanges":["record:0"]})).unwrap()]}).unwrap();
    let scope = LessonDraftScope {
        contract_version: 1,
        workspace_id: db.id.clone(),
        unit_id: unit.clone(),
        draft_id: draft.id.clone(),
        expected_draft_revision: draft.revision,
    };
    let preview = db
        .lesson_preview(RequestContext::local_ui(), &scope)
        .unwrap();
    db.lesson_apply(
        RequestContext::local_ui(),
        &ApplyLessonDraft {
            contract_version: 1,
            workspace_id: db.id.clone(),
            unit_id: unit.clone(),
            draft_id: draft.id,
            expected_draft_revision: draft.revision,
            expected_head_revision: None,
            preview_hash: preview.hash,
            operation_id: OperationId::generate(),
        },
    )
    .unwrap();
    db.lesson_create(
        RequestContext::local_ui(),
        &CreateLessonDraft {
            contract_version: 1,
            workspace_id: db.id.clone(),
            unit_id: unit.clone(),
            operation_id: OperationId::generate(),
            expected_head_revision: db
                .lesson_read(RequestContext::local_ui(), &unit, None)
                .unwrap()
                .map(|x| x.id),
        },
    )
    .unwrap();
    let saved = db
        .save_document(
            RequestContext::local_ui(),
            &SaveDocument {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                resource_id: resource,
                expected_revision: 0,
                document: crate::document::Document {
                    schema_version: 1,
                    blocks: vec![crate::document::Block {
                        id: Uuid::new_v4(),
                        content: crate::document::BlockContent::Code {
                            text: "a😀b".into(),
                            language: None,
                        },
                    }],
                },
            },
        )
        .unwrap();
    let anchor = crate::document::Anchor {
        resource_id: resource.as_uuid(),
        revision_id: saved.id,
        block_id: saved.document.blocks[0].id,
        start_utf16: 1,
        end_utf16: 3,
        exact_quote: "😀".into(),
        quote_hash: None,
    };
    db.save_annotation(
        RequestContext::local_ui(),
        &SaveAnnotation {
            contract_version: 1,
            workspace_id: db.id.clone(),
            operation_id: OperationId::generate(),
            expected_revision: 0,
            annotation: Annotation {
                id: Uuid::new_v4(),
                resource_id: resource,
                revision_id: saved.id,
                revision: 0,
                anchor,
                body: "Remember".into(),
            },
        },
    )
    .unwrap();
    db.save_reading_position(
        RequestContext::local_ui(),
        &SaveReadingPosition {
            contract_version: 1,
            workspace_id: db.id.clone(),
            operation_id: OperationId::generate(),
            expected_revision: 0,
            position: ReadingPosition {
                resource_id: resource,
                revision_id: saved.id,
                block_id: saved.document.blocks[0].id,
                offset_utf16: 1,
                revision: 0,
            },
        },
    )
    .unwrap();
    (temp, db)
}

#[test]
fn native_snapshot_preserves_real_json_jobs_lessons_receipts_and_all_rows() {
    let (_temp, mut db) = populated();
    let package = roundtrip(
        &db.capture_native(RequestContext::local_ui(), Uuid::new_v4())
            .unwrap(),
    );
    let snapshot = database::open_connection(
        &package.staging.path().join("database.sqlite3"),
        database::Access::ReadOnly,
    )
    .unwrap();
    let tables: Vec<String> = db
        .connection
        .prepare("SELECT name FROM sqlite_schema WHERE type='table' ORDER BY name")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<std::result::Result<_, _>>()
        .unwrap();
    for table in tables {
        let sql = format!("SELECT * FROM \"{table}\" ORDER BY 1");
        fn rows(connection: &Connection, sql: &str) -> Vec<Vec<rusqlite::types::Value>> {
            let mut query = connection.prepare(sql).unwrap();
            let columns = query.column_count();
            query
                .query_map([], |r| (0..columns).map(|i| r.get(i)).collect())
                .unwrap()
                .collect::<std::result::Result<_, _>>()
                .unwrap()
        }
        assert_eq!(rows(&snapshot, &sql), rows(&db.connection, &sql), "{table}");
    }
    assert_eq!(
        snapshot
            .query_row("SELECT count(*) FROM ingestion_jobs", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        snapshot
            .query_row("SELECT count(*) FROM lesson_drafts", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
}

#[test]
fn service_capture_drains_with_shutdown_and_stopped_manager_cannot_restore() {
    tauri::async_runtime::block_on(async {
        let temp = tempfile::tempdir().unwrap();
        let manager = CourseManager::open(temp.path().join("app")).unwrap();
        let course = manager.list().unwrap().selected_course_id;
        let service = manager.service(course).unwrap();
        service.status(RequestContext::local_ui()).await.unwrap();
        let captured = service
            .capture_native(RequestContext::local_ui(), course)
            .await
            .unwrap();
        assert!(captured.manifest.complete);
        manager.shutdown().await.unwrap();
        assert!(
            service
                .capture_native(RequestContext::local_ui(), course)
                .await
                .is_err()
        );
        assert_eq!(
            manager
                .restore_native(Uuid::new_v4(), "Restore".into(), &captured, false)
                .unwrap_err()
                .code,
            ErrorCode::OwnerUnavailable
        );
        let _replacement = CourseManager::open(temp.path().join("app")).unwrap();
    });
}

#[test]
#[ignore = "private child process for killed native restore test"]
fn restore_crash_child() {
    let root = std::path::PathBuf::from(std::env::var_os("MEMOQUILL_NATIVE_CRASH_ROOT").unwrap());
    let operation =
        Uuid::parse_str(&std::env::var("MEMOQUILL_NATIVE_CRASH_OPERATION").unwrap()).unwrap();
    let after_commit = std::env::var_os("MEMOQUILL_NATIVE_CRASH_COMMITTED").is_some();
    let package = NativeCapture::read_from(File::open(root.join("package.zip")).unwrap()).unwrap();
    let manager = CourseManager::open(root.join("target")).unwrap();
    let ready = root.join("ready");
    faults::using(
        move |phase| {
            if phase
                == if after_commit {
                    faults::Phase::AfterCommit
                } else {
                    faults::Phase::Publish
                }
            {
                std::fs::write(&ready, b"ready").unwrap();
                loop {
                    std::thread::park_timeout(std::time::Duration::from_secs(1));
                }
            }
            Ok(())
        },
        || manager.restore_native(operation, "Restored".into(), &package, false),
    )
    .unwrap();
}

#[test]
fn killed_restore_recovers_owned_root_and_exact_receipt_without_duplicate_course() {
    for committed in [false, true] {
        let (temp, _db, package) = capture();
        let operation = Uuid::new_v4();
        package
            .write_to(File::create(temp.path().join("package.zip")).unwrap())
            .unwrap();
        let mut command = std::process::Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "workspace::packages::tests::restore_crash_child",
                "--ignored",
            ])
            .env("MEMOQUILL_NATIVE_CRASH_ROOT", temp.path())
            .env("MEMOQUILL_NATIVE_CRASH_OPERATION", operation.to_string())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        if committed {
            command.env("MEMOQUILL_NATIVE_CRASH_COMMITTED", "1");
        }
        let mut child = command.spawn().unwrap();
        let started = std::time::Instant::now();
        while !temp.path().join("ready").exists()
            && started.elapsed() < std::time::Duration::from_secs(10)
        {
            if child.try_wait().unwrap().is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let ready = temp.path().join("ready").exists();
        let _ = child.kill();
        let _ = child.wait();
        assert!(ready, "child failed before crash window");
        let manager = CourseManager::open(temp.path().join("target")).unwrap();
        manager
            .restore_native(operation, "Restored".into(), &package, false)
            .unwrap();
        assert_eq!(manager.list().unwrap().courses.len(), 2);
        assert_eq!(
            manager
                .restore_native(operation, "Restored".into(), &package, false)
                .unwrap()
                .course_id,
            operation
        );
    }
}

#[test]
fn native_wal_snapshot_roundtrips_without_changing_existing_course() {
    let temp = tempfile::tempdir().unwrap();
    let mut db = Database::open(&temp.path().join("source")).unwrap();
    db.connection
        .execute("UPDATE preferences SET revision=12", [])
        .unwrap();
    let source = Uuid::new_v4();
    let capture = db
        .capture_native(RequestContext::local_ui(), source)
        .unwrap();
    let mut bytes = Cursor::new(Vec::new());
    capture.write_to(&mut bytes).unwrap();
    let verified = NativeCapture::read_from(Cursor::new(bytes.into_inner())).unwrap();
    let manager = CourseManager::open(temp.path().join("target")).unwrap();
    let existing = manager.list().unwrap();
    let operation = Uuid::new_v4();
    let restored = manager
        .restore_native(operation, "Restored".into(), &verified, false)
        .unwrap();
    assert_eq!(restored.source_course_id, source);
    assert_eq!(restored.course_id, operation);
    assert!(restored.complete);
    assert_eq!(manager.list().unwrap().courses[0], existing.courses[0]);
    let reopened = Database::open(
        &temp
            .path()
            .join("target/courses/data")
            .join(operation.to_string()),
    )
    .unwrap();
    assert_eq!(reopened.id, db.id);
    assert_eq!(reopened.preferences().unwrap().revision, 12);
    assert_eq!(
        manager
            .restore_native(operation, "Restored".into(), &verified, false)
            .unwrap(),
        restored
    );
}

#[test]
fn malformed_annotation_domain_is_rejected_before_native_admission() {
    let (_temp, mut db) = populated();
    let mut package = db
        .capture_native(RequestContext::local_ui(), Uuid::new_v4())
        .unwrap();
    let path = package.staging.path().join("database.sqlite3");
    let connection = Connection::open(&path).unwrap();
    connection
        .execute("UPDATE annotations SET anchor='{}'", [])
        .unwrap();
    package.manifest.table_counts = validation::table_counts(&connection).unwrap();
    drop(connection);
    let (hash, size) = hash_file(&path, MAX_DATABASE).unwrap();
    package.manifest.payloads[0].sha256 = hash;
    package.manifest.payloads[0].byte_length = size;
    let mut archive = Cursor::new(Vec::new());
    package.write_to(&mut archive).unwrap();
    assert!(
        NativeCapture::read_from(Cursor::new(archive.into_inner())).is_err(),
        "malformed anchor admitted despite valid schema/FKs/counts/hashes"
    );
}

fn reject_mutation(sql: &str) {
    let (_temp, mut db) = populated();
    let mut package = db
        .capture_native(RequestContext::local_ui(), Uuid::new_v4())
        .unwrap();
    let path = package.staging.path().join("database.sqlite3");
    let connection = Connection::open(&path).unwrap();
    assert!(
        connection.execute(sql, []).unwrap() > 0,
        "mutation matched no rows: {sql}"
    );
    package.manifest.table_counts = validation::table_counts(&connection).unwrap();
    drop(connection);
    let (hash, size) = hash_file(&path, MAX_DATABASE).unwrap();
    package.manifest.payloads[0].sha256 = hash;
    package.manifest.payloads[0].byte_length = size;
    let mut archive = Cursor::new(Vec::new());
    package.write_to(&mut archive).unwrap();
    assert!(
        NativeCapture::read_from(Cursor::new(archive.into_inner())).is_err(),
        "domain mutation admitted: {sql}"
    );
}

#[test]
fn malformed_note_position_and_source_domains_are_rejected() {
    for sql in [
        "UPDATE annotations SET anchor=json_set(anchor,'$.startUtf16',2)",
        "UPDATE annotations SET anchor=json_set(anchor,'$.exactQuote','wrong')",
        "UPDATE annotations SET body=char(0)",
        "UPDATE reading_positions SET offset_utf16=2",
        "UPDATE reading_positions SET block_id='00000000-0000-4000-8000-000000000001'",
        "UPDATE original_sources SET filename='../source.json'",
        "UPDATE original_sources SET origin_url='file:///etc/passwd'",
        "UPDATE source_revisions SET revision=revision+1",
        "UPDATE original_sources SET revision=revision+1",
    ] {
        reject_mutation(sql);
    }
}

#[test]
fn malformed_operation_receipt_domains_are_rejected() {
    for sql in [
        "UPDATE operations SET result=json_set(result,'$.operationId','00000000-0000-4000-8000-000000000001')",
        "UPDATE operations SET state='interrupted' WHERE state='applied'",
        "UPDATE operations SET kind='preferences' WHERE kind='annotation'",
        "UPDATE operations SET result=json_set(result,'$.result.value.value.anchor.startUtf16',2) WHERE kind='annotation'",
        "UPDATE operations SET result=json_set(result,'$.result.value.value.offsetUtf16',2) WHERE kind='reading_position'",
        "UPDATE operations SET result=json_set(result,'$.result.value.value.filename','../invalid') WHERE json_extract(result,'$.result.value.kind')='source'",
        "UPDATE operations SET result=json_set(result,'$.result.value.value.saved.hash','wrong') WHERE json_extract(result,'$.result.value.kind')='lesson'",
    ] {
        reject_mutation(sql);
    }
}

#[test]
fn malformed_lesson_receipt_envelopes_are_rejected() {
    for sql in [
        "UPDATE operations SET result=json_set(result,'$.result.value.value.revision',9007199254740992) WHERE json_extract(result,'$.result.value.kind')='lesson_draft'",
        "UPDATE operations SET result=json_set(result,'$.result.value.value.unitId','00000000-0000-4000-8000-000000000001') WHERE json_extract(result,'$.result.value.kind')='lesson_draft'",
        "UPDATE operations SET result=json_set(result,'$.result.value.value.saved.unitId','00000000-0000-4000-8000-000000000001') WHERE json_extract(result,'$.result.value.kind')='lesson'",
    ] {
        reject_mutation(sql);
    }
}

#[test]
fn historical_receipts_and_notes_survive_changed_heads_and_mutable_values() {
    let (_temp, mut db) = populated();
    let raw: String = db
        .connection
        .query_row("SELECT resource_id FROM annotations", [], |r| r.get(0))
        .unwrap();
    let resource = serde_json::from_value(serde_json::Value::String(raw)).unwrap();
    let mut note = db
        .annotations(RequestContext::local_ui(), resource)
        .unwrap()
        .remove(0);
    let old_revision = note.revision;
    note.body = "Updated annotation".into();
    db.save_annotation(
        RequestContext::local_ui(),
        &SaveAnnotation {
            contract_version: 1,
            workspace_id: db.id.clone(),
            operation_id: OperationId::generate(),
            expected_revision: old_revision,
            annotation: note.clone(),
        },
    )
    .unwrap();
    let mut position = db
        .reading_position(RequestContext::local_ui(), resource)
        .unwrap()
        .unwrap();
    let old_revision = position.revision;
    position.offset_utf16 = 3;
    db.save_reading_position(
        RequestContext::local_ui(),
        &SaveReadingPosition {
            contract_version: 1,
            workspace_id: db.id.clone(),
            operation_id: OperationId::generate(),
            expected_revision: old_revision,
            position,
        },
    )
    .unwrap();
    let saved =
        super::super::content::documents::read(&db.connection, resource, Some(note.revision_id))
            .unwrap()
            .unwrap();
    db.save_document(
        RequestContext::local_ui(),
        &SaveDocument {
            contract_version: 1,
            workspace_id: db.id.clone(),
            operation_id: OperationId::generate(),
            resource_id: resource,
            expected_revision: saved.revision,
            document: saved.document,
        },
    )
    .unwrap();
    roundtrip(
        &db.capture_native(RequestContext::local_ui(), Uuid::new_v4())
            .unwrap(),
    );
}

#[test]
fn prepared_receipts_require_actual_asset_journal_shape() {
    reject_mutation("UPDATE operations SET state='prepared',result=NULL WHERE kind='annotation'");
}
