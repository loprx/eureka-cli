use crate::config::AppConfig;
use crate::error::Result;
use clap::Subcommand;
use colored::Colorize;
use comfy_table::{presets::NOTHING, Attribute, Cell, Color, ContentArrangement, Table};

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
                    let mut name_cell = Cell::new(&server.name);
                    let mut default_cell = if server.is_default {
                        Cell::new("✓")
                    } else {
                        Cell::new("-")
                    };
                    if server.is_default {
                        name_cell = name_cell.fg(Color::Green).add_attribute(Attribute::Bold);
                        default_cell = default_cell.fg(Color::Green);
                    }

                    table.add_row(vec![
                        name_cell,
                        Cell::new(server.url),
                        Cell::new(server.description.unwrap_or_else(|| "-".to_string())),
                        default_cell,
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
