use crate::cli::output;
use crate::cli::query::QueryOptions;
use crate::cli::watch;
use crate::client::EurekaClient;
use crate::error::Result;
use crate::models::{Instance, InstanceStatus};
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum InstancesCommands {
    /// List all instances
    #[command(visible_alias = "ls")]
    List,
    /// Get instance by ID
    Get {
        /// Instance ID
        instance_id: String,
        /// Application ID (optional)
        #[arg(short, long)]
        app_id: Option<String>,
    },
    /// Detailed multi-section view of an instance (kubectl describe style)
    #[command(visible_alias = "desc")]
    Describe {
        /// Instance ID
        instance_id: String,
        /// Application ID (optional)
        #[arg(short, long)]
        app_id: Option<String>,
    },
    /// List instances with non-UP status (shortcut for -l status!=UP)
    Unhealthy,
}

impl InstancesCommands {
    pub async fn execute(&self, client: &EurekaClient, opts: &QueryOptions<'_>) -> Result<()> {
        match self {
            InstancesCommands::List => {
                if opts.watch {
                    watch::run_loop(opts.watch_interval, || render_list(client, opts)).await
                } else {
                    render_list(client, opts).await
                }
            }
            InstancesCommands::Get {
                instance_id,
                app_id,
            } => {
                let instance = fetch_instance(client, instance_id, app_id.as_deref()).await?;
                output::print_with(opts.format, |f| f.format_instance(&instance))?;
                Ok(())
            }
            InstancesCommands::Describe {
                instance_id,
                app_id,
            } => {
                let instance = fetch_instance(client, instance_id, app_id.as_deref()).await?;
                output::print_with(opts.format, |f| f.format_describe_instance(&instance))?;
                Ok(())
            }
            InstancesCommands::Unhealthy => {
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
    let mut instances = collect_all_instances(client).await?;
    opts.refine(&mut instances);
    output::print_with(opts.format, |f| f.format_instances(&instances))
}

async fn render_unhealthy(client: &EurekaClient, opts: &QueryOptions<'_>) -> Result<()> {
    let mut instances: Vec<Instance> = collect_all_instances(client)
        .await?
        .into_iter()
        .filter(|i| !matches!(i.status, InstanceStatus::Up))
        .collect();
    opts.refine(&mut instances);
    output::print_with(opts.format, |f| f.format_instances(&instances))
}

async fn fetch_instance(
    client: &EurekaClient,
    instance_id: &str,
    app_id: Option<&str>,
) -> Result<Instance> {
    if let Some(app) = app_id {
        client.get_instance(app, instance_id).await
    } else {
        client.get_instance_by_id(instance_id).await
    }
}

async fn collect_all_instances(client: &EurekaClient) -> Result<Vec<Instance>> {
    let apps = client.get_applications().await?;
    Ok(apps
        .applications
        .apps
        .into_iter()
        .flat_map(|a| a.instance)
        .collect())
}
