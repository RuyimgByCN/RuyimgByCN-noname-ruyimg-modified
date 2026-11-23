//! U05-3 (`shaai-ow`) Batch B 真实事件闭环测试：
//! 莱因哈特 (`ow_laiyinhate`: `zhongdun`, `mengji`)、
//! 托比昂 (`ow_tuobiang`: `maoding`/`maoding2`, `paotai`/`paotai2`/`paotai3`)、
//! 半藏 (`ow_banzang`: `bfengshi`/`bfengshi2`/`bfengshi3`/`bfengshi4`, `yinbo`)、
//! 安娜 (`ow_ana`: `aqianghua`/`aqianghua2`, `zhiyuan`, `mianzhen`/`mianzhen2`)、
//! 狂鼠 (`ow_kuangshu`: `liudan`, `shoujia`/`shoujia2`/`shoujia3`, `shihuo`)、
//! 麦克雷 (`ow_maikelei`: `shanguang`/`shanguang2`, `tiandan`, `shenqiang`)、
//! 美 (`ow_mei`: `bingqiang`/`bingqiang2`/`bingqiang5`/`bingqiang_remove`, `jidong`/`jidong2`, `baoxue`)。

use engine_native::catalog::{CardKind, Suit};
use engine_native::event::DamageNature;
use engine_native::state::GameMode;
use engine_native::{ChoiceOption, CommandOutcome, GameAction, RejectReason};

use super::common::{
    activate, advance_one_turn_to_play, advance_to_play, advance_to_turn_end_prompt, answer,
    answer_cancel, assert_card_conservation, assert_open_prompt, clear_all_hands, drain_cancel,
    flag, place_any_suit, place_kind_suit, play, roundtrip_restore, start_formal_ow, submit,
};

fn set_versus_2v2(e: &mut engine_native::Engine) {
    e.state_core_mut().mode = GameMode::Versus {
        teams: vec![vec![0, 3], vec![1, 2]],
        substitutes: vec![vec![], vec![]],
    };
}

/// 莱因哈特 (`character/ow/skill.js:766-817`):
/// - `zhongdun` 建局/登场自获 +8 护甲；主动弃 1 牌为 `hujia==0` 的他人加 1 护甲（`hujia>0` 不入选项）；
/// - `mengji` 当持有者无护甲时造成的杀伤+1，有护甲时不加伤。
#[test]
fn ow_batch_b_reinhardt_zhongdun_and_mengji() {
    let (_verified, mut e) = start_formal_ow(2026_1140, 4);
    e.assign_general_named(0, "ow_laiyinhate").unwrap();
    advance_to_play(&mut e, 0);
    assert_eq!(e.state_core().player(0).hujia, 8, "重盾开局初始 +8 护甲");
    clear_all_hands(&mut e);

    // 1. 设 2 号已有 1 点护甲、1 号为 0 点护甲：重盾可选目标仅 1、3 号，排除 2 号！
    e.debug_set_hujia(2, 1);
    let cost = place_any_suit(&mut e, 0, Suit::Club);
    activate(&mut e, 0, "zhongdun");
    assert_open_prompt(&e, "ow_zhongdun_card", 0);
    answer(&mut e, vec![ChoiceOption::Card(cost)]);
    assert_open_prompt(&e, "ow_zhongdun_target", 0);
    let c = e.open_choice().unwrap();
    assert!(!c.spec.options.contains(&ChoiceOption::Player(2)), "已有护甲的 2 号不可选");
    assert!(c.spec.options.contains(&ChoiceOption::Player(1)));
    answer(&mut e, vec![ChoiceOption::Player(1)]);
    assert_eq!(e.state_core().player(0).hujia, 7, "重盾自扣 1 护甲 (8→7)");
    assert_eq!(e.state_core().player(1).hujia, 1);

    // 2. 猛击（`hujia > 0` 时不加伤；`hujia == 0` 时重盾禁止发动且杀伤 +1 = 2 点！）：
    e.debug_set_hp(1, 4);
    e.debug_set_hp(3, 4);
    let sha_to_1 = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Heart);
    play(&mut e, 0, sha_to_1, vec![1]);
    drain_cancel(&mut e);
    assert_eq!(e.state_core().player(1).hujia, 0, "持有者有 7 护甲时不加伤，仅破 1 号 1 护甲");
    assert_eq!(e.debug_hp(1), 4);

    // 护甲归零后：重盾不可发动（`hujia == 0`），且装连弩杀 3 号 → 猛击伤害 +1 = 2 点！
    e.debug_set_hujia(0, 0);
    let _extra0 = place_any_suit(&mut e, 0, Suit::Spade);
    assert!(
        matches!(activate(&mut e, 0, "zhongdun"), CommandOutcome::Rejected(_)),
        "无护甲时重盾禁止发动"
    );
    let zhuge = e.debug_place(0, "zhuge");
    play(&mut e, 0, zhuge, vec![]);
    let sha_to_3 = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    play(&mut e, 0, sha_to_3, vec![3]);
    drain_cancel(&mut e);
    assert_eq!(e.debug_hp(3), 2, "莱因哈特无护甲时猛击对杀伤 +1（造成 2 伤）");
    assert_card_conservation(&e, "reinhardt end");
}

/// 托比昂 (`character/ow/skill.js:818-940`):
/// - `maoding` 受到/造成伤害后从离场补给随机获得 1 张 `ow_hslingjian_*`；集齐 2 张发动 `maoding2` 为无护甲角色 +1 护甲；
/// - `paotai` 弃 1 张【杀】升级炮台（上限 3 级）：3 级时回合结束 100% 对敌方随机造成 1 点火焰伤害；受伤掉级；死亡清空。
#[test]
fn ow_batch_b_torbjorn_maoding_and_paotai() {
    let (_verified, mut e) = start_formal_ow(2026_1141, 4);
    set_versus_2v2(&mut e);
    e.assign_general_named(0, "ow_tuobiang").unwrap();
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e);

    // 1. 触发 2 次 maoding（1 次受伤、1 次造伤）严格各只弹窗 1 次并收 2 张零件，随后发动 `maoding2` 加 1 护甲
    e.debug_set_hp(0, 3);
    let out1 = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    assert!(matches!(out1, CommandOutcome::Pending { .. }));
    assert_open_prompt(&e, "ow_maoding_confirm", 0);
    answer(&mut e, vec![ChoiceOption::Confirm]);
    assert!(e.open_choice().is_none(), "受到伤害只触发 1 次 maoding");

    let out2 = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert!(matches!(out2, CommandOutcome::Pending { .. }));
    assert_open_prompt(&e, "ow_maoding_confirm", 0);
    answer(&mut e, vec![ChoiceOption::Confirm]);
    assert!(e.open_choice().is_none(), "造成伤害只触发 1 次 maoding，不得双触发");
    let lingjian: Vec<u32> = e
        .debug_hand(0)
        .iter()
        .copied()
        .filter(|c| e.catalog().card_kind(*c) == CardKind::Hslingjian)
        .collect();
    assert_eq!(lingjian.len(), 2, "铆钉从 detached_cards 获取 2 张 ow_hslingjian");
    activate(&mut e, 0, "maoding");
    assert_open_prompt(&e, "ow_maoding2_cards", 0);
    answer(&mut e, vec![ChoiceOption::Card(lingjian[0]), ChoiceOption::Card(lingjian[1])]);
    assert_open_prompt(&e, "ow_maoding2_target", 0);
    answer(&mut e, vec![ChoiceOption::Player(0)]);
    assert_eq!(e.state_core().player(0).hujia, 1, "弃 2 零件加 1 护甲");

    // 2. 弃 3 张【杀】堆满 3 级炮台（30% -> 60% -> 100% 对敌方 1/2 号火焰伤害，友方 3 号免疫）
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);
    e.debug_set_hp(3, 4);
    for suit in [Suit::Spade, Suit::Heart, Suit::Club] {
        let s = place_kind_suit(&mut e, 0, CardKind::Sha, suit);
        activate(&mut e, 0, "paotai");
        assert_open_prompt(&e, "ow_paotai_card", 0);
        answer(&mut e, vec![ChoiceOption::Card(s)]);
    }
    assert_eq!(flag(&e, 0, "ow_paotai_lv"), 3, "炮台升至 3 级");

    let enemy_hp_before = e.debug_hp(1) + e.debug_hp(2);
    advance_one_turn_to_play(&mut e, 1);
    let enemy_hp_after = e.debug_hp(1) + e.debug_hp(2);
    assert_eq!(enemy_hp_after, enemy_hp_before - 1, "3 级炮台 100% 对敌方造成 1 点火焰伤害");
    assert_eq!(e.debug_hp(3), 4, "友方 3 号不受炮台伤害");

    // 3. 受伤 2 点 → `paotai3` 炮台掉级 (3 -> 1)
    e.debug_set_hujia(0, 0);
    e.debug_set_hp(0, 4);
    e.debug_deal_damage(Some(1), 0, 2, DamageNature::Normal);
    drain_cancel(&mut e);
    assert_eq!(flag(&e, 0, "ow_paotai_lv"), 1, "受到 2 点伤害炮台 3->1 级");
    assert_card_conservation(&e, "torbjorn end");
}

/// 半藏 (`character/ow/skill.js:941-1035`) 与 安娜 (`1036-1140`):
/// - 半藏 `yinbo` 弃黑桃随机弃至多 3 名敌方各 1 牌（不弃队友）；
/// - 安娜 `aqianghua` 全手牌给队友 +1 护甲 + 一次性 `aqianghua2` +1 伤、`zhiyuan` 取消己方造成伤害回等量血、`mianzhen` 逼闪或封手牌至回合末。
#[test]
fn ow_batch_b_hanzo_and_ana() {
    let (verified, mut e) = start_formal_ow(2026_1142, 4);
    set_versus_2v2(&mut e);
    e.assign_general_named(0, "ow_banzang").unwrap();
    e.assign_general_named(3, "ow_ana").unwrap();
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e);

    // 1. 音波弃黑桃只弃敌方 1/2，不弃友方 3 号
    let spade0 = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    let _c1 = place_kind_suit(&mut e, 1, CardKind::Tao, Suit::Heart);
    let _c2 = place_kind_suit(&mut e, 2, CardKind::Shan, Suit::Diamond);
    let c3 = place_kind_suit(&mut e, 3, CardKind::Tao, Suit::Heart);
    activate(&mut e, 0, "yinbo");
    assert_open_prompt(&e, "ow_yinbo_card", 0);
    answer(&mut e, vec![ChoiceOption::Card(spade0)]);
    assert!(e.debug_hand(1).is_empty(), "敌方 1 号被音波弃牌");
    assert!(e.debug_hand(2).is_empty(), "敌方 2 号被音波弃牌");
    assert_eq!(e.debug_hand(3), vec![c3], "友方 3 号不受音波弃牌");

    // 2. 安娜强化与支援
    advance_to_play(&mut e, 3);
    clear_all_hands(&mut e);
    e.debug_set_hp(0, 3);
    e.debug_set_max_hp(0, 4);
    let gift = place_kind_suit(&mut e, 3, CardKind::Tao, Suit::Heart);
    activate(&mut e, 3, "aqianghua");
    assert_open_prompt(&e, "ow_aqianghua_target", 3);
    answer(&mut e, vec![ChoiceOption::Player(0)]);
    assert!(e.debug_hand(0).contains(&gift), "纳米强化把全手牌交给 0 号");
    assert_eq!(e.state_core().player(0).hujia, 1);
    assert!(e.debug_has_skill(0, "aqianghua2"));

    let duel = place_kind_suit(&mut e, 3, CardKind::Duel, Suit::Diamond);
    play(&mut e, 3, duel, vec![0]);
    assert_open_prompt(&e, "ow_zhiyuan_confirm", 3);
    answer(&mut e, vec![ChoiceOption::Confirm]);
    assert_eq!(e.debug_hp(0), 4, "支援取消伤害改令目标回复 1 点体力");

    // 3. 麻醉针：1 号有闪但取消出闪 → 获得 `mianzhen2`
    clear_all_hands(&mut e);
    let cost_mz = place_any_suit(&mut e, 3, Suit::Club);
    let _shan1 = place_kind_suit(&mut e, 1, CardKind::Shan, Suit::Heart);
    activate(&mut e, 3, "mianzhen");
    assert_open_prompt(&e, "ow_mianzhen_card", 3);
    answer(&mut e, vec![ChoiceOption::Card(cost_mz)]);
    assert_open_prompt(&e, "ow_mianzhen_target", 3);
    answer(&mut e, vec![ChoiceOption::Player(1)]);
    assert_open_prompt(&e, "ow_mianzhen_shan", 1);
    e = roundtrip_restore(&e, &verified);
    assert_open_prompt(&e, "ow_mianzhen_shan", 1);
    answer_cancel(&mut e);
    assert!(e.debug_has_skill(1, "mianzhen2"), "未交闪则获 mianzhen2");

    advance_to_play(&mut e, 1);
    let sha1 = place_kind_suit(&mut e, 1, CardKind::Sha, Suit::Heart);
    assert!(
        matches!(
            play(&mut e, 1, sha1, vec![2]),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ),
        "mianzhen2 封禁手牌"
    );
    advance_one_turn_to_play(&mut e, 2);
    assert!(!e.debug_has_skill(1, "mianzhen2"), "麻醉针在 1 号回合结束解除");
    assert_card_conservation(&e, "hanzo ana end");
}

/// 狂鼠 (`character/ow/skill.js:1141-1215`) 与 麦克雷 (`1216-1312`):
/// - 狂鼠 `shoujia` 把牌存入目标 `ow_shoujia` 暗桩；目标对他者使用**同花色**牌触发翻面+弃置陷阱；`shihuo` 火伤摸 1；
/// - 麦克雷 `shanguang` 弃方片封锁 `shanguang2`；`tiandan` 摸牌阶段前确认补 `ow_sha_*` 并跳过出牌与弃牌；`shenqiang` 杀伤后重置杀次限。
#[test]
fn ow_batch_b_junkrat_and_mccree() {
    let (_verified, mut e) = start_formal_ow(2026_1143, 4);
    e.assign_general_named(0, "ow_kuangshu").unwrap();
    e.assign_general_named(2, "ow_maikelei").unwrap();
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e);

    // 1. 兽夹挂方片牌给 1 号，检查 vm_aux.piles 守恒 + 1 号对他人使用方片牌触发翻面并拆牌
    let trap = place_kind_suit(&mut e, 0, CardKind::Shan, Suit::Diamond);
    activate(&mut e, 0, "shoujia");
    assert_open_prompt(&e, "ow_shoujia_card", 0);
    answer(&mut e, vec![ChoiceOption::Card(trap)]);
    assert_open_prompt(&e, "ow_shoujia_target", 0);
    answer(&mut e, vec![ChoiceOption::Player(1)]);
    assert!(e.debug_has_skill(1, "shoujia2"));
    assert_eq!(e.state_core().vm_aux.piles[&(1, "ow_shoujia".to_string())], vec![trap]);
    assert_card_conservation(&e, "shoujia placed");

    // 狂鼠测 shihuo：造成 1 点火焰伤害后摸 1
    let hand_before_fire = e.debug_hand(0).len();
    e.debug_deal_damage(Some(0), 3, 1, DamageNature::Fire);
    assert_eq!(e.debug_hand(0).len(), hand_before_fire + 1, "嗜火：火伤后摸 1");

    // 1 号回合对方以外 2 号使用方片【杀】（同花色且 target!=self）触发兽夹2 翻面且陷阱入弃牌堆
    advance_to_play(&mut e, 1);
    let sha_dia = place_kind_suit(&mut e, 1, CardKind::Sha, Suit::Diamond);
    play(&mut e, 1, sha_dia, vec![2]);
    drain_cancel(&mut e);
    assert!(e.debug_face_down(1), "兽夹2 同花色对外用牌触发令 1 号翻面");
    assert!(!e.debug_has_skill(1, "shoujia2"));
    assert_card_conservation(&e, "shoujia triggered");

    // 2. 麦克雷回合前走 `tiandan` 跳过 Play & Discard，补 `ow_sha_*` 至 min(5, hp)=4
    advance_to_play(&mut e, 2);
    clear_all_hands(&mut e);
    advance_one_turn_to_play(&mut e, 3);
    e.debug_clear_hand(2);
    e.debug_set_hp(2, 4);
    let mut saw_tiandan = false;
    for _ in 0..48 {
        if let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt == "ow_tiandan_confirm" {
                saw_tiandan = true;
                answer(&mut e, vec![ChoiceOption::Confirm]);
                continue;
            }
            drain_cancel(&mut e);
            continue;
        }
        if saw_tiandan && e.turn().current == 3 {
            break;
        }
        let cur = e.turn().current;
        let _ = submit(&mut e, cur, GameAction::NextPhase);
    }
    assert!(saw_tiandan, "填弹应在 2 号摸牌阶段前弹出确认");
    assert!(
        e.debug_hand(2)
            .iter()
            .any(|c| e.catalog().cards[*c as usize].name.starts_with("ow_sha_")),
        "填弹从离场区获得 ow_sha 补给牌"
    );

    // 3. 麦克雷自己回合：`shanguang` 弃方片封 3 号用牌 + `shenqiang` 杀伤后重置杀次限再出一杀
    advance_to_play(&mut e, 2);
    clear_all_hands(&mut e);
    e.debug_set_hp(1, 4);
    e.debug_set_hp(3, 4);
    let dia = place_kind_suit(&mut e, 2, CardKind::Shan, Suit::Diamond);
    let sha_a = place_kind_suit(&mut e, 2, CardKind::Sha, Suit::Heart);
    let sha_b = place_kind_suit(&mut e, 2, CardKind::Sha, Suit::Spade);
    activate(&mut e, 2, "shanguang");
    assert_open_prompt(&e, "ow_shanguang_card", 2);
    answer(&mut e, vec![ChoiceOption::Card(dia)]);
    assert_open_prompt(&e, "ow_shanguang_target", 2);
    answer(&mut e, vec![ChoiceOption::Player(3)]);
    assert!(e.debug_has_skill(3, "shanguang2"));

    let _shan3 = place_kind_suit(&mut e, 3, CardKind::Shan, Suit::Heart);
    play(&mut e, 2, sha_a, vec![3]);
    drain_cancel(&mut e);
    assert_eq!(e.debug_hp(3), 3, "闪光2 封锁响应导致 3 号直接受伤");
    assert!(
        matches!(
            play(&mut e, 2, sha_b, vec![1]),
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "神枪重置杀次限后可出第 2 张【杀】"
    );
    drain_cancel(&mut e);
    assert_eq!(e.debug_hp(1), 3);
    assert_card_conservation(&e, "junkrat mccree end");
}

/// 美 (`character/ow/skill.js:1690-1820`):
/// - `bingqiang`：弃 X 张同色牌选无邻冲突中心目标，红=`bingqiang2`(`globalTo +X`)、黑=`bingqiang5`(`globalFrom +X`)；
///   混色选择重弹提示；下回合开始自动拆墙；
/// - `jidong`：`hp==1` 且正面回合结束可翻面回 2 并获 `jidong2` 免疫伤害且不可被选作目标；
/// - `baoxue`：limited 技每局限 1 次，弃全部黑色牌指定至多黑牌数名其他角色随机弃 1 并翻面 + 自身翻面。
#[test]
fn ow_batch_b_mei_bingqiang_jidong_baoxue() {
    let (_verified, mut e) = start_formal_ow(2026_1144, 4);
    e.assign_general_named(0, "ow_mei").unwrap();
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e);

    // 1. `bingqiang`: 红 2 张选 2 号（相邻 1, 3 号同墙）→ 0 到 1/2/3 距离均 +2！混色选择则重弹
    let r1 = place_kind_suit(&mut e, 0, CardKind::Tao, Suit::Heart);
    let r2 = place_kind_suit(&mut e, 0, CardKind::Shan, Suit::Diamond);
    let b1 = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    activate(&mut e, 0, "bingqiang");
    assert_open_prompt(&e, "ow_bingqiang_cards", 0);
    answer(&mut e, vec![ChoiceOption::Card(r1), ChoiceOption::Card(b1)]);
    assert_open_prompt(&e, "ow_bingqiang_cards", 0);
    answer(&mut e, vec![ChoiceOption::Card(r1), ChoiceOption::Card(r2)]);
    assert_open_prompt(&e, "ow_bingqiang_target", 0);
    answer(&mut e, vec![ChoiceOption::Player(2)]);
    assert!(e.debug_has_skill(2, "bingqiang2"));
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 2), 4, "红色冰墙令他人到受墙者距离 +2 (2+2=4)");

    // 2. limited `baoxue` 弃黑牌 `b1` 指定 1 号弃 1 牌并翻面 + 自身翻面
    let _c1 = place_kind_suit(&mut e, 1, CardKind::Shan, Suit::Heart);
    activate(&mut e, 0, "baoxue");
    assert_open_prompt(&e, "ow_baoxue_targets", 0);
    answer(&mut e, vec![ChoiceOption::Player(1)]);
    drain_cancel(&mut e);
    assert!(e.debug_face_down(1), "暴雪令目标 1 号翻面");
    assert!(e.debug_face_down(0), "暴雪令美自身翻面");
    assert!(e.debug_hand(1).is_empty(), "暴雪弃目标 1 牌");
    assert_eq!(flag(&e, 0, "ow_baoxue_used"), 1, "暴雪 limited 已消费");

    // 3. 翻面被跳过回合不提前移除冰墙（phaseBefore 先翻回并取消 TurnBegin），直到美下次真实 TurnBegin 才移除冰墙，随后测急冻
    advance_to_play(&mut e, 2);
    advance_to_play(&mut e, 1);
    assert!(!e.debug_face_down(0), "0 号已被跳过回合并翻回正面");
    assert!(
        e.debug_has_skill(2, "bingqiang2"),
        "跳过回合不触发 phaseBegin，2 号冰墙仍保留"
    );

    advance_to_play(&mut e, 0);
    assert!(
        !e.debug_has_skill(2, "bingqiang2"),
        "美真正进入自身 TurnBegin 后移除全部冰墙"
    );
    clear_all_hands(&mut e);
    e.debug_set_hp(0, 1);
    advance_to_turn_end_prompt(&mut e, 0, "ow_jidong_confirm");
    answer(&mut e, vec![ChoiceOption::Confirm]);
    drain_cancel(&mut e);
    assert!(e.state_core().player(0).jidong_active, "急冻激活");
    assert_eq!(e.debug_hp(0), 3, "急冻回复 2 点体力");
    advance_to_play(&mut e, 2);
    e.debug_deal_damage(Some(2), 0, 2, DamageNature::Thunder);
    assert_eq!(e.debug_hp(0), 3, "急冻免疫一切伤害");
    let qilin = e.debug_place(2, "qilin");
    play(&mut e, 2, qilin, vec![]);
    let sha2 = place_kind_suit(&mut e, 2, CardKind::Sha, Suit::Spade);
    assert!(
        matches!(
            play(&mut e, 2, sha2, vec![0]),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ),
        "急冻状态下不能成为其他角色牌的目标"
    );
    assert_card_conservation(&e, "mei end");
}
