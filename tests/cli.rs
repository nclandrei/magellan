use std::fs;

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
        .stdout("Payload is valid.\n");
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
    assert!(rendered.contains("mermaid"));
}
