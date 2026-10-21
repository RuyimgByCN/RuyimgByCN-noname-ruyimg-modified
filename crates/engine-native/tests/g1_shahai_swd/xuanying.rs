use super::*;

/// T12 swd 武将面批 90（端蒙玄影/烈刃）：**末将**——79/79 名全入包。
#[test]
fn swd_generals_b90_xuanying_lieren() {
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
        pack.generals.len() >= 79,
        "79 将全入包（现 {}）",
        pack.generals.len()
    );
    let mut e = Engine::new_with_extensions(62057, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 玄影主动面：未横置 → 选目标 → 横置自身+虚拟杀
    e.debug_grant(0, "xuanying");
    e.debug_set_hp(1, 8);
    assert!(!e.state_core().players[0].linked, "玄影：初始未横置");
    let def_xy = e.catalog().skill_by_name("xuanying").expect("玄影在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_xy });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "玄影发动 {out:?}"
    );
    let c = e.open_choice().expect("玄影目标窗");
    assert_eq!(c.spec.prompt, "xuanying_target");
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
    assert!(e.state_core().players[0].linked, "玄影：横置自身");
    assert_eq!(e.debug_hp(1), 7, "玄影：虚拟杀命中 p1（8→7）");

    // ② 烈刃：p0 杀 p2 命中后拼点（赢则获得其一张牌）
    // 玄影虚拟杀已消耗本回合出杀额度——重置旗标（测试面）并解除横置
    e.state_core_mut().players[0].linked = false;
    e.state_core_mut().turn.sha_used_play_phase = false;
    e.debug_clear_hand(0);
    e.debug_grant(0, "lieren");
    e.debug_set_hp(2, 8);
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
    // p1 另留一张非拼点牌作为「赢得取其一张」的目标（拼点后手牌不空）
    let spare = {
        let id = (0..e.catalog().cards.len() as u32)
            .find(|c| *c != hi && *c != lo && e.catalog().card_kind(*c) != CardKind::Sha)
            .expect("备用牌");
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
    let _ = spare;
    // 目标 p1（邻座，距离 1 在射程内）；p1 hp 重置
    e.debug_set_hp(1, 8);
    let s0 = e.debug_place(0, "sha");
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
    // 推进：闪窗（Pass）→伤害→烈刃确认窗
    let mut guard = 0;
    let step = loop {
        guard += 1;
        assert!(guard < 32, "烈刃窗未出现");
        let s = match e.open_choice() {
            None => None,
            Some(ch) => Some((
                ch.id,
                ch.observer,
                ch.spec.prompt.to_string(),
                ch.spec.min,
                ch.spec.options.clone(),
            )),
        };
        let Some((id, observer, prompt, min, options)) = s else {
            panic!("无挂起");
        };
        if prompt == "trigger_confirm" && observer == 0 {
            break (id, observer);
        }
        let sel: Vec<ChoiceOption> = if min == 0
            || prompt == "sha_ask"
            || prompt == "respond_shan"
            || prompt == "nullify_window"
        {
            vec![ChoiceOption::Pass]
        } else {
            options.iter().take(min.max(1)).copied().collect()
        };
        submit(
            &mut e,
            observer,
            GameAction::Answer {
                choice_id: id,
                selection: sel,
            },
        );
    };
    let (cid, obs) = step;
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let c = e.open_choice().expect("烈刃目标选牌窗");
    assert_eq!(c.spec.prompt, "lieren_target_pick");
    assert_eq!(c.observer, 1, "拼点牌由受害者选");
    let (cid, obs) = (c.id, c.observer);
    let pick_t = c.spec.options[0].clone();
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![pick_t],
        },
    );
    let c = e.open_choice().expect("烈刃己方选牌窗");
    assert_eq!(c.spec.prompt, "lieren_player_pick");
    let (cid, obs) = (c.id, c.observer);
    assert_eq!(obs, 0);
    // 己方出大点牌
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
    assert_eq!(e.debug_hp(1), 7, "烈刃前序：p1 受 1 伤（8→7）");
    assert!(hand_len_w(&e, 0) >= 1, "烈刃赢：获得 p1 一张牌（手牌≥1）");
}
