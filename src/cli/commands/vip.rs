use crate::client::EurekaClient;
use crate::error::Result;
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum VipCommands {
    /// Query by VIP address
    Get {
        /// VIP address
        vip_address: String,
    },
    /// Query by secure VIP address
    #[command(visible_alias = "gs")]
    GetSecure {
        /// Secure VIP address
        svip_address: String,
    },
}

impl VipCommands {
    pub async fn execute(&self, client: &EurekaClient, output_format: &str) -> Result<()> {
        match self {
            VipCommands::Get { vip_address } => {
                let apps = client.get_vip(vip_address).await?;
                super::super::output::print_applications(&apps, output_format)?;
                Ok(())
            }
            VipCommands::GetSecure { svip_address } => {
                let apps = client.get_secure_vip(svip_address).await?;
                super::super::output::print_applications(&apps, output_format)?;
                Ok(())
            }
        }
    }
}
