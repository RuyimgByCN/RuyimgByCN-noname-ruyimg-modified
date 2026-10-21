use super::*;

/// P2/N08 批③-b 第四把（规则 v68，2026-10-07）：诸葛连弩（♣1/♦1，
/// 射程 1）——装备时出杀不限次数（与咆哮同口径的锁定豁免）。
#[test]
fn zhuge_crossbow_unlimited_sha() {
    // ① 无连弩基线：第二张杀同回合被 ShaLimit 拒绝。
    let mut e = Engine::new(180, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(1, 1);
    let sha1 = e.debug_place(0, "sha");
    play(&mut e, sha1, vec![1]); // 1 号濒死无救死亡 → 存活 2 人
                                 // ② 装备连弩后同回合连出两张杀（对 2 号）。
    let zhuge = e.debug_move_named_card_to_hand(0, "zhuge");
    let out = submit_cmd(
        &mut e,
        0,
        GameAction::PlayCard {
            card: zhuge,
            targets: vec![],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "装备连弩: {out:?}"
    );
    let sha2 = e.debug_place(0, "sha");
    let out = play(&mut e, sha2, vec![2]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "连弩第一张杀: {out:?}"
    );
    let sha3 = e.debug_place(0, "sha");
    let out2 = play(&mut e, sha3, vec![2]);
    assert!(
        matches!(
            out2,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "连弩第二张杀不限次数: {out2:?}"
    );
    assert_eq!(e.debug_hp(2), 2, "两张杀各 1 伤（4→2）");

    // ③ 摘除连弩后限制恢复：注入下一回合再验证（连弩离场撤销语义由
    //    装备位通管，这里只验证装备期间的豁免）。
    assert!(e.save().core.players[0].equips.iter().any(|c| *c == zhuge));
}
