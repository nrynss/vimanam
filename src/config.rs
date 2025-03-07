use clap::{Parser, ValueEnum};
use std::path::PathBuf;

use crate::models::{DetailLevel, DocConfig, GroupBy, OutputFormat, SortMethod};

#[derive(Parser, Debug)]
#[command(name = "vimanam")]
#[command(about = "OpenAPI to Markdown documentation generator", long_about = None)]
pub struct Cli {
    /// Path to the OpenAPI JSON file
    #[arg(value_name = "FILE")]
    pub input: PathBuf,

    /// Output file path
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Group endpoints by HTTP method instead of by service
    #[arg(long)]
    pub method: bool,

    /// Grouping method for endpoints
    #[arg(long, value_enum, default_value = "service")]
    pub group_by: Option<GroupByArg>,

    /// Generate a flat list without hierarchical structure
    #[arg(long)]
    pub flat: bool,

    /// Include only specific services (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub service_filter: Option<Vec<String>>,

    /// Filter endpoints by path pattern
    #[arg(long)]
    pub path_filter: Option<String>,

    /// Filter by HTTP methods (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub method_filter: Option<Vec<String>>,

    /// Hide deprecated endpoints
    #[arg(long)]
    pub exclude_deprecated: bool,

    /// Only show required parameters
    #[arg(long)]
    pub required_only: bool,

    /// Control amount of information
    #[arg(long, value_enum, default_value = "summary")]
    pub detail: DetailLevelArg,

    /// Include request/response schemas
    #[arg(long)]
    pub include_schemas: bool,

    /// Include request/response examples
    #[arg(long)]
    pub include_examples: bool,

    /// Show authentication requirements
    #[arg(long)]
    pub include_auth: bool,

    /// Skip table of contents
    #[arg(long)]
    pub no_toc: bool,

    /// Output format
    #[arg(long, value_enum, default_value = "markdown")]
    pub format: FormatArg,

    /// Use custom template
    #[arg(long)]
    pub template: Option<PathBuf>,

    /// Sorting method
    #[arg(long, value_enum, default_value = "alpha")]
    pub sort: SortArg,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum GroupByArg {
    Service,
    Method,
    Path,
    Tag,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum DetailLevelArg {
    Summary,
    Basic,
    Standard,
    Full,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum FormatArg {
    Markdown,
    Html,
    Docusaurus,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum SortArg {
    Alpha,
    PathLength,
    None,
}

impl From<GroupByArg> for GroupBy {
    fn from(arg: GroupByArg) -> Self {
        match arg {
            GroupByArg::Service => GroupBy::Service,
            GroupByArg::Method => GroupBy::Method,
            GroupByArg::Path => GroupBy::Path,
            GroupByArg::Tag => GroupBy::Tag,
        }
    }
}

impl From<DetailLevelArg> for DetailLevel {
    fn from(arg: DetailLevelArg) -> Self {
        match arg {
            DetailLevelArg::Summary => DetailLevel::Summary,
            DetailLevelArg::Basic => DetailLevel::Basic,
            DetailLevelArg::Standard => DetailLevel::Standard,
            DetailLevelArg::Full => DetailLevel::Full,
        }
    }
}

impl From<FormatArg> for OutputFormat {
    fn from(arg: FormatArg) -> Self {
        match arg {
            FormatArg::Markdown => OutputFormat::Markdown,
            FormatArg::Html => OutputFormat::Html,
            FormatArg::Docusaurus => OutputFormat::Docusaurus,
        }
    }
}

impl From<SortArg> for SortMethod {
    fn from(arg: SortArg) -> Self {
        match arg {
            SortArg::Alpha => SortMethod::Alphabetical,
            SortArg::PathLength => SortMethod::PathLength,
            SortArg::None => SortMethod::None,
        }
    }
}

pub fn build_config(cli: &Cli) -> DocConfig {
    // Determine grouping method
    let group_by = if cli.flat {
        GroupBy::Flat
    } else if cli.method {
        GroupBy::Method
    } else if let Some(group_by) = cli.group_by {
        group_by.into()
    } else {
        GroupBy::Service
    };

    DocConfig {
        group_by,
        service_filter: cli.service_filter.clone(),
        path_filter: cli.path_filter.clone(),
        method_filter: cli.method_filter.clone(),
        exclude_deprecated: cli.exclude_deprecated,
        required_only: cli.required_only,
        detail_level: cli.detail.into(),
        include_schemas: cli.include_schemas,
        include_examples: cli.include_examples,
        include_auth: cli.include_auth,
        include_toc: !cli.no_toc,
        output_format: cli.format.into(),
        sort_method: cli.sort.into(),
    }
}
