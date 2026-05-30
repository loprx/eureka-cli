use crate::cli::output;
use crate::cli::query::QueryOptions;
use crate::client::EurekaClient;
use crate::error::Result;
use crate::models::{Application, Instance, InstanceStatus};
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum AppsCommands {
    /// List all applications
    #[command(visible_alias = "ls")]
    List,
    /// Get application details
    Get {
        /// Application ID
        app_id: String,
    },
    /// List instances of an application
    #[command(visible_aliases = ["i", "inst"])]
    Instances {
        /// Application ID
        app_id: String,
    },
    /// List applications with non-UP instances (shortcut for -l status!=UP)
    Unhealthy,
}

impl AppsCommands {
    pub async fn execute(&self, client: &EurekaClient, opts: &QueryOptions<'_>) -> Result<()> {
        match self {
            AppsCommands::List => {
                let apps = client.get_applications().await?;
                output::print_with(opts.format, |f| f.format_applications(&apps))?;
                Ok(())
            }
            AppsCommands::Get { app_id } => {
                let app = client.get_application(app_id).await?;
                output::print_with(opts.format, |f| f.format_application(&app))?;
                Ok(())
            }
            AppsCommands::Instances { app_id } => {
                let app = client.get_app_instances(app_id).await?;
                let mut instances = app.instance;
                opts.refine(&mut instances);
                output::print_with(opts.format, |f| f.format_instances(&instances))?;
                Ok(())
            }
            AppsCommands::Unhealthy => {
                let apps = client.get_applications().await?;
                let unhealthy: Vec<Application> = apps
                    .applications
                    .apps
                    .iter()
                    .filter(|a| has_unhealthy_instance(&a.instance))
                    .cloned()
                    .collect();
                let wrapper = wrap_apps(unhealthy);
                output::print_with(opts.format, |f| f.format_applications(&wrapper))?;
                Ok(())
            }
        }
    }
}

fn has_unhealthy_instance(instances: &[Instance]) -> bool {
    instances
        .iter()
        .any(|i| !matches!(i.status, InstanceStatus::Up))
}

fn wrap_apps(apps: Vec<Application>) -> crate::models::ApplicationsWrapper {
    crate::models::ApplicationsWrapper {
        applications: crate::models::Applications {
            versions_delta: None,
            apps_hashcode: None,
            apps,
        },
    }
}
