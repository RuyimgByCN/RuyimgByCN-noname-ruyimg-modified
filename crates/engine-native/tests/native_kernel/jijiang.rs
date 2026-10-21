use super::*;

/// 激将（响应入口例外）在判定阶段发动：不被阶段门控拒绝，
/// 正常进入 helper 询问（蜀势力候选由明置势力与持杀决定）。
#[test]
fn jijiang_activate_not_phase_locked() {
    let e = Engine::new(953, 4);
    let mut snap = e.save();
    snap.core.mode = GameMode::Kingdoms;
    for p in &mut snap.core.players {
        p.revealed_generals = [true, false];
    }
    snap.core.players[0].general_slots[0] = Some(22); // 刘备（蜀）
    snap.core.players[1].general_slots[0] = Some(5); // 蜀
    snap.core.players[2].general_slots[0] = Some(5); // 蜀
    snap.core.players[3].general_slots[0] = Some(2); // 魏
    let mut e = Engine::restore(&snap).unwrap();
    e.debug_grant(0, "激将");
    e.debug_place(1, "sha");
    // 新开局：判定阶段
    assert_eq!(e.turn().phase, Phase::Judge);
    assert_eq!(e.turn().current, 0);
    let skill = e.catalog().skill_by_name("激将").unwrap();
    let out = submit_cmd(&mut e, 0, GameAction::ActivateSkill { skill });
    match out {
        CommandOutcome::Pending { choice } => {
            // 逐人询问：首个询问 = 座位序首位蜀将（1 号）的响应确认
            assert_eq!(
                (choice.spec.prompt, choice.observer),
                ("jijiang_respond", 1),
                "应进入激将逐人询问"
            );
        }
        other => panic!("激将响应入口不受出牌阶段门控: {other:?}"),
    }
}

/// 激将逐人询问挂起点的内存恢复：首位拒绝后的第二位询问可恢复续行。
#[test]
fn jijiang_pending_ask_restore_continues() {
    let mut e = Engine::new(2807, 4);
    let mut snap = e.save();
    snap.core.mode = GameMode::Kingdoms;
    for p in snap.core.players.iter_mut() {
        p.revealed_generals = [true, false];
    }
    snap.core.players[0].general_slots[0] = Some(22); // 刘备 蜀
    snap.core.players[1].general_slots[0] = Some(5); // 蜀
    snap.core.players[2].general_slots[0] = Some(5); // 蜀
    snap.core.players[3].general_slots[0] = Some(2); // 魏
    e = Engine::restore(&snap).unwrap();
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "激将");
    let sha2 = e.debug_place(2, "sha");
    e.debug_set_hp(3, 4);
    let jj = e.catalog().skill_by_name("激将").unwrap();
    submit_cmd(&mut e, 0, GameAction::ActivateSkill { skill: jj });
    // 1 号拒绝
    let r = e.open_choice().expect("首位蜀将被问");
    assert_eq!((r.spec.prompt, r.observer), ("jijiang_respond", 1));
    answer(&mut e, ChoiceOption::Pass);
    // 保存 → 恢复 → 2 号的询问继续
    let mid = e.save();
    let mut r_engine = Engine::restore(&mid).expect("恢复");
    let r2 = r_engine.open_choice().expect("恢复件仍挂起 2 号询问");
    assert_eq!((r2.spec.prompt, r2.observer), ("jijiang_respond", 2));
    let cid = r2.id;
    submit_cmd(
        &mut r_engine,
        2,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let t = r_engine.open_choice().expect("借牌后询问目标");
    assert_eq!(t.spec.prompt, "jijiang_target");
    let tid = t.id;
    submit_cmd(
        &mut r_engine,
        0,
        GameAction::Answer {
            choice_id: tid,
            selection: vec![ChoiceOption::Player(3)],
        },
    );
    assert_eq!(r_engine.debug_hp(3), 3, "借牌直结算：无闪目标受 1 点伤害");
    assert!(!r_engine.debug_hand(2).contains(&sha2), "蜀将的杀被借出");
}
