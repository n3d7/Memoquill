//! Private, bounded customization; independent of learning evidence and study policy.
use crate::workspace::{AppError, ErrorCode, SettingsLayer, error::Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShortcutBindings {
    pub open_settings: Option<String>,
    pub focus_search: Option<String>,
    pub focus_workspace: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShortcutPreferences {
    pub contract_version: u8,
    pub bindings: ShortcutBindings,
}
impl ShortcutPreferences {
    pub(in crate::workspace) fn validate(&self) -> Result<()> {
        let mut used = std::collections::HashSet::new();
        if self.contract_version != 1 {
            return Err(AppError::new(ErrorCode::InvalidRequest));
        }
        for binding in [
            &self.bindings.open_settings,
            &self.bindings.focus_search,
            &self.bindings.focus_workspace,
        ]
        .into_iter()
        .flatten()
        {
            let key = binding.strip_prefix("Ctrl+Alt+Shift+").unwrap_or("");
            if key.len() != 1
                || !key.as_bytes()[0].is_ascii_uppercase()
                || key == "R"
                || !used.insert(binding)
            {
                return Err(AppError::new(ErrorCode::InvalidRequest));
            }
        }
        Ok(())
    }
}
pub type SubjectSettings = BTreeMap<String, SettingsLayer>;
pub(in crate::workspace) fn validate_subject_settings(value: &SubjectSettings) -> Result<()> {
    if value.len() > 128
        || value.keys().any(|key| {
            key.is_empty()
                || key.len() > 128
                || ["__proto__", "prototype", "constructor"].contains(&key.as_str())
                || !key
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
        })
    {
        return Err(AppError::new(ErrorCode::InvalidRequest));
    }
    for layer in value.values() {
        layer.validate()?;
    }
    if serde_json::to_vec(value)
        .map_err(|_| AppError::new(ErrorCode::InvalidRequest))?
        .len()
        > 262_144
    {
        return Err(AppError::new(ErrorCode::InvalidRequest));
    }
    Ok(())
}
pub(in crate::workspace) fn deserialize_subject_settings<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<Box<SubjectSettings>>, D::Error> {
    let value = Option::<Box<SubjectSettings>>::deserialize(d)?;
    if let Some(settings) = &value {
        validate_subject_settings(settings).map_err(serde::de::Error::custom)?;
    }
    Ok(value)
}
pub(in crate::workspace) fn deserialize_shortcuts<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<Box<ShortcutPreferences>>, D::Error> {
    let value = Option::<Box<ShortcutPreferences>>::deserialize(d)?;
    if let Some(settings) = &value {
        settings.validate().map_err(serde::de::Error::custom)?;
    }
    Ok(value)
}
