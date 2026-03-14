# Magellan Plan

## Working Idea

Magellan is a presentation engine for AI-generated technical walkthroughs.

The problem it solves is not "show me the diff". The problem is "help me understand what we just built without dumping a wall of text on me."

The intended output is a short, paced explanation made of:

- a small summary
- a few sections with short paragraphs
- diagrams that explain behavior or architecture
- optional HTML output for richer presentation without forcing the engineer to leave the agent

## Product Boundary

Magellan is not the LLM and should not try to infer meaning from raw code on its own.

The AI agent is responsible for:

- inspecting the session, code, diff, commits, tests, and notes
- deciding what changed in behavioral terms
- writing short summary text
- choosing useful diagram types
- providing the structured data for those diagrams

Magellan is responsible for:

- teaching the agent how to use it via `--help`
- validating a strict input schema
- enforcing compact presentation rules
- rendering terminal-friendly and HTML-friendly outputs

In short:

- agent = brains
- Magellan = presentation engine

## Desired Agent Interaction

Primary flow:

1. User asks the agent to explain what happened using Magellan.
2. Agent runs `magellan --help`.
3. `--help` tells the agent what content Magellan expects.
4. Agent analyzes the current task and creates a structured payload.
5. Agent calls Magellan to render that payload.
6. The final explanation appears in-chat, and optionally as an HTML file in `/tmp`.

This keeps the interaction inside Codex/Claude while still allowing richer visual output when needed.

## CLI Direction

Initial CLI shape:

```text
magellan --help
magellan schema
magellan render --input /path/to/payload.json
magellan render --input /path/to/payload.json --format html --out /tmp/magellan.html
magellan render --input /path/to/payload.json --format terminal
```

Potential future additions:

```text
magellan examples
magellan validate --input /path/to/payload.json
magellan render --stdin --format html --out /tmp/magellan.html
```

The CLI should be self-describing enough that an agent can discover it by running `--help`.

## Content Contract

The agent should provide structured content rather than freeform prose.

Suggested top-level shape:

```json
{
  "title": "What we built",
  "summary": [
    "A short opening paragraph.",
    "An optional second short paragraph."
  ],
  "sections": [
    {
      "title": "New request flow",
      "text": [
        "One to three short sentences.",
        "Another short sentence if needed."
      ],
      "diagram": {
        "type": "sequence",
        "nodes": ["User", "UI", "API"],
        "edges": [
          ["User", "UI", "submit"],
          ["UI", "API", "valid request"]
        ]
      }
    }
  ],
  "verification": {
    "text": [
      "What was verified and how."
    ]
  }
}
```

Presentation constraints should be strict:

- summary: 1-2 short paragraphs
- sections: usually 3-6
- section text: short paragraphs, not long essays
- diagrams: optional but encouraged when they improve comprehension
- output should explain behavior, flow, or decisions, not just file churn

## Diagram Strategy

The agent should choose the diagram type.
Magellan should validate the diagram spec and render it deterministically.

Useful first-wave diagram types:

- sequence
- flow
- component graph
- timeline
- before/after comparison

Useful render targets:

- Mermaid for Markdown / Codex app output
- ASCII for terminal-only output
- HTML for richer visual presentation

## MVP

V1 should stay narrow:

1. Accept structured JSON input.
2. Validate it against a schema.
3. Render compact terminal output.
4. Render a styled HTML report to a local path.
5. Support a small number of diagram types well.

V1 should not:

- call an LLM directly
- own session parsing
- infer meaning from a repository without agent help
- become a generic documentation generator

## Design Principles

- Explain behavior, not file lists.
- Prefer diagrams when they clarify flow.
- Keep paragraphs short and sectioned.
- Make the tool agent-friendly through discovery and schema.
- Keep rendering deterministic and testable.

## Open Questions For Implementation

- Which schema format should be the source of truth: JSON Schema, Zod, or both?
- Should terminal rendering emit plain text only, or also Mermaid blocks when available?
- How opinionated should the default HTML theme be?
- Should examples live inside `magellan examples`, the README, or both?

## Stack Discussion Starting Point

We have not chosen the implementation stack yet, but the architecture suggests a few requirements:

- strong JSON/schema validation
- easy CLI ergonomics
- good text templating
- simple HTML generation
- predictable packaging and local installation

That makes TypeScript and Python the most obvious first options to compare in the next discussion.
