use super::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResourceRevision {
    resource_id: Uuid,
    id: Uuid,
    number: u64,
    document: Document,
}

impl ResourceRevision {
    pub fn new(resource_id: Uuid, number: u64, document: Document) -> Result<Self, DocumentError> {
        Self::from_record(resource_id, Uuid::new_v4(), number, document)
    }
    pub fn from_record(
        resource_id: Uuid,
        id: Uuid,
        number: u64,
        document: Document,
    ) -> Result<Self, DocumentError> {
        let revision = Self {
            resource_id,
            id,
            number,
            document,
        };
        revision.validate()?;
        Ok(revision)
    }
    pub fn resource_id(&self) -> Uuid {
        self.resource_id
    }
    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn number(&self) -> u64 {
        self.number
    }
    pub fn document(&self) -> &Document {
        &self.document
    }

    fn validate(&self) -> Result<(), DocumentError> {
        if self.resource_id.is_nil() || self.id.is_nil() || self.number == 0 {
            return Err(DocumentError::Invalid);
        }
        self.document.validate()
    }
}

impl<'de> Deserialize<'de> for ResourceRevision {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct StoredRevision {
            resource_id: Uuid,
            id: Uuid,
            number: u64,
            document: Document,
        }
        let stored = StoredRevision::deserialize(deserializer)?;
        Self::from_record(
            stored.resource_id,
            stored.id,
            stored.number,
            stored.document,
        )
        .map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Anchor {
    pub resource_id: Uuid,
    pub revision_id: Uuid,
    pub block_id: Uuid,
    pub start_utf16: usize,
    pub end_utf16: usize,
    pub exact_quote: String,
    pub quote_hash: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum AnchorResolution {
    Anchored { anchor: Anchor },
    Ambiguous,
    Missing,
}

impl Anchor {
    pub fn validate(&self, revision: &ResourceRevision) -> Result<(), DocumentError> {
        if self.resource_id != revision.resource_id || self.revision_id != revision.id {
            return Err(DocumentError::InvalidAnchor);
        }
        let block = revision
            .document
            .all_blocks()
            .into_iter()
            .find(|b| b.id == self.block_id)
            .ok_or(DocumentError::InvalidAnchor)?;
        let text = block.text();
        let range = utf16_range(&text, self.start_utf16, self.end_utf16)?;
        if text[range] != self.exact_quote {
            return Err(DocumentError::InvalidAnchor);
        }
        self.validate_quote()
    }

    fn validate_quote(&self) -> Result<(), DocumentError> {
        validation::text_value(&self.exact_quote)?;
        if let Some(hash) = &self.quote_hash {
            use sha2::{Digest, Sha256};
            if *hash != format!("{:x}", Sha256::digest(self.exact_quote.as_bytes())) {
                return Err(DocumentError::InvalidAnchor);
            }
        }
        Ok(())
    }

    pub fn reattach(&self, revision: &ResourceRevision) -> Result<AnchorResolution, DocumentError> {
        self.validate_quote()?;
        if self.resource_id != revision.resource_id {
            return Err(DocumentError::InvalidAnchor);
        }
        let Some(block) = revision
            .document
            .all_blocks()
            .into_iter()
            .find(|b| b.id == self.block_id)
        else {
            return Ok(AnchorResolution::Missing);
        };
        let text = block.text();
        if self.exact_quote.is_empty() {
            return Ok(AnchorResolution::Missing);
        }
        let mut matches = text
            .char_indices()
            .map(|(byte, _)| byte)
            .filter(|byte| text[*byte..].starts_with(&self.exact_quote));
        let Some(start) = matches.next() else {
            return Ok(AnchorResolution::Missing);
        };
        if matches.next().is_some() {
            return Ok(AnchorResolution::Ambiguous);
        }
        let mut anchor = self.clone();
        anchor.revision_id = revision.id;
        anchor.start_utf16 = text[..start].encode_utf16().count();
        anchor.end_utf16 = anchor.start_utf16 + self.exact_quote.encode_utf16().count();
        Ok(AnchorResolution::Anchored { anchor })
    }
}

/// Convert browser coordinates without slicing inside a UTF-8 sequence or UTF-16 surrogate pair.
pub fn utf16_range(
    text: &str,
    start: usize,
    end: usize,
) -> Result<std::ops::Range<usize>, DocumentError> {
    if start > end {
        return Err(DocumentError::InvalidAnchor);
    }
    let mut offset = 0;
    let mut begin = (start == 0).then_some(0);
    let mut finish = (end == 0).then_some(0);
    for (byte, character) in text.char_indices() {
        offset += character.len_utf16();
        let next = byte + character.len_utf8();
        if offset == start {
            begin = Some(next);
        }
        if offset == end {
            finish = Some(next);
        }
    }
    Ok(begin.ok_or(DocumentError::InvalidAnchor)?..finish.ok_or(DocumentError::InvalidAnchor)?)
}
