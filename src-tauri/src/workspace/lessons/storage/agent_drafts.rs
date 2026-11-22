use super::*;
use crate::workspace::mcp_drafts::{DraftAction, DraftRequest, DraftResult, DraftSummary};

impl Database {
    pub(in crate::workspace) fn mcp_draft(
        &mut self,
        request: DraftRequest,
        sources: bool,
    ) -> Result<DraftResult> {
        let unit_id = request.unit_id.to_string();
        let workspace_id = self.id.clone();
        let actor = request.actor(&workspace_id)?;
        let context = RequestContext::local_ui().restrict_to_active_records();
        let scope = |draft_id: Uuid, revision| LessonDraftScope {
            contract_version: 1,
            workspace_id: workspace_id.clone(),
            unit_id: unit_id.clone(),
            draft_id: draft_id.to_string(),
            expected_draft_revision: revision,
        };
        match request.action {
            DraftAction::Materials { offset } => {
                unit(&self.connection, &unit_id, true)?;
                let mut query = self.connection.prepare(
                    "SELECT r.id,r.title,h.revision_id FROM unit_resources u JOIN resources r ON r.id=u.resource_id
                     LEFT JOIN resource_heads h ON h.resource_id=r.id WHERE u.unit_id=?1 AND r.archived=0
                     ORDER BY u.position LIMIT 25 OFFSET ?2"
                )?;
                let rows = query.query_map(params![unit_id, offset], |row| {
                    Ok(serde_json::json!({
                        "resourceId": row.get::<_, String>(0)?, "title": row.get::<_, String>(1)?,
                        "revisionId": row.get::<_, Option<String>>(2)?,
                    }))
                })?.collect::<rusqlite::Result<Vec<_>>>()?;
                Ok(DraftResult::Materials(material_page(rows, offset)))
            }
            DraftAction::Sources {
                resource_id,
                offset,
            } => {
                if !sources {
                    return Err(external_denied());
                }
                self.authorize_linked_resource(&unit_id, &resource_id.to_string())?;
                let id = crate::organization::ResourceId::try_from(resource_id)
                    .map_err(|_| external_denied())?;
                let originals = self.sources(context, id)?;
                let rows = originals
                    .sources
                    .into_iter()
                    .skip(offset as usize)
                    .take(25)
                    .map(|original| {
                        serde_json::json!({"sourceId": original.id, "filename": original.filename,
                            "attribution": original.attribution, "license": original.license,
                        })
                    })
                    .collect();
                Ok(DraftResult::Materials(material_page(rows, offset)))
            }

            DraftAction::List => {
                unit(&self.connection, &unit_id, true)?;
                let mut query = self.connection.prepare(
                    "SELECT id FROM lesson_drafts WHERE unit_id=?1 AND actor=?2 ORDER BY id LIMIT 21"
                )?;
                let ids = query
                    .query_map(params![unit_id, actor.as_str()], |row| {
                        row.get::<_, String>(0)
                    })?
                    .collect::<rusqlite::Result<Vec<_>>>()?;
                if ids.len() > 20 {
                    return Err(corrupt());
                }
                let summaries = ids
                    .into_iter()
                    .map(|id| {
                        let draft =
                            state_for(&self.connection, &id, &unit_id, actor.as_str())?.draft;
                        Ok(DraftSummary {
                            id,
                            revision: draft.revision,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok(DraftResult::List(summaries))
            }

            DraftAction::Create {
                operation_id,
                expected_head_revision,
            } => {
                let expected_head_revision = expected_head_revision.map(|id| id.to_string());
                if let Some(revision) = &expected_head_revision {
                    let base = self
                        .lesson_read(context, &unit_id, Some(revision))?
                        .ok_or_else(external_denied)?;
                    self.authorize_draft_bindings(&base.composition, &[], sources)?;
                }
                let draft = self.lesson_create_for(
                    context,
                    &CreateLessonDraft {
                        contract_version: 1,
                        workspace_id,
                        unit_id,
                        expected_head_revision,
                        operation_id: actor.operation_id(operation_id)?,
                    },
                    actor.as_str(),
                )?;
                self.authorize_draft_bindings(&draft.composition, &[], sources)?;
                Ok(DraftResult::Draft(draft))
            }
            DraftAction::Read { draft_id } => {
                unit(&self.connection, &unit_id, true)?;
                let draft = state_for(
                    &self.connection,
                    &draft_id.to_string(),
                    &unit_id,
                    actor.as_str(),
                )?;
                self.authorize_draft_bindings(
                    &draft.draft.composition,
                    &draft.dependencies,
                    sources,
                )?;
                Ok(DraftResult::Draft(draft.draft))
            }
            DraftAction::Edit {
                draft_id,
                revision,
                operation_id,
                operations,
            } => {
                let state = state_for(
                    &self.connection,
                    &draft_id.to_string(),
                    &unit_id,
                    actor.as_str(),
                )?;
                self.authorize_draft_bindings(
                    &state.draft.composition,
                    &state.dependencies,
                    sources,
                )?;
                for operation in &operations {
                    match operation {
                        LessonOperation::CreateSourceBindingRevision { source_id, .. }
                            if sources =>
                        {
                            self.authorize_linked_source(&unit_id, source_id)?;
                        }
                        LessonOperation::BindReference { binding }
                            if binding.target_kind == "resource" =>
                        {
                            self.authorize_linked_resource(&unit_id, &binding.target_id)?;
                        }
                        _ if allowed_edit(operation, &state.draft.composition) => {}
                        _ => return Err(external_denied()),
                    }
                }
                self.lesson_edit_for(
                    context,
                    &EditLessonDraft {
                        contract_version: 1,
                        workspace_id,
                        unit_id,
                        draft_id: draft_id.to_string(),
                        expected_draft_revision: revision,
                        operation_id: actor.operation_id(operation_id)?,
                        operations,
                    },
                    actor.as_str(),
                )
                .map(DraftResult::Draft)
            }
            DraftAction::Validate { draft_id, revision } => {
                let state = state_for(
                    &self.connection,
                    &draft_id.to_string(),
                    &unit_id,
                    actor.as_str(),
                )?;
                self.authorize_draft_bindings(
                    &state.draft.composition,
                    &state.dependencies,
                    sources,
                )?;
                self.lesson_validate_for(context, &scope(draft_id, revision), actor.as_str())
                    .map(DraftResult::Validation)
            }
            DraftAction::Preview { draft_id, revision } => {
                let state = state_for(
                    &self.connection,
                    &draft_id.to_string(),
                    &unit_id,
                    actor.as_str(),
                )?;
                self.authorize_draft_bindings(
                    &state.draft.composition,
                    &state.dependencies,
                    sources,
                )?;
                self.lesson_preview_for(context, &scope(draft_id, revision), actor.as_str())
                    .map(DraftResult::Preview)
            }
            DraftAction::Discard {
                draft_id,
                revision,
                operation_id,
            } => {
                self.lesson_discard_for(
                    context,
                    &DiscardLessonDraft {
                        contract_version: 1,
                        workspace_id,
                        unit_id,
                        draft_id: draft_id.to_string(),
                        expected_draft_revision: revision,
                        operation_id: actor.operation_id(operation_id)?,
                    },
                    actor.as_str(),
                )?;
                Ok(DraftResult::Discarded { draft_id })
            }
        }
    }
    fn authorize_linked_resource(&self, unit_id: &str, resource_id: &str) -> Result<()> {
        let allowed: bool = self.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM unit_resources ur JOIN units u ON u.id=ur.unit_id JOIN resources r ON r.id=ur.resource_id
             WHERE ur.unit_id=?1 AND ur.resource_id=?2 AND u.archived=0 AND r.archived=0)",
            params![unit_id, resource_id], |row| row.get(0),
        )?;
        if allowed {
            Ok(())
        } else {
            Err(external_denied())
        }
    }

    fn authorize_linked_source(&self, unit_id: &str, source_id: &str) -> Result<()> {
        let resource: Option<String> = self
            .connection
            .query_row(
                "SELECT resource_id FROM original_sources WHERE id=?1",
                [source_id],
                |row| row.get(0),
            )
            .optional()?;
        self.authorize_linked_resource(unit_id, &resource.ok_or_else(external_denied)?)
    }

    fn authorize_draft_bindings(
        &self,
        composition: &Composition,
        pending: &[Pending],
        sources: bool,
    ) -> Result<()> {
        use crate::{organization::ResourceId, workspace::content::documents};
        for binding in &composition.bindings {
            match binding.target_kind.as_str() {
                "resource" => {
                    let id = Uuid::parse_str(&binding.target_id).map_err(|_| external_denied())?;
                    let id = ResourceId::try_from(id).map_err(|_| external_denied())?;
                    documents::ensure_resource(&self.connection, id, true).map_err(|error| {
                        if error.code == ErrorCode::InvalidRequest {
                            external_denied()
                        } else {
                            error
                        }
                    })?;
                }
                "source" => {
                    if !sources {
                        return Err(external_denied());
                    }
                    if let Some(Pending::Source { snapshot }) =
                        pending.iter().find(|pin| pin.id() == binding.revision_id)
                    {
                        self.authorize_linked_source(&composition.unit_id, &snapshot.source_id)?;
                    } else if self
                        .mcp_active_source(&composition.unit_id, &binding.revision_id)?
                        .is_none()
                    {
                        return Err(external_denied());
                    }
                }
                "activity" => {}
                _ => return Err(external_denied()),
            }
        }
        Ok(())
    }
}

fn material_page(mut items: Vec<Value>, offset: u32) -> Value {
    let next = (items.len() > 24).then(|| offset.saturating_add(24));
    items.truncate(24);
    serde_json::json!({"items": items, "nextOffset": next})
}

fn external_denied() -> AppError {
    AppError::new(ErrorCode::PermissionDenied)
}

fn allowed_edit(operation: &LessonOperation, composition: &Composition) -> bool {
    match operation {
        LessonOperation::InsertSubtree { bindings, .. } => bindings.is_empty(),
        LessonOperation::CreateActivityRevision {
            activity_id,
            expected_revision_id,
            ..
        } => expected_revision_id.as_ref().is_none_or(|revision| {
            composition.bindings.iter().any(|binding| {
                binding.target_kind == "activity"
                    && &binding.target_id == activity_id
                    && &binding.revision_id == revision
            })
        }),
        LessonOperation::InsertNode { .. }
        | LessonOperation::RemoveNode { .. }
        | LessonOperation::MoveNode { .. }
        | LessonOperation::ConfigureNode { .. }
        | LessonOperation::ConfigureInteraction { .. }
        | LessonOperation::SetAppearance { .. } => true,
        _ => false,
    }
}

impl Database {
    pub(in crate::workspace) fn review_mcp_draft(
        &mut self,
        context: RequestContext,
        unit_id: &str,
        review: crate::workspace::mcp_drafts::DraftReview,
    ) -> Result<crate::workspace::mcp_drafts::DraftReviewResult> {
        use crate::workspace::mcp_drafts::{DraftInspection, DraftReview, DraftReviewResult};
        context.authorize()?;
        unit(&self.connection, unit_id, false)?;
        match review {
            DraftReview::List => {
                let mut query = self.connection.prepare(
                    "SELECT id,actor FROM lesson_drafts WHERE unit_id=?1 AND actor!='local-ui' ORDER BY id LIMIT 21"
                )?;
                let rows = query
                    .query_map([unit_id], |row| {
                        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                    })?
                    .collect::<rusqlite::Result<Vec<_>>>()?;
                if rows.len() > 20 {
                    return Err(corrupt());
                }
                let summaries = rows
                    .into_iter()
                    .map(|(id, actor)| {
                        if !valid_draft_actor(&actor) {
                            return Err(corrupt());
                        }
                        let draft = state_for(&self.connection, &id, unit_id, &actor)?.draft;
                        Ok(DraftSummary {
                            id,
                            revision: draft.revision,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok(DraftReviewResult::List(summaries))
            }
            DraftReview::Inspect { draft_id, revision } => {
                let id = draft_id.to_string();
                let actor = external_actor(&self.connection, unit_id, &id)?;
                let state = state_for(&self.connection, &id, unit_id, &actor)?;
                let validation = self.lesson_validate_for(
                    context,
                    &LessonDraftScope {
                        contract_version: 1,
                        workspace_id: self.id.clone(),
                        unit_id: unit_id.to_owned(),
                        draft_id: id,
                        expected_draft_revision: revision,
                    },
                    &actor,
                )?;
                let review_hash = if validation.valid {
                    Some(preview(&state)?.hash)
                } else {
                    None
                };
                let bindings = self.resolve_draft_review(context, &state)?;
                Ok(DraftReviewResult::Inspection(Box::new(DraftInspection {
                    draft: state.draft,
                    validation,
                    bindings,
                    review_hash,
                })))
            }
            DraftReview::Apply {
                draft_id,
                revision,
                expected_head_revision,
                hash,
                operation_id,
            } => {
                let receipt = self.lesson_apply_for(
                    context,
                    &ApplyLessonDraft {
                        contract_version: 1,
                        workspace_id: self.id.clone(),
                        unit_id: unit_id.to_owned(),
                        draft_id: draft_id.to_string(),
                        expected_draft_revision: revision,
                        expected_head_revision: expected_head_revision.map(|id| id.to_string()),
                        preview_hash: hash,
                        operation_id,
                    },
                    true,
                )?;
                Ok(DraftReviewResult::Applied(Box::new(receipt)))
            }
            DraftReview::Reject {
                draft_id,
                revision,
                operation_id,
            } => {
                let draft_id = draft_id.to_string();
                let actor = external_actor(&self.connection, unit_id, &draft_id)?;
                self.lesson_discard_for(
                    context,
                    &DiscardLessonDraft {
                        contract_version: 1,
                        workspace_id: self.id.clone(),
                        unit_id: unit_id.to_owned(),
                        draft_id,
                        expected_draft_revision: revision,
                        operation_id,
                    },
                    &actor,
                )?;
                Ok(DraftReviewResult::Rejected)
            }
        }
    }
    fn resolve_draft_review(
        &self,
        context: RequestContext,
        state: &DraftState,
    ) -> Result<crate::workspace::mcp_drafts::ResolvedDraftBindings> {
        let mut resolved = crate::workspace::mcp_drafts::ResolvedDraftBindings::default();
        let unit = &state.draft.unit_id;
        for binding in &state.draft.composition.bindings {
            let revision = &binding.revision_id;
            match binding.target_kind.as_str() {
                "resource" => {
                    let id = Uuid::parse_str(&binding.target_id).map_err(|_| corrupt())?;
                    let id =
                        crate::organization::ResourceId::try_from(id).map_err(|_| corrupt())?;
                    let pin = Uuid::parse_str(revision).map_err(|_| corrupt())?;
                    if let Some(document) = self.read_document(context, id, Some(pin))? {
                        resolved.resources.insert(revision.clone(), document);
                    }
                }
                "activity" => {
                    let pending = state.dependencies.iter().find_map(|pending| match pending {
                        Pending::Activity {
                            id,
                            target,
                            definition,
                            ..
                        } if id == revision => Some(ActivityRevision {
                            id: id.clone(),
                            activity_id: target.clone(),
                            unit_id: unit.clone(),
                            definition: definition.clone(),
                        }),
                        _ => None,
                    });
                    let value = match pending {
                        Some(value) => Some(value),
                        None => self.lesson_activity(context, unit, revision)?,
                    };
                    if let Some(value) = value {
                        resolved.activities.insert(revision.clone(), value);
                    }
                }
                "source" => {
                    let pending = state.dependencies.iter().find_map(|pending| match pending {
                        Pending::Source { snapshot } if &snapshot.id == revision => {
                            Some(snapshot.clone())
                        }
                        _ => None,
                    });
                    let value = match pending {
                        Some(value) => Some(value),
                        None => self.published_lesson_source(unit, revision)?,
                    };
                    if let Some(source) = value {
                        resolved.sources.insert(revision.clone(), source);
                    }
                }
                _ => return Err(corrupt()),
            }
        }
        if bytes(&resolved)?.len() > MAX_COMPOSITION {
            return Err(AppError::new(ErrorCode::BudgetExceeded));
        }
        Ok(resolved)
    }
}

pub(super) fn external_actor(connection: &Connection, unit: &str, draft: &str) -> Result<String> {
    let actor: Option<String> = connection
        .query_row(
            "SELECT actor FROM lesson_drafts WHERE id=?1 AND unit_id=?2 AND actor!='local-ui'",
            params![draft, unit],
            |row| row.get(0),
        )
        .optional()?;
    actor
        .filter(|actor| valid_draft_actor(actor))
        .ok_or_else(external_denied)
}
