use super::*;

// ---------------------------------------------------------------------------
// 官方武将迁移：张飞（咆哮）——杀无次数限制（ADR 0752）
// ---------------------------------------------------------------------------

/// 咆哮：出牌阶段使用杀无次数限制（连续出杀均被接受）。
#[test]
fn zhangfei_paoxiao_removes_sha_limit() {
    let mut e = Engine::new(613, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "咆哮");
    // 三张杀连续使用全部接受（对比无咆哮时第二条 ShaLimit 拒绝）
    let shas: Vec<u32> = (0..3).map(|_| e.debug_place(0, "sha")).collect();
    let mut damaged = 0;
    for sha in &shas {
        let out = play(&mut e, *sha, vec![1]); // 1 无闪 → 受伤
        assert!(
            matches!(out, CommandOutcome::Accepted { .. }),
            "咆哮下连续出杀不应受限: {out:?}"
        );
        damaged += 1;
    }
    assert_eq!(damaged, 3, "咆哮应允许连续出三杀");
    assert_eq!(hp(&e, 1), 1, "三杀各造成 1 点伤害");
    assert_zone_conservation(&e);
}
