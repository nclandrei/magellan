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
                "Book View",
                "Overview",
                "Sequence diagram",
                "Flow diagram",
                "Component diagram",
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
                "Book View",
                "Overview",
                "Timeline",
                "Flow diagram",
                "Verification",
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
                "Book View",
                "Overview",
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
            html.contains("data-book-track"),
            "html render for {} should include the paged book track",
            case.path
        );
        assert!(
            html.contains("data-view=\"overview\" hidden"),
            "html render for {} should include the overview view",
            case.path
        );
    }
}

#[test]
fn session_fixture_html_includes_expected_book_paging_structure() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let output_path = temp_dir.path().join("session-book.html");

    Command::cargo_bin("magellan")
        .expect("binary should build")
        .args(["render", "--input"])
        .arg(fixture_path("examples/session-walkthrough.json"))
        .args(["--format", "html", "--out"])
        .arg(&output_path)
        .assert()
        .success();

    let html = fs::read_to_string(&output_path).expect("html output should be readable");
    let page_count = html.matches("data-page-title=").count();
    let dot_count = html.matches("data-page-dot=").count();

    assert_eq!(page_count, 5, "summary + 3 sections + verification");
    assert_eq!(dot_count, 5, "book navigation should mirror the page count");
    assert!(html.contains("Book view shows one technical slice at a time."));
}
