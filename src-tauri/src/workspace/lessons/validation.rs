use super::*;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
pub(super) const MAX_COMPOSITION: usize = 1024 * 1024;
pub(super) fn bytes<T: Serialize>(v: &T) -> Result<Vec<u8>> {
    serde_json::to_vec(v).map_err(|_| invalid())
}
pub(super) fn valid_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
        && !["__proto__", "constructor", "prototype"].contains(&s)
}
fn text(v: &Value, max: usize) -> bool {
    v.as_str()
        .is_some_and(|s| !s.trim().is_empty() && s.len() <= max && !s.contains('\0'))
}
fn fields(v: &Value, required: &[&str], optional: &[&str]) -> Result<()> {
    let o = v.as_object().ok_or_else(invalid)?;
    if required.iter().any(|k| !o.contains_key(*k))
        || o.keys()
            .any(|k| !required.contains(&k.as_str()) && !optional.contains(&k.as_str()))
    {
        return Err(invalid());
    }
    Ok(())
}
fn one(v: &Value, allowed: &[&str]) -> bool {
    v.as_str().is_some_and(|s| allowed.contains(&s))
}
pub(super) fn discover() -> Vec<RegistryDescriptor> {
    super::catalog::discover()
}

fn view_fields() -> Vec<&'static str> {
    vec![
        "section.expanded",
        "response.present",
        "assistance.hint",
        "assistance.revealed",
        "preview.submitted",
    ]
}
fn field(v: &Value) -> bool {
    one(v, &view_fields())
}
fn scalar(v: &Value) -> bool {
    v.is_null() || v.is_boolean() || v.is_number() || v.as_str().is_some_and(|s| s.len() <= 4096)
}
fn condition(v: &Value) -> Result<()> {
    let mut pending = vec![(v, 1)];
    let mut count = 0;
    while let Some((v, depth)) = pending.pop() {
        count += 1;
        if count > 32 || depth > 4 {
            return Err(invalid());
        }
        match v["kind"].as_str() {
            Some("literal") => {
                fields(v, &["kind", "value"], &[])?;
                if !v["value"].is_boolean() {
                    return Err(invalid());
                }
            }
            Some("eq") => {
                fields(v, &["kind", "field", "value"], &[])?;
                if !field(&v["field"]) || !v["value"].is_boolean() {
                    return Err(invalid());
                }
            }
            // v1 declares boolean view fields only; numeric comparison has no valid field.
            Some("compare") => return Err(invalid()),
            Some("not") => {
                fields(v, &["kind", "expr"], &[])?;
                pending.push((&v["expr"], depth + 1));
            }
            Some("all" | "any") => {
                fields(v, &["kind", "exprs"], &[])?;
                let a = v["exprs"].as_array().ok_or_else(invalid)?;
                if a.is_empty() || a.len() > 32 {
                    return Err(invalid());
                }
                for v in a {
                    pending.push((v, depth + 1));
                }
            }
            _ => return Err(invalid()),
        }
    }
    Ok(())
}
fn action(a: &ActionBinding, c: &Composition, event: Option<&str>) -> Result<()> {
    if a.version != 1 || bytes(a)?.len() > 16384 || a.inputs.len() > 1 {
        return Err(invalid());
    }
    let target = a
        .target_node_id
        .as_ref()
        .and_then(|id| c.nodes.iter().find(|n| &n.node_id == id));
    let allowed = match a.action_id.as_str() {
        "view.toggleSection" => {
            if target.is_none_or(|n| n.type_id != "section") {
                return Err(invalid());
            }
            Some(("expanded", true))
        }
        "view.focusNode" => {
            if target.is_none() {
                return Err(invalid());
            }
            None
        }
        "source.open" => {
            if !a.binding_id.as_ref().is_some_and(|id| {
                c.bindings
                    .iter()
                    .any(|b| &b.binding_id == id && b.target_kind == "source")
            }) {
                return Err(invalid());
            }
            None
        }
        "response.update" => {
            if target.is_none_or(|n| n.type_id != "activity") {
                return Err(invalid());
            }
            Some(("value", false))
        }
        "study.hint" | "study.reveal" | "study.submit" | "study.next" => {
            if target.is_none_or(|n| n.type_id != "activity") {
                return Err(invalid());
            }
            None
        }
        _ => return Err(invalid()),
    };
    if let Some(event) = event {
        let permitted = match event {
            "response" => a.action_id == "response.update",
            "hint" => a.action_id == "study.hint",
            "reveal" => a.action_id == "study.reveal",
            "submit" => a.action_id == "study.submit",
            "openSource" => a.action_id == "source.open",
            "toggle" => matches!(
                a.action_id.as_str(),
                "view.toggleSection" | "view.focusNode"
            ),
            "activate" => true,
            _ => false,
        };
        if !permitted {
            return Err(invalid());
        }
    }
    if let Some((required, _)) = allowed
        && !a.inputs.contains_key(required)
    {
        return Err(invalid());
    }
    for (key, v) in &a.inputs {
        let (name, is_bool) = allowed.ok_or_else(invalid)?;
        if key != name {
            return Err(invalid());
        }
        if v.is_object() {
            fields(v, &["eventField"], &[])?;
            if v["eventField"] != "value" || !matches!(event, Some("response" | "toggle")) {
                return Err(invalid());
            }
        } else if (is_bool && !v.is_boolean()) || (!is_bool && (!scalar(v) || !v.is_string())) {
            return Err(invalid());
        }
    }
    if a.binding_id.is_some() && a.action_id != "source.open" {
        return Err(invalid());
    }
    if a.target_node_id.is_some() && a.action_id == "source.open" {
        return Err(invalid());
    }
    Ok(())
}
fn props(n: &LessonNode, c: &Composition) -> Result<()> {
    let p = &n.props;
    match n.type_id.as_str() {
        "stack" => {
            fields(p, &["gap", "headingLevel"], &[])?;
            if !one(&p["gap"], &["compact", "normal", "wide"])
                || !p["headingLevel"]
                    .as_u64()
                    .is_some_and(|x| (1..=6).contains(&x))
            {
                return Err(invalid());
            }
        }
        "columns" => {
            fields(p, &["ratio"], &[])?;
            if !one(&p["ratio"], &["equal", "widePrimary", "wideSecondary"]) {
                return Err(invalid());
            }
        }
        "section" => {
            fields(p, &["title", "collapsible"], &[])?;
            if !text(&p["title"], 512) || !p["collapsible"].is_boolean() {
                return Err(invalid());
            }
        }
        "content" | "source" | "activity" => {
            let optional = match n.type_id.as_str() {
                "content" => vec!["blockRange"],
                "source" => vec!["locator"],
                _ => vec![],
            };
            let required = if n.type_id == "activity" {
                vec!["bindingId", "presentation"]
            } else {
                vec!["bindingId"]
            };
            fields(p, &required, &optional)?;
            let kind = if n.type_id == "content" {
                "resource"
            } else {
                &n.type_id
            };
            if !c.bindings.iter().any(|b| {
                Some(b.binding_id.as_str()) == p["bindingId"].as_str() && b.target_kind == kind
            }) {
                return Err(invalid());
            }
            if n.type_id == "activity" && !one(&p["presentation"], &["default", "compact"]) {
                return Err(invalid());
            }
            if let Some(range) = p.get("blockRange") {
                fields(range, &["startBlockId", "endBlockId"], &[])?;
                if !text(&range["startBlockId"], 128) || !text(&range["endBlockId"], 128) {
                    return Err(invalid());
                }
            }
            if p.get("locator").is_some_and(|v| !text(v, 1024)) {
                return Err(invalid());
            }
        }
        "button" => {
            fields(p, &["label", "style", "action"], &[])?;
            if !text(&p["label"], 512) || !one(&p["style"], &["primary", "secondary"]) {
                return Err(invalid());
            }
            let a: ActionBinding =
                serde_json::from_value(p["action"].clone()).map_err(|_| invalid())?;
            action(&a, c, Some("activate"))?;
            if n.on.contains_key("activate") {
                return Err(invalid());
            }
        }
        "status" => {
            fields(p, &["field", "label"], &["targetNodeId"])?;
            if !field(&p["field"]) || !text(&p["label"], 512) {
                return Err(invalid());
            }
            if let Some(target) = p.get("targetNodeId") {
                let target = target
                    .as_str()
                    .filter(|s| valid_id(s))
                    .ok_or_else(invalid)?;
                let expected = if p["field"] == "section.expanded" {
                    "section"
                } else {
                    "activity"
                };
                if !c.nodes.iter().any(|node| {
                    node.node_id == target && node.type_version == 1 && node.type_id == expected
                }) {
                    return Err(AppError::new(ErrorCode::InvalidReference));
                }
            }
        }
        _ => return Err(invalid()),
    }
    Ok(())
}
pub(super) fn validate_composition(v: &Value, allow_unknown: bool) -> Result<()> {
    if bytes(v)?.len() > MAX_COMPOSITION {
        return Err(AppError::new(ErrorCode::BudgetExceeded));
    }
    let c: Composition = serde_json::from_value(v.clone()).map_err(|_| invalid())?;
    if c.contract_version != 1
        || !valid_id(&c.unit_id)
        || c.nodes.is_empty()
        || c.nodes.len() > 256
        || c.bindings.len() > 256
        || c.source_binding_refs.len() > 64
    {
        return Err(invalid());
    }
    let ap = &c.appearance;
    if !["compact", "normal", "wide"].contains(&ap.spacing.as_str())
        || !["system", "light", "dark"].contains(&ap.theme.as_str())
        || !["default", "serif"].contains(&ap.typography.as_str())
        || !["normal", "wide"].contains(&ap.width.as_str())
        || !["normal", "strong"].contains(&ap.emphasis.as_str())
        || !["manual", "assisted"].contains(&c.authoring_provenance.kind.as_str())
        || c.authoring_provenance
            .note
            .as_ref()
            .is_some_and(|s| s.len() > 4096)
    {
        return Err(invalid());
    }
    let mut bindings = HashSet::new();
    for b in &c.bindings {
        if !valid_id(&b.binding_id)
            || !bindings.insert(&b.binding_id)
            || !["resource", "activity", "source"].contains(&b.target_kind.as_str())
            || uuid::Uuid::parse_str(&b.target_id).is_err()
            || uuid::Uuid::parse_str(&b.revision_id).is_err()
            || b.locator
                .as_ref()
                .is_some_and(|s| s.len() > 1024 || s.contains('\0'))
        {
            return Err(invalid());
        }
    }
    let mut source = HashSet::new();
    for id in &c.source_binding_refs {
        if !source.insert(id)
            || !c
                .bindings
                .iter()
                .any(|b| &b.binding_id == id && b.target_kind == "source")
        {
            return Err(invalid());
        }
    }
    let nodes: HashMap<_, _> = c.nodes.iter().map(|n| (&n.node_id, n)).collect();
    if nodes.len() != c.nodes.len() || !nodes.contains_key(&c.root_node_id) {
        return Err(AppError::new(ErrorCode::InvalidTree));
    }
    let registry = discover();
    let mut parents = HashSet::new();
    for n in &c.nodes {
        if !valid_id(&n.node_id) || bytes(&n.props)?.len() > 16384 || n.on.len() > 4 {
            return Err(invalid());
        }
        let descriptor = registry
            .iter()
            .find(|d| d.type_id == n.type_id && d.type_version == n.type_version);
        if let Some(d) = descriptor {
            props(n, &c)?;
            if n.slots.len() != d.slots.len()
                || n.slots.keys().any(|s| !d.slots.contains(s))
                || n.on.keys().any(|e| !d.events.contains(e))
            {
                return Err(invalid());
            }
            for (e, a) in &n.on {
                action(a, &c, Some(e))?;
            }
            if let Some(v) = &n.visible {
                condition(v)?;
            }
        } else if !allow_unknown {
            return Err(AppError::new(ErrorCode::UnsupportedVersion));
        }
        for children in n.slots.values() {
            if children.len() > 64 {
                return Err(invalid());
            }
            for id in children {
                if !nodes.contains_key(id) || !parents.insert(id) || id == &c.root_node_id {
                    return Err(AppError::new(ErrorCode::InvalidTree));
                }
            }
        }
    }
    let mut visited = HashSet::new();
    let mut pending = vec![(&c.root_node_id, 1)];
    while let Some((id, depth)) = pending.pop() {
        if depth > 12 || !visited.insert(id) {
            return Err(AppError::new(ErrorCode::InvalidTree));
        }
        for ch in nodes[id].slots.values().flatten() {
            pending.push((ch, depth + 1));
        }
    }
    if visited.len() != nodes.len() {
        return Err(AppError::new(ErrorCode::InvalidTree));
    }
    Ok(())
}
pub(super) fn activity(v: &Value) -> Result<()> {
    if bytes(v)?.len() > 65536 {
        return Err(invalid());
    }
    let family = v["family"].as_str().ok_or_else(invalid)?;
    let extra = match family {
        "recall" => vec!["answer"],
        "singleChoice" => vec!["options", "correctOptionId"],
        "constructed" => vec!["rubric"],
        "procedure" => vec!["steps", "reflection"],
        _ => return Err(invalid()),
    };
    let mut req = vec!["family", "version", "prompt", "hints", "evaluation"];
    req.extend(extra);
    if family == "constructed" && (v["version"] == 2 || v["version"] == 3) {
        req.push("instruction");
    }
    fields(
        v,
        &req,
        if family == "recall" && v["version"] == 2 {
            &["responseFormat"]
        } else if family == "constructed" {
            &["workedExampleBindingId"]
        } else {
            &[]
        },
    )?;
    if !(v["version"] == 1
        || (["recall", "constructed"].contains(&family) && v["version"] == 2)
        || (family == "constructed" && v["version"] == 3))
        || !text(&v["prompt"], 16384)
        || !v["hints"]
            .as_array()
            .is_some_and(|a| a.len() <= 16 && a.iter().all(|v| text(v, 4096)))
    {
        return Err(invalid());
    }
    if family == "recall" {
        if v["version"] == 2 {
            let prompt = v["prompt"].as_str().ok_or_else(invalid)?;
            let remainder = prompt.replace("{{answer}}", "");
            if v["responseFormat"] != "boundedCloze"
                || prompt.matches("{{answer}}").count() != 1
                || remainder.contains("{{")
                || remainder.contains("}}")
                || !text(&v["answer"], 1024)
            {
                return Err(invalid());
            }
        }
        if !text(&v["answer"], 16384) || !one(&v["evaluation"], &["self", "exact"]) {
            return Err(invalid());
        }
    } else if family == "singleChoice" {
        if v["evaluation"] != "exact" {
            return Err(invalid());
        }
        items(&v["options"])?;
        if !v["options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|o| o["id"] == v["correctOptionId"])
        {
            return Err(invalid());
        }
    } else {
        if v["evaluation"] != "self" {
            return Err(invalid());
        }
        if family == "constructed" {
            if v["version"] == 2 {
                let instruction = &v["instruction"];
                fields(instruction, &["phase", "material"], &[])?;
                if !one(&instruction["phase"], &["workedExample", "completion"])
                    || !text(&instruction["material"], 16384)
                {
                    return Err(invalid());
                }
            }
            if v["version"] == 3 {
                let instruction = &v["instruction"];
                fields(
                    instruction,
                    &["phase", "material", "label", "expectedCorrection", "review"],
                    &[],
                )?;
                fields(&instruction["review"], &["reviewer", "reference"], &[])?;
                if instruction["phase"] != "errorRepair"
                    || !text(&instruction["material"], 16384)
                    || !text(&instruction["label"], 1024)
                    || !text(&instruction["expectedCorrection"], 16384)
                    || !text(&instruction["review"]["reviewer"], 1024)
                    || !text(&instruction["review"]["reference"], 4096)
                {
                    return Err(invalid());
                }
            }
            if !text(&v["rubric"], 16384)
                || v.get("workedExampleBindingId")
                    .is_some_and(|x| !text(x, 128))
            {
                return Err(invalid());
            }
        } else {
            items(&v["steps"])?;
            if !text(&v["reflection"], 4096) {
                return Err(invalid());
            }
        }
    }
    Ok(())
}
fn items(v: &Value) -> Result<()> {
    let a = v.as_array().ok_or_else(invalid)?;
    if a.is_empty() || a.len() > 64 {
        return Err(invalid());
    }
    let mut ids = HashSet::new();
    for i in a {
        fields(i, &["id", "text"], &[])?;
        if !i["id"].as_str().is_some_and(valid_id)
            || !ids.insert(i["id"].as_str().unwrap())
            || !text(&i["text"], 4096)
        {
            return Err(invalid());
        }
    }
    Ok(())
}

pub(super) fn diagnose(c: &Composition, allow_unknown: bool) -> Vec<LessonDiagnostic> {
    use super::diagnostics::{diagnostic, schema_field};
    let mut found = vec![];
    if c.nodes.len() > 256
        || c.bindings.len() > 256
        || c.source_binding_refs.len() > 64
        || bytes(c).is_ok_and(|b| b.len() > MAX_COMPOSITION)
    {
        return vec![diagnostic(
            LessonDiagnosticCode::BudgetExceeded,
            "composition",
            LessonRepair::SplitRequest,
            None,
            None,
        )];
    }
    for node in &c.nodes {
        if bytes(&node.props).is_ok_and(|b| b.len() > 16384) {
            found.push(diagnostic(
                LessonDiagnosticCode::BudgetExceeded,
                "props",
                LessonRepair::SplitRequest,
                Some(&node.node_id),
                None,
            ));
            if found.len() == 16 {
                break;
            }
            continue;
        }
        let known = node.type_version == 1
            && [
                "stack", "columns", "section", "content", "source", "activity", "button", "status",
            ]
            .contains(&node.type_id.as_str());
        let issue = if !known && !allow_unknown {
            Some(diagnostic(
                LessonDiagnosticCode::UnsupportedType,
                "typeVersion",
                LessonRepair::RemoveOrReplaceUnsupported,
                Some(&node.node_id),
                None,
            ))
        } else if known {
            if let Some(field) = schema_field(
                &super::catalog::props_schema(&node.type_id),
                &node.props,
                "props",
                0,
            ) {
                Some(diagnostic(
                    LessonDiagnosticCode::InvalidProps,
                    &field,
                    LessonRepair::UseDeclaredValue,
                    Some(&node.node_id),
                    None,
                ))
            } else if let Some(v) = &node.visible
                && condition(v).is_err()
            {
                Some(diagnostic(
                    LessonDiagnosticCode::InvalidCondition,
                    if v.get("field").is_some() {
                        "visible.field"
                    } else {
                        "visible"
                    },
                    LessonRepair::UseDeclaredField,
                    Some(&node.node_id),
                    None,
                ))
            } else if node
                .on
                .iter()
                .any(|(event, a)| action(a, c, Some(event)).is_err())
            {
                Some(diagnostic(
                    LessonDiagnosticCode::InvalidAction,
                    "on",
                    LessonRepair::UseDeclaredValue,
                    Some(&node.node_id),
                    None,
                ))
            } else if props(node, c).is_err() {
                Some(diagnostic(
                    LessonDiagnosticCode::InvalidProps,
                    "props",
                    LessonRepair::SelectPinnedRevision,
                    Some(&node.node_id),
                    None,
                ))
            } else {
                None
            }
        } else {
            None
        };
        if let Some(d) = issue {
            found.push(d);
        }
        if found.len() == 16 {
            break;
        }
    }
    if found.is_empty()
        && let Err(error) = validate_composition(
            &serde_json::to_value(c).unwrap_or(Value::Null),
            allow_unknown,
        )
    {
        found.push(diagnostic(
            if error.code == ErrorCode::BudgetExceeded {
                LessonDiagnosticCode::BudgetExceeded
            } else {
                LessonDiagnosticCode::InvalidTree
            },
            "composition",
            if error.code == ErrorCode::BudgetExceeded {
                LessonRepair::SplitRequest
            } else {
                LessonRepair::CorrectTree
            },
            None,
            None,
        ));
    }
    found
}
pub(super) fn validate_diagnosed(c: &Composition, allow_unknown: bool) -> Result<()> {
    let diagnostics = diagnose(c, allow_unknown);
    if diagnostics.is_empty() {
        return Ok(());
    }
    let code = if diagnostics
        .iter()
        .any(|d| d.code == LessonDiagnosticCode::UnsupportedType)
    {
        ErrorCode::UnsupportedVersion
    } else if diagnostics
        .iter()
        .any(|d| d.code == LessonDiagnosticCode::BudgetExceeded)
    {
        ErrorCode::BudgetExceeded
    } else {
        ErrorCode::InvalidRequest
    };
    Err(AppError::new(code).with_diagnostics(diagnostics))
}

#[cfg(test)]
mod response_tests {
    use super::*;
    #[test]
    fn response_scalar_uses_utf8_bytes_at_ascii_and_multibyte_boundaries() {
        for value in ["a".repeat(4096), "é".repeat(2048), "🙂".repeat(1024)] {
            assert!(scalar(&Value::String(value.clone())));
            assert!(!scalar(&Value::String(value + "a")));
        }
    }
}
