use super::*;

/// T12 swd 武将面批 25：双技部分转正——协律（壶中仙 tag 284：弃牌阶段
/// 结束[End 帧入场]手牌≥2 同色，可选回复 1 并弃置场上全部判定牌）；朱
/// 羽（芝茵 tag 285：横置角色受到非传导伤害，可选弃红牌使伤害+1 且改
/// 火属性）。
#[test]
fn swd_generals_b25_xielv_zhuyu() {
    use engine_native::catalog::Suit;
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
    let mut e = Engine::new_with_extensions(62008, 4, &[&pack]).expect("建局");
    let confirm = |e: &mut Engine, want_obs: u8| {
        let (cid, obs) = e.open_choice().map(|c| (c.id, c.observer)).expect("确认窗");
        assert_eq!(obs, want_obs, "确认窗归属");
        let conf: Vec<ChoiceOption> = {
            let ch = e.open_choice().expect("still open");
            ch.spec
                .options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Confirm))
                .cloned()
                .collect()
        };
        assert_eq!(conf.len(), 1);
        submit(
            e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: conf,
            },
        );
    };
    let pick_red = |e: &mut Engine| {
        let (cid, obs) = e
            .open_choice()
            .map(|c| (c.id, c.observer))
            .expect("朱羽选牌窗");
        let sel: Vec<ChoiceOption> = {
            let ch = e.open_choice().expect("still open");
            ch.spec
                .options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Card(c) if matches!(e.catalog().cards[*c as usize].suit, Suit::Heart | Suit::Diamond)))
                .cloned()
                .take(1)
                .collect()
        };
        assert_eq!(sel.len(), 1, "朱羽红牌面");
        submit(
            e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: sel,
            },
        );
    };

    // ① 协律：p0 手 2 红（桃+红杀），受伤断言回 1；p1 判定区有乐 → 被清
    e.debug_grant(0, "xielv");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _t1 = e.debug_place(0, "tao");
    {
        let id = e.debug_set_deck_top_by(|d| {
            d.kind.is_sha_like() && matches!(d.suit, Suit::Heart | Suit::Diamond)
        });
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.players[0].hand.push(id);
    }
    let _le = e.debug_place_judge(1, "le");
    assert_eq!(
        e.state_core().player(1).judge.len(),
        1,
        "布阵：p1 判定区 1 张"
    );
    e.debug_set_hp(0, 3);
    submit(&mut e, 0, GameAction::NextPhase); // Play→Discard（手 2 ≤ 上限无弃牌窗）
    submit(&mut e, 0, GameAction::NextPhase); // Discard→End，End 入场触发
    confirm(&mut e, 0);
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(0), 4, "协律：回 1（3→4）");
    assert!(
        e.state_core().player(1).judge.is_empty(),
        "协律：判定牌全清"
    );

    // 反例 ①b：手牌异色 → 下一回合 End 入场无窗
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _t2 = e.debug_place(0, "tao");
    {
        let id = e.debug_set_deck_top_by(|d| {
            d.kind.is_sha_like() && matches!(d.suit, Suit::Spade | Suit::Club)
        });
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.players[0].hand.push(id);
    }
    e.debug_set_hp(0, 3);
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    assert!(e.open_choice().is_none(), "协律反例：异色不触发");

    // 反例 ①c：手牌不足 2 张 → 无窗
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _t3 = e.debug_place(0, "tao");
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    assert!(e.open_choice().is_none(), "协律反例：单张不触发");
    {
        let def_xl = e.catalog().skill_by_name("xielv").expect("协律 def");
        let mut snap = e.save();
        snap.core.skills.remove(&(0, def_xl));
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("撤销协律");
    }

    // ② 朱羽：撤销协律后 p0 持朱羽，p1 横置，p2 杀 p1 → 确认+弃红 → 2 伤
    e.debug_grant(0, "zhuyu");
    advance_to_play(&mut e, 2);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    {
        let core = e.state_core_mut();
        core.players[1].linked = true;
    }
    let _red = {
        let id = e.debug_set_deck_top_by(|d| {
            d.kind.is_sha_like() && matches!(d.suit, Suit::Heart | Suit::Diamond)
        });
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.players[0].hand.push(id);
        id
    };
    let sha_z = e.debug_place(2, "sha");
    e.debug_set_hp(1, 8);
    submit(
        &mut e,
        2,
        GameAction::PlayCard {
            card: sha_z,
            targets: vec![1],
        },
    );
    confirm(&mut e, 0);
    pick_red(&mut e);
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 6, "朱羽：横置目标伤害 +1（8-2）");
    assert_eq!(hand_len_w(&e, 0), 0, "朱羽：红牌已弃");

    // 反例 ②b：取消 → 1 伤
    {
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("重置");
    }
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _red2 = {
        let id = e.debug_set_deck_top_by(|d| {
            d.kind.is_sha_like() && matches!(d.suit, Suit::Heart | Suit::Diamond)
        });
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.players[0].hand.push(id);
        id
    };
    let sha_z2 = e.debug_place(2, "sha");
    e.debug_set_hp(1, 8);
    submit(
        &mut e,
        2,
        GameAction::PlayCard {
            card: sha_z2,
            targets: vec![1],
        },
    );
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("朱羽取消窗");
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 7, "朱羽反例：取消 → 1 伤");

    // 反例 ②c：目标未横置 → 无窗
    {
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("重置");
    }
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    {
        let core = e.state_core_mut();
        core.players[1].linked = false;
    }
    let _red3 = {
        let id = e.debug_set_deck_top_by(|d| {
            d.kind.is_sha_like() && matches!(d.suit, Suit::Heart | Suit::Diamond)
        });
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.players[0].hand.push(id);
        id
    };
    let sha_z3 = e.debug_place(2, "sha");
    e.debug_set_hp(1, 8);
    submit(
        &mut e,
        2,
        GameAction::PlayCard {
            card: sha_z3,
            targets: vec![1],
        },
    );
    assert!(e.open_choice().is_none(), "朱羽反例：未横置不触发");

    // 反例 ②d：持有者无红牌 → 无窗
    {
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("重置");
    }
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    {
        let core = e.state_core_mut();
        core.players[1].linked = true;
    }
    let sha_z4 = e.debug_place(2, "sha");
    e.debug_set_hp(1, 8);
    submit(
        &mut e,
        2,
        GameAction::PlayCard {
            card: sha_z4,
            targets: vec![1],
        },
    );
    assert!(e.open_choice().is_none(), "朱羽反例：无红牌不触发");
}
