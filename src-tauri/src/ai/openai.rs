//! OpenAI Responses transport for the Sign in with ChatGPT HTTP preview.
use reqwest::{Client, StatusCode, header};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use tokio::sync::watch;

const RESPONSES: &str = "https://api.openai.com/v1/responses";
const MAX_REQUEST_BYTES: usize = 512 * 1024;
const MAX_EVENT_BYTES: usize = 1024 * 1024;
const MAX_STREAM_BYTES: usize = 4 * 1024 * 1024;
const MAX_TEXT_BYTES: usize = 256 * 1024;
const MAX_CALLS: usize = 8;
const MAX_ARGUMENT_BYTES: usize = 16 * 1024;

/// Native input built from approved context and the previous round's output.
pub struct RoundRequest<'a> {
    pub model: &'a str,
    pub instructions: &'a str,
    pub input: &'a [Value],
    pub tools: &'a [Value],
}

#[derive(Debug, PartialEq, Eq)]
pub enum TransportError {
    InvalidRequest,
    Unauthorized,
    RateLimited,
    Unavailable,
    Incomplete,
    InvalidStream,
    LimitExceeded,
    Cancelled,
}

/// A model request to the application broker. Receipt confers no authority.
pub struct FunctionCall {
    pub call_id: String,
    pub name: String,
    pub namespace: Option<String>,
    pub arguments: Value,
}

pub struct RoundOutput {
    pub text: String,
    /// Completion items are needed for stateless function-call continuation.
    pub items: Vec<Value>,
    pub calls: Vec<FunctionCall>,
}

pub struct ResponsesClient {
    client: Client,
}

impl ResponsesClient {
    pub fn new() -> Result<Self, TransportError> {
        let client = Client::builder()
            .https_only(true)
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy()
            .connect_timeout(Duration::from_secs(10))
            .read_timeout(Duration::from_secs(30))
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|_| TransportError::Unavailable)?;
        Ok(Self { client })
    }

    pub async fn stream(
        &self,
        access_token: &str,
        request: RoundRequest<'_>,
        mut cancelled: watch::Receiver<bool>,
        on_text: impl FnMut(&str) -> bool,
    ) -> Result<RoundOutput, TransportError> {
        tokio::select! {
            biased;
            _ = cancelled.wait_for(|value| *value) => Err(TransportError::Cancelled),
            result = self.receive(access_token, request, on_text) => result,
        }
    }

    async fn receive(
        &self,
        access_token: &str,
        request: RoundRequest<'_>,
        mut on_text: impl FnMut(&str) -> bool,
    ) -> Result<RoundOutput, TransportError> {
        let body = encode(request)?;
        if access_token.is_empty() || access_token.len() > 16 * 1024 {
            return Err(TransportError::Unauthorized);
        }
        let mut auth = header::HeaderValue::from_str(&format!("Bearer {access_token}"))
            .map_err(|_| TransportError::Unauthorized)?;
        auth.set_sensitive(true);
        let mut response = self
            .client
            .post(RESPONSES)
            .header(header::AUTHORIZATION, auth)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::ACCEPT, "text/event-stream")
            .body(body)
            .send()
            .await
            .map_err(|_| TransportError::Unavailable)?;
        match response.status() {
            StatusCode::OK => {}
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                return Err(TransportError::Unauthorized);
            }
            StatusCode::TOO_MANY_REQUESTS => return Err(TransportError::RateLimited),
            status if status.is_client_error() => return Err(TransportError::InvalidRequest),
            _ => return Err(TransportError::Unavailable),
        }
        let is_event_stream = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(';').next())
            .is_some_and(|value| value.trim().eq_ignore_ascii_case("text/event-stream"));
        if !is_event_stream {
            return Err(TransportError::InvalidStream);
        }
        let mut decoder = EventStream::default();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| TransportError::Incomplete)?
        {
            if let Some(output) = decoder.feed(&chunk, &mut on_text)? {
                return Ok(output);
            }
        }
        Err(TransportError::Incomplete)
    }
}

fn encode(request: RoundRequest<'_>) -> Result<Vec<u8>, TransportError> {
    #[derive(Serialize)]
    struct Wire<'a> {
        model: &'a str,
        instructions: &'a str,
        input: &'a [Value],
        #[serde(skip_serializing_if = "slice_is_empty")]
        tools: &'a [Value],
        stream: bool,
        store: bool,
    }

    fn slice_is_empty(values: &[Value]) -> bool {
        values.is_empty()
    }

    if request.model.is_empty()
        || request.model.len() > 128
        || !request
            .model
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.:".contains(&b))
        || request.input.is_empty()
        || request.input.len() > 128
        || request.tools.len() > MAX_CALLS
        || request.instructions.len() > 16 * 1024
    {
        return Err(TransportError::InvalidRequest);
    }
    let encoded = serde_json::to_vec(&Wire {
        model: request.model,
        instructions: request.instructions,
        input: request.input,
        tools: request.tools,
        stream: true,
        store: false,
    })
    .map_err(|_| TransportError::InvalidRequest)?;
    if encoded.len() > MAX_REQUEST_BYTES {
        return Err(TransportError::LimitExceeded);
    }
    Ok(encoded)
}

#[derive(Default)]
struct EventStream {
    pending_line: Vec<u8>,
    data: Vec<u8>,
    total: usize,
    text: String,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ServerEvent {
    #[serde(rename = "response.output_text.delta")]
    Text { delta: String },
    #[serde(rename = "response.completed")]
    Complete { response: Completion },
    #[serde(rename = "response.failed")]
    Failed,
    #[serde(rename = "response.incomplete")]
    Incomplete,
    #[serde(rename = "error")]
    Error,
    #[serde(other)]
    Metadata,
}

#[derive(Deserialize)]
struct Completion {
    status: String,
    output: Vec<Value>,
}

impl EventStream {
    fn feed(
        &mut self,
        chunk: &[u8],
        notify: &mut impl FnMut(&str) -> bool,
    ) -> Result<Option<RoundOutput>, TransportError> {
        self.total = self.total.saturating_add(chunk.len());
        if self.total > MAX_STREAM_BYTES {
            return Err(TransportError::LimitExceeded);
        }
        for part in chunk.split_inclusive(|byte| *byte == b'\n') {
            if self.pending_line.len() + part.len() > MAX_EVENT_BYTES {
                return Err(TransportError::LimitExceeded);
            }
            if let Some(line) = part.strip_suffix(b"\n") {
                let mut buffer = std::mem::take(&mut self.pending_line);
                buffer.extend_from_slice(line);
                let line = buffer.strip_suffix(b"\r").unwrap_or(&buffer);
                if let Some(output) = self.line(line, notify)? {
                    return Ok(Some(output));
                }
                buffer.clear();
                self.pending_line = buffer;
            } else {
                self.pending_line.extend_from_slice(part);
            }
        }
        Ok(None)
    }

    fn line(
        &mut self,
        line: &[u8],
        notify: &mut impl FnMut(&str) -> bool,
    ) -> Result<Option<RoundOutput>, TransportError> {
        if let Some(value) = line.strip_prefix(b"data:") {
            let value = value.strip_prefix(b" ").unwrap_or(value);
            if self.data.len() + value.len() + 1 > MAX_EVENT_BYTES {
                return Err(TransportError::LimitExceeded);
            }
            self.data.extend_from_slice(value);
            self.data.push(b'\n');
        }
        if !line.is_empty() || self.data.is_empty() {
            return Ok(None);
        }
        let event =
            serde_json::from_slice(&self.data).map_err(|_| TransportError::InvalidStream)?;
        self.data.clear();
        match event {
            ServerEvent::Text { delta } => {
                if self.text.len() + delta.len() > MAX_TEXT_BYTES {
                    return Err(TransportError::LimitExceeded);
                }
                if !notify(&delta) {
                    return Err(TransportError::Cancelled);
                }
                self.text.push_str(&delta);
                Ok(None)
            }
            ServerEvent::Complete { response } => self.complete(response).map(Some),
            ServerEvent::Failed | ServerEvent::Incomplete | ServerEvent::Error => {
                Err(TransportError::Incomplete)
            }
            ServerEvent::Metadata => Ok(None),
        }
    }

    fn complete(&mut self, response: Completion) -> Result<RoundOutput, TransportError> {
        if response.status != "completed" {
            return Err(TransportError::Incomplete);
        }
        let mut calls: Vec<FunctionCall> = Vec::new();
        for item in &response.output {
            if item.get("type").and_then(Value::as_str) != Some("function_call") {
                continue;
            }
            if calls.len() == MAX_CALLS {
                return Err(TransportError::LimitExceeded);
            }
            let call = parse_call(item)?;
            if calls.iter().any(|old| old.call_id == call.call_id) {
                return Err(TransportError::InvalidStream);
            }
            calls.push(call);
        }
        Ok(RoundOutput {
            text: std::mem::take(&mut self.text),
            items: response.output,
            calls,
        })
    }
}

fn parse_call(item: &Value) -> Result<FunctionCall, TransportError> {
    #[derive(Deserialize)]
    struct Incoming<'a> {
        call_id: &'a str,
        name: &'a str,
        #[serde(default)]
        namespace: Option<&'a str>,
        arguments: &'a str,
    }
    let call = Incoming::deserialize(item).map_err(|_| TransportError::InvalidStream)?;
    let valid_name = |name: &str| {
        !name.is_empty()
            && name.len() <= 128
            && name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
    };
    if !valid_name(call.call_id)
        || !valid_name(call.name)
        || call.namespace.is_some_and(|name| !valid_name(name))
    {
        return Err(TransportError::InvalidStream);
    }
    if call.arguments.len() > MAX_ARGUMENT_BYTES {
        return Err(TransportError::LimitExceeded);
    }
    let arguments: Value =
        serde_json::from_str(call.arguments).map_err(|_| TransportError::InvalidStream)?;
    if !arguments.is_object() {
        return Err(TransportError::InvalidStream);
    }
    Ok(FunctionCall {
        call_id: call.call_id.to_owned(),
        name: call.name.to_owned(),
        namespace: call.namespace.map(str::to_owned),
        arguments,
    })
}

#[cfg(test)]
mod tests;
