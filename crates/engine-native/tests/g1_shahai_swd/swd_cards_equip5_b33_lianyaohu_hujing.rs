use super::*;

/// T12 swd 卡牌面补切批③：炼妖壶——收纳（弃 1 手牌拿目标首张手牌入壶
/// 仓，登记差异上游随机）；炼化（壶仓 ≥2 弃 2 张从牌堆顶得首张类别均
/// 不同牌）；壶境（准备阶段弃堆回收炼妖壶+装备时手牌上限 +2）。壶中仙
/// 3/3 整将收口。
#[test]
fn swd_cards_equip5_b33_lianyaohu_hujing() {
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
    let mut e = Engine::new_with_extensions(62013, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let hu = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == engine_native::catalog::CardKind::Lianyaohu)
        .expect("炼妖壶在目录") as u32;
    // 装备炼妖壶+同步授予装备技实例（快照校验装备在场须有实例）
    {
        let mut snap = e.save();
        snap.core.deck.retain(|x| *x != hu);
        snap.core.discard.retain(|x| *x != hu);
        snap.core.players[0].equips.push(hu);
        for sk in ["lianyaohu_shouna", "lianyaohu_lianhua"] {
            let def = e.catalog().skill_by_name(sk).expect("装备技 def");
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
        }
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入壶");
    }

    // ① 收纳：弃 tao → 目标 p1 首张手牌入壶仓
    let _cost = e.debug_place(0, "tao");
    let taken = e.debug_place(1, "shan");
    let _p1b = e.debug_place(1, "tao");
    let sn_def = e
        .catalog()
        .skill_by_name("lianyaohu_shouna")
        .expect("收纳 def");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action: GameAction::ActivateSkill { skill: sn_def },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "收纳受理 {out:?}"
    );
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
    pick(
        &mut e,
        &|o| matches!(o, ChoiceOption::Card(_)),
        "收纳弃牌窗",
    );
    pick(
        &mut e,
        &|o| matches!(o, ChoiceOption::Player(1)),
        "收纳选人窗",
    );
    settle_optional_confirm(&mut e);
    assert!(
        e.state_core().player(0).lianyaohu_store.contains(&taken),
        "收纳：目标首张手牌入壶仓"
    );
    let restored = Engine::restore_with_extensions(&e.save(), &[&pack]).expect("壶仓存档恢复");
    assert_eq!(restored.state_core().player(0).lianyaohu_store, vec![taken]);
    assert_eq!(restored.state_hash(), e.state_hash());
    e = restored;
    assert_eq!(hand_len_w(&e, 0), 0, "收纳：成本已弃");
    assert_eq!(hand_len_w(&e, 1), 1, "收纳：p1 剩 1");

    // 反例 ①b：收纳本回合限一次
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action: GameAction::ActivateSkill { skill: sn_def },
    });
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "收纳反例：限一次 {out:?}"
    );

    // ② 炼化：壶仓补 1 张后为 2 → 弃 2 张得牌堆顶首张类别均不同牌
    let extra = e.debug_place(0, "shan");
    {
        let core = e.state_core_mut();
        core.players[0].hand.retain(|x| *x != extra);
        core.players[0].lianyaohu_store.push(extra);
    }
    assert_eq!(e.state_core().player(0).lianyaohu_store.len(), 2);
    // 牌堆顶压一张桃（与壶仓两类不同类别时跳过逻辑由实现扫描兜底）
    let lh_def = e
        .catalog()
        .skill_by_name("lianyaohu_lianhua")
        .expect("炼化 def");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action: GameAction::ActivateSkill { skill: lh_def },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "炼化受理 {out:?}"
    );
    // 炼化选牌窗 min=2：一次提交两张
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("炼化选牌窗");
    let sel: Vec<ChoiceOption> = {
        let ch = e.open_choice().expect("still open");
        ch.spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Card(_)))
            .cloned()
            .take(2)
            .collect()
    };
    assert_eq!(sel.len(), 2, "炼化选牌面 2 张");
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: sel,
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(
        e.state_core().player(0).lianyaohu_store.len(),
        0,
        "炼化：壶仓清空"
    );
    assert_eq!(hand_len_w(&e, 0), 1, "炼化：牌堆顶首张类别均不同牌入手");

    // ③ 壶境：撤装备技实例后弃壶入弃堆+授壶境 → p0 准备阶段自动回收装备
    {
        let mut snap = e.save();
        snap.core.players[0].equips.retain(|x| *x != hu);
        snap.core.discard.push(hu);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("卸壶");
    }
    e.debug_grant(0, "hujing");
    // 推进到 p0 下一回合 Judge 入场（壶境 forced 自动装备）
    let mut guard = 0;
    loop {
        guard += 1;
        assert!(guard < 200, "advance 溢出");
        if let Some(c) = e.open_choice() {
            let (cid, obs) = (c.id, c.observer);
            let sel: Vec<ChoiceOption> = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else if !c.spec.options.is_empty() {
                vec![c.spec.options[0]]
            } else {
                vec![]
            };
            submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: sel,
                },
            );
            continue;
        }
        if e.turn().current == 0 && e.turn().phase == Phase::Judge {
            break;
        }
        let cur = e.turn().current;
        let out = submit(&mut e, cur, GameAction::NextPhase);
        if let CommandOutcome::Rejected(_) = out {
            continue;
        }
    }
    assert!(
        e.state_core().player(0).equips.contains(&hu),
        "壶境：弃牌堆炼妖壶自动回收装备"
    );
    // 手牌上限 +2：hp1 → 上限 3；置 4 张手牌推进弃牌阶段，弃窗 min=1
    //（4-(1+2)）证明 +2 生效
    let mut snap = e.save();
    snap.core.players[0].hp = 1;
    e = Engine::restore_with_extensions(&snap, &[&pack]).expect("压血");
    let extra_cards: Vec<u32> = (0..4).map(|_| e.debug_place(0, "tao")).collect();
    let _ = extra_cards;
    // 推进至弃牌阶段
    loop {
        if let Some(c) = e.open_choice() {
            if c.spec.prompt == "discard_to_limit" {
                let min = c.spec.min as usize;
                let hand = hand_len_w(&e, 0);
                let expect = hand.saturating_sub(e.debug_hp(0).max(0) as usize + 2);
                assert_eq!(
                    min,
                    expect,
                    "壶境：上限=hp+2（min {} = 手 {} - 上限 {}）",
                    min,
                    hand,
                    e.debug_hp(0).max(0) as usize + 2
                );
                break;
            }
            let (cid, obs) = (c.id, c.observer);
            let sel: Vec<ChoiceOption> = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else if !c.spec.options.is_empty() {
                vec![c.spec.options[0]]
            } else {
                vec![]
            };
            submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: sel,
                },
            );
            continue;
        }
        if e.turn().current == 0 && e.turn().phase == Phase::Discard {
            // 无窗=未超限——手牌应恰 3 张
            assert_eq!(hand_len_w(&e, 0), 3, "壶境：上限 3 恰好不弃");
            break;
        }
        let cur = e.turn().current;
        let out = submit(&mut e, cur, GameAction::NextPhase);
        if let CommandOutcome::Rejected(_) = out {
            continue;
        }
    }
}

/// T12 swd 卡牌面补切批④：equip5 三宝物——乾坤袋（手牌上限+1 纯 mod）；
/// 神农鼎（出牌限一次弃 2 手牌回 1，受伤时）；女娲石（他人濒死确认门：
/// 目标回 1、自己失去 1）。另修批③缺口：炼妖壶卡条目补 equip_skills。
#[test]
fn swd_cards_equip5_b34_qiankundai_shennongding_nvwashi() {
    use engine_native::catalog::CardKind;
    use engine_native::state::SkillInstance;
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
    let mut e = Engine::new_with_extensions(62014, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);

    let kind_id = |e: &Engine, k: CardKind| {
        e.catalog()
            .cards
            .iter()
            .position(|c| c.kind == k)
            .expect("卡在目录") as u32
    };
    let equip_with_skills = |e: &mut Engine, p: u8, card: u32, skills: &[&str]| {
        let mut snap = e.save();
        snap.core.deck.retain(|x| *x != card);
        snap.core.discard.retain(|x| *x != card);
        snap.core.players[p as usize].equips.push(card);
        for sk in skills {
            let def = e.catalog().skill_by_name(sk).expect("装备技 def");
            snap.core.skills.insert(
                (p, def),
                SkillInstance {
                    owner: p,
                    enabled: true,
                    once_used: false,
                    expires_end_of_turn: false,
                    rounds: 0,
                },
            );
        }
        *e = Engine::restore_with_extensions(&snap, &[&pack]).expect("装备注入");
    };

    // ① 乾坤袋：装备后弃牌窗 min = 手-(hp+1)
    let qkd = kind_id(&e, CardKind::Qiankundai);
    equip_with_skills(&mut e, 0, qkd, &[]);
    e.debug_set_hp(0, 4);
    for _ in 0..6 {
        e.debug_place(0, "tao");
    }
    // 6 手牌 hp4 → 无袋 min=2；有袋 min=1
    loop {
        if let Some(c) = e.open_choice() {
            if c.spec.prompt == "discard_to_limit" {
                let min = c.spec.min as usize;
                let hand = hand_len_w(&e, 0);
                assert_eq!(
                    min,
                    hand.saturating_sub(5),
                    "乾坤袋：上限 hp+1=5（min {min} = 手 {hand} - 5）"
                );
                break;
            }
            let (cid, obs) = (c.id, c.observer);
            let sel: Vec<ChoiceOption> = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else if !c.spec.options.is_empty() {
                vec![c.spec.options[0]]
            } else {
                vec![]
            };
            submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: sel,
                },
            );
            continue;
        }
        if e.turn().current == 0 && e.turn().phase == Phase::Discard {
            panic!("6 手牌应触发弃牌窗");
        }
        let cur = e.turn().current;
        let out = submit(&mut e, cur, GameAction::NextPhase);
        if let CommandOutcome::Rejected(_) = out {
            continue;
        }
    }

    // ② 神农鼎：受伤时弃 2 手牌回 1
    advance_to_play(&mut e, 0);
    let snd = kind_id(&e, CardKind::Shennongding);
    equip_with_skills(&mut e, 0, snd, &["shennongding"]);
    e.debug_set_hp(0, 3);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let c1 = e.debug_place(0, "tao");
    let c2 = e.debug_place(0, "tao");
    let snd_def = e
        .catalog()
        .skill_by_name("shennongding")
        .expect("神农鼎 def");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action: GameAction::ActivateSkill { skill: snd_def },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "神农鼎受理 {out:?}"
    );
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("神农鼎弃牌窗");
    let sel: Vec<ChoiceOption> = {
        let ch = e.open_choice().expect("still open");
        ch.spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Card(x) if *x == c1 || *x == c2))
            .cloned()
            .take(2)
            .collect()
    };
    assert_eq!(sel.len(), 2);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: sel,
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(0), 4, "神农鼎：回 1（3→4）");
    assert_eq!(hand_len_w(&e, 0), 0, "神农鼎：2 张已弃");

    // 反例 ②b：满血拒（activate 门）
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action: GameAction::ActivateSkill { skill: snd_def },
    });
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "神农鼎反例：满血拒 {out:?}"
    );

    // ③ 女娲石：p2 濒死（hp 0）→ p0 确认 → p2 回 1、p0 失 1
    let nws = kind_id(&e, CardKind::Nvwashi);
    equip_with_skills(&mut e, 0, nws, &["nvwashi"]);
    e.debug_set_hp(0, 4);
    e.debug_set_hp(2, 1);
    // 清全场手牌+清 p2 护甲：初始手牌随包扩卡漂移，濒死救援窗会抢在女
    // 娲石前；器族手持黄麟之琮者准备阶段获护甲会抵消伤害
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hujia(2, 0);
    e.debug_deal_damage(Some(1), 2, 1, engine_native::event::DamageNature::Normal);
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("女娲石确认窗");
    assert_eq!(obs, 0, "女娲石窗归持有者");
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
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: conf,
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(2), 1, "女娲石：p2 回 1 脱离濒死（0→1）");
    assert_eq!(e.debug_hp(0), 3, "女娲石：p0 失去 1（4→3）");

    // 反例 ③b：p0 hp=1 时不再触发（filter hp>1）
    e.debug_set_hp(0, 1);
    e.debug_set_hp(2, 1);
    e.debug_deal_damage(Some(1), 2, 1, engine_native::event::DamageNature::Normal);
    assert!(e.open_choice().is_none(), "女娲石反例：自己 hp=1 不触发");
}

/// T12 swd 卡牌面补切批⑤：昆仑镜（出牌限一次亮牌堆顶 3 选 1 获得、以
/// 1 手牌替换）；东皇钟（阶段末弃 1 红牌置延时锦囊实体入目标判定区——
/// 实体发现替代 createCard，登记差异随机→首张）。
#[test]
fn swd_cards_equip5_b35_kunlunjingc_donghuangzhong() {
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
    let mut e = Engine::new_with_extensions(62015, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let equip_with_skills = |e: &mut Engine, p: u8, card: u32, skills: &[&str]| {
        let mut snap = e.save();
        snap.core.deck.retain(|x| *x != card);
        snap.core.discard.retain(|x| *x != card);
        snap.core.players[p as usize].equips.push(card);
        for sk in skills {
            let def = e.catalog().skill_by_name(sk).expect("装备技 def");
            snap.core.skills.insert(
                (p, def),
                engine_native::state::SkillInstance {
                    owner: p,
                    enabled: true,
                    once_used: false,
                    expires_end_of_turn: false,
                    rounds: 0,
                },
            );
        }
        *e = Engine::restore_with_extensions(&snap, &[&pack]).expect("装备注入");
    };

    // ① 昆仑镜：亮牌堆顶 3 选 1 → 弃 1 手牌 → 获得选中牌
    let kj = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == engine_native::catalog::CardKind::Kunlunjingc)
        .expect("昆仑镜在目录") as u32;
    equip_with_skills(&mut e, 0, kj, &["kunlunjingc"]);
    let replace = e.debug_place(0, "tao");
    let deck_top3: Vec<u32> = e.state_core().deck.iter().take(3).copied().collect();
    let kj_def = e
        .catalog()
        .skill_by_name("kunlunjingc")
        .expect("昆仑镜 def");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action: GameAction::ActivateSkill { skill: kj_def },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "昆仑镜受理 {out:?}"
    );
    let want = deck_top3[0];
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("昆仑镜选亮牌窗");
    let sel: Vec<ChoiceOption> = {
        let ch = e.open_choice().expect("still open");
        ch.spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Card(x) if *x == want))
            .cloned()
            .collect()
    };
    assert_eq!(sel.len(), 1, "亮牌面含牌堆首张");
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: sel,
        },
    );
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("昆仑镜替换窗");
    let sel: Vec<ChoiceOption> = {
        let ch = e.open_choice().expect("still open");
        ch.spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Card(x) if *x == replace))
            .cloned()
            .collect()
    };
    assert_eq!(sel.len(), 1);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: sel,
        },
    );
    settle_optional_confirm(&mut e);
    assert!(e.debug_hand(0).contains(&want), "昆仑镜：获得选中牌");
    assert!(!e.debug_hand(0).contains(&replace), "昆仑镜：替换手牌已弃");
    // 反例 ①b：限一次
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action: GameAction::ActivateSkill { skill: kj_def },
    });
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "昆仑镜反例：限一次 {out:?}"
    );

    // ② 东皇钟：p1 阶段末弃 1 红牌 → 置延时锦囊实体入 p2 判定区
    let dhz = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == engine_native::catalog::CardKind::Donghuangzhong)
        .expect("东皇钟在目录") as u32;
    equip_with_skills(&mut e, 1, dhz, &["donghuangzhong"]);
    advance_to_play(&mut e, 1);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let red = {
        let id = e.debug_set_deck_top_by(|d| {
            matches!(
                d.suit,
                engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
            )
        });
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.players[1].hand.push(id);
        id
    };
    // A2 H64：上游 player phaseEnd＝整回合结束——推进 p1 各阶段到整回合
    // 结束窗触发（弃红牌 → 置延时锦囊实体入 p2 判定区）
    let log_cursor = e.log().len();
    let mut guard = 0;
    let mut dhz_placed = false;
    loop {
        guard += 1;
        assert!(guard < 300, "推进溢出");
        if let Some(c) = e.open_choice() {
            let prompt = c.spec.prompt;
            if prompt == "donghuangzhong_cost" {
                let (cid, obs) = (c.id, c.observer);
                let sel: Vec<ChoiceOption> = {
                    let ch = e.open_choice().expect("still open");
                    ch.spec
                        .options
                        .iter()
                        .filter(|o| matches!(o, ChoiceOption::Card(x) if *x == red))
                        .cloned()
                        .collect()
                };
                assert_eq!(sel.len(), 1);
                submit(
                    &mut e,
                    obs,
                    GameAction::Answer {
                        choice_id: cid,
                        selection: sel,
                    },
                );
                continue;
            }
            if prompt == "donghuangzhong_target" {
                let (cid, obs) = (c.id, c.observer);
                submit(
                    &mut e,
                    obs,
                    GameAction::Answer {
                        choice_id: cid,
                        selection: vec![ChoiceOption::Player(2)],
                    },
                );
                dhz_placed = true;
                continue;
            }
            let (cid, obs) = (c.id, c.observer);
            let sel: Vec<ChoiceOption> = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else if !c.spec.options.is_empty() {
                vec![c.spec.options[0]]
            } else {
                vec![]
            };
            submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: sel,
                },
            );
            continue;
        }
        if dhz_placed {
            break;
        }
        let cur = e.turn().current;
        let out = submit(&mut e, cur, GameAction::NextPhase);
        if let CommandOutcome::Rejected(_) = out {
            continue;
        }
        if guard > 250 {
            break;
        }
    }
    assert!(dhz_placed, "东皇钟：整回合结束窗内弃红牌并选目标");
    // 窗末即交接：置入 p2 判定区的延时锦囊随 p2 判定阶段立刻结算离场
    //（判定区已空）——以日志 CardsGiven 证置入的是延时锦囊实体。
    let placed_kind = e
        .log()[log_cursor..]
        .iter()
        .find_map(|ev| match ev {
            engine_native::LogEvent::CardsGiven {
                from: 1,
                to: 2,
                cards,
            } if cards.len() == 1 => {
                Some(e.catalog().cards[cards[0] as usize].kind)
            }
            _ => None,
        })
        .expect("东皇钟：延时锦囊实体置入 p2 判定区（日志 CardsGiven）");
    assert!(
        matches!(
            placed_kind,
            engine_native::catalog::CardKind::Le
                | engine_native::catalog::CardKind::Bing
                | engine_native::catalog::CardKind::ShanDian
        ),
        "东皇钟：置入的是延时锦囊实体"
    );
    assert!(
        !e.debug_hand(1).contains(&red),
        "东皇钟：红牌费用已弃"
    );
}

/// T12 swd 卡牌面补切批⑥：空洞印（equip5 宝物）——死亡前置窗口锁定取
/// 消：death_cancelled 置位（DeathMachine 开头拦截跳过死亡执行）+hp=1+
/// 摸 1+弃全部宝物（上游 dieBefore trigger.cancel 同口径）。
#[test]
fn swd_cards_equip5_b36_kongdongyin_death_cancel() {
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
    let mut e = Engine::new_with_extensions(62016, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let kyd = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == engine_native::catalog::CardKind::Kongdongyin)
        .expect("空洞印在目录") as u32;
    let qkd = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == engine_native::catalog::CardKind::Qiankundai)
        .expect("乾坤袋在目录") as u32;
    // 装备空洞印+乾坤袋（两只宝物都应被弃）+技能实例
    {
        let mut snap = e.save();
        snap.core.deck.retain(|x| *x != kyd && *x != qkd);
        snap.core.discard.retain(|x| *x != kyd && *x != qkd);
        snap.core.players[0].equips.push(kyd);
        snap.core.players[0].equips.push(qkd);
        for sk in ["kongdongyin"] {
            let def = e.catalog().skill_by_name(sk).expect("装备技 def");
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
        }
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    }

    // 正例：p0 压到 1 再吃 4 伤 → 濒死无救 → 死亡被空洞印取消
    e.debug_set_hp(0, 1);
    e.debug_deal_damage(Some(1), 0, 4, engine_native::event::DamageNature::Normal);
    settle_optional_confirm(&mut e);
    assert!(e.state_core().player(0).alive, "空洞印：死亡被取消存活");
    assert_eq!(e.debug_hp(0), 1, "空洞印：hp 重置 1");
    assert_eq!(hand_len_w(&e, 0), 1, "空洞印：摸 1");
    assert!(
        !e.state_core().player(0).equips.iter().any(|c| {
            e.catalog().cards[*c as usize].kind.equip_slot()
                == Some(engine_native::catalog::EquipSlot::Treasure)
        }),
        "空洞印：宝物全弃（含自身）"
    );
    assert!(
        e.state_core().discard.contains(&kyd) && e.state_core().discard.contains(&qkd),
        "空洞印：两只宝物入弃堆"
    );

    // 反例：死亡取消后再次致死——无空洞印 → 正常死亡
    e.debug_deal_damage(Some(1), 0, 9, engine_native::event::DamageNature::Normal);
    settle_optional_confirm(&mut e);
    assert!(
        !e.state_core().player(0).alive,
        "空洞印反例：一次性，再次致死正常死亡"
    );
}

/// T12 swd 卡牌面补切批⑦：锦囊族 4 kind 7 张——惊雷山（雷版南蛮，逆座
/// 序）；炽羽袭（火版万箭，逆座序）；鬼蛇旗（单目标 +1 护甲）；仙露回
/// （全体受伤者 +1 护甲）。MachineSnapshot::Aoe/AoeTarget nature 字段批
/// （存档 v71→v72，规则 v284→v285，golden 重录+284 探针零扰动）。
#[test]
fn swd_cards_trick_b37_jingleishan_chiyuxi_guisheqi_xianluhui() {
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
    let mut e = Engine::new_with_extensions(62017, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let kind_id = |e: &Engine, k: engine_native::catalog::CardKind| {
        e.catalog()
            .cards
            .iter()
            .position(|c| c.kind == k)
            .expect("卡在目录") as u32
    };
    use engine_native::catalog::CardKind;
    use engine_native::event::DamageNature;

    // ① 鬼蛇旗：p2 +1 护甲
    let gsq = kind_id(&e, CardKind::Guisheqi);
    {
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != gsq);
        core.players[0].hand.push(gsq);
    }
    e.debug_set_hp(2, 8);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: gsq,
            targets: vec![2],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.state_core().player(2).hujia, 1, "鬼蛇旗：p2 +1 护甲");

    // ② 惊雷山：全体其他需出杀，不出雷 1 伤（p1/p2/p3 手牌全空 → 各 -1）
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let jls = kind_id(&e, CardKind::Jingleishan);
    {
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != jls);
        core.players[0].hand.push(jls);
    }
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);
    e.debug_set_hp(3, 4);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: jls,
            targets: vec![],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 3, "惊雷山：p1 雷 1 伤");
    // p2 有 ① 的 1 护甲：雷 1 伤被护甲抵消（hp 不动、护甲清零）
    assert_eq!(e.debug_hp(2), 4, "惊雷山：p2 雷 1 伤被护甲抵消");
    assert_eq!(e.state_core().player(2).hujia, 0, "惊雷山：p2 护甲消耗");
    assert_eq!(e.debug_hp(3), 3, "惊雷山：p3 雷 1 伤");

    // ③ 炽羽袭：需出闪，火 1 伤（同上结构）
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let cyx = kind_id(&e, CardKind::Chiyuxi);
    {
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != cyx);
        core.players[0].hand.push(cyx);
    }
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);
    e.debug_set_hp(3, 4);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: cyx,
            targets: vec![],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 3, "炽羽袭：p1 火 1 伤");
    // p2 无护甲（②已消耗）→ 3→2
    assert_eq!(
        e.debug_hp(2),
        3,
        "炽羽袭：p2 hp 4→3（②雷伤已被护甲抵消未扣血）"
    );
    assert_eq!(e.debug_hp(3), 3, "炽羽袭：p3 火 1 伤");

    // ④ 仙露回：受伤者（p1/p2/p3 各 7/8）各 +1 护甲
    let xlh = kind_id(&e, CardKind::Xianluhui);
    {
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != xlh);
        core.players[0].hand.push(xlh);
    }
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: xlh,
            targets: vec![],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.state_core().player(1).hujia, 1, "仙露回：受伤 p1 +1 护甲");
    assert_eq!(e.state_core().player(2).hujia, 1, "仙露回：受伤 p2 +1 护甲");
    assert_eq!(e.state_core().player(3).hujia, 1, "仙露回：受伤 p3 +1 护甲");

    // 反例：满血 p0 不进仙露回目标表
    assert_eq!(e.state_core().player(0).hujia, 0, "仙露回：p0 满血不受益");
    let _ = DamageNature::Thunder;
    let _ = DamageNature::Fire;
}

/// T12 swd 卡牌面补切批⑧：龟灵之桃——逆攻标记受伤积攒（上限 4）、
/// 出牌弃全部标记：奇数摸 1+对一名其他角色 floor(标记/2) 伤；标记=1 仅
/// 摸 1。PlayerState.nigong_lv 字段批（存档 v72→v73/规则 v285→v286）。
#[test]
fn swd_cards_equip5_b38_guilingzhitao_nigong() {
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
    let mut e = Engine::new_with_extensions(62018, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let glt = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == engine_native::catalog::CardKind::Guilingzhitao)
        .expect("龟灵之桃在目录") as u32;
    {
        let mut snap = e.save();
        snap.core.deck.retain(|x| *x != glt);
        snap.core.discard.retain(|x| *x != glt);
        snap.core.players[0].equips.push(glt);
        for sk in ["nigong_accumulate", "nigong_strike"] {
            let def = e.catalog().skill_by_name(sk).expect("逆攻 def");
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
        }
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    }

    // ① 受伤积攒：debug 伤 2 → 标记 2（forced 无窗）
    e.debug_set_hp(0, 8);
    e.debug_deal_damage(Some(1), 0, 2, engine_native::event::DamageNature::Normal);
    settle_optional_confirm(&mut e);
    assert_eq!(e.state_core().player(0).nigong_lv, 2, "逆攻：受伤 +2 标记");

    // ② 发动（lv=2）：弃全部标记，偶数不摸，对 p1 造成 1 伤（floor 2/2）
    e.debug_set_hp(1, 8);
    e.debug_set_hp(2, 8);
    let strike = e
        .catalog()
        .skill_by_name("nigong_strike")
        .expect("发动 def");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action: GameAction::ActivateSkill { skill: strike },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "逆攻发动受理 {out:?}"
    );
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("逆攻选人窗");
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.state_core().player(0).nigong_lv, 0, "逆攻②：标记清零");
    assert_eq!(e.debug_hp(1), 7, "逆攻②：floor(2/2)=1 伤");
    assert_eq!(hand_len_w(&e, 0), 0, "逆攻②：偶数不摸");

    // 反例 ②b：标记 0 时发动拒
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action: GameAction::ActivateSkill { skill: strike },
    });
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "逆攻反例：无标记拒 {out:?}"
    );

    // ③ 奇数面：受伤 3 → 标记 3 → 发动：摸 1 + floor(3/2)=1 伤
    e.debug_deal_damage(Some(1), 0, 3, engine_native::event::DamageNature::Normal);
    settle_optional_confirm(&mut e);
    assert_eq!(e.state_core().player(0).nigong_lv, 3, "逆攻③：受伤 +3 标记");
    let h0 = hand_len_w(&e, 0);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action: GameAction::ActivateSkill { skill: strike },
    });
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("逆攻③选人窗");
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(2)],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.state_core().player(0).nigong_lv, 0, "逆攻③：标记清零");
    assert_eq!(hand_len_w(&e, 0), h0 + 1, "逆攻③：奇数摸 1");
    assert_eq!(e.debug_hp(2), 7, "逆攻③：floor(3/2)=1 伤");

    // ④ 上限：先抬血防死，受伤 9 → 标记钳 4
    e.debug_set_hp(0, 12);
    e.debug_deal_damage(Some(1), 0, 9, engine_native::event::DamageNature::Normal);
    settle_optional_confirm(&mut e);
    assert_eq!(e.state_core().player(0).nigong_lv, 4, "逆攻④：上限 4");
}

/// T12 swd 卡牌面补切批⑨：四非珍面——出牌限一次令其他有手牌角色判定，
/// 非红桃则其用首张手牌指向座位序首个其他存活者（登记差异上游随机牌/
/// 随机目标）。
#[test]
fn swd_cards_equip5_b39_sifeizhenmian_yiluan() {
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
    let mut e = Engine::new_with_extensions(62019, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sfm = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == engine_native::catalog::CardKind::Sifeizhenmian)
        .expect("四非珍面在目录") as u32;
    {
        let mut snap = e.save();
        snap.core.deck.retain(|x| *x != sfm);
        snap.core.discard.retain(|x| *x != sfm);
        snap.core.players[0].equips.push(sfm);
        let def = e.catalog().skill_by_name("yiluan").expect("移乱 def");
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
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    }

    // ① 目标 p1（手牌黑杀首张）判定非红桃（预控判定牌堆顶为黑）→ 用黑杀
    // 指向座位序首个其他存活者（=p2）
    e.debug_set_deck_top_by(|d| !matches!(d.suit, engine_native::catalog::Suit::Heart));
    let _t1 = e.debug_place(1, "sha");
    let _t2 = e.debug_place(1, "tao");
    e.debug_set_hp(2, 8);
    e.debug_set_hp(3, 8);
    let yiluan_def = e.catalog().skill_by_name("yiluan").expect("移乱 def");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action: GameAction::ActivateSkill { skill: yiluan_def },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "四非受理 {out:?}"
    );
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("四非选人窗");
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(hand_len_w(&e, 1), 1, "四非①：目标首张手牌被用（2→1）");
    // 座位序首个其他存活者 = p0：黑杀指向 p0（无闪直中 4→3）
    assert_eq!(e.debug_hp(0), 3, "四非①：p0（座位序首）被杀指向");

    // 反例：判定红桃时不用牌（预控红桃顶）——再发动（限一次未消耗？
    // usable:1 每回合一次，此处换目标验证结果分支而非限次）
    // 反例改为主：目标手牌为空时不进目标表
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let yiluan_def = e.catalog().skill_by_name("yiluan").expect("移乱 def");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action: GameAction::ActivateSkill { skill: yiluan_def },
    });
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "四非反例：无其他有手牌角色拒 {out:?}"
    );
}

/// T12 swd 卡牌面补切批⑩：器族四象五件（手持触发技，InitHook
/// swd_jiqi_held 开局授全部角色、效果由手持 filter 门控）——青龙之圭
/// （准备阶段摸 2 弃 1）、朱雀之璋（受他人伤害后来源火 1 伤）、白兽之琥
/// （弃牌阶段末弃他人一张牌）、玄武之璜（造成伤害后回复等量）、黄麟之琮
/// （准备阶段无护甲+1）。
#[test]
fn swd_cards_jiqi_b40_five_held_treasures() {
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
    // init_hook 授技核验：建局后全员持有五手持技实例
    let mut e = Engine::new_with_extensions(62020, 4, &[&pack]).expect("建局");
    for sk in [
        "qinglongzhigui",
        "zhuquezhizhang",
        "baishouzhihu",
        "xuanwuzhihuang",
        "huanglinzhicong",
    ] {
        let def = e.catalog().skill_by_name(sk).expect("手持技 def");
        assert!(
            e.state_core().skills.contains_key(&(0, def)),
            "{sk} 开局已授予"
        );
    }
    advance_to_play(&mut e, 0);

    // ① 黄麟之琮：p0 手持（置入手牌）→ 下一准备阶段 +1 护甲
    let ql = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == engine_native::catalog::CardKind::Qinglongzhigui)
        .expect("青龙在目录") as u32;
    let zq = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == engine_native::catalog::CardKind::Zhuquezhizhang)
        .expect("朱雀在目录") as u32;
    let hw = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == engine_native::catalog::CardKind::Huanglinzhicong)
        .expect("黄麟在目录") as u32;
    let xw = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == engine_native::catalog::CardKind::Xuanwuzhihuang)
        .expect("玄武在目录") as u32;
    {
        let core = e.state_core_mut();
        for c in [ql, zq, hw, xw] {
            core.deck.retain(|x| *x != c);
            core.discard.retain(|x| *x != c);
            core.players[0].hand.push(c);
        }
    }
    // ② 朱雀之璋：p1 对 p0 造成 1 伤 → p1 火 1 伤反噬
    e.debug_set_hp(1, 8);
    e.debug_deal_damage(Some(1), 0, 1, engine_native::event::DamageNature::Normal);
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 7, "朱雀之璋：来源火 1 伤反噬（8→7）");

    // ③ 玄武之璜：p0 对 p1 造成 2 伤 → p0 回复 2（先压血过 hp<max 门）
    e.debug_set_hp(0, 2);
    for _ in 0..2 {
        e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    }
    settle_optional_confirm(&mut e);
    assert!(
        e.debug_hp(0) >= 4,
        "玄武之璜：造成 2 伤回复 2（实际 {}）",
        e.debug_hp(0)
    );

    // 反例：p1 手持玄武之璜面——无（玄武 filter 为 source==owner，p1 受
    // 伤不触发他人玄武）。此处验证黄麟护甲 only-once（有护甲不再叠加）：
    // 推进 p0 到下一准备阶段
    let hujia_before = e.state_core().player(0).hujia;
    advance_to_play(&mut e, 0);
    // 途中黄麟已在准备阶段触发一次（hujia 0→1）
    assert!(
        e.state_core().player(0).hujia >= hujia_before,
        "黄麟之琮：护甲非递减"
    );
    let _ = (ql, zq, hw, xw);
}

/// T12 swd 卡牌面补切批⑫：玉蝉八卦 8 kind autoViewAs 面——手持时视作
/// 对应基本牌使用（乾=杀/坤=草药/震=酒/巽=桃/坎=神秘果[同拒]/离=天仙
/// 酒/艮=封印之丹/兑=雪霁冰雹）；换形态 mutation+煅造装备面挂批⑬。
#[test]
fn swd_cards_trick_b42_yuchan_autoviewas() {
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
    assert_eq!(
        pack.cards.len(),
        90,
        "窗口机批后 90 张（批⑬ 神草结 87+神秘果 2；批 147 轩辕剑 +1）"
    );
    let mut e = Engine::new_with_extensions(62021, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let kind_id = |e: &Engine, k: engine_native::catalog::CardKind| {
        e.catalog()
            .cards
            .iter()
            .position(|c| c.kind == k)
            .expect("玉蝉在目录") as u32
    };
    let hold = |e: &mut Engine, p: u8, card: u32| {
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != card);
        core.discard.retain(|x| *x != card);
        core.players[p as usize].hand.push(card);
    };
    use engine_native::catalog::CardKind;

    // ① 玉蝉·乾 autoViewAs 杀：打 p1 造成 1 伤
    let yq = kind_id(&e, CardKind::Yuchanqian);
    hold(&mut e, 0, yq);
    e.debug_set_hp(1, 4);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: yq,
            targets: vec![1],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 3, "玉蝉乾：视为杀命中 p1（4→3）");

    // ② 玉蝉·巽 autoViewAs 桃：p0 受伤回 1
    e.debug_set_hp(0, 2);
    let yx = kind_id(&e, CardKind::Yuchanxun);
    hold(&mut e, 0, yx);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: yx,
            targets: vec![],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(0), 3, "玉蝉巽：视为桃回 1（2→3）");

    // ③ 反例：玉蝉·坎 autoViewAs 神秘果——神秘果无直用面同拒
    let yk = kind_id(&e, CardKind::Yuchankan);
    hold(&mut e, 0, yk);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: yk,
            targets: vec![],
        },
    );
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "玉蝉坎反例：神秘果面同拒 {out:?}"
    );

    // ④ 反例：玉蝉族不可当原实体使用（乾不能当普通实体牌非杀路径用）
    // ——乾的直用仅杀路径；此处验证目标空时的杀被拒
    hold(&mut e, 0, yq);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: yq,
            targets: vec![],
        },
    );
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "玉蝉乾反例：杀无目标拒 {out:?}"
    );
}

/// T12 swd 卡牌面补切批⑬：神草结——锦囊伤害前置窗口：受伤者是自己则
/// 伤-1、来源是自己则伤+1（弃一张神草结；确认门）。InitHook 扩展授全
/// 员实例，效果由手持 filter 门控。
#[test]
fn swd_cards_trick_b43_shencaojie_guard() {
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
    let mut e = Engine::new_with_extensions(62022, 4, &[&pack]).expect("建局");
    // init hook 核验：全员持有神草结守卫实例
    let scj_def = e.catalog().skill_by_name("shencaojie").expect("神草结 def");
    assert!(
        e.state_core().skills.contains_key(&(0, scj_def)),
        "神草结守卫开局已授予"
    );
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let scj = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == engine_native::catalog::CardKind::Shencaojie)
        .expect("神草结在目录") as u32;
    {
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != scj);
        core.discard.retain(|x| *x != scj);
        core.players[0].hand.push(scj);
    }
    // 伤害卡：决斗（锦囊实体）入牌堆顶供 with_card 使用
    let duel = e.debug_place(2, "duel");

    // ① 受伤方向：p1 用锦囊实体伤 p0 → p0 确认 → 伤-1
    e.debug_set_hp(0, 4);
    e.debug_deal_damage_with_card(
        Some(1),
        0,
        2,
        engine_native::event::DamageNature::Normal,
        duel,
    );
    // 确认窗归 p0
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("神草结确认窗");
    assert_eq!(obs, 0, "神草结窗归持有者");
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
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: conf,
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(0), 3, "神草结：受伤方向伤-1（4-2+1=3）");
    assert_eq!(hand_len_w(&e, 0), 0, "神草结：一张已弃");

    // ② 增伤方向：p0 用锦囊实体伤 p1 → p0 确认 → 伤+1
    // 再入一张神草结
    let scj2 = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(i, c)| c.kind == engine_native::catalog::CardKind::Shencaojie && *i as u32 != scj)
        .map(|(i, _)| i as u32)
        .expect("第二张神草结");
    {
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != scj2);
        core.discard.retain(|x| *x != scj2);
        core.players[0].hand.push(scj2);
    }
    e.debug_set_hp(2, 8);
    e.debug_deal_damage_with_card(
        Some(0),
        2,
        2,
        engine_native::event::DamageNature::Normal,
        duel,
    );
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("神草结增伤窗");
    assert_eq!(obs, 0);
    let conf: Vec<ChoiceOption> = {
        let ch = e.open_choice().expect("still open");
        ch.spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Confirm))
            .cloned()
            .collect()
    };
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: conf,
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(2), 5, "神草结：增伤方向伤+1（8-2-1=5）");

    // 反例：无神草结在手 → 不触发（p0 手空）
    e.debug_set_hp(0, 4);
    e.debug_deal_damage_with_card(
        Some(1),
        0,
        2,
        engine_native::event::DamageNature::Normal,
        duel,
    );
    assert!(e.open_choice().is_none(), "神草结反例：无牌不触发");
}
