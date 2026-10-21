use super::*;

/// T12 swd deferred 消解批 109：聚悉（小黄龙 `swd_xiaohuanglong`）——计数达
/// 存活数时可发动：扣存活数并选一名角色（已受伤则自选受伤/回复，否则受伤）。
#[test]
fn swd_deferred_juxi_damage_or_recover() {
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

    // 未受伤目标：发动 → 造成 1 伤。
    let mut e = Engine::new_with_extensions(62069, 4, &[&pack]).expect("建局");
    let def = e.catalog().skill_by_name("juxi").expect("juxi 入目录");
    e.debug_grant(0, "juxi");
    e.debug_grant(0, "juxi_count");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.state_core_mut().players[0].juxi_count = 4;
    e.debug_set_hp(1, 4);
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
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
            "juxi_target" => vec![ChoiceOption::Player(1)],
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
    assert_eq!(e.debug_hp(1), 3, "聚悉：未受伤目标受 1 伤（4→3）");
    assert_eq!(e.state_core().player(0).juxi_count, 0, "聚悉：扣存活数 4→0");

    // 已受伤目标：发动 → 自选回复。
    let mut e = Engine::new_with_extensions(62070, 4, &[&pack]).expect("建局2");
    e.debug_grant(0, "juxi");
    e.debug_grant(0, "juxi_count");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.state_core_mut().players[0].juxi_count = 4;
    e.debug_set_hp(1, 2);
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
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
            "juxi_target" => vec![ChoiceOption::Player(1)],
            "juxi_choice" => vec![ChoiceOption::Tag(1)],
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
    assert_eq!(e.debug_hp(1), 3, "聚悉：已受伤目标自选回复（2→3）");
}
