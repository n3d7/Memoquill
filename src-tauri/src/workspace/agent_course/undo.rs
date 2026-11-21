//! Compensation for isolated, unmodified structural additions.
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
    if current.revision != receipt.result_revision || !receipt.documents.is_empty() {
        return Err(stale());
    }

    let mut overlay = Organization::from_snapshot(current).map_err(domain_error)?;
    for operation in record
        .chunks
        .iter()
        .rev()
        .flat_map(|chunk| chunk.operations.iter().rev())
    {
        let CourseOperation::Organization { command } = operation else {
            return Err(stale());
        };
        let compensation = match command {
            Command::CreateSubject { id, .. } => archive(ArchiveTarget::Subject(*id)),
            Command::CreateGoal { id, .. } => archive(ArchiveTarget::Goal(*id)),
            Command::CreateCollection { id, .. } => archive(ArchiveTarget::Collection(*id)),
            Command::CreatePath { id, .. } => archive(ArchiveTarget::Path(*id)),
            Command::Place { id, .. } => archive(ArchiveTarget::Placement(*id)),
            Command::Relate { id, .. } => Command::RemoveRelation { id: *id },
            Command::CreateResource { id, .. } => {
                let used: bool = connection.query_row(
                    "SELECT EXISTS(SELECT 1 FROM resource_heads WHERE resource_id=?1
                                   UNION ALL SELECT 1 FROM original_sources WHERE resource_id=?1
                                   UNION ALL SELECT 1 FROM lesson_dependencies WHERE kind='resource' AND target_id=?1)",
                    [id.to_string()], |row| row.get(0),
                )?;
                if used {
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
