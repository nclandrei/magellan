use std::fs;
use std::os::unix::fs::PermissionsExt;

use assert_cmd::Command;
use predicates::prelude::*;

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
    }
  ],
  "verification": {
    "text": [
      "Automated tests covered the regression."
    ]
  }
}"#
}

#[test]
fn schema_command_prints_a_document_schema() {
    Command::cargo_bin("magellan")
        .expect("binary should build")
        .arg("schema")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"title\": \"Document\""));
}

#[test]
fn help_mentions_prompt_workflow() {
    Command::cargo_bin("magellan")
        .expect("binary should build")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "magellan - Render structured technical walkthroughs into terminal, markdown, or HTML output.",
        ))
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("Commands:"))
        .stdout(predicate::str::contains("Normal workflow:"))
        .stdout(predicate::str::contains("Common requests:"))
        .stdout(predicate::str::contains("Session evidence:"))
        .stdout(predicate::str::contains("Fast paths:"))
        .stdout(predicate::str::contains("Prompt workflow:"))
        .stdout(predicate::str::contains(
            "what did the last commit implement?",
        ))
        .stdout(predicate::str::contains("what we built yesterday"))
        .stdout(predicate::str::contains(
            "what did we do on March 15, 2026?",
        ))
        .stdout(predicate::str::contains(
            "what we built in the last session",
        ))
        .stdout(predicate::str::contains(
            "what changed on branch <name>",
        ))
        .stdout(predicate::str::contains(
            "$CODEX_HOME/sessions/YYYY/MM/DD/*.jsonl",
        ))
        .stdout(predicate::str::contains(
            "~/.claude/projects/<workspace-slug>/<session-id>.jsonl",
        ))
        .stdout(predicate::str::contains(
            "In HTML, each section becomes a scrollable block with a sidebar table of contents.",
        ))
        .stdout(predicate::str::contains(
            "examples/session-walkthrough.json",
        ))
        .stdout(predicate::str::contains(
            "magellan prompt --agent-type claude --source branch --goal handoff --scope backend --scope tests",
        ))
        .stdout(predicate::str::contains(
            "magellan guide",
        ));
}

#[test]
fn running_without_arguments_prints_the_top_level_playbook() {
    Command::cargo_bin("magellan")
        .expect("binary should build")
        .assert()
        .success()
        .stdout(predicate::str::contains("Normal workflow:"))
        .stdout(predicate::str::contains("Content rules:"));
}

#[test]
fn guide_command_prints_checked_in_agent_playbook() {
    Command::cargo_bin("magellan")
        .expect("binary should build")
        .arg("guide")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "magellan - Render structured technical walkthroughs into terminal, markdown, or HTML output.",
        ))
        .stdout(predicate::str::contains("Normal workflow"))
        .stdout(predicate::str::contains("Common requests"))
        .stdout(predicate::str::contains("Diagram picking"))
        .stdout(predicate::str::contains("Sidebar:"))
        .stdout(predicate::str::contains(
            "magellan prompt --agent-type codex",
        ));
}

#[test]
fn prompt_command_prints_codex_template() {
    Command::cargo_bin("magellan")
        .expect("binary should build")
        .args(["prompt", "--agent-type", "codex"])
        .assert()
        .success()
        .stdout(predicate::str::contains("You are Codex."))
        .stdout(predicate::str::contains(
            "focused on this topic: what we built in this task",
        ))
        .stdout(predicate::str::contains("magellan schema"))
        .stdout(predicate::str::contains(
            "inspect persisted session transcripts, tool actions, and timestamps",
        ))
        .stdout(predicate::str::contains(
            "label any diff or commit reconstruction as fallback evidence",
        ))
        .stdout(predicate::str::contains(
            "produce a broad technical walkthrough that covers the full change without drifting into fluff",
        ))
        .stdout(predicate::str::contains(
            "Each section becomes a scrollable block in the HTML sidebar layout, so keep one idea per section.",
        ))
        .stdout(predicate::str::contains(
            "magellan go --input /tmp/magellan.json",
        ));
}

#[test]
fn prompt_help_mentions_source_and_goal_options() {
    Command::cargo_bin("magellan")
        .expect("binary should build")
        .args(["prompt", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--source <SOURCE>"))
        .stdout(predicate::str::contains("--goal <GOAL>"))
        .stdout(predicate::str::contains("--question <QUESTION>"))
        .stdout(predicate::str::contains("--scope <SCOPE>"))
        .stdout(predicate::str::contains("Goals:"))
        .stdout(predicate::str::contains("Sources:"))
        .stdout(predicate::str::contains("Session-source reminders:"))
        .stdout(predicate::str::contains("sessions-index.json"))
        .stdout(predicate::str::contains("Diagram picking:"))
        .stdout(predicate::str::contains(
            "timeline         Ordered work, debugging steps, or event progression",
        ))
        .stdout(predicate::str::contains("Need the full Magellan playbook:"))
        .stdout(predicate::str::contains("magellan --help"))
        .stdout(predicate::str::contains(
            "examples/branch-handoff-timeline.json",
        ));
}

#[test]
fn schema_help_explains_the_contract_workflow() {
    Command::cargo_bin("magellan")
        .expect("binary should build")
        .args(["schema", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Use this when an agent needs the exact payload contract before writing JSON.",
        ))
        .stdout(predicate::str::contains(
            "magellan schema > /tmp/magellan-schema.json",
        ))
        .stdout(predicate::str::contains(
            "summary` with 1-2 short paragraphs",
        ));
}

#[test]
fn example_help_points_to_presets_and_realistic_references() {
    Command::cargo_bin("magellan")
        .expect("binary should build")
        .args(["example", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Starter presets:"))
        .stdout(predicate::str::contains(
            "followup      Narrower follow-up explainer that answers one focused implementation question",
        ))
        .stdout(predicate::str::contains(
            "timeline      Ordered sequence when implementation order matters",
        ))
        .stdout(predicate::str::contains(
            "examples/followup-validation-question.json",
        ));
}

#[test]
fn example_command_prints_followup_preset() {
    Command::cargo_bin("magellan")
        .expect("binary should build")
        .args(["example", "--preset", "followup"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"title\": \"Follow-up: why the retry guard moved into the background worker\"",
        ))
        .stdout(predicate::str::contains(
            "\"title\": \"Why the worker owns retries now\"",
        ))
        .stdout(predicate::str::contains("\"verification\": {"));
}

#[test]
fn validate_help_explains_the_pipeline() {
    Command::cargo_bin("magellan")
        .expect("binary should build")
        .args(["validate", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Validate before rendering."))
        .stdout(predicate::str::contains(
            "magellan validate --input examples/session-walkthrough.json",
        ))
        .stdout(predicate::str::contains(
            "Validation checks pacing and diagram structure.",
        ));
}

#[test]
fn render_help_explains_formats_and_diagram_types() {
    Command::cargo_bin("magellan")
        .expect("binary should build")
        .args(["render", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Format guide:"))
        .stdout(predicate::str::contains(
            "timeline         Ordered work, debugging steps, or event progression",
        ))
        .stdout(predicate::str::contains(
            "magellan render --input examples/followup-validation-question.json --format html --open",
        ))
        .stdout(predicate::str::contains(
            "HTML reports use a sidebar scroll layout with a table of contents and dark/light theme toggle.",
        ))
        .stdout(predicate::str::contains("`--open` requires `--format html`."));
}

#[test]
fn prompt_command_can_customize_topic_source_goal_artifact_and_focus() {
    Command::cargo_bin("magellan")
        .expect("binary should build")
        .args([
            "prompt",
            "--agent-type",
            "claude",
            "--source",
            "pr",
            "--goal",
            "handoff",
            "--topic",
            "what we built in this session",
            "--artifact",
            "/tmp/session-walkthrough.json",
            "--render-format",
            "markdown",
            "--focus",
            "behavior",
            "--focus",
            "verification",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Use Magellan to produce a compact walkthrough focused on this topic: what we built in this session",
        ))
        .stdout(predicate::str::contains(
            "inspect the pull request description, review comments, and diff before writing the walkthrough",
        ))
        .stdout(predicate::str::contains(
            "optimize for another engineer picking up the work quickly, including decisions and verification",
        ))
        .stdout(predicate::str::contains(
            "magellan go --input /tmp/session-walkthrough.json",
        ))
        .stdout(predicate::str::contains(
            "- prioritize what the system now does differently for the user or caller",
        ))
        .stdout(predicate::str::contains(
            "- give verification its own section and be explicit about evidence",
        ));
}

#[test]
fn prompt_command_can_target_followup_goal() {
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
            "why did the API validation flow change?",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "focused on this topic: why did the API validation flow change?",
        ))
        .stdout(predicate::str::contains(
            "inspect the current diff or commit range and use it as the main evidence for what changed",
        ))
        .stdout(predicate::str::contains(
            "answer a narrower follow-up question and stay tighter than a full walkthrough",
        ))
        .stdout(predicate::str::contains(
            "2-4 focused steps centered on the specific question",
        ))
        .stdout(predicate::str::contains(
            "make sure the walkthrough answers this explicitly near the top: why did the API validation flow change?",
        ))
        .stdout(predicate::str::contains(
            "use `flow` for branching logic, validation gates, or state movement",
        ))
        .stdout(predicate::str::contains(
            "use `before_after` when the main point is how behavior changed for the user or caller",
        ));
}

#[test]
fn prompt_command_without_question_mentions_inferred_framing() {
    Command::cargo_bin("magellan")
        .expect("binary should build")
        .args(["prompt", "--agent-type", "codex"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "no explicit question was provided; infer the most useful framing from the topic and goal",
        ));
}

#[test]
fn prompt_command_can_constrain_scope() {
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
            "--scope",
            "tests",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "no explicit question was provided; infer the most useful framing from the topic and goal",
        ))
        .stdout(predicate::str::contains(
            "keep the walkthrough centered on this scope: backend",
        ))
        .stdout(predicate::str::contains(
            "keep the walkthrough centered on this scope: tests",
        ));
}

#[test]
fn prompt_command_with_handoff_and_timeline_focus_recommends_timeline_sections() {
    Command::cargo_bin("magellan")
        .expect("binary should build")
        .args([
            "prompt",
            "--agent-type",
            "codex",
            "--source",
            "branch",
            "--goal",
            "handoff",
            "--focus",
            "timeline",
            "--focus",
            "architecture",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "for this artifact, include a `timeline` section when implementation order helps another engineer pick up the work",
        ))
        .stdout(predicate::str::contains(
            "architecture-focused explanations usually benefit from a `component_graph` section",
        ));
}

#[test]
fn validate_command_accepts_a_valid_payload() {
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

#[test]
fn example_command_prints_a_valid_starter_payload() {
    Command::cargo_bin("magellan")
        .expect("binary should build")
        .args(["example", "--preset", "timeline"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"title\": \"Search flow cleanup\"",
        ));
}

#[test]
fn render_command_can_write_html_to_a_file() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let input_path = temp_dir.path().join("payload.json");
    let output_path = temp_dir.path().join("magellan.html");
    fs::write(&input_path, sample_payload()).expect("payload should be written");

    Command::cargo_bin("magellan")
        .expect("binary should build")
        .args(["render", "--input"])
        .arg(&input_path)
        .args(["--format", "html", "--out"])
        .arg(&output_path)
        .assert()
        .success()
        .stdout("");

    let rendered = fs::read_to_string(&output_path).expect("rendered html should be readable");
    assert!(rendered.contains("<!DOCTYPE html>"));
    assert!(rendered.contains("Order validation moved earlier"));
    assert!(rendered.contains("<svg viewBox="));
    assert!(rendered.contains("class=\"sidebar\""));
    assert!(rendered.contains("class=\"toc-link"));
    assert!(rendered.contains("id=\"section-1\""));
    assert!(rendered.contains("data-theme-toggle"));
    assert!(rendered.contains("ASCII fallback"));
    assert!(!rendered.contains("cdn.jsdelivr"));
}

#[test]
fn render_open_uses_explicit_output_path_and_invokes_opener() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let input_path = temp_dir.path().join("payload.json");
    let output_path = temp_dir.path().join("magellan.html");
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
    .expect("opener script should be written");
    let mut permissions = fs::metadata(&opener_path)
        .expect("opener metadata should exist")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&opener_path, permissions).expect("opener should be executable");

    let assert = Command::cargo_bin("magellan")
        .expect("binary should build")
        .env("MAGELLAN_OPEN_BIN", &opener_path)
        .args(["render", "--input"])
        .arg(&input_path)
        .args(["--format", "html", "--out"])
        .arg(&output_path)
        .arg("--open")
        .assert()
        .success();

    let stdout =
        String::from_utf8(assert.get_output().stdout.clone()).expect("stdout should be utf8");
    assert!(stdout.contains(&format!("Opened {}\n", output_path.display())));
    assert_eq!(
        fs::read_to_string(&opened_path).expect("opener output should exist"),
        output_path.display().to_string()
    );
    assert!(
        fs::read_to_string(&output_path)
            .expect("html should exist")
            .contains("<!DOCTYPE html>")
    );
}

#[test]
fn render_open_requires_html_output() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let input_path = temp_dir.path().join("payload.json");
    fs::write(&input_path, sample_payload()).expect("payload should be written");

    Command::cargo_bin("magellan")
        .expect("binary should build")
        .args(["render", "--input"])
        .arg(&input_path)
        .args(["--format", "terminal", "--open"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--open currently requires --format html",
        ));
}
