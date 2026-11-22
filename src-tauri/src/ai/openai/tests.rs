use super::*;
use serde_json::json;

fn event(value: &Value) -> Vec<u8> {
    format!(
        "event: {}\r\ndata: {value}\r\n\r\n",
        value["type"].as_str().unwrap()
    )
    .into_bytes()
}

#[test]
fn openai_request_uses_stateless_streaming() {
    let body = encode(RoundRequest {
        model: "gpt-test",
        instructions: "Use only approved course context.",
        input: &[json!({"role": "user", "content": "Synthetic question"})],
        tools: &[],
    })
    .unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["store"], false);
    assert_eq!(body["stream"], true);
    assert_eq!(body["input"][0]["role"], "user");
    assert!(body.get("previous_response_id").is_none());
    assert!(body.get("conversation").is_none());
}

#[test]
fn openai_stream_assembles_chunks_and_admits_calls_only_at_completion() {
    let item = json!({
        "type": "function_call",
        "id": "fc_1",
        "call_id": "call_1",
        "name": "search",
        "namespace": "memoquill",
        "arguments": r#"{"query":"Lesson"}"#,
        "status": "completed"
    });
    let mut bytes = event(&json!({"type": "response.output_text.delta", "delta": "Found ä"}));
    bytes.extend(event(
        &json!({"type": "response.output_item.done", "item": item}),
    ));
    bytes.extend(event(&json!({
        "type": "response.completed",
        "response": {"status": "completed", "output": [item]}
    })));
    let mut stream = EventStream::default();
    let mut displayed = String::new();
    let mut completed = None;
    for chunk in bytes.chunks(1) {
        let output = stream
            .feed(chunk, &mut |text| {
                displayed.push_str(text);
                true
            })
            .unwrap();
        if output.is_some() {
            assert!(completed.is_none());
            completed = output;
        }
    }
    let output = completed.unwrap();
    assert_eq!(displayed, "Found ä");
    assert_eq!(output.text, displayed);
    assert_eq!(output.items.as_slice(), &[item]);
    assert_eq!(output.calls.len(), 1);
    let call = &output.calls[0];
    assert_eq!(call.namespace.as_deref(), Some("memoquill"));
    assert_eq!(call.name, "search");
    assert_eq!(call.arguments["query"], "Lesson");
}

#[test]
fn openai_stream_rejects_failed_cancelled_and_oversized_events() {
    let failed = event(&json!({"type": "response.failed"}));
    assert_eq!(
        EventStream::default().feed(&failed, &mut |_| true).err(),
        Some(TransportError::Incomplete),
    );
    let delta = event(&json!({"type": "response.output_text.delta", "delta": "Text"}));
    assert_eq!(
        EventStream::default().feed(&delta, &mut |_| false).err(),
        Some(TransportError::Cancelled),
    );
    let oversized = vec![b'a'; MAX_EVENT_BYTES + 1];
    assert_eq!(
        EventStream::default().feed(&oversized, &mut |_| true).err(),
        Some(TransportError::LimitExceeded),
    );
    let invalid_call = json!({
        "type": "function_call",
        "call_id": "call_1",
        "name": "search",
        "arguments": "[]",
    });
    let completed = event(&json!({
        "type": "response.completed",
        "response": {"status": "completed", "output": [invalid_call]},
    }));
    assert_eq!(
        EventStream::default().feed(&completed, &mut |_| true).err(),
        Some(TransportError::InvalidStream),
    );
}

#[tokio::test]
async fn openai_transport_observes_cancellation_before_dispatch() {
    let client = ResponsesClient::new().unwrap();
    let (_cancel, receiver) = watch::channel(true);
    let result = client
        .stream(
            "",
            RoundRequest {
                model: "gpt-test",
                instructions: "",
                input: &[json!({"role": "user", "content": "Synthetic prompt"})],
                tools: &[],
            },
            receiver,
            |_| panic!("text after cancellation"),
        )
        .await;
    assert_eq!(result.err(), Some(TransportError::Cancelled));
}
