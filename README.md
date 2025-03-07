# Vimanam

OpenAPI/Swagger JSON to Markdown documentation generator.

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
```

## Options

```
Usage: vimanam [OPTIONS] <FILE>

Arguments:
  <FILE>  Path to the OpenAPI JSON file

Options:
  -o, --output <FILE>                      Output file path
      --method                             Group endpoints by HTTP method instead of by service
      --service-filter <SERVICE[,...]>     Include only specific services (comma-separated)
      --path-filter <PATTERN>              Filter endpoints by path pattern
      --method-filter <METHOD[,...]>       Filter by HTTP methods (comma-separated)
      --exclude-deprecated                 Hide deprecated endpoints
      --required-only                      Only show required parameters
      --detail <summary|basic|standard|full> Control amount of information [default: basic]
      --no-toc                             Skip table of contents
      --sort <alpha|path-length|none>      Sorting method [default: alpha]
  -h, --help                               Print help
```

## License

Apache License 2.0
