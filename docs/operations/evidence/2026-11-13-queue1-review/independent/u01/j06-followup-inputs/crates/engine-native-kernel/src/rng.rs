//! 确定性规则随机数：xorshift64*，与现有引擎同族。
//!
//! 流状态保存在对局状态内（参与摘要哈希），保证重放一致。
//! 展示层/AI 专用随机不得使用此流。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuleRng {
    state: u64,
    calls: u64,
}

impl RuleRng {
    /// 存档编码用：原始内部状态（state, calls）。
    pub(crate) fn raw_parts(&self) -> (u64, u64) {
        (self.state, self.calls)
    }

    /// 存档解码用：由原始内部状态重建。
    pub(crate) fn from_raw_parts(state: u64, calls: u64) -> Self {
        Self { state, calls }
    }

    /// 种子 0 重映射为固定常数，避免零状态锁死。
    pub fn from_seed(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed },
            calls: 0,
        }
    }

    pub fn state(&self) -> (u64, u64) {
        (self.state, self.calls)
    }

    pub fn calls(&self) -> u64 {
        self.calls
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        self.calls += 1;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// [0, bound) 无偏采样（Lemire 除法拒绝法）。
    pub fn gen_range(&mut self, bound: u32) -> u32 {
        debug_assert!(bound > 0);
        let bound = bound as u64;
        loop {
            let x = self.next_u64();
            let product = x as u128 * bound as u128;
            let m = (product >> 64) as u64;
            let lo = product & u128::from(u64::MAX);
            if lo < bound as u128 {
                continue;
            }
            return m as u32;
        }
    }

    /// 确定性 Fisher–Yates 洗牌。
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        for i in (1..slice.len()).rev() {
            let j = self.gen_range((i + 1) as u32) as usize;
            slice.swap(i, j);
        }
    }
}
