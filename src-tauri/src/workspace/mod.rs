//! Local application services. Native adapters receive no database connection.
pub(crate) mod agent_course;
mod assessments;
mod assets;
mod content;
mod courses;
mod database;
mod error;
mod faults;
mod lessons;
#[cfg(all(test, target_os = "linux"))]
mod m6_source_fixture;
#[cfg(target_os = "linux")]
pub(crate) mod mcp_drafts;
#[cfg(target_os = "linux")]
pub(crate) mod mcp_reads;
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
mod settings;
mod types;
pub use settings::{SettingsLayer, SettingsOverrides};

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
    JudgmentProvenance, JudgmentSource, JudgmentUncertainty, LessonProgress, ProgressCoverage,
    ProgressEvidence, ProgressJudgmentCount, ResumePage, ResumeSession, StudyAction, StudyAttempt,
    StudyDraft, StudyHelp, StudyItem, StudyMode, StudyMutation, StudyPhase, StudyResponse,
    StudySession, StudyView,
};
pub use types::*;

#[cfg(test)]
mod tests;

pub(crate) use lessons::admission as lesson_admission;

#[cfg(test)]
mod navigation_tests;
