pub mod commands;
pub mod format;
pub mod output;
pub mod query;
pub mod selector;

use crate::client::EurekaClient;
use crate::config::AppConfig;
use crate::error::Result;
use clap::{Parser, Subcommand};
use std::time::Duration;

pub use format::OutputFormat;
pub use selector::Selector;

#[derive(Parser, Debug)]
#[command(name = "eureka-cli")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Eureka server (name from config or direct URL)
    /// Examples: --server test1, --server http://localhost:8761/eureka
    #[arg(short, long, env = "EUREKA_SERVER")]
    pub server: Option<String>,

    /// Output format: table, wide, json, yaml, jsonpath=<expr>
    #[arg(short, long, default_value = "table", value_parser = clap::value_parser!(OutputFormat))]
    pub output: OutputFormat,

    /// Selector expression for filtering (e.g., status=UP,app=foo)
    #[arg(short = 'l', long)]
    pub selector: Option<String>,

    /// Watch mode: continuously poll and refresh output
    #[arg(short = 'w', long)]
    pub watch: bool,

    /// Watch interval in seconds (default: 2)
    #[arg(long, default_value = "2")]
    pub watch_interval: u64,

    /// Sort output by field (e.g., status, ip_addr, instance_id)
    #[arg(long)]
    pub sort_by: Option<String>,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Quiet mode
    #[arg(short, long)]
    pub quiet: bool,

    /// Request timeout in seconds
    #[arg(long, default_value = "30")]
    pub timeout: u64,

    /// Config file path
    #[arg(long)]
    pub config: Option<String>,

    /// Profile to use
    #[arg(short, long)]
    pub profile: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Application operations
    #[command(visible_aliases = ["a", "app"])]
    Apps {
        #[command(subcommand)]
        command: commands::AppsCommands,
    },
    /// Instance operations
    #[command(visible_aliases = ["i", "inst"])]
    Instances {
        #[command(subcommand)]
        command: commands::InstancesCommands,
    },
    /// Manage server configurations
    #[command(visible_aliases = ["s", "srv"])]
    Servers {
        #[command(subcommand)]
        command: commands::ServersCommands,
    },
    /// Register a new service instance
    #[command(visible_alias = "reg")]
    Register {
        #[command(flatten)]
        args: commands::RegisterArgs,
    },
    /// Deregister a service instance
    #[command(visible_alias = "dereg")]
    Deregister {
        #[command(flatten)]
        args: commands::DeregisterArgs,
    },
    /// Send heartbeat for an instance
    #[command(visible_alias = "hb")]
    Heartbeat {
        #[command(flatten)]
        args: commands::HeartbeatArgs,
    },
    /// Manage instance status
    #[command(visible_alias = "st")]
    Status {
        #[command(subcommand)]
        command: commands::StatusCommands,
    },
    /// Update instance metadata
    #[command(visible_aliases = ["meta", "md"])]
    Metadata {
        #[command(subcommand)]
        command: commands::MetadataCommands,
    },
    /// Query by VIP address
    Vip {
        #[command(subcommand)]
        command: commands::VipCommands,
    },
    /// Show version information
    #[command(visible_alias = "v")]
    Version,
}

impl Cli {
    pub async fn execute(&self) -> Result<()> {
        // Handle servers command separately (doesn't need client)
        if let Commands::Servers { command } = &self.command {
            return command.execute().await;
        }

        let config = AppConfig::load()?;
        let server_url = self
            .server
            .as_deref()
            .map(|s| config.get_server_url(Some(s)))
            .unwrap_or_else(|| config.get_server_url(self.profile.as_deref()))?;

        let timeout = Duration::from_secs(self.timeout);
        let client = EurekaClient::new(server_url, timeout)?;

        match &self.command {
            Commands::Apps { command } => {
                let opts = query::QueryOptions::new(
                    &self.output,
                    self.selector.as_deref(),
                    self.sort_by.clone(),
                )?;
                command.execute(&client, &opts).await
            }
            Commands::Instances { command } => {
                let opts = query::QueryOptions::new(
                    &self.output,
                    self.selector.as_deref(),
                    self.sort_by.clone(),
                )?;
                command.execute(&client, &opts).await
            }
            Commands::Servers { .. } => unreachable!(), // Already handled above
            Commands::Register { args } => args.execute(&client).await,
            Commands::Deregister { args } => {
                args.execute(&client, self.output.as_legacy_str()).await
            }
            Commands::Heartbeat { args } => {
                args.execute(&client, self.output.as_legacy_str()).await
            }
            Commands::Status { command } => command.execute(&client).await,
            Commands::Metadata { command } => command.execute(&client).await,
            Commands::Vip { command } => {
                command.execute(&client, self.output.as_legacy_str()).await
            }
            Commands::Version => {
                output::print_success(
                    &format!("eureka-cli {}", env!("CARGO_PKG_VERSION")),
                    self.output.as_legacy_str(),
                )?;
                Ok(())
            }
        }
    }
}
