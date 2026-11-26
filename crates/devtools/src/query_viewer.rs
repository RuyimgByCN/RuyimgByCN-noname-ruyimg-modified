//! Query Explanation Viewer (architecture §35).
//!
//! Computes and renders native distance/attack-range breakdown from base value +
//! modifier delta → final result.

use engine_native::Engine;

/// Single modifier contribution in a query explanation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModifierContribution {
    /// Human-readable label key.
    pub label_key: String,
    /// Signed integer contribution.
    pub value: i32,
}

/// Breakdown of a native query result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryExplanation {
    /// Base metric value (e.g. raw alive seat distance or default attack range).
    pub base: Option<i32>,
    /// Applied modifier contributions.
    pub sources: Vec<ModifierContribution>,
    /// Final computed integer result.
    pub final_value: i32,
}

/// Explain native `computed_distance(from -> to)` with base seat distance and modifier delta.
pub fn explain_distance(engine: &Engine, from: u8, to: u8) -> QueryExplanation {
    let core = engine.state_core();
    let base = core.seat_distance_alive(from, to) as i32;
    let computed = core.computed_distance(engine.catalog(), from, to) as i32;
    let delta = computed - base;
    let sources = if delta != 0 {
        vec![ModifierContribution {
            label_key: "distance_modifiers".into(),
            value: delta,
        }]
    } else {
        Vec::new()
    };
    QueryExplanation {
        base: Some(base),
        sources,
        final_value: computed,
    }
}

/// Explain native `attack_range(player)` with base range (1) and equip/skill modifier delta.
pub fn explain_attack_range(engine: &Engine, player: u8) -> QueryExplanation {
    let base = 1;
    let final_value = engine.state_core().attack_range(engine.catalog(), player) as i32;
    let delta = final_value - base;
    let sources = if delta != 0 {
        vec![ModifierContribution {
            label_key: "range_modifiers".into(),
            value: delta,
        }]
    } else {
        Vec::new()
    };
    QueryExplanation {
        base: Some(base),
        sources,
        final_value,
    }
}

/// Render a query explanation as a text tree.
pub fn render_explanation(expl: &QueryExplanation, query_label: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!("=== Query: {query_label} ===\n"));

    if let Some(base) = expl.base {
        out.push_str(&format!("  base:               {base}\n"));
    }

    if expl.sources.is_empty() {
        out.push_str("  modifiers:          (none)\n");
    } else {
        out.push_str("  modifiers:\n");
        for src in &expl.sources {
            let sign = if src.value >= 0 { "+" } else { "" };
            out.push_str(&format!("    {:<24} {}{}\n", src.label_key, sign, src.value));
        }
    }

    out.push_str(&format!("  final:              {}\n", expl.final_value));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_distance_explanation() {
        let eng = Engine::new(42, 4);
        let expl = explain_distance(&eng, 0, 1);
        let rendered = render_explanation(&expl, "distance(0→1)");
        assert!(rendered.contains("distance(0→1)"));
        assert!(rendered.contains("final:"));
        assert_eq!(expl.final_value, 1);
    }
}
