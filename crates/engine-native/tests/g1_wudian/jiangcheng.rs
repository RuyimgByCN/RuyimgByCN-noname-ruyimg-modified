use super::*;

// ---------------------------------------------------------------------------
// U05 xianjian 第十六切片（规则 v157）：姜城（pal_jiangcheng）焚逝
//（fenshi，濒死救回强制觉醒一次）+龙火（longhuo，自身阶段结束确认，
// 焚逝获得后永久）——觉醒 +2 护甲、摸 2、获得龙火；龙火对全场按座位
// 序各 1 火伤。yanzhan 系（viewAs/mod）登记暂缓。
// ---------------------------------------------------------------------------

/// 焚逝：濒死救回→觉醒（+2 甲+摸 2+龙火 flag）；二次濒死不再触发。
#[test]
fn jiangcheng_fenshi_awaken_once_on_rescue() {
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

    let mut e = Engine::new_with_extensions(10017, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "fenshi");
    e.debug_set_hp(0, 1);
    let tao = e.debug_place(0, "tao");

    // ① 直伤 1→0 号濒死→0 号先被问（current=0）→用桃救回→焚逝觉醒
    e.debug_deal_damage(Some(1), 0, 1, engine_native::event::DamageNature::Normal);
    let c = e.open_choice().expect("濒死求桃挂起");
    assert_eq!(c.observer, 0, "current=0 → 0 号先答");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(tao)],
        },
    );
    assert_eq!(e.debug_hp(0), 1, "桃救回（0→1）");
    assert_eq!(e.state_core().player(0).hujia, 2, "焚逝：+2 护甲");
    assert_eq!(e.debug_hand(0).len(), 2, "焚逝：摸 2（桃已耗）");
    assert!(
        e.state_core().player(0).jiangcheng_longhuo,
        "龙火获得（flag 置位）"
    );

    // ② 二次濒死：伤 3（甲 2 吸 2 → 1→0 濒死）→再救→焚逝不重触发
    let tao2 = e.debug_place(0, "tao");
    e.debug_deal_damage(Some(1), 0, 3, engine_native::event::DamageNature::Normal);
    let c = e.open_choice().expect("二次濒死求桃挂起");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(tao2)],
        },
    );
    assert_eq!(e.debug_hp(0), 1, "二次救回");
    assert_eq!(e.state_core().player(0).hujia, 0, "甲 2 吸 2 耗尽");
    assert_eq!(e.debug_hand(0).len(), 2, "觉醒一次：无补牌（3-1 桃=2）");
    assert!(e.state_core().player(0).jiangcheng_longhuo);
}

/// 龙火：焚逝觉醒后自身阶段结束确认→全场按座位序各 1 火伤（甲吸伤
/// 可见；含自己）。
#[test]
fn jiangcheng_longhuo_fires_at_own_phase_end() {
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

    let mut e = Engine::new_with_extensions(10018, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "fenshi");
    e.debug_grant(0, "longhuo");
    e.debug_set_hp(0, 1);
    let tao = e.debug_place(0, "tao");

    // 觉醒：直伤濒死→桃救回
    e.debug_deal_damage(Some(1), 0, 1, engine_native::event::DamageNature::Normal);
    let c = e.open_choice().expect("濒死求桃挂起");
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(tao)],
        },
    );
    assert!(e.state_core().player(0).jiangcheng_longhuo, "已觉醒");

    // 提血防连锁濒死；清手防弃牌门；推进到 0 号 End 阶段收束
    e.debug_set_hp(0, 3);
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);
    e.debug_clear_hand(0);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let out = submit(&mut e, 0, GameAction::NextPhase); // End→龙火确认挂起
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().expect("龙火确认挂起");
    assert_eq!(c.spec.prompt, "trigger_confirm");
    assert_eq!(c.observer, 0);
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );

    // 全场座位序各 1 火伤：0 号甲 2 吸 1（体力不变 3）；1/2 号 4→3
    assert_eq!(e.debug_hp(0), 3, "0 号：火伤被甲吸收");
    assert_eq!(e.state_core().player(0).hujia, 1, "甲 2→1");
    assert_eq!(e.debug_hp(1), 3, "1 号 4→3");
    assert_eq!(e.debug_hp(2), 3, "2 号 4→3");
}
