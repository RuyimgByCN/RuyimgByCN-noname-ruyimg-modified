use super::*;

/// T12 swd deferred 消解批 95（狩猎 shoulie/不死 busi）：两技行为面+反例。
#[test]
fn swd_deferred_b95_shoulie_busi() {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
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
    let mut e = Engine::new_with_extensions(62052, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 狩猎（子达书）：杀指定后弃一张手牌 → 此杀不可闪避
    e.debug_grant(0, "shoulie");
    let cost = e.debug_place(0, "tao");
    let sha1 = e.debug_place(0, "sha");
    let shan1 = e.debug_place(1, "shan");
    e.debug_set_hp(1, 4);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![1],
        },
    );
    // 狩猎弃牌窗 → 弃 tao → 杀直击（p1 有闪不可避）
    let c = e.open_choice().expect("狩猎弃牌窗");
    assert_eq!((c.spec.prompt, c.observer), ("shoulie_discard", 0));
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(cost)],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 3, "狩猎：弃牌后杀直击（p1 有闪不可避）");
    assert!(e.debug_hand(1).contains(&shan1), "狩猎：p1 闪未消耗");
    // 反例面：放弃（空选）→ 杀照常可闪
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha2 = e.debug_place(0, "sha");
    let shan2 = e.debug_place(1, "shan");
    e.debug_set_hp(1, 4);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![1],
        },
    );
    let c = e.open_choice().expect("狩猎弃牌窗 2");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![],
        },
    );
    let c = e.open_choice().expect("p1 出闪窗（放弃狩猎后照常）");
    assert_eq!(c.spec.prompt, "respond_shan");
    let (cid, obs) = (c.id, c.observer);
    let sel: Vec<ChoiceOption> = c
        .spec
        .options
        .iter()
        .filter(|o| matches!(o, ChoiceOption::Card(x) if *x == shan2))
        .cloned()
        .collect();
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: sel,
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 4, "狩猎反例：放弃后杀被闪避");

    // ② 不死（康那里士）：濒死判定非黑桃 → 回至 1+翻面+移除实例
    e.debug_grant(1, "busi");
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(1, 1);
    // 推进到 p0 回合出杀
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    for _ in 0..16 {
        let c = e.turn().current;
        if c == 0 && e.state_core().turn.phase == engine_native::event::Phase::Play {
            break;
        }
        let _ = submit(&mut e, c, GameAction::NextPhase);
    }
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        if p != 0 {
            e.debug_clear_hand(p);
        }
    }
    e.debug_set_hp(1, 1);
    // 判定非黑桃（红桃）
    e.debug_set_deck_top_by(|def| def.suit.is_heart());
    let sha3 = e.debug_place(0, "sha");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha3,
            targets: vec![1],
        },
    );
    settle_optional_confirm(&mut e);
    assert!(e.state_core().player(1).alive, "不死：p1 存活");
    assert_eq!(e.debug_hp(1), 1, "不死：体力回至 1");
    assert!(e.state_core().player(1).face_down, "不死：翻面");
    let busi_def = e.catalog().skill_by_name("busi").expect("不死在目录");
    assert!(
        !e.state_core().skills.contains_key(&(1, busi_def)),
        "不死：实例已移除（unique）"
    );
}
