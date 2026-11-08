use super::*;

fn invalid() -> AppError {
    AppError::new(ErrorCode::InvalidRequest)
}

pub(super) fn new_source(source: &NewSource) -> Result<()> {
    if source.id.is_nil() {
        return Err(invalid());
    }
    filename(&source.filename)?;
    media_type(&source.media_type)?;
    optional_text(source.attribution.as_deref(), 4096)?;
    optional_text(source.license.as_deref(), 4096)?;
    origin_url(source.origin_url.as_deref())
}

fn filename(value: &str) -> Result<()> {
    if value.is_empty() || value.chars().count() > 256 {
        return Err(invalid());
    }
    if value
        .chars()
        .any(|c| c.is_control() || matches!(c, '/' | '\\'))
    {
        return Err(invalid());
    }
    Ok(())
}

fn media_type(value: &str) -> Result<()> {
    if value.is_empty() || value.len() > 128 {
        return Err(invalid());
    }
    if !value.bytes().all(|b| b.is_ascii() && !b.is_ascii_control()) {
        return Err(invalid());
    }
    Ok(())
}

fn optional_text(value: Option<&str>, limit: usize) -> Result<()> {
    if let Some(value) = value
        && (value.chars().count() > limit || value.contains('\0'))
    {
        return Err(invalid());
    }
    Ok(())
}

fn origin_url(value: Option<&str>) -> Result<()> {
    optional_text(value, 2048)?;
    if let Some(value) = value {
        if value.chars().any(char::is_control) {
            return Err(invalid());
        }
        let parsed = tauri::Url::parse(value).map_err(|_| invalid())?;
        if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
            return Err(invalid());
        }
    }
    Ok(())
}

pub(super) fn original(source: &OriginalSource) -> Result<()> {
    new_source(&NewSource {
        id: source.id,
        resource_id: source.resource_id,
        digest: source.digest.clone(),
        filename: source.filename.clone(),
        media_type: source.media_type.clone(),
        attribution: source.attribution.clone(),
        license: source.license.clone(),
        origin_url: source.origin_url.clone(),
    })?;
    if source.revision == 0
        || source.revision > super::super::types::MAX_REVISION as u64
        || source.byte_length > MAX_SOURCE_BYTES
    {
        return Err(invalid());
    }
    Ok(())
}
