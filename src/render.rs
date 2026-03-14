use std::fmt::Write;

use anyhow::{Context, Result};
use schemars::schema_for;

use crate::model::{Diagram, Document, Edge, Section};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Terminal,
    Markdown,
    Html,
}

pub fn schema_json() -> Result<String> {
    let schema = schema_for!(Document);
    serde_json::to_string_pretty(&schema).context("failed to serialize JSON schema")
}

pub fn render_document(document: &Document, format: OutputFormat) -> String {
    match format {
        OutputFormat::Terminal => render_terminal(document),
        OutputFormat::Markdown => render_markdown(document),
        OutputFormat::Html => render_html(document),
    }
}

fn render_terminal(document: &Document) -> String {
    let mut output = String::new();

    writeln!(&mut output, "{}", document.title).unwrap();
    writeln!(
        &mut output,
        "{}",
        "=".repeat(document.title.chars().count())
    )
    .unwrap();
    writeln!(&mut output).unwrap();

    for paragraph in &document.summary {
        writeln!(&mut output, "{paragraph}").unwrap();
        writeln!(&mut output).unwrap();
    }

    for section in &document.sections {
        writeln!(
            &mut output,
            "{}\n{}",
            section.title,
            "-".repeat(section.title.len())
        )
        .unwrap();
        writeln!(&mut output).unwrap();
        for paragraph in &section.text {
            writeln!(&mut output, "{paragraph}").unwrap();
            writeln!(&mut output).unwrap();
        }

        if let Some(diagram) = &section.diagram {
            writeln!(&mut output, "{}", render_ascii_diagram(diagram)).unwrap();
            writeln!(&mut output).unwrap();
        }
    }

    if let Some(verification) = &document.verification {
        writeln!(&mut output, "Verification\n------------").unwrap();
        writeln!(&mut output).unwrap();
        for paragraph in &verification.text {
            writeln!(&mut output, "{paragraph}").unwrap();
            writeln!(&mut output).unwrap();
        }
    }

    output
}

fn render_markdown(document: &Document) -> String {
    let mut output = String::new();

    writeln!(&mut output, "# {}", document.title).unwrap();
    writeln!(&mut output).unwrap();

    for paragraph in &document.summary {
        writeln!(&mut output, "{paragraph}").unwrap();
        writeln!(&mut output).unwrap();
    }

    for section in &document.sections {
        writeln!(&mut output, "## {}", section.title).unwrap();
        writeln!(&mut output).unwrap();
        for paragraph in &section.text {
            writeln!(&mut output, "{paragraph}").unwrap();
            writeln!(&mut output).unwrap();
        }

        if let Some(diagram) = &section.diagram {
            writeln!(&mut output, "```mermaid").unwrap();
            writeln!(&mut output, "{}", render_mermaid_diagram(diagram)).unwrap();
            writeln!(&mut output, "```").unwrap();
            writeln!(&mut output).unwrap();
        }
    }

    if let Some(verification) = &document.verification {
        writeln!(&mut output, "## Verification").unwrap();
        writeln!(&mut output).unwrap();
        for paragraph in &verification.text {
            writeln!(&mut output, "{paragraph}").unwrap();
            writeln!(&mut output).unwrap();
        }
    }

    output
}

fn render_html(document: &Document) -> String {
    let summary_html = paragraphs_to_html(&document.summary);
    let sections_html = document
        .sections
        .iter()
        .map(render_section_html)
        .collect::<Vec<_>>()
        .join("\n");
    let verification_html = document
        .verification
        .as_ref()
        .map(|verification| {
            format!(
                "<section class=\"panel\"><h2>Verification</h2>{}</section>",
                paragraphs_to_html(&verification.text)
            )
        })
        .unwrap_or_default();
    let has_diagrams = document
        .sections
        .iter()
        .any(|section| section.diagram.is_some());
    let mermaid_bootstrap = if has_diagrams {
        String::from(
            "<script type=\"module\">
  import mermaid from 'https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.esm.min.mjs';
  mermaid.initialize({
    startOnLoad: true,
    securityLevel: 'loose',
    theme: 'base',
    themeVariables: {
      primaryColor: '#f4efe6',
      primaryTextColor: '#1c1917',
      primaryBorderColor: '#0f766e',
      lineColor: '#0f766e',
      fontFamily: 'ui-monospace, SFMono-Regular, Menlo, monospace'
    }
  });
</script>",
        )
    } else {
        String::new()
    };

    format!(
        "<!DOCTYPE html>
<html lang=\"en\">
<head>
  <meta charset=\"utf-8\">
  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
  <title>{title}</title>
  <link rel=\"icon\" href=\"data:,\">
  <style>
    :root {{
      color-scheme: light;
      --bg: #f5efe4;
      --paper: rgba(255, 252, 246, 0.92);
      --ink: #1c1917;
      --muted: #57534e;
      --accent: #0f766e;
      --border: rgba(28, 25, 23, 0.12);
      --shadow: 0 24px 60px rgba(28, 25, 23, 0.12);
    }}
    * {{
      box-sizing: border-box;
    }}
    body {{
      margin: 0;
      font-family: ui-serif, Georgia, Cambria, \"Times New Roman\", Times, serif;
      background:
        radial-gradient(circle at top left, rgba(15, 118, 110, 0.18), transparent 28%),
        linear-gradient(160deg, #f8f3eb 0%, #ece3d3 100%);
      color: var(--ink);
    }}
    main {{
      max-width: 920px;
      margin: 0 auto;
      padding: 48px 20px 80px;
    }}
    .hero,
    .panel {{
      background: var(--paper);
      border: 1px solid var(--border);
      border-radius: 22px;
      box-shadow: var(--shadow);
      padding: 28px;
      backdrop-filter: blur(6px);
    }}
    .hero {{
      margin-bottom: 20px;
    }}
    .eyebrow {{
      text-transform: uppercase;
      letter-spacing: 0.08em;
      font-size: 0.78rem;
      color: var(--accent);
      margin: 0 0 8px;
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    }}
    h1, h2 {{
      margin: 0 0 12px;
      line-height: 1.1;
      font-weight: 700;
    }}
    h1 {{
      font-size: clamp(2.2rem, 6vw, 3.8rem);
    }}
    h2 {{
      font-size: clamp(1.4rem, 4vw, 2rem);
    }}
    p {{
      margin: 0 0 14px;
      color: var(--muted);
      font-size: 1.05rem;
      line-height: 1.65;
    }}
    .stack {{
      display: grid;
      gap: 20px;
    }}
    .diagram {{
      margin-top: 18px;
      border-radius: 18px;
      border: 1px solid rgba(15, 118, 110, 0.16);
      background: linear-gradient(180deg, rgba(248, 250, 252, 0.96) 0%, rgba(241, 245, 249, 0.94) 100%);
      padding: 18px;
    }}
    .diagram .mermaid {{
      margin: 0;
      text-align: center;
    }}
    details {{
      margin-top: 16px;
    }}
    summary {{
      cursor: pointer;
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
      color: var(--accent);
      font-size: 0.88rem;
    }}
    pre {{
      margin: 18px 0 0;
      padding: 18px;
      border-radius: 16px;
      border: 1px solid rgba(15, 118, 110, 0.2);
      background: #f8fafc;
      overflow-x: auto;
      color: #0f172a;
      font-size: 0.95rem;
      line-height: 1.4;
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    }}
  </style>
  {mermaid_bootstrap}
</head>
<body>
  <main>
    <section class=\"hero\">
      <p class=\"eyebrow\">Magellan walkthrough</p>
      <h1>{title}</h1>
      {summary_html}
    </section>
    <div class=\"stack\">
      {sections_html}
      {verification_html}
    </div>
  </main>
</body>
</html>",
        title = escape_html(&document.title),
        summary_html = summary_html,
        sections_html = sections_html,
        verification_html = verification_html,
        mermaid_bootstrap = mermaid_bootstrap
    )
}

fn render_section_html(section: &Section) -> String {
    let diagram_html = section
        .diagram
        .as_ref()
        .map(render_diagram_html)
        .unwrap_or_default();

    format!(
        "<section class=\"panel\"><h2>{}</h2>{}{}</section>",
        escape_html(&section.title),
        paragraphs_to_html(&section.text),
        diagram_html
    )
}

fn render_diagram_html(diagram: &Diagram) -> String {
    let mermaid = escape_html(&render_mermaid_diagram(diagram));
    let ascii = escape_html(&render_ascii_diagram(diagram));
    format!(
        "<div class=\"diagram\"><pre class=\"mermaid\">{mermaid}</pre><details><summary>ASCII fallback</summary><pre>{ascii}</pre></details></div>"
    )
}

fn paragraphs_to_html(paragraphs: &[String]) -> String {
    paragraphs
        .iter()
        .map(|paragraph| format!("<p>{}</p>", escape_html(paragraph)))
        .collect::<Vec<_>>()
        .join("")
}

fn render_ascii_diagram(diagram: &Diagram) -> String {
    match diagram {
        Diagram::Sequence { edges, .. } => render_ascii_edges("Sequence", edges),
        Diagram::Flow { edges, .. } => render_ascii_edges("Flow", edges),
        Diagram::ComponentGraph { edges, .. } => render_ascii_edges("Component graph", edges),
        Diagram::Timeline { events } => {
            let mut output = String::from("Timeline\n");
            for event in events {
                writeln!(&mut output, "  * {}: {}", event.label, event.detail).unwrap();
            }
            output.trim_end().to_owned()
        }
        Diagram::BeforeAfter(before_after) => {
            let mut output = String::from("Before\n");
            for entry in &before_after.before {
                writeln!(&mut output, "  - {entry}").unwrap();
            }
            writeln!(&mut output, "After").unwrap();
            for entry in &before_after.after {
                writeln!(&mut output, "  + {entry}").unwrap();
            }
            output.trim_end().to_owned()
        }
    }
}

fn render_ascii_edges(title: &str, edges: &[Edge]) -> String {
    let mut output = String::new();
    writeln!(&mut output, "{title}").unwrap();
    for edge in edges {
        match &edge.label {
            Some(label) => {
                writeln!(&mut output, "  {} --{}--> {}", edge.from, label, edge.to).unwrap()
            }
            None => writeln!(&mut output, "  {} -------> {}", edge.from, edge.to).unwrap(),
        }
    }
    output.trim_end().to_owned()
}

fn render_mermaid_diagram(diagram: &Diagram) -> String {
    match diagram {
        Diagram::Sequence { edges, .. } => {
            let mut output = String::from("sequenceDiagram\n");
            for edge in edges {
                let label = edge.label.as_deref().unwrap_or("");
                writeln!(
                    &mut output,
                    "    {}->>{}: {}",
                    sanitize_node(&edge.from),
                    sanitize_node(&edge.to),
                    label
                )
                .unwrap();
            }
            output.trim_end().to_owned()
        }
        Diagram::Flow { edges, .. } | Diagram::ComponentGraph { edges, .. } => {
            let mut output = String::from("flowchart LR\n");
            for edge in edges {
                let label = edge
                    .label
                    .as_ref()
                    .map(|label| format!("|{}|", label))
                    .unwrap_or_default();
                writeln!(
                    &mut output,
                    "    {} -->{} {}",
                    sanitize_node(&edge.from),
                    label,
                    sanitize_node(&edge.to)
                )
                .unwrap();
            }
            output.trim_end().to_owned()
        }
        Diagram::Timeline { events } => {
            let mut output = String::from("timeline\n");
            writeln!(&mut output, "    title Timeline").unwrap();
            for event in events {
                writeln!(
                    &mut output,
                    "    {} : {}",
                    escape_mermaid_text(&event.label),
                    escape_mermaid_text(&event.detail)
                )
                .unwrap();
            }
            output.trim_end().to_owned()
        }
        Diagram::BeforeAfter(before_after) => {
            let mut output = String::from("flowchart TB\n");
            writeln!(&mut output, "    subgraph Before").unwrap();
            for (index, entry) in before_after.before.iter().enumerate() {
                writeln!(
                    &mut output,
                    "        B{}[\"{}\"]",
                    index,
                    escape_mermaid_text(entry)
                )
                .unwrap();
            }
            writeln!(&mut output, "    end").unwrap();
            writeln!(&mut output, "    subgraph After").unwrap();
            for (index, entry) in before_after.after.iter().enumerate() {
                writeln!(
                    &mut output,
                    "        A{}[\"{}\"]",
                    index,
                    escape_mermaid_text(entry)
                )
                .unwrap();
            }
            writeln!(&mut output, "    end").unwrap();
            output.trim_end().to_owned()
        }
    }
}

fn sanitize_node(name: &str) -> String {
    name.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn escape_mermaid_text(value: &str) -> String {
    value.replace('"', "\\\"")
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Diagram, Document, Edge, Section, Verification};

    fn sample_document() -> Document {
        Document {
            title: "Magellan demo".into(),
            summary: vec![
                "A short summary explains the outcome in product terms.".into(),
                "A second paragraph adds only the necessary context.".into(),
            ],
            sections: vec![Section {
                title: "New flow".into(),
                text: vec![
                    "The UI validates first.".into(),
                    "Only valid requests continue to the backend.".into(),
                ],
                diagram: Some(Diagram::Sequence {
                    nodes: vec!["User".into(), "UI".into(), "API".into()],
                    edges: vec![
                        Edge {
                            from: "User".into(),
                            to: "UI".into(),
                            label: Some("submit".into()),
                        },
                        Edge {
                            from: "UI".into(),
                            to: "API".into(),
                            label: Some("valid request".into()),
                        },
                    ],
                }),
            }],
            verification: Some(Verification {
                text: vec!["An integration test and a quick manual check passed.".into()],
            }),
        }
    }

    #[test]
    fn renders_terminal_output_with_ascii_diagram() {
        let rendered = render_document(&sample_document(), OutputFormat::Terminal);

        assert!(rendered.contains("Magellan demo"));
        assert!(rendered.contains("Sequence"));
        assert!(rendered.contains("User --submit--> UI"));
    }

    #[test]
    fn renders_markdown_with_mermaid_blocks() {
        let rendered = render_document(&sample_document(), OutputFormat::Markdown);

        assert!(rendered.contains("```mermaid"));
        assert!(rendered.contains("sequenceDiagram"));
        assert!(rendered.contains("User->>UI: submit"));
    }

    #[test]
    fn renders_html_panels() {
        let rendered = render_document(&sample_document(), OutputFormat::Html);

        assert!(rendered.contains("<!DOCTYPE html>"));
        assert!(rendered.contains("Magellan walkthrough"));
        assert!(rendered.contains("cdn.jsdelivr.net/npm/mermaid@11"));
        assert!(rendered.contains("<link rel=\"icon\" href=\"data:,\">"));
        assert!(rendered.contains("<pre class=\"mermaid\">sequenceDiagram"));
        assert!(rendered.contains("ASCII fallback"));
    }
}
