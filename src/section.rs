//! Section component for AUI documents

use std::fmt::Write;

/// A section in an AUI document
#[derive(Debug, Clone)]
pub struct Section {
    /// Section title (rendered as H2)
    pub title: String,
    /// Section content
    pub content: SectionContent,
}

impl Section {
    /// Create a new section with raw markdown content
    pub fn new(title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            content: SectionContent::Markdown(content.into()),
        }
    }

    /// Create a key-value table section
    pub fn key_value(title: impl Into<String>, rows: Vec<(&str, String)>) -> Self {
        Self {
            title: title.into(),
            content: SectionContent::KeyValueTable(
                rows.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
            ),
        }
    }

    /// Create a status table section with headers
    pub fn table(title: impl Into<String>, headers: Vec<&str>, rows: Vec<Vec<String>>) -> Self {
        Self {
            title: title.into(),
            content: SectionContent::Table {
                headers: headers.into_iter().map(String::from).collect(),
                rows,
            },
        }
    }

    /// Create a list section
    pub fn list(title: impl Into<String>, items: Vec<String>) -> Self {
        Self {
            title: title.into(),
            content: SectionContent::List(items),
        }
    }

    /// Create an empty section with a message
    pub fn empty(title: impl Into<String>, message: &str) -> Self {
        Self {
            title: title.into(),
            content: SectionContent::Markdown(format!("_{message}_")),
        }
    }

    /// Render the section to Markdown
    #[must_use]
    pub fn render(&self) -> String {
        let mut output = String::new();
        writeln!(output, "## {}", self.title).unwrap();
        output.push('\n');
        output.push_str(&self.content.render());
        output.push('\n');
        output
    }
}

/// Content types for sections
#[derive(Debug, Clone)]
pub enum SectionContent {
    /// Raw markdown
    Markdown(String),
    /// Key-value table (2 columns: key, value)
    KeyValueTable(Vec<(String, String)>),
    /// Generic table with headers
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    /// Bullet list
    List(Vec<String>),
    /// Numbered list
    NumberedList(Vec<String>),
    /// Subsections
    Subsections(Vec<Subsection>),
}

impl SectionContent {
    /// Render the content to Markdown
    pub fn render(&self) -> String {
        match self {
            Self::Markdown(md) => md.clone(),

            Self::KeyValueTable(rows) => {
                if rows.is_empty() {
                    return "_无数据_".to_string();
                }
                let mut output = String::from("| 属性 | 值 |\n|------|----|\n");
                for (key, value) in rows {
                    writeln!(output, "| {key} | {value} |").unwrap();
                }
                output
            }

            Self::Table { headers, rows } => {
                if rows.is_empty() {
                    return "_无数据_".to_string();
                }
                let mut output = String::new();

                // Header row
                output.push_str("| ");
                output.push_str(&headers.join(" | "));
                output.push_str(" |\n");

                // Separator
                output.push('|');
                for _ in headers {
                    output.push_str("------|");
                }
                output.push('\n');

                // Data rows
                for row in rows {
                    output.push_str("| ");
                    output.push_str(&row.join(" | "));
                    output.push_str(" |\n");
                }

                output
            }

            Self::List(items) => {
                if items.is_empty() {
                    return "_无_".to_string();
                }
                items
                    .iter()
                    .map(|item| format!("- {item}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            }

            Self::NumberedList(items) => {
                if items.is_empty() {
                    return "_无_".to_string();
                }
                items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| format!("{}. {item}", i + 1))
                    .collect::<Vec<_>>()
                    .join("\n")
            }

            Self::Subsections(subs) => subs
                .iter()
                .map(Subsection::render)
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }
}

/// A subsection (H3)
#[derive(Debug, Clone)]
pub struct Subsection {
    pub title: String,
    pub content: String,
}

impl Subsection {
    pub fn new(title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
        }
    }

    #[must_use]
    pub fn render(&self) -> String {
        format!("### {}\n\n{}\n", self.title, self.content)
    }
}
