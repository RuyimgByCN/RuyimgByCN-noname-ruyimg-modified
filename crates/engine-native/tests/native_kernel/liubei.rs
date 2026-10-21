use super::*;

/// 仁德（刘备）：给 2 张 → 回复 1 点体力；目标收到牌；空应答 = 放弃。
#[test]
fn liubei_rende_gives_cards_and_heals() {
    let mut e = Engine::new(951, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "仁德");
    e.debug_set_hp(0, 3);
    let c1 = e.debug_place(0, "shan");
    let c2 = e.debug_place(0, "shan");
    let _c3 = e.debug_place(0, "tao");
    let skill = skill_def_id(&e, "仁德");
    let out = submit_cmd(&mut e, 0, GameAction::ActivateSkill { skill });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("rende_cards 应挂起");
    assert_eq!(c.spec.prompt, "rende_cards");
    answer_sel(&mut e, vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)]);
    let t = e.open_choice().expect("rende_target 应挂起");
    assert_eq!(t.spec.prompt, "rende_target");
    answer_sel(&mut e, vec![ChoiceOption::Player(1)]);
    assert_eq!(e.debug_hp(0), 4, "给出 ≥2 张应回复 1 点体力");
    assert_eq!(e.debug_hand(1).len(), 2, "目标应收到的两张牌");
    assert_eq!(e.debug_hand(0).len(), 1, "刘备剩一张");
    // 出牌阶段可再次发动（上游无每回合一次限制）；累计已过 2，
    // 再给 1 张不再回血
    let out2 = submit_cmd(&mut e, 0, GameAction::ActivateSkill { skill });
    assert!(
        matches!(out2, CommandOutcome::Pending { .. }),
        "可再次发动: {out2:?}"
    );
    let again = e.open_choice().expect("rende_cards 应再次挂起");
    let Some(ChoiceOption::Card(third)) = again.spec.options.first().copied() else {
        panic!("应存在手牌选项");
    };
    answer_sel(&mut e, vec![ChoiceOption::Card(third)]);
    let _t2 = e.open_choice().expect("rende_target 应挂起");
    answer_sel(&mut e, vec![ChoiceOption::Player(2)]);
    assert_eq!(e.debug_hp(0), 4, "累计已跨过 2 → 第二次不回血");
    assert_eq!(e.debug_hand(2).len(), 1, "第二目标收到一张");
    assert_zone_conservation(&e);

    // 空应答 = 放弃发动（新引擎）
    let mut e2 = Engine::new(952, 3);
    advance_to_play(&mut e2, 0);
    for p in 0..3 {
        e2.debug_clear_hand(p);
    }
    e2.debug_grant(0, "仁德");
    e2.debug_set_hp(0, 3);
    let _ = e2.debug_place(0, "shan");
    let skill2 = skill_def_id(&e2, "仁德");
    submit_cmd(&mut e2, 0, GameAction::ActivateSkill { skill: skill2 });
    answer_empty(&mut e2);
    assert_eq!(e2.debug_hp(0), 3, "放弃发动不得回血");
    assert_eq!(e2.debug_hand(0).len(), 1);
}

/// 激将（刘备）：蜀势力角色（明置）确认后交出一张杀；拒绝则无事；
/// 无蜀势力候选时无法发动。
#[test]
fn liubei_jijiang_borrows_sha_from_shu_helper() {
    use engine_native::state::GameMode;
    let build = |seed: u64, shu_helper: bool| -> Engine {
        let mut e = Engine::new(seed, 3);
        let mut snap = e.save();
        snap.core.mode = GameMode::Kingdoms;
        for p in snap.core.players.iter_mut() {
            p.revealed_generals = [true, false];
        }
        snap.core.players[0].general_slots[0] = Some(2); // 曹操 魏
        snap.core.players[1].general_slots[0] = if shu_helper { Some(5) } else { Some(2) };
        snap.core.players[2].general_slots[0] = Some(5); // 关羽 蜀
        e = Engine::restore(&snap).expect("restore");
        advance_to_play(&mut e, 0);
        e.debug_grant(0, "激将");
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        e
    };
    let skill = skill_def_id(&e_dummy(), "激将");

    // (1) 蜀势力 helper 确认 → 借牌直结算（视为刘备对目标使用杀）。
    //     新流程（ADR 0776）：按座位序逐人询问蜀将——首位即 1 号，
    //     不再有刘备候选选择步。
    let mut e = build(953, true);
    let sha = e.debug_place(1, "sha");
    e.debug_set_hp(2, 4);
    submit_cmd(&mut e, 0, GameAction::ActivateSkill { skill });
    let r = e.open_choice().expect("jijiang_respond 应挂起");
    assert_eq!((r.spec.prompt, r.observer), ("jijiang_respond", 1));
    answer(&mut e, ChoiceOption::Confirm);
    let t = e.open_choice().expect("jijiang_target 应挂起");
    assert_eq!(t.spec.prompt, "jijiang_target");
    answer_sel(&mut e, vec![ChoiceOption::Player(2)]);
    assert!(!e.debug_hand(1).contains(&sha), "helper 的杀已被借出");
    assert_eq!(e.debug_hp(2), 3, "借牌直结算：目标无闪受 1 点伤害");
    assert!(!e.debug_hand(0).contains(&sha), "借牌结算后不进刘备手牌");

    // (2) 首位拒绝 → 继续问下一位蜀将；全拒绝后取消且不消耗每回合一次
    let mut e2 = build(954, true);
    let sha2 = e2.debug_place(1, "sha");
    submit_cmd(&mut e2, 0, GameAction::ActivateSkill { skill });
    let r = e2.open_choice().expect("首位蜀将被询问");
    assert_eq!((r.spec.prompt, r.observer), ("jijiang_respond", 1));
    answer(&mut e2, ChoiceOption::Pass);
    let r2 = e2.open_choice().expect("拒绝后应继续问下一位蜀将");
    assert_eq!((r2.spec.prompt, r2.observer), ("jijiang_respond", 2));
    answer(&mut e2, ChoiceOption::Pass);
    assert!(e2.open_choice().is_none(), "全部拒绝后结束");
    assert!(e2.debug_hand(1).contains(&sha2), "拒绝后牌不动");
    assert!(!e2.debug_hand(0).contains(&sha2));
    // 取消语义：发动未被消耗 → 可再次发动
    let again = submit_cmd(&mut e2, 0, GameAction::ActivateSkill { skill });
    assert!(
        !matches!(
            again,
            CommandOutcome::Rejected(RejectReason::SkillAlreadyUsed)
        ),
        "全拒绝取消后不应消耗每回合一次: {again:?}"
    );

    // (3) 预筛移除：无蜀将持杀也可发动——逐人询问均无法响应后取消。
    //     build(955, false)：1 号=魏；2 号=蜀（无杀）。询问只落到 2 号。
    let mut e3 = build(955, false);
    let _ = e3.debug_place(1, "sha");
    let out3 = submit_cmd(&mut e3, 0, GameAction::ActivateSkill { skill });
    assert!(
        matches!(
            out3,
            CommandOutcome::Pending { .. } | CommandOutcome::Accepted { .. }
        ),
        "不以暗手牌预筛：应进入逐人询问而非拒绝: {out3:?}"
    );
    let r3 = e3.open_choice().expect("蜀将 2 被询问");
    assert_eq!((r3.spec.prompt, r3.observer), ("jijiang_respond", 2));
    answer(&mut e3, ChoiceOption::Pass);
    assert!(e3.open_choice().is_none(), "无响应后取消");
    assert!(!e3.debug_hand(1).is_empty(), "魏将的杀不被索取");
    assert_zone_conservation(&e);
}
