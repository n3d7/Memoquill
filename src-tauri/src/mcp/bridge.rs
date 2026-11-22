//! Packaged stdio adapter. This process has no course manager or storage ports.
use super::wire::{self, IO_TIMEOUT, REPLY_LIMIT, REQUEST_LIMIT, Reply, Request};
use crate::workspace::mcp_reads::CourseRead;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
    model::{
        CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation,
        InitializeRequestParams, InitializeResult, ListResourceTemplatesResult, ListToolsResult,
        PaginatedRequestParams, ProtocolVersion, ReadResourceRequestParams, ReadResourceResponse,
        ReadResourceResult, ResourceContents, ResourceTemplate, ServerCapabilities, ServerConfig,
        Tool,
    },
    service::RequestContext,
};
use serde_json::{Value, json};
use std::{borrow::Cow, io, sync::Arc};
use tokio::{net::UnixStream, sync::Mutex, time::timeout};
use uuid::Uuid;

const RESOURCE_PREFIX: &str = "memoquill://resource/";

struct Bridge {
    socket: Mutex<Option<UnixStream>>,
}

const SUPPORTED_VERSIONS: &[ProtocolVersion] = &[
    ProtocolVersion::V_2026_07_28,
    ProtocolVersion::V_2025_11_25,
    ProtocolVersion::V_2025_06_18,
];

pub(super) fn run() -> Result<(), &'static str> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| "Memoquill MCP could not start its runtime.")?
        .block_on(async {
            let service = Bridge {
                socket: Mutex::new(None),
            };
            let active = service
                .serve(rmcp::transport::stdio())
                .await
                .map_err(|_| "Memoquill MCP initialization ended.")?;
            active
                .waiting()
                .await
                .map_err(|_| "Memoquill MCP session ended.")?;
            Ok(())
        })
}

impl ServerHandler for Bridge {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
        )
        .with_server_info(Implementation::new("Memoquill", env!("CARGO_PKG_VERSION")))
        .with_protocol_version(ProtocolVersion::V_2026_07_28)
    }

    fn supported_protocol_versions(&self) -> Cow<'static, [ProtocolVersion]> {
        Cow::Borrowed(SUPPORTED_VERSIONS)
    }

    async fn initialize(
        &self,
        request: InitializeRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        if !SUPPORTED_VERSIONS.contains(&request.protocol_version) {
            return Err(McpError::unsupported_protocol_version(
                request.protocol_version,
                SUPPORTED_VERSIONS,
            ));
        }
        context.peer.set_peer_info(request.clone());
        self.negotiate_initialize(&request)
    }

    async fn list_tools(
        &self,
        _: Option<PaginatedRequestParams>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult::with_all_items(tools()))
    }

    async fn list_resource_templates(
        &self,
        _: Option<PaginatedRequestParams>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        Ok(ListResourceTemplatesResult::with_all_items(vec![
            ResourceTemplate::new(format!("{RESOURCE_PREFIX}{{id}}"), "Published resource")
                .with_mime_type("application/json"),
        ]))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, McpError> {
        let uri = request.uri;
        let id = uri
            .strip_prefix(RESOURCE_PREFIX)
            .and_then(|key| Uuid::parse_str(key).ok())
            .ok_or_else(|| McpError::invalid_params("Expected a Memoquill resource ID.", None))?;
        let reply = self
            .exchange_cancelable(
                Request::Read {
                    request: CourseRead::Resource {
                        id,
                        revision: None,
                        offset: 0,
                    },
                },
                &context,
            )
            .await?;
        match reply {
            Reply::Success { result } => Ok(ReadResourceResult::new(vec![
                ResourceContents::text(result.to_string(), uri).with_mime_type("application/json"),
            ])
            .into()),
            Reply::Error { code, message } => Err(McpError::invalid_params(
                message,
                Some(json!({"code": code})),
            )),
        }
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, McpError> {
        let args = request.arguments.unwrap_or_default();
        let wire_request = parse(request.name.as_ref(), args)?;
        let result = match self.exchange_cancelable(wire_request, &context).await? {
            Reply::Success { result } => CallToolResult::structured(result),
            Reply::Error { code, message } => failure(&code, &message),
        };
        Ok(result.into())
    }
}

impl Bridge {
    async fn exchange_cancelable(
        &self,
        request: Request,
        context: &RequestContext<RoleServer>,
    ) -> Result<Reply, McpError> {
        // Incomplete I/O drops the socket. Pair again to avoid consuming a stale
        // response after cancellation.
        let reply = tokio::select! {
            biased;
            _ = context.ct.cancelled() => return Err(McpError::invalid_request("Request cancelled.", None)),
            reply = self.exchange(request) => reply,
        };
        Ok(match reply {
            Ok(reply) => reply,
            Err(error) if error.kind() == io::ErrorKind::NotConnected => super::broker::error(
                "app_not_running",
                "Open Memoquill and enable Connections, then call memoquill_connection.",
            ),
            Err(_) => super::broker::error(
                "connection_lost",
                "The connection ended or was revoked. Call memoquill_connection to pair again.",
            ),
        })
    }

    async fn exchange(&self, request: Request) -> io::Result<Reply> {
        let bytes = serde_json::to_vec(&request).map_err(|_| io::ErrorKind::InvalidData)?;
        if bytes.len() > REQUEST_LIMIT {
            return Ok(super::broker::error(
                "budget_exceeded",
                "Reduce the request size.",
            ));
        }
        timeout(IO_TIMEOUT, async {
            let mut socket = self.socket.lock().await;
            let mut active = match socket.take() {
                Some(stream) => stream,
                None => {
                    let path = wire::socket_path().map_err(|_| io::ErrorKind::NotConnected)?;
                    wire::verify_dir(path.parent().ok_or(io::ErrorKind::NotConnected)?)
                        .map_err(|_| io::ErrorKind::NotConnected)?;
                    let stream = UnixStream::connect(path)
                        .await
                        .map_err(|_| io::ErrorKind::NotConnected)?;
                    if stream.peer_cred()?.uid() != wire::uid() {
                        return Err(io::ErrorKind::PermissionDenied.into());
                    }
                    stream
                }
            };
            wire::send(&mut active, &bytes).await?;
            let reply = wire::receive(&mut active, REPLY_LIMIT).await?;
            *socket = Some(active);
            Ok(reply)
        })
        .await
        .map_err(|_| io::ErrorKind::TimedOut)?
    }
}

fn failure(code: &str, message: &str) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(
        json!({"code": code, "message": message}).to_string(),
    )])
}

fn parse(name: &str, mut args: serde_json::Map<String, Value>) -> Result<Request, McpError> {
    if name == "memoquill_course_draft" {
        check_contract(&mut args)?;
        let request = serde_json::from_value(Value::Object(args)).map_err(|_| {
            McpError::invalid_params("Arguments do not match the course draft schema.", None)
        })?;
        return Ok(Request::CourseDraft { request });
    }
    if name == "memoquill_lesson_draft" {
        check_contract(&mut args)?;
        let request = serde_json::from_value(Value::Object(args)).map_err(|_| {
            McpError::invalid_params("Arguments do not match the draft tool schema.", None)
        })?;
        return Ok(Request::Draft { request });
    }

    let operation = match name {
        "memoquill_connection" if args.is_empty() => return Ok(Request::Status),
        "memoquill_capabilities" => "capabilities",
        "memoquill_structure_get" => "structure",
        "memoquill_content_search" => "search",
        "memoquill_registry_list" => "registry",
        "memoquill_resource_read" => "resource",
        "memoquill_lesson_get" => "lesson",
        "memoquill_progress_get" => "progress",
        "memoquill_source_get" => "source",
        "memoquill_source_original" => "source_original",
        _ => {
            return Err(McpError::invalid_params(
                "Unknown tool or invalid arguments.",
                None,
            ));
        }
    };
    if args.contains_key("operation") {
        return Err(McpError::invalid_params("Unexpected argument.", None));
    }
    check_contract(&mut args)?;
    if operation == "capabilities" && !args.is_empty() {
        return Err(McpError::invalid_params("Unexpected argument.", None));
    }
    if matches!(
        operation,
        "registry" | "resource" | "source" | "source_original" | "progress"
    ) {
        args.entry("offset").or_insert(json!(0));
    }
    if operation == "progress" {
        args.entry("includeResponses").or_insert(json!(false));
    }
    args.insert("operation".into(), json!(operation));
    let request = serde_json::from_value::<CourseRead>(Value::Object(args))
        .map_err(|_| McpError::invalid_params("Arguments do not match the tool schema.", None))?;
    Ok(Request::Read { request })
}

fn check_contract(args: &mut serde_json::Map<String, Value>) -> Result<(), McpError> {
    if args
        .remove("contractVersion")
        .is_some_and(|version| version != json!(1))
    {
        return Err(McpError::invalid_params(
            "Unsupported application contract. Use contractVersion 1.",
            None,
        ));
    }
    Ok(())
}

fn draft_tool(id: &Value) -> Tool {
    let operation = json!({"type":"string","format":"uuid"});
    let revision = json!({"type":"integer","minimum":1,"maximum":9007199254740991_u64});
    let object = |kind: &str, properties: Value, fields: &[&str]| {
        let mut properties = properties;
        properties["kind"] = json!({"const":kind});
        let mut required = vec!["kind"];
        required.extend_from_slice(fields);
        json!({"type":"object","additionalProperties":false,"required":required,"properties":properties})
    };
    tool(
        "memoquill_lesson_draft",
        "Manage an unpublished draft for a lesson: create, list, read, edit, validate, preview or discard. Requires a draft grant. Keep the same randomly generated v4 ownerKey private to resume.",
        json!({
            "unitId":id, "ownerKey":id,
            "action":{"oneOf":[
                object("list", json!({}), &[]),
                object("materials", json!({"offset":{"type":"integer","minimum":0}}), &["offset"]),
                object("sources", json!({"resourceId":id,"offset":{"type":"integer","minimum":0}}), &["resourceId","offset"]),
                object("create", json!({"operationId":operation,"expectedHeadRevision":id}), &["operationId"]),
                object("read", json!({"draftId":id}), &["draftId"]),
                object("edit", json!({"draftId":id,"revision":revision,"operationId":operation,
                    "operations":{"type":"array","minItems":1,"maxItems":64,"items":{"type":"object"}}}),
                    &["draftId","revision","operationId","operations"]),
                object("validate", json!({"draftId":id,"revision":revision}), &["draftId","revision"]),
                object("preview", json!({"draftId":id,"revision":revision}), &["draftId","revision"]),
                object("discard", json!({"draftId":id,"revision":revision,"operationId":operation}),
                    &["draftId","revision","operationId"]),
            ]},
        }),
        &["unitId", "ownerKey", "action"],
    )
}

fn course_draft_tool(id: &Value) -> Tool {
    let revision = json!({"type":"integer","minimum":0,"maximum":9007199254740991_u64});
    let offset = json!({"type":"integer","minimum":0,"maximum":4294967295_u64});
    let action = |kind: &str, properties: Value, fields: &[&str]| {
        let mut properties = properties;
        properties["kind"] = json!({"const":kind});
        let mut required = vec!["kind"];
        required.extend_from_slice(fields);
        json!({"type":"object","additionalProperties":false,"required":required,"properties":properties})
    };
    let finish = json!({"draftId":id,"revision":revision,"operationId":id});
    let finish_fields = &["draftId", "revision", "operationId"];
    tool(
        "memoquill_course_draft",
        "Stage resumable course organization and document batches. List, read, explore effective structure, validate, submit for review or discard.",
        json!({"ownerKey":id,"action":{"oneOf":[
            action("create", json!({"draftId":id,"title":{"type":"string","maxLength":256},"baseRevision":revision}), &["draftId","title","baseRevision"]),
            action("fork", json!({"fromId":id,"fromRevision":revision,"draftId":id,
                "title":{"type":"string","maxLength":256},"baseRevision":revision,"keepChunks":offset}),
                &["fromId","fromRevision","draftId","title","baseRevision","keepChunks"]),
            action("list", json!({}), &[]),
            action("read", json!({"draftId":id,"revision":revision,"offset":offset}), &["draftId","offset"]),
            action("structure", json!({"draftId":id,"revision":revision,"offset":offset,
                "category":{"type":"string","enum":["subjects","goals","collections","units","resources","paths","placements","relations"]}}),
                &["draftId","revision","offset","category"]),
            action("append", json!({"draftId":id,"revision":revision,"operationId":id,"operations":{
                "type":"array","minItems":1,"maxItems":64,"items":{"type":"object"}}}),
                &["draftId","revision","operationId","operations"]),
            action("validate", json!({"draftId":id,"revision":revision}), &["draftId","revision"]),
            action("submit", finish.clone(), finish_fields),
            action("apply_delegated", json!({"draftId":id,"revision":revision,"operationId":id,"hash":{"type":"string","pattern":"^[0-9a-f]{64}$"}}),
                &["draftId","revision","operationId","hash"]),
            action("discard", finish, finish_fields),
        ]}}),
        &["ownerKey", "action"],
    )
}

fn tools() -> Vec<Tool> {
    let id = json!({"type":"string", "format":"uuid"});
    let offset = json!({"type":"integer","minimum":0,"maximum":4294967295_u64,"default":0});

    vec![
        draft_tool(&id),
        course_draft_tool(&id),
        tool(
            "memoquill_connection",
            "Get the pairing challenge or current approval status.",
            json!({}),
            &[],
        ),
        tool(
            "memoquill_capabilities",
            "Get approved course and read capabilities (contract 1).",
            json!({}),
            &[],
        ),
        tool(
            "memoquill_structure_get",
            "Read one page of published course structure. Pass the previous nextCursor as cursor when continuing.",
            json!({
                "kind":{"type":"string","enum":["subjects","goals","collections","units","resources","paths","placements","relations"]},
                "cursor":id,
            }),
            &["kind"],
        ),
        tool(
            "memoquill_content_search",
            "Search published titles and normalized text. Continue with the same query and its nextCursor.",
            json!({
                "query":{"type":"string","minLength":1,"maxLength":256}, "cursor":id,
            }),
            &["query"],
        ),
        tool(
            "memoquill_registry_list",
            "Discover versioned lesson components, events and actions.",
            json!({"offset":offset}),
            &[],
        ),
        tool(
            "memoquill_resource_read",
            "Read a page of normalized document text. Continue with the returned revision and nextOffset.",
            json!({"id":id, "revision":id, "offset":offset}),
            &["id"],
        ),
        tool(
            "memoquill_lesson_get",
            "Read published lesson components and permitted source bindings.",
            json!({"id":id}),
            &["id"],
        ),
        tool(
            "memoquill_source_get",
            "Read extracted records for a source bound to a published lesson.",
            json!({"id":id, "revision":id, "offset":offset}),
            &["id", "revision"],
        ),
        tool(
            "memoquill_source_original",
            "Read a hex-encoded chunk of a lesson source with original-byte permission.",
            json!({"id":id, "revision":id, "offset":offset}),
            &["id", "revision"],
        ),
        tool(
            "memoquill_progress_get",
            "Read aggregate lesson progress. Raw responses require an additional grant.",
            json!({"id":id, "offset":offset, "includeResponses":{"type":"boolean","default":false}}),
            &["id"],
        ),
    ]
}

fn tool(
    name: &'static str,
    description: &'static str,
    mut properties: Value,
    required: &[&str],
) -> Tool {
    if name != "memoquill_connection" {
        properties["contractVersion"] = json!({"type":"integer","const":1,"default":1});
    }
    let Value::Object(schema) = json!({
        "type":"object", "additionalProperties":false, "properties":properties, "required":required,
    }) else {
        unreachable!()
    };
    Tool::new(name, description, Arc::new(schema))
}
