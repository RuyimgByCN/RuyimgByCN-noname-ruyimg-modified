//! VM writes, state queries and history filters through the existing bridge.
use super::*;

#[test]
fn t8_add_mark_writes() {
    let dsl = "\
skill hnl_mark(990022) {
  trigger damage owner_is_target {
    effect add_mark(self, 7, 2);
  }
}
";
    let pack = install_dsl(dsl);
    let mut e = Engine::new_with_extensions(961004, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "hnl_mark");
    keep_only(&mut e, 1, &["hnl_mark"]);
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(
        e.state_core().vm_aux.marks.get(&(1, 7)).copied(),
        Some(2)
    );
}

/// change_max_hp 改变体力上限。
#[test]
fn t8_change_max_hp_writes() {
    let dsl = "\
skill hnl_max(990025) {
  trigger damage owner_is_target {
    effect change_max_hp(self, 1);
  }
}
";
    let pack = install_dsl(dsl);
    let mut e = Engine::new_with_extensions(961005, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "hnl_max");
    keep_only(&mut e, 1, &["hnl_max"]);
    let before = e.state_core().player(1).max_hp;
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(e.state_core().player(1).max_hp, before + 1);
}

/// hand_count 查询为真才摸牌；空手不摸。
#[test]
fn t8_hand_count_query_gates_draw() {
    let dsl = "\
skill hnl_hc(990030) {
  trigger damage owner_is_target {
    if hand_count(self) > 0 {
      effect draw_cards(self, 1);
    }
  }
}
";
    let pack = install_dsl(dsl);
    let mut e = Engine::new_with_extensions(961006, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "hnl_hc");
    keep_only(&mut e, 1, &["hnl_hc"]);
    let h = hand_len(&e, 1);
    assert!(h > 0);
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(hand_len(&e, 1), h + 1, "有手牌时查询放行摸 1");
    let gone: Vec<_> = e.state_core_mut().player_mut(1).hand.drain(..).collect();
    e.state_core_mut().discard.extend(gone);
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(hand_len(&e, 1), 0, "空手查询为假，不摸");
}

/// 本回合丢失史：同一波弃牌尚不可见；下一事件才放行。他人事件不放行。
#[test]
fn t8_lost_history_dual_gate() {
    let dsl = "\
skill hnl_lost(990040) {
  trigger damage owner_alive_lost_hand_this_turn {
    effect draw_cards(self, 1);
  }
}
skill hnl_drop(990041) {
  trigger damage owner_is_target {
    effect discard_hand(self, 1);
  }
}
";
    let pack = install_dsl(dsl);
    let mut e = Engine::new_with_extensions(961007, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "hnl_lost");
    e.debug_grant(1, "hnl_drop");
    keep_only(&mut e, 1, &["hnl_lost", "hnl_drop"]);
    let h = hand_len(&e, 1);
    assert!(h > 0);
    e.debug_deal_damage(Some(0), 2, 1, DamageNature::Normal);
    assert_eq!(hand_len(&e, 1), h, "尚无丢失史，不摸");
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(hand_len(&e, 1), h, "同优先filter在弃牌子事件返回后重新放行");
    assert!(
        e.state_core().vm_aux.lost.get(&1).is_some_and(|c| !c.is_empty()),
        "弃牌写入丢失史"
    );
    e.debug_deal_damage(Some(0), 2, 1, DamageNature::Normal);
    assert_eq!(hand_len(&e, 1), h + 1, "下一事件仍可再次看到丢失史");
}

/// 移牌批次：真实 CardsMoved 放行；关掉技能后同一路径不再摸。
#[test]
fn t8_move_batch_dual_gate() {
    let dsl = "\
skill hnl_batch(990042) {
  trigger cards_moved owner_alive_discard_batch {
    effect draw_cards(self, 1);
  }
}
";
    let pack = install_dsl(dsl);
    let mut e = Engine::new_with_extensions(961008, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "hnl_batch");
    keep_only(&mut e, 1, &["hnl_batch"]);
    let h = hand_len(&e, 1);
    let card = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 2, card);
    assert_eq!(hand_len(&e, 1), h + 1, "移牌批次摸 1");
    e.debug_set_skill_enabled(1, "hnl_batch", false);
    let card2 = e.debug_place(0, "shan");
    e.debug_give_hand_card(0, 2, card2);
    assert_eq!(hand_len(&e, 1), h + 1, "技能关闭后不再摸");
}

/// 技能实例剩余次数：once_used 之前摸 1，置位后不摸。
#[test]
fn t8_skill_uses_remaining_dual_gate() {
    let dsl = "\
skill hnl_uses(990050) {
  trigger damage owner_is_target {
    let remaining = skill_uses_remaining(self_skill);
    if remaining > 0 {
      effect draw_cards(self, 1);
    }
  }
}
";
    let pack = install_dsl(dsl);
    let mut e = Engine::new_with_extensions(961009, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "hnl_uses");
    keep_only(&mut e, 1, &["hnl_uses"]);
    let h = hand_len(&e, 1);
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(hand_len(&e, 1), h + 1, "剩余次数为正时摸 1");
    let def = e.catalog().skill_by_name("hnl_uses").expect("技能");
    e.state_core_mut()
        .skills
        .get_mut(&(1, def))
        .expect("实例")
        .once_used = true;
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(hand_len(&e, 1), h + 1, "次数用尽后不摸");
}
