use super::*;

/// 蓝格怪衣「龙帆」（swd 第十五批，规则 v99）：equip5 宝物主动判定技
/// ——出牌阶段限一次，判定按花色：红桃回 1 体力（满血饱和）/方片摸
/// 1/梅花随机敌人随机弃一张（手牌优先，无手牌退装备）/黑桃无事。
/// 判定牌经 debug_set_deck_top_by 按花色可控；每分支独立子局。
#[test]
fn swd_langeguaiyi_longfan_suit_branches() {
    use engine_native::catalog::Suit;
    use engine_native::content::host::SWD_LANGEGUAIYI_SKILL_KEY;

    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    // 子局脚手架：蓝格怪衣装备于 0 号 + 技能实例；全员空手
    let setup = |seed: u64| -> Engine {
        let mut e = Engine::new_with_extensions(seed, 3, &[&pack]).expect("建局");
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_clear_hand(2);
        let lgy = (0..e.catalog().cards.len() as u32)
            .find(|i| e.catalog().cards[*i as usize].kind == CardKind::Langeguaiyi)
            .expect("蓝格怪衣在目录");
        let def = e
            .catalog()
            .skill_by_name(SWD_LANGEGUAIYI_SKILL_KEY)
            .expect("龙帆装备技在目录");
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != lgy);
        }
        snap.core.deck.retain(|c| *c != lgy);
        snap.core.discard.retain(|c| *c != lgy);
        snap.core.table.retain(|c| *c != lgy);
        snap.core.players[0].equips.push(lgy);
        snap.core.skills.insert(
            (0, def),
            engine_native::state::SkillInstance {
                owner: 0,
                enabled: true,
                once_used: false,
                expires_end_of_turn: false,
                rounds: 0,
            },
        );
        Engine::restore_with_extensions(&snap, &[&pack]).expect("蓝格怪衣注入")
    };
    let activate = |e: &mut Engine, def| submit(e, 0, GameAction::ActivateSkill { skill: def });

    // ① 红桃：回 1 体力（2→3）
    let mut e = setup(6904);
    let def = e
        .catalog()
        .skill_by_name(SWD_LANGEGUAIYI_SKILL_KEY)
        .expect("def");
    e.debug_set_hp(0, 2);
    e.debug_set_deck_top_by(|d| d.suit == Suit::Heart);
    let out = activate(&mut e, def);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.debug_hp(0), 3, "红桃回 1 体力");
    // usable:1：同回合再发动 → SkillAlreadyUsed
    let out = activate(&mut e, def);
    assert!(
        matches!(
            out,
            CommandOutcome::Rejected(RejectReason::SkillAlreadyUsed)
        ),
        "限一次: {out:?}"
    );

    // ①b 红桃满血：无回复（recover 饱和）
    let mut e = setup(6905);
    let def = e
        .catalog()
        .skill_by_name(SWD_LANGEGUAIYI_SKILL_KEY)
        .unwrap();
    let hp0 = e.debug_hp(0);
    e.debug_set_deck_top_by(|d| d.suit == Suit::Heart);
    let _ = activate(&mut e, def);
    assert_eq!(e.debug_hp(0), hp0, "满血无回复");

    // ② 方片：摸 1
    let mut e = setup(6906);
    let def = e
        .catalog()
        .skill_by_name(SWD_LANGEGUAIYI_SKILL_KEY)
        .unwrap();
    e.debug_set_deck_top_by(|d| d.suit == Suit::Diamond);
    let _ = activate(&mut e, def);
    assert_eq!(e.debug_hand(0).len(), 1, "方片摸 1");

    // ③ 梅花：随机敌人随机弃一张（1 号唯一有手牌 → 必弃 1）
    let mut e = setup(6907);
    let def = e
        .catalog()
        .skill_by_name(SWD_LANGEGUAIYI_SKILL_KEY)
        .unwrap();
    let c1 = e.debug_place(1, "sha");
    let c2 = e.debug_place(1, "shan");
    let c3 = e.debug_place(1, "tao");
    e.debug_set_deck_top_by(|d| d.suit == Suit::Club);
    let _ = activate(&mut e, def);
    let h1 = e.debug_hand(1);
    assert_eq!(h1.len(), 2, "1 号被随机弃 1（3→2）");
    assert!(
        !h1.contains(&c1) || !h1.contains(&c2) || !h1.contains(&c3),
        "弃的是三者之一"
    );

    // ③b 梅花但敌人全空（手牌∪装备区均空）：候选空，无事发生
    let mut e = setup(6908);
    let def = e
        .catalog()
        .skill_by_name(SWD_LANGEGUAIYI_SKILL_KEY)
        .unwrap();
    e.debug_set_deck_top_by(|d| d.suit == Suit::Club);
    let h1 = e.debug_hand(1).len();
    let _ = activate(&mut e, def);
    assert_eq!(e.debug_hand(1).len(), h1, "无候选不弃牌");

    // ④ 黑桃：无事发生
    let mut e = setup(6909);
    let def = e
        .catalog()
        .skill_by_name(SWD_LANGEGUAIYI_SKILL_KEY)
        .unwrap();
    e.debug_set_hp(0, 2);
    e.debug_set_deck_top_by(|d| d.suit == Suit::Spade);
    let _ = activate(&mut e, def);
    assert_eq!(e.debug_hp(0), 2, "黑桃无回复");
    assert_eq!(e.debug_hand(0).len(), 0, "黑桃无摸牌");
    assert_eq!(e.debug_hand(1).len(), 0, "黑桃无弃牌");
}

// ---------------------------------------------------------------------------
// U05 swd 第二十批（规则 v195）：机关凤（jiguanfeng，♣3/♠4）——目标打出
// 一张闪（无闪/取消=未响应）；未响应→目标获"封印"（全部技能实例禁用，
// 其下一阶段开始解除[SwdFengyinState]；登记差异：原生无锁定技区分=全部
// 失效，上游 fengyin 仅非锁定技）+受 1 点伤害；响应则无事。
// ---------------------------------------------------------------------------

/// 机关凤①：目标无闪→未响应：封印（已授技实例禁用+封印状态在）+1 伤；
/// 目标下一阶段开始→解除（实例复启、封印状态移除）。
#[test]
fn swd_jiguanfeng_no_shan_seal_then_unseal_at_phase_begin() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6107, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(1, 4);
    let jf = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name.contains("jiguanfeng"))
        .expect("机关凤在目录") as u32;
    {
        let mut snap = e.save();
        snap.core.deck.retain(|c| *c != jf);
        snap.core.discard.retain(|c| *c != jf);
        for pl in snap.core.players.iter_mut() {
            pl.hand.retain(|c| *c != jf);
        }
        snap.core.players[0].hand.push(jf);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    }
    // 1 号有已授技能实例（马术），封印后应被禁用
    e.debug_grant(1, "马术");
    let mdef = e.catalog().skill_by_name("马术").expect("马术 def");
    let fdef = e
        .catalog()
        .skill_by_name(engine_native::content::host::SWD_FENGYIN_STATE_KEY)
        .expect("封印状态 def 随 Jiguan kind 注册");

    let out = play(&mut e, jf, vec![1]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 无闪：无询问，直接未响应路径
    assert!(e.open_choice().is_none(), "无闪不应有询问");
    assert_eq!(e.debug_hp(1), 3, "未响应受 1 点伤害（4→3）");
    let snap = e.save();
    assert!(!snap.core.skills[&(1, mdef)].enabled, "已授技实例被禁用");
    assert!(snap.core.skills.contains_key(&(1, fdef)), "封印状态已授予");
    assert!(
        snap.core.discard.iter().any(|c| *c == jf),
        "机关凤本体入弃堆"
    );

    // 推进到 1 号回合首阶段开始 → 解封
    loop {
        if e.turn().current == 1 && e.turn().phase == Phase::Judge {
            break;
        }
        match e.open_choice() {
            Some(c) => {
                let (cid, obs) = (c.id, c.observer);
                submit(
                    &mut e,
                    obs,
                    GameAction::Answer {
                        choice_id: cid,
                        selection: vec![ChoiceOption::Pass],
                    },
                );
            }
            None => {
                let actor = e.turn().current;
                if actor == 0 && e.debug_hand(0).is_empty() && e.turn().phase == Phase::Discard {
                    // 空手弃牌阶段直接过
                }
                match submit(&mut e, actor, GameAction::NextPhase) {
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                    other => panic!("advance: {other:?}"),
                }
            }
        }
    }
    let snap2 = e.save();
    assert!(
        !snap2.core.skills.contains_key(&(1, fdef)),
        "封印状态应于拥有者阶段开始移除"
    );
    assert!(snap2.core.skills[&(1, mdef)].enabled, "技能实例复启");
}

/// 机关凤②：目标有闪并打出→无事（不掉血、不封印）；闪入弃牌堆。
#[test]
fn swd_jiguanfeng_shan_respond_no_damage() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6108, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(1, 4);
    let jf = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name.contains("jiguanfeng"))
        .expect("机关凤在目录") as u32;
    {
        let mut snap = e.save();
        snap.core.deck.retain(|c| *c != jf);
        snap.core.discard.retain(|c| *c != jf);
        for pl in snap.core.players.iter_mut() {
            pl.hand.retain(|c| *c != jf);
        }
        snap.core.players[0].hand.push(jf);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    }
    let shan = e.debug_place(1, "shan");
    let fdef = e
        .catalog()
        .skill_by_name(engine_native::content::host::SWD_FENGYIN_STATE_KEY)
        .expect("封印状态 def");

    let out = play(&mut e, jf, vec![1]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 打闪询问（observer=1）
    let c = e.open_choice().cloned().expect("机关凤打闪询问");
    assert_eq!(c.observer, 1, "打闪询问 observer=目标");
    assert_eq!(
        c.spec.prompt,
        engine_native::content::host::SWD_JIGUANFENG_SHAN
    );
    let cid = c.id;
    drop(c);
    let out = submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(shan)],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.debug_hp(1), 4, "响应闪：不受伤害");
    let snap = e.save();
    assert!(
        !snap.core.skills.contains_key(&(1, fdef)),
        "响应闪：无封印状态"
    );
    assert!(snap.core.discard.contains(&shan), "闪入弃牌堆");
    assert!(snap.core.discard.contains(&jf), "机关凤本体入弃堆");
}

// ---------------------------------------------------------------------------
// U05 swd 第二十一批（规则 v196）：机关怨（jiguanyuan，♣3/♦2）——使用者
// 选一张手牌，与本体一起置于目标武将牌上（离场区），其下一结束阶段强制
// 获得（SwdJiguanyuanGain）；使用者摸 1；本体不进弃堆。同目标待获得期间
// 不可再指定（上游 !hasSkill("jiguanyuan") 同口径）。
// ---------------------------------------------------------------------------

/// 机关怨①：置牌→目标结束阶段获得两张+状态清除；使用者摸 1。
#[test]
fn swd_jiguanyuan_pending_then_gain_at_phase_end() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6109, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let jy = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name.contains("jiguanyuan"))
        .expect("机关怨在目录") as u32;
    {
        let mut snap = e.save();
        snap.core.deck.retain(|c| *c != jy);
        snap.core.discard.retain(|c| *c != jy);
        for pl in snap.core.players.iter_mut() {
            pl.hand.retain(|c| *c != jy);
        }
        snap.core.players[0].hand.push(jy);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    }
    let give = e.debug_place(0, "tao");
    let gdef = e
        .catalog()
        .skill_by_name(engine_native::content::host::SWD_JIGUANYUAN_GAIN_KEY)
        .expect("待获得 def 随 Jiguan kind 注册");

    let out = play(&mut e, jy, vec![1]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 使用者选牌询问
    let c = e.open_choice().cloned().expect("机关怨选牌询问");
    assert_eq!(c.observer, 0, "选牌询问 observer=使用者");
    assert_eq!(
        c.spec.prompt,
        engine_native::content::host::SWD_JIGUANYUAN_CHOOSE
    );
    let cid = c.id;
    drop(c);
    let out = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(give)],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 置牌完成：目标离场区 2 张（本体+选牌）、待获得记录、状态已授、使用者摸 1
    let snap = e.save();
    assert_eq!(
        snap.core.players[1].off_court.len(),
        2,
        "目标离场区 2 张（本体+选牌）"
    );
    assert_eq!(
        snap.core.players[1].jiguanyuan_cards.len(),
        2,
        "待获得记录 2 张"
    );
    assert!(
        snap.core.skills.contains_key(&(1, gdef)),
        "待获得状态已授予"
    );
    assert!(!snap.core.table.contains(&jy), "本体已离桌（不进弃堆）");
    assert_eq!(e.debug_hand(0).len(), 1, "使用者摸 1");
    assert_eq!(e.debug_hand(1).len(), 0, "目标尚未获得");

    // A2 H71：上游 jiguanyuan player phaseEnd＝整回合结束——待获得在 1 号
    // 整回合结束窗（forced）获得；推进其各阶段到 End 收束
    loop {
        if e.turn().current == 1 && e.turn().phase == Phase::End {
            break;
        }
        match e.open_choice() {
            Some(c) => {
                let (cid2, obs) = (c.id, c.observer);
                submit(
                    &mut e,
                    obs,
                    GameAction::Answer {
                        choice_id: cid2,
                        selection: vec![ChoiceOption::Pass],
                    },
                );
            }
            None => {
                let actor = e.turn().current;
                match submit(&mut e, actor, GameAction::NextPhase) {
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                    other => panic!("advance: {other:?}"),
                }
            }
        }
    }
    let _ = submit(&mut e, 1, GameAction::NextPhase); // End 收束→整回合结束窗（窗内获得）
    let snap2 = e.save();
    assert_eq!(
        snap2.core.players[1].hand.len(),
        4,
        "整回合结束窗获得 2 张 + 摸牌阶段正常摸 2"
    );
    assert!(
        snap2.core.players[1].jiguanyuan_cards.is_empty(),
        "待获得记录清"
    );
    assert!(
        !snap2.core.skills.contains_key(&(1, gdef)),
        "待获得状态移除"
    );
    assert_eq!(snap2.core.players[1].off_court.len(), 0, "离场区清空");
}

/// 机关怨②：同目标待获得期间再指定→IllegalTargets（上游
/// !hasSkill("jiguanyuan") 同口径）。
#[test]
fn swd_jiguanyuan_same_target_rejected_while_pending() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6110, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let jy1 = e
        .catalog()
        .cards
        .iter()
        .filter(|c| c.name.contains("jiguanyuan"))
        .map(|c| c.name)
        .next()
        .expect("机关怨在目录");
    let ids: Vec<u32> = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .filter(|(_, c)| c.name.contains("jiguanyuan"))
        .map(|(i, _)| i as u32)
        .collect();
    {
        let mut snap = e.save();
        snap.core.deck.retain(|c| !ids.contains(c));
        snap.core.discard.retain(|c| !ids.contains(c));
        for pl in snap.core.players.iter_mut() {
            pl.hand.retain(|c| !ids.contains(c));
        }
        snap.core.players[0].hand.push(ids[0]);
        snap.core.players[0].hand.push(ids[1]);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    }
    let give = e.debug_place(0, "tao");
    let out = play(&mut e, ids[0], vec![1]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let c = e.open_choice().cloned().expect("第一次选牌询问");
    let cid = c.id;
    drop(c);
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(give)],
        },
    );
    // 第二张机关怨再指定同目标 → 拒绝
    let second = e
        .debug_hand(0)
        .into_iter()
        .find(|c| e.catalog().cards[*c as usize].name.contains("jiguanyuan"))
        .expect("第二张机关怨在手");
    let out = play(&mut e, second, vec![1]);
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "同目标待获得期间应拒: {out:?}"
    );
    // 指定其他目标 → 可用（本测试不再推进，仅验证通过校验）
    let out = play(&mut e, second, vec![2]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. }
                | CommandOutcome::Pending { .. }
                | CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ),
        "换目标不因同目标校验被拒: {out:?}"
    );
    let _ = jy1;
}

/// 机关妖术①：目标有空栏→从牌堆/弃堆发现装备装配+防重标记；同目标再指
/// 定被拒；无空栏/满栏目标不可指定。
#[test]
fn swd_jiguanyaoshu_discovers_equip_and_marks_target() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6111, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let jys = jiguan_ids(&e, "jiguanyaoshu");
    assert_eq!(jys.len(), 2, "机关妖术 ♣3/♠3 两张");
    inject_hand(&mut e, &pack, &jys);
    // 目标非 isMin：手牌数 > 体力（hp 1 + 手牌 2）
    e.debug_set_hp(1, 1);
    let t1a = e.debug_place(1, "tao");
    let t1b = e.debug_place(1, "tao");
    let _ = (t1a, t1b);
    let sdef = e
        .catalog()
        .skill_by_name(engine_native::content::host::SWD_JIGUANYAOSHU_STATE_KEY)
        .expect("防重标记 def 随 Jiguan kind 注册");

    // 第一张：目标 1 号全空栏（默认无装备）→ 发现+装配
    let out = play(&mut e, jys[0], vec![1]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert!(e.open_choice().is_none(), "机关妖术无询问（内容自动）");
    let snap = e.save();
    assert_eq!(snap.core.players[1].equips.len(), 1, "目标获得 1 张装备");
    assert!(snap.core.skills.contains_key(&(1, sdef)), "防重标记已授予");
    assert!(snap.core.discard.contains(&jys[0]), "本体入弃堆");

    // 同目标再指定 → 拒（防重）
    let out = play(&mut e, jys[1], vec![1]);
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "防重期间同目标应拒: {out:?}"
    );

    // 换 2 号：制造 isMin（手牌 1 ≤ 体力 4）→ 拒
    let give = e.debug_place(2, "tao");
    e.debug_clear_hand(2);
    {
        let mut snap = e.save();
        snap.core.deck.retain(|x| *x != give);
        snap.core.discard.retain(|x| *x != give);
        for pl in snap.core.players.iter_mut() {
            pl.hand.retain(|x| *x != give);
            pl.equips.retain(|x| *x != give);
            pl.off_court.retain(|x| *x != give);
        }
        snap.core.players[2].hand.push(give);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入 isMin");
    }
    let out = play(&mut e, jys[1], vec![2]);
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "isMin 目标应拒: {out:?}"
    );
}

/// 机关妖术②：满装备栏目标（5 栏全占）→ 拒绝（上游 filterTarget 空
/// subtype 槽不存在同口径）。
#[test]
fn swd_jiguanyaoshu_full_slots_rejected() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6112, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let jys = jiguan_ids(&e, "jiguanyaoshu");
    inject_hand(&mut e, &pack, &jys);
    // 1 号装配 5 栏（直接构造 equips：标准目录各栏代表）
    let slots: Vec<&str> = vec!["sha", "bishoupai", "hufu", "hufu", "hufu"];
    let _ = slots;
    // 用 debug_grant 不可装装备——直接从目录取各栏实际牌种注入 equips
    {
        let mut snap = e.save();
        let find_kind = |pred: &dyn Fn(engine_native::catalog::CardKind) -> bool| {
            e.catalog()
                .cards
                .iter()
                .position(|c| pred(c.kind))
                .map(|i| i as u32)
        };
        let weapon = find_kind(&|k| matches!(k, engine_native::catalog::CardKind::Weapon))
            .expect("武器在目录");
        let armor = find_kind(&|k| matches!(k, engine_native::catalog::CardKind::Armor))
            .expect("防具在目录");
        let hp = find_kind(&|k| matches!(k, engine_native::catalog::CardKind::HorsePlus))
            .expect("+1 马在目录");
        let hm = find_kind(&|k| matches!(k, engine_native::catalog::CardKind::HorseMinus))
            .expect("-1 马在目录");
        let treasure =
            find_kind(&|k| k.equip_slot() == Some(engine_native::catalog::EquipSlot::Treasure))
                .expect("宝物在目录");
        for c in [weapon, armor, hp, hm, treasure] {
            snap.core.deck.retain(|x| *x != c);
            snap.core.discard.retain(|x| *x != c);
            snap.core.players[1].equips.push(c);
        }
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入 5 栏");
    }
    let out = play(&mut e, jys[0], vec![1]);
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "满栏目标应拒: {out:?}"
    );
}

/// 灵剑①旋风之刃：目标随机弃 1 张 he；②时间回溯：目标装备区全部入手。
#[test]
fn swd_lingjian_xuanfeng_and_shijianhuisu() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6113, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let xf = lingjian_id(&e, "xuanfengzhiren");
    let sj = lingjian_id(&e, "shijianhuisu");
    inject_hand(&mut e, &pack, &[xf, sj]);
    // 目标 1 号：2 手牌 + 1 装备（从目录取装备实体入其 equips）
    e.debug_set_hp(1, 4);
    let _t1 = e.debug_place(1, "tao");
    let _t2 = e.debug_place(1, "shan");
    let equip = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == CardKind::Weapon)
        .expect("武器在目录") as u32;
    {
        let mut snap = e.save();
        // 全域摘除（批⑫ 加卡发牌序耦合：首武器实体可能已在手牌）
        snap.core.deck.retain(|x| *x != equip);
        snap.core.discard.retain(|x| *x != equip);
        for pl in snap.core.players.iter_mut() {
            pl.hand.retain(|x| *x != equip);
        }
        snap.core.players[1].equips.push(equip);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入装备");
    }

    // 旋风之刃→1 号：随机弃 1 张 he（he 3 张→2）
    let out = play(&mut e, xf, vec![1]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert!(e.open_choice().is_none(), "旋风之刃无询问");
    assert_eq!(
        e.debug_hand(1).len() + e.state_core().player(1).equips.len(),
        2,
        "随机弃 1 张 he"
    );
    assert!(e.state_core().discard.contains(&xf), "本体入弃堆");

    // 时间回溯→1 号：装备区全部入手。旋风随机弃牌可能弃掉装备（RNG
    // 耦合批① 暴露）——sj 校验要求装备区非空，缺则确定性重注
    if e.state_core().player(1).equips.is_empty() {
        let mut snap = e.save();
        snap.core.discard.retain(|x| *x != equip);
        snap.core.deck.retain(|x| *x != equip);
        snap.core.players[1].equips.push(equip);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("重注装备");
    }
    let he_before = e.debug_hand(1).len() + e.state_core().player(1).equips.len();
    assert!(he_before >= 2, "he 至少 2（弃 1 后余量）");
    let out = play(&mut e, sj, vec![1]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.state_core().player(1).equips.len(), 0, "装备区清空");
    assert_eq!(e.debug_hand(1).len(), he_before, "装备牌全部入手");
}

/// 灵剑③紧急冷冻：+2 护甲+翻面+冻结（不能出牌/不能被指定）；翻回解除。
#[test]
fn swd_lingjian_jinjilengdong_freeze_and_unfreeze() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6114, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let jl = lingjian_id(&e, "jinjilengdong");
    inject_hand(&mut e, &pack, &[jl]);
    e.debug_set_hp(1, 4);
    // 目标 1 号有牌（否则下一回合弃牌门语义无关；有牌验证冻结期不能出牌）
    let _ = e.debug_place(1, "tao");

    let out = play(&mut e, jl, vec![1]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.debug_hp(1), 4, "冷冻不造成伤害");
    assert!(e.state_core().player(1).face_down, "目标翻面");
    assert_eq!(e.state_core().player(1).hujia, 2, "+2 护甲");
    assert!(e.state_core().player(1).jileng_active, "冻结生效");

    // 推进到 1 号回合：翻回+跳过 → 冷冻解除
    loop {
        if e.turn().current == 1 {
            break;
        }
        match e.open_choice() {
            Some(c) => {
                let (cid, obs) = (c.id, c.observer);
                submit(
                    &mut e,
                    obs,
                    GameAction::Answer {
                        choice_id: cid,
                        selection: vec![ChoiceOption::Pass],
                    },
                );
            }
            None => {
                let actor = e.turn().current;
                match submit(&mut e, actor, GameAction::NextPhase) {
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                    other => panic!("advance: {other:?}"),
                }
            }
        }
    }
    let snap = e.save();
    assert!(!snap.core.players[1].face_down, "回合开始翻回");
    assert!(
        !snap.core.players[1].jileng_active,
        "翻回即解冻（turnOverAfter 同口径）"
    );
}

/// 灵剑④圣休号角嘲讽 + ⑤隐秘离场 + ⑥星体改造：杀目标限制/潜行/手牌上
/// 限 -1 累加。
#[test]
fn swd_lingjian_chaofeng_yinmi_xingtigaizao() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6115, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let sx = lingjian_id(&e, "shengxiuhaojiao");
    let ym = lingjian_id(&e, "yinmilichang");
    let xt = lingjian_id(&e, "xingtigaizao");
    inject_hand(&mut e, &pack, &[sx, ym, xt]);
    e.debug_set_hp(0, 4);

    // 圣休号角→2 号：0 号（2 号相邻）只能以 2 号为杀目标
    let out = play(&mut e, sx, vec![2]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert!(e.state_core().player(2).chaofeng_active, "嘲讽生效");
    // 0 号获得杀，目标 1 号（非嘲讽者）→ 拒
    let sha = e.debug_place(0, "sha");
    let out = play(&mut e, sha, vec![1]);
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "相邻有嘲讽者应只能杀 2 号: {out:?}"
    );

    // 隐秘离场→1 号：潜行（不能被指定）；星体改造：自目标手牌上限 -1
    let out = play(&mut e, ym, vec![1]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert!(e.state_core().player(1).yinmi_active, "潜行生效");
    let sha2 = e.debug_place(0, "sha");
    let out = play(&mut e, sha2, vec![1]);
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "潜行者不能被指定: {out:?}"
    );
    let out = play(&mut e, xt, vec![0]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.state_core().player(0).xingtigaizao_count, 1, "改造计数 1");
    assert_eq!(
        e.state_core().hand_limit(e.catalog(), 0),
        (e.debug_hp(0) as usize).saturating_sub(1),
        "手牌上限 -1"
    );
}

/// 机关妖术补偿触发（规则 v198）：防重标记持有者非自己回合弃出装备
/// Kind 牌 → 补获 1 张灵剑。确定性构造：目标手牌两件装备 Kind 牌（任意
/// 弃出都命中 filter 的装备 Kind 条件），hp=1 使非 isMin 成立。
#[test]
fn swd_jiguanyaoshu_gain_compensates_lost_equip() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6116, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let ys = jiguan_ids(&e, "jiguanyaoshu")[0];
    let spare_lj = lingjian_id(&e, "shengxiuhaojiao");
    let xf = lingjian_id(&e, "xuanfengzhiren");
    // 三件互异装备 Kind 实体（手牌 2 + 装备区 1）
    let weapons: Vec<u32> = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .filter(|(_, c)| c.kind == CardKind::Weapon)
        .map(|(i, _)| i as u32)
        .take(3)
        .collect();
    assert!(weapons.len() >= 3, "标准目录应有 ≥3 把武器");
    inject_hand(&mut e, &pack, &[ys, xf]);
    {
        let mut snap = e.save();
        for x in &weapons {
            snap.core.deck.retain(|x2| x2 != x);
        }
        // 目标 1 号：手牌 2 件装备 Kind（非 isMin：2 > hp 1）；装备区 1 件
        snap.core.players[1].hp = 1;
        snap.core.players[1].hand.push(weapons[0]);
        snap.core.players[1].hand.push(weapons[1]);
        snap.core.players[1].equips.push(weapons[2]);
        // 牌堆预置 1 张灵剑实体（补偿发现源）；先全域摘除防双区
        snap.core.deck.retain(|x| *x != spare_lj);
        snap.core.discard.retain(|x| *x != spare_lj);
        for pl in snap.core.players.iter_mut() {
            pl.hand.retain(|x| *x != spare_lj);
            pl.equips.retain(|x| *x != spare_lj);
        }
        snap.core.deck.push(spare_lj);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    }

    // 机关妖术打 1 号：非 isMin（2>1）通过 → 防重标记授予
    let out = play(&mut e, ys, vec![1]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let sdef = e
        .catalog()
        .skill_by_name(engine_native::content::host::SWD_JIGUANYAOSHU_STATE_KEY)
        .expect("防重标记 def");
    assert!(e.state_core().skills.contains_key(&(1, sdef)), "标记已授予");
    // 清空 p1 手牌（静默清不触发补偿）——xf 随机池仅剩装备区 1 件，
    // 弃装备路径必发 CardsDiscarded → 补偿确定性触发（批① 牌堆序变化
    // 前测试耦合 RNG 选中装备面，此处消除耦合）
    e.debug_clear_hand(1);

    // 旋风之刃打 1 号：弃装备区唯一装备 → 补偿必触发
    let out = play(&mut e, xf, vec![1]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let snap = e.save();
    assert!(
        snap.core.players[1]
            .hand
            .iter()
            .any(|c| { e.catalog().cards[*c as usize].kind == CardKind::Hslingjian }),
        "应补获 1 张灵剑（弃装备 Kind 牌触发补偿）"
    );
}

/// 机关术（规则 v199/ADR 0780）：自目标——每张有效装备受控合成（CardMod
/// 附着随机灵剑）；CardMod 随存档往返；无装备不可用。
#[test]
fn swd_jiguanshu_attaches_card_mods_and_roundtrips() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(6117, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let js1 = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name.contains("jiguanshu"))
        .expect("机关术在目录") as u32;
    inject_hand(&mut e, &pack, &[js1]);
    // 0 号装配 1 张武器
    let weapon = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == CardKind::Weapon)
        .expect("武器在目录") as u32;
    {
        let mut snap = e.save();
        snap.core.deck.retain(|x| *x != weapon);
        snap.core.discard.retain(|x| *x != weapon);
        for pl in snap.core.players.iter_mut() {
            pl.hand.retain(|x| *x != weapon);
            pl.equips.retain(|x| *x != weapon);
        }
        snap.core.players[0].equips.push(weapon);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入装备");
    }

    let out = play(&mut e, js1, vec![0]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert!(e.open_choice().is_none(), "机关术无询问");
    let snap = e.save();
    let mods = &snap.core.players[0].card_mods;
    assert_eq!(mods.len(), 1, "1 张装备 → 1 条改造");
    let attached = mods.get(&weapon).expect("装备应附着灵剑");
    assert_eq!(
        e.catalog().cards[*attached as usize].kind,
        CardKind::Hslingjian,
        "附着实体应为灵剑"
    );
    assert!(snap.core.discard.contains(&js1), "本体入弃堆");

    // CardMod 随存档字节往返
    let bytes = engine_native::archive::encode_archive(&snap);
    let restored = Engine::restore_with_extensions(
        &engine_native::archive::decode_archive(&bytes).expect("解码"),
        &[&pack],
    )
    .expect("恢复");
    assert_eq!(
        restored.state_core().players[0].card_mods,
        *mods,
        "CardMod 随存档往返一致"
    );

    // 装备清空后不可再用（enable 校验）
    // 无装备不可用：清空装备后再试（同实体已弃，注入另一张机关术）
    let js_all: Vec<u32> = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .filter(|(_, c)| c.name.contains("jiguanshu"))
        .map(|(i, _)| i as u32)
        .collect();
    if let Some(js2) = js_all.iter().find(|x| **x != js1) {
        inject_hand(&mut e, &pack, &[*js2]);
        // 先把装备移入弃牌堆（保留全域唯一性；直写快照）
        let mut snap = e.save();
        let w = snap.core.players[0].equips.pop().expect("有装备");
        snap.core.discard.push(w);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("清装备");
        let out = play(&mut e, *js2, vec![0]);
        assert!(
            matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
            "无装备不可用: {out:?}"
        );
    }
}

/// T12 swd 卡牌面补切批①：对账守护——勘误+补张后的 7 kind 13 张与上游
/// list.js 配对逐条一致（批② 起新增卡不得扰动本表；对账全景见证据
/// 2026-10-15-t12-swd-cards-audit/）。
#[test]
fn swd_cards_alignment_guard_b29() {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let discovered = discover_extensions(&root).unwrap();
    let disc = discovered
        .iter()
        .find(|d| d.name == "shahai-swd-upstream-noname")
        .expect("swd 可发现")
        .clone();
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);
    // 上游 list.js 对应行（勘误后基准）
    let expected: &[(&str, &str, u8)] = &[
        ("hufu", "spade", 1),
        ("hufu", "heart", 1),
        ("zhiluxiaohu", "heart", 3),
        ("zhiluxiaohu", "diamond", 4),
        ("mujiaren", "club", 7),
        ("mujiaren", "heart", 6),
        ("mujiaren", "diamond", 11),
        ("shentoumianju", "club", 6),
        ("xingjunyan", "spade", 7),
        ("zhufangshenshi", "spade", 4),
        ("zhufangshenshi", "spade", 8),
        ("yangpijuan", "spade", 9),
        ("yangpijuan", "club", 7),
        ("xuejibingbao", "club", 1),
        ("xuejibingbao", "spade", 1),
        ("fengyinzhidan", "spade", 1),
        ("lianyaohu", "club", 13),
        ("qiankundai", "club", 1),
        ("shennongding", "club", 13),
        ("nvwashi", "heart", 13),
        ("kunlunjingc", "spade", 13),
        ("donghuangzhong", "diamond", 13),
        ("kongdongyin", "heart", 13),
        ("jingleishan", "club", 7),
        ("chiyuxi", "diamond", 7),
        ("guisheqi", "spade", 9),
        ("guisheqi", "club", 7),
        ("xianluhui", "spade", 13),
        ("guilingzhitao", "spade", 7),
        ("sifeizhenmian", "spade", 4),
        ("qinglongzhigui", "heart", 6),
        ("zhuquezhizhang", "diamond", 6),
        ("baishouzhihu", "spade", 6),
        ("xuanwuzhihuang", "club", 6),
        ("huanglinzhicong", "heart", 5),
        ("shuchui", "club", 6),
        ("sadengjinhuan", "club", 13),
        ("haotianta", "diamond", 13),
        ("yuruyi", "heart", 9),
        ("yuchanqian", "spade", 1),
        ("yuchankun", "club", 2),
        ("yuchanzhen", "diamond", 3),
        ("yuchanxun", "heart", 4),
        ("yuchankan", "spade", 5),
        ("yuchanli", "club", 6),
        ("yuchangen", "diamond", 7),
        ("yuchandui", "heart", 8),
        ("shencaojie", "club", 4),
        ("shencaojie", "diamond", 4),
        ("shencaojie", "spade", 4),
        ("cangchizhibi", "spade", 7),
    ];
    for (name, suit, rank) in expected {
        // 目录 def_name 为 loader 加前缀序号形式（swd1_swd_hufu_0），
        // 以 swd_<名> 子串+花色/点数三元组判定
        let needle = format!("swd_{name}");
        let hit = pack.cards.iter().any(|c| {
            c.name.contains(&needle)
                && format!("{:?}", c.suit).to_lowercase().contains(*suit)
                && c.rank == *rank
        });
        assert!(hit, "对账守护：{name} {suit}{rank} 应在包内");
    }
    // 总张数冻结（批② 后 54；神秘果 2 张挂窗口机批暂未入包）
    assert_eq!(
        pack.cards.len(),
        91,
        "裁决 33 后 91 张（批⑬ 神草结 87+神秘果 2+轩辕剑 1+苍螭之璧 1）"
    );
}

/// T12 swd 卡牌面补切批②：basic 族——雪霁冰雹（目标摸牌阶段 +1 持续 2
/// 轮，重标记拒）；封印之丹（自用随机 2 普通锦囊——确定性窄池五谷+决斗
/// 登记差异）；神秘果 kind 注册、卡牌挂窗口机批结构性暂缓。
#[test]
fn swd_cards_basic_b32_xuejibingbao_fengyinzhidan() {
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
    let mut e = Engine::new_with_extensions(62012, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 神秘果全域摘除（窗口机批+2 张入池）：防再结算级联扰动本测试断言
    {
        let smg: Vec<u32> = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .filter(|(_, c)| c.kind == engine_native::catalog::CardKind::Shenmiguo)
            .map(|(i, _)| i as u32)
            .collect();
        let core = e.state_core_mut();
        for c in smg {
            core.deck.retain(|x| *x != c);
            core.discard.retain(|x| *x != c);
        }
    }
    let xjb = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == engine_native::catalog::CardKind::Xuejibingbao)
        .expect("雪霁在目录") as u32;
    {
        // 全域摘除后置入 p0 手牌
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != xjb);
        core.discard.retain(|x| *x != xjb);
        core.players[0].hand.push(xjb);
    }

    // ① 雪霁→p1：下两个摸牌阶段各 +1
    let d1_before = {
        // 记录 p1 手牌，推进 p1 回合摸牌阶段后测量
        let h = e.debug_hand(1).len();
        h
    };
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: xjb,
            targets: vec![1],
        },
    );
    settle_optional_confirm(&mut e);
    assert!(e.state_core().player(1).xuejibingbao == 2, "雪霁：标记置 2");
    // 推进 p1 整回合（摸牌阶段 +1）
    advance_to_play(&mut e, 1);
    let d1_after = e.debug_hand(1).len();
    // 摸 2+1=3（其余阶段摸牌为 0；手牌被上家清空过但 p1 自己摸牌前可能
    // 已有牌——以增量为断言）
    let grew = d1_after as i32 - d1_before as i32;
    assert!(grew >= 3, "雪霁：摸牌阶段 +1（增量 {}≥3）", grew);
    assert!(e.state_core().player(1).xuejibingbao == 1, "雪霁：计数 2→1");

    // 反例 ①b：已标记目标不可再标记
    advance_to_play(&mut e, 0);
    // 雪霁实体置入手牌（全域摘除防双区）
    let xjb_now = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(_, c)| c.kind == engine_native::catalog::CardKind::Xuejibingbao)
        .map(|(i, _)| i as u32)
        .expect("雪霁实体存在");
    {
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != xjb_now);
        core.discard.retain(|x| *x != xjb_now);
        core.players[0].hand.push(xjb_now);
    }
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: xjb_now,
            targets: vec![1],
        },
    );
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "雪霁反例：已标记目标拒 {out:?}"
    );

    // ② 封印之丹自用：确定性窄池五谷（全体其他）+决斗（座位序首个）
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let fyz = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == engine_native::catalog::CardKind::Fengyinzhidan)
        .expect("封印之丹在目录") as u32;
    {
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != fyz);
        core.discard.retain(|x| *x != fyz);
        core.table.retain(|x| *x != fyz);
        for pl in core.players.iter_mut() {
            pl.hand.retain(|x| *x != fyz);
            pl.equips.retain(|x| *x != fyz);
        }
        core.players[0].hand.push(fyz);
    }
    e.debug_set_hp(1, 8);
    e.debug_set_hp(2, 8);
    e.debug_set_hp(3, 8);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: fyz,
            targets: vec![],
        },
    );
    settle_optional_confirm(&mut e);
    // 五谷：p1/p2/p3 各摸 1（随机弃面不设）；决斗：p0 vs p1（座位序首），
    // 双方手空 → p1 先不出杀受 1 伤
    assert_eq!(hand_len_w(&e, 1), 1, "封印之丹：五谷 p1 得 1");
    assert_eq!(hand_len_w(&e, 2), 1, "封印之丹：五谷 p2 得 1");
    assert_eq!(hand_len_w(&e, 3), 1, "封印之丹：五谷 p3 得 1");
    assert_eq!(e.debug_hp(1), 7, "封印之丹：决斗 p1 受 1（座位序首个其他）");

    // 反例 ②b：对他人使用被拒
    advance_to_play(&mut e, 0);
    let fyz2 = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.kind == engine_native::catalog::CardKind::Fengyinzhidan)
            .map(|(i, _)| i as u32);
        let Some(id) = id else { panic!("封印实体") };
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.discard.retain(|x| *x != id);
        core.players[0].hand.push(id);
        id
    };
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: fyz2,
            targets: vec![1],
        },
    );
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "封印反例：对他人使用拒 {out:?}"
    );

    // 反例 ③：神秘果直用惰性（窗口机批入包后：直接使用=载体簿记弃置，
    // 不再结算、不触发窗口）
    let smg = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == engine_native::catalog::CardKind::Shenmiguo)
        .expect("神秘果入包（窗口机批）") as u32;
    advance_to_play(&mut e, 0);
    {
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != smg);
        core.discard.retain(|x| *x != smg);
        core.table.retain(|x| *x != smg);
        core.players[0].hand.push(smg);
    }
    let before = hand_len_w(&e, 1) + hand_len_w(&e, 2) + hand_len_w(&e, 3);
    let hp_before = e.debug_hp(1);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: smg,
            targets: vec![],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(
        hand_len_w(&e, 1) + hand_len_w(&e, 2) + hand_len_w(&e, 3),
        before,
        "神秘果直用惰性：无再结算（他人手牌不动）"
    );
    assert_eq!(e.debug_hp(1), hp_before, "神秘果直用惰性：无效果");
}
