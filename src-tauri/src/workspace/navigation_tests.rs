use super::{database::Database, *};
use serde_json::json;
use std::io::Cursor;
use uuid::Uuid;

fn navigation() -> NavigationContext {
    NavigationContext {
        contract_version: 1,
        subject_id: "subject".into(),
        collection_id: None,
        resource_id: "material".into(),
        query: "Private remembered query".into(),
        page: 2,
        search_offset: 64,
        search_active: true,
    }
}
#[test]
fn legacy_preferences_remain_byte_compatible_and_navigation_is_strictly_versioned() {
    let prefs: ReadingPreferences =
        serde_json::from_value(json!({"theme":"system","textScale":100})).unwrap();
    assert!(prefs.navigation.is_none());
    assert_eq!(
        serde_json::to_value(prefs).unwrap(),
        json!({"theme":"system","textScale":100})
    );
    for patch in [
        json!({"contractVersion":2}),
        json!({"query":"x".repeat(257)}),
        json!({"searchOffset":1000001}),
        json!({"page":1000001}),
        json!({"subjectId":"../outside"}),
        json!({"unknown":true}),
    ] {
        let mut nav = serde_json::to_value(navigation()).unwrap();
        for (k, v) in patch.as_object().unwrap() {
            nav[k] = v.clone();
        }
        assert!(
            serde_json::from_value::<ReadingPreferences>(
                json!({"theme":"dark","textScale":125,"navigation":nav})
            )
            .is_err()
        );
    }
}
#[test]
fn navigation_survives_exact_retry_restart_and_native_restore_but_not_shareable_export() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("source");
    let mut db = Database::open(&path).unwrap();
    let mut request = SavePreferences {
        contract_version: 1,
        workspace_id: db.id.clone(),
        operation_id: OperationId::generate(),
        expected_revision: 0,
        preferences: ReadingPreferences {
            subject_settings: None,
            shortcuts: None,
            theme: Theme::Dark,
            text_scale: TextScale::try_from(125).unwrap(),
            navigation: Some(navigation()),
            customization: Some(Box::new(SettingsLayer {
                contract_version: 1,
                overrides: SettingsOverrides {
                    goal: Some("Private goal".into()),
                    focus: Some("Private focus".into()),
                    ..Default::default()
                },
            })),
        },
    };
    request.preferences.subject_settings = Some(Box::new(serde_json::from_value(json!({"subject": {"contractVersion":1,"overrides":{"goal":"Private subject goal","theme":"light"}}})).unwrap()));
    request.preferences.shortcuts = Some(Box::new(serde_json::from_value(json!({"contractVersion":1,"bindings":{"openSettings":"Ctrl+Alt+Shift+S","focusSearch":null,"focusWorkspace":null}})).unwrap()));
    let expected = db
        .save_preferences(RequestContext::local_ui(), &request)
        .unwrap();
    assert_eq!(
        db.save_preferences(RequestContext::local_ui(), &request)
            .unwrap(),
        expected
    );
    drop(db);
    let mut db = Database::open(&path).unwrap();
    assert_eq!(db.preferences().unwrap(), expected);
    let capture = db
        .capture_native(RequestContext::local_ui(), Uuid::new_v4())
        .unwrap();
    let mut bytes = Cursor::new(Vec::new());
    capture.write_to(&mut bytes).unwrap();
    let package = NativeCapture::read_from(Cursor::new(bytes.into_inner())).unwrap();
    let target = temp.path().join("native");
    let manager = CourseManager::open(target.clone()).unwrap();
    let course = Uuid::new_v4();
    manager
        .restore_native(course, "Synthetic context".into(), &package, false)
        .unwrap();
    let restored = Database::open(&target.join("courses/data").join(course.to_string())).unwrap();
    assert_eq!(
        restored.preferences().unwrap().preferences,
        expected.preferences
    );
    let portable = db
        .capture_portable_with_history(RequestContext::local_ui(), Uuid::new_v4(), false, true)
        .unwrap();
    let mut bytes = Cursor::new(Vec::new());
    portable.write_to(&mut bytes).unwrap();
    let package = PortableCapture::read_from(Cursor::new(bytes.into_inner())).unwrap();
    let target = temp.path().join("portable");
    let manager = CourseManager::open(target.clone()).unwrap();
    let course = Uuid::new_v4();
    manager
        .import_portable(course, "Synthetic context".into(), &package)
        .unwrap();
    let restored = Database::open(&target.join("courses/data").join(course.to_string())).unwrap();
    assert!(
        restored
            .preferences()
            .unwrap()
            .preferences
            .navigation
            .is_none()
    );
    assert!(
        restored
            .preferences()
            .unwrap()
            .preferences
            .customization
            .as_ref()
            .is_none_or(|layer| layer.overrides.goal.is_none() && layer.overrides.focus.is_none())
    );
    assert!(
        restored
            .preferences()
            .unwrap()
            .preferences
            .subject_settings
            .is_none()
    );
    assert!(
        restored
            .preferences()
            .unwrap()
            .preferences
            .shortcuts
            .is_none()
    );
    request.expected_revision = expected.revision;
    request.operation_id = OperationId::generate();
    request.preferences.navigation = None;
    let cleared = db
        .save_preferences(RequestContext::local_ui(), &request)
        .unwrap();
    assert!(cleared.preferences.navigation.is_none());
    assert_eq!(cleared.preferences.theme, Theme::Dark);
}

#[test]
fn customization_contract_rejects_invalid_subject_layers_and_shortcut_conflicts() {
    let base = json!({"theme":"system","textScale":100});
    for subject in [
        json!({"../escape":{"contractVersion":1,"overrides":{}}}),
        json!({"__proto__":{"contractVersion":1,"overrides":{}}}),
        json!({"subject":{"contractVersion":2,"overrides":{}}}),
    ] {
        let mut value = base.clone();
        value["subjectSettings"] = subject;
        assert!(serde_json::from_value::<ReadingPreferences>(value).is_err());
    }
    for bindings in [
        json!({"openSettings":"Ctrl+Alt+Shift+S","focusSearch":"Ctrl+Alt+Shift+S"}),
        json!({"openSettings":"Ctrl+Alt+Shift+R"}),
        json!({"focusWorkspace":"Ctrl+W"}),
        json!({"unknown":"Ctrl+Alt+Shift+X"}),
    ] {
        let mut value = base.clone();
        value["shortcuts"] = json!({"contractVersion":1,"bindings":bindings});
        assert!(serde_json::from_value::<ReadingPreferences>(value).is_err());
    }
    let many: serde_json::Map<String, serde_json::Value> = (0..129)
        .map(|n| {
            (
                format!("subject-{n}"),
                json!({"contractVersion":1,"overrides":{}}),
            )
        })
        .collect();
    let mut value = base;
    value["subjectSettings"] = json!(many);
    assert!(serde_json::from_value::<ReadingPreferences>(value).is_err());
}
#[test]
fn customization_typed_writes_revalidate_before_committing() {
    let temp = tempfile::tempdir().unwrap();
    let mut db = Database::open(&temp.path().join("source")).unwrap();
    let before = db.preferences().unwrap();
    let mut preferences: ReadingPreferences = serde_json::from_value(json!({"theme":"system","textScale":100,"shortcuts":{"contractVersion":1,"bindings":{"openSettings":"Ctrl+Alt+Shift+S"}}})).unwrap();
    preferences
        .shortcuts
        .as_mut()
        .unwrap()
        .bindings
        .focus_search = Some("Ctrl+Alt+Shift+S".into());
    let request = SavePreferences {
        contract_version: 1,
        workspace_id: db.id.clone(),
        operation_id: OperationId::generate(),
        expected_revision: before.revision,
        preferences,
    };
    assert!(
        db.save_preferences(RequestContext::local_ui(), &request)
            .is_err()
    );
    assert_eq!(db.preferences().unwrap(), before);
}
