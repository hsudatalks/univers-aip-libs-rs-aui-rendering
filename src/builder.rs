//! Domain AUI Builder
//!
//! Provides `Window` enum, `AccumulatedWindow` renderer, and `DomainAui` builder
//! for declarative domain dimension window configuration.

use super::{
    standard_domain_windows, CliHistoryWindow, ContentFormat, DefaultCliRenderer,
    DimensionRenderer, InsightsWindow, RenderInput, RenderStrategy, RendererRegistry,
    SuggestionsWindow,
};

/// Pre-defined window types for domain AUI configuration
///
/// Provides a declarative way to configure dimension windows without
/// implementing custom renderers.
#[derive(Debug, Clone, Copy)]
pub enum Window {
    /// CLI command history window (uses `CliHistoryWindow`)
    CliHistory,
    /// Insights window with severity levels (uses `InsightsWindow`)
    Insights,
    /// Suggestions window with priority levels (uses `SuggestionsWindow`)
    Suggestions,

    /// Generic rolling window with configurable entries
    Rolling {
        /// Display name for the window header
        name: &'static str,
        /// Maximum entries to show (default: 5)
        max_entries: usize,
    },

    /// Accumulated window showing full history
    Accumulated {
        /// Display name for the window header
        name: &'static str,
    },

    /// Equipment status window (HVAC domain)
    Equipment(&'static str),

    /// Data table window (Data domain)
    DataTable(&'static str),

    /// Metrics display window (Resource domain)
    Metrics(&'static str),

    /// Conversation history window (Cognitive domain)
    Conversation,

    /// Todo list window (Task domain)
    TodoList,

    /// Pipeline status window (Computation domain)
    Pipeline(&'static str),
}

impl Window {
    // === Quick constructors ===

    /// Create an equipment status window
    #[must_use]
    pub const fn equipment(name: &'static str) -> Self {
        Self::Equipment(name)
    }

    /// Create a data table window
    #[must_use]
    pub const fn data(name: &'static str) -> Self {
        Self::DataTable(name)
    }

    /// Create a metrics display window
    #[must_use]
    pub const fn metrics(name: &'static str) -> Self {
        Self::Metrics(name)
    }

    /// Create a conversation history window
    #[must_use]
    pub const fn conversation() -> Self {
        Self::Conversation
    }

    /// Create a todo list window
    #[must_use]
    pub const fn todo_list() -> Self {
        Self::TodoList
    }

    /// Create a pipeline status window
    #[must_use]
    pub const fn pipeline(name: &'static str) -> Self {
        Self::Pipeline(name)
    }

    /// Create a rolling window with max entries
    #[must_use]
    pub const fn rolling(name: &'static str, max_entries: usize) -> Self {
        Self::Rolling { name, max_entries }
    }

    /// Create an accumulated window showing full history
    #[must_use]
    pub const fn accumulated(name: &'static str) -> Self {
        Self::Accumulated { name }
    }

    /// Convert window configuration to a boxed renderer
    #[must_use]
    pub fn into_renderer(self) -> Box<dyn DimensionRenderer> {
        match self {
            Self::CliHistory => Box::new(CliHistoryWindow::new("CLI History").with_max_entries(10)),
            Self::Insights => Box::new(InsightsWindow::new("Insights").with_max_entries(20)),
            Self::Suggestions => {
                Box::new(SuggestionsWindow::new("Suggestions").with_max_entries(10))
            }
            Self::Rolling { name, max_entries } => {
                Box::new(DefaultCliRenderer::new(name).with_max_entries(max_entries))
            }
            Self::Accumulated { name } => Box::new(AccumulatedWindow::new(name)),
            Self::Equipment(name) => Box::new(DefaultCliRenderer::new(name).with_max_entries(5)),
            Self::DataTable(name) => Box::new(
                DefaultCliRenderer::new(name)
                    .with_max_entries(10)
                    .with_format(ContentFormat::Auto),
            ),
            Self::Metrics(name) | Self::Pipeline(name) => Box::new(
                DefaultCliRenderer::new(name)
                    .with_max_entries(5)
                    .with_format(ContentFormat::Auto),
            ),
            Self::Conversation => Box::new(AccumulatedWindow::new("Conversation")),
            Self::TodoList => Box::new(AccumulatedWindow::new("Todo List")),
        }
    }
}

// ============================================================================
// Accumulated Window
// ============================================================================

/// Accumulated window renderer - shows full chronological history
///
/// Unlike rolling windows that limit entries, accumulated windows
/// show the complete history in chronological order.
#[derive(Debug, Clone, Copy)]
pub struct AccumulatedWindow {
    name: &'static str,
}

impl AccumulatedWindow {
    /// Create a new accumulated window
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self { name }
    }
}

impl DimensionRenderer for AccumulatedWindow {
    fn strategy(&self) -> RenderStrategy {
        RenderStrategy::Aui
    }

    fn render(&self, input: &RenderInput) -> String {
        let mut lines = Vec::new();
        lines.push(format!("## {}", self.name));
        lines.push(String::new());

        if input.history.is_empty() && input.content.is_empty() {
            lines.push(format!("_(No {} yet)_", self.name.to_lowercase()));
        } else {
            // Show history in chronological order (oldest first)
            for entry in input.history.iter().rev() {
                lines.push(entry.content.clone());
                lines.push("---".to_string());
            }

            // Add current content if present
            if !input.content.is_empty() {
                lines.push(input.content.clone());
            } else if !lines.is_empty() && lines.last() == Some(&"---".to_string()) {
                lines.pop(); // Remove trailing separator
            }
        }

        lines.join("\n")
    }

    fn dimension_name(&self) -> &'static str {
        self.name
    }
}

// ============================================================================
// Domain AUI Builder
// ============================================================================

/// Domain AUI Builder - Declarative configuration for domain dimension windows
///
/// Provides a unified, fluent API for configuring AUI windows across all domains.
///
/// # Examples
///
/// ```rust
/// use univers_aip_lib_aui_rendering::{DomainAui, Window, RendererRegistry};
///
/// // Simple: standard windows only
/// let mut registry = RendererRegistry::new();
/// DomainAui::new("agent").standard().register(&mut registry);
///
/// // Custom: standard + domain-specific windows
/// let mut registry = RendererRegistry::new();
/// DomainAui::new("hvac")
///     .standard()
///     .window("chillers", Window::equipment("Chiller"))
///     .window("ahus", Window::equipment("AHU"))
///     .register(&mut registry);
/// ```
#[derive(Debug)]
pub struct DomainAui {
    domain: String,
    windows: Vec<(String, Window)>,
    include_standard: bool,
}

impl DomainAui {
    /// Create a new domain AUI configuration
    #[must_use]
    pub fn new(domain: &str) -> Self {
        Self {
            domain: domain.to_string(),
            windows: Vec::new(),
            include_standard: false,
        }
    }

    /// Add standard windows: {domain}_cli, {domain}_insights, {domain}_suggestions
    ///
    /// These are the most common windows needed by all domains.
    #[must_use]
    pub const fn standard(mut self) -> Self {
        self.include_standard = true;
        self
    }

    /// Add a custom window with the specified key
    #[must_use]
    pub fn window(mut self, key: &str, window: Window) -> Self {
        self.windows.push((key.to_string(), window));
        self
    }

    /// Register all configured windows to the renderer registry
    pub fn register(self, registry: &mut RendererRegistry) {
        // Add standard windows if requested
        if self.include_standard {
            for (key, renderer) in standard_domain_windows(&self.domain) {
                registry.register_boxed(key, renderer);
            }
        }

        // Add custom windows
        for (key, window) in self.windows {
            registry.register_boxed(key, window.into_renderer());
        }
    }

    /// Build and return a new registry with all configured windows
    #[must_use]
    pub fn build(self) -> RendererRegistry {
        let mut registry = RendererRegistry::new();
        self.register(&mut registry);
        registry
    }
}
