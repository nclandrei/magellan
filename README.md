# Magellan

Magellan is a deterministic presentation engine for AI-generated technical walkthroughs.

It does not call an LLM. Instead, an agent or engineer prepares a structured payload with a short summary, paced sections, and optional diagram data. Magellan validates that payload and renders it into terminal, Markdown, or HTML output.

## Current CLI

```text
magellan schema
magellan example --preset walkthrough
magellan validate --input payload.json
magellan render --input payload.json --format terminal
magellan render --input payload.json --format markdown
magellan render --input payload.json --format html --out /tmp/magellan.html
magellan render --input payload.json --format html --open
```

Use `--input -` to read a JSON payload from stdin.
Use `magellan example --preset walkthrough` when you want a starter payload to edit.

## Why This Exists

Normal agent explanations often fail in one of two ways:

- they dump a wall of text that is hard to follow
- they focus on file churn instead of behavior

Magellan pushes the output into a better shape:

- 1-2 short summary paragraphs
- 3-6 sections
- short section text instead of essays
- diagrams when they help explain flow, architecture, timing, or before/after changes

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
- `html`: a styled local report with Mermaid diagrams and ASCII fallbacks

When you pass `--open` with `--format html`, Magellan writes the report and opens it in the default browser. If `--out` is omitted, Magellan creates a temp file automatically.

## Development

Build and verify with:

```bash
cargo fmt
cargo test
```

Run locally with:

```bash
cargo run -- schema
cargo run -- example --preset walkthrough
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
