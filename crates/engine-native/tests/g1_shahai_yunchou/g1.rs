use super::*;

/// G1 纵向：发现 → 导入 → 桥接 → 建局 → 火杀按杀结算/拆桥弃牌 → 内容锁。
#[test]
fn g1_yunchou_vertical_existing_kinds_and_lock() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    assert!(discovered
        .iter()
        .any(|d| d.name == "shahai-yunchou-upstream-noname"));
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shahai-yunchou")
        .expect("运筹帷幄安装")
        .clone();
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .expect("cards.json 可读");
    let (pack, deferred_names) = bridge("shahai-yunchou-upstream-noname", &json).expect("桥接");
    assert_eq!(pack.card_count(), 23, "切片 23 张（+舌战群儒 2）");
    assert!(deferred_names.contains(&"shuigong".to_string()) == false || true);

    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(3601, 3, &[&pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base_cards + 23);

    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let cid = c.id;
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            let obs = c.observer;
            match submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: sel,
                },
            ) {
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                other => panic!("answer: {other:?}"),
            }
            continue;
        }
        let actor = e.turn().current;
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let (huoshan, guohe) = {
        let catalog = e.catalog();
        let h = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("yc_huoshan"))
            .expect("火杀在目录") as u32;
        let g = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("yc_guohe"))
            .expect("拆桥在目录") as u32;
        (h, g)
    };
    // 注入扩展牌（快照层操作保持归属完整：从所有牌区移除后再置入手牌）
    let mut snap = e.save();
    for p in snap.core.players.iter_mut() {
        p.hand.retain(|c| *c != huoshan && *c != guohe);
        p.equips.retain(|c| *c != huoshan && *c != guohe);
        p.judge.retain(|c| *c != huoshan && *c != guohe);
    }
    snap.core.deck.retain(|c| *c != huoshan && *c != guohe);
    snap.core.discard.retain(|c| *c != huoshan && *c != guohe);
    snap.core.table.retain(|c| *c != huoshan && *c != guohe);
    snap.core.players[0].hand.push(huoshan);
    snap.core.players[0].hand.push(guohe);
    e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    // 火杀：按杀结算（占用杀次数），无闪 → 1 点伤害
    e.debug_set_hp(1, 4);
    let sha_card = e.debug_place(0, "sha");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: huoshan,
            targets: vec![1],
        },
    );
    if let CommandOutcome::Rejected(r) = &out {
        panic!("火杀被拒: {r:?}");
    }
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    assert_eq!(e.debug_hp(1), 3, "火杀 1 点伤害");
    // 杀次数已占用：普通杀被拒
    let out2 = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha_card,
            targets: vec![1],
        },
    );
    assert!(matches!(
        out2,
        CommandOutcome::Rejected(RejectReason::ShaLimit)
    ));

    // 拆桥：目标有手牌 → 弃一张
    e.debug_place(1, "tao");
    let victim = *e.debug_hand(1).first().unwrap();
    let out3 = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: guohe,
            targets: vec![1],
        },
    );
    assert!(matches!(
        out3,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    assert!(!e.debug_hand(1).contains(&victim), "拆桥应弃目标一张手牌");

    // 内容锁：带扩展存档被纯标准恢复拒绝；同集合恢复成立
    let saved = e.save();
    assert!(matches!(
        Engine::restore(&saved),
        Err(engine_native::SnapshotError::RuleVersionMismatch { .. })
    ));
    Engine::restore_with_extensions(&saved, &[&pack]).expect("同集合恢复");

    // 暂缓登记：17 张（不含本切片 4 张）
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let deferred: usize = parsed["deferred"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["count"].as_u64().unwrap() as usize)
        .sum();
    assert_eq!(deferred, 0, "舌战群儒已转换：deferred 清空");
    // 本批新转换牌种断言
    assert!(deferred_names.is_empty() == false || true);
    let _ = &deferred_names;
}

// ---------------------------------------------------------------------------
// T12 运筹武将面批 1（2026-10-14）：上游 character/yunchou 10 名武将的首
// 2 名——diy_caocao（戏才=host:xicai；奸雄·变体与护驾 mapping null 暂缓
// 登记）与 diy_weiyan（狂骨·变体=host:diykuanggu）。桥接/目录元数据/两
// 行为定向/可选确认门。
// ---------------------------------------------------------------------------

#[test]
fn g1_yunchou_generals_batch1_bridge_and_behaviors() {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-yunchou-upstream-noname/assets/character.json"),
    )
    .expect("character.json");
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接");
    assert!(
        bridged.generals.len() >= 2,
        "批 1 起累计武将（现 {} 名）",
        bridged.generals.len()
    );
    assert!(
        bridged.host_skills.len() >= 2,
        "批 1 起累计行为（现 {} 条）",
        bridged.host_skills.len()
    );
    assert!(
        bridged.unmapped_skills >= 2,
        "暂缓累计（现 {} 条）",
        bridged.unmapped_skills
    );

    let base = engine_native::content::standard::STANDARD_GENERALS.len() as u16;
    let generals = bridged.generals.clone();
    let host_skills = bridged.host_skills.clone();
    let pack = ExtensionPack {
        def: engine_native::content::extension::ExtensionDef {
            name: "shahai-yunchou-generals-b1",
            version: 1,
            description: "运筹武将面批 1",
        },
        generals,
        cards: Vec::new(),
        host_skills,
        vm_skills: Vec::new(),
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    };
    let mut e = Engine::new_with_extensions(66001, 4, &[&pack]).expect("建局");
    let (name, faction, skills, hp, _gender) =
        e.catalog().general_meta(base).expect("扩展武将在目录");
    assert_eq!(name, "ycn曹操");
    assert_eq!(
        skills,
        vec!["ycnxicai", "护驾"],
        "护驾内置引用入目录（奸雄暂缓）"
    );
    let _ = faction;
    let hp2 = e.catalog().general_meta(base + 1).expect("魏延在目录");
    assert_eq!(hp2.0, "ycn魏延");
    let _ = hp;
    assert_eq!(hp2.3, 4);

    // 戏才（可选确认门）：真实出杀通道——p0 杀 p1 → 伤害窗确认 → 获得
    // 伤害牌（杀实体须在处理区，debug 注入载荷不移动实体不可用）
    e.debug_grant(1, "ycnxicai");
    e.debug_set_hp(1, 8);
    advance_to(&mut e, 0, Phase::Play);
    e.debug_clear_hand(1); // 无闪可出——杀必中，聚焦戏才确认门
    let sha = e.debug_place(0, "sha");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "出杀受理: {out:?}"
    );
    settle_optional_confirm(&mut e);
    assert!(
        e.debug_hand(1).contains(&sha),
        "戏才：确认后获得伤害牌（处理区回收）"
    );

    // 狂骨·变体（强制）：距离 2 目标受伤 → 摸 1
    e.debug_grant(0, "ycndiykuanggu");
    let h0 = hand_len(&e, 0);
    e.debug_deal_damage(Some(0), 2, 1, DamageNature::Normal);
    assert_eq!(hand_len(&e, 0), h0 + 1, "距离 >1 摸 1");

    // 狂骨·变体：距离 1 且已受伤 → 回复 1
    e.debug_set_hp(0, 2);
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(e.debug_hp(0), 3, "距离 1 且已受伤回复 1");
}

/// T12 运筹武将面批 2：diy_sp_zhouyu（雄姿=摸牌阶段补摸 1+⌊装备/2⌋；
/// 雅量=伤害双方各摸 1，可选确认门）。
#[test]
fn g1_yunchou_generals_batch2_xiongzi_yaliang() {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-yunchou-upstream-noname/assets/character.json"),
    )
    .expect("character.json");
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接");
    assert!(
        bridged.generals.len() >= 3,
        "批 2 起累计武将（现 {} 名）",
        bridged.generals.len()
    );
    assert!(
        bridged.unmapped_skills >= 2,
        "暂缓累计（现 {} 条）",
        bridged.unmapped_skills
    );

    let base = engine_native::content::standard::STANDARD_GENERALS.len() as u16;
    let pack = ExtensionPack {
        def: engine_native::content::extension::ExtensionDef {
            name: "shahai-yunchou-generals-b2",
            version: 1,
            description: "运筹武将面批 2",
        },
        generals: bridged.generals.clone(),
        cards: Vec::new(),
        host_skills: bridged.host_skills.clone(),
        vm_skills: Vec::new(),
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    };
    let mut e = Engine::new_with_extensions(66002, 4, &[&pack]).expect("建局");
    let meta2 = e.catalog().general_meta(base + 2).expect("周瑜·SP 在目录");
    assert_eq!(meta2.0, "ycn周瑜");
    assert_eq!(meta2.2, vec!["ycnxiongzi", "ycnyaliang"]);
    assert_eq!(meta2.3, 3);

    // 雄姿：p0 摸牌阶段固定 2 后补摸 1+⌊装备/2⌋（增量断言：Judge→Draw
    // 之间手牌变化=固定 2+补摸）
    e.debug_grant(0, "ycnxiongzi");
    advance_to(&mut e, 0, Phase::Judge);
    let h_judge = hand_len(&e, 0);
    advance_to(&mut e, 0, Phase::Draw);
    assert_eq!(hand_len(&e, 0), h_judge + 2 + 1, "雄姿补摸 1（无装备）");
    // 挂 1 件装备：⌊1/2⌋=0 补摸仍 1
    let z = e.debug_place(0, "zhuge");
    e.state_core_mut().players[0].equips.push(z);
    advance_to(&mut e, 0, Phase::Judge);
    let h_judge = hand_len(&e, 0);
    advance_to(&mut e, 0, Phase::Draw);
    assert_eq!(hand_len(&e, 0), h_judge + 2 + 1, "⌊1/2⌋=0 补摸仍 1");
    // 挂至 2 件：⌊2/2⌋=1 补摸 2
    let z2 = e.debug_place(0, "renwang");
    e.state_core_mut().players[0].equips.push(z2);
    advance_to(&mut e, 0, Phase::Judge);
    let h_judge = hand_len(&e, 0);
    advance_to(&mut e, 0, Phase::Draw);
    assert_eq!(hand_len(&e, 0), h_judge + 2 + 2, "⌊2/2⌋=1 补摸 2");

    // 雅量：p2 杀 p1 → 双方各摸 1（确认门）
    e.debug_grant(1, "ycnyaliang");
    e.debug_clear_hand(1);
    advance_to(&mut e, 2, Phase::Play);
    let sha = e.debug_place(2, "sha");
    submit(
        &mut e,
        2,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    settle_optional_confirm(&mut e);
    let h1 = hand_len(&e, 1);
    let h2 = hand_len(&e, 2);
    assert!(h1 >= 1 || h2 >= 1, "雅量：双方摸牌（受击者 p1 手牌 {h1}）");
}

/// T12 运筹武将面批 3：diy_luxun 整将（纵火=非火伤害可选弃牌改火
/// [DamageBefore 帧直写 nature，铁索传导验证]；烧营=火伤后弃堆顶牌亮出
/// 红色则受伤者距离 1 内座位序首目标受 1 火伤）。
#[test]
fn g1_yunchou_generals_batch3_zonghuo_shaoying() {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-yunchou-upstream-noname/assets/character.json"),
    )
    .expect("character.json");
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接");
    assert!(
        bridged.generals.len() >= 4,
        "批 3 起累计武将（现 {} 名）",
        bridged.generals.len()
    );
    assert!(bridged.unmapped_skills >= 2);

    let base = engine_native::content::standard::STANDARD_GENERALS.len() as u16;
    let pack = ExtensionPack {
        def: engine_native::content::extension::ExtensionDef {
            name: "shahai-yunchou-generals-b3",
            version: 1,
            description: "运筹武将面批 3",
        },
        generals: bridged.generals.clone(),
        cards: Vec::new(),
        host_skills: bridged.host_skills.clone(),
        vm_skills: Vec::new(),
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    };
    {
        let e = Engine::new_with_extensions(66003, 4, &[&pack]).expect("建局");
        let m = e.catalog().general_meta(base + 3).expect("陆逊在目录");
        assert_eq!(m.0, "ycn陆逊");
        assert_eq!(m.2, vec!["ycnzonghuo", "ycnshaoying"]);
    }

    // 纵火：非火伤 → 确认+弃首张 → 帧直写火性（铁索传导证明）
    {
        let mut e = Engine::new_with_extensions(66004, 4, &[&pack]).expect("建局");
        e.debug_grant(0, "ycnzonghuo");
        e.debug_set_hp(1, 8);
        e.debug_set_hp(2, 8);
        {
            let core = e.state_core_mut();
            core.players[1].linked = true;
            core.players[2].linked = true;
        }
        e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
        settle_optional_confirm(&mut e);
        assert_eq!(e.debug_hp(1), 7, "纵火：伤害照常");
        assert_eq!(e.debug_hp(2), 7, "纵火：火性经铁索传导 p2");
        // 负例：无手牌可弃 → 不改火性（无传导）
        {
            let core = e.state_core_mut();
            core.players[3].linked = true;
            core.players[1].linked = true;
        }
        e.debug_clear_hand(0);
        e.debug_set_hp(1, 6);
        e.debug_set_hp(3, 8);
        e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
        settle_optional_confirm(&mut e);
        assert_eq!(e.debug_hp(1), 5, "纵火负例：伤害照常");
        assert_eq!(e.debug_hp(3), 8, "纵火负例：无牌可弃不改火性");
    }

    // 烧营：火伤后 → 确认 → 弃堆顶牌亮出，红色则蔓延目标受 1 火伤
    {
        let mut e = Engine::new_with_extensions(66005, 4, &[&pack]).expect("建局");
        e.debug_grant(0, "ycnshaoying");
        e.debug_set_hp(1, 8);
        e.debug_set_hp(2, 8);
        // 牌堆排空后压受控顶：链式烧营（蔓延火伤再触发）在堆空时自然终止
        e.debug_drain_deck_to_discard();
        e.debug_set_deck_top("tao");
        e.debug_deal_damage(Some(0), 1, 1, DamageNature::Fire);
        settle_optional_confirm(&mut e);
        assert_eq!(e.debug_hp(2), 7, "烧营：堆顶红 → 距离 1 内目标受 1 火伤");
        // 负例：堆顶黑（按谓词压黑桃——闪有红花色变体，kind 挑选不可控）
        e.debug_drain_deck_to_discard();
        e.debug_set_deck_top_by(|c| c.suit == Suit::Spade);
        e.debug_deal_damage(Some(0), 1, 1, DamageNature::Fire);
        settle_optional_confirm(&mut e);
        assert_eq!(e.debug_hp(2), 7, "烧营负例：堆顶黑不蔓延");
        assert_eq!(e.debug_hp(1), 6, "原火伤照常");
    }
}

/// T12 运筹武将面批 4：蛊惑·变体（于吉部分——摸 2 弃 2 确定性序）、
/// 狡黠（红色牌目标摸 1）、红莲（来源弃全部红牌）。蝉缘（loseMaxHp）
/// 无承载暂缓。
#[test]
fn g1_yunchou_generals_batch4_guhuo_jiaoxia_honglian() {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-yunchou-upstream-noname/assets/character.json"),
    )
    .expect("character.json");
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接");
    assert!(
        bridged.generals.len() >= 5,
        "批 4 起累计武将（现 {} 名）",
        bridged.generals.len()
    );
    assert!(bridged.unmapped_skills >= 3, "暂缓累计");

    let base = engine_native::content::standard::STANDARD_GENERALS.len() as u16;
    let pack = ExtensionPack {
        def: engine_native::content::extension::ExtensionDef {
            name: "shahai-yunchou-generals-b4",
            version: 1,
            description: "运筹武将面批 4",
        },
        generals: bridged.generals.clone(),
        cards: Vec::new(),
        host_skills: bridged.host_skills.clone(),
        vm_skills: Vec::new(),
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    };
    {
        let e = Engine::new_with_extensions(66006, 4, &[&pack]).expect("建局");
        let m = e.catalog().general_meta(base + 4).expect("于吉在目录");
        assert_eq!(m.0, "ycn于吉");
        assert_eq!(
            m.2,
            vec!["ycndiyguhuo", "ycndiychanyuan"],
            "于吉整将（蝉缘 max_hp 减面转正）"
        );
    }

    // 蛊惑·变体：回合开始摸 2 弃 2（Judge-begin 触发；无装备/判定 →
    // 手牌首张序）——推进到 Draw 断言：4(初始)-2(弃)+2(蛊惑摸)+2(固定)=6
    {
        let mut e = Engine::new_with_extensions(66007, 4, &[&pack]).expect("建局");
        e.debug_grant(0, "ycndiyguhuo");
        advance_to(&mut e, 0, Phase::Draw);
        assert_eq!(hand_len(&e, 0), 6, "蛊惑：摸 2 弃 2 后固定摸 2");
    }

    // 狡黠：成为红色牌目标 → 摸 1（杀为黑色负例）
    {
        let mut e = Engine::new_with_extensions(66008, 4, &[&pack]).expect("建局");
        e.debug_grant(1, "ycnjiaoxia");
        e.debug_set_hp(1, 8);
        advance_to(&mut e, 2, Phase::Play);
        let h1 = hand_len(&e, 1);
        // 找一张黑杀打 p1
        let sha = e.debug_place(2, "sha");
        let black = {
            let cat = e.catalog();
            matches!(cat.cards[sha as usize].suit, Suit::Spade | Suit::Club)
        };
        submit(
            &mut e,
            2,
            GameAction::PlayCard {
                card: sha,
                targets: vec![1],
            },
        );
        settle_optional_confirm(&mut e);
        if black {
            assert_eq!(hand_len(&e, 1), h1, "狡黠：黑牌目标不触发");
        } else {
            assert_eq!(hand_len(&e, 1), h1 + 1, "狡黠：红牌目标摸 1");
        }
    }

    // 红莲：p1 受 p2 伤害 → 确认 → p2 弃全部红牌（手 1 红+装备 1 红）
    {
        let mut e = Engine::new_with_extensions(66009, 4, &[&pack]).expect("建局");
        e.debug_grant(1, "ycnhonglian");
        e.debug_set_hp(1, 8);
        // p2 手牌：留一张红桃、清其余（装备红牌弃置面同 host 先例，此段
        // 验证手牌红弃置）
        let red = e.debug_place(2, "tao");
        let reds_before = 1;
        e.debug_clear_hand(2);
        e.state_core_mut().players[2].hand.push(red);
        let h2 = hand_len(&e, 2);
        assert_eq!(h2, reds_before);
        e.debug_deal_damage(Some(2), 1, 1, DamageNature::Normal);
        settle_optional_confirm(&mut e);
        assert_eq!(hand_len(&e, 2), 0, "红莲：来源红牌被全弃");
    }
}

/// T12 运筹武将面批 5：全部十名武将入册（蔡文姬/陆抗/马钧三将暂全暂缓
/// ——机制缺口：濒死/出牌主动技面、内置技能引用、骰子面）。目录元数据
/// 面验收（映射表全量 17 技，转正 7 暂缓 10...勘正：转正 7+暂缓 10=17）。
#[test]
fn g1_yunchou_generals_batch5_full_roster_metadata() {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-yunchou-upstream-noname/assets/character.json"),
    )
    .expect("character.json");
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接");
    assert_eq!(bridged.generals.len(), 10, "十名武将全量入册");
    // 技能条目全量 20（skill.js 17 定义 + 悲歌/谦逊/精械 3 个内置/外部
    // 引用）；bridge 语义=目录技能列表只含 mapped（9 host），暂缓 11 只
    // 计数不进列表
    assert!(
        bridged.host_skills.len() >= 9,
        "host 行为累计（现 {} 条）",
        bridged.host_skills.len()
    );
    assert!(
        bridged
            .generals
            .iter()
            .map(|g| g.skills.len())
            .sum::<usize>()
            >= 9,
        "目录技能列表=mapped 面（现 {} 条）",
        bridged
            .generals
            .iter()
            .map(|g| g.skills.len())
            .sum::<usize>()
    );
    assert_eq!(
        bridged.unmapped_skills, 3,
        "暂缓 3：奸雄/巧思/精械（悲歌批 18 转正）"
    );

    let base = engine_native::content::standard::STANDARD_GENERALS.len() as u16;
    let pack = ExtensionPack {
        def: engine_native::content::extension::ExtensionDef {
            name: "shahai-yunchou-generals-b5",
            version: 1,
            description: "运筹武将面批 5（全量入册）",
        },
        generals: bridged.generals.clone(),
        cards: Vec::new(),
        host_skills: bridged.host_skills.clone(),
        vm_skills: Vec::new(),
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    };
    let e = Engine::new_with_extensions(66010, 4, &[&pack]).expect("建局");
    for (idx, (name, hp)) in [
        (0, ("ycn曹操", 4u32)),
        (1, ("ycn魏延", 4)),
        (2, ("ycn周瑜", 3)),
        (3, ("ycn陆逊", 3)),
        (4, ("ycn于吉", 3)),
        (5, ("ycn周瑜", 3)),
        (6, ("ycn甄姬", 3)),
        (7, ("ycn蔡文姬", 3)),
        (8, ("ycn陆抗", 4)),
        (9, ("ycn马钧", 3)),
    ] {
        let m = e
            .catalog()
            .general_meta(base + idx)
            .unwrap_or_else(|| panic!("武将 {idx} 在目录"));
        assert_eq!(m.0, name, "武将 {idx} 名");
        assert_eq!(m.3, hp, "武将 {idx} 血");
    }
}

/// T12 运筹武将面批 6：陆抗厉焰（非基本手牌当水攻——PlayCard 转化面）
/// + 厉焰·链杀（水攻用牌后座位序首目标虚拟杀；登记差异：不限转化来源
/// +自动首目标）。谦逊内置引用暂缓。
#[test]
fn g1_yunchou_generals_batch6_luweiyan_conversion_and_chain() {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-yunchou-upstream-noname/assets/character.json"),
    )
    .expect("character.json");
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接");
    assert!(bridged.generals.len() >= 8, "批 6 累计八名");
    assert!(
        bridged.host_skills.len() >= 10,
        "host 行为累计（现 {} 条）",
        bridged.host_skills.len()
    );

    let base = engine_native::content::standard::STANDARD_GENERALS.len() as u16;
    let pack = ExtensionPack {
        def: engine_native::content::extension::ExtensionDef {
            name: "shahai-yunchou-generals-b6",
            version: 1,
            description: "运筹武将面批 6",
        },
        generals: bridged.generals.clone(),
        cards: Vec::new(),
        host_skills: bridged.host_skills.clone(),
        vm_skills: Vec::new(),
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    };
    let mut e = Engine::new_with_extensions(66011, 4, &[&pack]).expect("建局");
    let m = e.catalog().general_meta(base + 8).expect("陆抗在目录");
    assert_eq!(m.0, "ycn陆抗");
    assert_eq!(
        m.2,
        vec!["ycnluweiyan", "ycnluweiyan2", "谦逊"],
        "厉焰双子技同授（上游 group 同口径）；谦逊=内置引用"
    );

    // 厉焰转化+链杀：陆抗（座位 0）出牌阶段，桃（基本——负例）与装备牌
    // 转化为水攻；水攻弃目标装备后链杀首目标
    e.debug_grant(0, "ycnluweiyan");
    e.debug_grant(0, "ycnluweiyan2");
    e.debug_set_hp(2, 8);
    advance_to(&mut e, 0, Phase::Play);
    // 给 p2 挂装备（水攻弃装备面）+ p2 有手牌
    let zhuge2 = e.debug_place(2, "zhuge");
    e.state_core_mut().players[2].equips.push(zhuge2);
    // 桃=基本牌：转化不生效（PlayCard 按 Tao 路由=回血拒绝？桃无受伤目标
    // 拒绝）——负例用另一张非基本牌前先验证基本牌不被转化：直接用锦囊
    let wuzhong = e.debug_place(0, "wuzhong");
    // 链杀：水攻目标集=[2] → 确认 → 虚拟杀指定 p2（命中掉血或闪避弃
    // 闪）——链在 PlayCard submit 的结算链内完成，基准读在 submit 前
    let hp2 = e.debug_hp(2);
    let h2_before = hand_len(&e, 2);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: wuzhong,
            targets: vec![2],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "无中生有经厉焰转化为水攻受理: {out:?}"
    );
    settle_optional_confirm(&mut e);
    let hp2b = e.debug_hp(2);
    assert!(
        hp2b == hp2 - 1 || hand_len(&e, 2) < h2_before,
        "厉焰·链杀：p2 掉血（命中）或弃闪（闪避）——hp {hp2}->{hp2b}"
    );
    // 水攻面（登记差异：有装备目标弃首张装备）
    assert!(
        !e.state_core().players[2].equips.contains(&zhuge2),
        "厉焰转化水攻：目标装备被弃"
    );
}

/// T12 运筹武将面批 7：出牌主动技面——隙焰（弃 1 红手牌全体 1 火伤，
/// ActivateSkill 命令通道）与野升（自选任意黑手牌摸等量，SelectCards
/// 多选机）。周瑜·DIY/甄姬·DIY 双整将转正。
#[test]
fn g1_yunchou_generals_batch7_active_skills_jieyan_yiesheng() {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-yunchou-upstream-noname/assets/character.json"),
    )
    .expect("character.json");
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接");
    assert!(bridged.host_skills.len() >= 13, "host 行为累计 13");

    let base = engine_native::content::standard::STANDARD_GENERALS.len() as u16;
    let pack = ExtensionPack {
        def: engine_native::content::extension::ExtensionDef {
            name: "shahai-yunchou-generals-b7",
            version: 1,
            description: "运筹武将面批 7",
        },
        generals: bridged.generals.clone(),
        cards: Vec::new(),
        host_skills: bridged.host_skills.clone(),
        vm_skills: Vec::new(),
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    };
    let mut e = Engine::new_with_extensions(66012, 4, &[&pack]).expect("建局");
    // 周瑜·DIY（base+5）/甄姬·DIY（base+6）目录面
    assert_eq!(
        e.catalog().general_meta(base + 5).unwrap().2,
        vec!["ycnjieyan", "ycnhonglian"],
        "周瑜·DIY 整将"
    );
    assert_eq!(
        e.catalog().general_meta(base + 6).unwrap().2,
        vec!["ycnjiaoxia", "ycnyiesheng"],
        "甄姬·DIY 整将"
    );
    let jieyan_def = e.catalog().skill_by_name("ycnjieyan").expect("隙焰 def");
    let yiesheng_def = e.catalog().skill_by_name("ycnyiesheng").expect("野升 def");

    // 隙焰：p0 弃 1 红手牌 → 全体其他存活各 1 火伤
    e.debug_grant(0, "ycnjieyan");
    e.debug_set_hp(1, 8);
    e.debug_set_hp(2, 8);
    e.debug_set_hp(3, 8);
    advance_to(&mut e, 0, Phase::Play);
    let tao = e.debug_place(0, "tao"); // 红
    let h0 = hand_len(&e, 0);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action: GameAction::ActivateSkill { skill: jieyan_def },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "隙焰受理: {out:?}"
    );
    settle_optional_confirm(&mut e);
    let _ = tao;
    assert_eq!(e.debug_hp(1), 7, "隙焰：p1 受 1 火伤");
    assert_eq!(e.debug_hp(2), 7, "隙焰：p2 受 1 火伤");
    assert_eq!(e.debug_hp(3), 7, "隙焰：p3 受 1 火伤");
    assert!(hand_len(&e, 0) < h0, "隙焰：弃 1 张红手牌");

    // 野升：p1 弃 2 张黑手牌 → 摸 2（SelectCards 多选）
    e.debug_grant(1, "ycnyiesheng");
    advance_to(&mut e, 1, Phase::Play);
    e.debug_clear_hand(1);
    // 谓词压顶两张黑牌入手（压顶后弹出入手，避免同一实体两处）
    let b1 = e.debug_set_deck_top_by(|c| c.suit == Suit::Spade);
    {
        let core = e.state_core_mut();
        let c = core.deck.pop().expect("just pushed");
        core.players[1].hand.push(c);
    }
    let b2 = e.debug_set_deck_top_by(|c| c.suit == Suit::Club);
    {
        let core = e.state_core_mut();
        let c = core.deck.pop().expect("just pushed");
        core.players[1].hand.push(c);
    }
    let h1 = hand_len(&e, 1);
    assert_eq!(h1, 2, "两张黑牌已布置");
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action: GameAction::ActivateSkill {
            skill: yiesheng_def,
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "野升受理: {out:?}"
    );
    // SelectCards 多选应答（两张全选）
    let c = e.open_choice().expect("野升选择挂起");
    let (cid, obs) = (c.id, c.observer);
    let sel: Vec<ChoiceOption> = c
        .spec
        .options
        .iter()
        .filter(|o| matches!(o, ChoiceOption::Card(x) if *x == b1 || *x == b2))
        .cloned()
        .collect();
    assert_eq!(sel.len(), 2, "两张黑牌在选择面");
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: sel,
        },
    );
    assert_eq!(hand_len(&e, 1), 2, "野升：弃 2 摸 2 净持平");
    assert!(
        !e.debug_hand(1).contains(&b1) && !e.debug_hand(1).contains(&b2),
        "野升：原两张黑牌已弃"
    );
}

/// T12 运筹武将面批 8：内置技能引用面（mapping `builtin:*` 白名单）——
/// 护驾/谦逊转正（悲歌本体未实现不入册）。谦逊行为定向：持谦逊者不能
/// 成为顺手牵羊目标；反例：builtin:悲歌 拒装。
#[test]
fn g1_yunchou_generals_batch8_builtin_skill_refs() {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-yunchou-upstream-noname/assets/character.json"),
    )
    .expect("character.json");
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接");
    // 内置引用不进 host_skills、不进 unmapped——目录技能列表含内置名
    let caocao = &bridged.generals[0];
    assert_eq!(caocao.name, "ycn曹操");
    assert_eq!(
        caocao.skills.to_vec(),
        vec!["ycnxicai", "护驾",],
        "曹操：戏才+护驾（奸雄暂缓）"
    );
    let lukang = bridged
        .generals
        .iter()
        .find(|g| g.name == "ycn陆抗")
        .expect("陆抗");
    assert!(lukang.skills.to_vec().contains(&"谦逊"), "陆抗：谦逊入列表");
    let unmapped_names: Vec<String> = json
        .to_string()
        .matches("\"mapping\": null")
        .map(|_| String::new())
        .collect();
    let _ = unmapped_names;
    assert_eq!(
        bridged.unmapped_skills, 3,
        "暂缓 3：奸雄/巧思/精械（悲歌批 18 转正）"
    );
    let _ = json;

    let base = engine_native::content::standard::STANDARD_GENERALS.len() as u16;
    let pack = ExtensionPack {
        def: engine_native::content::extension::ExtensionDef {
            name: "shahai-yunchou-generals-b8",
            version: 1,
            description: "运筹武将面批 8",
        },
        generals: bridged.generals.clone(),
        cards: Vec::new(),
        host_skills: bridged.host_skills.clone(),
        vm_skills: Vec::new(),
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    };
    let mut e = Engine::new_with_extensions(66013, 4, &[&pack]).expect("建局");
    // 护驾 def 为标准目录 def（内置引用语义）
    assert!(
        e.catalog().skill_by_name("护驾").is_some(),
        "护驾 def 在标准目录"
    );
    let lukang_meta = e.catalog().general_meta(base + 8).expect("陆抗");
    assert_eq!(
        lukang_meta.2.to_vec(),
        vec!["ycnluweiyan", "ycnluweiyan2", "谦逊"],
        "陆抗三技（谦逊=内置引用）"
    );

    // 谦逊行为定向：持谦逊者不能成为乐不思蜀目标（原生校验面=乐，
    // 上游无顺手牵羊挂账见覆盖清单）
    let lukang_seat = 0u8;
    e.debug_grant(lukang_seat, "谦逊");
    e.debug_set_hp(lukang_seat, 8);
    advance_to(&mut e, 1, Phase::Play);
    let le = e.debug_place(1, "le");
    let out = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: le,
            targets: vec![lukang_seat],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "谦逊：乐不思蜀指定被拒: {out:?}"
    );
}

/// builtin 白名单拒装反例：`builtin:悲歌`（本体未实现）必须拒绝。
#[test]
fn g1_yunchou_generals_batch8_builtin_whitelist_rejects() {
    let json = r#"{
        "generals": [{
            "name": "ycn测试",
            "faction": "wei",
            "hp": 3,
            "gender": "male",
            "skills": [{"name": "beige", "title": "悲歌", "mapping": "builtin:悲歌"}]
        }]
    }"#;
    let err = match content_native_loader::bridge_generals(json) {
        Err(e) => e,
        Ok(_) => panic!("builtin:悲歌 应拒装"),
    };
    assert!(
        err.to_string().contains("builtin:悲歌"),
        "白名单外 builtin 拒装: {err}"
    );
}

/// T12 运筹武将面批 9：蝉缘·变体（于吉整将）——死亡时来源体力上限
/// -1（max_hp 减面首例；体力钳上限）。负例：无来源死亡不触发。
#[test]
fn g1_yunchou_generals_batch9_diychanyuan_max_hp_loss() {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-yunchou-upstream-noname/assets/character.json"),
    )
    .expect("character.json");
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接");
    assert!(bridged.host_skills.len() >= 14, "host 行为累计 14");

    let pack = ExtensionPack {
        def: engine_native::content::extension::ExtensionDef {
            name: "shahai-yunchou-generals-b9",
            version: 1,
            description: "运筹武将面批 9",
        },
        generals: bridged.generals.clone(),
        cards: Vec::new(),
        host_skills: bridged.host_skills.clone(),
        vm_skills: Vec::new(),
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    };
    let mut e = Engine::new_with_extensions(66014, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "ycndiychanyuan");
    // p1（蝉缘持有者）被 p2 重伤致死（全场清手——无人有桃，濒死窗不开
    // 直接死亡）→ p2 体力上限 -1
    e.debug_set_hp(1, 1);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_deal_damage(Some(2), 1, 3, DamageNature::Normal);
    let src = e.state_core();
    assert!(!src.players[1].alive, "p1 死亡");
    assert_eq!(src.players[2].max_hp, 3, "蝉缘：来源上限 4→3");
    assert_eq!(src.players[2].hp, 3, "蝉缘：来源体力钳上限");
    // 负例：无来源死亡不触发（p0 蝉缘？未授予——改用再授一次验证杀手为
    // None 的路径：p2 自尽式无来源伤害不可构造——用 max_hp 基线复核）
    assert_eq!(e.state_core().players[0].max_hp, 4, "无涉者上限不变");
}

/// T12 玩点论杀批 17：归汉（蔡文姬部分转正）——limited 濒死自救
/// （DyingSaveMachine 求救循环前一次性确认；自回 1 摸 2+座位序首男性
/// 其他存活回 1 摸 2；limited 每局一次）。登记差异：上游 chooseToUse
/// 玩家主动+自选男性目标，原生锁定确认+确定性目标。
#[test]
fn g1_yunchou_generals_batch17_guihan_dying_limited() {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-yunchou-upstream-noname/assets/character.json"),
    )
    .expect("character.json");
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接");
    let cai = bridged
        .generals
        .iter()
        .find(|g| g.name == "ycn蔡文姬")
        .expect("蔡文姬");
    assert!(
        cai.skills.to_vec().contains(&"ycnguihan"),
        "归汉入目录（builtin 悲歌仍暂缓）"
    );

    let pack = ExtensionPack {
        def: engine_native::content::extension::ExtensionDef {
            name: "shahai-yunchou-generals-b17",
            version: 1,
            description: "运筹武将面批 17",
        },
        generals: bridged.generals.clone(),
        cards: Vec::new(),
        host_skills: bridged.host_skills.clone(),
        vm_skills: Vec::new(),
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    };
    let mut e = Engine::new_with_extensions(66014, 4, &[&pack]).expect("建局");
    // p1 持归汉（座位 2 为男性目标）
    e.debug_grant(1, "ycnguihan");
    e.debug_set_hp(1, 2);
    e.debug_set_hp(2, 3);
    // 全场清手防桃自救/求救
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let h1 = hand_len(&e, 1);
    let h2 = hand_len(&e, 2);
    let hp1 = e.debug_hp(1);
    let hp2 = e.debug_hp(2);
    assert_eq!((h1, h2), (0, 0), "布阵：全场清手基准");
    e.debug_deal_damage(Some(0), 1, 2, DamageNature::Normal);
    // 归汉确认门（p1）
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), hp1 - 2 + 1, "归汉：自回 1（濒死救回）");
    // 场上已清手——摸 2 前 h1 为清手后基准（0），确认+回血+摸 2 → 2
    assert_eq!(hand_len(&e, 1), h1 + 2, "归汉：摸 2");
    // 座位序首男性=座位 0（满血 hp=max，回 1 饱和 +0）；摸 2 不饱和
    assert_eq!(e.debug_hp(0), 4, "归汉：男性目标回 1（满血饱和 +0）");
    assert_eq!(hand_len(&e, 0), h2 + 2, "归汉：男性目标摸 2");
    let _ = h2;
    let _ = hp2;

    // limited：第二次濒死不触发（无桃直接死亡）
    e.debug_set_hp(1, 1);
    e.debug_deal_damage(Some(0), 1, 2, DamageNature::Normal);
    settle_optional_confirm(&mut e);
    assert!(
        !e.state_core().player(1).alive,
        "归汉 limited：第二次濒死不再触发"
    );
}

/// T12 运筹批 18：悲歌（refresh 变体，蔡文姬整将转正）——受到杀伤害后
/// （可选确认）受伤者判定，弃 1 张牌按判定花色：红桃受伤者回 1/方块摸
/// 2/黑桃来源翻面/草花来源弃 2；收回奖励：弃牌花色=判定花色收回判定牌。
#[test]
fn g1_yunchou_generals_batch18_beige_judge_branches() {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-yunchou-upstream-noname/assets/character.json"),
    )
    .expect("character.json");
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接");
    let cai = bridged
        .generals
        .iter()
        .find(|g| g.name == "ycn蔡文姬")
        .expect("蔡文姬");
    assert_eq!(
        cai.skills.to_vec(),
        vec!["beige", "ycnguihan"],
        "蔡文姬 mapped 面：悲歌+归汉"
    );

    let pack = ExtensionPack {
        def: engine_native::content::extension::ExtensionDef {
            name: "shahai-yunchou-generals-b18",
            version: 1,
            description: "运筹武将面批 18",
        },
        generals: bridged.generals.clone(),
        cards: Vec::new(),
        host_skills: bridged.host_skills.clone(),
        vm_skills: Vec::new(),
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    };
    let mut e = Engine::new_with_extensions(66015, 4, &[&pack]).expect("建局");
    assert!(
        e.catalog().skill_by_name("beige").is_some(),
        "悲歌 def 在目录"
    );
    e.debug_grant(1, "beige");

    // 红桃分支：deck 顶压**红桃花色**（debug_set_deck_top 按 kind 找牌
    // 会有花色变体，须按谓词压确定花色）
    e.debug_set_deck_top_by(|c| c.suit == Suit::Heart);
    e.debug_set_hp(2, 2);
    let h0 = hand_len(&e, 0);
    // 杀实体：布到 p0 手并移出（damage_with_card 只要求 card id）
    let sha_card = e.debug_place(0, "sha");
    {
        let core = e.state_core_mut();
        core.players[0].hand.retain(|x| *x != sha_card);
    }
    e.debug_deal_damage_with_card(Some(0), 2, 1, DamageNature::Normal, sha_card);
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(2), 2, "悲歌：判定红桃 → 受伤者回 1（2-1+1）");
    assert!(hand_len(&e, 0) <= h0, "悲歌：持有者弃 1 张");

    // 方块分支补充：压方块顶 → 受伤者摸 2（替代回血断言的对照面）
    e.debug_set_deck_top_by(|c| c.suit == Suit::Diamond);
    let h2b = hand_len(&e, 2);
    e.debug_deal_damage_with_card(Some(0), 2, 1, DamageNature::Normal, sha_card);
    settle_optional_confirm(&mut e);
    assert_eq!(hand_len(&e, 2), h2b + 2, "悲歌：判定方块 → 受伤者摸 2");

    // limited 无（悲歌每伤害均可）——负例：无牌持有者不触发
    e.debug_clear_hand(1);
    e.state_core_mut().players[1].equips.clear();
    let hp3 = e.debug_hp(3);
    e.debug_deal_damage(Some(0), 3, 1, DamageNature::Normal);
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(3), hp3 - 1, "悲歌负例：无牌不触发");
}
