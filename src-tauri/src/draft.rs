use serde::{Deserialize, Serialize};

/// Ephemeral qualification contract, not the future persistent content schema.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Draft {
    pub schema_version: u8,
    pub title: String,
    pub prose: String,
    pub formula: String,
    pub code: String,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DraftSummary {
    pub title: String,
    pub prose_characters: usize,
    pub code_lines: usize,
}

pub fn check_draft(draft: Draft) -> Result<DraftSummary, String> {
    if draft.schema_version != 1 {
        return Err("Unsupported draft schema.".into());
    }
    let title = draft.title.trim();
    if !(1..=128).contains(&title.chars().count()) {
        return Err("Title must contain 1–128 characters.".into());
    }
    for (name, value, limit) in [
        ("Title", draft.title.as_str(), 256),
        ("Notes", draft.prose.as_str(), 20_000),
        ("Formula", draft.formula.as_str(), 512),
        ("Code", draft.code.as_str(), 20_000),
    ] {
        validate_text(name, value, limit)?;
    }
    Ok(DraftSummary {
        title: title.into(),
        prose_characters: draft.prose.chars().count(),
        code_lines: draft.code.lines().count(),
    })
}

fn validate_text(name: &str, value: &str, limit: usize) -> Result<(), String> {
    if value.contains('\0') {
        return Err("Content cannot contain NUL characters.".into());
    }
    if value.chars().count() > limit {
        return Err(format!("{name} must not exceed {limit} characters."));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft() -> Draft {
        serde_json::from_str(include_str!("../../tests/fixtures/draft.json")).unwrap()
    }

    #[test]
    fn preserves_unicode_and_summarizes_without_claiming_to_save() {
        assert_eq!(
            check_draft(draft()).unwrap(),
            DraftSummary {
                title: "Quantum cooking".into(),
                prose_characters: 14,
                code_lines: 2,
            }
        );
    }

    #[test]
    fn rejects_unknown_schema() {
        let mut value = draft();
        value.schema_version = 2;
        assert_eq!(check_draft(value).unwrap_err(), "Unsupported draft schema.");
    }

    #[test]
    fn rejects_empty_titles_and_nul_characters() {
        let mut value = draft();
        value.title = "  ".into();
        assert_eq!(
            check_draft(value).unwrap_err(),
            "Title must contain 1–128 characters."
        );
        let mut value = draft();
        value.code = "print(\0)".into();
        assert_eq!(
            check_draft(value).unwrap_err(),
            "Content cannot contain NUL characters."
        );
    }

    #[test]
    fn rejects_oversized_fields() {
        for field in ["title", "prose", "formula", "code"] {
            let mut value = draft();
            let text = "a".repeat(20_001);
            match field {
                "title" => value.title = text,
                "prose" => value.prose = text,
                "formula" => value.formula = text,
                _ => value.code = text,
            }
            assert!(check_draft(value).is_err(), "{field}");
        }
    }

    #[test]
    fn accepts_exact_limits_using_unicode_scalars_not_utf8_bytes() {
        let mut value = draft();
        value.title = "é".repeat(128);
        value.prose = "界".repeat(20_000);
        value.formula = "x".repeat(512);
        value.code = "x".repeat(20_000);
        let result = check_draft(value).unwrap();
        assert_eq!(result.title.chars().count(), 128);
        assert_eq!(result.prose_characters, 20_000);
        assert_eq!(result.code_lines, 1);
    }

    #[test]
    fn accepts_empty_optional_fields_and_bounds_padding() {
        let mut value = draft();
        value.prose.clear();
        value.formula.clear();
        value.code.clear();
        let result = check_draft(value).unwrap();
        assert_eq!(result.prose_characters, 0);
        assert_eq!(result.code_lines, 0);
        let mut value = draft();
        value.title = format!("{}x", " ".repeat(256));
        assert!(check_draft(value).is_err());
    }

    #[test]
    fn input_contract_denies_extra_fields_and_wrong_types() {
        let mut value: serde_json::Value =
            serde_json::from_str(include_str!("../../tests/fixtures/draft.json")).unwrap();
        value["path"] = "/etc/passwd".into();
        assert!(serde_json::from_value::<Draft>(value.clone()).is_err());
        value.as_object_mut().unwrap().remove("path");
        value["schemaVersion"] = serde_json::json!("1");
        assert!(serde_json::from_value::<Draft>(value).is_err());
    }
}
