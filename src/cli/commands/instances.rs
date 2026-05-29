use crate::client::EurekaClient;
use crate::error::Result;
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum InstancesCommands {
    /// List all instances
    #[command(visible_alias = "ls")]
    List,
    /// Get instance by ID
    Get {
        /// Instance ID
        instance_id: String,
        /// Application ID (optional)
        #[arg(short, long)]
        app_id: Option<String>,
    },
}

impl InstancesCommands {
    pub async fn execute(&self, client: &EurekaClient, output_format: &str) -> Result<()> {
        match self {
            InstancesCommands::List => {
                let apps = client.get_applications().await?;
                let instances: Vec<_> = apps
                    .applications
                    .apps
                    .iter()
                    .flat_map(|app| app.instance.clone())
                    .collect();
                super::super::output::print_instances(&instances, output_format)?;
                Ok(())
            }
            InstancesCommands::Get {
                instance_id,
                app_id,
            } => {
                let instance = if let Some(app) = app_id {
                    client.get_instance(app, instance_id).await?
                } else {
                    client.get_instance_by_id(instance_id).await?
                };
                super::super::output::print_instance(&instance, output_format)?;
                Ok(())
            }
        }
    }
}
