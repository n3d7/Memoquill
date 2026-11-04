use super::*;
pub(super) fn dependency_base(
    c: &Connection,
    s: &DraftState,
    kind: &str,
    target: &str,
    expected: Option<&str>,
) -> Result<()> {
    // Proposals never mutate shared logical targets. Resources must be newly forked or owned by this Unit.
    if let Some(previous) = s
        .dependencies
        .iter()
        .rev()
        .find(|p| p.kind() == kind && p.target() == target)
    {
        if expected != Some(previous.id()) {
            return Err(conflict());
        }
        return Ok(());
    }
    let current = dependency_head(c, kind, target, &s.draft.unit_id)?;
    if current.as_deref() != expected {
        return Err(conflict());
    }
    Ok(())
}
fn dependency_head(c: &Connection, kind: &str, target: &str, unit: &str) -> Result<Option<String>> {
    if kind == "activity" {
        let row: Option<(String, String)> = c
            .query_row(
                "SELECT unit_id,revision_id FROM lesson_activity_heads WHERE activity_id=?1",
                [target],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        match row {
            Some((owner, head)) if owner == unit => Ok(Some(head)),
            Some(_) => Err(AppError::new(ErrorCode::PermissionDenied)),
            None => Ok(None),
        }
    } else {
        let row:Option<Option<String>>=c.query_row("SELECT (SELECT revision_id FROM resource_heads WHERE resource_id=r.id) FROM resources r WHERE r.id=?1",[target],|r|r.get(0)).optional()?;
        if let Some(head) = row {
            let owner: Option<String> = c
                .query_row(
                    "SELECT unit_id FROM lesson_resource_owners WHERE resource_id=?1",
                    [target],
                    |r| r.get(0),
                )
                .optional()?;
            if owner.as_deref() != Some(unit) {
                return Err(AppError::new(ErrorCode::PermissionDenied));
            }
            Ok(head)
        } else {
            Ok(None)
        }
    }
}
pub(super) fn publish(c: &Connection, unit: &str, p: &Pending) -> Result<()> {
    match p {
        Pending::Source { snapshot } => {
            validate_source(c, snapshot, unit)?;
            c.execute(
                "INSERT INTO lesson_source_bindings VALUES(?1,?2,?3,?4,?5)",
                params![
                    snapshot.id,
                    unit,
                    snapshot.source_id,
                    encode(snapshot)?,
                    hash(snapshot)?
                ],
            )?;
        }
        Pending::Activity {
            id,
            target,
            expected,
            definition,
        } => {
            if dependency_head(c, "activity", target, unit)?.as_deref() != expected.as_deref() {
                return Err(conflict());
            }
            activity(definition)?;
            c.execute(
                "INSERT INTO lesson_activity_revisions VALUES(?1,?2,?3,?4,?5)",
                params![id, target, unit, encode(definition)?, hash(definition)?],
            )?;
            c.execute("INSERT INTO lesson_activity_heads VALUES(?1,?2,?3) ON CONFLICT(activity_id) DO UPDATE SET revision_id=excluded.revision_id",params![target,unit,id])?;
        }
        Pending::Resource {
            id,
            target,
            expected,
            title,
            document,
        } => {
            if dependency_head(c, "resource", target, unit)?.as_deref() != expected.as_deref() {
                return Err(conflict());
            }
            if expected.is_none() {
                c.execute(
                    "INSERT INTO resources VALUES(?1,?2,0)",
                    params![target, title],
                )?;
                c.execute(
                    "INSERT INTO lesson_resource_owners VALUES(?1,?2)",
                    params![target, unit],
                )?;
            }
            let revision: i64 = c.query_row(
                "SELECT coalesce(max(revision),0)+1 FROM resource_revisions WHERE resource_id=?1",
                [target],
                |r| r.get(0),
            )?;
            if revision > crate::workspace::types::MAX_REVISION {
                return Err(invalid());
            }
            c.execute(
                "INSERT INTO resource_revisions VALUES(?1,?2,?3,?4,?5)",
                params![
                    id,
                    target,
                    revision,
                    String::from_utf8(document.canonical_bytes().map_err(|_| invalid())?)
                        .map_err(|_| invalid())?,
                    document.digest().map_err(|_| invalid())?
                ],
            )?;
            c.execute("INSERT INTO resource_heads VALUES(?1,?2) ON CONFLICT(resource_id) DO UPDATE SET revision_id=excluded.revision_id",params![target,id])?;
        }
    }
    Ok(())
}
pub(super) fn references(
    c: &Connection,
    composition: &Composition,
    pending: &[Pending],
) -> Result<()> {
    check_references(c, composition, pending).map_err(|e| {
        if e.code == ErrorCode::InvalidRequest {
            AppError::new(ErrorCode::InvalidReference)
        } else {
            e
        }
    })
}
fn check_references(c: &Connection, composition: &Composition, pending: &[Pending]) -> Result<()> {
    let mut needed: HashSet<&str> = composition
        .bindings
        .iter()
        .map(|b| b.revision_id.as_str())
        .collect();
    for p in pending.iter().rev() {
        if needed.contains(p.id()) {
            let previous = match p {
                Pending::Activity { expected, .. } | Pending::Resource { expected, .. } => {
                    expected.as_deref()
                }
                Pending::Source { .. } => None,
            };
            if let Some(previous) = previous {
                needed.insert(previous);
            }
        }
    }
    if needed.len() > 512 {
        return Err(invalid());
    }
    for b in &composition.bindings {
        check_reference(c, composition, pending, b).map_err(|error| {
            if error.code == ErrorCode::InvalidRequest || error.code == ErrorCode::InvalidReference
            {
                AppError::new(ErrorCode::InvalidReference).with_diagnostics(vec![
                    super::super::diagnostics::diagnostic(
                        LessonDiagnosticCode::InvalidReference,
                        "revisionId",
                        LessonRepair::SelectPinnedRevision,
                        None,
                        Some(&b.binding_id),
                    ),
                ])
            } else {
                error
            }
        })?;
    }
    Ok(())
}
fn check_reference(
    c: &Connection,
    composition: &Composition,
    pending: &[Pending],
    b: &Binding,
) -> Result<()> {
    if let Some(p) = pending
        .iter()
        .find(|p| p.id() == b.revision_id && p.target() == b.target_id && p.kind() == b.target_kind)
    {
        match p {
            Pending::Resource { document, .. } => {
                resource_locator(document, b, composition)?;
                assets(c, document)?;
            }
            Pending::Source { snapshot } => {
                validate_source(c, snapshot, &composition.unit_id)?;
                if snapshot.locator != b.locator {
                    return Err(invalid());
                }
            }
            Pending::Activity { definition, .. } => activity_links(definition, composition)?,
        }
        return Ok(());
    }
    match b.target_kind.as_str() {
        "resource" => {
            let data: Option<(String, String)> = c
                .query_row(
                    "SELECT document,digest FROM resource_revisions WHERE id=?1 AND resource_id=?2",
                    params![b.revision_id, b.target_id],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()?;
            let (data, digest) = data.ok_or_else(invalid)?;
            let document =
                crate::document::Document::parse(data.as_bytes()).map_err(|_| invalid())?;
            if document.digest().map_err(|_| invalid())? != digest {
                return Err(invalid());
            }
            resource_locator(&document, b, composition)?;
            assets(c, &document)?;
        }
        "activity" => {
            let row:Option<(String,String)>=c.query_row("SELECT definition,digest FROM lesson_activity_revisions WHERE id=?1 AND activity_id=?2",params![b.revision_id,b.target_id],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
            let (data, digest) = row.ok_or_else(invalid)?;
            if data.len() > 65536 {
                return Err(invalid());
            }
            let v: Value = serde_json::from_str(&data).map_err(|_| invalid())?;
            activity(&v)?;
            activity_links(&v, composition)?;
            if hash(&v)? != digest {
                return Err(invalid());
            }
        }
        "source" => {
            let row:Option<(String,String)>=c.query_row("SELECT data,digest FROM lesson_source_bindings WHERE id=?1 AND source_id=?2 AND unit_id=?3",params![b.revision_id,b.target_id,composition.unit_id],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
            let (data, digest) = row.ok_or_else(invalid)?;
            if data.len() > 131072 {
                return Err(invalid());
            }
            let snapshot: SourceBindingRevision =
                serde_json::from_str(&data).map_err(|_| invalid())?;
            validate_source(c, &snapshot, &composition.unit_id)?;
            if hash(&snapshot)? != digest
                || snapshot.id != b.revision_id
                || snapshot.locator != b.locator
            {
                return Err(invalid());
            }
        }
        _ => return Err(invalid()),
    }

    Ok(())
}

fn resource_locator(
    document: &crate::document::Document,
    b: &Binding,
    composition: &Composition,
) -> Result<()> {
    let ids: Vec<String> = document
        .all_blocks()
        .iter()
        .map(|b| b.id.to_string())
        .collect();
    if b.locator.as_ref().is_some_and(|id| !ids.contains(id)) {
        return Err(invalid());
    }
    for n in &composition.nodes {
        if n.type_id == "content"
            && n.props["bindingId"].as_str() == Some(&b.binding_id)
            && let Some(range) = n.props.get("blockRange")
        {
            let start = ids
                .iter()
                .position(|id| Some(id.as_str()) == range["startBlockId"].as_str());
            let end = ids
                .iter()
                .position(|id| Some(id.as_str()) == range["endBlockId"].as_str());
            if start.is_none() || end.is_none() || start > end {
                return Err(invalid());
            }
        }
    }
    Ok(())
}
fn assets(c: &Connection, document: &crate::document::Document) -> Result<()> {
    for block in document.all_blocks() {
        let digest = match &block.content {
            crate::document::BlockContent::Media { asset_digest, .. } => Some(asset_digest),
            crate::document::BlockContent::Unsupported {
                source: Some(source),
                ..
            } => Some(&source.asset_digest),
            _ => None,
        };
        if let Some(digest) = digest {
            let exists: bool = c.query_row(
                "SELECT EXISTS(SELECT 1 FROM assets WHERE digest=?1)",
                [digest],
                |r| r.get(0),
            )?;
            if !exists {
                return Err(invalid());
            }
        }
    }
    Ok(())
}

pub(super) fn closure(s: &DraftState) -> Result<HashSet<String>> {
    let mut needed: HashSet<String> = s
        .draft
        .composition
        .bindings
        .iter()
        .map(|b| b.revision_id.clone())
        .collect();
    for p in s.dependencies.iter().rev() {
        if needed.contains(p.id()) {
            let previous = match p {
                Pending::Activity { expected, .. } | Pending::Resource { expected, .. } => expected,
                Pending::Source { .. } => &None,
            };
            if let Some(id) = previous {
                needed.insert(id.clone());
            }
        }
    }
    if needed.len() > 512 {
        return Err(invalid());
    }
    Ok(needed)
}

fn activity_links(definition: &Value, composition: &Composition) -> Result<()> {
    if let Some(id) = definition.get("workedExampleBindingId")
        && !composition
            .bindings
            .iter()
            .any(|b| Some(b.binding_id.as_str()) == id.as_str() && b.target_kind == "resource")
    {
        return Err(invalid());
    }
    Ok(())
}
pub(super) fn check_bases(c: &Connection, s: &DraftState) -> Result<()> {
    let selected = closure(s)?;
    for (index, p) in s.dependencies.iter().enumerate() {
        if !selected.contains(p.id()) {
            continue;
        }
        let expected = match p {
            Pending::Source { .. } => continue,
            Pending::Activity { expected, .. } | Pending::Resource { expected, .. } => expected,
        };
        if s.dependencies[..index].iter().any(|prior| {
            prior.kind() == p.kind()
                && prior.target() == p.target()
                && Some(prior.id()) == expected.as_deref()
        }) {
            continue;
        }
        if dependency_head(c, p.kind(), p.target(), &s.draft.unit_id)?.as_deref()
            != expected.as_deref()
        {
            return Err(conflict());
        }
    }
    Ok(())
}
