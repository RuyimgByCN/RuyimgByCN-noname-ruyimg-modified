//! U05-3 (`shaai-ow`) Round 2 审查 6 项等价缺陷真实事件闭环回归（反例先行，0 `debug_execute_effect` / `debug_resolve`）：
//! 1. `yihun2` 须阻断 `Nullify`（含 `reguhuo` `resp_ctx=3`）、杀响应 `Shan`（含 `reguhuo` `resp_ctx=1`）、
//!    `mbcmjiushi` 濒死自救与 `host_response_skills`（濒死），同时保留 `Nanman`/`Wanjian`/`Duel` `chooseToRespond`；
//! 2. `huoyu` (`chiyuxi`) 与 `dianji` (`jingleishan`) 须尊重 `targetEnabled`（跳过 `yihun2`/`sheng2`/`jidong2`/`yinshen`/`yinmi`/`jileng`）；
//! 3. `shanxian` 在下家 `face_down` 时仍于 `phaseBefore` 触发、于 `phaseAfter` 后翻面、且劫持目标途中阵亡时从 `next_alive(victim_b)` 续跑；
//! 4. `bfengshi`/`bfengshi3` 于 `useCard`（首张杀 `shaBegin`/`damageBegin` 前）即添加 `bfengshi4`；
//! 5. `lianpo` 为可选确认（可拒绝且拒绝后下轮仍可触发）；
//! 6. `feiren3` 保留火/雷属性及黑桃（含 `use_virtual_card` 回落）以触发 `feiren` +1 伤；`claim_off_deck_supply` 池扩充且走非摸牌 `gain2`（不发 `CardsDrawn`）。

use content_native_loader::{
    prepare_locked_content_with_bridge, start_locked_engine, LockedStartMode,
};
use engine_native::catalog::{CardKind, Suit};
use engine_native::event::DamageNature;
use engine_native::{ChoiceOption, CommandOutcome, GameAction, LogEvent, Phase};

use super::common::{
    activate, advance_one_turn_to_play, advance_to_play, advance_to_turn_end_prompt, answer,
    answer_cancel, assert_card_conservation, assert_open_prompt, clear_all_hands, drain_cancel,
    freeze_pkg, place_any_suit, place_kind, place_kind_suit, play, roundtrip_restore,
    start_formal_ow, submit,
};

/// 缺陷 1：`yihun2` 封禁 `cardEnabled`/`cardSavable` 族：
/// - 阻断 `Nullify`（真实无懈与 `reguhuo` `resp_ctx=3`）
/// - 阻断被【杀】时的【闪】与 `reguhuo` `resp_ctx=1`
/// - 阻断濒死 `mbcmjiushi`（`cardSavable`）与 `host_response_skills`
/// - 保留 `Nanman`/`Wanjian` 的 `chooseToRespond`（`cardRespondable` 不被 `yihun2` 阻断）
#[test]
fn r2_defect1_yihun2_blocks_nullify_sha_dodge_mbcmjiushi_and_host_resp() {
    let verified = prepare_locked_content_with_bridge(vec![
        freeze_pkg("shaai-ow-upstream-noname"),
        freeze_pkg("epic-cards-upstream-liuli"),
        freeze_pkg("longxue-upstream-liuli"),
    ])
    .expect("prepare ow + epic + longxue");
    let mut e = start_locked_engine(2026_1201, 4, LockedStartMode::Prototype, &verified)
        .expect("start engine");

    e.assign_general_named(0, "ow_sishen").unwrap();
    e.assign_general_named(3, "ow_orisa").unwrap();
    e.debug_grant(0, "mbcmjiushi");
    e.debug_grant(0, "reguhuo");

    // 3 号回合结束发动 `woliu`，赋予 0、1、3 号 `woliu2`（以便 1 号杀 3 号时将 0 号拉入 `Sha` 结算目标）
    advance_to_play(&mut e, 3);
    clear_all_hands(&mut e);
    advance_to_turn_end_prompt(&mut e, 3, "ow_woliu_targets");
    answer(&mut e, vec![ChoiceOption::Player(0), ChoiceOption::Player(1)]);
    assert!(e.debug_has_skill(0, "woliu2"));
    assert!(e.debug_has_skill(3, "woliu2"));

    // 0 号回合结束发动 `yihun`，弃黑牌指定 2 号获 `yihun2`
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e);
    let blk = place_any_suit(&mut e, 0, Suit::Club);
    advance_to_turn_end_prompt(&mut e, 0, "ow_yihun_card");
    answer(&mut e, vec![ChoiceOption::Card(blk)]);
    assert_open_prompt(&e, "ow_yihun_target", 0);
    answer(&mut e, vec![ChoiceOption::Player(2)]);
    drain_cancel(&mut e);
    assert!(e.debug_has_skill(0, "yihun2"));

    // 进入 1 号出牌阶段验证 yihun2 四条口径
    advance_to_play(&mut e, 1);
    clear_all_hands(&mut e);
    e.debug_set_hp(0, 4);
    e.debug_set_hp(3, 4);

    // 1a. `Wuzhong`：0 号手握真实【无懈】与【蛊惑】(`reguhuo`)，仍不得弹出 `nullify_window`
    let wx0 = place_kind_suit(&mut e, 0, CardKind::Nullify, Suit::Spade);
    let _gh_fodder = place_kind_suit(&mut e, 0, CardKind::Tao, Suit::Heart);
    let wz1 = place_kind_suit(&mut e, 1, CardKind::Wuzhong, Suit::Heart);
    let h1_before = e.debug_hand(1).len() - 1;
    let out_wz = play(&mut e, 1, wz1, vec![]);
    assert!(
        matches!(out_wz, CommandOutcome::Accepted { .. }) && e.open_choice().is_none(),
        "移魂2 下 0 号不得被询问无懈可击或蛊惑无懈: {out_wz:?}"
    );
    assert_eq!(e.debug_hand(1).len(), h1_before + 2, "无懈被移魂2 封禁，1 号直接摸 2 张");
    assert!(e.debug_hand(0).contains(&wx0));

    // 1b. `Sha` 被 `woliu2` 拉入 0 号：0 号手握【闪】与【蛊惑】，不得进入 `respond_shan` 闪避
    e.debug_clear_hand(0);
    let _range_wep1 = e.debug_place(1, "guanshi");
    let _ = play(&mut e, 1, _range_wep1, vec![]);
    let shan0 = place_kind_suit(&mut e, 0, CardKind::Shan, Suit::Diamond);
    let sha1 = place_kind_suit(&mut e, 1, CardKind::Sha, Suit::Heart);
    let hp0_before = e.debug_hp(0);
    let out_sha = play(&mut e, 1, sha1, vec![3]);
    assert!(
        matches!(out_sha, CommandOutcome::Accepted { .. }) && e.open_choice().is_none(),
        "移魂2 封禁出闪(`cardEnabled`)与蛊惑闪(`host_response_skills` resp_ctx=1): {out_sha:?}"
    );
    assert_eq!(e.debug_hp(0), hp0_before - 1, "0 号无法使用闪或蛊惑闪，直接受 1 点杀伤");
    assert!(e.debug_hand(0).contains(&shan0));

    // 1c. `chooseToRespond` (`cardRespondable` 不被 `yihun2` 阻断)：
    // - `Nanman` 按 `targetEnabled` 跳过 `yihun2` 目标 `0` 号；
    // - `ow_ana` `mianzhen` 询问 `0` 号打出【闪】(`response_kind_cards_core` / `chooseToRespond`) 时，仍允许 `0` 号打出【闪】免遭 `mianzhen2`！
    e.debug_clear_hand(0);
    let _resp_sha0 = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Club);
    let nm1 = place_kind(&mut e, 1, CardKind::SavageAttack);
    let hp0_nm = e.debug_hp(0);
    play(&mut e, 1, nm1, vec![]);
    drain_cancel(&mut e);
    assert_eq!(e.debug_hp(0), hp0_nm, "Nanman 须跳过 yihun2 目标 0 号");

    e.assign_general_named(1, "ow_ana").unwrap();
    let mz_cost = place_any_suit(&mut e, 1, Suit::Club);
    let resp_shan0 = place_kind_suit(&mut e, 0, CardKind::Shan, Suit::Heart);
    activate(&mut e, 1, "mianzhen");
    assert_open_prompt(&e, "ow_mianzhen_card", 1);
    answer(&mut e, vec![ChoiceOption::Card(mz_cost)]);
    assert_open_prompt(&e, "ow_mianzhen_target", 1);
    answer(&mut e, vec![ChoiceOption::Player(0)]);
    assert_open_prompt(&e, "ow_mianzhen_shan", 0);
    assert!(
        e.open_choice()
            .unwrap()
            .spec
            .options
            .contains(&ChoiceOption::Card(resp_shan0)),
        "移魂2 不阻 cardRespondable：0 号被麻醉针逼闪时仍可打出【闪】"
    );
    answer(&mut e, vec![ChoiceOption::Card(resp_shan0)]);
    assert!(!e.debug_has_skill(0, "mianzhen2"));

    // 1d. `mbcmjiushi` + `reguhuo` + `Tao` 濒死自救：`0` 号正面朝上持 `mbcmjiushi` + `reguhuo` + `Tao`，
    // 1 号以 `paoxiao`+`guanshi` 再对 `3` 号出杀借 `woliu2` 击倒 1 血 `0` 号进入濒死 -> `dying_save` 仅问 `1` 号、绝不可问 `0` 号！
    e.debug_clear_hand(0);
    let tao0 = place_kind_suit(&mut e, 0, CardKind::Tao, Suit::Heart);
    e.debug_set_hp(0, 1);
    e.debug_set_hp(3, 2);
    e.debug_clear_hand(3);
    e.debug_clear_hand(2);
    e.debug_grant(1, "咆哮");
    let _tao1 = place_kind_suit(&mut e, 1, CardKind::Tao, Suit::Diamond);
    let sha1b = place_kind_suit(&mut e, 1, CardKind::Sha, Suit::Club);
    let out_sha1b = play(&mut e, 1, sha1b, vec![3]);
    let mut saw_0_dying_save = false;
    let mut saw_1_dying_save = false;
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if c.spec.prompt == "dying_save" {
            if c.observer == 0 {
                saw_0_dying_save = true;
            } else if c.observer == 1 {
                saw_1_dying_save = true;
            }
        }
        answer_cancel(&mut e);
    }
    assert!(
        saw_1_dying_save && !saw_0_dying_save,
        "移魂2 须封禁 0 号自身的酒诗/桃/蛊惑濒死自救 (saw_0={saw_0_dying_save}, saw_1={saw_1_dying_save}, hp0={}, hp3={}, out={out_sha1b:?})",
        e.debug_hp(0),
        e.debug_hp(3)
    );
    assert!(e.debug_hand(0).contains(&tao0) || !e.debug_alive(0));
    assert_card_conservation(&e, "defect1 yihun2 end");
}

/// 缺陷 2：`huoyu` (`chiyuxi`) 与 `dianji` (`jingleishan`) 须遵循 `targetEnabled` 过滤。
#[test]
fn r2_defect2_huoyu_and_dianji_respect_target_enabled() {
    let (_verified, mut e) = start_formal_ow(2026_1202, 4);
    e.assign_general_named(0, "ow_falaozhiying").unwrap(); // huoyu
    e.assign_general_named(1, "ow_wensidun").unwrap(); // dianji
    e.assign_general_named(2, "ow_sishen").unwrap(); // yihun -> yihun2
    e.assign_general_named(3, "ow_mei").unwrap(); // jidong -> jidong2

    // 1. 2 号对 `1` 号挂 `yihun2` 并同时用 `ow_dulei` 在 2 号身上做 `CardUsed.targets` 探针?
    // 更好：直接让 2 号获 `yihun2`（目标 3，故经 3 后才解，0/1 回合均保留？不：2->3 先经过 3，所以设 target = 1 则 0 号回合时仍在！）
    // 让 3 号获 `jidong_active`（经过 3 号回合 end 发动急冻！）；且 2 号先由 `ow_heiying` 或自己拿黑桃对 `1` 挂 `yihun2`！
    advance_to_play(&mut e, 2);
    clear_all_hands(&mut e);
    let blk2 = place_any_suit(&mut e, 2, Suit::Spade);
    advance_to_turn_end_prompt(&mut e, 2, "ow_yihun_card");
    answer(&mut e, vec![ChoiceOption::Card(blk2)]);
    assert_open_prompt(&e, "ow_yihun_target", 2);
    answer(&mut e, vec![ChoiceOption::Player(1)]);
    assert!(e.debug_has_skill(2, "yihun2"));

    // 3 号回合 end 发动 `jidong`（`hp==1` -> 回 2 且 `jidong_active == true` + 翻面！）
    advance_to_play(&mut e, 3);
    clear_all_hands(&mut e);
    e.debug_set_hp(3, 1);
    advance_to_turn_end_prompt(&mut e, 3, "ow_jidong_confirm");
    answer(&mut e, vec![ChoiceOption::Confirm]);
    assert!(e.state_core().player(3).jidong_active);

    // 2. 回到 0 号（法老之鹰）发动 `huoyu`：此时 2 号有 `yihun2`、3 号有 `jidong_active`，只允许命中 1 号！
    // 若 `huoyu` 未按 `targetEnabled` 过滤 `targets`，则 `CardUsed` 会把 `2` 号（且 `AoeResolveMachine` 直接命中 `3` 号 `jidong_active`！）
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e);
    // 同测：若所有其他存活角色均被 `yihun2`/`jidong_active` 阻断，则 `huoyu` 应拒绝发动（不白弃牌）！
    e.debug_grant(1, "sheng2");
    assert!(
        matches!(activate(&mut e, 0, "huoyu"), CommandOutcome::Rejected(_)),
        "全体其他角色皆不可为目标时 huoyu 不可发动"
    );
    let sheng2_def = e.catalog().skill_by_name("sheng2").unwrap();
    e.state_core_mut().skills.remove(&(1, sheng2_def));

    let r1 = place_kind_suit(&mut e, 0, CardKind::Tao, Suit::Heart);
    let r2 = place_kind_suit(&mut e, 0, CardKind::Shan, Suit::Diamond);
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);
    e.debug_set_hp(3, 4);
    activate(&mut e, 0, "huoyu");
    assert_open_prompt(&e, "ow_huoyu_cards", 0);
    answer(&mut e, vec![ChoiceOption::Card(r1), ChoiceOption::Card(r2)]);
    drain_cancel(&mut e);
    assert_eq!(e.debug_hp(1), 2, "huoyu 两发炽羽袭应命中合法目标 1 号 (4->2)");
    assert_eq!(e.debug_hp(2), 4, "huoyu 须跳过 yihun2 目标 2 号");
    assert_eq!(e.debug_hp(3), 4, "huoyu 须跳过 jidong2 目标 3 号");

    // 3. 在 1 号（温斯顿）回合测 `dianji`（距离 <= 2）：
    // 先把 0 换 `ow_sishen` 且对 2 挂 `yihun2`，2 清白将 (`default`)，3 持 `yinshen_active`；
    // 然后先为 2 临时授 `sheng2` 验证全阻断，再撤 `sheng2` 验证 `dianji` 只命中 2 号！
    e.assign_general_named(0, "ow_sishen").unwrap();
    e.assign_general_named(2, "ow_luba").unwrap();
    let yihun2_def = e.catalog().skill_by_name("yihun2").unwrap();
    e.state_core_mut().skills.remove(&(2, yihun2_def));
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e);
    let blk0 = place_any_suit(&mut e, 0, Suit::Club);
    advance_to_turn_end_prompt(&mut e, 0, "ow_yihun_card");
    answer(&mut e, vec![ChoiceOption::Card(blk0)]);
    assert_open_prompt(&e, "ow_yihun_target", 0);
    answer(&mut e, vec![ChoiceOption::Player(2)]);
    assert!(e.debug_has_skill(0, "yihun2"));

    // 在 1 号（温斯顿）出牌前先借 `0` 与 `2` 测 `yihun2` 与 `yinshen_active` + `sheng2`
    advance_to_play(&mut e, 1);
    clear_all_hands(&mut e);
    e.debug_grant(2, "sheng2");
    e.state_core_mut().players[3].yinshen_active = true;
    // 现在对于 1 号（dist<=2 的其他座 = 0, 2, 3）：0 有 `yihun2`，2 有 `sheng2`，3 有 `yinshen_active` -> 全不可指定！
    let fuel_reject = place_kind_suit(&mut e, 1, CardKind::Sha, Suit::Heart);
    let _ = fuel_reject;
    assert!(
        matches!(activate(&mut e, 1, "dianji"), CommandOutcome::Rejected(_)),
        "全体 dist<=2 目标皆被 targetEnabled 排除时 dianji 不可发动"
    );
    // 解除 2 号 `sheng2`（留 0 号 `yihun2` 与 3 号 `yinshen_active`），再测 `dianji` 只命中 2 号！
    e.state_core_mut().skills.remove(&(2, sheng2_def));
    e.debug_set_hp(0, 4);
    e.debug_set_hp(2, 4);
    e.debug_set_hp(3, 4);
    let act_ok = activate(&mut e, 1, "dianji");
    assert!(
        matches!(act_ok, CommandOutcome::Pending { .. }),
        "dianji should activate cleanly when seat 2 is valid: {act_ok:?}"
    );
    assert_open_prompt(&e, "ow_dianji_card", 1);
    answer(&mut e, vec![ChoiceOption::Card(fuel_reject)]);
    drain_cancel(&mut e);
    assert_eq!(e.debug_hp(2), 3, "dianji 应命中合法目标 2 号 (4->3)");
    assert_eq!(e.debug_hp(0), 4, "dianji 须跳过 yihun2 目标 0 号");
    assert_eq!(e.debug_hp(3), 4, "dianji 须跳过 yinshen2 目标 3 号");
    assert_card_conservation(&e, "defect2 targets end");
}

/// 缺陷 3：`shanxian` 触发（下家 `face_down`）、翻面时机（`phaseAfter` 之后）、及途中击杀 `victim_b` 后从 `next_alive(victim_b)` 续跑。
#[test]
fn r2_defect3_shanxian_facedown_trigger_turnover_after_phase_and_dead_resume() {
    // Part (a)+(b): `1` 号处于背面朝上 (`face_down`)，轮到 `1` 号开始时 0 号 `ow_liekong` 仍可在 `phaseBefore` 触发 `shanxian`；
    // 且在 0 号劫持回合自己的 `TurnPhaseEnd` 时 `0` 仍未翻面，直到 `phaseAfter` 完成劫持后才翻面并由 1 号翻回跳过！
    let (verified, mut e) = start_formal_ow(2026_1203, 4);
    e.assign_general_named(0, "ow_liekong").unwrap();
    e.assign_general_named(3, "ow_mei").unwrap();

    // 3 号用 `baoxue` 将 1 号翻面（再在 3 号下个回合？不：在 3 号 Play 阶段对 1 号使用暴雪使 1 号背面朝上）
    advance_to_play(&mut e, 3);
    clear_all_hands(&mut e);
    let b3 = place_any_suit(&mut e, 3, Suit::Spade);
    let _c1 = place_kind_suit(&mut e, 1, CardKind::Tao, Suit::Heart);
    let _ = b3;
    activate(&mut e, 3, "baoxue");
    assert_open_prompt(&e, "ow_baoxue_targets", 3);
    answer(&mut e, vec![ChoiceOption::Player(1)]);
    drain_cancel(&mut e);
    assert!(e.debug_face_down(1), "1 号已被暴雪翻至背面");

    // 推进至 0 号 Play，再推进离开 0 号回合交接给背面朝上的 1 号：
    // 在 1 号翻回跳过之前，0 号（`ow_liekong`）必须在 `phaseBefore` 弹出 `ow_shanxian_confirm`！
    advance_to_play(&mut e, 0);
    let mut saw_sx_confirm = false;
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    for _ in 0..24 {
        if let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt == "ow_shanxian_confirm" {
                saw_sx_confirm = true;
                break;
            }
            answer_cancel(&mut e);
            continue;
        }
        if e.turn().current != 0 {
            break;
        }
        let _ = submit(&mut e, 0, GameAction::NextPhase);
    }
    assert!(
        saw_sx_confirm,
        "1 号背面朝上时仍须于 phaseBefore 触发猎空闪现询问"
    );
    assert!(
        e.debug_face_down(1),
        "在 ow_shanxian_confirm 弹出瞬间 1 号仍须处于背面朝上（phaseBefore 先于 turnOver 跳过）"
    );
    e = roundtrip_restore(&e, &verified);
    assert_open_prompt(&e, "ow_shanxian_confirm", 0);
    answer(&mut e, vec![ChoiceOption::Confirm]);

    // 0 号进入劫持回合，检查进入 Play 且跳过 Discard 至 `TurnEndWindow` (`TurnPhaseEnd`) 前 `0` 号尚未翻面
    advance_to_play(&mut e, 0);
    assert_eq!(e.turn().current, 0);
    assert!(e.debug_face_down(1), "劫持期间原受害者 1 号仍保持翻面");
    assert!(!e.debug_face_down(0), "劫持 Play 阶段 0 号正面朝上");

    // 推进 0 号从 Play -> Discard -> End -> TurnPhaseEnd（此时若有结束阶段观察或触发，0 仍为正面）
    let mut saw_turn_after = false;
    let mut turned_during_play_end = false;
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard
    drain_cancel(&mut e);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End -> TurnEndWindow
    for _ in 0..16 {
        if let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt == "ow_shanxian_confirm" {
                break;
            }
            answer_cancel(&mut e);
            continue;
        }
        // 在尚未推进完 0 号整个结束窗时检测
        if e.turn().current == 0 && e.debug_face_down(0) {
            turned_during_play_end = true;
        }
        if e.turn().current != 0 {
            saw_turn_after = true;
            break;
        }
        let cur = e.turn().current;
        let _ = submit(&mut e, cur, GameAction::NextPhase);
    }
    assert!(!turned_during_play_end, "不得在 TurnPhaseEnd(after=false) 提前翻面");
    assert!(saw_turn_after || e.debug_face_down(0));
    assert!(e.debug_face_down(0), "shanxian phaseAfter 结束后猎空翻面");
    assert!(!e.debug_face_down(1), "随后 1 号翻回正面并跳过其回合");
    assert_eq!(e.turn().current, 2, "1 号跳过后交接至 2 号");
    assert_card_conservation(&e, "defect3 ab end");

    // Part (c): `ow_liekong` 在座 2 劫持座 0 (`victim_b`) 回合，途中击杀 `0`，
    // 结束后必须从 `next_alive(0) = 1` 续跑（而非 `next_alive(2) = 3`）！
    let (_v2, mut e2) = start_formal_ow(2026_1204, 4);
    e2.assign_general_named(2, "ow_liekong").unwrap();
    advance_to_play(&mut e2, 3);
    clear_all_hands(&mut e2);
    // 3 -> 0 时 2 号 `ow_liekong` 触发 `shanxian` 劫持 0 号
    let _ = submit(&mut e2, 3, GameAction::NextPhase);
    for _ in 0..16 {
        if let Some(c) = e2.open_choice().cloned() {
            if c.spec.prompt == "ow_shanxian_confirm" {
                answer(&mut e2, vec![ChoiceOption::Confirm]);
                break;
            }
            drain_cancel(&mut e2);
            continue;
        }
        let cur = e2.turn().current;
        let _ = submit(&mut e2, cur, GameAction::NextPhase);
    }
    // 2 号在劫持 0 号的回合中击杀 0 号
    advance_to_play(&mut e2, 2);
    e2.debug_clear_hand(0);
    e2.debug_set_hp(0, 1);
    let sha2 = place_kind_suit(&mut e2, 2, CardKind::Sha, Suit::Spade);
    play(&mut e2, 2, sha2, vec![0]);
    drain_cancel(&mut e2);
    assert!(!e2.debug_alive(0), "被劫持对象 0 号阵亡");

    // 结束 2 号劫持回合 -> 必须交给 `next_alive(victim_b=0) == 1`，而不是 `next_alive(prev=2) == 3`！
    let _ = submit(&mut e2, 2, GameAction::NextPhase);
    drain_cancel(&mut e2);
    for _ in 0..16 {
        if e2.turn().current != 2 && e2.turn().phase == Phase::Play && e2.open_choice().is_none() {
            break;
        }
        if e2.open_choice().is_some() {
            drain_cancel(&mut e2);
            continue;
        }
        let cur = e2.turn().current;
        let _ = submit(&mut e2, cur, GameAction::NextPhase);
    }
    assert_eq!(
        e2.turn().current,
        1,
        "被劫持目标 0 号途中阵亡，shanxian 结束后须从 next_alive(victim_b=0) = 1 续跑，而非 next_alive(2) = 3"
    );
    assert_card_conservation(&e2, "defect3 c end");
}

/// 缺陷 4：`bfengshi`/`bfengshi3` 须在 `useCard` 时即授予 `bfengshi4`（先于同一【杀】的 `shaBegin`/`damageBegin`）。
#[test]
fn r2_defect4_bfengshi_adds_bfengshi4_at_use_card_before_sha_begin() {
    let (_verified, mut e) = start_formal_ow(2026_1205, 4);
    e.assign_general_named(0, "ow_banzang").unwrap();
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e);
    e.debug_set_hp(1, 4);

    // 先使用 5 张非杀牌（如无中生有），使 `cards_used_turn == 5`（概率 = 20% * 5 = 100%）
    for _ in 0..5 {
        let wz = place_kind_suit(&mut e, 0, CardKind::Wuzhong, Suit::Heart);
        play(&mut e, 0, wz, vec![]);
        drain_cancel(&mut e);
    }
    clear_all_hands(&mut e);
    assert!(!e.debug_has_skill(0, "bfengshi4"), "未用杀前尚无 bfengshi4");

    // 1 号持【闪】；0 号出第一张【杀】：`bfengshi3` 必须在 `useCard` 瞬间先挂 `bfengshi4`，
    // 使 `respond_shan` 正常打开（不被 `bfengshi` 直击），且 1 号不闪也仅受 1 点伤（不被 `bfengshi2` 加伤）！
    let shan1 = place_kind_suit(&mut e, 1, CardKind::Shan, Suit::Diamond);
    let sha0 = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    let out = play(&mut e, 0, sha0, vec![1]);
    assert!(matches!(out, CommandOutcome::Pending { .. }));
    assert!(
        e.debug_has_skill(0, "bfengshi4"),
        "bfengshi3 须在 useCard 时点即加入 bfengshi4（shaBegin 之前）"
    );
    assert_open_prompt(&e, "respond_shan", 1);
    assert!(e.open_choice().unwrap().spec.options.contains(&ChoiceOption::Card(shan1)));
    answer_cancel(&mut e);
    assert_eq!(
        e.debug_hp(1),
        3,
        "因 shaBegin/damageBegin 前已有 bfengshi4，bfengshi/bfengshi2 不触发加伤，仅掉 1 血"
    );
    assert_card_conservation(&e, "defect4 bfengshi end");
}

/// 缺陷 5：`lianpo` 为可选触发（`frequent: true`）：可拒绝且不插额外回合，拒绝后下一回合有击杀仍可再触发。
#[test]
fn r2_defect5_lianpo_optional_decline_and_retrigger() {
    let (verified, mut e) = start_formal_ow(2026_1206, 4);
    e.assign_general_named(0, "ow_yuanshi").unwrap();
    e.debug_grant(0, "feiren3");
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e);

    // 0 号击杀 1 号
    e.debug_set_hp(1, 1);
    let sha0 = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Heart);
    play(&mut e, 0, sha0, vec![1]);
    drain_cancel(&mut e);
    assert!(!e.debug_alive(1));

    // 推进出 0 号回合结束窗：须弹出 `ow_lianpo_confirm` 可选询问（可 Save/Restore + 可取消）
    let mut saw_lianpo = false;
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    for _ in 0..16 {
        if let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt == "ow_lianpo_confirm" {
                saw_lianpo = true;
                break;
            }
            answer_cancel(&mut e);
            continue;
        }
        let cur = e.turn().current;
        if cur != 0 {
            break;
        }
        let _ = submit(&mut e, cur, GameAction::NextPhase);
    }
    assert!(saw_lianpo, "lianpo 必须作为可选询问弹出而非强制执行");
    e = roundtrip_restore(&e, &verified);
    assert_open_prompt(&e, "ow_lianpo_confirm", 0);
    answer_cancel(&mut e);

    // 拒绝连破后不得插入 0 号额外回合，而是正常交接给存活下家 2 号
    advance_to_play(&mut e, 2);
    assert_eq!(e.turn().current, 2, "拒绝 lianpo 后正常交给 2 号");

    // 2 号回合内击杀 2 号自身（决斗 0 号由 0 号反杀 1 血 2 号），验证拒绝过一次的 lianpo 仍可再次触发并接受获额外回合！
    clear_all_hands(&mut e);
    e.debug_set_hp(2, 1);
    let sha0b = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    let duel2 = place_kind(&mut e, 2, CardKind::Duel);
    play(&mut e, 2, duel2, vec![0]);
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if c.spec.prompt == "duel_sha" && c.observer == 0 {
            answer(&mut e, vec![ChoiceOption::Card(sha0b)]);
            continue;
        }
        answer_cancel(&mut e);
    }
    assert!(!e.debug_alive(2), "2 号应被 0 号决斗反杀阵亡");
    let _ = submit(&mut e, 2, GameAction::NextPhase);
    let mut confirmed_second_lianpo = false;
    for _ in 0..24 {
        if let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt == "ow_lianpo_confirm" {
                answer(&mut e, vec![ChoiceOption::Confirm]);
                confirmed_second_lianpo = true;
                continue;
            }
            answer_cancel(&mut e);
            continue;
        }
        if e.turn().current == 0 && e.turn().phase == Phase::Play && confirmed_second_lianpo {
            break;
        }
        let cur = e.turn().current;
        let _ = submit(&mut e, cur, GameAction::NextPhase);
    }
    assert!(confirmed_second_lianpo, "之前拒绝过 lianpo 后仍可再次触发");
    assert_eq!(e.turn().current, 0, "确认 lianpo 后进入 0 号额外回合");
    assert_card_conservation(&e, "defect5 lianpo end");
}

/// 缺陷 6：
/// (a) `feiren3` 保留属性（火/雷）且回落虚拟杀仍带 `Suit::Spade` 令 `feiren` 加伤 +1；
/// (b) `claim_off_deck_supply` 打破 7/4/12 池上限且不触发 `CardsDrawn`（走非摸牌 `gain2`）。
#[test]
fn r2_defect6_feiren3_nature_spade_and_off_deck_supply_pool_gain2() {
    let (_verified, mut e) = start_formal_ow(2026_1207, 4);
    e.assign_general_named(0, "ow_yuanshi").unwrap();
    e.debug_grant(0, "feiren3");
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e);
    e.debug_set_hp(1, 5);
    e.debug_set_hp(2, 5);

    // 6a-1：使用黑桃【雷杀】，触发 `feiren3` 再发一次：须保留雷属性 + 黑桃 `feiren` +1 -> 2 + 2 = 4 伤！
    //（通过置 `1` 号横置并将 `3` 号同样横置，验证第二次飞刃杀同样带雷属性可传导铁索伤害！）
    let chain1 = place_kind(&mut e, 0, CardKind::Chain);
    play(&mut e, 0, chain1, vec![1, 3]);
    drain_cancel(&mut e);
    assert!(e.state_core().player(1).linked && e.state_core().player(3).linked);
    e.debug_set_hp(1, 5);
    e.debug_set_hp(3, 5);
    let spade_leisha = place_kind_suit(&mut e, 0, CardKind::LeiSha, Suit::Spade);
    play(&mut e, 0, spade_leisha, vec![1]);
    drain_cancel(&mut e); // 第 1 次雷杀解铁索传伤至 3 号（各 -2），随后弹出 `ow_feiren3_confirm`? Note: drain_cancel cancels feiren3! Instead handle explicitly:
    assert_card_conservation(&e, "after chain clear");

    // 精确手控回答 `ow_feiren3_confirm` 并检查 `feiren3` 两次均带 `Thunder` + `Spade` (+1 dmg)
    e.debug_set_hp(1, 5);
    e.debug_set_hp(3, 5);
    let zhuge = e.debug_place(0, "zhuge");
    play(&mut e, 0, zhuge, vec![]);
    // 给 1 与 3 重新横置，且仅验证第 2 张（由 `feiren3` 打出）也会触发铁索传导（证明 `feiren3` 保留 Thunder 属性）！
    let spade_leisha2 = place_kind_suit(&mut e, 0, CardKind::LeiSha, Suit::Spade);
    play(&mut e, 0, spade_leisha2, vec![1]);
    // 第 1 发命中后应弹出 `ow_feiren3_confirm`
    assert_open_prompt(&e, "ow_feiren3_confirm", 0);
    assert_eq!(e.debug_hp(1), 3, "原黑桃雷杀经飞刃造成 2 点雷伤 (5->3)");
    // 在回答 `ow_feiren3_confirm` 之前手动为 1 和 3 置铁索，验证 `feiren3` 发出的第 2 发本身是 Thunder（会传导给 3 号）而非 Normal！
    let chain2 = place_kind(&mut e, 0, CardKind::Chain);
    let _ = chain2; // 无法在中途插入玩牌，改为在 1 号 `CardUsed` 结算后直接设 core linked 状态或开局第 2 发测 fallback!
    e.state_core_mut().players[1].linked = true;
    e.state_core_mut().players[3].linked = true;
    answer(&mut e, vec![ChoiceOption::Confirm]);
    drain_cancel(&mut e);
    assert_eq!(
        e.debug_hp(1),
        1,
        "feiren3 复制发出的第 2 张黑桃雷杀同样带黑桃(+1)且为 2 伤 (3->1)"
    );
    assert_eq!(
        e.debug_hp(3),
        3,
        "feiren3 复制雷杀须保留 Thunder 属性从而通过铁索传导 2 点伤害给 3 号 (5->3)"
    );

    // 6a-2：抽干所有 `ow_sha_*` 离线/弃牌补给后，令牌堆顶非黑桃，再出黑桃雷杀走 `use_virtual_card` 回落分支：
    // 仍须保留黑桃(+1)与 Thunder 属性！
    let drained_ow_shas: Vec<u32> = e
        .state_core()
        .detached_cards
        .iter()
        .filter(|(c, _)| e.catalog().cards[*c as usize].name.starts_with("ow_sha_"))
        .map(|(c, _)| *c)
        .collect();
    for cid in drained_ow_shas {
        e.state_core_mut().detached_cards.retain(|(c, _)| *c != cid);
        e.state_core_mut().players[2].hand.push(cid);
    }
    let disc_ow_shas: Vec<u32> = e
        .state_core()
        .discard
        .iter()
        .copied()
        .filter(|c| e.catalog().cards[*c as usize].name.starts_with("ow_sha_"))
        .collect();
    for cid in disc_ow_shas {
        e.state_core_mut().discard.retain(|&c| c != cid);
        e.state_core_mut().players[2].hand.push(cid);
    }
    // 把牌堆首张置为红桃，防止恰好从 deck.first() 撞到黑桃
    if let Some(pos) = e
        .state_core()
        .deck
        .iter()
        .position(|c| e.catalog().cards[*c as usize].suit != Suit::Spade)
    {
        let non_sp = e.state_core_mut().deck.remove(pos);
        e.state_core_mut().deck.insert(0, non_sp);
    }
    e.debug_set_hp(1, 5);
    e.debug_set_hp(3, 5);
    let sp_thunder = place_kind_suit(&mut e, 0, CardKind::LeiSha, Suit::Spade);
    // 手动设置一张之前遗留或标准黑桃雷杀? Note: `cards.rs` only has 1 Spade LeiSha (`spade_leisha`), so let's recycle it from discard into hand via turn/play or re-place from discard!
    let _ = sp_thunder;
    // Actually `place_kind_suit` ALREADY picks from discard if not in deck (`common.rs:256`)!
    play(&mut e, 0, sp_thunder, vec![1]);
    assert_open_prompt(&e, "ow_feiren3_confirm", 0);
    assert_eq!(e.debug_hp(1), 3);
    e.state_core_mut().players[1].linked = true;
    e.state_core_mut().players[3].linked = true;
    answer(&mut e, vec![ChoiceOption::Confirm]);
    drain_cancel(&mut e);
    assert_eq!(
        e.debug_hp(1),
        1,
        "feiren3 use_virtual_card 回落仍须带 Suit::Spade 触发 feiren +1 伤 (2 伤)"
    );
    assert_eq!(
        e.debug_hp(3),
        3,
        "feiren3 use_virtual_card 回落仍须带 Thunder 属性传导铁索 2 伤"
    );
    assert_card_conservation(&e, "defect6a end");

    // 6b：验证池扩充 (>7 hslingjian, >4 zhiliaobo, >12 ow_sha) 与非摸牌 `gain2`（零 CardsDrawn，不触发连营）
    let (_v2, mut e2) = start_formal_ow(2026_1208, 4);
    e2.assign_general_named(0, "ow_tuobiang").unwrap();
    e2.assign_general_named(1, "ow_sishen").unwrap();
    e2.assign_general_named(2, "ow_maikelei").unwrap();
    e2.debug_grant(0, "连营");
    advance_to_play(&mut e2, 0);
    clear_all_hands(&mut e2);

    // 1) 托比昂连续发动 10 次 `maoding`（> 7 张上限），且每次不触发 CardsDrawn 或连营
    for _ in 0..10 {
        e2.debug_set_hp(0, 4);
        let out = e2.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
        assert!(matches!(out, CommandOutcome::Pending { .. }));
        assert_open_prompt(&e2, "ow_maoding_confirm", 0);
        let pre_hand = e2.debug_hand(0).len();
        let log_before = e2.log().len();
        let _res = answer(&mut e2, vec![ChoiceOption::Confirm]);
        assert_eq!(
            e2.debug_hand(0).len(),
            pre_hand + 1,
            "maoding 第 1..=10 次都必须成功获得 ow_hslingjian（不得在 7 张后枯竭）"
        );
        assert!(
            !e2.log()[log_before..]
                .iter()
                .any(|ev| matches!(ev, LogEvent::CardsDrawn { .. })),
            "maoding 走 gain2 而非摸牌，不得发出 LogEvent::CardsDrawn"
        );
        assert!(e2.open_choice().is_none(), "自得不得触发连营等额外弹窗");
    }

    // 2) 麦克雷 `tiandan` x4 次（每次至多 4 张）+ 斩龙 3 张（> 12 张 `ow_sha_*` 上限），且零 CardsDrawn
    advance_to_play(&mut e2, 1);
    for _round in 0..4 {
        e2.debug_clear_hand(2);
        e2.debug_set_hp(2, 4);
        advance_one_turn_to_play(&mut e2, 3);
    }
    assert_card_conservation(&e2, "defect6b mid");

    // 用显式循环测试连续 4 次 tiandan（全程不进弃牌堆，真正在场上保留 16 张 ow_sha > 12 上限），且零 CardsDrawn
    let (_v3, mut e3) = start_formal_ow(2026_1209, 4);
    e3.assign_general_named(2, "ow_maikelei").unwrap();
    advance_to_play(&mut e3, 1);
    clear_all_hands(&mut e3);
    let mut total_tiandan_cards = 0usize;
    for _turn in 0..4 {
        let moved: Vec<u32> = e3.state_core_mut().players[2].hand.drain(..).collect();
        e3.state_core_mut().players[0].hand.extend(moved);
        e3.debug_set_hp(2, 4);
        let mut gained_this_turn = 0usize;
        for _ in 0..40 {
            if let Some(c) = e3.open_choice().cloned() {
                if c.spec.prompt == "ow_tiandan_confirm" {
                    let log_before = e3.log().len();
                    let _out = answer(&mut e3, vec![ChoiceOption::Confirm]);
                    // 注：`tiandan` 在 DrawPhaseBefore 执行 `gain2`，随后原摸牌阶段仍正常摸 2 张，
                    // 故此处核验：所获 `ow_sha_*` 均通过 `CardsTaken` 发放，零 `ow_sha_*` 出现在 `CardsDrawn` 中。
                    assert!(
                        e3.log()[log_before..].iter().any(|ev| matches!(
                            ev,
                            LogEvent::CardsTaken { to: 2, cards, .. }
                                if cards.iter().any(|cid| e3.catalog().cards[*cid as usize].name.starts_with("ow_sha_"))
                        )),
                        "tiandan gain2 须通过 CardsTaken 发放 ow_sha_*"
                    );
                    assert!(
                        !e3.log()[log_before..].iter().any(|ev| matches!(
                            ev,
                            LogEvent::CardsDrawn { cards, .. }
                                if cards.iter().any(|cid| e3.catalog().cards[*cid as usize].name.starts_with("ow_sha_"))
                        )),
                        "tiandan gain2 发放的 ow_sha_* 不得走 CardsDrawn"
                    );
                    gained_this_turn = e3
                        .debug_hand(2)
                        .iter()
                        .filter(|cid| e3.catalog().cards[**cid as usize].name.starts_with("ow_sha_"))
                        .count();
                    continue;
                }
                drain_cancel(&mut e3);
                continue;
            }
            if gained_this_turn > 0 && e3.turn().current == 3 && e3.turn().phase == Phase::Play {
                break;
            }
            let cur = e3.turn().current;
            let _ = submit(&mut e3, cur, GameAction::NextPhase);
        }
        assert_eq!(gained_this_turn, 4, "每次 tiandan 须稳获 4 张 ow_sha");
        total_tiandan_cards += gained_this_turn;
        advance_to_play(&mut e3, 1);
    }
    assert_eq!(total_tiandan_cards, 16);
    assert_card_conservation(&e3, "defect6b tiandan end");

    // 6 人局死神 `shouge` 连续击杀 5 人（> 4 张上限）均获 `ow_zhiliaobo_` 且零 CardsDrawn
    let (_v4, mut e4) = start_formal_ow(2026_1210, 6);
    e4.assign_general_named(0, "ow_sishen").unwrap();
    advance_to_play(&mut e4, 0);
    clear_all_hands(&mut e4);
    for victim in 1..6u8 {
        e4.debug_set_hp(victim, 1);
        let before = e4.debug_hand(0).len();
        let out = e4.debug_deal_damage(Some(0), victim, 1, DamageNature::Normal);
        assert!(
            matches!(out, CommandOutcome::Accepted { .. }) && e4.open_choice().is_none(),
            "victim {victim} dying should settle cleanly: {out:?}"
        );
        assert_eq!(
            e4.debug_hand(0).len(),
            before + 1,
            "shouge 第 {victim} 杀须继续获得 ow_zhiliaobo_（突破 4 张池上限）"
        );
    }
    assert_eq!(
        e4.debug_hand(0)
            .iter()
            .filter(|cid| e4.catalog().cards[**cid as usize].name.starts_with("ow_zhiliaobo_"))
            .count(),
        5
    );
    assert_card_conservation(&e4, "defect6b shouge end");
}
