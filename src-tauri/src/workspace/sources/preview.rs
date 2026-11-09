use super::*;
use std::{fs::File, io::Read, path::Path};

const PREVIEW_BYTES: u64 = 64 * 1024;
const HEX_BYTES: usize = 256;

pub(super) fn read(root: &Path, source: OriginalSource) -> Result<SourcePreview> {
    let info = AssetInfo {
        digest: source.digest.clone(),
        byte_length: source.byte_length,
    };
    assets::verify(root, &info).map_err(|_| corrupt())?;
    let bytes = bounded_bytes(root, &info)?;
    let truncated = source.byte_length > bytes.len() as u64;
    let extraction = Some(extraction::extract(&source.filename, &bytes, truncated));
    match text(&bytes, truncated) {
        Some(value) => Ok(SourcePreview {
            source,
            text: Some(value),
            hex: None,
            truncated,
            extraction,
        }),
        None => {
            let shown = &bytes[..bytes.len().min(HEX_BYTES)];
            let hex = shown.iter().map(|b| format!("{b:02x}")).collect();
            let truncated = source.byte_length > shown.len() as u64;
            Ok(SourcePreview {
                source,
                text: None,
                hex: Some(hex),
                truncated,
                extraction,
            })
        }
    }
}

pub(super) fn bounded_bytes(root: &Path, info: &AssetInfo) -> Result<Vec<u8>> {
    read_bytes(root, info, PREVIEW_BYTES)
}

pub(super) fn read_bytes(root: &Path, info: &AssetInfo, limit: u64) -> Result<Vec<u8>> {
    let flags = rustix::fs::OFlags::RDONLY
        | rustix::fs::OFlags::NOFOLLOW
        | rustix::fs::OFlags::CLOEXEC
        | rustix::fs::OFlags::NONBLOCK;
    let fd = rustix::fs::open(
        assets::asset_path(root, &info.digest),
        flags,
        rustix::fs::Mode::empty(),
    )
    .map_err(|_| corrupt())?;
    let file = File::from(fd);
    let metadata = file.metadata().map_err(|_| corrupt())?;
    if !metadata.is_file() || metadata.len() != info.byte_length {
        return Err(corrupt());
    }
    let mut bytes = Vec::new();
    file.take(limit)
        .read_to_end(&mut bytes)
        .map_err(|_| corrupt())?;
    if bytes.len() as u64 != info.byte_length.min(limit) {
        return Err(corrupt());
    }
    Ok(bytes)
}

fn text(bytes: &[u8], truncated: bool) -> Option<String> {
    match std::str::from_utf8(bytes) {
        Ok(value) => Some(value.into()),
        Err(error) if truncated && error.error_len().is_none() => {
            std::str::from_utf8(&bytes[..error.valid_up_to()])
                .ok()
                .map(str::to_owned)
        }
        Err(_) => None,
    }
}
