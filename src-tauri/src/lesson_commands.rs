//! Native lesson adapter. Trusted context is supplied here, never accepted in IPC payloads.
use crate::workspace::*;
use tauri::State;
#[tauri::command]
pub fn lesson_discover() -> Result<Vec<RegistryDescriptor>, AppError> {
    ApplicationService::lesson_discover(RequestContext::local_ui())
}
#[tauri::command]
pub async fn lesson_read(
    service: State<'_, ApplicationService>,
    unit_id: String,
    revision_id: Option<String>,
) -> Result<Option<SavedComposition>, AppError> {
    service
        .lesson_read(RequestContext::local_ui(), unit_id, revision_id)
        .await
}
#[tauri::command]
pub async fn lesson_list_drafts(
    service: State<'_, ApplicationService>,
    unit_id: String,
) -> Result<Vec<LessonDraft>, AppError> {
    service
        .lesson_list_drafts(RequestContext::local_ui(), unit_id)
        .await
}
#[tauri::command]
pub async fn lesson_read_activity(
    service: State<'_, ApplicationService>,
    unit_id: String,
    revision_id: String,
) -> Result<Option<ActivityRevision>, AppError> {
    service
        .lesson_activity(RequestContext::local_ui(), unit_id, revision_id)
        .await
}
#[tauri::command]
pub async fn lesson_create_draft(
    service: State<'_, ApplicationService>,
    request: CreateLessonDraft,
) -> Result<LessonDraft, AppError> {
    service
        .lesson_create(RequestContext::local_ui(), request)
        .await
}
#[tauri::command]
pub async fn lesson_edit_draft(
    service: State<'_, ApplicationService>,
    request: EditLessonDraft,
) -> Result<LessonDraft, AppError> {
    service
        .lesson_edit(RequestContext::local_ui(), request)
        .await
}
#[tauri::command]
pub async fn lesson_preview_draft(
    service: State<'_, ApplicationService>,
    request: LessonDraftScope,
) -> Result<LessonPreview, AppError> {
    service
        .lesson_preview(RequestContext::local_ui(), request)
        .await
}
#[tauri::command]
pub async fn lesson_apply_draft(
    service: State<'_, ApplicationService>,
    request: ApplyLessonDraft,
) -> Result<LessonReceipt, AppError> {
    service
        .lesson_apply(RequestContext::local_ui(), request)
        .await
}
#[tauri::command]
pub async fn lesson_discard_draft(
    service: State<'_, ApplicationService>,
    request: DiscardLessonDraft,
) -> Result<(), AppError> {
    service
        .lesson_discard(RequestContext::local_ui(), request)
        .await
}
#[tauri::command]
pub async fn lesson_get_receipt(
    service: State<'_, ApplicationService>,
    operation_id: OperationId,
) -> Result<Option<LessonReceipt>, AppError> {
    service
        .lesson_receipt(RequestContext::local_ui(), operation_id)
        .await
}

#[tauri::command]
pub async fn lesson_read_source(
    service: State<'_, ApplicationService>,
    unit_id: String,
    revision_id: String,
) -> Result<Option<SourceBindingRevision>, AppError> {
    service
        .lesson_source(RequestContext::local_ui(), unit_id, revision_id)
        .await
}
