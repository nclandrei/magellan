use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;

struct FixtureCase {
    path: &'static str,
    title: &'static str,
    terminal_markers: &'static [&'static str],
    markdown_markers: &'static [&'static str],
    html_markers: &'static [&'static str],
}

fn fixture_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn fixture_cases() -> [FixtureCase; 3] {
    [
        FixtureCase {
            path: "examples/session-walkthrough.json",
            title: "Session walkthrough: checkout validation moved in front of order submission",
            terminal_markers: &[
                "How the request flow changed",
                "Sequence",
                "Flow",
                "Component graph",
                "Verification",
            ],
            markdown_markers: &[
                "## How the request flow changed",
                "sequenceDiagram",
                "flowchart LR",
                "## Verification",
            ],
            html_markers: &[
                "Magellan walkthrough",
                "class=\"sidebar\"",
                "class=\"toc-link",
                "Sequence diagram",
                "Flow diagram",
                "Component diagram",
                "ASCII fallback",
            ],
        },
        FixtureCase {
            path: "examples/branch-handoff-timeline.json",
            title: "Handoff: search results hydration cleanup across the branch",
            terminal_markers: &[
                "Timeline of the change",
                "Timeline",
                "Steady-state flow after the cleanup",
                "Flow",
            ],
            markdown_markers: &[
                "## Timeline of the change",
                "timeline",
                "title Timeline",
                "flowchart LR",
            ],
            html_markers: &[
                "Magellan walkthrough",
                "class=\"sidebar\"",
                "class=\"toc-link",
                "Timeline",
                "Flow diagram",
                "Verification",
                "ASCII fallback",
            ],
        },
        FixtureCase {
            path: "examples/followup-validation-question.json",
            title: "Follow-up: why the retry guard moved into the background worker",
            terminal_markers: &["Why the worker owns retries now", "Flow", "Before", "After"],
            markdown_markers: &[
                "## Why the worker owns retries now",
                "flowchart LR",
                "subgraph Before",
                "subgraph After",
            ],
            html_markers: &[
                "Magellan walkthrough",
                "class=\"sidebar\"",
                "class=\"toc-link",
                "Flow diagram",
                "Before / after",
                "ASCII fallback",
            ],
        },
    ]
}

#[test]
fn checked_in_fixture_payloads_validate_with_real_binary() {
    for case in fixture_cases() {
        Command::cargo_bin("magellan")
            .expect("binary should build")
            .args(["validate", "--input"])
            .arg(fixture_path(case.path))
            .assert()
            .success();
    }
}

#[test]
fn checked_in_fixtures_render_expected_terminal_and_markdown_output() {
    for case in fixture_cases() {
        let terminal_output = Command::cargo_bin("magellan")
            .expect("binary should build")
            .args(["render", "--input"])
            .arg(fixture_path(case.path))
            .args(["--format", "terminal"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let terminal = String::from_utf8(terminal_output).expect("terminal output should be utf-8");

        assert!(
            terminal.contains(case.title),
            "terminal render for {} should contain the title",
            case.path
        );
        for marker in case.terminal_markers {
            assert!(
                terminal.contains(marker),
                "terminal render for {} should contain marker {:?}",
                case.path,
                marker
            );
        }

        let markdown_output = Command::cargo_bin("magellan")
            .expect("binary should build")
            .args(["render", "--input"])
            .arg(fixture_path(case.path))
            .args(["--format", "markdown"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let markdown = String::from_utf8(markdown_output).expect("markdown output should be utf-8");

        assert!(
            markdown.contains(&format!("# {}", case.title)),
            "markdown render for {} should contain the title",
            case.path
        );
        for marker in case.markdown_markers {
            assert!(
                markdown.contains(marker),
                "markdown render for {} should contain marker {:?}",
                case.path,
                marker
            );
        }
    }
}

#[test]
fn checked_in_fixtures_render_expected_html_output() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");

    for (index, case) in fixture_cases().into_iter().enumerate() {
        let output_path = temp_dir.path().join(format!("fixture-{index}.html"));

        Command::cargo_bin("magellan")
            .expect("binary should build")
            .args(["render", "--input"])
            .arg(fixture_path(case.path))
            .args(["--format", "html", "--out"])
            .arg(&output_path)
            .assert()
            .success();

        let html = fs::read_to_string(&output_path).expect("html output should be readable");
        assert!(
            html.contains(&format!("<title>{}</title>", case.title)),
            "html render for {} should contain the title element",
            case.path
        );
        assert!(
            !html.contains("https://") && !html.contains("http://"),
            "html render for {} should stay self-contained",
            case.path
        );
        for marker in case.html_markers {
            assert!(
                html.contains(marker),
                "html render for {} should contain marker {:?}",
                case.path,
                marker
            );
        }
        assert!(
            html.contains("class=\"sidebar\""),
            "html render for {} should include the sidebar",
            case.path
        );
        assert!(
            html.contains("class=\"toc-link"),
            "html render for {} should include toc links",
            case.path
        );
        assert!(
            html.contains("data-theme-toggle"),
            "html render for {} should include the theme toggle",
            case.path
        );
    }
}

#[test]
fn session_fixture_html_includes_expected_scroll_structure() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let output_path = temp_dir.path().join("session-scroll.html");

    Command::cargo_bin("magellan")
        .expect("binary should build")
        .args(["render", "--input"])
        .arg(fixture_path("examples/session-walkthrough.json"))
        .args(["--format", "html", "--out"])
        .arg(&output_path)
        .assert()
        .success();

    let html = fs::read_to_string(&output_path).expect("html output should be readable");
    let section_count = html.matches("class=\"section\"").count();
    let verification_count = html.matches("class=\"section verification\"").count();
    let toc_link_count = html.matches("class=\"toc-link").count();
    let diagram_count = html.matches("class=\"diagram\"").count();

    assert_eq!(section_count, 3, "3 regular sections");
    assert_eq!(verification_count, 1, "1 verification section");
    assert_eq!(
        toc_link_count, 5,
        "summary + 3 sections + verification toc links"
    );
    assert_eq!(
        diagram_count, 3,
        "each section with a diagram should render one inline diagram"
    );
    assert!(html.contains("class=\"sidebar\""));
    assert!(html.contains("data-theme-toggle"));
    assert!(html.contains("[data-theme=\"light\"]"));
    assert!(!html.contains("data-book-track"));
    assert!(!html.contains("Book View"));
}
