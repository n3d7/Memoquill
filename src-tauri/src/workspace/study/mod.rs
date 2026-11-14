//! Durable, course-local study evidence. Author previews never enter this module.
use super::{AppError, ErrorCode, error::Result};
mod storage;
mod types;
pub(super) use storage::validate_native_data;
pub use types::*;
#[cfg(test)]
mod tests;

fn invalid() -> AppError {
    AppError::new(ErrorCode::InvalidRequest)
}
fn conflict() -> AppError {
    AppError::new(ErrorCode::RevisionConflict)
}
fn corrupt() -> AppError {
    AppError::new(ErrorCode::RecoveryRequired)
}

fn validate_response(item: &StudyItem, response: &StudyResponse) -> Result<()> {
    let definition = &item.activity.definition;
    let family = definition["family"].as_str().ok_or_else(corrupt)?;
    let valid = match (family, response) {
        ("recall" | "constructed", StudyResponse::Text { value }) => {
            value.len()
                <= if family == "recall" && definition["version"] == 2 {
                    1024
                } else {
                    16384
                }
        }
        ("singleChoice", StudyResponse::Choice { option_id }) => definition["options"]
            .as_array()
            .is_some_and(|options| options.iter().any(|o| o["id"].as_str() == Some(option_id))),
        (
            "procedure",
            StudyResponse::Procedure {
                completed_step_ids,
                observations,
            },
        ) => {
            observations.len() <= 16384
                && completed_step_ids.len() <= 64
                && completed_step_ids
                    .iter()
                    .collect::<std::collections::HashSet<_>>()
                    .len()
                    == completed_step_ids.len()
                && definition["steps"].as_array().is_some_and(|steps| {
                    completed_step_ids
                        .iter()
                        .all(|id| steps.iter().any(|step| step["id"].as_str() == Some(id)))
                })
        }
        _ => false,
    };
    if valid { Ok(()) } else { Err(invalid()) }
}

fn evaluate(item: &StudyItem, response: &StudyResponse) -> AssessmentOutcome {
    let d = &item.activity.definition;
    let exact = match response {
        StudyResponse::Choice { option_id } if d["family"] == "singleChoice" => {
            Some(d["correctOptionId"].as_str() == Some(option_id))
        }
        StudyResponse::Text { value } if d["family"] == "recall" && d["evaluation"] == "exact" => {
            Some(
                d["answer"]
                    .as_str()
                    .is_some_and(|answer| answer.trim() == value.trim()),
            )
        }
        _ => None,
    };
    match exact {
        Some(true) => AssessmentOutcome::Correct,
        Some(false) => AssessmentOutcome::Incorrect,
        None => AssessmentOutcome::Unknown,
    }
}
