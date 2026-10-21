use super::*;

/// T12 swd 武将面批㊱（少微焰羽/伽兰多摄引/齐破茧；jianji/mailun/
/// yaotong/heihuo deferred）：三将行为面 + 反例。
#[test]
fn swd_generals_b76_huangyu_xianyin_pojian() {
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
    let mut e = Engine::new_with_extensions(62044, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 焰羽（少微）：弃两张红色牌 → 虚拟炽羽袭（全体其他）
    e.debug_grant(0, "huangyu");
    let r1 = {
        let r = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                matches!(
                    e.catalog().cards[*c as usize].suit,
                    engine_native::catalog::Suit::Heart
                ) && e.catalog().card_kind(*c) == engine_native::catalog::CardKind::Tao
            })
            .expect("红桃桃在目录");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != r);
        core.players[0].hand.push(r);
        r
    };
    let r2 = {
        let r = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                matches!(
                    e.catalog().cards[*c as usize].suit,
                    engine_native::catalog::Suit::Diamond
                ) && e.catalog().card_kind(*c) != engine_native::catalog::CardKind::Tao
                    && e.state_core().deck.contains(c)
            })
            .expect("方块牌在牌库");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != r);
        core.players[0].hand.push(r);
        r
    };
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);
    e.debug_set_hp(3, 4);
    let def_hy = e.catalog().skill_by_name("huangyu").expect("焰羽在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_hy });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "焰羽发动 {out:?}"
    );
    let c = e.open_choice().expect("焰羽弃牌窗");
    assert_eq!(c.spec.prompt, "huangyu_discard");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(r1), ChoiceOption::Card(r2)],
        },
    );
    // 炽羽袭 AOE：逐目标闪或 1 火伤——全体空手各受 1
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 3, "焰羽：炽羽袭 p1 受 1 火伤");
    assert_eq!(e.debug_hp(2), 3, "焰羽：炽羽袭 p2 受 1 火伤");
    assert_eq!(e.debug_hp(3), 3, "焰羽：炽羽袭 p3 受 1 火伤");

    // ② 摄引（伽兰多）：弃尽判定区有牌者的判定牌 + 令其交一张手牌
    e.debug_grant(0, "xianyin");
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // p1 判定区置乐 + 手牌 2
    let le_j = e.debug_place_judge(1, "le");
    let v_hand = e.debug_place(1, "tao");
    let def_xy = e.catalog().skill_by_name("xianyin").expect("摄引在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_xy });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "摄引发动 {out:?}"
    );
    settle_optional_confirm(&mut e);
    assert!(
        !e.state_core().player(1).judge.contains(&le_j),
        "摄引：判定区乐被弃"
    );
    assert!(
        !e.debug_hand(1).contains(&v_hand),
        "摄引：p1 交出一张手牌（确定性首张）"
    );
    assert!(e.debug_hand(0).contains(&v_hand), "摄引：手牌入己方");

    // ③ 破茧（齐）：手牌空后弃置 → 牌堆首张装备牌装备之
    e.debug_grant(0, "pojian");
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // p1 连续过河拆桥拆空 p0 → CardsDiscarded 触发破茧
    advance_to_play(&mut e, 1);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 清空牌堆/弃堆全部装备实体（防摸牌抽走目标坐骑），eq_top 留原区
    let eq_top = {
        let id = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                e.catalog().card_kind(*c).equip_slot()
                    == Some(engine_native::catalog::EquipSlot::HorsePlus)
            })
            .expect("坐骑在目录");
        let all: Vec<u32> = e
            .state_core()
            .deck
            .iter()
            .chain(e.state_core().discard.iter())
            .copied()
            .filter(|c| e.catalog().card_kind(*c).equip_slot().is_some() && *c != id)
            .collect();
        let core = e.state_core_mut();
        for x in all {
            core.deck.retain(|y| *y != x);
            core.discard.retain(|y| *y != x);
        }
        id
    };
    // 清手后再布 p0 两张（拆桥目标牌）与 p1 拆桥
    let only = e.debug_place(0, "tao");
    let extra = e.debug_place(0, "shan");
    let dis1 = {
        let d = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                e.catalog().card_kind(*c) == engine_native::catalog::CardKind::Dismantle
                    && e.state_core().deck.contains(c)
            })
            .or_else(|| {
                (0..e.catalog().cards.len() as u32).find(|c| {
                    e.catalog().card_kind(*c) == engine_native::catalog::CardKind::Dismantle
                        && e.state_core().discard.contains(c)
                })
            })
            .expect("拆桥在牌堆/弃堆");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != d);
        core.discard.retain(|x| *x != d);
        core.players[1].hand.push(d);
        d
    };
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: dis1,
            targets: vec![0],
        },
    );
    // DismantleMachine 自动弃目标首张（无选牌窗）
    settle_optional_confirm(&mut e);
    let dis2 = {
        let d = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                e.catalog().card_kind(*c) == engine_native::catalog::CardKind::Dismantle
                    && (e.state_core().deck.contains(c) || e.state_core().discard.contains(c))
            })
            .expect("第二张拆桥");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != d);
        core.discard.retain(|x| *x != d);
        core.players[1].hand.push(d);
        d
    };
    advance_to_play(&mut e, 1);
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: dis2,
            targets: vec![0],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hand(0).len(), 0, "破茧：p0 手牌拆空");
    assert!(
        e.debug_equips(0).iter().any(|x| *x == eq_top),
        "破茧：牌堆装备已装备"
    );
    let _ = (only, extra);
}
