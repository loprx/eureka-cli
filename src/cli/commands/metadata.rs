use crate::client::EurekaClient;
use crate::error::Result;
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum MetadataCommands {
    /// Set metadata key-value pair
    Set {
        /// Application ID
        app_id: String,
        /// Instance ID
        instance_id: String,
        /// Metadata key
        key: String,
        /// Metadata value
        value: String,
    },
}

impl MetadataCommands {
    pub async fn execute(&self, client: &EurekaClient) -> Result<()> {
        match self {
            MetadataCommands::Set {
                app_id,
                instance_id,
                key,
                value,
            } => {
                client
                    .update_metadata(app_id, instance_id, key, value)
                    .await?;
                println!(
                    "Metadata updated for {}/{}: {}={}",
                    app_id, instance_id, key, value
                );
                Ok(())
            }
        }
    }
}
