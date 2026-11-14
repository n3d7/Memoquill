//! Local application services. Native adapters receive no database connection.
mod assessments;
mod assets;
mod content;
mod courses;
mod database;
mod error;
mod faults;
mod lessons;
mod migrations;
mod operations;
mod owner;
mod packages;
mod recovery;
mod review;
pub use assessments::*;
mod search;
mod service;
mod sources;
mod study;
pub use review::*;
mod types;

pub use content::*;
pub use courses::*;
pub use error::{AppError, ErrorCode};
pub use lessons::*;
pub use packages::*;
pub use search::{SearchHit, SearchPage};
pub use service::ApplicationService;
pub use sources::*;
pub use study::{
    AssessmentJudgment, AssessmentOutcome, AssessorIdentity, DeclaredAssistance,
    JudgmentProvenance, JudgmentSource, JudgmentUncertainty, StudyAction, StudyAttempt, StudyDraft,
    StudyHelp, StudyItem, StudyMode, StudyMutation, StudyPhase, StudyResponse, StudySession,
    StudyView,
};
pub use types::*;

#[cfg(test)]
mod tests;

pub(crate) use lessons::admission as lesson_admission;
