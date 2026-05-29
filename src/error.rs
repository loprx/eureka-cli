use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Eureka server returned error: {status} - {message}")]
    ServerError { status: u16, message: String },

    #[error("Instance not found: {app}/{instance}")]
    InstanceNotFound { app: String, instance: String },

    #[error("Application not found: {0}")]
    ApplicationNotFound(String),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid status: {0}")]
    InvalidStatus(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("{0}")]
    Other(String),
}

impl Error {
    pub fn server_error(status: u16, message: impl Into<String>) -> Self {
        Error::ServerError {
            status,
            message: message.into(),
        }
    }

    pub fn instance_not_found(app: impl Into<String>, instance: impl Into<String>) -> Self {
        Error::InstanceNotFound {
            app: app.into(),
            instance: instance.into(),
        }
    }
}
