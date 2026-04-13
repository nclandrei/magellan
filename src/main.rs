mod examples;
mod model;
mod render;

use std::ffi::OsStr;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};

use examples::{ExamplePreset, example_document};
use model::Document;
use render::{OutputFormat, render_document, schema_json};

const DEFAULT_TOPIC: &str = "what we built in this task";
const AGENT_GUIDE: &str = include_str!("../help.txt");

const AFTER_HELP: &str = "\
Top-level `magellan --help` prints the full checked-in agent playbook from `help.txt`, including common request recipes for commit, branch, and session explainers.

Use `magellan <command> --help` when you need command-specific workflow guidance.
Use `magellan guide` if you want the same top-level playbook through an explicit command.";

const PROMPT_AFTER_HELP: &str = "\
Examples:
  magellan prompt --agent-type codex --source session --goal walkthrough --topic \"what we built in this session\"
  magellan prompt --agent-type claude --source diff --goal followup --question \"why did this API flow change?\"
  magellan prompt --agent-type codex --source pr --goal handoff --scope backend --scope tests --artifact /tmp/handoff.json --focus verification --focus decisions

Goals:
  walkthrough  Create a broad technical walkthrough of the change.
  followup     Answer a narrower question with a tighter artifact.
  handoff      Prepare another engineer to pick up the work quickly.

Sources:
  session      Use persisted session transcripts, tool actions, and timestamps first.
  diff         Use the active diff or commit range as the main evidence.
  branch       Compare the current branch to trunk.
  pr           Use pull request description, comments, and diff.

Session-source reminders:
  - Check Codex transcripts under `$CODEX_HOME/sessions/YYYY/MM/DD/*.jsonl` (usually `~/.codex/sessions/...`).
  - Check Claude Code transcripts under `~/.claude/projects/<workspace-slug>/<session-id>.jsonl` and use `sessions-index.json` to find the right one.
  - Stay scoped to the current workspace.
  - If transcript evidence is unavailable, say that explicitly and label any diff or commit reconstruction as fallback evidence, not the session itself.

Diagram picking:
  sequence         Request or actor-by-actor interaction flow
  flow             Branching logic or state movement
  component_graph  Steady-state relationships between modules or layers
  timeline         Ordered work, debugging steps, or event progression
  before_after     User-visible behavior change
  layer_stack      Vertical stack of architectural layers or boundaries
  state_machine    States connected by labeled transitions
  table            Rows and columns for comparisons or field mappings
  dependency_tree  Hierarchical parent-child relationships

Reference outputs:
  examples/session-walkthrough.json
  examples/branch-handoff-timeline.json
  examples/followup-validation-question.json

Need the full Magellan playbook:
  magellan --help
  magellan guide";

const SCHEMA_AFTER_HELP: &str = "\
Use this when an agent needs the exact payload contract before writing JSON.

Typical flow:
  magellan schema > /tmp/magellan-schema.json
  magellan example --preset walkthrough > /tmp/magellan.json
  magellan validate --input /tmp/magellan.json

Payload shape reminders:
  - `title`
  - `summary` with 1-2 short paragraphs
  - `sections` with 3-6 focused chunks
  - optional `verification`";

const EXAMPLE_AFTER_HELP: &str = "\
Starter presets:
  walkthrough   Broad technical walkthrough with request-flow emphasis
  timeline      Ordered sequence when implementation order matters
  before_after  Behavior comparison when the change is best shown side by side
  followup      Narrower follow-up explainer that answers one focused implementation question

Checked-in realistic references:
  examples/session-walkthrough.json
  examples/branch-handoff-timeline.json
  examples/followup-validation-question.json";

const VALIDATE_AFTER_HELP: &str = "\
Validate before rendering.

Examples:
  magellan validate --input /tmp/magellan.json
  magellan validate --input examples/session-walkthrough.json
  cat payload.json | magellan validate --input -

Validation checks pacing and diagram structure. It does not verify factual truth.";

const GO_AFTER_HELP: &str = "\
Validate, render HTML, open it, and write markdown — all in one command.

This is the expected final step after creating a JSON payload.
Agents should always use `go` instead of separate validate + render calls.

Examples:
  magellan go --input /tmp/magellan.json
  magellan go --input /tmp/magellan.json --markdown-out /tmp/walkthrough.md --out /tmp/walkthrough.html
  cat payload.json | magellan go --input -

When --out is omitted, the HTML file is auto-named from the title and opened.
When --markdown-out is omitted, the markdown file is derived from --input with a .md extension.";

const RENDER_AFTER_HELP: &str = "\
Format guide:
  terminal  Fast in-chat or terminal explanation with ASCII diagrams
  markdown  Good for chat messages, docs, or PR comments with Mermaid blocks
  html      Best for paced visual walkthroughs; opens with a sidebar TOC, continuous scroll, and dark/light theme toggle

Diagram guide:
  sequence         Request or actor-by-actor interaction flow
  flow             Branching logic or state movement
  component_graph  Steady-state relationships between modules or layers
  timeline         Ordered work, debugging steps, or event progression
  before_after     User-visible behavior change
  layer_stack      Vertical stack of architectural layers or boundaries
  state_machine    States connected by labeled transitions
  table            Rows and columns for comparisons or field mappings
  dependency_tree  Hierarchical parent-child relationships

Examples:
  magellan render --input /tmp/magellan.json --format html --open --markdown-out /tmp/magellan.md
  magellan render --input examples/session-walkthrough.json --format terminal
  magellan render --input examples/branch-handoff-timeline.json --format markdown
  magellan render --input examples/followup-validation-question.json --format html --open
  cat payload.json | magellan render --input - --format html --open

Use --markdown-out to also write a markdown version alongside the primary render.
HTML reports use a sidebar scroll layout with a table of contents and dark/light theme toggle.

`--open` requires `--format html`.";

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
    /// Print the checked-in agent playbook that Magellan points to from --help.
    Guide,
    /// Print the JSON Schema for Magellan's input payload.
    #[command(after_help = SCHEMA_AFTER_HELP)]
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
        #[arg(long, default_value = DEFAULT_TOPIC)]
        topic: String,
        /// A specific question the walkthrough must answer directly.
        #[arg(long)]
        question: Option<String>,
        /// Limit the walkthrough to specific parts of the system or flow. Repeat to provide multiple scopes.
        #[arg(long)]
        scope: Vec<String>,
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
    #[command(after_help = EXAMPLE_AFTER_HELP)]
    Example {
        /// Which starter payload to print.
        #[arg(long, default_value = "walkthrough")]
        preset: CliExamplePreset,
    },
    /// Validate, render HTML, open it, and write markdown — all in one step.
    #[command(after_help = GO_AFTER_HELP)]
    Go {
        /// JSON file to load, or '-' to read from stdin.
        #[arg(long)]
        input: PathBuf,
        /// Where to write the HTML report. Auto-named from the title when omitted.
        #[arg(long)]
        out: Option<PathBuf>,
        /// Where to write the markdown version. Derived from --input with .md extension when omitted.
        #[arg(long)]
        markdown_out: Option<PathBuf>,
    },
    /// Validate a JSON payload without rendering it.
    #[command(after_help = VALIDATE_AFTER_HELP)]
    Validate {
        /// JSON file to load, or '-' to read from stdin.
        #[arg(long)]
        input: PathBuf,
    },
    /// Render a JSON payload into terminal, markdown, or HTML output.
    #[command(after_help = RENDER_AFTER_HELP)]
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
        /// Also write a markdown version to this path alongside the primary render.
        #[arg(long)]
        markdown_out: Option<PathBuf>,
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
    Followup,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum CliAgentType {
    Codex,
    Claude,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
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
            CliExamplePreset::Followup => ExamplePreset::Followup,
        }
    }
}

fn main() -> Result<()> {
    if should_print_top_level_help() {
        print!("{AGENT_GUIDE}");
        return Ok(());
    }

    let cli = Cli::parse();

    match cli.command {
        Command::Guide => {
            print!("{AGENT_GUIDE}");
        }
        Command::Schema => {
            println!("{}", schema_json()?);
        }
        Command::Prompt {
            agent_type,
            source,
            goal,
            topic,
            question,
            scope,
            artifact,
            render_format,
            focus,
        } => {
            let options = PromptOptions {
                agent_type,
                source,
                goal,
                topic: topic.as_str(),
                question: question.as_deref(),
                scope: &scope,
                artifact: artifact.as_path(),
                render_format,
                focus: &focus,
            };
            println!("{}", prompt_text(options));
        }
        Command::Example { preset } => {
            let document = example_document(preset.into());
            println!("{}", serde_json::to_string_pretty(&document)?);
        }
        Command::Go {
            input,
            out,
            markdown_out,
        } => {
            let document = read_document(&input)?;
            document.validate()?;

            let html_rendered = render_document(&document, OutputFormat::Html);
            let html_path = out.unwrap_or_else(|| default_html_output_path(&document.title));
            write_output(Some(&html_path), &html_rendered)?;
            open_path(&html_path)?;
            println!("Opened {}", html_path.display());

            let md_path = markdown_out.unwrap_or_else(|| {
                if input == Path::new("-") {
                    default_html_output_path(&document.title).with_extension("md")
                } else {
                    input.with_extension("md")
                }
            });
            let md_rendered = render_document(&document, OutputFormat::Markdown);
            write_output(Some(&md_path), &md_rendered)?;
            println!("Wrote {}", md_path.display());
        }
        Command::Validate { input } => {
            let document = read_document(&input)?;
            document.validate()?;
            println!("Payload is valid. Now render it:");
            println!("  magellan go --input {}", input.display());
        }
        Command::Render {
            input,
            format,
            out,
            open,
            markdown_out,
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

            if let Some(md_path) = markdown_out {
                let md_rendered = render_document(&document, OutputFormat::Markdown);
                write_output(Some(&md_path), &md_rendered)?;
                println!("Wrote {}", md_path.display());
            }
        }
    }

    Ok(())
}

fn should_print_top_level_help() -> bool {
    let mut args = std::env::args_os();
    let _binary = args.next();
    let first = args.next();
    let second = args.next();

    match (first.as_deref(), second.as_deref()) {
        (None, None) => true,
        (Some(flag), None) if flag == OsStr::new("--help") || flag == OsStr::new("-h") => true,
        (Some(command), None) if command == OsStr::new("help") => true,
        _ => false,
    }
}

struct PromptOptions<'a> {
    agent_type: CliAgentType,
    source: CliPromptSource,
    goal: CliPromptGoal,
    topic: &'a str,
    question: Option<&'a str>,
    scope: &'a [String],
    artifact: &'a Path,
    render_format: CliOutputFormat,
    focus: &'a [CliPromptFocus],
}

fn prompt_text(options: PromptOptions<'_>) -> String {
    let agent_name = match options.agent_type {
        CliAgentType::Codex => "Codex",
        CliAgentType::Claude => "Claude Code",
    };
    let effective_topic = match (options.topic, options.question) {
        (DEFAULT_TOPIC, Some(question)) => question,
        (topic, _) => topic,
    };
    let focus_guidance = prompt_focus_guidance(options.focus);
    let source_guidance = prompt_source_guidance(options.source);
    let goal_guidance = prompt_goal_guidance(options.goal);
    let section_guidance = prompt_goal_section_guidance(options.goal);
    let question_guidance = prompt_question_guidance(options.question);
    let scope_guidance = prompt_scope_guidance(options.scope);
    let diagram_guidance = prompt_diagram_guidance(options.goal, options.focus);

    let (render_step, render_target_line) =
        prompt_render_step(options.render_format, options.artifact);

    format!(
        "You are {agent_name}. Use Magellan to produce a compact walkthrough focused on this topic: {effective_topic}

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
   - optional `diagram` objects when they clarify the technical flow
   - optional `verification`
5. {render_step}
   Do not skip this step. The rendered artifacts are the deliverable, not a prose summary.

Content rules:
- Explain behavior, flow, or decisions.
- Do not narrate file churn.
- Do not invent details that are not grounded in evidence.
- Keep the walkthrough paced and scannable.
- Each section becomes a scrollable block in the HTML sidebar layout, so keep one idea per section.
- Prefer diagrams only when they make the technical explanation easier to follow.
- Do not describe the walkthrough in prose and then ask if the user wants a report.
  The rendered artifacts are always the expected output.

Diagram selection:
{diagram_guidance}

Goal for this walkthrough:
{goal_guidance}

Specific question to answer:
{question_guidance}

Scope for this walkthrough:
{scope_guidance}

Focus for this walkthrough:
{focus_guidance}

Required final step:
`{render_target_line}`",
        effective_topic = effective_topic,
        artifact = options.artifact.display(),
        render_step = render_step,
        render_target_line = render_target_line,
        source_guidance = source_guidance,
        section_guidance = section_guidance,
        goal_guidance = goal_guidance,
        question_guidance = question_guidance,
        scope_guidance = scope_guidance,
        focus_guidance = focus_guidance,
        diagram_guidance = diagram_guidance
    )
}

fn prompt_render_step(format: CliOutputFormat, artifact: &Path) -> (String, String) {
    let artifact_display = artifact.display().to_string();
    match format {
        CliOutputFormat::Html => {
            let command = format!("magellan go --input {artifact_display}");
            let step = format!(
                "Run `{command}`.\n   This validates, renders HTML (opens it in the browser), and writes markdown."
            );
            (step, command)
        }
        CliOutputFormat::Markdown => {
            let markdown_path = artifact_with_extension(artifact, "md");
            let command = format!(
                "magellan render --input {artifact_display} --format markdown --out {markdown_path}"
            );
            let step = format!(
                "Run `{command}`.\n   This validates the payload and writes a markdown file for sharing, docs, or PR comments."
            );
            (step, command)
        }
        CliOutputFormat::Terminal => {
            let command = format!("magellan render --input {artifact_display} --format terminal");
            let step = format!(
                "Run `{command}`.\n   This validates the payload and prints the terminal-friendly walkthrough."
            );
            (step, command)
        }
    }
}

fn artifact_with_extension(artifact: &Path, extension: &str) -> String {
    if artifact == Path::new("-") {
        return format!("-.{extension}");
    }
    artifact.with_extension(extension).display().to_string()
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

fn prompt_focus_guidance(focuses: &[CliPromptFocus]) -> String {
    if focuses.is_empty() {
        return String::from(
            "- no explicit focus was requested; choose the clearest technical framing",
        );
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
            "- inspect persisted session transcripts, tool actions, and timestamps before using git history as a proxy\n- for Codex, check `$CODEX_HOME/sessions/YYYY/MM/DD/*.jsonl` first; `~/.codex/sessions/...` is the usual default\n- for Claude Code, check `~/.claude/projects/<workspace-slug>/<session-id>.jsonl` and `sessions-index.json` to find the right project transcript\n- stay scoped to the current workspace or explicitly relevant session\n- if transcript persistence is unavailable, say that explicitly and label any diff or commit reconstruction as fallback evidence, not as the session itself"
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
            "- produce a broad technical walkthrough that covers the full change without drifting into fluff"
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
        CliPromptGoal::Walkthrough => "3-6 focused steps that cover the main technical flow",
        CliPromptGoal::Followup => "2-4 focused steps centered on the specific question",
        CliPromptGoal::Handoff => {
            "3-6 focused steps, with explicit attention to decisions, risks, and verification"
        }
    }
}

fn prompt_question_guidance(question: Option<&str>) -> String {
    match question {
        Some(question) => {
            format!("- make sure the walkthrough answers this explicitly near the top: {question}")
        }
        None => String::from(
            "- no explicit question was provided; infer the most useful framing from the topic and goal",
        ),
    }
}

fn prompt_scope_guidance(scopes: &[String]) -> String {
    if scopes.is_empty() {
        return String::from(
            "- no explicit scope was provided; use the full relevant surface implied by the source and goal",
        );
    }

    scopes
        .iter()
        .map(|scope| format!("- keep the walkthrough centered on this scope: {scope}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn prompt_diagram_guidance(goal: CliPromptGoal, focuses: &[CliPromptFocus]) -> String {
    let mut guidance = vec![
        String::from(
            "- use `sequence` when engineers need to follow actors or request flow step by step",
        ),
        String::from("- use `flow` for branching logic, validation gates, or state movement"),
        String::from(
            "- use `component_graph` for steady-state relationships between modules, layers, or services",
        ),
        String::from(
            "- use `timeline` when the order of work or events is part of the explanation",
        ),
        String::from(
            "- use `before_after` when the main point is how behavior changed for the user or caller",
        ),
        String::from(
            "- use `layer_stack` for vertical architectural layers, boundaries, or abstraction levels",
        ),
        String::from(
            "- use `state_machine` for lifecycle or status flows with named transitions between states",
        ),
        String::from(
            "- use `table` for comparisons, field mappings, permission matrices, or any structured rows-and-columns data",
        ),
        String::from(
            "- use `dependency_tree` for module dependencies, call trees, or hierarchical relationships",
        ),
        String::from(
            "- prefer at most one diagram per section and skip diagrams when a short paragraph is clearer",
        ),
    ];

    if goal == CliPromptGoal::Handoff || focuses.contains(&CliPromptFocus::Timeline) {
        guidance.push(String::from(
            "- for this artifact, include a `timeline` section when implementation order helps another engineer pick up the work",
        ));
    }

    if focuses.contains(&CliPromptFocus::Architecture) {
        guidance.push(String::from(
            "- architecture-focused explanations usually benefit from a `component_graph` section",
        ));
    }

    if focuses.contains(&CliPromptFocus::Behavior) {
        guidance.push(String::from(
            "- behavior-focused explanations usually benefit from `sequence`, `flow`, or `before_after`, depending on the technical change",
        ));
    }

    guidance.join("\n")
}
