use super::validate_native_data;
use super::*;
use crate::workspace::{database::Database, faults, *};
use serde_json::json;
use uuid::Uuid;

fn context() -> RequestContext {
    RequestContext::local_ui()
}
fn setup(definition: serde_json::Value) -> (tempfile::TempDir, Database, String, String) {
    setup_sequence(vec![definition])
}
fn setup_sequence(
    definitions: Vec<serde_json::Value>,
) -> (tempfile::TempDir, Database, String, String) {
    let temp = tempfile::tempdir().unwrap();
    let mut db = Database::open(&temp.path().join("workspace")).unwrap();
    let (unit, revision) = publish_test_lesson(&mut db, definitions);
    (temp, db, unit, revision)
}
fn publish_test_lesson(db: &mut Database, definitions: Vec<serde_json::Value>) -> (String, String) {
    let unit = Uuid::new_v4().to_string();
    db.connection
        .execute("INSERT INTO units VALUES(?1,'Lesson','goal',0)", [&unit])
        .unwrap();
    let draft = db
        .lesson_create(
            context(),
            &CreateLessonDraft {
                contract_version: 1,
                workspace_id: db.id.clone(),
                unit_id: unit.clone(),
                operation_id: OperationId::generate(),
                expected_head_revision: None,
            },
        )
        .unwrap();
    let mut operations = Vec::new();
    for (index, definition) in definitions.into_iter().enumerate() {
        let binding = if index == 0 {
            "practice".into()
        } else {
            format!("practice-{index}")
        };
        let node_id = if index == 0 {
            "exercise".into()
        } else {
            format!("exercise-{index}")
        };
        let node = json!({"nodeId":node_id,"typeId":"activity","typeVersion":1,"props":{"bindingId":binding,"presentation":"default"},"slots":{},"on":{}});
        operations.push(LessonOperation::CreateActivityRevision {
            activity_id: Uuid::new_v4().to_string(),
            expected_revision_id: None,
            binding_id: binding,
            definition,
        });
        operations.push(serde_json::from_value(json!({"kind":"insertSubtree","parent":draft.composition.root_node_id,"slot":"body","index":index,"rootNodeId":node_id,"nodes":[node]})).unwrap());
    }
    let draft = db
        .lesson_edit(
            context(),
            &EditLessonDraft {
                contract_version: 1,
                workspace_id: db.id.clone(),
                unit_id: unit.clone(),
                draft_id: draft.id,
                expected_draft_revision: draft.revision,
                operation_id: OperationId::generate(),
                operations,
            },
        )
        .unwrap();
    let scope = LessonDraftScope {
        contract_version: 1,
        workspace_id: db.id.clone(),
        unit_id: unit.clone(),
        draft_id: draft.id.clone(),
        expected_draft_revision: draft.revision,
    };
    let preview = db.lesson_preview(context(), &scope).unwrap();
    let saved = db
        .lesson_apply(
            context(),
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
        .unwrap()
        .saved;
    (unit, saved.id)
}
fn recall() -> serde_json::Value {
    json!({"family":"recall","version":1,"prompt":"English: recall the word","answer":"Answer","hints":["First letter: A"],"evaluation":"exact"})
}
fn start(db: &mut Database, unit: &str, revision: &str, mode: StudyMode) -> StudySession {
    db.study_apply(
        context(),
        &StudyMutation {
            operation_id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            expected_revision: 0,
            action: StudyAction::Start {
                unit_id: unit.into(),
                composition_revision_id: revision.into(),
                mode,
                phase: StudyPhase::Independent,
                binding_ids: None,
            },
        },
    )
    .unwrap()
}
fn request(s: &StudySession, action: StudyAction) -> StudyMutation {
    StudyMutation {
        operation_id: Uuid::new_v4(),
        session_id: s.id,
        expected_revision: s.revision,
        action,
    }
}
fn save(db: &mut Database, s: &StudySession, response: StudyResponse) -> StudySession {
    db.study_apply(
        context(),
        &request(
            s,
            StudyAction::SaveResponse {
                response,
                external_assistance: DeclaredAssistance::Unknown,
            },
        ),
    )
    .unwrap()
}

#[test]
fn study_acknowledged_draft_and_exactly_once_attempt_survive_reopen() {
    let (temp, mut db, unit, revision) = setup(recall());
    let s = start(&mut db, &unit, &revision, StudyMode::Practice);
    let hint = request(&s, StudyAction::RevealHint);
    let s = db.study_apply(context(), &hint).unwrap();
    let s = save(
        &mut db,
        &s,
        StudyResponse::Text {
            value: "Answer".into(),
        },
    );
    assert_eq!(s.draft.help.hints_revealed, 1);
    drop(db);
    let mut db = Database::open(&temp.path().join("workspace")).unwrap();
    assert_eq!(db.study_read(context(), s.id).unwrap().session, s);
    let submit = request(
        &s,
        StudyAction::Submit {
            self_rating: Some(4),
            self_outcome: Some(AssessmentOutcome::Correct),
            recall_rating: None,
        },
    );
    let committed = db.study_apply(context(), &submit).unwrap();
    assert_eq!(db.study_apply(context(), &submit).unwrap(), committed);
    let mut conflict = submit.clone();
    conflict.action = StudyAction::Submit {
        self_rating: Some(1),
        self_outcome: None,
        recall_rating: None,
    };
    assert_eq!(
        db.study_apply(context(), &conflict).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    let view = db.study_read(context(), s.id).unwrap();
    assert_eq!(view.attempts.len(), 1);
    assert_eq!(view.judgments.len(), 2);
    assert_eq!(view.attempts[0].help.external, DeclaredAssistance::Unknown);
    assert_eq!(view.attempts[0].help.hints_revealed, 1);
    assert_eq!(view.attempts[0].self_rating, Some(4));
    assert_eq!(view.judgments[0].outcome, AssessmentOutcome::Correct);
    validate_native_data(&db.connection).unwrap();
    drop(db);
    let db = Database::open(&temp.path().join("workspace")).unwrap();
    assert_eq!(db.study_read(context(), s.id).unwrap(), view);
}

#[test]
fn study_submit_rolls_back_all_evidence_and_cursor_before_commit() {
    let (_temp, mut db, unit, revision) = setup(recall());
    let s = start(&mut db, &unit, &revision, StudyMode::Practice);
    let s = save(
        &mut db,
        &s,
        StudyResponse::Text {
            value: "Answer".into(),
        },
    );
    let submit = request(
        &s,
        StudyAction::Submit {
            self_rating: None,
            self_outcome: None,
            recall_rating: None,
        },
    );
    let error = faults::using(
        |phase| {
            if phase == faults::Phase::BeforeCommit {
                Err(AppError::new(ErrorCode::StorageFailure))
            } else {
                Ok(())
            }
        },
        || db.study_apply(context(), &submit),
    );
    assert!(error.is_err());
    let view = db.study_read(context(), s.id).unwrap();
    assert_eq!(view.session, s);
    assert!(view.attempts.is_empty());
    assert!(view.judgments.is_empty());
    let s = db.study_apply(context(), &submit).unwrap();
    assert_eq!(s.cursor, 1);
}

#[test]
fn study_uncertain_commit_is_recovered_by_the_same_operation() {
    let (_temp, mut db, unit, revision) = setup(recall());
    let s = start(&mut db, &unit, &revision, StudyMode::Practice);
    let s = save(
        &mut db,
        &s,
        StudyResponse::Text {
            value: "Answer".into(),
        },
    );
    let submit = request(
        &s,
        StudyAction::Submit {
            self_rating: None,
            self_outcome: None,
            recall_rating: None,
        },
    );
    let error = faults::using(
        |phase| {
            if phase == faults::Phase::AfterCommit {
                Err(AppError::new(ErrorCode::StorageFailure))
            } else {
                Ok(())
            }
        },
        || db.study_apply(context(), &submit),
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::OutcomeUnconfirmed);
    let recovered = db.study_apply(context(), &submit).unwrap();
    assert_eq!(recovered.cursor, 1);
    assert_eq!(db.study_read(context(), s.id).unwrap().attempts.len(), 1);
}

#[test]
fn study_four_subject_families_keep_deterministic_and_self_report_separate() {
    let fixtures = [
        (
            recall(),
            StudyResponse::Text {
                value: "Answer".into(),
            },
            AssessmentOutcome::Correct,
        ),
        (
            json!({"family":"singleChoice","version":1,"prompt":"Python: choose a list","options":[{"id":"a","text":"[]"},{"id":"b","text":"{}"}],"correctOptionId":"a","hints":[],"evaluation":"exact"}),
            StudyResponse::Choice {
                option_id: "a".into(),
            },
            AssessmentOutcome::Correct,
        ),
        (
            json!({"family":"constructed","version":1,"prompt":"Math: explain why","rubric":"A valid argument","hints":[],"evaluation":"self"}),
            StudyResponse::Text {
                value: "My argument".into(),
            },
            AssessmentOutcome::Unknown,
        ),
        (
            json!({"family":"procedure","version":1,"prompt":"Cooking: prepare safely","steps":[{"id":"wash","text":"Wash hands"}],"reflection":"Describe observations","hints":[],"evaluation":"self"}),
            StudyResponse::Procedure {
                completed_step_ids: vec!["wash".into()],
                observations: "Done".into(),
            },
            AssessmentOutcome::Unknown,
        ),
    ];
    for (definition, response, expected) in fixtures {
        let (_temp, mut db, unit, revision) = setup(definition);
        let s = start(&mut db, &unit, &revision, StudyMode::Practice);
        let s = save(&mut db, &s, response);
        let s = db
            .study_apply(
                context(),
                &request(
                    &s,
                    StudyAction::Submit {
                        self_rating: Some(4),
                        self_outcome: Some(AssessmentOutcome::Correct),
                        recall_rating: None,
                    },
                ),
            )
            .unwrap();
        let view = db.study_read(context(), s.id).unwrap();
        assert_eq!(view.judgments[0].outcome, expected);
        assert_eq!(view.judgments[0].source, JudgmentSource::Deterministic);
        assert_eq!(view.judgments[1].source, JudgmentSource::SelfReport);
        validate_native_data(&db.connection).unwrap();
    }
}

#[test]
fn study_assessment_refuses_help_and_wrong_response_or_scope() {
    let (_temp, mut db, unit, revision) = setup(recall());
    let s = start(&mut db, &unit, &revision, StudyMode::Assessment);
    for action in [
        StudyAction::RevealHint,
        StudyAction::RevealSolution,
        StudyAction::SaveResponse {
            response: StudyResponse::Choice {
                option_id: "a".into(),
            },
            external_assistance: DeclaredAssistance::None,
        },
    ] {
        assert!(db.study_apply(context(), &request(&s, action)).is_err());
    }
    let wrong = StudyMutation {
        operation_id: Uuid::new_v4(),
        session_id: Uuid::new_v4(),
        expected_revision: 0,
        action: StudyAction::Start {
            unit_id: Uuid::new_v4().to_string(),
            composition_revision_id: revision.clone(),
            mode: StudyMode::Practice,
            phase: StudyPhase::Independent,
            binding_ids: None,
        },
    };
    assert!(db.study_apply(context(), &wrong).is_err());
    assert!(db.study_read(context(), Uuid::new_v4()).is_err());
    assert_eq!(db.study_read(context(), s.id).unwrap().session, s);
}

#[test]
fn study_corrections_append_without_rewriting_response_or_original_judgment() {
    let (_temp, mut db, unit, revision) = setup(recall());
    let s = start(&mut db, &unit, &revision, StudyMode::Practice);
    let s = save(
        &mut db,
        &s,
        StudyResponse::Text {
            value: "Wrong".into(),
        },
    );
    let s = db
        .study_apply(
            context(),
            &request(
                &s,
                StudyAction::Submit {
                    self_rating: Some(4),
                    self_outcome: Some(AssessmentOutcome::Correct),
                    recall_rating: None,
                },
            ),
        )
        .unwrap();
    let before = db.study_read(context(), s.id).unwrap();
    let action = StudyAction::CorrectSelfJudgment {
        attempt_id: before.attempts[0].id,
        supersedes: before.judgments[1].id,
        outcome: AssessmentOutcome::Incorrect,
    };
    let s = db
        .study_apply(context(), &request(&s, action.clone()))
        .unwrap();
    assert!(db.study_apply(context(), &request(&s, action)).is_err());
    let after = db.study_read(context(), s.id).unwrap();
    assert_eq!(before.attempts, after.attempts);
    assert_eq!(before.judgments, after.judgments[..2]);
    assert_eq!(after.judgments.len(), 3);
    assert_eq!(after.judgments[2].supersedes, Some(before.judgments[1].id));
    validate_native_data(&db.connection).unwrap();
}

#[test]
fn study_history_remains_pageable_after_sixty_five_sessions() {
    let (_temp, mut db, unit, revision) = setup(recall());
    let mut ids = std::collections::HashSet::new();
    for _ in 0..65 {
        ids.insert(start(&mut db, &unit, &revision, StudyMode::Practice).id);
    }
    let first = db.study_sessions(context(), &unit, 0).unwrap();
    let second = db.study_sessions(context(), &unit, 64).unwrap();
    assert_eq!(first.len(), 64);
    assert_eq!(second.len(), 1);
    assert_eq!(
        first
            .into_iter()
            .chain(second)
            .map(|s| s.id)
            .collect::<std::collections::HashSet<_>>(),
        ids
    );
    assert!(db.study_sessions(context(), &unit, 128).unwrap().is_empty());
}

#[test]
fn study_native_admission_rejects_attempt_relabeling_even_with_rehashed_rows() {
    let (_temp, mut db, unit, revision) = setup(recall());
    let s = start(&mut db, &unit, &revision, StudyMode::Practice);
    let s = save(
        &mut db,
        &s,
        StudyResponse::Text {
            value: "Answer".into(),
        },
    );
    let s = db
        .study_apply(
            context(),
            &request(
                &s,
                StudyAction::Submit {
                    self_rating: None,
                    self_outcome: None,
                    recall_rating: None,
                },
            ),
        )
        .unwrap();
    let view = db.study_read(context(), s.id).unwrap();
    validate_native_data(&db.connection).unwrap();
    let old_id = view.attempts[0].id;
    let new_id = Uuid::new_v4();
    let mut attempt = view.attempts[0].clone();
    attempt.id = new_id;
    let data = serde_json::to_string(&attempt).unwrap();
    let tx = db.connection.transaction().unwrap();
    tx.execute_batch("PRAGMA defer_foreign_keys=ON").unwrap();
    tx.execute(
        "UPDATE study_attempts SET id=?1,data=?2,digest=?3 WHERE id=?4",
        rusqlite::params![
            new_id.to_string(),
            data,
            crate::workspace::migrations::checksum(data.as_bytes()),
            old_id.to_string()
        ],
    )
    .unwrap();
    for mut judgment in view.judgments {
        judgment.attempt_id = new_id;
        let data = serde_json::to_string(&judgment).unwrap();
        tx.execute(
            "UPDATE study_judgments SET attempt_id=?1,data=?2,digest=?3 WHERE id=?4",
            rusqlite::params![
                new_id.to_string(),
                data,
                crate::workspace::migrations::checksum(data.as_bytes()),
                judgment.id.to_string()
            ],
        )
        .unwrap();
    }
    tx.commit().unwrap();
    assert!(validate_native_data(&db.connection).is_err());
}

#[test]
fn study_schema_seven_upgrade_retains_existing_units_and_initializes_empty_history() {
    let mut connection = rusqlite::Connection::open_in_memory().unwrap();
    connection
        .pragma_update(None, "foreign_keys", true)
        .unwrap();
    let migrations = crate::workspace::migrations::MIGRATIONS;
    crate::workspace::migrations::apply(&mut connection, 0, &migrations[..7]).unwrap();
    let unit = Uuid::new_v4().to_string();
    connection
        .execute(
            "INSERT INTO units VALUES(?1,'Existing lesson','Existing objective',0)",
            [&unit],
        )
        .unwrap();
    crate::workspace::migrations::apply(&mut connection, 7, migrations).unwrap();
    crate::workspace::migrations::validate(&connection, crate::workspace::database::SCHEMA_VERSION)
        .unwrap();
    assert_eq!(
        connection
            .query_row("SELECT title FROM units WHERE id=?1", [unit], |r| r
                .get::<_, String>(0))
            .unwrap(),
        "Existing lesson"
    );
    assert_eq!(
        connection
            .query_row("SELECT count(*) FROM study_sessions", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    validate_native_data(&connection).unwrap();
    assert!(
        connection
            .pragma_query_value(None, "foreign_keys", |r| r.get::<_, bool>(0))
            .unwrap()
    );
}

#[test]
fn study_context_and_provenance_survive_retry_without_rewriting_evidence() {
    let (_temp, mut db, unit, revision) = setup(recall());
    let start_request = StudyMutation {
        operation_id: Uuid::new_v4(),
        session_id: Uuid::new_v4(),
        expected_revision: 0,
        action: StudyAction::Start {
            unit_id: unit,
            composition_revision_id: revision,
            mode: StudyMode::Practice,
            phase: StudyPhase::DelayedCheck,
            binding_ids: None,
        },
    };
    let s = db.study_apply(context(), &start_request).unwrap();
    assert!(s.started_at_unix_ms.is_some());
    let s = save(
        &mut db,
        &s,
        StudyResponse::Text {
            value: "Answer".into(),
        },
    );
    let first_saved = s.draft.first_saved_at_unix_ms;
    assert!(first_saved.is_some());
    let s = save(
        &mut db,
        &s,
        StudyResponse::Text {
            value: "Answer ".into(),
        },
    );
    assert_eq!(s.draft.first_saved_at_unix_ms, first_saved);
    let submit = request(
        &s,
        StudyAction::Submit {
            self_rating: Some(3),
            self_outcome: Some(AssessmentOutcome::Disputed),
            recall_rating: None,
        },
    );
    let s = db.study_apply(context(), &submit).unwrap();
    let before = db.study_read(context(), s.id).unwrap();
    assert_eq!(db.study_apply(context(), &submit).unwrap(), s);
    assert_eq!(db.study_read(context(), s.id).unwrap(), before);
    assert_eq!(before.session.phase, StudyPhase::DelayedCheck);
    assert_eq!(before.attempts[0].first_saved_at_unix_ms, first_saved);
    assert!(before.attempts[0].submitted_at_unix_ms.is_some());
    assert_eq!(before.judgments[0].outcome, AssessmentOutcome::Correct);
    assert_eq!(before.judgments[1].outcome, AssessmentOutcome::Disputed);
    let basis = before.judgments[0].provenance.as_ref().unwrap();
    assert_eq!(
        basis.activity_revision_id,
        before.attempts[0].activity_revision_id
    );
    assert_eq!(
        basis.composition_revision_id,
        before.session.composition_revision_id
    );
    assert_eq!(basis.assessor, AssessorIdentity::Local);
    assert_eq!(
        before.judgments[1].provenance.as_ref().unwrap().assessor,
        AssessorIdentity::Learner
    );
    validate_native_data(&db.connection).unwrap();
}

#[test]
fn study_legacy_records_keep_unknown_times_and_provenance_on_recovery() {
    let (_temp, mut db, unit, revision) = setup(recall());
    let s = start(&mut db, &unit, &revision, StudyMode::Practice);
    let s = save(
        &mut db,
        &s,
        StudyResponse::Text {
            value: "Answer".into(),
        },
    );
    let s = db
        .study_apply(
            context(),
            &request(
                &s,
                StudyAction::Submit {
                    self_rating: None,
                    self_outcome: Some(AssessmentOutcome::Unknown),
                    recall_rating: None,
                },
            ),
        )
        .unwrap();
    for (table, column) in [
        ("study_sessions", "data"),
        ("study_operations", "result"),
        ("study_attempts", "data"),
        ("study_judgments", "data"),
    ] {
        let mut stmt = db
            .connection
            .prepare(&format!("SELECT id,{column} FROM {table}"))
            .unwrap();
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap();
        drop(stmt);
        for (id, data) in rows {
            let mut value: serde_json::Value = serde_json::from_str(&data).unwrap();
            let object = value.as_object_mut().unwrap();
            for key in [
                "phase",
                "startedAtUnixMs",
                "firstSavedAtUnixMs",
                "submittedAtUnixMs",
                "provenance",
                "uncertainty",
                "recordedAtUnixMs",
            ] {
                object.remove(key);
            }
            if let Some(draft) = object.get_mut("draft").and_then(|v| v.as_object_mut()) {
                draft.remove("firstSavedAtUnixMs");
            }
            let data = serde_json::to_string(&value).unwrap();
            db.connection
                .execute(
                    &format!("UPDATE {table} SET {column}=?1,digest=?2 WHERE id=?3"),
                    rusqlite::params![
                        data,
                        super::super::migrations::checksum(data.as_bytes()),
                        id
                    ],
                )
                .unwrap();
        }
    }
    let view = db.study_read(context(), s.id).unwrap();
    assert_eq!(view.session.started_at_unix_ms, None);
    assert_eq!(view.attempts[0].submitted_at_unix_ms, None);
    assert!(
        view.judgments
            .iter()
            .all(|j| j.provenance.is_none() && j.uncertainty == JudgmentUncertainty::NotEstimated)
    );
    validate_native_data(&db.connection).unwrap();
}

#[test]
fn study_admission_rejects_rehashed_assessor_or_content_provenance_changes() {
    let (_temp, mut db, unit, revision) = setup(recall());
    let s = start(&mut db, &unit, &revision, StudyMode::Practice);
    let s = save(
        &mut db,
        &s,
        StudyResponse::Text {
            value: "Answer".into(),
        },
    );
    let s = db
        .study_apply(
            context(),
            &request(
                &s,
                StudyAction::Submit {
                    self_rating: None,
                    self_outcome: None,
                    recall_rating: None,
                },
            ),
        )
        .unwrap();
    let original = db.study_read(context(), s.id).unwrap().judgments[0].clone();
    for assessor in [false, true] {
        let mut changed = original.clone();
        let basis = changed.provenance.as_mut().unwrap();
        if assessor {
            basis.assessor = AssessorIdentity::Learner;
        } else {
            basis.definition_sha256 = "0".repeat(64);
        }
        let data = serde_json::to_string(&changed).unwrap();
        db.connection
            .execute(
                "UPDATE study_judgments SET data=?1,digest=?2 WHERE id=?3",
                rusqlite::params![
                    data,
                    super::super::migrations::checksum(data.as_bytes()),
                    changed.id.to_string()
                ],
            )
            .unwrap();
        assert!(validate_native_data(&db.connection).is_err());
    }
}

#[test]
fn study_native_package_restores_frozen_responses_judgments_and_receipts() {
    use crate::workspace::{CourseManager, NativeCapture};
    use std::io::Cursor;
    let (temp, mut db, unit, revision) = setup(recall());
    let s = start(&mut db, &unit, &revision, StudyMode::Practice);
    let s = save(
        &mut db,
        &s,
        StudyResponse::Text {
            value: "Answer".into(),
        },
    );
    let submit = request(
        &s,
        StudyAction::Submit {
            self_rating: Some(3),
            self_outcome: Some(AssessmentOutcome::Disputed),
            recall_rating: None,
        },
    );
    let s = db.study_apply(context(), &submit).unwrap();
    let view = db.study_read(context(), s.id).unwrap();
    let captured = db.capture_native(context(), Uuid::new_v4()).unwrap();
    let mut bytes = Cursor::new(Vec::new());
    captured.write_to(&mut bytes).unwrap();
    let verified = NativeCapture::read_from(Cursor::new(bytes.into_inner())).unwrap();
    let manager = CourseManager::open(temp.path().join("target")).unwrap();
    let course = Uuid::new_v4();
    manager
        .restore_native(course, "Study recovery".into(), &verified, false)
        .unwrap();
    let mut restored = Database::open(
        &temp
            .path()
            .join("target/courses/data")
            .join(course.to_string()),
    )
    .unwrap();
    assert_eq!(restored.study_read(context(), s.id).unwrap(), view);
    assert_eq!(restored.study_apply(context(), &submit).unwrap(), s);
    assert_eq!(
        restored.study_read(context(), s.id).unwrap().attempts.len(),
        1
    );
}

#[test]
fn study_resaving_a_legacy_draft_keeps_first_save_time_unknown() {
    let (_temp, mut db, unit, revision) = setup(recall());
    let s = start(&mut db, &unit, &revision, StudyMode::Practice);
    let mut s = save(
        &mut db,
        &s,
        StudyResponse::Text {
            value: "Earlier response".into(),
        },
    );
    s.draft.first_saved_at_unix_ms = None;
    let data = serde_json::to_string(&s).unwrap();
    db.connection
        .execute(
            "UPDATE study_sessions SET data=?1,digest=?2 WHERE id=?3",
            rusqlite::params![
                data,
                super::super::migrations::checksum(data.as_bytes()),
                s.id.to_string()
            ],
        )
        .unwrap();
    let s = save(
        &mut db,
        &s,
        StudyResponse::Text {
            value: "Edited later".into(),
        },
    );
    assert_eq!(s.draft.first_saved_at_unix_ms, None);
    let s = db
        .study_apply(
            context(),
            &request(
                &s,
                StudyAction::Submit {
                    self_rating: None,
                    self_outcome: None,
                    recall_rating: None,
                },
            ),
        )
        .unwrap();
    assert_eq!(
        db.study_read(context(), s.id).unwrap().attempts[0].first_saved_at_unix_ms,
        None
    );
}

#[test]
fn study_portable_history_requires_opt_in_and_roundtrips_into_a_new_course() {
    use crate::workspace::{CourseManager, PortableCapture};
    use std::io::{Cursor, Read};
    let (temp, mut db, unit, revision) = setup(recall());
    let s = start(&mut db, &unit, &revision, StudyMode::Practice);
    let s = save(
        &mut db,
        &s,
        StudyResponse::Text {
            value: "Private study response".into(),
        },
    );
    let submit = request(
        &s,
        StudyAction::Submit {
            self_rating: Some(3),
            self_outcome: Some(AssessmentOutcome::Disputed),
            recall_rating: None,
        },
    );
    let s = db.study_apply(context(), &submit).unwrap();
    let view = db.study_read(context(), s.id).unwrap();
    for history in [false, true] {
        let captured = db
            .capture_portable_with_history(context(), Uuid::new_v4(), false, history)
            .unwrap();
        assert_eq!(
            captured.manifest().sections.contains_key("studyHistory"),
            history
        );
        let mut bytes = Cursor::new(Vec::new());
        captured.write_to(&mut bytes).unwrap();
        let bytes = bytes.into_inner();
        let mut zip = zip::ZipArchive::new(Cursor::new(&bytes)).unwrap();
        let mut records = String::new();
        zip.by_name("content.ndjson")
            .unwrap()
            .read_to_string(&mut records)
            .unwrap();
        assert_eq!(records.contains("Private study response"), history);
        let verified = PortableCapture::read_from(Cursor::new(bytes)).unwrap();
        let root = temp.path().join(format!("portable-{history}"));
        let manager = CourseManager::open(root.clone()).unwrap();
        let course = Uuid::new_v4();
        manager
            .import_portable(course, "Portable study".into(), &verified)
            .unwrap();
        let mut restored =
            Database::open(&root.join("courses/data").join(course.to_string())).unwrap();
        if history {
            assert_eq!(restored.study_read(context(), s.id).unwrap(), view);
            assert_eq!(restored.study_apply(context(), &submit).unwrap(), s);
        } else {
            assert!(
                restored
                    .study_sessions(context(), &unit, 0)
                    .unwrap()
                    .is_empty()
            );
        }
    }
}

#[test]
fn study_bounded_cloze_retains_v2_definition_and_checks_only_the_missing_answer() {
    let definition = json!({"family":"recall","version":2,"responseFormat":"boundedCloze","prompt":"The word is {{answer}}.","answer":"Answer","hints":[],"evaluation":"exact"});
    let (_temp, mut db, unit, revision) = setup(definition.clone());
    let s = start(&mut db, &unit, &revision, StudyMode::Practice);
    assert_eq!(s.items[0].activity.definition, definition);
    let oversized = request(
        &s,
        StudyAction::SaveResponse {
            response: StudyResponse::Text {
                value: "a".repeat(1025),
            },
            external_assistance: DeclaredAssistance::None,
        },
    );
    assert!(db.study_apply(context(), &oversized).is_err());
    let s = save(
        &mut db,
        &s,
        StudyResponse::Text {
            value: "Answer".into(),
        },
    );
    let s = db
        .study_apply(
            context(),
            &request(
                &s,
                StudyAction::Submit {
                    self_rating: None,
                    self_outcome: None,
                    recall_rating: None,
                },
            ),
        )
        .unwrap();
    assert_eq!(
        db.study_read(context(), s.id).unwrap().judgments[0].outcome,
        AssessmentOutcome::Correct
    );
    validate_native_data(&db.connection).unwrap();
}

#[test]
fn study_instructional_phases_pin_material_and_never_become_unassisted_assessment() {
    for phase in ["workedExample", "completion"] {
        let definition = json!({"family":"constructed","version":2,"prompt":"Explain or complete the missing reasoning","rubric":"Check the justification","hints":[],"evaluation":"self","instruction":{"phase":phase,"material":"A sound solution or partial derivation"}});
        let (_temp, mut db, unit, revision) = setup(definition.clone());
        let s = start(&mut db, &unit, &revision, StudyMode::Practice);
        assert!(s.draft.help.solution_revealed);
        assert_eq!(s.items[0].activity.definition, definition);
        let s = save(
            &mut db,
            &s,
            StudyResponse::Text {
                value: "My explanation / missing step".into(),
            },
        );
        let s = db
            .study_apply(
                context(),
                &request(
                    &s,
                    StudyAction::Submit {
                        self_rating: Some(4),
                        self_outcome: Some(AssessmentOutcome::Correct),
                        recall_rating: None,
                    },
                ),
            )
            .unwrap();
        assert_eq!(s.draft, StudyDraft::default());
        let view = db.study_read(context(), s.id).unwrap();
        assert!(view.attempts[0].help.solution_revealed);
        assert_eq!(view.judgments[0].outcome, AssessmentOutcome::Unknown);
        validate_native_data(&db.connection).unwrap();
        let assessment = StudyMutation {
            operation_id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            expected_revision: 0,
            action: StudyAction::Start {
                unit_id: unit,
                composition_revision_id: revision,
                mode: StudyMode::Assessment,
                phase: StudyPhase::Independent,
                binding_ids: None,
            },
        };
        assert_eq!(
            db.study_apply(context(), &assessment).unwrap_err().code,
            ErrorCode::InvalidRequest
        );
    }
}

#[test]
fn error_repair_pins_review_and_keeps_correction_hidden_until_requested() {
    let definition = json!({"family":"constructed","version":3,"prompt":"Find the error","rubric":"Explain the repair","hints":[],"evaluation":"self","instruction":{"phase":"errorRepair","material":"2 + 2 = 5","label":"Deliberate error","expectedCorrection":"2 + 2 = 4","review":{"reviewer":"Fixture author","reference":"Addition reference"}}});
    let (_temp, mut db, unit, revision) = setup(definition.clone());
    let s = start(&mut db, &unit, &revision, StudyMode::Practice);
    assert_eq!(s.items[0].activity.definition, definition);
    assert!(!s.draft.help.solution_revealed);
    let s = save(
        &mut db,
        &s,
        StudyResponse::Text {
            value: "Replace 5 with 4".into(),
        },
    );
    let s = db
        .study_apply(context(), &request(&s, StudyAction::RevealSolution))
        .unwrap();
    assert!(s.draft.help.solution_revealed);
    validate_native_data(&db.connection).unwrap();
    let reopened = db.study_read(context(), s.id).unwrap();
    assert_eq!(reopened.session.items[0].activity.definition, definition);
    let assessment = StudyMutation {
        operation_id: Uuid::new_v4(),
        session_id: Uuid::new_v4(),
        expected_revision: 0,
        action: StudyAction::Start {
            unit_id: unit,
            composition_revision_id: revision,
            mode: StudyMode::Assessment,
            phase: StudyPhase::Independent,
            binding_ids: None,
        },
    };
    assert_eq!(
        db.study_apply(context(), &assessment).unwrap_err().code,
        ErrorCode::InvalidRequest
    );
}

#[test]
#[ignore = "private child process killed by study_process_crash_preserves_acknowledged_draft_and_exactly_once_submit"]
fn study_crash_child() {
    let root = std::path::PathBuf::from(std::env::var_os("MEMOQUILL_STUDY_CRASH_ROOT").unwrap());
    let mut db = Database::open(&root.join("workspace")).unwrap();
    let request: StudyMutation =
        serde_json::from_slice(&std::fs::read(root.join("submit.json")).unwrap()).unwrap();
    let committed = std::env::var_os("MEMOQUILL_STUDY_CRASH_COMMITTED").is_some();
    faults::using(
        move |phase| {
            if phase
                == if committed {
                    faults::Phase::AfterCommit
                } else {
                    faults::Phase::BeforeCommit
                }
            {
                std::fs::write(root.join("ready"), b"ready").unwrap();
                loop {
                    std::thread::park_timeout(std::time::Duration::from_secs(1));
                }
            }
            Ok(())
        },
        || db.study_apply(context(), &request),
    )
    .unwrap();
}

#[test]
fn study_process_crash_preserves_acknowledged_draft_and_exactly_once_submit() {
    check_study_process_crash(false);
}
fn check_study_process_crash(scheduled: bool) {
    for committed in [false, true] {
        let (temp, mut db, unit, revision) = setup(recall());
        let s = start(&mut db, &unit, &revision, StudyMode::Practice);
        let s = if scheduled {
            review_save(&mut db, &s, "Answer")
        } else {
            save(
                &mut db,
                &s,
                StudyResponse::Text {
                    value: "Answer".into(),
                },
            )
        };
        let submit = request(
            &s,
            StudyAction::Submit {
                self_rating: Some(3),
                self_outcome: Some(AssessmentOutcome::Correct),
                recall_rating: scheduled.then_some(3),
            },
        );
        std::fs::write(
            temp.path().join("submit.json"),
            serde_json::to_vec(&submit).unwrap(),
        )
        .unwrap();
        drop(db);
        let mut command = std::process::Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "workspace::study::tests::study_crash_child",
                "--ignored",
            ])
            .env("MEMOQUILL_STUDY_CRASH_ROOT", temp.path())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        if committed {
            command.env("MEMOQUILL_STUDY_CRASH_COMMITTED", "1");
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
        assert!(ready, "owned child did not reach commit boundary");
        let mut db = Database::open(&temp.path().join("workspace")).unwrap();
        let view = db.study_read(context(), s.id).unwrap();
        assert_eq!(view.attempts.len(), usize::from(committed));
        assert_eq!(
            db.connection
                .query_row("SELECT count(*) FROM recall_review_events", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            i64::from(scheduled && committed)
        );
        if !committed {
            assert_eq!(view.session, s);
        }
        let finished = db.study_apply(context(), &submit).unwrap();
        assert_eq!(finished.cursor, 1);
        assert_eq!(db.study_apply(context(), &submit).unwrap(), finished);
        assert_eq!(db.study_read(context(), s.id).unwrap().attempts.len(), 1);
        validate_native_data(&db.connection).unwrap();
        super::super::review::validate_native_data(&db.connection).unwrap();
        assert_eq!(
            db.connection
                .query_row("SELECT count(*) FROM recall_review_events", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            i64::from(scheduled)
        );
    }
}

#[test]
fn study_assessment_and_review_native_ipc_preserve_course_and_window_scope() {
    use std::io::Cursor;
    use tauri::test::{INVOKE_KEY, get_ipc_response, mock_builder};
    use tauri::{
        ipc::{CallbackFn, InvokeBody},
        webview::InvokeRequest,
    };
    let (temp, mut db, unit, revision) = setup(recall());
    let (subject, _) = scoped_assessment_tests::scope(&mut db, std::slice::from_ref(&unit));
    let s = start(&mut db, &unit, &revision, StudyMode::Practice);
    let capture = db
        .capture_portable_with_history(context(), Uuid::new_v4(), false, true)
        .unwrap();
    let mut bytes = Cursor::new(Vec::new());
    capture.write_to(&mut bytes).unwrap();
    let package = PortableCapture::read_from(Cursor::new(bytes.into_inner())).unwrap();
    let manager = CourseManager::open(temp.path().join("courses-app")).unwrap();
    let first = Uuid::new_v4();
    let second = Uuid::new_v4();
    manager
        .import_portable(first, "First synthetic course".into(), &package)
        .unwrap();
    manager
        .import_portable(second, "Second synthetic course".into(), &package)
        .unwrap();
    let services = [
        manager.service(first).unwrap(),
        manager.service(second).unwrap(),
    ];
    let app = mock_builder()
        .manage(manager)
        .invoke_handler(tauri::generate_handler![
            crate::review_commands::review_overview,
            crate::review_commands::review_apply,
            crate::assessment_commands::assessment_create,
            crate::assessment_commands::assessment_read,
            crate::assessment_commands::assessment_list,
            crate::study_commands::study_read,
            crate::study_commands::study_apply,
            crate::study_commands::study_sessions
        ])
        .build(tauri::generate_context!())
        .unwrap();
    let main = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    let foreign = tauri::WebviewWindowBuilder::new(&app, "foreign", Default::default())
        .build()
        .unwrap();
    let call = |window: &tauri::WebviewWindow<tauri::test::MockRuntime>,
                command: &str,
                url: &str,
                body: serde_json::Value| {
        get_ipc_response(
            window,
            InvokeRequest {
                cmd: command.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: url.parse().unwrap(),
                body: InvokeBody::Raw(serde_json::to_vec(&body).unwrap()),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.into(),
            },
        )
        .map(|v| v.deserialize::<serde_json::Value>().unwrap())
    };
    let read = json!({"courseId":first,"sessionId":s.id});
    assert!(call(&foreign, "study_read", "tauri://localhost", read.clone()).is_err());
    assert!(
        call(
            &main,
            "study_read",
            "https://untrusted.example",
            read.clone()
        )
        .is_err()
    );
    let save = request(
        &s,
        StudyAction::SaveResponse {
            response: StudyResponse::Text {
                value: "Only first course".into(),
            },
            external_assistance: DeclaredAssistance::None,
        },
    );
    let body = json!({"courseId":first,"request":save});
    let result = call(&main, "study_apply", "tauri://localhost", body.clone()).unwrap();
    assert_eq!(
        call(&main, "study_apply", "tauri://localhost", body).unwrap(),
        result
    );
    let first_view = call(&main, "study_read", "tauri://localhost", read).unwrap();
    assert_eq!(
        first_view["session"]["draft"]["response"]["value"],
        "Only first course"
    );
    let second_view = call(
        &main,
        "study_read",
        "tauri://localhost",
        json!({"courseId":second,"sessionId":s.id}),
    )
    .unwrap();
    assert!(second_view["session"]["draft"]["response"].is_null());
    let mut tampered = json!({"courseId":first,"sessionId":s.id});
    tampered["workspacePath"] = json!(temp.path());
    assert!(call(&main, "study_read", "tauri://localhost", tampered).is_err());
    assert!(
        call(
            &main,
            "study_read",
            "tauri://localhost",
            json!({"courseId":Uuid::new_v4(),"sessionId":s.id})
        )
        .is_err()
    );
    let review_scope = json!({"courseId":first});
    assert!(
        call(
            &foreign,
            "review_overview",
            "tauri://localhost",
            review_scope.clone()
        )
        .is_err()
    );
    assert!(
        call(
            &main,
            "review_overview",
            "https://untrusted.example",
            review_scope.clone()
        )
        .is_err()
    );
    let policy = ReviewMutation {
        operation_id: Uuid::new_v4(),
        expected_revision: 0,
        action: ReviewAction::SetPolicy {
            time_zone: "Etc/UTC".into(),
            day_start_hour: 4,
            daily_new_limit: 0,
            daily_review_limit: 50,
            retention_basis_points: 9000,
        },
    };
    call(
        &main,
        "review_apply",
        "tauri://localhost",
        json!({"courseId":first,"request":policy}),
    )
    .unwrap();
    assert_eq!(
        call(
            &main,
            "review_overview",
            "tauri://localhost",
            json!({"courseId":second})
        )
        .unwrap()["plan"]["dailyNewLimit"],
        10
    );
    let request = CreateAssessment {
        operation_id: Uuid::new_v4(),
        assessment_id: Uuid::new_v4(),
        scope: AssessmentScope::Subject { id: subject },
    };
    let body = json!({"courseId":first,"request":request});
    assert!(
        call(
            &foreign,
            "assessment_create",
            "tauri://localhost",
            body.clone()
        )
        .is_err()
    );
    assert!(
        call(
            &main,
            "assessment_create",
            "https://untrusted.example",
            body.clone()
        )
        .is_err()
    );
    let frozen = call(&main, "assessment_create", "tauri://localhost", body).unwrap();
    assert_eq!(frozen["parts"].as_array().unwrap().len(), 1);
    assert!(
        call(
            &main,
            "assessment_read",
            "tauri://localhost",
            json!({"courseId":second,"assessmentId":request.assessment_id})
        )
        .is_err()
    );
    assert_eq!(
        call(
            &main,
            "assessment_list",
            "tauri://localhost",
            json!({"courseId":first})
        )
        .unwrap()
        .as_array()
        .unwrap()
        .len(),
        1
    );
    for service in services {
        tauri::async_runtime::block_on(service.shutdown()).unwrap();
    }
}

#[test]
fn study_native_admission_rejects_future_instructional_item_in_assessment() {
    let (_temp, mut db, unit, revision) = setup(recall());
    let s = start(&mut db, &unit, &revision, StudyMode::Assessment);
    let mut tampered = s.clone();
    let mut item = s.items[0].clone();
    item.binding_id = "future-scaffold".into();
    item.activity.definition = json!({"family":"constructed","version":2,"prompt":"Complete","rubric":"Justify","hints":[],"evaluation":"self","instruction":{"phase":"completion","material":"Partial solution"}});
    tampered.items.push(item);
    let data = serde_json::to_string(&tampered).unwrap();
    db.connection
        .execute(
            "UPDATE study_sessions SET data=?1,digest=?2 WHERE id=?3",
            rusqlite::params![
                data,
                crate::workspace::migrations::checksum(data.as_bytes()),
                s.id.to_string()
            ],
        )
        .unwrap();
    assert_eq!(
        db.study_read(context(), s.id).unwrap_err().code,
        ErrorCode::RecoveryRequired
    );
    assert_eq!(
        validate_native_data(&db.connection).unwrap_err().code,
        ErrorCode::RecoveryRequired
    );
}

#[test]
fn study_instruction_sequence_resumes_scaffold_then_fades_to_independent_response() {
    let instruction = |phase| json!({"family":"constructed","version":2,"prompt":"Explain or complete","rubric":"Justify reasoning","hints":[],"evaluation":"self","instruction":{"phase":phase,"material":"Worked or partial solution"}});
    let (temp, mut db, unit, revision) = setup_sequence(vec![
        instruction("workedExample"),
        instruction("completion"),
        recall(),
    ]);
    let mut s = start(&mut db, &unit, &revision, StudyMode::Practice);
    assert_eq!(s.items.len(), 3);
    for index in 0..3 {
        assert_eq!(s.cursor, index);
        assert_eq!(s.draft.help.solution_revealed, index < 2);
        s = save(
            &mut db,
            &s,
            StudyResponse::Text {
                value: "Answer".into(),
            },
        );
        s = db
            .study_apply(
                context(),
                &request(
                    &s,
                    StudyAction::Submit {
                        self_rating: None,
                        self_outcome: None,
                        recall_rating: None,
                    },
                ),
            )
            .unwrap();
        drop(db);
        db = Database::open(&temp.path().join("workspace")).unwrap();
        assert_eq!(db.study_read(context(), s.id).unwrap().session, s);
    }
    let view = db.study_read(context(), s.id).unwrap();
    assert_eq!(
        view.attempts
            .iter()
            .map(|a| a.help.solution_revealed)
            .collect::<Vec<_>>(),
        vec![true, true, false]
    );
    assert_eq!(
        view.judgments.iter().map(|j| j.outcome).collect::<Vec<_>>(),
        vec![
            AssessmentOutcome::Unknown,
            AssessmentOutcome::Unknown,
            AssessmentOutcome::Correct
        ]
    );
    validate_native_data(&db.connection).unwrap();
}

fn review_save(db: &mut Database, s: &StudySession, value: &str) -> StudySession {
    db.study_apply(
        context(),
        &request(
            s,
            StudyAction::SaveResponse {
                response: StudyResponse::Text {
                    value: value.into(),
                },
                external_assistance: DeclaredAssistance::None,
            },
        ),
    )
    .unwrap()
}
fn scheduled_submit(s: &StudySession, rating: u8) -> StudyMutation {
    request(
        s,
        StudyAction::Submit {
            self_rating: Some(1),
            recall_rating: Some(rating),
            self_outcome: Some(AssessmentOutcome::Unknown),
        },
    )
}
#[test]
fn study_review_recall_event_state_attempt_cursor_and_receipt_commit_atomically() {
    let (_temp, mut db, unit, revision) = setup(recall());
    let s = start(&mut db, &unit, &revision, StudyMode::Practice);
    let s = review_save(&mut db, &s, "Answer");
    let submit = scheduled_submit(&s, 3);
    let error = faults::using(
        |phase| {
            if phase == faults::Phase::BeforeCommit {
                Err(AppError::new(ErrorCode::StorageFailure))
            } else {
                Ok(())
            }
        },
        || db.study_apply(context(), &submit),
    );
    assert!(error.is_err());
    assert_eq!(db.study_read(context(), s.id).unwrap().session, s);
    for table in ["study_attempts", "recall_review_events", "recall_states"] {
        assert_eq!(
            db.connection
                .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
    let finished = db.study_apply(context(), &submit).unwrap();
    assert_eq!(db.study_apply(context(), &submit).unwrap(), finished);
    let view = db.study_read(context(), s.id).unwrap();
    assert_eq!(view.attempts[0].self_rating, Some(1));
    assert_eq!(view.attempts[0].recall_rating, Some(3));
    let mut conflicting = submit.clone();
    if let StudyAction::Submit { recall_rating, .. } = &mut conflicting.action {
        *recall_rating = Some(4);
    }
    assert_eq!(
        db.study_apply(context(), &conflicting).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        db.connection
            .query_row("SELECT count(*) FROM recall_review_events", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    super::super::review::validate_native_data(&db.connection).unwrap();
    let now = view.attempts[0].submitted_at_unix_ms.unwrap();
    assert_eq!(db.review_overview_at(context(), now).unwrap().new_total, 0);
    let overdue = db
        .review_overview_at(context(), now + 6 * 86_400_000)
        .unwrap();
    assert_eq!(overdue.due_total, 1);
    assert_eq!(overdue.offered.len(), 1);
    let rollback = db.review_overview_at(context(), now - 1).unwrap();
    assert!(rollback.clock_rollback);
    assert!(rollback.offered.is_empty());
}
#[test]
fn study_review_rejects_assisted_non_independent_recall_and_recognition() {
    for help in ["hint", "solution", "external", "guided", "assessment"] {
        let (_temp, mut db, unit, revision) = setup(recall());
        let mut s = start(
            &mut db,
            &unit,
            &revision,
            if help == "assessment" {
                StudyMode::Assessment
            } else {
                StudyMode::Practice
            },
        );
        if help == "guided" {
            let req = StudyMutation {
                operation_id: Uuid::new_v4(),
                session_id: Uuid::new_v4(),
                expected_revision: 0,
                action: StudyAction::Start {
                    unit_id: unit,
                    composition_revision_id: revision,
                    mode: StudyMode::Practice,
                    phase: StudyPhase::Guided,
                    binding_ids: None,
                },
            };
            s = db.study_apply(context(), &req).unwrap();
        }
        s = review_save(&mut db, &s, "Answer");
        if help == "hint" {
            s = db
                .study_apply(context(), &request(&s, StudyAction::RevealHint))
                .unwrap();
        }
        if help == "solution" {
            s = db
                .study_apply(context(), &request(&s, StudyAction::RevealSolution))
                .unwrap();
        }
        if help == "external" {
            s = db
                .study_apply(
                    context(),
                    &request(
                        &s,
                        StudyAction::SaveResponse {
                            response: StudyResponse::Text {
                                value: "Answer".into(),
                            },
                            external_assistance: DeclaredAssistance::Present,
                        },
                    ),
                )
                .unwrap();
        }
        assert_eq!(
            db.study_apply(context(), &scheduled_submit(&s, 3))
                .unwrap_err()
                .code,
            ErrorCode::InvalidRequest
        );
        assert!(db.study_read(context(), s.id).unwrap().attempts.is_empty());
    }
    let (_temp, mut db, unit, revision) = setup(
        json!({"family":"singleChoice","version":1,"prompt":"Recognize","options":[{"id":"a","text":"A"},{"id":"b","text":"B"}],"correctOptionId":"a","hints":[],"evaluation":"exact"}),
    );
    let s = start(&mut db, &unit, &revision, StudyMode::Practice);
    let s = db
        .study_apply(
            context(),
            &request(
                &s,
                StudyAction::SaveResponse {
                    response: StudyResponse::Choice {
                        option_id: "a".into(),
                    },
                    external_assistance: DeclaredAssistance::None,
                },
            ),
        )
        .unwrap();
    assert_eq!(
        db.study_apply(context(), &scheduled_submit(&s, 3))
            .unwrap_err()
            .code,
        ErrorCode::InvalidRequest
    );
}
#[test]
fn study_review_policy_generations_and_budget_preserve_existing_history() {
    let (_temp, mut db, unit, revision) = setup(recall());
    let now = db.review_overview(context()).unwrap().now_unix_ms;
    let policy = |expected, limit, zone: &str| ReviewMutation {
        operation_id: Uuid::new_v4(),
        expected_revision: expected,
        action: ReviewAction::SetPolicy {
            time_zone: zone.into(),
            day_start_hour: 4,
            daily_new_limit: limit,
            daily_review_limit: 50,
            retention_basis_points: 9000,
        },
    };
    let request = policy(0, 0, "Etc/UTC");
    let plan = db.review_apply_at(context(), &request, now).unwrap();
    assert_eq!(plan.generation, 1);
    assert_eq!(db.review_apply_at(context(), &request, now).unwrap(), plan);
    let s = start(&mut db, &unit, &revision, StudyMode::Practice);
    let s = review_save(&mut db, &s, "Answer");
    assert_eq!(
        db.study_apply(context(), &scheduled_submit(&s, 3))
            .unwrap_err()
            .code,
        ErrorCode::ReviewBudgetExceeded
    );
    assert!(db.study_read(context(), s.id).unwrap().attempts.is_empty());
    db.review_apply_at(context(), &policy(1, 10, "Etc/UTC"), now)
        .unwrap();
    db.study_apply(context(), &scheduled_submit(&s, 3)).unwrap();
    let before: String = db
        .connection
        .query_row("SELECT data FROM recall_review_events", [], |r| r.get(0))
        .unwrap();
    let changed = db
        .review_apply_at(context(), &policy(2, 10, "Europe/Kyiv"), now)
        .unwrap();
    assert_eq!(changed.generation, 2);
    assert_eq!(
        db.connection
            .query_row("SELECT data FROM recall_review_events", [], |r| r
                .get::<_, String>(0))
            .unwrap(),
        before
    );
    super::super::review::validate_native_data(&db.connection).unwrap();
}
#[test]
fn study_review_history_is_opt_in_and_native_admission_recomputes_states() {
    use std::io::Cursor;
    let (temp, mut db, unit, revision) = setup(recall());
    let s = start(&mut db, &unit, &revision, StudyMode::Practice);
    let s = review_save(&mut db, &s, "Answer");
    db.study_apply(context(), &scheduled_submit(&s, 3)).unwrap();
    for history in [false, true] {
        let capture = db
            .capture_portable_with_history(context(), Uuid::new_v4(), false, history)
            .unwrap();
        assert_eq!(
            capture.manifest().sections.get("studyHistory"),
            history.then_some(&2)
        );
        let mut bytes = Cursor::new(Vec::new());
        capture.write_to(&mut bytes).unwrap();
        let package = PortableCapture::read_from(Cursor::new(bytes.into_inner())).unwrap();
        let root = temp.path().join(format!("review-{history}"));
        let manager = CourseManager::open(root.clone()).unwrap();
        let course = Uuid::new_v4();
        manager
            .import_portable(course, "Synthetic review".into(), &package)
            .unwrap();
        let restored = Database::open(&root.join("courses/data").join(course.to_string())).unwrap();
        assert_eq!(
            restored
                .connection
                .query_row("SELECT count(*) FROM recall_review_events", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            i64::from(history)
        );
        super::super::review::validate_native_data(&restored.connection).unwrap();
    }
    let mut event: serde_json::Value = serde_json::from_str(
        &db.connection
            .query_row("SELECT data FROM recall_review_events", [], |r| {
                r.get::<_, String>(0)
            })
            .unwrap(),
    )
    .unwrap();
    event["memory"]["stability"] = json!(99.0);
    let data = serde_json::to_string(&event).unwrap();
    db.connection
        .execute(
            "UPDATE recall_review_events SET data=?1,digest=?2",
            rusqlite::params![
                data,
                crate::workspace::migrations::checksum(data.as_bytes())
            ],
        )
        .unwrap();
    assert!(super::super::review::validate_native_data(&db.connection).is_err());
}

#[test]
fn study_review_native_replay_checks_historical_budgets_and_active_policy_order() {
    for future_policy in [false, true] {
        let (_temp, mut db, unit, revision) = setup_sequence(vec![recall(), recall()]);
        let overview = db.review_overview(context()).unwrap();
        let policy = ReviewMutation {
            operation_id: Uuid::new_v4(),
            expected_revision: 0,
            action: ReviewAction::SetPolicy {
                time_zone: "Etc/UTC".into(),
                day_start_hour: 4,
                daily_new_limit: 2,
                daily_review_limit: 50,
                retention_basis_points: 9000,
            },
        };
        if !future_policy {
            db.review_apply_at(context(), &policy, overview.now_unix_ms)
                .unwrap();
        }
        let mut s = start(&mut db, &unit, &revision, StudyMode::Practice);
        for _ in 0..2 {
            s = review_save(&mut db, &s, "Answer");
            s = db.study_apply(context(), &scheduled_submit(&s, 3)).unwrap();
        }
        if future_policy {
            db.review_apply_at(context(), &policy, overview.now_unix_ms)
                .unwrap();
            let rows = db
                .connection
                .prepare("SELECT id,data FROM recall_review_events")
                .unwrap()
                .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
                .unwrap()
                .collect::<rusqlite::Result<Vec<_>>>()
                .unwrap();
            for (id, data) in rows {
                let mut event: serde_json::Value = serde_json::from_str(&data).unwrap();
                event["planRevision"] = json!(1);
                let data = serde_json::to_string(&event).unwrap();
                db.connection
                    .execute(
                        "UPDATE recall_review_events SET data=?1,digest=?2 WHERE id=?3",
                        rusqlite::params![
                            data,
                            crate::workspace::migrations::checksum(data.as_bytes()),
                            id
                        ],
                    )
                    .unwrap();
            }
        } else {
            let (request, result): (String, String) = db
                .connection
                .query_row("SELECT request,result FROM review_operations", [], |r| {
                    Ok((r.get(0)?, r.get(1)?))
                })
                .unwrap();
            let mut request: serde_json::Value = serde_json::from_str(&request).unwrap();
            request["action"]["dailyNewLimit"] = json!(1);
            let request = serde_json::to_string(&request).unwrap();
            let mut result: serde_json::Value = serde_json::from_str(&result).unwrap();
            result["dailyNewLimit"] = json!(1);
            let result = serde_json::to_string(&result).unwrap();
            let digest = crate::workspace::migrations::checksum(result.as_bytes());
            db.connection
                .execute(
                    "UPDATE review_operations SET request=?1,request_hash=?2,result=?3,digest=?4",
                    rusqlite::params![
                        request,
                        crate::workspace::migrations::checksum(request.as_bytes()),
                        result,
                        digest
                    ],
                )
                .unwrap();
            db.connection
                .execute(
                    "UPDATE review_plan SET data=?1,digest=?2",
                    rusqlite::params![result, digest],
                )
                .unwrap();
        }
        assert!(super::super::review::validate_native_data(&db.connection).is_err());
    }
}
#[test]
fn study_review_practical_reminders_are_replayable_and_separate_from_recall_events() {
    let (_temp, mut db, unit, revision) = setup(
        json!({"family":"procedure","version":1,"prompt":"Cooking practice","steps":[{"id":"one","text":"Observe the process"}],"reflection":"Record observations","hints":[],"evaluation":"self"}),
    );
    let s = start(&mut db, &unit, &revision, StudyMode::Practice);
    let overview = db.review_overview(context()).unwrap();
    let id = Uuid::new_v4();
    let add = ReviewMutation {
        operation_id: Uuid::new_v4(),
        expected_revision: 0,
        action: ReviewAction::AddReminder {
            id,
            unit_id: unit,
            activity_revision_id: s.items[0].activity.id.clone(),
            due_day: overview.today,
            interval_days: Some(7),
        },
    };
    let first = db
        .review_apply_at(context(), &add, overview.now_unix_ms)
        .unwrap();
    assert_eq!(first.reminders[0].due_day, overview.today);
    let complete = ReviewMutation {
        operation_id: Uuid::new_v4(),
        expected_revision: 1,
        action: ReviewAction::CompleteReminder { id },
    };
    let next = db
        .review_apply_at(context(), &complete, overview.now_unix_ms)
        .unwrap();
    assert_eq!(next.reminders[0].due_day, overview.today + 7);
    assert_eq!(
        db.review_apply_at(context(), &complete, overview.now_unix_ms + 1000)
            .unwrap(),
        next
    );
    assert_eq!(
        db.connection
            .query_row("SELECT count(*) FROM study_attempts", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        db.connection
            .query_row("SELECT count(*) FROM recall_review_events", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    super::super::review::validate_native_data(&db.connection).unwrap();
}

#[test]
fn study_review_native_replay_rejects_journal_event_reordering() {
    let (_temp, mut db, unit, revision) = setup(recall());
    for _ in 0..2 {
        let s = start(&mut db, &unit, &revision, StudyMode::Practice);
        let s = review_save(&mut db, &s, "Answer");
        db.study_apply(context(), &scheduled_submit(&s, 3)).unwrap();
    }
    super::super::review::validate_native_data(&db.connection).unwrap();
    let ids = db
        .connection
        .prepare("SELECT record_id FROM review_journal ORDER BY position")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap();
    db.connection
        .execute(
            "UPDATE review_journal SET record_id=?1 WHERE position=1",
            [Uuid::new_v4().to_string()],
        )
        .unwrap();
    db.connection
        .execute(
            "UPDATE review_journal SET record_id=?1 WHERE position=2",
            [&ids[0]],
        )
        .unwrap();
    db.connection
        .execute(
            "UPDATE review_journal SET record_id=?1 WHERE position=1",
            [&ids[1]],
        )
        .unwrap();
    assert!(super::super::review::validate_native_data(&db.connection).is_err());
}

mod scoped_assessment_tests {
    use super::*;
    pub(super) fn scope(db: &mut Database, units: &[String]) -> (Uuid, Uuid) {
        let subject = Uuid::new_v4();
        let topic = Uuid::new_v4();
        let child = Uuid::new_v4();
        db.connection
            .execute(
                "INSERT INTO subjects VALUES(?1,'Synthetic subject',NULL,0)",
                [subject.to_string()],
            )
            .unwrap();
        db.connection
            .execute(
                "INSERT INTO collections VALUES(?1,?2,NULL,'Topic',0,0)",
                rusqlite::params![topic.to_string(), subject.to_string()],
            )
            .unwrap();
        db.connection
            .execute(
                "INSERT INTO collections VALUES(?1,?2,?3,'Subtopic',0,0)",
                rusqlite::params![child.to_string(), subject.to_string(), topic.to_string()],
            )
            .unwrap();
        for (index, unit) in units.iter().enumerate() {
            db.connection
                .execute(
                    "INSERT INTO placements VALUES(?1,?2,NULL,?3,?4,0)",
                    rusqlite::params![
                        Uuid::new_v4().to_string(),
                        if index == 0 { topic } else { child }.to_string(),
                        unit,
                        index as i64
                    ],
                )
                .unwrap();
        }
        // Multiple placements must not duplicate a lesson in the frozen scope.
        db.connection
            .execute(
                "INSERT INTO placements VALUES(?1,?2,NULL,?3,0,0)",
                rusqlite::params![Uuid::new_v4().to_string(), child.to_string(), units[0]],
            )
            .unwrap();
        (subject, topic)
    }
    fn create(scope: AssessmentScope) -> CreateAssessment {
        CreateAssessment {
            operation_id: Uuid::new_v4(),
            assessment_id: Uuid::new_v4(),
            scope,
        }
    }
    fn part(db: &mut Database, run: &FrozenAssessment, index: usize) -> StudySession {
        db.study_apply(
            context(),
            &StudyMutation {
                operation_id: Uuid::new_v4(),
                session_id: run.parts[index].session_id,
                expected_revision: 0,
                action: StudyAction::StartScopedAssessment {
                    assessment_id: run.id,
                    part_index: index,
                },
            },
        )
        .unwrap()
    }
    fn finish(db: &mut Database, s: &StudySession) -> StudySession {
        let s = review_save(db, s, "Answer");
        db.study_apply(
            context(),
            &request(
                &s,
                StudyAction::Submit {
                    self_rating: None,
                    recall_rating: None,
                    self_outcome: Some(AssessmentOutcome::Correct),
                },
            ),
        )
        .unwrap()
    }
    fn revise(db: &mut Database, unit: &str, head: &str) {
        let current = db
            .lesson_read(context(), unit, Some(head))
            .unwrap()
            .unwrap();
        let binding = &current.composition.bindings[0];
        let draft = db
            .lesson_create(
                context(),
                &CreateLessonDraft {
                    contract_version: 1,
                    workspace_id: db.id.clone(),
                    unit_id: unit.into(),
                    operation_id: OperationId::generate(),
                    expected_head_revision: Some(head.into()),
                },
            )
            .unwrap();
        let draft=db.lesson_edit(context(),&EditLessonDraft {contract_version:1,workspace_id:db.id.clone(),unit_id:unit.into(),draft_id:draft.id,expected_draft_revision:draft.revision,operation_id:OperationId::generate(),operations:vec![LessonOperation::CreateActivityRevision {activity_id:binding.target_id.clone(),expected_revision_id:Some(binding.revision_id.clone()),binding_id:binding.binding_id.clone(),definition:json!({"family":"recall","version":1,"prompt":"Changed prompt","answer":"Changed answer","hints":[],"evaluation":"exact"})}]}).unwrap();
        let preview = db
            .lesson_preview(
                context(),
                &LessonDraftScope {
                    contract_version: 1,
                    workspace_id: db.id.clone(),
                    unit_id: unit.into(),
                    draft_id: draft.id.clone(),
                    expected_draft_revision: draft.revision,
                },
            )
            .unwrap();
        db.lesson_apply(
            context(),
            &ApplyLessonDraft {
                contract_version: 1,
                workspace_id: db.id.clone(),
                unit_id: unit.into(),
                draft_id: draft.id,
                expected_draft_revision: draft.revision,
                expected_head_revision: Some(head.into()),
                preview_hash: preview.hash,
                operation_id: OperationId::generate(),
            },
        )
        .unwrap();
    }
    #[test]
    fn assessment_scope_pins_content_deduplicates_subtopics_and_withholds_global_feedback() {
        let (_temp, mut db, unit, revision) = setup(recall());
        let (other, _) = publish_test_lesson(
            &mut db,
            vec![
                json!({"family":"constructed","version":1,"prompt":"Explain","rubric":"Justify","hints":[],"evaluation":"self"}),
                json!({"family":"constructed","version":2,"prompt":"Complete","rubric":"Compare","hints":[],"evaluation":"self","instruction":{"phase":"completion","material":"Partial solution"}}),
            ],
        );
        let (subject, topic) = scope(&mut db, &[unit.clone(), other]);
        let request = create(AssessmentScope::Subject { id: subject });
        let run = db.assessment_create(context(), &request).unwrap();
        assert_eq!(db.assessment_create(context(), &request).unwrap(), run);
        assert_eq!(run.parts.len(), 2);
        assert_eq!(run.omissions.len(), 1);
        assert_eq!(
            db.assessment_create(context(), &create(AssessmentScope::Topic { id: topic }))
                .unwrap()
                .parts
                .len(),
            2
        );
        revise(&mut db, &unit, &revision);
        let recall_index = run.parts.iter().position(|p| p.unit_id == unit).unwrap();
        let s = part(&mut db, &run, recall_index);
        assert_eq!(s.items[0].activity.definition["answer"], "Answer");
        let s = finish(&mut db, &s);
        let view = db.study_read(context(), s.id).unwrap();
        assert!(!view.feedback_available);
        assert!(view.judgments.is_empty());
        let partial = db.assessment_read(context(), run.id).unwrap();
        assert!(!partial.feedback_available);
        assert!(partial.coverage.iter().all(|c| c.feedback.is_none()));
        assert!(partial.parts[recall_index].changed_content);
        let s = part(&mut db, &run, 1 - recall_index);
        finish(&mut db, &s);
        let complete = db.assessment_read(context(), run.id).unwrap();
        assert!(complete.feedback_available);
        let recall = complete
            .coverage
            .iter()
            .find(|c| c.outcome == "recall")
            .unwrap();
        assert_eq!(recall.feedback.as_ref().unwrap().deterministic_correct, 1);
        let written = complete
            .coverage
            .iter()
            .find(|c| c.outcome == "constructedResponse")
            .unwrap();
        assert_eq!(written.feedback.as_ref().unwrap().deterministic_unknown, 1);
        assert_eq!(written.feedback.as_ref().unwrap().self_correct, 1);
        super::super::super::assessments::validate_native_data(&db.connection).unwrap();
    }
    #[test]
    fn assessment_creation_recovers_uncertain_commit_and_reserved_sessions_cannot_be_rebound() {
        let (_temp, mut db, unit, _) = setup(recall());
        let (subject, _) = scope(&mut db, &[unit]);
        let request = create(AssessmentScope::Subject { id: subject });
        let result = faults::using(
            |phase| {
                if phase == faults::Phase::AfterCommit {
                    Err(AppError::new(ErrorCode::StorageFailure))
                } else {
                    Ok(())
                }
            },
            || db.assessment_create(context(), &request),
        );
        assert_eq!(result.unwrap_err().code, ErrorCode::OutcomeUnconfirmed);
        let run = db.assessment_create(context(), &request).unwrap();
        assert_eq!(db.assessment_list(context(), 0).unwrap().len(), 1);
        let mut conflict = request.clone();
        conflict.assessment_id = Uuid::new_v4();
        assert_eq!(
            db.assessment_create(context(), &conflict).unwrap_err().code,
            ErrorCode::RevisionConflict
        );
        let p = &run.parts[0];
        let rebound = StudyMutation {
            operation_id: Uuid::new_v4(),
            session_id: p.session_id,
            expected_revision: 0,
            action: StudyAction::Start {
                unit_id: p.unit_id.clone(),
                composition_revision_id: p.composition_revision_id.clone(),
                mode: StudyMode::Practice,
                phase: StudyPhase::Independent,
                binding_ids: None,
            },
        };
        assert_eq!(
            db.study_apply(context(), &rebound).unwrap_err().code,
            ErrorCode::InvalidRequest
        );
        let wrong = StudyMutation {
            operation_id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            expected_revision: 0,
            action: StudyAction::StartScopedAssessment {
                assessment_id: run.id,
                part_index: 0,
            },
        };
        assert_eq!(
            db.study_apply(context(), &wrong).unwrap_err().code,
            ErrorCode::InvalidRequest
        );
        let s = part(&mut db, &run, 0);
        assert!(
            db.study_apply(context(), &super::request(&s, StudyAction::RevealSolution))
                .is_err()
        );
    }
    #[test]
    fn assessment_partial_history_roundtrips_only_when_opted_in_and_rejects_relabeling() {
        use std::io::Cursor;
        let (temp, mut db, unit, _) = setup(recall());
        let (subject, _) = scope(&mut db, &[unit]);
        let run = db
            .assessment_create(context(), &create(AssessmentScope::Subject { id: subject }))
            .unwrap();
        let s = part(&mut db, &run, 0);
        let s = review_save(&mut db, &s, "Saved assessment draft");
        for history in [false, true] {
            let capture = db
                .capture_portable_with_history(context(), Uuid::new_v4(), false, history)
                .unwrap();
            let mut bytes = Cursor::new(Vec::new());
            capture.write_to(&mut bytes).unwrap();
            let package = PortableCapture::read_from(Cursor::new(bytes.into_inner())).unwrap();
            let root = temp.path().join(format!("assessment-{history}"));
            let manager = CourseManager::open(root.clone()).unwrap();
            let course = Uuid::new_v4();
            manager
                .import_portable(course, "Synthetic assessment".into(), &package)
                .unwrap();
            let restored =
                Database::open(&root.join("courses/data").join(course.to_string())).unwrap();
            assert_eq!(
                restored.assessment_list(context(), 0).unwrap().len(),
                usize::from(history)
            );
            if history {
                assert_eq!(
                    restored
                        .assessment_read(context(), run.id)
                        .unwrap()
                        .assessment,
                    run
                );
                assert_eq!(
                    restored.study_read(context(), s.id).unwrap().session.draft,
                    s.draft
                );
            }
        }
        let mut forged = s.clone();
        forged.assessment_id = None;
        let data = serde_json::to_string(&forged).unwrap();
        db.connection
            .execute(
                "UPDATE study_sessions SET data=?1,digest=?2 WHERE id=?3",
                rusqlite::params![
                    data,
                    crate::workspace::migrations::checksum(data.as_bytes()),
                    s.id.to_string()
                ],
            )
            .unwrap();
        assert!(super::super::super::assessments::validate_native_data(&db.connection).is_err());
    }
}

#[test]
fn study_review_process_crash_preserves_schedule_event_and_state() {
    check_study_process_crash(true);
}
