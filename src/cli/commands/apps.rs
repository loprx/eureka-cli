use crate::client::EurekaClient;
use crate::error::Result;
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum AppsCommands {
    /// List all applications
    #[command(visible_alias = "ls")]
    List,
    /// Get application details
    Get {
        /// Application ID
        app_id: String,
    },
    /// List instances of an application
    #[command(visible_aliases = ["i", "inst"])]
    Instances {
        /// Application ID
        app_id: String,
    },
}

impl AppsCommands {
    pub async fn execute(&self, client: &EurekaClient, output_format: &str) -> Result<()> {
        match self {
            AppsCommands::List => {
                let apps = client.get_applications().await?;
                super::super::output::print_applications(&apps, output_format)?;
                Ok(())
            }
            AppsCommands::Get { app_id } => {
                let app = client.get_application(app_id).await?;
                super::super::output::print_application(&app, output_format)?;
                Ok(())
            }
            AppsCommands::Instances { app_id } => {
                let app = client.get_app_instances(app_id).await?;
                super::super::output::print_instances(&app.instance, output_format)?;
                Ok(())
            }
        }
    }
}
