//! App-owned documents. Editor state and imported markup are never storage authority.
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

mod anchors;
mod validation;
pub use anchors::{Anchor, AnchorResolution, ResourceRevision, utf16_range};

pub const SCHEMA_VERSION: u32 = 1;
pub const MAX_DOCUMENT_BYTES: usize = 1024 * 1024;
pub const MAX_NODES: usize = 10_000;
pub const MAX_DEPTH: usize = 16;
pub const MAX_TEXT_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Document {
    pub schema_version: u32,
    pub blocks: Vec<Block>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Block {
    pub id: uuid::Uuid,
    pub content: BlockContent,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum BlockContent {
    Paragraph {
        inlines: Vec<Inline>,
    },
    Heading {
        level: u8,
        inlines: Vec<Inline>,
    },
    List {
        ordered: bool,
        items: Vec<Vec<Block>>,
    },
    Code {
        text: String,
        language: Option<String>,
    },
    Math {
        source: String,
    },
    Table {
        rows: Vec<Vec<Vec<Block>>>,
    },
    Media {
        asset_digest: String,
        mime: String,
        caption: String,
        alt: String,
    },
    Unsupported {
        original: String,
        source: Option<SourceReference>,
        reason: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Inline {
    pub text: String,
    #[serde(default)]
    pub marks: Vec<Mark>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Mark {
    Emphasis {},
    Strong {},
    Code {},
    Link { href: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceReference {
    pub asset_digest: String,
    pub locator: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentError {
    Invalid,
    UnsupportedVersion,
    TooLarge,
    InvalidAnchor,
}

impl std::fmt::Display for DocumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for DocumentError {}

impl Document {
    pub fn parse(bytes: &[u8]) -> Result<Self, DocumentError> {
        if bytes.len() > MAX_DOCUMENT_BYTES {
            return Err(DocumentError::TooLarge);
        }
        let document: Self = serde_json::from_slice(bytes).map_err(|_| DocumentError::Invalid)?;
        document.validate()?;
        Ok(document)
    }

    pub fn validate(&self) -> Result<(), DocumentError> {
        validation::validate(self)
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, DocumentError> {
        self.validate()?;
        let value = serde_json::to_value(self).map_err(|_| DocumentError::Invalid)?;
        serde_json::to_vec(&sorted(value)).map_err(|_| DocumentError::Invalid)
    }

    pub fn digest(&self) -> Result<String, DocumentError> {
        use sha2::{Digest, Sha256};
        Ok(format!("{:x}", Sha256::digest(self.canonical_bytes()?)))
    }

    pub fn text(&self) -> String {
        self.all_blocks()
            .iter()
            .map(|block| block.text())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub(crate) fn all_blocks(&self) -> Vec<&Block> {
        let mut pending: Vec<_> = self.blocks.iter().rev().collect();
        let mut blocks = Vec::new();
        while let Some(block) = pending.pop() {
            blocks.push(block);
            pending.extend(block.children().into_iter().rev());
        }
        blocks
    }
}

fn sorted(value: serde_json::Value) -> serde_json::Value {
    use serde_json::Value;
    match value {
        Value::Object(map) => {
            let entries: std::collections::BTreeMap<_, _> = map.into_iter().collect();
            Value::Object(
                entries
                    .into_iter()
                    .map(|(key, value)| (key, sorted(value)))
                    .collect(),
            )
        }
        Value::Array(array) => Value::Array(array.into_iter().map(sorted).collect()),
        other => other,
    }
}

impl Block {
    pub fn text(&self) -> String {
        match &self.content {
            BlockContent::Paragraph { inlines } | BlockContent::Heading { inlines, .. } => {
                inlines.iter().map(|i| i.text.as_str()).collect()
            }
            BlockContent::Code { text, .. } => text.clone(),
            BlockContent::Math { source } => source.clone(),
            BlockContent::Media { caption, .. } => caption.clone(),
            BlockContent::Unsupported { .. }
            | BlockContent::List { .. }
            | BlockContent::Table { .. } => String::new(),
        }
    }

    pub(crate) fn children(&self) -> Vec<&Block> {
        match &self.content {
            BlockContent::List { items, .. } => items.iter().flatten().collect(),
            BlockContent::Table { rows } => rows.iter().flatten().flatten().collect(),
            _ => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paragraph(text: &str) -> Document {
        Document {
            schema_version: 1,
            blocks: vec![Block {
                id: uuid::Uuid::from_u128(1),
                content: BlockContent::Paragraph {
                    inlines: vec![Inline {
                        text: text.into(),
                        marks: vec![],
                    }],
                },
            }],
        }
    }

    #[test]
    fn canonical_document_preserves_authored_unicode_and_identity() {
        let bytes = br#"{"schemaVersion":1,"blocks":[{"id":"00000000-0000-4000-8000-000000000001","content":{"type":"paragraph","inlines":[{"text":"Quantum physics"}]}}]}"#;
        let document = Document::parse(bytes).unwrap();
        assert_eq!(document.schema_version, 1);
        assert_eq!(
            document.blocks[0].id.to_string(),
            "00000000-0000-4000-8000-000000000001"
        );
        assert!(
            matches!(&document.blocks[0].content, BlockContent::Paragraph { inlines } if inlines[0].text == "Quantum physics")
        );
    }

    #[test]
    fn parser_rejects_future_versions_empty_documents_and_duplicate_identity() {
        let valid = r#"{"schemaVersion":1,"blocks":[{"id":"00000000-0000-4000-8000-000000000001","content":{"type":"paragraph","inlines":[{"text":"Quantum physics"}]}}]}"#;
        let mut value: serde_json::Value = serde_json::from_str(valid).unwrap();
        value["schemaVersion"] = 2.into();
        assert!(Document::parse(&serde_json::to_vec(&value).unwrap()).is_err());
        value["schemaVersion"] = 1.into();
        let block = value["blocks"][0].clone();
        value["blocks"].as_array_mut().unwrap().push(block);
        assert!(Document::parse(&serde_json::to_vec(&value).unwrap()).is_err());
        value["blocks"] = serde_json::json!([]);
        assert!(Document::parse(&serde_json::to_vec(&value).unwrap()).is_err());
    }

    #[test]
    fn all_four_subjects_round_trip_the_same_passive_structured_contract() {
        for fixture in [
            include_bytes!("../../tests/fixtures/documents/quantum.json").as_slice(),
            include_bytes!("../../tests/fixtures/documents/english.json").as_slice(),
            include_bytes!("../../tests/fixtures/documents/python.json").as_slice(),
            include_bytes!("../../tests/fixtures/documents/cooking.json").as_slice(),
        ] {
            let document = Document::parse(fixture).unwrap();
            assert_eq!(
                Document::parse(&document.canonical_bytes().unwrap()).unwrap(),
                document
            );
        }
    }

    #[test]
    fn canonical_bytes_are_independent_of_json_property_order() {
        let document = paragraph("😀 e\u{301} العربية\n  code");
        let bytes = document.canonical_bytes().unwrap();
        let reordered =
            serde_json::to_vec_pretty(&serde_json::to_value(&document).unwrap()).unwrap();
        assert_eq!(
            Document::parse(&reordered)
                .unwrap()
                .canonical_bytes()
                .unwrap(),
            bytes
        );
        assert_eq!(
            Document::parse(&bytes).unwrap().text(),
            "😀 e\u{301} العربية\n  code"
        );
        assert_eq!(paragraph("A").canonical_bytes().unwrap(), br#"{"blocks":[{"content":{"inlines":[{"marks":[],"text":"A"}],"type":"paragraph"},"id":"00000000-0000-0000-0000-000000000001"}],"schemaVersion":1}"#);
        assert_ne!(
            document.digest().unwrap(),
            paragraph("Changed").digest().unwrap()
        );
    }

    #[test]
    fn untrusted_urls_shapes_and_unknown_blocks_are_rejected_without_losing_source() {
        let mut document = paragraph("Source");
        for href in [
            "javascript:alert(1)",
            "data:text/html,active",
            "file:///private",
            "https://example.org\\x",
            "https://user@host",
            "http://",
            "https://x\ninvalid",
            "%6aavascript:alert(1)",
        ] {
            document.blocks[0].content = BlockContent::Paragraph {
                inlines: vec![Inline {
                    text: "link".into(),
                    marks: vec![Mark::Link { href: href.into() }],
                }],
            };
            assert!(document.validate().is_err(), "{href}");
        }
        let mut value = serde_json::to_value(paragraph("source")).unwrap();
        value["blocks"][0]["content"]["onerror"] = "execute".into();
        assert!(Document::parse(&serde_json::to_vec(&value).unwrap()).is_err());
        value["blocks"][0]["content"] = serde_json::json!({"type":"future_widget","source":"keep"});
        assert!(Document::parse(&serde_json::to_vec(&value).unwrap()).is_err());
        document.blocks[0].content = BlockContent::Unsupported {
            original: "<svg onload='active()'>😀</svg>".into(),
            source: None,
            reason: "Unsupported source retained".into(),
        };
        let restored = Document::parse(&document.canonical_bytes().unwrap()).unwrap();
        assert_eq!(restored, document);
        assert!(!restored.text().contains("active"));
    }

    #[test]
    fn parsing_limits_and_semantic_limits_are_enforced_at_the_document_seam() {
        assert_eq!(
            Document::parse(&vec![b' '; MAX_DOCUMENT_BYTES + 1]),
            Err(DocumentError::TooLarge)
        );
        assert!(
            paragraph(&"a".repeat(MAX_TEXT_BYTES + 1))
                .validate()
                .is_err()
        );
        assert!(paragraph("NUL\0").validate().is_err());
        let mut document = paragraph("");
        document.blocks[0].content = BlockContent::Math {
            source: "x".repeat(2049),
        };
        assert!(document.validate().is_err());
        document.blocks[0].content = BlockContent::Heading {
            level: 7,
            inlines: vec![],
        };
        assert!(document.validate().is_err());
        document.blocks[0].content = BlockContent::Table {
            rows: vec![vec![], vec![]],
        };
        assert!(document.validate().is_err());
        document.blocks[0].content = BlockContent::Media {
            asset_digest: "a".repeat(64),
            mime: "image/svg+xml".into(),
            caption: "x".into(),
            alt: "x".into(),
        };
        assert!(document.validate().is_err());
        document.blocks[0].content = BlockContent::Paragraph {
            inlines: (0..MAX_NODES)
                .map(|_| Inline {
                    text: "".into(),
                    marks: vec![],
                })
                .collect(),
        };
        assert!(document.validate().is_err());
    }

    #[test]
    fn exact_text_formula_and_node_limits_remain_usable() {
        paragraph(&"a".repeat(MAX_TEXT_BYTES)).validate().unwrap();
        let mut document = paragraph("");
        document.blocks[0].content = BlockContent::Math {
            source: "x".repeat(2048),
        };
        document.validate().unwrap();
        document.blocks[0].content = BlockContent::Paragraph {
            inlines: (0..MAX_NODES - 1)
                .map(|_| Inline {
                    text: "".into(),
                    marks: vec![],
                })
                .collect(),
        };
        document.validate().unwrap();
        let resource = uuid::Uuid::from_u128(1);
        let id = uuid::Uuid::from_u128(2);
        let revision = ResourceRevision::from_record(resource, id, 1, paragraph("stored")).unwrap();
        assert_eq!(
            (revision.resource_id(), revision.id(), revision.number()),
            (resource, id, 1)
        );
        assert!(
            ResourceRevision::from_record(resource, uuid::Uuid::nil(), 1, paragraph("stored"))
                .is_err()
        );
        assert!(ResourceRevision::from_record(resource, id, 0, paragraph("stored")).is_err());
    }

    #[test]
    fn utf16_anchors_preserve_emoji_combining_marks_and_reject_surrogate_splits() {
        let text = "A😀e\u{301} العربية";
        assert_eq!(&text[utf16_range(text, 1, 3).unwrap()], "😀");
        assert_eq!(&text[utf16_range(text, 3, 5).unwrap()], "e\u{301}");
        for (start, end) in [(1, 2), (2, 3), (100, 101), (4, 3)] {
            assert!(utf16_range(text, start, end).is_err());
        }
        let revision = ResourceRevision::new(uuid::Uuid::from_u128(9), 1, paragraph(text)).unwrap();
        let anchor = Anchor {
            resource_id: revision.resource_id(),
            revision_id: revision.id(),
            block_id: revision.document().blocks[0].id,
            start_utf16: 1,
            end_utf16: 3,
            exact_quote: "😀".into(),
            quote_hash: None,
        };
        anchor.validate(&revision).unwrap();
        let changed =
            ResourceRevision::new(revision.resource_id(), 2, paragraph("Prefix 😀 suffix"))
                .unwrap();
        let AnchorResolution::Anchored { anchor: moved } = anchor.reattach(&changed).unwrap()
        else {
            panic!("unique quote should attach")
        };
        assert_eq!((moved.start_utf16, moved.end_utf16), (7, 9));
        moved.validate(&changed).unwrap();
        assert!(anchor.validate(&changed).is_err());
    }

    #[test]
    fn anchor_reattachment_flags_missing_and_overlapping_ambiguity() {
        let revision = ResourceRevision::new(uuid::Uuid::from_u128(9), 1, paragraph("aa")).unwrap();
        let anchor = Anchor {
            resource_id: revision.resource_id(),
            revision_id: revision.id(),
            block_id: revision.document().blocks[0].id,
            start_utf16: 0,
            end_utf16: 2,
            exact_quote: "aa".into(),
            quote_hash: None,
        };
        let ambiguous = ResourceRevision::new(revision.resource_id(), 2, paragraph("aaa")).unwrap();
        assert_eq!(
            anchor.reattach(&ambiguous).unwrap(),
            AnchorResolution::Ambiguous
        );
        let missing = ResourceRevision::new(revision.resource_id(), 2, paragraph("bb")).unwrap();
        assert_eq!(
            anchor.reattach(&missing).unwrap(),
            AnchorResolution::Missing
        );
        let foreign = ResourceRevision::new(uuid::Uuid::from_u128(10), 2, paragraph("aa")).unwrap();
        assert!(anchor.reattach(&foreign).is_err());
    }

    #[test]
    fn nested_documents_and_tables_have_bounded_valid_structure() {
        let mut document = paragraph("leaf");
        for depth in 2..=MAX_DEPTH + 1 {
            let child = document.blocks.remove(0);
            document.blocks.push(Block {
                id: uuid::Uuid::from_u128(depth as u128),
                content: BlockContent::List {
                    ordered: false,
                    items: vec![vec![child]],
                },
            });
            assert_eq!(document.validate().is_ok(), depth <= MAX_DEPTH);
        }
        let cell = paragraph("cell").blocks.remove(0);
        document.blocks = vec![Block {
            id: uuid::Uuid::from_u128(100),
            content: BlockContent::Table {
                rows: vec![vec![vec![cell.clone()]], vec![vec![]]],
            },
        }];
        assert!(document.validate().is_err());
        document.blocks[0].content = BlockContent::Table {
            rows: vec![vec![vec![cell.clone()], vec![cell]]],
        };
        assert!(
            document.validate().is_err(),
            "reused cell ID must not duplicate block identity"
        );
    }

    #[test]
    fn invalid_marks_hashes_and_revision_envelopes_fail_closed() {
        let mut document = paragraph("source");
        if let BlockContent::Paragraph { inlines } = &mut document.blocks[0].content {
            inlines[0].marks = vec![Mark::Strong {}, Mark::Strong {}];
        }
        assert!(document.validate().is_err());
        let mut value = serde_json::to_value(paragraph("source")).unwrap();
        value["blocks"][0]["content"]["inlines"][0]["marks"] =
            serde_json::json!([{"type":"strong","onclick":"active"}]);
        assert!(Document::parse(&serde_json::to_vec(&value).unwrap()).is_err());
        let revision =
            ResourceRevision::new(uuid::Uuid::from_u128(9), 1, paragraph("source")).unwrap();
        let anchor = Anchor {
            resource_id: revision.resource_id(),
            revision_id: revision.id(),
            block_id: revision.document().blocks[0].id,
            start_utf16: 0,
            end_utf16: 6,
            exact_quote: "source".into(),
            quote_hash: Some("0".repeat(64)),
        };
        assert!(anchor.validate(&revision).is_err());
        let mut value = serde_json::to_value(&revision).unwrap();
        value["number"] = 0.into();
        assert!(serde_json::from_value::<ResourceRevision>(value).is_err());
    }
}
