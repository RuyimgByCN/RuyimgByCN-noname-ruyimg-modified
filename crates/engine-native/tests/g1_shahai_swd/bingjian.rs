use super::*;

/// T12 swd 武将面批㉞（海大胖冰箭/麦星神鼎/冯天凌鬼眼；rumeng/toudan/
/// guiyan2 deferred）：三将行为面 + 反例。
#[test]
fn swd_generals_b74_bingjian_shending_guiyan() {
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
    let mut e = Engine::new_with_extensions(62042, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 冰箭（海大胖）：弃黑杀 → 目标闪全弃（无闪受 1 雷伤反例面）
    e.debug_grant(0, "bingjian");
    let bsha = {
        let b = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                e.catalog().card_kind(*c) == engine_native::catalog::CardKind::Sha
                    && matches!(
                        e.catalog().cards[*c as usize].suit,
                        engine_native::catalog::Suit::Spade
                    )
            })
            .expect("黑杀在目录");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != b);
        core.discard.retain(|x| *x != b);
        core.table.retain(|x| *x != b);
        for pl in core.players.iter_mut() {
            pl.hand.retain(|x| *x != b);
        }
        core.players[0].hand.push(b);
        b
    };
    let shan_v = e.debug_place(1, "shan");
    let shan_v2 = e.debug_place(1, "shan");
    let def_bj = e.catalog().skill_by_name("bingjian").expect("冰箭在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_bj });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "冰箭发动 {out:?}"
    );
    let c = e.open_choice().expect("冰箭弃牌窗");
    assert_eq!(c.spec.prompt, "bingjian_discard");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(bsha)],
        },
    );
    let c = e.open_choice().expect("冰箭目标窗");
    assert_eq!(c.spec.prompt, "bingjian_target");
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
    assert!(
        !e.debug_hand(1).contains(&shan_v) && !e.debug_hand(1).contains(&shan_v2),
        "冰箭：目标闪全弃"
    );
    assert_eq!(e.debug_hand(1).len(), 0, "冰箭：p1 手牌清空（两张均闪）");

    // ② 神鼎（麦星）：判定红桃回 1（压顶）
    e.debug_grant(0, "shending");
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, 2);
    e.debug_set_deck_top_by(|def| def.suit.is_heart());
    let def_sd = e.catalog().skill_by_name("shending").expect("神鼎在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_sd });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "神鼎发动 {out:?}"
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(0), 3, "神鼎：红桃判定回 1（2→3）");
    // 反例面：黑桃判定无效果
    e.debug_set_deck_top_by(|def| def.suit.is_spade());
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_sd });
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "神鼎反例：同回合限一次 {out:?}"
    );

    // ③ 鬼眼（冯天凌）：取目标手牌一张梅花
    e.debug_grant(0, "guiyan");
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let club_v = {
        let cl = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                matches!(
                    e.catalog().cards[*c as usize].suit,
                    engine_native::catalog::Suit::Club
                ) && e.state_core().deck.contains(c)
            })
            .expect("梅花在牌库");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != cl);
        core.players[1].hand.push(cl);
        cl
    };
    let def_gy = e.catalog().skill_by_name("guiyan").expect("鬼眼在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_gy });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "鬼眼发动 {out:?}"
    );
    let c = e.open_choice().expect("鬼眼目标窗");
    assert_eq!(c.spec.prompt, "guiyan_target");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    let c = e.open_choice().expect("鬼眼取牌窗");
    assert_eq!(c.spec.prompt, "guiyan_take");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(club_v)],
        },
    );
    settle_optional_confirm(&mut e);
    assert!(
        e.debug_hand(0).contains(&club_v) && !e.debug_hand(1).contains(&club_v),
        "鬼眼：梅花牌换手"
    );
}
