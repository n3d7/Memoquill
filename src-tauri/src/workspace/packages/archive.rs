use super::*;
use std::{
    collections::HashSet,
    io::{Cursor, SeekFrom},
};
use zip::{CompressionMethod, ZipArchive, ZipWriter, write::SimpleFileOptions};

fn zip_error(_: zip::result::ZipError) -> AppError {
    invalid()
}

pub(super) fn write<W: Write + Seek>(capture: &NativeCapture, writer: W) -> Result<()> {
    let mut archive = ZipWriter::new(writer);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o600);
    archive
        .start_file("manifest.json", options)
        .map_err(zip_error)?;
    let manifest = json(&capture.manifest)?;
    if manifest.len() as u64 > MAX_MANIFEST {
        return Err(budget());
    }
    archive.write_all(&manifest)?;
    for payload in &capture.manifest.payloads {
        archive
            .start_file(&payload.name, options)
            .map_err(zip_error)?;
        let (hash, size) = stream_hash(
            &mut open_regular(&capture.staging.path().join(&payload.name))?,
            &mut archive,
            payload.byte_length,
        )?;
        if hash != payload.sha256 || size != payload.byte_length {
            return Err(invalid());
        }
    }
    let mut writer = archive.finish().map_err(zip_error)?;
    if writer.stream_position()? > MAX_ARCHIVE {
        return Err(budget());
    }
    Ok(())
}

pub(super) fn read<R: Read + Seek>(mut reader: R) -> Result<NativeCapture> {
    let directory = preflight(&mut reader)?;
    reader.seek(SeekFrom::Start(0))?;
    let mut archive = ZipArchive::new(reader).map_err(zip_error)?;
    if archive.len() != directory.count
        || archive.has_duplicate_names()
        || archive.has_overlapping_entries().map_err(zip_error)?
    {
        return Err(invalid());
    }
    let mut names = HashSet::new();
    let mut total = 0u64;
    for index in 0..archive.len() {
        let entry = archive.by_index(index).map_err(zip_error)?;
        let name = entry.name().map_err(zip_error)?.into_owned();
        let mode = entry.unix_mode().unwrap_or(0) & 0o170000;
        let start = entry.data_start().ok_or_else(invalid)?;
        let end = start
            .checked_add(entry.compressed_size())
            .ok_or_else(budget)?;
        total = total.checked_add(entry.size()).ok_or_else(budget)?;
        if entry.encrypted()
            || entry.is_symlink()
            || !matches!(mode, 0 | 0o100000)
            || entry.is_dir()
            || !valid_name(&name)
            || !names.insert(name.clone())
            || end > directory.offset
            || entry.size() > entry_limit(&name)
            || entry.compressed_size() > MAX_ARCHIVE
            || total > MAX_EXPANDED
            || !matches!(
                entry.compression(),
                CompressionMethod::Stored | CompressionMethod::Deflated
            )
        {
            return Err(invalid());
        }
    }
    let manifest_bytes = {
        let mut entry = archive.by_name("manifest.json").map_err(zip_error)?;
        let mut output = Vec::new();
        stream_hash(&mut entry, &mut output, MAX_MANIFEST)?;
        output
    };
    let manifest: NativeManifest =
        serde_json::from_slice(&manifest_bytes).map_err(|_| invalid())?;
    validate_manifest(&manifest)?;
    let expected: HashSet<_> = manifest
        .payloads
        .iter()
        .map(|p| p.name.clone())
        .chain(std::iter::once("manifest.json".into()))
        .collect();
    if names != expected || expected.len() != manifest.payloads.len() + 1 {
        return Err(invalid());
    }
    let staging = tempfile::Builder::new()
        .permissions(std::fs::Permissions::from_mode(0o700))
        .prefix("memoquill-verified-")
        .tempdir()?;
    owner::create_private_tree(&staging.path().join("assets"))?;
    for payload in &manifest.payloads {
        let mut entry = archive.by_name(&payload.name).map_err(zip_error)?;
        if entry.size() != payload.byte_length {
            return Err(invalid());
        }
        let mut output = owner::create_private_file(&staging.path().join(&payload.name))?;
        let (hash, size) = stream_hash(&mut entry, &mut output, payload.byte_length)?;
        if hash != payload.sha256 || size != payload.byte_length {
            return Err(invalid());
        }
        output.sync_all()?;
    }
    validation::validate(&staging.path().join("database.sqlite3"), &manifest)?;
    Ok(NativeCapture { manifest, staging })
}

fn valid_name(name: &str) -> bool {
    matches!(name, "manifest.json" | "database.sqlite3")
        || name
            .strip_prefix("assets/")
            .is_some_and(|d| AssetDigest::try_from(d.to_owned()).is_ok())
}
fn entry_limit(name: &str) -> u64 {
    match name {
        "manifest.json" => MAX_MANIFEST,
        "database.sqlite3" => MAX_DATABASE,
        _ => MAX_ASSET,
    }
}
fn validate_manifest(manifest: &NativeManifest) -> Result<()> {
    if manifest.format_version != 1
        || manifest.kind != "nativeFull"
        || manifest.scope != "selectedCourse"
        || manifest.database_section_version != 1
        || manifest.assets_section_version != 1
    {
        return Err(AppError::new(ErrorCode::UnsupportedVersion));
    }
    if manifest.schema_version != SCHEMA_VERSION {
        return Err(AppError::new(ErrorCode::UnsupportedSchema));
    }
    if manifest.source_course_id.get_version_num() != 4
        || manifest.complete != manifest.missing_assets.is_empty()
        || manifest.payloads.is_empty()
        || manifest.payloads.len() >= MAX_ENTRIES
        || manifest.missing_assets.len() > MAX_ENTRIES - 2
        || manifest.omissions
            != [
                "Other courses, catalogue, credentials and runtime files are outside selected-course scope.",
            ]
    {
        return Err(invalid());
    }
    let mut names = HashSet::new();
    for payload in &manifest.payloads {
        if !valid_name(&payload.name)
            || payload.name == "manifest.json"
            || !names.insert(&payload.name)
            || payload.byte_length > entry_limit(&payload.name)
            || AssetDigest::try_from(payload.sha256.clone()).is_err()
            || (payload.name.starts_with("assets/")
                && payload.name != format!("assets/{}", payload.sha256))
        {
            return Err(invalid());
        }
    }
    if !names.contains(&"database.sqlite3".to_string()) {
        return Err(invalid());
    }
    Ok(())
}

struct Directory {
    count: usize,
    offset: u64,
}
fn u16_at(bytes: &[u8], index: usize) -> u16 {
    u16::from_le_bytes([bytes[index], bytes[index + 1]])
}
fn u32_at(bytes: &[u8], index: usize) -> u32 {
    u32::from_le_bytes(bytes[index..index + 4].try_into().expect("bounded header"))
}

/// Bound metadata BEFORE ZipArchive allocates its central directory structures.
fn preflight<R: Read + Seek>(reader: &mut R) -> Result<Directory> {
    let length = reader.seek(SeekFrom::End(0))?;
    if !(22..=MAX_ARCHIVE).contains(&length) {
        return Err(budget());
    }
    let tail_len = length.min(65557) as usize;
    reader.seek(SeekFrom::End(-(tail_len as i64)))?;
    let mut tail = vec![0; tail_len];
    reader.read_exact(&mut tail)?;
    let eocd = (0..=tail_len - 22)
        .rev()
        .find(|&i| {
            tail[i..i + 4] == *b"PK\x05\x06" && i + 22 + u16_at(&tail, i + 20) as usize == tail_len
        })
        .ok_or_else(invalid)?;
    let header = &tail[eocd..];
    let count = u16_at(header, 10) as usize;
    let size = u32_at(header, 12) as u64;
    let offset = u32_at(header, 16) as u64;
    let end = length - tail_len as u64 + eocd as u64;
    if u16_at(header, 4) != 0
        || u16_at(header, 6) != 0
        || u16_at(header, 8) as usize != count
        || count == 0
        || count > MAX_ENTRIES
        || count == 65535
        || size > MAX_MANIFEST
        || size < count as u64 * 46
        || offset.checked_add(size) != Some(end)
    {
        return Err(invalid());
    }
    reader.seek(SeekFrom::Start(0))?;
    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic)?;
    if magic != *b"PK\x03\x04" {
        return Err(invalid());
    }
    reader.seek(SeekFrom::Start(offset))?;
    let mut consumed = 0u64;
    let mut names = HashSet::new();
    let mut first = u64::MAX;
    for _ in 0..count {
        if consumed + 46 > size {
            return Err(invalid());
        }
        let mut header = [0u8; 46];
        reader.read_exact(&mut header)?;
        let name_len = u16_at(&header, 28) as usize;
        let extra_len = u16_at(&header, 30) as usize;
        let comment_len = u16_at(&header, 32) as usize;
        let flags = u16_at(&header, 8);
        let method = u16_at(&header, 10);
        let local_offset = u32_at(&header, 42) as u64;
        consumed += 46 + name_len as u64 + extra_len as u64 + comment_len as u64;
        if header[..4] != *b"PK\x01\x02"
            || name_len > 128
            || extra_len > 1024
            || comment_len > 1024
            || consumed > size
            || u16_at(&header, 34) != 0
            || flags & !0x0808 != 0
            || !matches!(method, 0 | 8)
            || u32_at(&header, 20) == u32::MAX
            || u32_at(&header, 24) == u32::MAX
            || local_offset >= offset
        {
            return Err(invalid());
        }
        let mut name = vec![0; name_len];
        reader.read_exact(&mut name)?;
        let decoded = std::str::from_utf8(&name).map_err(|_| invalid())?;
        if !valid_name(decoded) || !names.insert(name.clone()) {
            return Err(invalid());
        }
        let mut extra = vec![0; extra_len];
        reader.read_exact(&mut extra)?;
        reject_zip64(&extra)?;
        reader.seek(SeekFrom::Current(comment_len as i64))?;
        let resume = reader.stream_position()?;
        reader.seek(SeekFrom::Start(local_offset))?;
        let mut local = [0u8; 30];
        reader.read_exact(&mut local)?;
        let ln = u16_at(&local, 26) as usize;
        let le = u16_at(&local, 28) as usize;
        if local[..4] != *b"PK\x03\x04"
            || u16_at(&local, 6) != flags
            || u16_at(&local, 8) != method
            || ln != name_len
            || le > 1024
            || local_offset + 30 + ln as u64 + le as u64 > offset
        {
            return Err(invalid());
        }
        let mut local_name = vec![0; ln];
        reader.read_exact(&mut local_name)?;
        if local_name != name {
            return Err(invalid());
        }
        let mut local_extra = vec![0; le];
        reader.read_exact(&mut local_extra)?;
        reject_zip64(&local_extra)?;
        first = first.min(local_offset);
        reader.seek(SeekFrom::Start(resume))?;
    }
    if consumed != size || first != 0 {
        return Err(invalid());
    }
    Ok(Directory { count, offset })
}
fn reject_zip64(extra: &[u8]) -> Result<()> {
    let mut cursor = Cursor::new(extra);
    while cursor.position() < extra.len() as u64 {
        let mut header = [0u8; 4];
        cursor.read_exact(&mut header).map_err(|_| invalid())?;
        let size = u16_at(&header, 2) as u64;
        if u16_at(&header, 0) == 1 || cursor.position() + size > extra.len() as u64 {
            return Err(invalid());
        }
        cursor.seek(SeekFrom::Current(size as i64))?;
    }
    Ok(())
}
