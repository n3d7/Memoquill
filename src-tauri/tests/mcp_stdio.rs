#![cfg(target_os = "linux")]

use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Write},
    os::unix::fs::PermissionsExt,
    process::{Child, ChildStdout, Command, Stdio},
};

struct Client {
    child: Child,
    stdout: BufReader<ChildStdout>,
}

impl Client {
    fn start(runtime: &std::path::Path) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_memoquill"))
            .arg("mcp")
            .env("XDG_RUNTIME_DIR", runtime)
            .env_remove("DISPLAY")
            .env_remove("WAYLAND_DISPLAY")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        Self { child, stdout }
    }

    fn send(&mut self, message: Value) {
        let input = self.child.stdin.as_mut().unwrap();
        writeln!(input, "{message}").unwrap();
        input.flush().unwrap();
    }

    fn request(&mut self, id: u32, method: &str, params: Value) -> Value {
        self.send(json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}));
        let mut line = String::new();
        assert_ne!(self.stdout.read_line(&mut line).unwrap(), 0);
        let response: Value = serde_json::from_str(&line).expect("stdio wrote non-protocol output");
        assert_eq!(response["id"], id);
        response
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn runtime() -> tempfile::TempDir {
    let runtime = tempfile::tempdir().unwrap();
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    runtime
}

#[test]
fn mcp_stdio_legacy_discovery_and_closed_app() {
    legacy_discovery_and_closed_app("2025-11-25");
}

#[test]
fn mcp_stdio_codex_legacy_discovery_and_closed_app() {
    legacy_discovery_and_closed_app("2025-06-18");
}

fn legacy_discovery_and_closed_app(version: &str) {
    let runtime = runtime();
    let mut client = Client::start(runtime.path());
    let init = client.request(
        1,
        "initialize",
        json!({
            "protocolVersion": version,
            "capabilities": {},
            "clientInfo": {"name": "Qualification fixture", "version": "1"}
        }),
    );
    assert_eq!(init["result"]["protocolVersion"], version);
    client.send(json!({"jsonrpc": "2.0", "method": "notifications/initialized"}));
    let discovery = client.request(2, "tools/list", json!({}));
    let tools = discovery["result"]["tools"].as_array().unwrap();
    assert!(
        tools
            .iter()
            .any(|tool| tool["name"] == "memoquill_connection")
    );
    let call = client.request(
        3,
        "tools/call",
        json!({"name": "memoquill_connection", "arguments": {}}),
    );
    assert_eq!(call["result"]["isError"], true);
    assert!(
        call["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("app_not_running")
    );
    assert!(!runtime.path().join("memoquill-mcp").exists());
}

fn metadata(version: &str) -> Value {
    json!({
        "io.modelcontextprotocol/protocolVersion": version,
        "io.modelcontextprotocol/clientInfo": {"name": "Qualification fixture", "version": "1"},
        "io.modelcontextprotocol/clientCapabilities": {}
    })
}

#[test]
fn mcp_stdio_modern_discovery_and_tool_validation() {
    let runtime = runtime();
    let mut client = Client::start(runtime.path());
    let meta = metadata("2026-07-28");
    let discovery = client.request(1, "tools/list", json!({"_meta": meta}));
    let tools = discovery["result"]["tools"].as_array().unwrap();
    assert!(
        tools
            .iter()
            .any(|tool| tool["name"] == "memoquill_structure_get")
    );
    let call = client.request(
        2,
        "tools/call",
        json!({
            "_meta": meta,
            "name": "memoquill_structure_get",
            "arguments": {"kind": "units", "courseId": "different-course"}
        }),
    );
    assert_eq!(call["error"]["code"], -32602);
    let future = client.request(3, "tools/list", json!({"_meta": metadata("2099-01-01")}));
    assert!(future.get("error").is_some());
}

#[test]
fn mcp_stdio_unknown_handshake_revision_has_a_clear_error() {
    let runtime = runtime();
    let mut client = Client::start(runtime.path());
    let reply = client.request(
        1,
        "initialize",
        json!({
            "protocolVersion": "2099-01-01",
            "capabilities": {},
            "clientInfo": {"name": "Unsupported client fixture", "version": "1"}
        }),
    );
    assert_eq!(reply["error"]["message"], "Unsupported protocol version");
    assert_eq!(reply["error"]["data"]["requested"], "2099-01-01");
    assert!(
        reply["error"]["data"]["supported"]
            .as_array()
            .unwrap()
            .contains(&json!("2025-11-25"))
    );
}
