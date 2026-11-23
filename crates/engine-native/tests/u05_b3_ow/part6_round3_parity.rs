//! U05-3 (`shaai-ow`) Round 3 审查 2 项等价缺陷真实事件闭环回归（反例先行，0 `debug_execute_effect` / `debug_resolve`）：
//! 1. `shanxian`：
//!    (a) 退出后必须清零 `ow_shanxian_target`，恢复对原目标的距离/攻击范围限制，并允许下轮再次发动；
//!    (b) 劫持期间自身若已背面朝上（`face_down`），`phaseAfter` 不得再翻面将其翻回正面（`player.turnOver(true)`）；
//!    (c) 轮末跨 `Seat 0`（`3 -> 0`）劫持背面朝上的 `Seat 0`（或拒绝劫持、`Seat 0` 途中阵亡）仍必须正确推进 `turn.round`。
//! 2. `huoyu`（无色虚拟【炽羽袭】）与 `dianji`（保留所选牌实体与花色的【惊雷闪】）：
//!    - 不得把无色/红色牌当作 `cards[0]`（黑桃 7）传入 `eval_card_target_canceled`（及 AOE 结算载体）；
//!    - `weimu`（`BlackTrick` / `BOSS_WEIMU`）、`molicha`（`u05_mod:molicha` / `gj_food_molicha`）、`dddanzhi_black_ward`
//!      只阻断**黑色**牌/黑色锦囊，不得阻断无色 `huoyu` 与红色 `dianji`，且黑色 `dianji` 须受其阻断。

use content_native_loader::{
    prepare_locked_content_with_bridge, start_locked_engine, LockedStartMode,
};
use engine_native::catalog::{CardKind, Suit};
use engine_native::{ChoiceOption, CommandOutcome, GameAction, Phase, RejectReason};

use super::common::{
    activate, advance_one_turn_to_play, advance_to_play, answer, answer_cancel,
    assert_card_conservation, assert_open_prompt, clear_all_hands, drain_cancel, flag, freeze_pkg,
    place_kind_suit, play, roundtrip_restore, start_formal_ow, submit,
};

fn advance_to_shanxian_confirm(e: &mut engine_native::Engine, liekong: u8) {
    for _ in 0..32 {
        if let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt == "ow_shanxian_confirm" && c.observer == liekong {
                return;
            }
            drain_cancel(e);
            continue;
        }
        let cur = e.turn().current;
        let _ = submit(e, cur, GameAction::NextPhase);
    }
    assert_open_prompt(e, "ow_shanxian_confirm", liekong);
}

fn finish_current_turn_to_next_play(e: &mut engine_native::Engine) {
    let start = e.turn().current;
    for _ in 0..32 {
        if e.turn().current != start && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            return;
        }
        if e.open_choice().is_some() {
            drain_cancel(e);
            continue;
        }
        let cur = e.turn().current;
        let _ = submit(e, cur, GameAction::NextPhase);
    }
    assert_ne!(e.turn().current, start, "turn should advance past {start}");
}

/// 缺陷 1a：`shanxian` 退出后清空 `ow_shanxian_target`，恢复对原 `victim_b` 距离/范围限制，并支持第 2 轮再次触发。
#[test]
fn r3_defect1a_shanxian_clears_target_allows_second_round_and_restores_range() {
    let (verified, mut e) = start_formal_ow(2026_1301, 4);
    e.assign_general_named(0, "ow_liekong").unwrap();
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e);
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 2), 2);
    assert_eq!(e.state_core().attack_range(e.catalog(), 0), 1);

    // 推进至 1 号结束交接给 2 号，0 号（猎空）触发第 1 次 `shanxian` 劫持 2 号（dist=2 > range=1）
    advance_to_play(&mut e, 1);
    clear_all_hands(&mut e);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    advance_to_shanxian_confirm(&mut e, 0);
    answer(&mut e, vec![ChoiceOption::Confirm]);
    advance_to_play(&mut e, 0);
    assert_eq!(flag(&e, 0, "ow_shanxian_target"), 3, "劫持 2 号期间 ow_shanxian_target == 3");

    // 劫持回合内无视距离对 2 号出【杀】须成功
    let sha_hijack = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Heart);
    e.debug_set_hp(2, 4);
    let out_hit = play(&mut e, 0, sha_hijack, vec![2]);
    assert!(
        matches!(out_hit, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "闪现劫持期间对 victim_b=2 应无视攻击范围出杀: {out_hit:?}"
    );
    drain_cancel(&mut e);
    assert_eq!(e.debug_hp(2), 3);

    // 结束劫持回合：0 号翻面、交回 2 号，且 `ow_shanxian_target` 必须清零
    finish_current_turn_to_next_play(&mut e);
    assert_eq!(e.turn().current, 2, "劫持结束后应交接至原受害者 2 号");
    assert!(e.debug_face_down(0), "0 号猎空已翻面");
    assert_eq!(
        flag(&e, 0, "ow_shanxian_target"),
        0,
        "shanxian 退出后必须清零 ow_shanxian_target"
    );

    // 到 0 号正常翻回并轮转至下一个 0 号出牌阶段：不得仍对 2 号豁免距离/范围！
    advance_to_play(&mut e, 0);
    assert!(!e.debug_face_down(0));
    clear_all_hands(&mut e);
    let sha_after = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    assert!(
        matches!(
            play(&mut e, 0, sha_after, vec![2]),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ),
        "shanxian 结束后对原 victim_b=2 的距离/范围豁免必须收回"
    );
    e.debug_clear_hand(0);

    // 推进至 1 号（或 2 号）回合开始，猎空第 2 轮必须能再次触发 `ow_shanxian_confirm`
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    advance_to_shanxian_confirm(&mut e, 0);
    e = roundtrip_restore(&e, &verified);
    assert_open_prompt(&e, "ow_shanxian_confirm", 0);
    answer(&mut e, vec![ChoiceOption::Confirm]);
    advance_to_play(&mut e, 0);
    assert_eq!(flag(&e, 0, "ow_shanxian_target"), 2, "第 2 次闪现成功劫持 1 号");
    assert_card_conservation(&e, "r3 defect1a end");
}

/// 缺陷 1b+1c：
/// - 劫持途中 `ow_liekong` 已翻至背面（`face_down`）时，`phaseAfter` 不得误将其翻回正面；
/// - 跨 `Seat 0`（`3 -> 0`）面对背面朝上的 `Seat 0`（接受劫持、拒绝劫持、途中击杀 `Seat 0`）仍须正确推进 `turn.round`。
#[test]
fn r3_defect1bc_shanxian_already_facedown_guard_and_seat0_wrap_round_bump() {
    // Part (1): 劫持途中自身变为 face_down -> phaseAfter 保持 face_down 且清 ow_shanxian_target
    let (_v0, mut e0) = start_formal_ow(2026_1302, 4);
    e0.assign_general_named(0, "ow_liekong").unwrap();
    advance_to_play(&mut e0, 0);
    clear_all_hands(&mut e0);
    let _ = submit(&mut e0, 0, GameAction::NextPhase);
    advance_to_shanxian_confirm(&mut e0, 0);
    answer(&mut e0, vec![ChoiceOption::Confirm]);
    advance_to_play(&mut e0, 0);
    // 模拟劫持回合中被翻至背面（upstream player.turnOver(true) 幂等护栏）
    e0.state_core_mut().players[0].face_down = true;
    finish_current_turn_to_next_play(&mut e0);
    assert!(
        e0.debug_face_down(0),
        "劫持途中已背面朝上时，shanxian phaseAfter 的 turnOver(true) 不得反翻为正面"
    );
    assert_eq!(flag(&e0, 0, "ow_shanxian_target"), 0);
    assert_card_conservation(&e0, "r3 defect1b part1");

    // Part (2): `ow_liekong` 在 2 号，`ow_mei` 在 3 号；3 号用 `baoxue` 令 `Seat 0` 翻面；
    // `3 -> 0` 跨轮交接时 `ow_liekong` 劫持背面朝上的 `Seat 0` 并正常结束后，轮次必须推进至 `round = 2`！
    let (v1, mut e1) = start_formal_ow(2026_1303, 4);
    e1.assign_general_named(2, "ow_liekong").unwrap();
    e1.assign_general_named(3, "ow_mei").unwrap();
    advance_to_play(&mut e1, 3);
    clear_all_hands(&mut e1);
    let b3 = place_kind_suit(&mut e1, 3, CardKind::Sha, Suit::Spade);
    let _ = b3;
    activate(&mut e1, 3, "baoxue");
    assert_open_prompt(&e1, "ow_baoxue_targets", 3);
    answer(&mut e1, vec![ChoiceOption::Player(0)]);
    drain_cancel(&mut e1);
    assert!(e1.debug_face_down(0), "Seat 0 已被翻面");
    assert_eq!(e1.state_core().turn.round, 1);

    let _ = submit(&mut e1, 3, GameAction::NextPhase);
    advance_to_shanxian_confirm(&mut e1, 2);
    e1 = roundtrip_restore(&e1, &v1);
    assert_open_prompt(&e1, "ow_shanxian_confirm", 2);
    answer(&mut e1, vec![ChoiceOption::Confirm]);
    advance_to_play(&mut e1, 2);
    finish_current_turn_to_next_play(&mut e1);
    assert_eq!(e1.turn().current, 1, "Seat 0 翻回跳过后轮到 1 号");
    assert!(e1.debug_face_down(2), "2 号猎空劫持后翻面");
    assert!(!e1.debug_face_down(0), "Seat 0 已翻回正面");
    assert_eq!(
        e1.state_core().turn.round,
        2,
        "跨 Seat 0 劫持背面朝上的 Seat 0 后 turn.round 须推进至 2"
    );
    assert_card_conservation(&e1, "r3 defect1c accept");

    // Part (3): 同样是背面朝上的 `Seat 0` 在 `3 -> 0` 被 `ow_liekong` 触发但**拒绝**劫持，轮次仍须变为 2！
    let (_v2, mut e2) = start_formal_ow(2026_1304, 4);
    e2.assign_general_named(2, "ow_liekong").unwrap();
    e2.assign_general_named(3, "ow_mei").unwrap();
    advance_to_play(&mut e2, 3);
    clear_all_hands(&mut e2);
    let _b = place_kind_suit(&mut e2, 3, CardKind::Sha, Suit::Spade);
    activate(&mut e2, 3, "baoxue");
    assert_open_prompt(&e2, "ow_baoxue_targets", 3);
    answer(&mut e2, vec![ChoiceOption::Player(0)]);
    drain_cancel(&mut e2);
    assert!(e2.debug_face_down(0));
    let _ = submit(&mut e2, 3, GameAction::NextPhase);
    advance_to_shanxian_confirm(&mut e2, 2);
    answer_cancel(&mut e2);
    advance_to_play(&mut e2, 1);
    assert_eq!(e2.turn().current, 1);
    assert_eq!(
        e2.state_core().turn.round,
        2,
        "拒绝劫持背面朝上 Seat 0 后 turn.round 同样须推进至 2"
    );

    // Part (4): 劫持背面朝上的 `Seat 0` 途中击杀 `Seat 0`，交接给 `next_alive(0) = 1` 且 `round == 2`！
    let (_v3, mut e3) = start_formal_ow(2026_1305, 4);
    e3.assign_general_named(2, "ow_liekong").unwrap();
    e3.assign_general_named(3, "ow_mei").unwrap();
    advance_to_play(&mut e3, 3);
    clear_all_hands(&mut e3);
    let _b3 = place_kind_suit(&mut e3, 3, CardKind::Sha, Suit::Spade);
    activate(&mut e3, 3, "baoxue");
    assert_open_prompt(&e3, "ow_baoxue_targets", 3);
    answer(&mut e3, vec![ChoiceOption::Player(0)]);
    drain_cancel(&mut e3);
    let _ = submit(&mut e3, 3, GameAction::NextPhase);
    advance_to_shanxian_confirm(&mut e3, 2);
    answer(&mut e3, vec![ChoiceOption::Confirm]);
    advance_to_play(&mut e3, 2);
    e3.debug_clear_hand(0);
    e3.debug_set_hp(0, 1);
    let sha_kill = place_kind_suit(&mut e3, 2, CardKind::Sha, Suit::Spade);
    play(&mut e3, 2, sha_kill, vec![0]);
    drain_cancel(&mut e3);
    assert!(!e3.debug_alive(0));
    finish_current_turn_to_next_play(&mut e3);
    assert_eq!(e3.turn().current, 1);
    assert_eq!(
        e3.state_core().turn.round,
        2,
        "劫持背面朝上 Seat 0 途中击杀后交接至 1 号，turn.round 须推进至 2"
    );
    assert_card_conservation(&e3, "r3 defect1c kill");
}

/// 缺陷 2：无色 `huoyu`（炽羽袭）与保留实体花色的 `dianji`（惊雷闪）对抗黑色免疫（`weimu`/`molicha`/`dddanzhi_black_ward`）。
#[test]
fn r3_defect2_huoyu_colorless_and_dianji_preserves_suit_against_black_wards() {
    let verified = prepare_locked_content_with_bridge(vec![
        freeze_pkg("shaai-ow-upstream-noname"),
        freeze_pkg("shahai-gujian-upstream-noname"),
    ])
    .expect("prepare ow + gujian");
    let mut e = start_locked_engine(2026_1306, 4, LockedStartMode::Prototype, &verified)
        .expect("start engine");
    e.assign_general_named(0, "ow_falaozhiying").unwrap();
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e);

    // 在其余 3 座分别施加三类黑色免疫：`weimu`(BOSS_WEIMU)、`molicha`(`u05_mod:molicha` + `gj_food_molicha`)、`dddanzhi_black_ward`
    e.debug_grant(1, "帷幕");
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((2, "u05_mod:molicha".to_string()), 1);
    e.debug_grant(2, "gj_food_molicha");
    let stamp = e.state_core().turn.round.saturating_mul(100) + u32::from(e.turn().current) + 1;
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((3, "dddanzhi_black_ward".to_string()), 1);
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((3, "dddanzhi_immune".to_string()), 1);
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((3, "dddanzhi_stamp".to_string()), stamp);

    // 将一张黑桃牌置入牌堆顶（下标 0）：旧 `use_virtual_card` 会摸走该黑桃作载体，被 `weimu_blocks_aoe`/`dddanzhi` 误挡
    let black_top = e
        .state_core()
        .deck
        .iter()
        .copied()
        .find(|&c| e.catalog().cards[c as usize].suit == Suit::Spade)
        .expect("spade in deck");
    e.state_core_mut().deck.retain(|&c| c != black_top);
    e.state_core_mut().deck.insert(0, black_top);

    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);
    e.debug_set_hp(3, 4);
    let r1 = place_kind_suit(&mut e, 0, CardKind::Tao, Suit::Heart);
    let r2 = place_kind_suit(&mut e, 0, CardKind::Shan, Suit::Diamond);
    // 确定牌堆顶仍为黑桃
    assert_eq!(e.state_core().deck.first().copied(), Some(black_top));

    let act_hy = activate(&mut e, 0, "huoyu");
    assert!(
        matches!(act_hy, CommandOutcome::Pending { .. }),
        "全员仅持黑色免疫时，无色 huoyu 仍必须允许发动: {act_hy:?}"
    );
    assert_open_prompt(&e, "ow_huoyu_cards", 0);
    answer(&mut e, vec![ChoiceOption::Card(r1), ChoiceOption::Card(r2)]);
    drain_cancel(&mut e);
    assert_eq!(e.debug_hp(1), 2, "无色炽羽袭须命中 weimu 目标 1 号 (4->2)");
    assert_eq!(e.debug_hp(2), 2, "无色炽羽袭须命中 molicha 目标 2 号 (4->2)");
    assert_eq!(e.debug_hp(3), 2, "无色炽羽袭须命中 dddanzhi 目标 3 号 (4->2)");
    assert_eq!(
        e.state_core().deck.first().copied(),
        Some(black_top),
        "无色虚拟 huoyu 不应消耗牌堆顶载体"
    );
    assert_card_conservation(&e, "r3 defect2 huoyu end");

    // 换温斯顿测 `dianji`（按所选牌真实花色区分红/黑对抗黑色免疫）
    e.assign_general_named(0, "ow_wensidun").unwrap();
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e);
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);
    e.debug_set_hp(3, 4);
    let stamp2 = e.state_core().turn.round.saturating_mul(100) + u32::from(e.turn().current) + 1;
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((3, "dddanzhi_black_ward".to_string()), 1);
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((3, "dddanzhi_immune".to_string()), 1);
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((3, "dddanzhi_stamp".to_string()), stamp2);

    // 若 0 号仅持黑牌，且所有 dist<=2 目标（1、3）均为黑色免疫（2 距为 2 在 4 人局，也加黑免）：必须拒绝发动！
    let blk_only = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    assert!(
        matches!(activate(&mut e, 0, "dianji"), CommandOutcome::Rejected(_)),
        "仅持黑牌且全部 dist<=2 目标具备黑色免疫时，dianji 不得发动"
    );
    e.debug_clear_hand(0);
    let _ = blk_only;

    // 换成**红牌**（即使 `weimu`/`molicha`/`dddanzhi` 存在）：`dianji` 须发动并命中 1 号与 3 号（dist<=2），不消耗 deck[0]！
    let red_fuel = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Heart);
    let deck_top_before = e.state_core().deck.first().copied();
    let act_dj = activate(&mut e, 0, "dianji");
    assert!(
        matches!(act_dj, CommandOutcome::Pending { .. }),
        "持有红牌时 dianji 须忽略黑色免疫正常发动: {act_dj:?}"
    );
    assert_open_prompt(&e, "ow_dianji_card", 0);
    answer(&mut e, vec![ChoiceOption::Card(red_fuel)]);
    drain_cancel(&mut e);
    assert_eq!(e.debug_hp(3), 3, "红牌惊雷闪须命中 dddanzhi 目标 3 号");
    assert_eq!(e.debug_hp(1), 3, "红牌惊雷闪须命中 weimu 目标 1 号");
    assert_eq!(e.debug_hp(2), 3, "红牌惊雷闪须命中 molicha 目标 2 号 (4人局距=2<=2)");
    assert_eq!(
        e.state_core().deck.first().copied(),
        deck_top_before,
        "dianji 须直接使用选定手牌 cid，不得再摸牌堆载体"
    );

    // 下一回合：撤除 3 号的 dddanzhi，保持 1 号 weimu 与 2 号 molicha；用**黑牌**发动 dianji -> 只命中 3 号，跳过 1 和 2！
    advance_one_turn_to_play(&mut e, 0);
    clear_all_hands(&mut e);
    e.state_core_mut()
        .vm_aux
        .flags
        .remove(&(3, "dddanzhi_black_ward".to_string()));
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);
    e.debug_set_hp(3, 4);
    let blk_fuel = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Club);
    let act_dj_blk = activate(&mut e, 0, "dianji");
    assert!(matches!(act_dj_blk, CommandOutcome::Pending { .. }));
    assert_open_prompt(&e, "ow_dianji_card", 0);
    answer(&mut e, vec![ChoiceOption::Card(blk_fuel)]);
    drain_cancel(&mut e);
    assert_eq!(e.debug_hp(3), 3, "黑牌惊雷闪须命中无黑免的 3 号");
    assert_eq!(e.debug_hp(1), 4, "黑牌惊雷闪须跳过 weimu 目标 1 号");
    assert_eq!(e.debug_hp(2), 4, "黑牌惊雷闪须跳过 molicha 目标 2 号");
    assert_card_conservation(&e, "r3 defect2 dianji end");
}
