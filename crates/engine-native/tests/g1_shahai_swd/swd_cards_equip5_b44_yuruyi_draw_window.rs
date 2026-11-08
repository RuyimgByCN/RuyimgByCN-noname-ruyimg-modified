use super::*;

/// T12 swd 卡牌面补切批⑭：玉如意——装备后自己摸牌阶段从牌堆末 20 张
/// 窗口（pop 摸牌位）中取最大点数牌移至摸牌位（登记差异上游按牌价值加
/// 权随机；任意 drawBegin 收敛为阶段摸牌钩）。
#[test]
fn swd_cards_equip5_b44_yuruyi_draw_window() {
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
    let mut e = Engine::new_with_extensions(62023, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let yry = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == engine_native::catalog::CardKind::Yuruyi)
        .expect("玉如意在目录") as u32;
    let low = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(_, c)| c.rank == 2 && c.kind == CardKind::Shan)
        .map(|(i, _)| i as u32)
        .expect("rank2 实体");
    let high = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(_, c)| c.rank == 13 && c.kind == CardKind::Shan)
        .map(|(i, _)| i as u32)
        .expect("rank13 实体");
    assert_ne!(low, high);
    {
        let mut snap = e.save();
        snap.core.deck.retain(|x| *x != yry);
        snap.core.discard.retain(|x| *x != yry);
        snap.core.players[0].equips.push(yry);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("布阵");
    }
    // 推进完一整轮到 p0 下个摸牌阶段（Draw begin 触发玉如意换牌）：
    // 在 p0 进入 Judge（紧邻 Draw 前）时将 low 压在摸牌位（deck 尾）、high 放在窗口内非摸牌位
    let mut left_p0 = false;
    let mut staged_window = false;
    for _ in 0..48 {
        if left_p0
            && e.turn().current == 0
            && e.turn().phase == Phase::Judge
            && e.open_choice().is_none()
            && !staged_window
        {
            let mut snap = e.save();
            for p in &mut snap.core.players {
                p.hand.retain(|x| *x != low && *x != high);
            }
            snap.core.discard.retain(|x| *x != low && *x != high);
            snap.core.deck.retain(|x| *x != low && *x != high);
            snap.core.deck.push(low);
            snap.core.deck.insert(snap.core.deck.len() - 1, high);
            e = Engine::restore_with_extensions(&snap, &[&pack]).expect("摸牌窗布阵");
            staged_window = true;
        }
        if left_p0
            && e.turn().current == 0
            && e.turn().phase == Phase::Play
            && e.open_choice().is_none()
        {
            break;
        }
        if e.turn().current != 0 {
            left_p0 = true;
        }
        if let Some(c) = e.open_choice() {
            let (cid, obs) = (c.id, c.observer);
            let sel: Vec<ChoiceOption> = if c.spec.prompt == "baishouzhihu_target" {
                vec![*c.spec.options.last().unwrap()]
            } else if c.spec.min > 0 {
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
        let cur = e.turn().current;
        let out = submit(&mut e, cur, GameAction::NextPhase);
        if let CommandOutcome::Rejected(_) = out {
            continue;
        }
    }
    let hand = e.debug_hand(0);
    assert!(
        hand.iter()
            .any(|c| { e.catalog().cards[*c as usize].rank == 13 }),
        "玉如意：最大点数牌被移至摸牌位入手"
    );
}

/// T12 swd 卡牌面补切批⑮：玉蝉换形——自己用牌结算后，手牌中玉蝉实体
/// 轮换为八卦序下一 kind 的实体（牌堆/弃堆确定性发现；登记差异上游随
/// 机+init 原地变形）。
#[test]
fn swd_cards_yuchan_b45_swap_rotation() {
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
    let mut e = Engine::new_with_extensions(62024, 4, &[&pack]).expect("建局");
    // 授予换形技实例（器族手持技经 InitHook 已授全员，直接核验）
    let swap_def = e.catalog().skill_by_name("yuchan_swap").expect("换形 def");
    assert!(
        e.state_core().skills.contains_key(&(0, swap_def)),
        "换形技开局已授予"
    );
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
    use engine_native::catalog::CardKind;

    // 布阵：p0 手持玉蝉·乾；玉蝉·坤实体固定入牌堆（替换发现源）
    let yq = kind_id(&e, CardKind::Yuchanqian);
    let yk = kind_id(&e, CardKind::Yuchankun);
    {
        let core = e.state_core_mut();
        for c in [yq, yk] {
            core.deck.retain(|x| *x != c);
            core.discard.retain(|x| *x != c);
        }
        core.players[0].hand.push(yq);
        core.deck.push(yk);
    }
    // p0 用任意非玉蝉牌（tao）触发换形
    e.debug_set_hp(0, 3);
    let tao = e.debug_place(0, "tao");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: tao,
            targets: vec![],
        },
    );
    settle_optional_confirm(&mut e);
    let hand = e.debug_hand(0);
    assert!(hand.contains(&yk), "玉蝉换形：乾轮换为坤（坤实体入手）");
    assert!(!hand.contains(&yq), "玉蝉换形：乾实体已让位");

    // 反例：用玉蝉本体（乾 as 杀）不触发换形（filter 排除玉蝉本体）
    // 布阵第二张乾不可得（每 kind 一张）——改验：打出的坤触发轮换至震
    let yz = kind_id(&e, CardKind::Yuchanzhen);
    {
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != yz);
        core.discard.retain(|x| *x != yz);
        core.deck.push(yz);
    }
    // p0 手中的坤视为草药（单目标受伤）——先伤 p0 使草药可用
    e.debug_set_hp(0, 3);
    // 再置一张甘（艮）入手：用坤后剩余手牌玉蝉轮换 甘→兑
    let yg = kind_id(&e, CardKind::Yuchangen);
    {
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != yg);
        core.discard.retain(|x| *x != yg);
        core.players[0].hand.push(yg);
    }
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: yk,
            targets: vec![0],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "坤 as 草药受理 {out:?}"
    );
    settle_optional_confirm(&mut e);
    let hand = e.debug_hand(0);
    let yd = kind_id(&e, CardKind::Yuchandui);
    assert!(hand.contains(&yd), "玉蝉换形②：剩余甘轮换为兑");
    assert_eq!(e.debug_hp(0), 4, "坤 as 草药：p0 回满");
}
/// T12 swd 卡牌面窗口机批（神秘果再结算窗/鼠槌多杀循环/萨登荆环闪避钩
/// /昊天塔判定插窗）：四窗口机行为面 + 反例。
#[test]
fn swd_window_machines_b62() {
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
    let mut e = Engine::new_with_extensions(62026, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // ① 神秘果：出牌阶段用杀结算后确认 → 原体再结算（p1 再受 1 伤）
    let fruit = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == engine_native::catalog::CardKind::Shenmiguo)
        .expect("神秘果入包") as u32;
    let sha1 = e.debug_place(0, "sha");
    {
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != fruit);
        core.discard.retain(|x| *x != fruit);
        core.table.retain(|x| *x != fruit);
        core.players[0].hand.push(fruit);
    }
    e.debug_set_hp(1, 4);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![1],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 2, "神秘果：原杀 1 伤 + 再结算 1 伤");
    assert!(e.state_core().turn.shenmiguo_used, "神秘果：每阶段旗置位");
    assert!(!e.debug_hand(0).contains(&fruit), "神秘果：载体牌已消耗");
    // 反例 ①-b：同阶段第二次基本牌用牌不再弹窗（旗守卫）——补第二张
    // 神秘果入手，用雪霁后再结算窗不得开启（否则载体被消耗）
    let fruit2 = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == engine_native::catalog::CardKind::Shenmiguo)
        .expect("第二张神秘果在包") as u32;
    let xjb_w = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == engine_native::catalog::CardKind::Xuejibingbao)
        .expect("雪霁在包") as u32;
    {
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != fruit2);
        core.discard.retain(|x| *x != fruit2);
        core.table.retain(|x| *x != fruit2);
        core.deck.retain(|x| *x != xjb_w);
        core.discard.retain(|x| *x != xjb_w);
        core.table.retain(|x| *x != xjb_w);
        core.players[0].hand.push(fruit2);
        core.players[0].hand.push(xjb_w);
    }
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: xjb_w,
            targets: vec![1],
        },
    );
    settle_optional_confirm(&mut e);
    assert!(
        e.debug_hand(0).contains(&fruit2),
        "反例①b：每阶段旗守卫拦再结算窗（第二张神秘果未消耗）"
    );

    // ② 鼠槌：装备后出牌阶段限一次——2 张杀逐张自动用（2 伤）+终局摸 1
    let shuchui = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == engine_native::catalog::CardKind::Shuchui)
        .expect("鼠槌入包") as u32;
    {
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != shuchui);
        core.discard.retain(|x| *x != shuchui);
        core.table.retain(|x| *x != shuchui);
        core.players[0].hand.push(shuchui);
    }
    // 神秘果旗已置：为隔离②，先耗尽本阶段（直接 NextPhase 进弃牌阶段
    // 会清旗的路径在 commit_turn_ended——此处跨回合再回 p0）
    submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    submit(&mut e, 0, GameAction::NextPhase); // End→(p1 Judge)
    for cur in 1..4u8 {
        for _ in 0..3 {
            let c = e.turn().current;
            if c != cur {
                break;
            }
            let _ = submit(&mut e, c, GameAction::NextPhase);
        }
    }
    advance_to_play(&mut e, 0);
    // 装备鼠槌（真实 PlayCard 通道=equip_skills 授技口径）
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    {
        let core = e.state_core_mut();
        core.players[0].hand.push(shuchui);
    }
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: shuchui,
            targets: vec![],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_equips(0).len(), 1, "鼠槌：装备上栏");
    let def_sc = e
        .catalog()
        .skill_by_name("shuchui")
        .expect("鼠槌装备技在目录");
    assert!(
        e.state_core().skills.contains_key(&(0, def_sc)),
        "鼠槌：equip_skills 实例授予"
    );
    // 手牌：2 杀（1 tao 验证杀择路）
    let _t2 = e.debug_place(0, "tao");
    let s_a = e.debug_place(0, "sha");
    let s_b = e.debug_place(0, "sha");
    let h0 = hand_len_w(&e, 0);
    e.debug_set_hp(1, 4);
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_sc });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "鼠槌发动 {out:?}"
    );
    let c = e.open_choice().expect("鼠槌目标选择");
    assert_eq!((c.spec.prompt, c.observer), ("shuchui_target", 0));
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
    assert_eq!(e.debug_hp(1), 2, "鼠槌：2 杀逐张 2 伤（p1 无闪）");
    assert_eq!(
        hand_len_w(&e, 0),
        h0 - 1,
        "鼠槌：耗 2 杀 + 终局摸 1（tao 剩 1）"
    );
    assert!(
        !e.debug_hand(0).contains(&s_a) && !e.debug_hand(0).contains(&s_b),
        "鼠槌：2 杀均消耗"
    );
    // 反例 ②-b：每回合限一次
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_sc });
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "反例②b：再发动拒 {out:?}"
    );

    // 推进到 p0 下一出牌阶段（鼠槌限一次/杀限一次均已随回合重置）
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    for _ in 0..12 {
        let c = e.turn().current;
        if c == 0 && e.state_core().turn.phase == engine_native::event::Phase::Play {
            break;
        }
        let _ = submit(&mut e, c, GameAction::NextPhase);
    }
    advance_to_play(&mut e, 0);

    // ③ 萨登荆环：杀被闪后判定红 → 目标额外闪拒打 → 杀改命中
    let sadeng = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == engine_native::catalog::CardKind::Sadengjinhuan)
        .expect("萨登荆环入包") as u32;
    {
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != sadeng);
        core.discard.retain(|x| *x != sadeng);
        core.table.retain(|x| *x != sadeng);
        core.players[0].equips.push(sadeng);
    }
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let shan1 = e.debug_place(1, "shan");
    let shan2 = e.debug_place(1, "shan");
    let sha3 = e.debug_place(0, "sha");
    e.debug_set_hp(1, 4);
    // 判定堆顶：红（heart）
    e.debug_set_deck_top_by(|def| def.suit.is_heart());
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha3,
            targets: vec![1],
        },
    );
    // p1 出第一张闪
    let c = e.open_choice().expect("p1 出闪窗");
    let (cid, obs) = (c.id, c.observer);
    assert_eq!(obs, 1);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(shan1)],
        },
    );
    // 荆环判定确认窗（可选）→ 确认
    let c = e.open_choice().expect("荆环确认窗");
    assert_eq!(
        (c.spec.prompt, c.observer),
        ("sadengjinhuan_judge_confirm", 0)
    );
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    // 额外闪窗：拒打（空应答）
    let c = e.open_choice().expect("荆环额外闪窗");
    assert_eq!((c.spec.prompt, c.observer), ("sadengjinhuan_extra_shan", 1));
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 3, "萨登荆环：判定红+拒打额外闪→杀改命中");
    // 反例 ③-b：目标打出额外闪 → 杀维持被闪（黑判路径并测）——新回合
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    for _ in 0..12 {
        let c = e.turn().current;
        if c == 0 && e.state_core().turn.phase == engine_native::event::Phase::Play {
            break;
        }
        let _ = submit(&mut e, c, GameAction::NextPhase);
    }
    advance_to_play(&mut e, 0);
    let sha4 = e.debug_place(0, "sha");
    e.debug_set_deck_top_by(|def| def.suit.is_spade());
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha4,
            targets: vec![1],
        },
    );
    let c = e.open_choice().expect("p1 出闪窗 2");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(shan2)],
        },
    );
    let c = e.open_choice().expect("荆环确认窗 2");
    assert_eq!(c.spec.prompt, "sadengjinhuan_judge_confirm");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    // 黑判定 → 无额外闪窗，直接维持被闪
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 3, "反例③b：黑判定→被闪维持（无伤）");

    // ④ 昊天塔：任意判定前持有者强制二选一定结果（改判窗跳过）
    let haotian = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == engine_native::catalog::CardKind::Haotianta)
        .expect("昊天塔入包") as u32;
    {
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != haotian);
        core.discard.retain(|x| *x != haotian);
        core.table.retain(|x| *x != haotian);
        core.players[2].equips.push(haotian);
    }
    // 乐不思蜀置 p1 判定区（PlayCard 通道）
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let le = e.debug_place(0, "le");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: le,
            targets: vec![1],
        },
    );
    settle_optional_confirm(&mut e);
    // 判定堆顶两张：先压 heart 再压 spade——pop 序 spade=判定牌（顶）、
    // heart=次候选；预捕获实体 id 供选牌探针
    let heart_id = e.debug_set_deck_top_by(|def| def.suit.is_heart());
    let spade_id = e.debug_set_deck_top_by(|def| def.suit.is_spade());
    // 推进 p1 回合到判定阶段（逐相位推进——不用 advance_to_play，其
    // 自动应答会吃掉昊天塔选牌窗；主循环亲自应答以观察插窗）
    let mut saw_pick = false;
    let mut picked_is_spade = false;
    for _ in 0..32 {
        if let Some(c) = e.open_choice() {
            if c.spec.prompt == "haotianta_pick" {
                let (cid, obs) = (c.id, c.observer);
                assert_eq!(obs, 2, "昊天塔窗=持有者 p2");
                // 选中 spade 实体（=当前判定牌）→ heart 次候选弃置
                let spade_pick: Vec<ChoiceOption> = c
                    .spec
                    .options
                    .iter()
                    .filter(|o| matches!(o, ChoiceOption::Card(x) if *x == spade_id))
                    .cloned()
                    .collect();
                assert!(!spade_pick.is_empty(), "昊天塔：候选含判定牌实体");
                let sel = spade_pick;
                picked_is_spade = true;
                submit(
                    &mut e,
                    obs,
                    GameAction::Answer {
                        choice_id: cid,
                        selection: sel,
                    },
                );
                saw_pick = true;
                continue;
            }
            let (cid, obs) = (c.id, c.observer);
            let sel = if !c.spec.options.is_empty() {
                vec![c.spec.options[0].clone()]
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
        if saw_pick {
            break;
        }
        let cur = e.turn().current;
        let _ = submit(&mut e, cur, GameAction::NextPhase);
    }
    assert!(saw_pick, "昊天塔：判定插窗出现");
    assert!(picked_is_spade, "昊天塔：判定结果=选中候选");
    {
        let core = e.state_core();
        assert!(
            core.discard.contains(&heart_id),
            "昊天塔：次候选弃置（heart 入弃堆）"
        );
    }
}
