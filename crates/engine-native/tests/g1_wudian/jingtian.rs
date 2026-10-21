use super::*;

// ---------------------------------------------------------------------------
// U05 xianjian 第十七切片（规则 v158）：景天（pal_jingtian，wu 3 男）洒
// 金（sajin，出牌阶段限一次主动）+聚宝（jtjubao，global 弃牌 frequent）
// ——弃一张手牌指定任意名已受伤角色各判定，同色者回复 1；他人回合外
// 弃非基本牌入弃堆时（本回合首次）随机回收一张。
// ---------------------------------------------------------------------------

/// 洒金：弃牌→单目标判定同色回复 1；异色不回复；每回合限一次。
#[test]
fn jingtian_sajin_judge_color_recover_and_once() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    use engine_native::catalog::Suit;
    let same_color_top = |e: &Engine, card: u32| {
        let red = matches!(
            e.catalog().cards[card as usize].suit,
            Suit::Heart | Suit::Diamond
        );
        move |d: &engine_native::catalog::CardDef| {
            let dred = matches!(d.suit, Suit::Heart | Suit::Diamond);
            dred == red
        }
    };

    let mut e = Engine::new_with_extensions(10019, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 2); // 已受伤
    e.debug_set_hp(2, 4); // 满血（不可指定）
    e.debug_grant(0, "sajin");
    let disc = e.debug_place(0, "wuzhong");

    // ① 发动→弃 wuzhong→指定 1 号→判定同色→回复 1（2→3）
    let _ = e.debug_set_deck_top_by(same_color_top(&e, disc));
    let def = e.catalog().skill_by_name("sajin").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(disc)]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let c = e.open_choice().expect("洒金目标挂起");
    assert_eq!(
        (c.spec.prompt, c.observer),
        (engine_native::content::host::WUDIAN_XJ_SAJIN_TARGETS, 0)
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
    assert_eq!(e.debug_hp(1), 3, "同色判定：回复 1（2→3）");
    assert_eq!(e.debug_hp(2), 4, "满血者不可指定");
    assert!(e.state_core().discard.contains(&disc), "所弃牌入弃堆");

    // ② 每回合限一次：本回合再发动被拒
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Rejected(_)), "{out:?}");

    // ③ 推进到 0 号下回合：异色判定不回复
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    for _ in 0..5 {
        let _ = submit(&mut e, 1, GameAction::NextPhase); // 1 回合走完（Judge 起 5 次）
    }
    for _ in 0..5 {
        let _ = submit(&mut e, 2, GameAction::NextPhase); // 2 回合走完
    }
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Draw→Play
    let disc2 = e.debug_place(0, "wuzhong");
    let opposite = |e: &Engine, card: u32| {
        let red = matches!(
            e.catalog().cards[card as usize].suit,
            Suit::Heart | Suit::Diamond
        );
        move |d: &engine_native::catalog::CardDef| {
            let dred = matches!(d.suit, Suit::Heart | Suit::Diamond);
            dred != red
        }
    };
    let _ = e.debug_set_deck_top_by(opposite(&e, disc2));
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let _out = answer_open(&mut e, vec![ChoiceOption::Card(disc2)]);
    let c = e.open_choice().expect("洒金目标挂起②");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    assert_eq!(e.debug_hp(1), 3, "异色判定：不回复（3→3）");
}

/// 聚宝：他人回合外弃非基本牌→随机回收一张（本回合首次）；自己弃牌
/// 不触发；换回合门控复位。
#[test]
fn jingtian_jtjubao_gains_nonbasic_and_gates() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    use engine_native::catalog::CardKind as K;

    let mut e = Engine::new_with_extensions(10020, 4, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "jtjubao");
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 1); // 手牌上限 1 → 弃牌阶段弃 3

    // 0 号过回合 → 1 号回合：摸 2 后手牌 4，弃 3（保证含非基本）
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    let wz = e.debug_place(1, "wuzhong");
    let tao = e.debug_place(1, "tao");
    let hand1 = e.debug_hand(1);
    assert!(hand1.contains(&wz) && hand1.contains(&tao), "布阵在手");
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Play→Discard
    let c = e.open_choice().expect("弃牌阶段挂起");
    assert_eq!(c.observer, 1);
    let keep = *c
        .spec
        .options
        .iter()
        .find(|o| matches!(o, ChoiceOption::Card(x) if *x != wz))
        .expect("留牌项");
    let (cid, obs) = (c.id, c.observer);
    let thrown: Vec<ChoiceOption> = e
        .debug_hand(1)
        .into_iter()
        .filter(|x| *x != wz)
        .map(ChoiceOption::Card)
        .collect();
    let _ = keep;
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: thrown,
        },
    );

    // 聚宝自动回收：0 号手牌 +1，且为弃入弃堆的非基本牌之一
    assert_eq!(e.debug_hand(0).len(), 1, "回收 1 张");
    let gained = e.debug_hand(0)[0];
    assert_ne!(gained, tao, "非基本候选（桃=基本不入池）");
    assert!(
        matches!(
            e.catalog().cards[gained as usize].kind,
            K::Wuzhong
                | K::Snatch
                | K::Dismantle
                | K::Duel
                | K::Nullify
                | K::Chain
                | K::Le
                | K::Bing
                | K::ShanDian
                | K::Taoyuan
                | K::Wugu
                | K::Jiedao
                | K::SavageAttack
                | K::ArcheryAttack
        ),
        "回收=非基本牌（{:?}）",
        e.catalog().cards[gained as usize].kind
    );
    assert!(e.state_core().turn.jtjubao_used, "本回合已回收门控置位");

    // 0 号自己回合弃牌：不触发（player==owner 门控）
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 1, GameAction::NextPhase); // End→2 Judge
    for _ in 0..5 {
        let _ = submit(&mut e, 2, GameAction::NextPhase); // 2 回合走完（Judge 起 5 次）
    }
    for _ in 0..5 {
        let _ = submit(&mut e, 3, GameAction::NextPhase); // 3 回合走完
    }
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Draw→Play
    assert!(!e.state_core().turn.jtjubao_used, "换回合门控复位");
    e.debug_set_hp(0, 1); // 手牌上限 1
    let extra = e.debug_place(0, "wuzhong");
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let c = e.open_choice().expect("0 号弃牌阶段挂起");
    let (cid, obs) = (c.id, c.observer);
    let keep0 = ChoiceOption::Card(gained);
    let thrown0: Vec<ChoiceOption> = e
        .debug_hand(0)
        .into_iter()
        .filter(|x| *x != gained)
        .map(ChoiceOption::Card)
        .collect();
    assert!(!thrown0.is_empty());
    let _ = keep0;
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: thrown0,
        },
    );
    assert_eq!(e.debug_hand(0).len(), 1, "弃至上限（保留回收牌）");
    assert!(!e.state_core().turn.jtjubao_used, "自己弃牌不触发");
    assert!(
        e.state_core().discard.contains(&extra),
        "所弃非基本在弃堆未回收"
    );
}
