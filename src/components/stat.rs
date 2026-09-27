//! `Stat` — a compact, consistent overview summary for the Layer-1 catalog.
//!
//! Every domain's `*_overview` should read the same way ("331 total, 20
//! critical, 127 high"), so the World State catalog is scannable and the
//! "total" / count semantics don't drift across 23 hand-rolled providers.
//! `Stat` is the cheap-by-construction building block a `ComponentProvider`'s
//! `overview()` returns — it carries counts only, never row scans.

use std::fmt::Display;

/// A compact `value label` summary line.
#[derive(Default)]
pub struct Stat {
    items: Vec<(String, String)>,
}

impl Stat {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a `count label` pair, e.g. `count("total", 331)` → `331 total`.
    pub fn count(mut self, label: impl Into<String>, n: i64) -> Self {
        self.items.push((n.to_string(), label.into()));
        self
    }

    /// Add an arbitrary `value label` pair (non-integer metrics).
    pub fn metric(mut self, label: impl Into<String>, value: impl Display) -> Self {
        self.items.push((value.to_string(), label.into()));
        self
    }

    /// Add a count only when it is non-zero (keeps overviews terse).
    pub fn count_if_nonzero(self, label: impl Into<String>, n: i64) -> Self {
        if n != 0 {
            self.count(label, n)
        } else {
            self
        }
    }

    /// Render the comma-joined summary, e.g. `331 total, 20 critical, 127 high`.
    pub fn render(&self) -> String {
        if self.items.is_empty() {
            return "0".to_string();
        }
        self.items
            .iter()
            .map(|(v, l)| format!("{v} {l}"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_consistent_summary() {
        let s = Stat::new()
            .count("total", 331)
            .count("critical", 20)
            .count("high", 127);
        assert_eq!(s.render(), "331 total, 20 critical, 127 high");
    }

    #[test]
    fn count_if_nonzero_drops_zeros() {
        let s = Stat::new()
            .count("total", 50)
            .count_if_nonzero("overdue", 0)
            .count_if_nonzero("running", 3);
        assert_eq!(s.render(), "50 total, 3 running");
    }

    #[test]
    fn empty_is_zero() {
        assert_eq!(Stat::new().render(), "0");
    }

    #[test]
    fn metric_accepts_non_integers() {
        let s = Stat::new().metric("avg cop", 4.25).count("units", 9);
        assert_eq!(s.render(), "4.25 avg cop, 9 units");
    }
}
