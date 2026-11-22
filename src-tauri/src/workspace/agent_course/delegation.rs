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
    revise_owned_documents: bool,
}

impl Delegation {
    pub(in crate::workspace) fn new(duration: Duration, revise_owned_documents: bool) -> Self {
        Self(Mutex::new(Policy {
            expires: Instant::now() + duration,
            remaining: MAX_OPERATIONS,
            revoked: false,
            revise_owned_documents,
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
    pub(super) fn can_revise_owned_documents(&self) -> bool {
        self.revise_owned_documents
    }

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

/// Delegation covers additions and their organization. A separate permission
/// allows revisions to unlinked documents with a verifiable owner and head.
pub(super) fn allows(
    connection: &Connection,
    record: &Record,
    actor: &str,
    revise_owned_documents: bool,
) -> Result<bool> {
    use crate::organization::Target;
    let mut subjects = HashSet::new();
    let mut goals = HashSet::new();
    let mut collections = HashSet::new();
    let mut resources = HashSet::new();
    let mut units = HashSet::new();
    let mut owned_documents = HashSet::new();
    let mut paths = HashSet::new();
    let mut placements = HashSet::new();
    for operation in record.chunks.iter().flat_map(|chunk| &chunk.operations) {
        let eligible = match operation {
            CourseOperation::Document { resource_id, .. } => {
                if resources.contains(resource_id) || owned_documents.contains(resource_id) {
                    true
                } else if revise_owned_documents
                    && owns_unlinked_document(connection, actor, *resource_id)?
                {
                    owned_documents.insert(*resource_id);
                    true
                } else {
                    false
                }
            }
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
            return Ok(false);
        }
    }
    Ok(true)
}

fn owns_unlinked_document(
    connection: &Connection,
    actor: &str,
    resource: ResourceId,
) -> Result<bool> {
    let id = resource.to_string();
    if resource_has_external_references(connection, resource)? {
        return Ok(false);
    }
    let title: Option<String> = connection.query_row(
        "SELECT r.title FROM resources r JOIN resource_heads h ON h.resource_id=r.id
         JOIN agent_authored_documents a ON a.revision_id=h.revision_id
         JOIN agent_course_drafts d ON d.id=a.proposal_id
         WHERE r.id=?1 AND r.archived=0 AND d.actor=?2 AND json_extract(d.data,'$.status')='applied'",
         params![id, actor], |row| row.get(0),
    ).optional()?;
    let Some(title) = title else { return Ok(false) };
    // A document review on a manually created resource grants no ownership.
    // Require the actor's original creation and the unmodified resource title.
    let mut query = connection.prepare(
        "SELECT d.id,d.data FROM agent_course_drafts d
         WHERE d.actor=?1 AND json_extract(d.data,'$.status')='applied' ORDER BY d.id LIMIT 65",
    )?;
    let mut rows = query.query([actor])?;
    let mut checked = 0;
    while let Some(row) = rows.next()? {
        checked += 1;
        if checked > MAX_DRAFTS {
            return Err(corrupt());
        }
        let id: String = row.get(0)?;
        let data: String = row.get(1)?;
        let earlier = decode::<Record>(&data)?;
        earlier.validate(Uuid::parse_str(&id).map_err(|_| corrupt())?, actor)?;
        if earlier.chunks.iter().flat_map(|chunk| &chunk.operations).any(|operation| matches!(
            operation,
            CourseOperation::Organization { command: Command::CreateResource { id, title: original } }
                if *id == resource && original == &title
        )) {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(super) fn resource_has_external_references(
    connection: &Connection,
    resource: ResourceId,
) -> Result<bool> {
    Ok(connection.query_row(
        "SELECT EXISTS(
            SELECT 1 FROM unit_resources WHERE resource_id=?1
            UNION ALL SELECT 1 FROM placements WHERE resource_id=?1
            UNION ALL SELECT 1 FROM original_sources WHERE resource_id=?1
            UNION ALL SELECT 1 FROM lesson_dependencies WHERE kind='resource' AND target_id=?1
            UNION ALL SELECT 1 FROM lesson_drafts d,json_each(d.data,'$.draft.composition.bindings') b
                WHERE json_extract(b.value,'$.targetKind')='resource' AND json_extract(b.value,'$.targetId')=?1
        )",
        [resource.to_string()], |row| row.get(0),
    )?)
}
