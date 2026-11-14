//! Shipped offline schemas. Metadata describes preview and unavailable study capabilities separately.
use super::*;
use serde_json::json;
use std::collections::BTreeMap;
pub(super) fn object(properties: Value, required: &[&str]) -> Value {
    json!({"type":"object","properties":properties,"required":required,"additionalProperties":false})
}
fn string(max: usize) -> Value {
    json!({"type":"string","minLength":1,"maxLength":max,"x-maxUtf8Bytes":max,"x-whitespaceOnly":false})
}
// JSON Schema maxLength counts code points; native scalar validation counts UTF-8 bytes.
fn response_value() -> Value {
    json!({"type":"string","maxLength":4096,"x-maxUtf8Bytes":4096,"description":"At most 4096 UTF-8 bytes; code-point length alone is insufficient."})
}
fn enumeration(values: &[&str]) -> Value {
    json!({"type":"string","enum":values})
}
pub(super) fn view_fields() -> Vec<&'static str> {
    vec![
        "section.expanded",
        "response.present",
        "assistance.hint",
        "assistance.revealed",
        "preview.submitted",
    ]
}
fn event_value(schema: Value) -> Value {
    json!({"oneOf":[schema,object(json!({"eventField":{"const":"value"}}),&["eventField"]) ]})
}
fn action_binding() -> Value {
    let variants = actions()
        .into_iter()
        .map(|a| {
            let mut input = a.input_schema;
            if let Some(properties) = input.get_mut("properties").and_then(Value::as_object_mut) {
                for schema in properties.values_mut() {
                    *schema = event_value(schema.clone());
                }
            }
            let target = if a.action_id == "source.open" {
                "bindingId"
            } else {
                "targetNodeId"
            };
            let mut properties =
                json!({"actionId":{"const":a.action_id},"version":{"const":1},"inputs":input});
            properties[target] = string(128);
            object(properties, &["actionId", "version", target, "inputs"])
        })
        .collect::<Vec<_>>();
    json!({"oneOf":variants,"x-maxBytes":16384,"x-eventFieldRule":"value may be mapped only from response/toggle events; one action per event"})
}
fn condition() -> Value {
    let reference = json!({"$ref":"#/$defs/condition"});
    let mut schema = json!({"oneOf":[object(json!({"kind":{"const":"literal"},"value":{"type":"boolean"}}),&["kind","value"]),object(json!({"kind":{"const":"eq"},"field":enumeration(&view_fields()),"value":{"type":"boolean"}}),&["kind","field","value"]),object(json!({"kind":{"const":"compare"},"field":false,"op":{"enum":["lt","lte","gt","gte"]},"value":{"type":"number"}}),&["kind","field","op","value"]),object(json!({"kind":{"const":"not"},"expr":reference}),&["kind","expr"]),object(json!({"kind":{"enum":["all","any"]},"exprs":{"type":"array","items":reference,"minItems":1,"maxItems":31}}),&["kind","exprs"])],"x-maxDepth":4,"x-maxNodes":32,"x-fields":view_fields(),"x-missingField":"disabledWithDiagnostic"});
    let definition = schema.clone();
    schema["$defs"] = json!({"condition":definition});
    schema
}
fn appearance() -> Value {
    object(
        json!({"spacing":enumeration(&["compact","normal","wide"]),"theme":enumeration(&["system","light","dark"]),"typography":enumeration(&["default","serif"]),"width":enumeration(&["normal","wide"]),"emphasis":enumeration(&["normal","strong"])}),
        &["spacing", "theme", "typography", "width", "emphasis"],
    )
}
fn activity_definition() -> Value {
    let items = json!({"type":"array","minItems":1,"maxItems":64,"items":object(json!({"id":string(128),"text":string(4096)}),&["id","text"]),"x-uniqueField":"id"});
    let mut variants=["recall","singleChoice","constructed","procedure"].into_iter().map(|family|{let mut p=json!({"family":{"const":family},"version":{"const":1},"prompt":string(16384),"hints":{"type":"array","maxItems":16,"items":string(4096)},"evaluation":{"const":"self"}});let mut required=vec!["family","version","prompt","hints","evaluation"];match family {
 "recall"=>{p["answer"]=string(16384);p["evaluation"]=enumeration(&["self","exact"]);required.push("answer");},
 "singleChoice"=>{p["options"]=items.clone();p["correctOptionId"]=string(128);p["evaluation"]=json!({"const":"exact"});required.extend(["options","correctOptionId"]);},
 "constructed"=>{p["rubric"]=string(16384);p["workedExampleBindingId"]=string(128);required.push("rubric");},
 _=>{p["steps"]=items.clone();p["reflection"]=string(4096);required.extend(["steps","reflection"]);}}
 object(p,&required)}).collect::<Vec<_>>();
    let mut cloze = variants[0].clone();
    cloze["properties"]["version"] = json!({"const":2});
    cloze["properties"]["responseFormat"] = json!({"const":"boundedCloze"});
    cloze["properties"]["answer"] = string(1024);
    cloze["required"]
        .as_array_mut()
        .unwrap()
        .push(json!("responseFormat"));
    variants.push(cloze);
    let mut instruction = variants[2].clone();
    instruction["properties"]["version"] = json!({"const":2});
    instruction["properties"]["instruction"] = object(
        json!({"phase":enumeration(&["workedExample","completion"]),"material":string(16384)}),
        &["phase", "material"],
    );
    instruction["required"]
        .as_array_mut()
        .unwrap()
        .push(json!("instruction"));
    variants.push(instruction);
    json!({"oneOf":variants,"x-maxBytes":65536,"x-evidence":"previewNeverWritesStudyEvidence"})
}
pub(super) fn props_schema(t: &str) -> Value {
    match t {
        "stack" => object(
            json!({"gap":enumeration(&["compact","normal","wide"]),"headingLevel":{"type":"integer","minimum":1,"maximum":6}}),
            &["gap", "headingLevel"],
        ),
        "columns" => object(
            json!({"ratio":enumeration(&["equal","widePrimary","wideSecondary"])}),
            &["ratio"],
        ),
        "section" => object(
            json!({"title":string(512),"collapsible":{"type":"boolean"}}),
            &["title", "collapsible"],
        ),
        "content" => object(
            json!({"bindingId":string(128),"blockRange":object(json!({"startBlockId":string(128),"endBlockId":string(128)}),&["startBlockId","endBlockId"])}),
            &["bindingId"],
        ),
        "source" => object(
            json!({"bindingId":string(128),"locator":string(1024)}),
            &["bindingId"],
        ),
        "activity" => object(
            json!({"bindingId":string(128),"presentation":enumeration(&["default","compact"])}),
            &["bindingId", "presentation"],
        ),
        "button" => object(
            json!({"label":string(512),"style":enumeration(&["primary","secondary"]),"action":action_binding()}),
            &["label", "style", "action"],
        ),
        "status" => object(
            json!({"field":enumeration(&view_fields()),"label":string(512),"targetNodeId":string(128)}),
            &["field", "label"],
        ),
        _ => json!(false),
    }
}
pub(super) fn actions() -> Vec<ActionDescriptor> {
    [
        "view.toggleSection",
        "view.focusNode",
        "source.open",
        "response.update",
        "study.hint",
        "study.reveal",
        "study.submit",
        "study.next",
    ]
    .into_iter()
    .map(|id| {
        let study = id.starts_with("study.") || id == "response.update";
        let input = match id {
            "view.toggleSection" => object(json!({"expanded":{"type":"boolean"}}), &["expanded"]),
            "response.update" => object(json!({"value":response_value()}), &["value"]),
            _ => object(json!({}), &[]),
        };
        let modes = if study {
            vec!["preview", "study"]
        } else {
            vec!["authoring", "preview", "reading"]
        };
        let availability = modes
            .iter()
            .map(|m| {
                (
                    m.to_string(),
                    if study && *m == "study" {
                        "capability_unavailable"
                    } else if study {
                        "simulated"
                    } else {
                        "available"
                    }
                    .into(),
                )
            })
            .collect();
        ActionDescriptor {
            action_id: id.into(),
            version: 1,
            input_schema: input,
            result_schema: if study {
                object(
                    json!({"status":{"const":"previewOnly"},"evidenceWritten":{"const":false}}),
                    &["status", "evidenceWritten"],
                )
            } else {
                object(
                    json!({"status":{"enum":["applied","unavailable"]}}),
                    &["status"],
                )
            },
            target_kinds: vec![
                match id {
                    "view.toggleSection" => "section",
                    "view.focusNode" => "node",
                    "source.open" => "sourceBinding",
                    _ => "activityOccurrence",
                }
                .into(),
            ],
            required_capabilities: vec![
                if study {
                    "study:preview"
                } else if id == "source.open" {
                    "source:read"
                } else {
                    "view:write"
                }
                .into(),
            ],
            valid_modes: modes.into_iter().map(String::from).collect(),
            availability,
            side_effect_class: if study {
                "isolatedPreviewState"
            } else if id == "source.open" {
                "managedSourceView"
            } else {
                "ephemeralViewState"
            }
            .into(),
            idempotency: "setOrExplicitControllerSequence;neverAutoChain".into(),
            event_field_sources: match id {
                "view.toggleSection" => vec!["toggle".into()],
                "response.update" => vec!["response".into()],
                _ => vec![],
            },
            error_codes: vec![
                "invalid_request".into(),
                "permission_denied".into(),
                "capability_unavailable".into(),
            ],
        }
    })
    .collect()
}
pub(super) fn discover() -> Vec<RegistryDescriptor> {
    ["stack","columns","section","content","source","activity","button","status"].into_iter().map(|t|{
 let slots:Vec<String>=match t{"stack"|"section"=>vec!["body".into()],"columns"=>vec!["primary".into(),"secondary".into()],_=>vec![]};
 let events:Vec<String>=match t{"section"=>vec!["toggle"],"source"=>vec!["openSource"],"activity"=>vec!["response","hint","reveal","submit"],"button"=>vec!["activate"],_=>vec![]}.into_iter().map(String::from).collect();
 let event_schemas=events.iter().map(|e|{let allowed=match e.as_str(){"toggle"=>vec!["view.toggleSection","view.focusNode"],"openSource"=>vec!["source.open"],"response"=>vec!["response.update"],"hint"=>vec!["study.hint"],"reveal"=>vec!["study.reveal"],"submit"=>vec!["study.submit"],_=>vec![]};let allowed=if e=="activate"{actions().into_iter().map(|a|a.action_id).collect()}else{allowed.into_iter().map(String::from).collect()};let field=match e.as_str(){"response"=>Some(response_value()),"toggle"=>Some(json!({"type":"boolean"})),_=>None};let event_fields=field.as_ref().map(|f|BTreeMap::from([("value".into(),f.clone())])).unwrap_or_default();let payload=if let Some(f)=field{object(json!({"value":f}),&["value"])}else{object(json!({}),&[])};(e.clone(),EventDescriptor{payload_schema:payload,allowed_actions:allowed,event_fields,maximum_actions:1})}).collect();
 let defaults=json!(match t{"stack"=>json!({"gap":"normal","headingLevel":1}),"columns"=>json!({"ratio":"equal"}),"section"=>json!({"title":"Section","collapsible":false}),"activity"=>json!({"presentation":"default"}),"button"=>json!({"label":"Continue","style":"secondary"}),"status"=>json!({"field":"response.present","label":"Status"}),_=>json!({})});
 let props_schema=props_schema(t);let props=props_schema["properties"].as_object().unwrap().iter().map(|(k,v)|(k.clone(),v.clone())).collect();
 RegistryDescriptor{type_id:t.into(),type_version:1,slots:slots.clone(),events,props,props_schema,defaults:serde_json::from_value(defaults).expect("shipped defaults"),slot_schemas:slots.into_iter().map(|s|(s,SlotDescriptor{minimum:0,maximum:64,ordered:true,allowed_children:"shippedOrPreservedOpaqueNodes;oneParent".into()})).collect(),event_schemas,actions:actions(),required_capabilities:vec!["lesson:read".into()],valid_modes:vec!["authoring".into(),"preview".into(),"reading".into()],state:StateDescriptor{scope:if t=="activity"{"previewInstance+nodeOccurrence"}else{"viewInstance+node"}.into(),schema:if t=="section"{object(json!({"expanded":{"type":"boolean","default":true}}),&["expanded"])}else if t=="activity"{object(json!({"response":{"oneOf":[response_value(),{"type":"array","maxItems":64,"items":{"type":"string","maxLength":128}}]},"hintShown":{"type":"boolean","default":false},"revealed":{"type":"boolean","default":false},"submitted":{"type":"boolean","default":false}}),&["hintShown","revealed","submitted"])}else{object(json!({}),&[])},maximum_bytes:65536,persistent:false},accessibility_contract:vec!["semantic reading order follows slot order".into(),"keyboard alternative for every authoring action".into(),"named controls and focus restoration".into()],fallback:FallbackDescriptor{mode:"inertPlaceholder".into(),preserves_opaque_payload:true,activation_allowed:false,diagnostic_code:LessonDiagnosticCode::UnsupportedType,repairs:vec![LessonRepair::RemoveOrReplaceUnsupported]},supported_migrations:vec![],schemas:BTreeMap::from([("condition".into(),condition()),("actionBinding".into(),action_binding()),("appearance".into(),appearance()),("activityDefinition".into(),activity_definition())])}
 }).collect()
}

#[cfg(test)]
mod response_tests {
    use super::*;
    #[test]
    fn response_metadata_declares_utf8_byte_bound_consistently() {
        let activity = discover()
            .into_iter()
            .find(|d| d.type_id == "activity")
            .unwrap();
        let action = activity
            .actions
            .iter()
            .find(|a| a.action_id == "response.update")
            .unwrap();
        let expected = json!({"type":"string","maxLength":4096,"x-maxUtf8Bytes":4096,"description":"At most 4096 UTF-8 bytes; code-point length alone is insufficient."});
        assert_eq!(action.input_schema["properties"]["value"], expected);
        assert_eq!(
            activity.event_schemas["response"].event_fields["value"],
            expected
        );
        assert_eq!(
            activity.event_schemas["response"].payload_schema["properties"]["value"],
            expected
        );
        assert_eq!(
            activity.state.schema["properties"]["response"]["oneOf"][0],
            expected
        );
    }
}
