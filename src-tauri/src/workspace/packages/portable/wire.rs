use super::*;
use std::{collections::HashSet, io::SeekFrom};
use zip::{CompressionMethod, ZipArchive, ZipWriter, write::SimpleFileOptions};
fn z(_: zip::result::ZipError) -> AppError {
    invalid()
}
fn valid_name(n: &str) -> bool {
    matches!(
        n,
        "manifest.json" | "content.ndjson" | "readable/content.txt"
    ) || n
        .strip_prefix("assets/")
        .is_some_and(|d| AssetDigest::try_from(d.to_owned()).is_ok())
}
fn limit(n: &str) -> u64 {
    if n == "manifest.json" {
        MAX_MANIFEST
    } else {
        MAX_CONTENT
    }
}
pub(super) fn validate_manifest(m: &PortableManifest) -> Result<()> {
    if m.format_version != 1
        || m.kind != "portableContent"
        || m.scope != "selectedCourse"
        || m.sections != sections()
        || !m.required_adapters.is_empty()
    {
        return Err(AppError::new(ErrorCode::UnsupportedVersion));
    }
    if m.source_course_id.get_version_num() != 4
        || m.omissions != OMISSIONS
        || m.payloads.len() < 2
        || m.payloads.len() >= MAX_ENTRIES
        || (m.originals_requested && !m.intentionally_omitted_assets.is_empty())
    {
        return Err(invalid());
    }
    let mut names = HashSet::new();
    let mut total = 0u64;
    for p in &m.payloads {
        total = total.checked_add(p.byte_length).ok_or_else(budget)?;
        if !valid_name(&p.name)
            || p.name == "manifest.json"
            || !names.insert(p.name.as_str())
            || p.byte_length > limit(&p.name)
            || AssetDigest::try_from(p.sha256.clone()).is_err()
            || (p.name.starts_with("assets/")
                && (p.name != format!("assets/{}", p.sha256) || !m.originals_requested))
        {
            return Err(invalid());
        }
    }
    if total > MAX_EXPANDED
        || !names.contains("content.ndjson")
        || !names.contains("readable/content.txt")
    {
        return Err(invalid());
    }
    Ok(())
}
pub(super) fn write<W: Write + Seek>(p: &PortableCapture, writer: W) -> Result<()> {
    let mut zip = ZipWriter::new(writer);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o600);
    zip.start_file("manifest.json", options).map_err(z)?;
    let bytes = json(&p.manifest)?;
    if bytes.len() as u64 > MAX_MANIFEST {
        return Err(budget());
    }
    zip.write_all(&bytes)?;
    for payload in &p.manifest.payloads {
        zip.start_file(&payload.name, options).map_err(z)?;
        let (h, n) = stream_hash(
            &mut open_regular(&p.native.staging.path().join(&payload.name))?,
            &mut zip,
            payload.byte_length,
        )?;
        if h != payload.sha256 || n != payload.byte_length {
            return Err(invalid());
        }
    }
    if zip.finish().map_err(z)?.stream_position()? > MAX_ARCHIVE {
        return Err(budget());
    }
    Ok(())
}
pub(super) fn read<R: Read + Seek>(mut reader: R) -> Result<PortableCapture> {
    let directory = archive::preflight(&mut reader, valid_name)?;
    reader.seek(SeekFrom::Start(0))?;
    let mut zip = ZipArchive::new(reader).map_err(z)?;
    if zip.len() != directory.count
        || zip.has_duplicate_names()
        || zip.has_overlapping_entries().map_err(z)?
    {
        return Err(invalid());
    }
    let mut names = HashSet::new();
    let mut total = 0u64;
    for i in 0..zip.len() {
        let entry = zip.by_index(i).map_err(z)?;
        let name = entry.name().map_err(z)?.into_owned();
        let mode = entry.unix_mode().unwrap_or(0) & 0o170000;
        let end = entry
            .data_start()
            .ok_or_else(invalid)?
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
            || entry.size() > limit(&name)
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
    let mut bytes = Vec::new();
    stream_hash(
        &mut zip.by_name("manifest.json").map_err(z)?,
        &mut bytes,
        MAX_MANIFEST,
    )?;
    let manifest: PortableManifest = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
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
    let staging = stage()?;
    owner::create_private_tree(&staging.path().join("assets"))?;
    owner::create_private_tree(&staging.path().join("readable"))?;
    for p in &manifest.payloads {
        let mut entry = zip.by_name(&p.name).map_err(z)?;
        if entry.size() != p.byte_length {
            return Err(invalid());
        }
        let mut output = owner::create_private_file(&staging.path().join(&p.name))?;
        let (h, n) = stream_hash(&mut entry, &mut output, p.byte_length)?;
        if h != p.sha256 || n != p.byte_length {
            return Err(invalid());
        }
        output.sync_all()?;
    }
    let content = Content::decode(&std::fs::read(staging.path().join("content.ndjson"))?)?;
    PortableCapture::assemble(manifest, content, staging)
}
