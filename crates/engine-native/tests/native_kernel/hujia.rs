use super::*;

// ---------------------------------------------------------------------------
// N10/N08 激将/护驾逐人询问重构（ADR 0776，规则版本 29）：
// 按资格（魏/蜀势力）与座位顺序逐人询问；不以暗手牌预筛；拒绝后
// 继续、合法响应后结束；主公资格；公开询问不泄漏有无响应牌。
// ---------------------------------------------------------------------------

/// 护驾逐人询问：两名魏将按座位序被问；首位 Pass 后第二位被问；
/// 第二位 Confirm 代出闪抵消伤害。
#[test]
fn hujia_asks_wei_helpers_in_seat_order_and_continues_after_pass() {
    let mut e = Engine::new(2801, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(1, "护驾");
    e.debug_set_hp(1, 4);
    e.debug_set_general(2, "夏侯惇"); // 魏
    e.debug_set_general(3, "张辽"); // 魏
    let shan3 = e.debug_place(3, "shan");
    let sha = e.debug_place(0, "sha");
    play(&mut e, sha, vec![1]);
    // 首位 = 座位序首名魏将（2 号），即使 2 号无闪也被问（不预筛）
    let c = e.open_choice().expect("护驾询问应挂起");
    assert_eq!(
        (c.spec.prompt, c.observer),
        ("hujia_respond", 2),
        "应先问 2 号魏将"
    );
    answer_empty(&mut e); // 2 号无闪 → 空选=拒绝
                          // 首位拒绝后继续问 3 号
    let c2 = e.open_choice().expect("拒绝后应继续问下一位魏将");
    assert_eq!((c2.spec.prompt, c2.observer), ("hujia_respond", 3));
    answer_sel(&mut e, vec![ChoiceOption::Card(shan3)]); // 自选自己的闪代出
    assert_eq!(e.debug_hp(1), 4, "代出闪应抵消伤害");
    assert!(!e.debug_hand(3).contains(&shan3), "3 号的闪应被弃置");
    assert_zone_conservation(&e);
}

/// 护驾公开询问不泄漏暗牌：互换两名魏将暗手牌中的闪，攻击者可见
/// 视图与公开询问序列完全一致（询问资格公开、应答私密）。
#[test]
fn hujia_public_asks_do_not_leak_hidden_shan() {
    let build = |seed: u64| -> Engine {
        let mut e = Engine::new(seed, 4);
        advance_to_play(&mut e, 0);
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        e.debug_grant(1, "护驾");
        e.debug_set_general(2, "夏侯惇");
        e.debug_set_general(3, "张辽");
        e.debug_set_hp(1, 4);
        e
    };
    // 布局 A：2=闪/3=杀；布局 B：2=杀/3=闪——两名魏将各持一张暗牌，
    // 手牌数相同（数量公开），内容互换（内容私密）
    let mut a = build(2802);
    let _shan2 = a.debug_place(2, "shan");
    let _sha3 = a.debug_place(3, "sha");
    let mut b = build(2802);
    let _sha2 = b.debug_place(2, "sha");
    let _shan3 = b.debug_place(3, "shan");
    let sha_a = a.debug_place(0, "sha");
    let sha_b = b.debug_place(0, "sha");
    // 攻击者视角：他人暗手牌不可见 → 两局攻击者视图相等
    let view_a = a.view_for(0);
    let view_b = b.view_for(0);
    assert_eq!(view_a, view_b, "互换暗闪后攻击者可见视图必须相同");
    play(&mut a, sha_a, vec![1]);
    play(&mut b, sha_b, vec![1]);
    // 公开询问序列相同：都是先问 2 再问 3（不因持闪与否改变）
    let ca = a.open_choice().expect("A 询问挂起");
    let cb = b.open_choice().expect("B 询问挂起");
    assert_eq!((ca.observer, ca.spec.prompt), (cb.observer, cb.spec.prompt));
    answer(&mut a, ChoiceOption::Pass);
    answer(&mut b, ChoiceOption::Pass);
    let ca2 = a.open_choice().expect("A 第二问");
    let cb2 = b.open_choice().expect("B 第二问");
    assert_eq!(
        (ca2.observer, ca2.spec.prompt),
        (cb2.observer, cb2.spec.prompt)
    );
}

/// 护驾只问魏势力：非魏（蜀）角色持闪也不被问。
#[test]
fn hujia_skips_non_wei_helpers() {
    let mut e = Engine::new(2803, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(1, "护驾");
    e.debug_set_general(2, "关羽"); // 蜀——持闪也不合格
    e.debug_set_general(3, "夏侯惇"); // 魏
    let _shan2 = e.debug_place(2, "shan");
    let sha = e.debug_place(0, "sha");
    e.debug_set_hp(1, 4);
    play(&mut e, sha, vec![1]);
    let c = e.open_choice().expect("护驾询问应挂起");
    assert_eq!(c.observer, 3, "应跳过蜀势力、直接问魏将 3 号");
}

/// 护驾 + 无双：代出闪计入两张需求——helper 出一张后仍需目标自己出闪。
#[test]
fn hujia_shan_counts_toward_wushuang_requirement() {
    let mut e = Engine::new(2804, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(1, "护驾");
    e.debug_grant(0, "无双");
    e.debug_set_general(2, "夏侯惇");
    e.debug_set_hp(1, 4);
    let shan2 = e.debug_place(2, "shan");
    let shan1 = e.debug_place(1, "shan");
    let sha = e.debug_place(0, "sha");
    play(&mut e, sha, vec![1]);
    // 护驾询问 → 2 号自选自己的闪代出（第一张）
    let c = e.open_choice().expect("护驾询问");
    assert_eq!((c.spec.prompt, c.observer), ("hujia_respond", 2));
    answer_sel(&mut e, vec![ChoiceOption::Card(shan2)]);
    // 无双第二张：新响应窗口——护驾再次逐人询问（2 号已无闪，空选拒绝）
    let c2 = e.open_choice().expect("新窗口应重开护驾逐人询问");
    assert_eq!((c2.spec.prompt, c2.observer), ("hujia_respond", 2));
    answer_empty(&mut e);
    // 然后目标 1 自己被问出闪
    let t = e.open_choice().expect("无双第二张应询问目标本人");
    assert_eq!((t.spec.prompt, t.observer), ("respond_shan", 1));
    answer_sel(&mut e, vec![ChoiceOption::Card(shan1)]);
    assert_eq!(e.debug_hp(1), 4, "两张闪齐 → 免伤");
    assert_zone_conservation(&e);
}

/// 护驾逐人询问挂起点的字节恢复（v2 存档：队列/持牌者入快照）。
#[test]
fn hujia_pending_ask_byte_roundtrip_restores() {
    use engine_native::archive::{decode_archive, encode_archive};
    let mut e = Engine::new(2806, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(1, "护驾");
    e.debug_set_general(2, "夏侯惇");
    e.debug_set_general(3, "张辽");
    e.debug_set_hp(1, 4);
    let shan3 = e.debug_place(3, "shan");
    let sha = e.debug_place(0, "sha");
    play(&mut e, sha, vec![1]);
    // 2 号空选（无闪）拒绝后挂起在 3 号的询问 → 字节往返 → 恢复件继续应答
    answer_empty(&mut e);
    let bytes = encode_archive(&e.save());
    let mut restored = Engine::restore(&decode_archive(&bytes).expect("decode")).expect("字节恢复");
    let c = restored.open_choice().expect("恢复件仍挂起");
    assert_eq!((c.spec.prompt, c.observer), ("hujia_respond", 3));
    let cid = c.id;
    let out = restored.submit(GameCommand {
        actor: 3,
        expected_revision: restored.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(shan3)],
        },
    });
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    assert_eq!(restored.debug_hp(1), 4, "恢复件代出闪仍抵消伤害");
    assert!(!restored.debug_hand(3).contains(&shan3));
}
