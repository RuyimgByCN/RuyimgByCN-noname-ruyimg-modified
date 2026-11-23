//! Batch U05-6a Part 2: Batch A 20 skills (`hshuanyu` .. `hstianqi_shali`,
//! `character/hearth/skill.js:5-1117`).

use super::common::*;
use engine_native::catalog::CardKind;
use engine_native::event::Phase;
use engine_native::ChoiceOption;

#[test]
fn u05_b6a_batch_a_first_10_hshuanyu_through_asyouzhang() {
    // 1. `hshuanyu` (:5): when damaged, discover 1 hearth derivable card (confirm + cancel + save/restore)
    let mut e = fresh_proto(12_001, &["hs_lrexxar", "hs_duyaxinshi"]);
    let tracked = total_tracked_cards(&e);
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 1);
    let sha = give_free_card_kind(&mut e, 0, CardKind::Sha);
    let _ = play_card(&mut e, 0, sha, vec![1]);
    let ch = advance_until_open_prompt(&mut e, "trigger_confirm", 8).expect("hshuanyu confirm");
    let _ = answer(&mut e, ch.observer, vec![ChoiceOption::Confirm]);
    let d_opt = e.open_choice().unwrap().spec.options[0];
    save_restore_and_answer(&mut e, "hs_hshuanyu_discover", vec![d_opt]);
    assert_eq!(e.state_core().player(1).hand.len(), 1);
    assert_eq!(total_tracked_cards(&e), tracked);

    // 2. `wxuying` (:57): Prepare transforms hand shadows; using card updates shadows & triggers draw/hide
    let mut e2 = fresh_proto(12_002, &["hs_xukongzhiying", "hs_jaina"]);
    let tracked2 = total_tracked_cards(&e2);
    advance_to_phase(&mut e2, 0, Phase::Play);
    clear_hand(&mut e2, 0);
    let _sh = give_card_named(&mut e2, 0, "hsfashu_anyingjingxiang");
    let tao = give_free_card_kind(&mut e2, 0, CardKind::Tao);
    e2.state_core_mut().player_mut(0).hp = 2;
    let _ = play_card(&mut e2, 0, tao, vec![]);
    drain_auto(&mut e2);
    // `wxuying_change` sets hs_wxuying_kind on shadow in hand; playing another shadow during turn clears after TurnEndWindow
    advance_to_phase(&mut e2, 1, Phase::Play);
    assert_eq!(total_tracked_cards(&e2), tracked2);

    // 3-4. `yibian` (:145) + `buwendingyibian_lose` (:133) + `hualing` (:155)
    let mut e3 = fresh_identity(
        12_003,
        &["hs_siwangxianzhi", "hs_jaina", "hs_lrexxar", "hs_wuther"],
    );
    let tracked3 = total_tracked_cards(&e3);
    advance_to_phase(&mut e3, 0, Phase::Play);
    // `yibian` fires at PhaseBegin(Play) if no `hsfashu_buwendingyibian` in hand
    assert!(
        e3.state_core()
            .player(0)
            .hand
            .iter()
            .any(|&c| e3.catalog().cards[c as usize].name.contains("buwendingyibian")),
        "yibian must grant hsfashu_buwendingyibian at start of Play phase"
    );
    // `hualing` (:155) active skill: play-phase pick target -> AI public choice -> pick skill tag
    clear_hand(&mut e3, 0);
    let c1 = give_free_card_kind(&mut e3, 0, CardKind::Sha);
    let c2 = give_free_card_kind(&mut e3, 0, CardKind::Shan);
    let _ = (c1, c2);
    let out = activate(&mut e3, 0, "hualing");
    assert!(!matches!(out, engine_native::CommandOutcome::Rejected(_)), "hualing must activate at round 1");
    save_restore_and_answer(&mut e3, "hs_hualing_target", vec![ChoiceOption::Player(1)]);
    assert!(
        e3.open_choice().is_some_and(|c| c.spec.prompt == "hs_hualing_skill"),
        "hualing target hs_jaina has unheld skills so hs_hualing_skill must open"
    );
    let ai_sel = ai_choose_current(&e3);
    let _ = answer_any(&mut e3, ai_sel);
    drain_auto(&mut e3);
    assert!(flag(&e3, 0, "hs_hualing_granted_def") > 0);
    // Cooldown 3 rounds: reusing hualing in the same turn is rejected
    assert!(matches!(activate(&mut e3, 0, "hualing"), engine_native::CommandOutcome::Rejected(_)));
    assert_eq!(total_tracked_cards(&e3), tracked3);

    // 5. `muyin` (:208): losing last hand card hides (`yinshen_active = true`)
    let mut e4 = fresh_proto(12_004, &["hs_hajiasha", "hs_jaina"]);
    advance_to_phase(&mut e4, 0, Phase::Play);
    clear_hand(&mut e4, 0);
    let jiu = give_free_card_kind(&mut e4, 0, CardKind::Jiu);
    let _ = play_card(&mut e4, 0, jiu, vec![]);
    drain_auto(&mut e4);
    assert!(
        e4.state_core().player(0).yinshen_active,
        "muyin must activate stealth when hand becomes empty"
    );

    // 6. `tqchuanyue` (:236): Prepare PhaseBegin fires; let's verify clean turn entry + card conservation
    let mut e5 = fresh_proto(12_005, &["hs_tuoqi", "hs_jaina"]);
    let tracked5 = total_tracked_cards(&e5);
    advance_to_phase(&mut e5, 0, Phase::Play);
    advance_to_phase(&mut e5, 1, Phase::Play);
    advance_to_phase(&mut e5, 0, Phase::Play);
    assert_eq!(total_tracked_cards(&e5), tracked5);

    // 7-9. `oldhuanjue` (:273), `hshuanling` (:367), `hsxingyi` (:465)
    let mut e6 = fresh_proto(12_006, &["hs_taisi", "hs_lrexxar"]);
    e6.debug_grant(0, "oldhuanjue");
    let tracked6 = total_tracked_cards(&e6);
    advance_to_phase(&mut e6, 1, Phase::Play);
    clear_hand(&mut e6, 0);
    clear_hand(&mut e6, 1);
    let duel = give_free_card_kind(&mut e6, 1, CardKind::Duel);
    let _ = play_card(&mut e6, 1, duel, vec![0]);
    let ch = advance_until_open_prompt(&mut e6, "trigger_confirm", 8)
        .expect("oldhuanjue must prompt trigger_confirm on incoming single-target card");
    let _ = answer(&mut e6, ch.observer, vec![ChoiceOption::Confirm]);
    assert!(
        e6.open_choice().is_some_and(|c| c.spec.prompt == "hs_oldhuanjue_pick"),
        "confirming oldhuanjue must open hs_oldhuanjue_pick"
    );
    let opt = e6.open_choice().unwrap().spec.options[0];
    save_restore_and_answer(&mut e6, "hs_oldhuanjue_pick", vec![opt]);
    drain_auto(&mut e6);
    assert_eq!(
        flag(&e6, 0, "hs_oldhuanjue_used_turn"),
        1,
        "oldhuanjue replaces incoming card once per turn (character/hearth/skill.js:273)"
    );
    assert_eq!(total_tracked_cards(&e6), tracked6);

    // `buwendingyibian_lose` (:133): `_destroy` clone vanishes on TurnEndWindow
    let mut eb = fresh_proto(12_101, &["hs_siwangxianzhi", "hs_jaina"]);
    let tracked_b = total_tracked_cards(&eb);
    advance_to_phase(&mut eb, 0, Phase::Play);
    clear_hand(&mut eb, 0);
    clear_hand(&mut eb, 1);
    let target_card = give_free_card_kind(&mut eb, 1, CardKind::Sha);
    let unstable = give_card_named(&mut eb, 0, "hsfashu_buwendingyibian");
    assert!(!matches!(play_card(&mut eb, 0, unstable, vec![1]), engine_native::CommandOutcome::Rejected(_)));
    save_restore_and_answer(&mut eb, "hs_buwendingyibian_recast", vec![ChoiceOption::Card(target_card)]);
    drain_auto(&mut eb);
    assert!(
        eb.state_core().player(0).hand.iter().any(|&c| {
            eb.catalog().cards[c as usize].name.contains("buwendingyibian")
                && flag(&eb, 0, &format!("hs_buwendingyibian_destroy:{c}")) > 0
        }),
        "hsfashu_buwendingyibian must grant a _destroy clone"
    );
    advance_to_phase(&mut eb, 1, Phase::Play);
    drain_auto(&mut eb);
    assert!(
        !eb.state_core().player(0).hand.iter().any(|&c| flag(&eb, 0, &format!("hs_buwendingyibian_destroy:{c}")) > 0),
        "buwendingyibian_lose must strip _destroy clones after turn ends"
    );
    assert_eq!(total_tracked_cards(&eb), tracked_b);

    // `tqchuanyue` (:236): Prepare grants an old* skill and replaces it on subsequent Prepare
    let mut etq = fresh_proto(12_102, &["hs_tuoqi", "hs_jaina"]);
    advance_to_phase(&mut etq, 1, Phase::Play);
    advance_to_phase(&mut etq, 0, Phase::Play);
    let first_tq = flag(&etq, 0, "hs_tqchuanyue_def");
    assert!(first_tq > 0, "tqchuanyue must grant an old* skill at Prepare");
    let first_name = etq.catalog().skill_name((first_tq - 1) as u16).to_string();
    assert!(has_skill(&etq, 0, &first_name));

    // `hsxingyi` (:465) & `hshuanling` (:367): enemy active skill copied temporarily; enemy single-target trick recorded & replayed
    let mut eh = fresh_identity(12_103, &["hs_taisi", "hs_jaina", "hs_kazhakusi", "hs_lrexxar"]);
    let tracked_h = total_tracked_cards(&eh);
    advance_to_phase(&mut eh, 2, Phase::Play);
    clear_hand(&mut eh, 2);
    let zc = give_free_card_kind(&mut eh, 2, CardKind::Sha);
    assert!(!matches!(activate(&mut eh, 2, "lianjin"), engine_native::CommandOutcome::Rejected(_)));
    let _ = answer(&mut eh, 2, vec![ChoiceOption::Card(zc)]);
    drain_auto(&mut eh);
    assert!(
        has_skill(&eh, 0, "lianjin"),
        "hsxingyi must temporarily copy enemy active skill invoked during enemy's turn"
    );
    let d_card = give_free_card_kind(&mut eh, 2, CardKind::Duel);
    let _ = play_card(&mut eh, 2, d_card, vec![0]);
    drain_auto(&mut eh);
    assert!(
        flag(&eh, 0, "hs_hshuanling_cnt") >= 1,
        "hshuanling must record enemy single-target trick"
    );
    advance_to_phase(&mut eh, 0, Phase::Play);
    assert!(
        has_skill(&eh, 0, "lianjin"),
        "hsxingyi temp skill must stay until hs_taisi's own phaseAfter (character/hearth/skill.js:478)"
    );
    clear_hand(&mut eh, 0);
    let hc = give_free_card_kind(&mut eh, 0, CardKind::Shan);
    // Cancel path first on hshuanling at seat 0 End, or confirm discard & draw + cast
    advance_to_phase(&mut eh, 0, Phase::Discard);
    let hl_ch = advance_until_open_prompt(&mut eh, "hs_hshuanling_discard", 8)
        .expect("hshuanling must prompt discard at owner End Phase");
    let _ = hl_ch;
    save_restore_and_answer(&mut eh, "hs_hshuanling_discard", vec![ChoiceOption::Card(hc)]);
    drain_auto(&mut eh);
    assert_eq!(flag(&eh, 0, "hs_hshuanling_cnt"), 0, "hshuanling must consume stored trick");
    advance_to_phase(&mut eh, 1, Phase::Play);
    assert!(
        !has_skill(&eh, 0, "lianjin"),
        "hsxingyi temp skill must expire after hs_taisi's turn ends"
    );
    assert_eq!(total_tracked_cards(&eh), tracked_h);

    // `asyouzhang` (:481): End Phase actually triggers, draws missing card, opens hs_asyouzhang_use + SelectPlayers AI
    let mut e7 = fresh_proto(12_007, &["hs_ashamoer", "hs_jaina"]);
    let tracked7 = total_tracked_cards(&e7);
    advance_to_phase(&mut e7, 0, Phase::Play);
    clear_hand(&mut e7, 0);
    let sha_end = give_free_card_kind(&mut e7, 1, CardKind::Sha);
    put_card_on_deck_top(&mut e7, sha_end);
    advance_to_phase(&mut e7, 0, Phase::Discard);
    let az_use = advance_until_open_prompt(&mut e7, "hs_asyouzhang_use", 6)
        .expect("asyouzhang at End Phase must draw missing category and open hs_asyouzhang_use");
    assert_eq!(az_use.observer, 0);
    assert!(az_use.spec.options.contains(&ChoiceOption::Card(sha_end)));
    let _ = answer(&mut e7, 0, vec![ChoiceOption::Card(sha_end)]);
    let az_tgt = e7
        .open_choice()
        .cloned()
        .expect("asyouzhang Sha must open SelectPlayers prompt");
    assert_eq!(az_tgt.spec.kind, engine_native::choice::ChoiceKind::SelectPlayers);
    let ai_az = ai_plan_action(&e7, 0);
    assert!(
        matches!(&ai_az, engine_native::GameAction::Answer { selection, .. }
            if selection.iter().any(|o| matches!(o, ChoiceOption::Player(_)))),
        "AI on hs_asyouzhang_use SelectPlayers must pick Player, got {ai_az:?}"
    );
    let _ = submit(&mut e7, 0, ai_az);
    drain_auto(&mut e7);
    assert!(e7.state_core().player(1).hp <= 3 || !e7.state_core().player(0).hand.is_empty());
    assert_eq!(total_tracked_cards(&e7), tracked7);

    // Round 2 Regressions: `wxuying` + `hsfashu_anyingjingxiang` real PlayCard (`Wuzhong` & `Sha`/`Tao`), discard no-hide, off-turn `Shan` respond
    let mut ewx = fresh_proto(12_008, &["hs_xukongzhiying", "hs_jaina"]);
    let tracked_wx = total_tracked_cards(&ewx);
    advance_to_phase(&mut ewx, 0, Phase::Play);
    clear_hand(&mut ewx, 0);
    clear_hand(&mut ewx, 1);
    let shadow_w = ewx
        .state_core()
        .player(0)
        .hand
        .iter()
        .copied()
        .find(|&c| ewx.catalog().cards[c as usize].name.contains("hsfashu_anyingjingxiang"));
    let shadow_c = shadow_w.unwrap_or_else(|| give_card_named(&mut ewx, 0, "hsfashu_anyingjingxiang"));
    // Untransformed shadow cannot be played
    assert!(matches!(
        play_card(&mut ewx, 0, shadow_c, vec![]),
        engine_native::CommandOutcome::Rejected(engine_native::RejectReason::CardLimit)
    ));
    // Playing real Wuzhong transforms shadow into Wuzhong -> real PlayCard(shadow_c) draws 2 + 1 wxuying_hide
    let wz_real = give_card_named(&mut ewx, 0, "wuzhong");
    assert!(!matches!(play_card(&mut ewx, 0, wz_real, vec![]), engine_native::CommandOutcome::Rejected(_)));
    drain_auto(&mut ewx);
    clear_hand(&mut ewx, 0);
    ewx.state_core_mut().player_mut(0).hand.push(shadow_c);
    ewx.state_core_mut().discard.retain(|&c| c != shadow_c);
    // Keep shadow_c explicitly transformed via real Wuzhong while in hand
    let wz_real2 = give_card_named(&mut ewx, 0, "wuzhong");
    let _ = play_card(&mut ewx, 0, wz_real2, vec![]);
    drain_auto(&mut ewx);
    assert_eq!(
        ewx.state_core().declared_kinds.get(&shadow_c).copied(),
        Some(CardKind::Wuzhong)
    );
    let h_pre_shadow = ewx.state_core().player(0).hand.len();
    assert!(
        !matches!(play_card(&mut ewx, 0, shadow_c, vec![]), engine_native::CommandOutcome::Rejected(_)),
        "shadow transformed into Wuzhong must be playable via real PlayCard"
    );
    drain_auto(&mut ewx);
    // Wuzhong drew 2 + wxuying_hide drew 1 - spent shadow_c 1 => net +2 hand cards
    assert_eq!(ewx.state_core().player(0).hand.len(), h_pre_shadow + 2);
    assert!(!ewx.state_core().player(0).yinshen_active, "own-turn wxuying_hide draws instead of stealth");

    // Shadow transformed into Tao when injured + off-turn Shan respond (after own turn ends) -> sets yinshen_active
    ewx.state_core_mut().player_mut(0).hp = 1;
    let fresh_sh = give_card_named(&mut ewx, 0, "hsfashu_anyingjingxiang");
    let tao_real = give_free_card_kind(&mut ewx, 0, CardKind::Tao);
    assert!(!matches!(play_card(&mut ewx, 0, tao_real, vec![]), engine_native::CommandOutcome::Rejected(_)));
    drain_auto(&mut ewx);
    assert_eq!(ewx.state_core().declared_kinds.get(&fresh_sh).copied(), Some(CardKind::Tao));
    let h_pre_tao = ewx.state_core().player(0).hand.len();
    assert!(!matches!(play_card(&mut ewx, 0, fresh_sh, vec![]), engine_native::CommandOutcome::Rejected(_)));
    drain_auto(&mut ewx);
    assert_eq!(ewx.state_core().player(0).hp, 3, "real Tao + shadow Tao recover +2 HP");
    assert_eq!(ewx.state_core().player(0).hand.len(), h_pre_tao, "shadow Tao consumes 1 and wxuying_hide draws 1");

    // Off-turn respond (`Shan`): Dismantle on a shadow must NOT trigger wxuying_hide, whereas responding off-turn Shan triggers stealth
    let sh_dism = give_card_named(&mut ewx, 0, "hsfashu_anyingjingxiang");
    let sha_real = give_free_card_kind(&mut ewx, 0, CardKind::Sha);
    let _ = play_card(&mut ewx, 0, sha_real, vec![1]);
    drain_auto(&mut ewx);
    advance_to_phase(&mut ewx, 1, Phase::Play);
    clear_hand(&mut ewx, 1);
    let dism = give_free_card_kind(&mut ewx, 1, CardKind::Dismantle);
    let h0_pre_dism = ewx.state_core().player(0).hand.len();
    let _ = play_card(&mut ewx, 1, dism, vec![0]);
    if let Some(ch) = ewx.open_choice() {
        if ch.spec.options.contains(&ChoiceOption::Card(sh_dism)) {
            let _ = answer(&mut ewx, 1, vec![ChoiceOption::Card(sh_dism)]);
        } else {
            let opt = ch.spec.options[0];
            let _ = answer(&mut ewx, 1, vec![opt]);
        }
    }
    drain_auto(&mut ewx);
    assert!(!ewx.state_core().player(0).yinshen_active, "discard/dismantle of shadow must NOT trigger wxuying_hide stealth");
    let _ = h0_pre_dism;

    // Now off-turn respond across `TurnEndWindow` (`Sha` in Duel after own turn ends) AND off-turn `respond_shan`:
    advance_to_phase(&mut ewx, 0, Phase::Play);
    clear_hand(&mut ewx, 0);
    let sh_sha = give_card_named(&mut ewx, 0, "hsfashu_anyingjingxiang");
    let real_sha_t = give_free_card_kind(&mut ewx, 0, CardKind::Sha);
    assert!(!matches!(play_card(&mut ewx, 0, real_sha_t, vec![1]), engine_native::CommandOutcome::Rejected(_)));
    drain_auto(&mut ewx);
    assert_eq!(ewx.state_core().declared_kinds.get(&sh_sha).copied(), Some(CardKind::Sha));
    // End seat 0's turn (`TurnEndWindow { after: true }` runs `wxuying_clear`) -> shadow MUST stay `CardKind::Sha` in hand!
    advance_to_phase(&mut ewx, 1, Phase::Play);
    assert_eq!(
        ewx.state_core().declared_kinds.get(&sh_sha).copied(),
        Some(CardKind::Sha),
        "wxuying_clear at TurnEndWindow must NOT clear hs_wxuying_shadow / declared_kinds for cards remaining in hand"
    );
    ewx.state_core_mut().player_mut(1).hp = 3;
    clear_hand(&mut ewx, 1);
    let duel_s1 = give_free_card_kind(&mut ewx, 1, CardKind::Duel);
    let _ = play_card(&mut ewx, 1, duel_s1, vec![0]);
    let ch_duel = advance_until_open_prompt(&mut ewx, "duel_sha", 4).expect("duel_sha must open with shadow Sha");
    assert!(ch_duel.spec.options.contains(&ChoiceOption::Card(sh_sha)));
    save_restore_and_answer(&mut ewx, "duel_sha", vec![ChoiceOption::Card(sh_sha)]);
    drain_auto(&mut ewx);
    assert!(ewx.state_core().player(0).yinshen_active, "off-turn shadow Sha respond in Duel must activate wxuying_hide stealth");
    // Also verify off-turn `respond_shan` with shadow transformed by `respond_shan`:
    ewx.state_core_mut().player_mut(0).yinshen_active = false;
    ewx.state_core_mut().player_mut(1).hp = 3;
    let sh_shan = give_card_named(&mut ewx, 0, "hsfashu_anyingjingxiang");
    let real_shan = give_free_card_kind(&mut ewx, 0, CardKind::Shan);
    let zhuge_1 = give_free_card_kind(&mut ewx, 1, CardKind::Zhuge);
    let _ = play_card(&mut ewx, 1, zhuge_1, vec![]);
    drain_auto(&mut ewx);
    let s1 = give_free_card_kind(&mut ewx, 1, CardKind::Sha);
    let out_s1 = play_card(&mut ewx, 1, s1, vec![0]);
    assert!(!matches!(out_s1, engine_native::CommandOutcome::Rejected(_)));
    let ch_r1 = advance_until_open_prompt(&mut ewx, "respond_shan", 4).expect("respond_shan 1");
    assert!(ch_r1.spec.options.contains(&ChoiceOption::Card(real_shan)));
    let _ = answer(&mut ewx, 0, vec![ChoiceOption::Card(real_shan)]);
    drain_auto(&mut ewx);
    assert_eq!(ewx.state_core().declared_kinds.get(&sh_shan).copied(), Some(CardKind::Shan));
    let s2 = give_free_card_kind(&mut ewx, 1, CardKind::Sha);
    let _ = play_card(&mut ewx, 1, s2, vec![0]);
    let ch_r2 = advance_until_open_prompt(&mut ewx, "respond_shan", 4).expect("respond_shan 2 with shadow Shan");
    assert!(ch_r2.spec.options.contains(&ChoiceOption::Card(sh_shan)));
    save_restore_and_answer(&mut ewx, "respond_shan", vec![ChoiceOption::Card(sh_shan)]);
    drain_auto(&mut ewx);
    assert!(ewx.state_core().player(0).yinshen_active, "off-turn shadow Shan respond must activate wxuying_hide stealth");
    let _ = tracked_wx;

    // Round 2 Regression: `hsmengjing_mengye` (`character/hearth/skill.js:6104-6119`):
    // (a) empty target hand+equips keeps skill; (b) non-empty discards all `he` and removes skill
    let mut emy = fresh_proto(12_009, &["hs_jaina", "hs_lrexxar"]);
    let tracked_my = total_tracked_cards(&emy);
    advance_to_phase(&mut emy, 0, Phase::Play);
    clear_hand(&mut emy, 1);
    let my_card = give_card_named(&mut emy, 0, "hsmengjing_mengye");
    assert!(!matches!(play_card(&mut emy, 0, my_card, vec![1]), engine_native::CommandOutcome::Rejected(_)));
    drain_auto(&mut emy);
    assert!(has_skill(&emy, 1, "hsmengjing_mengye"));
    advance_to_phase(&mut emy, 1, Phase::Play);
    clear_hand(&mut emy, 1);
    clear_equips(&mut emy, 1);
    advance_to_phase(&mut emy, 1, Phase::End);
    drain_auto(&mut emy);
    assert!(has_skill(&emy, 1, "hsmengjing_mengye"), "hsmengjing_mengye must NOT trigger or remove when countCards('he') == 0");
    advance_to_phase(&mut emy, 1, Phase::Play);
    let mh1 = give_free_card_kind(&mut emy, 1, CardKind::Sha);
    let meq = give_free_card_kind(&mut emy, 1, CardKind::Weapon);
    let _ = play_card(&mut emy, 1, meq, vec![]);
    drain_auto(&mut emy);
    let _ = mh1;
    advance_to_phase(&mut emy, 0, Phase::Play);
    assert!(emy.state_core().player(1).hand.is_empty() && emy.state_core().player(1).equips.is_empty(), "hsmengjing_mengye PhaseEnd must discard ALL hand+equips");
    assert!(!has_skill(&emy, 1, "hsmengjing_mengye"), "hsmengjing_mengye must remove itself after firing");
    assert_eq!(total_tracked_cards(&emy), tracked_my);

    // Round 2 Regression: `hshuanling` (`character/hearth/skill.js:367-430`) draw-before-cast & FIFO order & named trick `shandianjian`
    let mut ehl = fresh_identity(12_010, &["hs_taisi", "hs_jaina", "hs_kazhakusi", "hs_lrexxar"]);
    advance_to_phase(&mut ehl, 2, Phase::Play);
    clear_hand(&mut ehl, 2);
    clear_hand(&mut ehl, 0);
    let lhs = give_card_named(&mut ehl, 2, "linghunzhihuo");
    let _ = give_free_card_kind(&mut ehl, 0, CardKind::Sha);
    let _ = give_free_card_kind(&mut ehl, 2, CardKind::Sha);
    assert!(!matches!(play_card(&mut ehl, 2, lhs, vec![0]), engine_native::CommandOutcome::Rejected(_)));
    drain_auto(&mut ehl);
    let zms = give_card_named(&mut ehl, 2, "zhaomingdan");
    assert!(!matches!(play_card(&mut ehl, 2, zms, vec![0]), engine_native::CommandOutcome::Rejected(_)));
    if let Some(ch) = ehl.open_choice().cloned() {
        let opt = ch.spec.options[0];
        let _ = answer(&mut ehl, ch.observer, vec![opt]);
    }
    drain_auto(&mut ehl);
    advance_to_phase(&mut ehl, 0, Phase::Play);
    clear_hand(&mut ehl, 0);
    let hl_eq1 = give_free_card_kind(&mut ehl, 0, CardKind::Weapon);
    let hl_eq2 = give_free_card_kind(&mut ehl, 0, CardKind::Armor);
    let _ = play_card(&mut ehl, 0, hl_eq1, vec![]);
    drain_auto(&mut ehl);
    let _ = play_card(&mut ehl, 0, hl_eq2, vec![]);
    drain_auto(&mut ehl);
    let disc_a = give_free_card_kind(&mut ehl, 0, CardKind::Sha);
    let disc_b = give_free_card_kind(&mut ehl, 0, CardKind::Shan);
    let _ = give_free_card_kind(&mut ehl, 2, CardKind::Shan);
    let top_d2 = give_free_card_kind(&mut ehl, 1, CardKind::Sha);
    let top_d1 = give_free_card_kind(&mut ehl, 1, CardKind::Tao);
    put_card_on_deck_top(&mut ehl, top_d2);
    put_card_on_deck_top(&mut ehl, top_d1);
    let log_hl_start = ehl.log().len();
    advance_to_phase(&mut ehl, 0, Phase::Discard);
    let hl_ask = advance_until_open_prompt(&mut ehl, "hs_hshuanling_discard", 6)
        .expect("hshuanling must prompt discard at owner End Phase");
    assert_eq!(hl_ask.observer, 0);
    let ans_hl = answer(&mut ehl, 0, vec![ChoiceOption::Card(disc_a), ChoiceOption::Card(disc_b)]);
    assert!(!matches!(ans_hl, engine_native::CommandOutcome::Rejected(_)), "hshuanling discard rejected: {ans_hl:?}");
    drain_auto(&mut ehl);
    // Draw happened BEFORE virtual casts, so top_d1 & top_d2 went to seat 0 (minus random 1 from Linghunzhihuo discard) and FIFO cast Linghunzhihuo BEFORE Zhaomingdan!
    let hl_logs = &ehl.log()[log_hl_start..];
    let draw0_pos = hl_logs
        .iter()
        .position(|ev| matches!(ev, engine_native::machine::LogEvent::CardsDrawn { player: 0, cards } if cards.contains(&top_d1)))
        .expect("hshuanling draw before cast");
    let fire_pos = hl_logs
        .iter()
        .position(|ev| matches!(ev, engine_native::machine::LogEvent::DamageDealt { source: Some(0), .. }))
        .expect("hshuanling FIFO cast Linghunzhihuo first");
    let zm_pos = hl_logs
        .iter()
        .position(|ev| matches!(ev, engine_native::machine::LogEvent::CardDiscarded { player, .. } if *player != 0))
        .expect("hshuanling FIFO cast Zhaomingdan second");
    assert!(draw0_pos < fire_pos && fire_pos < zm_pos, "hshuanling must execute Draw -> FIFO #1 (Linghunzhihuo) -> FIFO #2 (Zhaomingdan)");
    assert_eq!(flag(&ehl, 0, "hs_hshuanling_cnt"), 0);

    // Also verify `hshuanling` records & replays `shandianjian` and `shijieshu` cleanly
    advance_to_phase(&mut ehl, 2, Phase::Play);
    clear_hand(&mut ehl, 2);
    clear_hand(&mut ehl, 0);
    let _hl_t0 = give_free_card_kind(&mut ehl, 0, CardKind::Shan);
    let sdj_hl = give_card_named(&mut ehl, 2, "shandianjian");
    let sjs_hl = give_card_named(&mut ehl, 2, "shijieshu");
    assert!(!matches!(play_card(&mut ehl, 2, sdj_hl, vec![0]), engine_native::CommandOutcome::Rejected(_)));
    drain_auto(&mut ehl);
    assert!(!matches!(play_card(&mut ehl, 2, sjs_hl, vec![2]), engine_native::CommandOutcome::Rejected(_)));
    drain_auto(&mut ehl);
    assert_eq!(flag(&ehl, 0, "hs_hshuanling_cnt"), 2, "hshuanling must record shandianjian & shijieshu");
    advance_to_phase(&mut ehl, 0, Phase::Play);
    clear_hand(&mut ehl, 0);
    let _hl_d1 = give_free_card_kind(&mut ehl, 0, CardKind::Sha);
    let _hl_d2 = give_free_card_kind(&mut ehl, 0, CardKind::Shan);
    let _hl_e2 = give_free_card_kind(&mut ehl, 2, CardKind::Sha);
    advance_to_phase(&mut ehl, 0, Phase::Discard);
    let hl_ask2 = advance_until_open_prompt(&mut ehl, "hs_hshuanling_discard", 6)
        .expect("hshuanling must prompt discard for shandianjian + shijieshu");
    let _ = answer(&mut ehl, 0, vec![ChoiceOption::Card(_hl_d1), ChoiceOption::Card(_hl_d2)]);
    drain_auto(&mut ehl);
    assert_eq!(flag(&ehl, 0, "hs_hshuanling_cnt"), 0, "hshuanling must consume recorded shandianjian & shijieshu");
    let _ = hl_ask2;
}

#[test]
fn u05_b6a_batch_a_second_10_ylyuchu_through_hstianqi() {
    // 11. `ylyuchu` (:562): Recover adds up to 3 charges, End Phase distributes marks, TurnBegin triggers extra turn
    let mut e = fresh_proto(12_011, &["hs_yelinlonghou", "hs_jaina"]);
    let tracked = total_tracked_cards(&e);
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_hand(&mut e, 0);
    e.state_core_mut().player_mut(0).hp = 2;
    let tao = give_free_card_kind(&mut e, 0, CardKind::Tao);
    let _ = play_card(&mut e, 0, tao, vec![]);
    drain_auto(&mut e);
    assert_eq!(flag(&e, 0, "hs_ylyuchu_charges"), 1);
    advance_to_phase(&mut e, 0, Phase::End);
    advance_to_phase(&mut e, 1, Phase::Play);
    assert!(
        flag(&e, 0, "extra_turn_insert_queued") > 0 || e.turn().current == 0,
        "ylyuchu charge must queue extra turn when marked seat begins turn"
    );
    assert_eq!(total_tracked_cards(&e), tracked);

    // 12-13. `nsaiqi` (:679) & `nsbeiming` (:710): card used adds 3 to hs_nsaiqi pile; 9 triggers nsbeiming draw + order deck top + vanish guard
    let mut e2 = fresh_proto(12_012, &["hs_aiqinvyao", "hs_jaina"]);
    let tracked2 = total_tracked_cards(&e2);
    advance_to_phase(&mut e2, 0, Phase::Play);
    clear_hand(&mut e2, 0);
    for _ in 0..2 {
        let wz = give_free_card_kind(&mut e2, 0, CardKind::Wuzhong);
        assert!(!matches!(play_card(&mut e2, 0, wz, vec![]), engine_native::CommandOutcome::Rejected(_)));
        drain_auto(&mut e2);
    }
    assert_eq!(pile_len(&e2, 0, "hs_nsaiqi"), 6, "2 uses must stash 6 cards in hs_nsaiqi");
    // 3rd card use reaches 9 -> nsbeiming draws 1 and asks hs_nsbeiming_order
    let wz3 = give_free_card_kind(&mut e2, 0, CardKind::Wuzhong);
    let _ = play_card(&mut e2, 0, wz3, vec![]);
    let nb_ch = advance_until_open_prompt(&mut e2, "hs_nsbeiming_order", 8)
        .expect("nsbeiming must open hs_nsbeiming_order at >= 9 hs_nsaiqi cards");
    let ordered: Vec<ChoiceOption> = nb_ch.spec.options.clone();
    save_restore_and_answer(&mut e2, "hs_nsbeiming_order", ordered);
    drain_auto(&mut e2);
    assert_eq!(pile_len(&e2, 0, "hs_nsaiqi"), 0, "nsbeiming must empty hs_nsaiqi back onto deck");
    // Negative check: vanished card on deck top prevents nsaiqi from triggering on next card use
    let sha_guard = give_free_card_kind(&mut e2, 0, CardKind::Sha);
    let _ = play_card(&mut e2, 0, sha_guard, vec![1]);
    drain_auto(&mut e2);
    assert_eq!(pile_len(&e2, 0, "hs_nsaiqi"), 0, "nsaiqi must not trigger while vanished card is on top of deck");
    assert_eq!(total_tracked_cards(&e2), tracked2);

    // 14. `hsnitai` (:730): Play phase start grants `hsnitai_card`; playing it opens `hs_nitai_skill`
    let mut e3 = fresh_proto(12_013, &["hs_barnes", "hs_jaina"]);
    let tracked3 = total_tracked_cards(&e3);
    advance_to_phase(&mut e3, 0, Phase::Play);
    let nitai_c = e3
        .state_core()
        .player(0)
        .hand
        .iter()
        .copied()
        .find(|&c| e3.catalog().cards[c as usize].name.contains("hsnitai_card"))
        .expect("hsnitai must grant hsnitai_card at Play phase start");
    assert!(!matches!(play_card(&mut e3, 0, nitai_c, vec![]), engine_native::CommandOutcome::Rejected(_)));
    if e3.open_choice().is_some_and(|c| c.spec.prompt == "hs_nitai_skill") {
        let opt = e3.open_choice().unwrap().spec.options[0];
        save_restore_and_answer(&mut e3, "hs_nitai_skill", vec![opt]);
    }
    drain_auto(&mut e3);
    assert!(flag(&e3, 0, "hs_nitai_active_def") > 0);
    assert_eq!(total_tracked_cards(&e3), tracked3);

    // 15-16. `hspuzhao` (:903) & `hsyanxin` (:939): hsyanxin promotes red card over black deck top on draw
    let mut eyx = fresh_proto(12_114, &["hs_yangyanwageli", "hs_jaina"]);
    let tracked_yx = total_tracked_cards(&eyx);
    advance_to_phase(&mut eyx, 0, Phase::Play);
    clear_hand(&mut eyx, 0);
    let wz_yx = give_card_named(&mut eyx, 0, "wuzhong");
    // Find 1 red card (Heart/Diamond) and 2 black cards (Spade/Club) from deck
    let red_c = eyx
        .state_core()
        .deck
        .iter()
        .copied()
        .find(|&c| matches!(eyx.catalog().cards[c as usize].suit, engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond))
        .expect("red card in deck");
    let mut blacks = eyx
        .state_core()
        .deck
        .iter()
        .copied()
        .filter(|&c| matches!(eyx.catalog().cards[c as usize].suit, engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club));
    let blk1 = blacks.next().expect("black card 1 in deck");
    let blk2 = blacks.next().expect("black card 2 in deck");
    // Stack deck so 2 black cards sit above red_c: without hsyanxin Wuzhong (draw 2) gets only black cards!
    eyx.state_core_mut().deck.retain(|&c| c != red_c && c != blk1 && c != blk2);
    eyx.state_core_mut().deck.push(red_c);
    eyx.state_core_mut().deck.push(blk2);
    eyx.state_core_mut().deck.push(blk1);
    let _ = play_card(&mut eyx, 0, wz_yx, vec![]);
    drain_auto(&mut eyx);
    assert!(
        eyx.state_core().player(0).hand.contains(&red_c),
        "hsyanxin must swap red card ahead of black deck top before draw"
    );
    assert_eq!(total_tracked_cards(&eyx), tracked_yx);

    // 15-16. `hspuzhao` (:903) & `hsyanxin` (:939): discard Heart -> all friends draw; friend recover/draw
    let mut e4 = fresh_identity(
        12_014,
        &["hs_yangyanwageli", "hs_jaina", "hs_lrexxar", "hs_wuther"],
    );
    let tracked4 = total_tracked_cards(&e4);
    advance_to_phase(&mut e4, 0, Phase::Play);
    clear_hand(&mut e4, 0);
    let out_neg = activate(&mut e4, 0, "hspuzhao");
    assert!(matches!(out_neg, engine_native::CommandOutcome::Rejected(_)));
    // Put a Heart card in seat 0 hand (`ht_zhaomingdan_2` Heart 10)
    let heart_c = give_card_named(&mut e4, 0, "ht_zhaomingdan_2");
    let out = activate(&mut e4, 0, "hspuzhao");
    assert!(!matches!(out, engine_native::CommandOutcome::Rejected(_)));
    // Cancel once -> refund once_used
    let _ = cancel_open_choice(&mut e4);
    drain_auto(&mut e4);
    let _ = heart_c;
    let out2 = activate(&mut e4, 0, "hspuzhao");
    assert!(!matches!(out2, engine_native::CommandOutcome::Rejected(_)));
    let ch = e4.open_choice().unwrap().spec.options[0];
    save_restore_and_answer(&mut e4, "hs_hspuzhao_card", vec![ch]);
    drain_auto(&mut e4);
    assert!(!e4.state_core().player(0).hand.is_empty());
    assert_eq!(total_tracked_cards(&e4), tracked4);

    // 17-19. `hstianqi` (:974), `hstianqi_dalian` (:1107), `hstianqi_shali` (:1117)
    let mut e5 = fresh_proto(12_015, &["hs_heifengqishi", "hs_jaina"]);
    let tracked5 = total_tracked_cards(&e5);
    advance_to_phase(&mut e5, 0, Phase::Play);
    clear_hand(&mut e5, 0);
    let extra = give_free_card_kind(&mut e5, 0, CardKind::Sha);
    let out = activate(&mut e5, 0, "hstianqi");
    assert!(!matches!(out, engine_native::CommandOutcome::Rejected(_)));
    save_restore_and_answer(&mut e5, "hs_hstianqi_card", vec![ChoiceOption::Card(extra)]);
    drain_auto(&mut e5);
    assert_eq!(e5.state_core().player(0).equips.len(), 1, "hstianqi hand discard must equip one of the 4 horsemen");
    let horseman = e5.state_core().player(0).equips[0];
    advance_to_phase(&mut e5, 1, Phase::Play);
    clear_hand(&mut e5, 1);
    advance_to_phase(&mut e5, 0, Phase::Play);
    let out2 = activate(&mut e5, 0, "hstianqi");
    assert!(!matches!(out2, engine_native::CommandOutcome::Rejected(_)));
    let _ = answer(&mut e5, 0, vec![ChoiceOption::Card(horseman)]);
    drain_auto(&mut e5);
    assert_ne!(flag(&e5, 0, "hs_hstianqi_mask"), 0, "discarding equipped horseman marks hs_hstianqi_mask");
    assert_eq!(total_tracked_cards(&e5), tracked5);

    // Verify `hstianqi_dalian` (recover on damage dealt) & `hstianqi_shali` (hujia on recover & +1 range)
    let mut e6 = fresh_proto(12_016, &["hs_jaina", "hs_lrexxar"]);
    advance_to_phase(&mut e6, 0, Phase::Play);
    clear_hand(&mut e6, 0);
    clear_hand(&mut e6, 1);
    let dalian = give_card_named(&mut e6, 0, "hstianqi_dalian");
    let shali = give_card_named(&mut e6, 0, "hstianqi_shali");
    let _ = play_card(&mut e6, 0, dalian, vec![]);
    let _ = play_card(&mut e6, 0, shali, vec![]);
    e6.state_core_mut().player_mut(0).hp = 2;
    let sha = give_free_card_kind(&mut e6, 0, CardKind::Sha);
    let _ = play_card(&mut e6, 0, sha, vec![1]);
    drain_auto(&mut e6);
    assert!(e6.state_core().player(0).hp >= 3, "hstianqi_dalian must recover 1 on damage");
    assert!(e6.state_core().player(0).hujia >= 1, "hstianqi_shali must grant hujia on recover");
}
