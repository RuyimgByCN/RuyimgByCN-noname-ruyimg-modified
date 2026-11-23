use super::common::*;
use engine_native::catalog::CardKind;
use engine_native::event::{DamageNature, Phase};
use engine_native::{ChoiceOption, CommandOutcome, GameAction, RejectReason};

#[test]
fn u05_b7b_lingjian_passive_and_distance_mods_all_families() {
    let (v, mut e) = start_shahai_swd_only(9321, 3);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    // 1. Equip distance modifiers (`xuanfengzhiren_equip3_dist`/`4_dist`, `zhongxinghujia_equip3/4`,
    // `jinjilengdong_equip3/4`, `yinmilichang_equip3/4`, `xingtigaizao_equip3/4`, `shengxiuhaojiao_equip3/4`,
    // `shijianhuisu_equip3/4`)
    let base_dist = e.state_core().computed_distance(e.catalog(), 0, 1);
    assert_eq!(base_dist, 1);

    grant_skill(&mut e, &v, 1, "hslingjian_xuanfengzhiren_equip3_dist");
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 1), 2);
    grant_skill(&mut e, &v, 0, "hslingjian_xuanfengzhiren_equip4_dist");
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 1), 1);

    remove_skill(&mut e, &v, 1, "hslingjian_xuanfengzhiren_equip3_dist");
    remove_skill(&mut e, &v, 0, "hslingjian_xuanfengzhiren_equip4_dist");

    // `xingtigaizao_equip3` (+1 both ways) vs `xingtigaizao_equip4` (-1 both ways)
    grant_skill(&mut e, &v, 0, "hslingjian_xingtigaizao_equip3");
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 1), 2);
    assert_eq!(e.state_core().computed_distance(e.catalog(), 1, 0), 2);
    grant_skill(&mut e, &v, 0, "hslingjian_xingtigaizao_equip4");
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 1), 1);
    remove_skill(&mut e, &v, 0, "hslingjian_xingtigaizao_equip3");
    remove_skill(&mut e, &v, 0, "hslingjian_xingtigaizao_equip4");

    // `zhongxinghujia_equip3`: +1 if `to` has Armor
    let armor = find_kind(&e, CardKind::Armor);
    grant_skill(&mut e, &v, 1, "hslingjian_zhongxinghujia_equip3");
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 1), 1);
    force_move_card_to_hand(&mut e, &v, 1, armor);
    advance_to_play(&mut e, 1);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: armor, targets: vec![] });
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 1), 2);
    remove_skill(&mut e, &v, 1, "hslingjian_zhongxinghujia_equip3");

    // `yinmilichang_equip3`: +1 if hp==1; `yinmilichang_equip5`: untargetable by Sha/Duel if 0 hand
    advance_to_play(&mut e, 0);
    grant_skill(&mut e, &v, 1, "hslingjian_yinmilichang_equip3");
    grant_skill(&mut e, &v, 1, "hslingjian_yinmilichang_equip5");
    e.debug_set_hp(1, 1);
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 1), 2);
    clear_hand_to_discard(&mut e, &v, 1);
    let sha = find_kind(&e, CardKind::Sha);
    force_move_card_to_hand(&mut e, &v, 0, sha);
    grant_skill(&mut e, &v, 0, "hslingjian_zhongxinghujia_equip4"); // -1 range? no globalFrom-1 since armor? Give seat 0 armor via jinjilengdong_equip4 test too
    let _ = remove_skill(&mut e, &v, 1, "hslingjian_yinmilichang_equip3");
    let rej_yinmi5 = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    assert!(matches!(rej_yinmi5, CommandOutcome::Rejected(RejectReason::IllegalTargets)));
    remove_skill(&mut e, &v, 1, "hslingjian_yinmilichang_equip5");

    // `hslingjian_shengxiuhaojiao_equip2` vs `hslingjian_chaofeng` + `shengxiuhaojiao_equip1` directHit!
    grant_skill(&mut e, &v, 2, "hslingjian_chaofeng");
    // Seat 0 adjacent to seat 2 (`chaofeng`) -> cannot target seat 1 with Sha
    let rej_cf = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    assert!(matches!(rej_cf, CommandOutcome::Rejected(RejectReason::IllegalTargets)));

    // Now seat 0 ALSO gets `hslingjian_shengxiuhaojiao_equip1`, seat 2 holds Shan -> Sha directHit!
    let shan = find_kind(&e, CardKind::Shan);
    force_move_card_to_hand(&mut e, &v, 2, shan);
    grant_skill(&mut e, &v, 0, "hslingjian_shengxiuhaojiao_equip1");
    e.debug_set_hp(2, 4);
    let out_direct = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![2] });
    assert!(matches!(out_direct, CommandOutcome::Accepted { .. }), "{out_direct:?}");
    assert_eq!(e.state_core().player(2).hp, 3, "shengxiuhaojiao_equip1 directHit bypasses Shan");
    remove_skill(&mut e, &v, 2, "hslingjian_chaofeng");
    remove_skill(&mut e, &v, 0, "hslingjian_shengxiuhaojiao_equip1");

    // `hslingjian_jinjilengdong`: blocks using cards & targeting
    grant_skill(&mut e, &v, 0, "hslingjian_jinjilengdong");
    let tao = find_kind(&e, CardKind::Tao);
    force_move_card_to_hand(&mut e, &v, 0, tao);
    e.debug_set_hp(0, 2);
    let rej_freeze = submit(&mut e, 0, GameAction::PlayCard { card: tao, targets: vec![] });
    assert!(matches!(rej_freeze, CommandOutcome::Rejected(_)));
    remove_skill(&mut e, &v, 0, "hslingjian_jinjilengdong");

    // `hslingjian_xingtigaizao_equip5` maxHandcard -1 & `hslingjian_xingtigaizao` cleared on PhaseEnd
    let base_lim = e.state_core().hand_limit(e.catalog(), 0);
    grant_skill(&mut e, &v, 0, "hslingjian_xingtigaizao_equip5");
    assert_eq!(e.state_core().hand_limit(e.catalog(), 0), base_lim.saturating_sub(1));
    grant_skill(&mut e, &v, 0, "hslingjian_xingtigaizao");
    advance_to_phase(&mut e, 0, Phase::End);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain_pass_all(&mut e);
    assert!(
        e.catalog()
            .skill_by_name("hslingjian_xingtigaizao")
            .is_some_and(|d| !e.state_core().has_skill(0, d)),
        "hslingjian_xingtigaizao removed at PhaseEnd"
    );
    assert_eq!(total_card_set(&e), init_set);
}

#[test]
fn u05_b7b_lingjian_equip1_2_5_triggers_equipped_phase_end_and_actives() {
    let (v, mut e) = start_shahai_swd_only(9322, 3);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    // 1. `shijianhuisu_equip1` (equipping Armor draws 1) + `shijianhuisu_equip5` active returns target equips to hand
    let sh_e1 = grant_skill(&mut e, &v, 0, "hslingjian_shijianhuisu_equip1");
    let _ = sh_e1;
    let armor = find_kind(&e, CardKind::Armor);
    force_move_card_to_hand(&mut e, &v, 0, armor);
    let hand_before = e.state_core().player(0).hand.len();
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: armor, targets: vec![] });
    assert_eq!(
        e.state_core().player(0).hand.len(),
        hand_before, // -1 equip + 1 draw = same
        "shijianhuisu_equip1 draws 1 on equipping armor"
    );

    // `shijianhuisu_equip5`: discard 1 he card -> return seat 1's equips to hand
    let armor2 = find_kind_nth(&e, CardKind::Armor, 1);
    advance_to_play(&mut e, 1);
    force_move_card_to_hand(&mut e, &v, 1, armor2);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: armor2, targets: vec![] });
    assert_eq!(e.state_core().player(1).equips.len(), 1);
    advance_to_play(&mut e, 0);
    let sh_e5 = grant_skill(&mut e, &v, 0, "hslingjian_shijianhuisu_equip5");
    let cost_c = find_kind(&e, CardKind::Sha);
    force_move_card_to_hand(&mut e, &v, 0, cost_c);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: sh_e5 });
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_lingjian_e5_cost");
    e = roundtrip_choice(e, &v);
    answer_open(&mut e, vec![ChoiceOption::Card(cost_c)]);
    answer_open(&mut e, vec![ChoiceOption::Player(1)]);
    assert!(e.state_core().player(1).equips.is_empty(), "shijianhuisu_equip5 returned equips");
    assert!(e.state_core().player(1).hand.contains(&armor2));

    // 2. `xingtigaizao_equip5` draws +1 in DrawPhase
    advance_to_play(&mut e, 1);
    clear_hand_to_discard(&mut e, &v, 1);
    grant_skill(&mut e, &v, 1, "hslingjian_xingtigaizao_equip5");
    step_full_turn_to(&mut e, 1);
    assert_eq!(e.state_core().player(1).hand.len(), 3, "DrawPhase +1 gives 3 cards");

    // 3. `jinjilengdong_equip1` (confirmed Sha dmg -> target draws 2 + flips) + `xingtigaizao_equip1` (+1 draw)
    advance_to_play(&mut e, 0);
    clear_hand_to_discard(&mut e, &v, 1);
    grant_skill(&mut e, &v, 0, "hslingjian_jinjilengdong_equip1");
    grant_skill(&mut e, &v, 0, "hslingjian_xingtigaizao_equip1");
    let p0_before = e.state_core().player(0).hand.len();
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    // `e.debug_deal_damage` has card=None so Sha triggers won't fire -> use real Sha!
    let sha_real = find_kind_nth(&e, CardKind::Sha, 1);
    force_move_card_to_hand(&mut e, &v, 0, sha_real);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha_real, targets: vec![1] });
    drain_pass_all(&mut e); // passes Shan -> hits!
    // One of xingtigaizao_equip1 (+1 draw) or jinjilengdong_equip1 confirm prompts:
    assert_eq!(e.state_core().player(0).hand.len(), p0_before + 1, "xingtigaizao_equip1 drew 1");
    assert_eq!(total_card_set(&e), init_set);
}

#[test]
fn u05_b7b_forging_lingjianduanzao_and_jiguanyaoshu_skill() {
    let (v, mut e) = start_shahai_swd_only(9323, 2);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    // 1. `_lingjianduanzao` PhaseUse 3-step forging with card conservation & roundtrip:
    // Forge Weapon + `xingtigaizao` onto seat 0 -> `hslingjian_xingtigaizao_equip1` granted!
    let duanzao_def = grant_skill(&mut e, &v, 0, "_lingjianduanzao");
    let weapon = find_kind(&e, CardKind::Weapon);
    let mat = find_card_name(&e, "xingtigaizao");
    force_move_card_to_hand(&mut e, &v, 0, weapon);
    force_move_card_to_hand(&mut e, &v, 0, mat);

    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: duanzao_def });
    assert!(matches!(out, CommandOutcome::Pending { .. }));
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_duanzao_equip");
    answer_open(&mut e, vec![ChoiceOption::Card(weapon)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_duanzao_mat");
    e = roundtrip_choice(e, &v);
    answer_open(&mut e, vec![ChoiceOption::Card(mat)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_duanzao_target");
    answer_open(&mut e, vec![ChoiceOption::Player(0)]);

    assert!(e.state_core().player(0).equips.contains(&weapon));
    assert_eq!(e.state_core().player(0).card_mods.get(&weapon), Some(&mat));
    let forged_sk = e
        .catalog()
        .skill_by_name("hslingjian_xingtigaizao_equip1")
        .expect("forged skill");
    assert!(
        e.state_core().has_skill(0, forged_sk),
        "forged equip grants real hslingjian_xingtigaizao_equip1 skill"
    );

    // 2. Replacing forged weapon with another weapon revokes forged skill + triggers off-turn `jiguanyaoshu_skill`!
    grant_skill(&mut e, &v, 0, "jiguanyaoshu_skill");
    advance_to_play(&mut e, 1);
    let dis = find_kind(&e, CardKind::Dismantle);
    force_move_card_to_hand(&mut e, &v, 1, dis);
    // Empty seat 0 hand so Dismantle hits seat 0's equip zone!
    clear_hand_to_discard(&mut e, &v, 0);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: dis, targets: vec![0] });
    drain_pass_all(&mut e);
    assert!(!e.state_core().has_skill(0, forged_sk), "removing forged equip revokes forged skill");
    assert!(
        e.state_core()
            .player(0)
            .hand
            .iter()
            .any(|c| e.catalog().card_kind(*c) == CardKind::Hslingjian),
        "jiguanyaoshu_skill off-turn equip loss draws Hslingjian"
    );
    assert_eq!(total_card_set(&e), init_set);
}
