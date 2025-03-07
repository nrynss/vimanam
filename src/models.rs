use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OpenAPI spec model. This is a simplified version focused on what we need for documentation.
#[derive(Debug, Deserialize, Serialize)]
pub struct OpenApiSpec {
    pub swagger: String,
    pub info: Info,
    pub tags: Option<Vec<Tag>>,
    pub paths: HashMap<String, PathItem>,
    // We're not including all OpenAPI fields, just what we need
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Tag {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PathItem {
    #[serde(rename = "get", skip_serializing_if = "Option::is_none")]
    pub get: Option<Operation>,
    #[serde(rename = "put", skip_serializing_if = "Option::is_none")]
    pub put: Option<Operation>,
    #[serde(rename = "post", skip_serializing_if = "Option::is_none")]
    pub post: Option<Operation>,
    #[serde(rename = "delete", skip_serializing_if = "Option::is_none")]
    pub delete: Option<Operation>,
    #[serde(rename = "options", skip_serializing_if = "Option::is_none")]
    pub options: Option<Operation>,
    #[serde(rename = "head", skip_serializing_if = "Option::is_none")]
    pub head: Option<Operation>,
    #[serde(rename = "patch", skip_serializing_if = "Option::is_none")]
    pub patch: Option<Operation>,
    #[serde(rename = "trace", skip_serializing_if = "Option::is_none")]
    pub trace: Option<Operation>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Operation {
    pub tags: Option<Vec<String>>,
    pub summary: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "operationId")]
    pub operation_id: Option<String>,
    pub parameters: Option<Vec<Parameter>>,
    pub responses: HashMap<String, Response>,
    pub deprecated: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Parameter {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "in")]
    pub parameter_in: String,
    pub required: Option<bool>,
    pub schema: Option<Schema>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Schema {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub schema_type: Option<String>,
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Response {
    pub description: Option<String>,
    pub schema: Option<Schema>,
}

/// Intermediate representation for documentation generation
#[derive(Debug)]
pub struct ApiDocumentation {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
    pub services: Vec<Service>,
    pub endpoints: Vec<Endpoint>,
}

#[derive(Debug, Clone)]
pub struct Service {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Endpoint {
    pub path: String,
    pub method: String,
    pub services: Vec<String>, // References to service names
    pub summary: Option<String>,
    pub description: Option<String>,
    pub operation_id: Option<String>,
    pub parameters: Vec<Parameter>,
    pub responses: HashMap<String, Response>,
    pub deprecated: bool,
}

/// Configuration for documentation generation
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DocConfig {
    pub group_by: GroupBy,
    pub service_filter: Option<Vec<String>>,
    pub path_filter: Option<String>,
    pub method_filter: Option<Vec<String>>,
    pub exclude_deprecated: bool,
    pub required_only: bool,
    pub detail_level: DetailLevel,
    pub include_schemas: bool,
    pub include_examples: bool,
    pub include_auth: bool,
    pub include_toc: bool,
    pub output_format: OutputFormat,
    pub sort_method: SortMethod,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GroupBy {
    Service,
    Method,
    Path,
    Tag,
    Flat,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DetailLevel {
    Summary,
    Basic,
    Standard,
    Full,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Markdown,
    Html,
    Docusaurus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortMethod {
    Alphabetical,
    PathLength,
    None,
}
