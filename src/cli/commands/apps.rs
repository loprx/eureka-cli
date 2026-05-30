use crate::cli::output;
use crate::cli::query::QueryOptions;
use crate::cli::watch;
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
    /// Detailed multi-section view of an application (kubectl describe style)
    #[command(visible_alias = "desc")]
    Describe {
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
                if opts.watch {
                    watch::run_loop(opts.watch_interval, || render_list(client, opts)).await
                } else {
                    render_list(client, opts).await
                }
            }
            AppsCommands::Get { app_id } => {
                let app = client.get_application(app_id).await?;
                output::print_with(opts.format, |f| f.format_application(&app))?;
                Ok(())
            }
            AppsCommands::Describe { app_id } => {
                let app = client.get_application(app_id).await?;
                output::print_with(opts.format, |f| f.format_describe_application(&app))?;
                Ok(())
            }
            AppsCommands::Instances { app_id } => {
                if opts.watch {
                    watch::run_loop(opts.watch_interval, || {
                        render_instances(client, opts, app_id)
                    })
                    .await
                } else {
                    render_instances(client, opts, app_id).await
                }
            }
            AppsCommands::Unhealthy => {
                if opts.watch {
                    watch::run_loop(opts.watch_interval, || render_unhealthy(client, opts)).await
                } else {
                    render_unhealthy(client, opts).await
                }
            }
        }
    }
}

async fn render_list(client: &EurekaClient, opts: &QueryOptions<'_>) -> Result<()> {
    let apps = client.get_applications().await?;
    output::print_with(opts.format, |f| f.format_applications(&apps))
}

async fn render_instances(
    client: &EurekaClient,
    opts: &QueryOptions<'_>,
    app_id: &str,
) -> Result<()> {
    let app = client.get_app_instances(app_id).await?;
    let mut instances = app.instance;
    opts.refine(&mut instances);
    output::print_with(opts.format, |f| f.format_instances(&instances))
}

async fn render_unhealthy(client: &EurekaClient, opts: &QueryOptions<'_>) -> Result<()> {
    let apps = client.get_applications().await?;
    let unhealthy: Vec<Application> = apps
        .applications
        .apps
        .iter()
        .filter(|a| has_unhealthy_instance(&a.instance))
        .cloned()
        .collect();
    let wrapper = wrap_apps(unhealthy);
    output::print_with(opts.format, |f| f.format_applications(&wrapper))
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
