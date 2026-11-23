use super::common::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::event::DamageNature;
use engine_native::machine::LogEvent;
use engine_native::{ChoiceKind, ChoiceOption, GameAction};

#[test]
fn u05_b4_audited_existing_host_fixes() {
    // 1. liefeng grants temp yanzhan (2nd), tianjian (3rd), xjyufeng (4th + refill when nh < 2) and all expire at turn end!
    let (_, mut el) = start_xianjian_only(8460, 2);
    advance_to_play(&mut el, 0);
    el.debug_clear_hand(0);
    el.debug_clear_hand(1);
    el.debug_grant(0, "liefeng");
    for idx in 1..=3 {
        let wz = el.debug_place(0, "wuzhong");
        submit(&mut el, 0, GameAction::PlayCard { card: wz, targets: vec![] });
        match idx {
            2 => assert!(el.state_core().skills.contains_key(&(0, el.catalog().skill_by_name("yanzhan").unwrap()))),
            3 => assert!(el.state_core().skills.contains_key(&(0, el.catalog().skill_by_name("tianjian").unwrap()))),
            _ => {}
        }
    }
    el.debug_clear_hand(0);
    let fourth = el.debug_place(0, "sha");
    submit(&mut el, 0, GameAction::PlayCard { card: fourth, targets: vec![1] });
    drain_pass_all(&mut el);
    let yz_def = el.catalog().skill_by_name("yanzhan").unwrap();
    let tj_k = el.catalog().skill_by_name("tianjian").unwrap();
    let yf_def = el.catalog().skill_by_name("xjyufeng").unwrap();
    assert!(el.state_core().skills.contains_key(&(0, yf_def)), "liefeng granted xjyufeng on 4th card");
    assert_eq!(el.debug_hand(0).len(), 2, "liefeng refilled hand to 2 when granting xjyufeng with nh < 2");
    assert!(el.state_core().turn.xjyufeng_used >= 1, "xjyufeng_used incremented on refill");
    assert!(el.state_core().skills[&(0, yz_def)].expires_end_of_turn);
    assert!(el.state_core().skills[&(0, tj_k)].expires_end_of_turn);
    assert!(el.state_core().skills[&(0, yf_def)].expires_end_of_turn);
    step_full_turn_to(&mut el, 1);
    assert!(!el.state_core().skills.contains_key(&(0, yz_def)), "temp yanzhan expired at TurnEnd");
    assert!(!el.state_core().skills.contains_key(&(0, tj_k)), "temp tianjian expired at TurnEnd");
    assert!(!el.state_core().skills.contains_key(&(0, yf_def)), "temp xjyufeng expired at TurnEnd");

    // 2. tianjian discards enemy "he" ONLY when damaged, NOT when enemy dodges with Shan!
    let (vt, mut et) = start_xianjian_only(8461, 3);
    advance_to_play(&mut et, 0);
    for p in 0..3 {
        et.debug_clear_hand(p);
        et.debug_set_hp(p, 5);
    }
    et.debug_grant(0, "tianjian");
    let sha = et.debug_place(0, "sha");
    let p1_shan = et.debug_place(1, "shan");
    let p1_extra = et.debug_place(1, "wuzhong");
    let p2_keep = et.debug_place(2, "tao");
    let tj_def = et.catalog().skill_by_name("tianjian").unwrap();
    submit(&mut et, 0, GameAction::ActivateSkill { skill: tj_def });
    answer_open(&mut et, vec![ChoiceOption::Card(sha)]);
    // Player 1 dodges with Shan; player 2 has no Shan so AoeTargetMachine hits directly
    answer_open(&mut et, vec![ChoiceOption::Card(p1_shan)]);
    let _ = vt;
    assert_eq!(et.debug_hp(1), 5, "player 1 dodged");
    assert_eq!(et.debug_hand(1), vec![p1_extra], "dodging enemy keeps non-discarded cards");
    assert_eq!(et.debug_hp(2), 4, "player 2 damaged");
    assert!(!et.debug_hand(2).contains(&p2_keep), "damaged enemy discarded by tianjian DamageEnd");

    // 3. qianfang draws per damaged enemy on DamageEnd (num=0 also draws; dodged enemy does NOT draw)
    let (vq2, mut eq2) = start_xianjian_only(8463, 3);
    for p in 0..3 {
        eq2.debug_clear_hand(p);
        eq2.debug_set_max_hp(p, 5);
        eq2.debug_set_hp(p, 5);
    }
    eq2.debug_grant(0, "qianfang");
    let mut snap2 = eq2.save();
    snap2.core.players[0].xuanning_mark = 3;
    eq2 = content_native_loader::restore_locked_engine(&snap2, &vq2).unwrap();
    advance_to_play(&mut eq2, 0);
    // Advance until player 2 End phase, right before player 0 TurnBegin!
    while !(eq2.turn().current == 2 && eq2.turn().phase == engine_native::event::Phase::End) {
        if let Some(c) = eq2.open_choice().cloned() {
            let sel = if c.spec.allow_cancel {
                Vec::new()
            } else if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else {
                c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
            };
            let _ = submit(&mut eq2, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
        } else {
            let cur = eq2.turn().current;
            let _ = submit(&mut eq2, cur, GameAction::NextPhase);
        }
    }
    eq2.debug_clear_hand(0);
    eq2.debug_clear_hand(1);
    eq2.debug_clear_hand(2);
    let p1_dodge = eq2.debug_place(1, "shan");
    // Player 2 End -> TurnEnded -> player 0 TurnBegin triggers qianfang (`num=0`) optional Confirm!
    let _ = submit(&mut eq2, 2, GameAction::NextPhase);
    let qf_conf = eq2.open_choice().expect("qianfang num=0 confirm ask").clone();
    assert_eq!(qf_conf.spec.prompt, "xj_qianfang_confirm");
    assert_eq!(qf_conf.spec.kind, ChoiceKind::Confirm);
    let mut eq2_r = roundtrip_choice(eq2, &vq2);
    answer_open(&mut eq2_r, vec![ChoiceOption::Confirm]);
    answer_open(&mut eq2_r, vec![ChoiceOption::Card(p1_dodge)]);
    drain_pass_all(&mut eq2_r);
    assert!(eq2_r.debug_hp(2) == 4 && eq2_r.debug_hp(1) == 5, "only 2 took damage");
    assert_eq!(eq2_r.debug_hand(0).len(), 1, "qianfang num=0 draws 1 per enemy damaged (dodged skipped)");

    // qianfang num=0 Cancel keeps xuanning_mark == 3 and fires no ArcheryAttack
    let mut eq_cancel = eq2_r;
    eq_cancel.debug_set_hp(1, 5);
    eq_cancel.debug_set_hp(2, 5);
    let mut sn_c = eq_cancel.save();
    sn_c.core.players[0].xuanning_mark = 3;
    eq_cancel = content_native_loader::restore_locked_engine(&sn_c, &vq2).unwrap();
    while !(eq_cancel.turn().current == 2 && eq_cancel.turn().phase == engine_native::event::Phase::End) {
        if let Some(c) = eq_cancel.open_choice().cloned() {
            let sel = if c.spec.allow_cancel { Vec::new() } else if c.spec.options.contains(&ChoiceOption::Pass) { vec![ChoiceOption::Pass] } else { c.spec.options.iter().take(c.spec.min.max(1)).copied().collect() };
            let _ = submit(&mut eq_cancel, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
        } else {
            let cur = eq_cancel.turn().current;
            let _ = submit(&mut eq_cancel, cur, GameAction::NextPhase);
        }
    }
    let _ = submit(&mut eq_cancel, 2, GameAction::NextPhase);
    assert_eq!(eq_cancel.open_choice().expect("qianfang confirm").spec.prompt, "xj_qianfang_confirm");
    answer_open(&mut eq_cancel, vec![]);
    assert_eq!(eq_cancel.state_core().player(0).xuanning_mark, 3, "cancel preserves xuanning_mark");
    assert_eq!((eq_cancel.debug_hp(1), eq_cancel.debug_hp(2)), (5, 5), "cancel emits no ArcheryAttack");

    // 4. xshuangren triggers ONLY on weapon replacement (not hand discard); yuexing grants yuexing2 to BOTH 0 and 2; guanri LoseHp before Damage
    let (vx, mut ex) = start_xianjian_only(8464, 3);
    advance_to_play(&mut ex, 0);
    for p in 0..3 {
        ex.debug_clear_hand(p);
        ex.debug_set_max_hp(p, 5);
        ex.debug_set_hp(p, 5);
    }
    ex.debug_grant(0, "xshuangren");
    ex.debug_grant(0, "yuexing");
    let w1 = (0..ex.catalog().cards.len() as u32).find(|&c| ex.catalog().card_kind(c) == CardKind::Zhuge).unwrap();
    let w2 = (0..ex.catalog().cards.len() as u32).find(|&c| ex.catalog().card_kind(c) == CardKind::Qinggang).unwrap();
    let w3 = (0..ex.catalog().cards.len() as u32).find(|&c| ex.catalog().card_kind(c) == CardKind::Fangtian).unwrap();
    force_move_card_to_hand(&mut ex, &vx, 0, w1);
    force_move_card_to_hand(&mut ex, &vx, 0, w2);
    force_move_card_to_hand(&mut ex, &vx, 0, w3);
    // Test yuexing2 granted to BOTH self and target during Play and removed from BOTH at Play end
    let yx2_def = ex.catalog().skill_by_name("yuexing2").unwrap();
    let yx_def = ex.catalog().skill_by_name("yuexing").unwrap();
    submit(&mut ex, 0, GameAction::ActivateSkill { skill: yx_def });
    answer_open(&mut ex, vec![ChoiceOption::Player(2)]);
    assert!(ex.state_core().skills.contains_key(&(0, yx2_def)), "owner got yuexing2");
    assert!(ex.state_core().skills.contains_key(&(2, yx2_def)), "target also got yuexing2");

    // Negative check: hand discard of w3 in Discard phase must NOT trigger xshuangren
    ex.debug_set_hp(0, 2);
    let _ = submit(&mut ex, 0, GameAction::NextPhase); // Play -> Discard
    assert!(!ex.state_core().skills.contains_key(&(0, yx2_def)), "owner yuexing2 removed on Play end");
    assert!(!ex.state_core().skills.contains_key(&(2, yx2_def)), "target yuexing2 removed on Play end");
    answer_open(&mut ex, vec![ChoiceOption::Card(w3)]);
    assert!(ex.state_core().player(0).off_court.is_empty(), "hand weapon discard must NOT trigger xshuangren");
    assert!(ex.state_core().discard.contains(&w3), "discarded hand weapon went straight to discard");

    // Positive check: real PlayCard weapon replacement (w1 -> w2 -> w3) moves replaced weapon into off_court and displaces previous off_court
    step_full_turn_to(&mut ex, 0);
    ex.debug_set_hp(0, 5);
    force_move_card_to_hand(&mut ex, &vx, 0, w1);
    force_move_card_to_hand(&mut ex, &vx, 0, w2);
    force_move_card_to_hand(&mut ex, &vx, 0, w3);
    submit(&mut ex, 0, GameAction::PlayCard { card: w1, targets: vec![] });
    assert!(ex.state_core().player(0).off_court.is_empty(), "initial weapon equip does not trigger xshuangren");
    submit(&mut ex, 0, GameAction::PlayCard { card: w2, targets: vec![] });
    assert_eq!(ex.state_core().player(0).off_court, vec![w1], "replacing w1 stores w1 in off_court");
    assert!(!ex.state_core().discard.contains(&w1), "replaced w1 held off_court instead of discard");
    submit(&mut ex, 0, GameAction::PlayCard { card: w3, targets: vec![] });
    assert_eq!(ex.state_core().player(0).off_court, vec![w2], "old off_court weapon w1 displaced by w2");
    assert!(ex.state_core().discard.contains(&w1), "displaced off_court weapon w1 moved to discard");

    // 5. guanri LoseHp executes BEFORE Damage (with xfenxin on source: hp 5 -> lose 1 draws before damage!) & changnian logs only given cards
    let (vg, mut eg) = start_xianjian_only(8465, 2);
    advance_to_play(&mut eg, 0);
    eg.debug_clear_hand(0);
    eg.debug_clear_hand(1);
    eg.debug_set_hp(0, 5);
    eg.debug_set_hp(1, 5);
    eg.debug_grant(0, "guanri");
    eg.debug_grant(1, "changnian");
    let gr1 = place_kind_suit(&mut eg, 0, CardKind::Tao, |s| s == Suit::Heart).unwrap();
    let gr2 = place_kind_suit(&mut eg, 0, CardKind::Shan, |s| s == Suit::Diamond).unwrap();
    let p1_pre = eg.debug_place(0, "wuzhong");
    let p1_give = eg.debug_place(1, "shan");
    force_move_card_to_hand(&mut eg, &vg, 0, gr1);
    force_move_card_to_hand(&mut eg, &vg, 0, gr2);
    let gr_def = eg.catalog().skill_by_name("guanri").unwrap();
    let gr_mark = eg.log().len();
    submit(&mut eg, 0, GameAction::ActivateSkill { skill: gr_def });
    answer_open(&mut eg, vec![ChoiceOption::Card(gr1), ChoiceOption::Card(gr2)]);
    answer_open(&mut eg, vec![ChoiceOption::Player(1)]);
    assert_eq!((eg.debug_hp(0), eg.debug_hp(1)), (4, 3));
    let lose_pos = eg.log()[gr_mark..].iter().position(|l| matches!(l, LogEvent::HpLost { player: 0, .. })).expect("guanri source LoseHp logged");
    let dmg_pos = eg.log()[gr_mark..].iter().position(|l| matches!(l, LogEvent::DamageDealt { target: 1, .. })).expect("guanri target Damage logged");
    assert!(lose_pos < dmg_pos, "guanri LoseHp must execute BEFORE Damage on LIFO stack");
    let mark = eg.log().len();
    deal_damage(&mut eg, Some(0), 1, 3, DamageNature::Fire);
    while let Some(c) = eg.open_choice().cloned() {
        if c.spec.kind == engine_native::ChoiceKind::SelectPlayers {
            break;
        }
        let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else {
            Vec::new()
        };
        let _ = submit(&mut eg, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
    }
    answer_open(&mut eg, vec![ChoiceOption::Player(0)]);
    let given_logs: Vec<&Vec<u32>> = eg.log()[mark..]
        .iter()
        .filter_map(|l| match l {
            LogEvent::CardsGiven { cards, .. } => Some(cards),
            _ => None,
        })
        .collect();
    assert!(given_logs.contains(&&vec![p1_give]), "changnian logs only moved cards: {given_logs:?}");
    assert!(given_logs.iter().all(|v| !v.contains(&p1_pre)), "recipient pre-existing hand excluded");

    // 6. minsha: single target (max=1) and uniform random discard over neighbor hand + equips
    let mut saw_equip_discard = false;
    for seed_off in 0..24u64 {
        let (vm, mut em) = start_xianjian_only(8470 + seed_off, 4);
        advance_to_play(&mut em, 0);
        for p in 0..4 {
            em.debug_clear_hand(p);
            em.debug_set_max_hp(p, 5);
            em.debug_set_hp(p, 3);
        }
        em.debug_grant(0, "minsha");
        let ms_card1 = em.debug_place(0, "wuzhong");
        let ms_card2 = em.debug_place(0, "sha");
        let nb_hand = em.debug_place(2, "tao");
        let nb_eq = (0..em.catalog().cards.len() as u32)
            .find(|&c| {
                !em.catalog().cards[c as usize].off_deck
                    && em.catalog().card_kind(c) == CardKind::Zhuge
                    && c != ms_card1
                    && c != ms_card2
                    && c != nb_hand
            })
            .unwrap();
        let mut sn_m = em.save();
        sn_m.core.deck.retain(|&c| c != nb_eq);
        sn_m.core.discard.retain(|&c| c != nb_eq);
        for pl in &mut sn_m.core.players {
            pl.hand.retain(|&c| c != nb_eq);
            pl.equips.retain(|&c| c != nb_eq);
        }
        sn_m.core.players[2].equips.push(nb_eq);
        em = content_native_loader::restore_locked_engine(&sn_m, &vm).unwrap();
        let ms_def = em.catalog().skill_by_name("minsha").unwrap();
        submit(&mut em, 0, GameAction::ActivateSkill { skill: ms_def });
        answer_open(&mut em, vec![ChoiceOption::Card(ms_card1), ChoiceOption::Card(ms_card2)]);
        let tgt_ask = em.open_choice().expect("minsha target ask").clone();
        assert_eq!((tgt_ask.spec.min, tgt_ask.spec.max), (1, 1), "minsha must be strictly single-target [1, 1]");
        answer_open(&mut em, vec![ChoiceOption::Player(1)]);
        drain_pass_all(&mut em);
        assert_eq!(em.debug_hp(1), 2, "primary target took 1 thunder damage");
        assert_eq!(em.debug_hp(3), 3, "non-selected enemy untouched");
        if em.state_core().player(2).equips.is_empty() && em.state_core().discard.contains(&nb_eq) {
            saw_equip_discard = true;
            break;
        }
    }
    assert!(saw_equip_discard, "minsha neighbor random discard must sample uniformly over hand + equips");
}
