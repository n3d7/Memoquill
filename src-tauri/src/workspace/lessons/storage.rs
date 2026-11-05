use super::validation::{MAX_COMPOSITION, activity, bytes};
use super::*;
use crate::workspace::{
    OperationId, OperationOutcome, OperationReceipt, ReceiptResult, RequestContext,
    database::Database, operations,
};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use uuid::Uuid;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DraftState {
    draft: LessonDraft,
    dependencies: Vec<Pending>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum Pending {
    Source {
        snapshot: SourceBindingRevision,
    },
    Activity {
        id: String,
        target: String,
        expected: Option<String>,
        definition: Value,
    },
    Resource {
        id: String,
        target: String,
        expected: Option<String>,
        title: String,
        document: crate::document::Document,
    },
}
impl Pending {
    fn id(&self) -> &str {
        match self {
            Self::Activity { id, .. } | Self::Resource { id, .. } => id,
            Self::Source { snapshot } => &snapshot.id,
        }
    }
    fn target(&self) -> &str {
        match self {
            Self::Activity { target, .. } | Self::Resource { target, .. } => target,
            Self::Source { snapshot } => &snapshot.source_id,
        }
    }
    fn kind(&self) -> &str {
        match self {
            Self::Activity { .. } => "activity",
            Self::Source { .. } => "source",
            Self::Resource { .. } => "resource",
        }
    }
}
fn request_budget(request: &impl Serialize) -> Result<()> {
    if bytes(request)?.len() > 2 * MAX_COMPOSITION {
        return Err(super::diagnostics::envelope(
            ErrorCode::BudgetExceeded,
            "request",
        ));
    }
    Ok(())
}
fn contract(v: u32) -> Result<()> {
    if v != 1 {
        return Err(AppError::new(ErrorCode::UnsupportedVersion));
    }
    Ok(())
}
fn encode(v: &impl Serialize) -> Result<String> {
    serde_json::to_string(v).map_err(|_| invalid())
}
fn hash(v: &impl Serialize) -> Result<String> {
    Ok(crate::workspace::migrations::checksum(&bytes(v)?))
}
fn uuid(s: &str) -> Result<()> {
    if !Uuid::parse_str(s).is_ok_and(|id| id.get_version_num() == 4 && id.to_string() == s) {
        return Err(invalid());
    }
    Ok(())
}
fn unit(c: &Connection, id: &str, write: bool) -> Result<()> {
    uuid(id)?;
    let archived: Option<bool> = c
        .query_row("SELECT archived FROM units WHERE id=?1", [id], |r| r.get(0))
        .optional()?;
    if archived.is_none() || write && archived == Some(true) {
        return Err(invalid());
    }
    Ok(())
}
fn head(c: &Connection, id: &str) -> Result<Option<String>> {
    Ok(c.query_row(
        "SELECT revision_id FROM lesson_heads WHERE unit_id=?1",
        [id],
        |r| r.get(0),
    )
    .optional()?)
}
fn valid(c: &Composition, unknown: bool) -> Result<()> {
    super::validation::validate_diagnosed(c, unknown)
}

fn read(c: &Connection, id: &str, revision: Option<&str>) -> Result<Option<SavedComposition>> {
    let reading_head = revision.is_none();
    let revision = match revision {
        Some(r) => Some(r.to_owned()),
        None => head(c, id)?,
    };
    let Some(revision) = revision else {
        return Ok(None);
    };
    uuid(&revision).map_err(|_| corrupt())?;
    let row: Option<(Option<String>, String, String)> = c
        .query_row(
            "SELECT parent_id,data,digest FROM lesson_revisions WHERE id=?1 AND unit_id=?2",
            params![revision, id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    if reading_head && row.is_none() {
        return Err(corrupt());
    }
    row.map(|(parent, data, digest)| {
        if let Some(parent_id) = &parent {
            uuid(parent_id).map_err(|_| corrupt())?;
            let exists: bool = c.query_row(
                "SELECT EXISTS(SELECT 1 FROM lesson_revisions WHERE id=?1 AND unit_id=?2)",
                params![parent_id, id],
                |r| r.get(0),
            )?;
            if !exists || parent_id == &revision {
                return Err(corrupt());
            }
        }
        if data.len() > MAX_COMPOSITION {
            return Err(corrupt());
        }
        let composition: Composition = serde_json::from_str(&data).map_err(|_| corrupt())?;
        valid(&composition, true).map_err(|_| corrupt())?;
        if composition.unit_id != id || hash(&composition)? != digest {
            return Err(corrupt());
        }
        references(c, &composition, &[]).map_err(|_| corrupt())?;
        Ok(SavedComposition {
            id: revision,
            unit_id: id.into(),
            composition,
            parent_revision_id: parent,
            registry_contract_version: 1,
            hash: digest,
        })
    })
    .transpose()
}
fn state(c: &Connection, id: &str, unit_id: &str) -> Result<DraftState> {
    let data: Option<String> = c
        .query_row(
            "SELECT data FROM lesson_drafts WHERE id=?1 AND unit_id=?2 AND actor='local-ui'",
            params![id, unit_id],
            |r| r.get(0),
        )
        .optional()?;
    let data = data.ok_or_else(invalid)?;
    if data.len() > 2 * MAX_COMPOSITION {
        return Err(corrupt());
    }
    let s: DraftState = serde_json::from_str(&data).map_err(|_| corrupt())?;
    if s.draft.id != id
        || s.draft.unit_id != unit_id
        || s.draft.composition.unit_id != unit_id
        || s.draft.revision == 0
        || s.draft.revision > crate::workspace::types::MAX_REVISION as u64
        || s.dependencies.len() > 512
    {
        return Err(corrupt());
    }
    valid(&s.draft.composition, true).map_err(|_| corrupt())?;
    let mut ids = HashSet::new();
    for p in &s.dependencies {
        uuid(p.id()).map_err(|_| corrupt())?;
        uuid(p.target()).map_err(|_| corrupt())?;
        if !ids.insert(p.id()) {
            return Err(corrupt());
        }
        match p {
            Pending::Activity { definition, .. } => activity(definition).map_err(|_| corrupt())?,
            Pending::Source { snapshot } => {
                validate_source(c, snapshot, unit_id).map_err(|_| corrupt())?
            }
            Pending::Resource {
                document, title, ..
            } => {
                document.validate().map_err(|_| corrupt())?;
                if title.is_empty() || title.len() > 512 {
                    return Err(corrupt());
                }
            }
        }
    }
    references(c, &s.draft.composition, &s.dependencies).map_err(|_| corrupt())?;
    Ok(s)
}
fn expected(s: &DraftState, revision: u64) -> Result<()> {
    if s.draft.revision != revision {
        return Err(conflict());
    }
    Ok(())
}
fn save_draft(c: &Connection, s: &DraftState) -> Result<()> {
    let data = encode(s)?;
    if data.len() > 2 * MAX_COMPOSITION {
        return Err(super::diagnostics::envelope(
            ErrorCode::BudgetExceeded,
            "request",
        ));
    }
    let(count,total):(i64,i64)=c.query_row("SELECT count(*),coalesce(sum(length(CAST(data AS BLOB))),0) FROM lesson_drafts WHERE unit_id=?1 AND actor='local-ui' AND id!=?2",params![s.draft.unit_id,s.draft.id],|r|Ok((r.get(0)?,r.get(1)?)))?;
    if count >= 20 || total + data.len() as i64 > 32 * 1024 * 1024 {
        return Err(super::diagnostics::envelope(
            ErrorCode::BudgetExceeded,
            "request",
        ));
    }
    c.execute("INSERT INTO lesson_drafts VALUES(?1,?2,'local-ui',?3) ON CONFLICT(id) DO UPDATE SET data=excluded.data",params![s.draft.id,s.draft.unit_id,data])?;
    Ok(())
}
fn finish(
    tx: rusqlite::Transaction<'_>,
    workspace: &crate::workspace::WorkspaceId,
    id: &OperationId,
    h: &str,
    outcome: OperationOutcome,
) -> Result<()> {
    operations::insert_applied(
        &tx,
        workspace,
        &operations::applied(id.clone(), outcome),
        h,
        "lesson",
    )?;
    crate::workspace::faults::check(crate::workspace::faults::Phase::BeforeCommit)?;
    operations::durable_commit(tx)?;
    crate::workspace::faults::check(crate::workspace::faults::Phase::AfterCommit)
        .map_err(|_| AppError::new(ErrorCode::OutcomeUnconfirmed))
}
fn draft_outcome(r: OperationReceipt) -> Result<LessonDraft> {
    match r.result {
        ReceiptResult::Applied(OperationOutcome::LessonDraft(d)) => {
            valid(&d.composition, true).map_err(|_| corrupt())?;
            Ok(*d)
        }
        ReceiptResult::Interrupted(e) => Err(AppError::new(e)),
        _ => Err(corrupt()),
    }
}
fn receipt_outcome(r: OperationReceipt) -> Result<LessonReceipt> {
    match r.result {
        ReceiptResult::Applied(OperationOutcome::Lesson(r)) => {
            valid(&r.saved.composition, true).map_err(|_| corrupt())?;
            if !r.committed
                || r.new_head != r.saved.id
                || hash(&r.saved.composition)? != r.saved.hash
            {
                return Err(corrupt());
            }
            Ok(*r)
        }
        ReceiptResult::Interrupted(e) => Err(AppError::new(e)),
        _ => Err(corrupt()),
    }
}
fn preview(s: &DraftState) -> Result<LessonPreview> {
    valid(&s.draft.composition, false)?;
    Ok(LessonPreview {
        draft_id: s.draft.id.clone(),
        draft_revision: s.draft.revision,
        composition: s.draft.composition.clone(),
        hash: hash(&(1, &s.draft, &s.dependencies))?,
        warnings: vec![
            "Study actions are preview simulations; no learning evidence is written.".into(),
        ],
    })
}
impl Database {
    pub(in crate::workspace) fn lesson_read(
        &self,
        context: RequestContext,
        id: &str,
        revision: Option<&str>,
    ) -> Result<Option<SavedComposition>> {
        context.authorize()?;
        unit(&self.connection, id, false)?;
        if let Some(revision) = revision {
            uuid(revision)?;
        }
        read(&self.connection, id, revision)
    }
    pub(in crate::workspace) fn lesson_list_drafts(
        &self,
        context: RequestContext,
        id: &str,
    ) -> Result<Vec<LessonDraft>> {
        context.authorize()?;
        unit(&self.connection, id, false)?;
        let mut q=self.connection.prepare("SELECT id FROM lesson_drafts WHERE unit_id=?1 AND actor='local-ui' ORDER BY id LIMIT 21")?;
        let ids = q
            .query_map([id], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        if ids.len() > 20 {
            return Err(corrupt());
        }
        ids.iter()
            .map(|draft| state(&self.connection, draft, id).map(|s| s.draft))
            .collect()
    }
    pub(in crate::workspace) fn lesson_create(
        &mut self,
        context: RequestContext,
        r: &CreateLessonDraft,
    ) -> Result<LessonDraft> {
        self.authorize_workspace(context, &r.workspace_id)?;
        contract(r.contract_version)?;
        request_budget(r)?;
        let h = operations::request_hash("lesson.create", r)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(receipt) = operations::exact_receipt(&tx, &self.id, &r.operation_id, &h)? {
            return draft_outcome(receipt);
        }
        unit(&tx, &r.unit_id, true)?;
        if head(&tx, &r.unit_id)? != r.expected_head_revision {
            return Err(conflict());
        }
        let composition = match read(&tx, &r.unit_id, None)? {
            Some(s) => s.composition,
            None => Composition {
                contract_version: 1,
                unit_id: r.unit_id.clone(),
                root_node_id: Uuid::new_v4().to_string(),
                nodes: vec![],
                bindings: vec![],
                source_binding_refs: vec![],
                appearance: Appearance {
                    spacing: "normal".into(),
                    theme: "system".into(),
                    typography: "default".into(),
                    width: "normal".into(),
                    emphasis: "normal".into(),
                },
                authoring_provenance: Provenance {
                    kind: "manual".into(),
                    note: None,
                },
            },
        };
        let mut s = DraftState {
            draft: LessonDraft {
                id: Uuid::new_v4().to_string(),
                unit_id: r.unit_id.clone(),
                revision: 1,
                composition,
                base_head_revision: r.expected_head_revision.clone(),
            },
            dependencies: vec![],
        };
        if s.draft.composition.nodes.is_empty() {
            s.draft.composition.nodes.push(LessonNode {
                node_id: s.draft.composition.root_node_id.clone(),
                type_id: "stack".into(),
                type_version: 1,
                props: serde_json::json!({"gap":"normal","headingLevel":1}),
                slots: BTreeMap::from([("body".into(), vec![])]),
                on: BTreeMap::new(),
                visible: None,
            });
        }
        save_draft(&tx, &s)?;
        let d = s.draft;
        finish(
            tx,
            &self.id,
            &r.operation_id,
            &h,
            OperationOutcome::LessonDraft(Box::new(d.clone())),
        )?;
        Ok(d)
    }
    pub(in crate::workspace) fn lesson_edit(
        &mut self,
        context: RequestContext,
        r: &EditLessonDraft,
    ) -> Result<LessonDraft> {
        self.authorize_workspace(context, &r.workspace_id)?;
        contract(r.contract_version)?;
        request_budget(r)?;
        if r.operations.is_empty() {
            return Err(invalid());
        }
        if r.operations.len() > 64 || bytes(r)?.len() > 2 * MAX_COMPOSITION {
            return Err(super::diagnostics::envelope(
                ErrorCode::BudgetExceeded,
                "request",
            ));
        }
        let h = operations::request_hash("lesson.edit", r)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(receipt) = operations::exact_receipt(&tx, &self.id, &r.operation_id, &h)? {
            return draft_outcome(receipt);
        }
        unit(&tx, &r.unit_id, true)?;
        let mut s = state(&tx, &r.draft_id, &r.unit_id)?;
        expected(&s, r.expected_draft_revision)?;
        for (index, op) in r.operations.iter().enumerate() {
            edit(&tx, &mut s, op)
                .map_err(|error| super::diagnostics::for_operation(error, index, op))?;
        }
        valid(&s.draft.composition, true)
            .map_err(|error| super::diagnostics::attach_operations(error, &r.operations))?;
        references(&tx, &s.draft.composition, &s.dependencies)
            .map_err(|error| super::diagnostics::attach_operations(error, &r.operations))?;
        s.draft.revision = s
            .draft
            .revision
            .checked_add(1)
            .filter(|n| *n <= crate::workspace::types::MAX_REVISION as u64)
            .ok_or_else(invalid)?;
        save_draft(&tx, &s)?;
        let d = s.draft;
        finish(
            tx,
            &self.id,
            &r.operation_id,
            &h,
            OperationOutcome::LessonDraft(Box::new(d.clone())),
        )?;
        Ok(d)
    }
    pub(in crate::workspace) fn lesson_preview(
        &self,
        context: RequestContext,
        r: &LessonDraftScope,
    ) -> Result<LessonPreview> {
        context.authorize()?;
        contract(r.contract_version)?;
        request_budget(r)?;
        if self.id != r.workspace_id {
            return Err(AppError::new(ErrorCode::PermissionDenied));
        }
        unit(&self.connection, &r.unit_id, false)?;
        let s = state(&self.connection, &r.draft_id, &r.unit_id)?;
        expected(&s, r.expected_draft_revision)?;
        references(&self.connection, &s.draft.composition, &s.dependencies)?;
        check_bases(&self.connection, &s)?;
        preview(&s)
    }
    pub(in crate::workspace) fn lesson_apply(
        &mut self,
        context: RequestContext,
        r: &ApplyLessonDraft,
    ) -> Result<LessonReceipt> {
        self.authorize_workspace(context, &r.workspace_id)?;
        contract(r.contract_version)?;
        request_budget(r)?;
        let h = operations::request_hash("lesson.apply", r)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(receipt) = operations::exact_receipt(&tx, &self.id, &r.operation_id, &h)? {
            return receipt_outcome(receipt);
        }
        unit(&tx, &r.unit_id, true)?;
        let s = state(&tx, &r.draft_id, &r.unit_id)?;
        expected(&s, r.expected_draft_revision)?;
        check_bases(&tx, &s)?;
        let p = preview(&s)?;
        if p.hash != r.preview_hash
            || s.draft.base_head_revision != r.expected_head_revision
            || head(&tx, &r.unit_id)? != r.expected_head_revision
        {
            return Err(conflict());
        }
        references(&tx, &s.draft.composition, &s.dependencies)?;
        let closure = closure(&s)?;
        let mut created = vec![];
        for dependency in &s.dependencies {
            if closure.contains(dependency.id()) {
                publish(&tx, &r.unit_id, dependency)?;
                created.push(dependency.id().into());
            }
        }
        references(&tx, &s.draft.composition, &[])?;
        let saved = SavedComposition {
            id: Uuid::new_v4().to_string(),
            unit_id: r.unit_id.clone(),
            composition: s.draft.composition,
            parent_revision_id: r.expected_head_revision.clone(),
            registry_contract_version: 1,
            hash: hash(&p.composition)?,
        };
        tx.execute(
            "INSERT INTO lesson_revisions VALUES(?1,?2,?3,?4,?5)",
            params![
                saved.id,
                saved.unit_id,
                saved.parent_revision_id,
                encode(&saved.composition)?,
                saved.hash
            ],
        )?;
        for b in &saved.composition.bindings {
            tx.execute(
                "INSERT OR IGNORE INTO lesson_dependencies VALUES(?1,?2,?3,?4)",
                params![saved.id, b.target_kind, b.target_id, b.revision_id],
            )?;
        }
        tx.execute("INSERT INTO lesson_heads VALUES(?1,?2) ON CONFLICT(unit_id) DO UPDATE SET revision_id=excluded.revision_id",params![r.unit_id,saved.id])?;
        tx.execute("DELETE FROM lesson_drafts WHERE id=?1", [&r.draft_id])?;
        created.push(saved.id.clone());
        let receipt = LessonReceipt {
            operation_id: r.operation_id.clone(),
            old_head: r.expected_head_revision.clone(),
            new_head: saved.id.clone(),
            saved,
            actor: "local-ui".into(),
            proposal_hash: p.hash,
            created_revision_ids: created,
            id_map: BTreeMap::new(),
            warnings: p.warnings,
            committed: true,
        };
        finish(
            tx,
            &self.id,
            &r.operation_id,
            &h,
            OperationOutcome::Lesson(Box::new(receipt.clone())),
        )?;
        Ok(receipt)
    }
    pub(in crate::workspace) fn lesson_discard(
        &mut self,
        context: RequestContext,
        r: &DiscardLessonDraft,
    ) -> Result<()> {
        self.authorize_workspace(context, &r.workspace_id)?;
        contract(r.contract_version)?;
        request_budget(r)?;
        let h = operations::request_hash("lesson.discard", r)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(receipt) = operations::exact_receipt(&tx, &self.id, &r.operation_id, &h)? {
            return match receipt.result {
                ReceiptResult::Applied(OperationOutcome::LessonDiscarded(id))
                    if id == r.draft_id =>
                {
                    Ok(())
                }
                _ => Err(corrupt()),
            };
        }
        let s = state(&tx, &r.draft_id, &r.unit_id)?;
        expected(&s, r.expected_draft_revision)?;
        tx.execute("DELETE FROM lesson_drafts WHERE id=?1", [&r.draft_id])?;
        finish(
            tx,
            &self.id,
            &r.operation_id,
            &h,
            OperationOutcome::LessonDiscarded(r.draft_id.clone()),
        )
    }
    pub(in crate::workspace) fn lesson_receipt(
        &self,
        context: RequestContext,
        id: &OperationId,
    ) -> Result<Option<LessonReceipt>> {
        context.authorize()?;
        self.operation_receipt(context, id)?
            .map(receipt_outcome)
            .transpose()
    }
    /// Resolve only this Unit's validated durable draft pins or saved dependency pins.
    pub(in crate::workspace) fn lesson_resource(
        &self,
        context: RequestContext,
        unit_id: &str,
        revision: &str,
    ) -> Result<Option<crate::workspace::SavedDocument>> {
        context.authorize()?;
        unit(&self.connection, unit_id, false)?;
        uuid(revision)?;
        for draft in self.lesson_list_drafts(context, unit_id)? {
            let s = state(&self.connection, &draft.id, unit_id)?;
            // state() validates the owned composition and every reference's kind,
            // target, revision, digest and locator before this pin grants a read.
            let published_target = s
                .draft
                .composition
                .bindings
                .iter()
                .find(|binding| {
                    binding.target_kind == "resource" && binding.revision_id == revision
                })
                .map(|binding| binding.target_id.clone());
            let mut staged_counts = BTreeMap::<String, i64>::new();
            for pending in s.dependencies {
                if let Pending::Resource {
                    id,
                    target,
                    document,
                    ..
                } = pending
                {
                    let count = staged_counts.entry(target.clone()).or_default();
                    *count += 1;
                    if id == revision {
                        let published: i64 = self.connection.query_row(
                            "SELECT coalesce(max(revision),0) FROM resource_revisions WHERE resource_id=?1",
                            [&target], |r| r.get(0),
                        )?;
                        let number = published
                            .checked_add(*count)
                            .filter(|n| *n > 0 && *n <= crate::workspace::types::MAX_REVISION)
                            .ok_or_else(corrupt)?;
                        return Ok(Some(crate::workspace::SavedDocument {
                            id: Uuid::parse_str(&id).map_err(|_| corrupt())?,
                            resource_id: crate::organization::ResourceId::try_from(
                                Uuid::parse_str(&target).map_err(|_| corrupt())?,
                            )
                            .map_err(|_| corrupt())?,
                            revision: number as u64,
                            document,
                        }));
                    }
                }
            }
            if let Some(target) = published_target {
                let resource = crate::organization::ResourceId::try_from(
                    Uuid::parse_str(&target).map_err(|_| corrupt())?,
                )
                .map_err(|_| corrupt())?;
                return self.read_document(
                    context,
                    resource,
                    Some(Uuid::parse_str(revision).map_err(|_| corrupt())?),
                );
            }
        }
        let target: Option<String> = self.connection.query_row(
            "SELECT d.target_id FROM lesson_dependencies d JOIN lesson_revisions r ON r.id=d.composition_id WHERE r.unit_id=?1 AND d.kind='resource' AND d.revision_id=?2 LIMIT 1",
            params![unit_id, revision], |r| r.get(0),
        ).optional()?;
        let Some(target) = target else {
            return Ok(None);
        };
        let resource = crate::organization::ResourceId::try_from(
            Uuid::parse_str(&target).map_err(|_| corrupt())?,
        )
        .map_err(|_| corrupt())?;
        self.read_document(
            context,
            resource,
            Some(Uuid::parse_str(revision).map_err(|_| corrupt())?),
        )
    }
    pub(in crate::workspace) fn lesson_activity(
        &self,
        context: RequestContext,
        unit_id: &str,
        revision: &str,
    ) -> Result<Option<ActivityRevision>> {
        context.authorize()?;
        unit(&self.connection, unit_id, false)?;
        uuid(revision)?;
        let drafts = self.lesson_list_drafts(context, unit_id)?;
        for draft in drafts {
            let s = state(&self.connection, &draft.id, unit_id)?;
            for d in s.dependencies {
                if let Pending::Activity {
                    id,
                    target,
                    definition,
                    ..
                } = d
                    && id == revision
                {
                    return Ok(Some(ActivityRevision {
                        id,
                        activity_id: target,
                        unit_id: unit_id.into(),
                        definition,
                    }));
                }
            }
        }
        let allowed:bool=self.connection.query_row("SELECT EXISTS(SELECT 1 FROM lesson_dependencies d JOIN lesson_revisions r ON r.id=d.composition_id WHERE r.unit_id=?1 AND d.kind='activity' AND d.revision_id=?2)",params![unit_id,revision],|r|r.get(0))?;
        if !allowed {
            return Ok(None);
        }
        let row: Option<(String, String, String)> = self
            .connection
            .query_row(
                "SELECT activity_id,definition,digest FROM lesson_activity_revisions WHERE id=?1",
                [revision],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?;
        row.map(|(activity_id, data, digest)| {
            if data.len() > 65536 {
                return Err(corrupt());
            }
            let definition: Value = serde_json::from_str(&data).map_err(|_| corrupt())?;
            activity(&definition).map_err(|_| corrupt())?;
            if hash(&definition)? != digest {
                return Err(corrupt());
            }
            Ok(ActivityRevision {
                id: revision.into(),
                activity_id,
                unit_id: unit_id.into(),
                definition,
            })
        })
        .transpose()
    }
}

mod dependencies;
mod edits;
mod source_snapshots;
use dependencies::{check_bases, closure, dependency_base, publish, references};
use edits::edit;
use source_snapshots::{source_snapshot, validate_source};

impl Database {
    pub(in crate::workspace) fn lesson_validate(
        &self,
        context: RequestContext,
        r: &LessonDraftScope,
    ) -> Result<LessonValidation> {
        context.authorize()?;
        contract(r.contract_version)?;
        request_budget(r)?;
        if r.workspace_id != self.id {
            return Err(AppError::new(ErrorCode::PermissionDenied));
        }
        unit(&self.connection, &r.unit_id, false)?;
        let s = state(&self.connection, &r.draft_id, &r.unit_id)?;
        expected(&s, r.expected_draft_revision)?;
        let mut diagnostics = super::validation::diagnose(&s.draft.composition, false);
        if diagnostics.len() < 16
            && let Err(error) = references(&self.connection, &s.draft.composition, &s.dependencies)
        {
            diagnostics.extend(error.diagnostics);
        }
        if diagnostics.len() < 16
            && (check_bases(&self.connection, &s).is_err()
                || head(&self.connection, &r.unit_id)? != s.draft.base_head_revision)
        {
            diagnostics.push(super::diagnostics::diagnostic(
                LessonDiagnosticCode::RevisionConflict,
                "baseHeadRevision",
                LessonRepair::RefreshBase,
                None,
                None,
            ));
        }
        diagnostics.truncate(16);
        Ok(LessonValidation {
            draft_id: s.draft.id,
            draft_revision: s.draft.revision,
            valid: diagnostics.is_empty(),
            diagnostics,
        })
    }
}
