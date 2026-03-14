mod examples;
mod model;
mod render;

pub use examples::{ExamplePreset, example_document};
pub use model::{
    BeforeAfterDiagram, Diagram, Document, Edge, Section, TimelineEvent, ValidationError,
    Verification,
};
pub use render::{OutputFormat, render_document, schema_json};
