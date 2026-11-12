//! L28 雷霆万钧 Batch B（武将 10–14）与装备/多技能交互的 6 维真实对局验收（2026-11-12）。
//! 共用 `l28_thunder_eight_match.rs` 的命令驱动与存档辅助函数；六维编号同该文件头注释。

use super::l28_thunder_eight_match::*;
use engine_native::catalog::{CardKind, EquipSlot, Suit};
use engine_native::event::Phase;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction};

fn is_rejected(out: CommandOutcome) -> bool {
    matches!(out, CommandOutcome::Rejected(_))
}

fn clear_hand(e: &mut Engine, seat: u8) {
    let cards: Vec<u32> = e.state_core_mut().player_mut(seat).hand.drain(..).collect();
    let eqs: Vec<u32> = e.state_core_mut().player_mut(seat).equips.drain(..).collect();
    e.state_core_mut().discard.extend(cards);
    e.state_core_mut().discard.extend(eqs);
}

/// 4. Batch B：精械/巧思/天算/鼓舌/激词/割圆/解术/孤算/业炎/琴音。
#[test]
fn l28_thunder_eight_batch_b_skills() {
    let p = pack();

    // ---- th_jingxie 精械（`extension.js:7429`）：出牌阶段展示升级；濒死时重铸防具回复至 1 ----
    let mut e = fresh(&p, 2841, "th_majun");
    clear_hand(&mut e, 0);
    assert!(
        is_rejected(activate(&mut e, 0, "th_jingxie")),
        "④ no armor/zhuge -> cannot activate"
    );
    let armor = card_by(&e, |c| c.kind.equip_slot() == Some(EquipSlot::Armor));
    give(&mut e, 0, armor);
    let foreign = e.state_core().player(1).hand[0];
    activate(&mut e, 0, "th_jingxie");
    answer(&mut e, "th_jingxie:choose_card", Vec::new());
    assert!(
        !once_used(&e, 0, "th_jingxie") && flag(&e, 0, "th_jingxie_upgraded") == 0,
        "② cancel leaves card un-upgraded"
    );
    activate(&mut e, 0, "th_jingxie");
    assert!(
        rejected(&mut e, vec![cd(foreign)]),
        "③ cannot pick another seat's card"
    );
    answer(&mut e, "th_jingxie:choose_card", vec![cd(armor)]);
    assert_eq!(flag(&e, 0, "th_jingxie_upgraded"), 1, "① upgraded armor");
    assert_eq!(flag(&e, 0, &format!("th_jingxie_upgraded:{armor}")), 1);
    assert!(
        is_rejected(activate(&mut e, 0, "th_jingxie")),
        "④ already upgraded card cannot be upgraded again"
    );

    // 精械②：濒死重铸防具回复至 1 点体力
    let mut e = fresh(&p, 2842, "th_majun");
    let armor = card_by(&e, |c| c.kind.equip_slot() == Some(EquipSlot::Armor));
    give(&mut e, 0, armor);
    e.debug_set_hp(0, 1);
    e.debug_set_turn(1, Phase::Play, 1);
    let sha = card_by(&e, |c| c.kind == CardKind::Sha);
    give(&mut e, 1, sha);
    play(&mut e, 1, sha, vec![0]);
    let c = open(&e, "respond_shan");
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c.id,
            selection: vec![ChoiceOption::Pass],
        },
    );
    let pick = open(&e, "th_jingxie:choose_card").spec.options[0];
    snapshot_roundtrip(&e, &p, vec![pick]);
    answer(&mut e, "th_jingxie:choose_card", vec![pick]);
    drain(&mut e);
    assert!(
        e.state_core().player(0).alive && hp(&e, 0) == 1,
        "① dying recast saves to 1"
    );
    assert_eq!(flag(&e, 0, "th_jingxie_saved_to_1hp"), 1);

    // ---- th_qiaosi 巧思（`extension.js:7030`）：赢取 n 张，再弃置或交出 n 张 ----
    let mut e = fresh(&p, 2843, "th_majun");
    let h0 = hand(&e, 0);
    activate(&mut e, 0, "th_qiaosi");
    answer(&mut e, "th_qiaosi:choose_count", Vec::new());
    assert!(
        !once_used(&e, 0, "th_qiaosi") && hand(&e, 0) == h0,
        "② cancel before winning"
    );
    activate(&mut e, 0, "th_qiaosi");
    assert!(rejected(&mut e, vec![tag(4)]), "③ count 1..=3");
    answer(&mut e, "th_qiaosi:choose_count", vec![tag(2)]);
    assert_eq!(hand(&e, 0), h0 + 2, "① won 2 cards");
    assert!(rejected(&mut e, vec![tag(2)]), "③ mode 0..=1");
    snapshot_roundtrip(&e, &p, vec![tag(0)]);
    answer(&mut e, "th_qiaosi:choose_mode", vec![tag(1)]);
    assert!(rejected(&mut e, vec![pl(0)]), "③ give to another player");
    let h1 = hand(&e, 1);
    answer(&mut e, "th_qiaosi:choose_target", vec![pl(1)]);
    assert_eq!((hand(&e, 0), hand(&e, 1)), (h0, h1 + 2), "① gave 2");
    assert!(
        is_rejected(activate(&mut e, 0, "th_qiaosi")),
        "④ once per turn"
    );

    // ---- th_tiansuan 天算（`extension.js:7806-8216`）：命运签效果持续到下回合开始 ----
    let mut e = fresh(&p, 2844, "th_zhouqun");
    activate(&mut e, 0, "th_tiansuan");
    answer(&mut e, "th_tiansuan:choose_lot", Vec::new());
    assert!(
        !once_used(&e, 0, "th_tiansuan") && flag(&e, 0, "th_tiansuan2_active") == 0,
        "② cancel lot selection"
    );
    activate(&mut e, 0, "th_tiansuan");
    assert!(rejected(&mut e, vec![tag(5)]), "③ five lots 0..=4");
    answer(&mut e, "th_tiansuan:choose_lot", vec![tag(3)]);
    snapshot_roundtrip(&e, &p, vec![pl(1)]);
    answer(&mut e, "th_tiansuan:choose_target", vec![pl(1)]);
    assert!(has_skill(&e, 0, "th_tiansuan2") && flag(&e, 1, "th_tiansuan2_3_extra_damage") == 1);
    let (hp1, sha) = (hp(&e, 1), card_by(&e, |c| c.kind == CardKind::Sha));
    give(&mut e, 0, sha);
    play(&mut e, 0, sha, vec![1]);
    drain(&mut e);
    assert_eq!(hp(&e, 1), hp1 - 2, "⑤ 下签: damage +1");
    assert!(
        is_rejected(activate(&mut e, 0, "th_tiansuan")),
        "④ lot already active"
    );
    next_turn(&mut e, 1);
    assert_eq!(
        flag(&e, 0, "th_tiansuan2_active") + flag(&e, 1, "th_tiansuan2_3_extra_damage"),
        0
    );
    assert!(
        !has_skill(&e, 0, "th_tiansuan2"),
        "① th_tiansuan2 clears lots at turn begin"
    );
    // 上上签（lot 0）：获得目标区域一张牌 + 防止受到的伤害
    let (h0, h1) = (hand(&e, 0), hand(&e, 1));
    activate(&mut e, 0, "th_tiansuan");
    answer(&mut e, "th_tiansuan:choose_lot", vec![tag(0)]);
    answer(&mut e, "th_tiansuan:choose_target", vec![pl(1)]);
    let gain_opt = open(&e, "th_tiansuan:gain_card").spec.options[0];
    answer(&mut e, "th_tiansuan:gain_card", vec![gain_opt]);
    assert_eq!(
        (hand(&e, 0), hand(&e, 1)),
        (h0 + 1, h1 - 1),
        "① 上上签: gain a card from target"
    );
    let (hp1, sha) = (hp(&e, 1), card_by(&e, |c| c.kind == CardKind::Sha));
    give(&mut e, 0, sha);
    play(&mut e, 0, sha, vec![1]);
    drain(&mut e);
    assert_eq!(hp(&e, 1), hp1, "⑤ 上上签 prevents damage");

    // 上签（lot 1）：获得目标区域一张牌 + 伤害封顶 1 + 受伤后摸 1（th_tiansuan2_1 + th_tiansuan2_damage）
    let mut e = fresh(&p, 2849, "th_zhouqun");
    let (h0, h1_init) = (hand(&e, 0), hand(&e, 1));
    activate(&mut e, 0, "th_tiansuan");
    answer(&mut e, "th_tiansuan:choose_lot", vec![tag(1)]);
    answer(&mut e, "th_tiansuan:choose_target", vec![pl(1)]);
    let gain_opt = open(&e, "th_tiansuan:gain_card").spec.options[0];
    answer(&mut e, "th_tiansuan:gain_card", vec![gain_opt]);
    assert_eq!((hand(&e, 0), hand(&e, 1)), (h0 + 1, h1_init - 1));
    assert!(has_skill(&e, 1, "th_tiansuan2_1") && has_skill(&e, 1, "th_tiansuan2_damage"));
    let (jiu, sha) = (
        card_by(&e, |c| c.kind == CardKind::Jiu),
        card_by(&e, |c| c.kind == CardKind::Sha),
    );
    give(&mut e, 0, jiu);
    give(&mut e, 0, sha);
    let (hp1, h1) = (hp(&e, 1), hand(&e, 1));
    play(&mut e, 0, jiu, Vec::new());
    play(&mut e, 0, sha, vec![1]);
    drain(&mut e);
    assert_eq!(
        (hp(&e, 1), hand(&e, 1)),
        (hp1 - 1, h1 + 1),
        "⑤ 酒杀 capped to 1, then draw 1"
    );

    // 中签（lot 2）：th_tiansuan2_fire 将非火伤害改为火属性（配合太平要术防止火伤）
    next_turn(&mut e, 1);
    activate(&mut e, 0, "th_tiansuan");
    answer(&mut e, "th_tiansuan:choose_lot", vec![tag(2)]);
    answer(&mut e, "th_tiansuan:choose_target", vec![pl(1)]);
    assert!(
        has_skill(&e, 1, "th_tiansuan2_2")
            && has_skill(&e, 1, "th_tiansuan2_fire")
            && has_skill(&e, 1, "th_tiansuan2_ai")
    );
    let tp = card_named(&e, "th_taipingyaoshu");
    equip(&mut e, 1, tp);
    play_phase(&mut e, 0);
    let sha = card_by(&e, |c| c.kind == CardKind::Sha);
    give(&mut e, 0, sha);
    let hp1 = hp(&e, 1);
    play(&mut e, 0, sha, vec![1]);
    drain(&mut e);
    assert_eq!(
        hp(&e, 1),
        hp1,
        "⑤ 中签 converts normal Sha to Fire -> prevented by 太平要术"
    );

    // 下下签（lot 4）：不能使用【桃】/【酒】。对照：无签角色可正常使用。
    next_turn(&mut e, 2);
    activate(&mut e, 0, "th_tiansuan");
    answer(&mut e, "th_tiansuan:choose_lot", vec![tag(4)]);
    answer(&mut e, "th_tiansuan:choose_target", vec![pl(1)]);
    for (seat, banned) in [(2u8, false), (1, true)] {
        e.debug_set_hp(seat, 2);
        let tao = card_by(&e, |c| c.kind == CardKind::Tao);
        give(&mut e, seat, tao);
        e.debug_set_turn(seat, Phase::Play, 3);
        let out = submit(
            &mut e,
            seat,
            GameAction::PlayCard {
                card: tao,
                targets: Vec::new(),
            },
        );
        assert_eq!(is_rejected(out), banned, "⑤ 下下签 tao ban on seat {seat}");
        drain(&mut e);
    }

    // ---- th_gushe 鼓舌（`extension.js:8219`）+ th_gushe2 / rejici 激词 ----
    let mut e = fresh(&p, 2845, "th_wanglang");
    activate(&mut e, 0, "th_gushe");
    assert!(rejected(&mut e, vec![pl(0)]), "③ cannot pindian self");
    answer(&mut e, "th_gushe:choose_targets", Vec::new());
    assert!(!once_used(&e, 0, "th_gushe"), "② cancel refunds");
    let king = card_by(&e, |c| c.rank == 13);
    put_first(&mut e, 0, king);
    let low1 = card_by(&e, |c| c.rank == 2);
    let low2 = card_by(&e, |c| c.rank == 3);
    put_first(&mut e, 1, low1);
    put_first(&mut e, 2, low2);
    activate(&mut e, 0, "th_gushe");
    snapshot_roundtrip(&e, &p, vec![pl(1), pl(2)]);
    let (h0, h1) = (hand(&e, 0), hand(&e, 1));
    answer(&mut e, "th_gushe:choose_targets", vec![pl(1), pl(2)]);
    answer(&mut e, "th_gushe:pindian_card", vec![cd(king)]);
    answer(&mut e, "th_gushe:target_pindian_card", vec![cd(low1)]);
    answer(&mut e, "th_gushe:target_pindian_card", vec![cd(low2)]);
    // 座位 1 选择弃置 1 张牌；座位 2 取消（令王朗摸 1 张牌）。
    let t1_disc = open(&e, "th_gushe:target_discard").spec.options[0];
    answer(&mut e, "th_gushe:target_discard", vec![t1_disc]);
    answer(&mut e, "th_gushe:target_discard", Vec::new());
    assert_eq!(flag(&e, 0, "th_gushe2_marks"), 2, "① won both pindian");
    assert_eq!(hand(&e, 1), h1 - 2, "seat 1 spent pindian + discarded 1");
    assert_eq!(hand(&e, 0), h0, "seat 0 spent 1 pindian card and drew 1");
    assert!(has_skill(&e, 0, "th_gushe2"));

    // 王朗拼点没赢：获得 1 枚「饶舌」标记，选择取消弃牌则摸 1 张牌；再通过激词①加成反赢并获得最大拼点牌。
    let low0 = card_by(&e, |c| c.rank == 1);
    let high1 = card_by(&e, |c| c.rank == 12);
    put_first(&mut e, 0, low0);
    put_first(&mut e, 1, high1);
    activate(&mut e, 0, "th_gushe");
    answer(&mut e, "th_gushe:choose_targets", vec![pl(1)]);
    answer(&mut e, "th_gushe:pindian_card", vec![cd(low0)]);
    answer(&mut e, "th_gushe:target_pindian_card", vec![cd(high1)]);
    answer(&mut e, "th_gushe:owner_discard", Vec::new());
    assert_eq!(flag(&e, 0, "th_gushe_raosuo"), 1, "① lost pindian -> +1 raosuo");

    // 激词①：设有 4 枚饶舌标记，拼点牌点数 3 <= 4，点数 +4 变为 7 > 5，反败为胜并获得拼点牌 c1。
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "th_gushe_raosuo".into()), 4);
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "th_gushe2_marks".into()), 0);
    let mid0 = card_by(&e, |c| c.rank == 3);
    let mid1 = card_by(&e, |c| c.rank == 5);
    put_first(&mut e, 0, mid0);
    put_first(&mut e, 1, mid1);
    activate(&mut e, 0, "th_gushe");
    answer(&mut e, "th_gushe:choose_targets", vec![pl(1)]);
    answer(&mut e, "th_gushe:pindian_card", vec![cd(mid0)]);
    answer(&mut e, "th_gushe:target_pindian_card", vec![cd(mid1)]);
    answer(&mut e, "th_gushe:target_discard", Vec::new());
    assert!(
        e.state_core().player(0).hand.contains(&mid0),
        "① rejici +4 boost wins and gains winning pindian card"
    );

    // 饶舌 + 本回合胜场 >= 7 时不能发动鼓舌（④）；第 7 枚饶舌立即死亡（①）。
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "th_gushe_raosuo".into()), 6);
    assert!(
        is_rejected(activate(&mut e, 0, "th_gushe")),
        "④ raosuo(6) + wins(1) >= 7 blocks th_gushe"
    );
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "th_gushe2_marks".into()), 0);
    let c_lose = card_by(&e, |c| c.rank == 7);
    let c_win = card_by(&e, |c| c.rank == 11);
    put_first(&mut e, 0, c_lose);
    put_first(&mut e, 1, c_win);
    activate(&mut e, 0, "th_gushe");
    answer(&mut e, "th_gushe:choose_targets", vec![pl(1)]);
    answer(&mut e, "th_gushe:pindian_card", vec![cd(c_lose)]);
    answer(&mut e, "th_gushe:target_pindian_card", vec![cd(c_win)]);
    drain(&mut e);
    assert!(
        !e.state_core().player(0).alive && flag(&e, 0, "th_gushe_died") == 1,
        "① 7th raosuo mark causes death"
    );

    // 激词②（`rejici2`）：被其他角色杀死时，凶手弃置 7-X 张牌并失去 1 点体力。
    let mut e = fresh(&p, 2850, "th_wanglang");
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "th_gushe_raosuo".into()), 5);
    e.debug_set_hp(0, 1);
    for s in 0..4 {
        clear_hand(&mut e, s);
    }
    for r in 1..=4 {
        let c = card_by(&e, |d| d.rank == r);
        give(&mut e, 1, c);
    }
    let sha = card_by(&e, |c| c.kind == CardKind::Sha);
    give(&mut e, 1, sha);
    let hp1 = hp(&e, 1);
    play_phase(&mut e, 1);
    play(&mut e, 1, sha, vec![0]);
    let opts: Vec<_> = open(&e, "rejici:killer_discard")
        .spec
        .options
        .into_iter()
        .take(2)
        .collect();
    assert_eq!(opts.len(), 2, "7 - 5 = 2 cards to discard");
    answer(&mut e, "rejici:killer_discard", opts);
    drain(&mut e);
    assert_eq!(
        (hand(&e, 1), hp(&e, 1)),
        (2, hp1 - 1),
        "① rejici death trigger: killer discards 2 and loses 1 HP"
    );

    // ---- th_geyuan 割圆 / th_jieshu 解术 / th_gusuan 孤算（`extension.js:8384-8633`）----
    let mut e = fresh(&p, 2846, "th_liuhui");
    assert_eq!(e.state_core().player(0).max_hp, 4);
    let _ = hand_turn_to(&mut e, 0, 1);
    drain(&mut e);
    assert_eq!(
        flag(&e, 0, "th_geyuan_ring_len"),
        13,
        "① th_geyuan_qyubee initializes 13-rank ring"
    );
    // ④ 环长为 13（!= 3）时，回合结束不触发孤算觉醒
    next_turn(&mut e, 2);
    assert_eq!(
        (flag(&e, 0, "th_gusuan_awakened"), e.state_core().player(0).max_hp),
        (0, 4),
        "④ th_gusuan must not awaken when ring_len != 3"
    );
    // 解术 + 割圆（觉醒前）：设 5 点环 {r-4..=r}，已记弧 {r-4..r}（首=r-4），使用点数 r 的【杀】：
    // 解术先触发摸 1 张牌；割圆随后补齐 5 点弧，获得牌堆中首尾点数的牌，并将环收缩为 {r-3..=r-1}（长度 3）。
    play_phase(&mut e, 0);
    let sha_r = card_by(&e, |c| c.kind == CardKind::Sha && c.rank == 8);
    let r = 8u32;
    let ring5 = (r - 4..=r).fold(0u32, |m, x| m | (1 << x));
    let arc4 = (r - 4..r).fold(0u32, |m, x| m | (1 << x));
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "th_geyuan_ring_mask".into()), ring5);
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "th_geyuan_ring_len".into()), 5);
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "th_geyuan_homura_mask".into()), arc4);
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "th_geyuan_homura_first".into()), r - 4);
    give(&mut e, 0, sha_r);
    let h0_before = hand(&e, 0);
    play(&mut e, 0, sha_r, vec![1]);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "th_jieshu_drawn"), 1, "① th_jieshu drew 1");
    assert_ne!(flag(&e, 0, "th_jieshu_rank_mask") & (1 << r), 0);
    assert_eq!(flag(&e, 0, "th_geyuan_pi"), 1, "① th_geyuan completed arc");
    assert_eq!(
        flag(&e, 0, "th_geyuan_ring_len"),
        3,
        "① endpoints removed -> ring_len == 3"
    );
    assert!(hand(&e, 0) >= h0_before, "gained endpoint cards + jieshu draw");

    // 孤算：回合结束时环长为 3 -> 觉醒减 1 点体力上限
    next_turn(&mut e, 3);
    assert_eq!(
        (flag(&e, 0, "th_gusuan_awakened"), e.state_core().player(0).max_hp),
        (1, 3),
        "① th_gusuan awakens at turn end when ring_len == 3"
    );
    // 觉醒后割圆（`th_geyuan_magica`）：补齐 3 点弧 {5,6,7} 时选择 1~3 名角色执行摸3/弃4/换底牌
    play_phase(&mut e, 0);
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "th_geyuan_homura_mask".into()), (1 << 5) | (1 << 6));
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "th_geyuan_homura_first".into()), 5);
    let sha7 = card_by(&e, |c| c.kind == CardKind::Sha && c.rank == 7);
    give(&mut e, 0, sha7);
    let h0 = hand(&e, 0);
    play(&mut e, 0, sha7, vec![1]);
    snapshot_roundtrip(&e, &p, vec![pl(0), pl(1), pl(2)]);
    answer(&mut e, "th_geyuan:magica_targets", vec![pl(0), pl(1), pl(2)]);
    drain(&mut e);
    assert!(hand(&e, 0) >= h0 + 2, "① magica target[0] drew 3");
    assert_eq!(hand(&e, 2), 5, "① magica target[2] swapped with 5 bottom cards");
    next_turn(&mut e, 4);
    assert_eq!(
        e.state_core().player(0).max_hp,
        3,
        "④ awakening skill fires once"
    );

    // ---- th_yeyan 业炎（限定技，`extension.js:8636`）/ qinyin 琴音 ----
    let mut e = fresh(&p, 2847, "th_shen_zhouyu");
    // ④ 琴音负向门禁：弃牌阶段弃置不足 2 张牌时不触发琴音
    clear_hand(&mut e, 0);
    advance_to(&mut e, Phase::End);
    assert_eq!(
        flag(&e, 0, "qinyin_triggered"),
        0,
        "④ qinyin must not trigger when < 2 cards discarded in Discard phase"
    );
    // ① 琴音正向触发：弃牌阶段弃置 >= 2 张牌，阶段结束时触发选择全体回复或失去 1 点体力
    next_turn(&mut e, 2);
    e.debug_set_hp(0, 2);
    e.debug_set_hp(1, 2);
    for r in 1..=6 {
        let c = card_by(&e, |d| d.rank == r);
        give(&mut e, 0, c);
    }
    e.debug_set_turn(0, Phase::Play, 2);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let disc_c = open(&e, "discard_to_limit");
    let sel: Vec<_> = disc_c.spec.options.iter().take(disc_c.spec.min).copied().collect();
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: disc_c.id,
            selection: sel,
        },
    );
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert!(rejected(&mut e, vec![tag(2)]), "③ qinyin mode 0..=1");
    snapshot_roundtrip(&e, &p, vec![tag(1)]);
    answer(&mut e, "qinyin:choose_mode", vec![tag(0)]);
    drain(&mut e);
    assert_eq!(
        (flag(&e, 0, "qinyin_triggered"), hp(&e, 0), hp(&e, 1)),
        (1, 3, 3),
        "① qinyin mode 0 recovers 1 HP for wounded players"
    );

    // 业炎：取消退还 -> 小业炎（1~3 名目标各 1 点火伤）-> 限定技不可再发动
    play_phase(&mut e, 0);
    activate(&mut e, 0, "th_yeyan");
    answer(&mut e, "th_yeyan:choose_mode", Vec::new());
    assert!(
        !once_used(&e, 0, "th_yeyan") && flag(&e, 0, "th_yeyan_used") == 0,
        "② cancel yeyan"
    );
    activate(&mut e, 0, "th_yeyan");
    answer(&mut e, "th_yeyan:choose_mode", vec![tag(0)]);
    snapshot_roundtrip(&e, &p, vec![pl(1), pl(2), pl(3)]);
    let hps: Vec<i32> = (1..4).map(|s| hp(&e, s)).collect();
    answer(&mut e, "th_yeyan:choose_targets", vec![pl(1), pl(2), pl(3)]);
    drain(&mut e);
    assert_eq!(
        (1..4).map(|s| hp(&e, s)).collect::<Vec<_>>(),
        hps.iter().map(|h| h - 1).collect::<Vec<_>>(),
        "① small yeyan deals 1 Fire damage to up to 3 targets"
    );
    next_turn(&mut e, 3);
    assert!(
        is_rejected(activate(&mut e, 0, "th_yeyan")),
        "④ limited skill spent"
    );

    // 大业炎：④ 手牌不足四花色时不可选 mode 1；凑齐四花色后弃 4 张不同花色手牌并失去 3 点体力，造成 3 点火伤。
    let mut e = fresh(&p, 2848, "th_shen_zhouyu");
    clear_hand(&mut e, 0);
    let only_spade = card_by(&e, |c| c.suit == Suit::Spade);
    give(&mut e, 0, only_spade);
    activate(&mut e, 0, "th_yeyan");
    assert!(
        rejected(&mut e, vec![tag(1)]),
        "④ mode 1 unavailable without 4 distinct suits in hand"
    );
    answer(&mut e, "th_yeyan:choose_mode", Vec::new());
    let mut four_suits = Vec::new();
    for suit in [Suit::Spade, Suit::Heart, Suit::Club, Suit::Diamond] {
        let c = card_by(&e, |d| d.suit == suit);
        give(&mut e, 0, c);
        four_suits.push(cd(c));
    }
    activate(&mut e, 0, "th_yeyan");
    answer(&mut e, "th_yeyan:choose_mode", vec![tag(1)]);
    let hp1 = hp(&e, 1);
    answer(&mut e, "th_yeyan:choose_targets", vec![pl(1)]);
    answer(&mut e, "th_yeyan:discard_cards", four_suits);
    drain(&mut e);
    assert_eq!(
        (hp(&e, 0), hp(&e, 1)),
        (1, hp1 - 3),
        "① great yeyan loses 3 HP and deals 3 Fire damage"
    );
}

/// 交给 `seat` 一个新的出牌阶段（清本阶段已用【杀】标记）。
fn play_phase(e: &mut Engine, seat: u8) {
    e.debug_set_turn(seat, Phase::Play, 2);
    e.state_core_mut().turn.sha_used_play_phase = false;
}

/// 把 `card` 放到 `seat` 手牌首位（拼点取首张手牌）。
fn put_first(e: &mut Engine, seat: u8, card: u32) {
    give(e, seat, card);
    let h = &mut e.state_core_mut().player_mut(seat).hand;
    h.retain(|&c| c != card);
    h.insert(0, card);
}

fn equip(e: &mut Engine, seat: u8, card: u32) {
    give(e, seat, card);
    play_phase(e, seat);
    play(e, seat, card, Vec::new());
    drain(e);
    assert!(
        e.state_core().player(seat).equips.contains(&card),
        "equipped"
    );
}

/// 5. ⑤ 装备与多技能交互：太平要术三效果、忠壮攻击范围、天书授予太平要术。
#[test]
fn l28_thunder_eight_equip_and_interactions() {
    let p = pack();
    let mut e = fresh(&p, 2851, "th_nanhualaoxian");
    for (s, g) in [(1, "th_majun"), (2, "th_lukai"), (3, "th_puyuan")] {
        e.assign_general_named(s, g).unwrap();
    }
    let tp = card_named(&e, "th_taipingyaoshu");
    let base_limit = e.hand_limit_for_debug(0);
    equip(&mut e, 0, tp);
    assert!(has_skill(&e, 0, "th_taipingyaoshu"));
    assert_eq!(
        e.hand_limit_for_debug(0),
        base_limit + 3,
        "qun/wei/wu/shu alive → +3"
    );
    e.state_core_mut().player_mut(3).alive = false;
    assert_eq!(e.hand_limit_for_debug(0), base_limit + 2, "shu gone → +2");
    e.state_core_mut().player_mut(3).alive = true;
    // 属性伤害（火/雷）被防止，普通伤害照常。
    for kind in [CardKind::FireSha, CardKind::LeiSha, CardKind::Sha] {
        let c = card_by(&e, |d| d.kind == kind);
        give(&mut e, 1, c);
        play_phase(&mut e, 1);
        let hp0 = hp(&e, 0);
        play(&mut e, 1, c, vec![0]);
        drain(&mut e);
        let expect = if kind == CardKind::Sha { hp0 - 1 } else { hp0 };
        assert_eq!(hp(&e, 0), expect, "⑤ {kind:?} vs 太平要术");
    }
    // 失去其他装备不触发太平要术失去效果（回归：此前任何装备离场都会触发）。
    let weapons: Vec<u32> = (0..e.catalog().cards.len() as u32)
        .filter(|&c| e.catalog().card_kind(c).equip_slot() == Some(EquipSlot::Weapon))
        .take(2)
        .collect();
    equip(&mut e, 0, weapons[0]);
    let (h0, hp0) = (hand(&e, 0), hp(&e, 0));
    equip(&mut e, 0, weapons[1]);
    assert_eq!(
        (hand(&e, 0), hp(&e, 0)),
        (h0, hp0),
        "weapon swap leaves 太平要术 idle"
    );
    // 替换防具 → 失去太平要术：摸 2，体力 > 1 时失去 1 点体力，技能移除。
    let armor = card_by(&e, |d| {
        d.kind.equip_slot() == Some(EquipSlot::Armor) && d.name != "th_taipingyaoshu"
    });
    let (h0, hp0) = (hand(&e, 0), hp(&e, 0));
    equip(&mut e, 0, armor);
    assert_eq!(
        (hand(&e, 0), hp(&e, 0)),
        (h0 + 2, hp0 - 1),
        "⑤ lose 太平要术: draw 2, lose 1"
    );
    assert!(!has_skill(&e, 0, "th_taipingyaoshu") && e.state_core().discard.contains(&tp));
    // 体力为 1 时失去太平要术只摸牌、不失去体力。
    let mut e = fresh(&p, 2852, "th_nanhualaoxian");
    let tp = card_named(&e, "th_taipingyaoshu");
    equip(&mut e, 0, tp);
    e.debug_set_hp(0, 1);
    let armor = card_by(&e, |d| {
        d.kind.equip_slot() == Some(EquipSlot::Armor) && d.name != "th_taipingyaoshu"
    });
    let h0 = hand(&e, 0);
    equip(&mut e, 0, armor);
    assert_eq!((hand(&e, 0), hp(&e, 0)), (h0 + 2, 1), "hp 1: draw only");

    // th_zhongzhuang 忠壮（`extension.js:4388`）：攻击范围 < 3 伤害 +1；> 3 只能造成 1 点。
    let mut e = fresh(&p, 2853, "th_lukai");
    assert!(e.state_core().attack_range(e.catalog(), 0) < 3);
    let sha = card_by(&e, |c| c.kind == CardKind::Sha);
    give(&mut e, 0, sha);
    let hp1 = hp(&e, 1);
    play(&mut e, 0, sha, vec![1]);
    drain(&mut e);
    assert_eq!(hp(&e, 1), hp1 - 2, "⑤ range 1 → +1");
    let qilin = card_by(&e, |c| c.kind == CardKind::Qilin);
    equip(&mut e, 0, qilin);
    assert!(e.state_core().attack_range(e.catalog(), 0) > 3);
    e.debug_set_hp(1, 4);
    let (jiu, sha) = (
        card_by(&e, |c| c.kind == CardKind::Jiu),
        card_by(&e, |c| c.kind == CardKind::Sha),
    );
    give(&mut e, 0, jiu);
    give(&mut e, 0, sha);
    play_phase(&mut e, 0);
    play(&mut e, 0, jiu, Vec::new());
    play(&mut e, 0, sha, vec![1]);
    drain(&mut e);
    assert_eq!(hp(&e, 1), 3, "⑤ range 5: 酒杀 capped at 1");
}
