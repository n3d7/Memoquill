use super::*;
use serde_json::{Value, json};
use std::io::Cursor;
fn fixture() -> (tempfile::TempDir, Database) {
    let (temp, mut db) = super::super::tests::populated();
    let unit: String = db
        .connection
        .query_row("SELECT id FROM units LIMIT 1", [], |r| r.get(0))
        .unwrap();
    let draft = db
        .lesson_list_drafts(RequestContext::local_ui(), &unit)
        .unwrap()
        .remove(0);
    let resource = Uuid::new_v4();
    let doc = crate::document::Document {
        schema_version: 1,
        blocks: vec![crate::document::Block {
            id: Uuid::new_v4(),
            content: crate::document::BlockContent::Code {
                text: "old pinned content".into(),
                language: None,
            },
        }],
    };
    let draft=db.lesson_edit(RequestContext::local_ui(),&EditLessonDraft{contract_version:1,workspace_id:db.id.clone(),unit_id:unit.clone(),draft_id:draft.id,expected_draft_revision:draft.revision,operation_id:OperationId::generate(),operations:vec![LessonOperation::CreateResourceRevision{resource_id:resource.to_string(),expected_revision_id:None,binding_id:"reading".into(),title:"Pinned material".into(),document:doc.clone()},LessonOperation::CreateActivityRevision{activity_id:Uuid::new_v4().to_string(),expected_revision_id:None,binding_id:"recall".into(),definition:json!({"family":"recall","version":1,"prompt":"Recall","answer":"Answer","hints":[],"evaluation":"self"})}]}).unwrap();
    let preview = db
        .lesson_preview(
            RequestContext::local_ui(),
            &LessonDraftScope {
                contract_version: 1,
                workspace_id: db.id.clone(),
                unit_id: unit.clone(),
                draft_id: draft.id.clone(),
                expected_draft_revision: draft.revision,
            },
        )
        .unwrap();
    db.lesson_apply(
        RequestContext::local_ui(),
        &ApplyLessonDraft {
            contract_version: 1,
            workspace_id: db.id.clone(),
            unit_id: unit.clone(),
            draft_id: draft.id,
            expected_draft_revision: draft.revision,
            expected_head_revision: db
                .lesson_read(RequestContext::local_ui(), &unit, None)
                .unwrap()
                .map(|r| r.id),
            preview_hash: preview.hash,
            operation_id: OperationId::generate(),
        },
    )
    .unwrap();
    let mut new = doc;
    new.blocks[0].content = crate::document::BlockContent::Code {
        text: "new head content".into(),
        language: None,
    };
    db.save_document(
        RequestContext::local_ui(),
        &SaveDocument {
            contract_version: 1,
            workspace_id: db.id.clone(),
            operation_id: OperationId::generate(),
            resource_id: crate::organization::ResourceId::try_from(resource).unwrap(),
            expected_revision: 1,
            document: new,
        },
    )
    .unwrap();
    (temp, db)
}
fn bytes(p: &PortableCapture) -> Vec<u8> {
    let mut w = Cursor::new(Vec::new());
    p.write_to(&mut w).unwrap();
    w.into_inner()
}
fn reopen(p: &PortableCapture) -> PortableCapture {
    PortableCapture::read_from(Cursor::new(bytes(p))).unwrap()
}
#[test]
fn portable_capture_accepts_real_authored_course() {
    let (temp, mut db) = fixture();
    let p = db
        .capture_portable(RequestContext::local_ui(), Uuid::new_v4(), false)
        .unwrap();
    let p = reopen(&p);
    let serialized = bytes(&p);
    let mut zip = zip::ZipArchive::new(Cursor::new(serialized)).unwrap();
    assert_eq!(zip.len(), 3);
    assert!(zip.by_name("database.sqlite3").is_err());
    let mut json = String::new();
    zip.by_name("content.ndjson")
        .unwrap()
        .read_to_string(&mut json)
        .unwrap();
    for private in [
        "Remember",
        "annotations",
        "readingPositions",
        "lessonDrafts",
        "ingestionJobs",
        "operations",
        "preferences",
    ] {
        assert!(!json.contains(private), "private leak: {private}");
    }
    assert_eq!(p.manifest.authored_versions["lessonRevisions"], 2);
    assert_eq!(p.manifest.authored_versions["resourceRevisions"], 4);
    assert_eq!(p.manifest.authored_versions["activityRevisions"], 1);
    assert_eq!(p.manifest.intentionally_omitted_assets.len(), 1);
    assert!(p.manifest.unavailable_assets.is_empty());
    let manager = CourseManager::open(temp.path().join("target")).unwrap();
    let operation = Uuid::new_v4();
    let report = manager
        .import_portable(operation, "Imported".into(), &p)
        .unwrap();
    assert!(!report.full_recovery);
    assert_eq!(report.kind, "portableContent");
    assert_eq!(report.course_id, operation);
    let imported = Database::open(
        &temp
            .path()
            .join("target/courses/data")
            .join(operation.to_string()),
    )
    .unwrap();
    let expected = Content::capture(&db.connection).unwrap();
    let actual = Content::capture(&imported.connection).unwrap();
    assert_eq!(
        serde_json::to_value(expected).unwrap(),
        serde_json::to_value(actual).unwrap()
    );
    for table in [
        "annotations",
        "reading_positions",
        "lesson_drafts",
        "ingestion_jobs",
    ] {
        assert_eq!(
            imported
                .connection
                .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
    let pinned:i64=imported.connection.query_row("SELECT count(*) FROM lesson_dependencies d JOIN resource_heads h ON h.resource_id=d.target_id WHERE d.kind='resource' AND d.revision_id!=h.revision_id",[],|r|r.get(0)).unwrap();
    assert_eq!(pinned, 1);
    let coverage: String = imported
        .connection
        .query_row(
            "SELECT json_extract(data,'$.readCoverage') FROM lesson_source_bindings LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(coverage, "unknown");
}
#[test]
fn originals_opt_in_missing_assets_and_captured_bytes_are_distinct() {
    let (_temp, mut db) = fixture();
    let p = db
        .capture_portable(RequestContext::local_ui(), Uuid::new_v4(), true)
        .unwrap();
    assert!(p.manifest.intentionally_omitted_assets.is_empty());
    assert!(p.manifest.unavailable_assets.is_empty());
    assert_eq!(p.manifest.payloads.len(), 3);
    let digest = validation::asset_rows(&db.connection).unwrap()[0].0.clone();
    std::fs::remove_file(assets::asset_path(db.owner.root(), &digest)).unwrap();
    assert_eq!(reopen(&p).manifest.payloads.len(), 3);
    for originals in [false, true] {
        let missing = db
            .capture_portable(RequestContext::local_ui(), Uuid::new_v4(), originals)
            .unwrap();
        assert_eq!(
            reopen(&missing).manifest.unavailable_assets,
            vec![digest.clone()]
        );
        assert!(missing.manifest.intentionally_omitted_assets.is_empty());
    }
}
// Recompute every envelope digest/count, so these mutants exercise semantic validation.
fn mutant(p: &PortableCapture, edit: impl FnOnce(&mut Value, &mut Value)) -> Vec<u8> {
    let mut manifest = serde_json::to_value(&p.manifest).unwrap();
    let mut content: Value = serde_json::to_value(
        Content::decode(&std::fs::read(p.native.staging.path().join("content.ndjson")).unwrap())
            .unwrap(),
    )
    .unwrap();
    edit(&mut manifest, &mut content);
    let mut payloads = BTreeMap::new();
    for payload in &p.manifest.payloads {
        payloads.insert(
            payload.name.clone(),
            std::fs::read(p.native.staging.path().join(&payload.name)).unwrap(),
        );
    }
    payloads.insert(
        "content.ndjson".into(),
        b"{\"family\":\"unknown\",\"record\":{}}\n".to_vec(),
    );
    if let Ok(typed) = serde_json::from_value::<Content>(content) {
        payloads.insert("content.ndjson".into(), typed.encode().unwrap());
        payloads.insert("readable/content.txt".into(), projection(&typed).unwrap());
        manifest["includedRecords"] = serde_json::to_value(typed.counts().unwrap()).unwrap();
        manifest["authoredVersions"] = serde_json::to_value(versions(&typed)).unwrap();
    }
    for payload in manifest["payloads"].as_array_mut().unwrap() {
        let value = &payloads[payload["name"].as_str().unwrap()];
        payload["sha256"] = json!(hash_bytes(value));
        payload["byteLength"] = json!(value.len());
    }
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default();
    writer.start_file("manifest.json", options).unwrap();
    writer
        .write_all(&serde_json::to_vec(&manifest).unwrap())
        .unwrap();
    for (name, value) in payloads {
        writer.start_file(name, options).unwrap();
        writer.write_all(&value).unwrap();
    }
    writer.finish().unwrap().into_inner()
}
#[test]
fn semantic_mutants_unknown_contracts_and_duplicate_rows_are_rejected() {
    let (_temp, mut db) = fixture();
    let p = db
        .capture_portable(RequestContext::local_ui(), Uuid::new_v4(), false)
        .unwrap();
    type Edit = fn(&mut Value, &mut Value);
    let cases: [(&str, Edit); 12] = [
        ("future format", |m, _| m["formatVersion"] = json!(2)),
        ("future section", |m, _| m["sections"]["content"] = json!(2)),
        ("M4 required adapter", |m, _| {
            m["requiredAdapters"] = json!({"history":1})
        }),
        ("private field", |_, c| c["annotations"] = json!([])),
        ("duplicate identity", |_, c| {
            let v = c["resources"][0].clone();
            c["resources"].as_array_mut().unwrap().push(v);
        }),
        ("invalid document", |_, c| {
            c["resourceRevisions"][0]["document"]["schemaVersion"] = json!(99)
        }),
        ("broken source closure", |_, c| {
            c["originalSources"][0]["resourceId"] = json!(Uuid::new_v4().to_string())
        }),
        ("missing pins", |_, c| c["lessonDependencies"] = json!([])),
        ("malformed source metadata", |_, c| {
            c["originalSources"][0]["filename"] = json!("bad\u{0}")
        }),
        ("dishonest read coverage", |_, c| {
            c["lessonSourceBindings"][0]["data"]["readCoverage"] = json!("complete")
        }),
        ("invalid activity", |_, c| {
            c["lessonActivityRevisions"][0]["definition"] = json!({"family":"recall","version":1})
        }),
        ("invalid uuid", |_, c| {
            c["resources"][0]["id"] = json!("not-an-id")
        }),
    ];
    for (name, edit) in cases {
        assert!(
            PortableCapture::read_from(Cursor::new(mutant(&p, edit))).is_err(),
            "admitted {name}"
        );
    }
}
#[test]
fn portable_retry_after_restart_binds_package_title_and_preserves_existing_courses() {
    for phase in [faults::Phase::Publish, faults::Phase::AfterCommit] {
        let (temp, mut db) = fixture();
        let p = db
            .capture_portable(RequestContext::local_ui(), Uuid::new_v4(), false)
            .unwrap();
        let data = bytes(&p);
        let target = temp.path().join("target");
        let manager = CourseManager::open(target.clone()).unwrap();
        let before = manager.list().unwrap();
        let op = Uuid::new_v4();
        let result = faults::using(
            move |p| {
                if p == phase {
                    Err(AppError::new(ErrorCode::OutcomeUnconfirmed))
                } else {
                    Ok(())
                }
            },
            || manager.import_portable(op, "Imported".into(), &p),
        );
        assert_eq!(result.unwrap_err().code, ErrorCode::OutcomeUnconfirmed);
        drop(manager);
        let manager = CourseManager::open(target).unwrap();
        let p = PortableCapture::read_from(Cursor::new(data)).unwrap();
        assert_eq!(
            manager
                .import_portable(op, "Changed".into(), &p)
                .unwrap_err()
                .code,
            ErrorCode::IdempotencyConflict
        );
        let report = manager.import_portable(op, "Imported".into(), &p).unwrap();
        assert_eq!(report.course_id, op);
        assert_eq!(
            manager.import_portable(op, "Imported".into(), &p).unwrap(),
            report
        );
        let changed = db
            .capture_portable(RequestContext::local_ui(), Uuid::new_v4(), false)
            .unwrap();
        assert_eq!(
            manager
                .import_portable(op, "Imported".into(), &changed)
                .unwrap_err()
                .code,
            ErrorCode::IdempotencyConflict
        );
        let after = manager.list().unwrap();
        assert_eq!(after.courses.len(), before.courses.len() + 1);
        for course in before.courses {
            assert!(after.courses.contains(&course));
        }
    }
}

#[test]
fn ndjson_rejects_unknown_rows_blank_lines_truncation_and_oversize_rows() {
    for bytes in [
        b"{\"family\":\"privateDraft\",\"record\":{}}\n".as_slice(),
        b"\n",
        b"{\"family\":\"resources\",\"record\":{}}\n",
        b"{}",
        b"{not json}\n",
    ] {
        assert!(Content::decode(bytes).is_err());
    }
    let mut too_long = vec![b' '; 2 * 1024 * 1024 + 1];
    too_long.push(b'\n');
    assert_eq!(
        Content::decode(&too_long).unwrap_err().code,
        ErrorCode::BudgetExceeded
    );
    assert!(Content::decode(b"").unwrap().resources.is_empty());
}

#[test]
fn orphan_authored_activity_is_validated_even_without_any_pin() {
    let (_temp, mut db) = fixture();
    let p = db
        .capture_portable(RequestContext::local_ui(), Uuid::new_v4(), false)
        .unwrap();
    let malformed = mutant(&p, |_, c| {
        let definition = json!({"family":"recall","version":1});
        let row = json!({"id":Uuid::new_v4().to_string(),"activityId":Uuid::new_v4().to_string(),"unitId":c["units"][0]["id"],"digest":hash_bytes(&serde_json::to_vec(&definition).unwrap()),"definition":definition});
        c["lessonActivityRevisions"]
            .as_array_mut()
            .unwrap()
            .push(row);
    });
    assert!(
        PortableCapture::read_from(Cursor::new(malformed)).is_err(),
        "malformed orphan authored activity admitted"
    );
}

#[test]
fn opted_in_study_history_rejects_undeclared_sections_and_invalid_receipts() {
    let (_temp, mut db) = fixture();
    let (unit, revision): (String, String) = db
        .connection
        .query_row(
            "SELECT unit_id,revision_id FROM lesson_heads LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    let request = serde_json::from_value(json!({"operationId":Uuid::new_v4(),"sessionId":Uuid::new_v4(),"expectedRevision":0,"action":{"kind":"start","unitId":unit,"compositionRevisionId":revision,"mode":"practice"}})).unwrap();
    db.study_apply(RequestContext::local_ui(), &request)
        .unwrap();
    let p = db
        .capture_portable_with_history(RequestContext::local_ui(), Uuid::new_v4(), false, true)
        .unwrap();
    reopen(&p);
    let undeclared = mutant(&p, |manifest, _| {
        manifest["sections"]
            .as_object_mut()
            .unwrap()
            .remove("studyHistory");
        manifest["omissions"] = json!(omissions(false));
    });
    assert!(PortableCapture::read_from(Cursor::new(undeclared)).is_err());
    let legacy = mutant(&p, |manifest, _| {
        manifest["sections"]["studyHistory"] = json!(1);
    });
    assert!(PortableCapture::read_from(Cursor::new(legacy)).is_ok());
    let unsupported = mutant(&p, |manifest, _| {
        manifest["sections"]["studyHistory"] = json!(3);
    });
    assert!(PortableCapture::read_from(Cursor::new(unsupported)).is_err());
    let invalid_receipt = mutant(&p, |_, content| {
        content["studyOperations"][0]["requestHash"] = json!("not-a-hash");
    });
    assert!(PortableCapture::read_from(Cursor::new(invalid_receipt)).is_err());
}

#[test]
fn old_content_only_omission_declaration_remains_readable() {
    let (_temp, mut db) = fixture();
    let p = db
        .capture_portable(RequestContext::local_ui(), Uuid::new_v4(), false)
        .unwrap();
    let legacy = mutant(&p, |manifest, _| {
        let items = manifest["omissions"].as_array_mut().unwrap();
        *items.last_mut().unwrap() = json!("M4 attempts, history and schedules (not implemented)");
    });
    PortableCapture::read_from(Cursor::new(legacy)).unwrap();
}
