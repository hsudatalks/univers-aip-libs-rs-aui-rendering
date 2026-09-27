//! Tests for AUI module

use super::*;
use std::collections::HashMap;

#[test]
fn test_default_cli_renderer_markdown() {
    let renderer = DefaultCliRenderer::new("Test Dimension").with_max_entries(3);

    let input = RenderInput {
        content: "current".to_string(),
        history: vec![
            HistoryEntry::new("entry1"),
            HistoryEntry::new("entry2"),
            HistoryEntry::new("entry3"),
            HistoryEntry::new("entry4"), // Should be excluded
        ],
        metadata: HashMap::new(),
        insights: Vec::new(),
    };

    let result = renderer.render(&input);

    // Should have markdown header
    assert!(result.starts_with("## Test Dimension\n"));
    // Should contain entries
    assert!(result.contains("entry1"));
    assert!(result.contains("entry2"));
    assert!(result.contains("entry3"));
    assert!(!result.contains("entry4")); // Excluded by max_entries
    assert!(result.contains("current"));
    // Should have separators
    assert!(result.contains("---"));
}

#[test]
fn test_json_renders_as_key_value_list() {
    let renderer = DefaultCliRenderer::new("JSON Data")
        .with_header(false)
        .with_format(ContentFormat::Auto);

    let input = RenderInput::with_content(
        r#"{"consumption_kwh": 123, "power_factor": 0.95, "is_active": true}"#,
    );
    let result = renderer.render(&input);

    // Should render as key-value list, NOT code block
    assert!(!result.contains("```"));
    // Keys should be Title Case
    assert!(result.contains("**Consumption Kwh**"));
    assert!(result.contains("**Power Factor**"));
    assert!(result.contains("**Is Active**"));
    // Values should be formatted
    assert!(result.contains("123"));
    assert!(result.contains("0.95"));
    assert!(result.contains("\u{2705} Yes"));
}

#[test]
fn test_json_array_renders_as_list() {
    let renderer = DefaultCliRenderer::new("Array Data")
        .with_header(false)
        .with_format(ContentFormat::Auto);

    let input = RenderInput::with_content(r#"[{"id": 1, "name": "a"}, {"id": 2, "name": "b"}]"#);
    let result = renderer.render(&input);

    // Should render as numbered list
    assert!(result.contains("1."));
    assert!(result.contains("2."));
}

#[test]
fn test_plain_text_unchanged() {
    let renderer = DefaultCliRenderer::new("Plain Text")
        .with_header(false)
        .with_format(ContentFormat::Plain);

    let input = RenderInput::with_content("Just plain text");
    let result = renderer.render(&input);

    // Should NOT have code block
    assert!(!result.contains("```"));
    assert!(result.contains("Just plain text"));
}

#[test]
fn test_empty_input() {
    let renderer = DefaultCliRenderer::new("Empty").with_header(true);
    let input = RenderInput::empty();
    let result = renderer.render(&input);

    assert!(result.contains("## Empty"));
    assert!(result.contains("_(No data)_"));
}

#[test]
fn test_format_key_snake_to_title() {
    assert_eq!(
        DefaultCliRenderer::format_key("power_factor"),
        "Power Factor"
    );
    assert_eq!(
        DefaultCliRenderer::format_key("daily_consumption_kwh"),
        "Daily Consumption Kwh"
    );
    assert_eq!(DefaultCliRenderer::format_key("id"), "Id");
}

#[test]
fn test_renderer_registry() {
    let mut registry = RendererRegistry::new();
    registry.register("test", DefaultCliRenderer::new("test"));

    assert!(registry.has("test"));
    assert!(!registry.has("unknown"));
}

// ============================================================================
// Standard AUI Window Component Tests
// ============================================================================

#[test]
fn test_severity_level() {
    assert_eq!(SeverityLevel::parse("critical"), SeverityLevel::Critical);
    assert_eq!(SeverityLevel::parse("CRITICAL"), SeverityLevel::Critical);
    assert_eq!(SeverityLevel::parse("error"), SeverityLevel::Error);
    assert_eq!(SeverityLevel::parse("warning"), SeverityLevel::Warning);
    assert_eq!(SeverityLevel::parse("info"), SeverityLevel::Info);
    assert_eq!(SeverityLevel::parse("unknown"), SeverityLevel::Note);

    assert_eq!(SeverityLevel::Critical.indicator(), "[CRITICAL]");
    assert_eq!(SeverityLevel::Error.indicator(), "[error]");
    assert_eq!(SeverityLevel::Warning.indicator(), "[warning]");
    assert_eq!(SeverityLevel::Info.indicator(), "[info]");
    assert_eq!(SeverityLevel::Note.indicator(), "[note]");
}

#[test]
fn test_priority_level() {
    assert_eq!(PriorityLevel::parse("urgent"), PriorityLevel::Urgent);
    assert_eq!(PriorityLevel::parse("critical"), PriorityLevel::Urgent);
    assert_eq!(PriorityLevel::parse("high"), PriorityLevel::High);
    assert_eq!(PriorityLevel::parse("medium"), PriorityLevel::Medium);
    assert_eq!(PriorityLevel::parse("low"), PriorityLevel::Low);
    assert_eq!(PriorityLevel::parse("unknown"), PriorityLevel::None);

    assert_eq!(PriorityLevel::Urgent.indicator(), "[URGENT]");
    assert_eq!(PriorityLevel::High.indicator(), "[high]");
    assert_eq!(PriorityLevel::Medium.indicator(), "[medium]");
    assert_eq!(PriorityLevel::Low.indicator(), "[low]");
    assert_eq!(PriorityLevel::None.indicator(), "[-]");
}

#[test]
fn test_cli_history_window_empty() {
    let renderer = CliHistoryWindow::new("CLI History");
    let input = RenderInput::empty();
    let result = renderer.render(&input);

    assert!(result.contains("## CLI History"));
    assert!(result.contains("_(No commands executed yet)_"));
}

#[test]
fn test_cli_history_window_with_entries() {
    let renderer = CliHistoryWindow::new("CLI History").with_max_entries(10);

    let mut metadata = HashMap::new();
    metadata.insert("command".to_string(), "chillers list".to_string());

    let input = RenderInput {
        content: String::new(),
        history: vec![HistoryEntry {
            content: "Found 3 chillers".to_string(),
            timestamp: chrono::Utc::now(),
            metadata,
            insights: Vec::new(),
        }],
        metadata: HashMap::new(),
        insights: Vec::new(),
    };

    let result = renderer.render(&input);

    assert!(result.contains("## CLI History"));
    assert!(result.contains("> chillers list"));
    assert!(result.contains("Found 3 chillers"));
}

#[test]
fn test_cli_history_window_custom_prefix() {
    let renderer = CliHistoryWindow::new("Shell").with_command_prefix("$ ");

    let mut metadata = HashMap::new();
    metadata.insert("command".to_string(), "ls -la".to_string());

    let input = RenderInput {
        content: String::new(),
        history: vec![HistoryEntry {
            content: "file1.txt\nfile2.txt".to_string(),
            timestamp: chrono::Utc::now(),
            metadata,
            insights: Vec::new(),
        }],
        metadata: HashMap::new(),
        insights: Vec::new(),
    };

    let result = renderer.render(&input);
    assert!(result.contains("$ ls -la"));
}

#[test]
fn test_insights_window_empty() {
    let renderer = InsightsWindow::new("Insights");
    let input = RenderInput::empty();
    let result = renderer.render(&input);

    assert!(result.contains("## Insights"));
    assert!(result.contains("_(No insights detected)_"));
}

#[test]
fn test_insights_window_with_warning() {
    let renderer = InsightsWindow::new("Insights");

    let mut metadata = HashMap::new();
    metadata.insert("severity".to_string(), "warning".to_string());
    metadata.insert("type".to_string(), "efficiency_degraded".to_string());
    metadata.insert("title".to_string(), "Chiller efficiency low".to_string());

    let input = RenderInput {
        content: String::new(),
        history: vec![HistoryEntry {
            content: "COP dropped from 5.2 to 4.2".to_string(),
            timestamp: chrono::Utc::now(),
            metadata,
            insights: Vec::new(),
        }],
        metadata: HashMap::new(),
        insights: Vec::new(),
    };

    let result = renderer.render(&input);

    assert!(result.contains("## Insights"));
    assert!(result.contains("[warning]"));
    assert!(result.contains("efficiency_degraded"));
    assert!(result.contains("Chiller efficiency low"));
    assert!(result.contains("COP dropped from 5.2 to 4.2"));
}

#[test]
fn test_suggestions_window_empty() {
    let renderer = SuggestionsWindow::new("Suggestions");
    let input = RenderInput::empty();
    let result = renderer.render(&input);

    assert!(result.contains("## Suggestions"));
    assert!(result.contains("_(No suggestions)_"));
}

#[test]
fn test_suggestions_window_with_command() {
    let renderer = SuggestionsWindow::new("Suggestions");

    let mut metadata = HashMap::new();
    metadata.insert("priority".to_string(), "high".to_string());
    metadata.insert("action".to_string(), "Run efficiency diagnosis".to_string());
    metadata.insert(
        "command".to_string(),
        "chillers efficiency CH-001".to_string(),
    );

    let input = RenderInput {
        content: String::new(),
        history: vec![HistoryEntry {
            content: String::new(),
            timestamp: chrono::Utc::now(),
            metadata,
            insights: Vec::new(),
        }],
        metadata: HashMap::new(),
        insights: Vec::new(),
    };

    let result = renderer.render(&input);

    assert!(result.contains("## Suggestions"));
    assert!(result.contains("[high]"));
    assert!(result.contains("Run efficiency diagnosis"));
    assert!(result.contains("-> chillers efficiency CH-001"));
}

#[test]
fn test_suggestions_window_custom_indicator() {
    let renderer = SuggestionsWindow::new("Actions").with_command_indicator("  \u{2192} ");

    let mut metadata = HashMap::new();
    metadata.insert("priority".to_string(), "medium".to_string());
    metadata.insert("action".to_string(), "Check status".to_string());
    metadata.insert("command".to_string(), "status".to_string());

    let input = RenderInput {
        content: String::new(),
        history: vec![HistoryEntry {
            content: String::new(),
            timestamp: chrono::Utc::now(),
            metadata,
            insights: Vec::new(),
        }],
        metadata: HashMap::new(),
        insights: Vec::new(),
    };

    let result = renderer.render(&input);
    assert!(result.contains("\u{2192} status"));
}

#[test]
fn test_standard_domain_windows() {
    let windows = standard_domain_windows("hvac");

    assert_eq!(windows.len(), 3);
    assert_eq!(windows[0].0, "hvac_cli");
    assert_eq!(windows[1].0, "hvac_insights");
    assert_eq!(windows[2].0, "hvac_suggestions");

    // Verify renderers are functional
    let input = RenderInput::empty();
    for (_, renderer) in &windows {
        let _ = renderer.render(&input);
    }
}

#[test]
fn test_standard_domain_windows_registration() {
    let mut registry = RendererRegistry::new();
    for (key, renderer) in standard_domain_windows("resource") {
        registry.register_boxed(key, renderer);
    }

    assert!(registry.has("resource_cli"));
    assert!(registry.has("resource_insights"));
    assert!(registry.has("resource_suggestions"));
}

// =========================================================================
// DomainAui Builder Tests
// =========================================================================

#[test]
fn test_domain_aui_standard_only() {
    let registry = DomainAui::new("agent").standard().build();

    assert!(registry.has("agent_cli"));
    assert!(registry.has("agent_insights"));
    assert!(registry.has("agent_suggestions"));
}

#[test]
fn test_domain_aui_with_custom_windows() {
    let registry = DomainAui::new("hvac")
        .standard()
        .window("chillers", Window::equipment("Chiller"))
        .window("ahus", Window::equipment("AHU"))
        .build();

    // Standard windows
    assert!(registry.has("hvac_cli"));
    assert!(registry.has("hvac_insights"));
    assert!(registry.has("hvac_suggestions"));

    // Custom windows
    assert!(registry.has("chillers"));
    assert!(registry.has("ahus"));

    // Verify renderers work
    let input = RenderInput::empty();
    let chiller = registry.get("chillers");
    assert_eq!(chiller.dimension_name(), "Chiller");
    let _ = chiller.render(&input);
}

#[test]
fn test_domain_aui_without_standard() {
    let registry = DomainAui::new("cognitive")
        .window("conversation", Window::conversation())
        .window("todo", Window::todo_list())
        .build();

    // Should NOT have standard windows
    assert!(!registry.has("cognitive_cli"));

    // Should have custom windows
    assert!(registry.has("conversation"));
    assert!(registry.has("todo"));
}

#[test]
fn test_window_types() {
    // Test all window constructors
    let _rolling = Window::rolling("Test", 10);
    let _accumulated = Window::accumulated("History");
    let _equipment = Window::equipment("Chiller");
    let _data = Window::data("Signals");
    let _metrics = Window::metrics("Energy");
    let _conversation = Window::conversation();
    let _todo = Window::todo_list();
    let _pipeline = Window::pipeline("Expression");

    // Test into_renderer for each type
    let input = RenderInput::empty();
    assert!(!Window::equipment("Test")
        .into_renderer()
        .render(&input)
        .is_empty());
    assert!(!Window::accumulated("Test")
        .into_renderer()
        .render(&input)
        .is_empty());
}

#[test]
fn test_accumulated_window_chronological_order() {
    let renderer = AccumulatedWindow::new("History");
    // History is stored newest-first per RenderInput convention
    let input = RenderInput {
        content: "current".to_string(),
        history: vec![
            HistoryEntry::new("newest"),
            HistoryEntry::new("middle"),
            HistoryEntry::new("oldest"),
        ],
        metadata: HashMap::new(),
        insights: Vec::new(),
    };

    let result = renderer.render(&input);

    // Verify chronological order (oldest first)
    let oldest_pos = result.find("oldest").unwrap();
    let middle_pos = result.find("middle").unwrap();
    let newest_pos = result.find("newest").unwrap();
    let current_pos = result.find("current").unwrap();

    assert!(oldest_pos < middle_pos);
    assert!(middle_pos < newest_pos);
    assert!(newest_pos < current_pos);
}
