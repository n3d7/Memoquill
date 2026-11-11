//! Selected-course native recovery packages. No renderer filesystem authority.
mod archive;
mod validation;

use super::{
    assets,
    database::{self, Database, SCHEMA_VERSION},
    error::Result,
    owner, *,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Seek, Write},
    os::unix::fs::PermissionsExt,
    path::Path,
};
use uuid::Uuid;

const MAX_ARCHIVE: u64 = 512 * 1024 * 1024;
const MAX_EXPANDED: u64 = 1024 * 1024 * 1024;
const MAX_DATABASE: u64 = 256 * 1024 * 1024;
const MAX_ASSET: u64 = 64 * 1024 * 1024;
const MAX_ENTRIES: usize = 4098;
const MAX_MANIFEST: u64 = 2 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativePayload {
    pub name: String,
    pub sha256: String,
    pub byte_length: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeManifest {
    pub format_version: u32,
    pub kind: String,
    pub scope: String,
    pub schema_version: i64,
    pub database_section_version: u32,
    pub assets_section_version: u32,
    pub source_course_id: Uuid,
    pub workspace_id: WorkspaceId,
    pub complete: bool,
    pub missing_assets: Vec<AssetDigest>,
    pub omissions: Vec<String>,
    pub payloads: Vec<NativePayload>,
    pub table_counts: std::collections::BTreeMap<String, u64>,
}

/// Immutable, privately staged bytes; neither paths nor connections cross this boundary.
pub struct NativeCapture {
    manifest: NativeManifest,
    staging: tempfile::TempDir,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeRestoreReport {
    pub source_course_id: Uuid,
    pub course_id: Uuid,
    pub workspace_id: WorkspaceId,
    pub complete: bool,
    pub missing_assets: Vec<AssetDigest>,
    pub omissions: Vec<String>,
}

impl NativeCapture {
    pub fn manifest(&self) -> &NativeManifest {
        &self.manifest
    }
    pub fn write_to<W: Write + Seek>(&self, writer: W) -> Result<()> {
        archive::write(self, writer)
    }
    pub fn read_from<R: Read + Seek>(reader: R) -> Result<Self> {
        archive::read(reader)
    }
    pub(super) fn identity(&self) -> Result<String> {
        Ok(hash_bytes(&json(&self.manifest)?))
    }
    pub(super) fn report(&self, course_id: Uuid) -> NativeRestoreReport {
        NativeRestoreReport {
            source_course_id: self.manifest.source_course_id,
            course_id,
            workspace_id: self.manifest.workspace_id.clone(),
            complete: self.manifest.complete,
            missing_assets: self.manifest.missing_assets.clone(),
            omissions: self.manifest.omissions.clone(),
        }
    }
    pub(super) fn materialize(&self, root: &Path) -> Result<()> {
        owner::private_directory(root)?;
        for payload in &self.manifest.payloads {
            let destination = if payload.name == "database.sqlite3" {
                root.join("memoquill.sqlite3")
            } else {
                let digest = AssetDigest::try_from(payload.sha256.clone())?;
                let destination = assets::asset_path(root, &digest);
                owner::create_private_tree(destination.parent().ok_or_else(invalid)?)?;
                destination
            };
            copy_verified(
                &self.staging.path().join(&payload.name),
                &destination,
                payload,
            )?;
            owner::sync_directory(destination.parent().ok_or_else(invalid)?)?;
        }
        if root.join("assets/sha256").exists() {
            owner::sync_directory(&root.join("assets/sha256"))?;
        }
        if root.join("assets").exists() {
            owner::sync_directory(&root.join("assets"))?;
        }
        owner::sync_directory(root)?;
        Ok(())
    }
    pub(super) fn matches_root(&self, root: &Path) -> Result<()> {
        owner::private_directory(root)?;
        for payload in &self.manifest.payloads {
            let path = if payload.name == "database.sqlite3" {
                root.join("memoquill.sqlite3")
            } else {
                assets::asset_path(root, &AssetDigest::try_from(payload.sha256.clone())?)
            };
            let (hash, size) = hash_file(&path, payload.byte_length)?;
            if hash != payload.sha256 || size != payload.byte_length {
                return Err(invalid());
            }
        }
        Ok(())
    }
}

impl Database {
    pub(super) fn capture_native(
        &mut self,
        context: RequestContext,
        source_course_id: Uuid,
    ) -> Result<NativeCapture> {
        context.authorize()?;
        if source_course_id.get_version_num() != 4 {
            return Err(invalid());
        }
        let staging = tempfile::Builder::new()
            .permissions(std::fs::Permissions::from_mode(0o700))
            .prefix("memoquill-native-")
            .tempdir()?;
        let path = staging.path().join("database.sqlite3");
        drop(owner::create_private_file(&path)?);
        let mut destination = database::open_connection(&path, database::Access::ReadWrite)?;
        database::bounded_backup(&self.connection, &mut destination)?;
        destination.pragma_update(None, "journal_mode", "DELETE")?;
        drop(destination);
        File::open(&path)?.sync_all()?;
        let (hash, size) = hash_file(&path, MAX_DATABASE)?;
        let mut payloads = vec![NativePayload {
            name: "database.sqlite3".into(),
            sha256: hash,
            byte_length: size,
        }];
        let mut missing_assets = Vec::new();
        let mut total = size;
        // The writer owns both registration/GC and this capture. Every immutable file is
        // copied before releasing it, so later unlink/GC cannot invalidate the capture.
        let registered = validation::asset_rows(&self.connection)?;
        owner::create_private_tree(&staging.path().join("assets"))?;
        for (digest, length) in registered {
            total = total.checked_add(length).ok_or_else(budget)?;
            if total > MAX_EXPANDED {
                return Err(budget());
            }
            let payload = NativePayload {
                name: format!("assets/{}", digest.as_str()),
                sha256: digest.as_str().into(),
                byte_length: length,
            };
            let source_path = assets::asset_path(self.owner.root(), &digest);
            let safe_parent = [
                self.owner.root().join("assets"),
                self.owner.root().join("assets/sha256"),
                source_path.parent().ok_or_else(invalid)?.to_path_buf(),
            ]
            .iter()
            .all(|path| owner::private_directory(path).is_ok());
            let copied = if safe_parent {
                copy_verified(&source_path, &staging.path().join(&payload.name), &payload)
            } else {
                Err(invalid())
            };
            match copied {
                Ok(()) => payloads.push(payload),
                Err(error) if error.code == ErrorCode::RecoveryRequired => {
                    missing_assets.push(digest);
                }
                Err(error) => return Err(error),
            }
        }
        let manifest = NativeManifest { format_version:1, kind:"nativeFull".into(), scope:"selectedCourse".into(),
            schema_version:SCHEMA_VERSION, database_section_version:1, assets_section_version:1,
            source_course_id, workspace_id:self.id.clone(), complete:missing_assets.is_empty(), missing_assets,
            omissions:vec!["Other courses, catalogue, credentials and runtime files are outside selected-course scope.".into()], payloads,
            table_counts:validation::table_counts(&self.connection)? };
        validation::validate(&path, &manifest)?;
        Ok(NativeCapture { manifest, staging })
    }
}

fn invalid() -> AppError {
    AppError::new(ErrorCode::RecoveryRequired)
}
fn budget() -> AppError {
    AppError::new(ErrorCode::BudgetExceeded)
}
fn json<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    serde_json::to_vec(value).map_err(|_| invalid())
}
fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub(super) fn open_regular(path: &Path) -> Result<File> {
    use rustix::fs::{Mode, OFlags, open};
    let file = open(
        path,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK,
        Mode::empty(),
    )
    .map(File::from)
    .map_err(|_| invalid())?;
    if !file.metadata()?.is_file() {
        return Err(invalid());
    }
    Ok(file)
}
fn stream_hash(reader: &mut dyn Read, writer: &mut dyn Write, limit: u64) -> Result<(String, u64)> {
    let mut hash = Sha256::new();
    let mut size = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        size = size.checked_add(n as u64).ok_or_else(budget)?;
        if size > limit {
            return Err(budget());
        }
        hash.update(&buffer[..n]);
        writer.write_all(&buffer[..n])?;
    }
    Ok((format!("{:x}", hash.finalize()), size))
}
fn hash_file(path: &Path, limit: u64) -> Result<(String, u64)> {
    stream_hash(&mut open_regular(path)?, &mut std::io::sink(), limit)
}
fn copy_verified(source: &Path, destination: &Path, payload: &NativePayload) -> Result<()> {
    let mut input = open_regular(source)?;
    let mut output = owner::create_private_file(destination)?;
    let result = stream_hash(&mut input, &mut output, payload.byte_length);
    match result {
        Ok((hash, size)) if hash == payload.sha256 && size == payload.byte_length => {
            output.sync_all()?;
            Ok(())
        }
        _ => {
            drop(output);
            std::fs::remove_file(destination)?;
            Err(invalid())
        }
    }
}
#[cfg(test)]
mod tests;
