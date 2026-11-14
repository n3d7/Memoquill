//! Course-local review policy and queue transport; caller identity is native-owned.
use crate::workspace::*;
use serde::Deserialize;
use tauri::{State, ipc::Request};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Mutation {
    course_id: uuid::Uuid,
    request: ReviewMutation,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Scope {
    course_id: uuid::Uuid,
}

#[tauri::command]
pub async fn review_apply(
    manager: State<'_, CourseManager>,
    request: Request<'_>,
) -> Result<ReviewPlan, AppError> {
    let args: Mutation = lesson_admission::decode(request.body())?;
    manager
        .service(args.course_id)?
        .review_apply(RequestContext::local_ui(), args.request)
        .await
}
#[tauri::command]
pub async fn review_overview(
    manager: State<'_, CourseManager>,
    request: Request<'_>,
) -> Result<ReviewOverview, AppError> {
    let args: Scope = lesson_admission::decode(request.body())?;
    manager
        .service(args.course_id)?
        .review_overview(RequestContext::local_ui())
        .await
}
