use {anyhow::Result, clap::Subcommand};

/// `moltis nodes` subcommands — view connected nodes on a gateway.
#[derive(Subcommand)]
pub enum NodesAction {
    /// List all connected nodes.
    List {
        /// Gateway HTTP URL (defaults to http://localhost:9090).
        #[arg(long, default_value = "http://localhost:9090")]
        host: String,
        /// API key or password for authentication.
        #[arg(long, env = "MOLTIS_API_KEY")]
        api_key: Option<String>,
    },
}

pub async fn handle_nodes(action: NodesAction) -> Result<()> {
    match action {
        NodesAction::List { host, api_key } => {
            let url = format!("{}/api/nodes", host.trim_end_matches('/'));
            let client = reqwest::Client::new();
            let mut req = client.get(&url);

            if let Some(key) = api_key {
                req = req.header("Authorization", format!("Bearer {key}"));
            }

            let resp = req.send().await?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("failed to list nodes: {status} — {body}");
            }

            let nodes: serde_json::Value = resp.json().await?;

            if let Some(arr) = nodes.as_array() {
                if arr.is_empty() {
                    println!("No nodes connected.");
                    return Ok(());
                }

                for node in arr {
                    let id = node.get("nodeId").and_then(|v| v.as_str()).unwrap_or("?");
                    let name = node
                        .get("displayName")
                        .and_then(|v| v.as_str())
                        .unwrap_or("(unnamed)");
                    let platform = node.get("platform").and_then(|v| v.as_str()).unwrap_or("?");
                    let caps = node
                        .get("caps")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                        .unwrap_or_default();

                    println!("{id}  {name}  ({platform})  caps=[{caps}]");
                }
            } else {
                println!("{}", serde_json::to_string_pretty(&nodes)?);
            }

            Ok(())
        },
    }
}
