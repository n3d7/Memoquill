use super::*;

fn submitted(db: &mut Database, unit: &str, revision: &str) -> StudySession {
    let s = start(db, unit, revision, StudyMode::Practice);
    let s = save(
        db,
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
                self_rating: Some(4),
                recall_rating: None,
                self_outcome: Some(AssessmentOutcome::Correct),
            },
        ),
    )
    .unwrap()
}

#[test]
fn progress_empty_scope_and_authorization_are_explicit() {
    let (_temp, db, unit, _) = setup(recall());
    let p = db.study_progress(context(), &unit, 0).unwrap();
    assert_eq!(p.total_attempts, 0);
    assert_eq!(p.coverage[0].current_activity_bindings, 1);
    assert_eq!(p.coverage[0].evidenced_current_activity_bindings, 0);
    assert_eq!(p.next_offset, None);
    assert_eq!(
        db.study_progress(RequestContext::unverified_external(), &unit, 0)
            .unwrap_err()
            .code,
        ErrorCode::PermissionDenied
    );
    assert!(
        db.study_progress(context(), &Uuid::new_v4().to_string(), 0)
            .is_err()
    );
    assert!(db.study_progress(context(), &unit, 1).is_err());
    let (_other, other, _, _) = setup(recall());
    assert!(other.study_progress(context(), &unit, 0).is_err());
}

#[test]
fn progress_paging_keeps_complete_counts_and_does_not_write_history() {
    let (_temp, mut db, unit, revision) = setup(recall());
    let mut ids = Vec::new();
    for _ in 0..35 {
        ids.push(submitted(&mut db, &unit, &revision).id);
    }
    let changes = db.connection.total_changes();
    let first = db.study_progress(context(), &unit, 0).unwrap();
    let second = db
        .study_progress(context(), &unit, first.next_offset.unwrap())
        .unwrap();
    assert_eq!(db.connection.total_changes(), changes);
    assert_eq!(first.total_attempts, 35);
    assert_eq!(first.evidence.len(), 32);
    assert_eq!(second.evidence.len(), 3);
    assert_eq!(second.next_offset, None);
    assert_eq!(first.coverage, second.coverage);
    assert_eq!(first.coverage[0].evidenced_current_activity_bindings, 1);
    let unique: std::collections::HashSet<_> = first
        .evidence
        .iter()
        .chain(&second.evidence)
        .map(|e| e.attempt.id)
        .collect();
    assert_eq!(unique.len(), 35);
    assert_eq!(first.evidence[0].attempt.session_id, ids[34]);
    assert_eq!(db.study_progress(context(), &unit, 0).unwrap(), first);
}

#[test]
fn progress_retains_superseded_judgments_and_unknown_assistance() {
    let (_temp, mut db, unit, revision) = setup(recall());
    let s = submitted(&mut db, &unit, &revision);
    let view = db.study_read(context(), s.id).unwrap();
    let original = view
        .judgments
        .iter()
        .find(|j| j.source == JudgmentSource::SelfReport)
        .unwrap();
    db.study_apply(
        context(),
        &request(
            &s,
            StudyAction::CorrectSelfJudgment {
                attempt_id: view.attempts[0].id,
                supersedes: original.id,
                outcome: AssessmentOutcome::Disputed,
            },
        ),
    )
    .unwrap();
    let p = db.study_progress(context(), &unit, 0).unwrap();
    assert_eq!(p.total_attempts, 1);
    assert_eq!(p.evidence[0].judgments.len(), 3);
    assert_eq!(
        p.evidence[0].attempt.help.external,
        DeclaredAssistance::Unknown
    );
    assert!(
        p.coverage[0]
            .judgments
            .iter()
            .any(|j| j.source == JudgmentSource::SelfReport
                && j.outcome == AssessmentOutcome::Disputed
                && j.count == 1)
    );
    assert!(
        !p.coverage[0]
            .judgments
            .iter()
            .any(|j| j.source == JudgmentSource::SelfReport
                && j.outcome == AssessmentOutcome::Correct)
    );
    assert!(
        p.coverage[0]
            .judgments
            .iter()
            .any(|j| j.source == JudgmentSource::Deterministic
                && j.outcome == AssessmentOutcome::Correct)
    );
}

#[test]
fn progress_recomputes_after_reopen_native_and_opted_in_portable_restore() {
    use std::io::Cursor;
    let (temp, mut db, unit, revision) = setup(recall());
    submitted(&mut db, &unit, &revision);
    let expected = db.study_progress(context(), &unit, 0).unwrap();
    drop(db);
    let mut db = Database::open(&temp.path().join("workspace")).unwrap();
    assert_eq!(db.study_progress(context(), &unit, 0).unwrap(), expected);
    let captured = db.capture_native(context(), Uuid::new_v4()).unwrap();
    let mut bytes = Cursor::new(Vec::new());
    captured.write_to(&mut bytes).unwrap();
    let package = NativeCapture::read_from(Cursor::new(bytes.into_inner())).unwrap();
    let target = temp.path().join("restored-native");
    let manager = CourseManager::open(target.clone()).unwrap();
    let course = Uuid::new_v4();
    manager
        .restore_native(course, "Synthetic progress".into(), &package, false)
        .unwrap();
    let restored = Database::open(&target.join("courses/data").join(course.to_string())).unwrap();
    assert_eq!(
        restored.study_progress(context(), &unit, 0).unwrap(),
        expected
    );
    for history in [true, false] {
        let capture = db
            .capture_portable_with_history(context(), Uuid::new_v4(), false, history)
            .unwrap();
        let mut bytes = Cursor::new(Vec::new());
        capture.write_to(&mut bytes).unwrap();
        let package = PortableCapture::read_from(Cursor::new(bytes.into_inner())).unwrap();
        let target = temp.path().join(format!("portable-{history}"));
        let manager = CourseManager::open(target.clone()).unwrap();
        let course = Uuid::new_v4();
        manager
            .import_portable(course, "Synthetic progress".into(), &package)
            .unwrap();
        let restored =
            Database::open(&target.join("courses/data").join(course.to_string())).unwrap();
        let actual = restored.study_progress(context(), &unit, 0).unwrap();
        if history {
            assert_eq!(actual, expected);
        } else {
            assert_eq!(actual.total_attempts, 0);
            assert_eq!(actual.coverage[0].evidenced_current_activity_bindings, 0);
        }
    }
}

#[test]
fn progress_refuses_truncation_and_corrupt_evidence() {
    let (_temp, mut db, unit, revision) = setup(recall());
    let s = submitted(&mut db, &unit, &revision);
    db.connection
        .execute("UPDATE study_attempts SET digest='bad'", [])
        .unwrap();
    assert_eq!(
        db.study_progress(context(), &unit, 0).unwrap_err().code,
        ErrorCode::RecoveryRequired
    );
    // Synthetic duplicates hit the read budget before any individual record is
    // interpreted; a partial scan must never report complete coverage.
    for _ in 0..1024 {
        db.connection.execute("INSERT INTO study_sessions SELECT ?1,unit_id,composition_id,revision,data,digest FROM study_sessions WHERE id=?2", rusqlite::params![Uuid::new_v4().to_string(), s.id.to_string()]).unwrap();
    }
    assert_eq!(
        db.study_progress(context(), &unit, 0).unwrap_err().code,
        ErrorCode::BudgetExceeded
    );
}

#[test]
fn progress_composition_revision_does_not_invalidate_unchanged_activity_evidence() {
    let (_temp, mut db, unit, revision) = setup(recall());
    submitted(&mut db, &unit, &revision);
    let draft = db
        .lesson_create(
            context(),
            &CreateLessonDraft {
                contract_version: 1,
                workspace_id: db.id.clone(),
                unit_id: unit.clone(),
                operation_id: OperationId::generate(),
                expected_head_revision: Some(revision.clone()),
            },
        )
        .unwrap();
    let preview = db
        .lesson_preview(
            context(),
            &LessonDraftScope {
                contract_version: 1,
                workspace_id: db.id.clone(),
                unit_id: unit.clone(),
                draft_id: draft.id.clone(),
                expected_draft_revision: draft.revision,
            },
        )
        .unwrap();
    let saved = db
        .lesson_apply(
            context(),
            &ApplyLessonDraft {
                contract_version: 1,
                workspace_id: db.id.clone(),
                unit_id: unit.clone(),
                draft_id: draft.id,
                expected_draft_revision: draft.revision,
                expected_head_revision: Some(revision.clone()),
                preview_hash: preview.hash,
                operation_id: OperationId::generate(),
            },
        )
        .unwrap()
        .saved;
    assert_ne!(saved.id, revision);
    let p = db.study_progress(context(), &unit, 0).unwrap();
    assert!(p.evidence[0].changed_composition);
    assert!(p.evidence[0].current_activity);
    assert_eq!(p.coverage[0].evidenced_current_activity_bindings, 1);
    assert_eq!(p.coverage[0].previous_activity_attempts, 0);
}

#[test]
fn progress_legacy_missing_times_and_provenance_remain_unknown() {
    let (_temp, mut db, unit, revision) = setup(recall());
    let s = submitted(&mut db, &unit, &revision);
    let view = db.study_read(context(), s.id).unwrap();
    let mut attempt = serde_json::to_value(&view.attempts[0]).unwrap();
    attempt.as_object_mut().unwrap().remove("submittedAtUnixMs");
    attempt
        .as_object_mut()
        .unwrap()
        .remove("firstSavedAtUnixMs");
    let data = serde_json::to_string(&attempt).unwrap();
    db.connection
        .execute(
            "UPDATE study_attempts SET data=?1,digest=?2 WHERE id=?3",
            rusqlite::params![
                data,
                crate::workspace::migrations::checksum(data.as_bytes()),
                view.attempts[0].id.to_string()
            ],
        )
        .unwrap();
    for j in &view.judgments {
        let mut judgment = serde_json::to_value(j).unwrap();
        for key in ["provenance", "uncertainty", "recordedAtUnixMs"] {
            judgment.as_object_mut().unwrap().remove(key);
        }
        let data = serde_json::to_string(&judgment).unwrap();
        db.connection
            .execute(
                "UPDATE study_judgments SET data=?1,digest=?2 WHERE id=?3",
                rusqlite::params![
                    data,
                    crate::workspace::migrations::checksum(data.as_bytes()),
                    j.id.to_string()
                ],
            )
            .unwrap();
    }
    let p = db.study_progress(context(), &unit, 0).unwrap();
    assert_eq!(p.evidence[0].attempt.submitted_at_unix_ms, None);
    assert!(
        p.evidence[0]
            .judgments
            .iter()
            .all(|j| j.provenance.is_none()
                && j.recorded_at_unix_ms.is_none()
                && j.uncertainty == JudgmentUncertainty::NotEstimated)
    );
}
