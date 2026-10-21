use super::*;

// ---------------------------------------------------------------------------
// U05 xianjian 第二十切片（规则 v161）：云天河（pal_yuntianhe，wu 4 男）
// 逐月（zhuyue，出牌阶段限一次主动）+贯日（guanri，limited 一次性主
// 动）——弃非基本牌令至多两名有牌其他角色各随机弃一张，本出牌阶段杀
// 须以其为目标（距离豁免）；弃两红手牌失 1 体力对体力≥自己者 2 火伤
// 并弃其装备区全部牌。longxi 响应链缺口登记暂缓。
// ---------------------------------------------------------------------------

/// 逐月：弃非基本牌→两名目标各随机弃→杀须指定白名单（距离豁免）→
/// 出牌阶段结束白名单清空。
#[test]
fn yuntianhe_zhuyue_whitelist_and_clear() {
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

    let mut e = Engine::new_with_extensions(10025, 4, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "zhuyue");
    for p in 0..4 {
        e.debug_set_hp(p, 4);
    }
    let nb = e.debug_place(0, "wuzhong"); // 非基本弃牌
    let d1 = e.debug_place(1, "tao"); // 目标 1 被随机弃候选
    let _d2 = e.debug_place(2, "tao"); // 目标 2 被随机弃候选

    // 激活：弃 wuzhong → 指定 1、2 号
    let def = e.catalog().skill_by_name("zhuyue").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(nb)]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let c = e.open_choice().expect("逐月目标挂起");
    assert_eq!(
        c.spec.prompt,
        engine_native::content::host::WUDIAN_XJ_ZHUYUE_TARGETS
    );
    assert_eq!(c.spec.max, 2, "至多两名");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1), ChoiceOption::Player(2)],
        },
    );
    assert_eq!(e.debug_hand(1).len(), 0, "1 号随机弃 1（1 桃→0）");
    assert_eq!(e.debug_hand(2).len(), 0, "2 号随机弃 1（1 桃→0）");
    assert!(e.state_core().discard.contains(&d1), "1 号弃牌入弃堆");
    assert_eq!(e.state_core().turn.zhuyue_targets, vec![1, 2], "白名单装填");

    // 杀须指定白名单：对 3 号拒绝；对 2 号距离豁免放行（0→2 座位距 2）
    e.debug_place(0, "sha");
    let sha = e
        .debug_hand(0)
        .into_iter()
        .find(|c| e.catalog().cards[*c as usize].kind == CardKind::Sha)
        .expect("应有杀");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![3],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "非白名单拒绝：{out:?}"
    );
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![2],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "白名单+距离豁免放行：{out:?}"
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
    assert_eq!(e.debug_hp(2), 3, "杀照常命中（4→3）");

    // 出牌阶段结束白名单清空
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    assert!(e.state_core().turn.zhuyue_targets.is_empty(), "白名单清空");
}

/// 贯日：弃两红→失 1 体力→体力≥自己者 2 火伤→存活则弃其装备区全部；
/// 一次性（再发动拒绝）。
#[test]
fn yuntianhe_guanri_once_fire_and_equip_strip() {
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

    let mut e = Engine::new_with_extensions(10026, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    e.debug_grant(0, "guanri");
    let r1 = e.debug_place(0, "tao"); // 红
    let r2 = e.debug_place(0, "tao"); // 红
                                      // 1 号 hp=4≥4 且挂一件装备（将被全弃）
    let equip = e.debug_place(1, "shan");
    // 用 debug_grantEquip? 无——改用手牌弃候选：改为给 1 号两件不能直接装……
    // 简化：装备区断言改为「若有装备则全弃」——此处以 1 号手牌保留验证
    // 主体（火伤+失体力），装备全弃路径用 equips 直写布阵。
    {
        let mut snap = e.save();
        snap.core.players[1].equips.push(equip);
        snap.core.players[1].hand.retain(|c| *c != equip);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入");
    }

    // 激活：弃两红 → 指定 1 号
    let def = e.catalog().skill_by_name("guanri").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(r1), ChoiceOption::Card(r2)]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let c = e.open_choice().expect("贯日目标挂起");
    assert_eq!(
        c.spec.prompt,
        engine_native::content::host::WUDIAN_XJ_GUANRI_TARGET
    );
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    assert_eq!(e.debug_hp(0), 3, "失去 1 体力（4→3）");
    assert_eq!(e.debug_hp(1), 2, "2 火伤（4→2）");
    assert!(
        e.state_core().player(1).equips.is_empty(),
        "存活则装备区全弃"
    );
    assert!(e.state_core().discard.contains(&equip), "装备入弃堆");
    assert!(e.state_core().player(0).guanri_used, "一次性置位");

    // 再发动拒绝
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "一次性：{out:?}"
    );
}
