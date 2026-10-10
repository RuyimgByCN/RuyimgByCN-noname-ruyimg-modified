//! Search-only RNG — never touches RuleRng / authority state.

use serde::{Deserialize, Serialize};

/// Deterministic xorshift64* for AI search (independent stream).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchRng {
    state: u64,
    call_count: u64,
}

impl SearchRng {
    /// From seed (0 remapped).
    pub fn from_seed(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x9e3779b97f4a7c15 } else { seed },
            call_count: 0,
        }
    }

    /// Call count (for reproducibility checks).
    pub fn call_count(self) -> u64 {
        self.call_count
    }

    /// Next u64.
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        self.call_count = self.call_count.wrapping_add(1);
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    /// Uniform in `0..max` (max > 0).
    pub fn gen_range_u32(&mut self, max: u32) -> u32 {
        assert!(max > 0);
        let max = max as u64;
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

    /// Pick index weighted by non-negative scores (sum > 0).
    pub fn weighted_index(&mut self, weights: &[i32]) -> Option<usize> {
        let mut sum: i64 = 0;
        for w in weights {
            sum += i64::from((*w).max(0));
        }
        if sum <= 0 {
            return None;
        }
        let mut pick = self.gen_range_u32(sum as u32) as i64;
        for (i, w) in weights.iter().enumerate() {
            pick -= i64::from((*w).max(0));
            if pick < 0 {
                return Some(i);
            }
        }
        Some(weights.len() - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic() {
        let mut a = SearchRng::from_seed(1);
        let mut b = SearchRng::from_seed(1);
        for _ in 0..50 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }
}
