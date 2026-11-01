//! Native authoring adapter. Authorization context is constructed here, never read from IPC.
use crate::{
    organization::{ResourceId, Snapshot},
    workspace::{
        Annotation, AppError, ApplicationService, ApplyContent, ReadingPosition, RequestContext,
        SaveAnnotation, SaveDocument, SaveReadingPosition, SavedDocument,
    },
};
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn library(service: State<'_, ApplicationService>) -> Result<Snapshot, AppError> {
    service.library(RequestContext::local_ui()).await
}
#[tauri::command]
pub async fn apply_content(
    service: State<'_, ApplicationService>,
    request: ApplyContent,
) -> Result<Snapshot, AppError> {
    service
        .apply_content(RequestContext::local_ui(), request)
        .await
}
#[tauri::command]
pub async fn save_document(
    service: State<'_, ApplicationService>,
    request: SaveDocument,
) -> Result<SavedDocument, AppError> {
    service
        .save_document(RequestContext::local_ui(), request)
        .await
}
#[tauri::command]
pub async fn read_document(
    service: State<'_, ApplicationService>,
    resource_id: ResourceId,
    revision_id: Option<Uuid>,
) -> Result<Option<SavedDocument>, AppError> {
    service
        .read_document(RequestContext::local_ui(), resource_id, revision_id)
        .await
}
#[tauri::command]
pub async fn save_annotation(
    service: State<'_, ApplicationService>,
    request: SaveAnnotation,
) -> Result<Annotation, AppError> {
    service
        .save_annotation(RequestContext::local_ui(), request)
        .await
}
#[tauri::command]
pub async fn annotations(
    service: State<'_, ApplicationService>,
    resource_id: ResourceId,
) -> Result<Vec<Annotation>, AppError> {
    service
        .annotations(RequestContext::local_ui(), resource_id)
        .await
}
#[tauri::command]
pub async fn save_reading_position(
    service: State<'_, ApplicationService>,
    request: SaveReadingPosition,
) -> Result<ReadingPosition, AppError> {
    service
        .save_reading_position(RequestContext::local_ui(), request)
        .await
}
#[tauri::command]
pub async fn reading_position(
    service: State<'_, ApplicationService>,
    resource_id: ResourceId,
) -> Result<Option<ReadingPosition>, AppError> {
    service
        .reading_position(RequestContext::local_ui(), resource_id)
        .await
}
