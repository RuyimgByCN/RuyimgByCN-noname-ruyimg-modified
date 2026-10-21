use super::*;

// ---------------------------------------------------------------------------
// U05 ow 纵切批十八（规则 v135）：半藏（ow_banzang）音波（yinbo，出牌
// 阶段限一次）——弃 1 张黑桃，随机 ≤3 名有牌的其他角色各随机弃 1 张。
// bfengshi（随机直击+组技）登记暂缓。登记差异：原型无阵营——enemies=
// 其他存活玩家。
// ---------------------------------------------------------------------------

/// 音波：黑桃弃牌发动；全场其他 4 人有牌→随机 3 名各弃 1；无一有牌
/// 时不弹目标阶段直接结束（候选空）。
#[test]
fn banzang_yinbo_random_discard_three() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9991, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "yinbo");
    // 0 号黑桃弃牌素材
    let mut spade = None;
    for _ in 0..8 {
        e.debug_place(0, "sha");
        if let Some(c) = e
            .debug_hand(0)
            .into_iter()
            .find(|c| e.catalog().cards[*c as usize].suit == engine_native::catalog::Suit::Spade)
        {
            spade = Some(c);
            break;
        }
    }
    let sp = spade.expect("应有黑桃杀");
    // 其他 4 人各 1 张牌（候选全量）
    for p in 1..5 {
        e.debug_place(p, "tao");
    }
    let total_before: usize = (1..5).map(|p| e.debug_hand(p).len()).sum();

    // 发动：弃黑桃 → 随机 3 名各弃 1
    let def_yb = e.catalog().skill_by_name("yinbo").expect("def");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_yb });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(sp)]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert!(e.state_core().discard.contains(&sp), "音波弃牌入弃牌堆");
    let total_after: usize = (1..5).map(|p| e.debug_hand(p).len()).sum();
    assert_eq!(
        total_before - total_after,
        3,
        "随机 3 名各弃 1（4 人全有牌）"
    );
    // usable:1
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: def_yb });
    assert!(
        matches!(
            &out,
            CommandOutcome::Rejected(engine_native::RejectReason::SkillAlreadyUsed)
        ),
        "音波每回合限一次：{out:?}"
    );
}
