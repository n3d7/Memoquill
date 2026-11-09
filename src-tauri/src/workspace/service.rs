use super::{
    database::{Database, SCHEMA_VERSION},
    error::Result,
    *,
};
use std::{io::Read, path::PathBuf};
use tokio::sync::{mpsc, oneshot};

type Reply<T> = oneshot::Sender<Result<T>>;
const QUEUE_CAPACITY: usize = 64;

enum Request {
    Search(RequestContext, String, u64, Reply<SearchPage>),
    Content(ContentRequest),
    Notes(NotesRequest),
    Sources(SourcesRequest),
    Lessons(Box<LessonsRequest>),
    Status(Reply<WorkspaceStatus>),
    Save(RequestContext, SavePreferences, Reply<PreferencesSnapshot>),
    Receipt(RequestContext, OperationId, Reply<Option<OperationReceipt>>),
    Asset(
        RequestContext,
        RegisterAsset,
        Box<dyn Read + Send>,
        u64,
        Reply<AssetInfo>,
    ),
    Reopen(Reply<WorkspaceStatus>),
    Shutdown(Reply<()>),
}
enum ContentRequest {
    Library(RequestContext, Reply<crate::organization::Snapshot>),
    ApplyContent(
        RequestContext,
        ApplyContent,
        Reply<crate::organization::Snapshot>,
    ),
    SaveDocument(RequestContext, SaveDocument, Reply<SavedDocument>),
    ReadDocument(
        RequestContext,
        crate::organization::ResourceId,
        Option<uuid::Uuid>,
        Reply<Option<SavedDocument>>,
    ),
}

enum NotesRequest {
    SaveAnnotation(RequestContext, SaveAnnotation, Reply<Annotation>),
    Annotations(
        RequestContext,
        crate::organization::ResourceId,
        Reply<Vec<Annotation>>,
    ),
    SavePosition(RequestContext, SaveReadingPosition, Reply<ReadingPosition>),
    ReadPosition(
        RequestContext,
        crate::organization::ResourceId,
        Reply<Option<ReadingPosition>>,
    ),
}
enum SourcesRequest {
    List(
        RequestContext,
        crate::organization::ResourceId,
        Reply<SourceList>,
    ),
    Save(RequestContext, Box<SaveSource>, Reply<OriginalSource>),
    Preview(RequestContext, uuid::Uuid, Reply<SourcePreview>),
    CreateIngestion(RequestContext, Box<CreateIngestionJob>, Reply<IngestionJob>),
    Ingestion(RequestContext, uuid::Uuid, Reply<IngestionJob>),
    IngestionList(
        RequestContext,
        crate::organization::ResourceId,
        u64,
        Reply<Vec<IngestionJobSummary>>,
    ),
    CancelIngestion(RequestContext, Box<CancelIngestionJob>, Reply<IngestionJob>),
    ApplyIngestion(RequestContext, Box<ApplyIngestionJob>, Reply<IngestionJob>),
    ReimportPreview(
        RequestContext,
        uuid::Uuid,
        uuid::Uuid,
        IngestionMapping,
        Reply<ReimportPreview>,
    ),
    Reimport(RequestContext, Box<ApplyReimportJob>, Reply<IngestionJob>),
    UndoPreview(RequestContext, uuid::Uuid, Reply<UndoPreview>),
    UndoIngestion(RequestContext, Box<UndoIngestionJob>, Reply<IngestionJob>),
}

enum LessonsRequest {
    Validate(RequestContext, LessonDraftScope, Reply<LessonValidation>),
    Source(
        RequestContext,
        String,
        String,
        Reply<Option<SourceBindingRevision>>,
    ),
    Read(
        RequestContext,
        String,
        Option<String>,
        Reply<Option<SavedComposition>>,
    ),
    Drafts(RequestContext, String, Reply<Vec<LessonDraft>>),
    Resource(RequestContext, String, String, Reply<Option<SavedDocument>>),
    Activity(
        RequestContext,
        String,
        String,
        Reply<Option<ActivityRevision>>,
    ),
    Create(RequestContext, CreateLessonDraft, Reply<LessonDraft>),
    Edit(RequestContext, EditLessonDraft, Reply<LessonDraft>),
    Preview(RequestContext, LessonDraftScope, Reply<LessonPreview>),
    Apply(RequestContext, ApplyLessonDraft, Reply<LessonReceipt>),
    Discard(RequestContext, DiscardLessonDraft, Reply<()>),
    Receipt(RequestContext, OperationId, Reply<Option<LessonReceipt>>),
}
/// Typed application boundary; no connection, SQL, caller path or principal payload.
#[derive(Clone)]
pub struct ApplicationService {
    sender: mpsc::Sender<Request>,
}
impl ApplicationService {
    pub async fn lesson_source(
        &self,
        context: RequestContext,
        unit_id: String,
        revision_id: String,
    ) -> Result<Option<SourceBindingRevision>> {
        context.authorize()?;
        super::lessons::admission::admit(&(&unit_id, &revision_id))?;
        self.request(
            context,
            |reply| {
                Request::Lessons(Box::new(LessonsRequest::Source(
                    context,
                    unit_id,
                    revision_id,
                    reply,
                )))
            },
            false,
        )
        .await
    }
    pub fn lesson_discover(context: RequestContext) -> Result<Vec<RegistryDescriptor>> {
        context.authorize()?;
        Ok(super::lessons::registry())
    }
    pub async fn lesson_read(
        &self,
        context: RequestContext,
        unit_id: String,
        revision_id: Option<String>,
    ) -> Result<Option<SavedComposition>> {
        context.authorize()?;
        super::lessons::admission::admit(&(&unit_id, &revision_id))?;
        self.request(
            context,
            |reply| {
                Request::Lessons(Box::new(LessonsRequest::Read(
                    context,
                    unit_id,
                    revision_id,
                    reply,
                )))
            },
            false,
        )
        .await
    }
    pub async fn lesson_list_drafts(
        &self,
        context: RequestContext,
        unit_id: String,
    ) -> Result<Vec<LessonDraft>> {
        context.authorize()?;
        super::lessons::admission::admit(&&unit_id)?;
        self.request(
            context,
            |reply| Request::Lessons(Box::new(LessonsRequest::Drafts(context, unit_id, reply))),
            false,
        )
        .await
    }
    pub async fn lesson_resource(
        &self,
        context: RequestContext,
        unit_id: String,
        revision_id: String,
    ) -> Result<Option<SavedDocument>> {
        context.authorize()?;
        super::lessons::admission::admit(&(&unit_id, &revision_id))?;
        self.request(
            context,
            |reply| {
                Request::Lessons(Box::new(LessonsRequest::Resource(
                    context,
                    unit_id,
                    revision_id,
                    reply,
                )))
            },
            false,
        )
        .await
    }
    pub async fn lesson_activity(
        &self,
        context: RequestContext,
        unit_id: String,
        revision_id: String,
    ) -> Result<Option<ActivityRevision>> {
        context.authorize()?;
        super::lessons::admission::admit(&(&unit_id, &revision_id))?;
        self.request(
            context,
            |reply| {
                Request::Lessons(Box::new(LessonsRequest::Activity(
                    context,
                    unit_id,
                    revision_id,
                    reply,
                )))
            },
            false,
        )
        .await
    }
    pub async fn lesson_create(
        &self,
        context: RequestContext,
        request: CreateLessonDraft,
    ) -> Result<LessonDraft> {
        context.authorize()?;
        super::lessons::admission::admit(&request)?;
        self.request(
            context,
            |reply| Request::Lessons(Box::new(LessonsRequest::Create(context, request, reply))),
            true,
        )
        .await
    }
    pub async fn lesson_edit(
        &self,
        context: RequestContext,
        request: EditLessonDraft,
    ) -> Result<LessonDraft> {
        context.authorize()?;
        super::lessons::admission::admit_edit(&request)?;
        self.request(
            context,
            |reply| Request::Lessons(Box::new(LessonsRequest::Edit(context, request, reply))),
            true,
        )
        .await
    }
    pub async fn lesson_validate(
        &self,
        context: RequestContext,
        request: LessonDraftScope,
    ) -> Result<LessonValidation> {
        context.authorize()?;
        super::lessons::admission::admit(&request)?;
        self.request(
            context,
            |reply| Request::Lessons(Box::new(LessonsRequest::Validate(context, request, reply))),
            false,
        )
        .await
    }
    pub async fn lesson_preview(
        &self,
        context: RequestContext,
        request: LessonDraftScope,
    ) -> Result<LessonPreview> {
        context.authorize()?;
        super::lessons::admission::admit(&request)?;
        self.request(
            context,
            |reply| Request::Lessons(Box::new(LessonsRequest::Preview(context, request, reply))),
            false,
        )
        .await
    }
    pub async fn lesson_apply(
        &self,
        context: RequestContext,
        request: ApplyLessonDraft,
    ) -> Result<LessonReceipt> {
        context.authorize()?;
        super::lessons::admission::admit(&request)?;
        self.request(
            context,
            |reply| Request::Lessons(Box::new(LessonsRequest::Apply(context, request, reply))),
            true,
        )
        .await
    }
    pub async fn lesson_discard(
        &self,
        context: RequestContext,
        request: DiscardLessonDraft,
    ) -> Result<()> {
        context.authorize()?;
        super::lessons::admission::admit(&request)?;
        self.request(
            context,
            |reply| Request::Lessons(Box::new(LessonsRequest::Discard(context, request, reply))),
            true,
        )
        .await
    }
    pub async fn lesson_receipt(
        &self,
        context: RequestContext,
        id: OperationId,
    ) -> Result<Option<LessonReceipt>> {
        context.authorize()?;
        super::lessons::admission::admit(&&id)?;
        self.request(
            context,
            |reply| Request::Lessons(Box::new(LessonsRequest::Receipt(context, id, reply))),
            false,
        )
        .await
    }

    pub async fn search_resources(
        &self,
        context: RequestContext,
        query: String,
        offset: u64,
    ) -> Result<SearchPage> {
        self.request(
            context,
            |reply| Request::Search(context, query, offset, reply),
            false,
        )
        .await
    }
    pub async fn sources(
        &self,
        context: RequestContext,
        resource_id: crate::organization::ResourceId,
    ) -> Result<SourceList> {
        self.request(
            context,
            |reply| Request::Sources(SourcesRequest::List(context, resource_id, reply)),
            false,
        )
        .await
    }
    pub async fn save_source(
        &self,
        context: RequestContext,
        request: SaveSource,
    ) -> Result<OriginalSource> {
        self.request(
            context,
            |reply| Request::Sources(SourcesRequest::Save(context, Box::new(request), reply)),
            true,
        )
        .await
    }
    pub async fn source_preview(
        &self,
        context: RequestContext,
        source_id: uuid::Uuid,
    ) -> Result<SourcePreview> {
        self.request(
            context,
            |reply| Request::Sources(SourcesRequest::Preview(context, source_id, reply)),
            false,
        )
        .await
    }

    pub async fn create_ingestion_job(
        &self,
        context: RequestContext,
        request: CreateIngestionJob,
    ) -> Result<IngestionJob> {
        self.request(
            context,
            |reply| {
                Request::Sources(SourcesRequest::CreateIngestion(
                    context,
                    Box::new(request),
                    reply,
                ))
            },
            true,
        )
        .await
    }

    pub async fn ingestion_job(
        &self,
        context: RequestContext,
        id: uuid::Uuid,
    ) -> Result<IngestionJob> {
        self.request(
            context,
            |reply| Request::Sources(SourcesRequest::Ingestion(context, id, reply)),
            false,
        )
        .await
    }

    pub async fn list_ingestion_jobs(
        &self,
        context: RequestContext,
        resource_id: crate::organization::ResourceId,
        offset: u64,
    ) -> Result<Vec<IngestionJobSummary>> {
        self.request(
            context,
            |reply| {
                Request::Sources(SourcesRequest::IngestionList(
                    context,
                    resource_id,
                    offset,
                    reply,
                ))
            },
            false,
        )
        .await
    }

    pub async fn cancel_ingestion_job(
        &self,
        context: RequestContext,
        request: CancelIngestionJob,
    ) -> Result<IngestionJob> {
        self.request(
            context,
            |reply| {
                Request::Sources(SourcesRequest::CancelIngestion(
                    context,
                    Box::new(request),
                    reply,
                ))
            },
            true,
        )
        .await
    }

    pub async fn apply_ingestion_job(
        &self,
        context: RequestContext,
        request: ApplyIngestionJob,
    ) -> Result<IngestionJob> {
        self.request(
            context,
            |reply| {
                Request::Sources(SourcesRequest::ApplyIngestion(
                    context,
                    Box::new(request),
                    reply,
                ))
            },
            true,
        )
        .await
    }

    pub async fn preview_ingestion_reimport(
        &self,
        context: RequestContext,
        job_id: uuid::Uuid,
        previous_job_id: uuid::Uuid,
        mapping: IngestionMapping,
    ) -> Result<ReimportPreview> {
        self.request(
            context,
            |reply| {
                Request::Sources(SourcesRequest::ReimportPreview(
                    context,
                    job_id,
                    previous_job_id,
                    mapping,
                    reply,
                ))
            },
            false,
        )
        .await
    }

    pub async fn apply_ingestion_reimport(
        &self,
        context: RequestContext,
        request: ApplyReimportJob,
    ) -> Result<IngestionJob> {
        self.request(
            context,
            |reply| Request::Sources(SourcesRequest::Reimport(context, Box::new(request), reply)),
            true,
        )
        .await
    }

    pub async fn preview_ingestion_undo(
        &self,
        context: RequestContext,
        id: uuid::Uuid,
    ) -> Result<UndoPreview> {
        self.request(
            context,
            |reply| Request::Sources(SourcesRequest::UndoPreview(context, id, reply)),
            false,
        )
        .await
    }

    pub async fn undo_ingestion_job(
        &self,
        context: RequestContext,
        request: UndoIngestionJob,
    ) -> Result<IngestionJob> {
        self.request(
            context,
            |reply| {
                Request::Sources(SourcesRequest::UndoIngestion(
                    context,
                    Box::new(request),
                    reply,
                ))
            },
            true,
        )
        .await
    }
    pub async fn library(&self, context: RequestContext) -> Result<crate::organization::Snapshot> {
        self.request(
            context,
            |reply| Request::Content(ContentRequest::Library(context, reply)),
            false,
        )
        .await
    }
    pub async fn apply_content(
        &self,
        context: RequestContext,
        request: ApplyContent,
    ) -> Result<crate::organization::Snapshot> {
        self.request(
            context,
            |reply| Request::Content(ContentRequest::ApplyContent(context, request, reply)),
            true,
        )
        .await
    }
    pub async fn save_document(
        &self,
        context: RequestContext,
        request: SaveDocument,
    ) -> Result<SavedDocument> {
        self.request(
            context,
            |reply| Request::Content(ContentRequest::SaveDocument(context, request, reply)),
            true,
        )
        .await
    }
    pub async fn read_document(
        &self,
        context: RequestContext,
        resource_id: crate::organization::ResourceId,
        revision_id: Option<uuid::Uuid>,
    ) -> Result<Option<SavedDocument>> {
        self.request(
            context,
            |reply| {
                Request::Content(ContentRequest::ReadDocument(
                    context,
                    resource_id,
                    revision_id,
                    reply,
                ))
            },
            false,
        )
        .await
    }
    pub async fn save_annotation(
        &self,
        context: RequestContext,
        request: SaveAnnotation,
    ) -> Result<Annotation> {
        self.request(
            context,
            |reply| Request::Notes(NotesRequest::SaveAnnotation(context, request, reply)),
            true,
        )
        .await
    }
    pub async fn annotations(
        &self,
        context: RequestContext,
        resource_id: crate::organization::ResourceId,
    ) -> Result<Vec<Annotation>> {
        self.request(
            context,
            |reply| Request::Notes(NotesRequest::Annotations(context, resource_id, reply)),
            false,
        )
        .await
    }
    pub async fn save_reading_position(
        &self,
        context: RequestContext,
        request: SaveReadingPosition,
    ) -> Result<ReadingPosition> {
        self.request(
            context,
            |reply| Request::Notes(NotesRequest::SavePosition(context, request, reply)),
            true,
        )
        .await
    }
    pub async fn reading_position(
        &self,
        context: RequestContext,
        resource_id: crate::organization::ResourceId,
    ) -> Result<Option<ReadingPosition>> {
        self.request(
            context,
            |reply| Request::Notes(NotesRequest::ReadPosition(context, resource_id, reply)),
            false,
        )
        .await
    }
    /// The native adapter supplies the managed root. Opening/auditing happens off the UI thread.
    pub fn start(root: PathBuf) -> Result<Self> {
        let (sender, receiver) = mpsc::channel(QUEUE_CAPACITY);
        std::thread::Builder::new()
            .name("memoquill-storage".into())
            .spawn(move || worker(root, receiver))?;
        Ok(Self { sender })
    }
    pub async fn status(&self, context: RequestContext) -> Result<WorkspaceStatus> {
        self.request(context, Request::Status, false).await
    }
    pub async fn save_preferences(
        &self,
        context: RequestContext,
        request: SavePreferences,
    ) -> Result<PreferencesSnapshot> {
        self.request(
            context,
            |reply| Request::Save(context, request, reply),
            true,
        )
        .await
    }
    pub async fn operation_receipt(
        &self,
        context: RequestContext,
        id: OperationId,
    ) -> Result<Option<OperationReceipt>> {
        self.request(context, |reply| Request::Receipt(context, id, reply), false)
            .await
    }
    /// Rust-service seam for future ingestion. This is deliberately not a native IPC command.
    pub async fn register_asset(
        &self,
        context: RequestContext,
        request: RegisterAsset,
        reader: Box<dyn Read + Send>,
        limit: u64,
    ) -> Result<AssetInfo> {
        self.request(
            context,
            |reply| Request::Asset(context, request, reader, limit, reply),
            true,
        )
        .await
    }
    pub async fn reopen(&self, context: RequestContext) -> Result<WorkspaceStatus> {
        self.request(context, Request::Reopen, false).await
    }
    pub async fn shutdown(&self) -> Result<()> {
        let (reply, response) = oneshot::channel();
        self.sender
            .send(Request::Shutdown(reply))
            .await
            .map_err(|_| AppError::new(ErrorCode::OwnerUnavailable))?;
        response
            .await
            .map_err(|_| AppError::new(ErrorCode::OwnerUnavailable))?
    }
    async fn request<T>(
        &self,
        context: RequestContext,
        build: impl FnOnce(Reply<T>) -> Request,
        mutation: bool,
    ) -> Result<T> {
        let (reply, response) = oneshot::channel();
        context.authorize()?;
        self.sender.try_send(build(reply)).map_err(queue_error)?;
        response.await.map_err(|_| {
            AppError::new(if mutation {
                ErrorCode::OutcomeUnconfirmed
            } else {
                ErrorCode::OwnerUnavailable
            })
        })?
    }
}

fn queue_error(error: mpsc::error::TrySendError<Request>) -> AppError {
    AppError::new(match error {
        mpsc::error::TrySendError::Full(_) => ErrorCode::Busy,
        mpsc::error::TrySendError::Closed(_) => ErrorCode::OwnerUnavailable,
    })
}
struct Store {
    root: PathBuf,
    database: Result<Database>,
}
impl Store {
    fn access<T>(&mut self, action: impl FnOnce(&mut Database) -> Result<T>) -> Result<T> {
        let degraded = self
            .database
            .as_ref()
            .is_ok_and(|db| db.recovery.blocks_writes());
        let result = match &mut self.database {
            Ok(db) => action(db),
            Err(error) => Err(error.clone()),
        };
        if let Err(error) = &result
            && (error.code == ErrorCode::OutcomeUnconfirmed
                || (error.code == ErrorCode::RecoveryRequired && !degraded))
        {
            self.database = Err(error.clone()); // Close before another operation; caller must explicitly reopen.
        }
        result
    }
    fn handle(&mut self, request: Request) {
        match request {
            Request::Search(context, query, offset, reply) => {
                let _ = reply.send(self.access(|db| db.search_resources(context, &query, offset)));
            }
            Request::Content(request) => self.handle_content(request),
            Request::Notes(request) => self.handle_notes(request),
            Request::Sources(request) => self.handle_sources(request),
            Request::Lessons(request) => self.handle_lessons(*request),
            Request::Status(reply) => {
                let _ = reply.send(self.access(status));
            }
            Request::Save(context, request, reply) => {
                let _ = reply.send(self.access(|db| db.save_preferences(context, &request)));
            }
            Request::Receipt(context, id, reply) => {
                let _ = reply.send(self.access(|db| db.operation_receipt(context, &id)));
            }
            Request::Asset(context, request, mut reader, limit, reply) => {
                let _ = reply.send(
                    self.access(|db| db.register_asset(context, &request, &mut reader, limit)),
                );
            }
            Request::Reopen(reply) => {
                self.database = Err(AppError::new(ErrorCode::OwnerUnavailable));
                self.database = Database::open(&self.root);
                let _ = reply.send(self.access(status));
            }
            Request::Shutdown(_) => unreachable!("shutdown is owned by the worker loop"),
        }
    }
    fn handle_lessons(&mut self, request: LessonsRequest) {
        match request {
            LessonsRequest::Validate(c, r, reply) => {
                let _ = reply.send(self.access(|db| db.lesson_validate(c, &r)));
            }
            LessonsRequest::Source(c, id, revision, reply) => {
                let _ = reply.send(self.access(|db| db.lesson_source(c, &id, &revision)));
            }

            LessonsRequest::Read(c, id, revision, reply) => {
                let _ = reply.send(self.access(|db| db.lesson_read(c, &id, revision.as_deref())));
            }
            LessonsRequest::Drafts(c, id, reply) => {
                let _ = reply.send(self.access(|db| db.lesson_list_drafts(c, &id)));
            }
            LessonsRequest::Resource(c, id, revision, reply) => {
                let _ = reply.send(self.access(|db| db.lesson_resource(c, &id, &revision)));
            }
            LessonsRequest::Activity(c, id, revision, reply) => {
                let _ = reply.send(self.access(|db| db.lesson_activity(c, &id, &revision)));
            }
            LessonsRequest::Create(c, r, reply) => {
                let _ = reply.send(self.access(|db| db.lesson_create(c, &r)));
            }
            LessonsRequest::Edit(c, r, reply) => {
                let _ = reply.send(self.access(|db| db.lesson_edit(c, &r)));
            }
            LessonsRequest::Preview(c, r, reply) => {
                let _ = reply.send(self.access(|db| db.lesson_preview(c, &r)));
            }
            LessonsRequest::Apply(c, r, reply) => {
                let _ = reply.send(self.access(|db| db.lesson_apply(c, &r)));
            }
            LessonsRequest::Discard(c, r, reply) => {
                let _ = reply.send(self.access(|db| db.lesson_discard(c, &r)));
            }
            LessonsRequest::Receipt(c, id, reply) => {
                let _ = reply.send(self.access(|db| db.lesson_receipt(c, &id)));
            }
        }
    }
    fn handle_sources(&mut self, request: SourcesRequest) {
        match request {
            SourcesRequest::List(context, id, reply) => {
                let _ = reply.send(self.access(|db| db.sources(context, id)));
            }
            SourcesRequest::Save(context, request, reply) => {
                let _ = reply.send(self.access(|db| db.save_source(context, &request)));
            }
            SourcesRequest::Preview(context, id, reply) => {
                let _ = reply.send(self.access(|db| db.source_preview(context, id)));
            }
            SourcesRequest::CreateIngestion(context, request, reply) => {
                let _ = reply.send(self.access(|db| db.create_ingestion_job(context, &request)));
            }
            SourcesRequest::Ingestion(context, id, reply) => {
                let _ = reply.send(self.access(|db| db.ingestion_job(context, id)));
            }
            SourcesRequest::IngestionList(context, id, offset, reply) => {
                let _ = reply.send(self.access(|db| db.list_ingestion_jobs(context, id, offset)));
            }
            SourcesRequest::CancelIngestion(context, request, reply) => {
                let _ = reply.send(self.access(|db| db.cancel_ingestion_job(context, &request)));
            }
            SourcesRequest::ApplyIngestion(context, request, reply) => {
                let _ = reply.send(self.access(|db| db.apply_ingestion_job(context, &request)));
            }
            SourcesRequest::ReimportPreview(context, job, previous, mapping, reply) => {
                let _ =
                    reply.send(self.access(|db| {
                        db.preview_ingestion_reimport(context, job, previous, &mapping)
                    }));
            }
            SourcesRequest::Reimport(context, request, reply) => {
                let _ =
                    reply.send(self.access(|db| db.apply_ingestion_reimport(context, &request)));
            }
            SourcesRequest::UndoPreview(context, id, reply) => {
                let _ = reply.send(self.access(|db| db.preview_ingestion_undo(context, id)));
            }
            SourcesRequest::UndoIngestion(context, request, reply) => {
                let _ = reply.send(self.access(|db| db.undo_ingestion_job(context, &request)));
            }
        }
    }
    fn handle_content(&mut self, request: ContentRequest) {
        match request {
            ContentRequest::Library(context, reply) => {
                let _ = reply.send(self.access(|db| db.library(context)));
            }
            ContentRequest::ApplyContent(context, request, reply) => {
                let _ = reply.send(self.access(|db| db.apply_content(context, &request)));
            }
            ContentRequest::SaveDocument(context, request, reply) => {
                let _ = reply.send(self.access(|db| db.save_document(context, &request)));
            }
            ContentRequest::ReadDocument(context, id, revision, reply) => {
                let _ = reply.send(self.access(|db| db.read_document(context, id, revision)));
            }
        }
    }
    fn handle_notes(&mut self, request: NotesRequest) {
        match request {
            NotesRequest::SaveAnnotation(context, request, reply) => {
                let _ = reply.send(self.access(|db| db.save_annotation(context, &request)));
            }
            NotesRequest::Annotations(context, id, reply) => {
                let _ = reply.send(self.access(|db| db.annotations(context, id)));
            }
            NotesRequest::SavePosition(context, request, reply) => {
                let _ = reply.send(self.access(|db| db.save_reading_position(context, &request)));
            }
            NotesRequest::ReadPosition(context, id, reply) => {
                let _ = reply.send(self.access(|db| db.reading_position(context, id)));
            }
        }
    }
}
fn status(db: &mut Database) -> Result<WorkspaceStatus> {
    Ok(WorkspaceStatus {
        contract_version: 1,
        schema_version: SCHEMA_VERSION,
        preferences: db.preferences()?,
        recovery: db.recovery_report(),
        sqlite_version: rusqlite::version().to_owned(),
    })
}
fn worker(root: PathBuf, mut receiver: mpsc::Receiver<Request>) {
    let database = Database::open(&root);
    let mut store = Store { root, database };
    let mut shutdown = Vec::new();
    while let Some(request) = receiver.blocking_recv() {
        match request {
            Request::Shutdown(reply) => {
                receiver.close();
                shutdown.push(reply);
            }
            request => store.handle(request),
        }
    }
    drop(store); // SQLite closes, then the retained file lock drops, before acknowledgement.
    for reply in shutdown {
        let _ = reply.send(Ok(()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::owner::Owner;

    #[test]
    fn service_owns_one_workspace_and_shutdown_releases_its_lock() {
        tauri::async_runtime::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().join("workspace");
            let service = ApplicationService::start(root.clone()).unwrap();
            let status = service.status(RequestContext::local_ui()).await.unwrap();
            assert_eq!(status.preferences.revision, 0);
            assert_eq!(
                Owner::acquire(&root).err().unwrap().code,
                ErrorCode::AlreadyOpen
            );
            service.shutdown().await.unwrap();
            let _owner = Owner::acquire(&root).unwrap();
            assert!(service.status(RequestContext::local_ui()).await.is_err());
        });
    }
    #[test]
    fn unverified_context_cannot_read_workspace_status() {
        tauri::async_runtime::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().join("workspace");
            let service = ApplicationService::start(root).unwrap();
            assert_eq!(
                service
                    .status(RequestContext::unverified_external())
                    .await
                    .unwrap_err()
                    .code,
                ErrorCode::PermissionDenied
            );
            service.shutdown().await.unwrap();
        });
    }

    #[test]
    fn accepted_save_outlives_its_reply_and_shutdown_drains_the_queue() {
        tauri::async_runtime::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().join("workspace");
            let service = ApplicationService::start(root.clone()).unwrap();
            let initial = service.status(RequestContext::local_ui()).await.unwrap();
            let request = SavePreferences {
                contract_version: 1,
                workspace_id: initial.preferences.workspace_id,
                operation_id: OperationId::generate(),
                expected_revision: 0,
                preferences: ReadingPreferences {
                    theme: Theme::Dark,
                    text_scale: TextScale::try_from(125).unwrap(),
                },
            };
            let (reply, response) = oneshot::channel();
            drop(response);
            service
                .sender
                .try_send(Request::Save(
                    RequestContext::local_ui(),
                    request.clone(),
                    reply,
                ))
                .unwrap();
            service.shutdown().await.unwrap();
            let reopened = ApplicationService::start(root).unwrap();
            let status = reopened.status(RequestContext::local_ui()).await.unwrap();
            assert_eq!(status.preferences.preferences, request.preferences);
            assert_eq!(status.preferences.revision, 1);
            assert_eq!(
                reopened
                    .save_preferences(RequestContext::local_ui(), request.clone())
                    .await
                    .unwrap(),
                status.preferences
            );
            assert!(
                reopened
                    .operation_receipt(RequestContext::local_ui(), request.operation_id)
                    .await
                    .unwrap()
                    .is_some()
            );
            assert_eq!(
                reopened
                    .reopen(RequestContext::local_ui())
                    .await
                    .unwrap()
                    .preferences,
                status.preferences
            );
            reopened.shutdown().await.unwrap();
        });
    }

    #[test]
    fn bounded_admission_rejects_full_and_closed_owner_without_accepting_a_request() {
        tauri::async_runtime::block_on(async {
            let (sender, mut receiver) = mpsc::channel(1);
            let service = ApplicationService { sender };
            let (reply, _response) = oneshot::channel();
            service.sender.try_send(Request::Status(reply)).unwrap();
            assert_eq!(
                service
                    .status(RequestContext::local_ui())
                    .await
                    .unwrap_err()
                    .code,
                ErrorCode::Busy
            );
            assert_eq!(receiver.len(), 1);
            receiver.close();
            drop(receiver);
            assert_eq!(
                service
                    .status(RequestContext::local_ui())
                    .await
                    .unwrap_err()
                    .code,
                ErrorCode::OwnerUnavailable
            );
        });
    }

    #[test]
    fn failed_open_can_be_retried_without_replacing_unknown_data() {
        tauri::async_runtime::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().join("workspace");
            drop(Owner::acquire(&root).unwrap());
            let path = root.join("memoquill.sqlite3");
            std::fs::write(&path, b"unknown").unwrap();
            let service = ApplicationService::start(root.clone()).unwrap();
            assert!(service.status(RequestContext::local_ui()).await.is_err());
            assert!(
                service
                    .reopen(RequestContext::unverified_external())
                    .await
                    .is_err()
            );
            assert!(service.reopen(RequestContext::local_ui()).await.is_err());
            assert_eq!(std::fs::read(&path).unwrap(), b"unknown");
            service.shutdown().await.unwrap();
        });
    }

    #[test]
    fn recovery_guard_keeps_preferences_and_receipts_readable_after_a_denied_save() {
        tauri::async_runtime::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().join("workspace");
            let service = ApplicationService::start(root.clone()).unwrap();
            let initial = service.status(RequestContext::local_ui()).await.unwrap();
            let request = RegisterAsset {
                contract_version: 1,
                workspace_id: initial.preferences.workspace_id.clone(),
                operation_id: OperationId::generate(),
            };
            let info = service
                .register_asset(
                    RequestContext::local_ui(),
                    request.clone(),
                    Box::new(std::io::Cursor::new(b"original")),
                    1024,
                )
                .await
                .unwrap();
            service.shutdown().await.unwrap();
            std::fs::remove_file(super::super::assets::asset_path(&root, &info.digest)).unwrap();
            let reopened = ApplicationService::start(root).unwrap();
            let status = reopened.status(RequestContext::local_ui()).await.unwrap();
            assert_eq!(status.recovery.missing_assets, 1);
            let save = SavePreferences {
                contract_version: 1,
                workspace_id: initial.preferences.workspace_id,
                operation_id: OperationId::generate(),
                expected_revision: 0,
                preferences: ReadingPreferences::default(),
            };
            assert_eq!(
                reopened
                    .save_preferences(RequestContext::local_ui(), save)
                    .await
                    .unwrap_err()
                    .code,
                ErrorCode::RecoveryRequired
            );
            assert!(
                reopened
                    .operation_receipt(RequestContext::local_ui(), request.operation_id)
                    .await
                    .unwrap()
                    .is_some()
            );
            assert_eq!(
                reopened.status(RequestContext::local_ui()).await.unwrap(),
                status
            );
            reopened.shutdown().await.unwrap();
        });
    }
    #[test]
    fn accepted_mutation_with_a_disappearing_worker_has_an_unconfirmed_outcome() {
        tauri::async_runtime::block_on(async {
            for kind in ["preferences", "asset"] {
                let (sender, mut receiver) = mpsc::channel(1);
                let service = ApplicationService { sender };
                let accepted = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
                let observed = accepted.clone();
                let worker = std::thread::spawn(move || {
                    let message = receiver.blocking_recv();
                    observed.store(message.is_some(), std::sync::atomic::Ordering::Release);
                    drop(message);
                });
                let workspace_id = WorkspaceId::generate();
                let operation_id = OperationId::generate();
                let error = if kind == "preferences" {
                    service
                        .save_preferences(
                            RequestContext::local_ui(),
                            SavePreferences {
                                contract_version: 1,
                                workspace_id,
                                operation_id,
                                expected_revision: 0,
                                preferences: ReadingPreferences::default(),
                            },
                        )
                        .await
                        .unwrap_err()
                } else {
                    service
                        .register_asset(
                            RequestContext::local_ui(),
                            RegisterAsset {
                                contract_version: 1,
                                workspace_id,
                                operation_id,
                            },
                            Box::new(std::io::Cursor::new(b"source")),
                            1024,
                        )
                        .await
                        .unwrap_err()
                };
                assert_eq!(error.code, ErrorCode::OutcomeUnconfirmed);
                worker.join().unwrap();
                assert!(accepted.load(std::sync::atomic::Ordering::Acquire));
            }
        });
    }
    #[test]
    fn uncertain_commit_closes_the_owner_until_reopen_and_preserves_its_receipt() {
        use super::super::{
            database::Database,
            faults::{self, Phase},
        };
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        let database = Database::open(&root).unwrap();
        let request = RegisterAsset {
            contract_version: 1,
            workspace_id: database.id.clone(),
            operation_id: OperationId::generate(),
        };
        let mut store = Store {
            root: root.clone(),
            database: Ok(database),
        };
        let result = faults::using(
            |point| {
                if point == Phase::AfterCommit {
                    Err(AppError::new(ErrorCode::StorageFailure))
                } else {
                    Ok(())
                }
            },
            || {
                store.access(|db| {
                    db.register_asset(
                        RequestContext::local_ui(),
                        &request,
                        &mut std::io::Cursor::new(b"source"),
                        1024,
                    )
                })
            },
        );
        assert_eq!(result.unwrap_err().code, ErrorCode::OutcomeUnconfirmed);
        drop(Owner::acquire(&root).unwrap());
        assert_eq!(
            store.access(status).unwrap_err().code,
            ErrorCode::OutcomeUnconfirmed
        );
        let (reply, response) = oneshot::channel();
        store.handle(Request::Reopen(reply));
        let reopened = tauri::async_runtime::block_on(response).unwrap().unwrap();
        assert_eq!(reopened.contract_version, 1);
        let receipt = store
            .access(|db| db.operation_receipt(RequestContext::local_ui(), &request.operation_id))
            .unwrap()
            .unwrap();
        assert!(matches!(
            receipt.result,
            ReceiptResult::Applied(OperationOutcome::Asset(_))
        ));
    }
}

#[cfg(test)]
mod lesson_admission_tests {
    use super::*;
    #[test]
    fn oversized_operation_batch_never_enters_the_writer_queue() {
        tauri::async_runtime::block_on(async {
            let (sender, mut receiver) = mpsc::channel(64);
            let service = ApplicationService { sender };
            let request = EditLessonDraft {
                contract_version: 1,
                workspace_id: WorkspaceId::generate(),
                unit_id: uuid::Uuid::new_v4().to_string(),
                draft_id: uuid::Uuid::new_v4().to_string(),
                expected_draft_revision: 1,
                operation_id: OperationId::generate(),
                operations: vec![LessonOperation::RemoveNode { id: "node".into() }; 65],
            };
            use std::future::Future;
            let mut future = Box::pin(service.lesson_edit(RequestContext::local_ui(), request));
            let response = future
                .as_mut()
                .poll(&mut std::task::Context::from_waker(std::task::Waker::noop()));
            assert!(
                matches!(
                    response,
                    std::task::Poll::Ready(Err(AppError {
                        code: ErrorCode::BudgetExceeded,
                        ..
                    }))
                ),
                "invalid payload must fail before waiting for worker"
            );
            assert!(matches!(
                receiver.try_recv(),
                Err(mpsc::error::TryRecvError::Empty)
            ));
        });
    }
}
