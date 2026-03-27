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
    let diagram_count = document
        .sections
        .iter()
        .filter(|section| section.diagram.is_some())
        .count();
    let total_pages = 1 + document.sections.len() + usize::from(document.verification.is_some());
    let book_pages_html = render_book_pages(document, total_pages, diagram_count);
    let page_dots_html = render_page_dots(document);
    let sections_html = document
        .sections
        .iter()
        .enumerate()
        .map(|(index, section)| render_overview_section_html(index, section))
        .collect::<Vec<_>>()
        .join("\n");
    let verification_html = document
        .verification
        .as_ref()
        .map(|verification| {
            format!(
                "<section class=\"panel\">
                  <div class=\"panel-head\">
                    <p class=\"eyebrow\">Verification</p>
                    <h2>Verification</h2>
                  </div>
                  <div class=\"panel-body\">
                    <div class=\"panel-copy\">{}</div>
                  </div>
                </section>",
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
  <style>{style}</style>
</head>
<body>
  <main class=\"report-shell\" data-magellan-report data-layout=\"spread\">
    <header class=\"report-bar\">
      <div class=\"report-context\">
        <p class=\"eyebrow\">Magellan walkthrough</p>
        <p class=\"report-title\" title=\"{title}\">{title}</p>
      </div>
      <div class=\"report-toolbar\">
        <div class=\"page-status\" aria-live=\"polite\">
          <span class=\"page-label\" data-current-page-label>Summary</span>
          <span class=\"page-counter\" data-page-counter>Page 1 / {total_pages}</span>
        </div>
        <div class=\"toolbar-cluster\">
          <div class=\"view-toggle-group\" role=\"tablist\" aria-label=\"Report views\">
            <button class=\"view-toggle is-active\" type=\"button\" data-view-target=\"book\" aria-pressed=\"true\">Book View</button>
            <button class=\"view-toggle\" type=\"button\" data-view-target=\"overview\" aria-pressed=\"false\">Overview</button>
          </div>
        </div>
      </div>
    </header>

    <section class=\"book-view is-active\" data-view=\"book\">
      <div class=\"book-shell\">
        <div class=\"book-window\">
          <div class=\"book-track\" data-book-track>
            {book_pages_html}
          </div>
        </div>
        <div class=\"book-nav\">
          <button class=\"nav-button\" type=\"button\" data-prev-page>Previous</button>
          <div class=\"page-dots\" aria-label=\"Walkthrough pages\">
            {page_dots_html}
          </div>
          <button class=\"nav-button\" type=\"button\" data-next-page>Next</button>
        </div>
      </div>
    </section>

    <section class=\"overview-view\" data-view=\"overview\" hidden>
      <section class=\"hero\">
        <p class=\"eyebrow\">Unified view</p>
        <h2>{title}</h2>
        {summary_html}
      </section>
      <div class=\"stack\">
        {sections_html}
        {verification_html}
      </div>
    </section>

    {diagram_modal_html}
  </main>
  <script>{script}</script>
</body>
</html>",
        title = escape_html(&document.title),
        total_pages = total_pages,
        summary_html = summary_html,
        sections_html = sections_html,
        verification_html = verification_html,
        book_pages_html = book_pages_html,
        page_dots_html = page_dots_html,
        diagram_modal_html = render_diagram_modal_shell(),
        style = html_style(),
        script = html_script()
    )
}

fn render_book_pages(document: &Document, total_pages: usize, diagram_count: usize) -> String {
    let mut pages = vec![render_summary_page(
        &document.title,
        &document.summary,
        total_pages,
        document.sections.len(),
        diagram_count,
        document.verification.is_some(),
    )];

    pages.extend(
        document
            .sections
            .iter()
            .enumerate()
            .map(|(index, section)| render_section_page(index, total_pages, section)),
    );

    if let Some(verification) = &document.verification {
        pages.push(render_verification_page(
            total_pages - 1,
            total_pages,
            verification,
        ));
    }

    pages.join("\n")
}

fn render_summary_page(
    title: &str,
    summary: &[String],
    total_pages: usize,
    section_count: usize,
    diagram_count: usize,
    has_verification: bool,
) -> String {
    let verification_label = if has_verification { "Included" } else { "None" };

    format!(
        "<article class=\"page page-summary is-current\" data-page data-page-title=\"Summary\">
          <div class=\"page-head\">
            <p class=\"eyebrow\">Overview page</p>
            <p class=\"page-step\">Page 1 / {total_pages}</p>
          </div>
          <div class=\"page-grid summary-grid\">
            <div class=\"page-copy\">
              <h1 class=\"summary-title\">{title}</h1>
              {summary_html}
            </div>
            <aside class=\"summary-stats\" aria-label=\"Walkthrough summary\">
              <div class=\"stat-card\">
                <span class=\"stat-label\">Pages</span>
                <strong>{total_pages}</strong>
              </div>
              <div class=\"stat-card\">
                <span class=\"stat-label\">Sections</span>
                <strong>{section_count}</strong>
              </div>
              <div class=\"stat-card\">
                <span class=\"stat-label\">Diagrams</span>
                <strong>{diagram_count}</strong>
              </div>
              <div class=\"stat-card\">
                <span class=\"stat-label\">Verification</span>
                <strong>{verification_label}</strong>
              </div>
            </aside>
          </div>
        </article>",
        title = escape_html(title),
        summary_html = paragraphs_to_html(summary),
        total_pages = total_pages,
        section_count = section_count,
        diagram_count = diagram_count,
        verification_label = verification_label
    )
}

fn render_section_page(index: usize, total_pages: usize, section: &Section) -> String {
    let page_number = index + 2;
    let diagram_html = section
        .diagram
        .as_ref()
        .map(|diagram| {
            format!(
                "<div class=\"page-visual\">{}</div>",
                render_diagram_html(index, diagram, true)
            )
        })
        .unwrap_or_default();
    let page_class = if section.diagram.is_some() {
        "page has-diagram"
    } else {
        "page"
    };

    format!(
        "<article class=\"{page_class}\" data-page data-page-title=\"{title}\">
          <div class=\"page-head\">
            <p class=\"eyebrow\">Step {step}</p>
            <p class=\"page-step\">Page {page_number} / {total_pages}</p>
          </div>
          <div class=\"page-grid\">
            <div class=\"page-copy\">
              <h2>{title}</h2>
              {text_html}
            </div>
            {diagram_html}
          </div>
        </article>",
        page_class = page_class,
        step = index + 1,
        page_number = page_number,
        total_pages = total_pages,
        title = escape_html(&section.title),
        text_html = paragraphs_to_html(&section.text),
        diagram_html = diagram_html
    )
}

fn render_verification_page(
    page_index: usize,
    total_pages: usize,
    verification: &crate::model::Verification,
) -> String {
    format!(
        "<article class=\"page page-verification\" data-page data-page-title=\"Verification\">
          <div class=\"page-head\">
            <p class=\"eyebrow\">Verification</p>
            <p class=\"page-step\">Page {page_number} / {total_pages}</p>
          </div>
          <div class=\"page-grid verification-grid\">
            <div class=\"verification-badge\" aria-hidden=\"true\">Verified</div>
            <div class=\"page-copy\">
              <h2>Verification</h2>
              {text_html}
            </div>
          </div>
        </article>",
        page_number = page_index + 1,
        total_pages = total_pages,
        text_html = paragraphs_to_html(&verification.text)
    )
}

fn render_page_dots(document: &Document) -> String {
    let mut dots = vec![render_page_dot(0, "Summary", true)];

    dots.extend(
        document
            .sections
            .iter()
            .enumerate()
            .map(|(index, section)| render_page_dot(index + 1, &section.title, false)),
    );

    if document.verification.is_some() {
        dots.push(render_page_dot(
            document.sections.len() + 1,
            "Verification",
            false,
        ));
    }

    dots.join("\n")
}

fn render_page_dot(index: usize, label: &str, is_active: bool) -> String {
    let active_class = if is_active { " is-active" } else { "" };
    let current = if is_active { "true" } else { "false" };

    format!(
        "<button class=\"page-dot{active_class}\" type=\"button\" data-page-dot=\"{index}\" aria-label=\"Go to {label}\" aria-current=\"{current}\"></button>",
        active_class = active_class,
        index = index,
        label = escape_html(label),
        current = current
    )
}

fn render_overview_section_html(index: usize, section: &Section) -> String {
    let diagram_html = section
        .diagram
        .as_ref()
        .map(|diagram| {
            format!(
                "<div class=\"panel-visual\">{}</div>",
                render_diagram_html(index, diagram, false)
            )
        })
        .unwrap_or_default();
    let panel_class = if section.diagram.is_some() {
        "panel panel-has-diagram"
    } else {
        "panel"
    };

    format!(
        "<section class=\"{panel_class}\">
          <div class=\"panel-head\">
            <p class=\"eyebrow\">Step {step}</p>
            <h2>{title}</h2>
          </div>
          <div class=\"panel-body\">
            <div class=\"panel-copy\">{text_html}</div>
            {diagram_html}
          </div>
        </section>",
        panel_class = panel_class,
        step = index + 1,
        title = escape_html(&section.title),
        text_html = paragraphs_to_html(&section.text),
        diagram_html = diagram_html
    )
}

fn render_diagram_modal_shell() -> &'static str {
    r#"<div class="diagram-modal" data-diagram-modal hidden>
      <button class="diagram-modal-backdrop" type="button" data-diagram-close aria-label="Close expanded diagram"></button>
      <div class="diagram-modal-card" role="dialog" aria-modal="true" aria-labelledby="diagram-modal-title">
        <div class="diagram-modal-header">
          <div>
            <p class="eyebrow">Expanded diagram</p>
            <h2 class="diagram-modal-title" id="diagram-modal-title" data-diagram-modal-title>Diagram</h2>
          </div>
          <button class="nav-button diagram-modal-close" type="button" data-diagram-close>Close</button>
        </div>
        <div class="diagram-modal-body" data-diagram-modal-body></div>
      </div>
    </div>"#
}

fn html_style() -> &'static str {
    r#"
    :root {
      color-scheme: dark;
      --bg: #111111;
      --surface: #1a1a1a;
      --surface-strong: #1e1e1e;
      --surface-soft: #222222;
      --surface-elevated: #252525;
      --ink: #e8e8e8;
      --ink-soft: #cccccc;
      --muted: #999999;
      --accent: #7eb8ff;
      --accent-strong: #a0ccff;
      --accent-soft: rgba(126, 184, 255, 0.1);
      --accent-line: rgba(126, 184, 255, 0.15);
      --border: rgba(255, 255, 255, 0.1);
      --shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
      --shadow-soft: 0 1px 2px rgba(0, 0, 0, 0.2);
      --code-bg: #161616;
    }
    * {
      box-sizing: border-box;
    }
    html {
      scroll-behavior: smooth;
    }
    body {
      margin: 0;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
      background: var(--bg);
      color: var(--ink);
    }
    body[data-diagram-modal-open="true"] {
      overflow: hidden;
    }
    .report-shell {
      max-width: 1180px;
      margin: 0 auto;
      padding: 24px 18px 72px;
    }
    .hero,
    .panel,
    .page {
      background: var(--surface);
      border: 1px solid var(--border);
      border-radius: 10px;
      box-shadow: var(--shadow);
    }
    .report-bar {
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      justify-content: space-between;
      gap: 12px 18px;
      margin-bottom: 14px;
      padding: 14px 18px;
      border: 1px solid var(--border);
      border-radius: 10px;
      background: var(--surface);
    }
    .report-context {
      min-width: 0;
      display: grid;
      gap: 4px;
    }
    .report-title {
      margin: 0;
      max-width: 48ch;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      color: var(--ink);
      font-size: 1rem;
      line-height: 1.25;
      font-weight: 600;
    }
    .report-toolbar {
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      justify-content: flex-end;
      gap: 10px 14px;
      position: relative;
      z-index: 3;
    }
    .toolbar-cluster {
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      justify-content: flex-end;
      gap: 10px;
    }
    .eyebrow {
      text-transform: uppercase;
      letter-spacing: 0.08em;
      font-size: 0.78rem;
      color: var(--accent-strong);
      margin: 0 0 8px;
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    }
    h1,
    h2 {
      margin: 0 0 12px;
      line-height: 1.05;
      font-weight: 700;
      color: var(--ink);
    }
    h1 {
      max-width: 20ch;
      font-size: clamp(1.8rem, 3.5vw, 2.6rem);
      text-wrap: balance;
    }
    h2 {
      font-size: clamp(1.3rem, 2.5vw, 1.7rem);
      text-wrap: balance;
    }
    p {
      margin: 0 0 14px;
      color: var(--muted);
      font-size: 1.03rem;
      line-height: 1.76;
    }
    .view-toggle-group {
      display: inline-flex;
      align-items: center;
      gap: 2px;
      padding: 3px;
      border-radius: 8px;
      background: var(--surface-soft);
      border: 1px solid var(--border);
    }
    .view-toggle,
    .nav-button,
    .page-dot {
      appearance: none;
      border: 0;
      cursor: pointer;
      transition: background-color 120ms ease, color 120ms ease, opacity 120ms ease;
    }
    .view-toggle {
      padding: 6px 12px;
      border-radius: 6px;
      border: 1px solid transparent;
      background: transparent;
      color: var(--muted);
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
      font-size: 0.82rem;
    }
    .view-toggle.is-active {
      background: var(--accent);
      color: #111111;
    }
    .page-status {
      display: flex;
      flex-direction: column;
      align-items: flex-end;
      gap: 2px;
      min-width: 0;
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    }
    .page-label {
      max-width: 28ch;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      font-size: 0.88rem;
      color: var(--ink);
      font-weight: 600;
    }
    .page-counter {
      font-size: 0.74rem;
      color: var(--accent);
      text-transform: uppercase;
      letter-spacing: 0.06em;
    }
    .book-view[hidden],
    .overview-view[hidden] {
      display: none;
    }
    .book-shell {
      display: grid;
      gap: 14px;
      height: calc(100vh - 180px);
      min-height: 620px;
      grid-template-rows: minmax(0, 1fr) auto;
    }
    .book-window {
      overflow: hidden;
      border-radius: 10px;
      position: relative;
      z-index: 1;
      min-height: 0;
      height: 100%;
    }
    .book-track {
      display: flex;
      width: 100%;
      transition: transform 280ms ease;
    }
    .page {
      width: 100%;
      min-width: 100%;
      padding: 26px 28px 28px;
      height: 100%;
      display: flex;
      flex-direction: column;
      gap: 22px;
      overflow-y: auto;
    }
    .page-head {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 12px;
      padding-bottom: 16px;
      border-bottom: 1px solid var(--accent-line);
    }
    .page-step {
      margin: 0;
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
      font-size: 0.82rem;
      color: var(--accent);
      text-transform: uppercase;
      letter-spacing: 0.06em;
      white-space: nowrap;
    }
    .page-grid {
      display: grid;
      gap: 24px;
      align-items: start;
      grid-template-columns: 1fr;
    }
    .page-copy {
      min-width: 0;
      max-width: 64ch;
    }
    .page-copy p:last-child {
      margin-bottom: 0;
    }
    .page-visual {
      min-width: 0;
    }
    .summary-grid {
      align-items: stretch;
    }
    .summary-title {
      max-width: 28ch;
      margin-bottom: 16px;
      font-size: clamp(1.8rem, 3.5vw, 2.6rem);
      line-height: 1.1;
    }
    .summary-stats {
      display: grid;
      gap: 14px;
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
    .stat-card {
      background: var(--surface-soft);
      border: 1px solid var(--border);
      border-radius: 8px;
      padding: 14px;
      display: grid;
      gap: 6px;
      align-content: start;
    }
    .stat-label {
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
      font-size: 0.78rem;
      text-transform: uppercase;
      letter-spacing: 0.06em;
      color: var(--accent-strong);
    }
    .stat-card strong {
      font-size: 1.35rem;
      color: var(--ink);
    }
    .verification-grid {
      grid-template-columns: 110px minmax(0, 1fr);
      align-items: start;
    }
    .verification-badge {
      width: 80px;
      height: 80px;
      border-radius: 999px;
      background: var(--accent-soft);
      border: 1px solid var(--accent-line);
      display: grid;
      place-items: center;
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
      font-size: 0.82rem;
      text-transform: uppercase;
      letter-spacing: 0.08em;
      color: var(--accent);
    }
    .book-nav {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 12px;
      padding: 10px 16px;
      width: 100%;
      border-radius: 10px;
      background: var(--surface);
      border: 1px solid var(--border);
    }
    .nav-button {
      padding: 8px 16px;
      border-radius: 6px;
      background: var(--surface-soft);
      color: var(--ink);
      border: 1px solid var(--border);
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
      font-size: 0.84rem;
    }
    .nav-button[disabled] {
      opacity: 0.48;
      cursor: default;
      transform: none;
    }
    .page-dots {
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      justify-content: center;
      gap: 10px;
    }
    .page-dot {
      width: 8px;
      height: 8px;
      border-radius: 999px;
      background: rgba(255, 255, 255, 0.15);
      border: none;
      padding: 0;
    }
    .page-dot.is-active {
      width: 24px;
      background: var(--accent);
    }
    .hero,
    .panel {
      padding: 28px;
    }
    .hero {
      margin-bottom: 20px;
    }
    .panel-head {
      margin-bottom: 18px;
    }
    .panel-body {
      display: grid;
      gap: 24px;
      align-items: start;
      grid-template-columns: 1fr;
    }
    .panel-copy {
      max-width: 68ch;
    }
    .panel-copy p:last-child {
      margin-bottom: 0;
    }
    .stack {
      display: grid;
      gap: 20px;
    }
    .diagram {
      margin-top: 0;
      border-radius: 8px;
      border: 1px solid var(--border);
      background: var(--surface-soft);
      padding: 16px;
    }
    .diagram-expandable .diagram-hitbox {
      display: block;
      width: 100%;
      border: 0;
      padding: 0;
      background: transparent;
      text-align: left;
      color: inherit;
      cursor: zoom-in;
    }
    .diagram-label {
      margin: 0 0 12px;
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
      font-size: 0.82rem;
      font-weight: 600;
      letter-spacing: 0.04em;
      text-transform: uppercase;
      color: var(--accent-strong);
    }
    .diagram-stage {
      display: block;
    }
    .diagram svg {
      display: block;
      width: 100%;
      height: auto;
    }
    .diagram-hint {
      display: block;
      margin-top: 12px;
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
      font-size: 0.8rem;
      text-transform: uppercase;
      letter-spacing: 0.04em;
      color: var(--accent);
    }
    .diagram-modal[hidden] {
      display: none;
    }
    .diagram-modal {
      position: fixed;
      inset: 0;
      z-index: 40;
      display: grid;
      place-items: center;
      padding: 18px;
    }
    .diagram-modal-backdrop {
      position: absolute;
      inset: 0;
      border: 0;
      padding: 0;
      background: rgba(0, 0, 0, 0.6);
      cursor: pointer;
    }
    .diagram-modal-card {
      position: relative;
      z-index: 1;
      width: min(1240px, calc(100vw - 24px));
      max-height: calc(100vh - 24px);
      overflow: auto;
      border-radius: 10px;
      border: 1px solid var(--border);
      background: var(--surface-strong);
      box-shadow: 0 4px 24px rgba(0, 0, 0, 0.4);
      padding: 24px;
    }
    .diagram-modal-header {
      display: flex;
      align-items: start;
      justify-content: space-between;
      gap: 16px;
    }
    .diagram-modal-title {
      margin-bottom: 0;
    }
    .diagram-modal-body {
      margin-top: 18px;
    }
    .diagram-modal-figure {
      border-radius: 8px;
      border: 1px solid var(--border);
      background: var(--surface-soft);
      padding: 18px;
    }
    .diagram-modal-figure svg {
      display: block;
      width: 100%;
      height: auto;
      max-height: calc(100vh - 220px);
    }
    .diagram-modal-note {
      margin-top: 14px;
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
      font-size: 0.82rem;
      color: var(--accent-strong);
    }
    details {
      margin-top: 16px;
    }
    summary {
      cursor: pointer;
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
      color: var(--accent-strong);
      font-size: 0.88rem;
    }
    pre {
      margin: 18px 0 0;
      padding: 16px;
      border-radius: 6px;
      border: 1px solid var(--border);
      background: var(--code-bg);
      overflow-x: auto;
      color: #d0d0d0;
      font-size: 0.9rem;
      line-height: 1.5;
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    }
    .report-shell[data-layout="spread"] .summary-grid {
      grid-template-columns: minmax(0, 1.1fr) minmax(280px, 0.9fr);
    }
    .report-shell[data-layout="spread"] .page.has-diagram .page-grid,
    .report-shell[data-layout="spread"] .panel-has-diagram .panel-body {
      grid-template-columns: minmax(0, 1.06fr) minmax(320px, 0.94fr);
    }
    .report-shell[data-layout="spread"] .page.has-diagram .page-visual {
      position: sticky;
      top: 0;
    }
    .nav-button:hover:not([disabled]),
    .view-toggle:hover {
      background: var(--surface-elevated);
    }
    @media (max-width: 840px) {
      .report-bar {
        align-items: start;
      }
      .report-toolbar,
      .toolbar-cluster {
        justify-content: flex-start;
        align-items: flex-start;
      }
      .report-toolbar {
        width: 100%;
      }
      .report-title {
        max-width: none;
        white-space: normal;
      }
      .summary-grid,
      .page.has-diagram .page-grid,
      .verification-grid,
      .panel-has-diagram .panel-body {
        grid-template-columns: 1fr;
      }
      .page-status {
        align-items: flex-start;
      }
      .book-nav {
        flex-wrap: wrap;
        justify-content: center;
      }
      .report-toolbar {
        justify-content: space-between;
      }
      .book-shell {
        height: calc(100vh - 208px);
        min-height: 500px;
      }
    }
    @media (max-width: 560px) {
      .report-shell {
        padding: 16px 12px 48px;
      }
      .report-bar,
      .hero,
      .panel,
      .page {
        border-radius: 8px;
        padding: 16px;
      }
      .summary-title {
        max-width: none;
        font-size: clamp(1.6rem, 7vw, 2.2rem);
      }
      .summary-stats {
        grid-template-columns: 1fr 1fr;
      }
      .view-toggle {
        flex: 1;
        text-align: center;
      }
      .report-toolbar {
        gap: 12px;
      }
      .page-status {
        width: 100%;
      }
      .page-label {
        max-width: none;
      }
      .book-shell {
        height: calc(100vh - 198px);
        min-height: 460px;
      }
      .diagram-modal {
        padding: 10px;
      }
      .diagram-modal-card {
        width: calc(100vw - 20px);
        max-height: calc(100vh - 20px);
        padding: 18px;
      }
    }
    "#
}

fn html_script() -> &'static str {
    r#"
    (() => {
      const root = document.querySelector('[data-magellan-report]');
      if (!root) return;

      const views = {
        book: root.querySelector('[data-view="book"]'),
        overview: root.querySelector('[data-view="overview"]'),
      };
      const toggles = Array.from(root.querySelectorAll('[data-view-target]'));
      const track = root.querySelector('[data-book-track]');
      const pages = Array.from(root.querySelectorAll('[data-page]'));
      const dots = Array.from(root.querySelectorAll('[data-page-dot]'));
      const prev = root.querySelector('[data-prev-page]');
      const next = root.querySelector('[data-next-page]');
      const pageLabel = root.querySelector('[data-current-page-label]');
      const pageCounter = root.querySelector('[data-page-counter]');
      const modal = root.querySelector('[data-diagram-modal]');
      const modalBody = root.querySelector('[data-diagram-modal-body]');
      const modalTitle = root.querySelector('[data-diagram-modal-title]');
      const modalCloseButtons = Array.from(root.querySelectorAll('[data-diagram-close]'));
      const diagramTriggers = Array.from(root.querySelectorAll('[data-diagram-trigger]'));

      const state = { view: 'book', page: 0 };
      let lastTrigger = null;

      function setView(view) {
        state.view = view;
        Object.entries(views).forEach(([name, element]) => {
          if (!element) return;
          const active = name === view;
          element.hidden = !active;
          element.classList.toggle('is-active', active);
        });
        toggles.forEach((button) => {
          const active = button.dataset.viewTarget === view;
          button.classList.toggle('is-active', active);
          button.setAttribute('aria-pressed', String(active));
        });
      }

      function setPage(page) {
        const bounded = Math.max(0, Math.min(page, pages.length - 1));
        state.page = bounded;
        if (track) {
          track.style.transform = `translateX(-${bounded * 100}%)`;
        }
        pages.forEach((pageElement, index) => {
          pageElement.classList.toggle('is-current', index === bounded);
        });
        dots.forEach((dot, index) => {
          const active = index === bounded;
          dot.classList.toggle('is-active', active);
          dot.setAttribute('aria-current', String(active));
        });
        if (prev) prev.disabled = bounded === 0;
        if (next) next.disabled = bounded === pages.length - 1;
        if (pageLabel && pages[bounded]) {
          pageLabel.textContent = pages[bounded].dataset.pageTitle || `Page ${bounded + 1}`;
        }
        if (pageCounter) {
          pageCounter.textContent = `Page ${bounded + 1} / ${pages.length}`;
        }
      }

      function openDiagram(trigger) {
        if (!modal || !modalBody || !modalTitle) return;
        const templateId = trigger.dataset.diagramTemplateId;
        if (!templateId) return;
        const template = root.querySelector(`#${templateId}`);
        if (!(template instanceof HTMLTemplateElement)) return;

        modalTitle.textContent = trigger.dataset.diagramTitle || 'Diagram';
        modalBody.innerHTML = template.innerHTML;
        modal.hidden = false;
        document.body.setAttribute('data-diagram-modal-open', 'true');
        lastTrigger = trigger;
      }

      function closeDiagram() {
        if (!modal || modal.hidden) return;
        modal.hidden = true;
        if (modalBody) modalBody.innerHTML = '';
        document.body.removeAttribute('data-diagram-modal-open');
        if (lastTrigger instanceof HTMLElement) {
          lastTrigger.focus();
        }
      }

      toggles.forEach((button) => {
        button.addEventListener('click', () => setView(button.dataset.viewTarget || 'book'));
      });
      diagramTriggers.forEach((trigger) => {
        trigger.addEventListener('click', () => openDiagram(trigger));
      });
      modalCloseButtons.forEach((button) => {
        button.addEventListener('click', closeDiagram);
      });
      dots.forEach((dot, index) => {
        dot.addEventListener('click', () => {
          setView('book');
          setPage(index);
        });
      });
      if (prev) prev.addEventListener('click', () => setPage(state.page - 1));
      if (next) next.addEventListener('click', () => setPage(state.page + 1));
      window.addEventListener('keydown', (event) => {
        if (modal && !modal.hidden && event.key === 'Escape') {
          closeDiagram();
          return;
        }
        if (modal && !modal.hidden) return;
        if (state.view !== 'book') return;
        if (event.key === 'ArrowRight') setPage(state.page + 1);
        if (event.key === 'ArrowLeft') setPage(state.page - 1);
      });

      setView('book');
      setPage(0);
    })();
    "#
}

fn render_diagram_html(index: usize, diagram: &Diagram, expandable: bool) -> String {
    let svg = render_svg_diagram(index, diagram);
    let ascii = escape_html(&render_ascii_diagram(diagram));
    let title = diagram_title(diagram);

    if expandable {
        let template_id = format!("diagram-template-{index}");
        let modal_svg = render_svg_diagram(index + 10_000, diagram);

        return format!(
            "<div class=\"diagram diagram-expandable\">
              <button class=\"diagram-hitbox\" type=\"button\" data-diagram-trigger data-diagram-template-id=\"{template_id}\" data-diagram-title=\"{title}\" aria-label=\"Expand {title}\">
                <span class=\"diagram-label\">{title}</span>
                <span class=\"diagram-stage\">{svg}</span>
                <span class=\"diagram-hint\">Click to enlarge</span>
              </button>
              <details><summary>ASCII fallback</summary><pre>{ascii}</pre></details>
              <template id=\"{template_id}\">
                <div class=\"diagram-modal-figure\">{modal_svg}</div>
                <p class=\"diagram-modal-note\">Press Escape or click outside the diagram to close.</p>
              </template>
            </div>",
            template_id = template_id,
            title = title,
            svg = svg,
            ascii = ascii,
            modal_svg = modal_svg
        );
    }

    format!(
        "<div class=\"diagram\"><p class=\"diagram-label\">{}</p>{}<details><summary>ASCII fallback</summary><pre>{}</pre></details></div>",
        title, svg, ascii
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
    let rows = nodes.len().div_ceil(columns).max(1);
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
      fill: #1e1e1e;
      stroke: rgba(126, 184, 255, 0.4);
      stroke-width: 1;
    }}
    .lane {{
      stroke: rgba(255, 255, 255, 0.12);
      stroke-width: 1;
      stroke-dasharray: 5 5;
    }}
    .connector {{
      stroke: rgba(126, 184, 255, 0.6);
      stroke-width: 1.5;
      fill: none;
    }}
    .timeline-axis {{
      stroke: rgba(255, 255, 255, 0.15);
      stroke-width: 2;
    }}
    .timeline-dot {{
      fill: rgba(126, 184, 255, 0.8);
      stroke: rgba(126, 184, 255, 0.15);
      stroke-width: 4;
    }}
    .node-copy, .edge-copy, .event-copy, .event-label, .bullet-copy {{
      fill: #e0e0e0;
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    }}
    .node-copy {{
      font-size: 13px;
      font-weight: 600;
    }}
    .edge-copy {{
      font-size: 11px;
      font-weight: 500;
      fill: #999999;
    }}
    .event-label {{
      font-size: 13px;
      font-weight: 700;
    }}
    .event-copy, .bullet-copy {{
      font-size: 12px;
      fill: #aaaaaa;
    }}
  </style>
  <defs>
    <marker id=\"{marker_id}\" viewBox=\"0 0 10 10\" refX=\"8\" refY=\"5\" markerWidth=\"7\" markerHeight=\"7\" orient=\"auto-start-reverse\">
      <path d=\"M 0 0 L 10 5 L 0 10 z\" fill=\"rgba(126, 184, 255, 0.6)\" />
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
            "<circle cx=\"{}\" cy=\"{}\" r=\"2.6\" fill=\"rgba(99, 214, 198, 0.86)\"/>",
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

    fn css_block<'a>(html: &'a str, selector: &str) -> &'a str {
        let marker = format!("{selector} {{");
        let start = html
            .find(&marker)
            .unwrap_or_else(|| panic!("missing CSS block for {selector}"));
        let rest = &html[start..];
        let end = rest
            .find("\n    }")
            .unwrap_or_else(|| panic!("unterminated CSS block for {selector}"));
        &rest[..end]
    }

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
        let book_nav_css = css_block(&rendered, ".book-nav");

        assert!(rendered.contains("<!DOCTYPE html>"));
        assert!(rendered.contains("Magellan walkthrough"));
        assert!(rendered.contains("Book View"));
        assert!(rendered.contains("Overview"));
        assert!(!rendered.contains("Reader"));
        assert!(rendered.contains("data-view=\"book\""));
        assert!(rendered.contains("data-view=\"overview\" hidden"));
        assert!(rendered.contains("data-layout=\"spread\""));
        assert!(rendered.contains("class=\"report-title\""));
        assert!(rendered.contains("class=\"summary-title\""));
        assert!(rendered.contains("data-current-page-label"));
        assert!(rendered.contains("data-book-track"));
        assert!(rendered.contains("Page 1 / 3"));
        assert!(rendered.contains("data-diagram-modal"));
        assert!(rendered.contains("data-diagram-trigger"));
        assert!(rendered.contains("Click to enlarge"));
        assert!(rendered.contains("<link rel=\"icon\" href=\"data:,\">"));
        assert!(rendered.contains("<svg viewBox="));
        assert!(rendered.contains("ASCII fallback"));
        assert!(rendered.contains("color-scheme: dark;"));
        assert!(!rendered.contains("color-scheme: light;"));
        assert!(!rendered.contains("cdn.jsdelivr"));
        assert!(book_nav_css.contains("width: 100%;"));
        assert!(!book_nav_css.contains("position: fixed;"));
        assert!(!book_nav_css.contains("position: sticky;"));
    }

    #[test]
    fn html_output_is_self_contained_for_all_examples() {
        for preset in [
            ExamplePreset::Walkthrough,
            ExamplePreset::Timeline,
            ExamplePreset::BeforeAfter,
            ExamplePreset::Followup,
        ] {
            let rendered = render_document(&example_document(preset), OutputFormat::Html);
            assert!(rendered.contains("<svg viewBox="));
            assert!(!rendered.contains("https://"));
            assert!(!rendered.contains("cdn.jsdelivr"));
        }
    }
}
