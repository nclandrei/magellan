# AGENTS.md

## Purpose

This repository contains `magellan`, a Rust CLI for rendering structured technical walkthroughs into compact terminal, Markdown, and HTML output.

Magellan is a deterministic presentation engine. It does not inspect a repository autonomously and it does not call an LLM. An external agent provides structured content; Magellan validates and renders it.

## Current Architecture

- `src/main.rs`: CLI entrypoint and file/stdin IO
- `src/lib.rs`: public exports for the crate
- `src/model.rs`: payload types and validation rules
- `src/render.rs`: schema generation plus terminal/Markdown/HTML renderers
- `tests/cli.rs`: integration coverage for the executable
- `PLAN.md`: product and architecture notes from initial discovery

## Working Rules

- Keep the product boundary intact: agent supplies meaning, Magellan supplies validation and presentation.
- Prefer deterministic rendering over smart inference.
- Optimize for short, paced walkthroughs rather than exhaustive prose.
- Explain behavior and flow, not file churn.
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
cargo run -- validate --input payload.json
cargo run -- render --input payload.json --format terminal
cargo run -- render --input payload.json --format markdown
cargo run -- render --input payload.json --format html --out /tmp/magellan.html
```

Use `--input -` when piping JSON from another tool or agent.

## Testing Expectations

- Unit test validation logic in `src/model.rs`.
- Unit test format-specific rendering in `src/render.rs`.
- Use integration tests in `tests/cli.rs` for command behavior and file output.
- Keep fixture payloads compact and behavior-focused.

## Near-Term Direction

The current scaffold supports:

- JSON Schema generation
- payload validation
- deterministic rendering for `terminal`, `markdown`, and `html`

Likely next steps:

- refine the payload schema
- improve HTML presentation
- add richer diagram handling while keeping the tool deterministic
- improve agent-facing `--help` and examples
