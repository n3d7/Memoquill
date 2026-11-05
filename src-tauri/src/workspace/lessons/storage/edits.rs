use super::*;
fn find<'a>(c: &'a mut Composition, id: &str) -> Result<&'a mut LessonNode> {
    c.nodes
        .iter_mut()
        .find(|n| n.node_id == id)
        .ok_or_else(invalid)
}
fn detach(c: &mut Composition, id: &str) -> Result<()> {
    if id == c.root_node_id {
        return Err(invalid());
    }
    let mut found = false;
    for n in &mut c.nodes {
        for ch in n.slots.values_mut() {
            if let Some(i) = ch.iter().position(|x| x == id) {
                if n.type_version != 1
                    || !["stack", "columns", "section"].contains(&n.type_id.as_str())
                {
                    return Err(AppError::new(ErrorCode::UnsupportedVersion));
                }
                ch.remove(i);
                found = true;
            }
        }
    }
    if !found {
        return Err(invalid());
    }
    Ok(())
}
fn attach(c: &mut Composition, id: String, parent: &str, slot: &str, index: usize) -> Result<()> {
    let parent_node = find(c, parent)?;
    if !known(parent_node) {
        return Err(AppError::new(ErrorCode::UnsupportedVersion));
    }
    let children = parent_node.slots.get_mut(slot).ok_or_else(invalid)?;
    if index > children.len() || children.len() >= 64 {
        return Err(invalid());
    }
    children.insert(index, id);
    Ok(())
}
fn bind(c: &mut Composition, b: Binding) {
    if let Some(existing) = c
        .bindings
        .iter_mut()
        .find(|old| old.binding_id == b.binding_id)
    {
        *existing = b;
    } else {
        c.bindings.push(b);
    }
    c.source_binding_refs = c
        .bindings
        .iter()
        .filter(|b| b.target_kind == "source")
        .map(|b| b.binding_id.clone())
        .collect();
}
pub(super) fn edit(c: &Connection, s: &mut DraftState, op: &LessonOperation) -> Result<()> {
    match op {
        LessonOperation::ReplaceUnsupportedRoot { node } => {
            let root = s
                .draft
                .composition
                .nodes
                .iter()
                .find(|n| n.node_id == s.draft.composition.root_node_id)
                .ok_or_else(invalid)?;
            if known(root)
                || !known(node)
                || s.draft
                    .composition
                    .nodes
                    .iter()
                    .any(|n| n.node_id == node.node_id)
            {
                return Err(invalid());
            }
            let old = s.draft.composition.root_node_id.clone();
            s.draft.composition.nodes.retain(|n| n.node_id != old);
            s.draft.composition.root_node_id = node.node_id.clone();
            s.draft.composition.nodes.push(node.clone());
        }
        LessonOperation::CreateSourceBindingRevision {
            source_id,
            binding_id,
            locator,
            declared_used_ranges,
        } => {
            let snapshot = source_snapshot(
                c,
                &s.draft.unit_id,
                source_id,
                locator.clone(),
                declared_used_ranges.clone(),
            )?;
            let binding = Binding {
                binding_id: binding_id.clone(),
                target_kind: "source".into(),
                target_id: source_id.clone(),
                revision_id: snapshot.id.clone(),
                locator: locator.clone(),
            };
            s.dependencies.push(Pending::Source { snapshot });
            bind(&mut s.draft.composition, binding);
        }
        LessonOperation::InsertNode {
            parent,
            slot,
            index,
            node,
        } => {
            if !known(node) {
                return Err(AppError::new(ErrorCode::UnsupportedVersion));
            }
            if s.draft
                .composition
                .nodes
                .iter()
                .any(|n| n.node_id == node.node_id)
            {
                return Err(invalid());
            }
            attach(
                &mut s.draft.composition,
                node.node_id.clone(),
                parent,
                slot,
                *index,
            )?;
            s.draft.composition.nodes.push(node.clone());
        }
        LessonOperation::InsertSubtree {
            parent,
            slot,
            index,
            root_node_id,
            nodes,
            bindings,
        } => {
            if nodes.is_empty() || nodes.len() > 256 || bindings.len() > 256 {
                return Err(invalid());
            }
            let mut ids = HashSet::new();
            for node in nodes {
                if !known(node)
                    || !ids.insert(node.node_id.clone())
                    || s.draft
                        .composition
                        .nodes
                        .iter()
                        .any(|n| n.node_id == node.node_id)
                {
                    return Err(invalid());
                }
            }
            let mut visited = HashSet::new();
            let mut pending = vec![root_node_id.clone()];
            while let Some(id) = pending.pop() {
                if !visited.insert(id.clone()) {
                    return Err(invalid());
                }
                let node = nodes.iter().find(|n| n.node_id == id).ok_or_else(invalid)?;
                pending.extend(node.slots.values().flatten().cloned());
            }
            if visited.len() != nodes.len() {
                return Err(invalid());
            }
            for binding in bindings {
                bind(&mut s.draft.composition, binding.clone());
            }
            attach(
                &mut s.draft.composition,
                root_node_id.clone(),
                parent,
                slot,
                *index,
            )?;
            s.draft.composition.nodes.extend(nodes.iter().cloned());
        }
        LessonOperation::RemoveNode { id } => {
            detach(&mut s.draft.composition, id)?;
            let mut removed = HashSet::new();
            let mut pending = vec![id.clone()];
            while let Some(id) = pending.pop() {
                if !removed.insert(id.clone()) {
                    return Err(invalid());
                }
                let node = find(&mut s.draft.composition, &id)?;
                pending.extend(node.slots.values().flatten().cloned());
            }
            s.draft
                .composition
                .nodes
                .retain(|n| !removed.contains(&n.node_id));
        }
        LessonOperation::MoveNode {
            id,
            parent,
            slot,
            index,
        } => {
            detach(&mut s.draft.composition, id)?;
            attach(&mut s.draft.composition, id.clone(), parent, slot, *index)?;
        }
        LessonOperation::ConfigureNode { id, props } => {
            let n = find(&mut s.draft.composition, id)?;
            if !known(n) {
                return Err(AppError::new(ErrorCode::UnsupportedVersion));
            }
            n.props = props.clone();
        }
        LessonOperation::ConfigureInteraction { id, on, visible } => {
            let n = find(&mut s.draft.composition, id)?;
            if !known(n) {
                return Err(AppError::new(ErrorCode::UnsupportedVersion));
            }
            n.on = on.clone();
            n.visible = visible.clone();
        }
        LessonOperation::BindReference { binding } => {
            bind(&mut s.draft.composition, binding.clone())
        }
        LessonOperation::SetAppearance { tokens } => {
            s.draft.composition.appearance = tokens.clone()
        }
        LessonOperation::CreateActivityRevision {
            activity_id,
            expected_revision_id,
            binding_id,
            definition,
        } => {
            uuid(activity_id)?;
            activity(definition)?;
            dependency_base(
                c,
                s,
                "activity",
                activity_id,
                expected_revision_id.as_deref(),
            )?;
            let id = Uuid::new_v4().to_string();
            s.dependencies.push(Pending::Activity {
                id: id.clone(),
                target: activity_id.clone(),
                expected: expected_revision_id.clone(),
                definition: definition.clone(),
            });
            bind(
                &mut s.draft.composition,
                Binding {
                    binding_id: binding_id.clone(),
                    target_kind: "activity".into(),
                    target_id: activity_id.clone(),
                    revision_id: id,
                    locator: None,
                },
            );
        }
        LessonOperation::CreateResourceRevision {
            resource_id,
            expected_revision_id,
            title,
            binding_id,
            document,
        } => {
            uuid(resource_id)?;
            document.validate().map_err(|_| invalid())?;
            if title.trim().is_empty() || title.len() > 512 {
                return Err(invalid());
            }
            dependency_base(
                c,
                s,
                "resource",
                resource_id,
                expected_revision_id.as_deref(),
            )?;
            let id = Uuid::new_v4().to_string();
            s.dependencies.push(Pending::Resource {
                id: id.clone(),
                target: resource_id.clone(),
                expected: expected_revision_id.clone(),
                title: title.clone(),
                document: document.clone(),
            });
            bind(
                &mut s.draft.composition,
                Binding {
                    binding_id: binding_id.clone(),
                    target_kind: "resource".into(),
                    target_id: resource_id.clone(),
                    revision_id: id,
                    locator: None,
                },
            );
        }
    }
    if s.dependencies.len() > 512 || s.draft.composition.nodes.len() > 256 {
        return Err(invalid());
    }
    Ok(())
}

fn known(node: &LessonNode) -> bool {
    super::super::validation::discover()
        .iter()
        .any(|d| d.type_id == node.type_id && d.type_version == node.type_version)
}
