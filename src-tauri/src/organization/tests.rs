use super::*;

fn apply(org: &mut Organization, command: Command) {
    org.apply(org.snapshot().workspace_id, org.revision(), command)
        .unwrap();
}
fn subject(org: &mut Organization, name: &str) -> SubjectId {
    let id = SubjectId::generate();
    apply(
        org,
        Command::CreateSubject {
            id,
            title: name.into(),
            prior_knowledge: None,
        },
    );
    id
}
fn collection(
    org: &mut Organization,
    subject_id: SubjectId,
    parent_id: Option<CollectionId>,
) -> CollectionId {
    let id = CollectionId::generate();
    apply(
        org,
        Command::CreateCollection {
            id,
            subject_id,
            parent_id,
            title: "Collection".into(),
        },
    );
    id
}

#[test]
fn collection_subtree_move_preserves_shared_resource_identity_and_rejects_cycles() {
    let mut org = Organization::new(Uuid::new_v4());
    let first = subject(&mut org, "Quantum physics");
    let second = subject(&mut org, "Cooking");
    let root = collection(&mut org, first, None);
    let child = collection(&mut org, first, Some(root));
    let elsewhere = collection(&mut org, second, None);
    let resource = ResourceId::generate();
    apply(
        &mut org,
        Command::CreateResource {
            id: resource,
            title: "Shared explanation".into(),
        },
    );
    let placement = PlacementId::generate();
    apply(
        &mut org,
        Command::Place {
            id: placement,
            collection_id: child,
            target: Target::Resource(resource),
        },
    );
    apply(
        &mut org,
        Command::Place {
            id: PlacementId::generate(),
            collection_id: elsewhere,
            target: Target::Resource(resource),
        },
    );
    let before = org.clone();
    assert_eq!(
        org.apply(
            org.snapshot().workspace_id,
            org.revision(),
            Command::MoveCollection {
                id: root,
                subject_id: first,
                parent_id: Some(child)
            }
        ),
        Err(DomainError::Cycle)
    );
    assert_eq!(org, before);
    apply(
        &mut org,
        Command::MoveCollection {
            id: root,
            subject_id: second,
            parent_id: Some(elsewhere),
        },
    );
    assert_eq!(org.snapshot().collections[&child].subject_id, second);
    assert_eq!(
        org.snapshot().placements[&placement].target,
        Target::Resource(resource)
    );
    assert_eq!(org.snapshot().resources.len(), 1);
    assert_eq!(
        org.shared_impact(resource).unwrap().subject_ids,
        vec![second]
    );
}

#[test]
fn stale_foreign_and_invalid_reorder_requests_leave_the_snapshot_unchanged() {
    let workspace = Uuid::new_v4();
    let mut org = Organization::new(workspace);
    let scope = subject(&mut org, "English");
    let first = collection(&mut org, scope, None);
    let second = collection(&mut org, scope, None);
    let before = org.clone();
    let command = Command::ReorderCollections {
        subject_id: scope,
        parent_id: None,
        ordered_ids: vec![second, first],
    };
    assert_eq!(
        org.apply(Uuid::new_v4(), org.revision(), command.clone()),
        Err(DomainError::PermissionDenied)
    );
    assert_eq!(
        org.apply(workspace, 0, command.clone()),
        Err(DomainError::RevisionConflict)
    );
    assert_eq!(
        org.apply(
            workspace,
            org.revision(),
            Command::ReorderCollections {
                subject_id: scope,
                parent_id: None,
                ordered_ids: vec![first, first]
            }
        ),
        Err(DomainError::InvalidRequest)
    );
    assert_eq!(org, before);
    apply(&mut org, command);
    assert_eq!(org.snapshot().collections[&second].position, 0);
    assert_eq!(org.snapshot().collections[&first].position, 1);
}

fn unit(org: &mut Organization, resources: Vec<ResourceId>) -> UnitId {
    let id = UnitId::generate();
    apply(
        org,
        Command::CreateUnit {
            id,
            title: "Concept".into(),
            objective: "Explain it".into(),
            goal_ids: vec![],
            resource_ids: resources,
        },
    );
    id
}

#[test]
fn prerequisites_form_a_dag_but_related_links_can_cycle() {
    let mut org = Organization::new(Uuid::new_v4());
    let a = unit(&mut org, vec![]);
    let b = unit(&mut org, vec![]);
    apply(
        &mut org,
        Command::Relate {
            id: RelationId::generate(),
            kind: RelationKind::Prerequisite,
            from: a,
            to: b,
        },
    );
    let before = org.clone();
    assert_eq!(
        org.apply(
            org.snapshot().workspace_id,
            org.revision(),
            Command::Relate {
                id: RelationId::generate(),
                kind: RelationKind::Prerequisite,
                from: b,
                to: a
            }
        ),
        Err(DomainError::Cycle)
    );
    assert_eq!(org, before);
    for (from, to) in [(a, b), (b, a)] {
        apply(
            &mut org,
            Command::Relate {
                id: RelationId::generate(),
                kind: RelationKind::Related,
                from,
                to,
            },
        );
    }
    let path = PathId::generate();
    let steps = vec![
        PathStep {
            id: StepId::generate(),
            unit_id: a,
        },
        PathStep {
            id: StepId::generate(),
            unit_id: a,
        },
    ];
    apply(
        &mut org,
        Command::CreatePath {
            id: path,
            title: "Repeat intentionally".into(),
            steps: steps.clone(),
        },
    );
    assert_eq!(org.snapshot().paths[&path].steps, steps);
    assert_eq!(
        serde_json::from_str::<Organization>(&serde_json::to_string(&org).unwrap()).unwrap(),
        org
    );
}

#[test]
fn archiving_preserves_goal_resource_unit_path_and_placement_references() {
    let mut org = Organization::new(Uuid::new_v4());
    let scope = subject(&mut org, "Python");
    let goal = GoalId::generate();
    apply(
        &mut org,
        Command::CreateGoal {
            id: goal,
            subject_id: scope,
            title: "Understand functions".into(),
            criteria: "Explain a new example".into(),
        },
    );
    let root = collection(&mut org, scope, None);
    let resource = ResourceId::generate();
    apply(
        &mut org,
        Command::CreateResource {
            id: resource,
            title: "Functions".into(),
        },
    );
    let unit = unit(&mut org, vec![resource]);
    let placement = PlacementId::generate();
    apply(
        &mut org,
        Command::Place {
            id: placement,
            collection_id: root,
            target: Target::Unit(unit),
        },
    );
    let tag = TagId::generate();
    apply(
        &mut org,
        Command::SetTag {
            id: tag,
            title: "First steps".into(),
            resource_ids: vec![resource],
        },
    );
    let path = PathId::generate();
    apply(
        &mut org,
        Command::CreatePath {
            id: path,
            title: "Path".into(),
            steps: vec![PathStep {
                id: StepId::generate(),
                unit_id: unit,
            }],
        },
    );
    apply(
        &mut org,
        Command::Archive {
            target: ArchiveTarget::Resource(resource),
            archived: true,
        },
    );
    assert!(org.snapshot().resources[&resource].archived);
    assert_eq!(
        org.snapshot().placements[&placement].target,
        Target::Unit(unit)
    );
    assert_eq!(org.snapshot().units[&unit].resource_ids, vec![resource]);
    assert_eq!(org.shared_impact(resource).unwrap().path_ids, vec![path]);
    assert_eq!(org.snapshot().tags[&tag].resource_ids, vec![resource]);
    assert!(org.snapshot().goals.contains_key(&goal));
    let before = org.clone();
    assert_eq!(
        org.apply(
            org.snapshot().workspace_id,
            org.revision(),
            Command::Place {
                id: PlacementId::generate(),
                collection_id: root,
                target: Target::Resource(resource)
            }
        ),
        Err(DomainError::InvalidRequest)
    );
    assert_eq!(org, before);
}

#[test]
fn archived_ancestors_hide_descendants_and_reject_new_work() {
    let mut org = Organization::new(Uuid::new_v4());
    let scope = subject(&mut org, "Cooking");
    let root = collection(&mut org, scope, None);
    let child = collection(&mut org, scope, Some(root));
    apply(
        &mut org,
        Command::Archive {
            target: ArchiveTarget::Collection(root),
            archived: true,
        },
    );
    let before = org.clone();
    assert_eq!(
        org.apply(
            org.snapshot().workspace_id,
            org.revision(),
            Command::CreateCollection {
                id: CollectionId::generate(),
                subject_id: scope,
                parent_id: Some(child),
                title: "Hidden work".into()
            }
        ),
        Err(DomainError::InvalidRequest)
    );
    assert_eq!(org, before);
}

#[test]
fn rehydration_rejects_dangling_references_key_mismatch_and_invalid_orders() {
    let mut org = Organization::new(Uuid::new_v4());
    let scope = subject(&mut org, "English");
    let root = collection(&mut org, scope, None);
    let mut snapshot = org.snapshot().clone();
    snapshot.collections.get_mut(&root).unwrap().position = 3;
    assert_eq!(
        Organization::from_snapshot(snapshot),
        Err(DomainError::InvalidRequest)
    );
    let mut snapshot = org.snapshot().clone();
    snapshot.collections.get_mut(&root).unwrap().parent_id = Some(CollectionId::generate());
    assert_eq!(
        Organization::from_snapshot(snapshot),
        Err(DomainError::NotFound)
    );
    let mut snapshot = org.snapshot().clone();
    snapshot.subjects.get_mut(&scope).unwrap().id = SubjectId::generate();
    assert_eq!(
        Organization::from_snapshot(snapshot),
        Err(DomainError::InvalidRequest)
    );
}

#[test]
fn typed_ids_and_command_payloads_reject_ambiguous_or_nil_identity() {
    assert!(serde_json::from_str::<SubjectId>("\"00000000-0000-0000-0000-000000000000\"").is_err());
    assert!(serde_json::from_str::<Command>("{\"type\":\"createSubject\",\"id\":\"11111111-1111-4111-8111-111111111111\",\"title\":\"Physics\",\"trusted_ui\":true}").is_err());
    assert!(
        serde_json::from_str::<Organization>(
            "{\"workspaceId\":\"11111111-1111-4111-8111-111111111111\"}"
        )
        .is_err()
    );
}

#[test]
fn randomized_moves_and_reorders_preserve_all_ids_and_reject_cycles_atomically() {
    let mut org = Organization::new(Uuid::new_v4());
    let a = subject(&mut org, "Quantum physics");
    let b = subject(&mut org, "Python");
    let mut ids = vec![];
    for _ in 0..12 {
        ids.push(collection(&mut org, a, None));
    }
    let expected: std::collections::BTreeSet<_> = ids.iter().copied().collect();
    let mut seed = 0x1234_5678_u64;
    for _ in 0..250 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let id = ids[(seed as usize) % ids.len()];
        let parent = ids[((seed >> 12) as usize) % ids.len()];
        let subject = org.snapshot().collections[&parent].subject_id;
        let before = org.clone();
        let result = org.apply(
            org.snapshot().workspace_id,
            org.revision(),
            Command::MoveCollection {
                id,
                subject_id: subject,
                parent_id: Some(parent),
            },
        );
        if result.is_err() {
            assert_eq!(org, before);
        }
        assert_eq!(
            org.snapshot()
                .collections
                .keys()
                .copied()
                .collect::<std::collections::BTreeSet<_>>(),
            expected
        );
        assert_eq!(
            Organization::from_snapshot(org.snapshot().clone()).unwrap(),
            org
        );
    }
    apply(
        &mut org,
        Command::MoveCollection {
            id: ids[0],
            subject_id: b,
            parent_id: None,
        },
    );
    assert_eq!(org.snapshot().collections[&ids[0]].subject_id, b);
}
