//! AUI Document builder
//!
//! Provides the top-level `AuiDocument` container and `AuiRenderable` trait
//! for building Markdown-based viewport documents for AI agents.

use std::fmt::Write;

use crate::Section;

/// AUI Document - the top-level container for an AUI view
#[derive(Debug, Clone)]
pub struct AuiDocument {
    /// Document title (rendered as H1)
    pub title: String,
    /// Optional subtitle or description
    pub subtitle: Option<String>,
    /// Document sections
    pub sections: Vec<Section>,
    /// Footer notes
    pub footer: Option<String>,
}

impl AuiDocument {
    /// Create a new AUI document with the given title
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            sections: Vec::new(),
            footer: None,
        }
    }

    /// Add a subtitle
    #[must_use]
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Add a section
    #[must_use]
    pub fn section(mut self, section: Section) -> Self {
        self.sections.push(section);
        self
    }

    /// Add multiple sections
    #[must_use]
    pub fn sections(mut self, sections: impl IntoIterator<Item = Section>) -> Self {
        self.sections.extend(sections);
        self
    }

    /// Add a footer note
    #[must_use]
    pub fn footer(mut self, footer: impl Into<String>) -> Self {
        self.footer = Some(footer.into());
        self
    }

    /// Render the document to Markdown
    #[must_use]
    pub fn render(&self) -> String {
        let mut output = String::new();

        // Title
        writeln!(output, "# {}", self.title).unwrap();
        output.push('\n');

        // Subtitle
        if let Some(subtitle) = &self.subtitle {
            writeln!(output, "_{subtitle}_").unwrap();
            output.push('\n');
        }

        // Sections
        for section in &self.sections {
            output.push_str(&section.render());
            output.push('\n');
        }

        // Footer
        if let Some(footer) = &self.footer {
            output.push_str("---\n\n");
            writeln!(output, "_{footer}_").unwrap();
        }

        output
    }
}

/// Trait for types that can be rendered as AUI
pub trait AuiRenderable {
    /// Render this item as an AUI document
    fn to_aui(&self) -> AuiDocument;

    /// Render directly to Markdown string
    fn render_aui(&self) -> String {
        self.to_aui().render()
    }
}
