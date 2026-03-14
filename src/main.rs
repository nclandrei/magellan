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
  1. Choose the right source of evidence and goal for the artifact you want.
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
  magellan prompt --agent-type claude --source session --goal walkthrough
  magellan prompt --agent-type codex --source diff --goal followup --topic \"why did this flow change?\"
  magellan prompt --agent-type codex --source branch --goal handoff --focus verification --focus decisions

Use `--input -` to read JSON from stdin.";

const PROMPT_AFTER_HELP: &str = "\
Examples:
  magellan prompt --agent-type codex --source session --goal walkthrough --topic \"what we built in this session\"
  magellan prompt --agent-type claude --source diff --goal followup --topic \"why did this API flow change?\"
  magellan prompt --agent-type codex --source pr --goal handoff --artifact /tmp/handoff.json --focus verification --focus decisions

Goals:
  walkthrough  Create a broad narrated explainer of the change.
  followup     Answer a narrower question with a tighter artifact.
  handoff      Prepare another engineer to pick up the work quickly.

Sources:
  session      Use session messages, tool actions, and timestamps.
  diff         Use the active diff or commit range as the main evidence.
  branch       Compare the current branch to trunk.
  pr           Use pull request description, comments, and diff.";

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
    #[command(after_help = PROMPT_AFTER_HELP)]
    Prompt {
        /// Which coding agent the prompt should be tailored for.
        #[arg(long)]
        agent_type: CliAgentType,
        /// What source of evidence the agent should inspect first.
        #[arg(long, default_value = "session")]
        source: CliPromptSource,
        /// What kind of explainer artifact the agent should produce.
        #[arg(long, default_value = "walkthrough")]
        goal: CliPromptGoal,
        /// What the walkthrough should explain.
        #[arg(long, default_value = "what we built in this task")]
        topic: String,
        /// Where the agent should write the payload JSON before rendering it.
        #[arg(long, default_value = "/tmp/magellan.json")]
        artifact: PathBuf,
        /// Render target the agent should aim for in the final step.
        #[arg(long, default_value = "html")]
        render_format: CliOutputFormat,
        /// Areas to emphasize in the walkthrough. Repeat to provide multiple focuses.
        #[arg(long)]
        focus: Vec<CliPromptFocus>,
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

#[derive(Clone, Copy, Debug, ValueEnum)]
enum CliPromptFocus {
    Behavior,
    Architecture,
    Timeline,
    Verification,
    Decisions,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum CliPromptSource {
    Session,
    Diff,
    Branch,
    Pr,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum CliPromptGoal {
    Walkthrough,
    Followup,
    Handoff,
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
        Command::Prompt {
            agent_type,
            source,
            goal,
            topic,
            artifact,
            render_format,
            focus,
        } => {
            let options = PromptOptions {
                agent_type,
                source,
                goal,
                topic: topic.as_str(),
                artifact: artifact.as_path(),
                render_format: render_format.into(),
                focus: &focus,
            };
            println!("{}", prompt_text(options));
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

struct PromptOptions<'a> {
    agent_type: CliAgentType,
    source: CliPromptSource,
    goal: CliPromptGoal,
    topic: &'a str,
    artifact: &'a Path,
    render_format: OutputFormat,
    focus: &'a [CliPromptFocus],
}

fn prompt_text(options: PromptOptions<'_>) -> String {
    let agent_name = match options.agent_type {
        CliAgentType::Codex => "Codex",
        CliAgentType::Claude => "Claude Code",
    };
    let render_command = format_render_command(options.artifact, options.render_format);
    let focus_guidance = prompt_focus_guidance(options.focus);
    let source_guidance = prompt_source_guidance(options.source);
    let goal_guidance = prompt_goal_guidance(options.goal);
    let section_guidance = prompt_goal_section_guidance(options.goal);

    format!(
        "You are {agent_name}. Use Magellan to produce a compact walkthrough focused on this topic: {topic}

Workflow:
1. Gather evidence using this source of truth:
{source_guidance}
2. Run `magellan schema`.
3. Optionally run `magellan example --preset walkthrough` for a starter payload.
4. Create JSON at `{artifact}` with:
   - `title`
   - `summary` with 1-2 short paragraphs
   - `sections` shaped for this goal: {section_guidance}
   - short `text` arrays instead of long prose
   - optional `diagram` objects when they clarify the story
   - optional `verification`
5. Run `magellan validate --input {artifact}`.
6. Run `{render_command}`.

Content rules:
- Explain behavior, flow, or decisions.
- Do not narrate file churn.
- Do not invent details that are not grounded in evidence.
- Keep the walkthrough paced and scannable.
- Prefer diagrams only when they make the story easier to follow.

Diagram guide:
- `sequence` for request and interaction flow
- `flow` for branching logic or state movement
- `component_graph` for relationships between pieces of the system
- `timeline` when the order of work or events matters
- `before_after` when the user-facing change is the key story

Goal for this walkthrough:
{goal_guidance}

Focus for this walkthrough:
{focus_guidance}

Good final move:
`{render_command}`",
        topic = options.topic,
        artifact = options.artifact.display(),
        render_command = render_command,
        source_guidance = source_guidance,
        section_guidance = section_guidance,
        goal_guidance = goal_guidance,
        focus_guidance = focus_guidance
    )
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

fn format_render_command(path: &Path, render_format: OutputFormat) -> String {
    match render_format {
        OutputFormat::Html => {
            format!(
                "magellan render --input {} --format html --open",
                path.display()
            )
        }
        OutputFormat::Markdown => {
            format!(
                "magellan render --input {} --format markdown",
                path.display()
            )
        }
        OutputFormat::Terminal => {
            format!(
                "magellan render --input {} --format terminal",
                path.display()
            )
        }
    }
}

fn prompt_focus_guidance(focuses: &[CliPromptFocus]) -> String {
    if focuses.is_empty() {
        return String::from("- no explicit focus was requested; choose the clearest story arc");
    }

    focuses
        .iter()
        .map(|focus| match focus {
            CliPromptFocus::Behavior => {
                "- prioritize what the system now does differently for the user or caller"
            }
            CliPromptFocus::Architecture => {
                "- emphasize which parts of the system collaborate and why"
            }
            CliPromptFocus::Timeline => "- emphasize order: what happens first, next, and last",
            CliPromptFocus::Verification => {
                "- give verification its own section and be explicit about evidence"
            }
            CliPromptFocus::Decisions => {
                "- call out important implementation or product decisions, not just outcomes"
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn prompt_source_guidance(source: CliPromptSource) -> &'static str {
    match source {
        CliPromptSource::Session => {
            "- inspect session messages, tool actions, and timestamps to reconstruct intent and sequence"
        }
        CliPromptSource::Diff => {
            "- inspect the current diff or commit range and use it as the main evidence for what changed"
        }
        CliPromptSource::Branch => {
            "- compare the current branch to trunk and use that delta as the main evidence"
        }
        CliPromptSource::Pr => {
            "- inspect the pull request description, review comments, and diff before writing the walkthrough"
        }
    }
}

fn prompt_goal_guidance(goal: CliPromptGoal) -> &'static str {
    match goal {
        CliPromptGoal::Walkthrough => {
            "- produce a broad narrated explainer that tells the full story of the change"
        }
        CliPromptGoal::Followup => {
            "- answer a narrower follow-up question and stay tighter than a full walkthrough"
        }
        CliPromptGoal::Handoff => {
            "- optimize for another engineer picking up the work quickly, including decisions and verification"
        }
    }
}

fn prompt_goal_section_guidance(goal: CliPromptGoal) -> &'static str {
    match goal {
        CliPromptGoal::Walkthrough => "3-6 focused steps that cover the main story arc",
        CliPromptGoal::Followup => "2-4 focused steps centered on the specific question",
        CliPromptGoal::Handoff => {
            "3-6 focused steps, with explicit attention to decisions, risks, and verification"
        }
    }
}
