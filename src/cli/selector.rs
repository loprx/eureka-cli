use crate::models::Instance;
use serde_json::Value;

/// Selector expression for filtering resources
#[derive(Debug, Clone, PartialEq)]
pub struct Selector {
    /// List of conditions (AND logic)
    conditions: Vec<Condition>,
}

#[derive(Debug, Clone, PartialEq)]
struct Condition {
    field: String,
    operator: Operator,
    value: String,
}

#[derive(Debug, Clone, PartialEq)]
enum Operator {
    Equal,
    NotEqual,
}

impl Selector {
    /// Parse selector expression from string
    /// Format: "key=value,key2!=value2"
    /// Supports nested fields: "metadata.version=v2"
    ///
    /// Note on case sensitivity: matching is exact. Eureka uppercases `app`
    /// names server-side, so `-l app=foo` will not match a registered
    /// "FOO" — use the actual stored value (`-l app=FOO`). Other fields
    /// (status, metadata, ip_addr) preserve case as given.
    pub fn parse(expr: &str) -> Result<Self, String> {
        if expr.is_empty() {
            return Err("Empty selector expression".to_string());
        }

        let conditions: Result<Vec<_>, _> = expr
            .split(',')
            .map(|part| {
                let part = part.trim();
                if let Some(pos) = part.find("!=") {
                    let field = part[..pos].trim().to_string();
                    let value = part[pos + 2..].trim().to_string();
                    Ok(Condition {
                        field,
                        operator: Operator::NotEqual,
                        value,
                    })
                } else if let Some(pos) = part.find('=') {
                    let field = part[..pos].trim().to_string();
                    let value = part[pos + 1..].trim().to_string();
                    Ok(Condition {
                        field,
                        operator: Operator::Equal,
                        value,
                    })
                } else {
                    Err(format!("Invalid condition: {}", part))
                }
            })
            .collect();

        Ok(Selector {
            conditions: conditions?,
        })
    }

    /// Check if an instance matches this selector
    pub fn matches(&self, instance: &Instance) -> bool {
        // Convert instance to JSON for uniform field access
        let json = match serde_json::to_value(instance) {
            Ok(v) => v,
            Err(_) => return false,
        };

        // All conditions must match (AND logic)
        self.conditions.iter().all(|cond| {
            let field_value = get_field_value(&json, &cond.field);
            match cond.operator {
                Operator::Equal => field_value.as_deref() == Some(&cond.value),
                Operator::NotEqual => field_value.as_deref() != Some(&cond.value),
            }
        })
    }
}

/// Get field value from JSON, supporting nested paths like "metadata.version"
fn get_field_value(json: &Value, field_path: &str) -> Option<String> {
    let parts: Vec<&str> = field_path.split('.').collect();
    let mut current = json;

    for part in parts {
        current = current.get(part)?;
    }

    // Convert to string representation
    match current {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Null => Some("null".to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DataCenterInfo, InstanceStatus, Port};
    use std::collections::HashMap;

    fn create_test_instance(
        status: InstanceStatus,
        metadata: Option<HashMap<String, Value>>,
    ) -> Instance {
        Instance {
            instance_id: "test-instance".to_string(),
            host_name: "test-host".to_string(),
            app: "test-app".to_string(),
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
            home_page_url: None,
            status_page_url: None,
            health_check_url: None,
            vip_address: "test-vip".to_string(),
            secure_vip_address: "test-svip".to_string(),
            data_center_info: DataCenterInfo::default(),
            lease_info: None,
            metadata,
            last_updated_timestamp: None,
            last_dirty_timestamp: None,
            action_type: None,
            is_coordinating_discovery_server: None,
        }
    }

    #[test]
    fn test_parse_selector() {
        let selector = Selector::parse("status=UP").unwrap();
        assert_eq!(selector.conditions.len(), 1);

        let selector = Selector::parse("status=UP,app=test").unwrap();
        assert_eq!(selector.conditions.len(), 2);

        let selector = Selector::parse("status!=DOWN").unwrap();
        assert_eq!(selector.conditions[0].operator, Operator::NotEqual);
    }

    #[test]
    fn test_parse_invalid_selector() {
        assert!(Selector::parse("").is_err());
        assert!(Selector::parse("invalid").is_err());
    }

    #[test]
    fn test_matches_simple_field() {
        let selector = Selector::parse("status=UP").unwrap();
        let instance = create_test_instance(InstanceStatus::Up, None);
        assert!(selector.matches(&instance));

        let instance = create_test_instance(InstanceStatus::Down, None);
        assert!(!selector.matches(&instance));
    }

    #[test]
    fn test_matches_not_equal() {
        let selector = Selector::parse("status!=DOWN").unwrap();
        let instance = create_test_instance(InstanceStatus::Up, None);
        assert!(selector.matches(&instance));

        let instance = create_test_instance(InstanceStatus::Down, None);
        assert!(!selector.matches(&instance));
    }

    #[test]
    fn test_matches_multiple_conditions() {
        let selector = Selector::parse("status=UP,app=test-app").unwrap();
        let instance = create_test_instance(InstanceStatus::Up, None);
        assert!(selector.matches(&instance));

        let mut instance = create_test_instance(InstanceStatus::Up, None);
        instance.app = "other-app".to_string();
        assert!(!selector.matches(&instance));
    }

    #[test]
    fn test_matches_nested_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), Value::String("v2".to_string()));

        let selector = Selector::parse("metadata.version=v2").unwrap();
        let instance = create_test_instance(InstanceStatus::Up, Some(metadata));
        assert!(selector.matches(&instance));
    }
}
