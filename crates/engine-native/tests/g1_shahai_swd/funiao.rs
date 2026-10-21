use super::*;

/// T12 swd deferred 消解批 93（孵鸟 funiao/凝弦 ningxian/妖童 yaotong）：
/// 三技行为面 + 反例。
#[test]
fn swd_deferred_b93_funiao_ningxian_yaotong() {
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
    let mut e = Engine::new_with_extensions(62050, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 孵鸟（恒爱）：一张手牌交给其他角色 → 摸 1
    e.debug_grant(0, "funiao");
    let gift = e.debug_place(0, "tao");
    let keep = e.debug_place(0, "shan");
    let h1_before = e.debug_hand(1).len();
    let def_fn = e.catalog().skill_by_name("funiao").expect("孵鸟在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_fn });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "孵鸟发动 {out:?}"
    );
    let c = e.open_choice().expect("孵鸟交牌窗");
    assert_eq!(c.spec.prompt, "funiao_give");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(gift)],
        },
    );
    let c = e.open_choice().expect("孵鸟目标窗");
    assert_eq!(c.spec.prompt, "funiao_target");
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
    assert!(e.debug_hand(1).contains(&gift), "孵鸟：牌交给 p1");
    assert_eq!(e.debug_hand(1).len(), h1_before + 1, "孵鸟：p1 +1");
    // 自己：出 1 摸 1 → 净不变（keep 保留；2-1+1=2）
    assert!(
        e.debug_hand(0).contains(&keep) && e.debug_hand(0).len() == 2,
        "孵鸟：自己出 1 摸 1（实得 {:?}）",
        e.debug_hand(0)
    );

    // ② 凝弦（恒爱）：受伤后弃 2 张黑牌 → 2 名目标各受 1
    e.debug_grant(0, "ningxian");
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let b1 = {
        let b = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                e.catalog().card_kind(*c) != engine_native::catalog::CardKind::Sha
                    && !e.catalog().card_kind(*c).equip_slot().is_some()
                    && matches!(
                        e.catalog().cards[*c as usize].suit,
                        engine_native::catalog::Suit::Spade
                    )
                    && e.state_core().deck.contains(c)
            })
            .expect("黑牌 1 在牌库");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != b);
        core.players[0].hand.push(b);
        b
    };
    let b2 = {
        let b = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                e.catalog().card_kind(*c) != engine_native::catalog::CardKind::Sha
                    && !e.catalog().card_kind(*c).equip_slot().is_some()
                    && matches!(
                        e.catalog().cards[*c as usize].suit,
                        engine_native::catalog::Suit::Club
                    )
                    && e.state_core().deck.contains(c)
            })
            .expect("黑牌 2 在牌库");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != b);
        core.players[0].hand.push(b);
        b
    };
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);
    // p1 回合杀 p0（回合属主）
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    for _ in 0..16 {
        let c = e.turn().current;
        if c == 1 && e.state_core().turn.phase == engine_native::event::Phase::Play {
            break;
        }
        let _ = submit(&mut e, c, GameAction::NextPhase);
    }
    advance_to_play(&mut e, 1);
    for p in 0..4 {
        if p != 1 {
            e.debug_clear_hand(p);
        }
    }
    let b1 = {
        let core = e.state_core_mut();
        core.players[0].hand.push(b1);
        b1
    };
    let b2 = {
        let core = e.state_core_mut();
        core.players[0].hand.push(b2);
        b2
    };
    let sha1 = e.debug_place(1, "sha");
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![0],
        },
    );
    // 凝弦确认窗 → 确认 → 弃牌窗（选 2 黑）→ 目标窗（p2+p3 等量 2）
    let c = e.open_choice().expect("凝弦确认窗");
    assert_eq!((c.spec.prompt, c.observer), ("trigger_confirm", 0));
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let c = e.open_choice().expect("凝弦弃牌窗");
    assert_eq!(c.spec.prompt, "ningxian_discard");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(b1), ChoiceOption::Card(b2)],
        },
    );
    let c = e.open_choice().expect("凝弦目标窗");
    assert_eq!(c.spec.prompt, "ningxian_targets");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(2), ChoiceOption::Player(3)],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(2), 3, "凝弦：p2 受 1");
    assert_eq!(e.debug_hp(3), 3, "凝弦：p3 受 1");
    assert_eq!(e.debug_hp(0), 3, "凝弦：p0 受杀 1（4→3）");

    // ③ 妖童（齐）：手牌奇数任意手牌当杀（桃当杀命中）
    e.debug_grant(0, "yaotong");
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 手牌=1 张（奇数）→ 该杀类牌外任意牌可当杀：用闪实体
    let shan_as_sha = e.debug_place(0, "shan");
    e.debug_set_hp(1, 4);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: shan_as_sha,
            targets: vec![1],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 3, "妖童：奇数手牌闪当杀命中");
    // 反例：偶数手牌不转化——再放一张闪后（手牌=1?）布偶数：放 2 张
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let s1 = e.debug_place(0, "shan");
    let _s2 = e.debug_place(0, "shan");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: s1,
            targets: vec![1],
        },
    );
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "妖童反例：偶数手牌闪不可当杀 {out:?}"
    );
}
