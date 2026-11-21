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
    let file = open_file(root, info)?;
    let mut bytes = Vec::new();
    file.take(limit)
        .read_to_end(&mut bytes)
        .map_err(|_| corrupt())?;
    if bytes.len() as u64 != info.byte_length.min(limit) {
        return Err(corrupt());
    }
    Ok(bytes)
}

fn open_file(root: &Path, info: &AssetInfo) -> Result<File> {
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
    Ok(file)
}

/// Verify and capture the requested window from the same descriptor.
#[cfg(target_os = "linux")]
pub(super) fn verified_range(
    root: &Path,
    info: &AssetInfo,
    offset: u64,
    limit: u64,
) -> Result<Vec<u8>> {
    use sha2::{Digest, Sha256};
    if offset > info.byte_length {
        return Err(invalid());
    }
    let end = offset + limit.min(info.byte_length - offset);
    let mut file = open_file(root, info)?;
    let mut digest = Sha256::new();
    let mut bytes = Vec::with_capacity((end - offset) as usize);
    let mut cursor = 0;
    let mut buffer = [0; 16 * 1024];
    while cursor < info.byte_length {
        let count = (info.byte_length - cursor).min(buffer.len() as u64) as usize;
        let block = &mut buffer[..count];
        file.read_exact(block).map_err(|_| corrupt())?;
        digest.update(&*block);
        let next = cursor + count as u64;
        if offset < next && end > cursor {
            let from = offset.saturating_sub(cursor) as usize;
            let to = end.min(next).saturating_sub(cursor) as usize;
            bytes.extend_from_slice(&block[from..to]);
        }
        cursor = next;
    }
    if format!("{:x}", digest.finalize()) != info.digest.as_str() {
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

#[cfg(all(test, target_os = "linux"))]
mod range_tests {
    use super::*;
    use sha2::{Digest, Sha256};
    use std::fs;

    #[test]
    fn mcp_original_chunk_validates_the_complete_source() {
        let directory = tempfile::tempdir().unwrap();
        let data: Vec<_> = (0..40_000).map(|n| (n % 251) as u8).collect();
        let info = AssetInfo {
            digest: AssetDigest::try_from(format!("{:x}", Sha256::digest(&data))).unwrap(),
            byte_length: data.len() as u64,
        };
        let path = assets::asset_path(directory.path(), &info.digest);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, &data).unwrap();
        assert_eq!(
            verified_range(directory.path(), &info, 16_370, 32).unwrap(),
            data[16_370..16_402]
        );
        assert_eq!(
            verified_range(directory.path(), &info, 39_990, 32).unwrap(),
            data[39_990..]
        );
        assert!(verified_range(directory.path(), &info, 40_001, 32).is_err());
        let mut corrupted = data;
        corrupted[0] ^= 0x80;
        fs::write(path, corrupted).unwrap();
        assert!(verified_range(directory.path(), &info, 16_370, 32).is_err());
    }
}
