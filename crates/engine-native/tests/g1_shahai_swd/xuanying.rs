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

// Load the actual selected producer asset; general assignment, not debug_grant, owns the parent.
fn source_parent_pack() -> ExtensionPack {
    let asset = repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json");
    bridge(&std::fs::read_to_string(asset).unwrap())
}
fn source_parent_engine(pack: &ExtensionPack) -> Engine {
    let mut e = Engine::new_with_extensions(331801, 4, &[pack]).unwrap();
    e.assign_general_named(0, "swd_duanmeng").unwrap();
    for owner in 0..4 {
        e.debug_clear_hand(owner);
    }
    let def = e.catalog().skill_by_name("xuanying").unwrap();
    assert!(e.state_core().skills.contains_key(&(0, def)));
    // 父技 group 現在把傷害子技一併授予（C331 審查第 4 項的修法）。
    let child = e
        .catalog()
        .skill_by_name("xuanying_damage")
        .expect("damage child registered by the parent group");
    assert!(
        e.state_core().skills.contains_key(&(0, child)),
        "parent group grants the damage child"
    );
    e
}
fn source_answer(e: &mut Engine, selection: Vec<ChoiceOption>) {
    let c = e.open_choice().unwrap().clone();
    let result = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: e.save().last_dedup + 1,
        action: GameAction::Answer {
            choice_id: c.id,
            selection,
        },
    });
    assert!(!matches!(result, CommandOutcome::Rejected(_)), "{result:?}");
}
fn source_settle(e: &mut Engine) {
    for _ in 0..16 {
        let Some(c) = e.open_choice() else {
            return;
        };
        let selection = if c.spec.options.contains(&ChoiceOption::Confirm) {
            vec![ChoiceOption::Confirm]
        } else if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else {
            c.spec.options.iter().take(c.spec.min).copied().collect()
        };
        source_answer(e, selection);
    }
    panic!("source fixture did not settle");
}
#[test]
fn source_parent_group_grants_damage_child_counterattack_and_requires_the_grant() {
    let pack = source_parent_pack();
    let mut e = source_parent_engine(&pack);
    e.debug_set_turn(1, Phase::Play, 1);
    e.state_core_mut().players[0].linked = true;
    let attacker_hp = e.debug_hp(1);
    let owner_hp = e.debug_hp(0);
    let cursor = e.log().len();
    e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    source_settle(&mut e);
    assert_eq!(e.debug_hp(0), owner_hp - 1);
    assert_eq!(
        e.debug_hp(1),
        attacker_hp - 1,
        "granted damage child counterattacks the source"
    );
    assert!(
        !e.state_core().player(0).linked,
        "counterattack unlinks the owner"
    );
    assert!(
        e.log()[cursor..]
            .iter()
            .any(|event| matches!(event, engine_native::LogEvent::CardPlayed { player: 0, .. })),
        "counterattack plays a virtual Sha"
    );
    let snap = decode_archive(&encode_archive(&e.save())).unwrap();
    let r = Engine::restore_with_extensions(&snap, &[&pack]).unwrap();
    let expected = e.save();
    let mut actual = r.save();
    actual.auth_generation = expected.auth_generation;
    assert_eq!(actual, expected);

    // 只有父技、沒有子技授予 → 傷害面不可達（不解除橫置、不反擊）。
    let mut no_child = source_parent_engine(&pack);
    let child = no_child.catalog().skill_by_name("xuanying_damage").unwrap();
    no_child.state_core_mut().skills.remove(&(0, child));
    no_child.debug_set_turn(1, Phase::Play, 1);
    no_child.state_core_mut().players[0].linked = true;
    let attacker_hp = no_child.debug_hp(1);
    let owner_hp = no_child.debug_hp(0);
    let cursor = no_child.log().len();
    no_child.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    source_settle(&mut no_child);
    assert_eq!(no_child.debug_hp(0), owner_hp - 1);
    assert_eq!(
        no_child.debug_hp(1),
        attacker_hp,
        "ungranted damage child must not counterattack"
    );
    assert!(
        no_child.state_core().player(0).linked,
        "ungranted damage child cannot unlink the owner"
    );
    assert!(
        !no_child.log()[cursor..]
            .iter()
            .any(|event| matches!(event, engine_native::LogEvent::CardPlayed { player: 0, .. })),
        "ungranted child must not manufacture a Sha"
    );
}
#[test]
fn source_parent_grant_keeps_active_sha_and_cancel_across_byte_restore() {
    for target in [Some(2), None] {
        let pack = source_parent_pack();
        let mut e = source_parent_engine(&pack);
        e.debug_set_turn(0, Phase::Play, 1);
        let def = e.catalog().skill_by_name("xuanying").unwrap();
        let result = e.submit(GameCommand {
            actor: 0,
            expected_revision: e.revision(),
            dedup_id: e.save().last_dedup + 1,
            action: GameAction::ActivateSkill { skill: def },
        });
        assert!(!matches!(result, CommandOutcome::Rejected(_)), "{result:?}");
        assert_eq!(e.open_choice().unwrap().spec.prompt, "xuanying_target");
        let snap = decode_archive(&encode_archive(&e.save())).unwrap();
        let mut r = Engine::restore_with_extensions(&snap, &[&pack]).unwrap();
        let cursor = e.log().len();
        for engine in [&mut e, &mut r] {
            source_answer(
                engine,
                target.map(ChoiceOption::Player).into_iter().collect(),
            );
            source_settle(engine);
        }
        let expected = e.save();
        let mut actual = r.save();
        actual.auth_generation = expected.auth_generation;
        assert_eq!(actual, expected);
        assert_eq!(e.state_hash(), r.state_hash());
        assert_eq!(&e.log()[cursor..], r.log());
        assert_eq!(e.state_core().player(0).linked, target.is_some());
        assert_eq!(
            e.debug_hp(2),
            if target.is_some() { 3 } else { 4 },
            "granted sha keeps its out-of-range target branch; cancellation spends nothing"
        );
    }
}

#[test]
fn source_parent_active_restore_rejects_wrong_choice_fields() {
    let pack = source_parent_pack();
    let mut e = source_parent_engine(&pack);
    e.debug_set_turn(0, Phase::Play, 1);
    let def = e.catalog().skill_by_name("xuanying").unwrap();
    let result = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: e.save().last_dedup + 1,
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(!matches!(result, CommandOutcome::Rejected(_)));
    let snap = decode_archive(&encode_archive(&e.save())).unwrap();
    assert!(Engine::restore_with_extensions(&snap, &[&pack]).is_ok());
    let mut unknown = snap.clone();
    unknown.resolution.open_choice.as_mut().unwrap().spec.prompt = "c331_unknown_xuanying";
    assert!(matches!(
        decode_archive(&encode_archive(&unknown)),
        Err(engine_native::archive::ArchiveError::BadTag(
            "prompt_unknown",
            0
        ))
    ));
    for case in 0..8 {
        let mut bad = snap.clone();
        let c = bad.resolution.open_choice.as_mut().unwrap();
        match case {
            0 => c.spec.options.push(ChoiceOption::Player(0)),
            1 => c.observer = 1,
            2 => c.spec.prompt = "rende_target",
            3 => c.spec.allow_cancel = false,
            4 => c.spec.min = 0,
            5 => c.spec.kind = engine_native::choice::ChoiceKind::SelectCards,
            6 => c.spec.max = 2,
            7 => {
                let stage = bad
                    .resolution
                    .parks
                    .iter_mut()
                    .find_map(|p| match &mut p.machine {
                        engine_native::snap::MachineSnapshot::Host { stage, .. } => Some(stage),
                        _ => None,
                    })
                    .unwrap();
                *stage = 0;
            }
            _ => unreachable!(),
        }
        let decoded = decode_archive(&encode_archive(&bad)).unwrap();
        assert!(
            Engine::restore_with_extensions(&decoded, &[&pack]).is_err(),
            "case{case} must not relax the existing exact ask contract"
        );
    }
}
