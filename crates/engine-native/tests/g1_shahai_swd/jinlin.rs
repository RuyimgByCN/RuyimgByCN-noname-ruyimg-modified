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

    // A2 H30：锦鳞·退上游 player phaseBegin＝持有者**回合开始**（每回合
    // 一次；原锚任意自己阶段开始＝每回合 5→6 次）——阶段推进不再递减，
    // 须经真实回合交接（debug_set_turn 不发 TurnBegin，改自然导航）。
    // 前进到 seat 下一回合开始（引擎停在 Judge 帧；TurnBegin 窗已派发）。
    fn advance_to_next_turn_of(e: &mut Engine, seat: u8) {
        // 先离开 seat 当前回合
        while e.turn().current == seat {
            drain_choices(e);
            let actor = e.turn().current;
            let _ = submit(e, actor, GameAction::NextPhase);
        }
        for _ in 0..40 {
            drain_choices(e);
            if e.turn().current == seat && e.turn().phase == Phase::Judge {
                return;
            }
            let actor = e.turn().current;
            let _ = submit(e, actor, GameAction::NextPhase);
        }
        panic!("40 步内未到达 {seat} 号下一回合开始");
    }

    // 同回合阶段推进不递减；他人回合开始不递减
    drain_choices(&mut e);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    drain_choices(&mut e);
    assert_eq!(e.state_core().player(0).hujia, 3, "同回合阶段不再递减（A2 H30）");
    assert_eq!(rounds(&e, 0), Some(3), "计数不变");

    // 1 号自己的回合开始才递减
    advance_to_next_turn_of(&mut e, 1);
    drain_choices(&mut e);
    assert_eq!(e.state_core().player(1).hujia, 2, "1 号回合开始 -1");
    assert_eq!(rounds(&e, 1), Some(2));
    assert_eq!(e.state_core().player(0).hujia, 3, "他人回合不递减");

    // 0 号每回合开始 -1，三次归零：计数与护甲同时归零、实例移除
    advance_to_next_turn_of(&mut e, 0);
    drain_choices(&mut e);
    assert_eq!(e.state_core().player(0).hujia, 2);
    assert_eq!(rounds(&e, 0), Some(2));
    advance_to_next_turn_of(&mut e, 0);
    drain_choices(&mut e);
    assert_eq!(e.state_core().player(0).hujia, 1);
    assert_eq!(rounds(&e, 0), Some(1));
    advance_to_next_turn_of(&mut e, 0);
    drain_choices(&mut e);
    assert_eq!(e.state_core().player(0).hujia, 0, "护甲归零");
    assert_eq!(rounds(&e, 0), None, "计数归零移除实例");
    // 反例：移除后继续推进不再递减（护甲不下溢）
    advance_to_next_turn_of(&mut e, 0);
    drain_choices(&mut e);
    assert_eq!(e.state_core().player(0).hujia, 0, "移除后不再递减");
}
