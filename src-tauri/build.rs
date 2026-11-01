fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "check_draft",
            "workspace_status",
            "save_reading_preferences",
            "operation_receipt",
            "reopen_workspace",
            "library",
            "apply_content",
            "save_document",
            "read_document",
            "save_annotation",
            "annotations",
            "save_reading_position",
            "reading_position",
            "pick_source",
            "sources",
            "save_source",
            "source_preview",
            "search_resources",
        ]),
    ))
    .expect("Tauri build configuration must be valid");
}
