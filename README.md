# Vimanam

[![CI](https://github.com/noemaforge/vimanam/actions/workflows/ci.yml/badge.svg)](https://github.com/noemaforge/vimanam/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/vimanam.svg)](https://crates.io/crates/vimanam)
[![License: Apache-2.0](https://img.shields.io/crates/l/vimanam.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.96-blue.svg)](Cargo.toml)

Vimanam is an OpenAPI/Swagger (JSON or YAML) to Markdown documentation generator.

Vimanam stands for Aeroplane in Malayalam. Like an aeroplane, it can fly high and give you a 20,000 feet view of the APIs. It can fly low and give you a detailed view of the APIs. You can also run it along the ground to look deep into the API fields and descriptions.

It supports both OpenAPI 2.0 (Swagger) and OpenAPI 3.0 specifications.

Besides producing documentation for humans, Vimanam is built for **feeding API specs to LLMs**: a multi-megabyte enterprise spec doesn't fit in a context window, but a filtered, summary-level Markdown rendering of it does. See [Preparing API context for LLMs](#preparing-api-context-for-llms).

## Features

- Convert OpenAPI JSON or YAML files to Markdown documentation (format detected by `.json`/`.yaml`/`.yml` extension, with automatic fallback)
- Supports both OpenAPI 2.0 (Swagger) and OpenAPI 3.0 specifications
- Group endpoints by service, HTTP method, or path, or list them flat
- Filter by service, path, or method
- Multiple detail levels (summary, basic, standard, full)
- Token-budget-aware output (`--max-tokens`): steps the detail level down until the rendering fits, and reports what was trimmed on stderr
- Token-budget dry run (`--stats`): a per-service table of endpoint counts and estimated token sizes, for sizing slices before choosing filters
- Spec hygiene report appended to every run (`--no-report` to skip): counts and lists operations missing a description, `operationId` or responses, deprecated and untagged operations, duplicate `operationId`s, and undescribed parameters
- Schema expansion at `--detail full --include-schemas`: renders request/response schemas as nested field tables. Shared component schemas are expanded once into a trailing "Schema Definitions" section and linked from each use site, keeping output compact when schemas are reused across endpoints; `--inline-schemas` instead expands every `$ref` inline at each use site (larger, fully self-contained, with cycle detection)
- Example rendering at `--detail full --include-examples`: emits request/response examples as fenced JSON blocks, resolving `$ref`s into `components/examples`
- Server URL information extraction and documentation
- Authentication and security schemes documentation
- Proper content type detection for responses
- Sorting options for endpoints (alphabetical, path length)
- Clean anchor generation for better navigation
- Deterministic, byte-identical output across runs — friendly to diffs, caching, and LLM prompt caching

## Installation

### Homebrew (macOS / Linux)

```bash
brew install noemaforge/tap/vimanam
```

No Rust toolchain required. Supports macOS (Apple Silicon & Intel) and x86_64 Linux.

### Install script (no toolchain)

```bash
# macOS / Linux
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/noemaforge/vimanam/releases/latest/download/vimanam-installer.sh | sh
```

```powershell
# Windows (PowerShell)
powershell -ExecutionPolicy ByPass -c "irm https://github.com/noemaforge/vimanam/releases/latest/download/vimanam-installer.ps1 | iex"
```

### From crates.io

```bash
cargo binstall vimanam   # prebuilt binary, no compile (needs cargo-binstall)
cargo install vimanam    # builds from source
```

### Prebuilt binaries

Download the archive for your platform (Linux, macOS Intel/ARM64, Windows) from the
[latest release](https://github.com/noemaforge/vimanam/releases/latest) — no Rust toolchain needed.
Archives are named `vimanam-<target-triple>.tar.xz` (`.zip` on Windows) and ship with a matching
`.sha256` checksum; each bundles the binary, `README.md`, `CHANGELOG.md`, and `LICENSE`. Extract it
and put the `vimanam` binary on your `PATH`.

### From source

Requires [Rust](https://www.rust-lang.org/tools/install) 1.96.0 or later.

```bash
# Clone repository
git clone https://github.com/noemaforge/vimanam.git
cd vimanam

# Run directly without building (development)
cargo run -- input.json -o output.md

# Build and run
cargo build --release
./target/release/vimanam input.json -o output.md

# Or install system-wide
cargo install --path .
```

### Shell completions

`vimanam completions <SHELL>` prints a completion script for `bash`, `zsh`, `fish`,
`powershell`, or `elvish`. Install it once and re-run after upgrading Vimanam:

```bash
# Bash (relies on the bash-completion v2 package, which stock macOS bash 3.2
# lacks; alternatively add `eval "$(vimanam completions bash)"` to ~/.bashrc)
mkdir -p ~/.local/share/bash-completion/completions
vimanam completions bash > ~/.local/share/bash-completion/completions/vimanam

# Zsh (make sure ~/.zfunc is on fpath before compinit runs in ~/.zshrc:
#   fpath+=~/.zfunc; autoload -Uz compinit; compinit)
mkdir -p ~/.zfunc
vimanam completions zsh > ~/.zfunc/_vimanam

# Fish
mkdir -p ~/.config/fish/completions
vimanam completions fish > ~/.config/fish/completions/vimanam.fish
```

```elvish
# Elvish: add this line to ~/.config/elvish/rc.elv
eval (vimanam completions elvish | slurp)
```

```powershell
# PowerShell: add this line to $PROFILE
vimanam completions powershell | Out-String | Invoke-Expression
```

## Usage

```bash
# Basic usage
vimanam input.json -o output.md

# Group by HTTP method
vimanam input.json --method -o output.md

# Group by path (one section per path, methods listed underneath)
vimanam input.json --group-by path -o output.md

# Generate summary only
vimanam input.json --detail summary -o output.md

# Fit the output to a token budget, stepping detail down as needed
vimanam input.json --detail full --max-tokens 8000 -o output.md

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

# Print a shell completion script (see "Shell completions" above)
vimanam completions zsh

# Drop the spec hygiene report appended after the documentation
vimanam input.json --no-report -o output.md

# Size each service before choosing filters: endpoint counts and ~tokens per service
vimanam input.json --stats --detail standard
```

### Spec hygiene report

Every run appends a short report after the documentation, separated by a horizontal rule, that flags common gaps in the spec: operations with no summary or description, no `operationId`, no documented responses, deprecated operations, operations with no tag (attributed to the default service), duplicate `operationId`s, and parameters without a description (a request body counts once per operation, however many media types it offers). It covers the same endpoints the documentation does, so `--service-filter`, `--path-filter`, `--method-filter` and `--exclude-deprecated` narrow the report too. Detail lists appear only for checks that found something.

```markdown
---

## Spec Hygiene Report

**6 endpoints** across **2 services**

| Check | Count |
|-------|------:|
| Missing description | 1 |
| Missing operationId | 1 |
| No responses documented | 1 |
| Deprecated | 2 |
| Untagged (no service tag) | 1 |
| Duplicate operationIds | 1 |
| Parameters without description | 3 |

### Missing description (1)
- `GET /health`

### Deprecated (2)
- `GET /ping`
- `DELETE /users/{id}`

### Duplicate operationIds (1)
- `getUser`
  - `DELETE /users/{id}`
  - `GET /users/{id}`

### Parameters without description (3)
- `GET /users` — `limit`
- `POST /users` — `requestBody`
- `DELETE /users/{id}` — `id`
```

Pass `--no-report` to omit it. The report is not counted against `--max-tokens` — the budget fits the documentation body only — so combine `--max-tokens` with `--no-report` when the whole output must stay within the budget.

## Options

```
Usage: vimanam [OPTIONS] <FILE>
       vimanam <COMMAND>

Commands:
  completions  Generate shell completions and print them to stdout
  help         Print this message or the help of the given subcommand(s)

Arguments:
  <FILE>  Path to the OpenAPI JSON file

Options:
  -o, --output <FILE>                      Output file path
      --method                             Group endpoints by HTTP method instead of by service
      --group-by <service|method|path>     Grouping method for endpoints
      --flat                               Generate a flat list without hierarchical structure
      --service-filter <SERVICE[,...]>     Include only specific services (comma-separated)
      --path-filter <PATTERN>              Filter endpoints by path pattern
      --method-filter <METHOD[,...]>       Filter by HTTP methods (comma-separated)
      --exclude-deprecated                 Hide deprecated endpoints
      --required-only                      Only show required parameters
      --detail <summary|basic|standard|full> Control amount of information [default: summary]
      --include-schemas                    Include request/response schemas
      --inline-schemas                     Fully inline every $ref schema instead of linking to a shared "Schema Definitions" section
      --include-examples                   Include request/response examples
      --include-auth                       Show authentication requirements and server URLs
      --no-toc                             Skip table of contents
      --sort <alpha|path-length|none>      Sorting method [default: alpha]
      --max-tokens <N>                     Fit output to a token budget, stepping detail down as needed (the hygiene report is appended outside the budget)
      --no-report                          Skip the spec hygiene report appended after the documentation
      --stats                              Dry run: print a per-service table of visible endpoints and estimated tokens instead of Markdown (TOTAL is one whole-document render, not the sum of the rows)
  -h, --help                               Print help
```

## Preparing API context for LLMs

Large API specs are a poor fit for LLM context windows: a 3 MB swagger file is hundreds of thousands of tokens of JSON, most of it boilerplate. Vimanam's detail levels and filters act as a token-budget dial, letting you hand an LLM (or a coding agent) exactly the slice of the API it needs, as compact Markdown.

```bash
# 20,000-ft view: every service and operation name, usually <1% the size of the spec.
# Good as always-loaded context so the model knows what the API can do.
vimanam openapi.json --detail summary -o api-map.md

# Zoom into one service when the task touches it — parameters and responses
# included, everything else excluded
vimanam openapi.json --service-filter Findings --detail standard -o findings-api.md

# Slice by path or method instead
vimanam openapi.json --path-filter /v1/scans --detail standard -o scans-api.md
vimanam openapi.json --method-filter GET --detail basic -o read-api.md

# Or let Vimanam pick the detail level: ask for as much of a service as fits a
# token budget. It starts at --detail full and steps down until it fits,
# reporting any reduction on stderr.
vimanam openapi.json --service-filter Findings --detail full --max-tokens 8000 -o findings-api.md
```

`--max-tokens` uses a chars/4 token estimate — close enough to choose a detail level, but treat it as approximate rather than an exact cap. When the output is fed to a model, add `--no-report`: the spec hygiene report is useful to a human tidying the spec but is noise in an LLM prompt, and it is appended outside the token budget.

Before choosing which slice to generate, `--stats` sizes the candidates without writing any Markdown: it prints one row per service with its visible endpoint count and the estimated token size of rendering that service alone, at whatever `--detail`, grouping and filter flags you pass, plus a TOTAL row for the whole document. A service left with no visible endpoint (for example, one whose only operations are dropped by `--exclude-deprecated`) is omitted from the table.

```bash
$ vimanam openapi.json --stats --detail standard
SERVICE    ENDPOINTS   ~TOKENS
Findings          42      6231
Scans             18      2890
Projects          31      4410
TOTAL             91     13102
```

Read it as a menu: a service that fits your budget can be pulled with `--service-filter <name>` at that detail level; one that does not can be re-measured at a lower `--detail` or handed to `--max-tokens`. Like `--max-tokens`, `~TOKENS` is a chars/4 estimate of the documentation body (the hygiene report is excluded). Each row is a separate render of that service alone, while `TOTAL` is a single render of the whole filtered document, so the rows do not necessarily add up to it: an operation tagged with several services is counted in each of their rows but once in the total, and the preamble and shared schema definitions are counted once per row.

A workflow that works well with coding agents: generate the `--detail summary` map once and reference it from the project's agent instructions (e.g. `CLAUDE.md`); have the agent regenerate a `--service-filter ... --detail standard` slice on demand when a task involves specific endpoints.

Output is deterministic — the same spec and flags produce byte-identical Markdown — so generated context files diff cleanly in git and don't needlessly invalidate LLM prompt caches.

## Continuous integration

Generate your API docs in CI with the [vimanam GitHub Action](https://github.com/noemaforge/vimanam-action) — it downloads the matching prebuilt binary (no Rust toolchain on the runner), verifies its SHA256 checksum, and runs vimanam:

```yaml
- uses: noemaforge/vimanam-action@4599a14c84d9d7bce1ec34ed9f12f3036f06b518 # v1
  with:
    spec: openapi.json
    output: docs/api-map.md
    detail: summary

# Output is deterministic, so this fails CI when the committed docs drift from the spec:
- run: git diff --exit-code -- docs/api-map.md
```

Pin the action to a commit SHA, not a mutable tag — see the action's [Pinning](https://github.com/noemaforge/vimanam-action#pinning) notes. More patterns in [`examples/`](https://github.com/noemaforge/vimanam-action/tree/main/examples).

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

## Roadmap

Work is organized into [milestones](https://github.com/noemaforge/vimanam/milestones):

- **[v1.0.0](https://github.com/noemaforge/vimanam/milestone/1)** — first stable release: JSON + YAML input, plus code-quality polish, a security-audit CI gate, and broader test coverage before tagging.
- **[v1.1.0](https://github.com/noemaforge/vimanam/milestone/2)** — LLM/token ergonomics: token-budget tooling and shell completions.
- **[v1.2.0](https://github.com/noemaforge/vimanam/milestone/3)** — alternative output modes: multi-file cross-linked pages, an agent-navigable skill-tree mode, and spec diffing.
- **[Packaging](https://github.com/noemaforge/vimanam/milestone/4)** — distribution channels (Scoop, winget, Chocolatey, AUR, native `.deb`/`.rpm`), shipped independently of code releases.

See the [open issues](https://github.com/noemaforge/vimanam/issues) for the full backlog.

## License

Apache License 2.0
