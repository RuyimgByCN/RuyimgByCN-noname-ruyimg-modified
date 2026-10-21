use super::*;

// ---------------------------------------------------------------------------
// U05 xianjian 第二十七批切片（规则 v168）：南宫煌（pal_nangonghuang，
// wei 3 男）摄灵（sheling，CardUsed+CardResponded+CardsDiscarded 三事
// 件 frequent silent 每回合 3 次）——他人回合内牌入弃堆时回收之。
// zhaoyao（拼点时序偏差）/zhangmu（响应链）登记暂缓。
// ---------------------------------------------------------------------------

/// 摄灵：他人回合内弃牌→自动回收；自己回合内/自己弃牌不触发；每回合
/// 3 次上限；Judge begin 清零。
#[test]
fn nangonghuang_sheling_three_event_reclaim() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10037, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "sheling");

    // 0 号回合内：杀 1 号 → 1 号响应出闪（回合外角色打牌）→ 摄灵回收闪
    let d1 = e.debug_place(0, "sha");
    let shan = e.debug_place(1, "shan");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: d1,
            targets: vec![1],
        },
    );
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(shan)]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert!(
        e.state_core().player(0).hand.contains(&shan),
        "摄灵自动回收响应闪"
    );
    assert_eq!(e.state_core().turn.sheling_used, 1, "次数 +1");

    // 第二次（CardsDiscarded 事件面）：0 号用过河拆桥弃 1 号一张手牌
    // → 弃牌入弃堆 → 摄灵回收
    e.debug_clear_hand(1);
    let d3 = e.debug_place(1, "tao");
    let gq = e.debug_place(0, "dismantle");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: gq,
            targets: vec![1],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 拆桥弃牌目标侧询问（若有）→ 选 d3
    if let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Card(d3)],
            },
        );
    }
    assert!(
        e.state_core().player(0).hand.contains(&d3),
        "弃牌事件面：摄灵回收"
    );
    assert_eq!(e.state_core().turn.sheling_used, 2, "次数 +1");
}

// ---------------------------------------------------------------------------
// U05 xianjian 第三十批切片（规则 v172）：南宫煌照妖（zhaoyao，global
// Draw-begin 确认）——确定性首张拼点（舌战群儒同款登记差异）赢→摸
// 2+选两张手牌置牌堆顶（先选在上）。登记差异：原生触发在固定摸牌后。
// zhangmu 响应链暂缓。
// ---------------------------------------------------------------------------

/// 照妖：他人 Draw-begin→确定性拼点赢→摸 2+置顶两张（先选在上）。
#[test]
fn nangonghuang_zhaoyao_compare_and_top() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10043, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "zhaoyao");

    // 布阵保证赢：0 号首张点数 > 1 号首张点数（搜索压牌）
    let rank_of = |e: &Engine, c: u32| e.catalog().cards[c as usize].rank;
    let mut r1_card = None;
    'outer: for name in ["tao", "wuzhong", "shan", "sha", "jiu"] {
        let c = e.debug_place(1, name);
        if rank_of(&e, c) <= 5 {
            r1_card = Some(c);
            break 'outer;
        }
        e.debug_clear_hand(1);
    }
    let r1_card = r1_card.expect("找到低点数牌");
    let r1 = rank_of(&e, r1_card);
    let mut r0_card = None;
    for name in ["tao", "wuzhong", "shan", "sha", "jiu", "dismantle"] {
        let c = e.debug_place(0, name);
        if rank_of(&e, c) > r1 {
            r0_card = Some(c);
            break;
        }
        e.debug_clear_hand(0);
    }
    let r0_card = r0_card.expect("找到高点数牌");
    assert!(rank_of(&e, r0_card) > r1, "拼点必赢布阵");

    // 1 号摸牌阶段开始（0 过回合→1 Judge→Draw）：照妖比点赢→摸 2+询问
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw：照妖询问
    let c = e.open_choice().expect("照妖置顶询问挂起");
    assert_eq!(
        c.spec.prompt,
        engine_native::content::host::WUDIAN_XJ_ZHAOYAO_CARDS
    );
    assert_eq!((c.spec.min, c.spec.max), (2, 2));
    assert_eq!(c.observer, 0);
    let picks: Vec<ChoiceOption> = c.spec.options.iter().take(2).copied().collect();
    let top_expect = match &picks[0] {
        ChoiceOption::Card(x) => *x,
        _ => panic!("选项应为牌"),
    };
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: picks,
        },
    );
    // 摸 2-置 2=净 0；牌堆顶=先选者
    let deck_len = e.state_core().deck.len();
    assert_eq!(
        *e.state_core().deck.last().expect("非空"),
        top_expect,
        "先选在上"
    );
    let _ = deck_len;
    assert_eq!(e.debug_hand(0).len(), 1, "摸 2-置 2（首张保留）");
}

// ---------------------------------------------------------------------------
// U05 xianjian 第三十九批切片（规则 v181）：南宫煌（pal_nangonghuang）
// 张目（zhangmu，DamageBefore forced 每回合 1 次）——需要打出闪响应时
// 弃一张手牌闪直接视为已响应（杀不命中=伤害前置直防，登记差异：响应
// 询问改写简化；弃哪张闪确定性取首张闪）。
// ---------------------------------------------------------------------------

/// 张目：杀伤害前置弃首张闪直防（体力不变+闪离手）；每回合 1 次。
#[test]
fn nangonghuang_zhangmu_flash_discard_prevent() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10054, 2, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..2 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 6);
    }
    e.debug_grant(0, "zhangmu");
    let shan = e.debug_place(0, "shan");
    let sha = e.debug_place(1, "sha");
    // 0 号过回合（出牌阶段外杀才能指向他人）→ 1 号回合
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play

    // 1 号杀 0 号：张目前置直防（弃首张闪=shan，体力不变）
    let out = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha,
            targets: vec![0],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 闪响应 Pass → 张目 DamageBefore 直防
    while let Some(c) = e.open_choice() {
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
    assert_eq!(e.debug_hp(0), 6, "张目直防：体力不变");
    assert!(
        !e.state_core().player(0).hand.contains(&shan),
        "闪已弃（弃堆或已转化）"
    );
    assert!(e.state_core().turn.zhangmu_used, "回合位");

    // 同回合第二次杀：杀限次拒绝（1 号每回合一杀）
    let sha2 = e.debug_place(1, "sha");
    let out = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![0],
        },
    );
    assert!(
        matches!(&out, CommandOutcome::Rejected(r) if matches!(r, engine_native::RejectReason::ShaLimit)),
        "同回合再杀拒绝：{out:?}"
    );

    // 绕一圈回 1 号回合：张目回合位重置后再防一次
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 1, GameAction::NextPhase); // End→0 Judge
    for _ in 0..5 {
        let _ = submit(&mut e, 0, GameAction::NextPhase); // 0 号回合收束
    }
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    e.debug_clear_hand(1);
    e.debug_clear_hand(0);
    let sha3 = e.debug_place(1, "sha");
    let shan3 = e.debug_place(0, "shan");
    let _ = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha3,
            targets: vec![0],
        },
    );
    while let Some(c) = e.open_choice() {
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
    assert_eq!(e.debug_hp(0), 6, "新回合张目再防（两次均被防，6 不变）");
    assert!(
        !e.state_core().player(0).hand.contains(&shan3),
        "新回合闪再弃"
    );
}
