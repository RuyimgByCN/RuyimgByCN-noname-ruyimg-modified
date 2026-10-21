use super::*;

/// T12 swd deferred 消解批 114：锦鳞（司空蔷 `swd_situqiang`）——limited 一次性
/// 主动技：选任意名角色各 +3 护甲并授予「锦鳞·退」（每阶段开始 -1 护甲、
/// 计数 -1；护甲归零或计数归零时移除；计数 3）。
#[test]
fn swd_deferred_jinlin_limited_hujia_and_phase_ticks() {
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
    let mut e = Engine::new_with_extensions(64114, 4, &[&pack]).expect("建局");
    let def = e.catalog().skill_by_name("jinlin").expect("jinlin 在目录");
    let j2 = e
        .catalog()
        .skill_by_name("swd_jinlin2")
        .expect("swd_jinlin2 在目录");
    e.debug_grant(0, "jinlin");
    advance_to_play(&mut e, 0);

    // 发动：选自己与 1 号
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let choice = e.open_choice().expect("锦鳞目标窗").clone();
    assert_eq!(choice.spec.prompt, "jinlin_targets");
    assert_eq!(choice.spec.min, 1);
    let out = submit(
        &mut e,
        choice.observer,
        GameAction::Answer {
            choice_id: choice.id,
            selection: vec![ChoiceOption::Player(0), ChoiceOption::Player(1)],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.state_core().player(0).hujia, 3, "0 号 +3 护甲");
    assert_eq!(e.state_core().player(1).hujia, 3, "1 号 +3 护甲");
    let rounds = |e: &Engine, p: u8| e.state_core().skills.get(&(p, j2)).map(|i| i.rounds);
    assert_eq!(rounds(&e, 0), Some(3), "计数 3");
    assert_eq!(rounds(&e, 1), Some(3), "计数 3");
    assert_eq!(e.state_core().player(2).hujia, 0, "未选目标不获护甲");
    assert!(
        !e.state_core().skills.contains_key(&(2, j2)),
        "未选目标不获技"
    );

    // 反例：limited 一次性——再次发动被拒
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::InvalidChoice)),
        "锦鳞反例：已发动不可再发动 {out:?}"
    );

    // 0 号阶段开始递减；他人阶段不递减
    drain_choices(&mut e);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain_choices(&mut e);
    assert_eq!(e.state_core().player(0).hujia, 2, "阶段开始 -1 护甲");
    assert_eq!(rounds(&e, 0), Some(2), "阶段开始 -1 计数");
    assert_eq!(e.state_core().player(1).hujia, 3, "他人阶段不递减");

    // 1 号自己的阶段开始才递减
    e.debug_set_turn(1, Phase::Judge, e.turn().round);
    drain_choices(&mut e);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    for _ in 0..6 {
        let Some(c) = e.open_choice() else { break };
        let (cid, obs) = (c.id, c.observer);
        let _ = submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    drain_choices(&mut e);
    assert_eq!(e.state_core().player(1).hujia, 2, "1 号阶段开始 -1");
    assert_eq!(rounds(&e, 1), Some(2));

    // 0 号两次阶段开始 → 计数与护甲同时归零、实例移除
    e.debug_set_turn(0, Phase::Judge, e.turn().round);
    drain_choices(&mut e);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // → Draw
    drain_choices(&mut e);
    assert_eq!(e.state_core().player(0).hujia, 1);
    assert_eq!(rounds(&e, 0), Some(1));
    let _ = submit(&mut e, 0, GameAction::NextPhase); // → Play
    drain_choices(&mut e);
    assert_eq!(e.state_core().player(0).hujia, 0, "护甲归零");
    assert_eq!(rounds(&e, 0), None, "计数归零移除实例");
    // 反例：移除后继续推进不再递减（护甲不下溢）
    let _ = submit(&mut e, 0, GameAction::NextPhase); // → Discard
    drain_choices(&mut e);
    assert_eq!(e.state_core().player(0).hujia, 0, "移除后不再递减");
}
