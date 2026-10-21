use super::*;

impl Engine {
    /// N10 有界计算池：设置每座位每轮命令配额（None = 不限制；
    /// 会话策略，不入快照——恢复后需重新设置）。
    pub fn set_seat_quota(&mut self, quota: Option<u32>) {
        self.seat_quota = quota;
    }

    /// N11 认证：为座位签发会话令牌（重复签发覆盖旧令牌——旧令牌立即失效；
    /// 令牌为宿主会话数据，不入快照，恢复后需重新签发）。
    pub fn issue_token(&mut self, seat: PlayerId) -> AuthToken {
        self.auth_counter += 1;
        // N11-a：实例熵（OS 随机，跨实例不重叠）× 代次（恢复轮换，
        // 恢复前旧令牌失效）× 座位 × 计数器 混合派生。
        let id = self.auth_secret
            ^ self.auth_generation.wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ ((seat as u64).rotate_left(17))
            ^ (self.auth_counter << 24);
        self.auth_tokens.insert(seat, id);
        AuthToken { seat, id }
    }

    /// N11 认证：经认证的命令提交——令牌必须存在且与命令 actor 绑定一致。
    /// 令牌失效 → `NoSuchSession`；令牌座位 ≠ 命令 actor → `Impersonation`；
    /// 校验通过后走与 [`Engine::submit`] 完全相同的主通道。
    pub fn submit_authed(&mut self, token: AuthToken, cmd: GameCommand) -> CommandOutcome {
        match self.auth_tokens.get(&token.seat) {
            None => return CommandOutcome::Rejected(RejectReason::NoSuchSession),
            Some(&id) if id != token.id => {
                return CommandOutcome::Rejected(RejectReason::NoSuchSession)
            }
            Some(_) => {}
        }
        if token.seat != cmd.actor {
            return CommandOutcome::Rejected(RejectReason::Impersonation);
        }
        self.submit(cmd)
    }

    /// N10：座位本轮剩余配额（未设置配额返回 None）。
    pub fn seat_quota_remaining(&self, seat: PlayerId) -> Option<u32> {
        let q = self.seat_quota?;
        let round = self.state.core.turn.round;
        let used = self
            .seat_cmd_counts
            .get(&seat)
            .and_then(|(r, c)| (*r == round).then_some(*c))
            .unwrap_or(0);
        Some(q.saturating_sub(used))
    }
}
