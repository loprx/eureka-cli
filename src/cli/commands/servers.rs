use crate::config::AppConfig;
use crate::error::Result;
use clap::Subcommand;
use colored::Colorize;
use comfy_table::{presets::NOTHING, Cell, ContentArrangement, Table};

#[derive(Subcommand, Debug)]
pub enum ServersCommands {
    /// List all configured servers
    #[command(visible_alias = "ls")]
    List,
    /// Show current default server
    #[command(visible_alias = "cur")]
    Current,
    /// Set default server
    Use {
        /// Server name to set as default
        name: String,
    },
    /// Add a new server
    Add {
        /// Server name
        name: String,
        /// Server URL
        url: String,
        /// Optional description
        #[arg(short, long)]
        description: Option<String>,
        /// Set as default
        #[arg(short = 'D', long)]
        set_default: bool,
    },
    /// Remove a server
    #[command(visible_alias = "rm")]
    Remove {
        /// Server name to remove
        name: String,
    },
}

impl ServersCommands {
    pub async fn execute(&self) -> Result<()> {
        let mut config = AppConfig::load()?;

        match self {
            ServersCommands::List => {
                let servers = config.list_servers();
                if servers.is_empty() {
                    println!("No servers configured.");
                    return Ok(());
                }

                let mut table = Table::new();
                table.load_preset(NOTHING);
                table.set_content_arrangement(ContentArrangement::Disabled);
                table.set_header(vec!["NAME", "URL", "DESCRIPTION", "DEFAULT"]);

                for server in servers {
                    let name = if server.is_default {
                        server.name.green().bold().to_string()
                    } else {
                        server.name
                    };
                    let default_mark = if server.is_default {
                        "✓".green().to_string()
                    } else {
                        "-".to_string()
                    };
                    table.add_row(vec![
                        Cell::new(name),
                        Cell::new(server.url),
                        Cell::new(server.description.unwrap_or_else(|| "-".to_string())),
                        Cell::new(default_mark),
                    ]);
                }

                println!("{}", table);
            }
            ServersCommands::Current => {
                let default_name = &config.server.default;
                if let Some(server) = config.server.servers.get(default_name) {
                    println!("Current default server: {}", default_name.green().bold());
                    println!("  URL: {}", server.url);
                    if let Some(desc) = &server.description {
                        println!("  Description: {}", desc);
                    }
                } else {
                    println!(
                        "Default server '{}' not found in config!",
                        default_name.red()
                    );
                }
            }
            ServersCommands::Use { name } => {
                config.set_default_server(name)?;
                config.save()?;
                println!("Default server set to: {}", name.green().bold());
            }
            ServersCommands::Add {
                name,
                url,
                description,
                set_default,
            } => {
                config.add_server(name.clone(), url.clone(), description.clone())?;
                if *set_default {
                    config.set_default_server(name)?;
                }
                config.save()?;
                println!("Server '{}' added successfully", name.green());
                if *set_default {
                    println!("Set as default server");
                }
            }
            ServersCommands::Remove { name } => {
                config.remove_server(name)?;
                config.save()?;
                println!("Server '{}' removed successfully", name.green());
            }
        }

        Ok(())
    }
}
