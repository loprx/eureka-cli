//! Query-time options shared by list-style commands.
//!
//! Centralises `-l/--selector`, `--sort-by`, and the resolved `OutputFormat`
//! so individual commands don't grow long parameter lists.

use crate::cli::{format::OutputFormat, selector::Selector};
use crate::error::{Error, Result};
use crate::models::Instance;
use std::time::Duration;

#[derive(Debug)]
pub struct QueryOptions<'a> {
    pub format: &'a OutputFormat,
    pub selector: Option<Selector>,
    pub sort_by: Option<String>,
    pub watch: bool,
    pub watch_interval: Duration,
}

impl<'a> QueryOptions<'a> {
    pub fn new(
        format: &'a OutputFormat,
        selector_expr: Option<&str>,
        sort_by: Option<String>,
        watch: bool,
        watch_interval_secs: u64,
    ) -> Result<Self> {
        let selector = selector_expr
            .map(Selector::parse)
            .transpose()
            .map_err(|e| Error::ConfigError(format!("invalid selector: {}", e)))?;
        Ok(Self {
            format,
            selector,
            sort_by,
            watch,
            watch_interval: Duration::from_secs(watch_interval_secs.max(1)),
        })
    }

    /// Apply selector + sort-by to a list of instances in place.
    pub fn refine(&self, instances: &mut Vec<Instance>) {
        if let Some(sel) = &self.selector {
            instances.retain(|i| sel.matches(i));
        }
        if let Some(field) = &self.sort_by {
            sort_instances_by(instances, field);
        }
    }
}

/// Stable sort instances by a string field path (status / app / ip_addr / ...).
fn sort_instances_by(instances: &mut [Instance], field: &str) {
    instances.sort_by(|a, b| {
        let av = field_string(a, field);
        let bv = field_string(b, field);
        av.cmp(&bv)
    });
}

fn field_string(instance: &Instance, field: &str) -> String {
    let json = match serde_json::to_value(instance) {
        Ok(v) => v,
        Err(_) => return String::new(),
    };
    let mut current = &json;
    for part in field.split('.') {
        match current.get(part) {
            Some(v) => current = v,
            None => return String::new(),
        }
    }
    match current {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        _ => String::new(),
    }
}
