use std::str::FromStr;

/// Output format for CLI commands
#[derive(Debug, Clone, PartialEq, Default)]
pub enum OutputFormat {
    /// Table format (kubectl style, no borders)
    #[default]
    Table,
    /// Wide table format with additional columns
    Wide,
    /// JSON format
    Json,
    /// YAML format
    Yaml,
    /// JSONPath query format
    JsonPath(String),
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "table" => Ok(OutputFormat::Table),
            "wide" => Ok(OutputFormat::Wide),
            "json" => Ok(OutputFormat::Json),
            "yaml" => Ok(OutputFormat::Yaml),
            s if s.starts_with("jsonpath=") => {
                let path = s.strip_prefix("jsonpath=")
                    .ok_or_else(|| "Invalid jsonpath format".to_string())?;
                Ok(OutputFormat::JsonPath(path.to_string()))
            }
            _ => Err(format!(
                "Invalid output format: {}. Valid formats: table, wide, json, yaml, jsonpath=<expr>",
                s
            )),
        }
    }
}

impl OutputFormat {
    /// Convert to legacy string format for backward compatibility
    /// TODO: Remove this once output layer is fully refactored
    pub fn as_legacy_str(&self) -> &str {
        match self {
            OutputFormat::Table | OutputFormat::Wide => "table",
            OutputFormat::Json | OutputFormat::JsonPath(_) => "json",
            OutputFormat::Yaml => "yaml",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_output_format() {
        assert_eq!("table".parse::<OutputFormat>().unwrap(), OutputFormat::Table);
        assert_eq!("wide".parse::<OutputFormat>().unwrap(), OutputFormat::Wide);
        assert_eq!("json".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
        assert_eq!("yaml".parse::<OutputFormat>().unwrap(), OutputFormat::Yaml);

        let jsonpath = "jsonpath={.instances[*].ipAddr}".parse::<OutputFormat>().unwrap();
        assert!(matches!(jsonpath, OutputFormat::JsonPath(_)));
    }

    #[test]
    fn test_invalid_format() {
        assert!("invalid".parse::<OutputFormat>().is_err());
    }
}
