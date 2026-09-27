//! AUI rendering contracts and lightweight data types.

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use univers_aip_contracts_shared::insight_types::Insight;

/// Dimension rendering strategy.
#[derive(Debug, Clone, Copy)]
pub enum RenderStrategy {
    /// AUI - fixed rendering with accumulated state.
    Aui,
    /// CLI - rolling history with limits.
    Cli {
        /// Maximum number of history entries to show.
        max_entries: usize,
        /// Optional token limit for total content.
        max_tokens: Option<usize>,
    },
}

impl Default for RenderStrategy {
    fn default() -> Self {
        Self::Cli {
            max_entries: 5,
            max_tokens: None,
        }
    }
}

/// History entry for dimension content.
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    /// Entry content.
    pub content: String,
    /// When this entry was created.
    pub timestamp: DateTime<Utc>,
    /// Optional metadata.
    pub metadata: HashMap<String, String>,
    /// Insights from this entry.
    pub insights: Vec<Insight>,
}

impl HistoryEntry {
    /// Create a new history entry.
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
            insights: Vec::new(),
        }
    }

    /// Create with timestamp.
    pub fn with_timestamp(content: impl Into<String>, timestamp: DateTime<Utc>) -> Self {
        Self {
            content: content.into(),
            timestamp,
            metadata: HashMap::new(),
            insights: Vec::new(),
        }
    }

    /// Create with content and insights.
    pub fn with_insights(
        content: impl Into<String>,
        timestamp: DateTime<Utc>,
        insights: Vec<Insight>,
    ) -> Self {
        Self {
            content: content.into(),
            timestamp,
            metadata: HashMap::new(),
            insights,
        }
    }
}

/// Input for dimension rendering.
#[derive(Debug, Clone)]
pub struct RenderInput {
    /// Current content (latest state).
    pub content: String,
    /// History entries (newest first).
    pub history: Vec<HistoryEntry>,
    /// Dimension metadata.
    pub metadata: HashMap<String, String>,
    /// All accumulated insights (from history + current).
    pub insights: Vec<Insight>,
}

impl RenderInput {
    /// Create empty render input.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            content: String::new(),
            history: Vec::new(),
            metadata: HashMap::new(),
            insights: Vec::new(),
        }
    }

    /// Create with just content.
    pub fn with_content(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            history: Vec::new(),
            metadata: HashMap::new(),
            insights: Vec::new(),
        }
    }
}

/// Trait for rendering dimension content to agents.
pub trait DimensionRenderer: Send + Sync {
    /// Get the rendering strategy.
    fn strategy(&self) -> RenderStrategy;

    /// Render dimension content for agent observation.
    fn render(&self, input: &RenderInput) -> String;

    /// Get the dimension name for labeling.
    fn dimension_name(&self) -> &'static str;
}

/// Content format for rendering.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ContentFormat {
    /// Plain text (legacy).
    #[default]
    Plain,
    /// JSON data - rendered in code blocks.
    Json,
    /// Tabular data - rendered as markdown table.
    Table,
    /// Generic data - auto-detect format.
    Auto,
}
