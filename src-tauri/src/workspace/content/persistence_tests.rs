use super::*;
use crate::{
    document::{Anchor, Block, BlockContent, Document},
    organization::{Command, ResourceId},
};

async fn resource(
    service: &ApplicationService,
    workspace: &WorkspaceId,
    expected_revision: u64,
) -> ResourceId {
    let id = ResourceId::generate();
    service
        .apply_content(
            RequestContext::local_ui(),
            ApplyContent {
                contract_version: 1,
                workspace_id: workspace.clone(),
                operation_id: OperationId::generate(),
                expected_revision,
                command: Command::CreateResource {
                    id,
                    title: "Resource".into(),
                },
            },
        )
        .await
        .unwrap();
    id
}
fn document() -> Document {
    Document {
        schema_version: 1,
        blocks: vec![Block {
            id: Uuid::new_v4(),
            content: BlockContent::Code {
                text: "a😀b".into(),
                language: None,
            },
        }],
    }
}
async fn saved(
    service: &ApplicationService,
    workspace: &WorkspaceId,
    resource_id: ResourceId,
) -> SavedDocument {
    service
        .save_document(
            RequestContext::local_ui(),
            SaveDocument {
                contract_version: 1,
                workspace_id: workspace.clone(),
                operation_id: OperationId::generate(),
                resource_id,
                expected_revision: 0,
                document: document(),
            },
        )
        .await
        .unwrap()
}
fn annotation(value: &SavedDocument) -> Annotation {
    Annotation {
        id: Uuid::new_v4(),
        resource_id: value.resource_id,
        revision_id: value.id,
        revision: 0,
        anchor: Anchor {
            resource_id: value.resource_id.as_uuid(),
            revision_id: value.id,
            block_id: value.document.blocks[0].id,
            start_utf16: 1,
            end_utf16: 3,
            exact_quote: "😀".into(),
            quote_hash: None,
        },
        body: "Remember this".into(),
    }
}
fn note_request(
    workspace: &WorkspaceId,
    annotation: Annotation,
    expected_revision: u64,
) -> SaveAnnotation {
    SaveAnnotation {
        contract_version: 1,
        workspace_id: workspace.clone(),
        operation_id: OperationId::generate(),
        expected_revision,
        annotation,
    }
}

#[test]
fn document_revisions_and_receipts_are_immutable_across_restart() {
    tauri::async_runtime::block_on(async {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        let service = ApplicationService::start(root.clone()).unwrap();
        let workspace = service
            .status(RequestContext::local_ui())
            .await
            .unwrap()
            .preferences
            .workspace_id;
        let id = resource(&service, &workspace, 0).await;
        let request = SaveDocument {
            contract_version: 1,
            workspace_id: workspace.clone(),
            operation_id: OperationId::generate(),
            resource_id: id,
            expected_revision: 0,
            document: document(),
        };
        let first = service
            .save_document(RequestContext::local_ui(), request.clone())
            .await
            .unwrap();
        assert_eq!(
            service
                .save_document(RequestContext::local_ui(), request.clone())
                .await
                .unwrap(),
            first
        );
        let mut changed = request.clone();
        changed.document.blocks[0].content = BlockContent::Code {
            text: "Second".into(),
            language: None,
        };
        assert_eq!(
            service
                .save_document(RequestContext::local_ui(), changed.clone())
                .await
                .unwrap_err()
                .code,
            ErrorCode::IdempotencyConflict
        );
        changed.operation_id = OperationId::generate();
        assert_eq!(
            service
                .save_document(RequestContext::local_ui(), changed.clone())
                .await
                .unwrap_err()
                .code,
            ErrorCode::RevisionConflict
        );
        changed.expected_revision = 1;
        let second = service
            .save_document(RequestContext::local_ui(), changed)
            .await
            .unwrap();
        assert_ne!(first.id, second.id);
        assert_eq!(second.revision, 2);
        service.shutdown().await.unwrap();
        let service = ApplicationService::start(root).unwrap();
        assert_eq!(
            service
                .read_document(RequestContext::local_ui(), id, Some(first.id))
                .await
                .unwrap(),
            Some(first.clone())
        );
        assert_eq!(
            service
                .read_document(RequestContext::local_ui(), id, None)
                .await
                .unwrap(),
            Some(second)
        );
        assert_eq!(
            service
                .save_document(RequestContext::local_ui(), request)
                .await
                .unwrap(),
            first
        );
        service.shutdown().await.unwrap();
    });
}

#[test]
fn annotation_identity_cannot_change_its_resource_or_source_revision() {
    tauri::async_runtime::block_on(async {
        let temp = tempfile::tempdir().unwrap();
        let service = ApplicationService::start(temp.path().join("workspace")).unwrap();
        let workspace = service
            .status(RequestContext::local_ui())
            .await
            .unwrap()
            .preferences
            .workspace_id;
        let a = resource(&service, &workspace, 0).await;
        let b = resource(&service, &workspace, 1).await;
        let first = saved(&service, &workspace, a).await;
        let other = saved(&service, &workspace, b).await;
        let original = service
            .save_annotation(
                RequestContext::local_ui(),
                note_request(&workspace, annotation(&first), 0),
            )
            .await
            .unwrap();
        let mut moved = annotation(&other);
        moved.id = original.id;
        assert_eq!(
            service
                .save_annotation(
                    RequestContext::local_ui(),
                    note_request(&workspace, moved, 1)
                )
                .await
                .unwrap_err()
                .code,
            ErrorCode::InvalidRequest
        );
        let next = service
            .save_document(
                RequestContext::local_ui(),
                SaveDocument {
                    contract_version: 1,
                    workspace_id: workspace.clone(),
                    operation_id: OperationId::generate(),
                    resource_id: a,
                    expected_revision: 1,
                    document: document(),
                },
            )
            .await
            .unwrap();
        let mut moved = annotation(&next);
        moved.id = original.id;
        assert_eq!(
            service
                .save_annotation(
                    RequestContext::local_ui(),
                    note_request(&workspace, moved, 1)
                )
                .await
                .unwrap_err()
                .code,
            ErrorCode::InvalidRequest
        );
        assert_eq!(
            service
                .annotations(RequestContext::local_ui(), a)
                .await
                .unwrap(),
            vec![original]
        );
        assert!(
            service
                .annotations(RequestContext::local_ui(), b)
                .await
                .unwrap()
                .is_empty()
        );
        service.shutdown().await.unwrap();
    });
}

#[test]
fn notes_and_positions_validate_ownership_references_bounds_and_exact_retries() {
    tauri::async_runtime::block_on(async {
        let temp = tempfile::tempdir().unwrap();
        let service = ApplicationService::start(temp.path().join("workspace")).unwrap();
        let workspace = service
            .status(RequestContext::local_ui())
            .await
            .unwrap()
            .preferences
            .workspace_id;
        let id = resource(&service, &workspace, 0).await;
        let value = saved(&service, &workspace, id).await;
        let request = note_request(&workspace, annotation(&value), 0);
        let note = service
            .save_annotation(RequestContext::local_ui(), request.clone())
            .await
            .unwrap();
        assert_eq!(
            service
                .save_annotation(RequestContext::local_ui(), request.clone())
                .await
                .unwrap(),
            note
        );
        let mut stale = request.clone();
        stale.operation_id = OperationId::generate();
        assert_eq!(
            service
                .save_annotation(RequestContext::local_ui(), stale)
                .await
                .unwrap_err()
                .code,
            ErrorCode::RevisionConflict
        );
        let mut changed = request.clone();
        changed.annotation.body = "Different".into();
        assert_eq!(
            service
                .save_annotation(RequestContext::local_ui(), changed)
                .await
                .unwrap_err()
                .code,
            ErrorCode::IdempotencyConflict
        );
        let mut bad = request.clone();
        bad.operation_id = OperationId::generate();
        bad.expected_revision = 1;
        bad.annotation.anchor.start_utf16 = 2;
        assert_eq!(
            service
                .save_annotation(RequestContext::local_ui(), bad)
                .await
                .unwrap_err()
                .code,
            ErrorCode::InvalidRequest
        );
        let mut wrong_workspace = request;
        wrong_workspace.workspace_id = WorkspaceId::generate();
        assert_eq!(
            service
                .save_annotation(RequestContext::local_ui(), wrong_workspace)
                .await
                .unwrap_err()
                .code,
            ErrorCode::PermissionDenied
        );
        let mut request = SaveReadingPosition {
            contract_version: 1,
            workspace_id: workspace.clone(),
            operation_id: OperationId::generate(),
            expected_revision: 0,
            position: ReadingPosition {
                resource_id: id,
                revision_id: value.id,
                block_id: value.document.blocks[0].id,
                offset_utf16: 2,
                revision: 0,
            },
        };
        assert_eq!(
            service
                .save_reading_position(RequestContext::local_ui(), request.clone())
                .await
                .unwrap_err()
                .code,
            ErrorCode::InvalidRequest
        );
        request.position.offset_utf16 = 5;
        assert_eq!(
            service
                .save_reading_position(RequestContext::local_ui(), request.clone())
                .await
                .unwrap_err()
                .code,
            ErrorCode::InvalidRequest
        );
        request.position.offset_utf16 = 3;
        let position = service
            .save_reading_position(RequestContext::local_ui(), request.clone())
            .await
            .unwrap();
        assert_eq!(
            service
                .save_reading_position(RequestContext::local_ui(), request.clone())
                .await
                .unwrap(),
            position
        );
        assert_eq!(
            service
                .reading_position(RequestContext::local_ui(), id)
                .await
                .unwrap(),
            Some(position)
        );
        let mut stale = request.clone();
        stale.operation_id = OperationId::generate();
        assert_eq!(
            service
                .save_reading_position(RequestContext::local_ui(), stale)
                .await
                .unwrap_err()
                .code,
            ErrorCode::RevisionConflict
        );
        request.position.offset_utf16 = 4;
        assert_eq!(
            service
                .save_reading_position(RequestContext::local_ui(), request)
                .await
                .unwrap_err()
                .code,
            ErrorCode::IdempotencyConflict
        );
        assert_eq!(
            service
                .annotations(RequestContext::local_ui(), id)
                .await
                .unwrap(),
            vec![note]
        );
        service.shutdown().await.unwrap();
    });
}

#[test]
fn unsupported_contract_and_exhausted_revision_cannot_enter_content_storage() {
    tauri::async_runtime::block_on(async {
        let temp = tempfile::tempdir().unwrap();
        let service = ApplicationService::start(temp.path().join("workspace")).unwrap();
        let initial = service.library(RequestContext::local_ui()).await.unwrap();
        let workspace = service
            .status(RequestContext::local_ui())
            .await
            .unwrap()
            .preferences
            .workspace_id;
        for (contract_version, expected_revision) in [(2, 0), (1, 9_007_199_254_740_991)] {
            let request = ApplyContent {
                contract_version,
                workspace_id: workspace.clone(),
                operation_id: OperationId::generate(),
                expected_revision,
                command: Command::CreateResource {
                    id: ResourceId::generate(),
                    title: "Rejected boundary write".into(),
                },
            };
            assert_eq!(
                service
                    .apply_content(RequestContext::local_ui(), request.clone())
                    .await
                    .unwrap_err()
                    .code,
                ErrorCode::InvalidRequest
            );
            assert!(
                service
                    .operation_receipt(RequestContext::local_ui(), request.operation_id)
                    .await
                    .unwrap()
                    .is_none()
            );
            assert_eq!(
                service.library(RequestContext::local_ui()).await.unwrap(),
                initial
            );
        }
        service.shutdown().await.unwrap();
    });
}

#[test]
fn asset_references_require_registered_sources_and_survive_restart() {
    tauri::async_runtime::block_on(async {
        use crate::document::SourceReference;
        use std::io::Cursor;

        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        let service = ApplicationService::start(root.clone()).unwrap();
        let workspace = service
            .status(RequestContext::local_ui())
            .await
            .unwrap()
            .preferences
            .workspace_id;
        let resource_id = resource(&service, &workspace, 0).await;
        let media = |digest: String| BlockContent::Media {
            asset_digest: digest,
            mime: "image/png".into(),
            caption: "Preserved image".into(),
            alt: "Source illustration".into(),
        };
        let unsupported = |digest: String| BlockContent::Unsupported {
            original: "Original payload".into(),
            source: Some(SourceReference {
                asset_digest: digest,
                locator: "page:1".into(),
            }),
            reason: "Preserved unsupported source".into(),
        };
        for content in [media("a".repeat(64)), unsupported("b".repeat(64))] {
            let request = SaveDocument {
                contract_version: 1,
                workspace_id: workspace.clone(),
                operation_id: OperationId::generate(),
                resource_id,
                expected_revision: 0,
                document: Document {
                    schema_version: 1,
                    blocks: vec![Block {
                        id: Uuid::new_v4(),
                        content,
                    }],
                },
            };
            assert_eq!(
                service
                    .save_document(RequestContext::local_ui(), request.clone())
                    .await
                    .unwrap_err()
                    .code,
                ErrorCode::InvalidRequest
            );
            assert!(
                service
                    .operation_receipt(RequestContext::local_ui(), request.operation_id)
                    .await
                    .unwrap()
                    .is_none()
            );
            assert!(
                service
                    .read_document(RequestContext::local_ui(), resource_id, None)
                    .await
                    .unwrap()
                    .is_none()
            );
        }
        let asset = service
            .register_asset(
                RequestContext::local_ui(),
                RegisterAsset {
                    contract_version: 1,
                    workspace_id: workspace.clone(),
                    operation_id: OperationId::generate(),
                },
                Box::new(Cursor::new(b"Preserved source bytes".to_vec())),
                1024,
            )
            .await
            .unwrap();
        let document = Document {
            schema_version: 1,
            blocks: vec![Block {
                id: Uuid::new_v4(),
                content: BlockContent::List {
                    ordered: false,
                    items: vec![
                        vec![
                            media(asset.digest.as_str().into()),
                            unsupported(asset.digest.as_str().into()),
                            BlockContent::Unsupported {
                                original: "Inline source".into(),
                                source: None,
                                reason: "No separate asset".into(),
                            },
                        ]
                        .into_iter()
                        .map(|content| Block {
                            id: Uuid::new_v4(),
                            content,
                        })
                        .collect(),
                    ],
                },
            }],
        };
        let saved = service
            .save_document(
                RequestContext::local_ui(),
                SaveDocument {
                    contract_version: 1,
                    workspace_id: workspace,
                    operation_id: OperationId::generate(),
                    resource_id,
                    expected_revision: 0,
                    document: document.clone(),
                },
            )
            .await
            .unwrap();
        assert_eq!(saved.revision, 1);
        assert_eq!(saved.document, document);
        service.shutdown().await.unwrap();
        let service = ApplicationService::start(root).unwrap();
        assert_eq!(
            service
                .read_document(RequestContext::local_ui(), resource_id, None)
                .await
                .unwrap(),
            Some(saved)
        );
        service.shutdown().await.unwrap();
    });
}

#[test]
fn organization_links_survive_replacement_and_restart() {
    tauri::async_runtime::block_on(async {
        use crate::organization::{
            CollectionId, GoalId, PlacementId, RelationId, RelationKind, SubjectId, TagId, Target,
            UnitId,
        };

        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        let service = ApplicationService::start(root.clone()).unwrap();
        let workspace = service
            .status(RequestContext::local_ui())
            .await
            .unwrap()
            .preferences
            .workspace_id;
        let subject = SubjectId::generate();
        let goals = [GoalId::generate(), GoalId::generate()];
        let resources = [ResourceId::generate(), ResourceId::generate()];
        let unit = UnitId::generate();
        let next_unit = UnitId::generate();
        let tag = TagId::generate();
        let collections = [CollectionId::generate(), CollectionId::generate()];
        let placements = [
            PlacementId::generate(),
            PlacementId::generate(),
            PlacementId::generate(),
        ];
        let relations = [
            RelationId::generate(),
            RelationId::generate(),
            RelationId::generate(),
        ];
        let commands = [
            Command::CreateSubject {
                id: subject,
                title: "Subject".into(),
                prior_knowledge: None,
            },
            Command::CreateGoal {
                id: goals[0],
                subject_id: subject,
                title: "First goal".into(),
                criteria: "First criterion".into(),
            },
            Command::CreateGoal {
                id: goals[1],
                subject_id: subject,
                title: "Second goal".into(),
                criteria: "Second criterion".into(),
            },
            Command::CreateResource {
                id: resources[0],
                title: "First resource".into(),
            },
            Command::CreateResource {
                id: resources[1],
                title: "Second resource".into(),
            },
            Command::CreateUnit {
                id: unit,
                title: "Unit".into(),
                objective: "Learn".into(),
                goal_ids: goals.to_vec(),
                resource_ids: resources.to_vec(),
            },
            Command::SetTag {
                id: tag,
                title: "Tag".into(),
                resource_ids: resources.to_vec(),
            },
            Command::UpdateUnit {
                id: unit,
                title: "Unit".into(),
                objective: "Learn".into(),
                goal_ids: vec![goals[1], goals[0]],
                resource_ids: vec![resources[1]],
            },
            Command::SetTag {
                id: tag,
                title: "Tag".into(),
                resource_ids: vec![resources[1], resources[0]],
            },
            Command::CreateCollection {
                id: collections[0],
                subject_id: subject,
                parent_id: None,
                title: "First collection".into(),
            },
            Command::CreateCollection {
                id: collections[1],
                subject_id: subject,
                parent_id: None,
                title: "Second collection".into(),
            },
            Command::CreateUnit {
                id: next_unit,
                title: "Next unit".into(),
                objective: "Apply".into(),
                goal_ids: vec![],
                resource_ids: vec![],
            },
            Command::Place {
                id: placements[0],
                collection_id: collections[0],
                target: Target::Resource(resources[0]),
            },
            Command::Place {
                id: placements[1],
                collection_id: collections[1],
                target: Target::Resource(resources[0]),
            },
            Command::Place {
                id: placements[2],
                collection_id: collections[0],
                target: Target::Unit(unit),
            },
            Command::Relate {
                id: relations[0],
                kind: RelationKind::Prerequisite,
                from: unit,
                to: next_unit,
            },
            Command::Relate {
                id: relations[1],
                kind: RelationKind::Related,
                from: unit,
                to: next_unit,
            },
            Command::Relate {
                id: relations[2],
                kind: RelationKind::Reference,
                from: next_unit,
                to: unit,
            },
        ];
        let command_count = commands.len() as u64;
        for (revision, command) in commands.into_iter().enumerate() {
            service
                .apply_content(
                    RequestContext::local_ui(),
                    ApplyContent {
                        contract_version: 1,
                        workspace_id: workspace.clone(),
                        operation_id: OperationId::generate(),
                        expected_revision: revision as u64,
                        command,
                    },
                )
                .await
                .unwrap();
        }
        service.shutdown().await.unwrap();
        let service = ApplicationService::start(root.clone()).unwrap();
        let snapshot = service.library(RequestContext::local_ui()).await.unwrap();
        assert_eq!(snapshot.revision, command_count);
        assert_eq!(snapshot.units[&unit].goal_ids, vec![goals[1], goals[0]]);
        assert_eq!(snapshot.units[&unit].resource_ids, vec![resources[1]]);
        assert_eq!(
            snapshot.tags[&tag].resource_ids,
            vec![resources[1], resources[0]]
        );
        assert_eq!(snapshot.resources.len(), 2);
        assert_eq!(snapshot.placements.len(), 3);
        assert_eq!(
            snapshot.placements[&placements[0]].target,
            Target::Resource(resources[0])
        );
        assert_eq!(
            snapshot.placements[&placements[1]].target,
            Target::Resource(resources[0])
        );
        assert_eq!(
            snapshot.placements[&placements[0]].collection_id,
            collections[0]
        );
        assert_eq!(
            snapshot.placements[&placements[1]].collection_id,
            collections[1]
        );
        assert_eq!(
            snapshot.placements[&placements[2]].target,
            Target::Unit(unit)
        );
        assert_eq!(snapshot.placements[&placements[2]].position, 1);
        assert_eq!(
            snapshot.relations[&relations[0]].kind,
            RelationKind::Prerequisite
        );
        assert_eq!(
            snapshot.relations[&relations[1]].kind,
            RelationKind::Related
        );
        assert_eq!(
            snapshot.relations[&relations[2]].kind,
            RelationKind::Reference
        );
        assert_eq!(snapshot.relations[&relations[2]].from, next_unit);
        assert_eq!(snapshot.relations[&relations[2]].to, unit);
        let replaced = service
            .apply_content(
                RequestContext::local_ui(),
                ApplyContent {
                    contract_version: 1,
                    workspace_id: workspace,
                    operation_id: OperationId::generate(),
                    expected_revision: command_count,
                    command: Command::RemoveRelation { id: relations[0] },
                },
            )
            .await
            .unwrap();
        assert_eq!(replaced.relations.len(), 2);
        assert!(!replaced.relations.contains_key(&relations[0]));
        service.shutdown().await.unwrap();
        let service = ApplicationService::start(root).unwrap();
        assert_eq!(
            service.library(RequestContext::local_ui()).await.unwrap(),
            replaced
        );
        service.shutdown().await.unwrap();
    });
}

#[test]
fn organization_receipts_are_exact_and_invalid_nested_references_are_atomic() {
    tauri::async_runtime::block_on(async {
        let temp = tempfile::tempdir().unwrap();
        let service = ApplicationService::start(temp.path().join("workspace")).unwrap();
        let workspace = service
            .status(RequestContext::local_ui())
            .await
            .unwrap()
            .preferences
            .workspace_id;
        let request = ApplyContent {
            contract_version: 1,
            workspace_id: workspace.clone(),
            operation_id: OperationId::generate(),
            expected_revision: 0,
            command: Command::CreateResource {
                id: ResourceId::generate(),
                title: "Resource".into(),
            },
        };
        let first = service
            .apply_content(RequestContext::local_ui(), request.clone())
            .await
            .unwrap();
        assert_eq!(
            service
                .apply_content(RequestContext::local_ui(), request.clone())
                .await
                .unwrap(),
            first
        );
        let mut changed = request.clone();
        changed.command = Command::CreateResource {
            id: ResourceId::generate(),
            title: "Other".into(),
        };
        assert_eq!(
            service
                .apply_content(RequestContext::local_ui(), changed.clone())
                .await
                .unwrap_err()
                .code,
            ErrorCode::IdempotencyConflict
        );
        changed.operation_id = OperationId::generate();
        assert_eq!(
            service
                .apply_content(RequestContext::local_ui(), changed)
                .await
                .unwrap_err()
                .code,
            ErrorCode::RevisionConflict
        );
        let invalid = ApplyContent {
            contract_version: 1,
            workspace_id: workspace,
            operation_id: OperationId::generate(),
            expected_revision: 1,
            command: Command::CreateCollection {
                id: organization::CollectionId::generate(),
                subject_id: organization::SubjectId::generate(),
                parent_id: None,
                title: "Missing parent subject".into(),
            },
        };
        assert!(
            service
                .apply_content(RequestContext::local_ui(), invalid.clone())
                .await
                .is_err()
        );
        assert_eq!(
            service.library(RequestContext::local_ui()).await.unwrap(),
            first
        );
        assert!(
            service
                .operation_receipt(RequestContext::local_ui(), invalid.operation_id)
                .await
                .unwrap()
                .is_none()
        );
        service.shutdown().await.unwrap();
    });
}

#[test]
fn corrupt_note_or_reading_position_is_rejected_on_read() {
    tauri::async_runtime::block_on(async {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        let service = ApplicationService::start(root.clone()).unwrap();
        let workspace = service
            .status(RequestContext::local_ui())
            .await
            .unwrap()
            .preferences
            .workspace_id;
        let id = resource(&service, &workspace, 0).await;
        let value = saved(&service, &workspace, id).await;
        let note = service
            .save_annotation(
                RequestContext::local_ui(),
                note_request(&workspace, annotation(&value), 0),
            )
            .await
            .unwrap();
        let position = ReadingPosition {
            resource_id: id,
            revision_id: value.id,
            block_id: value.document.blocks[0].id,
            offset_utf16: 3,
            revision: 0,
        };
        service
            .save_reading_position(
                RequestContext::local_ui(),
                SaveReadingPosition {
                    contract_version: 1,
                    workspace_id: workspace.clone(),
                    operation_id: OperationId::generate(),
                    expected_revision: 0,
                    position,
                },
            )
            .await
            .unwrap();
        service.shutdown().await.unwrap();
        let connection = rusqlite::Connection::open(root.join("memoquill.sqlite3")).unwrap();
        let mut bad = note.anchor;
        bad.resource_id = Uuid::new_v4();
        connection
            .execute(
                "UPDATE annotations SET anchor=?1",
                [serde_json::to_string(&bad).unwrap()],
            )
            .unwrap();
        connection
            .execute("UPDATE reading_positions SET offset_utf16=2", [])
            .unwrap();
        drop(connection);
        let service = ApplicationService::start(root.clone()).unwrap();
        assert_eq!(
            service
                .reading_position(RequestContext::local_ui(), id)
                .await
                .unwrap_err()
                .code,
            ErrorCode::RecoveryRequired
        );
        service.shutdown().await.unwrap();
        let service = ApplicationService::start(root).unwrap();
        assert_eq!(
            service
                .annotations(RequestContext::local_ui(), id)
                .await
                .unwrap_err()
                .code,
            ErrorCode::RecoveryRequired
        );
        service.shutdown().await.unwrap();
    });
}

#[test]
fn corrupt_document_revision_identity_is_rejected_on_read() {
    tauri::async_runtime::block_on(async {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("workspace");
        let service = ApplicationService::start(root.clone()).unwrap();
        let workspace = service
            .status(RequestContext::local_ui())
            .await
            .unwrap()
            .preferences
            .workspace_id;
        let id = resource(&service, &workspace, 0).await;
        saved(&service, &workspace, id).await;
        service.shutdown().await.unwrap();
        let connection = rusqlite::Connection::open(root.join("memoquill.sqlite3")).unwrap();
        connection
            .execute_batch("BEGIN IMMEDIATE; PRAGMA defer_foreign_keys=ON;")
            .unwrap();
        connection
            .execute(
                "UPDATE resource_revisions SET id=?1",
                [Uuid::nil().to_string()],
            )
            .unwrap();
        connection
            .execute(
                "UPDATE resource_heads SET revision_id=?1",
                [Uuid::nil().to_string()],
            )
            .unwrap();
        connection.execute_batch("COMMIT").unwrap();
        drop(connection);
        let service = ApplicationService::start(root).unwrap();
        assert_eq!(
            service
                .read_document(RequestContext::local_ui(), id, None)
                .await
                .unwrap_err()
                .code,
            ErrorCode::RecoveryRequired
        );
        service.shutdown().await.unwrap();
    });
}
