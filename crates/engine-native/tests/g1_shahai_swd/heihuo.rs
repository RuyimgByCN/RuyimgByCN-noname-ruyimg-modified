use super::*;

/// T12 swd deferred 消解批（黑火 heihuo/真威 fzhenwei/遁行 dunxing）：
/// 三技行为面 + 反例。
#[test]
fn swd_deferred_b92_heihuo_fzhenwei_dunxing() {
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
    let mut e = Engine::new_with_extensions(62049, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 黑火（齐）：弃装备牌 → 手牌加倍；<8 无伤面（≥8 爆伤面反例留待
    // 大手牌布阵）
    e.debug_grant(0, "heihuo");
    let eq1 = {
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
    for _ in 0..2 {
        let _ = e.debug_place(0, "tao");
    }
    let h_before = e.debug_hand(0).len(); // =3（含装备牌）
    let def_hh = e.catalog().skill_by_name("heihuo").expect("黑火在目录");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_hh });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "黑火发动 {out:?}"
    );
    let c = e.open_choice().expect("黑火弃牌窗");
    assert_eq!(c.spec.prompt, "heihuo_discard");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(eq1)],
        },
    );
    settle_optional_confirm(&mut e);
    // 弃装备后手牌 2 → 摸 2 → 共 4（<8 无伤）
    assert_eq!(
        e.debug_hand(0).len(),
        h_before - 1 + (h_before - 1),
        "黑火：手牌加倍（2→4）"
    );
    assert_eq!(e.debug_hp(0), 4, "黑火：<8 无火伤");
    // 反例：每回合限一次
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_hh });
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "黑火反例：再发动拒 {out:?}"
    );

    // ② 真威（凤钰）：自己回合内他人响应牌 → 交给另一角色
    e.debug_grant(0, "fzhenwei");
    // p1 出杀 p2（目标空闪——响应无？响应牌=杀结算中目标出闪）：p2 出闪
    // 响应 → 闪入弃堆 → 真威窗（当前回合=p0? 需 p0 回合内他人响应——
    // p0 回合中 p2 出闪即满足）
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(1, 4);
    let sha1 = e.debug_place(0, "sha");
    let shan1 = e.debug_place(1, "shan");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![1],
        },
    );
    let c = e.open_choice().expect("p1 出闪窗");
    let (cid, obs) = (c.id, c.observer);
    let shan_sel: Vec<ChoiceOption> = c
        .spec
        .options
        .iter()
        .filter(|o| matches!(o, ChoiceOption::Card(x) if *x == shan1))
        .cloned()
        .collect();
    assert!(!shan_sel.is_empty(), "②：闪在候选");
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: shan_sel,
        },
    );
    // 真威确认窗 → 确认 → 目标窗选 p3（≠p2）
    let c = e.open_choice().expect("真威确认窗");
    assert_eq!((c.spec.prompt, c.observer), ("trigger_confirm", 0));
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let c = e.open_choice().expect("真威目标窗");
    assert_eq!(c.spec.prompt, "fzhenwei_target");
    assert!(
        !c.spec.options.contains(&ChoiceOption::Player(1)),
        "真威：响应者 p1 不在候选"
    );
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(3)],
        },
    );
    settle_optional_confirm(&mut e);
    assert!(e.debug_hand(3).contains(&shan1), "真威：响应牌闪交给 p3");

    // ③ 遁行（虎魄）：成为他人卡牌目标 → 弃一张判定非红桃 → 取消
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(1, "dunxing");
    let cost1 = e.debug_place(1, "tao");
    let sha2 = e.debug_place(0, "sha");
    e.debug_set_hp(1, 4);
    // 判定黑桃（非红桃→取消）
    e.debug_set_deck_top_by(|def| def.suit.is_spade());
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![1],
        },
    );
    // 遁行确认窗（p1）→ 确认 → 弃牌窗
    let c = e.open_choice().expect("遁行确认窗");
    assert_eq!((c.spec.prompt, c.observer), ("trigger_confirm", 1));
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let c = e.open_choice().expect("遁行弃牌窗");
    assert_eq!(c.spec.prompt, "dunxing_discard");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(cost1)],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 4, "遁行：非红桃判定 → 杀被取消（无伤）");
    assert!(!e.debug_hand(1).contains(&cost1), "遁行：代价牌已弃");
    assert!(
        e.debug_hand(0).contains(&sha2),
        "遁行：被取消的杀归还使用者手牌"
    );
}
