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

## Optional text components (0.1.1)

Enable `components` for `components::{Stat, List}`. `Stat` renders explicit
caller-provided counts/metrics. `List::new(headers, rows).render(view)` applies
the public Operation `DimensionViewConfig` to caller-owned text rows, retaining
filter/sort/pagination, escaping and character-budget behavior. It performs no
query or mutation. The release lock tests Operation and Shared rc.1.

`Stat` retains the legacy empty-builder text `0`; callers must distinguish an
empty summary from unavailable measurements before constructing presentation.
`List` retains its 20-row and 3000-character defaults, configurable view/character
budget, and legacy behavior for unknown filter/sort columns. This feature neither
loads owner data nor establishes World/tenant scope or observation provenance.
