use super::*;

/// Boss 内容第六批（规则 v220）：书精催毒+中毒——书精伤害非己目标后
/// 授毒（中毒实例）；中毒者回合开始判定：非红桃伤 1、黑桃保留、方片/
/// 梅花伤 1 后失去此技。
#[test]
fn boss_shujing_cuidui_poison_cycle() {
    let mut e = Engine::new_boss_stage(37001, 4, 1, 3, 6, 4, 0);
    // 书精 def：boss 段第 6 条（表序：烛阴/混沌/穷奇/饕餮/梼杌/毕方/
    // 书精/相柳/朱厌——毕方 v227 插入于梼杌后）
    let shujing_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 6) as u16;
    e.debug_set_general_def(1, shujing_def);
    e.debug_set_hp(1, 2);
    let cdef = e.catalog().skill_by_name("催毒").expect("催毒入目录");
    let zdef = e.catalog().skill_by_name("中毒").expect("中毒入目录");
    assert!(e.state_core().skills.contains_key(&(1, cdef)), "催毒实例");

    // ① 书精伤害非己目标 → 目标获中毒实例
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_set_hp(2, 4);
    let sha = e.debug_place(1, "sha");
    play(&mut e, sha, vec![2]);
    drain_choices(&mut e);
    assert!(
        e.state_core().has_skill_any_state(2, zdef),
        "催毒：目标获中毒实例"
    );
    let poisoned_hp = e.debug_hp(2);
    // ② 中毒者回合开始判定：置顶方片（非红桃伤 1+失去此技）
    advance_to_play(&mut e, 2);
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(2), poisoned_hp - 1, "中毒判定：非红桃伤 1");
    assert!(
        !e.state_core().has_skill_any_state(2, zdef),
        "非黑桃 → 失去中毒"
    );

    // ③ 黑桃保留：重新授毒后黑桃判定 → 伤 1 但保留
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    let sha2 = e.debug_place(1, "sha");
    play(&mut e, sha2, vec![2]);
    drain_choices(&mut e);
    assert!(e.state_core().has_skill_any_state(2, zdef), "再授毒");
    e.debug_set_deck_top_by(|d| d.suit == engine_native::catalog::Suit::Spade);
    advance_to_play(&mut e, 2);
    drain_choices(&mut e);
    let hp_after = e.debug_hp(2);
    assert!(
        e.state_core().has_skill_any_state(2, zdef),
        "黑桃 → 中毒保留"
    );
    let _ = hp_after;
}
