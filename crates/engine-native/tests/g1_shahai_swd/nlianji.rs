use super::*;

/// T12 swd deferred 消解批 105：拼点（独孤宁珂 `swd_duguningke`）——弃一张
/// 手牌，令两名角色拼点：赢者获得该牌、输者受赢者 1 伤。
#[test]
fn swd_deferred_nlianji_compare_reward_and_damage() {
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

    let mut e = Engine::new_with_extensions(62064, 4, &[&pack]).expect("建局");
    let def = e
        .catalog()
        .skill_by_name("nlianji")
        .expect("nlianji 入目录");
    e.debug_grant(0, "nlianji");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let throw_card = e.debug_place(0, "tao");
    let n = e.catalog().cards.len() as u32;
    let high = (0..n)
        .find(|c| e.catalog().cards[*c as usize].rank == 13)
        .expect("rank13 牌");
    let low = (0..n)
        .find(|c| e.catalog().cards[*c as usize].rank == 1)
        .expect("rank1 牌");
    inject_hand_at(&mut e, &pack, 1, &[high]);
    inject_hand_at(&mut e, &pack, 2, &[low]);
    e.debug_set_hp(2, 4);

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
            "nlianji_throw" => vec![ChoiceOption::Card(throw_card)],
            "nlianji_target0" => vec![ChoiceOption::Player(1)],
            "nlianji_target1" => vec![ChoiceOption::Player(2)],
            "nlianji_t0_pick" => vec![ChoiceOption::Card(high)],
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
    // 1 号（高 rank）赢：获得 0 号弃出的桃；2 号（低 rank）受 1 号 1 伤。
    assert!(e.debug_hand(1).contains(&throw_card), "赢者获得弃出牌");
    assert_eq!(e.debug_hp(2), 3, "输者受赢者 1 伤（4→3）");
    assert!(
        !e.debug_hand(0).contains(&throw_card),
        "弃出牌离开 0 号手牌"
    );
}
