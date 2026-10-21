use super::*;

/// 群雄乱入董卓片（规则 v258）：酒池（黑桃手牌当酒 viewAs）+强征/爆敛
/// 混入已落（v255）——①黑桃手牌当酒（第二坛拒）②非黑桃不可当酒③强征
/// End 收归+爆敛 End 摸 2。董卓条目（shen→群目录近似/hp20）。
#[test]
fn boss_dongzhuo_jiuchi_viewas() {
    let mut e = Engine::new_boss_stage(79001, 4, 1, 3, 6, 4, 0);
    let dz_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 35) as u16; // BOSS_GENERALS[35] 董卓
    e.debug_set_general_def(1, dz_def);
    e.debug_set_hp(1, 20);
    assert_eq!(
        e.catalog().general_meta(dz_def).map(|g| g.0),
        Some("董卓"),
        "董卓入目录"
    );
    let jc = e.catalog().skill_by_name("酒池").expect("酒池入目录");
    let qz = e.catalog().skill_by_name("强征").expect("强征入目录");
    let bl = e.catalog().skill_by_name("爆敛").expect("爆敛入目录");
    assert!(e.state_core().skills.contains_key(&(1, jc)));
    assert!(e.state_core().skills.contains_key(&(1, qz)));
    assert!(e.state_core().skills.contains_key(&(1, bl)));

    // ① 黑桃手牌当酒（非杀牌——plain Sha 不进转化链）：闪电=黑桃1
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    e.debug_move_named_card_to_hand(1, "shandian");
    let spade = e.debug_hand(1)[0];
    let out = play(&mut e, spade, vec![]);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "酒池：黑桃当酒 {out:?}"
    );
    assert!(e.state_core().turn.jiu_used, "酒池当酒后 jiu_used 置位");

    // ② 第二坛：jiu_used → 拒
    e.debug_move_named_card_to_hand(1, "shandian");
    let spade2 = e.debug_hand(1).last().copied().unwrap();
    let out = play(&mut e, spade2, vec![]);
    assert!(
        !matches!(out, CommandOutcome::Accepted { .. }),
        "第二坛酒应被拒"
    );

    // ③ 强征+爆敛混入：0/2/3 各 1 手牌 → 1 End 收 3 张+爆敛摸 2
    e.debug_clear_hand(1);
    next_phase(&mut e); // Play→Discard
    drain_choices(&mut e);
    next_phase(&mut e); // Discard→End
    drain_choices(&mut e);
    assert_eq!(e.debug_hand(1).len(), 5, "爆敛 2+强征 3（0/2/3 各一张）");
    assert_eq!(e.debug_hand(0).len(), 3, "强征：0 号 4-1（各收一张）");
    assert_eq!(e.debug_hand(2).len(), 3, "强征：2 号 4-1");
    assert_eq!(e.debug_hand(3).len(), 3, "强征：3 号 4-1");
    assert_zone_conservation(&e);

    // ④ 存档往返
    let snap = e.save();
    let r = Engine::restore(&snap).expect("往返");
    assert!(r.state_core().skills.contains_key(&(1, jc)), "酒池实例往返");
    assert_zone_conservation(&r);
}
