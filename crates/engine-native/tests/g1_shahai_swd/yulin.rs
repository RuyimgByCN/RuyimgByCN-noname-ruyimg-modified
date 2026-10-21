use super::*;

/// T12 swd 武将面批 22：三技三将部分转正——玉鳞（兰茵：受伤前置窗口
/// 弃 1 张装备牌[手/装备区]免此伤害）；凝霜（芝茵：成为黑色牌目标弃 1
/// 张黑牌，使用者阶梯：已翻面→失 1 体力/已横置→翻面/否则横置且仅此支
/// 自己摸 1——上游代码原样）；工神（黄雷：其他角色用基本/锦囊牌且有
/// 目标，弃 1 张装备牌令其失效=cancelled 写回强固守卫同款）。
#[test]
fn swd_generals_b22_yulin_ningshuang_gongshen() {
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
    let mut e = Engine::new_with_extensions(62005, 4, &[&pack]).expect("建局");
    let is_black =
        |e: &Engine, c: u32| matches!(e.catalog().cards[c as usize].suit, Suit::Spade | Suit::Club);
    let reset_sha = |e: &mut Engine| {
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        *e = Engine::restore_with_extensions(&snap, &[&pack]).expect("重置");
    };
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
    let pick_first_card = |e: &mut Engine| {
        let (cid, obs) = e.open_choice().map(|c| (c.id, c.observer)).expect("选牌窗");
        let sel: Vec<ChoiceOption> = {
            let ch = e.open_choice().expect("still open");
            ch.spec
                .options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Card(_)))
                .cloned()
                .take(1)
                .collect()
        };
        assert_eq!(sel.len(), 1, "选牌面非空");
        submit(
            e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: sel,
            },
        );
    };

    // 确定性花色布阵：按谓词定位实体（deck→discard→hands 兜底）后直接
    // 入手（debug_set_deck_top_by 只压牌堆底，debug_place 从堆前取——
    // 两者不衔接，颜色不保证）。
    let place_sha_color = |e: &mut Engine, p: u8, black: bool| {
        let id = e.debug_set_deck_top_by(move |d| {
            d.kind.is_sha_like()
                && if black {
                    matches!(d.suit, Suit::Spade | Suit::Club)
                } else {
                    matches!(d.suit, Suit::Heart | Suit::Diamond)
                }
        });
        {
            let core = e.state_core_mut();
            core.deck.retain(|x| *x != id);
            core.players[p as usize].hand.push(id);
        }
        id
    };

    // ① 玉鳞：p1 杀 p0（p0 手牌有武器 zhuge）→ 确认+弃 zhuge → 免伤
    e.debug_grant(0, "yulin");
    advance_to_play(&mut e, 1);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let zg = e.debug_place(0, "zhuge");
    assert!(
        e.catalog().cards[zg as usize].kind.equip_slot().is_some(),
        "zhuge 是装备牌"
    );
    let sha1 = e.debug_place(1, "sha");
    e.debug_set_hp(0, 4);
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![0],
        },
    );
    confirm(&mut e, 0);
    pick_first_card(&mut e);
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(0), 4, "玉鳞：弃装备牌免伤");
    assert_eq!(hand_len_w(&e, 0), 0, "玉鳞：zhuge 已弃");

    // 反例 ①a：取消确认窗 → 伤害照常
    reset_sha(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _zg2 = e.debug_place(0, "zhuge");
    let sha2 = e.debug_place(1, "sha");
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![0],
        },
    );
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("玉鳞取消窗");
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(0), 3, "玉鳞反例：取消 → 伤害照常（4-1）");

    // 反例 ①b：无装备牌 → 无窗（伤害照常）
    reset_sha(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha3 = e.debug_place(1, "sha");
    let hp0 = e.debug_hp(0);
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha3,
            targets: vec![0],
        },
    );
    assert!(e.open_choice().is_none(), "玉鳞反例：无装备牌不触发");
    assert_eq!(e.debug_hp(0), hp0 - 1, "玉鳞反例：伤害照常");

    // 正例 ①c：装备区的装备牌也可弃（equip 分支 discard idiom）
    reset_sha(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let zg3 = e.debug_place(0, "zhuge");
    {
        let core = e.state_core_mut();
        core.players[0].hand.retain(|x| *x != zg3);
        core.players[0].equips.push(zg3);
    }
    let sha4 = e.debug_place(1, "sha");
    e.debug_set_hp(0, 4);
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha4,
            targets: vec![0],
        },
    );
    confirm(&mut e, 0);
    pick_first_card(&mut e);
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(0), 4, "玉鳞：装备区装备牌同样免伤");
    assert!(
        e.state_core().player(0).equips.is_empty(),
        "玉鳞：装备区已弃空"
    );

    // ② 凝霜：撤销玉鳞（①所授），p1 黑杀 p0 → 确认+弃黑牌 → p1 横置、p0 摸 1
    {
        let def_yl = e.catalog().skill_by_name("yulin").expect("玉鳞 def");
        let mut snap = e.save();
        snap.core.skills.remove(&(0, def_yl));
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("撤销玉鳞");
    }
    e.debug_grant(0, "ningshuang");
    advance_to_play(&mut e, 1);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _blk_for_p0 = place_sha_color(&mut e, 0, true);
    reset_sha(&mut e);
    // 普通杀（非火/雷）：雷杀会使新横置的 p1 吃铁索传导并解除横置
    let bsha = {
        let id = e.debug_set_deck_top_by(|d| {
            d.kind == engine_native::catalog::CardKind::Sha
                && matches!(d.suit, Suit::Spade | Suit::Club)
        });
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.players[1].hand.push(id);
        id
    };
    assert!(is_black(&e, bsha), "p1 布到黑杀");
    e.debug_set_hp(0, 4);
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: bsha,
            targets: vec![0],
        },
    );
    confirm(&mut e, 0);
    pick_first_card(&mut e);
    settle_optional_confirm(&mut e);
    // 时间片暂停冲刷：Act 直接变更立见而 emit 排队帧需续行（批③ 包加卡
    // 改发牌序暴露——原测试耦合旧 RNG 流恰在窗内结算完毕）
    for _ in 0..8 {
        let r = e.resume();
        eprintln!("b22 dbg resume={:?}", r);
        if matches!(r, CommandOutcome::Rejected(RejectReason::NotSuspended)) {
            break;
        }
        settle_optional_confirm(&mut e);
    }
    {
        let log = e.log();
        let tail: Vec<String> = log.iter().rev().take(6).map(|l| format!("{l:?}")).collect();
        eprintln!("b22 dbg logtail={:?}", tail);
    }
    assert!(e.state_core().player(1).linked, "凝霜：使用者横置");
    assert!(!e.state_core().player(1).face_down, "凝霜：使用者未翻面");
    assert_eq!(hand_len_w(&e, 0), 1, "凝霜：横置支自己摸 1");
    assert_eq!(e.debug_hp(0), 3, "凝霜：杀伤害照常（免伤面无关）");

    // 正例 ②b：已横置 → 改翻面（不摸牌）
    reset_sha(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    {
        let core = e.state_core_mut();
        core.players[1].linked = true;
    }
    let _blk2 = place_sha_color(&mut e, 0, true);
    reset_sha(&mut e);
    let bsha2 = place_sha_color(&mut e, 1, true);
    e.debug_set_hp(0, 4);
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: bsha2,
            targets: vec![0],
        },
    );
    confirm(&mut e, 0);
    pick_first_card(&mut e);
    settle_optional_confirm(&mut e);
    assert!(e.state_core().player(1).face_down, "凝霜：已横置 → 翻面");
    assert_eq!(
        hand_len_w(&e, 0),
        0,
        "凝霜：翻面支不摸牌（弃 1 摸 0 净 -1）"
    );

    // 反例 ②c：已「横置且翻面」→ filter 恒否，无窗
    reset_sha(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    {
        let core = e.state_core_mut();
        core.players[1].linked = true;
        core.players[1].face_down = true;
    }
    let _blk3 = place_sha_color(&mut e, 0, true);
    reset_sha(&mut e);
    let bsha3 = place_sha_color(&mut e, 1, true);
    let hp0 = e.debug_hp(0);
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: bsha3,
            targets: vec![0],
        },
    );
    assert!(e.open_choice().is_none(), "凝霜反例：横置且翻面不触发");
    assert_eq!(e.debug_hp(0), hp0 - 1, "凝霜反例：伤害照常");

    // 反例 ②d：红杀不触发
    reset_sha(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    {
        let core = e.state_core_mut();
        core.players[1].linked = false;
        core.players[1].face_down = false;
    }
    let rsha = place_sha_color(&mut e, 1, false);
    assert!(!is_black(&e, rsha), "p1 布到红杀");
    let hp0 = e.debug_hp(0);
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: rsha,
            targets: vec![0],
        },
    );
    assert!(e.open_choice().is_none(), "凝霜反例：红杀不触发");
    assert_eq!(e.debug_hp(0), hp0 - 1, "凝霜反例：伤害照常");

    // ③ 工神：撤销凝霜，p3 持工神（手牌有 zhuge）；p1 杀 p2 → 确认+
    // 弃 zhuge → 该杀失效（cancelled，牌回 p1 手牌，p2 免伤）
    {
        let def_ns = e.catalog().skill_by_name("ningshuang").expect("凝霜 def");
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        snap.core.skills.remove(&(0, def_ns));
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("撤销凝霜");
    }
    e.debug_grant(3, "gongshen");
    advance_to_play(&mut e, 1);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _zg4 = e.debug_place(3, "zhuge");
    let sha5 = e.debug_place(1, "sha");
    e.debug_set_hp(2, 4);
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha5,
            targets: vec![2],
        },
    );
    confirm(&mut e, 3);
    pick_first_card(&mut e);
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(2), 4, "工神：p2 免伤");
    assert_eq!(hand_len_w(&e, 3), 0, "工神：zhuge 已弃");
    assert_eq!(hand_len_w(&e, 1), 1, "工神：取消的杀留在 p1 手牌");

    // 反例 ③a：取消确认窗 → 杀照常结算
    reset_sha(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _zg5 = e.debug_place(3, "zhuge");
    let sha6 = e.debug_place(1, "sha");
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha6,
            targets: vec![2],
        },
    );
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("工神取消窗");
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(2), 3, "工神反例：取消 → 伤害照常（4-1）");

    // 反例 ③b：p3 无装备牌 → 无窗
    reset_sha(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha7 = e.debug_place(1, "sha");
    let hp2 = e.debug_hp(2);
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha7,
            targets: vec![2],
        },
    );
    assert!(e.open_choice().is_none(), "工神反例：无装备牌不触发");
    assert_eq!(e.debug_hp(2), hp2 - 1, "工神反例：伤害照常");

    // 反例 ③c：无目标用牌（桃）不触发
    reset_sha(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _zg6 = e.debug_place(3, "zhuge");
    e.debug_set_hp(1, 3);
    let tao6 = e.debug_place(1, "tao");
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: tao6,
            targets: vec![],
        },
    );
    assert!(e.open_choice().is_none(), "工神反例：无目标牌不触发");
}
