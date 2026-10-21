use super::*;

/// T12 swd deferred 消解批 102：热血（墨耶/夜鸦熙）+ 流离内置引用（墨耶）。
#[test]
fn swd_deferred_rexue_and_liuli() {
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

    // 流离：墨耶的技能表含内置引用「流离」（标准目录实现）。
    let moye = pack
        .generals
        .iter()
        .find(|g| g.name == "swd_moye")
        .expect("swd_moye 在包内");
    assert!(
        moye.skills.contains(&"流离"),
        "墨耶技能含内置引用流离：{:?}",
        moye.skills
    );

    // 热血：0 号持热血 + 手牌杀；1 号阶段开始 → 0 号视为对其使用杀并摸 1。
    let mut e = Engine::new_with_extensions(62060, 4, &[&pack]).expect("建局");
    e.debug_grant(0, "rexue");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _sha = e.debug_place(0, "sha");
    e.debug_set_hp(1, 4);
    // 推进到 1 号阶段开始（0 号 Play→Discard→End→收束 → 1 号 Judge begin）
    for _ in 0..8 {
        if e.turn().current == 1 {
            break;
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
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
    }
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
    assert_eq!(e.debug_hp(1), 3, "热血：虚拟杀命中 1 号（4→3）");
    assert_eq!(e.debug_hand(0).len(), 2, "热血：使用后摸 1（杀+1）");
}
