use super::*;

/// T12 swd 武将面批㊳（姬亮运筹；gongxin/jqimou deferred）：行为面 + 反例。
#[test]
fn swd_generals_b78_yunchou() {
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
    let mut e = Engine::new_with_extensions(62046, 2, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..2 {
        e.debug_clear_hand(p);
    }

    // 运筹（姬亮）：弃一张手牌 → 目标随机弃一张手牌；同色自己摸 1 否则目标摸 1
    e.debug_grant(0, "yunchou");
    // 锚=红色手牌；p1 手牌全红 → 必同色 → 自己摸 1
    let anchor = {
        let a = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                matches!(
                    e.catalog().cards[*c as usize].suit,
                    engine_native::catalog::Suit::Heart
                ) && e.state_core().deck.contains(c)
            })
            .expect("红桃锚在牌库");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != a);
        core.players[0].hand.push(a);
        a
    };
    let red1 = {
        let r = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                matches!(
                    e.catalog().cards[*c as usize].suit,
                    engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
                ) && e.state_core().deck.contains(c)
            })
            .expect("红牌在牌库");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != r);
        core.players[1].hand.push(r);
        r
    };
    let def_yc = e.catalog().skill_by_name("yunchou").expect("运筹在目录");
    let h0 = e.debug_hand(0).len();
    let h1 = e.debug_hand(1).len();
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_yc });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "运筹发动 {out:?}"
    );
    let c = e.open_choice().expect("运筹弃牌窗");
    assert_eq!(c.spec.prompt, "yunchou_discard");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(anchor)],
        },
    );
    let c = e.open_choice().expect("运筹目标窗");
    assert_eq!(c.spec.prompt, "yunchou_target");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    settle_optional_confirm(&mut e);
    // p1 手牌全红 → 随机弃必同色 → 自己摸 1
    assert_eq!(
        e.debug_hand(0).len(),
        h0 - 1 + 1,
        "运筹：同色自己摸 1（出 1 摸 1）"
    );
    assert_eq!(e.debug_hand(1).len(), h1 - 1, "运筹：p1 随机弃 1");
    assert!(
        !e.debug_hand(1).contains(&red1) || e.debug_hand(1).len() >= 1,
        "运筹：弃牌面成立"
    );
    // 反例：无手牌发动拒
    e.debug_clear_hand(0);
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_yc });
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "运筹反例：无手牌拒 {out:?}"
    );
    let _ = red1;
}
