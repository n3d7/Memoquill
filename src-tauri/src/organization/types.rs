use super::DomainError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

macro_rules! identity {
    ($($name:ident),+ $(,)?) => {$ (
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(try_from = "Uuid", into = "Uuid")]
        pub struct $name(Uuid);
        impl $name {
            pub fn generate() -> Self { Self(Uuid::new_v4()) }
            pub fn as_uuid(self) -> Uuid { self.0 }
        }
        impl TryFrom<Uuid> for $name {
            type Error = DomainError;
            fn try_from(value: Uuid) -> Result<Self, Self::Error> {
                if value.is_nil() { Err(DomainError::InvalidRequest) } else { Ok(Self(value)) }
            }
        }
        impl From<$name> for Uuid { fn from(value: $name) -> Uuid { value.0 } }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { self.0.fmt(f) }
        }
    )+};
}
identity!(
    SubjectId,
    GoalId,
    CollectionId,
    ResourceId,
    PlacementId,
    UnitId,
    PathId,
    StepId,
    RelationId,
    TagId
);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Subject {
    pub id: SubjectId,
    pub title: String,
    pub prior_knowledge: Option<String>,
    pub archived: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Goal {
    pub id: GoalId,
    pub subject_id: SubjectId,
    pub title: String,
    pub criteria: String,
    pub archived: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Collection {
    pub id: CollectionId,
    pub subject_id: SubjectId,
    pub parent_id: Option<CollectionId>,
    pub title: String,
    pub position: u32,
    pub archived: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Resource {
    pub id: ResourceId,
    pub title: String,
    pub archived: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "id",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum Target {
    Resource(ResourceId),
    Unit(UnitId),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Placement {
    pub id: PlacementId,
    pub collection_id: CollectionId,
    pub target: Target,
    pub position: u32,
    pub archived: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Unit {
    pub id: UnitId,
    pub title: String,
    pub objective: String,
    pub goal_ids: Vec<GoalId>,
    pub resource_ids: Vec<ResourceId>,
    pub archived: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PathStep {
    pub id: StepId,
    pub unit_id: UnitId,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Path {
    pub id: PathId,
    pub title: String,
    pub steps: Vec<PathStep>,
    pub archived: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RelationKind {
    Prerequisite,
    Related,
    Reference,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Relation {
    pub id: RelationId,
    pub kind: RelationKind,
    pub from: UnitId,
    pub to: UnitId,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Tag {
    pub id: TagId,
    pub title: String,
    pub resource_ids: Vec<ResourceId>,
}

/// Transfer/persistence representation. Rehydrate only through Organization::from_snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Snapshot {
    pub workspace_id: Uuid,
    pub revision: u64,
    pub subjects: BTreeMap<SubjectId, Subject>,
    pub goals: BTreeMap<GoalId, Goal>,
    pub collections: BTreeMap<CollectionId, Collection>,
    pub resources: BTreeMap<ResourceId, Resource>,
    pub placements: BTreeMap<PlacementId, Placement>,
    pub units: BTreeMap<UnitId, Unit>,
    pub paths: BTreeMap<PathId, Path>,
    pub relations: BTreeMap<RelationId, Relation>,
    pub tags: BTreeMap<TagId, Tag>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Command {
    CreateSubject {
        id: SubjectId,
        title: String,
        prior_knowledge: Option<String>,
    },
    UpdateSubject {
        id: SubjectId,
        title: String,
        prior_knowledge: Option<String>,
    },
    CreateGoal {
        id: GoalId,
        subject_id: SubjectId,
        title: String,
        criteria: String,
    },
    UpdateGoal {
        id: GoalId,
        title: String,
        criteria: String,
    },
    CreateCollection {
        id: CollectionId,
        subject_id: SubjectId,
        parent_id: Option<CollectionId>,
        title: String,
    },
    MoveCollection {
        id: CollectionId,
        subject_id: SubjectId,
        parent_id: Option<CollectionId>,
    },
    RenameCollection {
        id: CollectionId,
        title: String,
    },
    ReorderCollections {
        subject_id: SubjectId,
        parent_id: Option<CollectionId>,
        ordered_ids: Vec<CollectionId>,
    },
    CreateResource {
        id: ResourceId,
        title: String,
    },
    RenameResource {
        id: ResourceId,
        title: String,
    },
    CreateUnit {
        id: UnitId,
        title: String,
        objective: String,
        goal_ids: Vec<GoalId>,
        resource_ids: Vec<ResourceId>,
    },
    UpdateUnit {
        id: UnitId,
        title: String,
        objective: String,
        goal_ids: Vec<GoalId>,
        resource_ids: Vec<ResourceId>,
    },
    Place {
        id: PlacementId,
        collection_id: CollectionId,
        target: Target,
    },
    MovePlacement {
        id: PlacementId,
        collection_id: CollectionId,
    },
    ReorderPlacements {
        collection_id: CollectionId,
        ordered_ids: Vec<PlacementId>,
    },
    CreatePath {
        id: PathId,
        title: String,
        steps: Vec<PathStep>,
    },
    SetPathSteps {
        id: PathId,
        steps: Vec<PathStep>,
    },
    RenamePath {
        id: PathId,
        title: String,
    },
    Relate {
        id: RelationId,
        kind: RelationKind,
        from: UnitId,
        to: UnitId,
    },
    RemoveRelation {
        id: RelationId,
    },
    SetTag {
        id: TagId,
        title: String,
        resource_ids: Vec<ResourceId>,
    },
    Archive {
        target: ArchiveTarget,
        archived: bool,
    },
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "id",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum ArchiveTarget {
    Subject(SubjectId),
    Goal(GoalId),
    Collection(CollectionId),
    Resource(ResourceId),
    Placement(PlacementId),
    Unit(UnitId),
    Path(PathId),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SharedImpact {
    pub placement_ids: Vec<PlacementId>,
    pub unit_ids: Vec<UnitId>,
    pub path_ids: Vec<PathId>,
    pub subject_ids: Vec<SubjectId>,
}
