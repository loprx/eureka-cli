pub mod commands;
pub mod format;
pub mod output;
pub mod query;
pub mod selector;
pub mod watch;

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
    #[arg(short, long, env = "EUREKA_SERVER", global = true)]
    pub server: Option<String>,

    /// Output format: table, wide, json, yaml, jsonpath=<expr>
    #[arg(short, long, default_value = "table", value_parser = clap::value_parser!(OutputFormat), global = true)]
    pub output: OutputFormat,

    /// Selector expression for filtering (e.g., status=UP,app=foo)
    #[arg(short = 'l', long, global = true)]
    pub selector: Option<String>,

    /// Watch mode: continuously poll and refresh output
    #[arg(short = 'w', long, global = true)]
    pub watch: bool,

    /// Watch interval in seconds (default: 2)
    #[arg(long, default_value = "2", global = true)]
    pub watch_interval: u64,

    /// Sort output by field (e.g., status, ip_addr, instance_id)
    #[arg(long, global = true)]
    pub sort_by: Option<String>,

    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Quiet mode
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Request timeout in seconds
    #[arg(long, default_value = "30", global = true)]
    pub timeout: u64,

    /// Config file path
    #[arg(long, global = true)]
    pub config: Option<String>,

    /// Profile to use
    #[arg(short, long, global = true)]
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
    /// Manage server configurations (kubectl/kubeconfig style)
    Config {
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
    /// Generate shell completion scripts
    Completion {
        /// Shell to generate completion for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
    /// Show version information
    #[command(visible_alias = "v")]
    Version,
}

impl Cli {
    pub async fn execute(&self) -> Result<()> {
        // Handle commands that don't need a client.
        match &self.command {
            Commands::Servers { command } => {
                eprintln!(
                    "Note: 'servers' is deprecated, use 'config' instead. Will be removed in v0.4."
                );
                return command.execute().await;
            }
            Commands::Config { command } => return command.execute().await,
            Commands::Completion { shell } => return print_completion(*shell),
            _ => {}
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
                    self.watch,
                    self.watch_interval,
                )?;
                command.execute(&client, &opts).await
            }
            Commands::Instances { command } => {
                let opts = query::QueryOptions::new(
                    &self.output,
                    self.selector.as_deref(),
                    self.sort_by.clone(),
                    self.watch,
                    self.watch_interval,
                )?;
                command.execute(&client, &opts).await
            }
            Commands::Servers { .. } => unreachable!(), // Already handled above
            Commands::Config { .. } => unreachable!(),  // Already handled above
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
            Commands::Completion { .. } => unreachable!(), // handled above
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

fn print_completion(shell: clap_complete::Shell) -> Result<()> {
    use clap::CommandFactory;
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();
    clap_complete::generate(shell, &mut cmd, bin_name, &mut std::io::stdout());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    /// Regression: every global flag must be accepted both BEFORE and AFTER a
    /// subcommand. Without `global = true` on the derive struct, kubectl-style
    /// `eureka-cli instances ls -l ...` fails parsing — the exact bug a user hit
    /// on v0.2.0-rc.
    #[test]
    fn global_flags_work_before_subcommand() {
        Cli::try_parse_from(["eureka-cli", "-l", "status=UP", "instances", "list"]).unwrap();
        Cli::try_parse_from(["eureka-cli", "-o", "wide", "instances", "list"]).unwrap();
        Cli::try_parse_from(["eureka-cli", "--sort-by", "status", "instances", "list"]).unwrap();
        Cli::try_parse_from(["eureka-cli", "-w", "instances", "list"]).unwrap();
    }

    #[test]
    fn global_flags_work_after_subcommand() {
        Cli::try_parse_from(["eureka-cli", "instances", "list", "-l", "status=UP"]).unwrap();
        Cli::try_parse_from(["eureka-cli", "instances", "list", "-o", "wide"]).unwrap();
        Cli::try_parse_from(["eureka-cli", "instances", "list", "--sort-by", "status"]).unwrap();
        Cli::try_parse_from(["eureka-cli", "instances", "list", "-w"]).unwrap();
    }

    #[test]
    fn global_flags_work_after_apps_subcommand() {
        // apps has more subcommands; make sure `-l` lands on each list-style one.
        Cli::try_parse_from(["eureka-cli", "apps", "list", "-l", "status=UP"]).unwrap();
        Cli::try_parse_from(["eureka-cli", "apps", "list", "-o", "wide"]).unwrap();
        Cli::try_parse_from(["eureka-cli", "apps", "instances", "FOO", "-l", "status=UP"]).unwrap();
    }

    #[test]
    fn jsonpath_output_parses() {
        let cli = Cli::try_parse_from([
            "eureka-cli",
            "instances",
            "list",
            "-o",
            "jsonpath=$.instances[*].ipAddr",
        ])
        .unwrap();
        assert!(matches!(cli.output, OutputFormat::JsonPath(_)));
    }

    #[test]
    fn config_subcommand_alias_of_servers_parses() {
        Cli::try_parse_from(["eureka-cli", "config", "list"]).unwrap();
        Cli::try_parse_from(["eureka-cli", "config", "current"]).unwrap();
        Cli::try_parse_from(["eureka-cli", "config", "use", "prod"]).unwrap();
    }

    #[test]
    fn unhealthy_and_describe_subcommands_parse() {
        Cli::try_parse_from(["eureka-cli", "apps", "unhealthy"]).unwrap();
        Cli::try_parse_from(["eureka-cli", "instances", "unhealthy"]).unwrap();
        Cli::try_parse_from(["eureka-cli", "apps", "describe", "FOO"]).unwrap();
        Cli::try_parse_from(["eureka-cli", "instances", "describe", "-a", "FOO", "BAR"]).unwrap();
    }
}
