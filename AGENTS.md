# AGENTS.md

## Purpose

This repository contains `magellan`, a Rust CLI for rendering structured technical walkthroughs into compact terminal, Markdown, and HTML output.

Magellan is a deterministic presentation engine. It does not inspect a repository autonomously and it does not call an LLM. An external agent provides structured content; Magellan validates and renders it.

## Current Architecture

- `src/main.rs`: CLI entrypoint and file/stdin IO
- `src/lib.rs`: public exports for the crate
- `src/model.rs`: payload types and validation rules
- `src/render.rs`: schema generation plus terminal/Markdown/HTML renderers
- `examples/*.json`: realistic fixture payloads for end-to-end usage and testing
- `tests/cli.rs`: integration coverage for the executable
- `PLAN.md`: product and architecture notes from initial discovery

## Working Rules

- Keep the product boundary intact: agent supplies meaning, Magellan supplies validation and presentation.
- Prefer deterministic rendering over smart inference.
- Optimize for short, paced walkthroughs rather than exhaustive prose.
- Explain behavior and flow, not file churn.
- Keep `--help` outputs instructional for agents: each command should teach the workflow, not only list flags.
- Remember that HTML now defaults to book view: one summary page, then one page per section, with an overview toggle in the same report.
- Book-mode diagrams are expandable, so they should stay technically dense enough to merit the larger modal view.
- Add tests whenever changing schema rules, CLI behavior, or renderers.

## Development Commands

Use these commands for the main feedback loop:

```bash
cargo fmt
cargo test
```

Useful manual checks:

```bash
cargo run -- schema
cargo run -- prompt --agent-type codex --source session --goal walkthrough
cargo run -- prompt --agent-type codex --source diff --goal followup --question "why did this flow change?"
cargo run -- prompt --agent-type claude --source branch --goal handoff --scope backend --scope tests --artifact /tmp/handoff.json
cargo run -- example --preset walkthrough
cargo run -- validate --input examples/session-walkthrough.json
cargo run -- render --input examples/branch-handoff-timeline.json --format markdown
cargo run -- render --input examples/followup-validation-question.json --format html --out /tmp/magellan-question.html
cargo run -- validate --input payload.json
cargo run -- render --input payload.json --format terminal
cargo run -- render --input payload.json --format markdown
cargo run -- render --input payload.json --format html --out /tmp/magellan.html
cargo run -- render --input payload.json --format html --open
```

Use `--input -` when piping JSON from another tool or agent.

## Testing Expectations

- Unit test validation logic in `src/model.rs`.
- Unit test format-specific rendering in `src/render.rs`.
- Use integration tests in `tests/cli.rs` for command behavior and file output.
- Use real-binary tests for checked-in example payloads under `examples/`.
- Keep fixture payloads compact and behavior-focused.

## Near-Term Direction

The current scaffold supports:

- JSON Schema generation
- agent-oriented prompt templates
- prompt templates that adapt to evidence source and artifact goal
- prompt templates that can target an explicit question
- prompt templates that can constrain scope to a subsystem or flow
- built-in starter payloads for agents
- realistic checked-in fixture payloads for walkthroughs, handoffs, and follow-ups
- payload validation
- deterministic rendering for `terminal`, `markdown`, and `html`
- self-contained HTML reports with inline diagrams
- optional browser opening for rendered HTML reports

Likely next steps:

- refine the payload schema
- improve HTML presentation
- add richer diagram handling while keeping the tool deterministic
- improve agent-facing `--help` and examples
