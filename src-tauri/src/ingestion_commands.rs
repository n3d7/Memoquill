//! Course-scoped ingestion commands; the caller supplies no filesystem paths.
use crate::workspace::*;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn ingestion_create(
    manager: State<'_, CourseManager>,
    course_id: Uuid,
    request: CreateIngestionJob,
) -> Result<IngestionJob, AppError> {
    manager
        .service(course_id)?
        .create_ingestion_job(RequestContext::local_ui(), request)
        .await
}
#[tauri::command]
pub async fn ingestion_read(
    manager: State<'_, CourseManager>,
    course_id: Uuid,
    job_id: Uuid,
) -> Result<IngestionJob, AppError> {
    manager
        .service(course_id)?
        .ingestion_job(RequestContext::local_ui(), job_id)
        .await
}
#[tauri::command]
pub async fn ingestion_list(
    manager: State<'_, CourseManager>,
    course_id: Uuid,
    resource_id: crate::organization::ResourceId,
    offset: u64,
) -> Result<Vec<IngestionJobSummary>, AppError> {
    manager
        .service(course_id)?
        .list_ingestion_jobs(RequestContext::local_ui(), resource_id, offset)
        .await
}
#[tauri::command]
pub async fn ingestion_cancel(
    manager: State<'_, CourseManager>,
    course_id: Uuid,
    request: CancelIngestionJob,
) -> Result<IngestionJob, AppError> {
    manager
        .service(course_id)?
        .cancel_ingestion_job(RequestContext::local_ui(), request)
        .await
}
#[tauri::command]
pub async fn ingestion_apply(
    manager: State<'_, CourseManager>,
    course_id: Uuid,
    request: ApplyIngestionJob,
) -> Result<IngestionJob, AppError> {
    manager
        .service(course_id)?
        .apply_ingestion_job(RequestContext::local_ui(), request)
        .await
}

#[tauri::command]
pub async fn ingestion_review(
    manager: State<'_, CourseManager>,
    course_id: Uuid,
    request: ReviewIngestionJob,
) -> Result<IngestionJob, AppError> {
    manager
        .service(course_id)?
        .review_ingestion_job(RequestContext::local_ui(), request)
        .await
}
#[tauri::command]
pub async fn ingestion_reimport_preview(
    manager: State<'_, CourseManager>,
    course_id: Uuid,
    job_id: Uuid,
    previous_job_id: Uuid,
    mapping: IngestionMapping,
) -> Result<ReimportPreview, AppError> {
    manager
        .service(course_id)?
        .preview_ingestion_reimport(RequestContext::local_ui(), job_id, previous_job_id, mapping)
        .await
}
#[tauri::command]
pub async fn ingestion_reimport_apply(
    manager: State<'_, CourseManager>,
    course_id: Uuid,
    request: ApplyReimportJob,
) -> Result<IngestionJob, AppError> {
    manager
        .service(course_id)?
        .apply_ingestion_reimport(RequestContext::local_ui(), request)
        .await
}
#[tauri::command]
pub async fn ingestion_undo_preview(
    manager: State<'_, CourseManager>,
    course_id: Uuid,
    job_id: Uuid,
) -> Result<UndoPreview, AppError> {
    manager
        .service(course_id)?
        .preview_ingestion_undo(RequestContext::local_ui(), job_id)
        .await
}
#[tauri::command]
pub async fn ingestion_undo(
    manager: State<'_, CourseManager>,
    course_id: Uuid,
    request: UndoIngestionJob,
) -> Result<IngestionJob, AppError> {
    manager
        .service(course_id)?
        .undo_ingestion_job(RequestContext::local_ui(), request)
        .await
}
