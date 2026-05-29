use crate::error::Result;
use crate::models::*;
use clap::Args;
use std::collections::HashMap;

#[derive(Args, Debug)]
pub struct RegisterArgs {
    /// Application ID
    #[arg(long)]
    pub app: String,

    /// Instance ID
    #[arg(long)]
    pub instance_id: String,

    /// Hostname
    #[arg(long)]
    pub hostname: String,

    /// IP address
    #[arg(long)]
    pub ip: String,

    /// Port number
    #[arg(long)]
    pub port: u16,

    /// Secure port number
    #[arg(long, default_value = "443")]
    pub secure_port: u16,

    /// VIP address
    #[arg(long)]
    pub vip_address: String,

    /// Secure VIP address
    #[arg(long)]
    pub secure_vip_address: Option<String>,

    /// Home page URL
    #[arg(long)]
    pub home_page_url: Option<String>,

    /// Status page URL
    #[arg(long)]
    pub status_page_url: Option<String>,

    /// Health check URL
    #[arg(long)]
    pub health_check_url: Option<String>,

    /// Metadata key=value pairs
    #[arg(long = "metadata", value_parser = parse_key_val)]
    pub metadata: Vec<(String, String)>,
}

pub fn parse_key_val(s: &str) -> Result<(String, String)> {
    let pos = s.find('=').ok_or_else(|| {
        crate::error::Error::Other(format!("invalid KEY=value: no `=` found in `{}`", s))
    })?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

impl RegisterArgs {
    /// Build the Instance to register. Pure function, no I/O.
    pub fn build_instance(&self) -> Instance {
        let metadata = if self.metadata.is_empty() {
            None
        } else {
            Some(
                self.metadata
                    .iter()
                    .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                    .collect::<HashMap<_, _>>(),
            )
        };

        Instance {
            instance_id: self.instance_id.clone(),
            host_name: self.hostname.clone(),
            app: self.app.clone(),
            ip_addr: self.ip.clone(),
            status: InstanceStatus::Up,
            overriddenstatus: None,
            port: Some(Port::new(self.port, true)),
            secure_port: Port::new(self.secure_port, false),
            country_id: Some(1),
            home_page_url: Some(
                self.home_page_url
                    .clone()
                    .unwrap_or_else(|| format!("http://{}:{}/", self.hostname, self.port)),
            ),
            status_page_url: Some(
                self.status_page_url
                    .clone()
                    .unwrap_or_else(|| format!("http://{}:{}/info", self.hostname, self.port)),
            ),
            health_check_url: Some(
                self.health_check_url
                    .clone()
                    .unwrap_or_else(|| format!("http://{}:{}/health", self.hostname, self.port)),
            ),
            vip_address: self.vip_address.clone(),
            secure_vip_address: self
                .secure_vip_address
                .clone()
                .unwrap_or_else(|| self.vip_address.clone()),
            data_center_info: DataCenterInfo::default(),
            lease_info: None,
            metadata,
            is_coordinating_discovery_server: None,
            last_updated_timestamp: None,
            last_dirty_timestamp: None,
            action_type: None,
        }
    }

    pub async fn execute(&self, client: &impl crate::client::EurekaService) -> Result<()> {
        let instance = self.build_instance();
        client.register_instance(&self.app, &instance).await?;
        crate::cli::output::print_success(
            &format!(
                "Instance {}/{} registered successfully",
                self.app, self.instance_id
            ),
            "table",
        )?;
        Ok(())
    }
}
