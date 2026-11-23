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

    // 热血：0 号持热血 + 手牌杀；1 号回合开始 → 可选询问使用杀（消耗真实杀）并在使用时摸 1。
    let mut e = Engine::new_with_extensions(62060, 4, &[&pack]).expect("建局");
    e.debug_grant(0, "rexue");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = e.debug_place(0, "sha");
    e.debug_set_hp(1, 4);
    let mut saw_rexue_sha = false;
    // 推进到 1 号回合并响应 0 号「热血」可选询问（Confirm + 选择真实杀）
    for _ in 0..24 {
        if let Some(c) = e.open_choice().cloned() {
            let sel: Vec<ChoiceOption> = match (c.observer, c.spec.prompt) {
                (0, "trigger_confirm") => vec![ChoiceOption::Confirm],
                (0, "rexue_sha") => {
                    saw_rexue_sha = true;
                    assert!(c.spec.options.contains(&ChoiceOption::Card(sha)));
                    vec![ChoiceOption::Card(sha)]
                }
                _ if c.spec.options.contains(&ChoiceOption::Pass) => vec![ChoiceOption::Pass],
                _ if c.spec.min == 0 || c.spec.allow_cancel => Vec::new(),
                _ => c
                    .spec
                    .options
                    .iter()
                    .filter(|o| matches!(o, ChoiceOption::Card(_)))
                    .copied()
                    .take(c.spec.min.max(1))
                    .collect(),
            };
            let _ = submit(&mut e, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
            continue;
        }
        if e.turn().current == 1 && saw_rexue_sha {
            break;
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }
    assert!(saw_rexue_sha, "应弹出 rexue_sha 询问");
    assert_eq!(e.debug_hp(1), 3, "热血：实牌杀命中 1 号（4→3）");
    assert!(!e.debug_hand(0).contains(&sha), "热血：消耗手牌真实杀");
    assert_eq!(e.debug_hand(0).len(), 1, "热血：消耗 1 张杀并使用时摸 1 张（净 1 张）");
}
