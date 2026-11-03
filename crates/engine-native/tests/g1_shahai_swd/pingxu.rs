use super::*;

/// T12 swd 武将面批 23：四将整将——平恤（秦叔宝 tag 278：全局距离字段
/// 技，computed_distance 消费：场上有持技者时攻击方无武器 -1/防守方无
/// 防具 +1）；还灵（轩辕剑侍 tag 279：自己阶段结束选一名其他角色同时翻
/// 面，取消=不发动）；义从/酒池经 builtin 白名单引用标准技（张高/程咬
/// 金整将，零新增行为）。
#[test]
fn swd_generals_b23_pingxu_huanling_builtin_four_full() {
    use engine_native::catalog::EquipSlot;
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
    assert!(pack.generals.len() >= 14, "14 将入册");
    // 5 人局（距离基线：对座 2）
    let mut e = Engine::new_with_extensions(62006, 5, &[&pack]).expect("建局");
    for n in ["pingxu", "huanling", "义从", "酒池"] {
        assert!(e.catalog().skill_by_name(n).is_some(), "{n} 在目录");
    }

    // ① 平恤：p3 持技（全局字段），d(0,2) 基线 2
    e.debug_grant(3, "pingxu");
    let dist = |e: &Engine, a: u8, b: u8| e.state_core().computed_distance(e.catalog(), a, b);
    assert_eq!(dist(&e, 0, 2), 2, "平恤：攻方无武器-1+守方无防具+1 净持平");
    // 守方装防具 → 仅 -1
    let armor = e.debug_set_deck_top_by(|d| d.kind.equip_slot() == Some(EquipSlot::Armor));
    {
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != armor);
        core.players[2].equips.push(armor);
    }
    assert_eq!(dist(&e, 0, 2), 1, "平恤：守方有防具 → 仅攻方无武器 -1");
    // 攻方装武器、卸守方防具 → 仅 +1（守方无防具支）
    let wep = e.debug_place(0, "zhuge");
    {
        let core = e.state_core_mut();
        core.players[0].hand.retain(|x| *x != wep);
        core.players[0].equips.push(wep);
        // 卸防具须回弃牌堆（直接 clear 会把牌逐出所有区域，restore 守卫拒）
        if let Some(pos) = core.players[2].equips.iter().position(|x| *x == armor) {
            let a = core.players[2].equips.remove(pos);
            core.discard.push(a);
        }
    }
    assert_eq!(dist(&e, 0, 2), 3, "平恤：攻方有武器 → 仅守方无防具 +1");
    // 反例：撤销持技者 → 字段失效回基线
    {
        let def_px = e.catalog().skill_by_name("pingxu").expect("平恤 def");
        let mut snap = e.save();
        snap.core.skills.remove(&(3, def_px));
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("撤销平恤");
    }
    assert_eq!(dist(&e, 0, 2), 2, "平恤反例：无持技者 → 基线 2");

    // ② 还灵（A2 H63）：上游 player phaseEnd＝整回合结束——确认后选 p1 →
    // 双翻面（先清手规避弃牌阶段 discard_to_limit 询问先于整回合结束窗）。
    // 推进 Play→Discard→End 到整回合结束窗（每回合仅此一窗）。
    advance_to_play(&mut e, 0);
    e.debug_grant(0, "huanling");
    e.debug_clear_hand(0);
    for _ in 0..3 {
        let out = submit(&mut e, 0, GameAction::NextPhase);
        assert!(matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ));
    }
    // 反例 ②a：整回合结束窗取消 → 不翻（原 Discard 末第二窗已随 A2 消失）
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("还灵确认窗");
    assert_eq!(obs, 0, "还灵窗归持有者");
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![],
        },
    );
    settle_optional_confirm(&mut e);
    assert!(
        !e.state_core().player(0).face_down && !e.state_core().player(1).face_down,
        "还灵反例：取消 → 不翻"
    );

    // ②b：p0 下一整回合结束窗确认 → 选 p1 → 双翻面（翻面后窗末即交接，
    // p1 跳过其回合并翻回——以日志 TurnOvered 证双翻面）
    next_p0_play_fn(&mut e);
    e.debug_clear_hand(0);
    for _ in 0..3 {
        let out = submit(&mut e, 0, GameAction::NextPhase);
        assert!(matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ));
    }
    // 可选触发门先确认，随后才是选人窗
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("还灵确认窗（第二轮）");
    assert_eq!(obs, 0, "还灵窗归持有者");
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
    let (cid, obs) = e
        .open_choice()
        .map(|c| (c.id, c.observer))
        .expect("还灵选人窗");
    let sel: Vec<ChoiceOption> = {
        let ch = e.open_choice().expect("still open");
        ch.spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Player(1)))
            .cloned()
            .collect()
    };
    assert_eq!(sel.len(), 1, "还灵目标面含 p1");
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: sel,
        },
    );
    settle_optional_confirm(&mut e);
    // 窗末即交接：p1 被翻面 → 其回合被跳过并翻回（face_down 已 false）。
    // 跳过本身即双翻面的证据（不翻面则轮至 1）。
    assert_eq!(
        e.turn().current, 2,
        "还灵：p1 被翻面跳过回合（窗末交接后翻回，轮至 2）"
    );
    assert!(e.state_core().player(0).face_down, "还灵：自己保持翻面");

    // 撤销还灵，回合收束不再有窗口
    {
        let def_hl = e.catalog().skill_by_name("huanling").expect("还灵 def");
        let mut snap = e.save();
        snap.core.skills.remove(&(0, def_hl));
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("撤销还灵");
    }

    // ③ builtin 酒池（程咬金 wiring）：黑桃手牌当酒
    advance_to_play(&mut e, 0);
    e.debug_grant(0, "酒池");
    e.debug_move_named_card_to_hand(0, "shandian");
    let spade = *e.debug_hand(0).last().unwrap();
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: spade,
            targets: vec![],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "酒池：黑桃当酒 {out:?}"
    );
    assert!(e.state_core().turn.jiu_used, "酒池：jiu_used 置位");
    e.debug_move_named_card_to_hand(0, "shandian");
    let spade2 = *e.debug_hand(0).last().unwrap();
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: spade2,
            targets: vec![],
        },
    );
    assert!(
        !matches!(out, CommandOutcome::Accepted { .. }),
        "酒池反例：第二坛拒"
    );

    // ④ builtin 义从（张高 wiring）：hp≤2 时距 -1/被算 +1
    e.debug_grant(1, "义从");
    e.debug_set_hp(1, 1);
    assert_eq!(dist(&e, 1, 3), 1, "义从：攻距 2-1（对座基线 2）");
    assert_eq!(dist(&e, 0, 1), 2, "义从：被算距离 1+1（hp≤2）");
    let _ = spade;
}
