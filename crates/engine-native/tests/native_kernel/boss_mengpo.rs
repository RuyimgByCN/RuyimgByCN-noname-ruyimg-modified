use super::*;

/// 往事：2 号回合开始获封禁——基本牌（杀）出牌被拒（CardLimit）、锦囊
/// （无中生有）可用；孟婆本人回合不封禁；回合结束全清；封禁态入存档。
#[test]
fn boss_mengpo_wangshi_basic_ban_turn_clear() {
    let mut e = engine_native::Engine::new_boss_stage(87040, 4, 0, 3, 6, 4, 0);
    let mp_def = mengpo_def_of(&e);
    e.debug_set_general_def(1, mp_def);
    e.debug_set_hp(1, 3);
    assert_eq!(
        e.catalog().general_meta(mp_def).map(|g| g.0),
        Some("孟婆"),
        "孟婆入目录"
    );
    for sk in ["拾忧", "忘魂", "往事"] {
        let d = e
            .catalog()
            .skill_by_name(sk)
            .unwrap_or_else(|| panic!("{sk} 入目录"));
        assert!(e.state_core().skills.contains_key(&(1, d)), "{sk} 实例");
    }

    advance_to_play(&mut e, 0);
    // 推进 0 号回合收束 → 1 号（孟婆）回合：持有者本人不封禁
    next_phase_settled(&mut e);
    next_phase_settled(&mut e);
    next_phase_settled(&mut e);
    advance_to_play(&mut e, 1);
    assert!(!e.state_core().player(1).wangshi_ban, "本人回合不封禁");
    // 推进 1 号回合收束 → 2 号回合：准备阶段获封禁
    next_phase_settled(&mut e);
    next_phase_settled(&mut e);
    next_phase_settled(&mut e);
    advance_to_play(&mut e, 2);
    assert!(e.state_core().player(2).wangshi_ban, "往事：2 号回合封禁");

    // 封禁态存档往返（wangshi_ban 入存档 v63）
    let snap = e.save();
    let r = engine_native::Engine::restore(&snap).expect("往返");
    assert!(r.state_core().player(2).wangshi_ban, "封禁态往返保留");
    drop(r);

    // 基本牌被拒（CardLimit）；非基本牌可用
    e.debug_clear_hand(2);
    let sha = e.debug_place(2, "sha");
    assert!(
        matches!(
            play(&mut e, sha, vec![1]),
            CommandOutcome::Rejected(RejectReason::CardLimit)
        ),
        "往事：基本牌（杀）被拒"
    );
    let wz = e.debug_place(2, "wuzhong");
    let out = play(&mut e, wz, vec![]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "往事：非基本牌可用（{out:?}）"
    );
    pass_nullify_windows(&mut e);
    drain_choices(&mut e);
    // 推进到回合结束：封禁全清（K52：清零时点＝整回合结束 phaseAfter 段，
    // 原结束阶段开始锚早一拍——多推一次收束）
    next_phase_settled(&mut e);
    next_phase_settled(&mut e);
    next_phase_settled(&mut e); // End→收束→phaseAfter 全清
    assert!(!e.state_core().player(2).wangshi_ban, "回合结束封禁全清");
}

/// 拾忧：2 号（hp 压 1，弃 2 张）弃牌阶段弃牌 → 孟婆获得其中花色互异
/// 者（按弃牌序贪心首个未见花色）；获得者手牌/弃牌堆/日志三面对账。
#[test]
fn boss_mengpo_shiyou_distinct_suit_gain() {
    let mut e = engine_native::Engine::new_boss_stage(87041, 4, 0, 3, 6, 4, 0);
    let mp_def = mengpo_def_of(&e);
    e.debug_set_general_def(1, mp_def);
    e.debug_set_hp(1, 3);
    advance_to_play(&mut e, 0);
    // 0 号回合收束 → 1 号回合 → 2 号回合
    next_phase_settled(&mut e);
    next_phase_settled(&mut e);
    next_phase_settled(&mut e);
    advance_to_play(&mut e, 1);
    next_phase_settled(&mut e);
    next_phase_settled(&mut e);
    next_phase_settled(&mut e);
    advance_to_play(&mut e, 2);

    // 2 号压 hp=1（手牌上限 1），清手后布 3 张（弃 need=2=手牌前两张）
    e.debug_set_hp(2, 1);
    e.debug_clear_hand(2);
    let c0 = e.debug_place(2, "sha");
    let c1 = e.debug_place(2, "wuzhong");
    let c2 = e.debug_place(2, "tao");
    let h1_before = e.state_core().player(1).hand.len();
    let suit = |e: &engine_native::Engine, id: u32| e.catalog().cards[id as usize].suit;

    next_phase_settled(&mut e); // Play→Discard 询问（弃前两张）
    next_phase_settled(&mut e); // Discard→End

    // 期望获得：按弃牌序贪心取首个未见花色
    let mut seen: Vec<engine_native::catalog::Suit> = Vec::new();
    let expect_gain: Vec<u32> = [c0, c1]
        .iter()
        .copied()
        .filter(|c| {
            let s = suit(&e, *c);
            if seen.contains(&s) {
                false
            } else {
                seen.push(s);
                true
            }
        })
        .collect();
    assert!(!expect_gain.is_empty(), "至少获得首张");
    let hand1 = &e.state_core().player(1).hand;
    for c in &expect_gain {
        assert!(hand1.contains(c), "拾忧：孟婆获得花色互异牌 {c}");
        assert!(
            !e.state_core().discard.contains(c),
            "拾忧：获得牌已离弃牌堆"
        );
    }
    for c in [c0, c1] {
        if !expect_gain.contains(&c) {
            assert!(
                e.state_core().discard.contains(&c),
                "拾忧：同花色未获牌留在弃牌堆"
            );
        }
    }
    assert_eq!(
        hand1.len(),
        h1_before + expect_gain.len(),
        "拾忧：恰获花色互异子集"
    );
    assert!(
        e.state_core().player(2).hand.contains(&c2),
        "2 号保留未弃牌"
    );
    assert!(
        log_has(&e, |l| matches!(
            l,
            LogEvent::CardsGivenPublic { from: 2, to: 1, .. }
        )),
        "拾忧：获得日志"
    );
    assert_zone_conservation(&e);
}

/// 忘魂：孟婆真死——全场往事封禁清除+座位序前两个有技的其他存活（0、
/// 2 号）各禁用首个启用技，3 号不受影响；禁用态与清旗存档往返。
#[test]
fn boss_mengpo_wanghun_death_clear_and_disable() {
    let mut e = engine_native::Engine::new_boss_stage(87042, 4, 0, 0, 6, 4, 0);
    let mp_def = mengpo_def_of(&e);
    e.debug_set_general_def(1, mp_def);
    e.debug_set_hp(1, 3);
    // 授可禁用技（碎盾=标准早位 def，各座位首个启用技）
    e.debug_grant(0, "碎盾");
    e.debug_grant(2, "碎盾");
    e.debug_grant(3, "碎盾");
    advance_to_play(&mut e, 0);
    // 推进到 2 号回合（准备阶段获往事封禁，作清旗观察面）
    next_phase_settled(&mut e);
    next_phase_settled(&mut e);
    next_phase_settled(&mut e);
    advance_to_play(&mut e, 1);
    next_phase_settled(&mut e);
    next_phase_settled(&mut e);
    next_phase_settled(&mut e);
    advance_to_play(&mut e, 2);
    assert!(e.state_core().player(2).wangshi_ban, "前置：2 号封禁在位");

    let suidun = e.catalog().skill_by_name("碎盾").unwrap();
    let enabled = |e: &engine_native::Engine, p: u8| {
        e.state_core().skills.get(&(p, suidun)).map(|i| i.enabled)
    };
    assert_eq!(enabled(&e, 0), Some(true), "前置：0 号碎盾启用");

    // 孟婆（1 号）真死：濒死救援逐问 Pass 放行 → 遗言
    let out = e.debug_deal_damage(Some(0), 1, 10, DamageNature::Normal);
    let _ = out;
    for _ in 0..3 {
        match e.open_choice() {
            Some(c) if c.spec.prompt == "dying_save" => {
                let (cid, obs) = (c.id, c.observer);
                submit_cmd(
                    &mut e,
                    obs,
                    GameAction::Answer {
                        choice_id: cid,
                        selection: vec![ChoiceOption::Pass],
                    },
                );
            }
            _ => break,
        }
    }
    assert!(!e.state_core().player(1).alive, "孟婆已死亡");
    assert!(!e.state_core().player(2).wangshi_ban, "忘魂：全场封禁清除");
    assert_eq!(enabled(&e, 0), Some(false), "忘魂：0 号首个技禁用");
    assert_eq!(enabled(&e, 2), Some(false), "忘魂：2 号首个技禁用");
    assert_eq!(enabled(&e, 3), Some(true), "忘魂：仅前两个，3 号不受影响");
    assert!(
        log_has(&e, |l| matches!(
            l,
            LogEvent::SkillToggled {
                player: 0,
                enabled: false,
                ..
            }
        )),
        "0 号禁用日志"
    );
    assert!(
        log_has(&e, |l| matches!(
            l,
            LogEvent::SkillToggled {
                player: 2,
                enabled: false,
                ..
            }
        )),
        "2 号禁用日志"
    );

    // 往返：禁用态与清旗保留
    let snap = e.save();
    let r = engine_native::Engine::restore(&snap).expect("往返");
    assert_zone_conservation(&r);
    assert_eq!(
        r.state_core().skills.get(&(2, suidun)).map(|i| i.enabled),
        Some(false),
        "往返保留禁用"
    );
    assert!(!r.state_core().player(2).wangshi_ban, "往返保留清旗");
}
