use crate::client::EurekaClient;
use crate::error::Result;
use crate::models::InstanceStatus;
use clap::Subcommand;
use std::str::FromStr;

#[derive(Subcommand, Debug)]
pub enum StatusCommands {
    /// Set instance status
    Set {
        /// Application ID
        app_id: String,
        /// Instance ID
        instance_id: String,
        /// Status (UP, DOWN, OUT_OF_SERVICE, STARTING, UNKNOWN)
        status: String,
    },
    /// Remove status override
    #[command(visible_alias = "rm")]
    Remove {
        /// Application ID
        app_id: String,
        /// Instance ID
        instance_id: String,
    },
}

impl StatusCommands {
    pub async fn execute(&self, client: &EurekaClient) -> Result<()> {
        match self {
            StatusCommands::Set {
                app_id,
                instance_id,
                status,
            } => {
                let status = InstanceStatus::from_str(status)?;
                client.update_status(app_id, instance_id, status).await?;
                println!("Status updated for {}/{}", app_id, instance_id);
                Ok(())
            }
            StatusCommands::Remove {
                app_id,
                instance_id,
            } => {
                client.remove_status_override(app_id, instance_id).await?;
                println!("Status override removed for {}/{}", app_id, instance_id);
                Ok(())
            }
        }
    }
}
