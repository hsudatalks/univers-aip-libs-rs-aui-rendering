//! In-process AUI text rendering over caller-supplied values and public Insight types.
//! Owners select and populate windows; rendering performs no IO or authorization.
mod builder;
mod default_renderer;
mod document;
mod registry;
mod section;
mod severity;
mod types;
mod windows;
pub use builder::{AccumulatedWindow, DomainAui, Window};
pub use default_renderer::DefaultCliRenderer;
pub use document::{AuiDocument, AuiRenderable};
pub use registry::RendererRegistry;
pub use section::{Section, SectionContent, Subsection};
pub use severity::SeverityLevel;
pub use types::{ContentFormat, DimensionRenderer, HistoryEntry, RenderInput, RenderStrategy};
pub use windows::{
    standard_domain_windows, CliHistoryWindow, InsightsWindow, PriorityLevel, SuggestionsWindow,
};
#[cfg(test)]
mod tests;

#[cfg(feature = "components")]
pub mod components;
