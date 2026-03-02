use {anyhow::Result, clap::Subcommand, std::time::Duration};

/// `moltis node` subcommands — connect this machine as a node to a gateway.
#[derive(Subcommand)]
pub enum NodeAction {
    /// Register this machine as a node for a gateway.
    ///
    /// Saves the connection parameters and installs an OS service (launchd on
    /// macOS, systemd on Linux) that starts on boot and reconnects on failure.
    /// Pass --foreground to run in the current terminal instead.
    Add {
        /// Gateway WebSocket URL (e.g. ws://your-server:9090/ws).
        #[arg(long, env = "MOLTIS_GATEWAY_URL")]
        host: String,
        /// Device token from the pairing flow.
        #[arg(long, env = "MOLTIS_DEVICE_TOKEN")]
        token: String,
        /// Display name for this node.
        #[arg(long)]
        name: Option<String>,
        /// Custom node ID (defaults to a random UUID).
        #[arg(long)]
        node_id: Option<String>,
        /// Working directory for command execution.
        #[arg(long)]
        working_dir: Option<String>,
        /// Maximum command timeout in seconds.
        #[arg(long, default_value = "300")]
        timeout: u64,
        /// Run in the foreground instead of installing as a service.
        #[arg(long)]
        foreground: bool,
    },

    /// Remove this machine as a node and uninstall the background service.
    Remove,

    /// Show the current node connection status.
    Status,

    /// Print the path to the node log file.
    Logs,
}

pub async fn handle_node(action: NodeAction) -> Result<()> {
    match action {
        NodeAction::Add {
            host,
            token,
            name,
            node_id,
            working_dir,
            timeout,
            foreground,
        } => {
            let resolved_node_id = node_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

            if foreground {
                let config = moltis_node_host::NodeConfig {
                    gateway_url: host,
                    device_token: token,
                    node_id: resolved_node_id,
                    display_name: name,
                    platform: std::env::consts::OS.into(),
                    caps: vec![
                        "system.run".into(),
                        "system.which".into(),
                        "system.providers".into(),
                    ],
                    commands: vec![
                        "system.run".into(),
                        "system.which".into(),
                        "system.providers".into(),
                    ],
                    exec_timeout: Duration::from_secs(timeout),
                    working_dir,
                };

                let node = moltis_node_host::NodeHost::new(config);
                node.run().await
            } else {
                let data_dir = moltis_config::data_dir();
                let svc_config = moltis_node_host::ServiceConfig {
                    gateway_url: host,
                    device_token: token,
                    node_id: Some(resolved_node_id),
                    display_name: name,
                    working_dir,
                    timeout,
                };

                moltis_node_host::service::install(&data_dir, &svc_config)?;
                println!("Node registered and service started.");
                println!(
                    "Logs: {}",
                    moltis_node_host::service::log_path(&data_dir).display()
                );
                Ok(())
            }
        },

        NodeAction::Remove => {
            let data_dir = moltis_config::data_dir();
            moltis_node_host::service::uninstall(&data_dir)?;
            println!("Node removed.");
            Ok(())
        },

        NodeAction::Status => {
            let data_dir = moltis_config::data_dir();
            let config_path = data_dir.join("node.json");

            if !config_path.exists() {
                println!("Not registered as a node.");
                return Ok(());
            }

            let config = moltis_node_host::ServiceConfig::load(&data_dir)?;
            let status = moltis_node_host::service::status()?;

            println!("Gateway: {}", config.gateway_url);
            if let Some(ref name) = config.display_name {
                println!("Name:    {name}");
            }
            println!("Service: {status}");
            Ok(())
        },

        NodeAction::Logs => {
            let data_dir = moltis_config::data_dir();
            println!(
                "{}",
                moltis_node_host::service::log_path(&data_dir).display()
            );
            Ok(())
        },
    }
}
