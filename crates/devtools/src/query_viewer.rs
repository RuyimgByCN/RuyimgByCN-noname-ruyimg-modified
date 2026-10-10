//! Query Explanation Viewer (architecture §35).
//!
//! Renders the trace from `query_explain` as a human-readable breakdown of
//! base value + modifier sources → final result.

use engine_core::QueryExplanation;
use rule_api::QueryResponse;

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
            out.push_str(&format!(
                "    {:<24} {}{}\n",
                src.label_key, sign, src.value
            ));
        }
    }

    match &expl.response {
        QueryResponse::Int(v) => out.push_str(&format!("  final:              {v}\n")),
        QueryResponse::Bool(b) => out.push_str(&format!("  final:              {b}\n")),
        QueryResponse::Error { code } => {
            out.push_str(&format!("  final:              ERROR({code:?})\n"))
        }
        _ => out.push_str("  final:              (other)\n"),
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_api::PlayerId;
    use engine_core::{query_explain, Engine, QueryContext};
    use rule_api::{ModifierSource, QueryRequest};

    #[test]
    fn renders_distance_explanation() {
        let mut eng = Engine::new();
        eng.submit(engine_api::Command {
            actor: PlayerId::NONE,
            expected_revision: engine_api::ExpectedRevision(0),
            idempotency_key: engine_api::IdempotencyKey(0),
            kind: engine_api::CommandKind::StartGame {
                player_count: 4,
                rng_seed: 42,
                ruleset: "identity-standard".into(),
            },
        });
        let mut ctx = QueryContext::new();
        let expl = query_explain(
            eng.state(),
            &QueryRequest::Distance {
                from: PlayerId::new(1),
                to: PlayerId::new(2),
            },
            &mut ctx,
        );
        let rendered = render_explanation(&expl, "distance(1→2)");
        assert!(rendered.contains("distance(1→2)"));
        assert!(rendered.contains("final:"));
    }
}
