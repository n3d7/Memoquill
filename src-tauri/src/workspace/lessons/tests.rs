use super::*;
use serde_json::json;
fn composition() -> Value {
    json!({"contractVersion":1,"unitId":"unit","rootNodeId":"root","nodes":[{"nodeId":"root","typeId":"stack","typeVersion":1,"props":{"gap":"normal","headingLevel":1},"slots":{"body":[]},"on":{}}],"bindings":[],"sourceBindingRefs":[],"appearance":{"spacing":"normal","theme":"system","typography":"default","width":"normal","emphasis":"normal"},"authoringProvenance":{"kind":"manual"}})
}
#[test]
fn rejects_cycles_unreachable_unknown_and_duplicate_children() {
    let mut c = composition();
    c["nodes"][0]["slots"]["body"] = json!(["root"]);
    assert!(validate_composition(&c, false).is_err());
    let mut c = composition();
    let mut n = c["nodes"][0].clone();
    n["nodeId"] = json!("orphan");
    c["nodes"].as_array_mut().unwrap().push(n);
    assert!(validate_composition(&c, false).is_err());
    let mut c = composition();
    c["nodes"][0]["props"]["script"] = json!("evil");
    assert!(validate_composition(&c, false).is_err());
    let mut c = composition();
    c["nodes"][0]["typeId"] = json!("future");
    assert!(validate_composition(&c, false).is_err());
    assert!(validate_composition(&c, true).is_ok());
}
#[test]
fn accepts_closed_root_but_enforces_condition_and_prop_budgets() {
    let mut c = composition();
    assert!(validate_composition(&c, false).is_ok());
    c["nodes"][0]["visible"] = json!({"kind":"not","expr":{"kind":"not","expr":{"kind":"not","expr":{"kind":"not","expr":{"kind":"literal","value":true}}}}});
    assert!(validate_composition(&c, false).is_err());
    let mut c = composition();
    c["nodes"][0]["props"]["gap"] = json!("x".repeat(17000));
    assert!(validate_composition(&c, false).is_err());
}

use crate::workspace::{OperationId, RequestContext, database::Database};
use uuid::Uuid;
fn setup() -> (tempfile::TempDir, Database, String) {
    let temp = tempfile::tempdir().unwrap();
    let db = Database::open(&temp.path().join("workspace")).unwrap();
    let unit = Uuid::new_v4().to_string();
    db.connection
        .execute("INSERT INTO units VALUES(?1,'Lesson','goal',0)", [&unit])
        .unwrap();
    (temp, db, unit)
}
fn create(db: &mut Database, unit: &str) -> LessonDraft {
    db.lesson_create(
        RequestContext::local_ui(),
        &CreateLessonDraft {
            contract_version: 1,
            workspace_id: db.id.clone(),
            unit_id: unit.into(),
            operation_id: OperationId::generate(),
            expected_head_revision: None,
        },
    )
    .unwrap()
}
fn scope(db: &Database, d: &LessonDraft) -> LessonDraftScope {
    LessonDraftScope {
        contract_version: 1,
        workspace_id: db.id.clone(),
        unit_id: d.unit_id.clone(),
        draft_id: d.id.clone(),
        expected_draft_revision: d.revision,
    }
}
fn edit(db: &Database, d: &LessonDraft, operations: Vec<LessonOperation>) -> EditLessonDraft {
    EditLessonDraft {
        contract_version: 1,
        workspace_id: db.id.clone(),
        unit_id: d.unit_id.clone(),
        draft_id: d.id.clone(),
        expected_draft_revision: d.revision,
        operation_id: OperationId::generate(),
        operations,
    }
}
fn apply(db: &Database, d: &LessonDraft, p: &LessonPreview) -> ApplyLessonDraft {
    ApplyLessonDraft {
        contract_version: 1,
        workspace_id: db.id.clone(),
        unit_id: d.unit_id.clone(),
        draft_id: d.id.clone(),
        expected_draft_revision: d.revision,
        expected_head_revision: d.base_head_revision.clone(),
        preview_hash: p.hash.clone(),
        operation_id: OperationId::generate(),
    }
}
#[test]
fn durable_apply_exact_retry_and_changed_payload_conflict() {
    let (temp, mut db, unit) = setup();
    let d = create(&mut db, &unit);
    let p = db
        .lesson_preview(RequestContext::local_ui(), &scope(&db, &d))
        .unwrap();
    let request = apply(&db, &d, &p);
    let receipt = db
        .lesson_apply(RequestContext::local_ui(), &request)
        .unwrap();
    assert!(receipt.committed);
    assert_eq!(
        db.lesson_apply(RequestContext::local_ui(), &request)
            .unwrap(),
        receipt
    );
    let mut changed = request.clone();
    changed.preview_hash = "different".into();
    assert_eq!(
        db.lesson_apply(RequestContext::local_ui(), &changed)
            .unwrap_err()
            .code,
        ErrorCode::IdempotencyConflict
    );
    assert_eq!(
        db.lesson_receipt(RequestContext::local_ui(), &request.operation_id)
            .unwrap(),
        Some(receipt.clone())
    );
    drop(db);
    let db = Database::open(&temp.path().join("workspace")).unwrap();
    assert_eq!(
        db.lesson_read(RequestContext::local_ui(), &unit, None)
            .unwrap()
            .unwrap(),
        receipt.saved
    );
}
#[test]
fn stale_previews_conflicts_and_scope_preserve_draft() {
    let (_temp, mut db, unit) = setup();
    let d = create(&mut db, &unit);
    let p = db
        .lesson_preview(RequestContext::local_ui(), &scope(&db, &d))
        .unwrap();
    let request = apply(&db, &d, &p);
    let e = edit(
        &db,
        &d,
        vec![LessonOperation::ConfigureNode {
            id: d.composition.root_node_id.clone(),
            props: json!({"gap":"wide","headingLevel":2}),
        }],
    );
    let changed = db.lesson_edit(RequestContext::local_ui(), &e).unwrap();
    assert_eq!(
        db.lesson_apply(RequestContext::local_ui(), &request)
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        db.lesson_edit(RequestContext::local_ui(), &e).unwrap(),
        changed
    );
    let mut wrong = scope(&db, &changed);
    wrong.unit_id = Uuid::new_v4().to_string();
    assert!(
        db.lesson_preview(RequestContext::local_ui(), &wrong)
            .is_err()
    );
    assert_eq!(
        db.lesson_list_drafts(RequestContext::local_ui(), &unit)
            .unwrap(),
        vec![changed]
    );
}
#[test]
fn independent_units_and_immutable_history() {
    let (_temp, mut db, u) = setup();
    let v = Uuid::new_v4().to_string();
    db.connection
        .execute("INSERT INTO units VALUES(?1,'Other','goal',0)", [&v])
        .unwrap();
    let a = create(&mut db, &u);
    let b = create(&mut db, &v);
    assert_ne!(a.id, b.id);
    assert_ne!(a.composition.root_node_id, b.composition.root_node_id);
    let p = db
        .lesson_preview(RequestContext::local_ui(), &scope(&db, &a))
        .unwrap();
    let r = db
        .lesson_apply(RequestContext::local_ui(), &apply(&db, &a, &p))
        .unwrap();
    assert!(
        db.lesson_read(RequestContext::local_ui(), &v, None)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        db.lesson_list_drafts(RequestContext::local_ui(), &v)
            .unwrap(),
        vec![b]
    );
    let d = db
        .lesson_create(
            RequestContext::local_ui(),
            &CreateLessonDraft {
                contract_version: 1,
                workspace_id: db.id.clone(),
                unit_id: u.clone(),
                operation_id: OperationId::generate(),
                expected_head_revision: Some(r.new_head.clone()),
            },
        )
        .unwrap();
    let p = db
        .lesson_preview(RequestContext::local_ui(), &scope(&db, &d))
        .unwrap();
    db.lesson_apply(RequestContext::local_ui(), &apply(&db, &d, &p))
        .unwrap();
    assert_eq!(
        db.lesson_read(RequestContext::local_ui(), &u, Some(&r.new_head))
            .unwrap()
            .unwrap(),
        r.saved
    );
}
#[test]
fn proposals_stage_activity_occurrence_pins_and_rollback_invalid_batch() {
    let (_temp, mut db, u) = setup();
    let d = create(&mut db, &u);
    let activity = Uuid::new_v4().to_string();
    let op = LessonOperation::CreateActivityRevision {
        activity_id: activity.clone(),
        expected_revision_id: None,
        binding_id: "exercise".into(),
        definition: json!({"family":"recall","version":1,"prompt":"Recall","answer":"Answer","hints":[],"evaluation":"self"}),
    };
    let mut bad = edit(
        &db,
        &d,
        vec![
            op.clone(),
            LessonOperation::RemoveNode {
                id: d.composition.root_node_id.clone(),
            },
        ],
    );
    assert!(db.lesson_edit(RequestContext::local_ui(), &bad).is_err());
    assert_eq!(
        db.lesson_list_drafts(RequestContext::local_ui(), &u)
            .unwrap(),
        vec![d.clone()]
    );
    bad.operations.pop();
    let mut changed = db.lesson_edit(RequestContext::local_ui(), &bad).unwrap();
    let revision = changed.composition.bindings[0].revision_id.clone();
    for id in ["first", "second"] {
        let node = LessonNode {
            node_id: id.into(),
            type_id: "activity".into(),
            type_version: 1,
            props: json!({"bindingId":"exercise","presentation":"default"}),
            slots: Default::default(),
            on: Default::default(),
            visible: None,
        };
        let e = edit(
            &db,
            &changed,
            vec![LessonOperation::InsertNode {
                parent: changed.composition.root_node_id.clone(),
                slot: "body".into(),
                index: 0,
                node,
            }],
        );
        changed = db.lesson_edit(RequestContext::local_ui(), &e).unwrap();
    }
    assert_eq!(changed.composition.nodes.len(), 3);
    let p = db
        .lesson_preview(RequestContext::local_ui(), &scope(&db, &changed))
        .unwrap();
    let r = db
        .lesson_apply(RequestContext::local_ui(), &apply(&db, &changed, &p))
        .unwrap();
    assert_eq!(r.saved.composition.bindings[0].revision_id, revision);
    assert!(
        db.lesson_activity(RequestContext::local_ui(), &u, &revision)
            .unwrap()
            .is_some()
    );
}
#[test]
fn untrusted_callers_and_invalid_persisted_data_cannot_bypass_validation() {
    let (_temp, mut db, u) = setup();
    let d = create(&mut db, &u);
    assert_eq!(
        db.lesson_preview(RequestContext::unverified_external(), &scope(&db, &d))
            .unwrap_err()
            .code,
        ErrorCode::PermissionDenied
    );
    db.connection
        .execute("UPDATE lesson_drafts SET data='{}' WHERE id=?1", [&d.id])
        .unwrap();
    assert_eq!(
        db.lesson_list_drafts(RequestContext::local_ui(), &u)
            .unwrap_err()
            .code,
        ErrorCode::RecoveryRequired
    );
}

#[test]
fn staged_revision_chains_commit_dependency_closed_resources_only_to_one_unit() {
    let (_temp, mut db, u) = setup();
    let d = create(&mut db, &u);
    let resource = Uuid::new_v4().to_string();
    let document = crate::document::Document {
        schema_version: 1,
        blocks: vec![crate::document::Block {
            id: Uuid::new_v4(),
            content: crate::document::BlockContent::Code {
                text: "first".into(),
                language: None,
            },
        }],
    };
    let op = LessonOperation::CreateResourceRevision {
        resource_id: resource.clone(),
        expected_revision_id: None,
        title: "Owned resource".into(),
        binding_id: "text".into(),
        document: document.clone(),
    };
    let d = db
        .lesson_edit(RequestContext::local_ui(), &edit(&db, &d, vec![op]))
        .unwrap();
    let first = d.composition.bindings[0].revision_id.clone();
    let mut changed = document;
    changed.blocks[0].content = crate::document::BlockContent::Code {
        text: "second".into(),
        language: None,
    };
    let d = db
        .lesson_edit(
            RequestContext::local_ui(),
            &edit(
                &db,
                &d,
                vec![LessonOperation::CreateResourceRevision {
                    resource_id: resource.clone(),
                    expected_revision_id: Some(first.clone()),
                    title: "Owned resource".into(),
                    binding_id: "text".into(),
                    document: changed,
                }],
            ),
        )
        .unwrap();
    assert_eq!(
        db.connection
            .query_row("SELECT count(*) FROM resource_revisions", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    let p = db
        .lesson_preview(RequestContext::local_ui(), &scope(&db, &d))
        .unwrap();
    let receipt = db
        .lesson_apply(RequestContext::local_ui(), &apply(&db, &d, &p))
        .unwrap();
    assert!(receipt.created_revision_ids.contains(&first));
    assert_eq!(
        db.connection
            .query_row(
                "SELECT count(*) FROM resource_revisions WHERE resource_id=?1",
                [&resource],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        2
    );
    let other = Uuid::new_v4().to_string();
    db.connection
        .execute("INSERT INTO units VALUES(?1,'Other','goal',0)", [&other])
        .unwrap();
    let draft = create(&mut db, &other);
    let op = LessonOperation::CreateResourceRevision {
        resource_id: resource.clone(),
        expected_revision_id: Some(receipt.saved.composition.bindings[0].revision_id.clone()),
        title: "Forbidden".into(),
        binding_id: "text".into(),
        document: crate::document::Document {
            schema_version: 1,
            blocks: vec![],
        },
    };
    assert!(
        db.lesson_edit(RequestContext::local_ui(), &edit(&db, &draft, vec![op]))
            .is_err()
    );
}
#[test]
fn rejects_missing_future_mutation_contracts_and_unknown_envelope_fields() {
    let (_temp, mut db, u) = setup();
    let value = json!({"workspaceId":db.id,"unitId":u,"operationId":OperationId::generate(),"expectedHeadRevision":null});
    assert!(serde_json::from_value::<CreateLessonDraft>(value.clone()).is_err());
    let mut future = value;
    future["contractVersion"] = json!(2);
    let request: CreateLessonDraft = serde_json::from_value(future.clone()).unwrap();
    assert!(
        db.lesson_create(RequestContext::local_ui(), &request)
            .is_err()
    );
    future["contractVersion"] = json!(1);
    future["trustedUi"] = json!(true);
    assert!(serde_json::from_value::<CreateLessonDraft>(future).is_err());
}

#[test]
fn unsupported_nodes_roundtrip_inert_and_block_apply_until_explicit_removal() {
    let (_temp, mut db, u) = setup();
    let d = create(&mut db, &u);
    let p = db
        .lesson_preview(RequestContext::local_ui(), &scope(&db, &d))
        .unwrap();
    let saved = db
        .lesson_apply(RequestContext::local_ui(), &apply(&db, &d, &p))
        .unwrap()
        .saved;
    let mut composition = saved.composition;
    composition.nodes.push(LessonNode {
        node_id: "future".into(),
        type_id: "future-widget".into(),
        type_version: 7,
        props: json!({"preserve":{"payload":[1,2,3]},"newField":"original"}),
        slots: Default::default(),
        on: Default::default(),
        visible: None,
    });
    composition.nodes[0]
        .slots
        .get_mut("body")
        .unwrap()
        .push("future".into());
    let data = serde_json::to_string(&composition).unwrap();
    let digest = crate::workspace::migrations::checksum(data.as_bytes());
    db.connection
        .execute(
            "UPDATE lesson_revisions SET data=?1,digest=?2 WHERE id=?3",
            rusqlite::params![data, digest, saved.id],
        )
        .unwrap();
    let read = db
        .lesson_read(RequestContext::local_ui(), &u, None)
        .unwrap()
        .unwrap();
    assert_eq!(read.composition, composition);
    let draft = db
        .lesson_create(
            RequestContext::local_ui(),
            &CreateLessonDraft {
                contract_version: 1,
                workspace_id: db.id.clone(),
                unit_id: u.clone(),
                operation_id: OperationId::generate(),
                expected_head_revision: Some(saved.id),
            },
        )
        .unwrap();
    assert!(
        db.lesson_preview(RequestContext::local_ui(), &scope(&db, &draft))
            .is_err()
    );
    let repaired = db
        .lesson_edit(
            RequestContext::local_ui(),
            &edit(
                &db,
                &draft,
                vec![LessonOperation::RemoveNode {
                    id: "future".into(),
                }],
            ),
        )
        .unwrap();
    assert!(
        db.lesson_preview(RequestContext::local_ui(), &scope(&db, &repaired))
            .is_ok()
    );
}
#[test]
fn invalid_event_action_pair_and_source_locator_do_not_gain_authority() {
    let mut c = composition();
    c["nodes"][0]["typeId"] = json!("section");
    c["nodes"][0]["props"] = json!({"title":"Section","collapsible":true});
    c["nodes"][0]["on"] = json!({"toggle":{"actionId":"view.focusNode","version":1,"targetNodeId":"root","inputs":{}}});
    assert!(validate_composition(&c, false).is_ok());
    // An input change must not submit an attempt, even if it names a valid activity target.
    let binding = json!({"bindingId":"a","targetKind":"activity","targetId":Uuid::new_v4(),"revisionId":Uuid::new_v4()});
    c["bindings"] = json!([binding]);
    c["nodes"][0]["typeId"] = json!("activity");
    c["nodes"][0]["props"] = json!({"bindingId":"a","presentation":"default"});
    c["nodes"][0]["slots"] = json!({});
    c["nodes"][0]["on"] = json!({"response":{"actionId":"study.submit","version":1,"targetNodeId":"root","inputs":{}}});
    assert!(validate_composition(&c, false).is_err());
}

#[test]
fn source_snapshots_are_unit_scoped_immutable_and_honest_about_coverage() {
    let (_temp, mut db, u) = setup();
    let source = Uuid::new_v4().to_string();
    let resource = Uuid::new_v4().to_string();
    let operation = OperationId::generate();
    let digest = "a".repeat(64);
    db.connection
        .execute(
            "INSERT INTO operations VALUES(?1,?2,'local-ui','hash','asset','applied','{}',NULL)",
            rusqlite::params![operation.as_str(), db.id.as_str()],
        )
        .unwrap();
    db.connection
        .execute(
            "INSERT INTO assets VALUES(?1,4,?2)",
            rusqlite::params![digest, operation.as_str()],
        )
        .unwrap();
    db.connection
        .execute("INSERT INTO resources VALUES(?1,'Original',0)", [&resource])
        .unwrap();
    db.connection.execute("INSERT INTO original_sources VALUES(?1,?2,?3,'source.pdf','application/pdf','Author','CC0',NULL,1,4)",rusqlite::params![source,resource,digest]).unwrap();
    let d = create(&mut db, &u);
    let op:LessonOperation=serde_json::from_value(json!({"kind":"createSourceBindingRevision","sourceId":source,"bindingId":"source","locator":"page:2","declaredUsedRanges":["page:2"]})).unwrap();
    let d = db
        .lesson_edit(RequestContext::local_ui(), &edit(&db, &d, vec![op]))
        .unwrap();
    let revision = d.composition.bindings[0].revision_id.clone();
    assert_ne!(revision, source);
    let p = db
        .lesson_preview(RequestContext::local_ui(), &scope(&db, &d))
        .unwrap();
    db.lesson_apply(RequestContext::local_ui(), &apply(&db, &d, &p))
        .unwrap();
    let data = db
        .lesson_source(RequestContext::local_ui(), &u, &revision)
        .unwrap()
        .unwrap();
    assert_eq!(data.extraction_coverage, "notExtracted");
    assert_eq!(data.read_coverage, "unknown");
    assert_eq!(data.declared_used_ranges, vec!["page:2"]);
    assert_eq!(data.asset_digest, digest);
    let other = Uuid::new_v4().to_string();
    db.connection
        .execute("INSERT INTO units VALUES(?1,'Other','goal',0)", [&other])
        .unwrap();
    assert!(
        db.lesson_source(RequestContext::local_ui(), &other, &revision)
            .unwrap()
            .is_none()
    );
    let other = create(&mut db, &other);
    let binding = d.composition.bindings[0].clone();
    assert!(
        db.lesson_edit(
            RequestContext::local_ui(),
            &edit(
                &db,
                &other,
                vec![LessonOperation::BindReference { binding }]
            )
        )
        .is_err()
    );
}

#[test]
fn closed_registry_accepts_all_types_and_rejects_duplicate_depth_and_condition_counts() {
    let mut c = composition();
    let mut child = c["nodes"][0].clone();
    child["nodeId"] = json!("child");
    c["nodes"].as_array_mut().unwrap().push(child);
    c["nodes"][0]["slots"]["body"] = json!(["child", "child"]);
    assert!(validate_composition(&c, false).is_err());
    let mut c = composition();
    for i in 1..13 {
        let mut node = c["nodes"][0].clone();
        node["nodeId"] = json!(format!("n{i}"));
        node["slots"]["body"] = json!([]);
        c["nodes"][i - 1]["slots"]["body"] = json!([format!("n{i}")]);
        c["nodes"].as_array_mut().unwrap().push(node);
    }
    assert!(validate_composition(&c, false).is_err());
    let mut c = composition();
    c["nodes"][0]["visible"] =
        json!({"kind":"all","exprs":vec![json!({"kind":"literal","value":true});32]});
    assert!(validate_composition(&c, false).is_err());
    let mut c = composition();
    c["nodes"][0]["slots"]["body"] = json!(vec!["absent"; 65]);
    assert!(validate_composition(&c, false).is_err());
    for (type_id, props, slots) in [
        (
            "columns",
            json!({"ratio":"equal"}),
            json!({"primary":[],"secondary":[]}),
        ),
        (
            "section",
            json!({"title":"Title","collapsible":true}),
            json!({"body":[]}),
        ),
        (
            "status",
            json!({"field":"response.present","label":"Response present"}),
            json!({}),
        ),
        (
            "button",
            json!({"label":"Focus","style":"secondary","action":{"actionId":"view.focusNode","version":1,"targetNodeId":"root","inputs":{}}}),
            json!({}),
        ),
    ] {
        let mut c = composition();
        c["nodes"][0]["typeId"] = json!(type_id);
        c["nodes"][0]["props"] = props;
        c["nodes"][0]["slots"] = slots;
        assert!(validate_composition(&c, false).is_ok(), "{type_id}");
        c["nodes"][0]["props"]["unknown"] = json!(true);
        assert!(validate_composition(&c, false).is_err());
    }
    for kind in ["content", "source", "activity"] {
        let mut c = composition();
        let binding_kind = if kind == "content" { "resource" } else { kind };
        c["bindings"] = json!([{"bindingId":"ref","targetKind":binding_kind,"targetId":Uuid::new_v4(),"revisionId":Uuid::new_v4()}]);
        if kind == "source" {
            c["sourceBindingRefs"] = json!(["ref"]);
        }
        c["nodes"][0]["typeId"] = json!(kind);
        c["nodes"][0]["props"] = if kind == "activity" {
            json!({"bindingId":"ref","presentation":"default"})
        } else {
            json!({"bindingId":"ref"})
        };
        c["nodes"][0]["slots"] = json!({});
        assert!(validate_composition(&c, false).is_ok());
    }
}
#[test]
fn all_activity_families_retain_explicit_evaluation_and_reject_executable_fields() {
    let definitions = [
        json!({"family":"recall","version":1,"prompt":"Word","answer":"Answer","evaluation":"exact","hints":[]}),
        json!({"family":"singleChoice","version":1,"prompt":"Choice","options":[{"id":"one","text":"First"}],"correctOptionId":"one","evaluation":"exact","hints":[]}),
        json!({"family":"constructed","version":1,"prompt":"Explain","rubric":"Compare","evaluation":"self","hints":[]}),
        json!({"family":"procedure","version":1,"prompt":"Perform","steps":[{"id":"step","text":"Observe"}],"reflection":"Reflect","evaluation":"self","hints":[]}),
    ];
    for mut d in definitions {
        assert!(validation::activity(&d).is_ok());
        d["runner"] = json!("python");
        assert!(validation::activity(&d).is_err());
    }
    let mut constructed = json!({"family":"constructed","version":1,"prompt":"Explain","rubric":"Compare","evaluation":"exact","hints":[]});
    assert!(validation::activity(&constructed).is_err());
    constructed["evaluation"] = json!("self");
    constructed["version"] = json!(2);
    assert!(validation::activity(&constructed).is_err());
}
#[test]
fn draft_and_batch_budgets_fail_atomically_and_missing_revision_pin_is_rejected() {
    let (_temp, mut db, u) = setup();
    let d = create(&mut db, &u);
    let op = LessonOperation::ConfigureNode {
        id: d.composition.root_node_id.clone(),
        props: json!({"gap":"wide","headingLevel":2}),
    };
    assert!(
        db.lesson_edit(RequestContext::local_ui(), &edit(&db, &d, vec![op; 65]))
            .is_err()
    );
    let missing = Binding {
        binding_id: "missing".into(),
        target_kind: "resource".into(),
        target_id: Uuid::new_v4().to_string(),
        revision_id: Uuid::new_v4().to_string(),
        locator: None,
    };
    assert!(
        db.lesson_edit(
            RequestContext::local_ui(),
            &edit(
                &db,
                &d,
                vec![LessonOperation::BindReference { binding: missing }]
            )
        )
        .is_err()
    );
    assert_eq!(
        db.lesson_list_drafts(RequestContext::local_ui(), &u)
            .unwrap(),
        vec![d]
    );
    for _ in 1..20 {
        create(&mut db, &u);
    }
    let request = CreateLessonDraft {
        contract_version: 1,
        workspace_id: db.id.clone(),
        unit_id: u.clone(),
        operation_id: OperationId::generate(),
        expected_head_revision: None,
    };
    assert!(
        db.lesson_create(RequestContext::local_ui(), &request)
            .is_err()
    );
    assert_eq!(
        db.lesson_list_drafts(RequestContext::local_ui(), &u)
            .unwrap()
            .len(),
        20
    );
}

#[test]
fn apply_lost_reply_and_precommit_failure_have_recoverable_exact_outcomes() {
    use crate::workspace::faults::{self, Phase};
    let (_temp, mut db, u) = setup();
    let d = create(&mut db, &u);
    let p = db
        .lesson_preview(RequestContext::local_ui(), &scope(&db, &d))
        .unwrap();
    let request = apply(&db, &d, &p);
    let before = faults::using(
        |phase| {
            if phase == Phase::BeforeCommit {
                Err(AppError::new(ErrorCode::StorageFailure))
            } else {
                Ok(())
            }
        },
        || db.lesson_apply(RequestContext::local_ui(), &request),
    );
    assert_eq!(before.unwrap_err().code, ErrorCode::StorageFailure);
    assert!(
        db.lesson_receipt(RequestContext::local_ui(), &request.operation_id)
            .unwrap()
            .is_none()
    );
    assert!(
        db.lesson_read(RequestContext::local_ui(), &u, None)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        db.lesson_list_drafts(RequestContext::local_ui(), &u)
            .unwrap(),
        vec![d]
    );
    let lost = faults::using(
        |phase| {
            if phase == Phase::AfterCommit {
                Err(AppError::new(ErrorCode::StorageFailure))
            } else {
                Ok(())
            }
        },
        || db.lesson_apply(RequestContext::local_ui(), &request),
    );
    assert_eq!(lost.unwrap_err().code, ErrorCode::OutcomeUnconfirmed);
    let receipt = db
        .lesson_receipt(RequestContext::local_ui(), &request.operation_id)
        .unwrap()
        .unwrap();
    assert_eq!(
        db.lesson_apply(RequestContext::local_ui(), &request)
            .unwrap(),
        receipt
    );
    assert!(
        db.lesson_list_drafts(RequestContext::local_ui(), &u)
            .unwrap()
            .is_empty()
    );
}

fn saved_opaque(
    db: &mut Database,
    unit: &str,
    root_unknown: bool,
) -> (SavedComposition, Composition) {
    let d = create(db, unit);
    let p = db
        .lesson_preview(RequestContext::local_ui(), &scope(db, &d))
        .unwrap();
    let saved = db
        .lesson_apply(RequestContext::local_ui(), &apply(db, &d, &p))
        .unwrap()
        .saved;
    let mut c = saved.composition.clone();
    c.nodes[0].type_id = if root_unknown {
        "future-root"
    } else {
        "columns"
    }
    .into();
    c.nodes[0].type_version = if root_unknown { 9 } else { 1 };
    c.nodes[0].props = if root_unknown {
        json!({"keep":"opaque-original"})
    } else {
        json!({"ratio":"equal"})
    };
    c.nodes[0].slots = std::collections::BTreeMap::from([
        ("primary".into(), vec![]),
        ("secondary".into(), vec![]),
    ]);
    for index in 0..65 {
        let id = format!("future-{index}");
        c.nodes[0]
            .slots
            .get_mut(if index < 64 { "primary" } else { "secondary" })
            .unwrap()
            .push(id.clone());
        c.nodes.push(LessonNode {
            node_id: id,
            type_id: "future-leaf".into(),
            type_version: 3,
            props: json!({"opaque":{"value":index}}),
            slots: Default::default(),
            on: Default::default(),
            visible: None,
        });
    }
    let data = serde_json::to_string(&c).unwrap();
    let digest = crate::workspace::migrations::checksum(data.as_bytes());
    db.connection
        .execute(
            "UPDATE lesson_revisions SET data=?1,digest=?2 WHERE id=?3",
            rusqlite::params![data, digest, saved.id],
        )
        .unwrap();
    (saved, c)
}
#[test]
fn unsupported_repair_can_span_bounded_batches_without_opaque_payload_edits() {
    let (_temp, mut db, u) = setup();
    let (saved, original) = saved_opaque(&mut db, &u, false);
    let d = db
        .lesson_create(
            RequestContext::local_ui(),
            &CreateLessonDraft {
                contract_version: 1,
                workspace_id: db.id.clone(),
                unit_id: u.clone(),
                operation_id: OperationId::generate(),
                expected_head_revision: Some(saved.id.clone()),
            },
        )
        .unwrap();
    let change = edit(
        &db,
        &d,
        vec![LessonOperation::ConfigureNode {
            id: "future-0".into(),
            props: json!({"edited":true}),
        }],
    );
    assert!(db.lesson_edit(RequestContext::local_ui(), &change).is_err());
    let ops = (0..64)
        .map(|i| LessonOperation::RemoveNode {
            id: format!("future-{i}"),
        })
        .collect();
    let partial = db
        .lesson_edit(RequestContext::local_ui(), &edit(&db, &d, ops))
        .unwrap();
    assert_eq!(partial.composition.nodes.len(), 2);
    assert_eq!(partial.composition.nodes[1].props, original.nodes[65].props);
    assert!(
        db.lesson_preview(RequestContext::local_ui(), &scope(&db, &partial))
            .is_err()
    );
    let apply_request = ApplyLessonDraft {
        contract_version: 1,
        workspace_id: db.id.clone(),
        unit_id: u.clone(),
        draft_id: partial.id.clone(),
        expected_draft_revision: partial.revision,
        expected_head_revision: partial.base_head_revision.clone(),
        preview_hash: "not-approved".into(),
        operation_id: OperationId::generate(),
    };
    assert!(
        db.lesson_apply(RequestContext::local_ui(), &apply_request)
            .is_err()
    );
    let repaired = db
        .lesson_edit(
            RequestContext::local_ui(),
            &edit(
                &db,
                &partial,
                vec![LessonOperation::RemoveNode {
                    id: "future-64".into(),
                }],
            ),
        )
        .unwrap();
    let p = db
        .lesson_preview(RequestContext::local_ui(), &scope(&db, &repaired))
        .unwrap();
    db.lesson_apply(RequestContext::local_ui(), &apply(&db, &repaired, &p))
        .unwrap();
    assert_eq!(
        db.lesson_read(RequestContext::local_ui(), &u, Some(&saved.id))
            .unwrap()
            .unwrap()
            .composition,
        original
    );
}
#[test]
fn unsupported_root_has_explicit_safe_replacement_preserving_original_revision() {
    let (_temp, mut db, u) = setup();
    let (saved, original) = saved_opaque(&mut db, &u, true);
    let d = db
        .lesson_create(
            RequestContext::local_ui(),
            &CreateLessonDraft {
                contract_version: 1,
                workspace_id: db.id.clone(),
                unit_id: u.clone(),
                operation_id: OperationId::generate(),
                expected_head_revision: Some(saved.id.clone()),
            },
        )
        .unwrap();
    let replacement = json!({"kind":"replaceUnsupportedRoot","node":{"nodeId":"repaired-root","typeId":"columns","typeVersion":1,"props":{"ratio":"equal"},"slots":original.nodes[0].slots,"on":{}}});
    let op: LessonOperation = serde_json::from_value(replacement).unwrap();
    let partial = db
        .lesson_edit(RequestContext::local_ui(), &edit(&db, &d, vec![op]))
        .unwrap();
    assert_eq!(partial.composition.root_node_id, "repaired-root");
    assert!(
        db.lesson_preview(RequestContext::local_ui(), &scope(&db, &partial))
            .is_err()
    );
    assert_eq!(
        db.lesson_read(RequestContext::local_ui(), &u, Some(&saved.id))
            .unwrap()
            .unwrap()
            .composition,
        original
    );
}

#[test]
fn rejected_lesson_edits_return_bounded_safe_field_and_operation_diagnostics() {
    let (_temp, mut db, u) = setup();
    let d = create(&mut db, &u);
    let e = edit(
        &db,
        &d,
        vec![LessonOperation::ConfigureNode {
            id: d.composition.root_node_id.clone(),
            props: json!({"gap":"script","headingLevel":1}),
        }],
    );
    let error = db.lesson_edit(RequestContext::local_ui(), &e).unwrap_err();
    let wire = serde_json::to_value(&error).unwrap();
    assert_eq!(wire["diagnostics"][0]["nodeId"], d.composition.root_node_id);
    assert_eq!(wire["diagnostics"][0]["operationIndex"], 0);
    assert_eq!(wire["diagnostics"][0]["field"], "props.gap");
    assert_eq!(wire["diagnostics"][0]["repair"], "use_declared_value");
    let binding = Binding {
        binding_id: "declared-local-ref".into(),
        target_kind: "resource".into(),
        target_id: Uuid::new_v4().to_string(),
        revision_id: Uuid::new_v4().to_string(),
        locator: None,
    };
    let hidden = binding.target_id.clone();
    let e = edit(&db, &d, vec![LessonOperation::BindReference { binding }]);
    let error = db.lesson_edit(RequestContext::local_ui(), &e).unwrap_err();
    let wire = serde_json::to_value(&error).unwrap();
    assert_eq!(wire["diagnostics"][0]["bindingId"], "declared-local-ref");
    assert_eq!(wire["diagnostics"][0]["field"], "revisionId");
    assert!(!serde_json::to_string(&wire).unwrap().contains(&hidden));
}

#[test]
fn offline_catalog_supplies_typed_event_action_state_and_fallback_contracts() {
    let catalog = validation::discover();
    assert_eq!(catalog.len(), 8);
    let button = catalog.iter().find(|d| d.type_id == "button").unwrap();
    assert_eq!(button.props_schema["additionalProperties"], false);
    assert_eq!(button.defaults["style"], "secondary");
    assert_eq!(button.event_schemas["activate"].maximum_actions, 1);
    let toggle = button
        .actions
        .iter()
        .find(|a| a.action_id == "view.toggleSection")
        .unwrap();
    assert_eq!(toggle.input_schema["required"], json!(["expanded"]));
    assert_eq!(
        toggle.input_schema["properties"]["expanded"]["type"],
        "boolean"
    );
    assert_eq!(toggle.event_field_sources, vec!["toggle"]);
    assert_eq!(toggle.target_kinds, vec!["section"]);
    assert!(toggle.required_capabilities.contains(&"view:write".into()));
    let submit = button
        .actions
        .iter()
        .find(|a| a.action_id == "study.submit")
        .unwrap();
    assert_eq!(submit.availability["study"], "capability_unavailable");
    assert_eq!(submit.availability["preview"], "simulated");
    assert_eq!(
        submit.result_schema["properties"]["evidenceWritten"]["const"],
        false
    );
    let section = catalog.iter().find(|d| d.type_id == "section").unwrap();
    assert_eq!(
        section.event_schemas["toggle"].event_fields["value"]["type"],
        "boolean"
    );
    assert_eq!(section.slot_schemas["body"].maximum, 64);
    assert_eq!(
        section.state.schema["properties"]["expanded"]["default"],
        true
    );
    let activity = catalog.iter().find(|d| d.type_id == "activity").unwrap();
    assert_eq!(
        activity.event_schemas["response"].allowed_actions,
        vec!["response.update"]
    );
    assert!(!activity.state.persistent);
    assert!(activity.state.scope.contains("Occurrence"));
    assert_eq!(
        activity.schemas["activityDefinition"]["oneOf"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    for descriptor in catalog {
        assert!(descriptor.fallback.preserves_opaque_payload);
        assert!(!descriptor.fallback.activation_allowed);
        assert_eq!(descriptor.fallback.mode, "inertPlaceholder");
        assert_eq!(descriptor.schemas["condition"]["x-maxNodes"], 32);
        assert!(descriptor.props_schema["required"].is_array());
    }
}
#[test]
fn scoped_validation_reports_unsupported_nodes_without_mutating_or_disclosing_targets() {
    let (_temp, mut db, u) = setup();
    let (saved, _) = saved_opaque(&mut db, &u, false);
    let d = db
        .lesson_create(
            RequestContext::local_ui(),
            &CreateLessonDraft {
                contract_version: 1,
                workspace_id: db.id.clone(),
                unit_id: u.clone(),
                operation_id: OperationId::generate(),
                expected_head_revision: Some(saved.id),
            },
        )
        .unwrap();
    let result = db
        .lesson_validate(RequestContext::local_ui(), &scope(&db, &d))
        .unwrap();
    assert!(!result.valid);
    assert_eq!(result.diagnostics.len(), 16);
    assert!(
        result
            .diagnostics
            .iter()
            .all(|d| d.code == LessonDiagnosticCode::UnsupportedType
                && d.node_id.is_some()
                && d.field == "typeVersion")
    );
    assert_eq!(
        db.lesson_list_drafts(RequestContext::local_ui(), &u)
            .unwrap(),
        vec![d.clone()]
    );
    let mut foreign = scope(&db, &d);
    foreign.workspace_id = crate::workspace::WorkspaceId::generate();
    let denied = db
        .lesson_validate(RequestContext::local_ui(), &foreign)
        .unwrap_err();
    assert_eq!(denied.code, ErrorCode::PermissionDenied);
    assert!(denied.diagnostics.is_empty());
}

#[test]
fn opaque_containment_and_conditions_cannot_be_edited_implicitly() {
    let (_temp, mut db, u) = setup();
    let (saved, c) = saved_opaque(&mut db, &u, true);
    let d = db
        .lesson_create(
            RequestContext::local_ui(),
            &CreateLessonDraft {
                contract_version: 1,
                workspace_id: db.id.clone(),
                unit_id: u.clone(),
                operation_id: OperationId::generate(),
                expected_head_revision: Some(saved.id),
            },
        )
        .unwrap();
    let mut node = c.nodes[0].clone();
    node.node_id = "known-child".into();
    node.type_id = "stack".into();
    node.type_version = 1;
    node.props = json!({"gap":"normal","headingLevel":1});
    node.slots = std::collections::BTreeMap::from([("body".into(), vec![])]);
    assert!(
        db.lesson_edit(
            RequestContext::local_ui(),
            &edit(
                &db,
                &d,
                vec![LessonOperation::InsertNode {
                    parent: c.root_node_id.clone(),
                    slot: "secondary".into(),
                    index: 0,
                    node
                }]
            )
        )
        .is_err()
    );
    assert!(
        db.lesson_edit(
            RequestContext::local_ui(),
            &edit(
                &db,
                &d,
                vec![LessonOperation::ConfigureInteraction {
                    id: c.root_node_id,
                    on: Default::default(),
                    visible: Some(json!({"kind":"literal","value":false}))
                }]
            )
        )
        .is_err()
    );
}
#[test]
fn invalid_condition_diagnostics_echo_only_safe_schema_locations() {
    let (_temp, mut db, u) = setup();
    let d = create(&mut db, &u);
    let secret = "../../unavailable-private-target";
    let e = edit(
        &db,
        &d,
        vec![LessonOperation::ConfigureInteraction {
            id: d.composition.root_node_id.clone(),
            on: Default::default(),
            visible: Some(json!({"kind":"eq","field":secret,"value":true})),
        }],
    );
    let error = db.lesson_edit(RequestContext::local_ui(), &e).unwrap_err();
    assert_eq!(
        error.diagnostics[0].code,
        LessonDiagnosticCode::InvalidCondition
    );
    assert_eq!(
        error.diagnostics[0].node_id.as_deref(),
        Some(d.composition.root_node_id.as_str())
    );
    assert_eq!(error.diagnostics[0].operation_index, Some(0));
    assert_eq!(error.diagnostics[0].field, "visible.field");
    assert!(!serde_json::to_string(&error).unwrap().contains(secret));
}
