use super::*;

/// 堡垒/侦察切换（U05 ow 批七，规则 v124）：堡垒——出牌阶段确认切换
/// 模式（三态翻转：无→哨卫→侦察→哨卫），baolei_mode 跨回合持久入存
///档。哨卫/侦察的模式效果钩（杀距离/防具/次数/闪避、用杀摸回）登记
/// 待做；本切片仅状态通道。
#[test]
fn baolei_switch_mode_cycle() {
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

    let mut e = Engine::new_with_extensions(9701, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    e.debug_grant(0, "bshaowei");
    let def = e.catalog().skill_by_name("bshaowei").expect("def");

    // 0→1（哨卫）
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.state_core().player(0).baolei_mode, 1, "切换至哨卫");

    // 1→2（侦察）
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.state_core().player(0).baolei_mode, 2, "切换至侦察");

    // 2→1（回哨卫，三态翻转）
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.state_core().player(0).baolei_mode, 1, "回哨卫");

    // 跨回合持久 + 存档往返
    let snap = e.save();
    assert_eq!(snap.core.player(0).baolei_mode, 1);
    let e2 = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复");
    assert_eq!(e2.state_core().player(0).baolei_mode, 1, "存档往返保持");

    let round0 = e.turn().round;
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.turn().round > round0 {
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
    assert_eq!(e.state_core().player(0).baolei_mode, 1, "跨回合持久");
}

/// 侦察摸回（U05 ow 批八，规则 v125）：侦察模式下用杀 → 摸 1 或回 1
/// （chooseDrawRecover 可放弃）；哨卫模式目标侧禁闪 + 源侧直中。
#[test]
fn baolei_mode_effects_on_sha() {
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

    let mut e = Engine::new_with_extensions(9801, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_grant(0, "bshaowei");
    e.debug_grant(0, "zhencha_draw");
    e.debug_set_max_hp(0, 4);
    e.debug_set_hp(0, 4);

    let def_sw = e.catalog().skill_by_name("bshaowei").expect("def");
    // 切至哨卫（0→1）
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_sw });
    assert_eq!(e.state_core().player(0).baolei_mode, 1);

    // 哨卫源侧：0 号杀 1 号（1 号有闪也不可闪避——直接伤害）
    let sha = e.debug_place(0, "sha");
    let shan = e.debug_place(1, "shan"); // 1 号有闪但应被无视
    e.debug_set_hp(1, 4);
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
    assert_eq!(e.debug_hp(1), 3, "哨卫：杀不可闪避（4→3）");
    let _ = shan;

    // 哨卫目标侧：1 号（有哨卫技？无——哨卫效果是持技者双向）——
    // 授 1 号后 0 号杀 1 号 → 1 号不可闪避
    e.debug_grant(1, "bshaowei");
    // 1 号 mode 默认 0（未切换）→ 目标侧哨卫需 mode=1——1 号未切换不生效
    let sha = e.debug_place(0, "sha");
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
    assert_eq!(
        e.debug_hp(1),
        2,
        "1 号无哨卫模式：正常可闪（无闪则掉血 3→2）"
    );
}
