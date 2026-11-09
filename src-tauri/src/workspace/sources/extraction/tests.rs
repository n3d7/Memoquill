use super::*;

#[test]
fn ingestion_reads_complete_inputs_beyond_preview_limits_and_never_returns_a_prefix_as_complete() {
    let input = "value\n".repeat(200);
    let result = extract_for_ingestion("rows.csv", input.as_bytes());
    assert_eq!(result.status, ExtractionStatus::Complete);
    assert_eq!(result.records.len(), 200);
    assert_eq!(result.records[199].locator.value, "200");

    let too_many = "x\n".repeat(10_001);
    let result = extract_for_ingestion("rows.txt", too_many.as_bytes());
    assert_eq!(result.status, ExtractionStatus::Partial);
    assert_eq!(result.diagnostic, Some(ExtractionDiagnostic::RecordLimit));
    assert!(result.records.is_empty());
}

#[test]
fn text_and_markdown_keep_unicode_lines_and_passive_markup() {
    for filename in ["NOTES.TXT", "notes.md"] {
        let result = extract(
            filename,
            "\u{feff}π\r\n<script>x()</script>\n".as_bytes(),
            false,
        );
        assert_eq!(result.status, ExtractionStatus::Complete);
        assert_eq!(result.records.len(), 2);
        assert_eq!(result.records[0].fields, ["π"]);
        assert_eq!(result.records[1].fields, ["<script>x()</script>"]);
        assert_eq!(result.records[1].locator.value, "2");
        assert_eq!(result.records[0].locator.kind, LocatorKind::Line);
    }
}

#[test]
fn csv_keeps_first_row_empty_fields_quoted_commas_and_multiline_records() {
    let result = extract(
        "data.csv",
        b"name,value,empty\r\n\"a,b\",\"x\ny\",\r\n",
        false,
    );
    assert_eq!(result.status, ExtractionStatus::Complete);
    assert_eq!(result.records[0].fields, ["name", "value", "empty"]);
    assert_eq!(result.records[1].fields, ["a,b", "x\ny", ""]);
    assert_eq!(result.records[1].locator.kind, LocatorKind::CsvRecord);
    assert_eq!(result.records[1].locator.value, "2");
    let escaped = extract("data.csv", b"\"say \"\"hi\"\"\",=1+2", false);
    assert_eq!(escaped.records[0].fields, ["say \"hi\"", "=1+2"]);
    assert_eq!(
        extract("bad.csv", b"a,b\nc", false).status,
        ExtractionStatus::Failed
    );
}

#[test]
fn json_keeps_raw_duplicate_keys_and_top_level_pointers() {
    let result = extract("data.json", br#"[{"a":1,"a":2},"<img src=x>",null]"#, false);
    assert_eq!(result.status, ExtractionStatus::Complete);
    assert_eq!(result.records[0].fields, [r#"{"a":1,"a":2}"#]);
    assert_eq!(result.records[1].locator.value, "/1");
    assert_eq!(result.records[1].locator.kind, LocatorKind::JsonPointer);
    assert_eq!(result.records[2].fields, ["null"]);
    let root = extract("root.json", br#"{ "a/b": [1] }"#, false);
    assert_eq!(root.records[0].locator.value, "");
    assert_eq!(root.records[0].fields, [r#"{ "a/b": [1] }"#]);
}

#[test]
fn invalid_json_encoding_and_unsupported_formats_have_honest_outcomes() {
    for input in [b"[1,".as_slice(), b"{\"a\":}".as_slice()] {
        let result = extract("bad.json", input, false);
        assert_eq!(result.status, ExtractionStatus::Failed);
        assert_eq!(
            result.diagnostic,
            Some(ExtractionDiagnostic::MalformedInput)
        );
        assert!(result.records.is_empty());
    }
    let deep = format!("{}0{}", "[".repeat(150), "]".repeat(150));
    assert_eq!(
        extract("deep.json", deep.as_bytes(), false).status,
        ExtractionStatus::Failed
    );
    for name in ["bad.txt", "bad.csv", "bad.json"] {
        let result = extract(name, &[0xff, 0], false);
        assert_eq!(result.diagnostic, Some(ExtractionDiagnostic::InvalidUtf8));
        assert_eq!(result.status, ExtractionStatus::Failed);
    }
    for name in [
        "book.pdf",
        "book.epub",
        "file.docx",
        "file.html",
        "file.bin",
    ] {
        let result = extract(name, b"<script>never execute</script>", false);
        assert_eq!(result.status, ExtractionStatus::AttachmentOnly);
        assert_eq!(
            result.diagnostic,
            Some(ExtractionDiagnostic::UnsupportedFormat)
        );
        assert!(result.records.is_empty());
    }
}

#[test]
fn structured_prefixes_never_claim_whole_file_success() {
    for name in ["data.csv", "data.json"] {
        let result = extract(name, b"[]", true);
        assert_eq!(result.status, ExtractionStatus::Partial);
        assert_eq!(result.diagnostic, Some(ExtractionDiagnostic::InputLimit));
        assert!(result.records.is_empty());
    }
    let result = extract("text.txt", b"ok\n\xf0\x9f", true);
    assert_eq!(result.status, ExtractionStatus::Partial);
    assert_eq!(result.records[0].fields, ["ok"]);
    assert_eq!(
        extract("text.txt", &vec![b'x'; 65_537], false).diagnostic,
        Some(ExtractionDiagnostic::InputLimit)
    );
}

#[test]
fn record_and_field_limits_preserve_honest_coverage() {
    let exact = "x\n".repeat(128);
    assert_eq!(
        extract("a.txt", exact.as_bytes(), false).status,
        ExtractionStatus::Complete
    );
    let over = format!("{exact}last");
    for (name, data) in [("a.txt", over.clone()), ("a.csv", over)] {
        let result = extract(name, data.as_bytes(), false);
        assert_eq!(result.records.len(), 128);
        assert_eq!(result.status, ExtractionStatus::Partial);
        assert_eq!(result.diagnostic, Some(ExtractionDiagnostic::RecordLimit));
    }
    let array = format!("[{}]", vec!["0"; 129].join(","));
    assert_eq!(
        extract("a.json", array.as_bytes(), false).records.len(),
        128
    );
    assert_eq!(
        extract("a.json", array.as_bytes(), false).diagnostic,
        Some(ExtractionDiagnostic::RecordLimit)
    );
    let fields = vec!["x"; 65].join(",");
    let result = extract("a.csv", fields.as_bytes(), false);
    assert_eq!(result.status, ExtractionStatus::Failed);
    assert_eq!(result.diagnostic, Some(ExtractionDiagnostic::FieldLimit));
    assert!(result.records.is_empty());
    assert_eq!(
        extract("a.csv", vec!["x"; 64].join(",").as_bytes(), false).status,
        ExtractionStatus::Complete
    );
}

#[test]
fn empty_files_have_no_invented_records_and_json_empty_is_invalid() {
    for name in ["a.txt", "a.md", "a.csv"] {
        let result = extract(name, b"", false);
        assert_eq!(result.status, ExtractionStatus::Complete);
        assert!(result.records.is_empty());
    }
    assert_eq!(
        extract("a.json", b"", false).status,
        ExtractionStatus::Failed
    );
    assert!(extract("a.json", b"[]", false).records.is_empty());
}

#[test]
fn malformed_csv_quotes_are_not_silently_recovered_as_complete_records() {
    for bytes in [
        b"a,\"unterminated".as_slice(),
        b"a,\"closed\"tail".as_slice(),
        b"a,b\"c".as_slice(),
    ] {
        let result = extract("bad.csv", bytes, false);
        assert_eq!(result.status, ExtractionStatus::Failed);
        assert_eq!(
            result.diagnostic,
            Some(ExtractionDiagnostic::MalformedInput)
        );
        assert!(result.records.is_empty());
    }
}
