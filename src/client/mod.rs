mod traits;

pub use traits::EurekaService;

use crate::error::{Error, Result};
use crate::models::*;
use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;
use std::time::Duration;
use tracing::{debug, info, warn};

pub struct EurekaClient {
    client: Client,
    base_url: String,
}

impl EurekaClient {
    pub fn new(base_url: impl Into<String>, timeout: Duration) -> Result<Self> {
        let base_url = base_url.into();
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(Error::HttpError)?;

        Ok(Self { client, base_url })
    }

    fn build_url(&self, path: &str) -> String {
        format!("{}/{}", self.base_url.trim_end_matches('/'), path)
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = self.build_url(path);
        debug!("GET {}", url);

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?;

        self.handle_response(response).await
    }

    async fn put(&self, path: &str) -> Result<()> {
        let url = self.build_url(path);
        debug!("PUT {}", url);

        let response = self.client.put(&url).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::server_error(status, message))
        }
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let url = self.build_url(path);
        debug!("DELETE {}", url);

        let response = self.client.delete(&url).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::server_error(status, message))
        }
    }

    async fn handle_response<T: DeserializeOwned>(&self, response: reqwest::Response) -> Result<T> {
        let status = response.status();

        if status.is_success() {
            let body = response.text().await?;
            debug!("Response body: {}", body);
            serde_json::from_str(&body).map_err(Error::SerializationError)
        } else {
            let message = response.text().await.unwrap_or_default();
            Err(Error::server_error(status.as_u16(), message))
        }
    }

    // API Methods

    /// Get all applications
    pub async fn get_applications(&self) -> Result<ApplicationsWrapper> {
        self.get("apps").await
    }

    /// Get a specific application
    pub async fn get_application(&self, app_id: &str) -> Result<Application> {
        let wrapper: ApplicationWrapper = self.get(&format!("apps/{}", app_id)).await?;
        Ok(wrapper.application)
    }

    /// Get all instances of an application
    pub async fn get_app_instances(&self, app_id: &str) -> Result<Application> {
        let wrapper: ApplicationWrapper = self.get(&format!("apps/{}", app_id)).await?;
        Ok(wrapper.application)
    }

    /// Get a specific instance
    pub async fn get_instance(&self, app_id: &str, instance_id: &str) -> Result<Instance> {
        let wrapper: InstanceWrapper = self
            .get(&format!("apps/{}/{}", app_id, instance_id))
            .await?;
        Ok(wrapper.instance)
    }

    /// Get instance by instance ID only
    pub async fn get_instance_by_id(&self, instance_id: &str) -> Result<Instance> {
        let wrapper: InstanceWrapper = self.get(&format!("instances/{}", instance_id)).await?;
        Ok(wrapper.instance)
    }

    /// Register a new instance
    pub async fn register_instance(&self, app_id: &str, instance: &Instance) -> Result<()> {
        let response = self
            .client
            .post(self.build_url(&format!("apps/{}", app_id)))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({ "instance": instance }))
            .send()
            .await?;

        if response.status() == StatusCode::NO_CONTENT || response.status().is_success() {
            info!("Instance registered successfully");
            Ok(())
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::server_error(status, message))
        }
    }

    /// Deregister an instance
    pub async fn deregister_instance(&self, app_id: &str, instance_id: &str) -> Result<()> {
        self.delete(&format!("apps/{}/{}", app_id, instance_id))
            .await
    }

    /// Send heartbeat
    pub async fn send_heartbeat(&self, app_id: &str, instance_id: &str) -> Result<()> {
        self.put(&format!("apps/{}/{}", app_id, instance_id)).await
    }

    /// Update instance status
    pub async fn update_status(
        &self,
        app_id: &str,
        instance_id: &str,
        status: InstanceStatus,
    ) -> Result<()> {
        use reqwest::Url;
        let base = self.build_url(&format!("apps/{}/{}/status", app_id, instance_id));
        let mut url =
            Url::parse(&base).map_err(|e| Error::ConfigError(format!("Invalid URL: {}", e)))?;
        url.query_pairs_mut()
            .append_pair("value", &status.to_string());

        let response = self.client.put(url).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status_code = response.status().as_u16();
            let message = response.text().await.unwrap_or_else(|e| {
                warn!("Failed to read error response: {}", e);
                "Failed to read error response".to_string()
            });
            Err(Error::server_error(status_code, message))
        }
    }

    /// Remove status override
    pub async fn remove_status_override(&self, app_id: &str, instance_id: &str) -> Result<()> {
        self.delete(&format!("apps/{}/{}/status", app_id, instance_id))
            .await
    }

    /// Update metadata
    pub async fn update_metadata(
        &self,
        app_id: &str,
        instance_id: &str,
        key: &str,
        value: &str,
    ) -> Result<()> {
        use reqwest::Url;
        let base = self.build_url(&format!("apps/{}/{}/metadata", app_id, instance_id));
        let mut url =
            Url::parse(&base).map_err(|e| Error::ConfigError(format!("Invalid URL: {}", e)))?;
        url.query_pairs_mut().append_pair(key, value);

        let response = self.client.put(url).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_else(|e| {
                warn!("Failed to read error response: {}", e);
                "Failed to read error response".to_string()
            });
            Err(Error::server_error(status, message))
        }
    }

    /// Query by VIP address
    pub async fn get_vip(&self, vip_address: &str) -> Result<ApplicationsWrapper> {
        self.get(&format!("vips/{}", vip_address)).await
    }

    /// Query by secure VIP address
    pub async fn get_secure_vip(&self, svip_address: &str) -> Result<ApplicationsWrapper> {
        self.get(&format!("svips/{}", svip_address)).await
    }
}

// Implement the EurekaService trait for EurekaClient
#[async_trait]
impl EurekaService for EurekaClient {
    async fn get_applications(&self) -> Result<ApplicationsWrapper> {
        self.get_applications().await
    }

    async fn get_application(&self, app_id: &str) -> Result<Application> {
        self.get_application(app_id).await
    }

    async fn get_app_instances(&self, app_id: &str) -> Result<Application> {
        self.get_app_instances(app_id).await
    }

    async fn get_instance(&self, app_id: &str, instance_id: &str) -> Result<Instance> {
        self.get_instance(app_id, instance_id).await
    }

    async fn get_instance_by_id(&self, instance_id: &str) -> Result<Instance> {
        self.get_instance_by_id(instance_id).await
    }

    async fn register_instance(&self, app_id: &str, instance: &Instance) -> Result<()> {
        self.register_instance(app_id, instance).await
    }

    async fn deregister_instance(&self, app_id: &str, instance_id: &str) -> Result<()> {
        self.deregister_instance(app_id, instance_id).await
    }

    async fn send_heartbeat(&self, app_id: &str, instance_id: &str) -> Result<()> {
        self.send_heartbeat(app_id, instance_id).await
    }

    async fn update_status(
        &self,
        app_id: &str,
        instance_id: &str,
        status: InstanceStatus,
    ) -> Result<()> {
        self.update_status(app_id, instance_id, status).await
    }

    async fn remove_status_override(&self, app_id: &str, instance_id: &str) -> Result<()> {
        self.remove_status_override(app_id, instance_id).await
    }

    async fn update_metadata(
        &self,
        app_id: &str,
        instance_id: &str,
        key: &str,
        value: &str,
    ) -> Result<()> {
        self.update_metadata(app_id, instance_id, key, value).await
    }

    async fn get_vip(&self, vip_address: &str) -> Result<ApplicationsWrapper> {
        self.get_vip(vip_address).await
    }

    async fn get_secure_vip(&self, svip_address: &str) -> Result<ApplicationsWrapper> {
        self.get_secure_vip(svip_address).await
    }
}
