use std::fmt;

use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Document {
    pub title: String,
    pub summary: Vec<String>,
    pub sections: Vec<Section>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification: Option<Verification>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Section {
    pub title: String,
    pub text: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagram: Option<Diagram>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Verification {
    pub text: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Diagram {
    Sequence {
        nodes: Vec<String>,
        edges: Vec<Edge>,
    },
    Flow {
        nodes: Vec<String>,
        edges: Vec<Edge>,
    },
    ComponentGraph {
        nodes: Vec<String>,
        edges: Vec<Edge>,
    },
    Timeline {
        events: Vec<TimelineEvent>,
    },
    BeforeAfter(BeforeAfterDiagram),
    LayerStack {
        layers: Vec<String>,
    },
    StateMachine {
        states: Vec<String>,
        transitions: Vec<Edge>,
    },
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Edge {
    pub from: String,
    pub to: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TimelineEvent {
    pub label: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BeforeAfterDiagram {
    pub before: Vec<String>,
    pub after: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    messages: Vec<String>,
}

impl ValidationError {
    fn new(messages: Vec<String>) -> Self {
        Self { messages }
    }

    #[cfg(test)]
    pub fn messages(&self) -> &[String] {
        &self.messages
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "payload validation failed:")?;
        for message in &self.messages {
            writeln!(f, "- {message}")?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationError {}

impl Document {
    pub fn validate(&self) -> Result<()> {
        let mut errors = Vec::new();

        validate_non_empty("title", &self.title, &mut errors);
        validate_paragraphs("summary", &self.summary, 1, 2, &mut errors);

        if !(1..=6).contains(&self.sections.len()) {
            errors.push(format!(
                "sections must contain between 1 and 6 entries, found {}",
                self.sections.len()
            ));
        }

        for (index, section) in self.sections.iter().enumerate() {
            section.validate(index, &mut errors);
        }

        if let Some(verification) = &self.verification {
            validate_paragraphs("verification.text", &verification.text, 1, 3, &mut errors);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError::new(errors).into())
        }
    }
}

impl Section {
    fn validate(&self, index: usize, errors: &mut Vec<String>) {
        let section_index = index + 1;
        validate_non_empty(&format!("sections[{index}].title"), &self.title, errors);
        validate_paragraphs(&format!("sections[{index}].text"), &self.text, 1, 3, errors);

        if let Some(diagram) = &self.diagram {
            diagram.validate(section_index, errors);
        }
    }
}

impl Diagram {
    fn validate(&self, section_index: usize, errors: &mut Vec<String>) {
        match self {
            Diagram::Sequence { nodes, edges }
            | Diagram::Flow { nodes, edges }
            | Diagram::ComponentGraph { nodes, edges } => {
                if nodes.len() < 2 {
                    errors.push(format!(
                        "sections[{section_index}].diagram requires at least 2 nodes"
                    ));
                }
                for (index, node) in nodes.iter().enumerate() {
                    validate_non_empty(
                        &format!("sections[{section_index}].diagram.nodes[{index}]"),
                        node,
                        errors,
                    );
                }
                if edges.is_empty() {
                    errors.push(format!(
                        "sections[{section_index}].diagram requires at least 1 edge"
                    ));
                }
                for (index, edge) in edges.iter().enumerate() {
                    edge.validate(section_index, index, errors);
                }
            }
            Diagram::Timeline { events } => {
                if events.is_empty() {
                    errors.push(format!(
                        "sections[{section_index}].diagram.timeline requires at least 1 event"
                    ));
                }
                for (index, event) in events.iter().enumerate() {
                    validate_non_empty(
                        &format!("sections[{section_index}].diagram.events[{index}].label"),
                        &event.label,
                        errors,
                    );
                    validate_non_empty(
                        &format!("sections[{section_index}].diagram.events[{index}].detail"),
                        &event.detail,
                        errors,
                    );
                }
            }
            Diagram::BeforeAfter(before_after) => {
                validate_paragraphs(
                    &format!("sections[{section_index}].diagram.before"),
                    &before_after.before,
                    1,
                    5,
                    errors,
                );
                validate_paragraphs(
                    &format!("sections[{section_index}].diagram.after"),
                    &before_after.after,
                    1,
                    5,
                    errors,
                );
            }
            Diagram::LayerStack { layers } => {
                if layers.len() < 2 {
                    errors.push(format!(
                        "sections[{section_index}].diagram.layers requires at least 2 layers"
                    ));
                }
                for (index, layer) in layers.iter().enumerate() {
                    validate_non_empty(
                        &format!("sections[{section_index}].diagram.layers[{index}]"),
                        layer,
                        errors,
                    );
                }
            }
            Diagram::Table { headers, rows } => {
                if headers.len() < 2 {
                    errors.push(format!(
                        "sections[{section_index}].diagram.headers requires at least 2 columns"
                    ));
                }
                for (index, header) in headers.iter().enumerate() {
                    validate_non_empty(
                        &format!("sections[{section_index}].diagram.headers[{index}]"),
                        header,
                        errors,
                    );
                }
                if rows.is_empty() {
                    errors.push(format!(
                        "sections[{section_index}].diagram.rows requires at least 1 row"
                    ));
                }
                for (row_index, row) in rows.iter().enumerate() {
                    if row.len() != headers.len() {
                        errors.push(format!(
                            "sections[{section_index}].diagram.rows[{row_index}] has {} cells but headers has {}",
                            row.len(),
                            headers.len()
                        ));
                    }
                }
            }
            Diagram::StateMachine {
                states,
                transitions,
            } => {
                if states.len() < 2 {
                    errors.push(format!(
                        "sections[{section_index}].diagram requires at least 2 states"
                    ));
                }
                for (index, state) in states.iter().enumerate() {
                    validate_non_empty(
                        &format!("sections[{section_index}].diagram.states[{index}]"),
                        state,
                        errors,
                    );
                }
                if transitions.is_empty() {
                    errors.push(format!(
                        "sections[{section_index}].diagram requires at least 1 transition"
                    ));
                }
                for (index, edge) in transitions.iter().enumerate() {
                    edge.validate(section_index, index, errors);
                }
            }
        }
    }
}

impl Edge {
    fn validate(&self, section_index: usize, edge_index: usize, errors: &mut Vec<String>) {
        validate_non_empty(
            &format!("sections[{section_index}].diagram.edges[{edge_index}].from"),
            &self.from,
            errors,
        );
        validate_non_empty(
            &format!("sections[{section_index}].diagram.edges[{edge_index}].to"),
            &self.to,
            errors,
        );
        if let Some(label) = &self.label {
            validate_non_empty(
                &format!("sections[{section_index}].diagram.edges[{edge_index}].label"),
                label,
                errors,
            );
        }
    }
}

fn validate_non_empty(field_name: &str, value: &str, errors: &mut Vec<String>) {
    if value.trim().is_empty() {
        errors.push(format!("{field_name} must not be empty"));
    }
}

fn validate_paragraphs(
    field_name: &str,
    paragraphs: &[String],
    min: usize,
    max: usize,
    errors: &mut Vec<String>,
) {
    if !(min..=max).contains(&paragraphs.len()) {
        errors.push(format!(
            "{field_name} must contain between {min} and {max} entries, found {}",
            paragraphs.len()
        ));
    }

    for (index, paragraph) in paragraphs.iter().enumerate() {
        validate_non_empty(&format!("{field_name}[{index}]"), paragraph, errors);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_document() -> Document {
        Document {
            title: "Order submission now validates in the UI".into(),
            summary: vec![
                "We moved validation earlier so bad requests fail before the network call.".into(),
            ],
            sections: vec![
                Section {
                    title: "New request flow".into(),
                    text: vec![
                        "The form validates locally before reaching the API.".into(),
                        "Valid submissions still continue to the backend.".into(),
                    ],
                    diagram: Some(Diagram::Sequence {
                        nodes: vec!["User".into(), "Form".into(), "API".into()],
                        edges: vec![
                            Edge {
                                from: "User".into(),
                                to: "Form".into(),
                                label: Some("submit".into()),
                            },
                            Edge {
                                from: "Form".into(),
                                to: "API".into(),
                                label: Some("valid request".into()),
                            },
                        ],
                    }),
                },
                Section {
                    title: "Verification".into(),
                    text: vec!["We covered the regression with an integration test.".into()],
                    diagram: None,
                },
            ],
            verification: Some(Verification {
                text: vec!["Manual verification and automated tests passed.".into()],
            }),
        }
    }

    #[test]
    fn validates_a_reasonable_payload() {
        let document = sample_document();
        assert!(document.validate().is_ok());
    }

    #[test]
    fn rejects_payloads_that_break_pacing_rules() {
        let mut document = sample_document();
        document.summary = vec![];
        document.sections[0].text = vec![];

        let error = document.validate().expect_err("payload should be invalid");
        let error = error
            .downcast_ref::<ValidationError>()
            .expect("validation error should downcast");

        assert!(
            error
                .messages()
                .iter()
                .any(|message| message.contains("summary must contain between 1 and 2 entries"))
        );
        assert!(error.messages().iter().any(|message| {
            message.contains("sections[0].text must contain between 1 and 3 entries")
        }));
    }
}
