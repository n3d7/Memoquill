use super::*;

#[test]
fn continuity_is_read_only_and_ranks_saved_work_without_wall_time() {
    let (_temp, mut db, unit, revision) = setup(recall());
    let first = start(&mut db, &unit, &revision, StudyMode::Practice);
    let second = start(&mut db, &unit, &revision, StudyMode::Practice);
    let first = save(
        &mut db,
        &first,
        StudyResponse::Text {
            value: "Acknowledged draft".into(),
        },
    );
    let changes = db.connection.total_changes();
    let page = db.study_resume_sessions(context(), 0).unwrap();
    assert_eq!(db.connection.total_changes(), changes);
    assert_eq!(
        page.sessions
            .iter()
            .map(|s| s.session_id)
            .collect::<Vec<_>>(),
        vec![first.id, second.id]
    );
    assert!(page.sessions[0].saved_response);
    assert_eq!(page.sessions[0].composition_revision_id, revision);
    assert_eq!(
        db.study_read(context(), first.id)
            .unwrap()
            .session
            .draft
            .response,
        first.draft.response
    );
    assert_eq!(
        db.study_resume_sessions(RequestContext::unverified_external(), 0)
            .unwrap_err()
            .code,
        ErrorCode::PermissionDenied
    );
    assert!(db.study_resume_sessions(context(), u64::MAX).is_err());
    let (_other, other, _, _) = setup(recall());
    assert!(
        other
            .study_resume_sessions(context(), 0)
            .unwrap()
            .sessions
            .is_empty()
    );
}

#[test]
fn continuity_excludes_completed_and_archived_lessons_and_pages_all_candidates() {
    let (_temp, mut db, unit, revision) = setup(recall());
    let s = start(&mut db, &unit, &revision, StudyMode::Practice);
    let s = save(
        &mut db,
        &s,
        StudyResponse::Text {
            value: "Answer".into(),
        },
    );
    db.study_apply(
        context(),
        &request(
            &s,
            StudyAction::Submit {
                self_rating: None,
                recall_rating: None,
                self_outcome: None,
            },
        ),
    )
    .unwrap();
    assert!(
        db.study_resume_sessions(context(), 0)
            .unwrap()
            .sessions
            .is_empty()
    );
    for _ in 0..65 {
        start(&mut db, &unit, &revision, StudyMode::Practice);
    }
    let a = db.study_resume_sessions(context(), 0).unwrap();
    let b = db
        .study_resume_sessions(context(), a.next_offset.unwrap())
        .unwrap();
    assert_eq!(a.sessions.len(), 64);
    assert_eq!(b.sessions.len(), 1);
    assert_eq!(b.next_offset, None);
    let ids: std::collections::HashSet<_> = a
        .sessions
        .iter()
        .chain(&b.sessions)
        .map(|s| s.session_id)
        .collect();
    assert_eq!(ids.len(), 65);
    db.connection
        .execute("UPDATE units SET archived=1 WHERE id=?1", [&unit])
        .unwrap();
    assert!(
        db.study_resume_sessions(context(), 0)
            .unwrap()
            .sessions
            .is_empty()
    );
}

#[test]
fn continuity_native_and_portable_recovery_preserve_drafts_and_operation_order() {
    use std::io::Cursor;
    let (temp, mut db, unit, revision) = setup(recall());
    let first = start(&mut db, &unit, &revision, StudyMode::Practice);
    start(&mut db, &unit, &revision, StudyMode::Assessment);
    let first = save(
        &mut db,
        &first,
        StudyResponse::Text {
            value: "Saved before interruption".into(),
        },
    );
    let expected = db.study_resume_sessions(context(), 0).unwrap();
    drop(db);
    let mut db = Database::open(&temp.path().join("workspace")).unwrap();
    assert_eq!(db.study_resume_sessions(context(), 0).unwrap(), expected);
    let capture = db.capture_native(context(), Uuid::new_v4()).unwrap();
    let mut bytes = Cursor::new(Vec::new());
    capture.write_to(&mut bytes).unwrap();
    let package = NativeCapture::read_from(Cursor::new(bytes.into_inner())).unwrap();
    let target = temp.path().join("native-continuity");
    let manager = CourseManager::open(target.clone()).unwrap();
    let course = Uuid::new_v4();
    manager
        .restore_native(course, "Synthetic continuity".into(), &package, false)
        .unwrap();
    let restored = Database::open(&target.join("courses/data").join(course.to_string())).unwrap();
    assert_eq!(
        restored.study_resume_sessions(context(), 0).unwrap(),
        expected
    );
    for history in [false, true] {
        let capture = db
            .capture_portable_with_history(context(), Uuid::new_v4(), false, history)
            .unwrap();
        let mut bytes = Cursor::new(Vec::new());
        capture.write_to(&mut bytes).unwrap();
        let package = PortableCapture::read_from(Cursor::new(bytes.into_inner())).unwrap();
        let target = temp.path().join(format!("portable-continuity-{history}"));
        let manager = CourseManager::open(target.clone()).unwrap();
        let course = Uuid::new_v4();
        manager
            .import_portable(course, "Synthetic continuity".into(), &package)
            .unwrap();
        let restored =
            Database::open(&target.join("courses/data").join(course.to_string())).unwrap();
        let actual = restored.study_resume_sessions(context(), 0).unwrap();
        if history {
            assert_eq!(actual, expected);
            assert_eq!(
                restored
                    .study_read(context(), first.id)
                    .unwrap()
                    .session
                    .draft,
                first.draft
            );
        } else {
            assert!(actual.sessions.is_empty());
        }
    }
}

#[test]
fn progress_raw_budget_counts_feedback_rows_before_deserializing_them() {
    let (_temp, mut db, unit, revision) = setup_sequence(vec![recall(), recall()]);
    let s = start(&mut db, &unit, &revision, StudyMode::Assessment);
    let s = save(
        &mut db,
        &s,
        StudyResponse::Text {
            value: "Answer".into(),
        },
    );
    // This incomplete assessment embargoes judgments. Raw work must be bounded
    // before any hidden/untrusted judgment is decoded, regardless
    // of whether its semantic validation would later succeed.
    let s = db
        .study_apply(
            context(),
            &request(
                &s,
                StudyAction::Submit {
                    self_rating: None,
                    recall_rating: None,
                    self_outcome: None,
                },
            ),
        )
        .unwrap();
    let view = db.study_read(context(), s.id).unwrap();
    assert!(!view.feedback_available);
    let attempt = view.attempts[0].id;
    let data = serde_json::to_string(&json!({"untrusted": "x".repeat(16_000)})).unwrap();
    for _ in 0..1050 {
        db.connection.execute("INSERT INTO study_judgments(id,attempt_id,data,digest) VALUES(?1,?2,?3,'unverified')",rusqlite::params![Uuid::new_v4().to_string(),attempt.to_string(),data]).unwrap();
    }
    assert_eq!(
        db.study_progress(context(), &unit, 0).unwrap_err().code,
        ErrorCode::BudgetExceeded
    );
}
