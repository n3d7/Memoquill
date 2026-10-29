//! Subject-independent organization. Changes are atomic and preserve logical identity.
mod collections;
mod types;
mod units;
mod validation;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub use types::*;
use uuid::Uuid;

pub const MAX_REVISION: u64 = 9_007_199_254_740_991;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DomainError {
    InvalidRequest,
    PermissionDenied,
    RevisionConflict,
    NotFound,
    Cycle,
}
impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for DomainError {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Snapshot", into = "Snapshot")]
pub struct Organization(Snapshot);

impl Organization {
    pub fn new(workspace_id: Uuid) -> Self {
        Self(Snapshot {
            workspace_id,
            revision: 0,
            subjects: BTreeMap::new(),
            goals: BTreeMap::new(),
            collections: BTreeMap::new(),
            resources: BTreeMap::new(),
            placements: BTreeMap::new(),
            units: BTreeMap::new(),
            paths: BTreeMap::new(),
            relations: BTreeMap::new(),
            tags: BTreeMap::new(),
        })
    }
    pub fn revision(&self) -> u64 {
        self.0.revision
    }
    pub fn snapshot(&self) -> &Snapshot {
        &self.0
    }
    pub fn shared_impact(&self, id: ResourceId) -> Result<SharedImpact, DomainError> {
        if !self.0.resources.contains_key(&id) {
            return Err(DomainError::NotFound);
        }
        let unit_ids: Vec<_> = self
            .0
            .units
            .values()
            .filter(|unit| unit.resource_ids.contains(&id))
            .map(|unit| unit.id)
            .collect();
        let placements: Vec<_> = self
            .0
            .placements
            .values()
            .filter(|p| match p.target {
                Target::Resource(resource) => resource == id,
                Target::Unit(unit) => unit_ids.contains(&unit),
            })
            .collect();
        let subject_ids = placements
            .iter()
            .filter_map(|p| {
                self.0
                    .collections
                    .get(&p.collection_id)
                    .map(|c| c.subject_id)
            })
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        let path_ids = self
            .0
            .paths
            .values()
            .filter(|path| {
                path.steps
                    .iter()
                    .any(|step| unit_ids.contains(&step.unit_id))
            })
            .map(|path| path.id)
            .collect();
        Ok(SharedImpact {
            placement_ids: placements.into_iter().map(|p| p.id).collect(),
            unit_ids,
            path_ids,
            subject_ids,
        })
    }
    pub fn from_snapshot(snapshot: Snapshot) -> Result<Self, DomainError> {
        let value = Self(snapshot);
        value.validate()?;
        Ok(value)
    }
    pub fn apply(
        &mut self,
        workspace: Uuid,
        expected_revision: u64,
        command: Command,
    ) -> Result<(), DomainError> {
        if workspace != self.0.workspace_id {
            return Err(DomainError::PermissionDenied);
        }
        if expected_revision != self.0.revision || expected_revision >= MAX_REVISION {
            return Err(DomainError::RevisionConflict);
        }
        let mut next = self.clone();
        next.apply_command(command)?;
        next.0.revision += 1;
        next.validate()?;
        *self = next;
        Ok(())
    }
    fn apply_command(&mut self, command: Command) -> Result<(), DomainError> {
        match command {
            Command::CreateSubject {
                id,
                title,
                prior_knowledge,
            } => {
                if self.0.subjects.contains_key(&id) {
                    return Err(DomainError::InvalidRequest);
                }
                self.0.subjects.insert(
                    id,
                    Subject {
                        id,
                        title,
                        prior_knowledge,
                        archived: false,
                    },
                );
                Ok(())
            }
            Command::UpdateSubject {
                id,
                title,
                prior_knowledge,
            } => {
                let item = self.0.subjects.get_mut(&id).ok_or(DomainError::NotFound)?;
                item.title = title;
                item.prior_knowledge = prior_knowledge;
                Ok(())
            }
            Command::CreateGoal {
                id,
                subject_id,
                title,
                criteria,
            } => {
                self.active_subject(subject_id)?;
                insert(
                    &mut self.0.goals,
                    id,
                    Goal {
                        id,
                        subject_id,
                        title,
                        criteria,
                        archived: false,
                    },
                )
            }
            other => self.apply_collections(other),
        }
    }
}
impl TryFrom<Snapshot> for Organization {
    type Error = DomainError;
    fn try_from(value: Snapshot) -> Result<Self, Self::Error> {
        Self::from_snapshot(value)
    }
}
impl From<Organization> for Snapshot {
    fn from(value: Organization) -> Self {
        value.0
    }
}
fn title(value: &str) -> Result<(), DomainError> {
    if value.trim().is_empty() || value.contains('\0') || value.chars().count() > 256 {
        Err(DomainError::InvalidRequest)
    } else {
        Ok(())
    }
}
fn insert<K: Ord, V>(map: &mut BTreeMap<K, V>, id: K, value: V) -> Result<(), DomainError> {
    if map.contains_key(&id) {
        return Err(DomainError::InvalidRequest);
    }
    map.insert(id, value);
    Ok(())
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod additional_tests;
#[cfg(test)]
mod initial_tests {
    use super::*;
    #[test]
    fn blank_subjects_use_one_contract_without_templates() {
        for title in ["Quantum physics", "English", "Python", "Cooking"] {
            let workspace = Uuid::new_v4();
            let mut organization = Organization::new(workspace);
            let id = SubjectId::generate();
            organization
                .apply(
                    workspace,
                    0,
                    Command::CreateSubject {
                        id,
                        title: title.into(),
                        prior_knowledge: None,
                    },
                )
                .unwrap();
            assert_eq!(organization.revision(), 1);
            assert_eq!(organization.snapshot().subjects[&id].title, title);
        }
    }
}
