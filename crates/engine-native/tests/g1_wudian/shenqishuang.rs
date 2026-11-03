use super::*;

// ---------------------------------------------------------------------------
// U05 xianjian 第二切片（规则 v143）：沈欺霜（pal_shenqishuang）冰心
//（binxin，global 阶段结束确认）——任何体力为 1 的角色阶段结束时令其
// +1 护甲。qixia（额外回合缺口）/jianzhen（chooseToUse 链）登记暂缓。
// ---------------------------------------------------------------------------

/// 冰心：体力 1 者阶段结束→确认→+1 甲；非体力 1 不触发。
#[test]
fn shenqishuang_binxin_grants_hujia_at_low_hp() {
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

    let mut e = Engine::new_with_extensions(9999, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "binxin");
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 4); // 先满血（避免早期阶段结束触发）

    // 0 号收束（体力 4 不触发）→ 1 号 Judge/Draw/Play——进入弃牌阶段
    // 前把 1 号压到 1 血。A2 H80：冰心已迁整回合结束窗 phaseEnd 段（上游
    // global phaseEnd＝任意玩家整回合结束）——确认门在 1 号整回合结束时
    // 挂起（observer=0）→Confirm →1 号 +1 甲
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play→Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // 0 收束→1 Judge
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Judge→Draw
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Draw→Play
    e.debug_set_hp(1, 1);
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Play→Discard（弃牌门挂起）
    let hand1 = e.debug_hand(1);
    let out = answer_open(&mut e, vec![ChoiceOption::Card(hand1[0])]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 弃牌结算后推进 Discard→End→整回合结束窗，冰心确认门在窗内挂起
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Discard→End
    let _ = submit(&mut e, 1, GameAction::NextPhase); // End→整回合结束窗
    let c = e.open_choice().cloned();
    let c = c.expect("冰心确认门");
    assert_eq!(c.observer, 0, "确认门 observer=持有者");
    drop(c);
    let out = answer_open(&mut e, vec![ChoiceOption::Confirm]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.save().core.players[1].hujia, 1, "冰心 +1 护甲");
}
