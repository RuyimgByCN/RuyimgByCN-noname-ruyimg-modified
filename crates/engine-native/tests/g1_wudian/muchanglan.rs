use super::*;

// ---------------------------------------------------------------------------
// U05 xianjian 第十三切片（规则 v154）：慕长兰（pal_muchanglan）掠影
//（lueying，ShaResolve 内联钩每回合限一次）——出杀结算开始：获得目标
// he 随机一张；另有其他有牌者→目标选人、来源随机弃其一张。feixia（虚
// 拟用牌）登记暂缓。
// ---------------------------------------------------------------------------

/// 掠影：获得目标随机牌+目标选人→来源随机弃其一张；每回合限一次。
#[test]
fn muchanglan_lueying_gain_and_victim_discard() {
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

    let mut e = Engine::new_with_extensions(10011, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "lueying");
    // 0 号 1 张（杀），1 号 2 张（目标），2 号 1 张（受害者候选）
    let sha0 = e.debug_place(0, "sha");
    let t1 = e.debug_place(1, "tao");
    e.debug_place(1, "shan");
    e.debug_place(2, "tao");
    e.debug_set_hp(1, 4);

    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha0,
            targets: vec![1],
        },
    );
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    // 掠影先于闪询问：确认门（observer=0，上游非强制）→ Confirm →
    // 目标选人询问（observer=1）
    let c = e.open_choice().cloned().expect("掠影确认门");
    assert_eq!(c.observer, 0, "确认门 observer=使用者（非强制发动）");
    drop(c);
    let out = answer_open(&mut e, vec![ChoiceOption::Confirm]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let c = e.open_choice().cloned().expect("掠影选人询问");
    assert_eq!(c.observer, 1, "选人询问 observer=目标");
    drop(c);
    let out = answer_open(&mut e, vec![ChoiceOption::Player(2)]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 0 号获得 1 号一张牌（手牌 1→2）；2 号被随机弃 1（1→0）
    assert_eq!(
        e.debug_hand(0).len(),
        1,
        "来源获得目标一张牌（杀已离手 0+1=1）"
    );
    assert_eq!(e.debug_hand(2).len(), 0, "受害者被随机弃 1");
    assert!(e.state_core().turn.lueying_used, "本回合已发动");

    // 出闪询问（1 号无闪）→杀照常
    while let Some(c) = e.open_choice() {
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
    assert_eq!(e.debug_hp(1), 3, "杀照常（4→3）");
    let _ = t1;
}

/// 掠影 v194 补齐①：无第三名有牌者→整个技能不可发动（不询问、不获得、
/// 不消耗次数）——上游 filter 要求"另有一名其他有牌角色"。
#[test]
fn muchanglan_lueying_no_third_party_never_fires() {
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

    let mut e = Engine::new_with_extensions(10071, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "lueying");
    // 目标 1 号有牌；2/3/4 号全空手——无第三名有牌者
    let sha0 = e.debug_place(0, "sha");
    e.debug_place(1, "tao");
    e.debug_set_hp(1, 4);

    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha0,
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
    // 首个询问直接是 1 号出闪/无懈等结算询问——绝无使用者确认门
    if let Some(c) = e.open_choice() {
        assert_ne!(
            c.observer, 0,
            "无第三者时不得出现掠影确认门: {}",
            c.spec.prompt
        );
        assert_ne!(c.spec.prompt, "lueying_confirm");
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
    while let Some(c) = e.open_choice() {
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
    assert_eq!(e.debug_hand(0).len(), 0, "无第三者：来源不获得任何牌");
    assert!(!e.state_core().turn.lueying_used, "技能未发动，不消耗次数");
    assert_eq!(e.debug_hp(1), 3, "杀照常（4→3）");
}

/// 掠影 v194 补齐②：确认门放弃→不获得、不消耗 usable；本回合再出杀
/// 仍可再次询问（上游非强制+usable:1 口径）。
#[test]
fn muchanglan_lueying_decline_keeps_usable_for_next_sha() {
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

    let mut e = Engine::new_with_extensions(10072, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "lueying");
    e.debug_grant(0, "咆哮"); // 出牌阶段杀不限次，便于同回合第二杀
    let sha1 = e.debug_place(0, "sha");
    let sha2 = e.debug_place(0, "sha");
    e.debug_place(1, "tao");
    e.debug_place(2, "tao");
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);

    // 第一杀：确认门→放弃（空选择）→不获得、不消耗
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1,
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
    let c = e.open_choice().cloned().expect("掠影确认门");
    assert_eq!(c.observer, 0, "确认门 observer=使用者");
    assert_eq!(c.spec.prompt, "lueying_confirm");
    drop(c);
    let out = answer_open(&mut e, vec![]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.debug_hand(0).len(), 1, "放弃：来源无获得（剩第二张杀）");
    assert!(!e.state_core().turn.lueying_used, "放弃不消耗 usable");
    while let Some(c) = e.open_choice() {
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
    assert_eq!(e.debug_hp(1), 3, "第一杀照常（4→3）");

    // 第二杀（仍打相邻的 1 号）：确认门再次出现（usable 未被放弃消耗）
    // → Confirm → 获得目标牌 → 目标选人（受害者=2 号）→ 来源弃其一张
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha2,
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
    let c = e.open_choice().cloned().expect("第二杀确认门应再次出现");
    assert_eq!(c.observer, 0, "确认门 observer=使用者");
    assert_eq!(c.spec.prompt, "lueying_confirm");
    drop(c);
    let out = answer_open(&mut e, vec![ChoiceOption::Confirm]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert!(e.state_core().turn.lueying_used, "确认后消耗 usable");
    assert_eq!(
        e.debug_hand(0).len(),
        1,
        "获得 1 号一张牌（杀已离手 0+1=1）"
    );
    let c = e.open_choice().cloned().expect("选人相");
    assert_eq!(c.observer, 1, "选人询问 observer=目标（1 号）");
    drop(c);
    let out = answer_open(&mut e, vec![ChoiceOption::Player(2)]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.debug_hand(2).len(), 0, "受害者（2 号）被随机弃 1");
    while let Some(c) = e.open_choice() {
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
    assert_eq!(e.debug_hp(1), 2, "第二杀照常（3→2）");
}
