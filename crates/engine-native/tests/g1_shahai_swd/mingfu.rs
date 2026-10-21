use super::*;

/// T12 swd 武将面批 88（姜子牙命符）：partial 转正（tianlun defer）。
#[test]
fn swd_generals_b88_mingfu() {
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
        pack.generals.len() >= 77,
        "77 将入册（现 {}）",
        pack.generals.len()
    );
    let mut e = Engine::new_with_extensions(62055, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // 命符：弃梅花 → 目标判定区获得鬼幽结
    e.debug_grant(0, "mingfu");
    let club = {
        let id = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                matches!(
                    e.catalog().cards[*c as usize].suit,
                    engine_native::catalog::Suit::Club
                ) && e.catalog().card_kind(*c) != CardKind::Guiyoujie
            })
            .expect("梅花牌");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.discard.retain(|x| *x != id);
        core.table.retain(|x| *x != id);
        for pl in core.players.iter_mut() {
            pl.hand.retain(|x| *x != id);
            pl.judge.retain(|x| *x != id);
        }
        core.players[0].hand.push(id);
        id
    };
    let judge_before = e.state_core().players[1].judge.len();
    let def_mf = e.catalog().skill_by_name("mingfu").expect("命符在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_mf });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "命符发动 {out:?}"
    );
    let c = e.open_choice().expect("命符选牌窗");
    assert_eq!(c.spec.prompt, "mingfu_pick");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(club)],
        },
    );
    let c = e.open_choice().expect("命符目标窗");
    assert_eq!(c.spec.prompt, "mingfu_target");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    assert_eq!(
        e.state_core().players[1].judge.len(),
        judge_before + 1,
        "命符：p1 判定区获得鬼幽结（前 {judge_before}）"
    );
    assert_eq!(
        e.catalog().card_kind(e.state_core().players[1].judge[0]),
        CardKind::Guiyoujie,
        "命符：置入牌为鬼幽结"
    );
    // 反例：无梅花手牌发动被拒
    e.debug_clear_hand(0);
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_mf });
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "命符反例：无梅花拒 {out:?}"
    );
}
