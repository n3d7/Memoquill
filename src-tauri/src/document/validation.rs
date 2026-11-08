use super::*;

pub(super) fn validate(document: &Document) -> Result<(), DocumentError> {
    if document.schema_version != SCHEMA_VERSION {
        return Err(DocumentError::UnsupportedVersion);
    }
    if document.blocks.is_empty() {
        return Err(DocumentError::Invalid);
    }
    validate_blocks(&document.blocks)?;
    let bytes = serde_json::to_vec(document).map_err(|_| DocumentError::Invalid)?;
    if bytes.len() > MAX_DOCUMENT_BYTES {
        return Err(DocumentError::TooLarge);
    }
    Ok(())
}

fn validate_blocks(blocks: &[Block]) -> Result<(), DocumentError> {
    let mut ids = HashSet::new();
    let mut count = 0;
    let mut pending: Vec<_> = blocks.iter().map(|b| (b, 1)).collect();
    while let Some((block, depth)) = pending.pop() {
        if depth > MAX_DEPTH || block.id.is_nil() || !ids.insert(block.id) {
            return Err(DocumentError::Invalid);
        }
        count += 1 + inline_count(block);
        if count > MAX_NODES {
            return Err(DocumentError::TooLarge);
        }
        validate_content(&block.content)?;
        pending.extend(block.children().into_iter().map(|child| (child, depth + 1)));
    }
    Ok(())
}

fn inline_count(block: &Block) -> usize {
    match &block.content {
        BlockContent::Paragraph { inlines } | BlockContent::Heading { inlines, .. } => {
            inlines.len()
        }
        _ => 0,
    }
}

fn validate_content(content: &BlockContent) -> Result<(), DocumentError> {
    match content {
        BlockContent::Paragraph { inlines } => validate_inlines(inlines),
        BlockContent::Heading { level, inlines } => validate_heading(*level, inlines),
        BlockContent::List { items, .. } => validate_list(items),
        BlockContent::Code { text, language } => validate_code(text, language.as_deref()),
        BlockContent::Math { source } => validate_math(source),
        BlockContent::Table { rows } => validate_table(rows),
        BlockContent::Media {
            asset_digest,
            mime,
            caption,
            alt,
        } => validate_media(asset_digest, mime, caption, alt),
        BlockContent::Unsupported {
            original,
            source,
            reason,
        } => validate_unsupported(original, source.as_ref(), reason),
    }
}
fn validate_heading(level: u8, inlines: &[Inline]) -> Result<(), DocumentError> {
    require((1..=6).contains(&level))?;
    validate_inlines(inlines)
}
fn validate_list(items: &[Vec<Block>]) -> Result<(), DocumentError> {
    require(!items.is_empty() && items.iter().all(|item| !item.is_empty()))
}
fn validate_code(text: &str, language: Option<&str>) -> Result<(), DocumentError> {
    text_value(text)?;
    language.map(label).transpose()?;
    Ok(())
}
fn validate_math(source: &str) -> Result<(), DocumentError> {
    text_value(source)?;
    require(source.chars().count() <= 2048)
}
fn validate_unsupported(
    original: &str,
    source: Option<&SourceReference>,
    reason: &str,
) -> Result<(), DocumentError> {
    text_value(original)?;
    label(reason)?;
    require(!reason.is_empty())?;
    if let Some(source) = source {
        require(digest(&source.asset_digest))?;
        text_value(&source.locator)?;
    }
    Ok(())
}
fn validate_inlines(inlines: &[Inline]) -> Result<(), DocumentError> {
    let mut bytes = 0;
    for inline in inlines {
        text_value(&inline.text)?;
        bytes += inline.text.len();
        require(bytes <= MAX_TEXT_BYTES && inline.marks.len() <= 4)?;
        let mut kinds = HashSet::new();
        for mark in &inline.marks {
            require(kinds.insert(std::mem::discriminant(mark)))?;
            if let Mark::Link { href } = mark {
                require(safe_url(href))?;
            }
        }
    }
    Ok(())
}

fn validate_table(rows: &[Vec<Vec<Block>>]) -> Result<(), DocumentError> {
    require(!rows.is_empty() && rows.len() <= 100)?;
    let width = rows[0].len();
    require((1..=32).contains(&width))?;
    require(
        rows.iter()
            .all(|row| row.len() == width && row.iter().all(|cell| !cell.is_empty())),
    )
}

fn validate_media(hash: &str, mime: &str, caption: &str, alt: &str) -> Result<(), DocumentError> {
    require(digest(hash))?;
    require(matches!(
        mime,
        "image/png"
            | "image/jpeg"
            | "image/webp"
            | "audio/ogg"
            | "audio/mpeg"
            | "video/webm"
            | "application/pdf"
            | "text/plain"
    ))?;
    text_value(caption)?;
    text_value(alt)
}

pub(super) fn text_value(text: &str) -> Result<(), DocumentError> {
    require(text.len() <= MAX_TEXT_BYTES && !text.contains('\0'))
}

fn label(text: &str) -> Result<(), DocumentError> {
    require(text.chars().count() <= 256 && !text.contains('\0'))
}

pub(super) fn digest(text: &str) -> bool {
    text.len() == 64
        && text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

pub(super) fn safe_url(url: &str) -> bool {
    if url.len() > 2048
        || url.chars().any(|c| c.is_whitespace() || c.is_control())
        || url.contains('\\')
    {
        return false;
    }
    let lower = url.to_ascii_lowercase();
    let rest = if lower.starts_with("https://") {
        &url[8..]
    } else if lower.starts_with("http://") {
        &url[7..]
    } else {
        return false;
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    !authority.is_empty() && !authority.contains('@')
}

fn require(ok: bool) -> Result<(), DocumentError> {
    if ok {
        Ok(())
    } else {
        Err(DocumentError::Invalid)
    }
}
