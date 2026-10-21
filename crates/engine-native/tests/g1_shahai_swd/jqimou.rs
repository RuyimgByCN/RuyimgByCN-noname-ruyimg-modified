use super::*;

/// T12 swd 武将面批 91（姬良极谋/少微剑击/麦星头弹）：三将补技。
#[test]
fn swd_generals_b91_jqimou_jianji_toudan() {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shahai-swd")
        .expect("swd 安装")
        .clone();
    let json = std::fs::read_to_string(
        std::path::PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(62058, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 剑击（少微）：弃装备牌 → 虚拟杀+摸 1
    e.debug_grant(0, "jianji");
    let eq = {
        let id = (0..e.catalog().cards.len() as u32)
            .find(|c| e.catalog().card_kind(*c).is_equip())
            .expect("装备牌");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.discard.retain(|x| *x != id);
        core.table.retain(|x| *x != id);
        for pl in core.players.iter_mut() {
            pl.hand.retain(|x| *x != id);
        }
        core.players[0].hand.push(id);
        id
    };
    e.debug_set_hp(1, 8);
    let def_jj = e.catalog().skill_by_name("jianji").expect("剑击在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_jj });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "剑击发动 {out:?}"
    );
    let c = e.open_choice().expect("剑击选牌窗");
    assert_eq!(c.spec.prompt, "jianji_pick");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(eq)],
        },
    );
    let c = e.open_choice().expect("剑击目标窗");
    assert_eq!(c.spec.prompt, "jianji_target");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    settle_optional_confirm(&mut e);
    pass_nullify(&mut e);
    assert_eq!(e.debug_hp(1), 7, "剑击：虚拟杀命中 p1（8→7）");

    // ② 头弹（麦星）：黑桃 → 目标火伤+全员弃牌
    e.debug_grant(0, "toudan");
    e.debug_clear_hand(0);
    let sp = {
        let id = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                matches!(
                    e.catalog().cards[*c as usize].suit,
                    engine_native::catalog::Suit::Spade
                )
            })
            .expect("黑桃牌");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.discard.retain(|x| *x != id);
        core.table.retain(|x| *x != id);
        for pl in core.players.iter_mut() {
            pl.hand.retain(|x| *x != id);
        }
        core.players[0].hand.push(id);
        id
    };
    // 各人布一张手牌供链式弃置
    let mut puts = Vec::new();
    for p in 0..4u8 {
        let id = (0..e.catalog().cards.len() as u32)
            .find(|c| !puts.contains(c) && *c != sp)
            .expect("布牌");
        puts.push(id);
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.discard.retain(|x| *x != id);
        core.table.retain(|x| *x != id);
        for pl in core.players.iter_mut() {
            pl.hand.retain(|x| *x != id);
        }
        core.players[p as usize].hand.push(id);
    }
    e.debug_set_hp(2, 8);
    let def_td = e.catalog().skill_by_name("toudan").expect("头弹在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_td });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "头弹发动 {out:?}"
    );
    let c = e.open_choice().expect("头弹选牌窗");
    assert_eq!(c.spec.prompt, "toudan_pick");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(sp)],
        },
    );
    let c = e.open_choice().expect("头弹目标窗");
    assert_eq!(c.spec.prompt, "toudan_target");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(2)],
        },
    );
    settle_optional_confirm(&mut e);
    pass_nullify(&mut e);
    assert_eq!(e.debug_hp(2), 7, "头弹：p2 受 1 火伤（8→7）");

    // ③ 极谋（姬良）：回合外受伤摸 1
    e.debug_grant(1, "jqimou");
    let h1 = hand_len_w(&e, 1);
    // 轮到 p2 时 p2 杀 p1（p1 回合外受伤）
    let mut guard = 0;
    loop {
        guard += 1;
        assert!(guard < 240, "p2 回合未到");
        if e.turn().current == 2 && e.turn().phase == Phase::Play {
            break;
        }
        if let Some(ch) = e.open_choice() {
            let sel: Vec<ChoiceOption> = if ch.spec.min > 0 {
                ch.spec.options.iter().take(ch.spec.min).copied().collect()
            } else {
                vec![]
            };
            let (cid, obs) = (ch.id, ch.observer);
            submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: sel,
                },
            );
            continue;
        }
        let actor = e.turn().current;
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    e.debug_set_hp(1, 6);
    e.debug_clear_hand(1);
    let s2 = e.debug_place(2, "sha");
    let out = submit(
        &mut e,
        2,
        GameAction::PlayCard {
            card: s2,
            targets: vec![1],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "p2 杀 p1 {out:?}"
    );
    // 推进：闪 Pass → 伤害 → 极谋摸 1（无窗）
    for _ in 0..24 {
        let Some(ch) = e.open_choice() else { break };
        if ch.spec.prompt == "type_never" {
            break;
        }
        let sel: Vec<ChoiceOption> = if ch.spec.min == 0
            || ch.spec.prompt == "respond_shan"
            || ch.spec.prompt == "nullify_window"
        {
            vec![ChoiceOption::Pass]
        } else {
            ch.spec
                .options
                .iter()
                .take(ch.spec.min.max(1))
                .copied()
                .collect()
        };
        let (cid, obs) = (ch.id, ch.observer);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: sel,
            },
        );
    }
    assert!(
        e.debug_hp(1) < 6,
        "极谋前序：p1 已受伤（6→{}）",
        e.debug_hp(1)
    );
    assert!(
        hand_len_w(&e, 1) >= h1 + 1,
        "极谋：回合外受伤后摸牌增量 ≥1（前 {h1} 后 {}）",
        hand_len_w(&e, 1)
    );
}
