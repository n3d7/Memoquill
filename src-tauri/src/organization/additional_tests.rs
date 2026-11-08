use super::*;

struct Fixture {
    org: Organization,
    subject: SubjectId,
    source: CollectionId,
    destination: CollectionId,
    resource: ResourceId,
    replacement_resource: ResourceId,
    goal: GoalId,
    replacement_goal: GoalId,
    unit: UnitId,
    other_unit: UnitId,
    placements: [PlacementId; 3],
    path: PathId,
}

impl Fixture {
    fn new() -> Self {
        let mut fixture = Self {
            org: Organization::new(Uuid::new_v4()),
            subject: SubjectId::generate(),
            source: CollectionId::generate(),
            destination: CollectionId::generate(),
            resource: ResourceId::generate(),
            replacement_resource: ResourceId::generate(),
            goal: GoalId::generate(),
            replacement_goal: GoalId::generate(),
            unit: UnitId::generate(),
            other_unit: UnitId::generate(),
            placements: std::array::from_fn(|_| PlacementId::generate()),
            path: PathId::generate(),
        };
        fixture.send(Command::CreateSubject {
            id: fixture.subject,
            title: "Materials science".into(),
            prior_knowledge: None,
        });
        for id in [fixture.source, fixture.destination] {
            fixture.send(Command::CreateCollection {
                id,
                subject_id: fixture.subject,
                parent_id: None,
                title: "Reading".into(),
            });
        }
        for id in [fixture.resource, fixture.replacement_resource] {
            fixture.send(Command::CreateResource {
                id,
                title: "Crystal structures".into(),
            });
        }
        for id in [fixture.goal, fixture.replacement_goal] {
            fixture.send(Command::CreateGoal {
                id,
                subject_id: fixture.subject,
                title: "Explain a structure".into(),
                criteria: "Use an unfamiliar example".into(),
            });
        }
        for id in [fixture.unit, fixture.other_unit] {
            fixture.send(Command::CreateUnit {
                id,
                title: "Introduction".into(),
                objective: "Compare two structures".into(),
                goal_ids: vec![fixture.goal],
                resource_ids: vec![fixture.resource],
            });
        }
        for (id, collection_id, target) in [
            (
                fixture.placements[0],
                fixture.source,
                Target::Unit(fixture.unit),
            ),
            (
                fixture.placements[1],
                fixture.source,
                Target::Resource(fixture.resource),
            ),
            (
                fixture.placements[2],
                fixture.destination,
                Target::Resource(fixture.resource),
            ),
        ] {
            fixture.send(Command::Place {
                id,
                collection_id,
                target,
            });
        }
        fixture.send(Command::CreatePath {
            id: fixture.path,
            title: "First course".into(),
            steps: vec![PathStep {
                id: StepId::generate(),
                unit_id: fixture.unit,
            }],
        });
        fixture
    }

    fn send(&mut self, command: Command) {
        self.org
            .apply(
                self.org.snapshot().workspace_id,
                self.org.revision(),
                command,
            )
            .unwrap();
    }

    fn reject(&mut self, command: Command, error: DomainError) {
        let before = self.org.clone();
        assert_eq!(
            self.org.apply(
                self.org.snapshot().workspace_id,
                self.org.revision(),
                command
            ),
            Err(error)
        );
        assert_eq!(
            self.org, before,
            "a rejected operation must preserve the entire snapshot and revision"
        );
    }

    fn replacement(&self) -> Command {
        Command::UpdateUnit {
            id: self.unit,
            title: "Advanced structures".into(),
            objective: "Predict a material's properties".into(),
            goal_ids: vec![self.replacement_goal],
            resource_ids: vec![self.replacement_resource],
        }
    }
}

#[test]
fn moving_and_reordering_placements_preserves_identity_targets_and_dense_order() {
    let mut fixture = Fixture::new();
    let [moving, remaining, destination] = fixture.placements;
    let units = fixture.org.snapshot().units.clone();
    let resources = fixture.org.snapshot().resources.clone();
    fixture.send(Command::MovePlacement {
        id: moving,
        collection_id: fixture.destination,
    });
    let placements = &fixture.org.snapshot().placements;
    assert_eq!(placements.len(), 3);
    assert_eq!(placements[&moving].id, moving);
    assert_eq!(placements[&moving].collection_id, fixture.destination);
    assert_eq!(placements[&moving].target, Target::Unit(fixture.unit));
    assert_eq!(placements[&moving].position, 1);
    assert_eq!(placements[&remaining].collection_id, fixture.source);
    assert_eq!(placements[&remaining].position, 0);
    assert_eq!(placements[&destination].position, 0);
    fixture.send(Command::ReorderPlacements {
        collection_id: fixture.destination,
        ordered_ids: vec![moving, destination],
    });
    assert_eq!(fixture.org.snapshot().placements[&moving].position, 0);
    assert_eq!(fixture.org.snapshot().placements[&destination].position, 1);
    fixture.send(Command::MovePlacement {
        id: moving,
        collection_id: fixture.destination,
    });
    assert_eq!(fixture.org.snapshot().placements[&moving].position, 1);
    assert_eq!(fixture.org.snapshot().placements[&destination].position, 0);
    assert_eq!(fixture.org.snapshot().units, units);
    assert_eq!(fixture.org.snapshot().resources, resources);
    for ordered_ids in [vec![moving, moving], vec![moving], vec![moving, remaining]] {
        fixture.reject(
            Command::ReorderPlacements {
                collection_id: fixture.destination,
                ordered_ids,
            },
            DomainError::InvalidRequest,
        );
    }
}

#[test]
fn moving_placements_rejects_missing_or_archived_records_without_side_effects() {
    let mut fixture = Fixture::new();
    fixture.reject(
        Command::MovePlacement {
            id: PlacementId::generate(),
            collection_id: fixture.destination,
        },
        DomainError::NotFound,
    );
    fixture.reject(
        Command::MovePlacement {
            id: fixture.placements[0],
            collection_id: CollectionId::generate(),
        },
        DomainError::NotFound,
    );
    for target in [
        ArchiveTarget::Placement(fixture.placements[0]),
        ArchiveTarget::Unit(fixture.unit),
        ArchiveTarget::Collection(fixture.source),
        ArchiveTarget::Collection(fixture.destination),
    ] {
        fixture.send(Command::Archive {
            target,
            archived: true,
        });
        fixture.reject(
            Command::MovePlacement {
                id: fixture.placements[0],
                collection_id: fixture.destination,
            },
            DomainError::InvalidRequest,
        );
        fixture.send(Command::Archive {
            target,
            archived: false,
        });
    }
}

#[test]
fn updating_a_unit_replaces_content_and_references_while_preserving_its_users() {
    let mut fixture = Fixture::new();
    let placements = fixture.org.snapshot().placements.clone();
    let paths = fixture.org.snapshot().paths.clone();
    let other = fixture.org.snapshot().units[&fixture.other_unit].clone();
    fixture.send(fixture.replacement());
    assert_eq!(
        fixture.org.snapshot().units[&fixture.unit],
        Unit {
            id: fixture.unit,
            title: "Advanced structures".into(),
            objective: "Predict a material's properties".into(),
            goal_ids: vec![fixture.replacement_goal],
            resource_ids: vec![fixture.replacement_resource],
            archived: false,
        }
    );
    assert_eq!(fixture.org.snapshot().units[&fixture.other_unit], other);
    assert_eq!(fixture.org.snapshot().placements, placements);
    assert_eq!(fixture.org.snapshot().paths, paths);
}

#[test]
fn invalid_or_archived_unit_updates_are_atomic() {
    let mut fixture = Fixture::new();
    fixture.reject(
        Command::UpdateUnit {
            id: fixture.unit,
            title: "Invalid reference".into(),
            objective: "Test a missing source".into(),
            goal_ids: vec![fixture.goal],
            resource_ids: vec![ResourceId::generate()],
        },
        DomainError::NotFound,
    );
    fixture.reject(
        Command::UpdateUnit {
            id: UnitId::generate(),
            title: "Missing unit".into(),
            objective: "Keep existing records".into(),
            goal_ids: vec![],
            resource_ids: vec![],
        },
        DomainError::NotFound,
    );
    for target in [
        ArchiveTarget::Goal(fixture.replacement_goal),
        ArchiveTarget::Resource(fixture.replacement_resource),
        ArchiveTarget::Unit(fixture.unit),
    ] {
        fixture.send(Command::Archive {
            target,
            archived: true,
        });
        fixture.reject(fixture.replacement(), DomainError::InvalidRequest);
        fixture.send(Command::Archive {
            target,
            archived: false,
        });
    }
}

#[test]
fn path_step_replacement_preserves_units_and_rejects_duplicate_or_inactive_steps() {
    let mut fixture = Fixture::new();
    let units = fixture.org.snapshot().units.clone();
    let steps = vec![
        PathStep {
            id: StepId::generate(),
            unit_id: fixture.other_unit,
        },
        PathStep {
            id: StepId::generate(),
            unit_id: fixture.unit,
        },
    ];
    fixture.send(Command::SetPathSteps {
        id: fixture.path,
        steps: steps.clone(),
    });
    assert_eq!(fixture.org.snapshot().paths[&fixture.path].id, fixture.path);
    assert_eq!(
        fixture.org.snapshot().paths[&fixture.path].title,
        "First course"
    );
    assert_eq!(fixture.org.snapshot().paths[&fixture.path].steps, steps);
    assert_eq!(fixture.org.snapshot().units, units);
    fixture.reject(
        Command::SetPathSteps {
            id: fixture.path,
            steps: vec![steps[0].clone(), steps[0].clone()],
        },
        DomainError::InvalidRequest,
    );
    fixture.reject(
        Command::SetPathSteps {
            id: fixture.path,
            steps: vec![PathStep {
                id: StepId::generate(),
                unit_id: UnitId::generate(),
            }],
        },
        DomainError::NotFound,
    );
    fixture.reject(
        Command::SetPathSteps {
            id: PathId::generate(),
            steps: steps.clone(),
        },
        DomainError::NotFound,
    );
    for target in [
        ArchiveTarget::Unit(fixture.unit),
        ArchiveTarget::Path(fixture.path),
    ] {
        fixture.send(Command::Archive {
            target,
            archived: true,
        });
        fixture.reject(
            Command::SetPathSteps {
                id: fixture.path,
                steps: steps.clone(),
            },
            DomainError::InvalidRequest,
        );
        fixture.send(Command::Archive {
            target,
            archived: false,
        });
    }
}

#[test]
fn renaming_resources_keeps_references_and_rejects_archived_or_missing_targets() {
    let mut fixture = Fixture::new();
    let placements = fixture.org.snapshot().placements.clone();
    let units = fixture.org.snapshot().units.clone();
    fixture.send(Command::RenameResource {
        id: fixture.resource,
        title: "New source title".into(),
    });
    assert_eq!(
        fixture.org.snapshot().resources[&fixture.resource].id,
        fixture.resource
    );
    assert_eq!(
        fixture.org.snapshot().resources[&fixture.resource].title,
        "New source title"
    );
    assert_eq!(fixture.org.snapshot().placements, placements);
    assert_eq!(fixture.org.snapshot().units, units);
    fixture.reject(
        Command::RenameResource {
            id: ResourceId::generate(),
            title: "Missing".into(),
        },
        DomainError::NotFound,
    );
    fixture.send(Command::Archive {
        target: ArchiveTarget::Resource(fixture.resource),
        archived: true,
    });
    fixture.reject(
        Command::RenameResource {
            id: fixture.resource,
            title: "Archived".into(),
        },
        DomainError::InvalidRequest,
    );
}

#[test]
fn archiving_each_target_kind_is_reversible_without_losing_references() {
    let mut fixture = Fixture::new();
    for target in [
        ArchiveTarget::Subject(fixture.subject),
        ArchiveTarget::Goal(fixture.goal),
        ArchiveTarget::Collection(fixture.source),
        ArchiveTarget::Resource(fixture.resource),
        ArchiveTarget::Placement(fixture.placements[0]),
        ArchiveTarget::Unit(fixture.unit),
        ArchiveTarget::Path(fixture.path),
    ] {
        let mut expected = fixture.org.snapshot().clone();
        expected.revision += 2;
        fixture.send(Command::Archive {
            target,
            archived: true,
        });
        let snapshot = fixture.org.snapshot();
        assert!(match target {
            ArchiveTarget::Subject(id) => snapshot.subjects[&id].archived,
            ArchiveTarget::Goal(id) => snapshot.goals[&id].archived,
            ArchiveTarget::Collection(id) => snapshot.collections[&id].archived,
            ArchiveTarget::Resource(id) => snapshot.resources[&id].archived,
            ArchiveTarget::Placement(id) => snapshot.placements[&id].archived,
            ArchiveTarget::Unit(id) => snapshot.units[&id].archived,
            ArchiveTarget::Path(id) => snapshot.paths[&id].archived,
        });
        fixture.send(Command::Archive {
            target,
            archived: false,
        });
        assert_eq!(fixture.org.snapshot(), &expected);
    }
}
