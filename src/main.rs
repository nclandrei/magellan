use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use std::time::{SystemTime, UNIX_EPOCH};

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
        /// Open the rendered HTML report after writing it.
        #[arg(long)]
        open: bool,
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
        Command::Render {
            input,
            format,
            out,
            open,
        } => {
            let document = read_document(&input)?;
            document.validate()?;
            let output_format: OutputFormat = format.into();
            let rendered = render_document(&document, output_format);
            let destination = resolve_render_destination(&document, output_format, out, open)?;

            match destination.as_deref() {
                Some(path) => {
                    write_output(Some(path), &rendered)?;
                    if open {
                        open_path(path)?;
                        println!("Opened {}", path.display());
                    }
                }
                None => write_output(None, &rendered)?,
            }
        }
    }

    Ok(())
}

fn resolve_render_destination(
    document: &Document,
    format: OutputFormat,
    out: Option<PathBuf>,
    open: bool,
) -> Result<Option<PathBuf>> {
    if !open {
        return Ok(out);
    }

    if format != OutputFormat::Html {
        bail!("--open currently requires --format html");
    }

    Ok(Some(out.unwrap_or_else(|| {
        default_html_output_path(&document.title)
    })))
}

fn default_html_output_path(title: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let slug = slugify(title);

    std::env::temp_dir().join(format!("magellan-{slug}-{timestamp}.html"))
}

fn slugify(title: &str) -> String {
    let slug: String = title
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();

    let slug = slug
        .split('-')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    if slug.is_empty() {
        String::from("walkthrough")
    } else {
        slug
    }
}

fn open_path(path: &Path) -> Result<()> {
    let override_bin = std::env::var_os("MAGELLAN_OPEN_BIN");
    let status = match override_bin {
        Some(bin) => ProcessCommand::new(bin).arg(path).status(),
        None if cfg!(target_os = "macos") => ProcessCommand::new("open").arg(path).status(),
        None if cfg!(target_os = "windows") => ProcessCommand::new("cmd")
            .args(["/C", "start", ""])
            .arg(path)
            .status(),
        None => ProcessCommand::new("xdg-open").arg(path).status(),
    }
    .with_context(|| format!("failed to launch opener for {}", path.display()))?;

    if !status.success() {
        bail!("opener exited with status {status}");
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
