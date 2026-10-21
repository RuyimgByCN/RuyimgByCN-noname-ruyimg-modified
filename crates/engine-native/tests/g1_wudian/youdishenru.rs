use super::*;

/// 诱敌深入（响应转化原语）：成为杀目标时可打出——杀无效，
/// 来源受 1 点反噬伤害（登记差异：无上游连用循环）。
#[test]
fn youdishenru_cancels_sha_with_reprisal() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = {
        let discovered = discover_extensions(&root).unwrap();
        discovered
            .into_iter()
            .find(|d| d.name == "wudian-upstream-noname")
            .expect("wudian 可发现")
    };
    let json = std::fs::read_to_string(meta.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(4201, 3, &[&pack]).expect("建局");
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let cid = c.id;
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            let obs = c.observer;
            match submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: sel,
                },
            ) {
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                other => panic!("answer: {other:?}"),
            }
            continue;
        }
        let actor = e.turn().current;
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    // 0 号（攻击方）手牌：杀；1 号（防御方）手牌：诱敌深入
    let (sha, yd) = {
        let catalog = e.catalog();
        let sh = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Sha)
            .expect("杀在目录") as u32;
        let y = catalog
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Youdishenru)
            .expect("诱敌深入在目录") as u32;
        (sh, y)
    };
    let mut snap = e.save();
    for p in snap.core.players.iter_mut() {
        p.hand.retain(|c| *c != sha && *c != yd);
    }
    snap.core.deck.retain(|c| *c != sha && *c != yd);
    snap.core.discard.retain(|c| *c != sha && *c != yd);
    snap.core.players[0].hand.push(sha);
    snap.core.players[1].hand.push(yd);
    e.debug_set_hp(1, 4);
    e.debug_set_hp(0, 4);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    // 0 号出杀 → 1 号以诱敌深入响应
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let c = e.open_choice().expect("响应询问挂起");
    assert_eq!(c.spec.prompt, "respond_shan");
    assert!(
        c.spec.options.contains(&ChoiceOption::Card(yd)),
        "响应选项应含诱敌深入"
    );
    let cid = c.id;
    submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(yd)],
        },
    );
    // 杀无效：1 号体力不变；来源受 1 点反噬
    assert_eq!(e.debug_hp(1), 4, "诱敌深入应使杀无效");
    assert_eq!(e.debug_hp(0), 3, "来源受 1 点反噬伤害");
    assert!(!e.debug_hand(1).contains(&yd), "诱敌深入已打出");
}
