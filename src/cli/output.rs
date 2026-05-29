use crate::error::Result;
use crate::models::*;
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Cell, Table};

/// Print a success message in the specified format
pub fn print_success(message: &str, format: &str) -> Result<()> {
    match format {
        "json" => {
            let output = serde_json::json!({
                "status": "success",
                "message": message
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        "yaml" => {
            let output = serde_json::json!({
                "status": "success",
                "message": message
            });
            println!("{}", serde_yaml::to_string(&output)?);
        }
        _ => {
            println!("{}", message.green());
        }
    }
    Ok(())
}

pub fn print_applications(apps: &ApplicationsWrapper, format: &str) -> Result<()> {
    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(apps)?);
        }
        "yaml" => {
            println!("{}", serde_yaml::to_string(apps)?);
        }
        _ => {
            let mut table = Table::new();
            table.load_preset(UTF8_FULL);
            table.set_header(vec!["Application", "Instances", "Status"]);

            for app in &apps.applications.apps {
                let instance_count = app.instance.len();
                let up_count = app
                    .instance
                    .iter()
                    .filter(|i| matches!(i.status, InstanceStatus::Up))
                    .count();

                let status = if up_count == instance_count && instance_count > 0 {
                    "UP".green().to_string()
                } else if up_count == 0 {
                    "DOWN".red().to_string()
                } else {
                    format!("PARTIAL ({}/{})", up_count, instance_count)
                        .yellow()
                        .to_string()
                };

                table.add_row(vec![
                    Cell::new(&app.name),
                    Cell::new(instance_count),
                    Cell::new(status),
                ]);
            }

            println!("{}", table);
        }
    }
    Ok(())
}

pub fn print_application(app: &Application, format: &str) -> Result<()> {
    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(app)?);
        }
        "yaml" => {
            println!("{}", serde_yaml::to_string(app)?);
        }
        _ => {
            println!("\n{}: {}", "Application".bold(), app.name.cyan());
            println!("{}: {}\n", "Instances".bold(), app.instance.len());
            print_instances(&app.instance, "table")?;
        }
    }
    Ok(())
}

pub fn print_instances(instances: &[Instance], format: &str) -> Result<()> {
    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(instances)?);
        }
        "yaml" => {
            println!("{}", serde_yaml::to_string(instances)?);
        }
        _ => {
            let mut table = Table::new();
            table.load_preset(UTF8_FULL);
            table.set_header(vec!["Instance ID", "Host", "IP", "Port", "Status"]);

            for instance in instances {
                let status_str = match instance.status {
                    InstanceStatus::Up => "UP".green().to_string(),
                    InstanceStatus::Down => "DOWN".red().to_string(),
                    InstanceStatus::Starting => "STARTING".yellow().to_string(),
                    InstanceStatus::OutOfService => "OUT_OF_SERVICE".red().to_string(),
                    InstanceStatus::Unknown => "UNKNOWN".dimmed().to_string(),
                };

                let port = instance
                    .port
                    .as_ref()
                    .map(|p| p.port.to_string())
                    .unwrap_or_else(|| "-".to_string());

                table.add_row(vec![
                    Cell::new(&instance.instance_id),
                    Cell::new(&instance.host_name),
                    Cell::new(&instance.ip_addr),
                    Cell::new(port),
                    Cell::new(status_str),
                ]);
            }

            println!("{}", table);
        }
    }
    Ok(())
}

pub fn print_instance(instance: &Instance, format: &str) -> Result<()> {
    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(instance)?);
        }
        "yaml" => {
            println!("{}", serde_yaml::to_string(instance)?);
        }
        _ => {
            println!("\n{}", "Instance Details".bold().cyan());
            println!("{}", "=".repeat(50));
            println!("{:20}: {}", "Instance ID", instance.instance_id);
            println!("{:20}: {}", "Application", instance.app);
            println!("{:20}: {}", "Hostname", instance.host_name);
            println!("{:20}: {}", "IP Address", instance.ip_addr);

            let status_str = match instance.status {
                InstanceStatus::Up => "UP".green(),
                InstanceStatus::Down => "DOWN".red(),
                InstanceStatus::Starting => "STARTING".yellow(),
                InstanceStatus::OutOfService => "OUT_OF_SERVICE".red(),
                InstanceStatus::Unknown => "UNKNOWN".dimmed(),
            };
            println!("{:20}: {}", "Status", status_str);

            if let Some(port) = &instance.port {
                println!("{:20}: {}", "Port", port.port);
            }
            println!("{:20}: {}", "Secure Port", instance.secure_port.port);
            println!("{:20}: {}", "VIP Address", instance.vip_address);
            println!(
                "{:20}: {}",
                "Home Page",
                instance.home_page_url.as_deref().unwrap_or("-")
            );
            println!(
                "{:20}: {}",
                "Status Page",
                instance.status_page_url.as_deref().unwrap_or("-")
            );
            println!(
                "{:20}: {}",
                "Health Check",
                instance.health_check_url.as_deref().unwrap_or("-")
            );

            if let Some(metadata) = &instance.metadata {
                if !metadata.is_empty() {
                    println!("\n{}", "Metadata:".bold());
                    for (key, value) in metadata {
                        println!("  {}: {}", key.cyan(), value);
                    }
                }
            }
            println!();
        }
    }
    Ok(())
}
