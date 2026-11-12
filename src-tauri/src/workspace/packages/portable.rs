//! Versioned content exchange. Rebuild only a trusted schema; never import a database.
mod records;
mod wire;
use super::*;
use records::{Asset, Content};
use rusqlite::params;
use std::collections::{BTreeMap, BTreeSet};
const MAX_CONTENT: u64 = 64 * 1024 * 1024;
const MAX_ROWS: usize = 20_000;
const OMISSIONS: &[&str] = &[
    "personal annotations",
    "reading positions",
    "learner responses and progress",
    "reading preferences",
    "private lesson drafts",
    "ingestion jobs",
    "operation receipts",
    "secrets, grants and tokens",
    "unrelated courses and catalogue",
    "subject prior knowledge",
    "unreferenced assets",
    "derived search indexes",
    "M4 attempts, history and schedules (not implemented)",
];
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortableManifest {
    pub format_version: u32,
    pub kind: String,
    pub scope: String,
    pub source_course_id: Uuid,
    pub workspace_id: WorkspaceId,
    pub sections: BTreeMap<String, u32>,
    /// Empty in V1. A required future adapter is rejected, never silently discarded.
    pub required_adapters: BTreeMap<String, u32>,
    pub originals_requested: bool,
    pub intentionally_omitted_assets: Vec<AssetDigest>,
    pub unavailable_assets: Vec<AssetDigest>,
    pub omissions: Vec<String>,
    pub included_records: BTreeMap<String, u64>,
    pub authored_versions: BTreeMap<String, u64>,
    pub payloads: Vec<NativePayload>,
}
pub struct PortableCapture {
    manifest: PortableManifest,
    native: NativeCapture,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PortableImportReport {
    pub kind: String,
    pub source_course_id: Uuid,
    pub course_id: Uuid,
    pub workspace_id: WorkspaceId,
    pub internal_ids_preserved: bool,
    pub full_recovery: bool,
    pub intentionally_omitted_assets: Vec<AssetDigest>,
    pub unavailable_assets: Vec<AssetDigest>,
    pub omissions: Vec<String>,
    pub included_records: BTreeMap<String, u64>,
    pub authored_versions: BTreeMap<String, u64>,
}
fn stage() -> Result<tempfile::TempDir> {
    Ok(tempfile::Builder::new()
        .permissions(std::fs::Permissions::from_mode(0o700))
        .prefix("memoquill-portable-")
        .tempdir()?)
}
fn sections() -> BTreeMap<String, u32> {
    BTreeMap::from([
        ("content".into(), 1),
        ("readable".into(), 1),
        ("assets".into(), 1),
    ])
}
fn versions(content: &Content) -> BTreeMap<String, u64> {
    BTreeMap::from([
        (
            "resourceRevisions".into(),
            content.resource_revisions.len() as u64,
        ),
        (
            "lessonRevisions".into(),
            content.lesson_revisions.len() as u64,
        ),
        (
            "activityRevisions".into(),
            content.lesson_activity_revisions.len() as u64,
        ),
        (
            "sourceBindings".into(),
            content.lesson_source_bindings.len() as u64,
        ),
        (
            "originalSourceVersions".into(),
            content.original_sources.len() as u64,
        ),
    ])
}
fn asset_references(content: &Content) -> Result<BTreeSet<String>> {
    fn visit(v: &serde_json::Value, out: &mut BTreeSet<String>) {
        match v {
            serde_json::Value::Object(map) => {
                for (k, v) in map {
                    if k == "assetDigest"
                        && let Some(s) = v.as_str()
                    {
                        out.insert(s.into());
                    }
                    visit(v, out);
                }
            }
            serde_json::Value::Array(a) => {
                for v in a {
                    visit(v, out);
                }
            }
            _ => {}
        }
    }
    let mut out = content
        .original_sources
        .iter()
        .map(|s| s.digest.clone())
        .collect();
    for row in &content.resource_revisions {
        visit(
            &serde_json::to_value(&row.document).map_err(|_| invalid())?,
            &mut out,
        );
    }
    for row in &content.lesson_activity_revisions {
        visit(&row.definition.0, &mut out);
    }
    for row in &content.lesson_source_bindings {
        out.insert(row.data.0.asset_digest.clone());
    }
    Ok(out)
}
fn projection(content: &Content) -> Result<Vec<u8>> {
    // Plain text avoids interpreting imported HTML/Markdown, scripts or resource URLs.
    let mut text =
        String::from("Memoquill portable content — content only; no learner history.\n\n");
    for r in &content.resource_revisions {
        let title = content
            .resources
            .iter()
            .find(|v| v.id == r.resource_id)
            .map(|v| v.title.as_str())
            .unwrap_or("Untitled");
        text.push_str(&format!(
            "RESOURCE {title}\nID {} / revision {} ({})\n",
            r.resource_id, r.revision, r.id
        ));
        text.push_str(&serde_json::to_string_pretty(&r.document).map_err(|_| invalid())?);
        text.push_str("\n\n");
    }
    for r in &content.lesson_revisions {
        text.push_str(&format!("LESSON {} / revision {}\n", r.unit_id, r.id));
        text.push_str(&serde_json::to_string_pretty(&r.data).map_err(|_| invalid())?);
        text.push_str("\n\n");
    }
    for s in &content.original_sources {
        text.push_str(&format!(
            "SOURCE {} / {}\nAttribution: {:?}\nLicense: {:?}\nOrigin: {:?}\n\n",
            s.id, s.filename, s.attribution, s.license, s.origin_url
        ));
    }
    if text.len() as u64 > MAX_CONTENT {
        return Err(budget());
    }
    Ok(text.into_bytes())
}
fn put(staging: &Path, name: &str, bytes: &[u8]) -> Result<NativePayload> {
    if bytes.len() as u64 > MAX_CONTENT {
        return Err(budget());
    }
    let mut f = owner::create_private_file(&staging.join(name))?;
    f.write_all(bytes)?;
    f.sync_all()?;
    Ok(NativePayload {
        name: name.into(),
        sha256: hash_bytes(bytes),
        byte_length: bytes.len() as u64,
    })
}
impl Database {
    pub(in crate::workspace) fn capture_portable(
        &mut self,
        context: RequestContext,
        source_course_id: Uuid,
        originals: bool,
    ) -> Result<PortableCapture> {
        context.authorize()?;
        if source_course_id.get_version_num() != 4 {
            return Err(invalid());
        }
        let mut content = Content::capture(&self.connection)?;
        // Learner's prior knowledge is not authored shareable content.
        for subject in &mut content.subjects {
            subject.prior_knowledge = None;
        }
        let references = asset_references(&content)?;
        let registered: BTreeMap<_, _> = validation::asset_rows(&self.connection)?
            .into_iter()
            .map(|(d, n)| (d.as_str().to_owned(), (d, n)))
            .collect();
        for reference in references {
            let (digest, byte_length) = registered.get(&reference).ok_or_else(invalid)?;
            content.assets.push(Asset {
                digest: digest.clone(),
                byte_length: *byte_length,
            });
        }
        let staging = stage()?;
        owner::create_private_tree(&staging.path().join("assets"))?;
        owner::create_private_tree(&staging.path().join("readable"))?;
        let mut payloads = vec![
            put(staging.path(), "content.ndjson", &content.encode()?)?,
            put(
                staging.path(),
                "readable/content.txt",
                &projection(&content)?,
            )?,
        ];
        let mut omitted = Vec::new();
        let mut unavailable = Vec::new();
        for asset in &content.assets {
            let payload = NativePayload {
                name: format!("assets/{}", asset.digest.as_str()),
                sha256: asset.digest.as_str().into(),
                byte_length: asset.byte_length,
            };
            let source = assets::asset_path(self.owner.root(), &asset.digest);
            let safe = [
                self.owner.root().join("assets"),
                self.owner.root().join("assets/sha256"),
                source.parent().ok_or_else(invalid)?.to_path_buf(),
            ]
            .iter()
            .all(|p| owner::private_directory(p).is_ok());
            let verified = if safe {
                hash_file(&source, asset.byte_length)
                    .is_ok_and(|(h, n)| h == payload.sha256 && n == asset.byte_length)
            } else {
                false
            };
            if !verified {
                unavailable.push(asset.digest.clone());
            } else if !originals {
                omitted.push(asset.digest.clone());
            } else {
                copy_verified(&source, &staging.path().join(&payload.name), &payload)?;
                payloads.push(payload);
            }
        }
        let manifest = PortableManifest {
            format_version: 1,
            kind: "portableContent".into(),
            scope: "selectedCourse".into(),
            source_course_id,
            workspace_id: self.id.clone(),
            sections: sections(),
            required_adapters: BTreeMap::new(),
            originals_requested: originals,
            intentionally_omitted_assets: omitted,
            unavailable_assets: unavailable,
            omissions: OMISSIONS.iter().map(|s| (*s).into()).collect(),
            included_records: content.counts()?,
            authored_versions: versions(&content),
            payloads,
        };
        PortableCapture::assemble(manifest, content, staging)
    }
}
impl PortableCapture {
    pub fn manifest(&self) -> &PortableManifest {
        &self.manifest
    }
    pub fn write_to<W: Write + Seek>(&self, writer: W) -> Result<()> {
        wire::write(self, writer)
    }
    pub fn read_from<R: Read + Seek>(reader: R) -> Result<Self> {
        wire::read(reader)
    }
    pub(in crate::workspace) fn native(&self) -> &NativeCapture {
        &self.native
    }
    pub(in crate::workspace) fn identity(&self) -> Result<String> {
        Ok(hash_bytes(&json(&self.manifest)?))
    }
    pub(in crate::workspace) fn report(&self, course_id: Uuid) -> PortableImportReport {
        PortableImportReport {
            kind: "portableContent".into(),
            source_course_id: self.manifest.source_course_id,
            course_id,
            workspace_id: self.manifest.workspace_id.clone(),
            internal_ids_preserved: true,
            full_recovery: false,
            intentionally_omitted_assets: self.manifest.intentionally_omitted_assets.clone(),
            unavailable_assets: self.manifest.unavailable_assets.clone(),
            omissions: self.manifest.omissions.clone(),
            included_records: self.manifest.included_records.clone(),
            authored_versions: self.manifest.authored_versions.clone(),
        }
    }
    fn assemble(
        manifest: PortableManifest,
        content: Content,
        staging: tempfile::TempDir,
    ) -> Result<Self> {
        wire::validate_manifest(&manifest)?;
        if content.counts()? != manifest.included_records
            || versions(&content) != manifest.authored_versions
            || content.subjects.iter().any(|s| s.prior_knowledge.is_some())
        {
            return Err(invalid());
        }
        let references = asset_references(&content)?;
        let mut registered = BTreeSet::new();
        for a in &content.assets {
            if a.byte_length > MAX_ASSET || !registered.insert(a.digest.as_str().to_owned()) {
                return Err(invalid());
            }
        }
        if references != registered {
            return Err(invalid());
        }
        let omitted: BTreeSet<_> = manifest
            .intentionally_omitted_assets
            .iter()
            .map(|d| d.as_str().to_owned())
            .collect();
        let unavailable: BTreeSet<_> = manifest
            .unavailable_assets
            .iter()
            .map(|d| d.as_str().to_owned())
            .collect();
        let included: BTreeMap<_, _> = manifest
            .payloads
            .iter()
            .filter(|p| p.name.starts_with("assets/"))
            .map(|p| (p.sha256.clone(), p.byte_length))
            .collect();
        if !omitted.is_disjoint(&unavailable)
            || omitted.len() != manifest.intentionally_omitted_assets.len()
            || unavailable.len() != manifest.unavailable_assets.len()
            || included.len() + omitted.len() + unavailable.len() != registered.len()
        {
            return Err(invalid());
        }
        for a in &content.assets {
            match included.get(a.digest.as_str()) {
                Some(n)
                    if *n == a.byte_length
                        && !omitted.contains(a.digest.as_str())
                        && !unavailable.contains(a.digest.as_str()) => {}
                None if omitted.contains(a.digest.as_str())
                    || unavailable.contains(a.digest.as_str()) => {}
                _ => return Err(invalid()),
            }
        }
        let readable = manifest
            .payloads
            .iter()
            .find(|p| p.name == "readable/content.txt")
            .ok_or_else(invalid)?;
        let expected = projection(&content)?;
        if readable.sha256 != hash_bytes(&expected) || readable.byte_length != expected.len() as u64
        {
            return Err(invalid());
        }
        let path = staging.path().join("database.sqlite3");
        database::bootstrap(&path, SCHEMA_VERSION)?;
        let mut connection = database::open_connection(&path, database::Access::ReadWrite)?;
        connection.pragma_update(None, "journal_mode", "DELETE")?;
        let tx = connection.transaction()?;
        tx.pragma_update(None, "defer_foreign_keys", true)?;
        tx.execute(
            "UPDATE workspace_meta SET id=?1",
            [manifest.workspace_id.as_str()],
        )?;
        tx.execute(
            "UPDATE preferences SET workspace_id=?1",
            [manifest.workspace_id.as_str()],
        )?;
        tx.execute(
            "INSERT INTO library_meta(workspace_id,revision) VALUES(?1,0)",
            [manifest.workspace_id.as_str()],
        )?;
        // New local receipts satisfy asset registration invariants. No source receipt is imported.
        for a in &content.assets {
            let mut bytes = [0u8; 16];
            for (i, b) in bytes.iter_mut().enumerate() {
                *b = u8::from_str_radix(&a.digest.as_str()[i * 2..i * 2 + 2], 16)
                    .map_err(|_| invalid())?;
            }
            bytes[6] = (bytes[6] & 15) | 64;
            bytes[8] = (bytes[8] & 63) | 128;
            let id = OperationId::try_from(Uuid::from_bytes(bytes).to_string())?;
            let receipt = operations::applied(
                id.clone(),
                OperationOutcome::Asset(AssetInfo {
                    digest: a.digest.clone(),
                    byte_length: a.byte_length,
                }),
            );
            operations::insert_applied(
                &tx,
                &manifest.workspace_id,
                &receipt,
                a.digest.as_str(),
                "asset",
            )?;
            tx.execute(
                "INSERT INTO assets(digest,byte_length,registered_by) VALUES(?1,?2,?3)",
                params![a.digest.as_str(), a.byte_length as i64, id.as_str()],
            )?;
        }
        content.insert(&tx)?;
        tx.execute("UPDATE search_index_state SET dirty=1", [])?;
        tx.commit()?;
        super::super::lessons::validate_portable_data(&connection)?;
        let mut native_payloads = manifest
            .payloads
            .iter()
            .filter(|p| p.name.starts_with("assets/"))
            .cloned()
            .collect::<Vec<_>>();
        let counts = validation::table_counts(&connection)?;
        connection.execute_batch("VACUUM")?;
        drop(connection);
        let (sha256, byte_length) = hash_file(&path, MAX_DATABASE)?;
        native_payloads.insert(
            0,
            NativePayload {
                name: "database.sqlite3".into(),
                sha256,
                byte_length,
            },
        );
        let missing_assets = manifest
            .intentionally_omitted_assets
            .iter()
            .chain(&manifest.unavailable_assets)
            .cloned()
            .collect::<Vec<_>>();
        let native_manifest=NativeManifest{format_version:1,kind:"nativeFull".into(),scope:"selectedCourse".into(),schema_version:SCHEMA_VERSION,database_section_version:1,assets_section_version:1,source_course_id:manifest.source_course_id,workspace_id:manifest.workspace_id.clone(),complete:missing_assets.is_empty(),missing_assets,omissions:vec!["Other courses, catalogue, credentials and runtime files are outside selected-course scope.".into()],payloads:native_payloads,table_counts:counts};
        validation::validate(&path, &native_manifest)?;
        Ok(Self {
            manifest,
            native: NativeCapture {
                manifest: native_manifest,
                staging,
            },
        })
    }
}
#[cfg(test)]
mod tests;
