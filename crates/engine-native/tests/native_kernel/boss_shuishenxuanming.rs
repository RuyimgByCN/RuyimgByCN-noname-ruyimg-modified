use super::*;

/// 星宿阶段2系自润片（规则 v247）：自润（玄冥）+玄冥条目——①准备阶段
/// 全体摸牌（有装备 2/无装备 1，含自身）②造异玄冥半门激活：玄冥在场
/// 颛顼锦囊不可指定敌方（基本牌不受此门限）③非造异持有者不受门控。
/// 玄冥条目（群目录近似/hp5/female/[神裔,自润]；玄麟3 已处置）。
#[test]
fn boss_shuishenxuanming_zirun_draw_all() {
    // 建局：1=玄冥；2=颛顼（造异持有者）
    let mut e = Engine::new_boss_stage(67001, 4, 1, 3, 6, 4, 0);
    let xm_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 24) as u16; // BOSS_GENERALS[24] 玄冥
    let zx_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 23) as u16; // BOSS_GENERALS[23] 颛顼
    e.debug_set_general_def(1, xm_def);
    e.debug_set_general_def(2, zx_def);
    e.debug_set_hp(1, 5);
    assert_eq!(
        e.catalog().general_meta(xm_def).map(|g| g.0),
        Some("玄冥"),
        "玄冥入目录"
    );
    let zr = e.catalog().skill_by_name("自润").expect("自润入目录");
    let zx = e.catalog().skill_by_name("造异").expect("造异入目录");
    assert!(e.state_core().skills.contains_key(&(1, zr)));
    assert!(e.state_core().skills.contains_key(&(2, zx)));

    // ① 自润摸牌非对称：给 0 号装备；各清手后推进到 1 号判定锚
    let armor = e.debug_place(0, "armor");
    {
        let core = e.state_core_mut();
        let pos = core.players[0]
            .hand
            .iter()
            .position(|c| *c == armor)
            .expect("armor in hand");
        let c = core.players[0].hand.remove(pos);
        core.players[0].equips.push(c);
    }
    for t in [0u8, 1, 2, 3] {
        e.debug_clear_hand(t);
    }
    // 推进到 1 号（玄冥）判定锚后（自润已触发）：0 号先手已清→摸 2
    for _ in 0..12 {
        if e.turn().current == 1 && e.turn().phase == Phase::Draw {
            break;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { choice } => {
                if choice.spec.prompt == "dying_save" {
                    answer(&mut e, ChoiceOption::Pass);
                    drain_choices(&mut e);
                } else {
                    answer_cards(&mut e, &choice);
                }
            }
            other => panic!("{other:?}"),
        }
    }
    // 0 号=3（回合摸 2+神恩 1）+自润 2=5；2/3 未到回合=自润 1
    assert_eq!(e.debug_hand(0).len(), 5, "自润：有装备 +2（回合摸 3 后）");
    assert_eq!(e.debug_hand(2).len(), 1, "自润：无装备摸 1");
    assert_eq!(e.debug_hand(3).len(), 1, "自润：无装备摸 1");
    assert_zone_conservation(&e);

    // ② 造异玄冥半门：玄冥在场——颛顼锦囊不可指定敌方；基本牌不受玄冥
    //    门限（共工不在场）。颛顼（2 号）出牌阶段验证
    advance_to_play(&mut e, 2);
    e.debug_clear_hand(2);
    let juedou = e.debug_place(2, "duel");
    let out = play(&mut e, juedou, vec![1]);
    assert!(
        !matches!(out, CommandOutcome::Accepted { .. }),
        "造异玄冥半门：颛顼锦囊不可指定敌方（玄冥在场）"
    );
    let sha = e.debug_place(2, "sha");
    let out = play(&mut e, sha, vec![1]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "玄冥半门不限基本牌 {out:?}"
    );
    drain_choices(&mut e);

    // ③ 非造异持有者不受门控：3 号锦囊→敌方可指定（3 的敌方=boss 方
    //    玄冥）——换 3 号出牌阶段
    next_phase(&mut e); // 2 Play→Discard
    drain_choices(&mut e);
    next_phase(&mut e); // Discard→End
    drain_choices(&mut e);
    advance_to_play(&mut e, 3);
    e.debug_clear_hand(3);
    let juedou2 = e.debug_place(3, "duel");
    let out = play(&mut e, juedou2, vec![1]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "非造异持有者不受门控 {out:?}"
    );
    drain_choices(&mut e);
    assert_zone_conservation(&e);

    // ④ 存档往返
    let snap = e.save();
    let r = Engine::restore(&snap).expect("往返");
    assert!(r.state_core().skills.contains_key(&(1, zr)), "自润实例往返");
    assert!(r.state_core().skills.contains_key(&(2, zx)), "造异实例往返");
    assert_zone_conservation(&r);
}
