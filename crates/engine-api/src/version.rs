//! Independent version axes (architecture §11.3).

use serde::{Deserialize, Serialize};

/// Current content schema version shared by hosts and package tooling.
pub const CONTENT_SCHEMA_VERSION: u32 = 3;

/// Current installable package format version.
pub const PACKAGE_FORMAT_VERSION: u32 = 1;

/// Current additive Rule VM host ABI version shared by packages and runtimes.
///
/// Parallel agents share the workspace; ABI numbers are allocated serially in
/// `ABI-COORDINATION.md` (repo root). Bumping this constant requires first
/// claiming the number there — the `abort_bump_without_claim` test below
/// enforces that invariant.
pub const RULE_VM_ABI_VERSION: u32 = 734;

/// Bundle of version identifiers recorded in replays and packages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionBundle {
    /// Engine API version.
    pub engine_api: u32,
    /// Rule VM ABI version.
    pub rule_vm_abi: u32,
    /// Content schema version.
    pub content_schema: u32,
    /// Network protocol version.
    pub network_protocol: u32,
    /// Replay schema version.
    pub replay_schema: u32,
    /// Snapshot schema version.
    pub snapshot_schema: u32,
    /// Package format version.
    pub package_format: u32,
    /// Build id string (exact binary identity).
    pub engine_build_id: String,
}

impl VersionBundle {
    /// Current Phase-1 workspace versions.
    pub fn current() -> Self {
        Self {
            engine_api: 1,
            rule_vm_abi: RULE_VM_ABI_VERSION,
            content_schema: CONTENT_SCHEMA_VERSION,
            network_protocol: 5,
            replay_schema: crate::REPLAY_SCHEMA_VERSION,
            snapshot_schema: crate::SNAPSHOT_SCHEMA_VERSION,
            package_format: PACKAGE_FORMAT_VERSION,
            engine_build_id: crate::ENGINE_BUILD_ID.to_string(),
        }
    }
}

#[cfg(test)]
mod abi_coordination_tests {
    use super::RULE_VM_ABI_VERSION;

    /// ABI-MAX marker in `ABI-COORDINATION.md`, kept machine-readable.
    const COORDINATION: &str = include_str!("../../../ABI-COORDINATION.md");

    fn abi_max() -> u32 {
        for line in COORDINATION.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("ABI-MAX:") {
                return rest
                    .trim()
                    .parse()
                    .expect("ABI-COORDINATION.md: ABI-MAX must be an integer");
            }
        }
        panic!("ABI-COORDINATION.md: missing ABI-MAX marker line");
    }

    /// Claimed numbers in the ABI claims table only (the section between the
    /// `## ABI 领取表` and `## ADR 领取表` headers). ADR rows (`| 0293 |`) share
    /// the same table shape but are a separate number space.
    fn claimed_numbers() -> Vec<u32> {
        let mut in_abi_table = false;
        let mut numbers = Vec::new();
        for line in COORDINATION.lines() {
            if line.starts_with("## ABI 领取表") {
                in_abi_table = true;
                continue;
            }
            if in_abi_table && line.starts_with("## ADR 领取表") {
                break;
            }
            if !in_abi_table {
                continue;
            }
            let line = line.trim_start();
            let Some(rest) = line.strip_prefix('|') else {
                continue;
            };
            let number = rest.trim_start().split('|').next().unwrap_or("").trim();
            if let Ok(n) = number.parse() {
                numbers.push(n);
            }
        }
        numbers
    }

    #[test]
    fn abort_bump_without_claim() {
        let max = abi_max();
        assert!(
            RULE_VM_ABI_VERSION <= max,
            "RULE_VM_ABI_VERSION={RULE_VM_ABI_VERSION} exceeds ABI-MAX={max} in \
             ABI-COORDINATION.md. Parallel agents share this workspace: claim the new ABI number \
             in ABI-COORDINATION.md (update ABI-MAX and the claims table) before bumping the \
             engine constant."
        );
    }

    #[test]
    fn claims_are_unique() {
        let mut numbers = claimed_numbers();
        numbers.sort_unstable();
        let duplicates: Vec<_> = numbers
            .windows(2)
            .filter(|pair| pair[0] == pair[1])
            .map(|pair| pair[0])
            .collect();
        assert!(
            duplicates.is_empty(),
            "ABI-COORDINATION.md claims table has duplicate ABI numbers: {duplicates:?}"
        );
    }
}
