use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

/// Helper: deserialize bool from either bool or string ("true"/"false")
fn deserialize_string_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let v = serde_json::Value::deserialize(deserializer)?;
    match v {
        serde_json::Value::Bool(b) => Ok(b),
        serde_json::Value::String(s) => s.parse::<bool>().map_err(D::Error::custom),
        _ => Err(D::Error::custom("expected bool or string")),
    }
}

fn serialize_bool_as_string<S>(value: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(if *value { "true" } else { "false" })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    pub instance_id: String,
    pub host_name: String,
    pub app: String,
    pub ip_addr: String,
    pub status: InstanceStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overriddenstatus: Option<InstanceStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<Port>,
    pub secure_port: Port,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country_id: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub home_page_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_page_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub health_check_url: Option<String>,
    #[serde(default)]
    pub vip_address: String,
    #[serde(default)]
    pub secure_vip_address: String,
    pub data_center_info: DataCenterInfo,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lease_info: Option<LeaseInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "isCoordinatingDiscoveryServer"
    )]
    pub is_coordinating_discovery_server: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_updated_timestamp: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_dirty_timestamp: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_type: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InstanceStatus {
    Up,
    Down,
    Starting,
    OutOfService,
    Unknown,
}

impl std::fmt::Display for InstanceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstanceStatus::Up => write!(f, "UP"),
            InstanceStatus::Down => write!(f, "DOWN"),
            InstanceStatus::Starting => write!(f, "STARTING"),
            InstanceStatus::OutOfService => write!(f, "OUT_OF_SERVICE"),
            InstanceStatus::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

impl std::str::FromStr for InstanceStatus {
    type Err = crate::error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "UP" => Ok(InstanceStatus::Up),
            "DOWN" => Ok(InstanceStatus::Down),
            "STARTING" => Ok(InstanceStatus::Starting),
            "OUT_OF_SERVICE" => Ok(InstanceStatus::OutOfService),
            "UNKNOWN" => Ok(InstanceStatus::Unknown),
            _ => Err(crate::error::Error::InvalidStatus(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    #[serde(rename = "$")]
    pub port: u16,
    #[serde(
        rename = "@enabled",
        deserialize_with = "deserialize_string_bool",
        serialize_with = "serialize_bool_as_string"
    )]
    pub enabled: bool,
}

impl Port {
    pub fn new(port: u16, enabled: bool) -> Self {
        Self { port, enabled }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataCenterInfo {
    #[serde(default = "default_data_center_class", rename = "@class")]
    pub class: String,
    pub name: DataCenterName,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<AmazonMetadata>,
}

fn default_data_center_class() -> String {
    "com.netflix.appinfo.InstanceInfo$DefaultDataCenterInfo".to_string()
}

impl Default for DataCenterInfo {
    fn default() -> Self {
        Self {
            class: default_data_center_class(),
            name: DataCenterName::MyOwn,
            metadata: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataCenterName {
    MyOwn,
    Amazon,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AmazonMetadata {
    pub ami_launch_index: String,
    pub local_hostname: String,
    pub availability_zone: String,
    pub instance_id: String,
    pub public_ipv4: String,
    pub public_hostname: String,
    pub ami_manifest_path: String,
    pub local_ipv4: String,
    pub hostname: String,
    pub ami_id: String,
    pub instance_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaseInfo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eviction_duration_in_secs: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub renewal_interval_in_secs: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_in_secs: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registration_timestamp: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_renewal_timestamp: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eviction_timestamp: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_up_timestamp: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
    pub name: String,
    #[serde(default)]
    pub instance: Vec<Instance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationWrapper {
    pub application: Application,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceWrapper {
    pub instance: Instance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Applications {
    #[serde(default, rename = "versions__delta")]
    pub versions_delta: Option<String>,
    #[serde(default, rename = "apps__hashcode")]
    pub apps_hashcode: Option<String>,
    #[serde(default, rename = "application")]
    pub apps: Vec<Application>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationsWrapper {
    pub applications: Applications,
}
