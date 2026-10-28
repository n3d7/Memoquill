fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "check_draft",
            "workspace_status",
            "save_reading_preferences",
            "operation_receipt",
            "reopen_workspace",
        ]),
    ))
    .expect("Tauri build configuration must be valid");
}
