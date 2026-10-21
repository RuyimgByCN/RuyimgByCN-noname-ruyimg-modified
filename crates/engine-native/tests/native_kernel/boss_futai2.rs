use super::*;

/// 覆泰2 全场禁桃两面（规则 v214）：场上存有穷奇持覆泰且非其回合——
/// ①他人出牌阶段用桃被拒；②濒死救援时他人真桃不可救（选项不含）；
/// 穷奇自己回合他人用桃恢复可用。
#[test]
fn boss_futai2_tao_banned_both_sides() {
    let mut e = Engine::new_boss_stage(31001, 4, 1, 3, 6, 4, 0);
    let qiongqi_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 2) as u16;
    e.debug_set_general_def(1, qiongqi_def);
    e.debug_set_hp(1, 20);
    let fdef = e.catalog().skill_by_name("覆泰").expect("覆泰入目录");
    assert!(e.state_core().skills.contains_key(&(1, fdef)));

    // ① 他人（2 号）出牌阶段用桃 → 拒（穷奇在场、当前回合非穷奇）
    advance_to_play(&mut e, 2);
    e.debug_clear_hand(2);
    e.debug_set_hp(2, 2); // 满血面让桃本来可用（hp<max）
    let tao = e.debug_place(2, "tao");
    let out = play(&mut e, tao, vec![]);
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "覆泰2：非穷奇回合他人用桃应被拒：{out:?}"
    );

    // ② 濒死救援面：0 号濒死，2 号（他人）持真桃不可救
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(2);
    e.debug_set_hp(0, 1);
    let tao0 = e.debug_place(2, "tao");
    e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    // 濒死询问轮转：救援者无可用桃（2 号真桃被禁）→ 无人救 → 死亡
    let mut saw_tao_option = false;
    for _ in 0..8 {
        match e.open_choice() {
            Some(c) if c.spec.prompt == "dying_save" => {
                if c.spec
                    .options
                    .iter()
                    .any(|o| matches!(o, ChoiceOption::Card(x) if *x == tao0))
                {
                    saw_tao_option = true;
                }
                answer(&mut e, ChoiceOption::Pass);
            }
            Some(_) => {}
            None => break,
        }
    }
    assert!(!saw_tao_option, "覆泰2：救援面真桃不可入选项");
    assert!(!e.debug_alive(0), "无人可救 → 死亡");

    // ③ 穷奇自己回合：禁令解除——穷奇自用桃可用（当前回合=boss）
    advance_to_play(&mut e, 1); // 穷奇回合
    e.debug_clear_hand(1);
    e.debug_set_hp(1, 15);
    let tao2 = e.debug_place(1, "tao");
    let out = play(&mut e, tao2, vec![]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "穷奇回合穷奇自用桃应可用（禁令解除）：{out:?}"
    );
}
