//! `List` — a view-state-aware tabular component for agent observations.
//!
//! `List` consumes the
//! agent-mutable [`DimensionViewConfig`] (filters / sort / pagination) and the
//! observation token budget, applying them centrally so a domain provider never
//! re-implements pagination, filtering, sorting, or budget-capping by hand.
//!
//! The agent drives the controls via `dimension configure <key> --page/--sort-by/
//! --filter` (which mutates the `DimensionViewConfig`); the next refresh re-renders
//! this component with the new view — i.e. the observe→act→observe loop is the
//! component's render→event→render loop, quantized by turn.

use univers_aip_contracts_operation::aui::DimensionViewConfig;

/// Default rows per page when the view config does not specify one.
const DEFAULT_PAGE_SIZE: usize = 20;
/// Soft cap on rendered characters (token-budget = the agent's viewport).
const DEFAULT_MAX_CHARS: usize = 3000;

/// A tabular list rendered to Markdown, honoring an agent's view config.
pub struct List {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    max_chars: usize,
}

impl List {
    /// Build a list from column headers and string rows.
    pub fn new(headers: Vec<impl Into<String>>, rows: Vec<Vec<String>>) -> Self {
        Self {
            headers: headers.into_iter().map(Into::into).collect(),
            rows,
            max_chars: DEFAULT_MAX_CHARS,
        }
    }

    /// Override the character budget for this list.
    pub fn with_max_chars(mut self, max: usize) -> Self {
        self.max_chars = max;
        self
    }

    /// Index of a header by case-insensitive name.
    fn col_index(&self, name: &str) -> Option<usize> {
        let name = name.trim().to_lowercase();
        self.headers
            .iter()
            .position(|h| h.trim().to_lowercase() == name)
    }

    /// Render to Markdown applying the view config's filter → sort → paginate,
    /// then a token-budget cap. Returns the table plus a footer that tells the
    /// agent the current page and how to drive the controls.
    pub fn render(&self, view: Option<&DimensionViewConfig>) -> String {
        let total = self.rows.len();

        // 1. Filter (each filter is matched as a case-insensitive substring on
        //    its named column; an unknown column name is ignored, never silently
        //    drops every row).
        let mut working: Vec<&Vec<String>> = self.rows.iter().collect();
        if let Some(v) = view {
            for (col, needle) in &v.filters {
                if let Some(idx) = self.col_index(col) {
                    let needle = needle.to_lowercase();
                    working.retain(|row| {
                        row.get(idx)
                            .map(|cell| cell.to_lowercase().contains(&needle))
                            .unwrap_or(false)
                    });
                }
            }
        }
        let filtered = working.len();

        // 2. Sort by the named column (numeric when both cells parse as f64,
        //    else lexicographic). sort_desc reverses.
        if let Some(v) = view {
            if let Some(col) = v.sort_by.as_ref().and_then(|c| self.col_index(c)) {
                working.sort_by(|a, b| {
                    let sa = a.get(col).map(String::as_str).unwrap_or("");
                    let sb = b.get(col).map(String::as_str).unwrap_or("");
                    match (sa.parse::<f64>(), sb.parse::<f64>()) {
                        (Ok(na), Ok(nb)) => {
                            na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal)
                        }
                        _ => sa.cmp(sb),
                    }
                });
                if v.sort_desc {
                    working.reverse();
                }
            }
        }

        // 3. Paginate (page is 1-based; clamp into range).
        let page_size = view
            .and_then(|v| v.page_size)
            .filter(|n| *n > 0)
            .unwrap_or(DEFAULT_PAGE_SIZE);
        let page_count = filtered.div_ceil(page_size).max(1);
        let page = view.and_then(|v| v.page).unwrap_or(1).clamp(1, page_count);
        let start = (page - 1) * page_size;
        let page_rows: Vec<&Vec<String>> =
            working.into_iter().skip(start).take(page_size).collect();

        // 4. Render Markdown table, capping to the budget.
        if total == 0 {
            return "_(none)_".to_string();
        }
        let mut out = String::new();
        out.push_str("| ");
        out.push_str(&self.headers.join(" | "));
        out.push_str(" |\n|");
        for _ in &self.headers {
            out.push_str("---|");
        }
        out.push('\n');

        let mut truncated_for_budget = 0usize;
        for (i, row) in page_rows.iter().enumerate() {
            let line = format!("| {} |\n", row.join(" | "));
            if out.len() + line.len() > self.max_chars {
                truncated_for_budget = page_rows.len() - i;
                break;
            }
            out.push_str(&line);
        }

        // 5. Footer: page position + filter note + budget note + control hint.
        let mut footer = format!("\n_{filtered} item(s)");
        if filtered != total {
            footer.push_str(&format!(" (filtered from {total})"));
        }
        footer.push_str(&format!(" · page {page}/{page_count}"));
        if truncated_for_budget > 0 {
            footer.push_str(&format!(
                " · {truncated_for_budget} row(s) hidden by token budget"
            ));
        }
        footer.push_str(" · `dimension configure <key> --page N --sort-by COL --filter COL=VAL`_");
        out.push_str(&footer);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn sample() -> List {
        List::new(
            vec!["id", "cop", "status"],
            vec![
                vec!["A".into(), "5.8".into(), "off".into()],
                vec!["B".into(), "2.1".into(), "on".into()],
                vec!["C".into(), "4.0".into(), "off".into()],
            ],
        )
    }

    fn view(
        filters: &[(&str, &str)],
        sort_by: Option<&str>,
        sort_desc: bool,
        page: Option<usize>,
        page_size: Option<usize>,
    ) -> DimensionViewConfig {
        DimensionViewConfig {
            filters: filters
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<HashMap<_, _>>(),
            sort_by: sort_by.map(str::to_string),
            sort_desc,
            page,
            page_size,
        }
    }

    #[test]
    fn filter_reduces_rows() {
        let out = sample().render(Some(&view(&[("status", "off")], None, false, None, None)));
        assert!(out.contains("| A |") && out.contains("| C |"));
        assert!(!out.contains("| B |"), "filtered-out row absent");
        assert!(out.contains("(filtered from 3)"));
    }

    #[test]
    fn numeric_sort_orders_rows() {
        // Sort by cop ascending: B(2.1) < C(4.0) < A(5.8). The first data row
        // must be B (numeric, not lexicographic where "2.1" < "4.0" < "5.8").
        let out = sample().render(Some(&view(&[], Some("cop"), false, None, None)));
        let b = out.find("| B |").unwrap();
        let c = out.find("| C |").unwrap();
        let a = out.find("| A |").unwrap();
        assert!(b < c && c < a, "ascending numeric order B<C<A");
    }

    #[test]
    fn sort_desc_reverses() {
        let out = sample().render(Some(&view(&[], Some("cop"), true, None, None)));
        let a = out.find("| A |").unwrap();
        let b = out.find("| B |").unwrap();
        assert!(a < b, "desc puts highest cop (A) first");
    }

    #[test]
    fn pagination_slices_and_reports() {
        let out = sample().render(Some(&view(&[], Some("id"), false, Some(2), Some(1))));
        assert!(out.contains("| B |"), "page 2 of size 1 (sorted by id) = B");
        assert!(!out.contains("| A |") && !out.contains("| C |"));
        assert!(out.contains("page 2/3"));
    }

    #[test]
    fn empty_list_is_explicit() {
        let out = List::new(vec!["id"], vec![]).render(None);
        assert_eq!(out, "_(none)_");
    }

    #[test]
    fn budget_cap_hides_overflow_rows() {
        let rows: Vec<Vec<String>> = (0..100)
            .map(|i| vec![format!("row{i}"), "x".repeat(50)])
            .collect();
        let out = List::new(vec!["id", "data"], rows)
            .with_max_chars(300)
            .render(Some(&view(&[], None, false, None, Some(100))));
        assert!(out.contains("hidden by token budget"), "budget note shown");
        assert!(out.len() < 600, "output stayed near the budget");
    }
}
