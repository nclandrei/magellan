# AGENTS.md

## Purpose

This repository contains `magellan`, a Rust CLI for rendering structured technical walkthroughs into compact terminal, Markdown, and HTML output.

Magellan is a deterministic presentation engine. It does not inspect a repository autonomously and it does not call an LLM. An external agent provides structured content; Magellan validates and renders it.

## Current Architecture

- `src/main.rs`: CLI entrypoint and file/stdin IO
- `src/model.rs`: payload types and validation rules
- `src/render.rs`: schema generation plus terminal/Markdown/HTML renderers
- `.github/workflows/ci.yml`: CI checks for formatting, clippy, packaging, tests, and release config
- `.github/workflows/release.yml`: automated tagging, GitHub release publishing, crates.io publishing, and Homebrew tap updates
- `scripts/smoke-test-installed-magellan.sh`: release smoke test for the built binary or tarball
- `scripts/generate-homebrew-formula.sh`: generate the Homebrew formula from release artifact checksums
- `examples/*.json`: realistic fixture payloads for end-to-end usage and testing
- `tests/cli.rs`: integration coverage for the executable
- `PLAN.md`: product and architecture notes from initial discovery

## Working Rules

- Keep the product boundary intact: agent supplies meaning, Magellan supplies validation and presentation.
- Prefer deterministic rendering over smart inference.
- Optimize for short, paced walkthroughs rather than exhaustive prose.
- Explain behavior and flow, not file churn.
- Keep `--help` outputs instructional for agents: each command should teach the workflow, not only list flags.
- Keep `help.txt` aligned with the real CLI. Top-level `magellan --help` should print that checked-in file directly, Showboat-style. `magellan guide` is only an explicit alias.
- Remember that HTML renders a self-contained sidebar-scroll layout: a sticky table of contents on the left, continuous scroll on the right, and a light/dark theme toggle.
- Inline SVG diagrams open in a lightbox on click, so they should stay technically dense enough to merit the larger modal view.
- Keep package metadata and release workflows aligned. The crates.io package name is `magellan-cli`, but the installed binary and Homebrew formula stay `magellan`.
- Add tests whenever changing schema rules, CLI behavior, or renderers.

## Development Commands

Use these commands for the main feedback loop:

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo package --locked
```

Useful manual checks:

```bash
cargo run -- schema
cargo run -- guide
cargo run -- prompt --agent-type codex --source session --goal walkthrough
cargo run -- prompt --agent-type codex --source diff --goal followup --question "why did this flow change?"
cargo run -- prompt --agent-type claude --source branch --goal handoff --scope backend --scope tests --artifact /tmp/handoff.json
cargo run -- example --preset walkthrough
cargo run -- example --preset walkthrough > /tmp/magellan.json
cargo run -- go --input /tmp/magellan.json
cargo run -- validate --input examples/session-walkthrough.json
cargo run -- render --input examples/branch-handoff-timeline.json --format markdown
cargo run -- render --input examples/followup-validation-question.json --format html --out /tmp/magellan-question.html
```

`go` is the primary feedback loop — it validates, renders HTML, opens it, and
writes markdown in one step. Reach for `validate` and `render` only when you
need to inspect a single format in isolation.

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
