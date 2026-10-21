use super::*;

/// T12 swd 武将面批 24：双将整将——断月（水镜 tag 280：出牌限一次弃
/// 装备牌对其他角色造成 1 伤，三段机）；屠阵（水镜 tag 281：自己伤害
/// 他人结算后弃其全部非基本手牌）；落梅（兰茵 tag 282：使用或打出梅花
/// 牌频率摸 1，双事件）；星点（兰茵 tag 283：出牌限一次弃手牌，确定性
/// 弃两名其他角色各手/装备首张——登记差异上游随机）。
#[test]
fn swd_generals_b24_duanyue_tuzhen_luomei_xingdian() {
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
    let mut e = Engine::new_with_extensions(62007, 4, &[&pack]).expect("建局");

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
    let pick = |e: &mut Engine, pred: &dyn Fn(&ChoiceOption) -> bool, msg: &'static str| {
        let (cid, obs) = e.open_choice().map(|c| (c.id, c.observer)).expect(msg);
        let sel: Vec<ChoiceOption> = {
            let ch = e.open_choice().expect("still open");
            ch.spec
                .options
                .iter()
                .filter(|o| pred(o))
                .cloned()
                .take(1)
                .collect()
        };
        assert_eq!(sel.len(), 1, "{}", msg);
        submit(
            e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: sel,
            },
        );
    };

    // ① 断月：p0 持技（手牌有 zhuge），对 p1 造成 1 伤
    e.debug_grant(0, "duanyue");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let duanyue_def = e.catalog().skill_by_name("duanyue").expect("断月 def");
    let h1 = hand_len_w(&e, 1);
    e.debug_set_hp(1, 4);

    // 反例 ①a（先测）：取消选牌 → 无效果（发动即耗 once_used，随后重置）
    let _zg0 = e.debug_place(0, "zhuge");
    let _ = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action: GameAction::ActivateSkill { skill: duanyue_def },
    });
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("断月取消窗");
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![],
        },
    );
    settle_optional_confirm(&mut e);
    assert!(e.open_choice().is_none(), "断月反例：取消后无窗");
    assert_eq!(e.debug_hp(1), 4, "断月反例：取消 → p1 不受伤");
    assert_eq!(hand_len_w(&e, 0), 1, "断月反例：取消 → zhuge 留在手牌");
    {
        let mut snap = e.save();
        snap.core
            .skills
            .get_mut(&(0, duanyue_def))
            .expect("断月实例")
            .once_used = false;
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("重置 once_used");
    }

    // 正例 ①：选 zhuge → 选 p1 → 弃牌+1 伤（清掉 ①a 留下的 zhuge）
    e.debug_clear_hand(0);
    let zg = e.debug_place(0, "zhuge");
    let _ = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action: GameAction::ActivateSkill { skill: duanyue_def },
    });
    pick(
        &mut e,
        &|o| matches!(o, ChoiceOption::Card(c2) if *c2 == zg),
        "断月选牌窗",
    );
    pick(
        &mut e,
        &|o| matches!(o, ChoiceOption::Player(1)),
        "断月选人窗",
    );
    settle_optional_confirm(&mut e);
    assert_eq!(hand_len_w(&e, 0), 0, "断月：zhuge 已弃");
    assert_eq!(e.debug_hp(1), 3, "断月：p1 受 1 伤");
    assert_eq!(hand_len_w(&e, 1), h1, "断月：p1 手牌不动");

    // 反例 ①b：限一次（本回合再发动被拒）
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action: GameAction::ActivateSkill { skill: duanyue_def },
    });
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "断月反例：本回合限一次 {out:?}"
    );

    // ② 屠阵：撤销断月，p0 杀伤 p1（p1 手有非基本牌）→ 确认 → 全弃
    {
        let def_dy = e.catalog().skill_by_name("duanyue").expect("断月 def");
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        snap.core.skills.remove(&(0, def_dy));
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("撤销断月");
    }
    e.debug_grant(0, "tuzhen");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_place(1, "tao");
    e.debug_place(1, "tao");
    e.debug_place(1, "duel");
    reset_sha_for(&mut e, &pack);
    let sha_t = e.debug_place(0, "sha");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha_t,
            targets: vec![1],
        },
    );
    confirm(&mut e, 0);
    settle_optional_confirm(&mut e);
    assert_eq!(hand_len_w(&e, 1), 2, "屠阵：p1 非基本（duel）被弃，剩 2 桃");

    // 反例 ②b：受伤者无非基本手牌 → 无窗
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_place(1, "tao");
    reset_sha_for(&mut e, &pack);
    let sha_t2 = e.debug_place(0, "sha");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha_t2,
            targets: vec![1],
        },
    );
    assert!(e.open_choice().is_none(), "屠阵反例：无非基本手牌不触发");

    // 反例 ②c：未造成伤害（prevented 面）不触发——p1 装备区清空+手里有
    // 桃可自救？改用无伤害来源路径：直接 debug 伤 p1 再用杀无伤不成立，
    // 以 filter 的 target!=owner 反例替代：p1 杀伤 p0（来源非持有者）
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_place(0, "duel");
    reset_sha_for(&mut e, &pack);
    let sha_t3 = e.debug_place(1, "sha");
    let h0 = hand_len_w(&e, 0);
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha_t3,
            targets: vec![0],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(
        hand_len_w(&e, 0),
        h0,
        "屠阵反例：来源非持有者不触发（p0 全留）"
    );

    // ③ 落梅：撤销屠阵，p0 持落梅——出梅花牌摸 1（找/布梅花杀）
    {
        let def_tz = e.catalog().skill_by_name("tuzhen").expect("屠阵 def");
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        snap.core.skills.remove(&(0, def_tz));
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("撤销屠阵");
    }
    e.debug_grant(0, "luomei");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let club_sha = {
        let id = e.debug_set_deck_top_by(|d| d.kind.is_sha_like() && matches!(d.suit, Suit::Club));
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.players[0].hand.push(id);
        id
    };
    assert!(matches!(
        e.catalog().cards[club_sha as usize].suit,
        Suit::Club
    ));
    e.debug_set_hp(1, 4);
    let h0 = hand_len_w(&e, 0);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: club_sha,
            targets: vec![1],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(
        hand_len_w(&e, 0),
        h0,
        "落梅：用 1 梅花杀摸 1（用 1-1+摸 1 净持平）"
    );
    assert_eq!(e.debug_hp(1), 3, "落梅：p1 存活（4-1，排除濒死噪声）");

    // 反例 ③b：非梅花不摸
    reset_sha_for(&mut e, &pack);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let red_sha = {
        let id = e.debug_set_deck_top_by(|d| {
            d.kind.is_sha_like() && matches!(d.suit, Suit::Heart | Suit::Diamond)
        });
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.players[0].hand.push(id);
        id
    };
    e.debug_set_hp(1, 4);
    let h0 = hand_len_w(&e, 0);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: red_sha,
            targets: vec![1],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(hand_len_w(&e, 0), h0 - 1, "落梅反例：红杀不摸（净 -1）");
    assert_eq!(e.debug_hp(1), 3, "落梅反例：p1 存活");

    // ④ 星点：撤销落梅，p0 持星点——弃 1 手牌，p1/p2 各弃手/装备首张
    {
        let def_lm = e.catalog().skill_by_name("luomei").expect("落梅 def");
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        snap.core.skills.remove(&(0, def_lm));
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("撤销落梅");
    }
    e.debug_grant(0, "xingdian");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let mine = e.debug_place(0, "tao");
    let v1 = e.debug_place(1, "shan");
    let _v2 = e.debug_place(2, "tao");
    let xingdian_def = e.catalog().skill_by_name("xingdian").expect("星点 def");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action: GameAction::ActivateSkill {
            skill: xingdian_def,
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "星点受理: {out:?}"
    );
    pick(
        &mut e,
        &|o| matches!(o, ChoiceOption::Card(c2) if *c2 == mine),
        "星点选牌窗",
    );
    settle_optional_confirm(&mut e);
    assert_eq!(hand_len_w(&e, 0), 0, "星点：自己弃 1");
    assert_eq!(hand_len_w(&e, 1), 0, "星点：p1 首张被弃");
    assert_eq!(hand_len_w(&e, 2), 0, "星点：p2 首张被弃");
    assert_eq!(hand_len_w(&e, 3), 0, "星点：p3 无牌不受影响");
    let _ = (v1, h1);
}
