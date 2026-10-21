use super::*;

/// T12 swd 武将面批 86（疆梧义绝/荡平）：整将行为面 + 反例。
#[test]
fn swd_generals_b86_yijue_dangping() {
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
    assert!(
        pack.generals.len() >= 75,
        "75 将入册（现 {}）",
        pack.generals.len()
    );
    let mut e = Engine::new_with_extensions(62053, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 义绝：p0 高点数 vs p1 低点数 → 赢：p1 封印+受 1 伤
    e.debug_grant(0, "yijue");
    let hi = {
        let id = (0..e.catalog().cards.len() as u32)
            .find(|c| e.catalog().cards[*c as usize].rank >= 12)
            .expect("大点牌");
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
    let lo = {
        let id = (0..e.catalog().cards.len() as u32)
            .find(|c| *c != hi && e.catalog().cards[*c as usize].rank <= 3)
            .expect("小点牌");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.discard.retain(|x| *x != id);
        core.table.retain(|x| *x != id);
        for pl in core.players.iter_mut() {
            pl.hand.retain(|x| *x != id);
        }
        core.players[1].hand.push(id);
        id
    };
    let _ = hi;
    let _ = lo;
    e.debug_set_hp(1, 4);
    let def_yj = e.catalog().skill_by_name("yijue").expect("义绝在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_yj });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "义绝发动 {out:?}"
    );
    let c = e.open_choice().expect("义绝目标窗");
    assert_eq!(c.spec.prompt, "yijue_target");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    let c = e.open_choice().expect("义绝目标选牌窗");
    assert_eq!(c.spec.prompt, "yijue_target_pick");
    let (cid, obs) = (c.id, c.observer);
    assert_eq!(obs, 1, "拼点牌由目标选");
    let pick_tp = c.spec.options[0].clone();
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![pick_tp],
        },
    );
    let c = e.open_choice().expect("义绝己方选牌窗");
    assert_eq!(c.spec.prompt, "yijue_player_pick");
    let (cid, obs) = (c.id, c.observer);
    assert_eq!(obs, 0);
    // 选择 p0 手上最大的牌
    let best = {
        let opts = c.spec.options.clone();
        let mut best = opts[0].clone();
        for o in &opts {
            if let ChoiceOption::Card(cc) = o {
                if let ChoiceOption::Card(bc) = &best {
                    if e.catalog().cards[*cc as usize].rank > e.catalog().cards[*bc as usize].rank {
                        best = o.clone();
                    }
                }
            }
        }
        best
    };
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![best],
        },
    );
    settle_optional_confirm(&mut e);
    pass_nullify(&mut e);
    assert_eq!(e.debug_hp(1), 3, "义绝赢：p1 受 1 伤（4→3）");
    assert!(e.state_core().players[1].fengyin_active, "义绝赢：p1 封印");

    // ② 荡平：造伤害后可弃牌 → 邻近另一角色受 1 伤（对座距离超）

    e.debug_grant(0, "dangping");
    // p0 杀 p1（距离 1 邻座）；受害者 p1 的邻近= p2
    e.debug_clear_hand(0);
    let s0 = e.debug_place(0, "sha");
    let d0 = e.debug_place(0, "tao");
    let _ = d0;
    e.debug_clear_hand(1);
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: s0,
            targets: vec![1],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "p0 杀 p2 {out:?}"
    );
    // 推进：闪窗→伤害→荡平确认窗
    let mut guard = 0;
    let dang_choice = loop {
        guard += 1;
        assert!(guard < 32, "荡平确认窗未出现");
        let Some(ch) = e.open_choice() else {
            panic!("荡平窗缺失");
        };
        if ch.spec.prompt == "trigger_confirm" && ch.observer == 0 {
            break ch;
        }
        let sel: Vec<ChoiceOption> = if ch.spec.prompt == "sha_ask"
            || ch.spec.prompt == "shan_ask"
            || ch.spec.prompt == "nullify_window"
            || ch.spec.min == 0
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
    };
    let (cid, obs) = (dang_choice.id, dang_choice.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    // 弃牌窗
    let c = e.open_choice().expect("荡平弃牌窗");
    assert_eq!(c.spec.prompt, "dangping_discard");
    let (cid, obs) = (c.id, c.observer);
    let pick = c.spec.options[0].clone();
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![pick],
        },
    );
    // 目标窗（受害者 p2 的邻近）
    let c = e.open_choice().expect("荡平目标窗");
    assert_eq!(c.spec.prompt, "dangping_target");
    let (cid, obs) = (c.id, c.observer);
    let hp2_before = e.debug_hp(2);
    let ok_target = c
        .spec
        .options
        .iter()
        .any(|o| matches!(o, ChoiceOption::Player(2)));
    let pick_t = c.spec.options[0].clone();
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![pick_t],
        },
    );
    settle_optional_confirm(&mut e);
    pass_nullify(&mut e);
    assert!(ok_target, "荡平：p2（与受害者 p1 距离 1）在候选内");
    assert!(e.debug_hp(1) <= 3, "荡平前序：p1 受杀伤");
    assert_eq!(
        e.debug_hp(2),
        hp2_before.saturating_sub(1),
        "荡平：邻近角色受 1 伤"
    );
}
