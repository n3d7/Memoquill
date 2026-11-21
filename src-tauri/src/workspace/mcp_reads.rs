//! The external adapter receives this read-only facade, never a course manager.
use super::{
    AppError, ApplicationService, CourseManager, ErrorCode, RequestContext, SourceBindingRevision,
    SourcePreview,
};
use crate::organization::{ResourceId, UnitId};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

const PAGE_SIZE: usize = 24;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ReadPermissions {
    pub content: bool,
    pub progress: bool,
    pub sources: bool,
    pub responses: bool,
    pub originals: bool,
    #[serde(default)]
    pub drafts: bool,
    #[serde(default)]
    pub delegate_additions: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum CourseRead {
    Capabilities,
    Structure {
        kind: StructureKind,
        cursor: Option<Uuid>,
    },
    Search {
        query: String,
        cursor: Option<Uuid>,
    },
    Registry {
        offset: u32,
    },
    Resource {
        id: Uuid,
        revision: Option<Uuid>,
        offset: u64,
    },
    Lesson {
        id: Uuid,
    },
    Progress {
        id: Uuid,
        offset: u64,
        #[serde(rename = "includeResponses")]
        include_responses: bool,
    },
    Source {
        id: Uuid,
        revision: Uuid,
        offset: u32,
    },
    SourceOriginal {
        id: Uuid,
        revision: Uuid,
        offset: u64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum StructureKind {
    Subjects,
    Goals,
    Collections,
    Units,
    Resources,
    Paths,
    Placements,
    Relations,
}

#[derive(Clone)]
pub(crate) struct CourseReads {
    course_id: Uuid,
    service: ApplicationService,
    permissions: ReadPermissions,
    cursors: Arc<Mutex<VecDeque<(Uuid, Cursor)>>>,
    delegation: Option<Arc<super::agent_course::Delegation>>,
}

#[derive(PartialEq, Eq)]
enum Scope {
    Structure(StructureKind),
    Search(String),
}

struct Cursor {
    scope: Scope,
    revision: u64,
    offset: u64,
}

pub(super) struct PublishedLesson {
    pub(super) saved: super::SavedComposition,
    pub(super) source_revisions: Vec<String>,
}

pub(super) enum BoundSource {
    Extracted {
        binding: SourceBindingRevision,
        preview: Box<SourcePreview>,
    },
    Original {
        binding: SourceBindingRevision,
        byte_length: u64,
        bytes: Vec<u8>,
    },
}

type Result<T> = std::result::Result<T, AppError>;

impl CourseReads {
    pub(crate) fn approve(
        manager: &CourseManager,
        selected_course: Uuid,
        permissions: ReadPermissions,
    ) -> Result<Self> {
        if !(permissions.content || permissions.progress)
            || (permissions.sources && !permissions.content)
            || (permissions.originals && !permissions.sources)
            || (permissions.responses && !permissions.progress)
            || (permissions.drafts && !permissions.content)
            || (permissions.delegate_additions && !permissions.drafts)
        {
            return Err(AppError::new(ErrorCode::InvalidRequest));
        }
        // The approval captures a service, so later selection cannot retarget a grant.
        let service = manager.selected_service(selected_course)?;
        Ok(Self {
            course_id: selected_course,
            service,
            permissions,
            cursors: Arc::default(),
            delegation: permissions.delegate_additions.then(|| {
                Arc::new(super::agent_course::Delegation::new(
                    std::time::Duration::from_secs(3600),
                ))
            }),
        })
    }

    pub(crate) async fn draft(&self, request: super::mcp_drafts::DraftRequest) -> Result<Value> {
        if !self.permissions.drafts {
            return Err(denied());
        }
        let result = self
            .service
            .mcp_draft(request, self.permissions.sources)
            .await?;
        serde_json::to_value(result).map_err(|_| AppError::new(ErrorCode::StorageFailure))
    }

    pub(crate) async fn course_draft(
        &self,
        request: super::agent_course::CourseDraftRequest,
    ) -> Result<Value> {
        if !self.permissions.drafts {
            return Err(denied());
        }
        self.service
            .mcp_course_draft(request, self.delegation.clone())
            .await
    }

    pub(crate) fn revoke_delegation(&self) {
        if let Some(lease) = &self.delegation {
            lease.revoke();
        }
    }

    pub(crate) async fn read(&self, request: CourseRead) -> Result<Value> {
        // This facade is constructed by trusted desktop approval and only exposes
        // an exhaustive list of read operations with their own projections.
        let context = RequestContext::local_ui().restrict_to_active_records();
        match request {
            CourseRead::Capabilities => Ok(json!({
                "contractVersion": 1,
                "courseId": self.course_id,
                "permissions": self.permissions,
                "structurePageSize": PAGE_SIZE,
            })),
            CourseRead::Progress {
                id,
                offset,
                include_responses,
            } if self.permissions.progress
                && (!include_responses || self.permissions.responses) =>
            {
                let id = UnitId::try_from(id).map_err(|_| denied())?;
                let progress = self
                    .service
                    .study_progress(context, id.to_string(), offset)
                    .await
                    .map_err(hide_reference)?;
                let responses = include_responses.then(|| {
                    progress
                        .evidence
                        .into_iter()
                        .map(|row| {
                            json!({
                                "attemptId": row.attempt.id,
                                "compositionRevision": row.composition_revision_id,
                                "activityRevision": row.attempt.activity_revision_id,
                                "response": row.attempt.response,
                            })
                        })
                        .collect::<Vec<_>>()
                });
                Ok(json!({
                    "unitId": id,
                    "sessionCount": progress.session_count,
                    "totalAttempts": progress.total_attempts,
                    "coverage": progress.coverage,
                    "responses": responses,
                    "nextOffset": if include_responses { progress.next_offset } else { None },
                }))
            }
            CourseRead::Progress { .. } => Err(denied()),
            CourseRead::Source { .. } if !self.permissions.sources => Err(denied()),
            CourseRead::SourceOriginal { .. } if !self.permissions.originals => Err(denied()),
            _ if !self.permissions.content => Err(denied()),
            CourseRead::Registry { offset } => {
                let descriptors = ApplicationService::lesson_discover(context)?;
                let (items, next) = page(descriptors.into_iter(), offset);
                Ok(json!({"items": items, "nextOffset": next}))
            }
            CourseRead::Structure { kind, cursor } => {
                let scope = Scope::Structure(kind);
                let (offset, revision) = self.position(cursor, &scope)?;
                let offset = u32::try_from(offset).map_err(|_| stale())?;
                let snapshot = self.service.library(context).await?;
                check_revision(revision, snapshot.revision)?;
                let items = structure(&snapshot, kind, offset);
                let cursor = self.continuation(scope, snapshot.revision, items.1.map(u64::from))?;
                Ok(json!({"revision": snapshot.revision, "items": items.0, "nextCursor": cursor}))
            }
            CourseRead::Search { query, cursor } => {
                let scope = Scope::Search(query.clone());
                let (offset, revision) = self.position(cursor, &scope)?;
                let result = self
                    .service
                    .search_resources(context, query, offset)
                    .await?;
                check_revision(revision, result.revision)?;
                let items: Vec<_> = result.items.into_iter().map(|row| {
                    json!({"id": row.resource_id, "title": row.title, "snippet": row.snippet})
                }).collect();
                let cursor = self.continuation(scope, result.revision, result.next_offset)?;
                Ok(json!({"revision": result.revision, "items": items, "nextCursor": cursor}))
            }
            CourseRead::Resource {
                id,
                revision,
                offset,
            } => {
                if offset != 0 && revision.is_none() {
                    return Err(stale());
                }
                let id = ResourceId::try_from(id).map_err(|_| denied())?;
                let value = self
                    .service
                    .read_document(context, id, revision)
                    .await
                    .map_err(hide_reference)?
                    .ok_or_else(denied)?;
                // Normalized text avoids widening access to source locators, attachments,
                // basenames, metadata or raw assets embedded in storage records.
                let text = value.document.text();
                let start = usize::try_from(offset).map_err(|_| invalid())?;
                if !text.is_char_boundary(start) {
                    return Err(invalid());
                }
                let end = text.floor_char_boundary((start + 16 * 1024).min(text.len()));
                Ok(json!({
                    "id": id, "revision": value.id, "text": &text[start..end],
                    "nextOffset": (end < text.len()).then_some(end),
                }))
            }
            CourseRead::Lesson { id } => {
                let id = UnitId::try_from(id).map_err(|_| denied())?;
                let lesson = self
                    .service
                    .mcp_lesson(context, id.to_string(), self.permissions.sources)
                    .await
                    .map_err(hide_reference)?
                    .ok_or_else(denied)?;
                let value = lesson.saved;
                let nodes: Vec<_> = value.composition.nodes.iter().map(|node| {
                    json!({"id": node.node_id, "type": node.type_id, "version": node.type_version})
                }).collect();
                Ok(json!({
                    "unitId": id,
                    "revision": value.id,
                    "compositionContractVersion": value.composition.contract_version,
                    "registryContractVersion": value.registry_contract_version,
                    "nodes": nodes,
                    "sourceRevisions": lesson.source_revisions,
                }))
            }
            CourseRead::Source {
                id,
                revision,
                offset,
            } => {
                let id = UnitId::try_from(id).map_err(|_| denied())?;
                let BoundSource::Extracted { binding, preview } = self
                    .service
                    .mcp_bound_source(context, id.to_string(), revision.to_string(), None)
                    .await
                    .map_err(hide_reference)?
                else {
                    return Err(denied());
                };
                let (records, next, extraction) = match preview.extraction {
                    Some(value) => {
                        let (records, next) = page(value.records.into_iter(), offset);
                        (
                            records,
                            next,
                            Some(json!({"format": value.format, "status": value.status})),
                        )
                    }
                    None => (Vec::new(), None, None),
                };
                Ok(json!({
                    "revision": binding.id,
                    "filename": binding.filename,
                    "attribution": binding.attribution,
                    "license": binding.license,
                    "locator": binding.locator,
                    "declaredUsedRanges": binding.declared_used_ranges,
                    "extractionCoverage": binding.extraction_coverage,
                    "readCoverage": binding.read_coverage,
                    "previewTruncated": preview.truncated,
                    "extraction": extraction,
                    "records": records,
                    "nextOffset": next,
                }))
            }
            CourseRead::SourceOriginal {
                id,
                revision,
                offset,
            } => {
                let id = UnitId::try_from(id).map_err(|_| denied())?;
                let BoundSource::Original {
                    binding,
                    byte_length,
                    bytes,
                } = self
                    .service
                    .mcp_bound_source(context, id.to_string(), revision.to_string(), Some(offset))
                    .await
                    .map_err(hide_reference)?
                else {
                    return Err(denied());
                };
                let next = offset + bytes.len() as u64;
                Ok(json!({
                    "revision": binding.id,
                    "digest": binding.asset_digest,
                    "byteLength": byte_length,
                    "offset": offset,
                    "hex": hex(&bytes),
                    "nextOffset": (next < byte_length).then_some(next),
                }))
            }
        }
    }

    fn position(&self, token: Option<Uuid>, scope: &Scope) -> Result<(u64, Option<u64>)> {
        let Some(token) = token else {
            return Ok((0, None));
        };
        let cursors = self.cursors.lock().map_err(|_| stale())?;
        let (_, cursor) = cursors
            .iter()
            .find(|(id, cursor)| *id == token && &cursor.scope == scope)
            .ok_or_else(stale)?;
        Ok((cursor.offset, Some(cursor.revision)))
    }

    fn continuation(
        &self,
        scope: Scope,
        revision: u64,
        offset: Option<u64>,
    ) -> Result<Option<Uuid>> {
        let Some(offset) = offset else {
            return Ok(None);
        };
        let mut cursors = self.cursors.lock().map_err(|_| stale())?;
        if cursors.len() >= 32 {
            cursors.pop_front();
        }
        let token = Uuid::new_v4();
        cursors.push_back((
            token,
            Cursor {
                scope,
                revision,
                offset,
            },
        ));
        Ok(Some(token))
    }
}

pub(super) fn structure(
    snapshot: &crate::organization::Snapshot,
    kind: StructureKind,
    offset: u32,
) -> (Vec<Value>, Option<u32>) {
    use crate::organization::Target;
    let subject = |id: crate::organization::SubjectId| {
        snapshot.subjects.get(&id).is_some_and(|row| !row.archived)
    };
    let collection = |id: crate::organization::CollectionId| {
        snapshot
            .collections
            .get(&id)
            .is_some_and(|row| !row.archived && subject(row.subject_id))
    };
    let unit =
        |id: crate::organization::UnitId| snapshot.units.get(&id).is_some_and(|row| !row.archived);
    let resource = |id: crate::organization::ResourceId| {
        snapshot.resources.get(&id).is_some_and(|row| !row.archived)
    };
    let goal = |id: crate::organization::GoalId| {
        snapshot
            .goals
            .get(&id)
            .is_some_and(|row| !row.archived && subject(row.subject_id))
    };
    match kind {
        StructureKind::Subjects => page(
            snapshot
                .subjects
                .values()
                .filter(|row| subject(row.id))
                .map(|row| json!({"id": row.id, "title": row.title})),
            offset,
        ),
        StructureKind::Goals => page(
            snapshot
                .goals
                .values()
                .filter(|row| goal(row.id))
                .map(|row| json!(row)),
            offset,
        ),
        StructureKind::Collections => page(
            snapshot
                .collections
                .values()
                .filter(|row| collection(row.id))
                .map(|row| {
                    let parent = row.parent_id.filter(|id| collection(*id));
                    json!({"id": row.id, "title": row.title, "subjectId": row.subject_id,
                    "parentId": parent, "position": row.position})
                }),
            offset,
        ),
        StructureKind::Units => page(
            snapshot
                .units
                .values()
                .filter(|row| unit(row.id))
                .map(|row| {
                    let goals: Vec<_> = row.goal_ids.iter().filter(|&&id| goal(id)).collect();
                    let resources: Vec<_> = row
                        .resource_ids
                        .iter()
                        .filter(|&&id| resource(id))
                        .collect();
                    json!({"id": row.id, "title": row.title, "objective": row.objective,
                    "goalIds": goals, "resourceIds": resources})
                }),
            offset,
        ),
        StructureKind::Resources => page(
            snapshot
                .resources
                .values()
                .filter(|row| resource(row.id))
                .map(|row| json!(row)),
            offset,
        ),
        StructureKind::Paths => page(
            snapshot
                .paths
                .values()
                .filter(|row| !row.archived)
                .map(|row| {
                    let steps: Vec<_> =
                        row.steps.iter().filter(|step| unit(step.unit_id)).collect();
                    json!({"id": row.id, "title": row.title, "steps": steps})
                }),
            offset,
        ),
        StructureKind::Placements => page(
            snapshot
                .placements
                .values()
                .filter(|row| {
                    !row.archived
                        && collection(row.collection_id)
                        && match row.target {
                            Target::Resource(id) => resource(id),
                            Target::Unit(id) => unit(id),
                        }
                })
                .map(|row| json!(row)),
            offset,
        ),
        StructureKind::Relations => page(
            snapshot
                .relations
                .values()
                .filter(|row| unit(row.from) && unit(row.to))
                .map(|row| json!(row)),
            offset,
        ),
    }
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(2 * bytes.len());
    for byte in bytes {
        result.push(char::from(DIGITS[(byte >> 4) as usize]));
        result.push(char::from(DIGITS[(byte & 15) as usize]));
    }
    result
}

fn denied() -> AppError {
    AppError::new(ErrorCode::PermissionDenied)
}

fn hide_reference(error: AppError) -> AppError {
    match error.code {
        ErrorCode::InvalidReference | ErrorCode::InvalidRequest => denied(),
        code => AppError::new(code),
    }
}

fn invalid() -> AppError {
    AppError::new(ErrorCode::InvalidRequest)
}

fn stale() -> AppError {
    AppError::new(ErrorCode::RevisionConflict)
}

fn check_revision(revision: Option<u64>, current: u64) -> Result<()> {
    if revision.is_some_and(|value| value != current) {
        return Err(stale());
    }
    Ok(())
}

fn page<T>(rows: impl Iterator<Item = T>, offset: u32) -> (Vec<T>, Option<u32>) {
    let mut rows = rows.skip(offset as usize);
    let items = rows.by_ref().take(PAGE_SIZE).collect();
    let next = rows.next().and(offset.checked_add(PAGE_SIZE as u32));
    (items, next)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mcp_a_known_archived_resource_is_indistinguishable_from_an_unknown_resource() {
        use crate::{
            document::{Block, BlockContent, Document},
            organization::{ArchiveTarget, Command},
            workspace::{ApplyContent, OperationId, SaveDocument},
        };

        let temp = tempfile::tempdir().unwrap();
        let manager = CourseManager::open(temp.path().to_owned()).unwrap();
        let course = manager.list().unwrap().selected_course_id;
        let service = manager.service(course).unwrap();
        let ui = RequestContext::local_ui();
        let workspace_id = service.status(ui).await.unwrap().preferences.workspace_id;
        let resource = ResourceId::generate();
        service
            .apply_content(
                ui,
                ApplyContent {
                    contract_version: 1,
                    workspace_id: workspace_id.clone(),
                    operation_id: OperationId::generate(),
                    expected_revision: service.library(ui).await.unwrap().revision,
                    command: Command::CreateResource {
                        id: resource,
                        title: "Synthetic document".into(),
                    },
                },
            )
            .await
            .unwrap();
        service
            .save_document(
                ui,
                SaveDocument {
                    contract_version: 1,
                    workspace_id: workspace_id.clone(),
                    operation_id: OperationId::generate(),
                    resource_id: resource,
                    expected_revision: 0,
                    document: Document {
                        schema_version: 1,
                        blocks: vec![Block {
                            id: Uuid::new_v4(),
                            content: BlockContent::Code {
                                text: "Synthetic document value".into(),
                                language: None,
                            },
                        }],
                    },
                },
            )
            .await
            .unwrap();

        let reads = CourseReads::approve(
            &manager,
            course,
            ReadPermissions {
                content: true,
                progress: false,
                responses: false,
                sources: false,
                originals: false,
                drafts: false,
                delegate_additions: false,
            },
        )
        .unwrap();
        let request = |id| CourseRead::Resource {
            id,
            revision: None,
            offset: 0,
        };
        assert_eq!(
            reads.read(request(resource.as_uuid())).await.unwrap()["text"],
            "Synthetic document value"
        );
        service
            .apply_content(
                ui,
                ApplyContent {
                    contract_version: 1,
                    workspace_id,
                    operation_id: OperationId::generate(),
                    expected_revision: service.library(ui).await.unwrap().revision,
                    command: Command::Archive {
                        target: ArchiveTarget::Resource(resource),
                        archived: true,
                    },
                },
            )
            .await
            .unwrap();
        assert!(
            service
                .read_document(ui, resource, None)
                .await
                .unwrap()
                .is_some()
        );
        let archived = reads.read(request(resource.as_uuid())).await.unwrap_err();
        let unknown = reads.read(request(Uuid::new_v4())).await.unwrap_err();
        assert_eq!(archived, unknown);
        assert_eq!(archived.code, ErrorCode::PermissionDenied);
        manager.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn mcp_continuation_is_bound_to_grant_and_query() {
        let temp = tempfile::tempdir().unwrap();
        let manager = CourseManager::open(temp.path().to_owned()).unwrap();
        let course = manager.list().unwrap().selected_course_id;
        let permissions = ReadPermissions {
            content: true,
            progress: false,
            responses: false,
            sources: false,
            originals: false,
            drafts: false,
            delegate_additions: false,
        };
        let owner = CourseReads::approve(&manager, course, permissions).unwrap();
        let other = CourseReads::approve(&manager, course, permissions).unwrap();
        let token = owner
            .continuation(Scope::Search("alpha".into()), 19, Some(24))
            .unwrap();
        assert_eq!(
            owner
                .position(token, &Scope::Search("alpha".into()))
                .unwrap(),
            (24, Some(19))
        );
        assert!(
            owner
                .position(token, &Scope::Search("beta".into()))
                .is_err()
        );
        assert!(
            other
                .position(token, &Scope::Search("alpha".into()))
                .is_err()
        );
        assert!(check_revision(Some(19), 20).is_err());
        manager.shutdown().await.unwrap();
    }
}
