//! Raw borrowed request preflight precedes application DTO decoding and queue admission.
//! Tauri owns its transport buffer; this adapter does not bound that framework allocation.
use crate::workspace::*;
use serde::Deserialize;
use tauri::{State, ipc::Request};
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Empty {
    course_id: uuid::Uuid,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Read {
    course_id: uuid::Uuid,
    unit_id: String,
    revision_id: Option<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Unit {
    course_id: uuid::Uuid,
    unit_id: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Revision {
    course_id: uuid::Uuid,
    unit_id: String,
    revision_id: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Receipt {
    course_id: uuid::Uuid,
    operation_id: OperationId,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Mutation<T> {
    course_id: uuid::Uuid,
    request: T,
}
#[tauri::command]
pub fn lesson_discover(
    manager: State<'_, CourseManager>,
    request: Request<'_>,
) -> Result<Vec<RegistryDescriptor>, AppError> {
    let args: Empty = lesson_admission::decode(request.body())?;
    manager.service(args.course_id)?;
    ApplicationService::lesson_discover(RequestContext::local_ui())
}
#[tauri::command]
pub async fn lesson_read(
    manager: State<'_, CourseManager>,
    request: Request<'_>,
) -> Result<Option<SavedComposition>, AppError> {
    let args: Read = lesson_admission::decode(request.body())?;
    let service = manager.service(args.course_id)?;
    service
        .lesson_read(RequestContext::local_ui(), args.unit_id, args.revision_id)
        .await
}
#[tauri::command]
pub async fn lesson_list_drafts(
    manager: State<'_, CourseManager>,
    request: Request<'_>,
) -> Result<Vec<LessonDraft>, AppError> {
    let args: Unit = lesson_admission::decode(request.body())?;
    let service = manager.service(args.course_id)?;
    service
        .lesson_list_drafts(RequestContext::local_ui(), args.unit_id)
        .await
}
#[tauri::command]
pub async fn lesson_read_resource(
    manager: State<'_, CourseManager>,
    request: Request<'_>,
) -> Result<Option<SavedDocument>, AppError> {
    let args: Revision = lesson_admission::decode(request.body())?;
    let service = manager.service(args.course_id)?;
    service
        .lesson_resource(RequestContext::local_ui(), args.unit_id, args.revision_id)
        .await
}
#[tauri::command]
pub async fn lesson_read_activity(
    manager: State<'_, CourseManager>,
    request: Request<'_>,
) -> Result<Option<ActivityRevision>, AppError> {
    let args: Revision = lesson_admission::decode(request.body())?;
    let service = manager.service(args.course_id)?;
    service
        .lesson_activity(RequestContext::local_ui(), args.unit_id, args.revision_id)
        .await
}
#[tauri::command]
pub async fn lesson_read_source(
    manager: State<'_, CourseManager>,
    request: Request<'_>,
) -> Result<Option<SourceBindingRevision>, AppError> {
    let args: Revision = lesson_admission::decode(request.body())?;
    let service = manager.service(args.course_id)?;
    service
        .lesson_source(RequestContext::local_ui(), args.unit_id, args.revision_id)
        .await
}
#[tauri::command]
pub async fn lesson_get_receipt(
    manager: State<'_, CourseManager>,
    request: Request<'_>,
) -> Result<Option<LessonReceipt>, AppError> {
    let args: Receipt = lesson_admission::decode(request.body())?;
    let service = manager.service(args.course_id)?;
    service
        .lesson_receipt(RequestContext::local_ui(), args.operation_id)
        .await
}
#[tauri::command]
pub async fn lesson_create_draft(
    manager: State<'_, CourseManager>,
    request: Request<'_>,
) -> Result<LessonDraft, AppError> {
    let args: Mutation<CreateLessonDraft> = lesson_admission::decode(request.body())?;
    let service = manager.service(args.course_id)?;
    service
        .lesson_create(RequestContext::local_ui(), args.request)
        .await
}
#[tauri::command]
pub async fn lesson_edit_draft(
    manager: State<'_, CourseManager>,
    request: Request<'_>,
) -> Result<LessonDraft, AppError> {
    let args: Mutation<EditLessonDraft> = lesson_admission::decode(request.body())?;
    let service = manager.service(args.course_id)?;
    service
        .lesson_edit(RequestContext::local_ui(), args.request)
        .await
}
#[tauri::command]
pub async fn lesson_preview_draft(
    manager: State<'_, CourseManager>,
    request: Request<'_>,
) -> Result<LessonPreview, AppError> {
    let args: Mutation<LessonDraftScope> = lesson_admission::decode(request.body())?;
    let service = manager.service(args.course_id)?;
    service
        .lesson_preview(RequestContext::local_ui(), args.request)
        .await
}
#[tauri::command]
pub async fn lesson_apply_draft(
    manager: State<'_, CourseManager>,
    request: Request<'_>,
) -> Result<LessonReceipt, AppError> {
    let args: Mutation<ApplyLessonDraft> = lesson_admission::decode(request.body())?;
    let service = manager.service(args.course_id)?;
    service
        .lesson_apply(RequestContext::local_ui(), args.request)
        .await
}
#[tauri::command]
pub async fn lesson_discard_draft(
    manager: State<'_, CourseManager>,
    request: Request<'_>,
) -> Result<(), AppError> {
    let args: Mutation<DiscardLessonDraft> = lesson_admission::decode(request.body())?;
    let service = manager.service(args.course_id)?;
    service
        .lesson_discard(RequestContext::local_ui(), args.request)
        .await
}

#[tauri::command]
pub async fn lesson_validate_draft(
    manager: State<'_, CourseManager>,
    request: Request<'_>,
) -> Result<LessonValidation, AppError> {
    let args: Mutation<LessonDraftScope> = lesson_admission::decode(request.body())?;
    let service = manager.service(args.course_id)?;
    service
        .lesson_validate(RequestContext::local_ui(), args.request)
        .await
}
