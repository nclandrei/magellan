use std::collections::HashSet;
use std::fmt::Write;

use anyhow::{Context, Result};
use schemars::schema_for;

use crate::model::{
    BeforeAfterDiagram, Cardinality, Diagram, Document, Edge, Entity, Relationship, Section,
    TimelineEvent, TreeNode,
};

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
        if let Some(sha) = &section.commit {
            writeln!(&mut output, "commit: {sha}").unwrap();
        }
        if !section.files.is_empty() {
            writeln!(&mut output, "files: {}", section.files.join(", ")).unwrap();
        }
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
        if section.commit.is_some() || !section.files.is_empty() {
            let mut parts = Vec::new();
            if let Some(sha) = &section.commit {
                parts.push(format!("`{sha}`"));
            }
            for file in &section.files {
                parts.push(format!("`{file}`"));
            }
            writeln!(&mut output, "{}", parts.join(" ")).unwrap();
            writeln!(&mut output).unwrap();
        }
        for paragraph in &section.text {
            writeln!(&mut output, "{paragraph}").unwrap();
            writeln!(&mut output).unwrap();
        }

        if let Some(diagram) = &section.diagram {
            match diagram {
                Diagram::Table { headers, rows } => {
                    writeln!(&mut output, "{}", render_markdown_table(headers, rows)).unwrap();
                    writeln!(&mut output).unwrap();
                }
                _ => {
                    writeln!(&mut output, "```mermaid").unwrap();
                    writeln!(&mut output, "{}", render_mermaid_diagram(diagram)).unwrap();
                    writeln!(&mut output, "```").unwrap();
                    writeln!(&mut output).unwrap();
                }
            }
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
    let toc_html = render_toc(document);
    let repo = document.repo.as_deref();
    let sections_html = document
        .sections
        .iter()
        .enumerate()
        .map(|(index, section)| render_section_html(index, section, repo))
        .collect::<Vec<_>>()
        .join("\n");
    let verification_html = document
        .verification
        .as_ref()
        .map(render_verification_html)
        .unwrap_or_default();
    let verification_toc = if document.verification.is_some() {
        "\n    <a class=\"toc-link\" href=\"#verification\">Verification</a>"
    } else {
        ""
    };

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
  <nav class=\"sidebar\" data-sidebar>
    <div class=\"sidebar-header\">
      <p class=\"eyebrow\">Contents</p>
      <button class=\"theme-toggle\" type=\"button\" data-theme-toggle aria-label=\"Toggle light/dark mode\"><span class=\"theme-icon-sun\">☀️</span><span class=\"theme-icon-moon\">🌙</span></button>
    </div>
    <a class=\"toc-link is-active\" href=\"#summary\">Summary</a>
{toc_html}{verification_toc}
    <button class=\"sidebar-close\" type=\"button\" data-sidebar-close aria-label=\"Close sidebar\">Close</button>
  </nav>
  <button class=\"hamburger\" type=\"button\" data-sidebar-open aria-label=\"Open sidebar\">Menu</button>
  <main class=\"content\">
    <header class=\"hero\" id=\"summary\">
      <p class=\"eyebrow\">Magellan walkthrough</p>
      <h1>{title}</h1>
      {summary_html}
      <p class=\"section-count\">{section_count} sections</p>
    </header>
{sections_html}
{verification_html}
  </main>
  <div class=\"lightbox\" data-lightbox hidden>
    <button class=\"lightbox-close\" type=\"button\" data-lightbox-close aria-label=\"Close enlarged diagram\">&times;</button>
    <div class=\"lightbox-body\" data-lightbox-body></div>
  </div>
  <script>{script}</script>
</body>
</html>",
        title = escape_html(&document.title),
        summary_html = summary_html,
        toc_html = toc_html,
        verification_toc = verification_toc,
        section_count = document.sections.len(),
        sections_html = sections_html,
        verification_html = verification_html,
        style = html_style(),
        script = html_script()
    )
}

fn render_toc(document: &Document) -> String {
    document
        .sections
        .iter()
        .enumerate()
        .map(|(index, section)| {
            format!(
                "    <a class=\"toc-link\" href=\"#section-{}\">{}</a>",
                index + 1,
                escape_html(&section.title)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_section_html(index: usize, section: &Section, repo: Option<&str>) -> String {
    let diagram_html = section
        .diagram
        .as_ref()
        .map(|diagram| render_diagram_html(index, diagram))
        .unwrap_or_default();

    let meta_html = render_section_meta(section, repo);

    format!(
        "    <section class=\"section\" id=\"section-{number}\">
      <div class=\"section-head\">
        <p class=\"eyebrow\">Step {number}</p>
        <h2>{title}</h2>
        {meta_html}
      </div>
      <div class=\"section-body\">
        {text_html}
        {diagram_html}
      </div>
    </section>",
        number = index + 1,
        title = escape_html(&section.title),
        meta_html = meta_html,
        text_html = paragraphs_to_html(&section.text),
        diagram_html = diagram_html
    )
}

fn render_section_meta(section: &Section, repo: Option<&str>) -> String {
    let has_commit = section.commit.is_some();
    let has_files = !section.files.is_empty();

    if !has_commit && !has_files {
        return String::new();
    }

    let repo_base = repo.map(|r| r.trim_end_matches('/'));

    let mut chips = String::new();

    if let Some(sha) = &section.commit {
        let short = if sha.len() > 8 { &sha[..8] } else { sha };
        if let Some(base) = repo_base {
            write!(
                &mut chips,
                "<a class=\"meta-chip\" href=\"{base}/commit/{sha}\" target=\"_blank\" rel=\"noopener\">{short}</a>",
                base = escape_html(base),
                sha = escape_html(sha),
                short = escape_html(short),
            )
            .unwrap();
        } else {
            write!(
                &mut chips,
                "<span class=\"meta-chip\">{short}</span>",
                short = escape_html(short),
            )
            .unwrap();
        }
    }

    for file in &section.files {
        if let Some(base) = repo_base {
            let blob_ref = section.commit.as_deref().unwrap_or("HEAD");
            write!(
                &mut chips,
                "<a class=\"meta-chip\" href=\"{base}/blob/{blob_ref}/{file}\" target=\"_blank\" rel=\"noopener\">{file}</a>",
                base = escape_html(base),
                blob_ref = escape_html(blob_ref),
                file = escape_html(file),
            )
            .unwrap();
        } else {
            write!(
                &mut chips,
                "<span class=\"meta-chip\">{file}</span>",
                file = escape_html(file),
            )
            .unwrap();
        }
    }

    format!("<div class=\"section-meta\">{chips}</div>")
}

fn render_verification_html(verification: &crate::model::Verification) -> String {
    format!(
        "    <section class=\"section verification\" id=\"verification\">
      <div class=\"section-head\">
        <p class=\"eyebrow\">Verification</p>
        <h2>Verification</h2>
      </div>
      <div class=\"section-body\">
        {text_html}
      </div>
    </section>",
        text_html = paragraphs_to_html(&verification.text)
    )
}

fn html_style() -> &'static str {
    r#"
    :root {
      color-scheme: dark;
      --bg: #131211;
      --surface: #1b1a18;
      --ink: #d9d5d0;
      --ink-soft: #b5b0a9;
      --muted: #8a847d;
      --accent: #a09890;
      --accent-strong: #c0b8b0;
      --border: rgba(255, 255, 255, 0.08);
      --shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
      --code-bg: #161514;
      --diagram-node-fill: #1b1a18;
      --diagram-stroke: rgba(160, 152, 144, 0.4);
      --diagram-lane: rgba(255, 255, 255, 0.12);
      --diagram-dot: rgba(160, 152, 144, 0.8);
      --diagram-dot-ring: rgba(160, 152, 144, 0.15);
      --diagram-text: #d9d5d0;
      --diagram-text-muted: #a09890;
      --sidebar-width: 240px;
    }
    [data-theme="light"] {
      color-scheme: light;
      --bg: #f5f3f0;
      --surface: #ffffff;
      --ink: #2a2725;
      --ink-soft: #4a4541;
      --muted: #7a736c;
      --accent: #7a736c;
      --accent-strong: #5a534c;
      --border: rgba(0, 0, 0, 0.1);
      --shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
      --code-bg: #edebe8;
      --diagram-node-fill: #ffffff;
      --diagram-stroke: rgba(90, 83, 76, 0.4);
      --diagram-lane: rgba(0, 0, 0, 0.1);
      --diagram-dot: rgba(90, 83, 76, 0.8);
      --diagram-dot-ring: rgba(90, 83, 76, 0.15);
      --diagram-text: #2a2725;
      --diagram-text-muted: #7a736c;
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
      display: flex;
      min-height: 100vh;
    }
    .sidebar {
      position: fixed;
      top: 0;
      left: 0;
      width: var(--sidebar-width);
      height: 100vh;
      overflow-y: auto;
      padding: 24px 16px;
      background: var(--surface);
      border-right: 1px solid var(--border);
      display: flex;
      flex-direction: column;
      gap: 4px;
      z-index: 10;
    }
    .sidebar-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      margin-bottom: 12px;
    }
    .sidebar-header .eyebrow {
      margin: 0;
    }
    .sidebar-close {
      display: none;
      appearance: none;
      border: 1px solid var(--border);
      background: var(--surface);
      color: var(--muted);
      border-radius: 6px;
      padding: 4px 10px;
      font-size: 0.82rem;
      cursor: pointer;
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    }
    .theme-toggle {
      appearance: none;
      border: 1px solid var(--border);
      background: var(--surface);
      color: var(--muted);
      border-radius: 6px;
      padding: 4px 10px;
      font-size: 0.78rem;
      cursor: pointer;
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
      transition: background-color 120ms ease, color 120ms ease;
    }
    .theme-toggle:hover {
      color: var(--ink);
      background: var(--bg);
    }
    .theme-icon-moon { display: none; }
    [data-theme="light"] .theme-icon-sun { display: none; }
    [data-theme="light"] .theme-icon-moon { display: inline; }
    .toc-link {
      display: block;
      padding: 6px 10px;
      border-radius: 6px;
      color: var(--muted);
      text-decoration: none;
      font-size: 0.88rem;
      line-height: 1.4;
      transition: background-color 120ms ease, color 120ms ease;
    }
    .toc-link:hover {
      color: var(--ink);
      background: var(--bg);
    }
    .toc-link.is-active {
      color: var(--ink);
      background: var(--bg);
      font-weight: 600;
    }
    .hamburger {
      display: none;
      position: fixed;
      top: 16px;
      left: 16px;
      z-index: 20;
      appearance: none;
      border: 1px solid var(--border);
      background: var(--surface);
      color: var(--ink);
      border-radius: 8px;
      padding: 8px 14px;
      font-size: 0.88rem;
      cursor: pointer;
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
      box-shadow: var(--shadow);
    }
    .content {
      margin-left: var(--sidebar-width);
      flex: 1;
      max-width: 1080px;
      padding: 32px 36px 72px;
    }
    .hero {
      margin-bottom: 32px;
    }
    .section-count {
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
      font-size: 0.82rem;
      color: var(--accent);
      text-transform: uppercase;
      letter-spacing: 0.06em;
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
      line-height: 1.1;
      font-weight: 700;
      color: var(--ink);
    }
    h1 {
      font-size: clamp(1.6rem, 3vw, 2.2rem);
      text-wrap: balance;
    }
    h2 {
      font-size: clamp(1.2rem, 2vw, 1.5rem);
      text-wrap: balance;
    }
    p {
      margin: 0 0 14px;
      color: var(--ink-soft);
      font-size: 1rem;
      line-height: 1.7;
    }
    p code {
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
      font-size: 0.92em;
      background: var(--code-bg);
      border: 1px solid var(--border);
      border-radius: 4px;
      padding: 1px 6px;
      color: var(--ink);
    }
    p a {
      color: var(--ink);
      text-decoration: underline;
      text-decoration-color: var(--accent);
      text-underline-offset: 2px;
    }
    p a:hover {
      text-decoration-color: var(--accent-strong);
    }
    .section {
      padding: 28px 0;
      border-top: 1px solid var(--border);
    }
    .section-head {
      margin-bottom: 16px;
    }
    .section-meta {
      display: flex;
      flex-wrap: wrap;
      gap: 6px;
      margin-top: 8px;
    }
    .meta-chip {
      display: inline-block;
      padding: 2px 8px;
      border-radius: 4px;
      border: 1px solid var(--border);
      background: var(--surface);
      color: var(--muted);
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
      font-size: 0.78rem;
      line-height: 1.5;
      text-decoration: none;
      transition: color 120ms ease, border-color 120ms ease;
    }
    a.meta-chip:hover {
      color: var(--ink);
      border-color: var(--accent);
    }
    .section-body > p {
      max-width: 64ch;
    }
    .section-body p:last-child {
      margin-bottom: 0;
    }
    .verification {
      background: var(--surface);
      border: 1px solid var(--border);
      border-radius: 10px;
      padding: 28px;
      margin-top: 12px;
    }
    .diagram {
      margin-top: 20px;
      border-radius: 8px;
      border: 1px solid var(--border);
      background: var(--surface);
      padding: 16px;
      cursor: zoom-in;
      width: 100%;
      max-width: none;
    }
    .lightbox[hidden] {
      display: none;
    }
    .lightbox {
      position: fixed;
      inset: 0;
      z-index: 50;
      background: rgba(0, 0, 0, 0.7);
      display: grid;
      place-items: center;
      padding: 24px;
      cursor: zoom-out;
    }
    .lightbox-close {
      position: absolute;
      top: 16px;
      right: 20px;
      appearance: none;
      border: 1px solid var(--border);
      background: var(--surface);
      color: var(--ink);
      border-radius: 8px;
      width: 40px;
      height: 40px;
      font-size: 1.4rem;
      cursor: pointer;
      display: grid;
      place-items: center;
      z-index: 1;
    }
    .lightbox-body {
      width: min(1200px, calc(100vw - 48px));
      max-height: calc(100vh - 48px);
      overflow: auto;
      border-radius: 10px;
      border: 1px solid var(--border);
      background: var(--surface);
      padding: 24px;
    }
    .lightbox-body svg {
      display: block;
      width: 100%;
      height: auto;
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
      min-height: 260px;
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
      color: var(--ink-soft);
      font-size: 0.9rem;
      line-height: 1.5;
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    }
    @media (max-width: 840px) {
      .sidebar {
        transform: translateX(-100%);
        transition: transform 200ms ease;
        box-shadow: none;
      }
      .sidebar.is-open {
        transform: translateX(0);
        box-shadow: 4px 0 24px rgba(0, 0, 0, 0.3);
      }
      .sidebar-close {
        display: block;
        margin-top: auto;
      }
      .hamburger {
        display: block;
      }
      .content {
        margin-left: 0;
        padding: 24px 20px 48px;
      }
    }
    @media (max-width: 560px) {
      .content {
        padding: 20px 14px 40px;
      }
    }
    "#
}

fn html_script() -> &'static str {
    r#"
    (() => {
      const sidebar = document.querySelector('[data-sidebar]');
      const openBtn = document.querySelector('[data-sidebar-open]');
      const closeBtn = document.querySelector('[data-sidebar-close]');
      const themeBtn = document.querySelector('[data-theme-toggle]');
      const tocLinks = Array.from(document.querySelectorAll('.toc-link'));
      const sections = Array.from(document.querySelectorAll('.hero, .section'));

      const stored = localStorage.getItem('magellan-theme');
      if (stored === 'light') document.documentElement.setAttribute('data-theme', 'light');

      function updateThemeIcon() {
        if (!themeBtn) return;
        const isLight = document.documentElement.getAttribute('data-theme') === 'light';
        const sun = themeBtn.querySelector('.theme-icon-sun');
        const moon = themeBtn.querySelector('.theme-icon-moon');
        if (sun) sun.style.display = isLight ? 'none' : 'inline';
        if (moon) moon.style.display = isLight ? 'inline' : 'none';
      }
      function toggleTheme() {
        const isLight = document.documentElement.getAttribute('data-theme') === 'light';
        if (isLight) {
          document.documentElement.removeAttribute('data-theme');
          localStorage.setItem('magellan-theme', 'dark');
        } else {
          document.documentElement.setAttribute('data-theme', 'light');
          localStorage.setItem('magellan-theme', 'light');
        }
        updateThemeIcon();
      }
      if (themeBtn) themeBtn.addEventListener('click', toggleTheme);
      updateThemeIcon();

      if (openBtn) openBtn.addEventListener('click', () => sidebar && sidebar.classList.add('is-open'));
      if (closeBtn) closeBtn.addEventListener('click', () => sidebar && sidebar.classList.remove('is-open'));

      tocLinks.forEach(link => {
        link.addEventListener('click', () => {
          if (sidebar && window.innerWidth <= 840) sidebar.classList.remove('is-open');
        });
      });

      // Lightbox
      const lightbox = document.querySelector('[data-lightbox]');
      const lightboxBody = document.querySelector('[data-lightbox-body]');
      const lightboxClose = document.querySelector('[data-lightbox-close]');
      const diagrams = Array.from(document.querySelectorAll('.diagram'));

      function openLightbox(diagram) {
        if (!lightbox || !lightboxBody) return;
        const svg = diagram.querySelector('svg');
        if (!svg) return;
        lightboxBody.innerHTML = svg.outerHTML;
        lightbox.hidden = false;
        document.body.style.overflow = 'hidden';
      }
      function closeLightbox() {
        if (!lightbox) return;
        lightbox.hidden = true;
        if (lightboxBody) lightboxBody.innerHTML = '';
        document.body.style.overflow = '';
      }
      diagrams.forEach(d => d.addEventListener('click', (e) => {
        if (e.target.closest('details') || e.target.closest('summary')) return;
        openLightbox(d);
      }));
      if (lightboxClose) lightboxClose.addEventListener('click', closeLightbox);
      if (lightbox) lightbox.addEventListener('click', (e) => {
        if (e.target === lightbox) closeLightbox();
      });
      window.addEventListener('keydown', (e) => {
        if (lightbox && !lightbox.hidden && e.key === 'Escape') closeLightbox();
      });

      const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
          if (entry.isIntersecting) {
            const id = entry.target.id;
            tocLinks.forEach(link => {
              link.classList.toggle('is-active', link.getAttribute('href') === '#' + id);
            });
          }
        });
      }, { rootMargin: '-20% 0px -60% 0px' });

      sections.forEach(section => {
        if (section.id) observer.observe(section);
      });
    })();
    "#
}

fn render_diagram_html(index: usize, diagram: &Diagram) -> String {
    let svg = render_svg_diagram(index, diagram);
    let ascii = escape_html(&render_ascii_diagram(diagram));
    let title = diagram_title(diagram);

    format!(
        "<figure class=\"diagram\">
          <p class=\"diagram-label\">{title}</p>
          <div class=\"diagram-stage\">{svg}</div>
          <details><summary>ASCII fallback</summary><pre>{ascii}</pre></details>
        </figure>",
        title = title,
        svg = svg,
        ascii = ascii
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
        Diagram::LayerStack { layers } => render_layer_stack_svg(&diagram_id, layers),
        Diagram::StateMachine {
            states,
            transitions,
        } => render_state_machine_svg(&diagram_id, states, transitions),
        Diagram::Table { headers, rows } => render_table_svg(&diagram_id, headers, rows),
        Diagram::DependencyTree { root, children } => {
            render_dependency_tree_svg(&diagram_id, root, children)
        }
        Diagram::EntityRelationship {
            entities,
            relationships,
        } => render_entity_relationship_svg(&diagram_id, entities, relationships),
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

fn render_layer_stack_svg(id: &str, layers: &[String]) -> String {
    let padding = 24;
    let width = 520;
    let layer_height = 48;
    let gap = 6;
    let layer_width = width - padding * 2;
    let height = padding * 2
        + layers.len() as i32 * layer_height
        + (layers.len().saturating_sub(1)) as i32 * gap;
    let marker_id = format!("{id}-arrow");
    let mut body = String::new();

    for (index, layer) in layers.iter().enumerate() {
        let y = padding + index as i32 * (layer_height + gap);
        let center_x = padding + layer_width / 2;
        let center_y = y + layer_height / 2 + 5;

        write!(
            &mut body,
            "<rect class=\"node\" x=\"{padding}\" y=\"{y}\" width=\"{layer_width}\" height=\"{layer_height}\" rx=\"8\" ry=\"8\"/>"
        )
        .unwrap();
        write_multiline_svg_text(
            &mut body,
            center_x,
            center_y,
            &wrap_text(layer, 40),
            "middle",
            "node-copy",
        );
    }

    svg_shell(id, width, height.max(120), &marker_id, "Layer stack", &body)
}

fn render_state_machine_svg(id: &str, states: &[String], transitions: &[Edge]) -> String {
    let ordered = ordered_nodes(states, transitions);
    let wrapped_states = ordered
        .iter()
        .map(|state| wrap_text(state, 16))
        .collect::<Vec<_>>();

    let columns = ordered.len().clamp(1, 3);
    let rows = ordered.len().div_ceil(columns).max(1);
    let box_width = wrapped_states
        .iter()
        .flat_map(|lines| lines.iter())
        .map(|line| estimate_text_width(line, 124, 200))
        .max()
        .unwrap_or(124);
    let box_height = wrapped_states
        .iter()
        .map(|lines| 28 + (lines.len() as i32 * 16))
        .max()
        .unwrap_or(60);

    let padding = 36;
    let gap_x = 64;
    let gap_y = 82;
    // Extra left padding so the start marker fits in front of the first state.
    let left_inset = 28;

    let width = padding * 2
        + left_inset
        + columns as i32 * box_width
        + (columns.saturating_sub(1)) as i32 * gap_x;
    let height = padding * 2 + rows as i32 * box_height + (rows.saturating_sub(1)) as i32 * gap_y;
    let marker_id = format!("{id}-arrow");

    let positions = ordered
        .iter()
        .enumerate()
        .map(|(index, state)| {
            let row = index / columns;
            let col = index % columns;
            let x = padding + left_inset + col as i32 * (box_width + gap_x);
            let y = padding + row as i32 * (box_height + gap_y);
            (state.as_str(), (x, y))
        })
        .collect::<Vec<_>>();

    let mut body = String::new();

    // Start marker: a small filled circle connected to the first state.
    if let Some((_, (first_x, first_y))) = positions.first() {
        let cx = first_x - 18;
        let cy = first_y + box_height / 2;
        write!(
            &mut body,
            "<circle class=\"state-start\" cx=\"{cx}\" cy=\"{cy}\" r=\"5\"/>"
        )
        .unwrap();
        write!(
            &mut body,
            "<line class=\"connector\" x1=\"{}\" y1=\"{cy}\" x2=\"{}\" y2=\"{cy}\" marker-end=\"url(#{marker_id})\"/>",
            cx + 5,
            first_x
        )
        .unwrap();
    }

    for edge in transitions {
        let Some((from_x, from_y)) = positions
            .iter()
            .find(|(node, _)| *node == edge.from.as_str())
            .map(|(_, position)| *position)
        else {
            continue;
        };

        if edge.from == edge.to {
            // Self-loop: draw a small arc above the state box.
            let center_x = from_x + box_width / 2;
            let top_y = from_y;
            let loop_radius = 18;
            let loop_left = center_x - loop_radius;
            let loop_right = center_x + loop_radius;
            let loop_top = top_y - 26;
            write!(
                &mut body,
                "<path class=\"self-loop\" d=\"M {loop_left} {top_y} C {loop_left} {loop_top}, {loop_right} {loop_top}, {loop_right} {top_y}\" fill=\"none\" marker-end=\"url(#{marker_id})\"/>"
            )
            .unwrap();

            if let Some(label) = &edge.label {
                write_multiline_svg_text(
                    &mut body,
                    center_x,
                    loop_top - 2,
                    &wrap_text(label, 14),
                    "middle",
                    "edge-copy",
                );
            }
            continue;
        }

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

    for ((_, (x, y)), lines) in positions.iter().zip(wrapped_states.iter()) {
        let center_x = *x + box_width / 2;
        write!(
            &mut body,
            "<rect class=\"state-node\" x=\"{}\" y=\"{}\" width=\"{box_width}\" height=\"{box_height}\" rx=\"22\" ry=\"22\"/>",
            x, y
        )
        .unwrap();
        write_multiline_svg_text(&mut body, center_x, *y + 28, lines, "middle", "node-copy");
    }

    svg_shell(
        id,
        width.max(360),
        height.max(200),
        &marker_id,
        "State machine",
        &body,
    )
}

fn render_dependency_tree_svg(id: &str, root: &str, children: &[TreeNode]) -> String {
    let padding = 24;
    let row_height = 32;
    let indent_width = 28;
    let marker_id = format!("{id}-arrow");

    // Flatten tree into positioned rows
    struct FlatRow {
        label: String,
        depth: i32,
    }
    fn flatten(node_children: &[TreeNode], depth: i32, rows: &mut Vec<FlatRow>) {
        for child in node_children {
            rows.push(FlatRow {
                label: child.label.clone(),
                depth,
            });
            flatten(&child.children, depth + 1, rows);
        }
    }
    let mut rows = vec![FlatRow {
        label: root.to_string(),
        depth: 0,
    }];
    flatten(children, 1, &mut rows);

    let max_depth = rows.iter().map(|r| r.depth).max().unwrap_or(0);
    let width = padding * 2 + max_depth * indent_width + 300;
    let height = padding * 2 + rows.len() as i32 * row_height;
    let mut body = String::new();

    for (index, row) in rows.iter().enumerate() {
        let x = padding + row.depth * indent_width;
        let y = padding + index as i32 * row_height;

        // Draw a small dot for non-root nodes
        if row.depth > 0 {
            write!(
                &mut body,
                "<circle class=\"timeline-dot\" cx=\"{}\" cy=\"{}\" r=\"4\" style=\"stroke-width:2\"/>",
                x + 4,
                y + 12
            )
            .unwrap();
        }

        let text_x = if row.depth > 0 { x + 16 } else { x };
        let class = if row.depth == 0 {
            "event-label"
        } else {
            "event-copy"
        };
        write_multiline_svg_text(
            &mut body,
            text_x,
            y + 16,
            &wrap_text(&row.label, 30),
            "start",
            class,
        );
    }

    svg_shell(
        id,
        width.max(320),
        height.max(120),
        &marker_id,
        "Dependency tree",
        &body,
    )
}

fn render_table_svg(id: &str, headers: &[String], rows: &[Vec<String>]) -> String {
    let padding = 24;
    let line_height = 16;
    let cell_pad_y = 12;
    let header_height = 40;
    let col_count = headers.len();
    let min_col_width: i32 = 120;
    let max_col_width: i32 = 300;
    let col_pad: i32 = 24;
    let px_per_char: f32 = 7.2;

    // Compute per-column widths based on content
    let col_widths: Vec<i32> = (0..col_count)
        .map(|col| {
            let mut max_chars = headers[col].len();
            for row in rows {
                if let Some(cell) = row.get(col) {
                    max_chars = max_chars.max(cell.len());
                }
            }
            let natural = (max_chars as f32 * px_per_char).ceil() as i32 + col_pad;
            natural.clamp(min_col_width, max_col_width)
        })
        .collect();

    // Max characters that fit in each column (for wrapping)
    let col_max_chars: Vec<usize> = col_widths
        .iter()
        .map(|w| ((*w - col_pad) as f32 / px_per_char).floor().max(8.0) as usize)
        .collect();

    // Wrap cell text and compute per-row heights
    let wrapped_rows: Vec<Vec<Vec<String>>> = rows
        .iter()
        .map(|row| {
            (0..col_count)
                .map(|col| {
                    let text = row.get(col).map(|s| s.as_str()).unwrap_or("");
                    let lines = wrap_text(text, col_max_chars[col]);
                    if lines.is_empty() {
                        vec![String::new()]
                    } else {
                        lines
                    }
                })
                .collect()
        })
        .collect();

    let row_heights: Vec<i32> = wrapped_rows
        .iter()
        .map(|row| {
            let max_lines = row
                .iter()
                .map(|lines| lines.len() as i32)
                .max()
                .unwrap_or(1);
            (max_lines * line_height + cell_pad_y).max(36)
        })
        .collect();

    // Layout
    let table_width: i32 = col_widths.iter().sum();
    let table_height: i32 = header_height + row_heights.iter().sum::<i32>();
    let width = padding * 2 + table_width;
    let height = padding * 2 + table_height + 2;
    let marker_id = format!("{id}-arrow");
    let mut body = String::new();

    let col_x_offsets: Vec<i32> = col_widths
        .iter()
        .scan(0, |acc, &w| {
            let offset = *acc;
            *acc += w;
            Some(offset)
        })
        .collect();

    // Table outline
    write!(
        &mut body,
        "<rect class=\"panel-box\" x=\"{padding}\" y=\"{padding}\" width=\"{table_width}\" height=\"{table_height}\" rx=\"6\" ry=\"6\"/>"
    )
    .unwrap();

    // Header separator line
    let sep_y = padding + header_height;
    write!(
        &mut body,
        "<line class=\"connector\" x1=\"{padding}\" y1=\"{sep_y}\" x2=\"{}\" y2=\"{sep_y}\" style=\"stroke-width:1\"/>",
        padding + table_width
    )
    .unwrap();

    // Column separator lines
    for offset in col_x_offsets.iter().skip(1) {
        let x = padding + offset;
        write!(
            &mut body,
            "<line class=\"lane\" x1=\"{x}\" y1=\"{padding}\" x2=\"{x}\" y2=\"{}\"/>",
            padding + table_height
        )
        .unwrap();
    }

    // Header text
    for (col, header) in headers.iter().enumerate() {
        let x = padding + col_x_offsets[col] + col_widths[col] / 2;
        let y = padding + header_height / 2 + 5;
        write_multiline_svg_text(
            &mut body,
            x,
            y,
            std::slice::from_ref(header),
            "middle",
            "event-label",
        );
    }

    // Data rows
    let mut row_y = padding + header_height;
    for (row_index, wrapped_row) in wrapped_rows.iter().enumerate() {
        if row_index > 0 {
            write!(
                &mut body,
                "<line class=\"lane\" x1=\"{padding}\" y1=\"{row_y}\" x2=\"{}\" y2=\"{row_y}\"/>",
                padding + table_width
            )
            .unwrap();
        }
        let rh = row_heights[row_index];
        for (col, lines) in wrapped_row.iter().enumerate() {
            let x = padding + col_x_offsets[col] + col_widths[col] / 2;
            let block_height = lines.len() as i32 * line_height;
            let y = row_y + (rh - block_height) / 2 + line_height - 2;
            write_multiline_svg_text(&mut body, x, y, lines, "middle", "event-copy");
        }
        row_y += rh;
    }

    svg_shell(
        id,
        width.max(320),
        height.max(120),
        &marker_id,
        "Table",
        &body,
    )
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
      fill: var(--diagram-node-fill, #1b1a18);
      stroke: var(--diagram-stroke, rgba(160, 152, 144, 0.4));
      stroke-width: 1;
    }}
    .state-node {{
      fill: var(--diagram-node-fill, #1b1a18);
      stroke: var(--diagram-stroke, rgba(160, 152, 144, 0.55));
      stroke-width: 1.5;
    }}
    .state-start {{
      fill: var(--diagram-dot, rgba(160, 152, 144, 0.85));
      stroke: var(--diagram-dot-ring, rgba(160, 152, 144, 0.2));
      stroke-width: 2;
    }}
    .self-loop {{
      stroke: var(--diagram-stroke, rgba(160, 152, 144, 0.6));
      stroke-width: 1.5;
      fill: none;
    }}
    .lane {{
      stroke: var(--diagram-lane, rgba(255, 255, 255, 0.12));
      stroke-width: 1;
      stroke-dasharray: 5 5;
    }}
    .connector {{
      stroke: var(--diagram-stroke, rgba(160, 152, 144, 0.6));
      stroke-width: 1.5;
      fill: none;
    }}
    .timeline-axis {{
      stroke: var(--diagram-lane, rgba(255, 255, 255, 0.15));
      stroke-width: 2;
    }}
    .timeline-dot {{
      fill: var(--diagram-dot, rgba(160, 152, 144, 0.8));
      stroke: var(--diagram-dot-ring, rgba(160, 152, 144, 0.15));
      stroke-width: 4;
    }}
    .node-copy, .edge-copy, .event-copy, .event-label, .bullet-copy {{
      fill: var(--diagram-text, #d9d5d0);
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    }}
    .node-copy {{
      font-size: 13px;
      font-weight: 600;
    }}
    .edge-copy {{
      font-size: 11px;
      font-weight: 500;
      fill: var(--diagram-text-muted, #a09890);
    }}
    .event-label {{
      font-size: 13px;
      font-weight: 700;
    }}
    .event-copy, .bullet-copy {{
      font-size: 12px;
      fill: var(--diagram-text-muted, #a09890);
    }}
    .er-entity {{
      fill: var(--diagram-node-fill, #1b1a18);
      stroke: var(--diagram-stroke, rgba(160, 152, 144, 0.55));
      stroke-width: 1.2;
    }}
    .er-entity-header {{
      fill: var(--diagram-lane, rgba(255, 255, 255, 0.08));
      stroke: var(--diagram-stroke, rgba(160, 152, 144, 0.55));
      stroke-width: 0;
    }}
    .er-entity-title {{
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
      font-size: 13px;
      font-weight: 700;
      fill: var(--diagram-text, #d9d5d0);
    }}
    .er-field {{
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
      font-size: 12px;
      font-weight: 600;
      fill: var(--diagram-text, #d9d5d0);
    }}
    .er-field-type {{
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
      font-size: 12px;
      fill: var(--diagram-text-muted, #a09890);
    }}
    .er-field-key {{
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
      font-size: 11px;
      font-weight: 700;
      letter-spacing: 0.04em;
      fill: var(--accent-strong, #c0b8b0);
    }}
    .er-field-note {{
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
      font-size: 11px;
      fill: var(--diagram-text-muted, #a09890);
    }}
    .er-relationship-label-bg {{
      fill: var(--surface, #1b1a18);
      stroke: var(--diagram-stroke, rgba(160, 152, 144, 0.6));
      stroke-width: 1;
    }}
    .er-relationship-label {{
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
      font-size: 11px;
      font-weight: 600;
      letter-spacing: 0.02em;
      fill: var(--diagram-text, #d9d5d0);
    }}
  </style>
  <defs>
    <marker id=\"{marker_id}\" viewBox=\"0 0 10 10\" refX=\"8\" refY=\"5\" markerWidth=\"7\" markerHeight=\"7\" orient=\"auto-start-reverse\">
      <path d=\"M 0 0 L 10 5 L 0 10 z\" fill=\"var(--diagram-stroke, rgba(160, 152, 144, 0.6))\" />
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
            "<circle cx=\"{}\" cy=\"{}\" r=\"2.6\" fill=\"var(--diagram-dot, rgba(160, 152, 144, 0.86))\"/>",
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
        .map(|paragraph| format!("<p>{}</p>", format_inline_html(paragraph)))
        .collect::<Vec<_>>()
        .join("")
}

/// Converts a single paragraph to HTML, escaping user content and then
/// applying minimal inline formatting: backtick-delimited spans become
/// `<code>`, and `[text](http(s)://...)` becomes an `<a>` with a safe target.
///
/// Escaping runs first so any generated tags wrap already-escaped content.
fn format_inline_html(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut rest = text;

    while !rest.is_empty() {
        if let Some(start) = rest.find('`') {
            // Emit everything before the backtick, with link substitution.
            output.push_str(&format_links(&escape_html(&rest[..start])));
            let after_open = &rest[start + 1..];
            if let Some(end) = after_open.find('`') {
                let code = &after_open[..end];
                output.push_str("<code>");
                output.push_str(&escape_html(code));
                output.push_str("</code>");
                rest = &after_open[end + 1..];
                continue;
            }
            // Unmatched backtick: treat as a literal and stop hunting.
            output.push_str(&format_links(&escape_html(&rest[start..])));
            return output;
        }

        output.push_str(&format_links(&escape_html(rest)));
        return output;
    }

    output
}

/// Replaces `[text](url)` spans with `<a>` tags. Only accepts http and https
/// URLs so that escaped markdown like `[x](javascript:...)` stays inert.
///
/// Operates on an already-HTML-escaped slice so `escape_html` does not have
/// to round-trip through the link text.
fn format_links(escaped: &str) -> String {
    let mut output = String::with_capacity(escaped.len());
    let bytes = escaped.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'['
            && let Some((text, url, consumed)) = parse_link(&escaped[i..])
        {
            output.push_str("<a href=\"");
            output.push_str(&url);
            output.push_str("\" target=\"_blank\" rel=\"noopener\">");
            output.push_str(&text);
            output.push_str("</a>");
            i += consumed;
            continue;
        }
        // Not a link start (or not a valid link): copy the byte.
        let ch_start = i;
        while i < bytes.len() && bytes[i] != b'[' {
            i += 1;
            if i == ch_start + 1 && ch_start + 1 < bytes.len() && bytes[ch_start] >= 0x80 {
                // For UTF-8 continuation bytes, just advance one at a time; the
                // push_str slice below remains valid because we always split at
                // char boundaries thanks to searching for single-byte markers.
                break;
            }
        }
        output.push_str(&escaped[ch_start..i]);
        if i == ch_start {
            // No progress was made (shouldn't happen given the loop condition);
            // bail out to avoid an infinite loop.
            break;
        }
    }

    output
}

/// Attempts to parse `[text](url)` starting at the opening bracket. Returns
/// the HTML-ready text, the HTML-ready URL, and the number of bytes consumed
/// from the input slice.
fn parse_link(rest: &str) -> Option<(String, String, usize)> {
    debug_assert!(rest.starts_with('['));
    let after_open = &rest[1..];
    let text_end = after_open.find(']')?;
    let text = &after_open[..text_end];
    let after_text = &after_open[text_end + 1..];
    if !after_text.starts_with('(') {
        return None;
    }
    let after_paren = &after_text[1..];
    let url_end = after_paren.find(')')?;
    let url = &after_paren[..url_end];

    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return None;
    }

    let consumed = 1 + text_end + 1 + 1 + url_end + 1;
    Some((text.to_string(), url.to_string(), consumed))
}

fn diagram_title(diagram: &Diagram) -> &'static str {
    match diagram {
        Diagram::Sequence { .. } => "Sequence diagram",
        Diagram::Flow { .. } => "Flow diagram",
        Diagram::ComponentGraph { .. } => "Component diagram",
        Diagram::Timeline { .. } => "Timeline",
        Diagram::BeforeAfter(_) => "Before / after",
        Diagram::LayerStack { .. } => "Layer stack",
        Diagram::StateMachine { .. } => "State machine",
        Diagram::Table { .. } => "Table",
        Diagram::DependencyTree { .. } => "Dependency tree",
        Diagram::EntityRelationship { .. } => "Entity relationship",
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
        Diagram::LayerStack { layers } => {
            let mut output = String::from("Layer stack\n");
            for layer in layers {
                writeln!(&mut output, "  [{layer}]").unwrap();
            }
            output.trim_end().to_owned()
        }
        Diagram::StateMachine { transitions, .. } => {
            render_ascii_edges("State machine", transitions)
        }
        Diagram::Table { headers, rows } => {
            let cols = headers.len();
            let widths: Vec<usize> = (0..cols)
                .map(|col| {
                    let header_len = headers[col].len();
                    let max_row = rows
                        .iter()
                        .map(|row| row.get(col).map_or(0, |cell| cell.len()))
                        .max()
                        .unwrap_or(0);
                    header_len.max(max_row).max(3)
                })
                .collect();
            let mut output = String::new();
            // Header row
            for (col, header) in headers.iter().enumerate() {
                if col > 0 {
                    output.push_str(" | ");
                }
                write!(&mut output, "{:<width$}", header, width = widths[col]).unwrap();
            }
            output.push('\n');
            // Separator
            for (col, width) in widths.iter().enumerate() {
                if col > 0 {
                    output.push_str("-+-");
                }
                output.push_str(&"-".repeat(*width));
            }
            output.push('\n');
            // Data rows
            for row in rows {
                for (col, cell) in row.iter().enumerate() {
                    if col > 0 {
                        output.push_str(" | ");
                    }
                    write!(&mut output, "{:<width$}", cell, width = widths[col]).unwrap();
                }
                output.push('\n');
            }
            output.trim_end().to_owned()
        }
        Diagram::DependencyTree { root, children } => {
            let mut output = format!("{root}\n");
            render_ascii_tree_children(&mut output, children, "");
            output.trim_end().to_owned()
        }
        Diagram::EntityRelationship {
            entities,
            relationships,
        } => render_ascii_entity_relationship(entities, relationships),
    }
}

fn render_ascii_tree_children(output: &mut String, children: &[TreeNode], prefix: &str) {
    for (index, child) in children.iter().enumerate() {
        let is_last = index == children.len() - 1;
        let connector = if is_last { "└── " } else { "├── " };
        writeln!(output, "{prefix}{connector}{}", child.label).unwrap();
        if !child.children.is_empty() {
            let child_prefix = if is_last {
                format!("{prefix}    ")
            } else {
                format!("{prefix}│   ")
            };
            render_ascii_tree_children(output, &child.children, &child_prefix);
        }
    }
}

fn render_ascii_entity_relationship(entities: &[Entity], relationships: &[Relationship]) -> String {
    let mut output = String::from("Entity relationship\n");
    for entity in entities {
        writeln!(&mut output, "  [{}]", entity.name).unwrap();
        for field in &entity.fields {
            match &field.note {
                Some(note) => writeln!(
                    &mut output,
                    "    - {} : {} ({})",
                    field.name, field.field_type, note
                )
                .unwrap(),
                None => {
                    writeln!(&mut output, "    - {} : {}", field.name, field.field_type).unwrap()
                }
            }
        }
    }
    if !relationships.is_empty() {
        output.push('\n');
        for relationship in relationships {
            let connector = ascii_cardinality_connector(relationship.cardinality);
            match &relationship.label {
                Some(label) => writeln!(
                    &mut output,
                    "  {} {} {} : {}",
                    relationship.from, connector, relationship.to, label
                )
                .unwrap(),
                None => writeln!(
                    &mut output,
                    "  {} {} {}",
                    relationship.from, connector, relationship.to
                )
                .unwrap(),
            }
        }
    }
    output.trim_end().to_owned()
}

fn ascii_cardinality_connector(cardinality: Cardinality) -> &'static str {
    match cardinality {
        Cardinality::OneToOne => "||--||",
        Cardinality::OneToMany => "||--o{",
        Cardinality::ManyToOne => "}o--||",
        Cardinality::ManyToMany => "}o--o{",
    }
}

fn render_mermaid_entity_relationship(
    entities: &[Entity],
    relationships: &[Relationship],
) -> String {
    let mut output = String::from("erDiagram\n");
    for relationship in relationships {
        let connector = mermaid_cardinality_connector(relationship.cardinality);
        let label = relationship
            .label
            .as_deref()
            .map(escape_mermaid_text)
            .unwrap_or_else(|| String::from("relates"));
        writeln!(
            &mut output,
            "    {} {connector} {} : {label}",
            mermaid_entity_id(&relationship.from),
            mermaid_entity_id(&relationship.to),
        )
        .unwrap();
    }
    for entity in entities {
        writeln!(&mut output, "    {} {{", mermaid_entity_id(&entity.name)).unwrap();
        for field in &entity.fields {
            let field_type = sanitize_node(&field.field_type);
            let field_name = sanitize_node(&field.name);
            match &field.note {
                Some(note) => writeln!(
                    &mut output,
                    "        {field_type} {field_name} {}",
                    mermaid_field_note(note)
                )
                .unwrap(),
                None => writeln!(&mut output, "        {field_type} {field_name}").unwrap(),
            }
        }
        writeln!(&mut output, "    }}").unwrap();
    }
    output.trim_end().to_owned()
}

fn mermaid_cardinality_connector(cardinality: Cardinality) -> &'static str {
    match cardinality {
        Cardinality::OneToOne => "||--||",
        Cardinality::OneToMany => "||--o{",
        Cardinality::ManyToOne => "}o--||",
        Cardinality::ManyToMany => "}o--o{",
    }
}

fn mermaid_entity_id(name: &str) -> String {
    sanitize_node(name).to_uppercase()
}

fn mermaid_field_note(note: &str) -> String {
    let upper = note.to_uppercase();
    if upper == "PK" || upper == "FK" || upper == "UK" {
        upper
    } else {
        format!("\"{}\"", escape_mermaid_text(note))
    }
}

fn render_entity_relationship_svg(
    id: &str,
    entities: &[Entity],
    relationships: &[Relationship],
) -> String {
    let padding = 24;
    let header_height = 32;
    let row_height = 22;
    // Wide enough that the relationship label pill (min width 48) fits in the
    // gap without nibbling into the rounded entity corners on either side.
    let entity_gap_x = 72;
    let entity_gap_y = 48;
    let columns: usize = if entities.len() <= 2 {
        entities.len()
    } else {
        2
    }
    .max(1);
    let rows = entities.len().div_ceil(columns).max(1);
    let marker_id = format!("{id}-arrow");

    // Size the field name and type columns from their actual longest entries so
    // long names like `subscription_id` don't overflow into the type column.
    let char_w: i32 = 8;
    let inner_pad: i32 = 12;
    let col_gap: i32 = 16;
    let key_col_w: i32 = 24;

    let mut max_name_chars: usize = 0;
    let mut max_type_chars: usize = 0;
    let mut max_header_chars: usize = 0;
    let mut has_key_column = false;
    for entity in entities {
        max_header_chars = max_header_chars.max(entity.name.len());
        for field in &entity.fields {
            max_name_chars = max_name_chars.max(field.name.len());
            max_type_chars = max_type_chars.max(field.field_type.len());
            if field.note.is_some() {
                has_key_column = true;
            }
        }
    }
    let name_col_w = max_name_chars as i32 * char_w;
    let type_col_w = max_type_chars as i32 * char_w;
    let key_section = if has_key_column {
        col_gap + key_col_w
    } else {
        0
    };
    let body_w = inner_pad + name_col_w + col_gap + type_col_w + key_section + inner_pad;
    let header_w = max_header_chars as i32 * char_w + 32;
    let entity_width: i32 = body_w.max(header_w).max(180);
    let type_col_x_offset = inner_pad + name_col_w + col_gap;
    let entity_height =
        |entity: &Entity| -> i32 { header_height + entity.fields.len() as i32 * row_height + 8 };
    let max_entity_height: i32 = entities.iter().map(entity_height).max().unwrap_or(64);

    let width = padding * 2
        + columns as i32 * entity_width
        + (columns.saturating_sub(1)) as i32 * entity_gap_x;
    let height = padding * 2
        + rows as i32 * max_entity_height
        + (rows.saturating_sub(1)) as i32 * entity_gap_y;

    let positions: Vec<(&str, (i32, i32))> = entities
        .iter()
        .enumerate()
        .map(|(index, entity)| {
            let row = index / columns;
            let col = index % columns;
            let x = padding + col as i32 * (entity_width + entity_gap_x);
            let y = padding + row as i32 * (max_entity_height + entity_gap_y);
            (entity.name.as_str(), (x, y))
        })
        .collect();

    let mut body = String::new();

    // Pass 1: connector lines, clipped to entity-box edges so they don't disappear
    // behind the rects we paint next.
    let mut endpoints: Vec<(i32, i32, i32, i32)> = Vec::with_capacity(relationships.len());
    for relationship in relationships {
        let Some((from_origin, from_h)) = entities
            .iter()
            .zip(positions.iter())
            .find(|(entity, _)| entity.name == relationship.from)
            .map(|(entity, (_, p))| (*p, entity_height(entity)))
        else {
            endpoints.push((0, 0, 0, 0));
            continue;
        };
        let Some((to_origin, to_h)) = entities
            .iter()
            .zip(positions.iter())
            .find(|(entity, _)| entity.name == relationship.to)
            .map(|(entity, (_, p))| (*p, entity_height(entity)))
        else {
            endpoints.push((0, 0, 0, 0));
            continue;
        };

        let from_cx = from_origin.0 + entity_width / 2;
        let from_cy = from_origin.1 + from_h / 2;
        let to_cx = to_origin.0 + entity_width / 2;
        let to_cy = to_origin.1 + to_h / 2;

        let (start_x, start_y) =
            clip_to_rect_edge(from_cx, from_cy, entity_width, from_h, to_cx, to_cy);
        let (end_x, end_y) = clip_to_rect_edge(to_cx, to_cy, entity_width, to_h, from_cx, from_cy);

        write!(
            &mut body,
            "<line class=\"connector\" x1=\"{start_x}\" y1=\"{start_y}\" x2=\"{end_x}\" y2=\"{end_y}\" marker-end=\"url(#{marker_id})\"/>"
        )
        .unwrap();
        endpoints.push((start_x, start_y, end_x, end_y));
    }

    // Pass 2: entity boxes and their fields. These paint over the connector lines.
    for (entity, (x, y)) in entities.iter().zip(positions.iter().map(|(_, p)| *p)) {
        let h = entity_height(entity);

        write!(
            &mut body,
            "<rect class=\"er-entity\" x=\"{x}\" y=\"{y}\" width=\"{entity_width}\" height=\"{h}\" rx=\"8\" ry=\"8\"/>"
        )
        .unwrap();
        write!(
            &mut body,
            "<rect class=\"er-entity-header\" x=\"{x}\" y=\"{y}\" width=\"{entity_width}\" height=\"{header_height}\" rx=\"8\" ry=\"8\"/>"
        )
        .unwrap();
        write_multiline_svg_text(
            &mut body,
            x + entity_width / 2,
            y + 20,
            std::slice::from_ref(&entity.name),
            "middle",
            "er-entity-title",
        );

        let field_x_left = x + 12;
        let field_x_right = x + entity_width - 12;
        for (field_index, field) in entity.fields.iter().enumerate() {
            let row_y = y + header_height + (field_index as i32) * row_height + 4;
            let baseline = row_y + 14;

            if field_index > 0 {
                let sep_y = row_y;
                write!(
                    &mut body,
                    "<line class=\"lane\" x1=\"{x}\" y1=\"{sep_y}\" x2=\"{}\" y2=\"{sep_y}\" style=\"stroke-dasharray:none;stroke-width:0.6\"/>",
                    x + entity_width
                )
                .unwrap();
            }

            if let Some(note) = &field.note {
                let upper = note.to_uppercase();
                let class = if upper == "PK" || upper == "FK" || upper == "UK" {
                    "er-field-key"
                } else {
                    "er-field-note"
                };
                write!(
                    &mut body,
                    "<text class=\"{class}\" x=\"{field_x_right}\" y=\"{baseline}\" text-anchor=\"end\">{}</text>",
                    escape_html(note)
                )
                .unwrap();
            }

            write!(
                &mut body,
                "<text class=\"er-field\" x=\"{field_x_left}\" y=\"{baseline}\" text-anchor=\"start\">{}</text>",
                escape_html(&field.name)
            )
            .unwrap();
            write!(
                &mut body,
                "<text class=\"er-field-type\" x=\"{}\" y=\"{baseline}\" text-anchor=\"start\">{}</text>",
                x + type_col_x_offset,
                escape_html(&field.field_type)
            )
            .unwrap();
        }
    }

    // Pass 3: relationship labels on top of everything else, sitting on a backing
    // pill so they stay readable when the line grazes another entity.
    for (relationship, (sx, sy, ex, ey)) in relationships.iter().zip(endpoints.iter()) {
        let Some(label) = relationship.label.as_ref() else {
            continue;
        };
        let mid_x = (sx + ex) / 2;
        let mid_y = (sy + ey) / 2;
        let pill_width = (label.chars().count() as i32 * 7).clamp(48, 220);
        let pill_height = 20;
        let pill_x = mid_x - pill_width / 2;
        let pill_y = mid_y - pill_height / 2;
        write!(
            &mut body,
            "<rect class=\"er-relationship-label-bg\" x=\"{pill_x}\" y=\"{pill_y}\" width=\"{pill_width}\" height=\"{pill_height}\" rx=\"10\" ry=\"10\"/>"
        )
        .unwrap();
        write!(
            &mut body,
            "<text class=\"er-relationship-label\" x=\"{mid_x}\" y=\"{}\" text-anchor=\"middle\">{}</text>",
            mid_y + 4,
            escape_html(label)
        )
        .unwrap();
    }

    svg_shell(
        id,
        width.max(360),
        height.max(160),
        &marker_id,
        "Entity relationship",
        &body,
    )
}

fn clip_to_rect_edge(cx: i32, cy: i32, w: i32, h: i32, tx: i32, ty: i32) -> (i32, i32) {
    let dx = (tx - cx) as f64;
    let dy = (ty - cy) as f64;
    if dx.abs() < 0.5 && dy.abs() < 0.5 {
        return (cx, cy);
    }
    let half_w = w as f64 / 2.0;
    let half_h = h as f64 / 2.0;
    let scale_x = if dx.abs() < 1e-6 {
        f64::INFINITY
    } else {
        half_w / dx.abs()
    };
    let scale_y = if dy.abs() < 1e-6 {
        f64::INFINITY
    } else {
        half_h / dy.abs()
    };
    let scale = scale_x.min(scale_y);
    (
        (cx as f64 + dx * scale).round() as i32,
        (cy as f64 + dy * scale).round() as i32,
    )
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

fn render_markdown_table(headers: &[String], rows: &[Vec<String>]) -> String {
    let mut output = String::new();
    output.push_str("| ");
    output.push_str(
        &headers
            .iter()
            .map(|header| escape_markdown_cell(header))
            .collect::<Vec<_>>()
            .join(" | "),
    );
    output.push_str(" |\n");

    output.push_str("| ");
    output.push_str(
        &headers
            .iter()
            .map(|_| "---".to_string())
            .collect::<Vec<_>>()
            .join(" | "),
    );
    output.push_str(" |\n");

    for row in rows {
        output.push_str("| ");
        output.push_str(
            &row.iter()
                .map(|cell| escape_markdown_cell(cell))
                .collect::<Vec<_>>()
                .join(" | "),
        );
        output.push_str(" |\n");
    }

    output.trim_end_matches('\n').to_string()
}

fn escape_markdown_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
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
        Diagram::LayerStack { layers } => {
            let mut output = String::from("block-beta\n");
            for (index, layer) in layers.iter().enumerate() {
                writeln!(
                    &mut output,
                    "    L{}[\"{}\"]",
                    index,
                    escape_mermaid_text(layer)
                )
                .unwrap();
            }
            output.trim_end().to_owned()
        }
        Diagram::StateMachine { transitions, .. } => {
            let mut output = String::from("stateDiagram-v2\n");
            for edge in transitions {
                let label = edge.label.as_deref().unwrap_or("");
                writeln!(
                    &mut output,
                    "    {} --> {}: {}",
                    sanitize_node(&edge.from),
                    sanitize_node(&edge.to),
                    label
                )
                .unwrap();
            }
            output.trim_end().to_owned()
        }
        Diagram::DependencyTree { root, children } => {
            let mut output = String::from("flowchart TD\n");
            fn emit_mermaid_tree(
                output: &mut String,
                parent: &str,
                children: &[TreeNode],
                counter: &mut usize,
            ) {
                for child in children {
                    let child_id = format!("N{}", *counter);
                    *counter += 1;
                    writeln!(
                        output,
                        "    {} --> {}[\"{}\"]",
                        parent,
                        child_id,
                        escape_mermaid_text(&child.label)
                    )
                    .unwrap();
                    emit_mermaid_tree(output, &child_id, &child.children, counter);
                }
            }
            let root_id = "ROOT";
            writeln!(
                &mut output,
                "    {root_id}[\"{}\"]",
                escape_mermaid_text(root)
            )
            .unwrap();
            let mut counter = 0;
            emit_mermaid_tree(&mut output, root_id, children, &mut counter);
            output.trim_end().to_owned()
        }
        Diagram::Table { headers, rows } => {
            // Mermaid has no first-class table type, so markdown callers render a
            // real GFM table instead. Leave a minimal representation here for any
            // future consumers that still funnel table diagrams through mermaid.
            render_markdown_table(headers, rows)
        }
        Diagram::EntityRelationship {
            entities,
            relationships,
        } => render_mermaid_entity_relationship(entities, relationships),
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
    use crate::model::{
        Cardinality, Diagram, Document, Edge, Entity, Field, Relationship, Section, TreeNode,
        Verification,
    };

    fn er_diagram_sample() -> Diagram {
        Diagram::EntityRelationship {
            entities: vec![
                Entity {
                    name: "User".into(),
                    fields: vec![
                        Field {
                            name: "id".into(),
                            field_type: "uuid".into(),
                            note: Some("PK".into()),
                        },
                        Field {
                            name: "email".into(),
                            field_type: "string".into(),
                            note: None,
                        },
                    ],
                },
                Entity {
                    name: "Order".into(),
                    fields: vec![
                        Field {
                            name: "id".into(),
                            field_type: "uuid".into(),
                            note: Some("PK".into()),
                        },
                        Field {
                            name: "user_id".into(),
                            field_type: "uuid".into(),
                            note: Some("FK".into()),
                        },
                        Field {
                            name: "total".into(),
                            field_type: "decimal".into(),
                            note: None,
                        },
                    ],
                },
            ],
            relationships: vec![Relationship {
                from: "User".into(),
                to: "Order".into(),
                cardinality: Cardinality::OneToMany,
                label: Some("places".into()),
            }],
        }
    }
    use crate::{ExamplePreset, example_document};

    fn sample_document() -> Document {
        Document {
            title: "Magellan demo".into(),
            summary: vec![
                "A short summary explains the outcome in product terms.".into(),
                "A second paragraph adds only the necessary context.".into(),
            ],
            sections: vec![
                Section {
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
                    commit: None,
                    files: vec![],
                },
                Section {
                    title: "Why it matters".into(),
                    text: vec!["Errors surface immediately rather than after a round-trip.".into()],
                    diagram: None,
                    commit: None,
                    files: vec![],
                },
            ],
            verification: Some(Verification {
                text: vec!["An integration test and a quick manual check passed.".into()],
            }),
            repo: None,
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
        assert!(rendered.contains("class=\"sidebar\""));
        assert!(rendered.contains("class=\"toc-link"));
        assert!(rendered.contains("id=\"section-1\""));
        assert!(rendered.contains("--bg: #131211"));
        assert!(rendered.contains("data-theme-toggle"));
        assert!(rendered.contains("[data-theme=\"light\"]"));
        assert!(rendered.contains("<link rel=\"icon\" href=\"data:,\">"));
        assert!(rendered.contains("<svg viewBox="));
        assert!(rendered.contains("ASCII fallback"));
        assert!(rendered.contains("color-scheme: dark;"));
        assert!(rendered.contains("color-scheme: light;"));
        assert!(!rendered.contains("cdn.jsdelivr"));
        assert!(!rendered.contains("Book View"));
        assert!(!rendered.contains("data-book-track"));
        assert!(!rendered.contains("data-diagram-modal"));
        assert!(!rendered.contains("Click to enlarge"));
    }

    #[test]
    fn table_diagram_renders_clean_markdown_table_in_mermaid_block() {
        let document = Document {
            title: "Table walkthrough".into(),
            summary: vec!["Summary.".into(), "More summary.".into()],
            sections: vec![
                Section {
                    title: "Permissions".into(),
                    text: vec!["What the table shows.".into()],
                    diagram: Some(Diagram::Table {
                        headers: vec!["Role".into(), "Create".into(), "Delete".into()],
                        rows: vec![
                            vec!["admin".into(), "yes".into(), "yes".into()],
                            vec!["user".into(), "yes".into(), "no".into()],
                        ],
                    }),
                    commit: None,
                    files: vec![],
                },
                Section {
                    title: "Notes".into(),
                    text: vec!["Why the mapping matters.".into()],
                    diagram: None,
                    commit: None,
                    files: vec![],
                },
            ],
            verification: None,
            repo: None,
        };

        let rendered = render_document(&document, OutputFormat::Markdown);

        assert!(
            rendered.contains("| Role | Create | Delete |"),
            "should emit a markdown table header row"
        );
        assert!(
            rendered.contains("| --- | --- | --- |"),
            "should emit a markdown table separator row"
        );
        assert!(
            rendered.contains("| admin | yes | yes |"),
            "should emit data rows"
        );
        assert!(
            !rendered.contains("\\n"),
            "table must not leak escaped newlines (was `{rendered}`)"
        );
        assert!(
            !rendered.contains("T[\""),
            "table must not wrap rows inside a flowchart node"
        );
    }

    fn doc_with_diagram(title: &str, diagram: Diagram) -> Document {
        Document {
            title: title.into(),
            summary: vec!["Short summary.".into()],
            sections: vec![
                Section {
                    title: "Diagram section".into(),
                    text: vec!["The diagram shows the structure.".into()],
                    diagram: Some(diagram),
                    commit: None,
                    files: vec![],
                },
                Section {
                    title: "Follow-up".into(),
                    text: vec!["Why it matters.".into()],
                    diagram: None,
                    commit: None,
                    files: vec![],
                },
            ],
            verification: None,
            repo: None,
        }
    }

    #[test]
    fn html_renders_inline_code_and_links_in_paragraphs() {
        let doc = Document {
            title: "Inline formatting".into(),
            summary: vec![
                "See the `Order` type in [the schema](https://example.com/schema) for details.".into(),
            ],
            sections: vec![
                Section {
                    title: "Request flow".into(),
                    text: vec![
                        "Call `validate_request(payload)` before [enqueueing](https://example.com/queue).".into(),
                    ],
                    diagram: None,
                    commit: None,
                    files: vec![],
                },
                Section {
                    title: "Verification".into(),
                    text: vec!["Tests use `assert_eq!` throughout.".into()],
                    diagram: None,
                    commit: None,
                    files: vec![],
                },
            ],
            verification: None,
            repo: None,
        };

        let html = render_document(&doc, OutputFormat::Html);

        assert!(
            html.contains("<code>Order</code>"),
            "summary inline code should become <code>"
        );
        assert!(
            html.contains("<a href=\"https://example.com/schema\""),
            "summary inline link should become <a>"
        );
        assert!(
            html.contains("<code>validate_request(payload)</code>"),
            "section inline code should become <code>"
        );
        assert!(
            html.contains("<a href=\"https://example.com/queue\""),
            "section inline link should become <a>"
        );
        assert!(
            html.contains("<code>assert_eq!</code>"),
            "later paragraphs should still get inline code rendering"
        );
        assert!(
            !html.contains("`Order`") && !html.contains("`validate_request(payload)`"),
            "raw backticks should not leak into the rendered HTML"
        );
    }

    #[test]
    fn html_inline_formatting_escapes_user_content() {
        let doc = Document {
            title: "Escaping".into(),
            summary: vec!["Plain summary.".into()],
            sections: vec![
                Section {
                    title: "Danger".into(),
                    text: vec!["Call `<script>alert(1)</script>` for fun.".into()],
                    diagram: None,
                    commit: None,
                    files: vec![],
                },
                Section {
                    title: "More".into(),
                    text: vec!["Second section.".into()],
                    diagram: None,
                    commit: None,
                    files: vec![],
                },
            ],
            verification: None,
            repo: None,
        };

        let html = render_document(&doc, OutputFormat::Html);

        assert!(
            html.contains("<code>&lt;script&gt;alert(1)&lt;/script&gt;</code>"),
            "inline code contents must be HTML-escaped"
        );
        assert!(
            !html.contains("<script>alert(1)</script>"),
            "raw script tags must never reach the output"
        );
    }

    #[test]
    fn markdown_passes_inline_formatting_through_unchanged() {
        let doc = Document {
            title: "Markdown".into(),
            summary: vec!["Plain summary.".into()],
            sections: vec![
                Section {
                    title: "Code".into(),
                    text: vec!["Use `cargo test` often.".into()],
                    diagram: None,
                    commit: None,
                    files: vec![],
                },
                Section {
                    title: "Links".into(),
                    text: vec!["See [docs](https://example.com).".into()],
                    diagram: None,
                    commit: None,
                    files: vec![],
                },
            ],
            verification: None,
            repo: None,
        };

        let rendered = render_document(&doc, OutputFormat::Markdown);

        assert!(rendered.contains("Use `cargo test` often."));
        assert!(rendered.contains("See [docs](https://example.com)."));
    }

    #[test]
    fn state_machine_svg_has_dedicated_start_marker_and_self_loop() {
        let doc = doc_with_diagram(
            "Dedicated state machine SVG",
            Diagram::StateMachine {
                states: vec!["Idle".into(), "Working".into(), "Done".into()],
                transitions: vec![
                    Edge {
                        from: "Idle".into(),
                        to: "Working".into(),
                        label: Some("start".into()),
                    },
                    Edge {
                        from: "Working".into(),
                        to: "Working".into(),
                        label: Some("retry".into()),
                    },
                    Edge {
                        from: "Working".into(),
                        to: "Done".into(),
                        label: Some("finish".into()),
                    },
                ],
            },
        );

        let html = render_document(&doc, OutputFormat::Html);

        assert!(
            html.contains("class=\"state-start\""),
            "state machine SVG should include an initial-state marker"
        );
        assert!(
            html.contains("class=\"self-loop\""),
            "state machine SVG should include a dedicated self-loop path"
        );
        assert!(
            html.contains("class=\"state-node\""),
            "state machine SVG should use dedicated state-node styling instead of the generic graph node"
        );
    }

    #[test]
    fn state_machine_renders_in_all_three_formats() {
        let doc = doc_with_diagram(
            "State machine rendering",
            Diagram::StateMachine {
                states: vec!["Idle".into(), "Running".into(), "Done".into()],
                transitions: vec![
                    Edge {
                        from: "Idle".into(),
                        to: "Running".into(),
                        label: Some("start".into()),
                    },
                    Edge {
                        from: "Running".into(),
                        to: "Done".into(),
                        label: Some("finish".into()),
                    },
                ],
            },
        );

        let terminal = render_document(&doc, OutputFormat::Terminal);
        assert!(terminal.contains("State machine"));
        assert!(terminal.contains("Idle --start--> Running"));
        assert!(terminal.contains("Running --finish--> Done"));

        let markdown = render_document(&doc, OutputFormat::Markdown);
        assert!(markdown.contains("stateDiagram-v2"));
        assert!(markdown.contains("Idle --> Running: start"));

        let html = render_document(&doc, OutputFormat::Html);
        assert!(html.contains("State machine"));
        assert!(html.contains("<svg viewBox="));
    }

    #[test]
    fn layer_stack_renders_in_all_three_formats() {
        let doc = doc_with_diagram(
            "Layer stack rendering",
            Diagram::LayerStack {
                layers: vec!["Edge".into(), "Auth".into(), "App".into(), "DB".into()],
            },
        );

        let terminal = render_document(&doc, OutputFormat::Terminal);
        assert!(terminal.contains("Layer stack"));
        assert!(terminal.contains("[Edge]"));
        assert!(terminal.contains("[DB]"));

        let markdown = render_document(&doc, OutputFormat::Markdown);
        assert!(markdown.contains("block-beta"));
        assert!(markdown.contains("L0[\"Edge\"]"));
        assert!(markdown.contains("L3[\"DB\"]"));

        let html = render_document(&doc, OutputFormat::Html);
        assert!(html.contains("Layer stack"));
        assert!(html.contains("<svg viewBox="));
    }

    #[test]
    fn table_diagram_renders_in_all_three_formats() {
        let doc = doc_with_diagram(
            "Table rendering",
            Diagram::Table {
                headers: vec!["Role".into(), "Read".into(), "Write".into()],
                rows: vec![
                    vec!["admin".into(), "yes".into(), "yes".into()],
                    vec!["viewer".into(), "yes".into(), "no".into()],
                ],
            },
        );

        let terminal = render_document(&doc, OutputFormat::Terminal);
        assert!(terminal.contains("Role"));
        assert!(terminal.contains("admin"));
        assert!(terminal.contains("-+-"));

        let markdown = render_document(&doc, OutputFormat::Markdown);
        assert!(markdown.contains("| Role | Read | Write |"));
        assert!(markdown.contains("| --- | --- | --- |"));
        assert!(markdown.contains("| admin | yes | yes |"));

        let html = render_document(&doc, OutputFormat::Html);
        assert!(html.contains("Table"));
        assert!(html.contains("<svg viewBox="));
    }

    #[test]
    fn dependency_tree_renders_in_all_three_formats() {
        let doc = doc_with_diagram(
            "Dependency tree rendering",
            Diagram::DependencyTree {
                root: "service".into(),
                children: vec![
                    TreeNode {
                        label: "api".into(),
                        children: vec![TreeNode {
                            label: "routes".into(),
                            children: vec![],
                        }],
                    },
                    TreeNode {
                        label: "worker".into(),
                        children: vec![],
                    },
                ],
            },
        );

        let terminal = render_document(&doc, OutputFormat::Terminal);
        assert!(terminal.contains("service"));
        assert!(terminal.contains("├── api"));
        assert!(terminal.contains("└── worker"));
        assert!(terminal.contains("└── routes"));

        let markdown = render_document(&doc, OutputFormat::Markdown);
        assert!(markdown.contains("flowchart TD"));
        assert!(markdown.contains("ROOT[\"service\"]"));
        assert!(markdown.contains("[\"api\"]"));
        assert!(markdown.contains("[\"routes\"]"));

        let html = render_document(&doc, OutputFormat::Html);
        assert!(html.contains("Dependency tree"));
        assert!(html.contains("<svg viewBox="));
    }

    #[test]
    fn html_inline_diagrams_break_out_of_paragraph_reading_width() {
        let rendered = render_document(&sample_document(), OutputFormat::Html);

        // Prose paragraphs should still cap around 64 characters for readability.
        assert!(
            rendered.contains(".section-body > p {")
                && rendered.contains(".section-body > p {\n      max-width: 64ch;"),
            "section-body paragraphs (not the whole body) should cap at 64ch for readable prose"
        );
        // The unscoped `.section-body { max-width: 64ch }` rule must be gone so that
        // diagrams can use the full content width.
        assert!(
            !rendered.contains(".section-body {\n      max-width: 64ch;"),
            "section-body itself must not cap width — that squeezes inline diagrams"
        );
        // Diagrams should explicitly opt out of the prose width and span the column.
        assert!(
            rendered.contains(".diagram {")
                && rendered.contains("max-width: none;")
                && rendered.contains("width: 100%;"),
            "the .diagram block should declare width: 100% and max-width: none so it can grow"
        );
        // Give the report column more horizontal room so the diagrams have space to breathe.
        assert!(
            rendered.contains("max-width: 1080px"),
            ".content max-width should grow to 1080px so diagrams render larger"
        );
        // Inline SVGs need a comfortable minimum rendered height so text inside them
        // stays readable and stops overflowing on small viewBoxes.
        assert!(
            rendered.contains(".diagram svg {") && rendered.contains("min-height: 260px;"),
            "diagram SVGs should get a min-height so inline text stays legible"
        );
    }

    #[test]
    fn entity_relationship_paints_relationship_labels_after_entity_rects() {
        let doc = doc_with_diagram("ER paint order", er_diagram_sample());
        let html = render_document(&doc, OutputFormat::Html);

        let last_entity_rect = html
            .rfind("class=\"er-entity\"")
            .expect("ER SVG should declare entity rects");
        let label_marker = "class=\"er-relationship-label\"";
        let first_label = html
            .find(label_marker)
            .expect("ER SVG should declare a relationship-label class");

        assert!(
            first_label > last_entity_rect,
            "relationship labels must paint after entity rects so they aren't occluded \
             (first label at {first_label}, last entity rect at {last_entity_rect})"
        );

        // The label tag should sit on top of a backing pill so it stays readable when
        // the connector line crosses or grazes another entity.
        assert!(
            html.contains("class=\"er-relationship-label-bg\""),
            "relationship labels should have a background pill"
        );
    }

    #[test]
    fn entity_relationship_renders_html_svg_with_entities_and_relationships() {
        let doc = doc_with_diagram("ER HTML rendering", er_diagram_sample());

        let html = render_document(&doc, OutputFormat::Html);

        assert!(
            html.contains("Entity relationship"),
            "HTML output should label the diagram type, got truncated:\n{}",
            html.chars().take(400).collect::<String>()
        );
        assert!(
            html.contains("<svg viewBox="),
            "HTML output should embed an SVG"
        );
        assert!(
            html.contains("class=\"er-entity\""),
            "ER SVG should expose dedicated entity styling"
        );
        assert!(
            html.contains("class=\"er-entity-header\""),
            "ER SVG should mark the entity title row"
        );
        assert!(
            html.contains("class=\"er-field\""),
            "ER SVG should mark field rows"
        );
        assert!(
            html.contains(">User<") && html.contains(">Order<"),
            "ER SVG should include entity names as text"
        );
        assert!(
            html.contains(">id<") && html.contains(">email<") && html.contains(">user_id<"),
            "ER SVG should include field names as text"
        );
        assert!(
            html.contains(">uuid<") && html.contains(">decimal<"),
            "ER SVG should include field types as text"
        );
        assert!(
            html.contains(">PK<") && html.contains(">FK<"),
            "ER SVG should render note markers"
        );
        // ASCII fallback should still be present in the figure
        assert!(
            html.contains("ASCII fallback"),
            "ER figure should keep an ASCII fallback section"
        );
        assert!(
            html.contains("User ||--o{ Order : places"),
            "ASCII fallback should include the relationship line"
        );
    }

    #[test]
    fn entity_relationship_renders_mermaid_block_in_markdown() {
        let doc = doc_with_diagram("ER mermaid rendering", er_diagram_sample());

        let markdown = render_document(&doc, OutputFormat::Markdown);

        assert!(
            markdown.contains("```mermaid"),
            "ER markdown output should be wrapped in a mermaid fence, got:\n{markdown}"
        );
        assert!(
            markdown.contains("erDiagram"),
            "mermaid block should declare an erDiagram, got:\n{markdown}"
        );
        assert!(
            markdown.contains("USER ||--o{ ORDER : places"),
            "mermaid block should encode the relationship with cardinality, got:\n{markdown}"
        );
        assert!(
            markdown.contains("USER {"),
            "mermaid block should open a USER entity definition, got:\n{markdown}"
        );
        assert!(
            markdown.contains("uuid id PK"),
            "mermaid block should encode field type, name, and note, got:\n{markdown}"
        );
        assert!(
            markdown.contains("string email"),
            "mermaid block should encode notes-less fields, got:\n{markdown}"
        );
        assert!(
            markdown.contains("ORDER {"),
            "mermaid block should open an ORDER entity definition, got:\n{markdown}"
        );
        assert!(
            markdown.contains("uuid user_id FK"),
            "mermaid block should encode foreign-key fields, got:\n{markdown}"
        );
    }

    #[test]
    fn entity_relationship_renders_ascii_with_entities_fields_and_relationships() {
        let doc = doc_with_diagram("ER ASCII rendering", er_diagram_sample());

        let terminal = render_document(&doc, OutputFormat::Terminal);

        assert!(
            terminal.contains("Entity relationship"),
            "ASCII output should label the diagram type, got:\n{terminal}"
        );
        assert!(
            terminal.contains("[User]"),
            "ASCII output should show the User entity header, got:\n{terminal}"
        );
        assert!(
            terminal.contains("[Order]"),
            "ASCII output should show the Order entity header, got:\n{terminal}"
        );
        assert!(
            terminal.contains("id : uuid (PK)"),
            "ASCII output should show field name, type, and note, got:\n{terminal}"
        );
        assert!(
            terminal.contains("email : string"),
            "ASCII output should show fields without notes, got:\n{terminal}"
        );
        assert!(
            terminal.contains("user_id : uuid (FK)"),
            "ASCII output should show foreign key fields, got:\n{terminal}"
        );
        assert!(
            terminal.contains("User ||--o{ Order : places"),
            "ASCII output should show relationship with cardinality and label, got:\n{terminal}"
        );
    }

    #[test]
    fn theme_toggle_uses_sun_moon_icons() {
        let rendered = render_document(&sample_document(), OutputFormat::Html);

        // Button should NOT contain the old text label
        assert!(
            !rendered.contains(">Toggle theme</button>"),
            "theme toggle should use icons, not text"
        );
        // Button should contain sun and moon symbols
        assert!(
            rendered.contains("☀️"),
            "theme toggle should contain a sun icon for light mode"
        );
        assert!(
            rendered.contains("🌙"),
            "theme toggle should contain a moon icon for dark mode"
        );
        // The JS should swap the icon on toggle
        assert!(
            rendered.contains("updateThemeIcon"),
            "theme toggle JS should include an icon-update function"
        );
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
