use {anyhow::Result, clap::Subcommand, std::time::Duration};

/// `moltis node` subcommands — run or manage a headless node.
#[derive(Subcommand)]
pub enum NodeAction {
    /// Connect to a gateway as a headless node and handle remote commands.
    Run {
        /// Gateway WebSocket URL (e.g. ws://localhost:9090/ws).
        #[arg(long, env = "MOLTIS_GATEWAY_URL")]
        host: String,
        /// Device token from the pairing flow.
        #[arg(long, env = "MOLTIS_DEVICE_TOKEN")]
        token: String,
        /// Custom node ID (defaults to a random UUID).
        #[arg(long)]
        node_id: Option<String>,
        /// Display name for this node.
        #[arg(long)]
        name: Option<String>,
        /// Working directory for command execution.
        #[arg(long)]
        working_dir: Option<String>,
        /// Maximum command timeout in seconds.
        #[arg(long, default_value = "300")]
        timeout: u64,
    },
}

pub async fn handle_node(action: NodeAction) -> Result<()> {
    match action {
        NodeAction::Run {
            host,
            token,
            node_id,
            name,
            working_dir,
            timeout,
        } => {
            let config = moltis_node_host::NodeConfig {
                gateway_url: host,
                device_token: token,
                node_id: node_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                display_name: name,
                platform: std::env::consts::OS.into(),
                caps: vec!["system.run".into(), "system.which".into()],
                commands: vec!["system.run".into(), "system.which".into()],
                exec_timeout: Duration::from_secs(timeout),
                working_dir,
            };

            let host = moltis_node_host::NodeHost::new(config);
            host.run().await
        },
    }
}
