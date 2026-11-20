//! Local-only catalogue metadata. Data commands resolve an explicit course independently.
use crate::workspace::{AppError, CourseCatalogue, CourseManager};
use serde::Deserialize;
use tauri::State;
use uuid::Uuid;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateCourse {
    operation_id: Uuid,
    title: String,
}
#[tauri::command]
pub fn courses_list(manager: State<'_, CourseManager>) -> Result<CourseCatalogue, AppError> {
    manager.list()
}
#[tauri::command]
pub fn course_create(
    manager: State<'_, CourseManager>,
    request: CreateCourse,
) -> Result<CourseCatalogue, AppError> {
    manager.create(request.operation_id, request.title)
}
#[tauri::command]
pub fn course_select(
    manager: State<'_, CourseManager>,
    course_id: Uuid,
) -> Result<CourseCatalogue, AppError> {
    manager.select(course_id)
}

#[tauri::command]
pub fn workspace_settings(
    manager: State<'_, CourseManager>,
) -> Result<crate::workspace::WorkspaceSettings, AppError> {
    manager.workspace_settings(crate::workspace::RequestContext::local_ui())
}
#[tauri::command]
pub fn save_workspace_settings(
    manager: State<'_, CourseManager>,
    request: crate::workspace::SaveWorkspaceSettings,
) -> Result<crate::workspace::WorkspaceSettings, AppError> {
    manager.save_workspace_settings(crate::workspace::RequestContext::local_ui(), request)
}
