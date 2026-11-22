//! Compensation for isolated additions. Immutable document revisions are retained.
use super::*;
use crate::{
    organization::{ArchiveTarget, Organization, Snapshot},
    workspace::{
        WorkspaceId,
        content::{domain_error, organization_store},
    },
};

pub(super) fn preview(
    connection: &Connection,
    workspace: &WorkspaceId,
    record: &Record,
) -> Result<Snapshot> {
    let stale = || AppError::new(ErrorCode::RevisionConflict);
    let receipt = record
        .commit
        .as_ref()
        .filter(|_| record.status == Status::Applied)
        .ok_or_else(stale)?;
    let current = organization_store::load_connection(connection, workspace)?;
    if current.revision != receipt.result_revision {
        return Err(stale());
    }

    let additions: std::collections::HashSet<_> = record
        .chunks
        .iter()
        .flat_map(|chunk| &chunk.operations)
        .filter_map(|operation| match operation {
            CourseOperation::Organization {
                command: Command::CreateResource { id, .. },
            } => Some(*id),
            _ => None,
        })
        .collect();
    let document_heads: std::collections::BTreeMap<_, _> = receipt
        .documents
        .iter()
        .map(|row| (row.resource_id, row.revision_id))
        .collect();
    for (resource, expected) in &document_heads {
        if !additions.contains(resource)
            || super::delegation::resource_has_external_references(connection, *resource)?
        {
            return Err(stale());
        }
        let actual: Option<String> = connection
            .query_row(
                "SELECT revision_id FROM resource_heads WHERE resource_id=?1",
                [resource.to_string()],
                |row| row.get(0),
            )
            .optional()?;
        if actual.as_deref() != Some(expected.to_string().as_str()) {
            return Err(stale());
        }
    }

    let mut overlay = Organization::from_snapshot(current).map_err(domain_error)?;
    for operation in record
        .chunks
        .iter()
        .rev()
        .flat_map(|chunk| chunk.operations.iter().rev())
    {
        let CourseOperation::Organization { command } = operation else {
            // Every document belongs to an addition with a checked live head.
            // Keep its revisions and provenance when archiving the resource.
            continue;
        };
        let compensation = match command {
            Command::CreateSubject { id, .. } => archive(ArchiveTarget::Subject(*id)),
            Command::CreateGoal { id, .. } => archive(ArchiveTarget::Goal(*id)),
            Command::CreateCollection { id, .. } => archive(ArchiveTarget::Collection(*id)),
            Command::CreatePath { id, .. } => archive(ArchiveTarget::Path(*id)),
            Command::Place { id, .. } => archive(ArchiveTarget::Placement(*id)),
            Command::Relate { id, .. } => Command::RemoveRelation { id: *id },
            Command::CreateResource { id, .. } => {
                let has_unexpected_head: bool = connection.query_row(
                    "SELECT EXISTS(SELECT 1 FROM resource_heads WHERE resource_id=?1)",
                    [id.to_string()],
                    |row| row.get(0),
                )?;
                if super::delegation::resource_has_external_references(connection, *id)?
                    || (has_unexpected_head && !document_heads.contains_key(id))
                {
                    return Err(stale());
                }
                archive(ArchiveTarget::Resource(*id))
            }
            Command::CreateUnit { id, .. } => {
                let used: bool = connection.query_row(
                    "SELECT EXISTS(SELECT 1 FROM lesson_revisions WHERE unit_id=?1
                                   UNION ALL SELECT 1 FROM lesson_drafts WHERE unit_id=?1
                                   UNION ALL SELECT 1 FROM study_sessions WHERE unit_id=?1)",
                    [id.to_string()],
                    |row| row.get(0),
                )?;
                if used {
                    return Err(stale());
                }
                archive(ArchiveTarget::Unit(*id))
            }
            _ => return Err(stale()),
        };
        overlay
            .apply(
                overlay.snapshot().workspace_id,
                overlay.revision(),
                compensation,
            )
            .map_err(domain_error)?;
    }
    Ok(overlay.into())
}

fn archive(target: ArchiveTarget) -> Command {
    Command::Archive {
        target,
        archived: true,
    }
}
