//! Default CLI Renderer
//!
//! Provides the `DefaultCliRenderer` with rolling history and Markdown output,
//! including JSON-aware rendering as key-value lists.

use super::{ContentFormat, DimensionRenderer, RenderInput, RenderStrategy};
use chrono::{DateTime, Utc};
use std::fmt::Write;

/// Default CLI renderer with rolling history and Markdown output
///
/// Renders dimension content as structured Markdown for better LLM comprehension:
/// - Header with dimension name
/// - Entries in code blocks (for JSON) or plain text
/// - Proper horizontal rules between entries
/// - Timestamp display (optional)
#[derive(Debug, Clone, Copy)]
pub struct DefaultCliRenderer {
    name: &'static str,
    max_entries: usize,
    max_tokens: Option<usize>,
    /// Whether to show dimension header
    show_header: bool,
    /// Whether to show timestamps
    show_timestamps: bool,
    /// Content format hint
    content_format: ContentFormat,
}

impl DefaultCliRenderer {
    /// Create a new CLI renderer
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            max_entries: 5,
            max_tokens: None,
            show_header: true,
            show_timestamps: false,
            content_format: ContentFormat::Auto,
        }
    }

    /// Set maximum entries
    #[must_use]
    pub const fn with_max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }

    /// Set maximum tokens
    #[must_use]
    pub const fn with_max_tokens(mut self, max: usize) -> Self {
        self.max_tokens = Some(max);
        self
    }

    /// Enable/disable header rendering
    #[must_use]
    pub const fn with_header(mut self, show: bool) -> Self {
        self.show_header = show;
        self
    }

    /// Enable/disable timestamp display
    #[must_use]
    pub const fn with_timestamps(mut self, show: bool) -> Self {
        self.show_timestamps = show;
        self
    }

    /// Set content format hint
    #[must_use]
    pub const fn with_format(mut self, format: ContentFormat) -> Self {
        self.content_format = format;
        self
    }

    /// Detect if content looks like JSON
    fn looks_like_json(content: &str) -> bool {
        let trimmed = content.trim();
        (trimmed.starts_with('{') && trimmed.ends_with('}'))
            || (trimmed.starts_with('[') && trimmed.ends_with(']'))
    }

    /// Convert `snake_case` to Title Case
    pub(crate) fn format_key(key: &str) -> String {
        key.split('_')
            .map(|word| {
                let mut chars = word.chars();
                chars
                    .next()
                    .map_or_else(String::new, |c| c.to_uppercase().chain(chars).collect())
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Format a JSON value for display
    fn format_json_value(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::Null => "_null_".to_string(),
            serde_json::Value::Bool(b) => {
                if *b {
                    "✅ Yes".to_string()
                } else {
                    "❌ No".to_string()
                }
            }
            serde_json::Value::Number(n) => {
                // Format numbers nicely
                n.as_f64().map_or_else(
                    || n.to_string(),
                    |f| {
                        if f.fract() == 0.0 && f.abs() < 1_000_000.0 {
                            format!("{f:.0}")
                        } else {
                            format!("{f:.2}")
                        }
                    },
                )
            }
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Array(arr) => {
                if arr.is_empty() {
                    "_empty_".to_string()
                } else if arr.len() <= 3 {
                    arr.iter()
                        .map(Self::format_json_value)
                        .collect::<Vec<_>>()
                        .join(", ")
                } else {
                    format!(
                        "{}, ... (+{} more)",
                        arr.iter()
                            .take(2)
                            .map(Self::format_json_value)
                            .collect::<Vec<_>>()
                            .join(", "),
                        arr.len() - 2
                    )
                }
            }
            serde_json::Value::Object(obj) => {
                // Nested object - show as compact representation
                if obj.is_empty() {
                    "_empty_".to_string()
                } else {
                    format!("{{...}} ({} fields)", obj.len())
                }
            }
        }
    }

    /// Render JSON object as key-value list
    fn render_json_as_list(content: &str) -> Option<String> {
        let value: serde_json::Value = serde_json::from_str(content).ok()?;

        match value {
            serde_json::Value::Object(obj) => {
                let mut lines = Vec::new();
                for (key, val) in &obj {
                    let formatted_key = Self::format_key(key);
                    let formatted_val = Self::format_json_value(val);
                    lines.push(format!("- **{formatted_key}**: {formatted_val}"));
                }
                Some(lines.join("\n"))
            }
            serde_json::Value::Array(arr) => {
                // For arrays, number each item
                let mut lines = Vec::new();
                for (i, item) in arr.iter().enumerate() {
                    if let serde_json::Value::Object(obj) = item {
                        // If array of objects, show first few fields
                        let summary: Vec<String> = obj
                            .iter()
                            .take(3)
                            .map(|(k, v)| format!("{}: {}", k, Self::format_json_value(v)))
                            .collect();
                        lines.push(format!("{}. {}", i + 1, summary.join(", ")));
                    } else {
                        lines.push(format!("{}. {}", i + 1, Self::format_json_value(item)));
                    }
                }
                Some(lines.join("\n"))
            }
            _ => None,
        }
    }

    /// Format a single entry as Markdown
    fn format_entry(&self, content: &str, timestamp: Option<&DateTime<Utc>>) -> String {
        let mut output = String::new();

        // Add timestamp if enabled
        if self.show_timestamps {
            if let Some(ts) = timestamp {
                let _ = writeln!(output, "_{}_ ", ts.format("%Y-%m-%d %H:%M:%S UTC"));
            }
        }

        // Determine format and render
        let is_json = match self.content_format {
            ContentFormat::Json => true,
            ContentFormat::Auto => Self::looks_like_json(content),
            ContentFormat::Plain | ContentFormat::Table => false,
        };

        if is_json {
            // Try to render as key-value list
            if let Some(list) = Self::render_json_as_list(content) {
                output.push_str(&list);
            } else {
                // Fallback to raw content
                output.push_str(content);
            }
        } else {
            output.push_str(content);
        }

        output
    }
}

impl DimensionRenderer for DefaultCliRenderer {
    fn strategy(&self) -> RenderStrategy {
        RenderStrategy::Cli {
            max_entries: self.max_entries,
            max_tokens: self.max_tokens,
        }
    }

    fn render(&self, input: &RenderInput) -> String {
        let mut output = String::new();

        // Header
        if self.show_header {
            let _ = write!(output, "## {}\n\n", self.name);
        }

        // Collect entries (history + current)
        let mut entries: Vec<(String, Option<DateTime<Utc>>)> = input
            .history
            .iter()
            .take(self.max_entries)
            .filter(|e| !e.content.is_empty())
            .map(|e| (e.content.clone(), Some(e.timestamp)))
            .collect();

        // Add current content at the end (most recent)
        if !input.content.is_empty() {
            entries.push((input.content.clone(), None));
        }

        if entries.is_empty() {
            output.push_str("_(No data)_\n");
        } else {
            // Render entries with separators
            for (i, (content, timestamp)) in entries.iter().enumerate() {
                if i > 0 {
                    output.push_str("\n---\n\n");
                }
                output.push_str(&self.format_entry(content, timestamp.as_ref()));
                output.push('\n');
            }
        }

        // Apply token limit if specified
        if let Some(max_tokens) = self.max_tokens {
            // Rough estimate: 1 token ≈ 4 chars
            let max_chars = max_tokens * 4;
            if output.len() > max_chars {
                // Find a safe truncation point (avoid breaking markdown)
                let truncate_at = output[..max_chars].rfind("\n---\n").unwrap_or(max_chars);
                output.truncate(truncate_at);
                output.push_str("\n\n_[...truncated]_\n");
            }
        }

        output
    }

    fn dimension_name(&self) -> &'static str {
        self.name
    }
}
