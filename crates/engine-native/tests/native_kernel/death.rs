use super::*;

/// 死亡结算清判定区：判定区延时锦囊随死亡入弃牌堆。
#[test]
fn death_clears_judge_zone() {
    let mut e = Engine::new(179, 4);
    e.debug_set_hp(1, 1);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _le = e.debug_place_judge(1, "le");
    let sha = e.debug_place(0, "sha");
    let discard_before = e.debug_discard_len();
    let out = play(&mut e, sha, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(!e.debug_alive(1));
    assert!(e.debug_judge(1).is_empty(), "死亡应清判定区");
    assert_eq!(
        e.debug_discard_len(),
        discard_before + 2,
        "判定区 1 张牌 + 造成死亡的杀 1 张应入弃牌堆（手牌已清空）"
    );
}
