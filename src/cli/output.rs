use crate::error::Result;
use crate::models::*;
use colored::Colorize;
use comfy_table::{presets::NOTHING, Attribute, Cell, Color, ContentArrangement, Table};

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

    /// Detailed multi-section view of a single instance (kubectl describe style).
    /// Defaults to format_instance — JSON/YAML want the same structured payload.
    fn format_describe_instance(&self, instance: &Instance) -> Result<String> {
        self.format_instance(instance)
    }

    /// Detailed view of an application: header + describe each instance.
    fn format_describe_application(&self, app: &Application) -> Result<String> {
        self.format_application(app)
    }
}

/// Build the formatter for the requested output format.
pub fn formatter_for(format: &OutputFormat) -> Box<dyn OutputFormatter> {
    match format {
        OutputFormat::Table => Box::new(TableFormatter { wide: false }),
        OutputFormat::Wide => Box::new(TableFormatter { wide: true }),
        OutputFormat::Json => Box::new(JsonFormatter),
        OutputFormat::JsonPath(expr) => Box::new(JsonPathFormatter { expr: expr.clone() }),
        OutputFormat::Yaml => Box::new(YamlFormatter),
    }
}

/// Print any renderable resource using the trait. Generic over a closure so
/// callers stay declarative — `print_with(fmt, |f| f.format_instances(&xs))`.
pub fn print_with<F>(format: &OutputFormat, f: F) -> Result<()>
where
    F: FnOnce(&dyn OutputFormatter) -> Result<String>,
{
    let formatter = formatter_for(format);
    println!("{}", f(formatter.as_ref())?);
    Ok(())
}

// ---------------------------------------------------------------------------
// TableFormatter — kubectl-style: no borders, space-aligned columns
// ---------------------------------------------------------------------------

struct TableFormatter {
    /// Wide mode adds extra columns (zone, vip, last_dirty, metadata summary).
    wide: bool,
}

impl TableFormatter {
    fn new_table() -> Table {
        let mut table = Table::new();
        table.load_preset(NOTHING);
        table.set_content_arrangement(ContentArrangement::Disabled);
        table
    }

    fn status_cell(status: &InstanceStatus) -> Cell {
        // Use comfy-table's own styling so it computes width correctly.
        // Returning a pre-colored String breaks alignment in NOTHING preset.
        match status {
            InstanceStatus::Up => Cell::new("UP").fg(Color::Green),
            InstanceStatus::Down => Cell::new("DOWN").fg(Color::Red),
            InstanceStatus::Starting => Cell::new("STARTING").fg(Color::Yellow),
            InstanceStatus::OutOfService => Cell::new("OUT_OF_SERVICE").fg(Color::Red),
            InstanceStatus::Unknown => Cell::new("UNKNOWN").add_attribute(Attribute::Dim),
        }
    }

    /// Plain (uncolored) status string for use in describe output and other
    /// non-table contexts where ANSI inflation doesn't matter.
    fn status_str(status: &InstanceStatus) -> String {
        match status {
            InstanceStatus::Up => "UP".green().to_string(),
            InstanceStatus::Down => "DOWN".red().to_string(),
            InstanceStatus::Starting => "STARTING".yellow().to_string(),
            InstanceStatus::OutOfService => "OUT_OF_SERVICE".red().to_string(),
            InstanceStatus::Unknown => "UNKNOWN".dimmed().to_string(),
        }
    }

    fn metadata_summary(instance: &Instance) -> String {
        let Some(map) = &instance.metadata else {
            return "-".to_string();
        };
        if map.is_empty() {
            return "-".to_string();
        }
        let mut keys: Vec<&String> = map.keys().collect();
        keys.sort();
        let preview: Vec<String> = keys.iter().take(3).map(|k| (*k).clone()).collect();
        if keys.len() > 3 {
            format!("{} (+{})", preview.join(","), keys.len() - 3)
        } else {
            preview.join(",")
        }
    }
}

impl OutputFormatter for TableFormatter {
    fn format_applications(&self, apps: &ApplicationsWrapper) -> Result<String> {
        let mut table = Self::new_table();
        if self.wide {
            table.set_header(vec!["NAME", "INSTANCES", "UP", "DOWN", "STATUS"]);
        } else {
            table.set_header(vec!["NAME", "INSTANCES", "STATUS"]);
        }

        for app in &apps.applications.apps {
            let total = app.instance.len();
            let up = app
                .instance
                .iter()
                .filter(|i| matches!(i.status, InstanceStatus::Up))
                .count();
            let down = app
                .instance
                .iter()
                .filter(|i| {
                    matches!(
                        i.status,
                        InstanceStatus::Down | InstanceStatus::OutOfService
                    )
                })
                .count();
            let status_cell = if up == total && total > 0 {
                Cell::new("UP").fg(Color::Green)
            } else if up == 0 {
                Cell::new("DOWN").fg(Color::Red)
            } else {
                Cell::new(format!("PARTIAL ({}/{})", up, total)).fg(Color::Yellow)
            };

            if self.wide {
                table.add_row(vec![
                    Cell::new(&app.name),
                    Cell::new(total),
                    Cell::new(up),
                    Cell::new(down),
                    status_cell,
                ]);
            } else {
                table.add_row(vec![Cell::new(&app.name), Cell::new(total), status_cell]);
            }
        }

        Ok(table.to_string())
    }

    fn format_application(&self, app: &Application) -> Result<String> {
        let mut out = String::new();
        out.push_str(&format!("{}: {}\n", "Application".bold(), app.name.cyan()));
        out.push_str(&format!(
            "{}: {}\n\n",
            "Instances".bold(),
            app.instance.len()
        ));
        out.push_str(&self.format_instances(&app.instance)?);
        Ok(out)
    }

    fn format_instances(&self, instances: &[Instance]) -> Result<String> {
        let mut table = Self::new_table();
        if self.wide {
            table.set_header(vec![
                "INSTANCE ID",
                "APP",
                "HOST",
                "IP",
                "PORT",
                "STATUS",
                "VIP",
                "METADATA",
            ]);
        } else {
            table.set_header(vec!["INSTANCE ID", "HOST", "IP", "PORT", "STATUS"]);
        }

        for inst in instances {
            let port = inst
                .port
                .as_ref()
                .map(|p| p.port.to_string())
                .unwrap_or_else(|| "-".to_string());

            if self.wide {
                let vip = if inst.vip_address.is_empty() {
                    "-".to_string()
                } else {
                    inst.vip_address.clone()
                };
                table.add_row(vec![
                    Cell::new(&inst.instance_id),
                    Cell::new(&inst.app),
                    Cell::new(&inst.host_name),
                    Cell::new(&inst.ip_addr),
                    Cell::new(port),
                    Self::status_cell(&inst.status),
                    Cell::new(vip),
                    Cell::new(Self::metadata_summary(inst)),
                ]);
            } else {
                table.add_row(vec![
                    Cell::new(&inst.instance_id),
                    Cell::new(&inst.host_name),
                    Cell::new(&inst.ip_addr),
                    Cell::new(port),
                    Self::status_cell(&inst.status),
                ]);
            }
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
            Self::status_str(&instance.status)
        ));

        if let Some(port) = &instance.port {
            out.push_str(&format!("{:20}: {}\n", "Port", port.port));
        }
        out.push_str(&format!(
            "{:20}: {}\n",
            "Secure Port", instance.secure_port.port
        ));
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

    fn format_describe_instance(&self, instance: &Instance) -> Result<String> {
        Ok(describe_instance(instance))
    }

    fn format_describe_application(&self, app: &Application) -> Result<String> {
        let mut out = String::new();
        out.push_str(&format!("Name:       {}\n", app.name.cyan()));
        out.push_str(&format!("Instances:  {}\n", app.instance.len()));

        let up = app
            .instance
            .iter()
            .filter(|i| matches!(i.status, InstanceStatus::Up))
            .count();
        out.push_str(&format!("Healthy:    {}/{}\n", up, app.instance.len()));

        for (idx, inst) in app.instance.iter().enumerate() {
            out.push_str(&format!(
                "\n--- Instance [{}/{}] ---\n",
                idx + 1,
                app.instance.len()
            ));
            out.push_str(&describe_instance(inst));
        }
        Ok(out)
    }
}

/// kubectl describe-style multi-section view of one instance.
/// Pure function: no I/O, deterministic, easy to unit-test.
fn describe_instance(instance: &Instance) -> String {
    let mut out = String::new();
    let pad = 24;

    // Identity
    out.push_str(&format!("{}\n", "Identity:".bold()));
    out.push_str(&format!(
        "  {:pad$}{}\n",
        "Instance ID:",
        instance.instance_id,
        pad = pad
    ));
    out.push_str(&format!(
        "  {:pad$}{}\n",
        "Application:",
        instance.app,
        pad = pad
    ));
    out.push_str(&format!(
        "  {:pad$}{}\n",
        "Hostname:",
        instance.host_name,
        pad = pad
    ));
    out.push_str(&format!(
        "  {:pad$}{}\n",
        "IP Address:",
        instance.ip_addr,
        pad = pad
    ));

    // Status
    out.push_str(&format!("\n{}\n", "Status:".bold()));
    out.push_str(&format!(
        "  {:pad$}{}\n",
        "Status:",
        TableFormatter::status_str(&instance.status),
        pad = pad
    ));
    if let Some(overridden) = &instance.overriddenstatus {
        out.push_str(&format!(
            "  {:pad$}{}\n",
            "Overridden:",
            TableFormatter::status_str(overridden),
            pad = pad
        ));
    }
    if let Some(action) = &instance.action_type {
        out.push_str(&format!("  {:pad$}{}\n", "Action Type:", action, pad = pad));
    }

    // Network
    out.push_str(&format!("\n{}\n", "Network:".bold()));
    let port_str = instance
        .port
        .as_ref()
        .map(|p| format!("{} (enabled={})", p.port, p.enabled))
        .unwrap_or_else(|| "-".to_string());
    out.push_str(&format!("  {:pad$}{}\n", "Port:", port_str, pad = pad));
    out.push_str(&format!(
        "  {:pad$}{} (enabled={})\n",
        "Secure Port:",
        instance.secure_port.port,
        instance.secure_port.enabled,
        pad = pad
    ));
    out.push_str(&format!(
        "  {:pad$}{}\n",
        "VIP Address:",
        instance.vip_address,
        pad = pad
    ));
    out.push_str(&format!(
        "  {:pad$}{}\n",
        "Secure VIP:",
        instance.secure_vip_address,
        pad = pad
    ));
    out.push_str(&format!(
        "  {:pad$}{}\n",
        "Home Page:",
        instance.home_page_url.as_deref().unwrap_or("-"),
        pad = pad
    ));
    out.push_str(&format!(
        "  {:pad$}{}\n",
        "Status Page:",
        instance.status_page_url.as_deref().unwrap_or("-"),
        pad = pad
    ));
    out.push_str(&format!(
        "  {:pad$}{}\n",
        "Health Check:",
        instance.health_check_url.as_deref().unwrap_or("-"),
        pad = pad
    ));

    // Lease
    if let Some(lease) = &instance.lease_info {
        out.push_str(&format!("\n{}\n", "Lease:".bold()));
        let lease_json = serde_json::to_value(lease).unwrap_or(serde_json::Value::Null);
        if let Some(map) = lease_json.as_object() {
            for (k, v) in map {
                out.push_str(&format!("  {:pad$}{}\n", format!("{}:", k), v, pad = pad));
            }
        }
    }

    // Data Center
    out.push_str(&format!("\n{}\n", "DataCenter:".bold()));
    out.push_str(&format!(
        "  {:pad$}{:?}\n",
        "Name:",
        instance.data_center_info.name,
        pad = pad
    ));

    // Metadata
    if let Some(metadata) = &instance.metadata {
        if !metadata.is_empty() {
            out.push_str(&format!("\n{}\n", "Metadata:".bold()));
            let mut keys: Vec<&String> = metadata.keys().collect();
            keys.sort();
            for k in keys {
                if let Some(v) = metadata.get(k) {
                    out.push_str(&format!("  {}: {}\n", k.cyan(), v));
                }
            }
        }
    }

    // Timestamps (raw — Eureka returns them as nested {$: <ts>})
    if instance.last_updated_timestamp.is_some() || instance.last_dirty_timestamp.is_some() {
        out.push_str(&format!("\n{}\n", "Timestamps:".bold()));
        if let Some(v) = &instance.last_updated_timestamp {
            out.push_str(&format!("  {:pad$}{}\n", "Last Updated:", v, pad = pad));
        }
        if let Some(v) = &instance.last_dirty_timestamp {
            out.push_str(&format!("  {:pad$}{}\n", "Last Dirty:", v, pad = pad));
        }
    }

    out
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
// JsonPathFormatter — applies a JSONPath expression to the JSON view.
// ---------------------------------------------------------------------------
//
// Mirrors `kubectl -o jsonpath=...`: serialize the resource to JSON, then
// evaluate the expression. Single results print bare; arrays print one
// element per line so the output stays pipe-friendly (xargs etc.).

struct JsonPathFormatter {
    expr: String,
}

impl JsonPathFormatter {
    fn apply<T: serde::Serialize>(&self, value: &T) -> Result<String> {
        use jsonpath_rust::JsonPath;
        let json = serde_json::to_value(value)?;
        let path = JsonPath::try_from(self.expr.as_str())
            .map_err(|e| crate::error::Error::ConfigError(format!("invalid jsonpath: {}", e)))?;
        let results = path.find_slice(&json);
        let mut out = String::new();
        for (i, jpv) in results.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            let v = jpv.clone().to_data();
            match &v {
                serde_json::Value::String(s) => out.push_str(s),
                other => out.push_str(&other.to_string()),
            }
        }
        Ok(out)
    }
}

impl OutputFormatter for JsonPathFormatter {
    fn format_applications(&self, apps: &ApplicationsWrapper) -> Result<String> {
        self.apply(apps)
    }
    fn format_application(&self, app: &Application) -> Result<String> {
        self.apply(app)
    }
    fn format_instances(&self, instances: &[Instance]) -> Result<String> {
        // Wrap so users can write `$.instances[*].ipAddr` consistently.
        let wrapped = serde_json::json!({ "instances": instances });
        self.apply(&wrapped)
    }
    fn format_instance(&self, instance: &Instance) -> Result<String> {
        self.apply(instance)
    }
    fn format_success(&self, message: &str) -> Result<String> {
        let json = serde_json::json!({ "status": "success", "message": message });
        self.apply(&json)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn instance_fixture(id: &str, status: InstanceStatus) -> Instance {
        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), serde_json::json!("v2"));
        metadata.insert("zone".to_string(), serde_json::json!("us-east-1"));
        Instance {
            instance_id: id.to_string(),
            host_name: "host-1".to_string(),
            app: "TEST-APP".to_string(),
            ip_addr: "10.0.0.1".to_string(),
            status,
            overriddenstatus: None,
            port: Some(Port {
                port: 8080,
                enabled: true,
            }),
            secure_port: Port {
                port: 8443,
                enabled: false,
            },
            country_id: None,
            home_page_url: Some("http://10.0.0.1:8080/".to_string()),
            status_page_url: None,
            health_check_url: None,
            vip_address: "test-vip".to_string(),
            secure_vip_address: "test-svip".to_string(),
            data_center_info: DataCenterInfo::default(),
            lease_info: None,
            metadata: Some(metadata),
            last_updated_timestamp: None,
            last_dirty_timestamp: None,
            action_type: None,
            is_coordinating_discovery_server: None,
        }
    }

    #[test]
    fn table_default_has_no_app_column() {
        let f = formatter_for(&OutputFormat::Table);
        let inst = instance_fixture("i-1", InstanceStatus::Up);
        let out = f.format_instances(&[inst]).unwrap();
        // narrow table: INSTANCE ID / HOST / IP / PORT / STATUS — no APP column
        assert!(out.contains("INSTANCE ID"));
        assert!(out.contains("STATUS"));
        assert!(!out.contains("APP "));
        assert!(!out.contains("VIP"));
    }

    #[test]
    fn wide_table_adds_app_vip_metadata_columns() {
        let f = formatter_for(&OutputFormat::Wide);
        let inst = instance_fixture("i-1", InstanceStatus::Up);
        let out = f.format_instances(&[inst]).unwrap();
        assert!(out.contains("APP"));
        assert!(out.contains("VIP"));
        assert!(out.contains("METADATA"));
    }

    #[test]
    fn describe_emits_kubectl_style_sections() {
        let f = formatter_for(&OutputFormat::Table);
        let inst = instance_fixture("i-1", InstanceStatus::Up);
        let out = f.format_describe_instance(&inst).unwrap();
        for section in &[
            "Identity:",
            "Status:",
            "Network:",
            "DataCenter:",
            "Metadata:",
        ] {
            assert!(
                out.contains(section),
                "missing section {} in:\n{}",
                section,
                out
            );
        }
        // Pad >= 24 means "Instance ID:" (12) and any other short label has at
        // least 12 spaces of padding before the value — easy to scan.
        assert!(out.contains("Instance ID:"));
    }

    #[test]
    fn jsonpath_formatter_extracts_array() {
        let f = formatter_for(&OutputFormat::JsonPath("$.instances[*].ipAddr".to_string()));
        let inst1 = instance_fixture("i-1", InstanceStatus::Up);
        let mut inst2 = instance_fixture("i-2", InstanceStatus::Down);
        inst2.ip_addr = "10.0.0.2".to_string();
        let out = f.format_instances(&[inst1, inst2]).unwrap();
        // JsonPath formatter prints one match per line for pipe friendliness.
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(out.contains("10.0.0.1"));
        assert!(out.contains("10.0.0.2"));
    }

    #[test]
    fn jsonpath_formatter_invalid_expr_returns_error() {
        let f = formatter_for(&OutputFormat::JsonPath("$..[malformed".to_string()));
        let inst = instance_fixture("i-1", InstanceStatus::Up);
        assert!(f.format_instances(&[inst]).is_err());
    }
}
