use super::*;

/// T12 swd deferred 消解批 112：孤行（慕容诗 `swd_murongshi`）——手牌数不大于
/// max(1,已损失体力) 时，将全部手牌当杀使用（1–3 目标）；结算后摸伤害数。
#[test]
fn swd_deferred_guxing_all_hand_sha_and_draw() {
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

    let mut e = Engine::new_with_extensions(62073, 4, &[&pack]).expect("建局");
    let def = e.catalog().skill_by_name("guxing").expect("guxing");
    e.debug_grant(0, "guxing");
    e.debug_grant(0, "guxing3");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号 hp 2/4 → 已损失 2 → 上限 2；手牌 1 张。
    e.debug_set_hp(0, 2);
    let _c = e.debug_place(0, "tao");
    e.debug_set_hp(1, 4);
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    for _ in 0..12 {
        let Some(c) = e.open_choice() else { break };
        let (cid, obs) = (c.id, c.observer);
        let sel = match c.spec.prompt {
            "guxing_targets" => vec![ChoiceOption::Player(1)],
            other => {
                // 杀响应/其他询问：放弃
                let _ = other;
                vec![ChoiceOption::Pass]
            }
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
    assert_eq!(e.debug_hp(1), 3, "孤行：全部手牌当杀命中 1 号（4→3）");
    assert_eq!(e.debug_hand(0).len(), 1, "孤行：结算后摸伤害数（1）");
}
