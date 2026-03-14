# Magellan

Magellan is a deterministic presentation engine for AI-generated technical walkthroughs.

It does not call an LLM. Instead, an agent or engineer prepares a structured payload with a short summary, paced sections, and optional diagram data. Magellan validates that payload and renders it into terminal, Markdown, or HTML output.

## Install

```bash
# Homebrew (recommended)
brew install nclandrei/tap/magellan

# crates.io
cargo install magellan-cli --locked
```

The installed command is still `magellan`.

## Current CLI

```text
magellan schema
magellan guide
magellan prompt --agent-type codex --source session --goal walkthrough
magellan example --preset walkthrough
magellan validate --input payload.json
magellan render --input payload.json --format terminal
magellan render --input payload.json --format markdown
magellan render --input payload.json --format html --out /tmp/magellan.html
magellan render --input payload.json --format html --open
```

Use `--input -` to read a JSON payload from stdin.
Use `magellan --help` when you want the long-form checked-in agent playbook, Showboat-style.
Use `magellan guide` if you want that same playbook via an explicit command.
Use `magellan example --preset walkthrough` when you want a starter payload to edit.
Use `magellan prompt --agent-type codex` or `magellan prompt --agent-type claude` when you want Magellan to teach an agent the workflow directly.
Use `magellan <command> --help` when you want workflow-oriented guidance for that exact step, not just flags.
Use `--source` and `--goal` on `magellan prompt` when you want the template to match where the evidence comes from and what artifact the agent should produce.
Use `--question` when the walkthrough should answer a specific question directly instead of only expanding a topic.
Use `--scope` when the walkthrough should stay inside a specific subsystem, layer, route, or flow.
Use the checked-in JSON fixtures under `examples/` when you want realistic end-to-end payloads instead of starter content.

## Why This Exists

Normal agent explanations often fail in one of two ways:

- they dump a wall of text that is hard to follow
- they focus on file churn instead of behavior

Magellan pushes the output into a better shape:

- 1-2 short summary paragraphs
- 3-6 sections
- short section text instead of essays
- diagrams when they help explain flow, architecture, timing, or before/after changes
- HTML reports default to a book-style view so engineers see one technical slice at a time instead of one long wall of text

## Payload Shape

The source of truth is `magellan schema`, but the payload looks like this at a high level:

```json
{
  "title": "Order validation moved earlier",
  "summary": [
    "The UI now validates required fields before sending the network request."
  ],
  "sections": [
    {
      "title": "Request flow",
      "text": [
        "The form blocks invalid submissions locally.",
        "Valid submissions still reach the API."
      ],
      "diagram": {
        "type": "sequence",
        "nodes": ["User", "Form", "API"],
        "edges": [
          { "from": "User", "to": "Form", "label": "submit" },
          { "from": "Form", "to": "API", "label": "valid request" }
        ]
      }
    }
  ],
  "verification": {
    "text": [
      "Automated tests covered the regression."
    ]
  }
}
```

## Render Targets

- `terminal`: compact text with ASCII diagrams
- `markdown`: sectioned Markdown with Mermaid blocks
- `html`: a styled local report with self-contained inline diagrams, a page-by-page book view, an overview toggle, and clickable enlarged diagrams in book mode

When you pass `--open` with `--format html`, Magellan writes the report and opens it in the default browser. If `--out` is omitted, Magellan creates a temp file automatically.
In HTML, the summary becomes the opening page and each section becomes its own page, so the agent should keep one idea per section.
In book view, clicking a diagram opens a larger modal so the technical detail is readable without leaving the current page.

## Example Reports

Magellan now ships with realistic fixture payloads that show the intended output shape:

- `examples/session-walkthrough.json`: a multi-section session explainer with sequence, flow, and component diagrams
- `examples/branch-handoff-timeline.json`: a branch handoff artifact with a timeline and steady-state flow
- `examples/followup-validation-question.json`: a narrow follow-up explainer with flow and before/after diagrams

End-to-end example:

```bash
cargo run -- validate --input examples/session-walkthrough.json
cargo run -- render --input examples/session-walkthrough.json --format terminal
cargo run -- render --input examples/session-walkthrough.json --format markdown
cargo run -- render --input examples/session-walkthrough.json --format html --open
```

That loop is the current reference path for a real payload:

1. agent writes JSON
2. `magellan validate` checks pacing and diagram structure
3. `magellan render` turns it into terminal, Markdown, or HTML output

## Development

Build and verify with:

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Run locally with:

```bash
cargo run -- schema
cargo run -- guide
cargo run -- prompt --agent-type codex --source session --goal walkthrough
cargo run -- prompt --agent-type codex --source diff --goal followup --question "why did this change?"
cargo run -- prompt --agent-type claude --source branch --goal handoff --scope backend --scope tests
cargo run -- example --preset walkthrough
cargo run -- validate --input examples/branch-handoff-timeline.json
cargo run -- render --input examples/followup-validation-question.json --format markdown
cargo run -- validate --input payload.json
cargo run -- render --input payload.json --format html --out /tmp/magellan.html
cargo run -- render --input payload.json --format html --open
```

## Design Boundary

Magellan is intentionally narrow.

- The agent decides what changed and how to explain it.
- Magellan validates the structure and renders it predictably.
- The output should explain behavior, not just changed files.

That boundary keeps the tool fast, deterministic, and testable.

## Release Automation

Pushing to `main` triggers the release workflow. When the version in [Cargo.toml](/Users/anicolae/code/magellan/Cargo.toml) has not been released yet, the workflow will:

- build native release artifacts for macOS (Intel + Apple Silicon) and Linux
- publish the crate to crates.io as `magellan-cli`
- update the Homebrew formula in `nclandrei/homebrew-tap`
- publish the GitHub release

If a non-draft GitHub release for the current version already exists, automatic `push` runs exit without rebuilding artifacts. For an intentional retry of the current version, use GitHub Actions `workflow_dispatch` on the `Release` workflow.

Required GitHub repository secrets:

- `CARGO_REGISTRY_TOKEN`: crates.io publish token for `magellan-cli`
- `HOMEBREW_TAP_TOKEN`: GitHub token with push access to `nclandrei/homebrew-tap`

Release process:

1. Update `version` in [Cargo.toml](/Users/anicolae/code/magellan/Cargo.toml).
2. Land the change on `main`.
3. Let the release workflow tag and publish that version automatically.
