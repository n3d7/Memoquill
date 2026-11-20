//! Explicit synthetic qualification helpers, never run in ordinary test profiles.
//! SQL replication is fixture setup, not an application/import API.
use super::*;
use rusqlite::params;
use rusqlite::types::Value;
use std::{collections::BTreeMap, fs, path::Path, time::Instant};

const TABLES: &[&str] = &[
    "resources",
    "units",
    "unit_resources",
    "resource_revisions",
    "resource_heads",
    "lesson_revisions",
    "lesson_heads",
    "lesson_activity_revisions",
    "lesson_activity_heads",
    "lesson_dependencies",
    "study_sessions",
    "study_attempts",
    "study_judgments",
    "study_operations",
];

struct Rows {
    name: &'static str,
    columns: Vec<String>,
    rows: Vec<Vec<Value>>,
}

fn uuid_tokens(text: &str) -> impl Iterator<Item = &str> {
    std::iter::once(text)
        .chain(text.split('"'))
        .filter(|s| s.len() == 36 && Uuid::parse_str(s).is_ok_and(|id| id.get_version_num() == 4))
}

fn replace_ids(text: &str, ids: &BTreeMap<String, String>) -> String {
    if let Some(id) = ids.get(text) {
        return id.clone();
    }
    // UUIDs in these typed records are complete JSON string values, never substrings.
    text.split('"')
        .map(|s| ids.get(s).map_or(s, String::as_str))
        .collect::<Vec<_>>()
        .join("\"")
}

fn fixed_id(group: u64, ordinal: u64) -> String {
    format!("{group:08x}-0000-4000-8000-{ordinal:012x}")
}

fn template(db: &Database) -> Vec<Rows> {
    TABLES
        .iter()
        .map(|name| {
            let mut statement = db
                .connection
                .prepare(&format!("SELECT * FROM {name} ORDER BY rowid"))
                .unwrap();
            let columns = statement
                .column_names()
                .map(|s| s.to_string())
                .collect::<Vec<_>>();
            let count = columns.len();
            let rows = statement
                .query_map([], |row| (0..count).map(|i| row.get(i)).collect())
                .unwrap()
                .collect::<rusqlite::Result<Vec<Vec<Value>>>>()
                .unwrap();
            Rows {
                name,
                columns,
                rows,
            }
        })
        .collect()
}

fn seed() -> (tempfile::TempDir, Database) {
    let (temp, mut db, unit, revision) = setup_sequence(vec![recall(); 10]);
    let resource = crate::organization::ResourceId::generate();
    db.connection
        .execute(
            "INSERT INTO resources VALUES(?1,'Scale resource',0)",
            [resource.to_string()],
        )
        .unwrap();
    db.connection
        .execute(
            "INSERT INTO unit_resources VALUES(?1,?2,0)",
            params![unit, resource.to_string()],
        )
        .unwrap();
    db.save_document(context(), &SaveDocument {
        contract_version: 1, workspace_id: db.id.clone(), operation_id: OperationId::generate(),
        resource_id: resource, expected_revision: 0,
        document: crate::document::Document { schema_version: 1, blocks: vec![crate::document::Block {
            id: Uuid::new_v4(), content: crate::document::BlockContent::Code {
                text: "Scale corpus: English; Русский; 日本語; العربية; e\u{301}; 😀\nfn square(x: i64) -> i64 { x * x }\nSELECT title FROM resources WHERE archived = 0;".repeat(24),
                language: Some("rust".into()),
            },
        }] },
    }).unwrap();
    for _ in 0..10 {
        let mut s = start(&mut db, &unit, &revision, StudyMode::Practice);
        for _ in 0..10 {
            s = save(
                &mut db,
                &s,
                StudyResponse::Text {
                    value: "Answer".into(),
                },
            );
            s = db
                .study_apply(
                    context(),
                    &request(
                        &s,
                        StudyAction::Submit {
                            self_rating: Some(4),
                            recall_rating: None,
                            self_outcome: Some(AssessmentOutcome::Correct),
                        },
                    ),
                )
                .unwrap();
        }
    }
    validate_native_data(&db.connection).unwrap();
    (temp, db)
}

fn validate_fixture(db: &Database) {
    crate::workspace::content::validate_native_data(&db.connection, &db.id).unwrap();
    crate::workspace::lessons::validate_native_data(&db.connection).unwrap();
    validate_native_data(&db.connection).unwrap();
    crate::workspace::sources::validate_native_data(&db.connection).unwrap();
    crate::workspace::operations::validate_native_data(&db.connection, &db.id).unwrap();
    let failures: i64 = db
        .connection
        .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(failures, 0);
    let integrity: String = db
        .connection
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .unwrap();
    assert_eq!(integrity, "ok");
}

fn asset_bytes(index: usize) -> (Vec<u8>, &'static str, &'static str) {
    let (extension, mime, header) = match index % 4 {
        0 => ("txt", "text/plain", format!("Synthetic notes {index}\n")),
        1 => (
            "md",
            "text/markdown",
            format!(
                "# Synthetic lesson {index}\n\n```rust\nfn square(x: i64) -> i64 {{ x*x }}\n```\n"
            ),
        ),
        2 => ("csv", "text/csv", "id,language,prompt,answer\n".into()),
        _ => ("json", "application/json", "[\n".into()),
    };
    let mut text = header;
    let mut line = 0;
    while text.len() < 1_000_000 - 200 {
        let row = match index % 4 {
            2 => format!(
                "{index}-{line},English,Explain square {line},{}\n",
                line * line
            ),
            3 => format!(
                "{{\"id\":\"{index}-{line}\",\"prompt\":\"平方 Русский العربية\",\"value\":{line}}},\n"
            ),
            _ => format!(
                "{index}:{line} — English Русский 日本語 العربية e\u{301} 😀; square({line}) = {}\n",
                line * line
            ),
        };
        text.push_str(&row);
        line += 1;
    }
    if index % 4 == 3 {
        text.pop(); // newline
        text.pop(); // comma
        text.push_str("\n]");
    }
    text.extend(std::iter::repeat_n(' ', 1_000_000 - text.len()));
    assert_eq!(text.len(), 1_000_000);
    (text.into_bytes(), extension, mime)
}

fn write_json(path: &Path, value: &serde_json::Value) {
    fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

#[test]
#[ignore = "Explicit runner-only synthetic fixture generation; retains generated data on SSD"]
fn generate_scale_fixture() {
    let mode = std::env::var("MEMOQUILL_SCALE_MODE").expect("explicit pilot or full required");
    let (subjects, resources) = match mode.as_str() {
        "pilot" => (2, 20),
        "full" => (100, 10_000),
        _ => panic!("unsupported scale mode"),
    };
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let folder = repository.join(format!(".qualification/m5-scale/{mode}-v1"));
    fs::create_dir_all(folder.parent().unwrap()).unwrap();
    fs::create_dir(&folder).expect("new fixture only; never overwrite an existing dataset");
    let began = Instant::now();
    let (_seed_temp, seed) = seed();
    // Preserve the real service-created seed for exact replay (UUIDs/times are captured).
    seed.connection
        .execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")
        .unwrap();
    fs::copy(
        seed.owner.root().join("memoquill.sqlite3"),
        folder.join("seed.sqlite3"),
    )
    .unwrap();
    let tables = template(&seed);
    let mut all_ids = std::collections::BTreeSet::new();
    for table in &tables {
        for row in &table.rows {
            for value in row {
                if let Value::Text(text) = value {
                    all_ids.extend(uuid_tokens(text).map(str::to_owned));
                }
            }
        }
    }
    all_ids.remove(seed.id.as_str());
    let mut db = Database::open(&folder.join("workspace")).unwrap();
    let workspace_id = db.id.as_str().to_owned();
    let tx = db.connection.transaction().unwrap();
    for subject in 0..subjects {
        let id = fixed_id(0xf001, subject);
        tx.execute(
            "INSERT INTO subjects VALUES(?1,?2,NULL,0)",
            params![id, format!("Scale subject {subject:03}")],
        )
        .unwrap();
        tx.execute(
            "INSERT INTO collections VALUES(?1,?2,NULL,'Scale collection',0,0)",
            params![fixed_id(0xf002, subject), id],
        )
        .unwrap();
    }
    for index in 0..resources {
        let mut ids = all_ids
            .iter()
            .enumerate()
            .map(|(i, id)| (id.clone(), fixed_id(index as u64 + 1, i as u64)))
            .collect::<BTreeMap<_, _>>();
        ids.insert(seed.id.as_str().to_owned(), workspace_id.clone());
        for table in &tables {
            let placeholders = vec!["?"; table.columns.len()].join(",");
            let mut insert = tx
                .prepare(&format!(
                    "INSERT INTO {} VALUES({placeholders})",
                    table.name
                ))
                .unwrap();
            for row in &table.rows {
                let mut row = row
                    .iter()
                    .map(|v| match v {
                        Value::Text(text) => Value::Text(replace_ids(text, &ids)),
                        other => other.clone(),
                    })
                    .collect::<Vec<_>>();
                if table.name == "resources" || table.name == "units" {
                    row[1] = Value::Text(format!(
                        "Scale {} {index:05}",
                        if table.name == "resources" {
                            "resource"
                        } else {
                            "lesson"
                        }
                    ));
                }
                if let Some(digest) = table.columns.iter().position(|c| c == "digest") {
                    let data = table
                        .columns
                        .iter()
                        .position(|c| {
                            matches!(c.as_str(), "data" | "document" | "definition" | "result")
                        })
                        .unwrap();
                    let Value::Text(text) = &row[data] else {
                        panic!("non-text canonical record")
                    };
                    row[digest] =
                        Value::Text(crate::workspace::migrations::checksum(text.as_bytes()));
                }
                insert.execute(rusqlite::params_from_iter(row)).unwrap();
            }
        }
        let resource: String = match &tables[0].rows[0][0] {
            Value::Text(id) => ids[id].clone(),
            _ => unreachable!(),
        };
        tx.execute(
            "INSERT INTO placements VALUES(?1,?2,?3,NULL,?4,0)",
            params![
                fixed_id(0xf003, index as u64),
                fixed_id(0xf002, index as u64 % subjects),
                resource,
                index as i64 / subjects as i64
            ],
        )
        .unwrap();
        if index % 100 == 0 {
            write_json(
                &folder.join("progress.json"),
                &json!({"phase":"database","resources":index + 1,"elapsedSeconds":began.elapsed().as_secs_f64()}),
            );
        }
    }
    tx.execute("UPDATE library_meta SET revision=1", [])
        .unwrap();
    tx.commit().unwrap();
    let ids = db
        .connection
        .prepare("SELECT id FROM resources ORDER BY title")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap();
    for (index, resource) in ids.iter().enumerate() {
        let (bytes, ext, mime) = asset_bytes(index);
        let info = db
            .register_asset(
                context(),
                &RegisterAsset {
                    contract_version: 1,
                    workspace_id: db.id.clone(),
                    operation_id: OperationId::generate(),
                },
                &mut bytes.as_slice(),
                1_000_000,
            )
            .unwrap();
        db.save_source(
            context(),
            &SaveSource {
                contract_version: 1,
                workspace_id: db.id.clone(),
                operation_id: OperationId::generate(),
                expected_revision: 0,
                source: NewSource {
                    id: Uuid::parse_str(&fixed_id(0xf004, index as u64)).unwrap(),
                    resource_id: serde_json::from_value(json!(resource)).unwrap(),
                    digest: info.digest,
                    filename: format!("synthetic-{index:05}.{ext}"),
                    media_type: mime.into(),
                    attribution: Some(
                        "Memoquill deterministic synthetic qualification corpus".into(),
                    ),
                    license: Some("CC0-1.0".into()),
                    origin_url: None,
                },
            },
        )
        .unwrap();
        if index % 100 == 0 {
            write_json(
                &folder.join("progress.json"),
                &json!({"phase":"assets","resources":index + 1,"elapsedSeconds":began.elapsed().as_secs_f64()}),
            );
        }
    }
    // Full admission is a separate bounded gate, never implied by row counts.
    if mode == "pilot" {
        validate_fixture(&db);
    }
    db.connection
        .execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")
        .unwrap();
    let mut counts = serde_json::Map::new();
    for table in [
        "subjects",
        "resources",
        "lesson_activity_revisions",
        "study_sessions",
        "study_attempts",
        "study_operations",
        "assets",
        "original_sources",
    ] {
        let count: i64 = db
            .connection
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
            .unwrap();
        counts.insert(table.into(), json!(count));
    }
    assert_eq!(counts["subjects"], json!(subjects));
    assert_eq!(counts["resources"], json!(resources));
    assert_eq!(counts["lesson_activity_revisions"], json!(resources * 10));
    assert_eq!(counts["study_attempts"], json!(resources * 100));
    let bytes: i64 = db
        .connection
        .query_row("SELECT sum(byte_length) FROM assets", [], |r| r.get(0))
        .unwrap();
    assert_eq!(bytes, resources as i64 * 1_000_000);
    write_json(
        &folder.join("manifest.json"),
        &json!({"schemaVersion":1,"synthetic":true,"mode":mode,"counts":counts,"assetBytes":bytes,"assetFormats":["text","markdown","csv","json"],"pdfCoverage":"not_run","seedSha256":crate::workspace::migrations::checksum(&fs::read(folder.join("seed.sqlite3")).unwrap()),"generationSeconds":began.elapsed().as_secs_f64(),"fullAdmission":if mode == "pilot" {"passed"} else {"not_run"},"performance":"not_run","gui":"not_run"}),
    );
}

#[test]
#[ignore = "Explicit release-mode scale qualification through the verification runner"]
fn measure_scale_fixture() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let folder = repository.join(".qualification/m5-scale/full-v1");
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(folder.join("manifest.json")).expect("completed full fixture required"),
    )
    .unwrap();
    assert_eq!(manifest["counts"]["study_attempts"], json!(1_000_000));
    assert_eq!(manifest["assetBytes"], json!(10_000_000_000_u64));
    let mut openings = Vec::new();
    let mut libraries = Vec::new();
    for _ in 0..3 {
        let start = Instant::now();
        let db = Database::open(&folder.join("workspace")).unwrap();
        openings.push(start.elapsed().as_secs_f64() * 1000.0);
        assert_eq!(
            serde_json::to_value(db.recovery_report()).unwrap(),
            serde_json::to_value(RecoveryReport::default()).unwrap()
        );
        let start = Instant::now();
        let library = db.library(context()).unwrap();
        libraries.push(start.elapsed().as_secs_f64() * 1000.0);
        assert_eq!(library.resources.len(), 10_000);
        assert_eq!(library.subjects.len(), 100);
    }
    let db = Database::open(&folder.join("workspace")).unwrap();
    let start = Instant::now();
    db.rebuild_search_index(context()).unwrap();
    let rebuild = start.elapsed().as_secs_f64() * 1000.0;
    let queries = [
        "Scale",
        "resource 00000",
        "resource 05000",
        "resource 09999",
        "Русский",
        "日本語",
        "العربية",
        "square",
        "WHERE archived",
        "nonexistentxyz",
    ];
    let mut samples = Vec::new();
    for _ in 0..5 {
        for query in queries {
            let start = Instant::now();
            let page = db.search_resources(context(), query, 0).unwrap();
            samples.push(json!({"query":query,"offset":0,"milliseconds":start.elapsed().as_secs_f64()*1000.0,"hits":page.items.len(),"nextOffset":page.next_offset,"indexRebuilt":page.index_rebuilt}));
            assert!(!page.index_rebuilt);
            assert_eq!(
                page.items.len(),
                if query == "nonexistentxyz" {
                    0
                } else if query == "resource 00000" {
                    1
                } else if query.starts_with("resource ") {
                    0
                } else {
                    20
                }
            );
        }
    }
    let mut timings = samples
        .iter()
        .map(|v| v["milliseconds"].as_f64().unwrap())
        .collect::<Vec<_>>();
    timings.sort_by(f64::total_cmp);
    let p95 = timings[(timings.len() * 95).div_ceil(100) - 1];
    let mut positions = Vec::new();
    for offset in [0, 5_000, 9_980] {
        let start = Instant::now();
        let page = db.search_resources(context(), "Scale", offset).unwrap();
        assert_eq!(page.items.len(), 20);
        positions.push(json!({"offset":offset,"milliseconds":start.elapsed().as_secs_f64()*1000.0,"first":page.items.first().unwrap().title,"last":page.items.last().unwrap().title,"nextOffset":page.next_offset}));
    }
    let mut complete_queries = Vec::new();
    for query in ["resource 00000", "resource 05000", "resource 09999"] {
        let start = Instant::now();
        let mut offset = 0;
        let mut batches = 0;
        loop {
            let page = db.search_resources(context(), query, offset).unwrap();
            batches += 1;
            if !page.items.is_empty() {
                assert_eq!(page.items.len(), 1);
                complete_queries.push(json!({"query":query,"milliseconds":start.elapsed().as_secs_f64()*1000.0,"batches":batches,"hit":page.items[0].title}));
                break;
            }
            offset = page
                .next_offset
                .expect("fixture target must eventually be reachable");
        }
    }
    let mut progress_samples = Vec::new();
    for index in [0, 5_000, 9_999] {
        let unit: String = db
            .connection
            .query_row(
                "SELECT id FROM units WHERE title=?1",
                [format!("Scale lesson {index:05}")],
                |r| r.get(0),
            )
            .unwrap();
        let start = Instant::now();
        let progress = db.study_progress(context(), &unit, 0).unwrap();
        assert_eq!(progress.total_attempts, 100);
        progress_samples.push(json!({"unit":unit,"milliseconds":start.elapsed().as_secs_f64()*1000.0,"attempts":progress.total_attempts,"visibleEvidence":progress.evidence.len(),"nextOffset":progress.next_offset}));
    }
    let mut plans = serde_json::Map::new();
    for (name, sql) in [
        (
            "studyOperationsBySession",
            "EXPLAIN QUERY PLAN SELECT id FROM study_operations WHERE session_id='synthetic'",
        ),
        (
            "judgmentsByAttempt",
            "EXPLAIN QUERY PLAN SELECT id FROM study_judgments WHERE attempt_id='synthetic'",
        ),
    ] {
        let rows = db
            .connection
            .prepare(sql)
            .unwrap()
            .query_map([], |r| r.get::<_, String>(3))
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap();
        plans.insert(name.into(), json!(rows));
    }
    let raw_counts: (i64, i64) = db
        .connection
        .query_row(
            "SELECT (SELECT count(*) FROM study_operations),(SELECT count(*) FROM study_attempts)",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    let mut total: i64 = 0;
    let mut counts = serde_json::Map::new();
    let names = db
        .connection
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap();
    for name in names {
        let count: i64 = db
            .connection
            .query_row(
                &format!("SELECT count(*) FROM \"{}\"", name.replace('"', "\"\"")),
                [],
                |r| r.get(0),
            )
            .unwrap();
        total += count;
        counts.insert(name, json!(count));
    }
    let mut asset_files = 0_u64;
    let mut asset_bytes = 0_u64;
    let mut allocated_bytes = 0_u64;
    for row in db
        .connection
        .prepare("SELECT digest,byte_length FROM assets")
        .unwrap()
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
        .unwrap()
    {
        let (digest, length) = row.unwrap();
        let length = u64::try_from(length).unwrap();
        let path = folder
            .join("workspace/assets/sha256")
            .join(&digest[..2])
            .join(&digest[2..]);
        let meta = fs::metadata(&path).unwrap();
        use std::os::unix::fs::MetadataExt;
        assert_eq!(meta.len(), length);
        assert!(
            meta.blocks() * 512 >= length,
            "sparse asset is not representative"
        );
        asset_files += 1;
        asset_bytes += length;
        allocated_bytes += meta.blocks() * 512;
    }
    assert_eq!(asset_files, 10_000);
    assert_eq!(asset_bytes, 10_000_000_000);
    let memory = fs::read_to_string("/proc/self/status").unwrap();
    let memory = memory
        .lines()
        .filter(|s| s.starts_with("VmHWM:") || s.starts_with("VmRSS:"))
        .collect::<Vec<_>>();
    write_json(
        &folder.join("performance.json"),
        &json!({"schemaVersion":1,"build":"release","cacheState":"OS caches retained; generation-warm; distinct process opens, no cold-cache claim","workspaceOpenMs":openings,"libraryReadMs":libraries,"indexRebuildMs":rebuild,"searchSamples":samples,"searchP95Ms":p95,"positions":positions,"searchToFirstHit":complete_queries,"progressSamples":progress_samples,"queryPlans":plans,"memory":memory,"tableCounts":counts,"assetFiles":asset_files,"physicalAssetBytes":asset_bytes,"allocatedAssetBytes":allocated_bytes,"totalRowsIncludingSearchInternals":total,"nativeArchiveRowAdmission":{"status":"blocked","studyOperationRows":raw_counts.0,"attemptRows":raw_counts.1,"perTableLimit":1_000_000,"totalLimit":2_000_000},"warmUsefulViewProxyWithin3s":openings.iter().zip(&libraries).skip(1).all(|(a,b)| a+b<=3000.0),"searchWithin500ms":p95<=500.0,"gui":"not_run","orca":"not_run","uiBlocking":"not_run"}),
    );
    assert!(
        p95 <= 500.0,
        "common search p95 exceeded provisional 500ms budget; see performance.json"
    );
    assert!(
        openings
            .iter()
            .zip(&libraries)
            .skip(1)
            .all(|(a, b)| a + b <= 3000.0),
        "workspace-open plus library-read proxy exceeded provisional warm-start budget; real GUI launch remains separate"
    );
}

#[test]
#[ignore = "Explicit release-mode progress measurement on the synthetic native copy"]
fn measure_indexed_progress() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let folder = repository.join(".qualification/m5-scale");
    let root = folder.join("native-data/io.github.n3d7.memoquill/workspace");
    assert!(root.join("memoquill.sqlite3").exists());
    let began = Instant::now();
    let db = Database::open(&root).unwrap();
    let open_ms = began.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(
        serde_json::to_value(db.recovery_report()).unwrap(),
        serde_json::to_value(RecoveryReport::default()).unwrap()
    );
    let attempts: i64 = db
        .connection
        .query_row("SELECT count(*) FROM study_attempts", [], |r| r.get(0))
        .unwrap();
    assert_eq!(attempts, 1_000_000);
    let mut samples = Vec::new();
    for index in [0, 5_000, 9_999] {
        let unit: String = db
            .connection
            .query_row(
                "SELECT id FROM units WHERE title=?1",
                [format!("Scale lesson {index:05}")],
                |r| r.get(0),
            )
            .unwrap();
        let start = Instant::now();
        let progress = db.study_progress(context(), &unit, 0).unwrap();
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
        assert_eq!(progress.total_attempts, 100);
        assert_eq!(progress.evidence.len(), 32);
        assert_eq!(db.study_progress(context(), &unit, 0).unwrap(), progress);
        samples.push(json!({"unit":unit,"milliseconds":elapsed_ms,"attempts":progress.total_attempts,"visibleEvidence":progress.evidence.len()}));
    }
    write_json(
        &folder.join("indexed-progress.json"),
        &json!({"schemaVersion":db.connection.pragma_query_value(None,"user_version",|r| r.get::<_,i64>(0)).unwrap(),"workspace":root,"openAndMigrationMs":open_ms,"build":"release","samples":samples,"assetFilesSharedByHardlinks":true,"baselineUnchanged":true}),
    );
}

#[test]
#[ignore = "Explicit release-mode search comparison on the migrated synthetic copy"]
fn measure_cached_search() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let folder = repository.join(".qualification/m5-scale");
    let root = folder.join("native-data/io.github.n3d7.memoquill/workspace");
    let baseline: serde_json::Value =
        serde_json::from_slice(&fs::read(folder.join("full-v1/performance.json")).unwrap())
            .unwrap();
    let db = Database::open(&root).unwrap();
    let mut samples = Vec::new();
    for old in baseline["searchSamples"].as_array().unwrap() {
        let query = old["query"].as_str().unwrap();
        let start = Instant::now();
        let page = db
            .search_resources(context(), query, old["offset"].as_u64().unwrap())
            .unwrap();
        let ms = start.elapsed().as_secs_f64() * 1000.0;
        assert_eq!(page.items.len() as u64, old["hits"].as_u64().unwrap());
        samples.push(json!({"query":query,"milliseconds":ms,"hits":page.items.len(),"nextOffset":page.next_offset}));
    }
    assert_eq!(samples.len(), 50);
    let mut times = samples
        .iter()
        .map(|s| s["milliseconds"].as_f64().unwrap())
        .collect::<Vec<_>>();
    times.sort_by(f64::total_cmp);
    let p95 = times[(times.len() * 95).div_ceil(100) - 1];
    let mut targets = Vec::new();
    for query in ["resource 00000", "resource 05000", "resource 09999"] {
        let start = Instant::now();
        let mut offset = 0;
        let mut batches = 0;
        loop {
            let page = db.search_resources(context(), query, offset).unwrap();
            batches += 1;
            if !page.items.is_empty() {
                assert_eq!(page.items.len(), 1);
                targets.push(json!({"query":query,"milliseconds":start.elapsed().as_secs_f64()*1000.0,"batches":batches,"title":page.items[0].title}));
                break;
            }
            offset = page
                .next_offset
                .expect("canonical target must remain reachable");
        }
    }
    write_json(
        &folder.join("cached-search.json"),
        &json!({"build":"release","scope":"same 50 queries; first verification included; no controlled OS cache-state claim","samples":samples,"p95Ms":p95,"baselineP95Ms":baseline["searchP95Ms"],"targets":targets,"workspace":root,"baselineDatabaseUnchanged":true}),
    );
    assert!(
        p95 <= 500.0,
        "common search p95 exceeds the provisional 500ms budget"
    );
}

#[test]
#[ignore = "Explicit release-mode timing of startup reads on the migrated synthetic copy"]
fn measure_startup_reads() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let folder = repository.join(".qualification/m5-optimized-scale");
    let root = folder.join("data/io.github.n3d7.memoquill/workspace");
    assert!(root.join("memoquill.sqlite3").is_file());

    let start = Instant::now();
    let db = Database::open(&root).unwrap();
    let open_ms = start.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(
        db.connection
            .pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0))
            .unwrap(),
        10
    );

    let start = Instant::now();
    let snapshot = db.library(context()).unwrap();
    let library_ms = start.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(snapshot.resources.len(), 10_000);

    let start = Instant::now();
    let resume = db.study_resume_sessions(context(), 0).unwrap();
    let resume_ms = start.elapsed().as_secs_f64() * 1000.0;
    assert!(resume.sessions.is_empty());

    let mut review_ms = Vec::new();
    for _ in 0..2 {
        let start = Instant::now();
        let queue = db.review_overview(context()).unwrap();
        review_ms.push(start.elapsed().as_secs_f64() * 1000.0);
        assert_eq!(queue.new_total, 100_000);
        assert_eq!(queue.due_total, 0);
    }
    write_json(
        &folder.join("startup-reads.json"),
        &json!({
            "scope": "backend reads in release mode on the existing synthetic v10 dataset",
            "open_ms": open_ms,
            "library_ms": library_ms,
            "resume_ms": resume_ms,
            "review_ms": review_ms,
        }),
    );
}
