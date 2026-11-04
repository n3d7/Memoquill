use super::validation::valid_id;
use super::*;
pub(super) fn diagnostic(
    code: LessonDiagnosticCode,
    field: &str,
    repair: LessonRepair,
    node: Option<&str>,
    binding: Option<&str>,
) -> LessonDiagnostic {
    LessonDiagnostic {
        code,
        node_id: node.filter(|id| valid_id(id)).map(String::from),
        binding_id: binding.filter(|id| valid_id(id)).map(String::from),
        operation_index: None,
        field: field.into(),
        repair,
    }
}
pub(super) fn envelope(code: ErrorCode, field: &str) -> AppError {
    AppError::new(code).with_diagnostics(vec![diagnostic(
        if code == ErrorCode::BudgetExceeded {
            LessonDiagnosticCode::BudgetExceeded
        } else {
            LessonDiagnosticCode::InvalidEnvelope
        },
        field,
        if code == ErrorCode::BudgetExceeded {
            LessonRepair::SplitRequest
        } else {
            LessonRepair::UseClosedEnvelope
        },
        None,
        None,
    )])
}
/// Reports only schema-owned field names, never unknown keys or target identities.
pub(super) fn schema_field(
    schema: &Value,
    value: &Value,
    path: &str,
    depth: usize,
) -> Option<String> {
    if depth > 32 {
        return Some(path.into());
    }
    if let Some(a) = schema.get("oneOf").and_then(Value::as_array) {
        return if a
            .iter()
            .any(|s| schema_field(s, value, path, depth + 1).is_none())
        {
            None
        } else {
            Some(path.into())
        };
    }
    if schema == &Value::Bool(false) {
        return Some(path.into());
    }
    if schema.get("const").is_some_and(|v| v != value)
        || schema
            .get("enum")
            .and_then(Value::as_array)
            .is_some_and(|a| !a.contains(value))
    {
        return Some(path.into());
    }
    let valid = match schema["type"].as_str() {
        Some("object") => value.is_object(),
        Some("array") => value.is_array(),
        Some("string") => value.is_string(),
        Some("boolean") => value.is_boolean(),
        Some("integer") => value.as_u64().is_some() || value.as_i64().is_some(),
        Some("number") => value.is_number(),
        _ => true,
    };
    if !valid {
        return Some(path.into());
    }
    if let Some(s) = value.as_str()
        && (schema["minLength"]
            .as_u64()
            .is_some_and(|n| s.trim().chars().count() < n as usize)
            || schema["maxLength"]
                .as_u64()
                .is_some_and(|n| s.len() > n as usize)
            || s.contains('\0'))
    {
        return Some(path.into());
    }
    if let Some(n) = value.as_f64()
        && (schema["minimum"].as_f64().is_some_and(|min| n < min)
            || schema["maximum"].as_f64().is_some_and(|max| n > max))
    {
        return Some(path.into());
    }
    if let Some(object) = value.as_object() {
        let properties = schema["properties"].as_object();
        if let Some(required) = schema["required"].as_array() {
            for key in required.iter().filter_map(Value::as_str) {
                if !object.contains_key(key) {
                    return Some(format!("{path}.{key}"));
                }
            }
        }
        if schema["additionalProperties"] == false
            && object
                .keys()
                .any(|key| properties.is_none_or(|p| !p.contains_key(key)))
        {
            return Some(path.into());
        }
        if let Some(properties) = properties {
            for (key, rule) in properties {
                if let Some(v) = object.get(key)
                    && let Some(field) = schema_field(rule, v, &format!("{path}.{key}"), depth + 1)
                {
                    return Some(field);
                }
            }
        }
    }
    if let Some(a) = value.as_array() {
        if schema["minItems"]
            .as_u64()
            .is_some_and(|n| a.len() < n as usize)
            || schema["maxItems"]
                .as_u64()
                .is_some_and(|n| a.len() > n as usize)
        {
            return Some(path.into());
        }
        if let Some(rule) = schema.get("items") {
            for v in a {
                if let Some(field) = schema_field(rule, v, path, depth + 1) {
                    return Some(field);
                }
            }
        }
    }
    None
}
pub(super) fn for_operation(
    mut error: AppError,
    index: usize,
    operation: &LessonOperation,
) -> AppError {
    if error.diagnostics.is_empty() {
        let (node, binding, field) = match operation {
            LessonOperation::ConfigureNode { id, .. } => (Some(id.as_str()), None, "props"),
            LessonOperation::ConfigureInteraction { id, .. } => (Some(id.as_str()), None, "on"),
            LessonOperation::InsertNode { node, .. }
            | LessonOperation::ReplaceUnsupportedRoot { node } => {
                (Some(node.node_id.as_str()), None, "node")
            }
            LessonOperation::RemoveNode { id } | LessonOperation::MoveNode { id, .. } => {
                (Some(id.as_str()), None, "nodeId")
            }
            LessonOperation::BindReference { binding } => {
                (None, Some(binding.binding_id.as_str()), "revisionId")
            }
            LessonOperation::CreateActivityRevision { binding_id, .. }
            | LessonOperation::CreateResourceRevision { binding_id, .. }
            | LessonOperation::CreateSourceBindingRevision { binding_id, .. } => {
                (None, Some(binding_id.as_str()), "definition")
            }
            LessonOperation::SetAppearance { .. } => (None, None, "appearance"),
        };
        error.diagnostics.push(diagnostic(
            if error.code == ErrorCode::UnsupportedVersion {
                LessonDiagnosticCode::UnsupportedType
            } else {
                LessonDiagnosticCode::InvalidOperation
            },
            field,
            if error.code == ErrorCode::UnsupportedVersion {
                LessonRepair::RemoveOrReplaceUnsupported
            } else {
                LessonRepair::UseDeclaredValue
            },
            node,
            binding,
        ));
    }
    for d in &mut error.diagnostics {
        d.operation_index = Some(index.min(63) as u32);
    }
    error
}
pub(super) fn attach_operations(mut error: AppError, operations: &[LessonOperation]) -> AppError {
    for d in &mut error.diagnostics {
        let relevant = operations
            .iter()
            .enumerate()
            .rev()
            .find(|(_, op)| match op {
                LessonOperation::ConfigureNode { id, .. }
                | LessonOperation::ConfigureInteraction { id, .. }
                | LessonOperation::RemoveNode { id }
                | LessonOperation::MoveNode { id, .. } => d.node_id.as_ref() == Some(id),
                LessonOperation::InsertNode { node, .. }
                | LessonOperation::ReplaceUnsupportedRoot { node } => {
                    d.node_id.as_ref() == Some(&node.node_id)
                }
                LessonOperation::BindReference { binding } => {
                    d.binding_id.as_ref() == Some(&binding.binding_id)
                }
                LessonOperation::CreateActivityRevision { binding_id, .. }
                | LessonOperation::CreateResourceRevision { binding_id, .. }
                | LessonOperation::CreateSourceBindingRevision { binding_id, .. } => {
                    d.binding_id.as_ref() == Some(binding_id)
                }
                LessonOperation::SetAppearance { .. } => d.field.starts_with("appearance"),
            });
        if let Some((index, _)) = relevant {
            d.operation_index = Some(index as u32);
        }
    }
    error
}
