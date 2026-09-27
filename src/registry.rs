//! Process-local registry for AUI dimension renderers.

use std::collections::HashMap;

use crate::{DimensionRenderer, RenderInput, RenderStrategy};

#[derive(Debug, Clone)]
struct FallbackRenderer;

impl DimensionRenderer for FallbackRenderer {
    fn strategy(&self) -> RenderStrategy {
        RenderStrategy::default()
    }

    fn render(&self, input: &RenderInput) -> String {
        if input.content.is_empty() {
            return "(empty)".to_string();
        }
        input.content.clone()
    }

    fn dimension_name(&self) -> &'static str {
        "unknown"
    }
}

/// Mutable renderer registry assembled by a process composition root.
pub struct RendererRegistry {
    renderers: HashMap<String, Box<dyn DimensionRenderer>>,
    fallback_renderer: FallbackRenderer,
}

impl std::fmt::Debug for RendererRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RendererRegistry")
            .field("renderers", &self.renderers.keys().collect::<Vec<_>>())
            .field("fallback_renderer", &"unknown")
            .finish()
    }
}

impl RendererRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            renderers: HashMap::new(),
            fallback_renderer: FallbackRenderer,
        }
    }

    pub fn register(
        &mut self,
        dimension_key: impl Into<String>,
        renderer: impl DimensionRenderer + 'static,
    ) {
        self.renderers
            .insert(dimension_key.into(), Box::new(renderer));
    }

    pub fn register_boxed(
        &mut self,
        dimension_key: impl Into<String>,
        renderer: Box<dyn DimensionRenderer>,
    ) {
        self.renderers.insert(dimension_key.into(), renderer);
    }

    #[must_use]
    pub fn get(&self, dimension_key: &str) -> &dyn DimensionRenderer {
        self.renderers
            .get(dimension_key)
            .map_or(&self.fallback_renderer, |renderer| renderer.as_ref())
    }

    #[must_use]
    pub fn has(&self, dimension_key: &str) -> bool {
        self.renderers.contains_key(dimension_key)
    }
}

impl Default for RendererRegistry {
    fn default() -> Self {
        Self::new()
    }
}
