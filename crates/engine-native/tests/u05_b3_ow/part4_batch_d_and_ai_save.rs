//! U05-3 (`shaai-ow`) Batch D + 公开信息 AI (`BoundedPlanner`) + 中途 `save -> restore -> Answer` 真实闭环测试：
//! 禅雅塔 (`ow_chanyata`: `xie`/`xie2`/`xie4`, `luan`/`luan2`/`luan3`/`luan4`, `sheng`/`sheng2`)、
//! 法老之鹰 (`ow_falaozhiying`: `feidan`, `huoyu`, `feiying`)、
//! 卢西奥 (`ow_luxiao`: `yuedong`, `huhuan`, `kuoyin`/`kuoyin2`)、
//! 秩序之光 (`ow_zhixuzhiguang`: `guangshu`/`guangshu_{heart,diamond,club,spade}`)、
//! 天使 (`ow_tianshi`: `shouhu`, `ziyu`, `feiying`)、
//! 猎空 (`ow_liekong`: `shanxian`/`shanxian2`, `shanhui`)。

use engine_native::ai::BoundedPlanner;
use engine_native::catalog::{CardKind, Suit};
use engine_native::event::DamageNature;
use engine_native::state::Role;
use engine_native::{ChoiceOption, CommandOutcome, GameAction, Phase, RejectReason};

use super::common::{
    activate, advance_one_turn_to_play, advance_to_play, advance_to_turn_end_prompt, answer,
    answer_cancel, assert_card_conservation, assert_open_prompt, clear_all_hands, drain_cancel,
    flag, place_any_suit, place_kind_suit, play, roundtrip_restore, start_formal_ow, submit,
};

/// 禅雅塔 (`character/ow/skill.js:2387-2649`):
/// - `xie` 弃红桃手牌赋予目标 `xie2`（先每满 6 阶段结束回 1 血；禅雅塔死则移除）；
/// - `luan` 弃黑桃手牌赋予目标 `luan2`（受伤后 LoseHp(1) 直至目标脱离濒死或禅雅塔死亡）；
/// - `sheng` limited 翻面并指定已受伤角色各回 1 + 自身 `sheng2`（不可成为目标，直至己回合开始）。
#[test]
fn ow_batch_d_zenyatta_xie_luan_sheng() {
    let (_verified, mut e) = start_formal_ow(2026_1160, 4);
    e.assign_general_named(0, "ow_chanyata").unwrap();
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e);
    e.debug_set_hp(0, 2);
    e.debug_set_hp(1, 2);
    e.debug_set_hp(2, 4);

    let h = place_kind_suit(&mut e, 0, CardKind::Tao, Suit::Heart);
    activate(&mut e, 0, "xie");
    assert_open_prompt(&e, "ow_xie_card", 0);
    answer(&mut e, vec![ChoiceOption::Card(h)]);
    assert_open_prompt(&e, "ow_xie_target", 0);
    answer(&mut e, vec![ChoiceOption::Player(1)]);
    assert!(e.debug_has_skill(1, "xie2"));

    let sp = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    activate(&mut e, 0, "luan");
    assert_open_prompt(&e, "ow_luan_card", 0);
    answer(&mut e, vec![ChoiceOption::Card(sp)]);
    assert_open_prompt(&e, "ow_luan_target", 0);
    answer(&mut e, vec![ChoiceOption::Player(2)]);
    assert!(e.debug_has_skill(2, "luan2"));
    e.debug_deal_damage(Some(0), 2, 1, DamageNature::Normal);
    assert_eq!(e.debug_hp(2), 2, "乱2 受伤后追加失去 1 点体力 (4-1 dmg -1 lose)");

    e.debug_set_hp(2, 1);
    let tao2 = place_kind_suit(&mut e, 2, CardKind::Tao, Suit::Heart);
    e.debug_deal_damage(Some(0), 2, 1, DamageNature::Normal);
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if c.observer == 2 && c.spec.options.contains(&ChoiceOption::Card(tao2)) {
            answer(&mut e, vec![ChoiceOption::Card(tao2)]);
        } else {
            drain_cancel(&mut e);
        }
    }
    assert_eq!(e.debug_hp(2), 1);
    assert!(!e.debug_has_skill(2, "luan2"), "脱离濒死后清除乱2");

    activate(&mut e, 0, "sheng");
    assert_open_prompt(&e, "ow_sheng_targets", 0);
    answer(&mut e, vec![ChoiceOption::Player(0), ChoiceOption::Player(1)]);
    assert!(e.debug_face_down(0));
    assert!(e.debug_has_skill(0, "sheng2"));
    assert_eq!(e.debug_hp(0), 3, "圣回复 0 号 1 体力");
    assert_eq!(e.debug_hp(1), 3, "圣回复 1 号 1 体力");
    assert_eq!(flag(&e, 0, "ow_sheng_used"), 1);

    advance_to_play(&mut e, 1);
    let sha1 = place_kind_suit(&mut e, 1, CardKind::Sha, Suit::Heart);
    assert!(
        matches!(
            play(&mut e, 1, sha1, vec![0]),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ),
        "sheng2 状态中不可成为他人牌的目标"
    );
    advance_one_turn_to_play(&mut e, 2);
    assert_eq!(e.debug_hp(1), 4, "1 号结束窗首次触发 xie2 回 1 体力 (3 -> 4)");
    assert_card_conservation(&e, "zenyatta end");
}

/// 法老之鹰 (`character/ow/skill.js:2780-2910`):
/// - `feidan` 杀伤后弃 1 牌对与受害者距离<=1 的其他存活角色造成 1 伤；
///   **核心**：拥有 `feidan` 时出杀严格只可指定**距离 > 1**的目标（`dist <= 1` 被拒绝！且 `dist > 1` 无视距离限制）；
/// - `huoyu` limited 弃 2 红牌依次对所有其他角色使用 2 张虚拟【炽羽袭】（Fire `Chiyuxi`）；
/// - `feiying` 他人算到自己距离 +1。
#[test]
fn ow_batch_d_pharah_feidan_range_gate_huoyu_and_feiying() {
    let (verified, mut e) = start_formal_ow(2026_1161, 4);
    e.assign_general_named(0, "ow_falaozhiying").unwrap();
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e);

    assert_eq!(e.state_core().computed_distance(e.catalog(), 1, 0), 2, "飞影令他人到己方距离+1");

    let sha_bad = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Heart);
    assert!(
        matches!(
            play(&mut e, 0, sha_bad, vec![1]),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ),
        "OW 飞弹严格禁止对距离 <= 1 的目标出【杀】"
    );

    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);
    e.debug_set_hp(3, 4);
    let ammo = place_any_suit(&mut e, 0, Suit::Club);
    play(&mut e, 0, sha_bad, vec![2]);
    assert_open_prompt(&e, "ow_feidan_card", 0);
    e = roundtrip_restore(&e, &verified);
    assert_open_prompt(&e, "ow_feidan_card", 0);
    answer(&mut e, vec![ChoiceOption::Card(ammo)]);
    assert_eq!(e.debug_hp(2), 3, "2 号中飞弹杀伤");
    assert_eq!(e.debug_hp(1), 3, "飞弹溅射 2 号相邻的 1 号");
    assert_eq!(e.debug_hp(3), 3, "飞弹溅射 2 号相邻的 3 号");

    let _shan1 = place_kind_suit(&mut e, 1, CardKind::Shan, Suit::Heart);
    let _shan2 = place_kind_suit(&mut e, 2, CardKind::Shan, Suit::Diamond);
    let _shan3 = place_kind_suit(&mut e, 3, CardKind::Shan, Suit::Heart);
    let r1 = place_kind_suit(&mut e, 0, CardKind::Tao, Suit::Heart);
    let r2 = place_kind_suit(&mut e, 0, CardKind::Shan, Suit::Diamond);
    activate(&mut e, 0, "huoyu");
    assert_open_prompt(&e, "ow_huoyu_cards", 0);
    e = roundtrip_restore(&e, &verified);
    assert_open_prompt(&e, "ow_huoyu_cards", 0);
    answer(&mut e, vec![ChoiceOption::Card(r1), ChoiceOption::Card(r2)]);
    let mut aoe_order: Vec<u8> = Vec::new();
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if c.spec.prompt == "aoe_response" {
            aoe_order.push(c.observer);
            answer(&mut e, vec![ChoiceOption::Pass]);
        } else {
            answer_cancel(&mut e);
        }
    }
    assert!(
        aoe_order.starts_with(&[3, 2, 1]),
        "火雨 炽羽袭 须按逆序 `[3, 2, 1]` 结算，实测 {aoe_order:?}"
    );
    assert_eq!(flag(&e, 0, "ow_huoyu_used"), 1, "火雨 limited 已消费");
    assert!(
        matches!(activate(&mut e, 0, "huoyu"), CommandOutcome::Rejected(_)),
        "火雨每局仅限 1 次"
    );
    assert!(e.debug_hp(1) < 3 && e.debug_hp(2) < 3 && e.debug_hp(3) < 3, "2 发炽羽袭命中其余各座");
    assert_card_conservation(&e, "pharah end");
}

/// 卢西奥 (`ow_luxiao`: `yuedong`, `huhuan`, `kuoyin`/`kuoyin2`)、
/// 秩序之光 (`ow_zhixuzhiguang`: `guangshu` H/D/C/S)、
/// 天使 (`ow_tianshi`: `shouhu`, `ziyu`)。
#[test]
fn ow_batch_d_lucio_symmetra_and_mercy() {
    let (_verified, mut e) = start_formal_ow(2026_1162, 4);
    e.assign_general_named(0, "ow_luxiao").unwrap();
    e.assign_general_named(1, "ow_zhixuzhiguang").unwrap();
    e.assign_general_named(2, "ow_tianshi").unwrap();
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e);
    e.debug_set_hp(0, 2);
    e.debug_set_hp(1, 2);

    // 1. 卢西奥 `huhuan`(弃2→改回复) + `kuoyin`(弃1→目标+2) → 结束出牌触发 `yuedong` 回复 0/1 号各 1 点！
    let a = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    let b = place_kind_suit(&mut e, 0, CardKind::Shan, Suit::Heart);
    let c = place_any_suit(&mut e, 0, Suit::Club);
    activate(&mut e, 0, "huhuan");
    assert_open_prompt(&e, "ow_huhuan_cards", 0);
    answer(&mut e, vec![ChoiceOption::Card(a), ChoiceOption::Card(b)]);
    activate(&mut e, 0, "kuoyin");
    assert_open_prompt(&e, "ow_kuoyin_cards", 0);
    answer(&mut e, vec![ChoiceOption::Card(c)]);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert_open_prompt(&e, "ow_yuedong_targets", 0);
    answer(&mut e, vec![ChoiceOption::Player(0), ChoiceOption::Player(1)]);
    drain_cancel(&mut e);
    assert_eq!(e.debug_hp(0), 3, "乐动切回复模式治疗 0 号");
    assert_eq!(e.debug_hp(1), 3, "乐动扩音+2目标同时治疗 1 号");

    // 2. 秩序之光 1 号回合：连续给 0/1/2/3 号上 4 种花色光束
    advance_to_play(&mut e, 1);
    clear_all_hands(&mut e);
    let club = place_any_suit(&mut e, 1, Suit::Club);
    let heart = place_kind_suit(&mut e, 1, CardKind::Tao, Suit::Heart);
    let diamond = place_kind_suit(&mut e, 1, CardKind::Shan, Suit::Diamond);
    let spade = place_kind_suit(&mut e, 1, CardKind::Sha, Suit::Spade);
    for (card, target) in [(club, 0u8), (heart, 2), (diamond, 3), (spade, 1)] {
        activate(&mut e, 1, "guangshu");
        assert_open_prompt(&e, "ow_guangshu_card", 1);
        answer(&mut e, vec![ChoiceOption::Card(card)]);
        assert_open_prompt(&e, "ow_guangshu_target", 1);
        answer(&mut e, vec![ChoiceOption::Player(target)]);
    }
    assert!(e.debug_has_skill(0, "guangshu_club"));
    assert!(e.debug_has_skill(2, "guangshu_heart"));
    assert!(e.debug_has_skill(3, "guangshu_diamond"));
    assert!(e.debug_has_skill(1, "guangshu_spade"));
    let hp1_before = e.debug_hp(1);
    // 1 号从 Play -> Discard -> TurnEndWindow 结算 guangshu_spade
    advance_one_turn_to_play(&mut e, 2);
    assert_eq!(e.debug_hp(1), hp1_before - 1, "黑桃光束于回合结束造成 1 点无源雷伤");
    assert!(!e.debug_has_skill(1, "guangshu_spade"));

    // 3. 天使 2 号回合：
    clear_all_hands(&mut e);
    e.debug_set_hp(2, 2);
    e.debug_set_hp(0, 2);
    let sha2 = place_kind_suit(&mut e, 2, CardKind::Sha, Suit::Heart);
    assert!(
        matches!(
            play(&mut e, 2, sha2, vec![1]),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ),
        "守护禁天使主动出【杀】"
    );
    activate(&mut e, 2, "shouhu");
    assert_open_prompt(&e, "ow_shouhu_card", 2);
    answer(&mut e, vec![ChoiceOption::Card(sha2)]);
    assert_open_prompt(&e, "ow_shouhu_target", 2);
    answer(&mut e, vec![ChoiceOption::Player(0)]);
    assert_eq!(e.debug_hp(0), 3, "守护弃杀令 0 号回 1 血");

    e.debug_deal_damage(Some(1), 2, 1, DamageNature::Normal);
    assert_eq!(e.debug_hp(2), 2, "2 号红桃光束抵消掉血节奏回 1 血");
    assert!(!e.debug_has_skill(2, "guangshu_heart"));

    // 回到 0 号回合验证草花光束禁出杀 & 结束时移除、扩音2 重置 yuedong
    advance_to_play(&mut e, 0);
    let sha0_ban = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    assert!(
        matches!(
            play(&mut e, 0, sha0_ban, vec![1]),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ),
        "草花光束禁止持有者出【杀】"
    );
    assert_eq!(flag(&e, 0, "ow_yuedong_recover"), 0, "扩音2 于卢西奥回合开始重置 yuedong 标记");
    clear_all_hands(&mut e);
    advance_one_turn_to_play(&mut e, 1);
    assert!(!e.debug_has_skill(0, "guangshu_club"), "草花光束于 0 号回合结束清除");
    let _ = advance_to_turn_end_prompt;
    assert_card_conservation(&e, "batch_d trio end");
}

/// 猎空 (`character/ow/skill.js:3120-3316`):
/// - `shanxian` 他人回合开始摸 1 并记录手牌/装备快照 `ow_shanxian_h/e`，劫持插入己方插队回合，
///   对 `victim_b` 距离/范围限制豁免；回合结束后自身翻面并归还至 `victim_b`；
/// - `shanhui` 受伤后收回快照手牌/装备并弃现牌，若快照少于现牌数额外回 1 体力，全程严格卡牌守恒！
#[test]
fn ow_batch_d_tracer_shanxian_and_shanhui_card_conservation() {
    let (verified, mut e) = start_formal_ow(2026_1163, 4);
    e.assign_general_named(0, "ow_liekong").unwrap();
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e);
    let h0 = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Heart);
    let eq0 = e.debug_place(0, "renwang");
    play(&mut e, 0, eq0, vec![]);
    for _ in 0..16 {
        if let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt == "ow_shanxian_confirm" {
                break;
            }
            drain_cancel(&mut e);
            continue;
        }
        let cur = e.turn().current;
        let _ = submit(&mut e, cur, GameAction::NextPhase);
    }
    assert_open_prompt(&e, "ow_shanxian_confirm", 0);
    e = roundtrip_restore(&e, &verified);
    assert_open_prompt(&e, "ow_shanxian_confirm", 0);
    answer(&mut e, vec![ChoiceOption::Confirm]);

    for _ in 0..8 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            break;
        }
        if e.open_choice().is_some() {
            drain_cancel(&mut e);
            continue;
        }
        let cur = e.turn().current;
        let _ = submit(&mut e, cur, GameAction::NextPhase);
    }
    assert_eq!(e.turn().current, 0, "闪现劫持 1 号回合，当前回合角色=猎空 0 号");
    assert_eq!(flag(&e, 0, "ow_shanxian_target"), 2, "victim_b = 1 (+1编码=2)");
    e.debug_set_hp(0, 2);
    play(&mut e, 0, h0, vec![1]);
    drain_cancel(&mut e);
    assert_card_conservation(&e, "tracer mid shanxian");

    let _x1 = place_any_suit(&mut e, 0, Suit::Club);
    let _x2 = place_kind_suit(&mut e, 0, CardKind::Shan, Suit::Diamond);
    let _x3 = place_any_suit(&mut e, 0, Suit::Spade);
    // 校验 shanhui 对 source == Some(owner) 仅触发 1 次（不得 DamageSource+DamageEnd 双触发），并恢复快照 + 回血
    let out_sh = e.debug_deal_damage(Some(0), 2, 1, DamageNature::Normal);
    assert!(matches!(out_sh, CommandOutcome::Pending { .. }));
    assert_open_prompt(&e, "ow_shanhui_confirm", 0);
    answer(&mut e, vec![ChoiceOption::Confirm]);
    assert!(
        e.open_choice().is_none(),
        "shanhui 造成伤害后仅触发 1 次（不得双弹出）"
    );
    assert_eq!(e.debug_hand(0).len(), 2, "闪回精准恢复快照 2 张手牌（h0 + 闪现所摸 1 张）");
    assert!(e.debug_hand(0).contains(&h0), "闪回手牌快照必含 h0");
    assert_eq!(e.debug_equips(0), vec![eq0], "闪回精准恢复快照装备 eq0");
    assert_eq!(e.debug_hp(0), 3, "现牌数 > 快照数，闪回额外回复 1 体力 (2→3)");
    assert_card_conservation(&e, "tracer after shanhui");

    advance_one_turn_to_play(&mut e, 1);
    assert!(e.debug_face_down(0), "闪现回合结束后猎空翻面");
    assert_card_conservation(&e, "tracer end");
}

/// 公开信息 AI (`BoundedPlanner`) 隔离测试 + 中途 `save -> encode -> decode -> restore -> Answer` 测试。
#[test]
fn ow_public_info_ai_and_mid_choice_save_restore_roundtrip() {
    let (verified, mut e) = start_formal_ow(2026_1164, 4);
    e.assign_general_named(0, "ow_zhixuzhiguang").unwrap();
    e.assign_general_named(1, "ow_heibaihe").unwrap();
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e);

    e.state_core_mut().mode = engine_native::state::GameMode::Identity {
        roles: vec![Role::Lord, Role::Rebel, Role::Loyalist, Role::Traitor],
    };
    let _c_heart = place_kind_suit(&mut e, 0, CardKind::Tao, Suit::Heart);
    let c_club = place_any_suit(&mut e, 0, Suit::Club);
    let _hidden2 = place_any_suit(&mut e, 2, Suit::Spade);
    activate(&mut e, 0, "guangshu");
    assert_open_prompt(&e, "ow_guangshu_card", 0);

    let mut s_alt = e.save();
    s_alt.core.mode = engine_native::state::GameMode::Identity {
        roles: vec![Role::Lord, Role::Loyalist, Role::Traitor, Role::Rebel],
    };
    std::mem::swap(&mut s_alt.core.players[2].hand[0], &mut s_alt.core.deck[0]);
    let e_alt = engine_native_kernel_restore_via_loader(&s_alt, &verified);

    assert_eq!(
        format!("{:?}", e.ai_session(0).input()),
        format!("{:?}", e_alt.ai_session(0).input()),
        "OW 公开信息输入视图不泄漏未明置身份/他人手牌"
    );
    let planner = BoundedPlanner::new(64).with_policy_variant(2);
    let plan_a = e.plan_action(0, &planner);
    let plan_b = e_alt.plan_action(0, &planner);
    assert_eq!(plan_a.action, plan_b.action, "OW AI 决策对隐藏信息保持不变");

    let mut restored = roundtrip_restore(&e, &verified);
    assert_open_prompt(&restored, "ow_guangshu_card", 0);
    restored.apply_plan(&plan_a).expect("AI 推荐动作可直接在恢复引擎上执行");
    assert_open_prompt(&restored, "ow_guangshu_target", 0);
    let plan_t = restored.plan_action(0, &planner);
    restored.apply_plan(&plan_t).expect("AI 选光束目标合法执行");

    answer(&mut e, vec![ChoiceOption::Card(c_club)]);
    assert_open_prompt(&e, "ow_guangshu_target", 0);
    let mut e_step2 = roundtrip_restore(&e, &verified);
    answer(&mut e_step2, vec![ChoiceOption::Player(1)]);
    assert!(e_step2.debug_has_skill(1, "guangshu_club"));
    assert_card_conservation(&e_step2, "ai save restore end");
}

fn engine_native_kernel_restore_via_loader(
    snap: &engine_native::StateSnapshot,
    verified: &content_native_loader::VerifiedLockedContent,
) -> engine_native::Engine {
    let bytes = engine_native::archive::encode_archive(snap);
    let decoded = engine_native::archive::decode_archive(&bytes).unwrap();
    content_native_loader::restore_locked_engine(&decoded, verified).unwrap()
}
