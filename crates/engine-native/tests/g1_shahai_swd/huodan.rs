use super::*;

/// T12 swd 武将面批 85（舒莞儿活弹）：partial 转正（sxianjing defer）。
#[test]
fn swd_generals_b85_huodan() {
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
    assert!(
        pack.generals.len() >= 74,
        "74 将入册（现 {}）",
        pack.generals.len()
    );
    let mut e = Engine::new_with_extensions(62052, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // 活弹：单目标 2 火伤
    e.debug_grant(0, "huodan");
    let red = {
        let id = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                matches!(
                    e.catalog().cards[*c as usize].suit,
                    engine_native::catalog::Suit::Heart
                )
            })
            .expect("红桃牌");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.discard.retain(|x| *x != id);
        core.table.retain(|x| *x != id);
        for pl in core.players.iter_mut() {
            pl.hand.retain(|x| *x != id);
        }
        core.players[0].hand.push(id);
        id
    };
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);
    let def_hd = e.catalog().skill_by_name("huodan").expect("活弹在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_hd });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "活弹发动 {out:?}"
    );
    let c = e.open_choice().expect("活弹选牌窗");
    assert_eq!(c.spec.prompt, "huodan_pick");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(red)],
        },
    );
    let c = e.open_choice().expect("活弹目标窗");
    assert_eq!(c.spec.prompt, "huodan_target");
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
    pass_nullify(&mut e);
    assert_eq!(e.debug_hp(1), 2, "活弹：单目标受 2 火伤（4→2）");
    assert_eq!(e.debug_hp(0), 3, "活弹：自失 1 体力（4→3）");
    // 反例：无手牌发动被拒
    e.debug_clear_hand(0);
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_hd });
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "活弹反例：无红牌拒 {out:?}"
    );
}
