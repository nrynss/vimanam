use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::models::{ApiDocumentation, Endpoint, OpenApiSpec, Service};

pub fn parse_openapi<P: AsRef<Path>>(path: P) -> Result<ApiDocumentation> {
    let file = File::open(path).context("Failed to open OpenAPI file")?;
    let reader = BufReader::new(file);
    let spec: OpenApiSpec =
        serde_json::from_reader(reader).context("Failed to parse OpenAPI JSON")?;

    let services = extract_services(&spec);
    let endpoints = extract_endpoints(&spec, &services);

    Ok(ApiDocumentation {
        title: spec.info.title,
        version: spec.info.version,
        description: spec.info.description,
        services,
        endpoints,
    })
}

fn extract_services(spec: &OpenApiSpec) -> Vec<Service> {
    // Extract services from tags
    let mut services = Vec::new();

    if let Some(tags) = &spec.tags {
        for tag in tags {
            services.push(Service {
                name: tag.name.clone(),
                description: tag.description.clone(),
            });
        }
    }

    // If no tags, try to infer services from endpoint tags
    if services.is_empty() {
        let mut service_names = HashSet::new();

        for (_, path_item) in &spec.paths {
            for operation in [
                &path_item.get,
                &path_item.post,
                &path_item.put,
                &path_item.delete,
                &path_item.options,
                &path_item.head,
                &path_item.patch,
                &path_item.trace,
            ] {
                if let Some(op) = operation {
                    if let Some(tags) = &op.tags {
                        for tag in tags {
                            service_names.insert(tag.clone());
                        }
                    }
                }
            }
        }

        // Convert HashSet to Vec of Services
        for name in service_names {
            services.push(Service {
                name,
                description: None,
            });
        }
    }

    services
}

fn extract_endpoints(spec: &OpenApiSpec, services: &[Service]) -> Vec<Endpoint> {
    let mut endpoints = Vec::new();

    // A map of service names to ensure all endpoints are associated with valid services
    let service_map: HashSet<String> = services.iter().map(|s| s.name.clone()).collect();

    for (path, path_item) in &spec.paths {
        let operations = [
            ("get", &path_item.get),
            ("post", &path_item.post),
            ("put", &path_item.put),
            ("delete", &path_item.delete),
            ("options", &path_item.options),
            ("head", &path_item.head),
            ("patch", &path_item.patch),
            ("trace", &path_item.trace),
        ];

        for (method, operation_opt) in operations {
            if let Some(operation) = operation_opt {
                let service_tags = if let Some(tags) = &operation.tags {
                    // Filter to only include valid services
                    tags.iter()
                        .filter(|tag| service_map.contains(*tag))
                        .cloned()
                        .collect()
                } else {
                    // If no tags, use "Untagged" as the service
                    vec!["Untagged".to_string()]
                };

                // Only collect parameters if they exist
                let parameters = operation.parameters.clone().unwrap_or_default();

                endpoints.push(Endpoint {
                    path: path.clone(),
                    method: method.to_uppercase(),
                    services: service_tags,
                    summary: operation.summary.clone(),
                    description: operation.description.clone(),
                    operation_id: operation.operation_id.clone(),
                    parameters,
                    responses: operation.responses.clone(),
                    deprecated: operation.deprecated.unwrap_or(false),
                });
            }
        }
    }

    endpoints
}
