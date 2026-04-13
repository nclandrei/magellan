use std::fs;
use std::os::unix::fs::PermissionsExt;

use assert_cmd::Command;
use predicates::prelude::*;

// ---------------------------------------------------------------------------
// Help text: mandatory render enforcement
// ---------------------------------------------------------------------------

#[test]
fn help_text_opens_with_mandatory_go_instruction() {
    Command::cargo_bin("magellan")
        .expect("binary should build")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "MANDATORY: Your job is to produce rendered artifacts, not prose explanations.",
        ))
        .stdout(predicate::str::contains(
            "After writing JSON, you MUST run:",
        ))
        .stdout(predicate::str::contains(
            "magellan go --input /tmp/magellan.json",
        ));
}

#[test]
fn help_text_forbids_skipping_and_prose_substitution() {
    Command::cargo_bin("magellan")
        .expect("binary should build")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Do not skip it."))
        .stdout(predicate::str::contains(
            "Do not summarize in chat instead.",
        ))
        .stdout(predicate::str::contains(
            "The rendered HTML and markdown files ARE the deliverable.",
        ));
}

#[test]
fn help_text_workflow_uses_single_go_step() {
    Command::cargo_bin("magellan")
        .expect("binary should build")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Validate and render. This step is mandatory, do not skip it.",
        ))
        .stdout(predicate::str::contains(
            "magellan go --input /tmp/magellan.json",
        ));
}

#[test]
fn help_text_commands_list_includes_go() {
    Command::cargo_bin("magellan")
        .expect("binary should build")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "go        Validate, render HTML, open it, and write markdown",
        ));
}

#[test]
fn help_text_render_targets_references_go() {
    Command::cargo_bin("magellan")
        .expect("binary should build")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Use `magellan go` to validate and produce both HTML and markdown in one step:",
        ));
}

#[test]
fn help_text_fast_paths_include_go() {
    Command::cargo_bin("magellan")
        .expect("binary should build")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Validate and render (the mandatory final step):",
        ))
        .stdout(predicate::str::contains(
            "magellan go --input /tmp/magellan.json",
        ));
}

#[test]
fn help_text_ends_with_go_reminder() {
    Command::cargo_bin("magellan")
        .expect("binary should build")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "REMINDER: You MUST run `magellan go` after creating the JSON.",
        ));
}

#[test]
fn guide_command_matches_help_text() {
    let help_output = Command::cargo_bin("magellan")
        .expect("binary should build")
        .arg("--help")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let guide_output = Command::cargo_bin("magellan")
        .expect("binary should build")
        .arg("guide")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert_eq!(
        String::from_utf8(help_output).expect("utf-8"),
        String::from_utf8(guide_output).expect("utf-8"),
        "--help and guide should print identical content"
    );
}

// ---------------------------------------------------------------------------
// `magellan go` command: validates + renders HTML + writes markdown
// ---------------------------------------------------------------------------

#[test]
fn go_command_produces_html_and_markdown() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let input_path = temp_dir.path().join("payload.json");
    let html_path = temp_dir.path().join("output.html");
    let md_path = temp_dir.path().join("output.md");
    let opened_path = temp_dir.path().join("opened.txt");
    let opener_path = temp_dir.path().join("fake-open.sh");

    fs::write(&input_path, sample_payload()).expect("payload should be written");
    fs::write(
        &opener_path,
        format!(
            "#!/bin/sh\nprintf '%s' \"$1\" > {}\n",
            opened_path.display()
        ),
    )
    .expect("opener should be written");
    let mut perms = fs::metadata(&opener_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&opener_path, perms).unwrap();

    Command::cargo_bin("magellan")
        .expect("binary should build")
        .env("MAGELLAN_OPEN_BIN", &opener_path)
        .args(["go", "--input"])
        .arg(&input_path)
        .arg("--out")
        .arg(&html_path)
        .arg("--markdown-out")
        .arg(&md_path)
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "Opened {}",
            html_path.display()
        )))
        .stdout(predicate::str::contains(format!(
            "Wrote {}",
            md_path.display()
        )));

    let html = fs::read_to_string(&html_path).expect("html should be readable");
    let md = fs::read_to_string(&md_path).expect("markdown should be readable");

    assert!(
        html.contains("<!DOCTYPE html>"),
        "html file should contain HTML"
    );
    assert!(
        html.contains("Order validation moved earlier"),
        "html should contain the title"
    );
    assert!(
        md.contains("# Order validation moved earlier"),
        "markdown should contain the title"
    );
    assert!(
        md.contains("sequenceDiagram"),
        "markdown should contain mermaid diagrams"
    );
}

#[test]
fn go_command_derives_markdown_path_from_input() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let input_path = temp_dir.path().join("my-walkthrough.json");
    let expected_md = temp_dir.path().join("my-walkthrough.md");
    let html_path = temp_dir.path().join("output.html");
    let opener_path = temp_dir.path().join("fake-open.sh");

    fs::write(&input_path, sample_payload()).expect("payload should be written");
    fs::write(&opener_path, "#!/bin/sh\n").expect("opener should be written");
    let mut perms = fs::metadata(&opener_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&opener_path, perms).unwrap();

    Command::cargo_bin("magellan")
        .expect("binary should build")
        .env("MAGELLAN_OPEN_BIN", &opener_path)
        .args(["go", "--input"])
        .arg(&input_path)
        .arg("--out")
        .arg(&html_path)
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "Wrote {}",
            expected_md.display()
        )));

    assert!(
        expected_md.exists(),
        "markdown should be derived from input path"
    );
}

#[test]
fn go_command_rejects_invalid_payload() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let input_path = temp_dir.path().join("bad.json");
    fs::write(&input_path, r#"{"title":"X","summary":[],"sections":[]}"#)
        .expect("payload should be written");

    Command::cargo_bin("magellan")
        .expect("binary should build")
        .args(["go", "--input"])
        .arg(&input_path)
        .assert()
        .failure();
}

#[test]
fn go_help_explains_the_workflow() {
    Command::cargo_bin("magellan")
        .expect("binary should build")
        .args(["go", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Validate, render HTML, open it, and write markdown",
        ))
        .stdout(predicate::str::contains(
            "magellan go --input /tmp/magellan.json",
        ))
        .stdout(predicate::str::contains(
            "Agents should always use `go` instead of separate validate + render calls.",
        ));
}

// ---------------------------------------------------------------------------
// Validate breadcrumb: points to go
// ---------------------------------------------------------------------------

#[test]
fn validate_output_points_to_go_command() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let input_path = temp_dir.path().join("payload.json");
    fs::write(&input_path, sample_payload()).expect("payload should be written");

    Command::cargo_bin("magellan")
        .expect("binary should build")
        .args(["validate", "--input"])
        .arg(&input_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Payload is valid. Now render it:"))
        .stdout(predicate::str::contains("magellan go --input"));
}

// ---------------------------------------------------------------------------
// Prompt template: uses go command
// ---------------------------------------------------------------------------

#[test]
fn prompt_template_uses_go_command() {
    Command::cargo_bin("magellan")
        .expect("binary should build")
        .args(["prompt", "--agent-type", "codex"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "magellan go --input /tmp/magellan.json",
        ))
        .stdout(predicate::str::contains(
            "This validates, renders HTML (opens it in the browser), and writes markdown.",
        ));
}

#[test]
fn prompt_template_forbids_prose_summary() {
    Command::cargo_bin("magellan")
        .expect("binary should build")
        .args(["prompt", "--agent-type", "claude"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Do not describe the walkthrough in prose and then ask if the user wants a report.",
        ))
        .stdout(predicate::str::contains(
            "The rendered artifacts are always the expected output.",
        ));
}

#[test]
fn prompt_template_ends_with_required_go_step() {
    Command::cargo_bin("magellan")
        .expect("binary should build")
        .args(["prompt", "--agent-type", "codex"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Required final step:"))
        .stdout(predicate::str::contains(
            "magellan go --input /tmp/magellan.json",
        ));
}

#[test]
fn prompt_with_custom_artifact_uses_go_with_that_path() {
    Command::cargo_bin("magellan")
        .expect("binary should build")
        .args([
            "prompt",
            "--agent-type",
            "claude",
            "--artifact",
            "/tmp/my-session.json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "magellan go --input /tmp/my-session.json",
        ));
}

#[test]
fn go_command_appears_twice_in_prompt() {
    let output = Command::cargo_bin("magellan")
        .expect("binary should build")
        .args(["prompt", "--agent-type", "codex"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let prompt = String::from_utf8(output).expect("prompt should be utf-8");
    let count = prompt
        .matches("magellan go --input /tmp/magellan.json")
        .count();

    assert_eq!(
        count, 2,
        "go command should appear in both step 5 and the required-final-step footer"
    );
}

// ---------------------------------------------------------------------------
// End-to-end: every goal uses go
// ---------------------------------------------------------------------------

#[test]
fn followup_prompt_uses_go() {
    Command::cargo_bin("magellan")
        .expect("binary should build")
        .args([
            "prompt",
            "--agent-type",
            "codex",
            "--source",
            "diff",
            "--goal",
            "followup",
            "--question",
            "what did the last commit implement?",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("magellan go --input"));
}

#[test]
fn handoff_prompt_uses_go() {
    Command::cargo_bin("magellan")
        .expect("binary should build")
        .args([
            "prompt",
            "--agent-type",
            "claude",
            "--source",
            "branch",
            "--goal",
            "handoff",
            "--scope",
            "backend",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("magellan go --input"))
        .stdout(predicate::str::contains("Required final step"));
}

#[test]
fn walkthrough_prompt_uses_go() {
    Command::cargo_bin("magellan")
        .expect("binary should build")
        .args([
            "prompt",
            "--agent-type",
            "codex",
            "--source",
            "session",
            "--goal",
            "walkthrough",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("magellan go --input"));
}

// ---------------------------------------------------------------------------
// Render --markdown-out still works independently
// ---------------------------------------------------------------------------

#[test]
fn render_markdown_out_writes_alongside_html() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let input_path = temp_dir.path().join("payload.json");
    let html_path = temp_dir.path().join("output.html");
    let md_path = temp_dir.path().join("output.md");

    fs::write(&input_path, sample_payload()).expect("payload should be written");

    Command::cargo_bin("magellan")
        .expect("binary should build")
        .args(["render", "--input"])
        .arg(&input_path)
        .args(["--format", "html", "--out"])
        .arg(&html_path)
        .arg("--markdown-out")
        .arg(&md_path)
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "Wrote {}",
            md_path.display()
        )));

    assert!(html_path.exists(), "html should be written");
    assert!(md_path.exists(), "markdown should be written");
}

// ---------------------------------------------------------------------------
// Docs stay in sync with the real renderer
// ---------------------------------------------------------------------------

#[test]
fn readme_does_not_advertise_book_view() {
    let readme = fs::read_to_string("README.md").expect("README.md should be readable");
    let lower = readme.to_lowercase();

    assert!(
        !lower.contains("book view"),
        "README.md must not advertise the removed book view"
    );
    assert!(
        readme.contains("sidebar"),
        "README.md should describe the current sidebar HTML layout"
    );
}

#[test]
fn readme_documents_the_mandatory_go_command() {
    let readme = fs::read_to_string("README.md").expect("README.md should be readable");

    assert!(
        readme.contains("magellan go"),
        "README.md must document the `magellan go` command"
    );
    assert!(
        readme.contains("go        Validate, render HTML, open it, and write markdown"),
        "README.md command list should describe what `go` does"
    );
    assert!(
        readme.contains("magellan go --input /tmp/magellan.json"),
        "README.md should include a concrete `magellan go` invocation example"
    );
}

#[test]
fn agent_docs_mention_go_in_development_commands() {
    for path in ["CLAUDE.md", "AGENTS.md"] {
        let contents =
            fs::read_to_string(path).unwrap_or_else(|_| panic!("{path} should be readable"));

        assert!(
            contents.contains("cargo run -- go --input"),
            "{path} Development Commands should teach the `go` feedback loop"
        );
    }
}

#[test]
fn agent_docs_do_not_contradict_renderer_with_book_view() {
    for path in ["CLAUDE.md", "AGENTS.md"] {
        let contents =
            fs::read_to_string(path).unwrap_or_else(|_| panic!("{path} should be readable"));
        let lower = contents.to_lowercase();

        assert!(
            !lower.contains("book view") && !lower.contains("book-mode"),
            "{path} must not reference the removed book view / book mode"
        );
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn sample_payload() -> &'static str {
    r#"{
  "title": "Order validation moved earlier",
  "summary": [
    "The UI now validates required fields before the network request."
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
    },
    {
      "title": "Why it matters",
      "text": [
        "Feedback arrives before a round-trip, so broken submissions never reach the backend."
      ]
    }
  ],
  "verification": {
    "text": [
      "Automated tests covered the regression."
    ]
  }
}"#
}
