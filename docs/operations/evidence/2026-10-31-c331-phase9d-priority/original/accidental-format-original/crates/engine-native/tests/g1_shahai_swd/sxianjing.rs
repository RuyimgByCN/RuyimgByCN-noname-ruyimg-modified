use super::*;

/// T12 swd deferred 消解批 111：陷阱（舒莞儿 `swd_shuwaner`）——置出同花色
/// 陷阱牌；他人以该花色指定你时弃陷阱并随机获得其一张牌；你受伤时取回。
#[test]
fn swd_deferred_sxianjing_trap_cycle() {
    use engine_native::catalog::Suit;
    use engine_native::event::DamageNature;
    let root = repo_extensions_root();
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

    let mut e = Engine::new_with_extensions(62072, 4, &[&pack]).expect("建局");
    let def = e.catalog().skill_by_name("sxianjing").expect("sxianjing");
    for sk in ["sxianjing", "sxianjing_gain", "sxianjing_damage"] {
        e.debug_grant(1, sk);
    }
    let n = e.catalog().cards.len() as u32;
    let spade_trap = (0..n)
        .find(|c| e.catalog().cards[*c as usize].suit == Suit::Spade)
        .expect("黑桃");
    // 场景一：1 号出牌阶段发动陷阱，置出黑桃牌。
    advance_to_play(&mut e, 1);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    inject_hand_at(&mut e, &pack, 1, &[spade_trap]);
    let out = submit(&mut e, 1, GameAction::ActivateSkill { skill: def });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    for _ in 0..8 {
        let Some(c) = e.open_choice() else { break };
        let (cid, obs) = (c.id, c.observer);
        let sel = match c.spec.prompt {
            "sxianjing_pick" => vec![ChoiceOption::Card(spade_trap)],
            other => panic!("未预期询问 {other}"),
        };
        let _ = submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: sel,
            },
        );
    }
    assert_eq!(
        e.state_core().player(1).sxianjing_traps,
        vec![spade_trap],
        "陷阱：置入一张陷阱牌"
    );
    let restored = Engine::restore_with_extensions(&e.save(), &[&pack]).expect("陷阱存档恢复");
    assert_eq!(
        restored.state_core().player(1).sxianjing_traps,
        vec![spade_trap]
    );
    assert_eq!(restored.state_hash(), e.state_hash());
    e = restored;

    // 场景二：0 号回合用黑桃杀指定 1 号 → 触发陷阱（陷阱直接置入）。
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    assert_eq!(e.state_core().players[1].sxianjing_traps, vec![spade_trap]);
    let spade_sha = (0..n)
        .find(|c| {
            *c != spade_trap
                && e.catalog().cards[*c as usize].suit == Suit::Spade
                && e.catalog().cards[*c as usize].kind == CardKind::Sha
        })
        .expect("黑桃杀");
    let give = e.debug_place(0, "tao");
    inject_hand_at(&mut e, &pack, 0, &[spade_sha, give]);
    let out = play(&mut e, spade_sha, vec![1]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    for _ in 0..8 {
        let Some(c) = e.open_choice() else { break };
        let sel: Vec<ChoiceOption> = if c.spec.min == 0 || c.spec.allow_cancel {
            vec![ChoiceOption::Pass]
        } else {
            c.spec
                .options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Card(_)))
                .copied()
                .take(c.spec.min.max(1))
                .collect()
        };
        let (cid, obs) = (c.id, c.observer);
        let _ = submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: sel,
            },
        );
    }
    assert!(
        e.state_core().player(1).sxianjing_traps.is_empty(),
        "陷阱：触发后弃置"
    );
    assert!(e.debug_hand(1).contains(&give), "陷阱：获得使用者一张牌");

    // 受伤取回：直接置入陷阱后受伤 → 取回。
    let core = e.state_core_mut();
    let pos = core
        .discard
        .iter()
        .position(|card| *card == spade_trap)
        .unwrap();
    core.discard.remove(pos);
    core.players[1].sxianjing_traps.push(spade_trap);
    let _ = e.debug_deal_damage(None, 1, 1, DamageNature::Normal);
    assert!(
        e.state_core().player(1).sxianjing_traps.is_empty(),
        "陷阱：受伤取回"
    );
    assert!(e.debug_hand(1).contains(&spade_trap), "陷阱：陷阱牌回手");
}
