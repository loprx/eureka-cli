use crate::client::EurekaService;
use crate::error::Result;
use clap::Args;

#[derive(Args, Debug)]
pub struct HeartbeatArgs {
    /// Application ID
    pub app_id: String,
    /// Instance ID
    pub instance_id: String,
}

impl HeartbeatArgs {
    pub async fn execute(&self, client: &impl EurekaService, output_format: &str) -> Result<()> {
        client
            .send_heartbeat(&self.app_id, &self.instance_id)
            .await?;

        let message = format!("Heartbeat sent for {}/{}", self.app_id, self.instance_id);
        super::super::output::print_success(&message, output_format)?;

        Ok(())
    }
}
