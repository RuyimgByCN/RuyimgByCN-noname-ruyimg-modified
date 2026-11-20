//! Regression suite for the 4 remaining L28 reviewer FAIL findings:
//! 1. `L28:generals:th_shen_zhouyu` (`pro_qinyin` unified discard tracking across conversions/responses/judges)
//! 2. `L28:skills:th_bushi` (multi-target `s2` non-first target + `effective_card_suit`)
//! 3. `L28:skills:th_jingxie` (`rewrite_lanyinjia` hand-as-Shan + Qinggang bypass + global rewritten guard)
//! 4. `L28:skills:th_shengong` (`th_shengong:assist` 1/2/3 candidate gating + all 18 exclusive equips)

use super::l28_thunder_eight_match::*;
use engine_native::catalog::{CardDef, CardKind, Suit};
use engine_native::content::extension::ExtensionPack;
use engine_native::content::host::{HostBehavior, HostSkillDecl, GJ_BINGPOTONG_CARD_KEY};
use engine_native::event::Phase;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, LogEvent};

fn test_pack() -> ExtensionPack {
    let mut p = pack();
    p.cards
        .push(CardDef::armor("lanyinjia").with_suit_rank(Suit::Spade, 2));
    p.cards
        .push(CardDef::armor("tengjia").with_suit_rank(Suit::Spade, 2));
    p.cards
        .push(CardDef::basic("huogong", CardKind::Wuzhong).with_suit_rank(Suit::Heart, 2));
    p.cards
        .push(CardDef::basic("bingpotong", CardKind::Bingpotong).with_suit_rank(Suit::Heart, 5));
    p.host_skills.push(HostSkillDecl {
        name: "xinhongyan",
        behavior: HostBehavior::L20MarkBatchB,
        presentation_binding: None,
    });
    p.host_skills.push(HostSkillDecl {
        name: GJ_BINGPOTONG_CARD_KEY,
        behavior: HostBehavior::JiguanBingpoCard,
        presentation_binding: None,
    });
    p
}

fn clear_seat_all(e: &mut Engine, seat: u8) {
    let cards: Vec<u32> = e.state_core_mut().player_mut(seat).hand.drain(..).collect();
    let eqs: Vec<u32> = e.state_core_mut().player_mut(seat).equips.drain(..).collect();
    e.state_core_mut().discard.extend(cards);
    e.state_core_mut().discard.extend(eqs);
}

fn clean_seats(p: &ExtensionPack, seed: u64) -> Engine {
    let mut e = Engine::new_with_extensions(seed, 4, &[p]).expect("engine");
    e.debug_set_turn(0, Phase::Play, 1);
    for s in 0..4 {
        clear_seat_all(&mut e, s);
    }
    e
}

fn equip_named_via_play(e: &mut Engine, seat: u8, name: &str) -> u32 {
    let cid = card_named(e, name);
    take_out(e, cid);
    give(e, seat, cid);
    let prev = (e.turn().current, e.turn().phase, e.turn().round);
    let need_switch = prev.0 != seat || prev.1 != Phase::Play;
    if need_switch {
        e.debug_set_turn(seat, Phase::Play, prev.2);
    }
    let out = submit(e, seat, GameAction::PlayCard { card: cid, targets: Vec::new() });
    assert!(
        !matches!(out, CommandOutcome::Rejected(_)),
        "equip {name} rejected: {out:?}"
    );
    if need_switch && e.open_choice().is_none() {
        e.debug_set_turn(prev.0, prev.1, prev.2);
    }
    cid
}

fn push_top_card(e: &mut Engine, pred: impl Fn(&CardDef) -> bool) -> u32 {
    let c = card_by(e, pred);
    take_out(e, c);
    e.state_core_mut().deck.push(c);
    c
}

fn card_other(e: &Engine, exclude: u32, pred: impl Fn(&CardDef) -> bool) -> u32 {
    e.catalog()
        .cards
        .iter()
        .enumerate()
        .position(|(i, c)| i as u32 != exclude && pred(c))
        .expect("matching distinct card") as u32
}

fn drain_pass_shan(e: &mut Engine) {
    if e.open_choice().is_some_and(|c| c.spec.prompt == "respond_shan") {
        answer(e, "respond_shan", vec![ChoiceOption::Pass]);
    }
}

fn pass_aoe_until_seat(e: &mut Engine, target_seat: u8) {
    while let Some(c) = e.open_choice().cloned() {
        if c.spec.prompt == "aoe_response" && c.observer != target_seat {
            let opt = if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else {
                Vec::new()
            };
            answer(e, "aoe_response", opt);
        } else {
            break;
        }
    }
}

#[test]
fn l28_reg_pro_qinyin_all_discard_paths() {
    let p = test_pack();

    // Positive A: red converted Sha (`武圣`) + black card used as `Shan` (`倾国`) both enter discard pile.
    let mut e = fresh(&p, 3101, "th_shen_zhouyu");
    e.debug_grant(0, "武圣");
    e.debug_grant(1, "倾国");
    clear_seat_all(&mut e, 0);
    clear_seat_all(&mut e, 1);
    let red_shan = card_by(&e, |c| c.kind == CardKind::Shan && c.suit == Suit::Heart);
    let black_sha = card_by(&e, |c| c.kind == CardKind::Sha && c.suit == Suit::Spade);
    give(&mut e, 0, red_shan);
    give(&mut e, 1, black_sha);
    play(&mut e, 0, red_shan, vec![1]);
    answer(&mut e, "respond_shan", vec![cd(black_sha)]);
    drain(&mut e);
    assert!(
        e.state_core().discard.contains(&red_shan) && e.state_core().discard.contains(&black_sha),
        "both converted red card and black Shan response enter discard"
    );
    e.debug_set_turn(0, Phase::Discard, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let hp1_before = hp(&e, 1);
    answer(&mut e, "pro_qinyin:choose_mode", vec![tag(1)]);
    assert_eq!(hp(&e, 1), hp1_before - 1, "pro_qinyin triggers and loses 1 HP");

    // Positive B: Bagua judge card (red) + black Sha entering discard pile in same turn.
    let mut e = fresh(&p, 3102, "th_shen_zhouyu");
    clear_seat_all(&mut e, 0);
    clear_seat_all(&mut e, 1);
    equip_named_via_play(&mut e, 1, "armor");
    let black_sha = card_by(&e, |c| c.kind == CardKind::Sha && c.suit == Suit::Spade);
    let _red_judge = push_top_card(&mut e, |c| c.suit == Suit::Heart);
    give(&mut e, 0, black_sha);
    play(&mut e, 0, black_sha, vec![1]);
    drain(&mut e);
    e.debug_set_turn(0, Phase::Discard, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert_eq!(open(&e, "pro_qinyin:choose_mode").observer, 0);
    drain(&mut e);

    // Negative: only red card enters discard pile -> pro_qinyin does not trigger.
    let mut e = fresh(&p, 3103, "th_shen_zhouyu");
    clear_seat_all(&mut e, 0);
    let r1 = card_by(&e, |c| c.kind == CardKind::Sha && c.suit == Suit::Heart);
    let r2 = card_by(&e, |c| c.kind == CardKind::Shan && c.suit == Suit::Diamond);
    e.state_core_mut().player_mut(0).hand = vec![r1, r2];
    e.state_core_mut().player_mut(0).hp = 1;
    e.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, "discard_to_limit", vec![cd(r1)]);
    drain(&mut e);
    assert!(
        e.open_choice()
            .is_none_or(|c| c.spec.prompt != "pro_qinyin:choose_mode"),
        "single-color discard must NOT trigger pro_qinyin"
    );
}

#[test]
fn l28_reg_th_bushi_multitarget_s2_and_effective_suit() {
    let p = test_pack();

    // Multi-target s2 (Club by default) with th_lukai at seat 2 (non-first target of SavageAttack).
    let mut e = Engine::new_with_extensions(3104, 4, &[&p]).expect("engine");
    e.assign_general_named(2, "th_lukai").expect("lukai");
    e.debug_set_turn(0, Phase::Play, 1);
    clear_seat_all(&mut e, 0);
    clear_seat_all(&mut e, 2);
    let club_savage = card_by(&e, |c| c.kind == CardKind::SavageAttack && c.suit == Suit::Club);
    let heart_deck = push_top_card(&mut e, |c| c.suit == Suit::Heart);
    give(&mut e, 0, club_savage);
    play(&mut e, 0, club_savage, Vec::new());
    drain(&mut e);
    assert!(
        e.state_core().player(2).hand.contains(&heart_deck),
        "th_bushi must draw s3 (Heart) even when th_lukai is a non-first target (targets[1])"
    );

    // effective_card_suit: pick a posture where s1 == Heart (1), grant xinhongyan, play Spade Sha -> draws 2!
    let mut e = fresh(&p, 3105, "th_lukai");
    e.debug_grant(0, "xinhongyan");
    clear_seat_all(&mut e, 0);
    clear_seat_all(&mut e, 1);
    e.debug_set_turn(3, Phase::End, 1);
    let _ = submit(&mut e, 3, GameAction::NextPhase);
    let perm_opt = (0..24u8)
        .find(|&idx| {
            let mut twin = Engine::restore_with_extensions(&e.save(), &[&p]).unwrap();
            answer(&mut twin, "th_bushi:choose_posture", vec![tag(idx)]);
            flag(&twin, 0, "th_bushi_s1") == 1
        })
        .expect("perm with s1 == Heart");
    answer(&mut e, "th_bushi:choose_posture", vec![tag(perm_opt)]);
    advance_to(&mut e, Phase::Play);
    let spade_sha = card_by(&e, |c| c.kind == CardKind::Sha && c.suit == Suit::Spade);
    give(&mut e, 0, spade_sha);
    let h0_before = hand(&e, 0);
    play(&mut e, 0, spade_sha, vec![1]);
    drain(&mut e);
    assert_eq!(
        hand(&e, 0),
        h0_before - 1 + 2,
        "Spade Sha turned Heart by xinhongyan must match s1=Heart via effective_card_suit and draw 2"
    );
}

#[test]
fn l28_reg_th_jingxie_rewrite_lanyinjia_and_cross_seat_guard() {
    let p = test_pack();

    // 1) Rewritten lanyinjia: non-Shan hand card as Shan against Sha (even Qinggang) AND ArcheryAttack.
    let mut e = fresh(&p, 3106, "th_majun");
    clear_seat_all(&mut e, 0);
    clear_seat_all(&mut e, 1);
    let lyj = card_named(&e, "lanyinjia");
    let tao = card_by(&e, |c| c.kind == CardKind::Tao && c.suit == Suit::Heart);
    let tao2 = card_other(&e, tao, |c| c.kind == CardKind::Tao && c.suit == Suit::Heart);
    give(&mut e, 0, lyj);
    give(&mut e, 0, tao);
    give(&mut e, 0, tao2);
    activate(&mut e, 0, "th_jingxie");
    answer(&mut e, "th_jingxie:choose_card", vec![cd(lyj)]);
    assert!(!matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: lyj, targets: Vec::new() }),
        CommandOutcome::Rejected(_)
    ));

    e.debug_set_turn(1, Phase::Play, 1);
    equip_named_via_play(&mut e, 1, "qinggang");
    let sha = card_by(&e, |c| c.kind == CardKind::Sha);
    give(&mut e, 1, sha);
    let hp0 = hp(&e, 0);
    play(&mut e, 1, sha, vec![0]);
    answer(&mut e, "respond_shan", vec![cd(tao)]);
    drain(&mut e);
    assert_eq!(hp(&e, 0), hp0, "rewrite_lanyinjia uses Tao as Shan even against Qinggang");

    let arrow = card_by(&e, |c| c.kind == CardKind::ArcheryAttack);
    give(&mut e, 1, arrow);
    play(&mut e, 1, arrow, Vec::new());
    pass_aoe_until_seat(&mut e, 0);
    answer(&mut e, "aoe_response", vec![cd(tao2)]);
    drain(&mut e);
    assert_eq!(hp(&e, 0), hp0, "rewrite_lanyinjia uses hand card as Shan against ArcheryAttack");

    // 2) Cross-seat second-upgrade guard: transferred rewritten lanyinjia cannot be upgraded again.
    e.assign_general_named(1, "th_majun").expect("seat 1 majun");
    take_out(&mut e, lyj);
    e.state_core_mut().player_mut(1).hand.push(lyj);
    assert!(
        matches!(activate(&mut e, 1, "th_jingxie"), CommandOutcome::Rejected(_)),
        "already-rewritten equip transferred to another th_jingxie owner must not be upgradable again"
    );
}

#[test]
fn l28_reg_th_shengong_assist_1_2_3_and_roundtrip() {
    let p = test_pack();

    // Result 2 (hamper == 0) -> 3 options + snapshot_roundtrip mid-assist
    let mut e = fresh(&p, 3107, "th_puyuan");
    clear_seat_all(&mut e, 0);
    let w1 = card_by(&e, |c| c.kind == CardKind::Qinggang);
    give(&mut e, 0, w1);
    activate(&mut e, 0, "th_shengong");
    answer(&mut e, "th_shengong:discard_card", vec![cd(w1)]);
    snapshot_roundtrip(&e, &p, vec![tag(0)]);
    while e.open_choice().is_some_and(|c| c.spec.prompt == "th_shengong:assist") {
        answer(&mut e, "th_shengong:assist", Vec::new());
    }
    assert_eq!(open(&e, "th_shengong:choose_card").spec.options.len(), 3, "hamper==0 -> 3 options");

    // Result 1 (help >= hamper > 0) -> 2 options
    let mut e = fresh(&p, 3108, "th_puyuan");
    clear_seat_all(&mut e, 0);
    let w1 = card_by(&e, |c| c.kind == CardKind::Qinggang);
    give(&mut e, 0, w1);
    push_top_card(&mut e, |c| c.rank == 4);
    push_top_card(&mut e, |c| c.rank == 10);
    activate(&mut e, 0, "th_shengong");
    answer(&mut e, "th_shengong:discard_card", vec![cd(w1)]);
    answer(&mut e, "th_shengong:assist", vec![tag(1)]);
    while e.open_choice().is_some_and(|c| c.spec.prompt == "th_shengong:assist") {
        answer(&mut e, "th_shengong:assist", Vec::new());
    }
    assert_eq!(open(&e, "th_shengong:choose_card").spec.options.len(), 2, "help>=hamper>0 -> 2 options");

    // Result 0 (help < hamper) -> 1 option
    let mut e = fresh(&p, 3109, "th_puyuan");
    clear_seat_all(&mut e, 0);
    let w1 = card_by(&e, |c| c.kind == CardKind::Qinggang);
    give(&mut e, 0, w1);
    push_top_card(&mut e, |c| c.rank == 13);
    push_top_card(&mut e, |c| c.rank == 2);
    activate(&mut e, 0, "th_shengong");
    answer(&mut e, "th_shengong:discard_card", vec![cd(w1)]);
    answer(&mut e, "th_shengong:assist", vec![tag(1)]);
    while e.open_choice().is_some_and(|c| c.spec.prompt == "th_shengong:assist") {
        answer(&mut e, "th_shengong:assist", Vec::new());
    }
    assert_eq!(open(&e, "th_shengong:choose_card").spec.options.len(), 1, "help<hamper -> 1 option");
}

#[test]
fn l28_reg_th_shengong_weapons_all_6() {
    let p = test_pack();

    // 1. wushuangfangtianji: mode 1 discard target, mode 0 draw 1, cancel skips extra, Duel non-trigger.
    let mut e = clean_seats(&p, 3110);
    equip_named_via_play(&mut e, 0, "wushuangfangtianji");
    let sha1 = card_by(&e, |c| c.kind == CardKind::Sha);
    let k1 = card_by(&e, |c| c.kind == CardKind::Shan);
    let k2 = card_other(&e, k1, |c| c.kind == CardKind::Shan);
    give(&mut e, 0, sha1);
    give(&mut e, 1, k1);
    let h0 = hand(&e, 0);
    play(&mut e, 0, sha1, vec![1]);
    drain_pass_shan(&mut e);
    answer(&mut e, "wushuangfangtianji:choose_mode", vec![tag(1)]);
    answer(&mut e, "wushuangfangtianji:discard_card", vec![cd(k1)]);
    assert!(!e.state_core().player(1).hand.contains(&k1), "mode 1 discards target card");
    assert_eq!(hand(&e, 0), h0 - 1, "mode 1 does not draw");

    e.debug_set_turn(0, Phase::Play, 2);
    e.state_core_mut().turn.sha_used_play_phase = false;
    let sha2 = card_other(&e, sha1, |c| c.kind == CardKind::Sha);
    give(&mut e, 0, sha2);
    let h0 = hand(&e, 0);
    play(&mut e, 0, sha2, vec![1]);
    answer(&mut e, "wushuangfangtianji:choose_mode", vec![tag(0)]);
    assert_eq!(hand(&e, 0), h0 - 1 + 1, "mode 0 draws 1");

    e.debug_set_turn(0, Phase::Play, 3);
    e.state_core_mut().turn.sha_used_play_phase = false;
    let sha3 = card_by(&e, |c| c.kind == CardKind::Sha && c.suit == Suit::Club);
    give(&mut e, 0, sha3);
    give(&mut e, 1, k2);
    let h0 = hand(&e, 0);
    play(&mut e, 0, sha3, vec![1]);
    drain_pass_shan(&mut e);
    answer(&mut e, "wushuangfangtianji:choose_mode", Vec::new());
    assert_eq!(hand(&e, 0), h0 - 1, "cancel wushuangfangtianji draws nothing");
    assert!(e.state_core().player(1).hand.contains(&k2), "cancel wushuangfangtianji discards nothing");

    let duel = card_by(&e, |c| c.kind == CardKind::Duel);
    give(&mut e, 0, duel);
    play(&mut e, 0, duel, vec![1]);
    drain(&mut e);
    assert_ne!(
        e.open_choice().map(|c| c.spec.prompt),
        Some("wushuangfangtianji:choose_mode"),
        "Duel damage must not trigger wushuangfangtianji"
    );

    // 2. guilongzhanyuedao: red Sha direct hit (skips Bagua & Shan); black Sha dodgeable.
    let mut e = clean_seats(&p, 3111);
    equip_named_via_play(&mut e, 0, "guilongzhanyuedao");
    equip_named_via_play(&mut e, 1, "armor");
    let shan = card_by(&e, |c| c.kind == CardKind::Shan);
    give(&mut e, 1, shan);
    let red_sha = card_by(&e, |c| c.kind == CardKind::Sha && c.suit == Suit::Heart);
    give(&mut e, 0, red_sha);
    let hp1 = hp(&e, 1);
    play(&mut e, 0, red_sha, vec![1]);
    assert!(e.open_choice().is_none(), "red Sha skips Bagua & Shan");
    assert_eq!(hp(&e, 1), hp1 - 1);
    e.state_core_mut().turn.sha_used_play_phase = false;
    let _spade_top = push_top_card(&mut e, |c| c.suit == Suit::Spade);
    let black_sha = card_by(&e, |c| c.kind == CardKind::Sha && c.suit == Suit::Spade);
    give(&mut e, 0, black_sha);
    play(&mut e, 0, black_sha, vec![1]);
    assert_eq!(open(&e, "respond_shan").observer, 1, "black Sha remains dodgeable");

    // 3. chixueqingfeng: blocks armor & hand Shan during Sha, restored afterward.
    let mut e = clean_seats(&p, 3112);
    equip_named_via_play(&mut e, 0, "chixueqingfeng");
    equip_named_via_play(&mut e, 1, "armor");
    let _heart_top = push_top_card(&mut e, |c| c.suit == Suit::Heart);
    let black_sha = card_by(&e, |c| c.kind == CardKind::Sha && c.suit == Suit::Spade);
    let shan1 = card_by(&e, |c| c.kind == CardKind::Shan);
    give(&mut e, 0, black_sha);
    give(&mut e, 1, shan1);
    let hp1 = hp(&e, 1);
    play(&mut e, 0, black_sha, vec![1]);
    assert!(e.open_choice().is_none(), "chixueqingfeng disables armor & locks target hand Shan");
    assert_eq!(hp(&e, 1), hp1 - 1);
    assert_eq!(flag(&e, 1, "chixueqingfeng2_count"), 0, "chixueqingfeng2 clears after Sha");

    // 4. bintieshuangji: confirm loses 1 HP, reclaims Sha, draws 1, allows 2nd Sha; cancel skips.
    let mut e = clean_seats(&p, 3113);
    equip_named_via_play(&mut e, 0, "bintieshuangji");
    let sha_a = card_by(&e, |c| c.kind == CardKind::Sha);
    let sha_b = card_other(&e, sha_a, |c| c.kind == CardKind::Sha);
    let shan1 = card_by(&e, |c| c.kind == CardKind::Shan);
    give(&mut e, 0, sha_a);
    give(&mut e, 0, sha_b);
    give(&mut e, 1, shan1);
    let hp0 = hp(&e, 0);
    play(&mut e, 0, sha_a, vec![1]);
    answer(&mut e, "respond_shan", vec![cd(shan1)]);
    snapshot_roundtrip(&e, &p, vec![ChoiceOption::Confirm]);
    answer(&mut e, "bintieshuangji:confirm", vec![ChoiceOption::Confirm]);
    drain(&mut e);
    assert_eq!(hp(&e, 0), hp0 - 1, "bintieshuangji costs 1 HP");
    assert!(e.state_core().player(0).hand.contains(&sha_a), "bintieshuangji reclaims dodged Sha");
    assert!(!matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: sha_b, targets: vec![1] }),
        CommandOutcome::Rejected(_)
    ), "bintieshuangji increases Sha limit +1");

    e.debug_set_turn(0, Phase::Play, 2);
    e.state_core_mut().turn.sha_used_play_phase = false;
    let sha_c = card_by(&e, |c| c.kind == CardKind::Sha && c.suit == Suit::Club);
    let sha_d = card_by(&e, |c| c.kind == CardKind::Sha && c.suit == Suit::Diamond);
    let shan2 = card_other(&e, shan1, |c| c.kind == CardKind::Shan);
    give(&mut e, 0, sha_c);
    give(&mut e, 0, sha_d);
    give(&mut e, 1, shan2);
    let hp0_c = hp(&e, 0);
    play(&mut e, 0, sha_c, vec![1]);
    answer(&mut e, "respond_shan", vec![cd(shan2)]);
    answer(&mut e, "bintieshuangji:confirm", Vec::new());
    drain(&mut e);
    assert_eq!(hp(&e, 0), hp0_c, "cancel bintieshuangji costs 0 HP");
    assert!(!e.state_core().player(0).hand.contains(&sha_c), "cancel bintieshuangji does not reclaim");
    assert!(
        matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: sha_d, targets: vec![1] }),
            CommandOutcome::Rejected(_)
        ),
        "cancel bintieshuangji grants no extra Sha"
    );

    // 5. wutiesuolian: Sha links unlinked target; does not unlink already linked target.
    let mut e = clean_seats(&p, 3114);
    equip_named_via_play(&mut e, 0, "wutiesuolian");
    let sha_l = card_by(&e, |c| c.kind == CardKind::Sha);
    give(&mut e, 0, sha_l);
    play(&mut e, 0, sha_l, vec![1]);
    drain(&mut e);
    assert!(e.state_core().player(1).linked, "wutiesuolian links Sha target");
    e.state_core_mut().turn.sha_used_play_phase = false;
    let sha_l2 = card_other(&e, sha_l, |c| c.kind == CardKind::Sha);
    give(&mut e, 0, sha_l2);
    play(&mut e, 0, sha_l2, vec![1]);
    drain(&mut e);
    assert!(e.state_core().player(1).linked, "already linked target stays linked");

    // 6. wuxinghelingshan: FireSha -> Thunder reduces rewritten tengjia extra dmg (1 vs 2 on cancel); normal Sha non-trigger.
    let mut e = clean_seats(&p, 3115);
    e.assign_general_named(1, "th_majun").expect("seat 1 majun");
    e.state_core_mut().player_mut(1).max_hp = 5;
    e.state_core_mut().player_mut(1).hp = 5;
    e.debug_set_turn(1, Phase::Play, 1);
    let tj = card_named(&e, "tengjia");
    give(&mut e, 1, tj);
    activate(&mut e, 1, "th_jingxie");
    answer(&mut e, "th_jingxie:choose_card", vec![cd(tj)]);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: tj, targets: Vec::new() });
    e.debug_set_turn(0, Phase::Play, 1);
    e.state_core_mut().turn.sha_used_play_phase = false;
    equip_named_via_play(&mut e, 0, "wuxinghelingshan");
    let fsha = card_by(&e, |c| c.kind == CardKind::FireSha);
    give(&mut e, 0, fsha);
    let hp1 = hp(&e, 1);
    play(&mut e, 0, fsha, vec![1]);
    answer(&mut e, "wuxinghelingshan:choose_nature", vec![ChoiceOption::Confirm]);
    assert_eq!(hp(&e, 1), hp1 - 1, "converting FireSha -> Thunder avoids tengjia +1 fire dmg");
    e.state_core_mut().turn.sha_used_play_phase = false;
    let fsha2 = card_other(&e, fsha, |c| c.kind == CardKind::FireSha);
    give(&mut e, 0, fsha2);
    let hp1 = hp(&e, 1);
    play(&mut e, 0, fsha2, vec![1]);
    answer(&mut e, "wuxinghelingshan:choose_nature", Vec::new());
    assert_eq!(hp(&e, 1), hp1 - 2, "declining conversion keeps Fire (+1 tengjia dmg)");
    e.state_core_mut().turn.sha_used_play_phase = false;
    let norm_sha = card_by(&e, |c| c.kind == CardKind::Sha);
    give(&mut e, 0, norm_sha);
    let hp1 = hp(&e, 1);
    play(&mut e, 0, norm_sha, vec![1]);
    assert!(e.open_choice().is_none(), "normal Sha must not trigger wuxinghelingshan");
    assert_eq!(hp(&e, 1), hp1, "rewritten tengjia blocks normal Sha");
}

#[test]
fn l28_reg_th_shengong_armors_all_6() {
    let p = test_pack();

    // 1. linglongshimandai: sole target Heart cancels; Spade fails; cancel skips judge; SavageAttack ignores.
    let mut e = clean_seats(&p, 3116);
    equip_named_via_play(&mut e, 1, "linglongshimandai");
    let d1 = card_by(&e, |c| c.kind == CardKind::Dismantle);
    let _heart_j = push_top_card(&mut e, |c| c.suit == Suit::Heart);
    give(&mut e, 0, d1);
    play(&mut e, 0, d1, vec![1]);
    answer(&mut e, "linglongshimandai:confirm", vec![ChoiceOption::Confirm]);
    assert!(
        e.state_core().player(1).equips.contains(&card_named(&e, "linglongshimandai")),
        "Heart judge cancels single-target Dismantle"
    );
    let savage = card_by(&e, |c| c.kind == CardKind::SavageAttack);
    give(&mut e, 0, savage);
    play(&mut e, 0, savage, Vec::new());
    assert_ne!(e.open_choice().map(|c| c.spec.prompt), Some("linglongshimandai:confirm"));
    drain(&mut e);
    let d2 = card_other(&e, d1, |c| c.kind == CardKind::Dismantle);
    let _spade_j = push_top_card(&mut e, |c| c.suit == Suit::Spade);
    give(&mut e, 0, d2);
    play(&mut e, 0, d2, vec![1]);
    answer(&mut e, "linglongshimandai:confirm", vec![ChoiceOption::Confirm]);
    assert!(e.state_core().player(1).equips.is_empty(), "Spade judge fails to cancel Dismantle");
    equip_named_via_play(&mut e, 1, "linglongshimandai");
    let d3 = card_by(&e, |c| c.kind == CardKind::Dismantle && c.suit == Suit::Club);
    give(&mut e, 0, d3);
    play(&mut e, 0, d3, vec![1]);
    answer(&mut e, "linglongshimandai:confirm", Vec::new());
    assert!(e.state_core().player(1).equips.is_empty(), "cancel linglongshimandai lets Dismantle resolve");

    // 2. hongmianbaihuapao: blocks Fire/Thunder; normal hits; Qinggang pierces.
    let mut e = clean_seats(&p, 3117);
    equip_named_via_play(&mut e, 1, "hongmianbaihuapao");
    let fsha = card_by(&e, |c| c.kind == CardKind::FireSha);
    give(&mut e, 0, fsha);
    let hp1 = hp(&e, 1);
    play(&mut e, 0, fsha, vec![1]);
    assert_eq!(hp(&e, 1), hp1, "hongmianbaihuapao prevents Fire dmg");
    e.state_core_mut().turn.sha_used_play_phase = false;
    let nsha = card_by(&e, |c| c.kind == CardKind::Sha);
    give(&mut e, 0, nsha);
    play(&mut e, 0, nsha, vec![1]);
    assert_eq!(hp(&e, 1), hp1 - 1, "normal Sha hits through hongmianbaihuapao");
    e.state_core_mut().turn.sha_used_play_phase = false;
    equip_named_via_play(&mut e, 0, "qinggang");
    let fsha2 = card_other(&e, fsha, |c| c.kind == CardKind::FireSha);
    give(&mut e, 0, fsha2);
    let hp1 = hp(&e, 1);
    play(&mut e, 0, fsha2, vec![1]);
    assert_eq!(hp(&e, 1), hp1 - 1, "Qinggang pierces hongmianbaihuapao");

    // 3. qimenbagua: cancels Sha without Bagua judge; Qinggang pierces.
    let mut e = clean_seats(&p, 3118);
    equip_named_via_play(&mut e, 1, "qimenbagua");
    let sha = card_by(&e, |c| c.kind == CardKind::Sha);
    give(&mut e, 0, sha);
    let hp1 = hp(&e, 1);
    play(&mut e, 0, sha, vec![1]);
    assert!(!e.log().iter().any(|ev| matches!(ev, LogEvent::ArmorJudged { .. })));
    assert_eq!(hp(&e, 1), hp1, "qimenbagua negates Sha");
    equip_named_via_play(&mut e, 0, "qinggang");
    e.state_core_mut().turn.sha_used_play_phase = false;
    let sha2 = card_other(&e, sha, |c| c.kind == CardKind::Sha);
    give(&mut e, 0, sha2);
    play(&mut e, 0, sha2, vec![1]);
    assert_eq!(hp(&e, 1), hp1 - 1, "Qinggang pierces qimenbagua");

    // 4. guofengyupao: other player trick targeting rejected even with qinggang; AOE excludes.
    let mut e = clean_seats(&p, 3119);
    equip_named_via_play(&mut e, 1, "guofengyupao");
    let duel = card_by(&e, |c| c.kind == CardKind::Duel);
    give(&mut e, 0, duel);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: duel, targets: vec![1] }),
        CommandOutcome::Rejected(_)
    ));
    let savage = card_by(&e, |c| c.kind == CardKind::SavageAttack);
    give(&mut e, 0, savage);
    let (hp1, hp2) = (hp(&e, 1), hp(&e, 2));
    play(&mut e, 0, savage, Vec::new());
    drain(&mut e);
    assert_eq!((hp(&e, 1), hp(&e, 2)), (hp1, hp2 - 1), "guofengyupao excludes wearer from AOE");
    equip_named_via_play(&mut e, 0, "qinggang");
    assert!(
        matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: duel, targets: vec![1] }),
            CommandOutcome::Rejected(_)
        ),
        "Qinggang only applies to Sha and must NOT pierce guofengyupao trick protection"
    );

    // 5. huxinjing: >1 dmg triggers & discards; 1 dmg at hp>=2 does not; lethal 1 dmg at hp==1 triggers; cancel lets dmg resolve.
    let mut e = clean_seats(&p, 3120);
    equip_named_via_play(&mut e, 1, "huxinjing");
    let jiu = card_by(&e, |c| c.kind == CardKind::Jiu);
    let sha = card_by(&e, |c| c.kind == CardKind::Sha);
    give(&mut e, 0, jiu);
    give(&mut e, 0, sha);
    let hp1 = hp(&e, 1);
    play(&mut e, 0, jiu, Vec::new());
    play(&mut e, 0, sha, vec![1]);
    answer(&mut e, "huxinjing:confirm", vec![ChoiceOption::Confirm]);
    assert_eq!(hp(&e, 1), hp1, "huxinjing prevents >1 damage");
    assert!(e.state_core().player(1).equips.is_empty(), "huxinjing discards itself");

    equip_named_via_play(&mut e, 1, "huxinjing");
    e.state_core_mut().player_mut(1).hp = 2;
    e.debug_set_turn(0, Phase::Play, 2);
    e.state_core_mut().turn.sha_used_play_phase = false;
    let sha2 = card_other(&e, sha, |c| c.kind == CardKind::Sha);
    give(&mut e, 0, sha2);
    play(&mut e, 0, sha2, vec![1]);
    assert!(e.open_choice().is_none(), "non-lethal 1 dmg at hp==2 must not trigger huxinjing");
    assert_eq!(hp(&e, 1), 1);
    assert!(!e.state_core().player(1).equips.is_empty());

    e.debug_set_turn(0, Phase::Play, 3);
    e.state_core_mut().turn.sha_used_play_phase = false;
    let sha3 = card_by(&e, |c| c.kind == CardKind::Sha && c.suit == Suit::Club);
    give(&mut e, 0, sha3);
    play(&mut e, 0, sha3, vec![1]);
    answer(&mut e, "huxinjing:confirm", vec![ChoiceOption::Confirm]);
    assert_eq!(hp(&e, 1), 1, "lethal 1 dmg at hp==1 triggers huxinjing and prevents death");

    equip_named_via_play(&mut e, 1, "huxinjing");
    e.debug_set_turn(0, Phase::Play, 4);
    e.state_core_mut().turn.sha_used_play_phase = false;
    let sha4 = card_by(&e, |c| c.kind == CardKind::Sha && c.suit == Suit::Diamond);
    give(&mut e, 0, sha4);
    play(&mut e, 0, sha4, vec![1]);
    answer(&mut e, "huxinjing:confirm", Vec::new());
    drain(&mut e);
    assert!(hp(&e, 1) <= 0 || !e.state_core().player(1).alive, "cancel huxinjing lets lethal dmg resolve");

    // 6. heiguangkai: excludes wearer from multi-target SavageAttack & 2-target black Chain; single Duel hits.
    let mut e = clean_seats(&p, 3121);
    equip_named_via_play(&mut e, 1, "heiguangkai");
    let savage = card_by(&e, |c| c.kind == CardKind::SavageAttack);
    let chain = card_by(&e, |c| c.kind == CardKind::Chain && matches!(c.suit, Suit::Spade | Suit::Club));
    let duel = card_by(&e, |c| c.kind == CardKind::Duel);
    give(&mut e, 0, savage);
    give(&mut e, 0, chain);
    give(&mut e, 0, duel);
    let (hp1, hp2) = (hp(&e, 1), hp(&e, 2));
    play(&mut e, 0, savage, Vec::new());
    drain(&mut e);
    assert_eq!((hp(&e, 1), hp(&e, 2)), (hp1, hp2 - 1), "heiguangkai blocks multi-target SavageAttack");
    play(&mut e, 0, chain, vec![1, 2]);
    assert_eq!((e.state_core().player(1).linked, e.state_core().player(2).linked), (false, true));
    play(&mut e, 0, duel, vec![1]);
    drain(&mut e);
    assert_eq!(hp(&e, 1), hp1 - 1, "single-target Duel hits through heiguangkai");
}

#[test]
fn l28_reg_th_shengong_treasures_all_6() {
    let p = test_pack();

    // 1. shufazijinguan: Prepare deals 1 dmg; cancel deals 0.
    let mut e = clean_seats(&p, 3122);
    equip_named_via_play(&mut e, 0, "shufazijinguan");
    let hp1 = hp(&e, 1);
    e.debug_set_turn(3, Phase::End, 1);
    let _ = submit(&mut e, 3, GameAction::NextPhase);
    answer(&mut e, "shufazijinguan:choose_target", vec![pl(1)]);
    assert_eq!(hp(&e, 1), hp1 - 1, "shufazijinguan deals 1 dmg at Prepare");
    e.debug_set_turn(3, Phase::End, 2);
    let _ = submit(&mut e, 3, GameAction::NextPhase);
    answer(&mut e, "shufazijinguan:choose_target", Vec::new());
    assert_eq!(hp(&e, 1), hp1 - 1, "cancel shufazijinguan deals 0 dmg");

    // 2. xuwangzhimian: Draw draws 4 (+2), hand limit -1 enforced in Discard.
    let mut e = clean_seats(&p, 3123);
    equip_named_via_play(&mut e, 0, "xuwangzhimian");
    e.debug_set_turn(0, Phase::Judge, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert_eq!(hand(&e, 0), 4, "xuwangzhimian draws 2+2=4");
    assert_eq!(e.state_core().hand_limit(e.catalog(), 0), (hp(&e, 0) - 1) as usize);
    let first = e.state_core().player(0).hand[0];
    e.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert_eq!(open(&e, "discard_to_limit").spec.min, 1, "xuwangzhimian enforces hp-1 in Discard");
    answer(&mut e, "discard_to_limit", vec![cd(first)]);
    assert_eq!(hand(&e, 0), (hp(&e, 0) - 1) as usize);

    // 3. tianjitu: on equip discards 1 other card; on lose draws up to 5 (0 if hand >= 5).
    let mut e = clean_seats(&p, 3124);
    let extra = card_by(&e, |c| c.kind == CardKind::Sha);
    give(&mut e, 0, extra);
    equip_named_via_play(&mut e, 0, "tianjitu");
    answer(&mut e, "tianjitu:discard_card", vec![cd(extra)]);
    assert!(e.state_core().discard.contains(&extra) && e.state_core().player(0).hand.is_empty());
    e.debug_set_turn(1, Phase::Play, 1);
    let d1 = card_by(&e, |c| c.kind == CardKind::Dismantle);
    give(&mut e, 1, d1);
    play(&mut e, 1, d1, vec![0]);
    drain(&mut e);
    assert_eq!(hand(&e, 0), 5, "losing tianjitu draws up to 5");
    e.debug_set_turn(0, Phase::Play, 2);
    let extra2 = card_other(&e, extra, |c| c.kind == CardKind::Sha);
    give(&mut e, 0, extra2);
    equip_named_via_play(&mut e, 0, "tianjitu");
    answer(&mut e, "tianjitu:discard_card", vec![cd(extra2)]);
    assert_eq!(hand(&e, 0), 5);
    // Replace tianjitu in the treasure slot while holding 5 hand cards -> on-lose hook draws 0.
    equip_named_via_play(&mut e, 0, "sanlve");
    drain(&mut e);
    assert_eq!(hand(&e, 0), 5, "losing tianjitu with >=5 hand draws 0");

    // 4. taigongyinfu: Play begin links target; Play end recasts 1 hand card; cancel skips both.
    let mut e = clean_seats(&p, 3125);
    equip_named_via_play(&mut e, 0, "taigongyinfu");
    let trash = card_by(&e, |c| c.kind == CardKind::Sha);
    give(&mut e, 0, trash);
    e.debug_set_turn(0, Phase::Draw, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, "taigongyinfu:choose_target", vec![pl(2)]);
    assert!(e.state_core().player(2).linked, "taigongyinfu links at Play begin");
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let h0 = hand(&e, 0);
    answer(&mut e, "taigongyinfu:recast_card", vec![cd(trash)]);
    assert!(e.state_core().discard.contains(&trash) && hand(&e, 0) == h0);
    e.debug_set_turn(0, Phase::Draw, 2);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, "taigongyinfu:choose_target", Vec::new());
    assert!(e.state_core().player(2).linked, "cancel taigongyinfu link preserves state");
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let hand_before_cancel = e.state_core().player(0).hand.clone();
    answer(&mut e, "taigongyinfu:recast_card", Vec::new());
    assert_eq!(e.state_core().player(0).hand, hand_before_cancel, "cancel recast preserves hand");

    // 5. sanlve: baseline distance 2 blocked without sanlve; with sanlve range +1, hand limit +1, Sha limit +1.
    let mut e = clean_seats(&p, 3126);
    let s0 = card_by(&e, |c| c.kind == CardKind::Sha && c.suit == Suit::Club);
    give(&mut e, 0, s0);
    assert!(
        matches!(submit(&mut e, 0, GameAction::PlayCard { card: s0, targets: vec![2] }), CommandOutcome::Rejected(_)),
        "distance 2 blocked without sanlve"
    );
    equip_named_via_play(&mut e, 0, "sanlve");
    assert_eq!(e.state_core().attack_range(e.catalog(), 0), 2);
    assert_eq!(e.state_core().hand_limit(e.catalog(), 0), (hp(&e, 0) + 1) as usize);
    let s1 = card_by(&e, |c| c.kind == CardKind::Sha && c.suit == Suit::Spade);
    let s2 = card_by(&e, |c| c.kind == CardKind::Sha && c.suit == Suit::Heart);
    give(&mut e, 0, s1);
    give(&mut e, 0, s2);
    play(&mut e, 0, s1, vec![2]);
    drain(&mut e);
    assert!(!matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: s2, targets: vec![2] }),
        CommandOutcome::Rejected(_)
    ));

    // 6. zhaogujing: Play end cancel draws 0; Duel asks target; Wuzhong keeps card & draws 2.
    let mut e = clean_seats(&p, 3127);
    equip_named_via_play(&mut e, 0, "zhaogujing");
    let wz = card_by(&e, |c| c.kind == CardKind::Wuzhong);
    let duel = card_by(&e, |c| c.kind == CardKind::Duel);
    give(&mut e, 0, wz);
    give(&mut e, 0, duel);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let h0 = hand(&e, 0);
    answer(&mut e, "zhaogujing:choose_card", Vec::new());
    assert_eq!(hand(&e, 0), h0, "cancel zhaogujing draws 0");
    drain(&mut e);

    e.debug_set_turn(0, Phase::Play, 2);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let hp1 = hp(&e, 1);
    answer(&mut e, "zhaogujing:choose_card", vec![cd(duel)]);
    answer(&mut e, "zhaogujing:choose_target", vec![pl(1)]);
    drain(&mut e);
    assert!(e.state_core().player(0).hand.contains(&duel), "zhaogujing preserves revealed Duel");
    assert_eq!(hp(&e, 1), hp1 - 1, "zhaogujing virtual Duel hits seat 1");

    e.debug_set_turn(0, Phase::Play, 3);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let h0 = hand(&e, 0);
    answer(&mut e, "zhaogujing:choose_card", vec![cd(wz)]);
    drain(&mut e);
    assert!(e.state_core().player(0).hand.contains(&wz), "zhaogujing preserves original Wuzhong");
    assert_eq!(hand(&e, 0), h0 + 2, "zhaogujing virtual Wuzhong draws 2");
}

#[test]
fn l28_reg_reviewer_round2_fixes() {
    let p = test_pack();

    // 1) Qinggang is Sha-only: does NOT pierce linglongshimandai/huxinjing/hongmianbaihuapao/rewrite_tengjia on non-Sha.
    let mut e = clean_seats(&p, 3201);
    equip_named_via_play(&mut e, 0, "qinggang");
    equip_named_via_play(&mut e, 1, "linglongshimandai");
    let d1 = card_by(&e, |c| c.kind == CardKind::Dismantle);
    let _heart_j = push_top_card(&mut e, |c| c.suit == Suit::Heart);
    give(&mut e, 0, d1);
    play(&mut e, 0, d1, vec![1]);
    answer(&mut e, "linglongshimandai:confirm", vec![ChoiceOption::Confirm]);
    assert!(
        e.state_core().player(1).equips.contains(&card_named(&e, "linglongshimandai")),
        "Qinggang must NOT block linglongshimandai on Dismantle"
    );

    let mut e = clean_seats(&p, 3202);
    equip_named_via_play(&mut e, 0, "qinggang");
    equip_named_via_play(&mut e, 1, "huxinjing");
    e.state_core_mut().player_mut(1).hp = 1;
    let duel = card_by(&e, |c| c.kind == CardKind::Duel);
    give(&mut e, 0, duel);
    play(&mut e, 0, duel, vec![1]);
    answer(&mut e, "huxinjing:confirm", vec![ChoiceOption::Confirm]);
    assert_eq!(hp(&e, 1), 1, "Qinggang must NOT pierce huxinjing on Duel lethal damage");

    // hongmianbaihuapao blocks virtual Fire damage (`huogong` via zhaogujing) even with Qinggang.
    let mut e = clean_seats(&p, 3203);
    equip_named_via_play(&mut e, 0, "qinggang");
    equip_named_via_play(&mut e, 0, "zhaogujing");
    equip_named_via_play(&mut e, 1, "hongmianbaihuapao");
    let hg = card_named(&e, "huogong");
    let h_fodder = card_by(&e, |c| c.kind == CardKind::Shan && c.suit == Suit::Heart);
    let h_target = card_other(&e, h_fodder, |c| c.kind == CardKind::Shan && c.suit == Suit::Heart);
    give(&mut e, 0, hg);
    give(&mut e, 0, h_fodder);
    give(&mut e, 1, h_target);
    let hp1 = hp(&e, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, "zhaogujing:choose_card", vec![cd(hg)]);
    answer(&mut e, "zhaogujing:choose_target", vec![pl(1)]);
    answer(&mut e, "zhaogujing:choose_card", vec![cd(h_fodder)]);
    assert_eq!(hp(&e, 1), hp1, "Qinggang must NOT pierce hongmianbaihuapao on Fire huogong");

    // 2) rewrite_tengjia vs Qinggang (Sha pierces; SavageAttack & huogong do not) AND vs zhaogujing virtual Sha/SavageAttack.
    let mut e = clean_seats(&p, 3204);
    e.assign_general_named(1, "th_majun").expect("seat 1 majun");
    e.state_core_mut().player_mut(1).max_hp = 5;
    e.state_core_mut().player_mut(1).hp = 5;
    e.debug_set_turn(1, Phase::Play, 1);
    let tj = card_named(&e, "tengjia");
    give(&mut e, 1, tj);
    activate(&mut e, 1, "th_jingxie");
    answer(&mut e, "th_jingxie:choose_card", vec![cd(tj)]);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: tj, targets: Vec::new() });
    e.debug_set_turn(0, Phase::Play, 1);
    equip_named_via_play(&mut e, 0, "qinggang");
    equip_named_via_play(&mut e, 0, "zhaogujing");
    let savage = card_by(&e, |c| c.kind == CardKind::SavageAttack);
    give(&mut e, 0, savage);
    let hp1 = hp(&e, 1);
    play(&mut e, 0, savage, Vec::new());
    drain(&mut e);
    assert_eq!(hp(&e, 1), hp1, "Qinggang must NOT pierce rewrite_tengjia on SavageAttack");

    let hg = card_named(&e, "huogong");
    let h_fodder = card_by(&e, |c| c.kind == CardKind::Shan && c.suit == Suit::Heart);
    let h_target = card_other(&e, h_fodder, |c| c.kind == CardKind::Shan && c.suit == Suit::Heart);
    let norm_sha = card_by(&e, |c| c.kind == CardKind::Sha);
    give(&mut e, 0, hg);
    give(&mut e, 0, h_fodder);
    give(&mut e, 1, h_target);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, "zhaogujing:choose_card", vec![cd(hg)]);
    answer(&mut e, "zhaogujing:choose_target", vec![pl(1)]);
    answer(&mut e, "zhaogujing:choose_card", vec![cd(h_fodder)]);
    assert_eq!(hp(&e, 1), hp1 - 2, "Qinggang must NOT bypass rewrite_tengjia +1 Fire dmg on huogong");

    // Rewritten tengjia blocks zhaogujing virtual Sha and virtual SavageAttack (no Qinggang on seat 0).
    let qg = card_named(&e, "qinggang");
    e.state_core_mut().player_mut(0).equips.retain(|&c| c != qg);
    e.state_core_mut().discard.push(qg);
    e.debug_set_turn(0, Phase::Play, 2);
    clear_seat_all(&mut e, 0);
    let h1_left: Vec<u32> = e.state_core_mut().player_mut(1).hand.drain(..).collect();
    e.state_core_mut().discard.extend(h1_left);
    equip_named_via_play(&mut e, 0, "zhaogujing");
    give(&mut e, 0, norm_sha);
    let hp1 = hp(&e, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, "zhaogujing:choose_card", vec![cd(norm_sha)]);
    answer(&mut e, "zhaogujing:choose_target", vec![pl(1)]);
    drain(&mut e);
    assert_eq!(hp(&e, 1), hp1, "rewrite_tengjia blocks zhaogujing virtual normal Sha");
    assert!(e.state_core().player(0).hand.contains(&norm_sha));

    e.debug_set_turn(0, Phase::Play, 3);
    clear_seat_all(&mut e, 0);
    equip_named_via_play(&mut e, 0, "zhaogujing");
    let savage2 = card_other(&e, savage, |c| c.kind == CardKind::SavageAttack);
    give(&mut e, 0, savage2);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, "zhaogujing:choose_card", vec![cd(savage2)]);
    drain(&mut e);
    assert_eq!(hp(&e, 1), hp1, "rewrite_tengjia blocks zhaogujing virtual SavageAttack");

    // 3) bintieshuangji triggers when target's Bagua Shield dodges Sha.
    let mut e = clean_seats(&p, 3205);
    equip_named_via_play(&mut e, 0, "bintieshuangji");
    equip_named_via_play(&mut e, 1, "armor");
    let _heart_j = push_top_card(&mut e, |c| c.suit == Suit::Heart);
    let sha_b1 = card_by(&e, |c| c.kind == CardKind::Sha);
    let sha_b2 = card_other(&e, sha_b1, |c| c.kind == CardKind::Sha);
    give(&mut e, 0, sha_b1);
    give(&mut e, 0, sha_b2);
    let hp0 = hp(&e, 0);
    play(&mut e, 0, sha_b1, vec![1]);
    answer(&mut e, "bintieshuangji:confirm", vec![ChoiceOption::Confirm]);
    drain(&mut e);
    assert_eq!(hp(&e, 0), hp0 - 1, "bintieshuangji triggers on Bagua dodge");
    assert!(e.state_core().player(0).hand.contains(&sha_b1), "reclaims Sha dodged by Bagua");
    assert!(!matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: sha_b2, targets: vec![1] }),
        CommandOutcome::Rejected(_)
    ));

    // 4) heiguangkai excludes wearer on non-black multi-target damage trick (`Heart` Bingpotong).
    let mut e = clean_seats(&p, 3206);
    equip_named_via_play(&mut e, 1, "heiguangkai");
    let bp = card_named(&e, "bingpotong");
    let h0_c = card_by(&e, |c| c.kind == CardKind::Shan && c.suit == Suit::Heart);
    let h1_c = card_by(&e, |c| c.kind == CardKind::Tao && c.suit == Suit::Heart);
    let h2_c = card_other(&e, h1_c, |c| c.kind == CardKind::Tao && c.suit == Suit::Heart);
    give(&mut e, 0, bp);
    give(&mut e, 0, h0_c);
    give(&mut e, 1, h1_c);
    give(&mut e, 2, h2_c);
    let (hp1, hp2) = (hp(&e, 1), hp(&e, 2));
    play(&mut e, 0, bp, vec![1, 2]);
    drain(&mut e);
    assert_eq!((hp(&e, 1), hp(&e, 2)), (hp1, hp2 - 1), "heiguangkai excludes wearer on Heart Bingpotong");

    // 5) zhaogujing supports Jiedao (holder -> victim -> virtual Jiedao) while preserving hand card.
    let mut e = clean_seats(&p, 3207);
    equip_named_via_play(&mut e, 0, "zhaogujing");
    equip_named_via_play(&mut e, 1, "qinggang");
    let jd = card_by(&e, |c| c.kind == CardKind::Jiedao);
    give(&mut e, 0, jd);
    let hp2 = hp(&e, 2);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, "zhaogujing:choose_card", vec![cd(jd)]);
    answer(&mut e, "zhaogujing:choose_target", vec![pl(1)]);
    answer(&mut e, "zhaogujing:choose_target", vec![pl(2)]);
    answer(&mut e, "jiedao_sha_or_give", vec![tag(0)]);
    assert_eq!(hp(&e, 2), hp2 - 1, "zhaogujing virtual Jiedao hits victim");
    assert!(e.state_core().player(0).hand.contains(&jd), "zhaogujing keeps Jiedao in hand");

    // 6) taigongyinfu recast moves card to discard and draws 1 WITHOUT CardsDiscarded (连营 on last card draws 0 extra).
    let mut e = clean_seats(&p, 3208);
    e.debug_grant(0, "连营");
    equip_named_via_play(&mut e, 0, "taigongyinfu");
    let trash = card_by(&e, |c| c.kind == CardKind::Sha);
    give(&mut e, 0, trash);
    e.debug_set_turn(0, Phase::Draw, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, "taigongyinfu:choose_target", Vec::new());
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, "taigongyinfu:recast_card", vec![cd(trash)]);
    assert!(e.state_core().discard.contains(&trash));
    assert_eq!(hand(&e, 0), 1, "recast must draw 1 without triggering 连营 discard draw");
    assert!(
        !e.log().iter().any(|ev| matches!(ev, LogEvent::CardDiscarded { player: 0, cards } if cards.contains(&trash))),
        "recast must not emit CardDiscarded"
    );

    // 7) wuxinghelingshan preserves u.use_as == None on physical FireSha so th_chengye still records it.
    let mut e = fresh(&p, 3209, "th_mamidi");
    clear_seat_all(&mut e, 0);
    clear_seat_all(&mut e, 1);
    clear_seat_all(&mut e, 2);
    e.debug_set_turn(1, Phase::Play, 1);
    equip_named_via_play(&mut e, 1, "wuxinghelingshan");
    let fsha = card_by(&e, |c| c.kind == CardKind::FireSha);
    give(&mut e, 1, fsha);
    assert_eq!(flag(&e, 0, "th_chengye_mask"), 0);
    play(&mut e, 1, fsha, vec![2]);
    answer(&mut e, "wuxinghelingshan:choose_nature", vec![ChoiceOption::Confirm]);
    drain(&mut e);
    assert_eq!(
        flag(&e, 0, "th_chengye_mask") & (1 << 1),
        1 << 1,
        "wuxinghelingshan physical FireSha flip must preserve u.use_as == None for th_chengye"
    );
}
