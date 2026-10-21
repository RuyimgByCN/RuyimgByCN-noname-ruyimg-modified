use super::*;

/// 光书（U07 ow 批二，规则 v119）：虚无之光——出牌阶段弃一张牌指定一
/// 名角色，按花色授子技：♥受伤后回 1（一次性）/♠下阶段结束受 1 无源
/// 雷伤（一次性）/♣不能用杀至下阶段结束。已有子技者不可再指定。
#[test]
fn zhixuzhiguang_guangshu_suit_marks() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9201, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_grant(0, "guangshu");
    e.debug_set_max_hp(0, 4);
    e.debug_set_hp(0, 3);

    // ① ♠光塔：弃黑桃 → 指定 1 号 → 1 号下阶段结束受 1 雷伤（4→3）
    // 注入真黑桃杀（debug_place 按名取首张，花色不定）
    let spade = (0..e.catalog().cards.len() as u32)
        .find(|i| {
            e.catalog().cards[*i as usize].name == "sha"
                && e.catalog().cards[*i as usize].suit == engine_native::catalog::Suit::Spade
        })
        .expect("黑桃杀在目录");
    {
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != spade);
        }
        snap.core.deck.retain(|c| *c != spade);
        snap.core.discard.retain(|c| *c != spade);
        snap.core.table.retain(|c| *c != spade);
        snap.core.players[0].hand.insert(0, spade);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("黑桃杀注入");
    }
    let def_gs = e.catalog().skill_by_name("guangshu").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_gs });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let c = e.open_choice().expect("光书弃牌挂起");
    assert_eq!(
        (c.spec.prompt, c.observer),
        (engine_native::content::host::WUDIAN_GUANGSHU_CARD, 0)
    );
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(spade)],
        },
    );
    let c = e.open_choice().expect("光书目标挂起");
    assert_eq!(
        (c.spec.prompt, c.observer),
        (engine_native::content::host::WUDIAN_GUANGSHU_TARGET, 0)
    );
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    let spade_def = e.catalog().skill_by_name("guangshu_spade").expect("def");
    assert!(e.state_core().has_skill(1, spade_def), "♠子技已授予");
    // 推进：任一阶段结束触发光塔
    let hp1 = e.debug_hp(1);
    for _ in 0..8 {
        if e.debug_hp(1) < hp1 {
            break;
        }
        if let Some(c) = e.open_choice() {
            let (cid, obs) = (c.id, c.observer);
            submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: vec![ChoiceOption::Pass],
                },
            );
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }
    assert_eq!(e.debug_hp(1), hp1 - 1, "♠光塔：阶段结束受 1 雷伤");
}
