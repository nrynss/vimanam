mod config;
mod markdown;
mod models;
mod parser;
mod report;
mod stats;
mod utils;

use std::fs::File;
use std::io::{BufWriter, Write, stdout};
use std::process;

use anyhow::{Context, Result, bail};
use clap::{CommandFactory, Parser};
use log::info;

use crate::config::{Cli, Commands, build_config};
use crate::markdown::generate_markdown;
use crate::models::{ApiDocumentation, DocConfig};
use crate::parser::parse_openapi;

/// Writes the documentation and, unless `--no-report` was given, the spec
/// hygiene report after it. Both the file and stdout paths go through here so
/// they can't drift apart.
fn write_output<W: Write>(
    writer: &mut W,
    api_doc: &ApiDocumentation,
    config: &DocConfig,
) -> Result<()> {
    if !config.include_report {
        return generate_markdown(writer, api_doc, config).context("Failed to generate markdown");
    }

    // The views end with differing trailing whitespace (one newline at
    // `--detail summary`, a blank line otherwise). Render the body into a
    // buffer and normalize it to exactly one trailing newline so the report's
    // rule is always preceded by exactly one blank line. Only this path
    // buffers: `--no-report` output stays byte-identical to the views' own.
    let mut body = Vec::new();
    generate_markdown(&mut body, api_doc, config).context("Failed to generate markdown")?;
    while body.last() == Some(&b'\n') {
        body.pop();
    }
    body.push(b'\n');
    writer
        .write_all(&body)
        .context("Failed to write documentation")?;

    let hygiene = report::analyze(api_doc, config);
    report::write_report(writer, &hygiene).context("Failed to write spec hygiene report")?;

    Ok(())
}

/// Parses CLI arguments, parses the spec, and writes markdown to the
/// requested output (file or stdout).
fn run() -> Result<()> {
    // Initialize logger
    env_logger::init();

    // Parse command-line arguments
    let cli = Cli::parse();

    // Subcommands short-circuit the conversion pipeline. The match is
    // exhaustive so a new `Commands` variant fails to compile here instead of
    // falling through to the "missing input file" error below.
    match cli.command {
        Some(Commands::Completions { shell }) => {
            clap_complete::generate(shell, &mut Cli::command(), "vimanam", &mut stdout());
            return Ok(());
        }
        None => {}
    }

    // clap marks `input` required (negated only by a subcommand), so it is
    // always set once no subcommand was given; the bail is a defensive guard.
    let Some(input) = &cli.input else {
        bail!("missing input file; run `vimanam --help` for usage");
    };

    // Build configuration
    let config = build_config(&cli);

    // Parse OpenAPI spec
    let api_doc =
        parse_openapi(input).with_context(|| format!("Failed to parse OpenAPI file: {input:?}"))?;

    // `--stats` is a dry run: print the per-service size table to stdout
    // instead of the documentation. clap rejects `-o` and `--max-tokens`
    // alongside it, and the hygiene report is never emitted in this mode.
    if cli.stats {
        let stats = stats::compute(&api_doc, &config).context("Failed to compute stats")?;
        stats::write_stats(&mut stdout(), &stats).context("Failed to write stats")?;
        return Ok(());
    }

    // Generate markdown
    if let Some(output_path) = &cli.output {
        // Write to file
        let file = File::create(output_path)
            .with_context(|| format!("Failed to create output file: {:?}", output_path))?;
        let mut writer = BufWriter::new(file);

        write_output(&mut writer, &api_doc, &config)?;

        info!("Documentation written to: {:?}", output_path);
    } else {
        // Write to stdout
        let mut writer = stdout();

        write_output(&mut writer, &api_doc, &config)?;
    }

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {:#}", err);
        process::exit(1);
    }
}
