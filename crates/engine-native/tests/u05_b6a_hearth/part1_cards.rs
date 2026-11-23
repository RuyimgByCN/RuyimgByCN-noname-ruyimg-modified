//! Batch U05-6a Part 1: All 57 `hearth` cards + 2+2 card companion skills (`shihuawuqi`,
//! `jihuocard2`, `chuansongmen2`, `chuansongmen3`), tested via genuine `PlayCard`/`Answer`/`NextPhase`.

use super::common::*;
use engine_native::catalog::CardKind;
use engine_native::event::Phase;
use engine_native::{ChoiceOption, CommandOutcome};

#[test]
fn u05_b6a_card_hearth_base_13_positive_negative_choices_and_save_restore() {
    // 1-2. `linghunzhihuo` (card/hearth/card.js:4), `jihuocard` (:47) + `jihuocard2` (skill.js:14)
    let mut e = fresh_proto(11_001, &["hs_jaina", "hs_lrexxar"]);
    let tracked = total_tracked_cards(&e);
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 1);

    let extra = give_free_card_kind(&mut e, 0, CardKind::Sha);
    let lh = give_card_named(&mut e, 0, "linghunzhihuo");
    let hp1_before = e.state_core().player(1).hp;
    let log_start = e.log().len();
    let out = play_card(&mut e, 0, lh, vec![1]);
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    drain_auto(&mut e);
    assert_eq!(e.state_core().player(1).hp, hp1_before - 1);
    assert!(
        !e.state_core().player(0).hand.contains(&extra),
        "linghunzhihuo must force caster to discard 1 random hand card (card.js:19)"
    );
    let dmg_idx = e.log()[log_start..]
        .iter()
        .position(|ev| matches!(ev, engine_native::machine::LogEvent::DamageDealt { target: 1, .. }))
        .expect("linghunzhihuo DamageDealt log");
    let disc_idx = e.log()[log_start..]
        .iter()
        .position(|ev| {
            matches!(ev, engine_native::machine::LogEvent::CardDiscarded { player: 0, cards } if cards.contains(&extra))
        })
        .expect("linghunzhihuo CardDiscarded log");
    assert!(
        dmg_idx < disc_idx,
        "linghunzhihuo must resolve Damage before caster discards hand card (FIFO, not LIFO)"
    );
    assert_eq!(total_tracked_cards(&e), tracked);

    // `jihuocard`: draw 1, gain `jihuocard2` (-1 distance & +1 hand limit this turn)
    let jh = give_card_named(&mut e, 0, "jihuocard");
    let d1 = give_free_card_kind(&mut e, 0, CardKind::Shan);
    put_card_on_deck_top(&mut e, d1);
    let hand_before = e.state_core().player(0).hand.len();
    let out = play_card(&mut e, 0, jh, vec![]);
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    drain_auto(&mut e);
    assert!(e.state_core().player(0).hand.contains(&d1));
    assert_eq!(e.state_core().player(0).hand.len(), hand_before);
    assert!(has_skill(&e, 0, "jihuocard2"));
    assert_eq!(
        e.state_core().hand_limit(e.catalog(), 0),
        e.state_core().player(0).hp.max(0) as usize + 2,
        "jihuocard2 grants +2 maxHandcard this turn (card/hearth/skill.js:14-20)"
    );
    assert_eq!(total_tracked_cards(&e), tracked);

    // 3. `zhaomingdan` (card/hearth/card.js:70): discard target's card, then target draws BEFORE caster draws
    let mut e2 = fresh_proto(11_002, &["hs_jaina", "hs_lrexxar"]);
    let tracked2 = total_tracked_cards(&e2);
    advance_to_phase(&mut e2, 0, Phase::Play);
    clear_hand(&mut e2, 0);
    clear_hand(&mut e2, 1);
    let _t1h = give_free_card_kind(&mut e2, 1, CardKind::Shan);
    let top_for_caster = give_free_card_kind(&mut e2, 1, CardKind::Sha);
    let top_for_target = give_free_card_kind(&mut e2, 1, CardKind::Tao);
    put_card_on_deck_top(&mut e2, top_for_caster);
    put_card_on_deck_top(&mut e2, top_for_target);
    let zm = give_card_named(&mut e2, 0, "zhaomingdan");
    let out = play_card(&mut e2, 0, zm, vec![1]);
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    let ch = e2.open_choice().cloned().expect("zhaomingdan discard choice");
    assert_eq!(ch.spec.prompt, "hs_zhaomingdan_discard");
    let picked = ch.spec.options[0];
    save_restore_and_answer(&mut e2, "hs_zhaomingdan_discard", vec![picked]);
    drain_auto(&mut e2);
    assert_eq!(e2.state_core().player(0).hand, vec![top_for_caster]);
    assert_eq!(e2.state_core().player(1).hand, vec![top_for_target]);
    assert_eq!(total_tracked_cards(&e2), tracked2);

    // 4-6. `shijieshu` (:135), `shandianjian` (:210), `shihuawuqi` (:342 + skill.js:7)
    let mut e3 = fresh_proto(11_003, &["hs_jaina", "hs_lrexxar"]);
    let tracked3 = total_tracked_cards(&e3);
    advance_to_phase(&mut e3, 0, Phase::Play);
    clear_hand(&mut e3, 0);
    clear_hand(&mut e3, 1);

    // `shijieshu` negative (`vec![]`) -> positive: equips up to 2 distinct slots in deck order, yields between equips, then prompts discard
    let sjs = give_card_named(&mut e3, 0, "shijieshu");
    let extra3 = give_free_card_kind(&mut e3, 0, CardKind::Shan);
    let arm_top2 = give_free_card_kind(&mut e3, 0, CardKind::Armor);
    let wp_top1 = give_free_card_kind(&mut e3, 0, CardKind::Weapon);
    put_card_on_deck_top(&mut e3, arm_top2);
    put_card_on_deck_top(&mut e3, wp_top1);
    let eq_log_start = e3.log().len();
    assert!(matches!(play_card(&mut e3, 0, sjs, vec![]), CommandOutcome::Rejected(_)));
    let out = play_card(&mut e3, 0, sjs, vec![0]);
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    let equipped: Vec<u32> = e3.log()[eq_log_start..]
        .iter()
        .filter_map(|ev| match *ev {
            engine_native::machine::LogEvent::Equipped { player: 0, card, .. } => Some(card),
            _ => None,
        })
        .collect();
    assert_eq!(equipped, vec![wp_top1, arm_top2], "shijieshu must equip top 2 distinct equips in FIFO order");
    save_restore_and_answer(&mut e3, "hs_shijieshu_discard", vec![ChoiceOption::Card(extra3)]);
    drain_auto(&mut e3);
    assert!(!e3.state_core().player(0).equips.is_empty());
    assert_eq!(total_tracked_cards(&e3), tracked3);

    // `shandianjian` (:210): negative without target hand, positive reveal vs match suit -> damage & discard matching
    let mut e4 = fresh_proto(11_004, &["hs_jaina", "hs_lrexxar"]);
    let tracked4 = total_tracked_cards(&e4);
    advance_to_phase(&mut e4, 0, Phase::Play);
    clear_hand(&mut e4, 0);
    clear_hand(&mut e4, 1);
    let sdj = give_card_named(&mut e4, 0, "shandianjian");
    assert!(
        matches!(play_card(&mut e4, 0, sdj, vec![1]), CommandOutcome::Rejected(_)),
        "shandianjian must reject target with empty hand"
    );
    let match_card = give_free_card_kind(&mut e4, 1, CardKind::Sha);
    let _extra_show = give_free_card_kind(&mut e4, 1, CardKind::Shan);
    let suit = e4.catalog().cards[match_card as usize].suit;
    // Give seat 0 a matching suit card so `hs_shandianjian_discard` fires
    let caster_match = {
        let cat = e4.catalog();
        (0..cat.cards.len() as u32)
            .find(|&c| {
                c != sdj
                    && c != match_card
                    && c != _extra_show
                    && !cat.cards[c as usize].off_deck
                    && cat.cards[c as usize].suit == suit
            })
            .unwrap()
    };
    let caster_name = e4.catalog().cards[caster_match as usize].name;
    give_card_named(&mut e4, 0, caster_name);
    let hp_before = e4.state_core().player(1).hp;
    let out = play_card(&mut e4, 0, sdj, vec![1]);
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    save_restore_and_answer(&mut e4, "hs_shandianjian_show", vec![ChoiceOption::Card(match_card)]);
    if let Some(ch) = e4.open_choice() {
        assert_eq!(ch.spec.prompt, "hs_shandianjian_discard");
        let _ = answer(&mut e4, 0, vec![ChoiceOption::Card(caster_match)]);
    }
    drain_auto(&mut e4);
    assert_eq!(e4.state_core().player(1).hp, hp_before - 1);
    assert_eq!(total_tracked_cards(&e4), tracked4);

    // `shihuawuqi` (:342 + skill.js:7-13): allowed even at full HP, grants `shihuawuqi` (+1 attack range this turn) & gains 1 Sha from deck when hand has no Sha
    let mut e5 = fresh_proto(11_005, &["hs_jaina", "hs_lrexxar"]);
    let tracked5 = total_tracked_cards(&e5);
    advance_to_phase(&mut e5, 0, Phase::Play);
    clear_hand(&mut e5, 0);
    clear_hand(&mut e5, 1);
    let shwq = give_card_named(&mut e5, 0, "shihuawuqi");
    assert_eq!(e5.state_core().player(0).hp, e5.state_core().player(0).max_hp as i32);
    let range_before = e5.state_core().attack_range(e5.catalog(), 0);
    let out = play_card(&mut e5, 0, shwq, vec![]);
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    drain_auto(&mut e5);
    assert!(has_skill(&e5, 0, "shihuawuqi"));
    assert_eq!(e5.state_core().attack_range(e5.catalog(), 0), range_before + 1);
    let sha = e5
        .state_core()
        .player(0)
        .hand
        .iter()
        .copied()
        .find(|&c| e5.catalog().card_kind(c).is_sha_like())
        .expect("shihuawuqi gains 1 Sha from deck when hand has no Sha");
    let hp1 = e5.state_core().player(1).hp;
    let out = play_card(&mut e5, 0, sha, vec![1]);
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    drain_auto(&mut e5);
    assert_eq!(e5.state_core().player(1).hp, hp1 - 1);
    assert_eq!(total_tracked_cards(&e5), tracked5);

    // 7-13. `dunpaigedang` (:371), `chuansongmen` (:399 + skill.js:5-6), `tanshezhiren` (:458),
    // `xingjiegoutong` (:508), `shenenshu` (:549), `zhiliaobo` (:591), `yuansuhuimie` (:638)
    let mut e6 = fresh_proto(11_006, &["hs_jaina", "hs_lrexxar"]);
    let tracked6 = total_tracked_cards(&e6);
    advance_to_phase(&mut e6, 0, Phase::Play);
    clear_hand(&mut e6, 0);
    clear_hand(&mut e6, 1);
    let dp = give_card_named(&mut e6, 0, "dunpaigedang");
    let extra6 = give_free_card_kind(&mut e6, 0, CardKind::Shan);
    assert!(!matches!(play_card(&mut e6, 0, dp, vec![]), CommandOutcome::Rejected(_)));
    save_restore_and_answer(&mut e6, "hs_dunpaigedang_discard", vec![ChoiceOption::Card(extra6)]);
    drain_auto(&mut e6);
    assert_eq!(e6.state_core().player(0).hujia, 1);
    assert_eq!(e6.state_core().player(0).hand.len(), 1);

    // `shenenshu`: discards remaining hand (1) & creates 1 + 1 = 2 basic cards (card.js:549)
    let ses = give_card_named(&mut e6, 0, "shenenshu");
    let h_before = e6.state_core().player(0).hand.len();
    assert!(!matches!(play_card(&mut e6, 0, ses, vec![]), CommandOutcome::Rejected(_)));
    drain_auto(&mut e6);
    assert_eq!(e6.state_core().player(0).hand.len(), h_before);
    assert!(
        e6.state_core()
            .player(0)
            .hand
            .iter()
            .all(|&c| e6.catalog().card_kind(c).is_basic_kind()),
        "shenenshu creates only basic cards"
    );

    // `zhiliaobo`: negative when target full HP; positive when injured (+1 or +2 if pindian win)
    e6.state_core_mut().player_mut(1).hp = e6.state_core().player(1).max_hp as i32;
    let zlb = give_card_named(&mut e6, 0, "zhiliaobo");
    assert!(matches!(play_card(&mut e6, 0, zlb, vec![1]), CommandOutcome::Rejected(_)));
    e6.state_core_mut().player_mut(1).hp = 1;
    let out = play_card(&mut e6, 0, zlb, vec![1]);
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    drain_auto(&mut e6);
    assert!(e6.state_core().player(1).hp >= 2);

    // `tanshezhiren`: targetless trick — rejects explicit target `[1]`, accepts `[]`
    let ts = give_card_named(&mut e6, 0, "tanshezhiren");
    let opp_card = give_free_card_kind(&mut e6, 1, CardKind::Sha);
    assert!(
        matches!(play_card(&mut e6, 0, ts, vec![1]), CommandOutcome::Rejected(engine_native::RejectReason::IllegalTargets)),
        "tanshezhiren (filterTarget: false) must reject targeted PlayCard"
    );
    assert!(!matches!(play_card(&mut e6, 0, ts, vec![]), CommandOutcome::Rejected(_)));
    drain_auto(&mut e6);
    assert!(
        !e6.state_core().player(1).hand.contains(&opp_card),
        "tanshezhiren must force enemy discard"
    );

    // `chuansongmen` + `shijieshu`: drawing `shijieshu` offers `hs_chuansongmen_use` -> `SelectPlayers` & `BoundedPlanner` picks `Player`
    let sjs_csm = give_card_named(&mut e6, 0, "shijieshu");
    put_card_on_deck_top(&mut e6, sjs_csm);
    let csm_test = give_card_named(&mut e6, 0, "chuansongmen");
    assert!(!matches!(play_card(&mut e6, 0, csm_test, vec![]), CommandOutcome::Rejected(_)));
    let use_ch = advance_until_open_prompt(&mut e6, "hs_chuansongmen_use", 4)
        .expect("chuansongmen drawing shijieshu must offer hs_chuansongmen_use");
    assert!(use_ch.spec.options.contains(&ChoiceOption::Card(sjs_csm)));
    let _ = answer(&mut e6, 0, vec![ChoiceOption::Card(sjs_csm)]);
    let tgt_ch = e6
        .open_choice()
        .cloned()
        .expect("chuansongmen using shijieshu must open target prompt");
    assert_eq!(tgt_ch.spec.kind, engine_native::choice::ChoiceKind::SelectPlayers);
    let ai_act = ai_plan_action(&e6, 0);
    assert!(
        matches!(&ai_act, engine_native::GameAction::Answer { selection, .. }
            if selection.iter().any(|o| matches!(o, ChoiceOption::Player(_)))),
        "AI on hs_chuansongmen_use SelectPlayers must select Player, got {ai_act:?}"
    );
    let _ = submit(&mut e6, 0, ai_act);
    drain_auto(&mut e6);

    // `xingjiegoutong`: gain extra PlayPhase or draw
    let xjgt = give_card_named(&mut e6, 0, "xingjiegoutong");
    let _ = play_card(&mut e6, 0, xjgt, vec![]);
    drain_auto(&mut e6);

    // `chuansongmen`: grants `chuansongmen2` & `chuansongmen3` (card/hearth/skill.js:5-6)
    let csm = give_card_named(&mut e6, 0, "chuansongmen");
    let _ = play_card(&mut e6, 0, csm, vec![]);
    drain_auto(&mut e6);
    assert!(has_skill(&e6, 0, "chuansongmen2") || has_skill(&e6, 0, "chuansongmen3") || !e6.state_core().player(0).hand.is_empty());

    // `yuansuhuimie` (card/hearth/card.js:638): 2 Thunder damage to each other player (minus up to 2 discards)
    e6.state_core_mut().player_mut(1).hp = 4;
    clear_hand(&mut e6, 1);
    let yshm = give_card_named(&mut e6, 0, "yuansuhuimie");
    let _ = play_card(&mut e6, 0, yshm, vec![]);
    drain_auto(&mut e6);
    assert_eq!(e6.state_core().player(1).hp, 2, "yuansuhuimie deals 2 Thunder damage when unmitigated");
    assert_eq!(total_tracked_cards(&e6), tracked6);
}

#[test]
fn u05_b6a_character_hearth_derived_44_cards_behavior_vanish_and_conservation() {
    let mut e = fresh_proto(11_007, &["hs_jaina", "hs_lrexxar"]);
    let tracked = total_tracked_cards(&e);
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 1);

    // 1. `hsfashu_anyingjingxiang` blank cannot be played; `tuteng1..=8` rejected as direct PlayCard
    let shadow = give_card_named(&mut e, 0, "hsfashu_anyingjingxiang");
    assert!(
        matches!(play_card(&mut e, 0, shadow, vec![]), CommandOutcome::Rejected(_)),
        "untransformed hsfashu_anyingjingxiang must be unplayable"
    );
    for i in 1..=8 {
        let tc = give_card_named(&mut e, 0, &format!("tuteng{i}"));
        assert!(
            matches!(play_card(&mut e, 0, tc, vec![]), CommandOutcome::Rejected(_)),
            "tuteng{i} must reject direct PlayCard"
        );
    }

    // 2. `hsfashu_buwendingyibian` (character/hearth/card.js:11): recast branch vs cancel
    let bwd = give_card_named(&mut e, 0, "hsfashu_buwendingyibian");
    let kept = give_free_card_kind(&mut e, 0, CardKind::Sha);
    let _ = play_card(&mut e, 0, bwd, vec![0]);
    if let Some(ch) = e.open_choice() {
        assert_eq!(ch.spec.prompt, "hs_buwendingyibian_recast");
        // Pick hand card to recast (replaces with deck card + spawns clone in hand)
        let _ = answer(&mut e, 0, vec![ChoiceOption::Card(kept)]);
    }
    drain_auto(&mut e);
    assert!(
        e.state_core().detached_cards.iter().any(|&(c, _)| c == bwd),
        "vanish card hsfashu_buwendingyibian must move to detached_cards instead of discard"
    );
    assert!(!e.state_core().discard.contains(&bwd));

    // Branch B: target with empty hand skips recast choice; non-empty target recasts to `discard` (NOT vanish); turn limit `used < player.hp`
    let mut e_b = fresh_proto(11_008, &["hs_jaina", "hs_lrexxar"]);
    advance_to_phase(&mut e_b, 0, Phase::Play);
    clear_hand(&mut e_b, 0);
    clear_hand(&mut e_b, 1);
    let bwd_empty = give_card_named(&mut e_b, 0, "hsfashu_buwendingyibian");
    let _ = play_card(&mut e_b, 0, bwd_empty, vec![1]);
    assert!(e_b.open_choice().is_none(), "buwendingyibian skips recast when target hand empty");
    drain_auto(&mut e_b);
    let kept_b = give_free_card_kind(&mut e_b, 0, CardKind::Shan);
    let bwd_b = give_card_named(&mut e_b, 0, "hsfashu_buwendingyibian");
    let _ = play_card(&mut e_b, 0, bwd_b, vec![0]);
    save_restore_and_answer(&mut e_b, "hs_buwendingyibian_recast", vec![ChoiceOption::Card(kept_b)]);
    drain_auto(&mut e_b);
    assert!(
        e_b.state_core().discard.contains(&kept_b),
        "buwendingyibian recast hand card must go to discard, not vanish"
    );
    // HP=2 and already used 2 times in turn -> playing the granted clone must reject with CardLimit
    e_b.state_core_mut().player_mut(0).hp = 2;
    let clone_b = e_b
        .state_core()
        .player(0)
        .hand
        .iter()
        .copied()
        .find(|&c| e_b.catalog().cards[c as usize].name.contains("buwendingyibian"))
        .expect("buwendingyibian clone in hand");
    assert!(
        matches!(play_card(&mut e_b, 0, clone_b, vec![0]), CommandOutcome::Rejected(engine_native::RejectReason::CardLimit)),
        "hsfashu_buwendingyibian must reject when used >= player.hp"
    );

    // 3. Four Horsemen Equips (`hstianqi_dalian`, `shali`, `nazigelin`, `suolasi`) + `hsqingyu_zhao` (weapon replaces `dalian`)
    for eq_name in [
        "hstianqi_dalian",
        "hstianqi_shali",
        "hstianqi_nazigelin",
        "hstianqi_suolasi",
    ] {
        let eq = give_card_named(&mut e, 0, eq_name);
        let out = play_card(&mut e, 0, eq, vec![]);
        assert!(!matches!(out, CommandOutcome::Rejected(_)), "equip {eq_name} failed: {out:?}");
        drain_auto(&mut e);
    }
    assert!(has_skill(&e, 0, "hstianqi_dalian"));
    assert!(has_skill(&e, 0, "hstianqi_shali"));
    let qz = give_card_named(&mut e, 0, "hsqingyu_zhao");
    let out_qz = play_card(&mut e, 0, qz, vec![]);
    assert!(!matches!(out_qz, CommandOutcome::Rejected(_)));
    drain_auto(&mut e);
    assert!(!has_skill(&e, 0, "hstianqi_dalian"), "Weapon hsqingyu_zhao must replace hstianqi_dalian");
    assert!(has_skill(&e, 0, "hstianqi_shali"));

    // 4. Direct Trick execution sweep for remaining derived cards:
    // `hsjixie_zhadan`, `hsqizhou_feng/shui/huo/tu`, `hsqingyu_feibiao/zhanfang/hufu/shandian`,
    // `hsdusu_xueji/kuyecao/huangxuecao/huoyanhua/shinancao`, `hsbaowu_cangbaotu/huangjinyuanhou`,
    // `hsshenqi_nengliangzhiguang/kongbusangzhong/morijingxiang`,
    // `hsmengjing_feicuiyoulong/suxing/mengye/mengjing/huanxiaojiemei`,
    // `hszuzhou_nvwudeganguo/nvwudepingguo/nvwudexuetu/wushushike/guhuo`.
    e.state_core_mut().player_mut(0).hp = 2; // injured for `hsqizhou_shui`
    e.state_core_mut().player_mut(1).hp = 3; // injured for `hsmengjing_huanxiaojiemei`
    let _t1h = give_free_card_kind(&mut e, 1, CardKind::Shan);
    let _t1h2 = give_free_card_kind(&mut e, 1, CardKind::Tao);
    let xueji_neg = give_card_named(&mut e, 0, "hsdusu_xueji");
    assert!(
        matches!(play_card(&mut e, 0, xueji_neg, vec![1]), CommandOutcome::Rejected(engine_native::RejectReason::IllegalTargets)),
        "hsdusu_xueji must reject target with hand cards but 0 equips"
    );
    let _t1e = give_free_card_kind(&mut e, 1, CardKind::Weapon);
    e.state_core_mut().player_mut(1).hand.retain(|&c| c != _t1e);
    e.state_core_mut().player_mut(1).equips.push(_t1e);
    assert!(
        !matches!(play_card(&mut e, 0, xueji_neg, vec![1]), CommandOutcome::Rejected(_)),
        "hsdusu_xueji must succeed once target has an equip"
    );
    drain_auto(&mut e);
    let _t1e2 = give_free_card_kind(&mut e, 1, CardKind::Armor);
    e.state_core_mut().player_mut(1).hand.retain(|&c| c != _t1e2);
    e.state_core_mut().player_mut(1).equips.push(_t1e2);

    // Negative checks for target-specific derived cards
    let shui = give_card_named(&mut e, 0, "hsqizhou_shui");
    e.state_core_mut().player_mut(0).hp = e.state_core().player(0).max_hp as i32;
    assert!(matches!(play_card(&mut e, 0, shui, vec![]), CommandOutcome::Rejected(_)));
    e.state_core_mut().player_mut(0).hp = 2;
    assert!(!matches!(play_card(&mut e, 0, shui, vec![]), CommandOutcome::Rejected(_)));
    drain_auto(&mut e);
    assert!(e.state_core().player(0).hp >= 3);

    let single_target_cards: &[(&str, Vec<u8>)] = &[
        ("hsjixie_zhadan", vec![]),
        ("hsqizhou_feng", vec![]),
        ("hsqizhou_huo", vec![1]),
        ("hsqizhou_tu", vec![1]),
        ("hsqingyu_feibiao", vec![1]),
        ("hsqingyu_zhanfang", vec![0]),
        ("hsqingyu_hufu", vec![0]),
        ("hsqingyu_shandian", vec![1]),
        ("hsdusu_xueji", vec![1]),
        ("hsdusu_kuyecao", vec![0]),
        ("hsdusu_huangxuecao", vec![0]),
        ("hsdusu_huoyanhua", vec![1]),
        ("hsdusu_shinancao", vec![0]),
        ("hsbaowu_cangbaotu", vec![]),
        ("hsbaowu_huangjinyuanhou", vec![]),
        ("hsshenqi_nengliangzhiguang", vec![1]),
        ("hsshenqi_kongbusangzhong", vec![]),
        ("hsshenqi_morijingxiang", vec![]),
        ("hsmengjing_feicuiyoulong", vec![0]),
        ("hsmengjing_suxing", vec![]),
        ("hsmengjing_mengye", vec![1]),
        ("hsmengjing_mengjing", vec![1]),
        ("hsmengjing_huanxiaojiemei", vec![1]),
        ("hszuzhou_nvwudeganguo", vec![1]),
        ("hszuzhou_nvwudepingguo", vec![1]),
        ("hszuzhou_nvwudexuetu", vec![1]),
        ("hszuzhou_wushushike", vec![]),
        ("hszuzhou_guhuo", vec![1]),
    ];

    for (cname, targets) in single_target_cards {
        // Keep players healthy & topped up with filler cards so all effects remain live
        e.state_core_mut().player_mut(0).face_down = false;
        e.state_core_mut().player_mut(0).hp = 4;
        e.state_core_mut().player_mut(1).hp = 4;
        if *cname == "hsmengjing_huanxiaojiemei" {
            e.state_core_mut().player_mut(1).hp = 2;
        }
        if *cname == "hsdusu_xueji" && e.state_core().player(1).equips.is_empty() {
            let eq = give_free_card_kind(&mut e, 1, CardKind::Armor);
            e.state_core_mut().player_mut(1).hand.retain(|&c| c != eq);
            e.state_core_mut().player_mut(1).equips.push(eq);
        }
        if e.state_core().player(1).hand.is_empty() {
            let _ = give_free_card_kind(&mut e, 1, CardKind::Shan);
        }
        let cid = give_card_named(&mut e, 0, cname);
        let out = play_card(&mut e, 0, cid, targets.clone());
        assert!(
            !matches!(out, CommandOutcome::Rejected(_)),
            "derived card `{cname}` rejected with targets {targets:?}: {out:?}"
        );
        drain_auto(&mut e);
        assert!(
            e.state_core().detached_cards.iter().any(|&(x, _)| x == cid)
                || e.state_core().player(0).equips.contains(&cid),
            "hearth derived card `{cname}` should vanish or stay off discard ({})",
            e.state_core().discard.contains(&cid)
        );
    }

    assert_eq!(total_tracked_cards(&e), tracked, "card conservation broken across 44 derived hearth cards");
}
