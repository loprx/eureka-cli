use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub output: OutputConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Default server name to use
    pub default: String,
    /// Named server configurations
    pub servers: HashMap<String, EurekaServer>,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default)]
    pub retry: RetryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EurekaServer {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,
    #[serde(default = "default_backoff_ms")]
    pub backoff_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: default_max_attempts(),
            backoff_ms: default_backoff_ms(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    #[serde(default = "default_format")]
    pub format: String,
    #[serde(default = "default_color")]
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    pub file: Option<PathBuf>,
}

fn default_timeout() -> u64 {
    30
}

fn default_max_attempts() -> u32 {
    3
}

fn default_backoff_ms() -> u64 {
    1000
}

fn default_format() -> String {
    "table".to_string()
}

fn default_color() -> String {
    "auto".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut servers = HashMap::new();
        servers.insert(
            "local".to_string(),
            EurekaServer {
                url: "http://localhost:8761/eureka".to_string(),
                description: Some("Local development server".to_string()),
            },
        );
        servers.insert(
            "test1".to_string(),
            EurekaServer {
                url: "http://eureka-test-1.example.com:8761/eureka".to_string(),
                description: Some("Test server 1".to_string()),
            },
        );
        servers.insert(
            "test2".to_string(),
            EurekaServer {
                url: "http://eureka-test-2.example.com:8761/eureka".to_string(),
                description: Some("Test server 2".to_string()),
            },
        );

        Self {
            server: ServerConfig {
                default: "local".to_string(),
                servers,
                timeout: default_timeout(),
                retry: RetryConfig::default(),
            },
            output: OutputConfig {
                format: default_format(),
                color: default_color(),
            },
            logging: LoggingConfig {
                level: default_log_level(),
                file: None,
            },
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let config_dir = dirs::config_dir().ok_or_else(|| {
            crate::error::Error::ConfigError("Cannot find config directory".to_string())
        })?;
        let config_path = config_dir.join("eureka-cli").join("config.yaml");

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).map_err(|e| {
                crate::error::Error::ConfigError(format!("Failed to read config: {}", e))
            })?;
            serde_yaml::from_str(&content).map_err(|e| {
                crate::error::Error::ConfigError(format!("Failed to parse config: {}", e))
            })
        } else {
            Ok(Self::default())
        }
    }

    /// Resolve server URL from name or direct URL
    /// - If server_name is None, use default server
    /// - If server_name starts with http:// or https://, treat as direct URL
    /// - Otherwise, lookup named server in config
    pub fn get_server_url(&self, server_name: Option<&str>) -> Result<String> {
        let (resolved_name, url) = match server_name {
            None => {
                let url = self
                    .server
                    .servers
                    .get(&self.server.default)
                    .map(|s| s.url.clone())
                    .ok_or_else(|| {
                        crate::error::Error::ConfigError(format!(
                            "Default server '{}' not found in config",
                            self.server.default
                        ))
                    })?;
                (self.server.default.clone(), url)
            }
            Some(name) if name.starts_with("http://") || name.starts_with("https://") => {
                return Ok(name.to_string());
            }
            Some(name) => {
                let url = self
                    .server
                    .servers
                    .get(name)
                    .map(|s| s.url.clone())
                    .ok_or_else(|| {
                        crate::error::Error::ConfigError(format!(
                            "Server '{}' not found in config. Available: {}",
                            name,
                            self.list_server_names().join(", ")
                        ))
                    })?;
                (name.to_string(), url)
            }
        };

        // Catch corrupt config (e.g. saved without http:// prefix)
        validate_server_url(&url).map_err(|msg| {
            crate::error::Error::ConfigError(format!(
                "Server '{}' has invalid URL '{}': {}. Fix it with:\n  \
             eureka-cli servers remove {}\n  \
             eureka-cli servers add {} http://{}",
                resolved_name, url, msg, resolved_name, resolved_name, url
            ))
        })?;
        Ok(url)
    }

    /// List all configured server names
    pub fn list_server_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.server.servers.keys().cloned().collect();
        names.sort();
        names
    }

    /// List all servers with details
    pub fn list_servers(&self) -> Vec<ServerInfo> {
        let mut servers: Vec<_> = self
            .server
            .servers
            .iter()
            .map(|(name, server)| ServerInfo {
                name: name.clone(),
                url: server.url.clone(),
                description: server.description.clone(),
                is_default: name == &self.server.default,
            })
            .collect();
        servers.sort_by(|a, b| a.name.cmp(&b.name));
        servers
    }

    /// Add or update a server. Validates the URL has http:// or https:// scheme.
    pub fn add_server(
        &mut self,
        name: String,
        url: String,
        description: Option<String>,
    ) -> Result<()> {
        validate_server_url(&url).map_err(|msg| {
            crate::error::Error::ConfigError(format!(
                "Invalid URL '{}': {}. Try: http://{}",
                url, msg, url
            ))
        })?;
        self.server
            .servers
            .insert(name, EurekaServer { url, description });
        Ok(())
    }

    /// Remove a server
    pub fn remove_server(&mut self, name: &str) -> Result<()> {
        if name == self.server.default {
            return Err(crate::error::Error::ConfigError(
                "Cannot remove default server. Set a different default first.".to_string(),
            ));
        }
        self.server.servers.remove(name).ok_or_else(|| {
            crate::error::Error::ConfigError(format!("Server '{}' not found", name))
        })?;
        Ok(())
    }

    /// Set default server
    pub fn set_default_server(&mut self, name: &str) -> Result<()> {
        if !self.server.servers.contains_key(name) {
            return Err(crate::error::Error::ConfigError(format!(
                "Server '{}' not found",
                name
            )));
        }
        self.server.default = name.to_string();
        Ok(())
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let config_dir = dirs::config_dir().ok_or_else(|| {
            crate::error::Error::ConfigError("Cannot find config directory".to_string())
        })?;
        let config_path = config_dir.join("eureka-cli").join("config.yaml");

        // Create directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                crate::error::Error::ConfigError(format!("Failed to create config dir: {}", e))
            })?;
        }

        let content = serde_yaml::to_string(self).map_err(|e| {
            crate::error::Error::ConfigError(format!("Failed to serialize config: {}", e))
        })?;
        std::fs::write(&config_path, content).map_err(|e| {
            crate::error::Error::ConfigError(format!("Failed to write config: {}", e))
        })?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ServerInfo {
    pub name: String,
    pub url: String,
    pub description: Option<String>,
    pub is_default: bool,
}

/// Validate a server URL has an http or https scheme.
pub fn validate_server_url(url: &str) -> std::result::Result<(), &'static str> {
    if url.starts_with("http://") || url.starts_with("https://") {
        Ok(())
    } else {
        Err("URL must start with http:// or https://")
    }
}
