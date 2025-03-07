// Function to generate just a summary of services and endpoints
fn generate_summary<W: Write>(
    writer: &mut W,
    doc: &ApiDocumentation,
    config: &DocConfig,
) -> Result<()> {
    // Write title
    writeln!(writer, "# {}", doc.title)?;
    if let Some(description) = &doc.description {
        writeln!(writer, "\n{}\n", description)?;
    }
    writeln!(writer, "API Version: {}\n", doc.version)?;

    // Filter services if needed
    let services = if let Some(filter) = &config.service_filter {
        let filter_set: HashSet<_> = filter.iter().collect();
        doc.services
            .iter()
            .filter(|s| filter_set.contains(&s.name))
            .collect::<Vec<_>>()
    } else {
        doc.services.iter().collect()
    };

    // Group endpoints by service
    let mut service_endpoints: HashMap<&str, Vec<&Endpoint>> = HashMap::new();
    for endpoint in &doc.endpoints {
        // Skip deprecated endpoints if configured
        if config.exclude_deprecated && endpoint.deprecated {
            continue;
        }

        // Apply method filter if configured
        if let Some(methods) = &config.method_filter {
            if !methods.contains(&endpoint.method) {
                continue;
            }
        }

        // Apply path filter if configured
        if let Some(path_pattern) = &config.path_filter {
            if !endpoint.path.contains(path_pattern) {
                continue;
            }
        }

        for service_name in &endpoint.services {
            service_endpoints
                .entry(service_name)
                .or_insert_with(Vec::new)
                .push(endpoint);
        }
    }

    // Write Services List
    writeln!(writer, "## Services")?;

    for service in &services {
        writeln!(writer, "- {}", service.name)?;

        // Add operation links under each service
        if let Some(endpoints) = service_endpoints.get(&service.name as &str) {
            let mut sorted_ops = endpoints.clone();
            match config.sort_method {
                crate::models::SortMethod::Alphabetical => {
                    sorted_ops.sort_by(|a, b| {
                        a.operation_id
                            .clone()
                            .unwrap_or_default()
                            .cmp(&b.operation_id.clone().unwrap_or_default())
                    });
                }
                crate::models::SortMethod::PathLength => {
                    sorted_ops.sort_by(|a, b| a.path.len().cmp(&b.path.len()));
                }
                crate::models::SortMethod::None => {}
            }

            for endpoint in sorted_ops {
                let op_name = if let Some(operation_id) = &endpoint.operation_id {
                    // Clean up the operation ID by removing the service name prefix if present
                    if operation_id.starts_with(&format!("{}_", service.name)) {
                        // Remove the "ServiceName_" prefix
                        operation_id.replacen(&format!("{}_", service.name), "", 1)
                    } else {
                        operation_id.clone()
                    }
                } else {
                    // Fallback if no operation ID
                    format!("{} {}", endpoint.method, endpoint.path)
                };

                writeln!(writer, "  * {}", op_name)?;
            }
        }
    }

    Ok(())
}
use std::collections::{HashMap, HashSet};
use std::io::Write;

use anyhow::Result;

use crate::models::{ApiDocumentation, DetailLevel, DocConfig, Endpoint, GroupBy};

pub fn generate_markdown<W: Write>(
    writer: &mut W,
    doc: &ApiDocumentation,
    config: &DocConfig,
) -> Result<()> {
    // For summary level, just generate the TOC
    if config.detail_level == DetailLevel::Summary {
        generate_summary(writer, doc, config)
    } else {
        // For other detail levels, use the existing grouping logic
        match config.group_by {
            GroupBy::Service => generate_by_service(writer, doc, config),
            GroupBy::Method => generate_by_method(writer, doc, config),
            GroupBy::Path => generate_by_path(writer, doc, config),
            GroupBy::Tag => generate_by_tag(writer, doc, config),
            GroupBy::Flat => generate_flat(writer, doc, config),
        }
    }
}

fn generate_by_service<W: Write>(
    writer: &mut W,
    doc: &ApiDocumentation,
    config: &DocConfig,
) -> Result<()> {
    // Write title
    writeln!(writer, "# {}", doc.title)?;
    if let Some(description) = &doc.description {
        writeln!(writer, "\n{}\n", description)?;
    }
    writeln!(writer, "API Version: {}\n", doc.version)?;

    // Filter services if needed
    let services = if let Some(filter) = &config.service_filter {
        let filter_set: HashSet<_> = filter.iter().collect();
        doc.services
            .iter()
            .filter(|s| filter_set.contains(&s.name))
            .collect::<Vec<_>>()
    } else {
        doc.services.iter().collect()
    };

    // Group endpoints by service - MOVED THIS UP before TOC generation
    let mut service_endpoints: HashMap<&str, Vec<&Endpoint>> = HashMap::new();
    for endpoint in &doc.endpoints {
        // Skip deprecated endpoints if configured
        if config.exclude_deprecated && endpoint.deprecated {
            continue;
        }

        // Apply method filter if configured
        if let Some(methods) = &config.method_filter {
            if !methods.contains(&endpoint.method) {
                continue;
            }
        }

        // Apply path filter if configured
        if let Some(path_pattern) = &config.path_filter {
            if !endpoint.path.contains(path_pattern) {
                continue;
            }
        }

        for service_name in &endpoint.services {
            service_endpoints
                .entry(service_name)
                .or_insert_with(Vec::new)
                .push(endpoint);
        }
    }

    // Table of Contents (if enabled)
    if config.include_toc {
        writeln!(writer, "## Services\n")?;
        for service in &services {
            let anchor = to_anchor(&service.name);
            writeln!(writer, "- [{}](#{anchor})", service.name)?;

            // Add operation links under each service
            if let Some(endpoints) = service_endpoints.get(&service.name as &str) {
                let mut sorted_ops = endpoints.clone();
                match config.sort_method {
                    crate::models::SortMethod::Alphabetical => {
                        sorted_ops.sort_by(|a, b| {
                            a.summary
                                .clone()
                                .unwrap_or_default()
                                .cmp(&b.summary.clone().unwrap_or_default())
                        });
                    }
                    crate::models::SortMethod::PathLength => {
                        sorted_ops.sort_by(|a, b| a.path.len().cmp(&b.path.len()));
                    }
                    crate::models::SortMethod::None => {}
                }

                for endpoint in sorted_ops {
                    // Extract a shorter title for the TOC entry
                    let op_title = get_short_title(endpoint);
                    let op_anchor = to_anchor(&op_title);
                    writeln!(writer, "  * [{}](#{op_anchor})", op_title)?;
                }
            }
        }
        writeln!(writer)?;
    }

    // Write each service section
    for service in &services {
        // Create anchor but use it directly in the writeln! call
        writeln!(writer, "## {}", service.name)?;

        if let Some(description) = &service.description {
            writeln!(writer, "\n{}", description)?;
        }

        // Get endpoints for this service
        if let Some(endpoints) = service_endpoints.get(&service.name as &str) {
            // Sort endpoints as configured
            let mut sorted_endpoints = endpoints.clone();
            match config.sort_method {
                crate::models::SortMethod::Alphabetical => {
                    sorted_endpoints.sort_by(|a, b| a.path.cmp(&b.path));
                }
                crate::models::SortMethod::PathLength => {
                    sorted_endpoints.sort_by(|a, b| a.path.len().cmp(&b.path.len()));
                }
                crate::models::SortMethod::None => {}
            }

            for endpoint in sorted_endpoints {
                write_endpoint(writer, endpoint, config, true)?;
            }
        } else {
            writeln!(writer, "\nNo endpoints found for this service.\n")?;
        }
    }

    Ok(())
}

fn generate_by_method<W: Write>(
    writer: &mut W,
    doc: &ApiDocumentation,
    config: &DocConfig,
) -> Result<()> {
    // Write title
    writeln!(writer, "# {}", doc.title)?;
    if let Some(description) = &doc.description {
        writeln!(writer, "\n{}\n", description)?;
    }
    writeln!(writer, "API Version: {}\n", doc.version)?;

    // Group endpoints by method
    let mut method_endpoints: HashMap<&str, Vec<&Endpoint>> = HashMap::new();
    for endpoint in &doc.endpoints {
        // Skip deprecated endpoints if configured
        if config.exclude_deprecated && endpoint.deprecated {
            continue;
        }

        // Apply service filter if configured
        if let Some(services) = &config.service_filter {
            if !endpoint.services.iter().any(|s| services.contains(s)) {
                continue;
            }
        }

        // Apply path filter if configured
        if let Some(path_pattern) = &config.path_filter {
            if !endpoint.path.contains(path_pattern) {
                continue;
            }
        }

        // Apply method filter if configured
        if let Some(methods) = &config.method_filter {
            if !methods.contains(&endpoint.method) {
                continue;
            }
        }

        method_endpoints
            .entry(&endpoint.method)
            .or_insert_with(Vec::new)
            .push(endpoint);
    }

    // Table of Contents (if enabled)
    if config.include_toc {
        writeln!(writer, "## HTTP Methods\n")?;
        for method in [
            "GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS", "HEAD", "TRACE",
        ] {
            if let Some(endpoints) = method_endpoints.get(method) {
                if !endpoints.is_empty() {
                    let anchor = to_anchor(method);
                    writeln!(writer, "- [{}](#{anchor})", method)?;
                }
            }
        }
        writeln!(writer)?;
    }

    // Write each method section
    for method in [
        "GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS", "HEAD", "TRACE",
    ] {
        if let Some(endpoints) = method_endpoints.get(method) {
            if !endpoints.is_empty() {
                let anchor = to_anchor(method);
                writeln!(writer, "## {} {{{}}}", method, anchor)?;

                // Sort endpoints as configured
                let mut sorted_endpoints = endpoints.clone();
                match config.sort_method {
                    crate::models::SortMethod::Alphabetical => {
                        sorted_endpoints.sort_by(|a, b| a.path.cmp(&b.path));
                    }
                    crate::models::SortMethod::PathLength => {
                        sorted_endpoints.sort_by(|a, b| a.path.len().cmp(&b.path.len()));
                    }
                    crate::models::SortMethod::None => {}
                }

                for endpoint in sorted_endpoints {
                    write_endpoint(writer, endpoint, config, true)?;
                }
            }
        }
    }

    Ok(())
}

fn generate_by_path<W: Write>(
    writer: &mut W,
    _doc: &ApiDocumentation,
    _config: &DocConfig,
) -> Result<()> {
    // Implementation for path-based grouping
    // This is a placeholder - you would implement similar to the other grouping methods
    writeln!(writer, "# Path-based grouping not fully implemented yet")?;
    Ok(())
}

fn generate_by_tag<W: Write>(
    writer: &mut W,
    _doc: &ApiDocumentation,
    _config: &DocConfig,
) -> Result<()> {
    // Implementation for tag-based grouping
    // This is a placeholder - you would implement similar to the other grouping methods
    writeln!(writer, "# Tag-based grouping not fully implemented yet")?;
    Ok(())
}

fn generate_flat<W: Write>(
    writer: &mut W,
    _doc: &ApiDocumentation,
    _config: &DocConfig,
) -> Result<()> {
    // Implementation for flat listing
    // This is a placeholder - you would implement similar to the other grouping methods
    writeln!(writer, "# Flat listing not fully implemented yet")?;
    Ok(())
}

fn write_endpoint<W: Write>(
    writer: &mut W,
    endpoint: &Endpoint,
    config: &DocConfig,
    include_heading: bool,
) -> Result<()> {
    let title = get_short_title(endpoint);

    if include_heading {
        writeln!(writer, "### {}", title)?;
    } else {
        writeln!(writer, "**{}**", title)?;
    }

    // Operation line (method + path)
    writeln!(
        writer,
        "**Operation:** {} {}",
        endpoint.method, endpoint.path
    )?;

    // Description/summary only if it exists
    if let Some(description) = &endpoint.description {
        writeln!(writer, "**Description:** {}", description)?;
    } else if let Some(summary) = &endpoint.summary {
        writeln!(writer, "**Description:** {}", summary)?;
    }

    if endpoint.deprecated {
        writeln!(writer, "\n> **Deprecated**: This endpoint is deprecated.")?;
    }

    // Write operation ID if available
    if let Some(operation_id) = &endpoint.operation_id {
        writeln!(writer, "**Operation ID:** `{}`", operation_id)?;
    }

    // Only include detailed information if detail level is not basic
    if config.detail_level != DetailLevel::Basic {
        // Write parameters based on detail level
        if !endpoint.parameters.is_empty() {
            writeln!(writer, "\n#### Parameters")?;

            // More detailed parameter listing
            writeln!(writer, "| Name | In | Required | Description |")?;
            writeln!(writer, "|------|----|---------:|-------------|")?;

            for param in &endpoint.parameters {
                // Skip non-required parameters if required_only is enabled
                if let Some(required) = param.required {
                    if !required && config.required_only {
                        continue;
                    }
                }

                let required_str = if let Some(req) = param.required {
                    if req {
                        "Yes"
                    } else {
                        "No"
                    }
                } else {
                    "No"
                };

                let desc = param.description.as_deref().unwrap_or("-");
                writeln!(
                    writer,
                    "| `{}` | {} | {} | {} |",
                    param.name, param.parameter_in, required_str, desc
                )?;
            }
        }

        // Write responses based on detail level
        writeln!(writer, "\n#### Responses")?;
        writeln!(writer, "| Code | Description |")?;
        writeln!(writer, "|------|-------------|")?;

        for (code, response) in &endpoint.responses {
            let desc = response.description.as_deref().unwrap_or("-");
            writeln!(writer, "| {} | {} |", code, desc)?;
        }

        // Add schemas if configured
        if config.include_schemas && config.detail_level == DetailLevel::Full {
            // This would be implemented in a full version
            writeln!(writer, "\n<!-- Schemas would be included here -->")?;
        }

        // Add examples if configured
        if config.include_examples && config.detail_level == DetailLevel::Full {
            // This would be implemented in a full version
            writeln!(writer, "\n<!-- Examples would be included here -->")?;
        }
    }

    writeln!(writer)?; // End with a blank line
    Ok(())
}

// Remove this function as we've merged it into write_endpoint
// fn write_endpoint_details<W: Write>(
//     writer: &mut W,
//     endpoint: &Endpoint,
//     config: &DocConfig,
// ) -> Result<()> {
//     // ...
// }

// Helper function to convert a string to a Markdown anchor
fn to_anchor(text: &str) -> String {
    text.to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect()
}

// Helper function to get a shorter title for an endpoint
fn get_short_title(endpoint: &Endpoint) -> String {
    if let Some(operation_id) = &endpoint.operation_id {
        // If we have an operation ID, use it
        return operation_id.clone();
    } else if let Some(summary) = &endpoint.summary {
        // If there's a summary, try to extract the operation name (first word or camelCase part)
        if let Some(first_word) = summary.split_whitespace().next() {
            if first_word.chars().any(|c| c.is_uppercase()) {
                // This is likely a camelCase operation name
                return first_word.to_string();
            }
        }
        // If no good first word, just use the whole summary
        return summary.clone();
    }

    // Fallback to method and path
    format!("{} {}", endpoint.method, endpoint.path)
}
