mod bridge;
mod broker;
mod wire;

use crate::workspace::{AppError, CourseManager, mcp_reads::ReadPermissions};
pub(crate) use broker::Connections;
use broker::Status;
use tauri::State;
use uuid::Uuid;

/// Entry point for the packaged stdio bridge. It runs before desktop startup.
pub fn run_stdio() -> Result<(), &'static str> {
    bridge::run()
}

#[tauri::command]
pub(crate) fn mcp_status(connections: State<'_, Connections>) -> Result<Status, AppError> {
    connections.status()
}

#[tauri::command]
pub(crate) fn mcp_set_enabled(
    connections: State<'_, Connections>,
    enabled: bool,
) -> Result<Status, AppError> {
    connections.set_enabled(enabled)?;
    connections.status()
}

#[tauri::command]
pub(crate) fn mcp_approve(
    connections: State<'_, Connections>,
    manager: State<'_, CourseManager>,
    challenge: Uuid,
    course_id: Uuid,
    permissions: ReadPermissions,
) -> Result<Status, AppError> {
    connections.approve(&manager, challenge, course_id, permissions)?;
    connections.status()
}

#[tauri::command]
pub(crate) fn mcp_revoke(
    connections: State<'_, Connections>,
    challenge: Uuid,
) -> Result<Status, AppError> {
    connections.revoke(challenge)?;
    connections.status()
}
