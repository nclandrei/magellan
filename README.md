# Magellan

[![Crates.io](https://img.shields.io/crates/v/magellan-cli.svg)](https://crates.io/crates/magellan-cli)
[![Changelog](https://img.shields.io/github/v/release/nclandrei/magellan?include_prereleases&label=changelog)](https://github.com/nclandrei/magellan/releases)
[![CI](https://github.com/nclandrei/magellan/actions/workflows/ci.yml/badge.svg)](https://github.com/nclandrei/magellan/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/nclandrei/magellan/blob/main/LICENSE)

Render structured technical walkthroughs into terminal, Markdown, or HTML.

Magellan is a deterministic presentation engine for AI-generated walkthroughs. An agent or engineer gathers evidence, writes a structured JSON payload, then uses Magellan to validate and render it. Magellan does not inspect a repository by itself, and it does not call an LLM.

Use it when you want a walkthrough that explains behavior, flow, architecture, timing, and verification instead of narrating file churn. HTML output is a self-contained sidebar-scroll layout with a sticky table of contents, inline SVG diagrams you can enlarge, and a light/dark theme toggle.

## Installation

Install with Homebrew:

```bash
brew install nclandrei/tap/magellan
```

Install from crates.io:

```bash
cargo install magellan-cli --locked
```

The crate name is `magellan-cli`. The installed command is `magellan`.

Compiled macOS and Linux tarballs are available on the [GitHub releases page](https://github.com/nclandrei/magellan/releases).

## Recommended usage

The intended pattern is simple: a human points an agent at Magellan, and Magellan teaches the agent how to produce the artifact.

Start by telling the agent to run `magellan --help`. That help text is the checked-in playbook. It explains the payload shape, the evidence model, the diagram choices, and the normal validate-then-render flow.

For example, a human can say:

```text
Run `magellan --help`, then use Magellan to create a walkthrough for this change.
Use the current diff as evidence.
Write the payload to /tmp/magellan.json, validate it, and render Markdown to /tmp/magellan.md.
Explain behavior and verification, not file churn.
```

If you want Magellan to generate the agent prompt for you, use `magellan prompt`:

```bash
magellan prompt --agent-type codex --source diff --goal walkthrough
magellan prompt --agent-type codex --source diff --goal followup --question "why did this flow change?"
magellan prompt --agent-type claude --source branch --goal handoff --scope backend --scope tests
```

## The mandatory final step: `magellan go`

After the agent writes the JSON payload, the expected final step is a single
command that validates, renders HTML, opens it in the browser, and writes a
markdown file alongside:

```bash
magellan go --input /tmp/magellan.json
```

Use `go` instead of stringing `validate` and `render` together yourself. The
rendered HTML and markdown files are the deliverable — a prose summary in chat
is not a substitute.

## Help

```text
magellan - Render structured technical walkthroughs into terminal, markdown, or HTML output.

Magellan is a deterministic presentation engine for AI-generated walkthroughs.
It does not inspect a repository by itself and it does not call an LLM.
An agent or engineer gathers evidence, writes a structured JSON payload, and
then uses Magellan to validate and render that payload.

Usage:
  magellan go --input <payload.json|-> [--out <path>] [--markdown-out <path>]
  magellan schema
  magellan prompt --agent-type <codex|claude> [--source <session|diff|branch|pr>] [--goal <walkthrough|followup|handoff>] [options]
  magellan example --preset <walkthrough|timeline|before-after|followup>
  magellan validate --input <payload.json|->
  magellan render --input <payload.json|-> --format <terminal|markdown|html> [--out <path>] [--open]
  magellan guide

Commands:
  go        Validate, render HTML, open it, and write markdown — all in one step.
  schema    Print the JSON Schema for Magellan's input payload.
  prompt    Print an agent-oriented prompt template for producing a Magellan walkthrough.
  example   Print a starter payload that agents can edit before rendering.
  validate  Validate a JSON payload without rendering it.
  render    Render a JSON payload into terminal, markdown, or HTML output.
  guide     Print this help text.

Global options:
  -h, --help     Print this help text.
  -V, --version  Print version.

Core rule:
  Explain behavior, flow, architecture, timeline, or verification.
  Do not narrate file churn.

Normal workflow:
  1. Decide what evidence you are using.
     - persisted session transcripts, tool actions, and timestamps
     - current diff or commit range
     - branch compared to trunk
     - pull request description, comments, and diff
  2. Learn the payload contract.
     - run: `magellan schema`
     - optional starter payload: `magellan example --preset walkthrough`
  3. Write a JSON payload with:
     - `title`
     - `summary`
     - `sections`
     - optional `verification`
  4. Validate and render. This step is mandatory, do not skip it.
     - run: `magellan go --input /tmp/magellan.json`

Common requests:
  Explain the last commit:
  - use the current diff or `git show HEAD` as the main evidence
  - `magellan prompt --agent-type codex --source diff --goal followup --question "what did the last commit implement?"`

  Explain what we did yesterday or on a specific day:
  - inspect persisted session transcripts for that day first; if they are unavailable, say that and label any diff or commit reconstruction as fallback evidence
  - `magellan prompt --agent-type codex --source session --goal walkthrough --topic "what we built yesterday"`
  - `magellan prompt --agent-type claude --source session --goal followup --question "what did we do on March 15, 2026?"`

  Explain what we did in the last session or a named session:
  - inspect persisted Codex or Claude session transcripts first and stay scoped to the relevant session
  - `magellan prompt --agent-type codex --source session --goal walkthrough --topic "what we built in the last session"`
  - `magellan prompt --agent-type claude --source session --goal followup --question "what did we implement in session X?"`

  Explain what changed in commit X or on branch Y:
  - use that commit diff or branch comparison as the main evidence
  - `magellan prompt --agent-type codex --source diff --goal followup --question "what changed in commit <sha>?"`
  - `magellan prompt --agent-type claude --source branch --goal walkthrough --topic "what changed on branch <name>"`

Session evidence:
  When the evidence source is a prior coding session:
  - inspect persisted session transcripts before using git history as a proxy
  - Codex usually stores them under `$CODEX_HOME/sessions/YYYY/MM/DD/*.jsonl` and commonly `~/.codex/sessions/...`
  - Claude Code usually stores per-project transcripts under `~/.claude/projects/<workspace-slug>/<session-id>.jsonl` and uses `sessions-index.json` to help locate them
  - stay scoped to the current workspace or clearly relevant session
  - if transcript persistence is unavailable, say that explicitly and label any diff or commit reconstruction as fallback evidence, not the session itself

Content rules:
  - Keep the summary to 1-2 short paragraphs.
  - Keep sections to 3-6 focused chunks.
  - Keep section text short.
  - In HTML, each section becomes a scrollable block with a sidebar table of contents.
  - Diagrams render inline with SVG and can be clicked to enlarge.
  - Use diagrams only when they make the technical explanation easier to follow.
  - Ground the content in real evidence from code, diffs, tests, and persisted session history.

Diagram picking:
  sequence         Request or actor-by-actor interaction flow
  flow             Branching logic or state movement
  component_graph  Steady-state relationships between modules or layers
  timeline         Ordered work, debugging steps, or event progression
  before_after     User-visible behavior change

Prompt workflow:
  Use `magellan prompt` when you want Magellan to teach the agent how to build
  the payload.

  Useful prompt knobs:
  - `--source`: where the evidence comes from
  - `--goal`: broad walkthrough, follow-up answer, or engineer handoff
  - `--question`: a narrow question to answer directly
  - `--scope`: keep the artifact inside a subsystem, route, or flow
  - `--focus`: emphasize behavior, architecture, timeline, verification, or decisions

  Examples:
  - `magellan prompt --agent-type codex`
  - `magellan prompt --agent-type codex --source session --goal walkthrough`
  - `magellan prompt --agent-type codex --source diff --goal followup --question "why did this flow change?"`
  - `magellan prompt --agent-type claude --source branch --goal handoff --scope backend --scope tests --focus verification --focus decisions`

Render targets:
  terminal  Best for in-chat or terminal output with ASCII diagrams
  markdown  Best for chat messages, docs, or PR comments with Mermaid blocks
  html      Best for a paced visual walkthrough

HTML behavior:
  HTML reports are self-contained with a sidebar scroll layout.

  Sidebar:
  - sticky table of contents with links to each section
  - dark/light theme toggle (preference stored in localStorage)
  - collapses to a hamburger menu on narrow screens

  Content:
  - continuous scroll with summary, sections, and verification
  - diagrams render inline with SVG and an ASCII fallback

Reference files:
  Use these checked-in payloads when you want realistic examples:
  - `examples/session-walkthrough.json`
  - `examples/branch-handoff-timeline.json`
  - `examples/followup-validation-question.json`

Fast paths:
  Learn the contract:
  - `magellan schema`

  Start from a built-in preset:
  - `magellan example --preset timeline`
  - `magellan example --preset followup`

  Study a realistic HTML report:
  - `magellan render --input examples/session-walkthrough.json --format html --open`

  Answer a focused follow-up:
  - `magellan prompt --agent-type codex --source diff --goal followup --question "why did this API flow change?"`

  Prepare a handoff:
  - `magellan prompt --agent-type claude --source branch --goal handoff --scope backend --scope tests`

  Validate and render (the mandatory final step):
  - `magellan go --input /tmp/magellan.json`
```

## Example

Start with a built-in payload, then use `magellan go` to validate, open the HTML
report in the browser, and write the markdown alongside in one step:

```bash
magellan example --preset walkthrough > /tmp/magellan.json
magellan go --input /tmp/magellan.json
```

If you need finer control, `render` is still available for one-off format
targets:

```bash
magellan render --input /tmp/magellan.json --format terminal
magellan render --input /tmp/magellan.json --format markdown > walkthrough.md
magellan render --input /tmp/magellan.json --format html --out /tmp/magellan.html --open
```

`--input -` reads JSON from stdin, so an agent can pipe a payload directly into
`go`, `validate`, or `render`.

## Payload shape

`magellan schema` is the source of truth, but the JSON looks like this at a high level:

```json
{
  "title": "Checkout validation moved before order submission",
  "summary": [
    "Invalid carts now fail locally before the order request is built."
  ],
  "sections": [
    {
      "title": "Request flow",
      "text": [
        "The checkout page validates the cart before it assembles the API payload."
      ],
      "diagram": {
        "type": "sequence",
        "nodes": ["User", "Checkout Page", "Validation Gate", "Orders API"],
        "edges": [
          { "from": "User", "to": "Checkout Page", "label": "submit" },
          { "from": "Checkout Page", "to": "Validation Gate", "label": "check cart" },
          { "from": "Validation Gate", "to": "Orders API", "label": "valid payload" }
        ]
      }
    }
  ],
  "verification": {
    "text": [
      "A regression test covers invalid submissions before any network request."
    ]
  }
}
```

Magellan currently supports `sequence`, `flow`, `component_graph`, `timeline`, and `before_after` diagrams.

## Included examples

The repository ships with compact example payloads you can validate and render directly:

- [`examples/session-walkthrough.json`](examples/session-walkthrough.json)
- [`examples/branch-handoff-timeline.json`](examples/branch-handoff-timeline.json)
- [`examples/followup-validation-question.json`](examples/followup-validation-question.json)

For example:

```bash
magellan validate --input examples/session-walkthrough.json
magellan render --input examples/session-walkthrough.json --format terminal
magellan render --input examples/session-walkthrough.json --format markdown > walkthrough.md
magellan render --input examples/session-walkthrough.json --format html --out /tmp/magellan.html --open
```

## Development

Build and verify with:

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo package --locked
```

Useful local commands:

```bash
cargo run -- schema
cargo run -- guide
cargo run -- prompt --agent-type codex --source session --goal walkthrough
cargo run -- prompt --agent-type codex --source diff --goal followup --question "why did this flow change?"
cargo run -- prompt --agent-type claude --source branch --goal handoff --scope backend --scope tests --artifact /tmp/handoff.json
cargo run -- example --preset walkthrough
cargo run -- example --preset followup
cargo run -- validate --input examples/session-walkthrough.json
cargo run -- render --input examples/branch-handoff-timeline.json --format markdown
cargo run -- render --input examples/followup-validation-question.json --format html --out /tmp/magellan-question.html
```

## Release

Pushing `main` triggers the release workflow. When the version in `Cargo.toml` has not already been published, the workflow tags the release, uploads macOS and Linux artifacts, publishes `magellan-cli` to crates.io, and updates the Homebrew formula in `nclandrei/homebrew-tap`.
