use content_native_loader::boss_play_extension_pack;
use engine_native::catalog::{CardKind, Suit};
use engine_native::state::{BossStageState, GameMode};
use engine_native::{ChoiceOption, Engine, GameAction, Phase};

use super::helpers::{
    answer_choice, assert_accepted_or_pending, assert_archive_roundtrip, clear_equips_to_discard,
    drain_default, drain_one_default, move_card_to_deck_top, move_card_to_hand,
    remove_card_from_all_zones, step_to_phase, submit,
};

#[test]
fn u02_real_event_biantian_bazhen_zhinang_jingmiao_gongshen_huodi_jueji_chiying_chuanyun_keding() {
    let pack = boss_play_extension_pack("noname", &[]);
    let mut e = Engine::new_with_extensions_mode(
        5050,
        4,
        &[&pack],
        GameMode::Boss {
            boss: 0,
            chongzheng: 0,
            fellows: vec![2],
            stage: BossStageState::default_none(),
        },
    )
    .unwrap();
    e.assign_general_named(0, "boss_tianhoukongming").unwrap();
    e.assign_general_named(2, "boss_gongshenyueying").unwrap();
    e.assign_general_named(1, "boss_juechenmiaocai").unwrap();
    e.assign_general_named(3, "boss_jiarenzidan").unwrap();

    // 1. Test boss_biantian black judge -> marks ally 0 & 2 with dawu (prevents non-thunder damage)
    let blk = e.debug_place_suit(0, Suit::Spade);
    e.state_core_mut().player_mut(0).hand.retain(|&c| c != blk);
    e.state_core_mut().deck.push(blk);
    e.debug_set_turn(3, Phase::End, 1);
    let _ = submit(&mut e, 3, GameAction::NextPhase);
    drain_default(&mut e);
    step_to_phase(&mut e, 0, Phase::Play);
    assert_eq!(
        e.state_core().vm_aux.flags.get(&(2, "q4_biantian_dawu".to_string())).copied(),
        Some(1),
        "black biantian grants dawu protection to ally"
    );

    // Non-thunder Sha against dawu ally seat 2 is prevented; and bazhen protects unarmed seat 0 when attacked!
    step_to_phase(&mut e, 1, Phase::Play);
    let sha_dawu = e.debug_place(1, "sha");
    let hp2 = e.state_core().player(2).hp;
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: sha_dawu, targets: vec![2] });
    drain_default(&mut e);
    assert_eq!(e.state_core().player(2).hp, hp2, "dawu prevented non-thunder damage");

    // bazhen auto Bagua dodge against red judge
    clear_equips_to_discard(&mut e, 0);
    e.debug_clear_hand(0);
    let j_red = e.debug_place_suit(1, Suit::Heart);
    e.state_core_mut().player_mut(1).hand.retain(|&c| c != j_red);
    e.state_core_mut().deck.push(j_red);
    let sha_bazhen = e.debug_place(1, "sha");
    let hp0 = e.state_core().player(0).hp;
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: sha_bazhen, targets: vec![0] });
    drain_default(&mut e);
    assert_eq!(e.state_core().player(0).hp, hp0, "bazhen red judge dodged Sha without armor equip");

    // 2. Test boss_chiying on seat 3 protects allied seat 1 from >1 damage
    e.assign_general_named(2, "boss_fuweizilong").unwrap();
    step_to_phase(&mut e, 2, Phase::Play);
    // Use Jiu + Sha on enemy 1 (2 damage reduced by chiying -> 1 damage!)
    let jiu = e.debug_place(2, "jiu");
    let sha_chi = e.debug_place(2, "sha");
    e.debug_clear_hand(1);
    let hp1_before = e.state_core().player(1).hp;
    let _ = submit(&mut e, 2, GameAction::PlayCard { card: jiu, targets: vec![] });
    drain_default(&mut e);
    let _ = submit(&mut e, 2, GameAction::PlayCard { card: sha_chi, targets: vec![1] });
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt.starts_with("boss_keding") {
            answer_choice(&mut e, Vec::new()); // skip keding here
        } else {
            drain_one_default(&mut e);
        }
    }
    assert_eq!(e.state_core().player(1).hp, hp1_before - 1, "boss_chiying capped ally damage to 1");

    // 3. Test boss_bashi & boss_danjing on seat 1 = boss_kumuyuanrang
    e.assign_general_named(1, "boss_kumuyuanrang").unwrap();
    e.assign_general_named(3, "boss_juechenmiaocai").unwrap();
    step_to_phase(&mut e, 0, Phase::Play);
    let sha_b = e.debug_place(0, "sha");
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha_b, targets: vec![1] });
    let mut confirmed_bashi = false;
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "boss_bashi:confirm" {
            confirmed_bashi = true;
            answer_choice(&mut e, vec![ChoiceOption::Confirm]);
        } else {
            drain_one_default(&mut e);
        }
    }
    assert!(confirmed_bashi, "boss_bashi prompted");
    assert!(e.state_core().player(1).face_down, "boss_bashi turned face down to cancel targeting");

    // Save/Restore check
    let restored = assert_archive_roundtrip(&e, &[&pack]);
    assert!(restored.state_core().player(1).face_down);
}

#[test]
fn u02_real_event_remaining_jiange_boss_skills_matrix_and_restore() {
    let pack = boss_play_extension_pack("noname", &[]);
    let mut e = Engine::new_with_extensions_mode(
        6060,
        4,
        &[&pack],
        GameMode::Boss {
            boss: 0,
            chongzheng: 0,
            fellows: vec![2],
            stage: BossStageState::default_none(),
        },
    )
    .unwrap();
    e.assign_general_named(0, "boss_gongshenyueying").unwrap();
    e.assign_general_named(2, "boss_yuhuoshiyuan").unwrap();
    e.assign_general_named(1, "boss_qiaokuijunyi").unwrap();
    e.assign_general_named(3, "boss_juechenmiaocai").unwrap();

    // 1. boss_zhinang: top 5 contains guohe -> Confirm -> give to ally seat 2
    let guohe = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == CardKind::Dismantle && !c.off_deck)
        .unwrap() as u32;
    move_card_to_deck_top(&mut e, guohe);
    let hand2_before = e.state_core().player(2).hand.len();
    e.debug_set_turn(3, Phase::End, 1);
    let _ = submit(&mut e, 3, GameAction::NextPhase);
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "boss_zhinang:target" {
            answer_choice(&mut e, vec![ChoiceOption::Player(2)]);
        } else if c.spec.options.contains(&ChoiceOption::Confirm) {
            answer_choice(&mut e, vec![ChoiceOption::Confirm]);
        } else {
            drain_one_default(&mut e);
        }
    }
    assert!(
        e.state_core().player(2).hand.contains(&guohe)
            && e.state_core().player(2).hand.len() > hand2_before,
        "boss_zhinang picked up trick/equip cards for ally"
    );

    // 2. boss_jingmiao: BUG-5 optional confirm -> Pass leaves HP unchanged; Confirm makes enemy lose 1 HP
    step_to_phase(&mut e, 0, Phase::Play);
    let wuzhong_name = e.catalog().cards.iter().find(|c| c.kind == CardKind::Wuzhong).unwrap().name;
    let wuxie_name = e.catalog().cards.iter().find(|c| c.kind == CardKind::Nullify).unwrap().name;
    let wuzhong_decline = e.debug_place(0, wuzhong_name);
    e.debug_clear_hand(2);
    e.debug_clear_hand(3);
    let wuxie_decline = e.debug_place(1, wuxie_name);
    let hp1_before_decline = e.state_core().player(1).hp;
    assert_accepted_or_pending(&submit(&mut e, 0, GameAction::PlayCard { card: wuzhong_decline, targets: vec![] }));
    let mut saw_jingmiao_decline = false;
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if let Some(opt) = c
            .spec
            .options
            .iter()
            .find(|o| matches!(o, ChoiceOption::Card(cid) if *cid == wuxie_decline))
        {
            answer_choice(&mut e, vec![*opt]);
        } else if c.spec.options.contains(&ChoiceOption::Confirm) {
            saw_jingmiao_decline = true;
            let decl = if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else {
                Vec::new()
            };
            answer_choice(&mut e, decl);
        } else {
            drain_one_default(&mut e);
        }
    }
    assert!(saw_jingmiao_decline, "boss_jingmiao prompted optional Confirm");
    assert_eq!(e.state_core().player(1).hp, hp1_before_decline, "declining boss_jingmiao leaves HP unchanged");

    let wuzhong = e.debug_place(0, wuzhong_name);
    let wuxie = e.debug_place(1, wuxie_name);
    let hp1_before = e.state_core().player(1).hp;
    assert_accepted_or_pending(&submit(&mut e, 0, GameAction::PlayCard { card: wuzhong, targets: vec![] }));
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if let Some(opt) = c
            .spec
            .options
            .iter()
            .find(|o| matches!(o, ChoiceOption::Card(cid) if *cid == wuxie))
        {
            answer_choice(&mut e, vec![*opt]);
        } else if c.spec.options.contains(&ChoiceOption::Confirm) {
            answer_choice(&mut e, vec![ChoiceOption::Confirm]);
        } else {
            drain_one_default(&mut e);
        }
    }
    assert_eq!(e.state_core().player(1).hp, hp1_before - 1, "confirming boss_jingmiao made Nullify user lose 1 HP");

    // 3. boss_gongshenjg: heal wounded mech ally (seat 2), then fire-damage mech enemy (seat 3)
    e.state_core_mut().vm_aux.flags.insert((2, "q4_is_mech".to_string()), 1);
    e.state_core_mut().vm_aux.flags.insert((3, "q4_is_mech".to_string()), 1);
    e.state_core_mut().player_mut(2).hp = 3;
    step_to_phase(&mut e, 0, Phase::Discard);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    for _ in 0..6 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.options.contains(&ChoiceOption::Confirm) {
            answer_choice(&mut e, vec![ChoiceOption::Confirm]);
        } else {
            drain_one_default(&mut e);
        }
    }
    assert_eq!(e.state_core().player(2).hp, 4, "boss_gongshenjg healed wounded mech ally +1");
    let hp3_before = e.state_core().player(3).hp;
    e.debug_set_turn(0, Phase::Discard, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    for _ in 0..6 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.options.contains(&ChoiceOption::Confirm) {
            answer_choice(&mut e, vec![ChoiceOption::Confirm]);
        } else {
            drain_one_default(&mut e);
        }
    }
    assert_eq!(e.state_core().player(3).hp, hp3_before - 1, "boss_gongshenjg dealt 1 Fire damage to mech enemy");

    // 4. boss_yuhuojg prevents Fire Sha & boss_tianyujg links unlinked enemies
    step_to_phase(&mut e, 1, Phase::Play);
    let huosha_name = e.catalog().cards.iter().find(|c| c.kind == CardKind::FireSha).unwrap().name;
    let huosha = e.debug_place(1, huosha_name);
    e.debug_clear_hand(2);
    assert_accepted_or_pending(&submit(&mut e, 1, GameAction::PlayCard { card: huosha, targets: vec![2] }));
    drain_default(&mut e);
    assert_eq!(e.state_core().player(2).hp, 4, "boss_yuhuojg prevented Fire damage");

    step_to_phase(&mut e, 2, Phase::Discard);
    let _ = submit(&mut e, 2, GameAction::NextPhase);
    drain_default(&mut e);
    assert!(e.state_core().player(1).linked && e.state_core().player(3).linked, "boss_tianyujg linked enemies");
    e.state_core_mut().player_mut(1).linked = false;
    e.state_core_mut().player_mut(3).linked = false;

    // 5. boss_huodi turns enemy face_down when ally is face_down; boss_jueji reduces wounded enemy draw
    step_to_phase(&mut e, 1, Phase::Discard);
    e.state_core_mut().player_mut(3).face_down = true;
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "boss_huodi:target" {
            answer_choice(&mut e, vec![ChoiceOption::Player(0)]);
        } else if c.spec.options.contains(&ChoiceOption::Confirm) {
            answer_choice(&mut e, vec![ChoiceOption::Confirm]);
        } else {
            drain_one_default(&mut e);
        }
    }
    assert!(e.state_core().player(0).face_down, "boss_huodi flipped enemy face_down");
    e.state_core_mut().player_mut(0).face_down = false;
    e.state_core_mut().player_mut(3).face_down = false;

    e.state_core_mut().player_mut(2).hp = 2;
    let hand2_pre_draw = e.state_core().player(2).hand.len();
    step_to_phase(&mut e, 2, Phase::Judge);
    let _ = submit(&mut e, 2, GameAction::NextPhase);
    for _ in 0..6 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.options.contains(&ChoiceOption::Confirm) {
            answer_choice(&mut e, vec![ChoiceOption::Confirm]);
        } else {
            drain_one_default(&mut e);
        }
    }
    assert_eq!(e.state_core().player(2).hand.len(), hand2_pre_draw + 1, "boss_jueji reduced wounded enemy draw by 1");

    // 6. boss_fengxing + boss_leili + boss_chuanyun on seat 3
    e.state_core_mut().player_mut(3).hp = 3;
    e.state_core_mut().player_mut(0).hp = 4;
    e.state_core_mut().player_mut(2).hp = 4;
    e.debug_clear_hand(0);
    step_to_phase(&mut e, 2, Phase::End);
    let _ = submit(&mut e, 2, GameAction::NextPhase);
    for _ in 0..10 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "boss_fengxing:target" {
            answer_choice(&mut e, vec![ChoiceOption::Player(0)]);
        } else {
            drain_one_default(&mut e);
        }
    }
    assert_eq!(e.state_core().player(0).hp, 3, "boss_fengxing dealt virtual Sha damage");

    step_to_phase(&mut e, 3, Phase::Play);
    e.debug_clear_hand(0);
    let sha_ll = e.debug_place(3, "sha");
    let _ = submit(&mut e, 3, GameAction::PlayCard { card: sha_ll, targets: vec![0] });
    for _ in 0..10 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "boss_leili:target" {
            answer_choice(&mut e, vec![ChoiceOption::Player(2)]);
        } else {
            drain_one_default(&mut e);
        }
    }
    assert_eq!(e.state_core().player(2).hp, 3, "boss_leili dealt 1 Thunder damage to another enemy");

    e.state_core_mut().player_mut(0).hp = 4;
    step_to_phase(&mut e, 3, Phase::Discard);
    let _ = submit(&mut e, 3, GameAction::NextPhase);
    for _ in 0..6 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "boss_chuanyun_target" {
            answer_choice(&mut e, vec![ChoiceOption::Player(0)]);
        } else {
            drain_one_default(&mut e);
        }
    }
    assert_eq!(e.state_core().player(0).hp, 3, "boss_chuanyun dealt 1 Fire damage to higher-HP enemy");

    // 7. boss_keding + boss_xiaorui + boss_huchen + xiuluo + shenqu
    e.assign_general_named(0, "boss_yihanyunchang").unwrap();
    e.assign_general_named(2, "boss_fuweizilong").unwrap();
    step_to_phase(&mut e, 2, Phase::Play);
    let sha_k = e.debug_place(2, "sha");
    let extra_k = e.debug_place(2, "sha");
    e.debug_clear_hand(1);
    e.debug_clear_hand(3);
    let _ = submit(&mut e, 2, GameAction::PlayCard { card: sha_k, targets: vec![1] });
    for _ in 0..10 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "boss_keding:cards" {
            answer_choice(&mut e, vec![ChoiceOption::Card(extra_k)]);
        } else if c.spec.prompt == "boss_keding:targets" {
            answer_choice(&mut e, vec![ChoiceOption::Player(3)]);
        } else {
            drain_one_default(&mut e);
        }
    }
    assert!(
        e.state_core().vm_aux.flags.get(&(2, "skill_turn:q4_xiaorui_sha_plus".to_string())).copied().unwrap_or(0) >= 1,
        "boss_xiaorui granted extra Sha capacity during friendly turn"
    );
    assert_eq!(
        e.state_core().vm_aux.flags.get(&(1, "q4_fengjian_ban:2".to_string())).copied(),
        Some(1),
        "boss_fengjian banned target from targeting seat 2"
    );

    // Trigger boss_huchen on seat 0 killing seat 1
    step_to_phase(&mut e, 0, Phase::Play);
    e.state_core_mut().player_mut(1).hp = 1;
    e.debug_clear_hand(1);
    e.debug_clear_hand(3);
    let sha_kill = e.debug_place(0, "sha");
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha_kill, targets: vec![1] });
    drain_default(&mut e);
    assert!(
        e.state_core().vm_aux.flags.get(&(0, "q4_huchen_kills".to_string())).copied().unwrap_or(0) >= 1,
        "boss_huchen recorded kill count"
    );

    // Test xiuluo + shenqu
    e.assign_general_named(1, "boss_kumuyuanrang").unwrap();
    e.assign_general_named(0, "boss_lvbu2").unwrap();
    let heart_le = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.suit == Suit::Heart && !c.off_deck)
        .unwrap() as u32;
    let heart_hand = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .position(|(i, c)| i as u32 != heart_le && c.suit == Suit::Heart && !c.off_deck)
        .unwrap() as u32;
    remove_card_from_all_zones(&mut e, heart_le);
    e.state_core_mut().player_mut(0).judge.push(heart_le);
    move_card_to_hand(&mut e, 0, heart_hand);
    e.debug_set_turn(3, Phase::End, 1);
    let _ = submit(&mut e, 3, GameAction::NextPhase);
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "xiuluo_q4:hand" {
            answer_choice(&mut e, vec![ChoiceOption::Card(heart_hand)]);
        } else {
            drain_one_default(&mut e);
        }
    }
    assert!(e.state_core().player(0).judge.is_empty(), "xiuluo discarded matching-suit judge card");

    e.assign_general_named(0, "boss_lvbu3").unwrap();
    e.debug_clear_hand(0);
    // BUG-1: Prepare shenqu is optional (Pass draws 0; Confirm draws 2)
    e.debug_set_turn(3, Phase::End, 1);
    let _ = submit(&mut e, 3, GameAction::NextPhase);
    let mut declined_sq_prep = false;
    for _ in 0..6 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.observer == 0 && c.spec.options.contains(&ChoiceOption::Confirm) {
            declined_sq_prep = true;
            let decl = if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else {
                Vec::new()
            };
            answer_choice(&mut e, decl);
        } else {
            drain_one_default(&mut e);
        }
    }
    assert!(declined_sq_prep, "shenqu Prepare prompted Confirm");
    assert_eq!(e.state_core().player(0).hand.len(), 0, "declining shenqu Prepare draws 0");

    e.debug_set_turn(3, Phase::End, 1);
    let _ = submit(&mut e, 3, GameAction::NextPhase);
    let mut accepted_sq_prep = false;
    for _ in 0..6 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.observer == 0 && c.spec.options.contains(&ChoiceOption::Confirm) {
            accepted_sq_prep = true;
            answer_choice(&mut e, vec![ChoiceOption::Confirm]);
        } else {
            drain_one_default(&mut e);
        }
    }
    assert!(accepted_sq_prep, "shenqu Prepare prompted Confirm");
    assert!(e.state_core().player(0).hand.len() >= 2, "shenqu drew +2 on Prepare when confirmed");

    // BUG-1: DamageAfter asks shenqu:tao directly (no extra trigger_confirm); Pass declines, picking Card consumes Tao without duplicating
    e.debug_clear_hand(0);
    let tao = e.debug_place(0, "tao");
    let hp0_before = e.state_core().player(0).hp;
    step_to_phase(&mut e, 1, Phase::Play);
    let sha_decl = e.debug_place(1, "sha");
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: sha_decl, targets: vec![0] });
    let mut direct_prompts = Vec::new();
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.observer == 0 && c.spec.prompt != "respond_shan" {
            direct_prompts.push(c.spec.prompt);
        }
        if c.spec.prompt == "shenqu:tao" {
            assert!(c.spec.allow_cancel, "shenqu:tao allows cancel");
            answer_choice(&mut e, Vec::new());
        } else {
            drain_one_default(&mut e);
        }
    }
    assert_eq!(direct_prompts, vec!["shenqu:tao"], "DamageAfter goes directly to shenqu:tao");
    assert_eq!(e.state_core().player(0).hp, hp0_before - 1, "declining shenqu:tao skips recovery");
    assert!(e.state_core().player(0).hand.contains(&tao), "declined tao remains in hand");

    e.state_core_mut().turn.sha_used_play_phase = false;
    let sha_sq = e.debug_place(1, "sha");
    assert_accepted_or_pending(&submit(&mut e, 1, GameAction::PlayCard { card: sha_sq, targets: vec![0] }));
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "shenqu:tao" {
            answer_choice(&mut e, vec![ChoiceOption::Card(tao)]);
        } else {
            drain_one_default(&mut e);
        }
    }
    assert_eq!(e.state_core().player(0).hp, hp0_before - 1, "shenqu healed back +1 HP via Tao after second damage");
    assert!(!e.state_core().player(0).hand.contains(&tao), "shenqu consumed tao from hand");
    assert!(e.state_core().discard.contains(&tao), "shenqu sent tao cleanly to discard");
    assert_archive_roundtrip(&e, &[&pack]);
}
