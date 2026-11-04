//! Raw borrowed request preflight precedes application DTO decoding and queue admission.
//! Tauri owns its transport buffer; this adapter does not bound that framework allocation.
use crate::workspace::*;
use serde::Deserialize;
use tauri::{State, ipc::Request};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Empty {}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Read {
    unit_id: String,
    revision_id: Option<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Unit {
    unit_id: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Revision {
    unit_id: String,
    revision_id: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Receipt {
    operation_id: OperationId,
}
#[tauri::command]
pub fn lesson_discover(request: Request<'_>) -> Result<Vec<RegistryDescriptor>, AppError> {
    let _: Empty = lesson_admission::decode(request.body())?;
    ApplicationService::lesson_discover(RequestContext::local_ui())
}
#[tauri::command]
pub async fn lesson_read(
    service: State<'_, ApplicationService>,
    request: Request<'_>,
) -> Result<Option<SavedComposition>, AppError> {
    let args: Read = lesson_admission::decode(request.body())?;
    service
        .lesson_read(RequestContext::local_ui(), args.unit_id, args.revision_id)
        .await
}
#[tauri::command]
pub async fn lesson_list_drafts(
    service: State<'_, ApplicationService>,
    request: Request<'_>,
) -> Result<Vec<LessonDraft>, AppError> {
    let args: Unit = lesson_admission::decode(request.body())?;
    service
        .lesson_list_drafts(RequestContext::local_ui(), args.unit_id)
        .await
}
#[tauri::command]
pub async fn lesson_read_activity(
    service: State<'_, ApplicationService>,
    request: Request<'_>,
) -> Result<Option<ActivityRevision>, AppError> {
    let args: Revision = lesson_admission::decode(request.body())?;
    service
        .lesson_activity(RequestContext::local_ui(), args.unit_id, args.revision_id)
        .await
}
#[tauri::command]
pub async fn lesson_read_source(
    service: State<'_, ApplicationService>,
    request: Request<'_>,
) -> Result<Option<SourceBindingRevision>, AppError> {
    let args: Revision = lesson_admission::decode(request.body())?;
    service
        .lesson_source(RequestContext::local_ui(), args.unit_id, args.revision_id)
        .await
}
#[tauri::command]
pub async fn lesson_get_receipt(
    service: State<'_, ApplicationService>,
    request: Request<'_>,
) -> Result<Option<LessonReceipt>, AppError> {
    let args: Receipt = lesson_admission::decode(request.body())?;
    service
        .lesson_receipt(RequestContext::local_ui(), args.operation_id)
        .await
}
#[tauri::command]
pub async fn lesson_create_draft(
    service: State<'_, ApplicationService>,
    request: Request<'_>,
) -> Result<LessonDraft, AppError> {
    let args: lesson_admission::Mutation<CreateLessonDraft> =
        lesson_admission::decode(request.body())?;
    service
        .lesson_create(RequestContext::local_ui(), args.request)
        .await
}
#[tauri::command]
pub async fn lesson_edit_draft(
    service: State<'_, ApplicationService>,
    request: Request<'_>,
) -> Result<LessonDraft, AppError> {
    let args: lesson_admission::Mutation<EditLessonDraft> =
        lesson_admission::decode(request.body())?;
    service
        .lesson_edit(RequestContext::local_ui(), args.request)
        .await
}
#[tauri::command]
pub async fn lesson_preview_draft(
    service: State<'_, ApplicationService>,
    request: Request<'_>,
) -> Result<LessonPreview, AppError> {
    let args: lesson_admission::Mutation<LessonDraftScope> =
        lesson_admission::decode(request.body())?;
    service
        .lesson_preview(RequestContext::local_ui(), args.request)
        .await
}
#[tauri::command]
pub async fn lesson_apply_draft(
    service: State<'_, ApplicationService>,
    request: Request<'_>,
) -> Result<LessonReceipt, AppError> {
    let args: lesson_admission::Mutation<ApplyLessonDraft> =
        lesson_admission::decode(request.body())?;
    service
        .lesson_apply(RequestContext::local_ui(), args.request)
        .await
}
#[tauri::command]
pub async fn lesson_discard_draft(
    service: State<'_, ApplicationService>,
    request: Request<'_>,
) -> Result<(), AppError> {
    let args: lesson_admission::Mutation<DiscardLessonDraft> =
        lesson_admission::decode(request.body())?;
    service
        .lesson_discard(RequestContext::local_ui(), args.request)
        .await
}

#[tauri::command]
pub async fn lesson_validate_draft(
    service: State<'_, ApplicationService>,
    request: Request<'_>,
) -> Result<LessonValidation, AppError> {
    let args: lesson_admission::Mutation<LessonDraftScope> =
        lesson_admission::decode(request.body())?;
    service
        .lesson_validate(RequestContext::local_ui(), args.request)
        .await
}
