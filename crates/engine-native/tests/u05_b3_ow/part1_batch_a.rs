//! U05-3 (`shaai-ow`) Batch A 真实事件闭环测试：
//! 奥丽莎 (`ow_orisa`: `woliu`/`woliu2`, `qianggu`/`qianggu2`)、
//! 温斯顿 (`ow_wensidun`: `feitiao`/`feitiao2`, `dianji`)、
//! 堡垒 (`ow_baolei`: `zhencha`/`zhencha2`, `bshaowei`)、
//! 查莉娅 (`ow_zhaliya`: `pingzhang`/`pingzhang2`/`pingzhang3`/`owliyong`)、
//! 路霸 (`ow_luba`: `liangou`/`liangou2`, `xiyang`)、
//! 黑影 (`ow_heiying`: `qinru`, `yinshen`, `maichong`)。

use engine_native::catalog::{CardKind, Suit};
use engine_native::event::DamageNature;
use engine_native::{ChoiceOption, CommandOutcome, GameAction, RejectReason};

use super::common::{
    activate, advance_one_turn_to_play, advance_to_play, advance_to_turn_end_prompt, answer,
    answer_cancel, assert_card_conservation, assert_open_prompt, clear_all_hands, drain_cancel,
    flag, place_any_suit, place_kind_suit, play, roundtrip_restore, start_formal_ow, submit,
};

/// 奥丽莎 (`character/ow/skill.js:74-188`):
/// - `qianggu` 弃 2 牌得 2 护甲 + `qianggu2`；杀指定持有 `qianggu2` 时若弃手牌基本牌则照常结算，
///   若无基本牌或选择取消则该【杀】对持有者无效（取消）；到奥丽莎下回合开始清除 `qianggu2`；
/// - `woliu` 回合结束阶段前选择至多 2 名其他角色获 `woliu2`；杀指定其中一人时其他 `woliu2` 额外成为目标；
///   到奥丽莎下回合开始或死亡清除 `woliu2`。
#[test]
fn ow_batch_a_orisa_qianggu_and_woliu() {
    let (verified, mut e) = start_formal_ow(2026_1130, 4);
    e.assign_general_named(0, "ow_orisa").unwrap();
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e);

    // 1. 正例：`qianggu` 仅 1 牌不可发动；凑足 2 牌发动后弃 2 得 +2 护甲并获 `qianggu2`
    let c1 = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    assert!(matches!(
        activate(&mut e, 0, "qianggu"),
        CommandOutcome::Rejected(_)
    ));
    let c2 = place_kind_suit(&mut e, 0, CardKind::Shan, Suit::Heart);
    activate(&mut e, 0, "qianggu");
    assert_open_prompt(&e, "ow_qianggu_cards", 0);
    // 取消退还 once_used
    answer_cancel(&mut e);
    assert_eq!(e.state_core().player(0).hujia, 0);
    activate(&mut e, 0, "qianggu");
    answer(&mut e, vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)]);
    assert_eq!(e.state_core().player(0).hujia, 2, "强固弃 2 得 +2 护甲");
    assert!(e.debug_has_skill(0, "qianggu2"));

    // 2. 结束阶段前 `woliu` 选 1、2 号：发动者 0 号与目标 1、2 号均获 `woliu2`
    advance_to_turn_end_prompt(&mut e, 0, "ow_woliu_targets");
    answer(&mut e, vec![ChoiceOption::Player(1), ChoiceOption::Player(2)]);
    drain_cancel(&mut e);
    assert!(e.debug_has_skill(0, "woliu2"), "涡流赋予者自身也获得 woliu2");
    assert!(e.debug_has_skill(1, "woliu2"));
    assert!(e.debug_has_skill(2, "woliu2"));

    // 3. 1 号杀 0 号：0 号持 `qianggu2`；1 号先弃手牌基本牌 → 杀不被取消（先过护甲）；
    //    同时 1、2 号持 `woliu2` → 2 号被涡流额外设为目标受 1 伤
    advance_to_play(&mut e, 1);
    clear_all_hands(&mut e);
    e.debug_set_hp(0, 4);
    e.debug_set_hp(2, 4);
    let sha1 = place_kind_suit(&mut e, 1, CardKind::Sha, Suit::Heart);
    let peach1 = place_kind_suit(&mut e, 1, CardKind::Tao, Suit::Heart);
    let out_q = play(&mut e, 1, sha1, vec![0]);
    assert!(
        matches!(out_q, CommandOutcome::Pending { .. }),
        "expected Pending ow_qianggu2_discard, got {out_q:?}, open={:?}, log={:?}",
        e.open_choice(),
        e.log()
    );
    assert_open_prompt(&e, "ow_qianggu2_discard", 1);
    answer(&mut e, vec![ChoiceOption::Card(peach1)]);
    drain_cancel(&mut e);
    assert_eq!(e.state_core().player(0).hujia, 1, "弃基本牌放行杀，消耗 1 护甲");
    assert_eq!(e.debug_hp(0), 4);
    assert_eq!(e.debug_hp(2), 3, "涡流2联动 2 号额外成为杀目标受 1 伤");

    // 4. 强固2 取消路径（在 2 号 Play 触发，不越过 0 号 TurnBegin）：2 号杀 0 号并取消弃基本牌 → 杀被取消，护甲不变
    advance_to_play(&mut e, 2);
    clear_all_hands(&mut e);
    let fangtian2 = e.debug_place(2, "fangtian");
    play(&mut e, 2, fangtian2, vec![]);
    drain_cancel(&mut e);
    let sha_b = place_kind_suit(&mut e, 2, CardKind::Sha, Suit::Club);
    let _peach_b = place_kind_suit(&mut e, 2, CardKind::Shan, Suit::Diamond);
    play(&mut e, 2, sha_b, vec![0]);
    assert_open_prompt(&e, "ow_qianggu2_discard", 2);
    answer_cancel(&mut e);
    drain_cancel(&mut e);
    assert_eq!(e.state_core().player(0).hujia, 1, "强固2 拒绝弃基本牌令杀无效");

    // 5. 3 号杀 0 号（0/1/2 号都持 woliu2）：0/1/2 号全部受到 1 伤（[3, 3, 3, 4]）；
    //    随后再以方天画戟多目标杀 `[1, 2]`（`paoxiao` 许出第 2 杀），验证 `woliu2` 同样联动未包含的 0 号
    advance_to_play(&mut e, 3);
    clear_all_hands(&mut e);
    e.debug_grant(3, "咆哮");
    e.debug_set_hujia(0, 0);
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);
    e.debug_set_hp(3, 4);
    let bow = e.debug_place(3, "qilin");
    play(&mut e, 3, bow, vec![]);
    drain_cancel(&mut e);
    let sha3 = place_kind_suit(&mut e, 3, CardKind::Sha, Suit::Spade);
    let basic3 = place_kind_suit(&mut e, 3, CardKind::Tao, Suit::Heart);
    play(&mut e, 3, sha3, vec![0]);
    assert_open_prompt(&e, "ow_qianggu2_discard", 3);
    answer(&mut e, vec![ChoiceOption::Card(basic3)]);
    drain_cancel(&mut e);
    assert_eq!(
        [e.debug_hp(0), e.debug_hp(1), e.debug_hp(2), e.debug_hp(3)],
        [3, 3, 3, 4],
        "3 号杀 0 号：0/1/2 号因 woliu2 联动共受 1 伤"
    );

    // 5b. 多目标【杀】（方天最后手牌打 `[1, 2]`）：`woliu2` 无 `targets.len() == 1` 限制，额外联动 0 号触发强固2
    clear_all_hands(&mut e);
    e.debug_remove_equip(2, fangtian2);
    e.state_core_mut().discard.push(fangtian2);
    let fangtian3 = e.debug_place(3, "fangtian");
    play(&mut e, 3, fangtian3, vec![]);
    drain_cancel(&mut e);
    let sha3_multi = place_kind_suit(&mut e, 3, CardKind::Sha, Suit::Heart);
    let log_mark = e.log().len();
    play(&mut e, 3, sha3_multi, vec![1, 2]);
    drain_cancel(&mut e);
    assert!(
        e.log()[log_mark..]
            .iter()
            .any(|ev| matches!(ev, engine_native::LogEvent::SkillTriggered { skill: "woliu2", .. })),
        "方天多目标杀触发 woliu2 联动 0 号"
    );
    assert!(
        e.log()[log_mark..]
            .iter()
            .any(|ev| matches!(ev, engine_native::LogEvent::SkillTriggered { skill: "qianggu2", owner: 0, .. })),
        "woliu2 联动 0 号触发强固2（最后手牌无基本牌可弃自动取消 0 号）"
    );
    assert_eq!(e.debug_hp(1), 2, "方天多目标杀命中 1 号");
    assert_eq!(e.debug_hp(2), 2, "方天多目标杀命中 2 号");
    assert_eq!(e.debug_hp(0), 3, "0 号被 woliu2 联动后经强固2 取消");

    // 6. 回到 0 号回合：TurnBegin 清 qianggu2 与全员 woliu2，存档恢复一致
    advance_to_play(&mut e, 0);
    assert!(!e.debug_has_skill(0, "qianggu2"));
    assert!(!e.debug_has_skill(0, "woliu2"));
    assert!(!e.debug_has_skill(1, "woliu2"));
    assert!(!e.debug_has_skill(2, "woliu2"));
    let _ = roundtrip_restore(&e, &verified);
    assert_card_conservation(&e, "orisa end");
}

/// 温斯顿 (`character/ow/skill.js:189-300`):
/// - `feitiao` 出牌阶段开始弃 1 牌令本回合对其距离=1 且随机弃其 1 牌（可取消）；
/// - `dianji` 弃 1 手牌视为对距离<=2 的其他角色**逆座次序**使用【惊雷闪】。
#[test]
fn ow_batch_a_winston_feitiao_and_dianji() {
    let (_verified, mut e) = start_formal_ow(2026_1131, 4);
    e.assign_general_named(0, "ow_wensidun").unwrap();

    // 推进至 0 号 Play 前（PhaseBegin 触发 feitiao）
    advance_to_play(&mut e, 3);
    clear_all_hands(&mut e);
    let fodder0 = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    let _target2_card = place_kind_suit(&mut e, 2, CardKind::Shan, Suit::Heart);
    let _ = submit(&mut e, 3, GameAction::NextPhase);
    drain_cancel(&mut e);
    for _ in 0..32 {
        if let Some(c) = e.open_choice() {
            if c.spec.prompt == "ow_feitiao_card" {
                break;
            }
            drain_cancel(&mut e);
            continue;
        }
        if e.turn().current == 0 && e.turn().phase == engine_native::Phase::Play {
            break;
        }
        let cur = e.turn().current;
        let _ = submit(&mut e, cur, GameAction::NextPhase);
    }
    assert_open_prompt(&e, "ow_feitiao_card", 0);
    answer(&mut e, vec![ChoiceOption::Card(fodder0)]);
    assert_open_prompt(&e, "ow_feitiao_target", 0);
    answer(&mut e, vec![ChoiceOption::Player(2)]);
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 2), 1, "飞跳令对 2 号距离恒 1");
    assert!(e.debug_hand(2).is_empty(), "飞跳随机弃 2 号 1 牌");
    assert!(e.debug_has_skill(0, "feitiao2"));

    // 清空手牌准备 `dianji`：1/2/3 距 0 号均 <= 2（1=1, 2=1因飞跳, 3=1），逆序为 3 -> 2 -> 1
    e.debug_clear_hand(0);
    let fuel = place_kind_suit(&mut e, 0, CardKind::Tao, Suit::Heart);
    e.debug_set_hp(1, 3);
    e.debug_set_hp(2, 3);
    e.debug_set_hp(3, 3);
    activate(&mut e, 0, "dianji");
    assert_open_prompt(&e, "ow_dianji_card", 0);
    answer(&mut e, vec![ChoiceOption::Card(fuel)]);
    drain_cancel(&mut e);
    assert_eq!(e.debug_hp(3), 2);
    assert_eq!(e.debug_hp(2), 2);
    assert_eq!(e.debug_hp(1), 2);
    // 反例：电击每回合限一次
    let _fuel2 = place_kind_suit(&mut e, 0, CardKind::Tao, Suit::Diamond);
    assert!(matches!(
        activate(&mut e, 0, "dianji"),
        CommandOutcome::Rejected(_)
    ));
    // 换回合清 feitiao2，对 2 号恢复距离 2
    advance_one_turn_to_play(&mut e, 1);
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 2), 2);
    assert_card_conservation(&e, "winston end");
}

/// 堡垒 (`character/ow/skill.js:301-423`):
/// - 回合结束阶段切为哨卫模式（`baolei_mode=1` + 获 `zhencha2`）：用杀无视防具、攻击范围无限、不计次数、强命中；
/// - 目标拥有 `baolei_mode=1` 且为 `bshaowei` 时对手对其用杀也不可闪避；
/// - 处于侦察模式（`baolei_mode != 1`）时用杀指定目标触发 `ow_zhencha_draw_recover`（摸 1 / 回 1 二选一）。
#[test]
fn ow_batch_a_bastion_bshaowei_and_zhencha() {
    let (_verified, mut e) = start_formal_ow(2026_1132, 4);
    e.assign_general_named(0, "ow_baolei").unwrap();
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e);
    e.debug_set_hp(0, 3);
    e.debug_set_max_hp(0, 4);
    e.debug_set_hp(1, 4);

    // 1. 侦察模式用杀触发 `ow_zhencha_draw_recover`：Tag(1) 回复 1 点体力
    let sha0 = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    let shan1 = place_kind_suit(&mut e, 1, CardKind::Shan, Suit::Heart);
    play(&mut e, 0, sha0, vec![1]);
    assert_open_prompt(&e, "ow_zhencha_draw_recover", 0);
    answer(&mut e, vec![ChoiceOption::Tag(1)]);
    assert_eq!(e.debug_hp(0), 4, "侦察模式选择 Tag(1) 回复 1 体力");
    assert_eq!(e.open_choice().unwrap().observer, 1);
    answer(&mut e, vec![ChoiceOption::Card(shan1)]);
    assert_eq!(e.debug_hp(1), 4);

    // 2. 回合结束前确认切到哨卫模式（`baolei_mode = 1`）
    advance_to_turn_end_prompt(&mut e, 0, "ow_zhencha_mode");
    answer(&mut e, vec![ChoiceOption::Confirm]);
    assert_eq!(e.state_core().player(0).baolei_mode, 1);
    drain_cancel(&mut e);

    // 3. 1 号杀哨卫模式 0 号：目标 `bshaowei` 令 1 号的【杀】也不可闪避！
    advance_to_play(&mut e, 1);
    clear_all_hands(&mut e);
    let _shan0 = place_kind_suit(&mut e, 0, CardKind::Shan, Suit::Diamond);
    let sha1 = place_kind_suit(&mut e, 1, CardKind::Sha, Suit::Heart);
    play(&mut e, 1, sha1, vec![0]);
    drain_cancel(&mut e);
    assert_eq!(e.debug_hp(0), 3, "哨卫模式 0 号不可闪避 1 号杀（直接扣血）");

    // 4. 给 2 号装仁王盾，回到 0 号回合（已清 `zhencha2`）测试无视仁王盾+无限距离+不计次数+强命
    advance_to_play(&mut e, 2);
    clear_all_hands(&mut e);
    let renwang = e.debug_place(2, "renwang");
    play(&mut e, 2, renwang, vec![]);
    drain_cancel(&mut e);

    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e);
    e.debug_set_hp(2, 4);
    let _shan2 = place_kind_suit(&mut e, 2, CardKind::Shan, Suit::Heart);
    let s1 = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    let s2 = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Club);
    assert!(matches!(
        play(&mut e, 0, s1, vec![2]),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    drain_cancel(&mut e);
    assert!(matches!(
        play(&mut e, 0, s2, vec![2]),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    drain_cancel(&mut e);
    assert_eq!(e.debug_hp(2), 2, "哨卫模式无视距离/次数/仁王盾且强命，连中 2 杀");
    assert_card_conservation(&e, "bastion end");
}

/// 查莉娅 (`character/ow/skill.js:424-521`):
/// - `pingzhang`：自己受伤害弃红桃牌 -1 伤（记录 `pingzhang2`）、他人受伤害弃黑桃牌 -1 伤（记录 `pingzhang3`），
///   本回合各自限一次；到查莉娅下回合开始汇总并由 `owliyong` 摸牌阶段额外摸等量牌。
#[test]
fn ow_batch_a_zarya_pingzhang_and_owliyong() {
    let (_verified, mut e) = start_formal_ow(2026_1133, 4);
    e.assign_general_named(0, "ow_zhaliya").unwrap();
    advance_to_play(&mut e, 1);
    clear_all_hands(&mut e);
    e.debug_set_hp(0, 4);
    e.debug_set_hp(2, 4);

    let heart0 = place_kind_suit(&mut e, 0, CardKind::Tao, Suit::Heart);
    let spade0 = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    let sha1a = place_kind_suit(&mut e, 1, CardKind::Sha, Suit::Club);
    let duel = place_kind_suit(&mut e, 1, CardKind::Duel, Suit::Spade);

    // 对 0 号出杀 → 触发 pingzhang 要红桃
    let out_pz = play(&mut e, 1, sha1a, vec![0]);
    assert!(
        matches!(out_pz, CommandOutcome::Pending { .. }),
        "expected Pending ow_pingzhang_discard, got {out_pz:?}, open={:?}, log={:?}",
        e.open_choice(),
        e.log()
    );
    assert_open_prompt(&e, "ow_pingzhang_discard", 0);
    answer(&mut e, vec![ChoiceOption::Card(heart0)]);
    drain_cancel(&mut e);
    assert_eq!(e.debug_hp(0), 4, "屏障弃红桃挡 1 伤");
    assert!(e.debug_has_skill(0, "pingzhang2"));

    // 用决斗打 2 号 → 触发 pingzhang 要黑桃
    play(&mut e, 1, duel, vec![2]);
    assert_open_prompt(&e, "ow_pingzhang_discard", 0);
    answer(&mut e, vec![ChoiceOption::Card(spade0)]);
    drain_cancel(&mut e);
    assert_eq!(e.debug_hp(2), 4, "屏障弃黑桃挡 2 号 1 伤");
    assert!(e.debug_has_skill(0, "pingzhang3"));

    // 推进到 0 号摸牌后出牌：正常摸 2 + 利用额外摸 2 = 4 张！
    advance_to_play(&mut e, 0);
    assert_eq!(e.debug_hand(0).len(), 4, "利用触发：摸牌 2 + 屏障记录 2 = 4 张");
    assert!(!e.debug_has_skill(0, "pingzhang2"));
    assert!(!e.debug_has_skill(0, "pingzhang3"));
    assert_card_conservation(&e, "zarya end");
}

/// 路霸 (`character/ow/skill.js:522-593`):
/// - `liangou`：弃 1 牌指定其他角色并真实 `OrdinaryJudge`，非红桃则赋予 `liangou2`（对其距离=1 且本回合首次受伤+1）；
/// - `xiyang`：体力未满且正面朝上回合结束前可翻面回 2（含取消分支）。
#[test]
fn ow_batch_a_roadhog_liangou_and_xiyang() {
    let (_verified, mut e) = start_formal_ow(2026_1134, 4);
    e.assign_general_named(0, "ow_luba").unwrap();
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e);
    e.debug_set_hp(0, 2);
    e.debug_set_hp(2, 4);

    // 1. 判定黑桃 → 链钩成功（距离 2 -> 1，首次杀伤 1 -> 2）
    e.debug_set_deck_top_by(|d| d.suit == Suit::Spade);
    let cost = place_any_suit(&mut e, 0, Suit::Club);
    let sha = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Diamond);
    activate(&mut e, 0, "liangou");
    assert_open_prompt(&e, "ow_liangou_card", 0);
    answer(&mut e, vec![ChoiceOption::Card(cost)]);
    assert_open_prompt(&e, "ow_liangou_target", 0);
    answer(&mut e, vec![ChoiceOption::Player(2)]);
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 2), 1);
    assert!(e.debug_has_skill(2, "liangou2"));
    play(&mut e, 0, sha, vec![2]);
    drain_cancel(&mut e);
    assert_eq!(e.debug_hp(2), 2, "链钩成功：0->2 无视原距离 2 命中且伤害+1 (4->2)");

    // 2. 回合结束前 `xiyang` 确认翻面回 2
    advance_to_turn_end_prompt(&mut e, 0, "ow_xiyang_confirm");
    answer(&mut e, vec![ChoiceOption::Confirm]);
    drain_cancel(&mut e);
    assert!(e.debug_face_down(0), "吸氧翻面");
    assert_eq!(e.debug_hp(0), 4, "吸氧恢复 2 点体力");
    assert_card_conservation(&e, "roadhog end");
}

/// 黑影 (`character/ow/skill.js:594-765`):
/// - `qinru`：用杀指定目标判定非红桃则封印目标手牌（`ow_qinru_fengyin` + 封非锁定技），且持续至目标自身回合结束；
/// - `yinshen`：手牌最后一张基本牌离开手牌后获得 `yinshen_active`（不可成为其他角色牌的目标），下回合开始清理；
/// - `maichong`：使用真实非延时锦囊对被入侵者随机弃 1 牌。
#[test]
fn ow_batch_a_sombra_qinru_yinshen_maichong() {
    let (_verified, mut e) = start_formal_ow(2026_1135, 4);
    e.assign_general_named(0, "ow_heiying").unwrap();
    e.assign_general_named(1, "ow_orisa").unwrap();
    e.assign_general_named(2, "ow_orisa").unwrap();
    e.debug_grant(0, "武圣");
    e.debug_grant(0, "咆哮");
    e.debug_grant(1, "咆哮");
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e);

    // 0 号只有唯一基本牌 sha0：打出后手牌无基本牌 → 触发 yinshen；同时 0->1 触发 qinru（梅花）
    e.debug_set_deck_top_by(|d| d.suit == Suit::Club);
    let sha0 = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    let guohe = place_kind_suit(&mut e, 0, CardKind::Dismantle, Suit::Spade);
    let red_wuzhong = place_kind_suit(&mut e, 0, CardKind::Wuzhong, Suit::Heart);
    let shan1b = place_kind_suit(&mut e, 1, CardKind::Shan, Suit::Heart);
    e.debug_set_hp(1, 4);

    play(&mut e, 0, sha0, vec![1]);
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if c.spec.prompt == "ow_qinru_confirm" {
            answer(&mut e, vec![ChoiceOption::Confirm]);
        } else if c.spec.prompt == "respond_shan" && c.observer == 1 {
            assert!(
                c.spec.options.contains(&ChoiceOption::Card(shan1b)),
                "ow_qinru_fengyin 不得封禁手牌闪响应"
            );
            answer(&mut e, vec![ChoiceOption::Card(shan1b)]);
        } else {
            drain_cancel(&mut e);
        }
    }
    assert!(e.state_core().player(0).yinshen_active, "最后基本牌离开手牌触发隐身");
    assert_eq!(flag(&e, 1, "ow_qinru_fengyin"), 1, "侵入判定非红桃封印 1 号非锁定技");
    assert_eq!(e.debug_hp(1), 4, "侵入不封响应，1 号成功出闪免伤");
    assert!(!e.debug_hand(1).contains(&shan1b), "1 号手中闪已打出");

    // `maichong` 过滤：以 `武圣` 将红桃 `wuzhong` 转化为【杀】对距离 1 的 1 号使用 → 不触发脉冲；再用真实过河拆桥 → 触发脉冲弃 1
    let extra1 = place_kind_suit(&mut e, 1, CardKind::Tao, Suit::Diamond);
    let _spade2 = place_any_suit(&mut e, 2, Suit::Spade);
    let before_conv = e.debug_hand(1).len();
    let out_va = play(&mut e, 0, red_wuzhong, vec![1]);
    assert!(
        matches!(out_va, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "武圣将红桃无中当【杀】使用须成功结算: {out_va:?}"
    );
    drain_cancel(&mut e);
    assert_eq!(
        e.debug_hand(1).len(),
        before_conv,
        "脉冲：经 viewAs 转化的锦囊牌不得触发 maichong"
    );
    play(&mut e, 0, guohe, vec![2]);
    drain_cancel(&mut e);
    assert!(
        e.debug_hand(1).len() < before_conv,
        "脉冲：使用真实过河拆桥后对被侵入者 1 号弃 1"
    );
    let _ = extra1;

    // 1 号回合内：非锁定技 `qianggu` 被封印拒绝；手牌【杀】与锁定技 `paoxiao` 仍可使用；到 1 号回合结束解除
    advance_to_play(&mut e, 1);
    clear_all_hands(&mut e);
    let _f1a = place_any_suit(&mut e, 1, Suit::Club);
    let _f1b = place_any_suit(&mut e, 1, Suit::Diamond);
    assert!(
        matches!(activate(&mut e, 1, "qianggu"), CommandOutcome::Rejected(_)),
        "侵入封印禁止 1 号发动非锁定技 qianggu"
    );
    let sha1a = place_kind_suit(&mut e, 1, CardKind::Sha, Suit::Heart);
    let sha1b = place_kind_suit(&mut e, 1, CardKind::Sha, Suit::Spade);
    assert!(
        matches!(
            play(&mut e, 1, sha1a, vec![2]),
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "侵入封印期间 1 号手牌【杀】仍可正常使用"
    );
    drain_cancel(&mut e);
    assert!(
        matches!(
            play(&mut e, 1, sha1b, vec![2]),
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "侵入封印不封锁定技咆哮，1 号第 2 张【杀】仍可用"
    );
    drain_cancel(&mut e);

    // 2 号启用 `qianggu2`，当 0 号对 2 号出杀被 `qianggu2` 取消时，`qinru` 不得对已取消目标触发
    advance_to_play(&mut e, 2);
    assert_eq!(flag(&e, 1, "ow_qinru_fengyin"), 0, "侵入封印在 1 号回合结束清除");
    clear_all_hands(&mut e);
    let qg_a = place_any_suit(&mut e, 2, Suit::Heart);
    let qg_b = place_any_suit(&mut e, 2, Suit::Club);
    let out_qg = activate(&mut e, 2, "qianggu");
    assert!(
        matches!(out_qg, CommandOutcome::Pending { .. }),
        "2 号发动 qianggu 失败: out={out_qg:?}, tieji_sealed={}, hand2={:?}, turn=({}, {:?})",
        e.catalog().is_tieji_sealed(e.state_core(), 2),
        e.debug_hand(2),
        e.turn().current,
        e.turn().phase
    );
    answer(&mut e, vec![ChoiceOption::Card(qg_a), ChoiceOption::Card(qg_b)]);
    assert!(e.debug_has_skill(2, "qianggu2"));

    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e);
    // 须用黑色武器（青龙偃月刀，黑桃射程 3）：0 号持「武圣」，红桃麒麟弓在空目标时会触发武圣转化逻辑
    let qinglong0 = e.debug_place(0, "qinglong");
    let out_eq = play(&mut e, 0, qinglong0, vec![]);
    assert!(
        matches!(out_eq, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "装备青龙刀失败: {out_eq:?}"
    );
    drain_cancel(&mut e);
    let sha_to_2 = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    let _basic0 = place_kind_suit(&mut e, 0, CardKind::Tao, Suit::Heart);
    play(&mut e, 0, sha_to_2, vec![2]);
    assert_open_prompt(&e, "ow_qianggu2_discard", 0);
    answer_cancel(&mut e);
    assert!(
        e.open_choice().is_none(),
        "强固2 取消目标后，qinru 不得继续对该已取消目标弹出询问"
    );
    assert_eq!(flag(&e, 2, "ow_qinru_fengyin"), 0);

    // 隐身校验：0 号持 `yinshen_active` 不能成为 2 号【杀】的目标
    clear_all_hands(&mut e);
    advance_to_play(&mut e, 2);
    e.state_core_mut().player_mut(0).yinshen_active = true;
    let sha2 = place_kind_suit(&mut e, 2, CardKind::Sha, Suit::Spade);
    let qilin = e.debug_place(2, "qilin");
    play(&mut e, 2, qilin, vec![]);
    drain_cancel(&mut e);
    let rej_hide = play(&mut e, 2, sha2, vec![0]);
    assert!(
        matches!(rej_hide, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "隐身状态不能被其他角色选为【杀】目标: {rej_hide:?}"
    );
    assert_card_conservation(&e, "sombra end");
    let _ = DamageNature::Normal;
}
