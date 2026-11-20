//! Course-scoped study transport; native code constructs the local caller context.
use crate::workspace::*;
use serde::Deserialize;
use tauri::{State, ipc::Request};
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Mutation {
    course_id: uuid::Uuid,
    request: StudyMutation,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Read {
    course_id: uuid::Uuid,
    session_id: uuid::Uuid,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct List {
    course_id: uuid::Uuid,
    unit_id: String,
    #[serde(default)]
    offset: u64,
}

#[tauri::command]
pub async fn study_apply(
    manager: State<'_, CourseManager>,
    request: Request<'_>,
) -> Result<StudySession, AppError> {
    let args: Mutation = lesson_admission::decode(request.body())?;
    manager
        .service(args.course_id)?
        .study_apply(RequestContext::local_ui(), args.request)
        .await
}
#[tauri::command]
pub async fn study_read(
    manager: State<'_, CourseManager>,
    request: Request<'_>,
) -> Result<StudyView, AppError> {
    let args: Read = lesson_admission::decode(request.body())?;
    manager
        .service(args.course_id)?
        .study_read(RequestContext::local_ui(), args.session_id)
        .await
}
#[tauri::command]
pub async fn study_sessions(
    manager: State<'_, CourseManager>,
    request: Request<'_>,
) -> Result<Vec<StudySession>, AppError> {
    let args: List = lesson_admission::decode(request.body())?;
    manager
        .service(args.course_id)?
        .study_sessions(RequestContext::local_ui(), args.unit_id, args.offset)
        .await
}

#[tauri::command]
pub async fn study_progress(
    manager: State<'_, CourseManager>,
    request: Request<'_>,
) -> Result<LessonProgress, AppError> {
    let args: List = lesson_admission::decode(request.body())?;
    manager
        .service(args.course_id)?
        .study_progress(RequestContext::local_ui(), args.unit_id, args.offset)
        .await
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ResumeList {
    course_id: uuid::Uuid,
    #[serde(default)]
    offset: u64,
}
#[tauri::command]
pub async fn study_resume_sessions(
    manager: State<'_, CourseManager>,
    request: Request<'_>,
) -> Result<ResumePage, AppError> {
    let args: ResumeList = lesson_admission::decode(request.body())?;
    manager
        .service(args.course_id)?
        .study_resume_sessions(RequestContext::local_ui(), args.offset)
        .await
}
