use super::*;

/// 主公资格（身份局）：非主公的护驾/激将持有者无法主动发动。
#[test]
fn zhu_skills_require_lord_role_in_identity_mode() {
    let mut e = Engine::new(2805, 4);
    // 默认身份局：座位 0 主公、1 忠、2/3 反贼
    for (p, skill) in [(0usize, "护驾"), (2, "激将")] {
        e.debug_grant(p as u8, skill);
    }
    advance_to_play(&mut e, 2);
    let hujia = e.catalog().skill_by_name("护驾").unwrap();
    let out = submit_cmd(&mut e, 2, GameAction::ActivateSkill { skill: hujia });
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::NoSuchSkill)),
        "反贼未持护驾应拒绝: {out:?}"
    );
    let jijiang = e.catalog().skill_by_name("激将").unwrap();
    e.debug_grant(1, "激将");
    // 忠臣须在自己的出牌阶段尝试（先过回合/阶段门控，只验主公资格）
    advance_to_play(&mut e, 1);
    let out2 = submit_cmd(&mut e, 1, GameAction::ActivateSkill { skill: jijiang });
    assert!(
        matches!(out2, CommandOutcome::Rejected(RejectReason::InvalidChoice)),
        "忠臣持激将但非主公 → 资格不符应拒绝: {out2:?}"
    );
    // 主公（座位 0）持激将可发动（当前是否轮到 0 由 turn 决定，仅验资格
    // 相关拒绝不出现 NotLord 类问题——直接用主公回合验证）
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "激将");
    e.debug_set_general(1, "关羽"); // 蜀候选
    let out3 = submit_cmd(&mut e, 0, GameAction::ActivateSkill { skill: jijiang });
    assert!(
        matches!(out3, CommandOutcome::Pending { .. }),
        "主公发动激将应进入逐人询问: {out3:?}"
    );
}
