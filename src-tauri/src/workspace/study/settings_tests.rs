use super::*;

fn settings() -> serde_json::Value {
    json!({"contractVersion":1,"theme":"dark","textScale":150,"goal":"Explain unaided","focus":"Compare examples"})
}
fn configured(unit: &str, revision: &str) -> StudyMutation {
    serde_json::from_value(json!({"operationId":Uuid::new_v4(),"sessionId":Uuid::new_v4(),"expectedRevision":0,
        "action":{"kind":"start","unitId":unit,"compositionRevisionId":revision,"mode":"practice","phase":"independent","settings":settings()}})).unwrap()
}

#[test]
fn canonical_settings_survive_uncertain_start_draft_reopen_and_portable_history() {
    let (temp, mut db, unit, revision) = setup(recall());
    let request = configured(&unit, &revision);
    let failure = faults::using(
        |phase| {
            if phase == faults::Phase::AfterCommit {
                Err(AppError::new(ErrorCode::StorageFailure))
            } else {
                Ok(())
            }
        },
        || db.study_apply(context(), &request),
    );
    assert!(failure.is_err());
    let started = db.study_apply(context(), &request).unwrap();
    assert_eq!(serde_json::to_value(&started.settings).unwrap(), settings());
    let saved = save(
        &mut db,
        &started,
        StudyResponse::Text {
            value: "Retained draft".into(),
        },
    );
    let mut changed = serde_json::to_value(&request).unwrap();
    changed["action"]["settings"]["goal"] = json!("Changed default");
    let changed: StudyMutation = serde_json::from_value(changed).unwrap();
    assert_eq!(
        db.study_apply(context(), &changed).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    validate_native_data(&db.connection).unwrap();
    drop(db);
    let mut db = Database::open(&temp.path().join("workspace")).unwrap();
    assert_eq!(db.study_read(context(), saved.id).unwrap().session, saved);
    assert_eq!(
        db.study_resume_sessions(context(), 0).unwrap().sessions[0].session_id,
        saved.id
    );
    for history in [false, true] {
        let capture = db
            .capture_portable_with_history(context(), Uuid::new_v4(), false, history)
            .unwrap();
        let mut bytes = std::io::Cursor::new(Vec::new());
        capture.write_to(&mut bytes).unwrap();
        let package = PortableCapture::read_from(std::io::Cursor::new(bytes.into_inner())).unwrap();
        let root = temp.path().join(format!("settings-{history}"));
        let manager = CourseManager::open(root.clone()).unwrap();
        let id = Uuid::new_v4();
        manager
            .import_portable(id, "Synthetic settings".into(), &package)
            .unwrap();
        let restored = Database::open(&root.join("courses/data").join(id.to_string())).unwrap();
        if history {
            assert_eq!(
                restored.study_read(context(), saved.id).unwrap().session,
                saved
            );
        } else {
            assert!(
                restored
                    .study_resume_sessions(context(), 0)
                    .unwrap()
                    .sessions
                    .is_empty()
            );
        }
        validate_native_data(&restored.connection).unwrap();
    }
}

#[test]
fn canonical_settings_boundaries_legacy_bytes_and_native_receipts_are_validated() {
    let (_temp, mut db, unit, revision) = setup(recall());
    for invalid in [json!(2), json!(0)] {
        let mut value = serde_json::to_value(configured(&unit, &revision)).unwrap();
        value["action"]["settings"]["contractVersion"] = invalid;
        let request: StudyMutation = serde_json::from_value(value).unwrap();
        assert_eq!(
            db.study_apply(context(), &request).unwrap_err().code,
            ErrorCode::InvalidRequest
        );
    }
    for goal in ["a".repeat(2049), "invalid\u{0001}".into()] {
        let mut value = serde_json::to_value(configured(&unit, &revision)).unwrap();
        value["action"]["settings"]["goal"] = json!(goal);
        assert_eq!(
            db.study_apply(context(), &serde_json::from_value(value).unwrap())
                .unwrap_err()
                .code,
            ErrorCode::InvalidRequest
        );
    }
    let mut bad = settings();
    bad["textScale"] = json!(175);
    assert!(serde_json::from_value::<StudySettings>(bad).is_err());
    let mut bad = settings();
    bad["feedback"] = json!("immediate");
    assert!(serde_json::from_value::<StudySettings>(bad).is_err());
    let legacy = start(&mut db, &unit, &revision, StudyMode::Practice);
    assert!(
        !serde_json::to_value(&legacy)
            .unwrap()
            .as_object()
            .unwrap()
            .contains_key("settings")
    );
    let request = configured(&unit, &revision);
    let mut session = db.study_apply(context(), &request).unwrap();
    session.settings.as_mut().unwrap().goal = "Tampered intent".into();
    let data = serde_json::to_string(&session).unwrap();
    db.connection
        .execute(
            "UPDATE study_sessions SET data=?1,digest=?2 WHERE id=?3",
            rusqlite::params![
                data,
                crate::workspace::migrations::checksum(data.as_bytes()),
                session.id.to_string()
            ],
        )
        .unwrap();
    assert!(validate_native_data(&db.connection).is_err());
}

#[test]
fn canonical_settings_enforce_the_declared_workload_and_activity_family() {
    let (_temp, mut db, unit, revision) = setup_sequence(vec![recall(), recall()]);
    let mut value = serde_json::to_value(configured(&unit, &revision)).unwrap();
    value["action"]["settings"]["itemLimit"] = json!(1);
    let too_many: StudyMutation = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(
        db.study_apply(context(), &too_many).unwrap_err().code,
        ErrorCode::InvalidRequest
    );
    value["action"]["bindingIds"] = json!(["practice"]);
    value["action"]["settings"]["activityFamily"] = json!("procedure");
    assert_eq!(
        db.study_apply(context(), &serde_json::from_value(value.clone()).unwrap())
            .unwrap_err()
            .code,
        ErrorCode::InvalidRequest
    );
    value["action"]["settings"]["activityFamily"] = json!("recall");
    let saved = db
        .study_apply(context(), &serde_json::from_value(value).unwrap())
        .unwrap();
    assert_eq!(saved.items.len(), 1);
    assert_eq!(saved.settings.as_ref().unwrap().item_limit, Some(1));
    validate_native_data(&db.connection).unwrap();
    for limit in [0, 65] {
        let mut value = serde_json::to_value(configured(&unit, &revision)).unwrap();
        value["action"]["settings"]["itemLimit"] = json!(limit);
        assert_eq!(
            db.study_apply(context(), &serde_json::from_value(value).unwrap())
                .unwrap_err()
                .code,
            ErrorCode::InvalidRequest
        );
    }
}
