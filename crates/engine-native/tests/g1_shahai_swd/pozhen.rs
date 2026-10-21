use super::*;

/// T12 swd deferred 消解批 94（破阵 pozhen/鬼眼复活面 guiyan2）：两技
/// 行为面 + 反例。
#[test]
fn swd_deferred_b94_pozhen_guiyan2() {
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
    let mut e = Engine::new_with_extensions(62051, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 破阵（赵云）：受伤后手牌多于来源 → 弃差值张 → 来源 1 伤
    e.debug_grant(0, "pozhen");
    // 推进到 p1 回合（杀须回合属主）
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    for _ in 0..16 {
        let c = e.turn().current;
        if c == 1 && e.state_core().turn.phase == engine_native::event::Phase::Play {
            break;
        }
        let _ = submit(&mut e, c, GameAction::NextPhase);
    }
    advance_to_play(&mut e, 1);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // p0 手牌 2 张（差 2=2-0）；p1 出杀后手牌 0 张
    let c1 = e.debug_place(0, "tao");
    let c2 = e.debug_place(0, "shan");
    let sha1 = e.debug_place(1, "sha");
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 4);
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![0],
        },
    );
    // p0 出闪窗——Pass（不吃闪，让伤害落地触发破阵）
    {
        let c = e.open_choice().expect("p0 出闪窗");
        let (cid, obs) = (c.id, c.observer);
        let pass: Vec<ChoiceOption> = c
            .spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Pass))
            .cloned()
            .collect();
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: pass,
            },
        );
    }
    // 破阵确认窗 → 确认 → 弃牌窗（2 张）→ 来源 1 伤
    let c = e.open_choice().expect("破阵确认窗");
    assert_eq!((c.spec.prompt, c.observer), ("trigger_confirm", 0));
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let c = e.open_choice().expect("破阵弃牌窗");
    assert_eq!(c.spec.prompt, "pozhen_discard");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(0), 3, "破阵：p0 受杀 1（4→3）");
    assert_eq!(e.debug_hp(1), 3, "破阵：来源受 1 反伤（4→3）");
    assert!(
        !e.debug_hand(0).contains(&c1) && !e.debug_hand(0).contains(&c2),
        "破阵：差值 2 张已弃"
    );

    // ② 鬼眼复活面（冯天凌）：用鬼眼后濒死强制回 1 并移除双技
    e.debug_grant(1, "guiyan");
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // p1 用鬼眼取 p0 梅花 → guiyan2 实例授予
    let club_v = {
        let cl = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                matches!(
                    e.catalog().cards[*c as usize].suit,
                    engine_native::catalog::Suit::Club
                ) && e.catalog().card_kind(*c) != engine_native::catalog::CardKind::Sha
                    && e.state_core().deck.contains(c)
            })
            .expect("梅花在牌库");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != cl);
        core.players[0].hand.push(cl);
        cl
    };
    // 推进到 p1 回合
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    for _ in 0..16 {
        let c = e.turn().current;
        if c == 1 && e.state_core().turn.phase == engine_native::event::Phase::Play {
            break;
        }
        let _ = submit(&mut e, c, GameAction::NextPhase);
    }
    advance_to_play(&mut e, 1);
    for p in 0..4 {
        if p != 1 {
            e.debug_clear_hand(p);
        }
    }
    let club_v = {
        let core = e.state_core_mut();
        core.players[0].hand.push(club_v);
        club_v
    };
    let def_gy = e.catalog().skill_by_name("guiyan").expect("鬼眼在目录");
    let out = submit(&mut e, 1, GameAction::ActivateSkill { skill: def_gy });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "鬼眼发动 {out:?}"
    );
    let c = e.open_choice().expect("鬼眼目标窗");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(0)],
        },
    );
    let c = e.open_choice().expect("鬼眼取牌窗");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(club_v)],
        },
    );
    settle_optional_confirm(&mut e);
    assert!(e.debug_hand(1).contains(&club_v), "鬼眼：梅花牌换手");
    let g2 = e
        .catalog()
        .skill_by_name("swd_guiyan2")
        .expect("复活面在目录");
    assert!(
        e.state_core().skills.contains_key(&(1, g2)),
        "鬼眼：复活面实例已授予"
    );
    // p1 压血 1 → p0 杀致死量 → 濒死强制回 1（不进救援询问）
    e.debug_set_hp(1, 1);
    // 推回 p0 回合
    submit(&mut e, 1, GameAction::NextPhase);
    submit(&mut e, 1, GameAction::NextPhase);
    submit(&mut e, 1, GameAction::NextPhase);
    for _ in 0..16 {
        let c = e.turn().current;
        if c == 0 && e.state_core().turn.phase == engine_native::event::Phase::Play {
            break;
        }
        let _ = submit(&mut e, c, GameAction::NextPhase);
    }
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        if p != 0 {
            e.debug_clear_hand(p);
        }
    }
    let sha2 = e.debug_place(0, "sha");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![1],
        },
    );
    settle_optional_confirm(&mut e);
    assert!(e.state_core().player(1).alive, "鬼眼复活：p1 存活");
    assert_eq!(e.debug_hp(1), 1, "鬼眼复活：hp 回至 1");
    assert!(
        !e.state_core().skills.contains_key(&(1, g2)),
        "鬼眼复活：复活面实例已移除"
    );
    assert!(
        !e.state_core().skills.contains_key(&(1, def_gy)),
        "鬼眼复活：鬼眼本体实例已移除"
    );
}
