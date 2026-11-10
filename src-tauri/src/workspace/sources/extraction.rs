//! Pure passive projections of bounded original bytes; never applies content.
use super::*;

const INPUT_BYTES: usize = 65_536;
const RECORDS: usize = 128;
const FIELDS: usize = 64;

pub(super) fn extract(filename: &str, bytes: &[u8], truncated: bool) -> SourceExtraction {
    extract_bounded(filename, bytes, truncated, INPUT_BYTES, RECORDS)
}

pub(super) fn extract_for_ingestion_legacy(filename: &str, bytes: &[u8]) -> SourceExtraction {
    let mut result = extract_bounded(filename, bytes, false, 8 * 1024 * 1024, 10_000);
    if result.status != ExtractionStatus::Complete {
        result.records.clear();
    }
    result
}

/// Version 2 is restricted to JSON jobs; preview and handler-1 jobs stay raw-only.
pub(super) fn extract_for_ingestion(filename: &str, bytes: &[u8]) -> SourceExtraction {
    let mut result = extract_for_ingestion_legacy(filename, bytes);
    if result.format != ExtractionFormat::Json {
        return result;
    }
    result.contract_version = 2;
    if result.status != ExtractionStatus::Complete {
        return result;
    }
    let mut names = std::collections::BTreeSet::new();
    for record in &mut result.records {
        let raw = &record.fields[0];
        if !raw.trim_start().starts_with('{') {
            continue;
        }
        // Parse entries before building a map, so escaped duplicate names cannot
        // be silently overwritten. The legacy parser already validated depth.
        let entries: JsonProperties = match serde_json::from_str(raw) {
            Ok(value) => value,
            Err(_) => {
                result.records.clear();
                return result
                    .with_diagnostic(ExtractionStatus::Failed, ExtractionDiagnostic::FieldLimit);
            }
        };
        for (name, value) in entries.0 {
            names.insert(name.clone());
            if names.len() > FIELDS {
                result.records.clear();
                return result
                    .with_diagnostic(ExtractionStatus::Failed, ExtractionDiagnostic::FieldLimit);
            }
            let value = if value.get().starts_with('"') {
                // The raw value has been validated; decode only string scalars.
                serde_json::from_str::<String>(value.get()).expect("validated JSON string")
            } else {
                value.get().to_owned()
            };
            if record.properties.insert(name.clone(), value).is_some() {
                record.duplicate_properties.push(name);
            }
        }
        record.duplicate_properties.sort();
        record.duplicate_properties.dedup();
        if !record.duplicate_properties.is_empty() {
            // No collapsed projection may be mistaken for a resolved record.
            record.properties.clear();
        }
    }
    result
}

struct JsonProperties(Vec<(String, Box<serde_json::value::RawValue>)>);
impl<'de> serde::Deserialize<'de> for JsonProperties {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        struct PropertiesVisitor;
        impl<'de> serde::de::Visitor<'de> for PropertiesVisitor {
            type Value = JsonProperties;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a bounded JSON object")
            }
            fn visit_map<M: serde::de::MapAccess<'de>>(
                self,
                mut map: M,
            ) -> std::result::Result<Self::Value, M::Error> {
                let mut entries = Vec::new();
                while let Some(entry) = map.next_entry()? {
                    if entries.len() == FIELDS {
                        return Err(serde::de::Error::custom("JSON property limit"));
                    }
                    entries.push(entry);
                }
                Ok(JsonProperties(entries))
            }
        }
        deserializer.deserialize_map(PropertiesVisitor)
    }
}

fn extract_bounded(
    filename: &str,
    bytes: &[u8],
    truncated: bool,
    input_limit: usize,
    record_limit: usize,
) -> SourceExtraction {
    let format = match filename
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "txt" => ExtractionFormat::Text,
        "md" | "markdown" => ExtractionFormat::Markdown,
        "csv" => ExtractionFormat::Csv,
        "json" => ExtractionFormat::Json,
        _ => ExtractionFormat::Unsupported,
    };
    let mut result = SourceExtraction {
        contract_version: 1,
        format,
        status: ExtractionStatus::Complete,
        records: Vec::new(),
        diagnostic: None,
    };
    if format == ExtractionFormat::Unsupported {
        return result.with_diagnostic(
            ExtractionStatus::AttachmentOnly,
            ExtractionDiagnostic::UnsupportedFormat,
        );
    }
    if bytes.len() > input_limit
        || (truncated && matches!(format, ExtractionFormat::Csv | ExtractionFormat::Json))
    {
        return result.with_diagnostic(ExtractionStatus::Partial, ExtractionDiagnostic::InputLimit);
    }
    let text = match std::str::from_utf8(bytes) {
        Ok(text) => text,
        Err(error) if truncated && error.error_len().is_none() => {
            // An incomplete final scalar is outside the available source prefix.
            std::str::from_utf8(&bytes[..error.valid_up_to()]).unwrap_or("")
        }
        Err(_) => {
            return result
                .with_diagnostic(ExtractionStatus::Failed, ExtractionDiagnostic::InvalidUtf8);
        }
    };
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let parsed = match format {
        ExtractionFormat::Text | ExtractionFormat::Markdown => {
            for (index, line) in text.lines().take(record_limit + 1).enumerate() {
                if !result.push(
                    LocatorKind::Line,
                    (index + 1).to_string(),
                    vec![line.into()],
                    record_limit,
                ) {
                    break;
                }
            }
            Ok(())
        }
        ExtractionFormat::Csv => csv_records(text, &mut result, record_limit),
        ExtractionFormat::Json => json_records(text, &mut result, record_limit),
        ExtractionFormat::Unsupported => unreachable!(),
    };
    if let Err(diagnostic) = parsed {
        result.records.clear();
        return result.with_diagnostic(ExtractionStatus::Failed, diagnostic);
    }
    if truncated {
        result.status = ExtractionStatus::Partial;
        result.diagnostic = Some(ExtractionDiagnostic::InputLimit);
    }
    result
}

impl SourceExtraction {
    fn with_diagnostic(
        mut self,
        status: ExtractionStatus,
        diagnostic: ExtractionDiagnostic,
    ) -> Self {
        self.status = status;
        self.diagnostic = Some(diagnostic);
        self
    }
    fn push(
        &mut self,
        kind: LocatorKind,
        value: String,
        fields: Vec<String>,
        record_limit: usize,
    ) -> bool {
        if self.records.len() == record_limit {
            self.status = ExtractionStatus::Partial;
            self.diagnostic = Some(ExtractionDiagnostic::RecordLimit);
            return false;
        }
        self.records.push(ExtractionRecord {
            locator: ExtractionLocator { kind, value },
            fields,
            properties: Default::default(),
            duplicate_properties: Vec::new(),
        });
        true
    }
}

fn csv_records(
    text: &str,
    result: &mut SourceExtraction,
    record_limit: usize,
) -> std::result::Result<(), ExtractionDiagnostic> {
    validate_csv_quotes(text)?;
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(text.as_bytes());
    for (index, record) in reader.records().take(record_limit + 1).enumerate() {
        let record = record.map_err(|_| ExtractionDiagnostic::MalformedInput)?;
        if record.len() > FIELDS {
            return Err(ExtractionDiagnostic::FieldLimit);
        }
        if !result.push(
            LocatorKind::CsvRecord,
            (index + 1).to_string(),
            record.iter().map(str::to_owned).collect(),
            record_limit,
        ) {
            break;
        }
    }
    Ok(())
}

fn validate_csv_quotes(text: &str) -> std::result::Result<(), ExtractionDiagnostic> {
    // csv intentionally recovers some malformed quoting. Reject it before
    // previewing records so recovery cannot masquerade as complete extraction.
    enum State {
        Start,
        Bare,
        Quoted,
        Closed,
    }
    let mut state = State::Start;
    for byte in text.bytes() {
        state = match (state, byte) {
            (State::Start, b'"') | (State::Closed, b'"') => State::Quoted,
            (State::Quoted, b'"') => State::Closed,
            (State::Quoted, _) => State::Quoted,
            (State::Start | State::Bare | State::Closed, b',' | b'\r' | b'\n') => State::Start,
            (State::Start | State::Bare, value) if value != b'"' => State::Bare,
            _ => return Err(ExtractionDiagnostic::MalformedInput),
        };
    }
    if matches!(state, State::Quoted) {
        return Err(ExtractionDiagnostic::MalformedInput);
    }
    Ok(())
}

fn json_records(
    text: &str,
    result: &mut SourceExtraction,
    record_limit: usize,
) -> std::result::Result<(), ExtractionDiagnostic> {
    // Validate with the standard recursion limit before retaining raw slices.
    // RawValue preserves duplicate keys and exact number/string spellings.
    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|_| ExtractionDiagnostic::MalformedInput)?;
    if value.is_array() {
        let values: Vec<Box<serde_json::value::RawValue>> =
            serde_json::from_str(text).map_err(|_| ExtractionDiagnostic::MalformedInput)?;
        for (index, raw) in values.iter().take(record_limit + 1).enumerate() {
            if !result.push(
                LocatorKind::JsonPointer,
                format!("/{index}"),
                vec![raw.get().into()],
                record_limit,
            ) {
                break;
            }
        }
    } else {
        result.push(
            LocatorKind::JsonPointer,
            String::new(),
            vec![text.trim().into()],
            record_limit,
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests;
