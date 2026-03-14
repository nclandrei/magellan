use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};

use magellan::{
    Document, ExamplePreset, OutputFormat, example_document, render_document, schema_json,
};

const AFTER_HELP: &str = "\
Magellan is a deterministic presentation engine for AI-generated walkthroughs.

Give it structured content, not long prose blobs:
  - title
  - 1-2 short summary paragraphs
  - 3-6 sections with short paragraphs
  - optional diagrams when they improve comprehension

Use `magellan schema` to inspect the expected JSON payload.
Use `magellan example --preset walkthrough` when you want a starter payload.
Pass `--input -` to read JSON from stdin.";

#[derive(Parser, Debug)]
#[command(
    name = "magellan",
    version,
    about = "Render structured technical walkthroughs into compact terminal, markdown, or HTML output.",
    after_help = AFTER_HELP
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Print the JSON Schema for Magellan's input payload.
    Schema,
    /// Print a starter payload that agents can edit before rendering.
    Example {
        /// Which starter payload to print.
        #[arg(long, default_value = "walkthrough")]
        preset: CliExamplePreset,
    },
    /// Validate a JSON payload without rendering it.
    Validate {
        /// JSON file to load, or '-' to read from stdin.
        #[arg(long)]
        input: PathBuf,
    },
    /// Render a JSON payload into terminal, markdown, or HTML output.
    Render {
        /// JSON file to load, or '-' to read from stdin.
        #[arg(long)]
        input: PathBuf,
        /// Output format to render.
        #[arg(long, default_value = "terminal")]
        format: CliOutputFormat,
        /// Optional output path. When omitted, rendered content is written to stdout.
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum CliOutputFormat {
    Terminal,
    Markdown,
    Html,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum CliExamplePreset {
    Walkthrough,
    Timeline,
    BeforeAfter,
}

impl From<CliOutputFormat> for OutputFormat {
    fn from(value: CliOutputFormat) -> Self {
        match value {
            CliOutputFormat::Terminal => OutputFormat::Terminal,
            CliOutputFormat::Markdown => OutputFormat::Markdown,
            CliOutputFormat::Html => OutputFormat::Html,
        }
    }
}

impl From<CliExamplePreset> for ExamplePreset {
    fn from(value: CliExamplePreset) -> Self {
        match value {
            CliExamplePreset::Walkthrough => ExamplePreset::Walkthrough,
            CliExamplePreset::Timeline => ExamplePreset::Timeline,
            CliExamplePreset::BeforeAfter => ExamplePreset::BeforeAfter,
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Schema => {
            println!("{}", schema_json()?);
        }
        Command::Example { preset } => {
            let document = example_document(preset.into());
            println!("{}", serde_json::to_string_pretty(&document)?);
        }
        Command::Validate { input } => {
            let document = read_document(&input)?;
            document.validate()?;
            println!("Payload is valid.");
        }
        Command::Render { input, format, out } => {
            let document = read_document(&input)?;
            document.validate()?;
            let rendered = render_document(&document, format.into());
            write_output(out.as_deref(), &rendered)?;
        }
    }

    Ok(())
}

fn read_document(path: &Path) -> Result<Document> {
    let raw = if path == Path::new("-") {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("failed to read payload from stdin")?;
        buffer
    } else {
        fs::read_to_string(path)
            .with_context(|| format!("failed to read payload from {}", path.display()))?
    };

    serde_json::from_str(&raw).context("failed to parse JSON payload")
}

fn write_output(path: Option<&Path>, rendered: &str) -> Result<()> {
    match path {
        Some(path) => {
            if rendered.is_empty() {
                bail!("refusing to write empty output to {}", path.display());
            }
            fs::write(path, rendered).with_context(|| {
                format!("failed to write rendered output to {}", path.display())
            })?;
        }
        None => {
            print!("{rendered}");
        }
    }

    Ok(())
}
