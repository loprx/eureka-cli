use crate::error::Result;
use crate::models::*;
use colored::Colorize;
use comfy_table::{presets::NOTHING, Cell, ContentArrangement, Table};

use super::format::OutputFormat;

// ---------------------------------------------------------------------------
// OutputFormatter trait — single source of truth for rendering
// ---------------------------------------------------------------------------

/// Renders Eureka resources in a specific format.
///
/// Each format (table / json / yaml / ...) implements this trait once; CLI
/// commands depend on the trait, not on a specific format string. New formats
/// only need to add a new implementor — no changes to call sites.
pub trait OutputFormatter {
    fn format_applications(&self, apps: &ApplicationsWrapper) -> Result<String>;
    fn format_application(&self, app: &Application) -> Result<String>;
    fn format_instances(&self, instances: &[Instance]) -> Result<String>;
    fn format_instance(&self, instance: &Instance) -> Result<String>;
    fn format_success(&self, message: &str) -> Result<String>;
}

/// Build the formatter for the requested output format.
pub fn formatter_for(format: &OutputFormat) -> Box<dyn OutputFormatter> {
    match format {
        OutputFormat::Table | OutputFormat::Wide => Box::new(TableFormatter),
        OutputFormat::Json | OutputFormat::JsonPath(_) => Box::new(JsonFormatter),
        OutputFormat::Yaml => Box::new(YamlFormatter),
    }
}

// ---------------------------------------------------------------------------
// TableFormatter — kubectl-style: no borders, space-aligned columns
// ---------------------------------------------------------------------------

struct TableFormatter;

impl TableFormatter {
    fn new_table() -> Table {
        let mut table = Table::new();
        table.load_preset(NOTHING);
        table.set_content_arrangement(ContentArrangement::Disabled);
        table
    }

    fn status_cell(status: &InstanceStatus) -> String {
        match status {
            InstanceStatus::Up => "UP".green().to_string(),
            InstanceStatus::Down => "DOWN".red().to_string(),
            InstanceStatus::Starting => "STARTING".yellow().to_string(),
            InstanceStatus::OutOfService => "OUT_OF_SERVICE".red().to_string(),
            InstanceStatus::Unknown => "UNKNOWN".dimmed().to_string(),
        }
    }
}

impl OutputFormatter for TableFormatter {
    fn format_applications(&self, apps: &ApplicationsWrapper) -> Result<String> {
        let mut table = Self::new_table();
        table.set_header(vec!["NAME", "INSTANCES", "STATUS"]);

        for app in &apps.applications.apps {
            let total = app.instance.len();
            let up = app
                .instance
                .iter()
                .filter(|i| matches!(i.status, InstanceStatus::Up))
                .count();
            let status = if up == total && total > 0 {
                "UP".green().to_string()
            } else if up == 0 {
                "DOWN".red().to_string()
            } else {
                format!("PARTIAL ({}/{})", up, total).yellow().to_string()
            };

            table.add_row(vec![
                Cell::new(&app.name),
                Cell::new(total),
                Cell::new(status),
            ]);
        }

        Ok(table.to_string())
    }

    fn format_application(&self, app: &Application) -> Result<String> {
        let mut out = String::new();
        out.push_str(&format!("{}: {}\n", "Application".bold(), app.name.cyan()));
        out.push_str(&format!("{}: {}\n\n", "Instances".bold(), app.instance.len()));
        out.push_str(&self.format_instances(&app.instance)?);
        Ok(out)
    }

    fn format_instances(&self, instances: &[Instance]) -> Result<String> {
        let mut table = Self::new_table();
        table.set_header(vec!["INSTANCE ID", "HOST", "IP", "PORT", "STATUS"]);

        for inst in instances {
            let port = inst
                .port
                .as_ref()
                .map(|p| p.port.to_string())
                .unwrap_or_else(|| "-".to_string());

            table.add_row(vec![
                Cell::new(&inst.instance_id),
                Cell::new(&inst.host_name),
                Cell::new(&inst.ip_addr),
                Cell::new(port),
                Cell::new(Self::status_cell(&inst.status)),
            ]);
        }

        Ok(table.to_string())
    }

    fn format_instance(&self, instance: &Instance) -> Result<String> {
        let mut out = String::new();
        out.push_str(&format!("\n{}\n", "Instance Details".bold().cyan()));
        out.push_str(&"=".repeat(50));
        out.push('\n');
        out.push_str(&format!("{:20}: {}\n", "Instance ID", instance.instance_id));
        out.push_str(&format!("{:20}: {}\n", "Application", instance.app));
        out.push_str(&format!("{:20}: {}\n", "Hostname", instance.host_name));
        out.push_str(&format!("{:20}: {}\n", "IP Address", instance.ip_addr));
        out.push_str(&format!(
            "{:20}: {}\n",
            "Status",
            Self::status_cell(&instance.status)
        ));

        if let Some(port) = &instance.port {
            out.push_str(&format!("{:20}: {}\n", "Port", port.port));
        }
        out.push_str(&format!("{:20}: {}\n", "Secure Port", instance.secure_port.port));
        out.push_str(&format!("{:20}: {}\n", "VIP Address", instance.vip_address));
        out.push_str(&format!(
            "{:20}: {}\n",
            "Home Page",
            instance.home_page_url.as_deref().unwrap_or("-")
        ));
        out.push_str(&format!(
            "{:20}: {}\n",
            "Status Page",
            instance.status_page_url.as_deref().unwrap_or("-")
        ));
        out.push_str(&format!(
            "{:20}: {}\n",
            "Health Check",
            instance.health_check_url.as_deref().unwrap_or("-")
        ));

        if let Some(metadata) = &instance.metadata {
            if !metadata.is_empty() {
                out.push_str(&format!("\n{}\n", "Metadata:".bold()));
                for (key, value) in metadata {
                    out.push_str(&format!("  {}: {}\n", key.cyan(), value));
                }
            }
        }
        Ok(out)
    }

    fn format_success(&self, message: &str) -> Result<String> {
        Ok(message.green().to_string())
    }
}

// ---------------------------------------------------------------------------
// JsonFormatter
// ---------------------------------------------------------------------------

struct JsonFormatter;

impl OutputFormatter for JsonFormatter {
    fn format_applications(&self, apps: &ApplicationsWrapper) -> Result<String> {
        Ok(serde_json::to_string_pretty(apps)?)
    }

    fn format_application(&self, app: &Application) -> Result<String> {
        Ok(serde_json::to_string_pretty(app)?)
    }

    fn format_instances(&self, instances: &[Instance]) -> Result<String> {
        Ok(serde_json::to_string_pretty(instances)?)
    }

    fn format_instance(&self, instance: &Instance) -> Result<String> {
        Ok(serde_json::to_string_pretty(instance)?)
    }

    fn format_success(&self, message: &str) -> Result<String> {
        let json = serde_json::json!({ "status": "success", "message": message });
        Ok(serde_json::to_string_pretty(&json)?)
    }
}

// ---------------------------------------------------------------------------
// YamlFormatter
// ---------------------------------------------------------------------------

struct YamlFormatter;

impl OutputFormatter for YamlFormatter {
    fn format_applications(&self, apps: &ApplicationsWrapper) -> Result<String> {
        Ok(serde_yaml::to_string(apps)?)
    }

    fn format_application(&self, app: &Application) -> Result<String> {
        Ok(serde_yaml::to_string(app)?)
    }

    fn format_instances(&self, instances: &[Instance]) -> Result<String> {
        Ok(serde_yaml::to_string(instances)?)
    }

    fn format_instance(&self, instance: &Instance) -> Result<String> {
        Ok(serde_yaml::to_string(instance)?)
    }

    fn format_success(&self, message: &str) -> Result<String> {
        let json = serde_json::json!({ "status": "success", "message": message });
        Ok(serde_yaml::to_string(&json)?)
    }
}

// ---------------------------------------------------------------------------
// Legacy string-based entry points
// ---------------------------------------------------------------------------
//
// Existing command files pass `output_format: &str` ("table" | "json" | "yaml").
// These thin wrappers translate the string into an OutputFormat, dispatch via
// the trait, and print. They will be removed once command files are migrated
// to take `&OutputFormat` directly.

fn format_from_legacy_str(s: &str) -> OutputFormat {
    match s {
        "json" => OutputFormat::Json,
        "yaml" => OutputFormat::Yaml,
        _ => OutputFormat::Table,
    }
}

pub fn print_applications(apps: &ApplicationsWrapper, format: &str) -> Result<()> {
    let f = formatter_for(&format_from_legacy_str(format));
    println!("{}", f.format_applications(apps)?);
    Ok(())
}

pub fn print_application(app: &Application, format: &str) -> Result<()> {
    let f = formatter_for(&format_from_legacy_str(format));
    println!("{}", f.format_application(app)?);
    Ok(())
}

pub fn print_instances(instances: &[Instance], format: &str) -> Result<()> {
    let f = formatter_for(&format_from_legacy_str(format));
    println!("{}", f.format_instances(instances)?);
    Ok(())
}

pub fn print_instance(instance: &Instance, format: &str) -> Result<()> {
    let f = formatter_for(&format_from_legacy_str(format));
    println!("{}", f.format_instance(instance)?);
    Ok(())
}

pub fn print_success(message: &str, format: &str) -> Result<()> {
    let f = formatter_for(&format_from_legacy_str(format));
    println!("{}", f.format_success(message)?);
    Ok(())
}
