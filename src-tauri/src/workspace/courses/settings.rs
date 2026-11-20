use super::*;
use crate::workspace::{RequestContext, SettingsLayer};
use rusqlite::OptionalExtension;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkspaceSettings {
    pub revision: i64,
    pub settings: SettingsLayer,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveWorkspaceSettings {
    pub operation_id: Uuid,
    pub expected_revision: i64,
    pub settings: SettingsLayer,
}
fn read(connection: &Connection) -> Result<WorkspaceSettings> {
    let (revision, data): (i64, String) = connection.query_row(
        "SELECT revision,data FROM workspace_settings WHERE singleton=1",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    decode(revision, &data)
}
fn decode(revision: i64, data: &str) -> Result<WorkspaceSettings> {
    if !(0..=super::super::types::MAX_REVISION).contains(&revision) || data.len() > 32768 {
        return Err(AppError::new(ErrorCode::RecoveryRequired));
    }
    let settings: SettingsLayer =
        serde_json::from_str(data).map_err(|_| AppError::new(ErrorCode::RecoveryRequired))?;
    settings
        .validate()
        .map_err(|_| AppError::new(ErrorCode::RecoveryRequired))?;
    Ok(WorkspaceSettings { revision, settings })
}
impl CourseManager {
    pub fn workspace_settings(&self, context: RequestContext) -> Result<WorkspaceSettings> {
        context.authorize()?;
        let state = self.state.lock().map_err(|_| unavailable())?;
        if state.closing {
            return Err(unavailable());
        }
        read(&state.connection)
    }
    pub fn save_workspace_settings(
        &self,
        context: RequestContext,
        request: SaveWorkspaceSettings,
    ) -> Result<WorkspaceSettings> {
        context.authorize()?;
        validate_id(request.operation_id)?;
        request.settings.validate()?;
        if !(0..super::super::types::MAX_REVISION).contains(&request.expected_revision) {
            return Err(AppError::new(ErrorCode::InvalidRequest));
        }
        let hash = super::super::operations::request_hash("workspace-settings", &request)?;
        let mut state = self.state.lock().map_err(|_| unavailable())?;
        if state.closing {
            return Err(unavailable());
        }
        let tx = state.connection.transaction()?;
        let known: Option<(String, String)> = tx
            .query_row(
                "SELECT hash,result FROM settings_receipts WHERE id=?1",
                [request.operation_id.to_string()],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        if let Some((old_hash, result)) = known {
            if old_hash != hash {
                return Err(AppError::new(ErrorCode::IdempotencyConflict));
            }
            let result: WorkspaceSettings = serde_json::from_str(&result)
                .map_err(|_| AppError::new(ErrorCode::RecoveryRequired))?;
            return decode(
                result.revision,
                &serde_json::to_string(&result.settings)
                    .map_err(|_| AppError::new(ErrorCode::RecoveryRequired))?,
            );
        }
        let current = read(&tx)?;
        if current.revision != request.expected_revision {
            return Err(AppError::new(ErrorCode::RevisionConflict));
        }
        let result = WorkspaceSettings {
            revision: current.revision + 1,
            settings: request.settings,
        };
        tx.execute(
            "UPDATE workspace_settings SET revision=?1,data=?2 WHERE singleton=1",
            params![
                result.revision,
                super::super::operations::json(&result.settings)?
            ],
        )?;
        tx.execute(
            "INSERT INTO settings_receipts VALUES(?1,?2,?3)",
            params![
                request.operation_id.to_string(),
                hash,
                super::super::operations::json(&result)?
            ],
        )?;
        tx.commit()?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn m5_settings_migrates_v1_without_rewriting_courses_and_preserves_retry_across_restart() {
        let temp = tempfile::tempdir().unwrap();
        let manager = CourseManager::open(temp.path().to_owned()).unwrap();
        let original = manager.list().unwrap();
        drop(manager);
        let path = temp.path().join("courses/catalogue.sqlite3");
        let connection = Connection::open(&path).unwrap();
        connection.execute_batch("DROP TABLE workspace_settings; DROP TABLE settings_receipts; PRAGMA user_version=1;").unwrap();
        drop(connection);
        let manager = CourseManager::open(temp.path().to_owned()).unwrap();
        assert_eq!(manager.list().unwrap(), original);
        let initial = manager
            .workspace_settings(RequestContext::local_ui())
            .unwrap();
        assert_eq!(
            initial,
            WorkspaceSettings {
                revision: 0,
                settings: SettingsLayer::default()
            }
        );
        let mut settings = SettingsLayer::default();
        settings.overrides.goal = Some("Explain a synthetic concept without help".into());
        let request = SaveWorkspaceSettings {
            operation_id: Uuid::new_v4(),
            expected_revision: 0,
            settings,
        };
        let saved = manager
            .save_workspace_settings(RequestContext::local_ui(), request.clone())
            .unwrap();
        assert_eq!(saved.revision, 1);
        assert_eq!(
            manager
                .save_workspace_settings(RequestContext::local_ui(), request.clone())
                .unwrap(),
            saved
        );
        let mut conflict = request.clone();
        conflict.settings.overrides.focus = Some("Different".into());
        assert_eq!(
            manager
                .save_workspace_settings(RequestContext::local_ui(), conflict)
                .unwrap_err()
                .code,
            ErrorCode::IdempotencyConflict
        );
        let stale = SaveWorkspaceSettings {
            operation_id: Uuid::new_v4(),
            ..request.clone()
        };
        assert_eq!(
            manager
                .save_workspace_settings(RequestContext::local_ui(), stale)
                .unwrap_err()
                .code,
            ErrorCode::RevisionConflict
        );
        drop(manager);
        let manager = CourseManager::open(temp.path().to_owned()).unwrap();
        assert_eq!(
            manager
                .workspace_settings(RequestContext::local_ui())
                .unwrap(),
            saved
        );
        assert_eq!(
            manager
                .save_workspace_settings(RequestContext::local_ui(), request)
                .unwrap(),
            saved
        );
        assert_eq!(manager.list().unwrap(), original);
    }
    #[test]
    fn m5_settings_denies_external_invalid_and_corrupt_values_without_reset() {
        let temp = tempfile::tempdir().unwrap();
        let manager = CourseManager::open(temp.path().to_owned()).unwrap();
        assert_eq!(
            manager
                .workspace_settings(RequestContext::unverified_external())
                .unwrap_err()
                .code,
            ErrorCode::PermissionDenied
        );
        let request = SaveWorkspaceSettings {
            operation_id: Uuid::new_v4(),
            expected_revision: 0,
            settings: SettingsLayer::default(),
        };
        assert_eq!(
            manager
                .save_workspace_settings(RequestContext::unverified_external(), request.clone())
                .unwrap_err()
                .code,
            ErrorCode::PermissionDenied
        );
        for settings in [
            SettingsLayer {
                contract_version: 2,
                ..SettingsLayer::default()
            },
            SettingsLayer {
                overrides: crate::workspace::SettingsOverrides {
                    goal: Some("x".repeat(2049)),
                    ..Default::default()
                },
                ..Default::default()
            },
        ] {
            assert_eq!(
                manager
                    .save_workspace_settings(
                        RequestContext::local_ui(),
                        SaveWorkspaceSettings {
                            settings,
                            ..request.clone()
                        }
                    )
                    .unwrap_err()
                    .code,
                ErrorCode::InvalidRequest
            );
        }
        manager
            .state
            .lock()
            .unwrap()
            .connection
            .execute(
                "UPDATE workspace_settings SET data=?1",
                ["{\"contractVersion\":999,\"overrides\":{}}"],
            )
            .unwrap();
        assert_eq!(
            manager
                .workspace_settings(RequestContext::local_ui())
                .unwrap_err()
                .code,
            ErrorCode::RecoveryRequired
        );
        assert_eq!(
            manager
                .save_workspace_settings(RequestContext::local_ui(), request)
                .unwrap_err()
                .code,
            ErrorCode::RecoveryRequired
        );
        let value: String = manager
            .state
            .lock()
            .unwrap()
            .connection
            .query_row("SELECT data FROM workspace_settings", [], |r| r.get(0))
            .unwrap();
        assert!(value.contains("999"));
    }
}
