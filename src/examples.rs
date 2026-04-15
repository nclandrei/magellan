use crate::model::{
    BeforeAfterDiagram, Diagram, Document, Edge, Section, TimelineEvent, Verification,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExamplePreset {
    Walkthrough,
    Timeline,
    BeforeAfter,
    Followup,
    Handoff,
}

pub fn example_document(preset: ExamplePreset) -> Document {
    match preset {
        ExamplePreset::Walkthrough => walkthrough_example(),
        ExamplePreset::Timeline => timeline_example(),
        ExamplePreset::BeforeAfter => before_after_example(),
        ExamplePreset::Followup => followup_example(),
        ExamplePreset::Handoff => handoff_example(),
    }
}

fn walkthrough_example() -> Document {
    Document {
        title: "Order validation moved earlier".into(),
        summary: vec![
            "The UI now validates required fields before sending the network request.".into(),
            "That shifts failures closer to the user and keeps obviously bad payloads away from the API.".into(),
        ],
        sections: vec![
            Section {
                title: "New request flow".into(),
                text: vec![
                    "The form validates locally before any network call happens.".into(),
                    "Only valid submissions continue to the backend.".into(),
                ],
                diagram: Some(Diagram::Sequence {
                    nodes: vec!["User".into(), "Order Form".into(), "API".into()],
                    edges: vec![
                        Edge {
                            from: "User".into(),
                            to: "Order Form".into(),
                            label: Some("submit".into()),
                        },
                        Edge {
                            from: "Order Form".into(),
                            to: "API".into(),
                            label: Some("valid request".into()),
                        },
                    ],
                }),
                commit: None,
                files: vec![],
            },
            Section {
                title: "Why this matters".into(),
                text: vec![
                    "Inline validation gives immediate feedback instead of failing after a round-trip.".into(),
                    "The backend still sees the same valid requests, but avoids obvious noise.".into(),
                ],
                diagram: Some(Diagram::Flow {
                    nodes: vec![
                        "Invalid input".into(),
                        "UI error".into(),
                        "Valid input".into(),
                        "API".into(),
                    ],
                    edges: vec![
                        Edge {
                            from: "Invalid input".into(),
                            to: "UI error".into(),
                            label: Some("stop locally".into()),
                        },
                        Edge {
                            from: "Valid input".into(),
                            to: "API".into(),
                            label: Some("continue".into()),
                        },
                    ],
                }),
                commit: None,
                files: vec![],
            },
        ],
        verification: None,
        repo: None,
    }
}

fn timeline_example() -> Document {
    Document {
        title: "Search flow cleanup".into(),
        summary: vec![
            "We simplified the search flow in a few discrete steps rather than one opaque rewrite.".into(),
        ],
        sections: vec![
            Section {
                title: "Implementation timeline".into(),
                text: vec![
                    "The work moved from request shaping, to UI loading state cleanup, to final verification.".into(),
                ],
                diagram: Some(Diagram::Timeline {
                    events: vec![
                        TimelineEvent {
                            label: "Step 1".into(),
                            detail: "Normalize search params before the API call.".into(),
                        },
                        TimelineEvent {
                            label: "Step 2".into(),
                            detail: "Show a single loading state instead of overlapping spinners.".into(),
                        },
                        TimelineEvent {
                            label: "Step 3".into(),
                            detail: "Add a regression test for stale result rendering.".into(),
                        },
                    ],
                }),
                commit: None,
                files: vec![],
            },
            Section {
                title: "Why the ordering mattered".into(),
                text: vec![
                    "Request shaping had to land first so the UI cleanup could rely on a single stable response shape.".into(),
                    "Only after that did the regression test have a deterministic target to assert against.".into(),
                ],
                diagram: None,
                commit: None,
                files: vec![],
            },
        ],
        verification: None,
        repo: None,
    }
}

fn before_after_example() -> Document {
    Document {
        title: "Error handling before and after".into(),
        summary: vec![
            "The new flow turns a late backend failure into an earlier, clearer frontend validation step.".into(),
        ],
        sections: vec![
            Section {
                title: "Behavior change".into(),
                text: vec![
                    "This is useful when the main point is how the user experience changed.".into(),
                ],
                diagram: Some(Diagram::BeforeAfter(BeforeAfterDiagram {
                    before: vec![
                        "User submits incomplete form".into(),
                        "API rejects the payload".into(),
                        "User sees a generic error".into(),
                    ],
                    after: vec![
                        "User submits incomplete form".into(),
                        "UI highlights the missing field".into(),
                        "API only receives valid requests".into(),
                    ],
                })),
                commit: None,
                files: vec![],
            },
            Section {
                title: "Why this matters".into(),
                text: vec![
                    "Moving the error to the UI makes feedback immediate and keeps the backend logs focused on real failures.".into(),
                ],
                diagram: None,
                commit: None,
                files: vec![],
            },
        ],
        verification: None,
        repo: None,
    }
}

fn followup_example() -> Document {
    Document {
        title: "Follow-up: why the retry guard moved into the background worker".into(),
        summary: vec![
            "The retry guard moved from the API handler into the background worker so duplicate retry logic now lives next to the queue state it depends on.".into(),
            "That makes the follow-up story narrower: the system still retries failed work, but the decision is now made where attempt counts and backoff data are already available.".into(),
        ],
        sections: vec![
            Section {
                title: "Why the worker owns retries now".into(),
                text: vec![
                    "The worker already has the attempt count, last failure detail, and backoff timing in memory when it picks up a job.".into(),
                    "Moving the guard there avoids re-deriving retry state in the API path and keeps enqueueing lightweight.".into(),
                ],
                diagram: Some(Diagram::Flow {
                    nodes: vec![
                        "API Handler".into(),
                        "Job Queue".into(),
                        "Background Worker".into(),
                        "Retry Guard".into(),
                        "External Service".into(),
                    ],
                    edges: vec![
                        Edge {
                            from: "API Handler".into(),
                            to: "Job Queue".into(),
                            label: Some("enqueue job".into()),
                        },
                        Edge {
                            from: "Job Queue".into(),
                            to: "Background Worker".into(),
                            label: Some("claim job".into()),
                        },
                        Edge {
                            from: "Background Worker".into(),
                            to: "Retry Guard".into(),
                            label: Some("attempt metadata".into()),
                        },
                        Edge {
                            from: "Retry Guard".into(),
                            to: "External Service".into(),
                            label: Some("retry if allowed".into()),
                        },
                    ],
                }),
                commit: None,
                files: vec![],
            },
            Section {
                title: "Before and after the move".into(),
                text: vec![
                    "Before the change, the API handler needed enough context to guess whether a retry should happen.".into(),
                    "After the change, the handler only enqueues work and the worker makes the retry decision with better local context.".into(),
                ],
                diagram: Some(Diagram::BeforeAfter(BeforeAfterDiagram {
                    before: vec![
                        "API handler checks retry eligibility before enqueueing".into(),
                        "Retry state is reconstructed from partial request context".into(),
                        "Worker executes the job after an earlier decision".into(),
                    ],
                    after: vec![
                        "API handler enqueues the job immediately".into(),
                        "Worker reads attempt count and backoff state from the queue record".into(),
                        "Retry guard decides locally before the external call".into(),
                    ],
                })),
                commit: None,
                files: vec![],
            },
        ],
        verification: None,
        repo: None,
    }
}

fn handoff_example() -> Document {
    Document {
        title: "Handoff: payments webhook retry boundary".into(),
        summary: vec![
            "Another engineer is picking up the payments webhook retry work midway through implementation.".into(),
            "This handoff walks through what already landed, the retry boundary decision, and the verification that still needs to happen.".into(),
        ],
        sections: vec![
            Section {
                title: "What has already landed".into(),
                text: vec![
                    "The webhook ingestion path now enqueues jobs immediately and the background worker owns the retry decision.".into(),
                    "Attempt counts and backoff timestamps are persisted on the queue record rather than reconstructed in the handler.".into(),
                ],
                diagram: Some(Diagram::Timeline {
                    events: vec![
                        TimelineEvent {
                            label: "Step 1".into(),
                            detail: "Move retry state onto the queue record schema.".into(),
                        },
                        TimelineEvent {
                            label: "Step 2".into(),
                            detail: "Move the retry guard into the background worker.".into(),
                        },
                        TimelineEvent {
                            label: "Step 3".into(),
                            detail: "Drop the retry checks that used to live in the API handler.".into(),
                        },
                    ],
                }),
                commit: None,
                files: vec![],
            },
            Section {
                title: "Retry boundary decision".into(),
                text: vec![
                    "The worker owns the retry decision because it already has attempt metadata and backoff state in memory.".into(),
                    "This keeps the API handler lightweight and avoids re-deriving retry state from partial request context.".into(),
                ],
                diagram: Some(Diagram::ComponentGraph {
                    nodes: vec![
                        "Webhook Handler".into(),
                        "Job Queue".into(),
                        "Background Worker".into(),
                        "Retry Guard".into(),
                        "Payments API".into(),
                    ],
                    edges: vec![
                        Edge {
                            from: "Webhook Handler".into(),
                            to: "Job Queue".into(),
                            label: Some("enqueue".into()),
                        },
                        Edge {
                            from: "Job Queue".into(),
                            to: "Background Worker".into(),
                            label: Some("claim job".into()),
                        },
                        Edge {
                            from: "Background Worker".into(),
                            to: "Retry Guard".into(),
                            label: Some("attempt metadata".into()),
                        },
                        Edge {
                            from: "Retry Guard".into(),
                            to: "Payments API".into(),
                            label: Some("retry if allowed".into()),
                        },
                    ],
                }),
                commit: None,
                files: vec![],
            },
            Section {
                title: "What still needs verification".into(),
                text: vec![
                    "The max-attempt boundary has unit coverage, but an end-to-end replay of a failed webhook still needs a manual run.".into(),
                    "The dashboard metric for retry exhaustion is wired up but has not been validated against real traffic yet.".into(),
                ],
                diagram: None,
                commit: None,
                files: vec![],
            },
        ],
        verification: Some(Verification {
            text: vec![
                "Unit tests cover attempt counting and backoff decisions.".into(),
                "A manual replay and the retry-exhaustion metric are the remaining verification steps.".into(),
            ],
        }),
        repo: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_examples_stay_valid() {
        for preset in [
            ExamplePreset::Walkthrough,
            ExamplePreset::Timeline,
            ExamplePreset::BeforeAfter,
            ExamplePreset::Followup,
            ExamplePreset::Handoff,
        ] {
            let document = example_document(preset);
            assert!(document.validate().is_ok());
        }
    }
}
