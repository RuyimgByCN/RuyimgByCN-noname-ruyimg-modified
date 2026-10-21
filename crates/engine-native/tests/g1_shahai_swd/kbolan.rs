use super::*;

/// T12 swd deferred 消解批 98（拷贝栏 kbolan/奥剑 aojian）：两技行为面
/// + 反例。
#[test]
fn swd_deferred_b98_kbolan_aojian() {
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
    let mut e = Engine::new_with_extensions(62055, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 拷贝栏（肯迪）：摸牌阶段 +1 且置回一张——净 +2（2 固定+1 额外-1 置回）
    e.debug_grant(0, "kbolan");
    let keep = e.debug_place(0, "tao");
    // 压牌堆顶两张已知牌（摸 3 后首张被置回）
    let h0 = e.debug_hand(0).len();
    let deck_len = e.state_core().deck.len();
    // 推进到 p0 下一回合的摸牌阶段（Draw）——经 p1/p2/p3 回合
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    for _ in 0..20 {
        let c = e.turn().current;
        if c == 0 && e.state_core().turn.phase == engine_native::event::Phase::Draw {
            break;
        }
        let _ = submit(&mut e, c, GameAction::NextPhase);
    }
    // 完成摸牌阶段（NextPhase 进 Play）
    submit(&mut e, 0, GameAction::NextPhase);
    let h_after = e.debug_hand(0).len();
    assert_eq!(h_after - h0, 2, "拷贝栏：净 +2（固定 2+额外 1-置回 1）");
    let _ = (keep, deck_len);

    // ② 奥剑（Miles）：弃 2 张手牌 → 攻击距离 1 内至多 2 名各 1 伤
    e.debug_grant(0, "aojian");
    // 回到 p0 出牌阶段
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, 2); // maxHp 4 → X=2
    let a1 = e.debug_place(0, "tao");
    let a2 = e.debug_place(0, "shan");
    e.debug_set_hp(1, 4);
    e.debug_set_hp(3, 4);
    let def_aj = e.catalog().skill_by_name("aojian").expect("奥剑在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_aj });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "奥剑发动 {out:?}"
    );
    let c = e.open_choice().expect("奥剑弃牌窗");
    assert_eq!(c.spec.prompt, "aojian_discard");
    assert!(c.spec.max == 2, "奥剑：上限 X=2");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(a1), ChoiceOption::Card(a2)],
        },
    );
    let c = e.open_choice().expect("奥剑目标窗");
    assert_eq!(c.spec.prompt, "aojian_targets");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1), ChoiceOption::Player(3)],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 3, "奥剑：p1 受 1");
    assert_eq!(e.debug_hp(3), 3, "奥剑：p3 受 1");
    assert!(
        !e.debug_hand(0).contains(&a1) && !e.debug_hand(0).contains(&a2),
        "奥剑：两张代价已弃"
    );
    // 反例：每回合限一次
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_aj });
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "奥剑反例：再发动拒 {out:?}"
    );
}
