//! Closed native scope for frozen assessments; never accepts a caller context/path.
use crate::workspace::*;
use serde::Deserialize;
use tauri::{State, ipc::Request};
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Create {
    course_id: uuid::Uuid,
    request: CreateAssessment,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Read {
    course_id: uuid::Uuid,
    assessment_id: uuid::Uuid,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct List {
    course_id: uuid::Uuid,
    #[serde(default)]
    offset: u64,
}
#[tauri::command]
pub async fn assessment_create(
    manager: State<'_, CourseManager>,
    request: Request<'_>,
) -> Result<FrozenAssessment, AppError> {
    let args: Create = lesson_admission::decode(request.body())?;
    manager
        .service(args.course_id)?
        .assessment_create(RequestContext::local_ui(), args.request)
        .await
}
#[tauri::command]
pub async fn assessment_read(
    manager: State<'_, CourseManager>,
    request: Request<'_>,
) -> Result<AssessmentView, AppError> {
    let args: Read = lesson_admission::decode(request.body())?;
    manager
        .service(args.course_id)?
        .assessment_read(RequestContext::local_ui(), args.assessment_id)
        .await
}
#[tauri::command]
pub async fn assessment_list(
    manager: State<'_, CourseManager>,
    request: Request<'_>,
) -> Result<Vec<AssessmentSummary>, AppError> {
    let args: List = lesson_admission::decode(request.body())?;
    manager
        .service(args.course_id)?
        .assessment_list(RequestContext::local_ui(), args.offset)
        .await
}
