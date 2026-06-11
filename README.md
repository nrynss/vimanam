# Vimanam

Vimanam is an OpenAPI/Swagger JSON to Markdown documentation generator.

Vimanam stands for Aeroplane in Malayalam. Like an aeroplane, it can fly high and give you a 20,000 feet view of the APIs. It can fly low and give you a detailed view of the APIs. You can also run it along the ground to look deep into the API fields and descriptions.

It supports both OpenAPI 2.0 (Swagger) and OpenAPI 3.0 specifications.

## Features

- Convert OpenAPI JSON files to Markdown documentation
- Supports both OpenAPI 2.0 (Swagger) and OpenAPI 3.0 specifications
- Group endpoints by service or HTTP method, or list them flat
- Filter by service, path, or method
- Multiple detail levels (summary, basic, standard, full)
- Reference resolution to follow JSON references (`$ref`) in specifications
- Server URL information extraction and documentation
- Authentication and security schemes documentation
- Proper content type detection for responses
- Sorting options for endpoints (alphabetical, path length)
- Clean anchor generation for better navigation

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70.0 or later)
- [Git](https://git-scm.com/downloads)

## Installation

```bash
# Clone repository
git clone https://github.com/nrynss/vimanam.git
cd vimanam

# Run directly without building (development)
cargo run -- input.json -o output.md

# Build
cargo build --release

# Run the built binary
./target/release/vimanam input.json -o output.md

# Or install system-wide
cargo install --path .
```

## Usage

```bash
# Basic usage
vimanam input.json -o output.md

# Group by HTTP method
vimanam input.json --method -o output.md

# Generate summary only
vimanam input.json --detail summary -o output.md

# Filter by specific services
vimanam input.json --service-filter Auth,Users -o output.md

# Filter by HTTP method
vimanam input.json --method-filter GET,POST -o output.md

# Show only paths containing a pattern
vimanam input.json --path-filter /api/v1 -o output.md

# Generate full details
vimanam input.json --detail full --include-schemas --include-examples -o output.md

# Include server and authentication information
vimanam input.json --include-auth -o output.md
```

## Options

```
Usage: vimanam [OPTIONS] <FILE>

Arguments:
  <FILE>  Path to the OpenAPI JSON file

Options:
  -o, --output <FILE>                      Output file path
      --method                             Group endpoints by HTTP method instead of by service
      --group-by <service|method>          Grouping method for endpoints
      --flat                               Generate a flat list without hierarchical structure
      --service-filter <SERVICE[,...]>     Include only specific services (comma-separated)
      --path-filter <PATTERN>              Filter endpoints by path pattern
      --method-filter <METHOD[,...]>       Filter by HTTP methods (comma-separated)
      --exclude-deprecated                 Hide deprecated endpoints
      --required-only                      Only show required parameters
      --detail <summary|basic|standard|full> Control amount of information [default: summary]
      --include-schemas                    Include request/response schemas
      --include-examples                   Include request/response examples
      --include-auth                       Show authentication requirements and server URLs
      --no-toc                             Skip table of contents
      --sort <alpha|path-length|none>      Sorting method [default: alpha]
  -h, --help                               Print help
```

## Supported OpenAPI Versions

Vimanam supports:
- OpenAPI 2.0 (Swagger) documents using the `swagger` field
- OpenAPI 3.0+ documents using the `openapi` field

## Output Examples

The generated documentation includes:

### Server and Authentication Information
```markdown
## Server URLs
* https://api.example.com/v1
* https://dev-api.example.com/v1

## Authentication
* **apiKeyAuth**: API Key authentication (apiKey)
* **oauth2**: OAuth 2.0 authorization (oauth2)
```

### Endpoint Documentation
```markdown
### createUser {#createuser}
**Operation:** POST /users

**Description:** Create a new user account
**Operation ID:** `createUser`

#### Parameters
| Name | In | Required | Description |
|------|----|---------:|-------------|
| `body` | body | Yes | User information |

#### Responses
| Code | Type | Description |
|------|------|-------------|
| 201 | application/json | User created successfully |
| 400 | application/json | Invalid request |
```

## License

Apache License 2.0
