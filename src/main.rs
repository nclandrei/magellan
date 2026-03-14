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

Agent workflow:
  1. Inspect the code, diff, tests, and session yourself.
  2. Run `magellan schema` and optionally `magellan example --preset walkthrough`.
  3. Create a JSON payload with short summaries, short sections, and optional diagrams.
  4. Validate it with `magellan validate --input WALKTHROUGH.json`.
  5. Render it with `magellan render --input WALKTHROUGH.json --format html --open`.

Rules:
  - explain behavior, not file churn
  - keep the summary to 1-2 short paragraphs
  - keep sections to 3-6 focused chunks
  - keep paragraph text short
  - use evidence from code, diffs, tests, and session history

Agent-specific prompt templates:
  magellan prompt --agent-type codex
  magellan prompt --agent-type claude

Use `--input -` to read JSON from stdin.";

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
    /// Print an agent-oriented prompt template for producing a Magellan walkthrough.
    Prompt {
        /// Which coding agent the prompt should be tailored for.
        #[arg(long)]
        agent_type: CliAgentType,
    },
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

#[derive(Clone, Copy, Debug, ValueEnum)]
enum CliAgentType {
    Codex,
    Claude,
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
        Command::Prompt { agent_type } => {
            println!("{}", prompt_text(agent_type));
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

fn prompt_text(agent_type: CliAgentType) -> &'static str {
    match agent_type {
        CliAgentType::Codex => CODEX_PROMPT,
        CliAgentType::Claude => CLAUDE_PROMPT,
    }
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

const CODEX_PROMPT: &str = "\
You are Codex. Use Magellan to produce a compact walkthrough of the work you already inspected.

Workflow:
1. Inspect the relevant code, diff, tests, and session context yourself.
2. Run `magellan schema`.
3. Optionally run `magellan example --preset walkthrough` for a starter payload.
4. Create a JSON payload with:
   - `title`
   - `summary` with 1-2 short paragraphs
   - `sections` with 3-6 focused steps
   - short `text` arrays instead of long prose
   - optional `diagram` objects when they clarify the story
   - optional `verification`
5. Run `magellan validate --input WALKTHROUGH.json`.
6. Run `magellan render --input WALKTHROUGH.json --format html --open`.

Content rules:
- Explain behavior, flow, or decisions.
- Do not narrate file churn.
- Do not invent details that are not grounded in evidence.
- Keep the walkthrough paced and scannable.

Good final move:
`magellan render --input /tmp/magellan.json --format html --open`
";

const CLAUDE_PROMPT: &str = "\
You are Claude Code. Use Magellan to turn your understanding of the task into a compact walkthrough.

Workflow:
1. Inspect the relevant code, diff, tests, and session context yourself.
2. Run `magellan schema`.
3. Optionally run `magellan example --preset walkthrough` for a starter payload.
4. Create a JSON payload with:
   - `title`
   - `summary` with 1-2 short paragraphs
   - `sections` with 3-6 focused steps
   - short `text` arrays instead of long prose
   - optional `diagram` objects when they clarify the story
   - optional `verification`
5. Run `magellan validate --input WALKTHROUGH.json`.
6. Run `magellan render --input WALKTHROUGH.json --format html --open`.

Content rules:
- Explain behavior, flow, or decisions.
- Do not narrate file churn.
- Do not invent details that are not grounded in evidence.
- Keep the walkthrough paced and scannable.

Good final move:
`magellan render --input /tmp/magellan.json --format html --open`
";
