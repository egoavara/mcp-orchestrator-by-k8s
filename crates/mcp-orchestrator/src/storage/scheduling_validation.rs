use crate::error::AppError;
use k8s_openapi::api::core::v1::{Affinity, NodeSelectorRequirement, NodeSelectorTerm};
use regex::Regex;
use std::collections::BTreeMap;

pub fn validate_label_key(key: &str) -> Result<(), AppError> {
    let parts: Vec<&str> = key.split('/').collect();

    match parts.len() {
        1 => validate_label_name(parts[0]),
        2 => {
            validate_dns_subdomain(parts[0])?;
            validate_label_name(parts[1])
        }
        _ => Err(AppError::InvalidInput(
            "Label key must be [prefix/]name format".into(),
        )),
    }
}

pub fn validate_label_name(name: &str) -> Result<(), AppError> {
    if name.is_empty() || name.len() > 63 {
        return Err(AppError::InvalidInput(format!(
            "Label name must be 1-63 chars, got {} chars",
            name.len()
        )));
    }

    let re = Regex::new(r"^[a-zA-Z0-9]([-_.a-zA-Z0-9]*[a-zA-Z0-9])?$").unwrap();
    if !re.is_match(name) {
        return Err(AppError::InvalidInput(format!(
            "Label name '{}' must start and end with alphanumeric, contain only alphanumeric, dash, underscore, dot",
            name
        )));
    }

    Ok(())
}

pub fn validate_dns_subdomain(subdomain: &str) -> Result<(), AppError> {
    if subdomain.is_empty() || subdomain.len() > 253 {
        return Err(AppError::InvalidInput(format!(
            "DNS subdomain must be 1-253 chars, got {} chars",
            subdomain.len()
        )));
    }

    let re =
        Regex::new(r"^[a-z0-9]([-a-z0-9]*[a-z0-9])?(\.[a-z0-9]([-a-z0-9]*[a-z0-9])?)*$").unwrap();
    if !re.is_match(subdomain) {
        return Err(AppError::InvalidInput(format!(
            "DNS subdomain '{}' must be lowercase, alphanumeric, dash, dot",
            subdomain
        )));
    }

    Ok(())
}

pub fn validate_label_value(value: &str) -> Result<(), AppError> {
    if value.len() > 63 {
        return Err(AppError::InvalidInput(format!(
            "Label value must be max 63 chars, got {} chars",
            value.len()
        )));
    }

    if value.is_empty() {
        return Ok(());
    }

    let re = Regex::new(r"^[a-zA-Z0-9]([-_.a-zA-Z0-9]*[a-zA-Z0-9])?$").unwrap();
    if !re.is_match(value) {
        return Err(AppError::InvalidInput(format!(
            "Label value '{}' must start and end with alphanumeric, contain only alphanumeric, dash, underscore, dot",
            value
        )));
    }

    Ok(())
}

pub fn validate_node_selector(selector: &BTreeMap<String, String>) -> Result<(), AppError> {
    if selector.is_empty() {
        return Err(AppError::InvalidInput(
            "NodeSelector cannot be empty".into(),
        ));
    }

    for (key, value) in selector {
        validate_label_key(key)?;
        validate_label_value(value)?;
    }

    Ok(())
}

pub fn validate_node_affinity(affinity: &Affinity) -> Result<(), AppError> {
    let Some(node_affinity) = &affinity.node_affinity else {
        return Ok(());
    };

    if let Some(required) = &node_affinity.required_during_scheduling_ignored_during_execution {
        if required.node_selector_terms.is_empty() {
            return Err(AppError::InvalidInput(
                "Required node affinity must have at least one term".into(),
            ));
        }
        for term in &required.node_selector_terms {
            validate_node_selector_term(term)?;
        }
    }

    if let Some(preferred) = &node_affinity.preferred_during_scheduling_ignored_during_execution {
        for pref_term in preferred {
            if pref_term.weight < 1 || pref_term.weight > 100 {
                return Err(AppError::InvalidInput(format!(
                    "Preferred term weight must be 1-100, got {}",
                    pref_term.weight
                )));
            }
            validate_node_selector_term(&pref_term.preference)?;
        }
    }

    Ok(())
}

pub fn validate_node_selector_term(term: &NodeSelectorTerm) -> Result<(), AppError> {
    let has_expressions = term
        .match_expressions
        .as_ref()
        .map_or(false, |e| !e.is_empty());
    let has_fields = term.match_fields.as_ref().map_or(false, |f| !f.is_empty());

    if !has_expressions && !has_fields {
        return Err(AppError::InvalidInput(
            "NodeSelectorTerm must have at least one match expression or match field".into(),
        ));
    }

    if let Some(expressions) = &term.match_expressions {
        for expr in expressions {
            validate_node_selector_requirement(expr)?;
        }
    }

    if let Some(fields) = &term.match_fields {
        for field in fields {
            validate_node_selector_requirement(field)?;
        }
    }

    Ok(())
}

pub fn validate_node_selector_requirement(req: &NodeSelectorRequirement) -> Result<(), AppError> {
    validate_label_key(&req.key)?;

    let valid_operators = ["In", "NotIn", "Exists", "DoesNotExist", "Gt", "Lt"];
    if !valid_operators.contains(&req.operator.as_str()) {
        return Err(AppError::InvalidInput(format!(
            "Invalid operator: '{}'. Must be one of: {:?}",
            req.operator, valid_operators
        )));
    }

    match req.operator.as_str() {
        "In" | "NotIn" | "Gt" | "Lt" => {
            if req.values.as_ref().map_or(true, |v| v.is_empty()) {
                return Err(AppError::InvalidInput(format!(
                    "Operator '{}' requires at least one value",
                    req.operator
                )));
            }
            if let Some(values) = &req.values {
                for value in values {
                    validate_label_value(value)?;
                }
            }
        }
        "Exists" | "DoesNotExist" => {
            if req.values.as_ref().map_or(false, |v| !v.is_empty()) {
                return Err(AppError::InvalidInput(format!(
                    "Operator '{}' must not have values",
                    req.operator
                )));
            }
        }
        _ => {}
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_label_key_simple() {
        assert!(validate_label_key("region").is_ok());
        assert!(validate_label_key("gpu-type").is_ok());
        assert!(validate_label_key("a1").is_ok());
    }

    #[test]
    fn test_validate_label_key_with_prefix() {
        assert!(validate_label_key("kubernetes.io/hostname").is_ok());
        assert!(validate_label_key("topology.kubernetes.io/zone").is_ok());
    }

    #[test]
    fn test_validate_label_key_invalid() {
        assert!(validate_label_key("").is_err());
        assert!(validate_label_key("-invalid").is_err());
        assert!(validate_label_key("invalid-").is_err());
        assert!(validate_label_key("a/b/c").is_err());
    }

    #[test]
    fn test_validate_label_value() {
        assert!(validate_label_value("us-west").is_ok());
        assert!(validate_label_value("true").is_ok());
        assert!(validate_label_value("").is_ok());
    }

    #[test]
    fn test_validate_label_value_invalid() {
        assert!(validate_label_value("-invalid").is_err());
        assert!(validate_label_value("invalid-").is_err());
        assert!(validate_label_value(&"a".repeat(64)).is_err());
    }

    #[test]
    fn test_validate_node_selector() {
        let mut selector = BTreeMap::new();
        selector.insert("region".to_string(), "us-west".to_string());
        selector.insert("gpu".to_string(), "true".to_string());
        assert!(validate_node_selector(&selector).is_ok());
    }

    #[test]
    fn test_validate_node_selector_empty() {
        let selector = BTreeMap::new();
        assert!(validate_node_selector(&selector).is_err());
    }

    #[test]
    fn test_validate_node_selector_requirement() {
        let req = NodeSelectorRequirement {
            key: "region".to_string(),
            operator: "In".to_string(),
            values: Some(vec!["us-west".to_string()]),
        };
        assert!(validate_node_selector_requirement(&req).is_ok());
    }

    #[test]
    fn test_validate_node_selector_requirement_exists() {
        let req = NodeSelectorRequirement {
            key: "gpu".to_string(),
            operator: "Exists".to_string(),
            values: None,
        };
        assert!(validate_node_selector_requirement(&req).is_ok());
    }

    #[test]
    fn test_validate_node_selector_requirement_invalid_operator() {
        let req = NodeSelectorRequirement {
            key: "region".to_string(),
            operator: "InvalidOp".to_string(),
            values: Some(vec!["us-west".to_string()]),
        };
        assert!(validate_node_selector_requirement(&req).is_err());
    }
}
