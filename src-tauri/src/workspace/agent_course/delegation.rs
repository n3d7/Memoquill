//! The desktop lease and the database writer share a commit/revocation lock.
use super::*;
use std::{
    collections::HashSet,
    sync::{Mutex, MutexGuard},
    time::{Duration, Instant},
};

const MAX_OPERATIONS: usize = 32;

pub(in crate::workspace) struct Delegation(Mutex<Policy>);

pub(super) struct Policy {
    expires: Instant,
    remaining: usize,
    revoked: bool,
}

impl Delegation {
    pub(in crate::workspace) fn new(duration: Duration) -> Self {
        Self(Mutex::new(Policy {
            expires: Instant::now() + duration,
            remaining: MAX_OPERATIONS,
            revoked: false,
        }))
    }

    pub(in crate::workspace) fn revoke(&self) {
        if let Ok(mut policy) = self.0.lock() {
            policy.revoked = true;
        }
    }

    pub(super) fn lock(&self) -> Result<MutexGuard<'_, Policy>> {
        self.0
            .lock()
            .map_err(|_| AppError::new(ErrorCode::PermissionDenied))
    }
}

impl Policy {
    pub(super) fn reserve(&mut self, operations: usize) -> Result<()> {
        self.authorize()?;
        self.remaining = self
            .remaining
            .checked_sub(operations)
            .ok_or_else(|| AppError::new(ErrorCode::BudgetExceeded))?;
        Ok(())
    }

    pub(super) fn authorize(&self) -> Result<()> {
        if self.revoked || self.expires <= Instant::now() {
            return Err(AppError::new(ErrorCode::PermissionDenied));
        }
        Ok(())
    }
}

/// Only new objects and their organization are eligible. Links to existing
/// objects never give permission to rewrite or archive those objects.
pub(super) fn allows(record: &Record) -> bool {
    use crate::organization::Target;
    let mut subjects = HashSet::new();
    let mut goals = HashSet::new();
    let mut collections = HashSet::new();
    let mut resources = HashSet::new();
    let mut units = HashSet::new();
    let mut paths = HashSet::new();
    let mut placements = HashSet::new();
    for operation in record.chunks.iter().flat_map(|chunk| &chunk.operations) {
        let eligible = match operation {
            CourseOperation::Document { resource_id, .. } => resources.contains(resource_id),
            CourseOperation::Organization { command } => match command {
                Command::CreateSubject { id, .. } => subjects.insert(*id),
                Command::UpdateSubject { id, .. } => subjects.contains(id),
                Command::CreateGoal { id, .. } => goals.insert(*id),
                Command::UpdateGoal { id, .. } => goals.contains(id),
                Command::CreateCollection { id, .. } => collections.insert(*id),
                Command::RenameCollection { id, .. } | Command::MoveCollection { id, .. } => {
                    collections.contains(id)
                }
                Command::CreateResource { id, .. } => resources.insert(*id),
                Command::RenameResource { id, .. } => resources.contains(id),
                Command::CreateUnit { id, .. } => units.insert(*id),
                Command::UpdateUnit { id, .. } => units.contains(id),
                Command::CreatePath { id, .. } => paths.insert(*id),
                Command::RenamePath { id, .. } | Command::SetPathSteps { id, .. } => {
                    paths.contains(id)
                }
                Command::Relate { from, to, .. } => units.contains(from) && units.contains(to),
                Command::Place {
                    id,
                    collection_id,
                    target,
                    ..
                } => {
                    let new_target = match target {
                        Target::Unit(id) => units.contains(id),
                        Target::Resource(id) => resources.contains(id),
                    };
                    (new_target || collections.contains(collection_id)) && placements.insert(*id)
                }
                Command::MovePlacement { id, .. } => placements.contains(id),
                Command::ReorderPlacements {
                    collection_id,
                    ordered_ids,
                } => {
                    collections.contains(collection_id)
                        && ordered_ids.iter().all(|id| placements.contains(id))
                }
                _ => false,
            },
        };
        if !eligible {
            return false;
        }
    }
    true
}
