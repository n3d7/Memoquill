use super::*;
use crate::workspace::{
    ApplicationService, CourseManager, NativeCapture, PortableCapture, migrations,
};
use std::io::Cursor;
use tempfile::tempdir;

#[tokio::test]
async fn chat_history_is_course_local_and_interrupted_runs_stay_inert() {
    let course_a = tempdir().unwrap();
    let course_b = tempdir().unwrap();
    let a = ApplicationService::start(course_a.path().join("course")).unwrap();
    let b = ApplicationService::start(course_b.path().join("course")).unwrap();
    let context = RequestContext::local_ui();
    let thread_id = Uuid::new_v4();
    let run_id = Uuid::new_v4();
    let title = "Synthetic course chat";
    assert_eq!(
        a.chat_history(
            RequestContext::unverified_external(),
            ChatHistoryRequest::CreateThread {
                id: thread_id,
                title: title.into()
            },
        )
        .await
        .unwrap_err()
        .code,
        ErrorCode::PermissionDenied,
    );
    for _ in 0..2 {
        let result = a
            .chat_history(
                context,
                ChatHistoryRequest::CreateThread {
                    id: thread_id,
                    title: title.into(),
                },
            )
            .await
            .unwrap();
        assert!(matches!(result, ChatHistoryReply::Thread(thread) if thread.sequence == 1));
    }
    assert!(matches!(
        b.chat_history(context, ChatHistoryRequest::ListThreads { before: None }).await.unwrap(),
        ChatHistoryReply::Threads(threads) if threads.is_empty()
    ));
    assert_eq!(
        b.chat_history(
            context,
            ChatHistoryRequest::ListRuns {
                thread_id,
                after: 0
            }
        )
        .await
        .unwrap_err()
        .code,
        ErrorCode::InvalidReference,
    );
    let result = a
        .chat_history(
            context,
            ChatHistoryRequest::StartRun {
                thread_id,
                id: run_id,
                prompt: "Find the example".into(),
            },
        )
        .await
        .unwrap();
    assert!(
        matches!(result, ChatHistoryReply::Start { run, created: true } if run.status == ChatRunStatus::Streaming)
    );
    a.shutdown().await.unwrap();

    let reopened = ApplicationService::start(course_a.path().join("course")).unwrap();
    let result = reopened
        .chat_history(
            context,
            ChatHistoryRequest::ListRuns {
                thread_id,
                after: 0,
            },
        )
        .await
        .unwrap();
    assert!(matches!(result, ChatHistoryReply::Runs(runs)
        if runs.len() == 1 && runs[0].id == run_id
            && runs[0].status == ChatRunStatus::Interrupted && runs[0].prompt == "Find the example"));
    assert_eq!(
        reopened
            .chat_history(
                context,
                ChatHistoryRequest::EndRun {
                    thread_id,
                    id: run_id,
                    finish: ChatRunFinish::Completed("late completion".into()),
                }
            )
            .await
            .unwrap_err()
            .code,
        ErrorCode::IdempotencyConflict,
    );
    let restarted = reopened
        .chat_history(
            context,
            ChatHistoryRequest::StartRun {
                thread_id,
                id: Uuid::new_v4(),
                prompt: "Find the example".into(),
            },
        )
        .await
        .unwrap();
    assert!(
        matches!(restarted, ChatHistoryReply::Start { run, created: true } if run.status == ChatRunStatus::Streaming)
    );
    reopened.shutdown().await.unwrap();
    b.shutdown().await.unwrap();
}

#[tokio::test]
async fn chat_history_rejects_concurrent_stream_and_deduplicates_completion() {
    let root = tempdir().unwrap();
    let service = ApplicationService::start(root.path().join("course")).unwrap();
    let context = RequestContext::local_ui();
    let thread_id = Uuid::new_v4();
    let id = Uuid::new_v4();
    assert_eq!(
        service
            .chat_history(
                context,
                ChatHistoryRequest::CreateThread {
                    id: thread_id,
                    title: "é".repeat(TITLE_BYTES),
                }
            )
            .await
            .unwrap_err()
            .code,
        ErrorCode::InvalidRequest,
    );
    service
        .chat_history(
            context,
            ChatHistoryRequest::CreateThread {
                id: thread_id,
                title: "Synthetic chat".into(),
            },
        )
        .await
        .unwrap();
    for newly_created in [true, false] {
        let result = service
            .chat_history(
                context,
                ChatHistoryRequest::StartRun {
                    thread_id,
                    id,
                    prompt: "Find the lesson".into(),
                },
            )
            .await
            .unwrap();
        assert!(
            matches!(result, ChatHistoryReply::Start { run, created } if run.sequence == 1 && created == newly_created)
        );
    }
    assert_eq!(
        service
            .chat_history(
                context,
                ChatHistoryRequest::StartRun {
                    thread_id,
                    id: Uuid::new_v4(),
                    prompt: "Another request".into(),
                }
            )
            .await
            .unwrap_err()
            .code,
        ErrorCode::Busy,
    );
    for _ in 0..2 {
        let result = service
            .chat_history(
                context,
                ChatHistoryRequest::EndRun {
                    thread_id,
                    id,
                    finish: ChatRunFinish::Completed("Synthetic answer".into()),
                },
            )
            .await
            .unwrap();
        assert!(matches!(result, ChatHistoryReply::Run(run)
            if run.sequence == 1 && run.status == ChatRunStatus::Completed
            && run.response.as_deref() == Some("Synthetic answer")));
    }
    assert_eq!(
        service
            .chat_history(
                context,
                ChatHistoryRequest::EndRun {
                    thread_id,
                    id,
                    finish: ChatRunFinish::Cancelled,
                }
            )
            .await
            .unwrap_err()
            .code,
        ErrorCode::IdempotencyConflict,
    );
    service.shutdown().await.unwrap();
}

#[test]
fn chat_history_migration_preserves_schema_eleven_data() {
    let mut connection = Connection::open_in_memory().unwrap();
    connection
        .pragma_update(None, "foreign_keys", true)
        .unwrap();
    migrations::apply(&mut connection, 0, &migrations::MIGRATIONS[..11]).unwrap();
    connection
        .execute(
            "INSERT INTO agent_course_drafts (id, actor, data) VALUES ('draft', 'agent', '{}')",
            [],
        )
        .unwrap();
    migrations::apply(&mut connection, 11, migrations::MIGRATIONS).unwrap();
    migrations::validate(&connection, 12).unwrap();
    validate_native_data(&connection).unwrap();
    let count: i64 = connection
        .query_row("SELECT count(*) FROM agent_course_drafts", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(count, 1);
    assert!(
        connection
            .pragma_query_value(None, "foreign_keys", |row| row.get::<_, bool>(0))
            .unwrap()
    );
}

#[test]
fn chat_history_is_private_to_native_recovery() {
    let temp = tempdir().unwrap();
    let mut database = Database::open(&temp.path().join("original")).unwrap();
    let context = RequestContext::local_ui();
    let course_id = Uuid::new_v4();
    let thread_id = Uuid::new_v4();
    let first_run = Uuid::new_v4();
    let second_run = Uuid::new_v4();
    database
        .chat_history(
            context,
            ChatHistoryRequest::CreateThread {
                id: thread_id,
                title: "Synthetic chat".into(),
            },
        )
        .unwrap();
    for id in [first_run, second_run] {
        database
            .chat_history(
                context,
                ChatHistoryRequest::StartRun {
                    thread_id,
                    id,
                    prompt: "Synthetic prompt".into(),
                },
            )
            .unwrap();
        if id == first_run {
            database
                .chat_history(
                    context,
                    ChatHistoryRequest::EndRun {
                        thread_id,
                        id,
                        finish: ChatRunFinish::Completed("Synthetic answer".into()),
                    },
                )
                .unwrap();
        }
    }

    let native = database.capture_native(context, course_id).unwrap();
    let mut bytes = Cursor::new(Vec::new());
    native.write_to(&mut bytes).unwrap();
    let package = NativeCapture::read_from(Cursor::new(bytes.into_inner())).unwrap();
    let native_root = temp.path().join("native");
    let manager = CourseManager::open(native_root.clone()).unwrap();
    manager
        .restore_native(course_id, "Native course".into(), &package, false)
        .unwrap();
    let mut restored =
        Database::open(&native_root.join("courses/data").join(course_id.to_string())).unwrap();
    let runs = restored
        .chat_history(
            context,
            ChatHistoryRequest::ListRuns {
                thread_id,
                after: 0,
            },
        )
        .unwrap();
    assert!(matches!(runs, ChatHistoryReply::Runs(runs)
        if runs.len() == 2
            && runs[0].response.as_deref() == Some("Synthetic answer")
            && runs[1].status == ChatRunStatus::Interrupted));

    let portable = database
        .capture_portable_with_history(context, course_id, false, true)
        .unwrap();
    let mut bytes = Cursor::new(Vec::new());
    portable.write_to(&mut bytes).unwrap();
    let package = PortableCapture::read_from(Cursor::new(bytes.into_inner())).unwrap();
    let portable_root = temp.path().join("portable");
    let manager = CourseManager::open(portable_root.clone()).unwrap();
    manager
        .import_portable(course_id, "Shared course".into(), &package)
        .unwrap();
    let mut imported = Database::open(
        &portable_root
            .join("courses/data")
            .join(course_id.to_string()),
    )
    .unwrap();
    let threads = imported
        .chat_history(context, ChatHistoryRequest::ListThreads { before: None })
        .unwrap();
    assert!(matches!(threads, ChatHistoryReply::Threads(threads) if threads.is_empty()));
}
