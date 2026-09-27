# AUI rendering

`univers-aip-lib-aui-rendering` contains in-process text rendering mechanisms:
rolling CLI history, accumulated content, insight and suggestion windows, a
caller-populated renderer map, and the `DomainAui` composition builder. It has
no HTTP server, command dispatch, persistence, business data loader or provider
selection. Inputs are caller-owned values; the Insight value remains the public
Shared contract type rather than a copied owner definition.

```rust
use univers_aip_lib_aui_rendering::{DomainAui, Window, RenderInput};
let renderers = DomainAui::new("example")
    .window("details", Window::rolling("Details", 3))
    .build();
let rendered = renderers.get("details").render(&RenderInput::with_content("ready"));
assert!(rendered.contains("ready"));
```

`DimensionRenderer`, `HistoryEntry`, `RenderInput` and `ContentFormat` are the
library's in-process extension API. Consumers with old command-framework types
must switch imports or adapt explicit values at that boundary. This library
neither implements those old nominal types nor depends on the old SDK.

Owners choose window keys, renderers, ordering and supplied metadata. Convenience
window names do not query the named domains. Presentation defaults are preserved
from the existing renderer implementation, including fallback text and severity
labels; they are not measured data or an acceptance/authorization decision.
History is supplied newest first. Token limits retain the existing approximate
text budget and are not tokenizer-exact guarantees. `HistoryEntry::new` uses the
current clock; `with_timestamp` accepts an explicit timestamp for deterministic
rendering. The renderer map is local and has no global singleton.

Development: `bash scripts/check.sh` (format, strict Clippy, tests),
`bash scripts/build.sh`, `bash scripts/package.sh` and
`bash scripts/publish.sh`. Dependencies come from registries; no sibling checkout
is required. The release lock tests Shared 1.0.0-rc.1 compatibility.
