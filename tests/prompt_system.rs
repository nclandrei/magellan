use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn prompt_question_flow_works_end_to_end_with_real_binary() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let prompt_path = temp_dir.path().join("prompt.txt");

    let output = Command::cargo_bin("magellan")
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
            "why did this validation path change?",
            "--artifact",
            "/tmp/question-flow.json",
            "--render-format",
            "markdown",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    fs::write(&prompt_path, &output).expect("prompt output should be saved");
    let prompt = fs::read_to_string(&prompt_path).expect("prompt output should be readable");

    assert!(prompt.contains("why did this validation path change?"));
    assert!(prompt.contains("magellan validate --input /tmp/question-flow.json"));
    assert!(prompt.contains("magellan render --input /tmp/question-flow.json --format markdown"));
    assert!(prompt.contains("inspect the current diff or commit range"));
}

#[test]
fn prompt_help_examples_include_question_flow() {
    Command::cargo_bin("magellan")
        .expect("binary should build")
        .args(["prompt", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "--question \"why did this API flow change?\"",
        ))
        .stdout(predicate::str::contains("followup"));
}

#[test]
fn prompt_scope_flow_works_end_to_end_with_real_binary() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let prompt_path = temp_dir.path().join("scope-prompt.txt");

    let output = Command::cargo_bin("magellan")
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
            "--artifact",
            "/tmp/scope-flow.json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    fs::write(&prompt_path, &output).expect("prompt output should be saved");
    let prompt = fs::read_to_string(&prompt_path).expect("prompt output should be readable");

    assert!(prompt.contains("keep the walkthrough centered on this scope: backend"));
    assert!(prompt.contains("keep the walkthrough centered on this scope: tests"));
    assert!(prompt.contains("magellan validate --input /tmp/scope-flow.json"));
    assert!(prompt.contains("optimize for another engineer picking up the work quickly"));
}
