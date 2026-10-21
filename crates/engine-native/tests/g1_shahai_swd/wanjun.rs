use super::*;

/// T12 swd 武将面批㉖（浊山铸万钧+隳延复用/莫耶森罗/夜鸦熙火扑；热血/
/// 流离/神焰/竹海/击空 deferred）：三行为行为面 + 反例。
#[test]
fn swd_generals_b66_wanjun_senluo_huopu() {
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
    let mut e = Engine::new_with_extensions(62034, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 万钧（浊山铸）：装备牌当南蛮入侵——全体无杀各受 1
    e.debug_grant(0, "wanjun");
    let eq = {
        let id = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                e.catalog().card_kind(*c).equip_slot()
                    == Some(engine_native::catalog::EquipSlot::HorsePlus)
            })
            .expect("坐骑在目录");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.discard.retain(|x| *x != id);
        core.table.retain(|x| *x != id);
        for pl in core.players.iter_mut() {
            pl.hand.retain(|x| *x != id);
        }
        core.players[0].hand.push(id);
        id
    };
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);
    e.debug_set_hp(3, 4);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: eq,
            targets: vec![],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 3, "万钧：p1 无杀受 1");
    assert_eq!(e.debug_hp(2), 3, "万钧：p2 无杀受 1");
    assert_eq!(e.debug_hp(3), 3, "万钧：p3 无杀受 1");
    assert!(
        !e.debug_hand(0).contains(&eq) && !e.debug_equips(0).contains(&eq),
        "万钧：装备牌已消耗"
    );
    // 反例：南蛮不可手动指定目标
    let eq2 = {
        let id = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                e.catalog().card_kind(*c).equip_slot()
                    == Some(engine_native::catalog::EquipSlot::Weapon)
            })
            .expect("武器在目录");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.discard.retain(|x| *x != id);
        core.table.retain(|x| *x != id);
        for pl in core.players.iter_mut() {
            pl.hand.retain(|x| *x != id);
        }
        core.players[0].hand.push(id);
        id
    };
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: eq2,
            targets: vec![1],
        },
    );
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "万钧反例：手动目标拒 {out:?}"
    );

    // ② 火扑（夜鸦熙）：红桃牌当流星火雨——目标 he 不足两张直接 1 火伤
    e.debug_grant(0, "huopu");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let heart = {
        let h = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                e.catalog().card_kind(*c) == engine_native::catalog::CardKind::Tao
                    && matches!(
                        e.catalog().cards[*c as usize].suit,
                        engine_native::catalog::Suit::Heart
                    )
            })
            .expect("红桃桃在目录");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != h);
        core.discard.retain(|x| *x != h);
        core.table.retain(|x| *x != h);
        for pl in core.players.iter_mut() {
            pl.hand.retain(|x| *x != h);
        }
        core.players[0].hand.push(h);
        h
    }; // 红桃桃（火扑门=Heart）
    e.debug_set_hp(1, 4);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: heart,
            targets: vec![1],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 3, "火扑：流星火雨 he<2 直接 1 火伤");
    // 反例：非红桃不作火扑转化——黑桃杀仍按杀结算（有闪则出闪窗）
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(1, 4);
    let black_sha = {
        let bs = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                e.catalog().card_kind(*c) == engine_native::catalog::CardKind::Sha
                    && matches!(
                        e.catalog().cards[*c as usize].suit,
                        engine_native::catalog::Suit::Spade
                    )
            })
            .expect("黑杀在目录");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != bs);
        core.discard.retain(|x| *x != bs);
        core.table.retain(|x| *x != bs);
        for pl in core.players.iter_mut() {
            pl.hand.retain(|x| *x != bs);
        }
        core.players[0].hand.push(bs);
        bs
    };
    let shan_def = e.debug_place(1, "shan");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: black_sha,
            targets: vec![1],
        },
    );
    let c = e.open_choice().expect("反例：黑杀正常出闪窗");
    assert_eq!((c.spec.prompt, c.observer), ("respond_shan", 1));
    // options=[Pass]+闪——选闪实体（防顺序歧义）
    let shan_sel: Vec<ChoiceOption> = c
        .spec
        .options
        .iter()
        .filter(|o| matches!(o, ChoiceOption::Card(x) if *x == shan_def))
        .cloned()
        .collect();
    assert!(!shan_sel.is_empty(), "反例：闪在候选");
    let (cid, obs) = (c.id, c.observer);
    let out_fl = submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: shan_sel,
        },
    );
    eprintln!(
        "DBG fanli ans={out_fl:?} hp1={} shan_in_hand={}",
        e.debug_hp(1),
        e.debug_hand(1).contains(&shan_def)
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 4, "反例：黑杀按杀结算被闪");

    // ③ 森罗（莫耶）：手牌全场最少→其他有手牌角色各弃 2 摸 1
    e.debug_grant(0, "senluo");
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // p0 手牌 1（并列最少）；p1 手牌 4；p2/p3 各 1（≥p0）
    let keep0 = e.debug_place(0, "tao");
    let _filler2 = e.debug_place(2, "tao");
    let _filler3 = e.debug_place(3, "tao");
    let a1 = e.debug_place(1, "tao");
    let a2 = e.debug_place(1, "shan");
    let a3 = e.debug_place(1, "tao");
    let a4 = e.debug_place(1, "shan");
    let def_sl = e.catalog().skill_by_name("senluo").expect("森罗在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_sl });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "森罗发动 {out:?}"
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hand(1).len(), 3, "森罗：p1 弃 2 摸 1（4→3）");
    assert!(
        !e.debug_hand(1).contains(&a1) && !e.debug_hand(1).contains(&a2),
        "森罗：确定性首两张被弃"
    );
    assert_eq!(e.debug_hand(0).len(), 1, "森罗：p0 手牌不变");
    let _ = (keep0, a3, a4);
    // 反例：手牌非全场最少 → 发动拒
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _k = e.debug_place(0, "tao");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_sl });
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "森罗反例：非最少拒（其余空手）{out:?}"
    );
}
