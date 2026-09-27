use univers_aip_lib_aui_rendering::{
    DimensionRenderer, DomainAui, HistoryEntry, RenderInput, RenderStrategy, Window,
};

struct OwnerRenderer;
impl DimensionRenderer for OwnerRenderer {
    fn strategy(&self) -> RenderStrategy {
        RenderStrategy::Aui
    }
    fn render(&self, input: &RenderInput) -> String {
        format!("owner:{}", input.content)
    }
    fn dimension_name(&self) -> &'static str {
        "Owner"
    }
}

#[test]
fn owner_can_compose_and_override_without_command_runtime() {
    let mut registry = DomainAui::new("example")
        .standard()
        .window("history", Window::accumulated("History"))
        .build();
    registry.register("custom", OwnerRenderer);
    assert_eq!(
        registry
            .get("custom")
            .render(&RenderInput::with_content("present")),
        "owner:present"
    );
    assert!(registry.has("example_cli"));
    assert!(!registry.has("missing"));
    assert_eq!(
        registry
            .get("missing")
            .render(&RenderInput::with_content("unmodified")),
        "unmodified"
    );
    let timestamp = "2026-09-27T00:00:00Z".parse().unwrap();
    let mut input = RenderInput::empty();
    input.history = vec![
        HistoryEntry::with_timestamp("newer", timestamp),
        HistoryEntry::with_timestamp("older", timestamp),
    ];
    let text = registry.get("history").render(&input);
    assert!(text.find("older").unwrap() < text.find("newer").unwrap());
}
