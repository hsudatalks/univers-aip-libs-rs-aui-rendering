//! Standard AUI Window Components
//!
//! Provides `SeverityLevel`, `PriorityLevel`, `CliHistoryWindow`, `InsightsWindow`,
//! `SuggestionsWindow`, and the `standard_domain_windows` factory function.

use super::{DimensionRenderer, HistoryEntry, RenderInput, RenderStrategy};

// Re-export SeverityLevel from the unified definition in components::format
use super::SeverityLevel;

/// Priority level for suggestions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriorityLevel {
    Urgent,
    High,
    Medium,
    Low,
    None,
}

impl PriorityLevel {
    /// Parse priority from string
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "critical" | "urgent" => Self::Urgent,
            "high" => Self::High,
            "medium" => Self::Medium,
            "low" => Self::Low,
            _ => Self::None,
        }
    }

    /// Get display indicator
    #[must_use]
    pub const fn indicator(&self) -> &'static str {
        match self {
            Self::Urgent => "[URGENT]",
            Self::High => "[high]",
            Self::Medium => "[medium]",
            Self::Low => "[low]",
            Self::None => "[-]",
        }
    }
}

// ============================================================================
// CLI History Window
// ============================================================================

/// Standard CLI History Window
///
/// Displays CLI command history with requests and responses in a scrollable window.
/// Uses AUI strategy to accumulate all interactions.
///
/// # Example
///
/// ```rust
/// use univers_aip_lib_aui_rendering::CliHistoryWindow;
///
/// let renderer = CliHistoryWindow::new("CLI History")
///     .with_max_entries(10)
///     .with_command_prefix("> ");
/// ```
#[derive(Debug, Clone, Copy)]
pub struct CliHistoryWindow {
    title: &'static str,
    max_entries: usize,
    command_prefix: &'static str,
    separator: &'static str,
    empty_message: &'static str,
}

impl CliHistoryWindow {
    /// Create a new CLI history window
    #[must_use]
    pub const fn new(title: &'static str) -> Self {
        Self {
            title,
            max_entries: 10,
            command_prefix: "> ",
            separator: "---",
            empty_message: "_(No commands executed yet)_",
        }
    }

    /// Set maximum entries
    #[must_use]
    pub const fn with_max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }

    /// Set command prefix (e.g., "> " or "$ ")
    #[must_use]
    pub const fn with_command_prefix(mut self, prefix: &'static str) -> Self {
        self.command_prefix = prefix;
        self
    }

    /// Set entry separator
    #[must_use]
    pub const fn with_separator(mut self, sep: &'static str) -> Self {
        self.separator = sep;
        self
    }

    /// Set empty state message
    #[must_use]
    pub const fn with_empty_message(mut self, msg: &'static str) -> Self {
        self.empty_message = msg;
        self
    }

    /// Format a single CLI entry
    fn format_entry(&self, entry: &HistoryEntry) -> String {
        let mut output = String::new();

        // Check for command in metadata
        if let Some(command) = entry.metadata.get("command") {
            output.push_str(self.command_prefix);
            output.push_str(command);
            output.push('\n');
        }

        // Add response content
        if !entry.content.is_empty() {
            output.push_str(&entry.content);
        }

        output
    }
}

impl DimensionRenderer for CliHistoryWindow {
    fn strategy(&self) -> RenderStrategy {
        RenderStrategy::Aui
    }

    fn render(&self, input: &RenderInput) -> String {
        let mut output = format!("## {}\n\n", self.title);

        // Collect entries (newest first in history, reverse for chronological)
        let entries: Vec<_> = input.history.iter().rev().take(self.max_entries).collect();

        if entries.is_empty() && input.content.is_empty() {
            output.push_str(self.empty_message);
            output.push('\n');
        } else {
            // Render history entries
            for (i, entry) in entries.iter().enumerate() {
                if i > 0 {
                    output.push('\n');
                    output.push_str(self.separator);
                    output.push_str("\n\n");
                }
                output.push_str(&self.format_entry(entry));
                output.push('\n');
            }

            // Add current content if any
            if !input.content.is_empty() {
                if !entries.is_empty() {
                    output.push('\n');
                    output.push_str(self.separator);
                    output.push_str("\n\n");
                }
                output.push_str(&input.content);
                output.push('\n');
            }
        }

        output
    }

    fn dimension_name(&self) -> &'static str {
        self.title
    }
}

// ============================================================================
// Insights Window
// ============================================================================

/// Standard Insights Window
///
/// Accumulates and displays insights triggered by operations.
/// Uses AUI strategy to collect insights over time.
///
/// # Example
///
/// ```rust
/// use univers_aip_lib_aui_rendering::InsightsWindow;
///
/// let renderer = InsightsWindow::new("Insights")
///     .with_max_entries(20);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct InsightsWindow {
    title: &'static str,
    max_entries: usize,
    separator: &'static str,
    empty_message: &'static str,
}

impl InsightsWindow {
    /// Create a new insights window
    #[must_use]
    pub const fn new(title: &'static str) -> Self {
        Self {
            title,
            max_entries: 20,
            separator: "---",
            empty_message: "_(No insights detected)_",
        }
    }

    /// Set maximum entries
    #[must_use]
    pub const fn with_max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }

    /// Set entry separator
    #[must_use]
    pub const fn with_separator(mut self, sep: &'static str) -> Self {
        self.separator = sep;
        self
    }

    /// Set empty state message
    #[must_use]
    pub const fn with_empty_message(mut self, msg: &'static str) -> Self {
        self.empty_message = msg;
        self
    }

    /// Format a single insight entry
    fn format_entry(entry: &HistoryEntry) -> String {
        let mut output = String::new();

        // Extract insight metadata
        let severity = entry
            .metadata
            .get("severity")
            .map_or(SeverityLevel::Info, |s| SeverityLevel::parse(s));
        let insight_type = entry
            .metadata
            .get("type")
            .map_or("observation", String::as_str);
        let title = entry.metadata.get("title").map(String::as_str);

        // Format: [severity] type
        output.push_str(severity.indicator());
        output.push(' ');
        output.push_str(insight_type);
        output.push('\n');

        // Add title if present
        if let Some(t) = title {
            output.push_str(t);
            output.push('\n');
        }

        // Add description (content)
        if !entry.content.is_empty() {
            output.push_str(&entry.content);
        }

        output
    }
}

impl DimensionRenderer for InsightsWindow {
    fn strategy(&self) -> RenderStrategy {
        RenderStrategy::Aui
    }

    fn render(&self, input: &RenderInput) -> String {
        let mut output = format!("## {}\n\n", self.title);

        let entries: Vec<_> = input.history.iter().rev().take(self.max_entries).collect();

        if entries.is_empty() && input.content.is_empty() {
            output.push_str(self.empty_message);
            output.push('\n');
        } else {
            for (i, entry) in entries.iter().enumerate() {
                if i > 0 {
                    output.push('\n');
                    output.push_str(self.separator);
                    output.push_str("\n\n");
                }
                output.push_str(&Self::format_entry(entry));
                output.push('\n');
            }

            // Add current content if any
            if !input.content.is_empty() {
                if !entries.is_empty() {
                    output.push('\n');
                    output.push_str(self.separator);
                    output.push_str("\n\n");
                }
                output.push_str(&input.content);
                output.push('\n');
            }
        }

        output
    }

    fn dimension_name(&self) -> &'static str {
        self.title
    }
}

// ============================================================================
// Suggestions Window
// ============================================================================

/// Standard Suggestions Window
///
/// Displays actionable suggestions with priority indicators.
/// Uses AUI strategy to accumulate suggestions.
///
/// # Example
///
/// ```rust
/// use univers_aip_lib_aui_rendering::SuggestionsWindow;
///
/// let renderer = SuggestionsWindow::new("Suggestions")
///     .with_max_entries(10)
///     .with_command_indicator("  -> ");
/// ```
#[derive(Debug, Clone, Copy)]
pub struct SuggestionsWindow {
    title: &'static str,
    max_entries: usize,
    command_indicator: &'static str,
    separator: &'static str,
    empty_message: &'static str,
}

impl SuggestionsWindow {
    /// Create a new suggestions window
    #[must_use]
    pub const fn new(title: &'static str) -> Self {
        Self {
            title,
            max_entries: 10,
            command_indicator: "  -> ",
            separator: "---",
            empty_message: "_(No suggestions)_",
        }
    }

    /// Set maximum entries
    #[must_use]
    pub const fn with_max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }

    /// Set command indicator prefix
    #[must_use]
    pub const fn with_command_indicator(mut self, indicator: &'static str) -> Self {
        self.command_indicator = indicator;
        self
    }

    /// Set entry separator
    #[must_use]
    pub const fn with_separator(mut self, sep: &'static str) -> Self {
        self.separator = sep;
        self
    }

    /// Set empty state message
    #[must_use]
    pub const fn with_empty_message(mut self, msg: &'static str) -> Self {
        self.empty_message = msg;
        self
    }

    /// Format a single suggestion entry
    fn format_entry(&self, entry: &HistoryEntry) -> String {
        let mut output = String::new();

        // Extract suggestion metadata
        let priority = entry
            .metadata
            .get("priority")
            .map_or(PriorityLevel::Medium, |s| PriorityLevel::parse(s));
        let action = entry.metadata.get("action").map(String::as_str);
        let command = entry.metadata.get("command").map(String::as_str);

        // Format: [priority] action/description
        output.push_str(priority.indicator());
        output.push(' ');

        if let Some(a) = action {
            output.push_str(a);
        } else if !entry.content.is_empty() {
            // Use first line of content as action description
            let first_line = entry.content.lines().next().unwrap_or(&entry.content);
            output.push_str(first_line);
        }
        output.push('\n');

        // Add command if present
        if let Some(cmd) = command {
            output.push_str(self.command_indicator);
            output.push_str(cmd);
            output.push('\n');
        }

        output
    }
}

impl DimensionRenderer for SuggestionsWindow {
    fn strategy(&self) -> RenderStrategy {
        RenderStrategy::Aui
    }

    fn render(&self, input: &RenderInput) -> String {
        let mut output = format!("## {}\n\n", self.title);

        let entries: Vec<_> = input.history.iter().rev().take(self.max_entries).collect();

        if entries.is_empty() && input.content.is_empty() {
            output.push_str(self.empty_message);
            output.push('\n');
        } else {
            for (i, entry) in entries.iter().enumerate() {
                if i > 0 {
                    output.push('\n');
                    output.push_str(self.separator);
                    output.push_str("\n\n");
                }
                output.push_str(&self.format_entry(entry));
            }

            // Add current content if any
            if !input.content.is_empty() {
                if !entries.is_empty() {
                    output.push('\n');
                    output.push_str(self.separator);
                    output.push_str("\n\n");
                }
                output.push_str(&input.content);
                output.push('\n');
            }
        }

        output
    }

    fn dimension_name(&self) -> &'static str {
        self.title
    }
}

// ============================================================================
// Standard Domain Windows Factory
// ============================================================================

/// Create standard domain windows for a given domain
///
/// Returns a vector of (`dimension_key`, renderer) pairs ready for registration.
///
/// # Arguments
///
/// * `domain` - The domain prefix (e.g., "hvac", "resource", "data")
///
/// # Example
///
/// ```rust
/// use univers_aip_lib_aui_rendering::{standard_domain_windows, RendererRegistry};
///
/// let mut registry = RendererRegistry::new();
/// for (key, renderer) in standard_domain_windows("hvac") {
///     registry.register_boxed(key, renderer);
/// }
/// ```
#[must_use]
pub fn standard_domain_windows(domain: &str) -> Vec<(String, Box<dyn DimensionRenderer>)> {
    vec![
        (
            format!("{domain}_cli"),
            Box::new(CliHistoryWindow::new("CLI History").with_max_entries(10)),
        ),
        (
            format!("{domain}_insights"),
            Box::new(InsightsWindow::new("Insights").with_max_entries(20)),
        ),
        (
            format!("{domain}_suggestions"),
            Box::new(SuggestionsWindow::new("Suggestions").with_max_entries(10)),
        ),
    ]
}
