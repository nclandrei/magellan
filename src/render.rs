use std::collections::HashSet;
use std::fmt::Write;

use anyhow::{Context, Result};
use schemars::schema_for;

use crate::model::{BeforeAfterDiagram, Diagram, Document, Edge, Section, TimelineEvent};

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
        .enumerate()
        .map(|(index, section)| render_section_html(index, section))
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
    .diagram-label {{
      margin: 0 0 12px;
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
      font-size: 0.82rem;
      font-weight: 600;
      letter-spacing: 0.04em;
      text-transform: uppercase;
      color: var(--accent);
    }}
    .diagram svg {{
      display: block;
      width: 100%;
      height: auto;
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
        verification_html = verification_html
    )
}

fn render_section_html(index: usize, section: &Section) -> String {
    let diagram_html = section
        .diagram
        .as_ref()
        .map(|diagram| render_diagram_html(index, diagram))
        .unwrap_or_default();

    format!(
        "<section class=\"panel\"><h2>{}</h2>{}{}</section>",
        escape_html(&section.title),
        paragraphs_to_html(&section.text),
        diagram_html
    )
}

fn render_diagram_html(index: usize, diagram: &Diagram) -> String {
    let svg = render_svg_diagram(index, diagram);
    let ascii = escape_html(&render_ascii_diagram(diagram));

    format!(
        "<div class=\"diagram\"><p class=\"diagram-label\">{}</p>{}<details><summary>ASCII fallback</summary><pre>{}</pre></details></div>",
        diagram_title(diagram),
        svg,
        ascii
    )
}

fn render_svg_diagram(index: usize, diagram: &Diagram) -> String {
    let diagram_id = format!("diagram-{index}");
    match diagram {
        Diagram::Sequence { nodes, edges } => render_sequence_svg(&diagram_id, nodes, edges),
        Diagram::Flow { nodes, edges } => render_graph_svg(&diagram_id, "Flow", nodes, edges),
        Diagram::ComponentGraph { nodes, edges } => {
            render_graph_svg(&diagram_id, "Component graph", nodes, edges)
        }
        Diagram::Timeline { events } => render_timeline_svg(&diagram_id, events),
        Diagram::BeforeAfter(before_after) => render_before_after_svg(&diagram_id, before_after),
    }
}

fn render_sequence_svg(id: &str, nodes: &[String], edges: &[Edge]) -> String {
    let nodes = ordered_nodes(nodes, edges);
    let wrapped_nodes = nodes
        .iter()
        .map(|node| wrap_text(node, 14))
        .collect::<Vec<_>>();
    let box_width = wrapped_nodes
        .iter()
        .flat_map(|lines| lines.iter())
        .map(|line| estimate_text_width(line, 112, 180))
        .max()
        .unwrap_or(112);
    let box_height = wrapped_nodes
        .iter()
        .map(|lines| 24 + (lines.len() as i32 * 16))
        .max()
        .unwrap_or(48);
    let padding = 24;
    let lane_gap = 42;
    let width = padding * 2
        + (nodes.len() as i32 * box_width)
        + ((nodes.len().saturating_sub(1)) as i32 * lane_gap);
    let lifeline_start = 28 + box_height + 14;
    let height = lifeline_start + (edges.len() as i32 * 66) + 48;
    let marker_id = format!("{id}-arrow");

    let mut body = String::new();

    for (index, node) in nodes.iter().enumerate() {
        let x = padding + index as i32 * (box_width + lane_gap);
        let center_x = x + box_width / 2;

        write!(
            &mut body,
            "<rect class=\"node\" x=\"{x}\" y=\"28\" width=\"{box_width}\" height=\"{box_height}\" rx=\"16\" ry=\"16\"/>"
        )
        .unwrap();
        write_multiline_svg_text(
            &mut body,
            center_x,
            28 + 26,
            &wrap_text(node, 14),
            "middle",
            "node-copy",
        );
        write!(
            &mut body,
            "<line class=\"lane\" x1=\"{center_x}\" y1=\"{}\" x2=\"{center_x}\" y2=\"{}\"/>",
            lifeline_start,
            height - 24
        )
        .unwrap();
    }

    for (index, edge) in edges.iter().enumerate() {
        let from_index = nodes
            .iter()
            .position(|node| node == &edge.from)
            .unwrap_or_default();
        let to_index = nodes
            .iter()
            .position(|node| node == &edge.to)
            .unwrap_or_default();
        let from_x = padding + from_index as i32 * (box_width + lane_gap) + box_width / 2;
        let to_x = padding + to_index as i32 * (box_width + lane_gap) + box_width / 2;
        let y = lifeline_start + 18 + index as i32 * 66;

        write!(
            &mut body,
            "<line class=\"connector\" x1=\"{from_x}\" y1=\"{y}\" x2=\"{to_x}\" y2=\"{y}\" marker-end=\"url(#{marker_id})\"/>"
        )
        .unwrap();

        if let Some(label) = &edge.label {
            write_multiline_svg_text(
                &mut body,
                (from_x + to_x) / 2,
                y - 12,
                &wrap_text(label, 18),
                "middle",
                "edge-copy",
            );
        }
    }

    svg_shell(
        id,
        width.max(320),
        height.max(180),
        &marker_id,
        "Sequence diagram",
        &body,
    )
}

fn render_graph_svg(id: &str, title: &str, nodes: &[String], edges: &[Edge]) -> String {
    let nodes = ordered_nodes(nodes, edges);
    let columns = nodes.len().clamp(1, 3);
    let wrapped_nodes = nodes
        .iter()
        .map(|node| wrap_text(node, 16))
        .collect::<Vec<_>>();
    let box_width = wrapped_nodes
        .iter()
        .flat_map(|lines| lines.iter())
        .map(|line| estimate_text_width(line, 120, 184))
        .max()
        .unwrap_or(120);
    let box_height = wrapped_nodes
        .iter()
        .map(|lines| 24 + (lines.len() as i32 * 16))
        .max()
        .unwrap_or(54);
    let padding = 28;
    let gap_x = 54;
    let gap_y = 68;
    let rows = ((nodes.len() + columns - 1) / columns).max(1);
    let width =
        padding * 2 + columns as i32 * box_width + (columns.saturating_sub(1)) as i32 * gap_x;
    let height = padding * 2 + rows as i32 * box_height + (rows.saturating_sub(1)) as i32 * gap_y;
    let marker_id = format!("{id}-arrow");

    let positions = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let row = index / columns;
            let col = index % columns;
            let x = padding + col as i32 * (box_width + gap_x);
            let y = padding + row as i32 * (box_height + gap_y);
            (node.as_str(), (x, y))
        })
        .collect::<Vec<_>>();

    let mut body = String::new();

    for edge in edges {
        let Some((from_x, from_y)) = positions
            .iter()
            .find(|(node, _)| *node == edge.from.as_str())
            .map(|(_, position)| *position)
        else {
            continue;
        };
        let Some((to_x, to_y)) = positions
            .iter()
            .find(|(node, _)| *node == edge.to.as_str())
            .map(|(_, position)| *position)
        else {
            continue;
        };

        let start_x = from_x + box_width / 2;
        let start_y = from_y + box_height / 2;
        let end_x = to_x + box_width / 2;
        let end_y = to_y + box_height / 2;

        write!(
            &mut body,
            "<line class=\"connector\" x1=\"{start_x}\" y1=\"{start_y}\" x2=\"{end_x}\" y2=\"{end_y}\" marker-end=\"url(#{marker_id})\"/>"
        )
        .unwrap();

        if let Some(label) = &edge.label {
            write_multiline_svg_text(
                &mut body,
                (start_x + end_x) / 2,
                (start_y + end_y) / 2 - 8,
                &wrap_text(label, 14),
                "middle",
                "edge-copy",
            );
        }
    }

    for ((_, (x, y)), lines) in positions.iter().zip(wrapped_nodes.iter()) {
        let center_x = *x + box_width / 2;
        write!(
            &mut body,
            "<rect class=\"node\" x=\"{}\" y=\"{}\" width=\"{box_width}\" height=\"{box_height}\" rx=\"16\" ry=\"16\"/>",
            x,
            y
        )
        .unwrap();
        write_multiline_svg_text(&mut body, center_x, *y + 26, lines, "middle", "node-copy");
    }

    svg_shell(
        id,
        width.max(320),
        height.max(220),
        &marker_id,
        title,
        &body,
    )
}

fn render_timeline_svg(id: &str, events: &[TimelineEvent]) -> String {
    let padding = 28;
    let width = 760;
    let height = 60 + events.len() as i32 * 92;
    let axis_x = 86;
    let marker_id = format!("{id}-arrow");
    let mut body = String::new();

    write!(
        &mut body,
        "<line class=\"timeline-axis\" x1=\"{axis_x}\" y1=\"32\" x2=\"{axis_x}\" y2=\"{}\"/>",
        height - 32
    )
    .unwrap();

    for (index, event) in events.iter().enumerate() {
        let y = 56 + index as i32 * 92;
        write!(
            &mut body,
            "<circle class=\"timeline-dot\" cx=\"{axis_x}\" cy=\"{y}\" r=\"9\"/>"
        )
        .unwrap();
        write!(
            &mut body,
            "<rect class=\"panel-box\" x=\"132\" y=\"{}\" width=\"{}\" height=\"64\" rx=\"16\" ry=\"16\"/>",
            y - 26,
            width - padding - 132
        )
        .unwrap();
        write_multiline_svg_text(
            &mut body,
            156,
            y - 4,
            &wrap_text(&event.label, 20),
            "start",
            "event-label",
        );
        write_multiline_svg_text(
            &mut body,
            156,
            y + 18,
            &wrap_text(&event.detail, 56),
            "start",
            "event-copy",
        );
    }

    svg_shell(id, width, height.max(180), &marker_id, "Timeline", &body)
}

fn render_before_after_svg(id: &str, before_after: &BeforeAfterDiagram) -> String {
    let gap = 26;
    let padding = 24;
    let width = 760;
    let panel_width = (width - (padding * 2) - gap) / 2;
    let left_x = padding;
    let right_x = padding + panel_width + gap;
    let before_lines = list_to_lines(&before_after.before, 24);
    let after_lines = list_to_lines(&before_after.after, 24);
    let line_height = 16;
    let list_height = before_lines.len().max(after_lines.len()) as i32 * line_height + 26;
    let panel_height = list_height + 46;
    let height = panel_height + 72;
    let marker_id = format!("{id}-arrow");
    let mut body = String::new();

    write!(
        &mut body,
        "<rect class=\"panel-box\" x=\"{left_x}\" y=\"34\" width=\"{panel_width}\" height=\"{panel_height}\" rx=\"18\" ry=\"18\"/>"
    )
    .unwrap();
    write!(
        &mut body,
        "<rect class=\"panel-box\" x=\"{right_x}\" y=\"34\" width=\"{panel_width}\" height=\"{panel_height}\" rx=\"18\" ry=\"18\"/>"
    )
    .unwrap();

    write_multiline_svg_text(
        &mut body,
        left_x + 22,
        62,
        &[String::from("Before")],
        "start",
        "event-label",
    );
    write_multiline_svg_text(
        &mut body,
        right_x + 22,
        62,
        &[String::from("After")],
        "start",
        "event-label",
    );
    write_bullet_lines(&mut body, left_x + 22, 92, &before_lines);
    write_bullet_lines(&mut body, right_x + 22, 92, &after_lines);

    svg_shell(id, width, height, &marker_id, "Before and after", &body)
}

fn svg_shell(
    id: &str,
    width: i32,
    height: i32,
    marker_id: &str,
    title: &str,
    body: &str,
) -> String {
    format!(
        "<svg viewBox=\"0 0 {width} {height}\" role=\"img\" aria-labelledby=\"{id}-title\">
  <title id=\"{id}-title\">{title}</title>
  <style>
    .node, .panel-box {{
      fill: rgba(255, 252, 246, 0.98);
      stroke: rgba(15, 118, 110, 0.86);
      stroke-width: 1.5;
    }}
    .lane {{
      stroke: rgba(15, 118, 110, 0.36);
      stroke-width: 1.4;
      stroke-dasharray: 7 7;
    }}
    .connector {{
      stroke: rgba(15, 118, 110, 0.9);
      stroke-width: 2;
      fill: none;
    }}
    .timeline-axis {{
      stroke: rgba(15, 118, 110, 0.42);
      stroke-width: 3;
    }}
    .timeline-dot {{
      fill: rgba(15, 118, 110, 0.92);
      stroke: rgba(15, 118, 110, 0.2);
      stroke-width: 6;
    }}
    .node-copy, .edge-copy, .event-copy, .event-label, .bullet-copy {{
      fill: #0f172a;
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    }}
    .node-copy {{
      font-size: 13px;
      font-weight: 600;
    }}
    .edge-copy {{
      font-size: 11px;
      font-weight: 600;
      fill: #475569;
    }}
    .event-label {{
      font-size: 13px;
      font-weight: 700;
    }}
    .event-copy, .bullet-copy {{
      font-size: 12px;
      fill: #475569;
    }}
  </style>
  <defs>
    <marker id=\"{marker_id}\" viewBox=\"0 0 10 10\" refX=\"8\" refY=\"5\" markerWidth=\"7\" markerHeight=\"7\" orient=\"auto-start-reverse\">
      <path d=\"M 0 0 L 10 5 L 0 10 z\" fill=\"rgba(15, 118, 110, 0.9)\" />
    </marker>
  </defs>
  {body}
</svg>",
        title = escape_html(title)
    )
}

fn write_multiline_svg_text(
    output: &mut String,
    x: i32,
    y: i32,
    lines: &[String],
    anchor: &str,
    class_name: &str,
) {
    if lines.is_empty() {
        return;
    }

    write!(
        output,
        "<text class=\"{class_name}\" x=\"{x}\" y=\"{y}\" text-anchor=\"{anchor}\">"
    )
    .unwrap();

    for (index, line) in lines.iter().enumerate() {
        let dy = if index == 0 { 0 } else { 15 };
        write!(
            output,
            "<tspan x=\"{x}\" dy=\"{dy}\">{}</tspan>",
            escape_html(line)
        )
        .unwrap();
    }

    output.push_str("</text>");
}

fn write_bullet_lines(output: &mut String, x: i32, start_y: i32, lines: &[String]) {
    for (index, line) in lines.iter().enumerate() {
        let y = start_y + index as i32 * 16;
        write!(
            output,
            "<circle cx=\"{}\" cy=\"{}\" r=\"2.6\" fill=\"rgba(15, 118, 110, 0.86)\"/>",
            x,
            y - 4
        )
        .unwrap();
        write_multiline_svg_text(
            output,
            x + 12,
            y,
            std::slice::from_ref(line),
            "start",
            "bullet-copy",
        );
    }
}

fn ordered_nodes(explicit_nodes: &[String], edges: &[Edge]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut nodes = Vec::new();

    for node in explicit_nodes {
        if seen.insert(node.clone()) {
            nodes.push(node.clone());
        }
    }

    for edge in edges {
        for node in [&edge.from, &edge.to] {
            if seen.insert(node.clone()) {
                nodes.push(node.clone());
            }
        }
    }

    nodes
}

fn list_to_lines(entries: &[String], max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for entry in entries {
        let wrapped = wrap_text(entry, max_chars);
        for line in wrapped {
            lines.push(line);
        }
    }
    lines
}

fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let mut lines = Vec::new();
    let mut current = String::new();

    for word in trimmed.split_whitespace() {
        let projected_len = if current.is_empty() {
            word.len()
        } else {
            current.len() + 1 + word.len()
        };

        if projected_len > max_chars && !current.is_empty() {
            lines.push(current);
            current = word.to_string();
        } else if current.is_empty() {
            current = word.to_string();
        } else {
            current.push(' ');
            current.push_str(word);
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    lines
}

fn estimate_text_width(text: &str, min: i32, max: i32) -> i32 {
    ((text.chars().count() as i32 * 8) + 28).clamp(min, max)
}

fn paragraphs_to_html(paragraphs: &[String]) -> String {
    paragraphs
        .iter()
        .map(|paragraph| format!("<p>{}</p>", escape_html(paragraph)))
        .collect::<Vec<_>>()
        .join("")
}

fn diagram_title(diagram: &Diagram) -> &'static str {
    match diagram {
        Diagram::Sequence { .. } => "Sequence diagram",
        Diagram::Flow { .. } => "Flow diagram",
        Diagram::ComponentGraph { .. } => "Component diagram",
        Diagram::Timeline { .. } => "Timeline",
        Diagram::BeforeAfter(_) => "Before / after",
    }
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
    use crate::{ExamplePreset, example_document};

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
        assert!(rendered.contains("<link rel=\"icon\" href=\"data:,\">"));
        assert!(rendered.contains("<svg viewBox="));
        assert!(rendered.contains("ASCII fallback"));
        assert!(!rendered.contains("cdn.jsdelivr"));
    }

    #[test]
    fn html_output_is_self_contained_for_all_examples() {
        for preset in [
            ExamplePreset::Walkthrough,
            ExamplePreset::Timeline,
            ExamplePreset::BeforeAfter,
        ] {
            let rendered = render_document(&example_document(preset), OutputFormat::Html);
            assert!(rendered.contains("<svg viewBox="));
            assert!(!rendered.contains("https://"));
            assert!(!rendered.contains("cdn.jsdelivr"));
        }
    }
}
