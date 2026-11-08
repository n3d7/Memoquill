//! Bounds application-controlled decoding/admission; Tauri's existing transport buffer is external.
use super::*;
use serde::de::{DeserializeOwned, DeserializeSeed, Error, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};
use std::io::Write;
use tauri::ipc::InvokeBody;
const MAX_BYTES: usize = 2 * 1024 * 1024;
const MAX_DEPTH: usize = 96;
const MAX_NODES: usize = 262_144;
fn budget() -> AppError {
    super::diagnostics::envelope(ErrorCode::BudgetExceeded, "request")
}
struct BoundedWriter {
    bytes: Vec<u8>,
    depth: usize,
    string: bool,
    escaped: bool,
}
impl BoundedWriter {
    fn new() -> Self {
        Self {
            bytes: Vec::new(),
            depth: 0,
            string: false,
            escaped: false,
        }
    }
}
impl Write for BoundedWriter {
    fn write(&mut self, chunk: &[u8]) -> std::io::Result<usize> {
        if chunk.len() > MAX_BYTES.saturating_sub(self.bytes.len()) {
            return Err(std::io::Error::other("envelope budget"));
        }
        for b in chunk {
            if self.string {
                if self.escaped {
                    self.escaped = false;
                } else if *b == b'\\' {
                    self.escaped = true;
                } else if *b == b'"' {
                    self.string = false;
                }
            } else if *b == b'"' {
                self.string = true;
            } else if matches!(*b, b'{' | b'[') {
                self.depth += 1;
                if self.depth > MAX_DEPTH {
                    return Err(std::io::Error::other("envelope depth"));
                }
            } else if matches!(*b, b'}' | b']') {
                self.depth = self.depth.saturating_sub(1);
            }
        }
        self.bytes.extend_from_slice(chunk);
        Ok(chunk.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
struct Scan {
    nodes: usize,
    operations: usize,
    budget: bool,
}
struct Seed<'a> {
    state: &'a mut Scan,
    depth: usize,
    operations: bool,
}
impl<'de> DeserializeSeed<'de> for Seed<'_> {
    type Value = ();
    fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> std::result::Result<(), D::Error> {
        self.state.nodes += 1;
        if self.depth > MAX_DEPTH || self.state.nodes > MAX_NODES {
            self.state.budget = true;
            return Err(D::Error::custom("budget"));
        }
        d.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for Seed<'_> {
    type Value = ();
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("bounded JSON value")
    }
    fn visit_bool<E: Error>(self, _: bool) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_i64<E: Error>(self, _: i64) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_u64<E: Error>(self, _: u64) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_f64<E: Error>(self, _: f64) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_unit<E: Error>(self) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_str<E: Error>(self, _: &str) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_string<E: Error>(self, _: String) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> std::result::Result<(), A::Error> {
        let mut count = 0;
        while a
            .next_element_seed(Seed {
                state: self.state,
                depth: self.depth + 1,
                operations: false,
            })?
            .is_some()
        {
            count += 1;
            if self.operations {
                self.state.operations += 1;
            }
            if count > crate::document::MAX_NODES || self.state.operations > 64 {
                self.state.budget = true;
                return Err(A::Error::custom("budget"));
            }
        }
        Ok(())
    }
    fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> std::result::Result<(), A::Error> {
        let mut count = 0;
        while let Some(key) = a.next_key::<String>()? {
            count += 1;
            if key.len() > 128 || count > crate::document::MAX_NODES {
                self.state.budget = true;
                return Err(A::Error::custom("budget"));
            }
            a.next_value_seed(Seed {
                state: self.state,
                depth: self.depth + 1,
                operations: key == "operations",
            })?;
        }
        Ok(())
    }
}
fn preflight(bytes: &[u8]) -> Result<()> {
    if bytes.len() > MAX_BYTES {
        return Err(budget());
    }
    let mut state = Scan {
        nodes: 0,
        operations: 0,
        budget: false,
    };
    let mut d = serde_json::Deserializer::from_slice(bytes);
    let result = Seed {
        state: &mut state,
        depth: 1,
        operations: false,
    }
    .deserialize(&mut d)
    .and_then(|_| d.end());
    result.map_err(|_| {
        if state.budget {
            budget()
        } else {
            super::diagnostics::envelope(ErrorCode::InvalidRequest, "request")
        }
    })
}
fn json_bytes(value: &Value) -> Result<Vec<u8>> {
    let mut pending = vec![(value, 1, false)];
    let mut count = 0;
    let mut operations = 0;
    while let Some((v, depth, ops)) = pending.pop() {
        count += 1;
        if depth > MAX_DEPTH || count + pending.len() > MAX_NODES {
            return Err(budget());
        }
        match v {
            Value::Array(a) => {
                if ops {
                    operations += a.len();
                }
                if a.len() > crate::document::MAX_NODES || operations > 64 {
                    return Err(budget());
                }
                pending.extend(a.iter().map(|v| (v, depth + 1, false)));
            }
            Value::Object(o) => {
                if o.len() > 4096 || o.keys().any(|k| k.len() > 128) {
                    return Err(budget());
                }
                pending.extend(o.iter().map(|(k, v)| (v, depth + 1, k == "operations")));
            }
            _ => {}
        }
    }
    serialize(value)
}
fn serialize(value: &impl Serialize) -> Result<Vec<u8>> {
    let mut writer = BoundedWriter::new();
    serde_json::to_writer(&mut writer, value).map_err(|_| budget())?;
    Ok(writer.bytes)
}
pub(crate) fn admit(value: &impl Serialize) -> Result<()> {
    preflight(&serialize(value)?)
}
pub(crate) fn admit_edit(value: &EditLessonDraft) -> Result<()> {
    if value.operations.len() > 64 {
        return Err(budget());
    }
    let mut inserted = 0usize;
    let mut bound = 0usize;
    for operation in &value.operations {
        match operation {
            super::types::LessonOperation::InsertNode { .. } => inserted += 1,
            super::types::LessonOperation::BindReference { .. } => bound += 1,
            super::types::LessonOperation::InsertSubtree {
                nodes, bindings, ..
            } => {
                inserted += nodes.len();
                bound += bindings.len();
            }
            _ => {}
        }
        if let super::types::LessonOperation::InsertSubtree {
            nodes, bindings, ..
        } = operation
            && (nodes.is_empty() || nodes.len() > 256 || bindings.len() > 256)
        {
            return Err(budget());
        }
    }
    if inserted > 256 || bound > 256 {
        return Err(budget());
    }
    admit(value)
}
pub(crate) fn decode<T: DeserializeOwned>(body: &InvokeBody) -> Result<T> {
    let owned;
    let bytes = match body {
        InvokeBody::Raw(bytes) => bytes.as_slice(),
        InvokeBody::Json(v) => {
            owned = json_bytes(v)?;
            &owned
        }
    };
    preflight(bytes)?;
    serde_json::from_slice(bytes)
        .map_err(|_| super::diagnostics::envelope(ErrorCode::InvalidRequest, "request"))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Mutation<T> {
    pub request: T,
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static DECODED: AtomicUsize = AtomicUsize::new(0);
    struct Probe;
    impl<'de> serde::Deserialize<'de> for Probe {
        fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
            DECODED.fetch_add(1, Ordering::SeqCst);
            let _ = serde::de::IgnoredAny::deserialize(d)?;
            Ok(Self)
        }
    }
    #[test]
    fn oversized_deep_and_operation_heavy_bodies_are_rejected_before_typed_expansion() {
        let oversized = format!("{{\"request\":\"{}\"}}", "x".repeat(2 * 1024 * 1024));
        let deep = format!(
            "{}null{}",
            "[".repeat(MAX_DEPTH + 8),
            "]".repeat(MAX_DEPTH + 8)
        );
        let operations =
            serde_json::json!({"request":{"operations":vec![serde_json::json!({});65]}});
        for body in [
            InvokeBody::Raw(oversized.into_bytes()),
            InvokeBody::Raw(deep.into_bytes()),
            InvokeBody::Raw(serde_json::to_vec(&operations).unwrap()),
            InvokeBody::Json(operations),
            InvokeBody::Raw(
                format!(
                    "{{\"request\":{{\"oper\\u0061tions\":[{}]}}}}",
                    vec!["{}"; 65].join(",")
                )
                .into_bytes(),
            ),
        ] {
            DECODED.store(0, Ordering::SeqCst);
            assert!(decode::<Probe>(&body).is_err());
            assert_eq!(DECODED.load(Ordering::SeqCst), 0);
        }
    }
    #[test]
    fn existing_maximum_depth_document_fits_the_bounded_mutation_decoder() {
        use crate::document::{Block, BlockContent, Document};
        let mut block = Block {
            id: uuid::Uuid::new_v4(),
            content: BlockContent::Code {
                text: "leaf".into(),
                language: None,
            },
        };
        for _ in 1..crate::document::MAX_DEPTH {
            block = Block {
                id: uuid::Uuid::new_v4(),
                content: BlockContent::List {
                    ordered: false,
                    items: vec![vec![block]],
                },
            };
        }
        let document = Document {
            schema_version: 1,
            blocks: vec![block],
        };
        document.validate().unwrap();
        let request = EditLessonDraft {
            contract_version: 1,
            workspace_id: crate::workspace::WorkspaceId::generate(),
            unit_id: uuid::Uuid::new_v4().to_string(),
            draft_id: uuid::Uuid::new_v4().to_string(),
            expected_draft_revision: 1,
            operation_id: crate::workspace::OperationId::generate(),
            operations: vec![LessonOperation::CreateResourceRevision {
                resource_id: uuid::Uuid::new_v4().to_string(),
                expected_revision_id: None,
                title: "Deep existing document".into(),
                binding_id: "resource".into(),
                document,
            }],
        };
        let body =
            InvokeBody::Raw(serde_json::to_vec(&serde_json::json!({"request":request})).unwrap());
        assert!(decode::<Mutation<EditLessonDraft>>(&body).is_ok());
    }
    #[test]
    fn existing_maximum_width_document_fits_the_bounded_mutation_decoder() {
        use crate::document::{Block, BlockContent, Document};
        let document = Document {
            schema_version: 1,
            blocks: (0..crate::document::MAX_NODES)
                .map(|_| Block {
                    id: uuid::Uuid::new_v4(),
                    content: BlockContent::Code {
                        text: "x".into(),
                        language: None,
                    },
                })
                .collect(),
        };
        document.validate().unwrap();
        let request = EditLessonDraft {
            contract_version: 1,
            workspace_id: crate::workspace::WorkspaceId::generate(),
            unit_id: uuid::Uuid::new_v4().to_string(),
            draft_id: uuid::Uuid::new_v4().to_string(),
            expected_draft_revision: 1,
            operation_id: crate::workspace::OperationId::generate(),
            operations: vec![LessonOperation::CreateResourceRevision {
                resource_id: uuid::Uuid::new_v4().to_string(),
                expected_revision_id: None,
                title: "Wide existing document".into(),
                binding_id: "resource".into(),
                document,
            }],
        };
        let body =
            InvokeBody::Raw(serde_json::to_vec(&serde_json::json!({"request":request})).unwrap());
        assert!(decode::<Mutation<EditLessonDraft>>(&body).is_ok());
    }
}
