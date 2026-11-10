//! Native authoring adapter. Authorization context is constructed here, never read from IPC.
use crate::{
    organization::{ResourceId, Snapshot},
    workspace::{
        Annotation, AppError, ApplyContent, CourseManager, ReadingPosition, RequestContext,
        SaveAnnotation, SaveDocument, SaveReadingPosition, SavedDocument,
    },
};
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn search_resources(
    manager: State<'_, CourseManager>,
    course_id: uuid::Uuid,
    query: String,
    offset: u64,
) -> Result<crate::workspace::SearchPage, AppError> {
    let service = manager.service(course_id)?;
    service
        .search_resources(RequestContext::local_ui(), query, offset)
        .await
}

#[tauri::command]
pub async fn rebuild_search_index(
    manager: State<'_, CourseManager>,
    course_id: uuid::Uuid,
) -> Result<(), AppError> {
    manager
        .service(course_id)?
        .rebuild_search_index(RequestContext::local_ui())
        .await
}

#[tauri::command]
pub async fn library(
    manager: State<'_, CourseManager>,
    course_id: uuid::Uuid,
) -> Result<Snapshot, AppError> {
    let service = manager.service(course_id)?;
    service.library(RequestContext::local_ui()).await
}
#[tauri::command]
pub async fn apply_content(
    manager: State<'_, CourseManager>,
    course_id: uuid::Uuid,
    request: ApplyContent,
) -> Result<Snapshot, AppError> {
    let service = manager.service(course_id)?;
    service
        .apply_content(RequestContext::local_ui(), request)
        .await
}
#[tauri::command]
pub async fn save_document(
    manager: State<'_, CourseManager>,
    course_id: uuid::Uuid,
    request: SaveDocument,
) -> Result<SavedDocument, AppError> {
    let service = manager.service(course_id)?;
    service
        .save_document(RequestContext::local_ui(), request)
        .await
}
#[tauri::command]
pub async fn read_document(
    manager: State<'_, CourseManager>,
    course_id: uuid::Uuid,
    resource_id: ResourceId,
    revision_id: Option<Uuid>,
) -> Result<Option<SavedDocument>, AppError> {
    let service = manager.service(course_id)?;
    service
        .read_document(RequestContext::local_ui(), resource_id, revision_id)
        .await
}
#[tauri::command]
pub async fn save_annotation(
    manager: State<'_, CourseManager>,
    course_id: uuid::Uuid,
    request: SaveAnnotation,
) -> Result<Annotation, AppError> {
    let service = manager.service(course_id)?;
    service
        .save_annotation(RequestContext::local_ui(), request)
        .await
}
#[tauri::command]
pub async fn annotations(
    manager: State<'_, CourseManager>,
    course_id: uuid::Uuid,
    resource_id: ResourceId,
) -> Result<Vec<Annotation>, AppError> {
    let service = manager.service(course_id)?;
    service
        .annotations(RequestContext::local_ui(), resource_id)
        .await
}
#[tauri::command]
pub async fn save_reading_position(
    manager: State<'_, CourseManager>,
    course_id: uuid::Uuid,
    request: SaveReadingPosition,
) -> Result<ReadingPosition, AppError> {
    let service = manager.service(course_id)?;
    service
        .save_reading_position(RequestContext::local_ui(), request)
        .await
}
#[tauri::command]
pub async fn reading_position(
    manager: State<'_, CourseManager>,
    course_id: uuid::Uuid,
    resource_id: ResourceId,
) -> Result<Option<ReadingPosition>, AppError> {
    let service = manager.service(course_id)?;
    service
        .reading_position(RequestContext::local_ui(), resource_id)
        .await
}
