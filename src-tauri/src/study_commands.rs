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
