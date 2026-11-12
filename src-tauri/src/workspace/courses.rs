//! Course routing metadata only. Each course keeps the existing isolated workspace format.
use super::{
    AppError, ApplicationService, ErrorCode, NativeCapture, NativeRestoreReport, PortableCapture,
    PortableImportReport, error::Result, owner,
};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::Mutex};
use uuid::Uuid;

const MAX_COURSES: usize = 32;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Course {
    pub id: Uuid,
    pub title: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CourseCatalogue {
    pub contract_version: u8,
    pub selected_course_id: Uuid,
    pub courses: Vec<Course>,
}

struct State {
    connection: Connection,
    services: HashMap<Uuid, ApplicationService>,
    closing: bool,
    owner: Option<owner::Owner>,
}

pub struct CourseManager {
    app_data: PathBuf,
    state: Mutex<State>,
}

impl CourseManager {
    /// Admit only a verified package, always to a NEW isolated course. Operation UUID
    /// is its catalogue ID; durable intent survives a root-publish/admission crash.
    pub fn restore_native(
        &self,
        operation_id: Uuid,
        title: String,
        package: &NativeCapture,
        allow_incomplete: bool,
    ) -> Result<NativeRestoreReport> {
        self.admit_package(
            operation_id,
            title,
            package,
            allow_incomplete,
            package.identity()?,
        )
    }
    /// Content-only import: trusted rebuilt data, new course, manifest-bound durable retry.
    pub fn import_portable(
        &self,
        operation_id: Uuid,
        title: String,
        package: &PortableCapture,
    ) -> Result<PortableImportReport> {
        self.admit_package(
            operation_id,
            title,
            package.native(),
            true,
            package.identity()?,
        )?;
        Ok(package.report(operation_id))
    }
    fn admit_package(
        &self,
        operation_id: Uuid,
        title: String,
        package: &NativeCapture,
        allow_incomplete: bool,
        identity: String,
    ) -> Result<NativeRestoreReport> {
        validate_id(operation_id)?;
        validate_title(&title)?;
        if !package.manifest().complete && !allow_incomplete {
            return Err(AppError::new(ErrorCode::RecoveryRequired));
        }
        let mut state = self.state.lock().map_err(|_| unavailable())?;
        if state.closing {
            return Err(unavailable());
        }
        let current = catalogue(&state.connection)?;
        let journal = self.app_data.join("courses/restore-operations");
        owner::create_private_tree(&journal)?;
        owner::private_directory(&journal)?;
        let record_path = journal.join(format!("{operation_id}.json"));
        let intent = RestoreIntent {
            operation_id,
            identity,
            title: title.clone(),
        };
        let root = self
            .app_data
            .join("courses/data")
            .join(operation_id.to_string());
        let known = match std::fs::symlink_metadata(&record_path) {
            Ok(_) => {
                let mut file = super::packages::open_regular(&record_path)?;
                let mut bytes = Vec::new();
                std::io::Read::read_to_end(&mut std::io::Read::take(&mut file, 16385), &mut bytes)?;
                if bytes.len() > 16384 {
                    return Err(AppError::new(ErrorCode::RecoveryRequired));
                }
                let existing: RestoreIntent = serde_json::from_slice(&bytes)
                    .map_err(|_| AppError::new(ErrorCode::RecoveryRequired))?;
                if existing != intent {
                    return Err(AppError::new(ErrorCode::IdempotencyConflict));
                }
                true
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
            Err(error) => return Err(error.into()),
        };
        if current
            .courses
            .iter()
            .any(|course| course.id == operation_id)
        {
            if !known {
                return Err(AppError::new(ErrorCode::IdempotencyConflict));
            }
            return Ok(package.report(operation_id));
        }
        if current.courses.len() >= MAX_COURSES {
            return Err(AppError::new(ErrorCode::BudgetExceeded));
        }
        if !known {
            if std::fs::symlink_metadata(&root).is_ok() {
                return Err(AppError::new(ErrorCode::RecoveryRequired));
            }
            let staged = tempfile::NamedTempFile::new_in(&journal)?;
            let bytes = serde_json::to_vec(&intent)
                .map_err(|_| AppError::new(ErrorCode::InvalidRequest))?;
            std::io::Write::write_all(&mut staged.as_file(), &bytes)?;
            staged.as_file().sync_all()?;
            owner::publish_no_replace(staged.path(), &record_path)?;
            owner::sync_directory(&journal)?;
        }
        let data = self.app_data.join("courses/data");
        owner::private_directory(&data)?;
        match std::fs::symlink_metadata(&root) {
            Ok(_) if known => {
                let marker = root.join("restore-intent.json");
                let mut file = super::packages::open_regular(&marker)?;
                let mut bytes = Vec::new();
                std::io::Read::read_to_end(&mut std::io::Read::take(&mut file, 16385), &mut bytes)?;
                if bytes.len() > 16384
                    || serde_json::from_slice::<RestoreIntent>(&bytes)
                        .ok()
                        .as_ref()
                        != Some(&intent)
                {
                    return Err(AppError::new(ErrorCode::RecoveryRequired));
                }
                package.matches_root(&root)?;
            }
            Ok(_) => return Err(AppError::new(ErrorCode::RecoveryRequired)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let staging = tempfile::Builder::new()
                    .permissions({
                        use std::os::unix::fs::PermissionsExt;
                        std::fs::Permissions::from_mode(0o700)
                    })
                    .prefix(".restore-")
                    .tempdir_in(&data)?;
                package.materialize(staging.path())?;
                let mut marker =
                    owner::create_private_file(&staging.path().join("restore-intent.json"))?;
                std::io::Write::write_all(
                    &mut marker,
                    &serde_json::to_vec(&intent)
                        .map_err(|_| AppError::new(ErrorCode::InvalidRequest))?,
                )?;
                marker.sync_all()?;
                owner::sync_directory(staging.path())?;
                owner::publish_no_replace(staging.path(), &root)?;
                owner::sync_directory(&data)?;
            }
            Err(error) => return Err(error.into()),
        }
        super::faults::check(super::faults::Phase::Publish)?;
        let tx = state.connection.transaction()?;
        tx.execute(
            "INSERT INTO courses(id,title,legacy,opened) VALUES(?1,?2,0,1)",
            params![operation_id.to_string(), title],
        )?;
        tx.execute(
            "UPDATE catalogue_meta SET selected=?1 WHERE singleton=1",
            [operation_id.to_string()],
        )?;
        tx.commit()?;
        super::faults::check(super::faults::Phase::AfterCommit)?;
        Ok(package.report(operation_id))
    }
    pub fn open(app_data: PathBuf) -> Result<Self> {
        let root = app_data.join("courses");
        let guard = owner::Owner::acquire(&root)?;
        let data = root.join("data");
        owner::create_private_tree(&data)?;
        owner::private_directory(&data)?;
        let has_managed_data = std::fs::read_dir(&data)?.next().transpose()?.is_some();
        let path = root.join("catalogue.sqlite3");
        match std::fs::symlink_metadata(&path) {
            Ok(meta) => {
                use std::os::unix::fs::PermissionsExt;
                if !meta.is_file()
                    || meta.permissions().mode() & 0o077 != 0
                    || (meta.len() == 0 && has_managed_data)
                {
                    return Err(AppError::new(ErrorCode::RecoveryRequired));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                if has_managed_data {
                    return Err(AppError::new(ErrorCode::RecoveryRequired));
                }
                owner::create_private_file(&path)?.sync_all()?;
                owner::sync_directory(&root)?;
            }
            Err(error) => return Err(error.into()),
        }
        let mut connection = Connection::open(path)?;
        initialize(
            &mut connection,
            app_data.join("workspace").exists(),
            has_managed_data,
        )?;
        let manager = Self {
            app_data,
            state: Mutex::new(State {
                connection,
                services: HashMap::new(),
                closing: false,
                owner: Some(guard),
            }),
        };
        manager.list()?;
        Ok(manager)
    }

    pub fn list(&self) -> Result<CourseCatalogue> {
        let state = self.state.lock().map_err(|_| unavailable())?;
        if state.closing {
            return Err(unavailable());
        }
        catalogue(&state.connection)
    }

    /// The operation UUID is also the new course identity; retries cannot allocate another root.
    pub fn create(&self, operation_id: Uuid, title: String) -> Result<CourseCatalogue> {
        validate_id(operation_id)?;
        validate_title(&title)?;
        let mut state = self.state.lock().map_err(|_| unavailable())?;
        if state.closing {
            return Err(unavailable());
        }
        let current = catalogue(&state.connection)?;
        if let Some(existing) = current
            .courses
            .iter()
            .find(|course| course.id == operation_id)
        {
            return if existing.title == title {
                Ok(current)
            } else {
                Err(AppError::new(ErrorCode::IdempotencyConflict))
            };
        }
        if current.courses.len() >= MAX_COURSES {
            return Err(AppError::new(ErrorCode::BudgetExceeded));
        }
        let tx = state.connection.transaction()?;
        tx.execute(
            "INSERT INTO courses(id,title,legacy,opened) VALUES(?1,?2,0,0)",
            params![operation_id.to_string(), title],
        )?;
        tx.execute(
            "UPDATE catalogue_meta SET selected=?1 WHERE singleton=1",
            [operation_id.to_string()],
        )?;
        tx.commit()?;
        catalogue(&state.connection)
    }

    pub fn select(&self, course_id: Uuid) -> Result<CourseCatalogue> {
        let state = self.state.lock().map_err(|_| unavailable())?;
        if state.closing {
            return Err(unavailable());
        }
        let current = catalogue(&state.connection)?;
        if !current.courses.iter().any(|course| course.id == course_id) {
            return Err(AppError::new(ErrorCode::InvalidReference));
        }
        state.connection.execute(
            "UPDATE catalogue_meta SET selected=?1 WHERE singleton=1",
            [course_id.to_string()],
        )?;
        catalogue(&state.connection)
    }

    /// Captured before awaiting: selection changes never retarget an accepted request.
    pub fn service(&self, course_id: Uuid) -> Result<ApplicationService> {
        let mut state = self.state.lock().map_err(|_| unavailable())?;
        if state.closing {
            return Err(unavailable());
        }
        if let Some(service) = state.services.get(&course_id) {
            return Ok(service.clone());
        }
        if !catalogue(&state.connection)?
            .courses
            .iter()
            .any(|course| course.id == course_id)
        {
            return Err(AppError::new(ErrorCode::InvalidReference));
        }
        let (legacy, opened): (bool, bool) = state.connection.query_row(
            "SELECT legacy,opened FROM courses WHERE id=?1",
            [course_id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        let root = if legacy {
            self.app_data.join("workspace")
        } else {
            let data = self.app_data.join("courses/data");
            owner::private_directory(&data)?;
            data.join(course_id.to_string())
        };
        // A known initialized course must never silently acquire a replacement identity.
        // Mark before first initialization: interruption requires explicit recovery.
        if opened && !root.join("memoquill.sqlite3").is_file() {
            return Err(AppError::new(ErrorCode::RecoveryRequired));
        }
        if !opened && root.try_exists()? {
            // Never adopt an orphan root merely because a caller reused its UUID.
            return Err(AppError::new(ErrorCode::RecoveryRequired));
        }
        if !opened {
            state.connection.execute(
                "UPDATE courses SET opened=1 WHERE id=?1",
                [course_id.to_string()],
            )?;
        }
        let service = ApplicationService::start(root)?;
        state.services.insert(course_id, service.clone());
        Ok(service)
    }

    pub async fn shutdown(&self) -> Result<()> {
        let services = {
            let mut state = self.state.lock().map_err(|_| unavailable())?;
            state.closing = true;
            state.services.values().cloned().collect::<Vec<_>>()
        };
        let mut result = Ok(());
        for service in services {
            if let Err(error) = service.shutdown().await {
                result = Err(error);
            }
        }
        if result.is_ok() {
            // Tauri may retain the stopped managed state. Only a confirmed complete
            // drain releases catalogue ownership; all subsequent calls stay closed.
            self.state.lock().map_err(|_| unavailable())?.owner.take();
        }
        result
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct RestoreIntent {
    operation_id: Uuid,
    identity: String,
    title: String,
}

fn unavailable() -> AppError {
    AppError::new(ErrorCode::OwnerUnavailable)
}
fn validate_id(id: Uuid) -> Result<()> {
    if id.get_version_num() != 4 {
        return Err(AppError::new(ErrorCode::InvalidRequest));
    }
    Ok(())
}
fn validate_title(title: &str) -> Result<()> {
    if title.trim().is_empty() || title.chars().count() > 256 || title.chars().any(char::is_control)
    {
        return Err(AppError::new(ErrorCode::InvalidRequest));
    }
    Ok(())
}

fn initialize(connection: &mut Connection, legacy: bool, has_managed_data: bool) -> Result<()> {
    connection.pragma_update(None, "foreign_keys", true)?;
    let version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if version > 1 {
        return Err(AppError::new(ErrorCode::UnsupportedSchema));
    }
    if version == 1 {
        return Ok(());
    }
    let tables: i64 = connection.query_row(
        "SELECT count(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
        [],
        |row| row.get(0),
    )?;
    if version != 0 || tables != 0 || has_managed_data {
        return Err(AppError::new(ErrorCode::RecoveryRequired));
    }
    let tx = connection.transaction()?;
    tx.execute_batch("CREATE TABLE courses(id TEXT PRIMARY KEY, title TEXT NOT NULL, legacy INTEGER NOT NULL CHECK(legacy IN (0,1)), opened INTEGER NOT NULL CHECK(opened IN (0,1)));
        CREATE UNIQUE INDEX one_legacy ON courses(legacy) WHERE legacy=1;
        CREATE TABLE catalogue_meta(singleton INTEGER PRIMARY KEY CHECK(singleton=1),selected TEXT NOT NULL REFERENCES courses(id));")?;
    let id = Uuid::new_v4().to_string();
    tx.execute(
        "INSERT INTO courses VALUES(?1,?2,1,?3)",
        params![
            id,
            if legacy {
                "Imported workspace"
            } else {
                "My course"
            },
            legacy
        ],
    )?;
    tx.execute("INSERT INTO catalogue_meta VALUES(1,?1)", [id])?;
    tx.pragma_update(None, "user_version", 1)?;
    tx.commit()?;
    Ok(())
}

fn catalogue(connection: &Connection) -> Result<CourseCatalogue> {
    let read = || -> Result<CourseCatalogue> {
        let legacy: i64 =
            connection.query_row("SELECT count(*) FROM courses WHERE legacy=1", [], |row| {
                row.get(0)
            })?;
        let invalid: i64 = connection.query_row(
            "SELECT count(*) FROM courses WHERE legacy NOT IN (0,1) OR opened NOT IN (0,1)",
            [],
            |row| row.get(0),
        )?;
        let metadata: i64 =
            connection.query_row("SELECT count(*) FROM catalogue_meta", [], |row| row.get(0))?;
        if legacy != 1 || invalid != 0 || metadata != 1 {
            return Err(AppError::new(ErrorCode::RecoveryRequired));
        }
        let mut query =
            connection.prepare("SELECT id,title FROM courses ORDER BY rowid LIMIT 33")?;
        let raw = query
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        if raw.is_empty() || raw.len() > MAX_COURSES {
            return Err(AppError::new(ErrorCode::RecoveryRequired));
        }
        let mut courses = Vec::with_capacity(raw.len());
        for (id, title) in raw {
            let parsed =
                Uuid::parse_str(&id).map_err(|_| AppError::new(ErrorCode::RecoveryRequired))?;
            validate_id(parsed)?;
            validate_title(&title)?;
            if parsed.to_string() != id {
                return Err(AppError::new(ErrorCode::RecoveryRequired));
            }
            courses.push(Course { id: parsed, title });
        }
        let selected: String = connection.query_row(
            "SELECT selected FROM catalogue_meta WHERE singleton=1",
            [],
            |row| row.get(0),
        )?;
        let selected_course_id =
            Uuid::parse_str(&selected).map_err(|_| AppError::new(ErrorCode::RecoveryRequired))?;
        if !courses.iter().any(|course| course.id == selected_course_id) {
            return Err(AppError::new(ErrorCode::RecoveryRequired));
        }
        Ok(CourseCatalogue {
            contract_version: 1,
            selected_course_id,
            courses,
        })
    };
    read().map_err(|_| AppError::new(ErrorCode::RecoveryRequired))
}

#[cfg(test)]
mod tests;
