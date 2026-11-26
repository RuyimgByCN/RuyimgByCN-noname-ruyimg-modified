//! State Hash Divergence Bisector (architecture §35).
//!
//! Finds the first revision where two native replay logs diverge in state hash.

use crate::timeline::NativeReplayLog;

/// Result of a bisection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DivergenceResult {
    /// The sequence index (0-based) of the first divergent record, or `None`
    /// if the logs are identical.
    pub divergent_index: Option<usize>,
    /// The revision at which divergence first occurred.
    pub divergent_revision: Option<u64>,
    /// Hash from log A at the divergence point.
    pub hash_a: Option<[u8; 32]>,
    /// Hash from log B at the divergence point.
    pub hash_b: Option<[u8; 32]>,
    /// Total records compared.
    pub compared: usize,
}

/// Find the first divergence between two native replay logs.
pub fn find_divergence(a: &NativeReplayLog, b: &NativeReplayLog) -> DivergenceResult {
    let n = a.records.len().min(b.records.len());
    for i in 0..n {
        if a.records[i].post_state_hash != b.records[i].post_state_hash {
            return DivergenceResult {
                divergent_index: Some(i),
                divergent_revision: Some(a.records[i].post_revision),
                hash_a: Some(a.records[i].post_state_hash),
                hash_b: Some(b.records[i].post_state_hash),
                compared: i + 1,
            };
        }
    }
    if a.records.len() != b.records.len() {
        return DivergenceResult {
            divergent_index: Some(n),
            divergent_revision: None,
            hash_a: None,
            hash_b: None,
            compared: n,
        };
    }
    DivergenceResult {
        divergent_index: None,
        divergent_revision: None,
        hash_a: None,
        hash_b: None,
        compared: n,
    }
}

/// Render a divergence result as text.
pub fn render_divergence(result: &DivergenceResult) -> String {
    match result.divergent_index {
        None => format!(
            "logs are identical ({} records compared)\n",
            result.compared
        ),
        Some(idx) => {
            let mut out = format!("divergence at index {} ", idx);
            if let Some(rev) = result.divergent_revision {
                out.push_str(&format!("(revision {})", rev));
            }
            out.push('\n');
            if let (Some(ha), Some(hb)) = (result.hash_a, result.hash_b) {
                out.push_str(&format!(
                    "  log A hash: {}\n",
                    ha.iter().map(|b| format!("{b:02x}")).collect::<String>()
                ));
                out.push_str(&format!(
                    "  log B hash: {}\n",
                    hb.iter().map(|b| format!("{b:02x}")).collect::<String>()
                ));
            }
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_native::{Engine, GameAction, GameCommand};

    fn run_trace(seed: u64) -> NativeReplayLog {
        let mut eng = Engine::new(seed, 2);
        let mut log = NativeReplayLog::new();
        let actor = eng.turn().current;
        let rev = eng.revision();
        log.submit_and_record(
            &mut eng,
            GameCommand {
                actor,
                expected_revision: rev,
                dedup_id: 1,
                action: GameAction::NextPhase,
            },
        );
        log
    }

    #[test]
    fn identical_logs_no_divergence() {
        let a = run_trace(5);
        let b = run_trace(5);
        let result = find_divergence(&a, &b);
        assert!(result.divergent_index.is_none());
    }

    #[test]
    fn different_seed_detected_as_divergence() {
        let a = run_trace(1);
        let b = run_trace(2);
        let result = find_divergence(&a, &b);
        assert_eq!(result.divergent_index, Some(0));
    }
}
