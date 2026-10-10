//! Deterministic rule RNG.
//!
//! All rule-relevant randomness must go through `RuleRng`.
//! AI / visual / asset randomness must use separate streams.

use serde::{Deserialize, Serialize};

/// Serializable RNG state (xorshift64* — deterministic across native/WASM).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleRngState {
    /// Current state (must never be zero after init).
    pub state: u64,
    /// Number of `next_u64` calls since init (for cross-target checks).
    pub call_count: u64,
}

impl Default for RuleRngState {
    fn default() -> Self {
        Self {
            state: 0x4d595df4d0f33173,
            call_count: 0,
        }
    }
}

/// Rule RNG handle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleRng {
    /// Internal state.
    state: RuleRngState,
}

impl RuleRng {
    /// Create from seed. Zero seed is remapped to a fixed non-zero constant.
    pub fn from_seed(seed: u64) -> Self {
        let state = if seed == 0 { 0x4d595df4d0f33173 } else { seed };
        Self {
            state: RuleRngState {
                state,
                call_count: 0,
            },
        }
    }

    /// Restore from saved state.
    pub fn from_state(state: RuleRngState) -> Self {
        let mut s = state;
        if s.state == 0 {
            s.state = 0x4d595df4d0f33173;
        }
        Self { state: s }
    }

    /// Borrow state for hashing / snapshot.
    pub fn state(&self) -> &RuleRngState {
        &self.state
    }

    /// Consume into state.
    pub fn into_state(self) -> RuleRngState {
        self.state
    }

    /// Next u64 (xorshift64*).
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state.state = x;
        self.state.call_count = self.state.call_count.wrapping_add(1);
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    /// Uniform in `0..max` (max > 0). Uses rejection sampling for lack of bias.
    pub fn gen_range_u32(&mut self, max: u32) -> u32 {
        assert!(max > 0, "gen_range_u32 max must be > 0");
        let max = max as u64;
        // Lemire-style nearly divisionless, simplified for clarity & determinism.
        let mut r = self.next_u64();
        let mut m = (r as u128) * (max as u128);
        let mut lo = m as u64;
        if lo < max {
            let t = max.wrapping_neg() % max;
            while lo < t {
                r = self.next_u64();
                m = (r as u128) * (max as u128);
                lo = m as u64;
            }
        }
        (m >> 64) as u32
    }

    /// Shuffle slice in place with deterministic Fisher–Yates.
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        let n = items.len();
        if n < 2 {
            return;
        }
        for i in (1..n).rev() {
            let j = self.gen_range_u32((i + 1) as u32) as usize;
            items.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_stream() {
        let mut a = RuleRng::from_seed(42);
        let mut b = RuleRng::from_seed(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
        assert_eq!(a.state().call_count, 100);
    }

    #[test]
    fn zero_seed_is_stable() {
        let mut a = RuleRng::from_seed(0);
        let mut b = RuleRng::from_seed(0);
        assert_eq!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn shuffle_is_deterministic() {
        let mut a = (0..20u32).collect::<Vec<_>>();
        let mut b = a.clone();
        RuleRng::from_seed(7).shuffle(&mut a);
        RuleRng::from_seed(7).shuffle(&mut b);
        assert_eq!(a, b);
        assert_ne!(a, (0..20u32).collect::<Vec<_>>());
    }
}
