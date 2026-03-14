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
    assert!(rendered.contains("<svg viewBox="));
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
