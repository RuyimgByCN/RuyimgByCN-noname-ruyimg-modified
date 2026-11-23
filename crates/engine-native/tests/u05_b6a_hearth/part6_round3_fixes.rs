//! Batch U05-6a Round-3 Regressions (`u05_b6a_hearth` Part 6):
//! Upstream references:
//! - `/Users/ruyimg/Documents/Claude/sanguosha/archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:11-983`
//!   (`hsqizhou_huo`:269, `hsqizhou_tu`:293-296, `hsqizhou_shui`:240, `hsqingyu_shandian`:406,
//!    `hsdusu_huoyanhua`:531-532, `hsshenqi_nengliangzhiguang`:667-671, `hsmengjing_mengye`:829,
//!    `hsmengjing_mengjing`:854-856, `hszuzhou_nvwudepingguo`:963, `hszuzhou_nvwudexuetu`:981-983,
//!    `tuteng1..=8`:1087-1136)
//! - `/Users/ruyimg/Documents/Claude/sanguosha/archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js:210,342`
//!   (`shandianjian`, `shihuawuqi`)
//! - `/Users/ruyimg/Documents/Claude/sanguosha/archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/skill.js:57-129,367-464,3688-3716,3859-3885`
//!   (`wxuying`, `hshuanling`, `zengli`, `tanmi`)

use engine_native::catalog::CardKind;
use engine_native::choice::ChoiceKind;
use engine_native::event::Phase;
use engine_native::{ChoiceOption, CommandOutcome, RejectReason};

use super::common::*;

#[test]
fn u05_b6a_r3_target_filters_and_tanmi_xiyong_multi_target() {
    // 1. Direct PlayCard validation & negative filters for targeted derived Hearth cards
    let mut e = fresh_proto(16_001, &["hs_lreno", "hs_jaina", "hs_fachaotuteng", "hs_yelinlonghou"]);
    let tracked = total_tracked_cards(&e);
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 1);
    clear_hand(&mut e, 2);

    for cname in [
        "hsqizhou_huo",
        "hsqizhou_tu",
        "hsqingyu_shandian",
        "hsdusu_huoyanhua",
        "hsshenqi_nengliangzhiguang",
        "hsmengjing_mengye",
        "hsmengjing_mengjing",
        "hszuzhou_nvwudepingguo",
        "hszuzhou_nvwudexuetu",
    ] {
        let c = give_card_named(&mut e, 0, cname);
        assert!(
            matches!(play_card(&mut e, 0, c, vec![]), CommandOutcome::Rejected(RejectReason::IllegalTargets)),
            "{cname} must reject empty targets vec![]"
        );
    }

    // `hsdusu_huoyanhua` (`character/hearth/card.js:531-532`): `range: { attack: 1 }` rejects dist=2 (`seat 2`), allows dist=1 (`seat 1`)
    let hyh = give_card_named(&mut e, 0, "hsdusu_huoyanhua");
    assert!(
        matches!(play_card(&mut e, 0, hyh, vec![2]), CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "hsdusu_huoyanhua must reject out-of-range seat 2 (dist=2 > attack_range=1)"
    );
    let hp1_before = e.state_core().player(1).hp;
    assert!(!matches!(play_card(&mut e, 0, hyh, vec![1]), CommandOutcome::Rejected(_)));
    drain_auto(&mut e);
    assert_eq!(e.state_core().player(1).hp, hp1_before - 1);

    // `hsmengjing_mengjing` (:854-856): rejects when target already has `lebu` & 0 equips; allows with equip
    let le = give_free_card_kind(&mut e, 1, CardKind::Le);
    e.state_core_mut().player_mut(1).hand.retain(|&c| c != le);
    e.state_core_mut().player_mut(1).judge.push(le);
    let mj = give_card_named(&mut e, 0, "hsmengjing_mengjing");
    assert!(
        matches!(play_card(&mut e, 0, mj, vec![1]), CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "hsmengjing_mengjing must reject target having Le and 0 equips"
    );
    let eq1 = give_free_card_kind(&mut e, 1, CardKind::Armor);
    e.state_core_mut().player_mut(1).hand.retain(|&c| c != eq1);
    e.state_core_mut().player_mut(1).equips.push(eq1);
    assert!(!matches!(play_card(&mut e, 0, mj, vec![1]), CommandOutcome::Rejected(_)));
    drain_auto(&mut e);
    assert!(e.state_core().player(1).equips.is_empty(), "hsmengjing_mengjing must return equips to hand");
    assert!(e.state_core().player(1).hand.contains(&eq1));

    // `hszuzhou_nvwudexuetu` (:981-983): grants `zhoujiang`; rejects target already having `zhoujiang`
    let xt1 = give_card_named(&mut e, 0, "hszuzhou_nvwudexuetu");
    assert!(!matches!(play_card(&mut e, 0, xt1, vec![1]), CommandOutcome::Rejected(_)));
    drain_auto(&mut e);
    assert!(has_skill(&e, 1, "zhoujiang"));
    let xt2 = give_card_named(&mut e, 0, "hszuzhou_nvwudexuetu");
    assert!(
        matches!(play_card(&mut e, 0, xt2, vec![1]), CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "hszuzhou_nvwudexuetu must reject target already possessing zhoujiang"
    );

    // `hsshenqi_nengliangzhiguang` (:667-671) flips caster `face_down`, gating all three `hsshenqi_*`
    let nlg1 = give_card_named(&mut e, 0, "hsshenqi_nengliangzhiguang");
    assert!(!matches!(play_card(&mut e, 0, nlg1, vec![2]), CommandOutcome::Rejected(_)));
    drain_auto(&mut e);
    assert!(e.state_core().player(0).face_down);
    for sq_name in [
        "hsshenqi_nengliangzhiguang",
        "hsshenqi_kongbusangzhong",
        "hsshenqi_morijingxiang",
    ] {
        let sq = give_card_named(&mut e, 0, sq_name);
        let t = if sq_name == "hsshenqi_nengliangzhiguang" { vec![2] } else { vec![] };
        assert!(
            matches!(play_card(&mut e, 0, sq, t), CommandOutcome::Rejected(RejectReason::CardLimit)),
            "{sq_name} must reject with CardLimit while face_down == true"
        );
    }
    e.state_core_mut().player_mut(0).face_down = false;

    // `hsqizhou_tu` (:293-296): rejects self `[0]`, allows multi-target `[1, 2]`
    let tu = give_card_named(&mut e, 0, "hsqizhou_tu");
    assert!(matches!(play_card(&mut e, 0, tu, vec![0]), CommandOutcome::Rejected(RejectReason::IllegalTargets)));
    let h1_hu = e.state_core().player(1).hujia;
    let h2_hu = e.state_core().player(2).hujia;
    assert!(!matches!(play_card(&mut e, 0, tu, vec![1, 2]), CommandOutcome::Rejected(_)));
    drain_auto(&mut e);
    assert_eq!(e.state_core().player(1).hujia, h1_hu + 1);
    assert_eq!(e.state_core().player(2).hujia, h2_hu + 1);

    // 2. `tanmi` picking `hsmengjing_mengye` MUST open `hs_tanmi_target` & apply curse to target (NOT owner!)
    let mut etm = fresh_proto(16_011, &["hs_lreno", "hs_lrexxar", "hs_fachaotuteng", "hs_yelinlonghou"]);
    let tracked_tm = total_tracked_cards(&etm);
    advance_to_phase(&mut etm, 1, Phase::Discard);
    clear_hand(&mut etm, 0);
    let my_card = give_card_named(&mut etm, 2, "hsmengjing_mengye");
    let tu_card = give_card_named(&mut etm, 2, "hsqizhou_tu");
    put_card_on_deck_top(&mut etm, tu_card);
    put_card_on_deck_top(&mut etm, my_card);
    let tm_conf = advance_until_open_prompt(&mut etm, "trigger_confirm", 6)
        .expect("tanmi must open trigger_confirm when seat 0 hand empty");
    assert_eq!(tm_conf.observer, 0);
    let _ = answer(&mut etm, 0, vec![ChoiceOption::Confirm]);
    let tm_use = etm.open_choice().cloned().expect("hs_tanmi_use");
    assert_eq!(tm_use.spec.prompt, "hs_tanmi_use");
    assert!(
        tm_use.spec.options.contains(&ChoiceOption::Card(my_card)),
        "tanmi options={:?}, my_card={my_card}, tu_card={tu_card}, hand0={:?}, turn={:?}",
        tm_use.spec.options,
        etm.state_core().player(0).hand,
        etm.turn()
    );
    assert!(tm_use.spec.options.contains(&ChoiceOption::Card(tu_card)));
    let _ = answer(&mut etm, 0, vec![ChoiceOption::Card(my_card)]);
    let tm_tgt = etm.open_choice().cloned().expect("hs_tanmi_target for hsmengjing_mengye");
    assert_eq!(tm_tgt.spec.prompt, "hs_tanmi_target");
    assert_eq!(tm_tgt.spec.kind, ChoiceKind::SelectPlayers);
    save_restore_and_answer(&mut etm, "hs_tanmi_target", vec![ChoiceOption::Player(1)]);
    assert!(has_skill(&etm, 1, "hsmengjing_mengye"), "tanmi hsmengjing_mengye must target chosen seat 1");
    assert!(!has_skill(&etm, 0, "hsmengjing_mengye"), "tanmi hsmengjing_mengye must NOT target owner 0");

    // 2nd use inside same `tanmi`: multi-target `hsqizhou_tu` allows picking seats 1 & 2 (`max >= 2`)
    let tm_use2 = etm.open_choice().cloned().expect("2nd hs_tanmi_use");
    assert_eq!(tm_use2.spec.prompt, "hs_tanmi_use");
    let _ = answer(&mut etm, 0, vec![ChoiceOption::Card(tu_card)]);
    let tm_tgt2 = etm.open_choice().cloned().expect("hs_tanmi_target for multi-target hsqizhou_tu");
    assert_eq!(tm_tgt2.spec.prompt, "hs_tanmi_target");
    assert!(!tm_tgt2.spec.options.contains(&ChoiceOption::Player(0)));
    assert!(tm_tgt2.spec.max >= 2, "hsqizhou_tu must allow selecting multiple non-self targets");
    let tu_h1 = etm.state_core().player(1).hujia;
    let tu_h2 = etm.state_core().player(2).hujia;
    save_restore_and_answer(
        &mut etm,
        "hs_tanmi_target",
        vec![ChoiceOption::Player(1), ChoiceOption::Player(2)],
    );
    drain_auto(&mut etm);
    assert_eq!(etm.state_core().player(1).hujia, tu_h1 + 1);
    assert_eq!(etm.state_core().player(2).hujia, tu_h2 + 1);

    // 3. `xiyong` picking `hsmengjing_mengye` MUST open `hs_xiyong_use` SelectPlayers (NOT self-curse!)
    advance_to_phase(&mut etm, 2, Phase::Discard);
    let my_xy = give_card_named(&mut etm, 1, "hsmengjing_mengye");
    put_card_on_deck_top(&mut etm, my_xy);
    let xy_conf = advance_until_open_prompt(&mut etm, "trigger_confirm", 6)
        .expect("xiyong on seat 2 must prompt trigger_confirm");
    assert_eq!(xy_conf.observer, 2);
    let _ = answer(&mut etm, 2, vec![ChoiceOption::Confirm]);
    let xy_use = etm.open_choice().cloned().expect("hs_xiyong_use card pick");
    assert_eq!(xy_use.spec.prompt, "hs_xiyong_use");
    let _ = answer(&mut etm, 2, vec![ChoiceOption::Card(my_xy)]);
    let xy_tgt = etm.open_choice().cloned().expect("hs_xiyong_use target pick for hsmengjing_mengye");
    assert_eq!(xy_tgt.spec.kind, ChoiceKind::SelectPlayers);
    save_restore_and_answer(&mut etm, "hs_xiyong_use", vec![ChoiceOption::Player(3)]);
    drain_auto(&mut etm);
    assert!(has_skill(&etm, 3, "hsmengjing_mengye"), "xiyong hsmengjing_mengye must target chosen seat 3");
    assert!(!has_skill(&etm, 2, "hsmengjing_mengye"), "xiyong hsmengjing_mengye must NOT self-target seat 2");
    assert_eq!(total_tracked_cards(&etm), tracked_tm);
    assert_eq!(total_tracked_cards(&e), tracked);
}

#[test]
fn u05_b6a_r3_hshuanling_empty_deck_after_draw_and_all_tricks() {
    let mut ehl = fresh_identity(12_010, &["hs_taisi", "hs_jaina", "hs_kazhakusi", "hs_lrexxar"]);
    let tracked = total_tracked_cards(&ehl);
    advance_to_phase(&mut ehl, 2, Phase::Play);
    clear_hand(&mut ehl, 2);
    clear_hand(&mut ehl, 0);
    let my = give_card_named(&mut ehl, 2, "hsmengjing_mengye");
    assert!(
        !matches!(play_card(&mut ehl, 2, my, vec![0]), CommandOutcome::Rejected(_)),
        "enemy seat 2 hsmengjing_mengye targeting 0 must play cleanly"
    );
    drain_auto(&mut ehl);
    assert_eq!(flag(&ehl, 0, "hs_hshuanling_cnt"), 1, "hshuanling must record enemy hsmengjing_mengye");

    // Empty BOTH deck (leave 1 card) & discard (park rest in seat 1 hand cleanly) so `start_draw(0, 1)` leaves `deck.is_empty() == true` without reshuffling!
    advance_to_phase(&mut ehl, 0, Phase::Play);
    clear_hand(&mut ehl, 0);
    let disc = give_free_card_kind(&mut ehl, 0, CardKind::Sha);
    while ehl.state_core().deck.len() > 1 {
        let c = ehl.state_core_mut().deck.pop().unwrap();
        ehl.state_core_mut().player_mut(1).hand.push(c);
    }
    let old_disc: Vec<u32> = ehl.state_core_mut().discard.drain(..).collect();
    ehl.state_core_mut().player_mut(1).hand.extend(old_disc);
    assert_eq!(ehl.state_core().deck.len(), 1);
    assert!(ehl.state_core().discard.is_empty());
    advance_to_phase(&mut ehl, 0, Phase::Discard);
    let hl_ask = advance_until_open_prompt(&mut ehl, "hs_hshuanling_discard", 6)
        .expect("hshuanling must prompt discard even with 1 card in deck");
    assert_eq!(hl_ask.observer, 0);
    save_restore_and_answer(&mut ehl, "hs_hshuanling_discard", vec![ChoiceOption::Card(disc)]);
    drain_auto(&mut ehl);
    assert!(ehl.state_core().deck.is_empty(), "deck must now be empty after start_draw");
    assert_eq!(flag(&ehl, 0, "hs_hshuanling_cnt"), 0, "hshuanling MUST STILL cast recorded trick when deck became empty");
    assert!(
        (1..4u8).any(|p| has_skill(&ehl, p, "hsmengjing_mengye")),
        "hshuanling virtual hsmengjing_mengye must still grant skill to an enemy even with empty deck"
    );
    assert_eq!(total_tracked_cards(&ehl), tracked);
}

#[test]
fn u05_b6a_r3_tanmi_asyouzhang_filtering_and_shihuawuqi_full_hp() {
    let mut e = fresh_proto(16_003, &["hs_lreno", "hs_jaina"]);
    let tracked = total_tracked_cards(&e);

    // 1. Blank `hsfashu_anyingjingxiang` & `tuteng1` drawn by `tanmi` -> NEVER usable, NO `hs_tanmi_use` prompt
    advance_to_phase(&mut e, 1, Phase::Discard);
    clear_hand(&mut e, 0);
    let tuteng = give_card_named(&mut e, 1, "tuteng1");
    let blank_sh = give_card_named(&mut e, 1, "hsfashu_anyingjingxiang");
    put_card_on_deck_top(&mut e, tuteng);
    put_card_on_deck_top(&mut e, blank_sh);
    let c1 = advance_until_open_prompt(&mut e, "trigger_confirm", 6).expect("tanmi confirm 1");
    assert_eq!(c1.observer, 0);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    assert!(
        e.open_choice().is_none_or(|c| c.spec.prompt != "hs_tanmi_use"),
        "tanmi must NOT prompt hs_tanmi_use for blank hsfashu_anyingjingxiang + tuteng1"
    );
    drain_auto(&mut e);

    // 2. Full HP (`hp == max_hp`): `hsqizhou_shui` is UNUSABLE while `shihuawuqi` IS USABLE!
    advance_to_phase(&mut e, 1, Phase::Discard);
    clear_hand(&mut e, 0);
    e.state_core_mut().player_mut(0).hp = e.state_core().player(0).max_hp as i32;
    let shui = give_card_named(&mut e, 1, "hsqizhou_shui");
    let shwq = give_card_named(&mut e, 1, "shihuawuqi");
    let sha_in_deck = give_free_card_kind(&mut e, 1, CardKind::Sha);
    put_card_on_deck_top(&mut e, sha_in_deck);
    let idx = e.state_core().deck.len().saturating_sub(2);
    e.state_core_mut().deck.retain(|&c| c != sha_in_deck);
    e.state_core_mut().deck.insert(idx, sha_in_deck);
    put_card_on_deck_top(&mut e, shui);
    put_card_on_deck_top(&mut e, shwq);
    let c2 = advance_until_open_prompt(&mut e, "trigger_confirm", 6).expect("tanmi confirm 2");
    assert_eq!(c2.observer, 0);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    let tm_use = e.open_choice().cloned().expect("hs_tanmi_use must open for shihuawuqi@fullHP");
    assert_eq!(tm_use.spec.prompt, "hs_tanmi_use");
    assert!(tm_use.spec.options.contains(&ChoiceOption::Card(shwq)), "shihuawuqi must be usable at full HP in tanmi");
    assert!(!tm_use.spec.options.contains(&ChoiceOption::Card(shui)), "hsqizhou_shui must be excluded at full HP in tanmi");
    save_restore_and_answer(&mut e, "hs_tanmi_use", vec![ChoiceOption::Card(shwq)]);
    drain_auto(&mut e);
    assert!(has_skill(&e, 0, "shihuawuqi"), "tanmi shihuawuqi at full HP must grant shihuawuqi skill");
    assert!(
        e.state_core().player(0).hand.iter().any(|&c| e.catalog().card_kind(c).is_sha_like()),
        "tanmi shihuawuqi must draw 1 Sha from deck"
    );

    // 3. `asyouzhang` on `hs_ashamoer` accepts `shihuawuqi` at full HP!
    let mut eay = fresh_proto(16_004, &["hs_ashamoer", "hs_jaina"]);
    let tracked_ay = total_tracked_cards(&eay);
    advance_to_phase(&mut eay, 0, Phase::Play);
    clear_hand(&mut eay, 0);
    // Keep a trick and equip in hand so `asyouzhang` ONLY searches for basic -> finds `shihuawuqi`!
    let _hk_tr = give_free_card_kind(&mut eay, 0, CardKind::Wuzhong);
    let _hk_eq = give_free_card_kind(&mut eay, 0, CardKind::Armor);
    let shwq_ay = give_card_named(&mut eay, 1, "shihuawuqi");
    // Ensure top basic from bottom-reverse search is `shihuawuqi`: remove other basics temporarily to discard, leave 1 sha for shihuawuqi draw!
    let deck_basics: Vec<u32> = eay
        .state_core()
        .deck
        .iter()
        .copied()
        .filter(|&c| eay.catalog().card_kind(c).is_basic_kind())
        .collect();
    for bc in &deck_basics[..deck_basics.len().saturating_sub(1)] {
        eay.state_core_mut().deck.retain(|&c| c != *bc);
        eay.state_core_mut().discard.push(*bc);
    }
    put_card_on_deck_top(&mut eay, shwq_ay);
    eay.state_core_mut().player_mut(0).hp = eay.state_core().player(0).max_hp as i32;
    advance_to_phase(&mut eay, 0, Phase::Discard);
    let ay_conf = advance_until_open_prompt(&mut eay, "trigger_confirm", 6).expect("asyouzhang confirm");
    assert_eq!(ay_conf.observer, 0);
    let _ = answer(&mut eay, 0, vec![ChoiceOption::Confirm]);
    let ay_use = eay.open_choice().cloned().expect("hs_asyouzhang_use must open for shihuawuqi@fullHP");
    assert_eq!(ay_use.spec.prompt, "hs_asyouzhang_use");
    assert!(ay_use.spec.options.contains(&ChoiceOption::Card(shwq_ay)));
    save_restore_and_answer(&mut eay, "hs_asyouzhang_use", vec![ChoiceOption::Card(shwq_ay)]);
    drain_auto(&mut eay);
    assert!(has_skill(&eay, 0, "shihuawuqi"));
    assert_eq!(total_tracked_cards(&e), tracked);
    assert_eq!(total_tracked_cards(&eay), tracked_ay);
}

#[test]
fn u05_b6a_r3_wxuying_preserves_hearth_wuzhong_shihuawuqi_zengli_and_huogong() {
    let mut ewx = fresh_proto(16_005, &["hs_xukongzhiying", "hs_jaina"]);
    let tracked = total_tracked_cards(&ewx);
    advance_to_phase(&mut ewx, 0, Phase::Play);
    clear_hand(&mut ewx, 0);
    clear_hand(&mut ewx, 1);

    // A. `wxuying` copying `shandianjian` (`CardKind::Wuzhong`): MUST preserve `shandianjian` identity!
    let sh1 = give_card_named(&mut ewx, 0, "hsfashu_anyingjingxiang");
    let sdj = give_card_named(&mut ewx, 0, "shandianjian");
    let target_hand = give_free_card_kind(&mut ewx, 1, CardKind::Shan);
    let suit = ewx.catalog().cards[target_hand as usize].suit;
    let caster_match = {
        let cat = ewx.catalog();
        (0..cat.cards.len() as u32)
            .find(|&c| {
                c != sdj
                    && c != sh1
                    && c != target_hand
                    && !cat.cards[c as usize].off_deck
                    && cat.cards[c as usize].suit == suit
                    && (ewx.state_core().deck.contains(&c) || ewx.state_core().discard.contains(&c))
            })
            .unwrap()
    };
    ewx.state_core_mut().deck.retain(|&c| c != caster_match);
    ewx.state_core_mut().discard.retain(|&c| c != caster_match);
    ewx.state_core_mut().player_mut(0).hand.push(caster_match);
    let _ = play_card(&mut ewx, 0, sdj, vec![1]);
    drain_auto(&mut ewx);
    assert_eq!(ewx.state_core().declared_kinds.get(&sh1).copied(), Some(CardKind::Wuzhong));
    // Playing copied shandianjian with empty targets MUST reject; playing with vec![1] MUST open `hs_shandianjian_discard` & deal Thunder dmg!
    assert!(
        matches!(play_card(&mut ewx, 0, sh1, vec![]), CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "wxuying copying shandianjian must enforce shandianjian target rule"
    );
    let hp1_pre = ewx.state_core().player(1).hp;
    let h0_pre = ewx.state_core().player(0).hand.len();
    assert!(
        !matches!(play_card(&mut ewx, 0, sh1, vec![1]), CommandOutcome::Rejected(_)),
        "wxuying copying shandianjian must play cleanly with vec![1]"
    );
    // Target hand has 1 card -> auto-revealed, prompts `hs_shandianjian_discard` for seat 0
    let ch = ewx.open_choice().cloned().expect("copied shandianjian must open hs_shandianjian_discard");
    assert_eq!(ch.spec.prompt, "hs_shandianjian_discard");
    let ChoiceOption::Card(mc) = ch.spec.options[0] else {
        panic!("expected Card option in hs_shandianjian_discard");
    };
    save_restore_and_answer(&mut ewx, "hs_shandianjian_discard", vec![ChoiceOption::Card(mc)]);
    drain_auto(&mut ewx);
    assert_eq!(ewx.state_core().player(1).hp, hp1_pre - 1, "copied shandianjian must deal 1 Thunder damage");
    assert_eq!(ewx.state_core().player(0).hand.len(), h0_pre - 1, "spent sh1 (-1) + discard (-1) + wxuying_hide draw (+1)");

    // B. `wxuying` copying `hsmengjing_mengye`: grants seat 1 `hsmengjing_mengye` + draws 1 + wxuying_hide draw 1
    clear_hand(&mut ewx, 0);
    let sh2 = give_card_named(&mut ewx, 0, "hsfashu_anyingjingxiang");
    let my_real = give_card_named(&mut ewx, 0, "hsmengjing_mengye");
    let _ = play_card(&mut ewx, 0, my_real, vec![1]);
    drain_auto(&mut ewx);
    // Play copied sh2 targeting seat 1 (`vec![1]`, NOT vec![])
    assert!(matches!(play_card(&mut ewx, 0, sh2, vec![]), CommandOutcome::Rejected(RejectReason::IllegalTargets)));
    let h1_pre_my = ewx.state_core().player(1).hand.len();
    assert!(!matches!(play_card(&mut ewx, 0, sh2, vec![1]), CommandOutcome::Rejected(_)));
    drain_auto(&mut ewx);
    assert!(has_skill(&ewx, 1, "hsmengjing_mengye"), "copied hsmengjing_mengye must grant hsmengjing_mengye to target");
    assert_eq!(ewx.state_core().player(1).hand.len(), h1_pre_my + 1, "copied hsmengjing_mengye must make target draw 1");

    // C. `wxuying` copying `shihuawuqi` AT FULL HP (`hp == max_hp`): MUST preserve `shihuawuqi` identity
    //    (reaching `CardLimit`, NOT `HpFull`) and enforce `shihuawuqi`'s once-per-turn cap on `sh3`!
    clear_hand(&mut ewx, 0);
    ewx.state_core_mut().player_mut(0).hp = ewx.state_core().player(0).max_hp as i32;
    let sh3 = give_card_named(&mut ewx, 0, "hsfashu_anyingjingxiang");
    let shwq_real = give_card_named(&mut ewx, 0, "shihuawuqi");
    let _ = play_card(&mut ewx, 0, shwq_real, vec![]);
    drain_auto(&mut ewx);
    assert!(
        ewx.state_core().player(0).hand.iter().any(|&c| ewx.catalog().card_kind(c).is_sha_like()),
        "shihuawuqi must gain 1 Sha from deck"
    );
    assert_eq!(ewx.state_core().declared_kinds.get(&sh3).copied(), Some(CardKind::Tao));
    let non_sh3: Vec<u32> = ewx.state_core().player(0).hand.iter().copied().filter(|&c| c != sh3).collect();
    for c in non_sh3 {
        ewx.state_core_mut().player_mut(0).hand.retain(|&x| x != c);
        ewx.state_core_mut().discard.push(c);
    }
    assert!(
        matches!(play_card(&mut ewx, 0, sh3, vec![]), CommandOutcome::Rejected(RejectReason::CardLimit)),
        "wxuying copying shihuawuqi MUST recognize shihuawuqi at full HP (rejecting CardLimit, NOT HpFull)"
    );
    drain_auto(&mut ewx);

    // D. `wxuying` copying `hsdusu_xueji`: requires target with equips (`IllegalTargets` otherwise) & discards target's equip
    clear_hand(&mut ewx, 0);
    clear_hand(&mut ewx, 1);
    let sh4 = give_card_named(&mut ewx, 0, "hsfashu_anyingjingxiang");
    let eq_v1 = give_free_card_kind(&mut ewx, 1, CardKind::Armor);
    ewx.state_core_mut().player_mut(1).hand.retain(|&c| c != eq_v1);
    ewx.state_core_mut().player_mut(1).equips.push(eq_v1);
    let xj_real = give_card_named(&mut ewx, 0, "hsdusu_xueji");
    let _ = play_card(&mut ewx, 0, xj_real, vec![1]);
    drain_auto(&mut ewx);
    assert_eq!(ewx.state_core().declared_kinds.get(&sh4).copied(), Some(CardKind::Wuzhong));
    assert!(matches!(play_card(&mut ewx, 0, sh4, vec![1]), CommandOutcome::Rejected(RejectReason::IllegalTargets)));
    let eq_v2 = give_free_card_kind(&mut ewx, 1, CardKind::Weapon);
    ewx.state_core_mut().player_mut(1).hand.retain(|&c| c != eq_v2);
    ewx.state_core_mut().player_mut(1).equips.push(eq_v2);
    assert!(!matches!(play_card(&mut ewx, 0, sh4, vec![1]), CommandOutcome::Rejected(_)));
    drain_auto(&mut ewx);
    assert!(ewx.state_core().player(1).equips.is_empty(), "copied hsdusu_xueji must discard target equip");
    assert_eq!(total_tracked_cards(&ewx), tracked);

    // D2. Coexistence formal pack (`shahai-hearth` + `taixu`) verifying `wxuying` copying `huogong` uses `discard_up_to` on equip when hand is empty!
    let load_frozen = |dir: &str| {
        let pkg_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/extensions")
            .join(dir);
        let raw_json = std::fs::read_to_string(pkg_dir.join("package.npack.json"))
            .unwrap()
            .replace("\r\n", "\n");
        let parsed: serde_json::Value = serde_json::from_str(&raw_json).unwrap();
        let mut assets = std::collections::BTreeMap::new();
        if let Some(paths) = parsed.get("asset_paths").and_then(|v| v.as_object()) {
            for rel in paths.values().filter_map(|v| v.as_str()) {
                let bytes: Vec<u8> = std::fs::read(pkg_dir.join(rel))
                    .unwrap()
                    .into_iter()
                    .filter(|&b| b != b'\r')
                    .collect();
                assets.insert(rel.to_string(), bytes);
            }
        }
        content_native_loader::validate_and_freeze_raw_package(raw_json, assets).unwrap()
    };
    let dual_verified = content_native_loader::prepare_locked_content_with_bridge(vec![
        load_frozen("shahai-hearth-upstream-noname"),
        load_frozen("taixu-upstream-liuli"),
    ])
    .expect("prepare hearth+taixu locked content");
    let mut ehg = content_native_loader::start_locked_engine(
        16_007,
        2,
        content_native_loader::LockedStartMode::Prototype,
        &dual_verified,
    )
    .expect("start dual engine");
    ehg.assign_general_named(0, "hs_xukongzhiying").unwrap();
    ehg.assign_general_named(1, "hs_jaina").unwrap();
    let tracked_hg = total_tracked_cards(&ehg);
    advance_to_phase(&mut ehg, 0, Phase::Play);
    clear_hand(&mut ehg, 0);
    clear_hand(&mut ehg, 1);
    let _hg_disc = give_free_card_kind(&mut ehg, 0, CardKind::Sha);
    let sh_hg = give_card_named(&mut ehg, 0, "hsfashu_anyingjingxiang");
    let hg_real = give_card_named(&mut ehg, 0, "huogong");
    let _ = give_free_card_kind(&mut ehg, 1, CardKind::Shan);
    let out_hg_real = play_card(&mut ehg, 0, hg_real, vec![1]);
    assert!(!matches!(out_hg_real, CommandOutcome::Rejected(_)), "hg_real rejected: {out_hg_real:?}");
    drain_auto(&mut ehg);
    let non_sh_hg: Vec<u32> = ehg.state_core().player(0).hand.iter().copied().filter(|&c| c != sh_hg).collect();
    for c in non_sh_hg {
        ehg.state_core_mut().player_mut(0).hand.retain(|&x| x != c);
        ehg.state_core_mut().discard.push(c);
    }
    let eq0 = give_free_card_kind(&mut ehg, 0, CardKind::Armor);
    ehg.state_core_mut().player_mut(0).hand.retain(|&c| c != eq0);
    ehg.state_core_mut().player_mut(0).equips.push(eq0);
    let hp1_hg = ehg.state_core().player(1).hp;
    let out_sh_hg = play_card(&mut ehg, 0, sh_hg, vec![1]);
    assert!(
        !matches!(out_sh_hg, CommandOutcome::Rejected(_)),
        "sh_hg rejected: {out_sh_hg:?}, decl={:?}, flags={:?}",
        ehg.state_core().declared_kinds,
        ehg.state_core().vm_aux.flags
    );
    drain_auto(&mut ehg);
    assert_eq!(ehg.state_core().player(1).hp, hp1_hg - 1, "copied huogong with empty hand must discard_up_to equip & deal 1 Fire dmg");
    assert!(ehg.state_core().player(0).equips.is_empty(), "copied huogong must discard equip when hand empty");
    assert_eq!(total_tracked_cards(&ehg), tracked_hg);

    // E. `zengli` over 12 activations across turns NEVER equips non-inpile `hsqingyu_zhao` and NEVER exhausts after 9 uses
    let mut ez = fresh_proto(16_006, &["hs_blingtron", "hs_jaina", "hs_lrexxar"]);
    let tracked_z = total_tracked_cards(&ez);
    for round in 0..12 {
        advance_to_phase(&mut ez, 0, Phase::Play);
        clear_hand(&mut ez, 0);
        clear_hand(&mut ez, 1);
        clear_hand(&mut ez, 2);
        ez.state_core_mut().player_mut(1).hp = 2;
        ez.state_core_mut().player_mut(2).hp = 4;
        let out = activate(&mut ez, 0, "zengli");
        assert!(!matches!(out, CommandOutcome::Rejected(_)), "zengli round {round} rejected: {out:?}");
        let _ = answer(&mut ez, 0, vec![ChoiceOption::Player(2)]);
        drain_auto(&mut ez);
        let wp0 = ez.state_core().player(0).equips.iter().copied().find(|&c| ez.catalog().card_kind(c).equip_slot() == Some(engine_native::catalog::EquipSlot::Weapon));
        let wp2 = ez.state_core().player(2).equips.iter().copied().find(|&c| ez.catalog().card_kind(c).equip_slot() == Some(engine_native::catalog::EquipSlot::Weapon));
        assert!(wp0.is_some() && wp2.is_some(), "zengli round {round} must NOT exhaust after 9 uses");
        for cid in [wp0.unwrap(), wp2.unwrap()] {
            let n = ez.catalog().cards[cid as usize].name;
            assert!(
                !n.starts_with("hsqingyu_") && !n.starts_with("hstianqi_") && !n.starts_with("gjyuheng"),
                "zengli must never equip non-inpile/unique weapon `{n}`"
            );
        }
        advance_to_phase(&mut ez, 1, Phase::Play);
    }
    assert_eq!(total_tracked_cards(&ez), tracked_z);
}
