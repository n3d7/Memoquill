//! Bounded semantic settings; no learning evidence or navigation is stored here.
use super::{AppError, ErrorCode, TextScale, Theme, error::Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SettingsOverrides {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<Theme>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_scale: Option<TextScale>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SettingsLayer {
    pub contract_version: u8,
    pub overrides: SettingsOverrides,
}
impl Default for SettingsLayer {
    fn default() -> Self {
        Self {
            contract_version: 1,
            overrides: SettingsOverrides::default(),
        }
    }
}
impl SettingsLayer {
    pub(super) fn validate(&self) -> Result<()> {
        if self.contract_version != 1
            || [&self.overrides.goal, &self.overrides.focus]
                .iter()
                .any(|value| {
                    value.as_ref().is_some_and(|text| {
                        text.chars().count() > 2048
                            || text
                                .chars()
                                .any(|c| c.is_control() && c != '\n' && c != '\t')
                    })
                })
        {
            return Err(AppError::new(ErrorCode::InvalidRequest));
        }
        Ok(())
    }
}

pub(super) fn deserialize_settings<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<Box<SettingsLayer>>, D::Error> {
    let value = Option::<Box<SettingsLayer>>::deserialize(d)?;
    if let Some(layer) = &value {
        layer.validate().map_err(serde::de::Error::custom)?;
    }
    Ok(value)
}

#[path = "customization.rs"]
mod customization;
pub use customization::ShortcutPreferences;
pub(super) use customization::{
    deserialize_shortcuts, deserialize_subject_settings, validate_subject_settings,
};
