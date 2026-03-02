//! WebSocket client that connects to a gateway as a headless node.

use std::{process::Stdio, time::Duration};

use futures::{SinkExt, StreamExt};
use tokio::process::Command;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

use moltis_protocol::{
    ConnectAuth, ConnectParamsV4, ClientInfo, GatewayFrame, ProtocolRange, RequestFrame,
    ResponseFrame, PROTOCOL_VERSION, roles,
};

/// Configuration for connecting a node to a gateway.
#[derive(Debug, Clone)]
pub struct NodeConfig {
    /// Gateway WebSocket URL (e.g. `ws://localhost:9090/ws`).
    pub gateway_url: String,
    /// Device token obtained from the pairing flow.
    pub device_token: String,
    /// Unique node identifier.
    pub node_id: String,
    /// Human-readable display name.
    pub display_name: Option<String>,
    /// Platform string (e.g. "macos", "linux").
    pub platform: String,
    /// Capabilities this node advertises (e.g. "system.run").
    pub caps: Vec<String>,
    /// Commands this node supports (e.g. "system.run", "system.which").
    pub commands: Vec<String>,
    /// Maximum time for a single command execution.
    pub exec_timeout: Duration,
    /// Working directory for commands (defaults to $HOME).
    pub working_dir: Option<String>,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            gateway_url: "ws://localhost:9090/ws".into(),
            device_token: String::new(),
            node_id: uuid::Uuid::new_v4().to_string(),
            display_name: None,
            platform: std::env::consts::OS.into(),
            caps: vec!["system.run".into(), "system.which".into()],
            commands: vec!["system.run".into(), "system.which".into()],
            exec_timeout: Duration::from_secs(300),
            working_dir: None,
        }
    }
}

/// A headless node host that connects to a gateway and handles commands.
pub struct NodeHost {
    config: NodeConfig,
}

impl NodeHost {
    pub fn new(config: NodeConfig) -> Self {
        Self { config }
    }

    /// Connect to the gateway and run the message loop until disconnected.
    ///
    /// Returns `Ok(())` on clean shutdown, `Err` on connection/protocol errors.
    pub async fn run(&self) -> anyhow::Result<()> {
        // Validate URL first, then pass string to connect_async.
        let _url = url::Url::parse(&self.config.gateway_url)?;
        info!(url = %self.config.gateway_url, node_id = %self.config.node_id, "connecting to gateway");

        let (ws_stream, _response) = connect_async(&self.config.gateway_url).await?;
        let (mut ws_tx, mut ws_rx) = ws_stream.split();

        info!("websocket connected, sending handshake");

        // Build and send connect request (v4 format).
        let connect_id = uuid::Uuid::new_v4().to_string();
        let connect_params = ConnectParamsV4 {
            protocol: ProtocolRange {
                min: PROTOCOL_VERSION,
                max: PROTOCOL_VERSION,
            },
            client: ClientInfo {
                id: self.config.node_id.clone(),
                display_name: self.config.display_name.clone(),
                version: env!("CARGO_PKG_VERSION").into(),
                platform: self.config.platform.clone(),
                device_family: None,
                model_identifier: None,
                mode: "headless".into(),
                instance_id: None,
            },
            role: Some(roles::NODE.into()),
            scopes: None,
            auth: Some(ConnectAuth {
                token: None,
                password: None,
                api_key: None,
                device_token: Some(self.config.device_token.clone()),
            }),
            locale: None,
            timezone: None,
            extensions: {
                let mut ext = std::collections::HashMap::new();
                ext.insert(
                    "moltis".into(),
                    serde_json::json!({
                        "caps": self.config.caps,
                        "commands": self.config.commands,
                    }),
                );
                ext
            },
        };

        let connect_req = RequestFrame {
            r#type: "req".into(),
            id: connect_id.clone(),
            method: "connect".into(),
            params: Some(serde_json::to_value(&connect_params)?),
            channel: None,
        };

        let connect_json = serde_json::to_string(&connect_req)?;
        ws_tx.send(Message::Text(connect_json.into())).await?;

        // Wait for hello-ok response.
        let hello = match tokio::time::timeout(Duration::from_secs(10), ws_rx.next()).await {
            Ok(Some(Ok(Message::Text(text)))) => {
                let resp: ResponseFrame = serde_json::from_str(&text)?;
                if resp.id != connect_id {
                    anyhow::bail!("unexpected response id: expected {connect_id}, got {}", resp.id);
                }
                if !resp.ok {
                    let err_msg = resp
                        .error
                        .map(|e| format!("{}: {}", e.code, e.message))
                        .unwrap_or_else(|| "unknown error".into());
                    anyhow::bail!("handshake failed: {err_msg}");
                }
                resp
            },
            Ok(Some(Ok(_))) => anyhow::bail!("unexpected non-text message during handshake"),
            Ok(Some(Err(e))) => anyhow::bail!("websocket error during handshake: {e}"),
            Ok(None) => anyhow::bail!("connection closed during handshake"),
            Err(_) => anyhow::bail!("handshake timeout"),
        };

        info!(
            server_version = hello.payload.as_ref()
                .and_then(|p| p.get("server"))
                .and_then(|s| s.get("version"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown"),
            "handshake complete, node registered"
        );

        // Main message loop.
        loop {
            tokio::select! {
                msg = ws_rx.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            self.handle_message(&text, &mut ws_tx).await;
                        },
                        Some(Ok(Message::Ping(data))) => {
                            if let Err(e) = ws_tx.send(Message::Pong(data)).await {
                                warn!(error = %e, "failed to send pong");
                                break;
                            }
                        },
                        Some(Ok(Message::Close(_))) => {
                            info!("gateway closed connection");
                            break;
                        },
                        Some(Ok(_)) => {},
                        Some(Err(e)) => {
                            error!(error = %e, "websocket error");
                            break;
                        },
                        None => {
                            info!("websocket stream ended");
                            break;
                        },
                    }
                },
            }
        }

        info!("node disconnected");
        Ok(())
    }

    async fn handle_message(
        &self,
        text: &str,
        ws_tx: &mut (impl SinkExt<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin),
    ) {
        // Try to parse as a GatewayFrame to determine the type.
        let frame: GatewayFrame = match serde_json::from_str(text) {
            Ok(f) => f,
            Err(e) => {
                debug!(error = %e, "failed to parse frame, ignoring");
                return;
            },
        };

        match frame {
            GatewayFrame::Event(event) => {
                if event.event == "node.invoke.request" {
                    self.handle_invoke(event.payload, ws_tx).await;
                } else {
                    debug!(event = %event.event, "ignoring event");
                }
            },
            GatewayFrame::Request(req) => {
                debug!(method = %req.method, id = %req.id, "ignoring server request");
            },
            GatewayFrame::Response(_) => {
                debug!("ignoring response frame");
            },
        }
    }

    async fn handle_invoke(
        &self,
        payload: Option<serde_json::Value>,
        ws_tx: &mut (impl SinkExt<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin),
    ) {
        let payload = match payload {
            Some(p) => p,
            None => return,
        };

        let invoke_id = match payload.get("invokeId").and_then(|v| v.as_str()) {
            Some(id) => id.to_string(),
            None => {
                warn!("invoke request missing invokeId");
                return;
            },
        };

        let command = match payload.get("command").and_then(|v| v.as_str()) {
            Some(c) => c,
            None => {
                warn!(invoke_id = %invoke_id, "invoke request missing command");
                self.send_invoke_error(&invoke_id, "missing command", ws_tx).await;
                return;
            },
        };

        let args = payload.get("args").cloned().unwrap_or_default();

        info!(invoke_id = %invoke_id, command = %command, "handling invoke");

        let result = match command {
            "system.run" => self.handle_system_run(&args).await,
            "system.which" => self.handle_system_which(&args).await,
            other => {
                warn!(command = %other, "unsupported invoke command");
                Err(anyhow::anyhow!("unsupported command: {other}"))
            },
        };

        match result {
            Ok(value) => {
                self.send_invoke_result(&invoke_id, value, ws_tx).await;
            },
            Err(e) => {
                self.send_invoke_error(&invoke_id, &e.to_string(), ws_tx).await;
            },
        }
    }

    async fn handle_system_run(&self, args: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let command = args
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing 'command' in args"))?;

        let timeout_ms = args
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(self.config.exec_timeout.as_millis() as u64);
        let timeout = Duration::from_millis(timeout_ms);

        let cwd = args
            .get("cwd")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or_else(|| self.config.working_dir.clone());

        info!(command = %command, timeout_ms, "system.run");

        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(command);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.stdin(Stdio::null());

        if let Some(ref dir) = cwd {
            cmd.current_dir(dir);
        }

        // Apply environment from args if provided.
        if let Some(env_obj) = args.get("env").and_then(|v| v.as_object()) {
            for (k, v) in env_obj {
                if let Some(val) = v.as_str() {
                    cmd.env(k, val);
                }
            }
        }

        let child = cmd.spawn()?;
        let result = tokio::time::timeout(timeout, child.wait_with_output()).await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
                let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
                let exit_code = output.status.code().unwrap_or(-1);

                Ok(serde_json::json!({
                    "stdout": stdout,
                    "stderr": stderr,
                    "exitCode": exit_code,
                }))
            },
            Ok(Err(e)) => Err(anyhow::anyhow!("failed to execute command: {e}")),
            Err(_) => Err(anyhow::anyhow!("command timed out after {timeout_ms}ms")),
        }
    }

    async fn handle_system_which(&self, args: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let binary = args
            .get("binary")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing 'binary' in args"))?;

        let output = Command::new("which")
            .arg(binary)
            .output()
            .await?;

        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let found = output.status.success();

        Ok(serde_json::json!({
            "found": found,
            "path": if found { Some(path) } else { None },
        }))
    }

    async fn send_invoke_result(
        &self,
        invoke_id: &str,
        result: serde_json::Value,
        ws_tx: &mut (impl SinkExt<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin),
    ) {
        let frame = serde_json::json!({
            "type": "req",
            "id": uuid::Uuid::new_v4().to_string(),
            "method": "node.invoke.result",
            "params": {
                "invokeId": invoke_id,
                "result": result,
            }
        });

        if let Ok(json) = serde_json::to_string(&frame) {
            if let Err(e) = ws_tx.send(Message::Text(json.into())).await {
                warn!(invoke_id = %invoke_id, error = %e, "failed to send invoke result");
            }
        }
    }

    async fn send_invoke_error(
        &self,
        invoke_id: &str,
        error: &str,
        ws_tx: &mut (impl SinkExt<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin),
    ) {
        let frame = serde_json::json!({
            "type": "req",
            "id": uuid::Uuid::new_v4().to_string(),
            "method": "node.invoke.result",
            "params": {
                "invokeId": invoke_id,
                "result": { "error": error },
            }
        });

        if let Ok(json) = serde_json::to_string(&frame) {
            if let Err(e) = ws_tx.send(Message::Text(json.into())).await {
                warn!(invoke_id = %invoke_id, error = %e, "failed to send invoke error");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn default_config_has_system_run_cap() {
        let config = NodeConfig::default();
        assert!(config.caps.contains(&"system.run".to_string()));
        assert!(config.commands.contains(&"system.run".to_string()));
    }

    #[test]
    fn default_config_platform_is_current_os() {
        let config = NodeConfig::default();
        assert_eq!(config.platform, std::env::consts::OS);
    }

    #[tokio::test]
    async fn system_which_finds_sh() {
        let config = NodeConfig::default();
        let host = NodeHost::new(config);
        let args = serde_json::json!({ "binary": "sh" });
        let result = host.handle_system_which(&args).await.unwrap();
        assert_eq!(result["found"], true);
        assert!(result["path"].as_str().unwrap().contains("sh"));
    }

    #[tokio::test]
    async fn system_which_missing_binary() {
        let config = NodeConfig::default();
        let host = NodeHost::new(config);
        let args = serde_json::json!({ "binary": "definitely_not_a_real_binary_123456" });
        let result = host.handle_system_which(&args).await.unwrap();
        assert_eq!(result["found"], false);
    }

    #[tokio::test]
    async fn system_run_echo() {
        let config = NodeConfig::default();
        let host = NodeHost::new(config);
        let args = serde_json::json!({
            "command": "echo hello",
            "timeout": 5000,
        });
        let result = host.handle_system_run(&args).await.unwrap();
        assert_eq!(result["stdout"].as_str().unwrap().trim(), "hello");
        assert_eq!(result["exitCode"], 0);
    }

    #[tokio::test]
    async fn system_run_captures_stderr() {
        let config = NodeConfig::default();
        let host = NodeHost::new(config);
        let args = serde_json::json!({
            "command": "echo err >&2",
            "timeout": 5000,
        });
        let result = host.handle_system_run(&args).await.unwrap();
        assert_eq!(result["stderr"].as_str().unwrap().trim(), "err");
    }

    #[tokio::test]
    async fn system_run_exit_code() {
        let config = NodeConfig::default();
        let host = NodeHost::new(config);
        let args = serde_json::json!({
            "command": "exit 42",
            "timeout": 5000,
        });
        let result = host.handle_system_run(&args).await.unwrap();
        assert_eq!(result["exitCode"], 42);
    }
}
