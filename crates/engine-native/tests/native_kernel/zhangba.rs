use super::*;

/// P2/N08 批③-b 第七把（规则 v71，2026-10-07）：丈八蛇矛（♠12，射程 3）
/// ——两张手牌当【杀】：装备即授予配套主动技（出牌阶段发动）→ 选两张
/// 手牌 → 选射程内目标 → 首张作 declared-Sha 载荷进入杀结算（次张弃置）。
#[test]
fn zhangba_two_cards_as_sha() {
    let mut e = Engine::new(210, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let zb = e.debug_move_named_card_to_hand(0, "zhangba");
    let out = submit_cmd(
        &mut e,
        0,
        GameAction::PlayCard {
            card: zb,
            targets: vec![],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "装备丈八: {out:?}"
    );
    // 装备授予配套主动技。
    let def_zb = e.catalog().skill_by_name("丈八蛇矛").expect("丈八技在目录");
    assert!(
        e.save().core.skills.contains_key(&(0, def_zb)),
        "装备授予丈八技实例"
    );
    let m1 = e.debug_place(0, "tao");
    let m2 = e.debug_place(0, "shan");
    // 发动主动技：选两张手牌（桃+闪）。
    let def_id = def_zb;
    let out = submit_cmd(&mut e, 0, GameAction::ActivateSkill { skill: def_id });
    assert!(
        matches!(out, CommandOutcome::Pending { .. }),
        "丈八选牌询问: {out:?}"
    );
    let pick = e.open_choice().expect("丈八选两张手牌询问");
    assert_eq!((pick.spec.prompt, pick.spec.min), ("zhangba_pick_two", 2));
    let (pid, pobs) = (pick.id, pick.observer);
    submit_cmd(
        &mut e,
        pobs,
        GameAction::Answer {
            choice_id: pid,
            selection: vec![ChoiceOption::Card(m1), ChoiceOption::Card(m2)],
        },
    );
    // 选目标 1 号（射程 3 内）。
    let pick2 = e.open_choice().expect("丈八目标询问");
    assert_eq!(pick2.spec.prompt, "zhangba_sha_target");
    let (pid2, pobs2) = (pick2.id, pick2.observer);
    submit_cmd(
        &mut e,
        pobs2,
        GameAction::Answer {
            choice_id: pid2,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    // 首张（桃）作 declared-Sha 载荷；1 号无闪 → 直接受伤。
    for _ in 0..8 {
        match e.open_choice() {
            Some(g) => {
                if g.spec.options.is_empty() {
                    let _ = g;
                    break;
                }
                let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                    g.spec.options.iter().take(g.spec.min).copied().collect()
                } else {
                    vec![g.spec.options[0]]
                };
                let (id2, obs2) = (g.id, g.observer);
                submit_cmd(
                    &mut e,
                    obs2,
                    GameAction::Answer {
                        choice_id: id2,
                        selection: sel,
                    },
                );
            }
            None => break,
        }
    }
    assert_eq!(e.debug_hp(1), 3, "丈八两牌当杀：1 号受 1 伤");
    assert!(
        !e.debug_hand(0).contains(&m1) && !e.debug_hand(0).contains(&m2),
        "两张素材牌离手"
    );
    // declared-Sha 生效：c1 按杀结算（弃牌堆），declared_kinds 有记录。
    assert!(
        e.state_core()
            .declared_kinds
            .values()
            .any(|k| matches!(k, engine_native::CardKind::Sha)),
        "declared-Sha 记录存在"
    );
}
