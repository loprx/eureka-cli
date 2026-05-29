//! Trait abstraction for Eureka service operations
//! This enables testing with mock implementations and future extensibility

use crate::error::Result;
use crate::models::*;
use async_trait::async_trait;

#[async_trait]
pub trait EurekaService: Send + Sync {
    /// Get all applications
    async fn get_applications(&self) -> Result<ApplicationsWrapper>;

    /// Get a specific application
    async fn get_application(&self, app_id: &str) -> Result<Application>;

    /// Get all instances of an application
    async fn get_app_instances(&self, app_id: &str) -> Result<Application>;

    /// Get a specific instance
    async fn get_instance(&self, app_id: &str, instance_id: &str) -> Result<Instance>;

    /// Get instance by instance ID only
    async fn get_instance_by_id(&self, instance_id: &str) -> Result<Instance>;

    /// Register a new instance
    async fn register_instance(&self, app_id: &str, instance: &Instance) -> Result<()>;

    /// Deregister an instance
    async fn deregister_instance(&self, app_id: &str, instance_id: &str) -> Result<()>;

    /// Send heartbeat
    async fn send_heartbeat(&self, app_id: &str, instance_id: &str) -> Result<()>;

    /// Update instance status
    async fn update_status(
        &self,
        app_id: &str,
        instance_id: &str,
        status: InstanceStatus,
    ) -> Result<()>;

    /// Remove status override
    async fn remove_status_override(&self, app_id: &str, instance_id: &str) -> Result<()>;

    /// Update metadata
    async fn update_metadata(
        &self,
        app_id: &str,
        instance_id: &str,
        key: &str,
        value: &str,
    ) -> Result<()>;

    /// Query by VIP address
    async fn get_vip(&self, vip_address: &str) -> Result<ApplicationsWrapper>;

    /// Query by secure VIP address
    async fn get_secure_vip(&self, svip_address: &str) -> Result<ApplicationsWrapper>;
}
