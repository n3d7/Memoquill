//! Sequential native packages use a canonical manifest and exact raw payload lengths.
use super::*;
use std::{
    io::ErrorKind,
    time::{Duration, Instant},
};

pub(super) const VERSION: u32 = 2;
const MAGIC: &[u8; 8] = b"MQNATIVE";
const DATABASE_NAME: &str = "database.sqlite3";
pub(super) const LIMITS: NativeLimits = NativeLimits {
    database_bytes: 16 * 1024 * 1024 * 1024,
    payload_bytes: 24 * 1024 * 1024 * 1024,
    entries: 16_384,
    manifest_bytes: 4 * 1024 * 1024,
    rows_per_table: 2_500_000,
    total_rows: 8_000_000,
};
pub(super) const VALIDATION_TIME: Duration = Duration::from_secs(480);

pub(super) fn has_magic(reader: &mut (impl Read + Seek)) -> Result<bool> {
    reader.rewind()?;
    let mut prefix = [0; MAGIC.len()];
    let result = reader.read_exact(&mut prefix);
    reader.rewind()?;
    match result {
        Ok(()) => Ok(&prefix == MAGIC),
        Err(error) if error.kind() == ErrorKind::UnexpectedEof => Ok(false),
        Err(error) => Err(error.into()),
    }
}

fn read_header_field<const N: usize>(reader: &mut impl Read) -> Result<[u8; N]> {
    let mut value = [0; N];
    reader.read_exact(&mut value).map_err(|error| {
        if error.kind() == ErrorKind::UnexpectedEof {
            invalid()
        } else {
            error.into()
        }
    })?;
    Ok(value)
}

fn validate_manifest(manifest: &NativeManifest, limits: NativeLimits) -> Result<u64> {
    archive::validate_manifest(manifest, VERSION, limits)?;
    if manifest.table_counts.len() > 160 {
        return Err(budget());
    }
    let rows = manifest
        .table_counts
        .values()
        .try_fold(0u64, |sum, &rows| {
            if rows > limits.rows_per_table as u64 {
                return Err(budget());
            }
            sum.checked_add(rows).ok_or_else(budget)
        })?;
    if rows > limits.total_rows as u64 {
        return Err(budget());
    }
    let Some((database, originals)) = manifest.payloads.split_first() else {
        return Err(invalid());
    };
    if database.name != DATABASE_NAME
        || originals
            .windows(2)
            .any(|pair| pair[0].name >= pair[1].name)
        || manifest
            .missing_assets
            .windows(2)
            .any(|pair| pair[0].as_str() >= pair[1].as_str())
    {
        return Err(invalid());
    }
    let size = manifest.payloads.iter().try_fold(0u64, |sum, payload| {
        sum.checked_add(payload.byte_length).ok_or_else(budget)
    })?;
    if size > limits.payload_bytes {
        return Err(budget());
    }
    Ok(size)
}

pub(super) fn write(capture: &NativeCapture, mut writer: impl Write) -> Result<()> {
    let mut manifest = capture.manifest.clone();
    manifest.format_version = VERSION;
    manifest.payloads.sort_unstable_by(|a, b| {
        (a.name != DATABASE_NAME, &a.name).cmp(&(b.name != DATABASE_NAME, &b.name))
    });
    manifest
        .missing_assets
        .sort_unstable_by(|a, b| a.as_str().cmp(b.as_str()));
    validate_manifest(&manifest, LIMITS)?;
    let content = json(&manifest)?;
    if content.len() as u64 > LIMITS.manifest_bytes {
        return Err(budget());
    }
    writer.write_all(MAGIC)?;
    writer.write_all(&VERSION.to_le_bytes())?;
    writer.write_all(&(content.len() as u32).to_le_bytes())?;
    writer.write_all(&content)?;
    for payload in &manifest.payloads {
        let mut file = open_regular(&capture.staging.path().join(&payload.name))?;
        let (hash, size) = stream_hash(&mut file, &mut writer, payload.byte_length)?;
        if size != payload.byte_length || hash != payload.sha256 {
            return Err(invalid());
        }
    }
    Ok(())
}

pub(super) fn preflight_staging_space(
    directory: &Path,
    payload_lengths: impl IntoIterator<Item = u64>,
) -> Result<()> {
    check_block_space(rustix::fs::statvfs(directory)?, payload_lengths)
}

pub(super) fn preflight_output_space(file: &File, manifest: &NativeManifest) -> Result<()> {
    let length = manifest
        .payloads
        .iter()
        .try_fold(json(manifest)?.len() as u64 + 16, |total, payload| {
            total.checked_add(payload.byte_length).ok_or_else(budget)
        })?;
    check_block_space(rustix::fs::fstatvfs(file)?, [length])
}

fn check_block_space(
    capacity: rustix::fs::StatVfs,
    payload_lengths: impl IntoIterator<Item = u64>,
) -> Result<()> {
    let block = capacity.f_frsize;
    if block == 0 {
        return Err(budget());
    }
    // Include the staging directories and round each original independently.
    let required = payload_lengths
        .into_iter()
        .try_fold(2u64, |blocks, length| {
            blocks
                .checked_add(length.div_ceil(block))
                .ok_or_else(budget)
        })?;
    if required > capacity.f_bavail {
        return Err(budget());
    }
    Ok(())
}

pub(super) fn read(
    mut reader: impl Read,
    directory: Option<&Path>,
    limits: NativeLimits,
    wire_limit: u64,
) -> Result<NativeCapture> {
    if &read_header_field::<8>(&mut reader)? != MAGIC {
        return Err(invalid());
    }
    if u32::from_le_bytes(read_header_field(&mut reader)?) != VERSION {
        return Err(AppError::new(ErrorCode::UnsupportedVersion));
    }
    let length = u64::from(u32::from_le_bytes(read_header_field(&mut reader)?));
    if length == 0 || length > limits.manifest_bytes {
        return Err(budget());
    }
    let mut bytes = Vec::with_capacity(length as usize);
    (&mut reader).take(length).read_to_end(&mut bytes)?;
    if bytes.len() as u64 != length {
        return Err(invalid());
    }
    let manifest: NativeManifest = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
    let payload_bytes = validate_manifest(&manifest, limits)?;
    if payload_bytes + length + 16 > wire_limit {
        return Err(budget());
    }
    if json(&manifest)? != bytes {
        return Err(invalid());
    }

    let mut builder = tempfile::Builder::new();
    builder
        .permissions(std::fs::Permissions::from_mode(0o700))
        .prefix("memoquill-stream-");
    let staging = match directory {
        Some(parent) => builder.tempdir_in(parent)?,
        None => builder.tempdir()?,
    };
    preflight_staging_space(
        staging.path(),
        manifest.payloads.iter().map(|p| p.byte_length),
    )?;
    owner::create_private_tree(&staging.path().join("assets"))?;
    for payload in &manifest.payloads {
        let mut output = owner::create_private_file(&staging.path().join(&payload.name))?;
        let (hash, size) = stream_hash(
            &mut (&mut reader).take(payload.byte_length),
            &mut output,
            payload.byte_length,
        )?;
        if size != payload.byte_length || hash != payload.sha256 {
            return Err(invalid());
        }
        output.sync_all()?;
    }
    if (&mut reader).take(1).read_to_end(&mut bytes)? != 0 {
        return Err(invalid());
    }
    validation::validate_with_limits(
        &staging.path().join(DATABASE_NAME),
        &manifest,
        limits,
        Some(Instant::now() + VALIDATION_TIME),
    )?;
    Ok(NativeCapture { manifest, staging })
}
