//! Batch U05-6a Part 3: Batch B 20 skills + `ysjqisha` 7 debuffs + subskills
//! (`ysjqisha` .. `zhuilie`, `character/hearth/skill.js:1127-1856`).

use super::common::*;
use engine_native::catalog::CardKind;
use engine_native::event::Phase;
use engine_native::ChoiceOption;

#[test]
fn u05_b6a_batch_b_ysjqisha_through_lieqi() {
    // 1. `ysjqisha` (:1127) + 7 debuffs (`ju`, `kuang`, `nu`, `yi`, `wang`, `hen`, `ao`)
    let mut e = fresh_proto(13_001, &["hs_yashaji", "hs_jaina"]);
    let tracked = total_tracked_cards(&e);
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 1);
    let sha = give_free_card_kind(&mut e, 0, CardKind::Sha);
    let _ = play_card(&mut e, 0, sha, vec![1]);
    drain_auto(&mut e);
    let debuffs = [
        "ysjqisha_ju",
        "ysjqisha_kuang",
        "ysjqisha_nu",
        "ysjqisha_yi",
        "ysjqisha_wang",
        "ysjqisha_hen",
        "ysjqisha_ao",
    ];
    assert!(
        debuffs.iter().any(|s| has_skill(&e, 1, s)),
        "ysjqisha must attach a random煞印 debuff to damaged target"
    );
    assert_eq!(total_tracked_cards(&e), tracked);

    // 2. `yindan` (:1280): active discard spade -> lose 1 hp + draw 2 + summon `hsjixie_zhadan`
    let mut e2 = fresh_proto(13_002, &["hs_pengpeng", "hs_jaina"]);
    let tracked2 = total_tracked_cards(&e2);
    advance_to_phase(&mut e2, 0, Phase::Play);
    clear_hand(&mut e2, 0);
    assert!(matches!(activate(&mut e2, 0, "yindan"), engine_native::CommandOutcome::Rejected(_)));
    let sp = give_card_named(&mut e2, 0, "hsjixie_zhadan"); // Spade 3
    let hp_before = e2.state_core().player(0).hp;
    let out = activate(&mut e2, 0, "yindan");
    assert!(!matches!(out, engine_native::CommandOutcome::Rejected(_)));
    save_restore_and_answer(&mut e2, "hs_yindan_card", vec![ChoiceOption::Card(sp)]);
    drain_auto(&mut e2);
    assert_eq!(e2.state_core().player(0).hp, hp_before - 1);
    assert!(e2.state_core().player(0).hand.len() >= 2);
    assert_eq!(total_tracked_cards(&e2), tracked2);

    // 3-6. `hllingxi` (:1321), `zhaochao` (:1392) + `zhaochao2` (:1414), `xiyong` (:1424)
    let mut e3 = fresh_proto(13_003, &["hs_huolituteng", "hs_fachaotuteng", "hs_manyututeng"]);
    let tracked3 = total_tracked_cards(&e3);
    e3.state_core_mut().player_mut(0).hp = 1;
    advance_to_phase(&mut e3, 0, Phase::Play);
    clear_hand(&mut e3, 0);
    let _c0 = give_free_card_kind(&mut e3, 0, CardKind::Sha);
    advance_to_phase(&mut e3, 0, Phase::Discard);
    let hl_ch = advance_until_open_prompt(&mut e3, "trigger_confirm", 6)
        .expect("hllingxi must prompt trigger_confirm when injured at End Phase");
    assert_eq!(hl_ch.observer, 0);
    let hp_before_hl = e3.state_core().player(0).hp;
    let _ = answer(&mut e3, 0, vec![ChoiceOption::Confirm]);
    drain_auto(&mut e3);
    assert_eq!(
        e3.state_core().player(0).hp,
        hp_before_hl + 1,
        "hllingxi End-Phase confirmation must recover 1 HP"
    );
    // `xiyong` on seat 1 (`hs_fachaotuteng`): End Phase confirm -> draws top card & SelectPlayers AI works; then `zhaochao` on seat 2 (`hs_manyututeng`)
    advance_to_phase(&mut e3, 1, Phase::Play);
    clear_hand(&mut e3, 1);
    let xy_sha = give_free_card_kind(&mut e3, 2, CardKind::Sha);
    put_card_on_deck_top(&mut e3, xy_sha);
    advance_to_phase(&mut e3, 1, Phase::Discard);
    let xy_ch = advance_until_open_prompt(&mut e3, "trigger_confirm", 8)
        .expect("xiyong must prompt trigger_confirm at End Phase");
    assert_eq!(xy_ch.observer, 1);
    let hand_before_xy = e3.state_core().player(1).hand.len();
    let _ = answer(&mut e3, 1, vec![ChoiceOption::Confirm]);
    if let Some(xy_use) = advance_until_open_prompt(&mut e3, "hs_xiyong_use", 4) {
        assert!(xy_use.spec.options.contains(&ChoiceOption::Card(xy_sha)));
        let _ = answer(&mut e3, 1, vec![ChoiceOption::Card(xy_sha)]);
        let xy_tgt = e3.open_choice().cloned().expect("hs_xiyong_use SelectPlayers");
        assert_eq!(xy_tgt.spec.kind, engine_native::choice::ChoiceKind::SelectPlayers);
        let ai_xy = ai_plan_action(&e3, 1);
        assert!(matches!(
            &ai_xy,
            engine_native::GameAction::Answer { selection, .. }
                if selection.iter().any(|o| matches!(o, ChoiceOption::Player(_)))
        ));
        let _ = submit(&mut e3, 1, ai_xy);
    }
    drain_auto(&mut e3);
    assert!(
        e3.state_core().player(1).hand.len() > hand_before_xy || e3.state_core().player(0).hp < 4 || e3.state_core().player(2).hp < 4,
        "xiyong End-Phase confirm must draw top card (and use if usable)"
    );
    advance_to_phase(&mut e3, 2, Phase::Play);
    advance_to_phase(&mut e3, 2, Phase::End);
    drain_auto(&mut e3);
    assert_eq!(total_tracked_cards(&e3), tracked3);

    // 7-9. `srjici` (:1454), `yinzong` (:1467), `tansuo` (:1482), `lieqi` (:1520)
    let mut e4 = fresh_proto(13_004, &["hs_shirencao", "hs_kaituozhe", "hs_yinggencao"]);
    let tracked4 = total_tracked_cards(&e4);
    advance_to_phase(&mut e4, 0, Phase::Play);
    clear_hand(&mut e4, 0);
    clear_hand(&mut e4, 1);
    let sha4 = give_free_card_kind(&mut e4, 0, CardKind::Sha);
    let _vic = give_free_card_kind(&mut e4, 1, CardKind::Sha);
    let log_sr_start = e4.log().len();
    let _ = play_card(&mut e4, 0, sha4, vec![1]);
    drain_auto(&mut e4);
    assert_eq!(
        e4.state_core().player(0).hand.len(),
        1,
        "srjici must draw 1 card on damage"
    );
    assert!(
        e4.state_core().player(1).hand.is_empty(),
        "srjici must randomly discard 1 card from damaged victim"
    );
    let sr_draw_idx = e4.log()[log_sr_start..]
        .iter()
        .position(|ev| matches!(ev, engine_native::machine::LogEvent::CardsDrawn { player: 0, .. }))
        .expect("srjici CardsDrawn log");
    let sr_disc_idx = e4.log()[log_sr_start..]
        .iter()
        .position(|ev| matches!(ev, engine_native::machine::LogEvent::CardDiscarded { player: 1, .. }))
        .expect("srjici victim CardDiscarded log");
    assert!(
        sr_draw_idx < sr_disc_idx,
        "srjici must complete draw BEFORE victim randomDiscard (not inverted)"
    );

    advance_to_phase(&mut e4, 1, Phase::Play);
    clear_hand(&mut e4, 1);
    // `yinzong`: losing equip card grants Shan to hand
    let w1 = give_free_card_kind(&mut e4, 1, CardKind::Weapon);
    let w2 = give_free_card_kind(&mut e4, 1, CardKind::Weapon);
    let _ = play_card(&mut e4, 1, w1, vec![]);
    drain_auto(&mut e4);
    let _ = play_card(&mut e4, 1, w2, vec![]);
    drain_auto(&mut e4);
    assert!(
        e4.state_core().player(1).hand.iter().any(|&c| e4.catalog().card_kind(c) == CardKind::Shan),
        "yinzong must grant Shan when an equipped card leaves equip zone"
    );
    // `tansuo`: active discover hearth derived card (once per turn)
    let dc = give_free_card_kind(&mut e4, 1, CardKind::Sha);
    let out = activate(&mut e4, 1, "tansuo");
    assert!(!matches!(out, engine_native::CommandOutcome::Rejected(_)));
    let _ = answer(&mut e4, 1, vec![ChoiceOption::Card(dc)]);
    assert!(
        e4.open_choice().is_some_and(|c| c.spec.prompt == "hs_tansuo_discover"),
        "tansuo must open hs_tansuo_discover after discard"
    );
    let pick = e4.open_choice().unwrap().spec.options[0];
    save_restore_and_answer(&mut e4, "hs_tansuo_discover", vec![pick]);
    drain_auto(&mut e4);
    assert!(matches!(activate(&mut e4, 1, "tansuo"), engine_native::CommandOutcome::Rejected(_)));
    // `lieqi` on seat 2 (`hs_yinggencao`): Prepare prompts hs_lieqi_target -> guess matching card from seat 0 hand
    clear_hand(&mut e4, 0);
    let secret0 = give_free_card_kind(&mut e4, 0, CardKind::Tao);
    advance_to_phase(&mut e4, 1, Phase::Discard);
    let lq_t = advance_until_open_prompt(&mut e4, "hs_lieqi_target", 8)
        .expect("lieqi at Prepare must prompt hs_lieqi_target");
    assert_eq!(lq_t.observer, 2);
    save_restore_and_answer(&mut e4, "hs_lieqi_target", vec![ChoiceOption::Player(0)]);
    let lq_g = e4.open_choice().cloned().expect("hs_lieqi_guess must open");
    assert_eq!(lq_g.spec.prompt, "hs_lieqi_guess");
    assert!(lq_g.spec.options.contains(&ChoiceOption::Card(secret0)));
    let hand_before_lq = e4.state_core().player(2).hand.len();
    save_restore_and_answer(&mut e4, "hs_lieqi_guess", vec![ChoiceOption::Card(secret0)]);
    drain_auto(&mut e4);
    assert!(
        e4.state_core().player(2).hand.len() > hand_before_lq,
        "guessing the real card in lieqi must grant a copy"
    );
    assert_eq!(total_tracked_cards(&e4), tracked4);
}

#[test]
fn u05_b6a_batch_b_azaowu_through_zhuilie() {
    // 10-12. `azaowu` (:1592), `shouwang` (:1677) & `shouwang2` (:1705)
    let mut e = fresh_proto(13_010, &["hs_amala", "hs_jaina"]);
    let tracked = total_tracked_cards(&e);
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 1);
    let c0 = give_free_card_kind(&mut e, 0, CardKind::Shan);
    // Cancel refund on all 3 stages + ASK_HS_AZAOWU_KIND excludes Tao at full HP
    let az_ch = activate(&mut e, 0, "azaowu");
    assert!(!matches!(az_ch, engine_native::CommandOutcome::Rejected(_)));
    assert!(
        !e.open_choice().unwrap().spec.options.contains(&ChoiceOption::Tag(4)),
        "azaowu at full HP must NOT offer Tao (Tag 4)"
    );
    let _ = cancel_open_choice(&mut e); // Stage 1 cancel refunds once_used
    assert!(!matches!(activate(&mut e, 0, "azaowu"), engine_native::CommandOutcome::Rejected(_)));
    let _ = answer(&mut e, 0, vec![ChoiceOption::Tag(0)]);
    let _ = cancel_open_choice(&mut e); // Stage 2 cancel refunds once_used
    assert!(!matches!(activate(&mut e, 0, "azaowu"), engine_native::CommandOutcome::Rejected(_)));
    let _ = answer(&mut e, 0, vec![ChoiceOption::Tag(0)]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(c0)]);
    let _ = cancel_open_choice(&mut e); // Stage 3 cancel refunds once_used
    assert_eq!(flag(&e, 0, "hs_azaowu_used_turn"), 0);
    let out = activate(&mut e, 0, "azaowu");
    assert!(!matches!(out, engine_native::CommandOutcome::Rejected(_)));
    save_restore_and_answer(&mut e, "hs_azaowu_kind", vec![ChoiceOption::Tag(0)]);
    save_restore_and_answer(&mut e, "hs_azaowu_card", vec![ChoiceOption::Card(c0)]);
    save_restore_and_answer(&mut e, "hs_azaowu_target", vec![ChoiceOption::Player(1)]);
    drain_auto(&mut e);
    assert_eq!(flag(&e, 0, "hs_azaowu_used_turn"), 1);
    assert_eq!(e.state_core().player(1).hp, 2, "azaowu virtual Sha must damage target (3 -> 2)");
    assert!(matches!(activate(&mut e, 0, "azaowu"), engine_native::CommandOutcome::Rejected(_)));
    // Negative legality when Sha & Jiu both spent and HP full: even with hs_azaowu_used_turn=0, activate MUST reject
    e.state_core_mut().vm_aux.flags.remove(&(0, "hs_azaowu_used_turn".to_string()));
    if let Some(def) = e.catalog().skill_by_name("azaowu") {
        if let Some(inst) = e.state_core_mut().skills.get_mut(&(0, def)) {
            inst.once_used = false;
        }
    }
    e.state_core_mut().turn.jiu_used = true;
    let _extra_b = give_free_card_kind(&mut e, 0, CardKind::Shan);
    assert!(
        matches!(activate(&mut e, 0, "azaowu"), engine_native::CommandOutcome::Rejected(_)),
        "azaowu must reject when full HP and Sha & Jiu limits are exhausted"
    );
    assert_eq!(total_tracked_cards(&e), tracked);

    // `shouwang`: once-per-game dying rescue recovers to 1, grants 1 hujia & `shouwang2`
    let mut e2 = fresh_proto(13_011, &["hs_jaina", "hs_amala"]);
    advance_to_phase(&mut e2, 0, Phase::Play);
    clear_hand(&mut e2, 0);
    clear_hand(&mut e2, 1);
    e2.state_core_mut().player_mut(1).hp = 1;
    let sha = give_free_card_kind(&mut e2, 0, CardKind::Sha);
    let sw_def = e2.catalog().skill_by_name("shouwang").unwrap();
    let _ = play_card(&mut e2, 0, sha, vec![1]);
    let ch = advance_until_open_prompt(&mut e2, "dying_save", 6)
        .expect("shouwang must offer dying_save rescue");
    assert!(ch.spec.options.contains(&ChoiceOption::Skill(sw_def)));
    save_restore_and_answer(&mut e2, "dying_save", vec![ChoiceOption::Skill(sw_def)]);
    drain_auto(&mut e2);
    assert!(e2.state_core().player(1).alive);
    assert!(e2.state_core().player(1).hp >= 1);
    assert_eq!(e2.state_core().player(1).hujia, 1);
    assert!(has_skill(&e2, 1, "shouwang2"));

    // 13-19. `qingtian` (:1712), `qianfu` (:1728), `qianfu2` (:1747), `shimo` (:1815),
    // `lieyang` (:1829), `lieyang2` (:1847), `zhuilie` (:1856)
    let mut e3 = fresh_proto(13_012, &["hs_jaina", "hs_bannabusi"]);
    advance_to_phase(&mut e3, 0, Phase::Play);
    clear_hand(&mut e3, 0);
    clear_hand(&mut e3, 1);
    // `qingtian`: owner is strictly max HP -> +1 damage taken
    e3.state_core_mut().player_mut(1).hp = 10;
    let sha3 = give_free_card_kind(&mut e3, 0, CardKind::Sha);
    let _ = play_card(&mut e3, 0, sha3, vec![1]);
    drain_auto(&mut e3);
    assert_eq!(e3.state_core().player(1).hp, 8, "qingtian must amplify damage taken by 1 when max hp");

    // `qianfu` + `qianfu2` + `shimo`: lethal hit grants `qianfu2` (immune to non-Fire damage); adjacent damage triggers `shimo`
    let mut e4 = fresh_proto(13_013, &["hs_jaina", "hs_selajin", "hs_lrexxar"]);
    let tracked4 = total_tracked_cards(&e4);
    advance_to_phase(&mut e4, 0, Phase::Play);
    clear_hand(&mut e4, 0);
    clear_hand(&mut e4, 1);
    clear_hand(&mut e4, 2);
    e4.state_core_mut().player_mut(1).hp = 1;
    let sha4 = give_free_card_kind(&mut e4, 0, CardKind::Sha);
    let _ = play_card(&mut e4, 0, sha4, vec![1]);
    drain_auto(&mut e4);
    assert!(e4.state_core().player(1).alive);
    assert!(has_skill(&e4, 1, "qianfu2"));
    // Non-fire damage against qianfu2 is prevented; adjacent seat 2 taking damage heals injured seat 1 (`shimo`)
    let norm_sha = give_free_card_kind(&mut e4, 0, CardKind::Sha);
    let _ = play_card(&mut e4, 0, norm_sha, vec![1]);
    drain_auto(&mut e4);
    assert_eq!(e4.state_core().player(1).hp, 1, "qianfu2 prevents non-Fire damage");
    let shimo_duel = give_free_card_kind(&mut e4, 0, CardKind::Duel);
    let _ = play_card(&mut e4, 0, shimo_duel, vec![2]);
    drain_auto(&mut e4);
    assert!(e4.state_core().player(1).hp >= 2, "shimo must heal damaged owner when distance-1 player takes damage");
    assert_eq!(total_tracked_cards(&e4), tracked4);

    // `lieyang` + `lieyang2`: using trick draws/gains flame; `zhuilie` on seat 1 (`hs_hemite`)
    let mut e5 = fresh_proto(13_014, &["hs_laila", "hs_hemite"]);
    let tracked5 = total_tracked_cards(&e5);
    advance_to_phase(&mut e5, 0, Phase::Play);
    let wz = give_card_named(&mut e5, 0, "wuzhong");
    let out_wz = play_card(&mut e5, 0, wz, vec![]);
    assert!(!matches!(out_wz, engine_native::CommandOutcome::Rejected(_)));
    drain_auto(&mut e5);
    assert_eq!(flag(&e5, 0, "hs_lieyang_turn_used"), 1);
    // Give seat 1 a hand card so `zhuilie` triggers at seat 1 Prepare -> cancel then check deck intact, or confirm
    advance_to_phase(&mut e5, 0, Phase::Discard);
    let zc = give_free_card_kind(&mut e5, 1, CardKind::Shan);
    let zl = advance_until_open_prompt(&mut e5, "hs_zhuilie_discard", 8)
        .expect("zhuilie must open hs_zhuilie_discard at Prepare when holding card");
    assert_eq!(zl.observer, 1);
    save_restore_and_answer(&mut e5, "hs_zhuilie_discard", vec![ChoiceOption::Card(zc)]);
    drain_auto(&mut e5);
    assert_eq!(total_tracked_cards(&e5), tracked5);
}
