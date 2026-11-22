use super::{
    ApplyContent, CourseManager, NewSource, OperationId, RegisterAsset, RequestContext, SaveSource,
};
use crate::organization::{Command, ResourceId};
use std::{fs, io::Cursor, path::Path};
use uuid::Uuid;

#[test]
fn m6_pdf_journey_seed() {
    let temporary = tempfile::tempdir().unwrap();
    let root = if std::env::var_os("MEMOQUILL_EXPORT_M6_FIXTURE").as_deref()
        == Some(std::ffi::OsStr::new("1"))
    {
        let checkout = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let root = checkout.join(".qualification/m6-seed");
        fs::create_dir(&root).expect("Create a fresh synthetic fixture directory");
        root
    } else {
        temporary.path().to_owned()
    };
    let manager = CourseManager::open(root.join("io.github.n3d7.memoquill")).unwrap();
    let default_course = manager.list().unwrap().selected_course_id;
    let fixture_course = Uuid::new_v4();
    manager
        .create(fixture_course, "Synthetic source course".into())
        .unwrap();
    manager.select(default_course).unwrap();
    let service = manager.service(fixture_course).unwrap();
    let resource = ResourceId::generate();
    let source = Uuid::new_v4();
    let pdf = include_bytes!("../../../tests/fixtures/m3/original.pdf");

    tauri::async_runtime::block_on(async {
        let context = RequestContext::local_ui();
        let workspace = service
            .status(context)
            .await
            .unwrap()
            .preferences
            .workspace_id;
        service
            .apply_content(
                context,
                ApplyContent {
                    contract_version: 1,
                    workspace_id: workspace.clone(),
                    operation_id: OperationId::generate(),
                    expected_revision: 0,
                    command: Command::CreateResource {
                        id: resource,
                        title: "Shared observation text".into(),
                    },
                },
            )
            .await
            .unwrap();
        let asset = service
            .register_asset(
                context,
                RegisterAsset {
                    contract_version: 1,
                    workspace_id: workspace.clone(),
                    operation_id: OperationId::generate(),
                },
                Box::new(Cursor::new(pdf.as_slice())),
                pdf.len() as u64,
            )
            .await
            .unwrap();
        service
            .save_source(
                context,
                SaveSource {
                    contract_version: 1,
                    workspace_id: workspace,
                    operation_id: OperationId::generate(),
                    expected_revision: 0,
                    source: NewSource {
                        id: source,
                        resource_id: resource,
                        digest: asset.digest,
                        filename: "original.pdf".into(),
                        media_type: "application/pdf".into(),
                        attribution: Some("Synthetic qualification original".into()),
                        license: Some("Original project test fixture".into()),
                        origin_url: None,
                    },
                },
            )
            .await
            .unwrap();
        let (name, bytes) = service.original_copy(context, source).await.unwrap();
        assert_eq!(name, "original.pdf");
        assert_eq!(bytes, pdf);
        manager.shutdown().await.unwrap();
    });
}
