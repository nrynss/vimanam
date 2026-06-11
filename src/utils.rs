use std::collections::HashMap;

use crate::models::{OpenApiSpec, Parameter, Response};

/// Resolves a JSON reference within the OpenAPI specification
pub fn resolve_ref(spec: &OpenApiSpec, reference: &str) -> Option<serde_json::Value> {
    if !reference.starts_with("#/") {
        return None; // We only support internal references for now
    }

    // Remove the #/ prefix
    let path = &reference[2..];
    let components = path.split('/');

    // Start with the spec as a JSON value
    let spec_json = serde_json::to_value(spec).ok()?;

    // Navigate the path
    let mut current = &spec_json;
    for component in components {
        // Handle escaped JSON pointer components
        let unescaped = component.replace("~1", "/").replace("~0", "~");

        if let Some(obj) = current.as_object() {
            if let Some(value) = obj.get(&unescaped) {
                current = value;
            } else {
                return None; // Component not found
            }
        } else if let Some(arr) = current.as_array() {
            if let Ok(index) = unescaped.parse::<usize>() {
                if index < arr.len() {
                    current = &arr[index];
                } else {
                    return None; // Index out of bounds
                }
            } else {
                return None; // Invalid array index
            }
        } else {
            return None; // Cannot navigate further
        }
    }

    Some(current.clone())
}

/// Resolves a parameter reference to a concrete parameter
pub fn resolve_parameter_ref(spec: &OpenApiSpec, parameter: &Parameter) -> Option<Parameter> {
    if let Some(extensions) = parameter.extensions.get("$ref") {
        if let Some(reference) = extensions.as_str() {
            if let Some(resolved) = resolve_ref(spec, reference) {
                return serde_json::from_value(resolved).ok();
            }
        }
    }
    Some(parameter.clone())
}

/// Resolves a response reference to a concrete response
pub fn resolve_response_ref(spec: &OpenApiSpec, response: &Response) -> Option<Response> {
    if let Some(extensions) = response.extensions.get("$ref") {
        if let Some(reference) = extensions.as_str() {
            if let Some(resolved) = resolve_ref(spec, reference) {
                return serde_json::from_value(resolved).ok();
            }
        }
    }
    Some(response.clone())
}

/// Extracts servers from the OpenAPI spec
pub fn extract_servers(spec: &OpenApiSpec) -> Vec<String> {
    let mut servers = Vec::new();

    // Check for servers array (OpenAPI 3.0+)
    if let Some(server_list) = &spec.servers {
        for server in server_list {
            servers.push(server.url.clone());
        }
    }
    // Check for host + basePath (OpenAPI 2.0)
    else if let Some(host) = spec.extensions.get("host") {
        if let Some(host_str) = host.as_str() {
            let mut base_url = if host_str.starts_with("http") {
                host_str.to_string()
            } else {
                format!("https://{}", host_str)
            };

            // Add basePath if present
            if let Some(base_path) = spec.extensions.get("basePath") {
                if let Some(path_str) = base_path.as_str() {
                    if !base_url.ends_with('/') && !path_str.starts_with('/') {
                        base_url.push('/');
                    }
                    base_url.push_str(path_str);
                }
            }

            servers.push(base_url);
        }
    }

    // Fallback to a default if empty
    if servers.is_empty() {
        servers.push("https://api.example.com".to_string());
    }

    servers
}

/// Extracts security schemes from the OpenAPI spec
pub fn extract_security_schemes(spec: &OpenApiSpec) -> HashMap<String, String> {
    let mut schemes = HashMap::new();

    // OpenAPI 3.0+: components.securitySchemes
    if let Some(components) = &spec.components {
        if let Some(security_schemes) = &components.security_schemes {
            for (name, scheme) in security_schemes {
                let desc = format!(
                    "{} ({})",
                    scheme.description.as_deref().unwrap_or(""),
                    scheme.security_type
                );
                schemes.insert(name.clone(), desc);
            }
        }
    }

    // OpenAPI 2.0: securityDefinitions
    if let Some(security_defs) = spec.extensions.get("securityDefinitions") {
        if let Some(defs_map) = security_defs.as_object() {
            for (name, def) in defs_map {
                if let Some(def_obj) = def.as_object() {
                    let type_str = def_obj
                        .get("type")
                        .and_then(|t| t.as_str())
                        .unwrap_or("unknown");

                    let desc = def_obj
                        .get("description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("");

                    schemes.insert(name.clone(), format!("{} ({})", desc, type_str));
                }
            }
        }
    }

    schemes
}

/// Cleans a string for use as an ID or anchor in Markdown
pub fn clean_for_id(input: &str) -> String {
    input
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "-")
        .replace("--", "-")
        .trim_matches('-')
        .to_string()
}

/// Extracts the primary content type from responses
pub fn extract_content_type(response: &Response) -> Option<String> {
    if let Some(content) = &response.content {
        if !content.is_empty() {
            return content.keys().next().map(|s| s.to_string());
        }
    }

    // For OpenAPI 2.0, infer from schema
    if response.schema.is_some() {
        return Some("application/json".to_string());
    }

    None
}
