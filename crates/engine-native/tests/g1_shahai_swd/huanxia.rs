use super::*;

/// T12 swd 武将面批 27：还侠（屈娴 tag 287）——viewAs 红牌当杀（武圣
/// 同族转化臂）；未造成伤害的转化杀在持有者阶段末从弃牌堆收回
/// （TurnState.huanxia_track 跟踪+commit_damage 置位+commit_phase 回收
/// 钩；登记差异上游经特殊区暂存）。
#[test]
fn swd_generals_b27_huanxia_viewas_reclaim() {
    use engine_native::catalog::Suit;
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
    let mut e = Engine::new_with_extensions(62010, 4, &[&pack]).expect("建局");
    e.debug_grant(0, "huanxia");
    advance_to_play(&mut e, 0);

    // 正例 ①：红闪当杀命中 → 已造成伤害 → 阶段末不收回
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let red_shan = {
        let id = e.debug_set_deck_top_by(|d| {
            d.kind == engine_native::catalog::CardKind::Shan
                && matches!(d.suit, Suit::Heart | Suit::Diamond)
        });
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.players[0].hand.push(id);
        id
    };
    e.debug_set_hp(1, 8);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: red_shan,
            targets: vec![1],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 7, "还侠：红闪当杀造成 1 伤");
    submit(&mut e, 0, GameAction::NextPhase); // Play→Discard（Play 末触发回收检查）
    settle_optional_confirm(&mut e);
    assert_eq!(hand_len_w(&e, 0), 0, "还侠：已造成伤害 → 不收回");
    assert!(
        e.state_core().discard.contains(&red_shan),
        "还侠：红闪在弃牌堆"
    );

    // 正例 ②：红桃当杀被闪避 → 未造成伤害 → Play 末收回
    {
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("重置");
    }
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let red_tao = e.debug_place(0, "tao");
    let _shan1 = e.debug_place(1, "shan");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: red_tao,
            targets: vec![1],
        },
    );
    settle_optional_confirm(&mut e); // 闪响应窗首选项=Pass → 闪避
    assert_eq!(e.debug_hp(1), 7, "还侠②：被闪避无伤");
    submit(&mut e, 0, GameAction::NextPhase);
    settle_optional_confirm(&mut e);
    assert!(
        e.debug_hand(0).contains(&red_tao),
        "还侠：未造成伤害 → 阶段末收回红桃"
    );

    // 反例 ③：无还侠时红闪不可当杀（转化缺位被拒；标准牌堆闪全红，
    // 黑闪不存在故以撤技面为转化反例）
    advance_to_play(&mut e, 0);
    {
        let def_hx0 = e.catalog().skill_by_name("huanxia").expect("还侠 def");
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        snap.core.skills.remove(&(0, def_hx0));
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("撤销还侠(③)");
    }
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let red_shan3 = {
        let id = e.debug_set_deck_top_by(|d| {
            d.kind == engine_native::catalog::CardKind::Shan
                && matches!(d.suit, Suit::Heart | Suit::Diamond)
        });
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != id);
        core.players[0].hand.push(id);
        id
    };
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: red_shan3,
            targets: vec![1],
        },
    );
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "还侠反例：无技红闪不当杀 {out:?}"
    );

    // 反例 ④：无还侠者红桃照常为桃（不转杀）
    {
        let def_hx = e.catalog().skill_by_name("huanxia").expect("还侠 def");
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        snap.core.skills.remove(&(0, def_hx));
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("撤销还侠");
    }
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let tao4 = e.debug_place(0, "tao");
    e.debug_set_hp(0, 3);
    let hp1 = e.debug_hp(1);
    let _out4 = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: tao4,
            targets: vec![],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(0), 4, "还侠反例：无技时红桃照常回血");
    assert_eq!(e.debug_hp(1), hp1, "还侠反例：p1 不受伤");
}
