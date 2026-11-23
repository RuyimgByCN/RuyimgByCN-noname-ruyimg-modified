use super::common::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::event::{DamageNature, DetachedKind, Phase};
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction};

fn find_non_offdeck_card(e: &Engine, kind: CardKind, suit_pred: fn(Suit) -> bool) -> Option<u32> {
    let cat = e.catalog();
    (0..cat.cards.len() as u32).find(|&c| {
        !cat.cards[c as usize].off_deck
            && cat.card_kind(c) == kind
            && suit_pred(cat.cards[c as usize].suit)
    })
}

fn find_offdeck_or_any_kind(e: &Engine, kind: CardKind) -> Option<u32> {
    let cat = e.catalog();
    (0..cat.cards.len() as u32).find(|&c| cat.card_kind(c) == kind)
}

#[test]
fn u05_b4_r4_group_subskills_in_real_general_assembly_and_guijin2() {
    let (v, mut e) = start_xianjian_only(8490, 4);
    assemble_general_by_name(&mut e, 0, "pal_jiangcheng");
    assemble_general_by_name(&mut e, 1, "pal_xuanxiao");
    assemble_general_by_name(&mut e, 2, "pal_changqing");
    assemble_general_by_name(&mut e, 3, "pal_zixuan");

    for (seat, expected) in [
        (0u8, &["yanzhan", "yanzhan2"][..]),
        (1u8, &["xuanyan", "xuanyan2", "xuanyan3", "xfenxin", "xfenxin2"][..]),
        (2u8, &["luanjian", "luanjian2", "ctianfu", "ctianfu3"][..]),
        (3u8, &["shuiyun", "shuiyun5", "wangyou", "wangyou2"][..]),
    ] {
        for &sk in expected {
            let def = e.catalog().skill_by_name(sk).unwrap_or_else(|| panic!("skill {sk}"));
            assert!(
                e.state_core().skills.contains_key(&(seat, def)),
                "real general assembly on seat {seat} must attach subskill {sk}"
            );
        }
    }

    // Check remaining group-bearing generals on fresh engines.
    let (_, mut e2) = start_xianjian_only(8491, 4);
    assemble_general_by_name(&mut e2, 0, "pal_zhaoliner");
    assemble_general_by_name(&mut e2, 1, "pal_wangxiaohu");
    assemble_general_by_name(&mut e2, 2, "pal_xuejian");
    assemble_general_by_name(&mut e2, 3, "pal_xingxuan");
    for (seat, expected) in [
        (0u8, &["tianshe", "tianshe2"][..]),
        (1u8, &["husha", "husha_count"][..]),
        (2u8, &["xshuangren", "xshuangren2"][..]),
        (3u8, &["leiyu", "leiyu2", "leiyu4"][..]),
    ] {
        for &sk in expected {
            let def = e2.catalog().skill_by_name(sk).unwrap();
            assert!(
                e2.state_core().skills.contains_key(&(seat, def)),
                "real general assembly on seat {seat} must attach subskill {sk}"
            );
        }
    }
    let (_, mut e2b) = start_xianjian_only(84910, 3);
    assemble_general_by_name(&mut e2b, 0, "pal_xiahoujinxuan");
    assemble_general_by_name(&mut e2b, 1, "pal_yuntianhe");
    assemble_general_by_name(&mut e2b, 2, "pal_liumengli");
    for (seat, expected) in [
        (0u8, &["danqing", "danqing_count"][..]),
        (1u8, &["zhuyue", "zhuyue2"][..]),
        (2u8, &["xjzhimeng", "xjzhimeng3"][..]),
    ] {
        for &sk in expected {
            let def = e2b.catalog().skill_by_name(sk).unwrap();
            assert!(
                e2b.state_core().skills.contains_key(&(seat, def)),
                "real general assembly on seat {seat} must attach subskill {sk}"
            );
        }
    }

    // Real pal_jiangcheng uses yanzhan without any debug_grant("yanzhan2"):
    // yanzhan2 grants yanzhan4 allowing an extra Sha in the same Play phase!
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_set_hp(1, 4);
    let red_c = find_non_offdeck_card(&e, CardKind::Wuzhong, |s| s == Suit::Heart).unwrap();
    force_move_card_to_hand(&mut e, &v, 0, red_c);
    let yz_def = e.catalog().skill_by_name("yanzhan").unwrap();
    submit(&mut e, 0, GameAction::ActivateSkill { skill: yz_def });
    answer_open(&mut e, vec![ChoiceOption::Card(red_c)]);
    answer_open(&mut e, vec![ChoiceOption::Player(1)]);
    drain_pass_all(&mut e);
    assert!(
        e.state_core().skills.contains_key(&(0, e.catalog().skill_by_name("yanzhan4").unwrap())),
        "assembled pal_jiangcheng yanzhan2 must grant yanzhan4 without debug_grant"
    );
    let extra_sha = e.debug_place(0, "sha");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: extra_sha, targets: vec![1] });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "yanzhan4 allows extra Sha: {out:?}");
    drain_pass_all(&mut e);

    // Real pal_mingxiu guijin attaches guijin2 to recipient!
    let (_, mut e3) = start_xianjian_only(8492, 2);
    assemble_general_by_name(&mut e3, 0, "pal_mingxiu");
    advance_to_play(&mut e3, 0);
    let gj_def = e3.catalog().skill_by_name("guijin").unwrap();
    submit(&mut e3, 0, GameAction::ActivateSkill { skill: gj_def });
    let pick_c = e3.open_choice().expect("guijin card pick").clone();
    let card_opt = pick_c.spec.options.into_iter().find(|o| matches!(o, ChoiceOption::Card(_))).unwrap();
    answer_open(&mut e3, vec![card_opt]);
    answer_open(&mut e3, vec![ChoiceOption::Player(1)]);
    drain_pass_all(&mut e3);
    assert!(
        e3.state_core().skills.contains_key(&(1, e3.catalog().skill_by_name("guijin2").unwrap())),
        "guijin must attach guijin2 to recipient"
    );
}

#[test]
fn u05_b4_r4_fengyin_debuff_lifetime_and_locked_vs_nonlocked_seal() {
    let (_, mut e) = start_xianjian_only(8493, 2);
    // Seat 0: guanyu / generic + qiongguang; Seat 1: pal_jiangyunfan (non-locked xunying + forced/locked liefeng).
    assemble_general_by_name(&mut e, 1, "pal_jiangyunfan");
    let xy_def = e.catalog().skill_by_name("xunying").unwrap();
    let lf_def = e.catalog().skill_by_name("liefeng").unwrap();
    assert!(e.state_core().has_skill(1, xy_def));
    assert!(e.state_core().has_skill(1, lf_def));

    advance_to_play(&mut e, 0);
    e.debug_grant(0, "qiongguang");
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    // Force u05_apply_debuff pool to ONLY [5] (fengyin):
    // seat 1 linked=true, empty hand/equips, hp=1, remove all delayed tricks from deck/discard (already none on seat 1 + purge deck/discard).
    let mut snap = e.save();
    snap.core.players[1].linked = true;
    snap.core.players[1].hp = 1;
    let delayed_kinds = [CardKind::Le, CardKind::Bing, CardKind::ShanDian, CardKind::HongShui, CardKind::Guiyoujie];
    let delayed_ids: Vec<u32> = (0..e.catalog().cards.len() as u32)
        .filter(|&c| delayed_kinds.contains(&e.catalog().card_kind(c)))
        .collect();
    snap.core.deck.retain(|c| !delayed_ids.contains(c));
    snap.core.discard.retain(|c| !delayed_ids.contains(c));
    snap.core.players[1].judge.clear();
    snap.core.detached_cards.extend(delayed_ids.into_iter().map(|c| (c, DetachedKind::Unclaimed)));
    let verified = prepare_formal_packages(&["shaai-xianjian-upstream-noname"]);
    e = content_native_loader::restore_locked_engine(&snap, &verified).expect("restore debuff setup");

    // Seat 0 discards 2 cards -> End phase -> qiongguang fires -> confirm -> applies fengyin ONLY!
    e.debug_set_hp(0, 1);
    e.debug_place(0, "tao");
    e.debug_place(0, "tao");
    e.debug_place(0, "shan");
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard
    let dc = e.open_choice().expect("discard 2 ask").clone();
    let picks: Vec<ChoiceOption> = dc.spec.options.iter().take(2).copied().collect();
    submit(&mut e, dc.observer, GameAction::Answer { choice_id: dc.id, selection: picks });
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard -> End -> triggers qiongguang
    let qg = e.open_choice().expect("qiongguang confirm").clone();
    assert_eq!(qg.spec.prompt, "xj_u05_qiongguang_confirm");
    answer_open(&mut e, vec![ChoiceOption::Confirm]);
    drain_pass_all(&mut e);

    // Seat 0 keeps enough HP not to die when Seat 1 plays Sha under fengyin.
    e.debug_set_hp(0, 5);

    // Owner (Seat 0) turn ended -> fengyin on Seat 1 MUST STILL BE ACTIVE during Seat 1's turn!
    assert!(!e.state_core().player(1).fengyin_active, "u05 fengyin must NOT set Boss 应昭 fengyin_active hand-block");
    assert!(!e.state_core().has_skill(1, xy_def), "non-locked xunying disabled under fengyin");
    assert!(e.state_core().has_skill(1, lf_def), "forced/locked liefeng stays enabled under fengyin");

    // Step into Seat 1's Play phase: hand Sha MUST be playable, and playing 2 cards MUST trigger liefeng (granting yanzhan),
    // while non-locked xunying stays sealed!
    if e.turn().current != 1 || e.turn().phase != Phase::Play {
        step_full_turn_to(&mut e, 1);
    }
    assert!(!e.state_core().has_skill(1, xy_def), "xunying stays disabled inside target turn");
    assert!(e.state_core().has_skill(1, lf_def), "liefeng stays enabled inside target turn");
    let p1_wz = e.debug_place(1, "wuzhong");
    let p1_sha = e.debug_place(1, "sha");
    let o1 = submit(&mut e, 1, GameAction::PlayCard { card: p1_wz, targets: vec![] });
    assert!(!matches!(o1, CommandOutcome::Rejected(_)), "hand card playable under fengyin: {o1:?}");
    drain_pass_all(&mut e);
    let o2 = submit(&mut e, 1, GameAction::PlayCard { card: p1_sha, targets: vec![0] });
    assert!(!matches!(o2, CommandOutcome::Rejected(_)), "hand Sha MUST be playable under fengyin: {o2:?}");
    drain_pass_all(&mut e);
    assert!(
        e.state_core().skills.contains_key(&(1, e.catalog().skill_by_name("yanzhan").unwrap())),
        "liefeng triggered on 2nd card played even under fengyin"
    );

    // End Seat 1's turn (target's phaseAfter) -> fengyin expires and xunying is re-enabled!
    step_full_turn_to(&mut e, 0);
    assert_eq!((e.turn().current, e.turn().phase), (0, Phase::Play));
    assert!(
        e.state_core().has_skill(1, xy_def),
        "xunying must re-enable after target's phaseAfter"
    );
}

#[test]
fn u05_b4_r4_yujia_yanshi_gujian_jiguan_and_use_as() {
    let (v, mut e) = start_xianjian_full(8494, 2);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_set_hp(0, 6);
    e.debug_set_hp(1, 6);
    e.debug_grant(0, "yujia");
    e.debug_grant(0, "yanshi");
    e.debug_grant(0, "xjzongyu");

    // 1) Move non-gujian CardKind::Jiguan out of deck/discard/detached (into seat 1 off_court to keep conservation),
    // leaving ONLY Gujian `Feibiao` / `Bingpotong` to prove yujia's jiguan pool matches Gujian type:"jiguan" cards too!
    let mut snap = e.save();
    let pure_jiguans: Vec<u32> = (0..e.catalog().cards.len() as u32)
        .filter(|&c| e.catalog().card_kind(c) == CardKind::Jiguan)
        .collect();
    snap.core.deck.retain(|c| !pure_jiguans.contains(c));
    snap.core.discard.retain(|c| !pure_jiguans.contains(c));
    snap.core.detached_cards.retain(|(c, _)| !pure_jiguans.contains(c));
    snap.core.players[1].off_court.extend(pure_jiguans);
    let feibiao = find_offdeck_or_any_kind(&e, CardKind::Feibiao).expect("gujian feibiao");
    let bingpotong = find_offdeck_or_any_kind(&e, CardKind::Bingpotong).expect("gujian bingpotong");
    snap.core.deck.retain(|&c| c != feibiao && c != bingpotong);
    snap.core.discard.retain(|&c| c != feibiao && c != bingpotong);
    snap.core.detached_cards.retain(|&(c, _)| c != feibiao && c != bingpotong);
    snap.core.deck.push(bingpotong);
    snap.core.players[0].hand.push(feibiao);
    // Keep one equip to trigger yujia later
    let eq = find_non_offdeck_card(&e, CardKind::Zhuge, |_| true).unwrap();
    snap.core.deck.retain(|&c| c != eq);
    snap.core.discard.retain(|&c| c != eq);
    snap.core.players[0].hand.push(eq);
    // Upstream Feibiao requires globalFrom: 2 (`play_card_targets.rs`); equip seat 1 with +1 horse so distance(0, 1) == 2 in 2-seat game.
    let plus_horse = find_non_offdeck_card(&e, CardKind::HorsePlus, |_| true).unwrap();
    snap.core.deck.retain(|&c| c != plus_horse);
    snap.core.discard.retain(|&c| c != plus_horse);
    snap.core.players[1].equips.push(plus_horse);
    e = content_native_loader::restore_locked_engine(&snap, &v).expect("restore gujian jiguan setup");

    // Use Gujian Feibiao -> increments both yujia count and yanshi count + sets yanshi2 turn mark!
    let fb_res = submit(&mut e, 0, GameAction::PlayCard { card: feibiao, targets: vec![1] });
    assert!(
        !matches!(fb_res, CommandOutcome::Rejected(_)),
        "PlayCard Feibiao rejected: {fb_res:?}"
    );
    drain_pass_all(&mut e);
    assert_eq!(e.state_core().vm_aux.flags.get(&(0, "xj_yujia_count".to_string())).copied().unwrap_or(0), 1);
    assert_eq!(e.state_core().vm_aux.flags.get(&(0, "xj_yanshi_count".to_string())).copied().unwrap_or(0), 1);
    assert_eq!(e.state_core().vm_aux.flags.get(&(0, "xj_yanshi2_turn".to_string())).copied().unwrap_or(0), 1);

    // Negative: non-jiguan Wuzhong does NOT increment counts
    let wz = e.debug_place(0, "wuzhong");
    submit(&mut e, 0, GameAction::PlayCard { card: wz, targets: vec![] });
    drain_pass_all(&mut e);
    assert_eq!(e.state_core().vm_aux.flags.get(&(0, "xj_yujia_count".to_string())).copied().unwrap_or(0), 1);

    // Activate yujia via real equip play when ONLY Gujian Bingpotong is in deck -> yujia finds it!
    submit(&mut e, 0, GameAction::PlayCard { card: eq, targets: vec![] });
    let yjc = e.open_choice().expect("yujia confirm with Gujian Bingpotong pool").clone();
    assert_eq!(yjc.spec.prompt, "xj_u05_yujia_confirm");
    answer_open(&mut e, vec![ChoiceOption::Confirm]);
    assert!(e.debug_hand(0).contains(&bingpotong), "yujia draws Gujian type:jiguan card");

    // Step to next Play phase so yanshi2 resets -> use xjzongyu (virtual CardKind::Feibiao via use_as, card: None!) -> counts for BOTH yujia and yanshi!
    step_full_turn_to(&mut e, 0);
    e.debug_clear_hand(0);
    let blk = find_non_offdeck_card(&e, CardKind::Sha, |s| s == Suit::Spade).unwrap();
    force_move_card_to_hand(&mut e, &v, 0, blk);
    let zy_def = e.catalog().skill_by_name("xjzongyu").unwrap();
    submit(&mut e, 0, GameAction::ActivateSkill { skill: zy_def });
    answer_open(&mut e, vec![ChoiceOption::Card(blk)]);
    drain_pass_all(&mut e);
    assert_eq!(e.state_core().vm_aux.flags.get(&(0, "xj_yujia_count".to_string())).copied().unwrap_or(0), 2);
    assert_eq!(e.state_core().vm_aux.flags.get(&(0, "xj_yanshi_count".to_string())).copied().unwrap_or(0), 2);
}

#[test]
fn u05_b4_r4_xjzongyu_emits_card_used_feibiao_through_nullify_and_triggers() {
    let (v, mut e) = start_xianjian_only(8495, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_max_hp(p, 5);
        e.debug_set_hp(p, 5);
    }
    e.debug_grant(0, "xjzongyu");
    e.debug_grant(0, "liefeng");
    e.debug_grant(0, "leiyu4");

    // Play 1 Wuzhong first so xjzongyu's virtual Feibiao becomes cards_used_play_phase == 2 -> triggers liefeng!
    let wz = e.debug_place(0, "wuzhong");
    submit(&mut e, 0, GameAction::PlayCard { card: wz, targets: vec![] });
    drain_pass_all(&mut e);

    // Give enemy 1 Nullify, enemy 2 armor=1 and no black cards.
    let wx = find_non_offdeck_card(&e, CardKind::Nullify, |_| true).unwrap();
    force_move_card_to_hand(&mut e, &v, 1, wx);
    {
        let mut snap = e.save();
        snap.core.players[2].hujia = 1;
        e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    }
    let blk = find_non_offdeck_card(&e, CardKind::Sha, |s| s == Suit::Spade).unwrap();
    force_move_card_to_hand(&mut e, &v, 0, blk);

    let zy_def = e.catalog().skill_by_name("xjzongyu").unwrap();
    submit(&mut e, 0, GameAction::ActivateSkill { skill: zy_def });
    answer_open(&mut e, vec![ChoiceOption::Card(blk)]);

    // Nullify window MUST open for enemy 1 holding wuxie!
    let nw = e.open_choice().expect("xjzongyu must open Nullify window").clone();
    assert_eq!(nw.observer, 1);
    assert!(nw.spec.options.contains(&ChoiceOption::Card(wx)), "enemy 1 offered wuxie");

    // Fork A: enemy 1 uses Nullify -> neither enemy loses HP / discards, yet liefeng triggers (2nd CardUsed)!
    let mut e_null = e.clone();
    answer_open(&mut e_null, vec![ChoiceOption::Card(wx)]);
    drain_pass_all(&mut e_null);
    assert_eq!(e_null.debug_hp(1), 5);
    assert_eq!(e_null.debug_hp(2), 5);
    assert!(
        e_null.state_core().skills.contains_key(&(0, e_null.catalog().skill_by_name("yanzhan").unwrap())),
        "nullified CardUsed(Feibiao) still counts as CardUsed #2 for liefeng"
    );

    // Branch B: enemy 1 passes Nullify -> Feibiao resolves:
    // picked enemies lose 1 HP (LoseHp bypasses hujia on seat 2, not Damage!).
    answer_open(&mut e, vec![ChoiceOption::Pass]);
    drain_pass_all(&mut e);
    assert!(
        e.debug_hp(1) == 4 || e.debug_hp(2) == 4,
        "declined Nullify resolves Feibiao LoseHp(1) (bypassing hujia)"
    );
    assert!(
        e.state_core().skills.contains_key(&(0, e.catalog().skill_by_name("yanzhan").unwrap())),
        "CardUsed(Feibiao) triggers liefeng"
    );
}

#[test]
fn u05_b4_r4_dujiang_facedown_unlimited_du_and_duxinshu_private_swap() {
    let (v, mut e) = start_xianjian_only(8496, 2);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_set_max_hp(0, 20);
    e.debug_set_hp(0, 20);
    e.debug_set_max_hp(1, 20);
    e.debug_set_hp(1, 20);
    e.debug_grant(0, "dujiang");
    e.debug_grant(0, "duxinshu");

    // Part A: > 4 simultaneous du cards generated while all previous du remain in seat 1 hand!
    let dj_def = e.catalog().skill_by_name("dujiang").unwrap();
    for round_idx in 0..6 {
        if round_idx > 0 {
            step_full_turn_to(&mut e, 0);
        }
        e.debug_clear_hand(0);
        let blk = find_non_offdeck_card(&e, CardKind::Sha, |s| s == Suit::Spade).unwrap();
        force_move_card_to_hand(&mut e, &v, 0, blk);
        submit(&mut e, 0, GameAction::ActivateSkill { skill: dj_def });
        answer_open(&mut e, vec![ChoiceOption::Card(blk)]);
        drain_pass_all(&mut e);
        let du_count = e
            .debug_hand(1)
            .into_iter()
            .filter(|&c| {
                let name = e.catalog().cards[c as usize].name;
                name == "du" || name == "毒" || name.starts_with("xj_du_")
            })
            .count();
        assert_eq!(
            du_count,
            round_idx + 1,
            "dujiang #{round_idx} must create fresh du beyond 4-card cap"
        );
    }

    // Part B: duxinshu private swap of a `du` card must NOT trigger LoseHp(1)!
    let du_to_steal = *e
        .debug_hand(1)
        .iter()
        .find(|&&c| {
            let n = e.catalog().cards[c as usize].name;
            n == "du" || n == "毒" || n.starts_with("xj_du_")
        })
        .expect("seat 1 holds du");
    e.debug_clear_hand(0);
    let give_card = e.debug_place(0, "wuzhong");
    let p0_hp_before = e.debug_hp(0);
    let p1_hp_before = e.debug_hp(1);
    let dx_def = e.catalog().skill_by_name("duxinshu").unwrap();
    submit(&mut e, 0, GameAction::ActivateSkill { skill: dx_def });
    answer_open(&mut e, vec![ChoiceOption::Player(1)]);
    answer_open(&mut e, vec![ChoiceOption::Card(du_to_steal)]);
    answer_open(&mut e, vec![ChoiceOption::Card(give_card)]);
    drain_pass_all(&mut e);
    assert_eq!(e.debug_hp(0), p0_hp_before, "duxinshu swap must not cause LoseHp on seat 0");
    assert_eq!(e.debug_hp(1), p1_hp_before, "duxinshu swap must not cause LoseHp on seat 1");
    assert!(e.debug_hand(0).contains(&du_to_steal), "seat 0 now holds swapped du");

    // Verify swapped du still triggers LoseHp when later removed from seat 0's hand (via Discard).
    {
        let mut snap = e.save();
        let shan_d = find_non_offdeck_card(&e, CardKind::Shan, |_| true).unwrap();
        let tao_d = find_non_offdeck_card(&e, CardKind::Tao, |_| true).unwrap();
        snap.core.deck.retain(|&c| c != shan_d && c != tao_d);
        snap.core.discard.retain(|&c| c != shan_d && c != tao_d);
        snap.core.players[0].hp = 2;
        snap.core.players[0].hand = vec![du_to_steal, shan_d, tao_d];
        e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    }
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard (hp=2, 3 cards -> must discard 1)
    answer_open(&mut e, vec![ChoiceOption::Card(du_to_steal)]);
    drain_pass_all(&mut e);
    assert_eq!(e.debug_hp(0), 1, "swapped du later discarded from seat 0 must still LoseHp(1)");

    // Part C: removing du while `face_down == true` STILL deals LoseHp(1)!
    step_full_turn_to(&mut e, 0);
    let p1_du = *e
        .debug_hand(1)
        .iter()
        .find(|&&c| {
            let n = e.catalog().cards[c as usize].name;
            n == "du" || n == "毒" || n.starts_with("xj_du_")
        })
        .unwrap();
    {
        let mut snap = e.save();
        // Keep only [p1_du] in seat 1 hand (move rest to off_court to preserve card conservation) so Dismantle guarantees picking p1_du.
        let extra_h: Vec<u32> = snap.core.players[1].hand.iter().copied().filter(|&c| c != p1_du).collect();
        snap.core.players[1].hand.retain(|&c| c == p1_du);
        snap.core.players[1].off_court.extend(extra_h);
        snap.core.players[1].face_down = true;
        e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    }
    e.debug_set_hp(0, 20);
    e.debug_clear_hand(0);
    let dismant = find_non_offdeck_card(&e, CardKind::Dismantle, |_| true).unwrap();
    force_move_card_to_hand(&mut e, &v, 0, dismant);
    let p1_hp_facedown = e.debug_hp(1);
    submit(&mut e, 0, GameAction::PlayCard { card: dismant, targets: vec![1] });
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel = if c.spec.options.contains(&ChoiceOption::Card(p1_du)) {
            vec![ChoiceOption::Card(p1_du)]
        } else if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            vec![c.spec.options[0]]
        };
        let _ = submit(&mut e, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
    }
    drain_pass_all(&mut e);
    assert!(e.state_core().player(1).face_down, "seat 1 stayed face_down during dismantle");
    assert_eq!(e.debug_hp(1), p1_hp_facedown - 1, "du removed while face_down must still LoseHp(1)");
}

#[test]
fn u05_b4_r4_xiaoyue_roundstart_order_facedown_guijiang_tuoqiao_ywuhun() {
    // 1) xiaoyue roundStart fires BEFORE Prepare/Judge/Draw AND fires even when next turn seat is face_down!
    let (v, mut e) = start_xianjian_only(8497, 2);
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_set_hp(0, 5);
    e.debug_set_hp(1, 5);
    e.debug_grant(1, "xiaoyue");
    let sha1 = e.debug_place(1, "sha");
    let mut snap = e.save();
    snap.core.players[0].face_down = true;
    e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    // Seat 1 ends turn -> round bumps from 1 -> 2, seat 0 is face_down: xiaoyue MUST STILL trigger at roundStart!
    step_full_turn_to_next_round(&mut e);
    assert_eq!(e.state_core().player(1).hujia, 1, "xiaoyue triggered at roundStart even with face_down next");
    assert_eq!(e.debug_hp(0), 4, "xiaoyue virtual/hand Sha hit seat 0 before seat 0 flipped/next turn");
    let _ = sha1;

    // Also test non-face_down roundStart ordering: xiaoyue Sha MUST resolve BEFORE seat 0 draws 2 cards in Draw phase!
    let (v_ord, mut e_ord) = start_xianjian_only(84971, 2);
    e_ord.debug_assemble_general(0, "关羽");
    e_ord.debug_assemble_general(1, "张飞");
    advance_to_play(&mut e_ord, 1);
    e_ord.debug_clear_hand(0);
    e_ord.debug_clear_hand(1);
    e_ord.debug_set_hp(0, 5);
    e_ord.debug_set_hp(1, 5);
    e_ord.debug_grant(1, "xiaoyue");
    let p0_shan = find_non_offdeck_card(&e_ord, CardKind::Shan, |s| s == Suit::Heart).unwrap();
    force_move_card_to_hand(&mut e_ord, &v_ord, 0, p0_shan);
    let _sha_ord = e_ord.debug_place(1, "sha");
    // Advance Seat 1 Play -> Discard -> End -> round 2 roundStart!
    let o_a = submit(&mut e_ord, 1, GameAction::NextPhase);
    let o_b = submit(&mut e_ord, 1, GameAction::NextPhase);
    let o_c = submit(&mut e_ord, 1, GameAction::NextPhase);
    // At roundStart, xiaoyue uses Sha BEFORE Seat 0 Draw -> when respond_shan opens on Seat 0, Seat 0 STILL has ONLY [p0_shan] (hasn't drawn 2 in Draw yet)!
    let shan_ask = e_ord
        .open_choice()
        .unwrap_or_else(|| {
            panic!(
                "xiaoyue respond_shan BEFORE seat 0 Draw phase: turn={:?}, hujia={}, hp0={}, hand0={:?}, outcomes=({o_a:?},{o_b:?},{o_c:?})",
                e_ord.turn(),
                e_ord.state_core().player(1).hujia,
                e_ord.debug_hp(0),
                e_ord.debug_hand(0),
            )
        })
        .clone();
    assert_eq!(shan_ask.spec.prompt, "respond_shan");
    assert_eq!(e_ord.debug_hand(0), vec![p0_shan], "xiaoyue resolved before Seat 0 Draw phase");
    drain_pass_all(&mut e_ord);

    // 2) guijiang2/guijiang3 blocks mbcmjiushi_dying_skill and dying_response_skills (e.g. reguhuo)!
    let v_gj = prepare_formal_packages(&[
        "shaai-xianjian-upstream-noname",
        "longxue-upstream-liuli",
        "epic-cards-upstream-liuli",
    ]);
    let mut e_gj = content_native_loader::start_locked_engine(
        8498,
        2,
        content_native_loader::LockedStartMode::Prototype,
        &v_gj,
    )
    .expect("start xianjian+longxue+epic");
    advance_to_play(&mut e_gj, 0);
    e_gj.debug_clear_hand(0);
    e_gj.debug_clear_hand(1);
    e_gj.debug_set_hp(1, 1);
    e_gj.debug_grant(0, "guijiang");
    e_gj.debug_grant(1, "mbcmjiushi");
    e_gj.debug_grant(1, "reguhuo");
    let blk = find_non_offdeck_card(&e_gj, CardKind::Sha, |s| s == Suit::Spade).unwrap();
    force_move_card_to_hand(&mut e_gj, &v_gj, 0, blk);
    let gj_def = e_gj.catalog().skill_by_name("guijiang").unwrap();
    submit(&mut e_gj, 0, GameAction::ActivateSkill { skill: gj_def });
    answer_open(&mut e_gj, vec![ChoiceOption::Card(blk)]);
    answer_open(&mut e_gj, vec![ChoiceOption::Player(1)]);
    let _ = e_gj.debug_place(1, "shan");
    deal_damage(&mut e_gj, Some(0), 1, 1, DamageNature::Normal);
    assert!(e_gj.open_choice().is_none(), "guijiang2/3 must block mbcmjiushi and reguhuo dying saves without asking");
    assert!(!e_gj.debug_alive(1), "guijiang2/3 must prevent mbcmjiushi/reguhuo dying save");

    // 3) restore_ywuhun preserves off_court & vm_aux.piles + xj_ywuhun_abort aborts IMMEDIATELY on damageEnd (before chain conduction to 2nd target)!
    let (v2, mut e_yw) = start_xianjian_only(8499, 3);
    e_yw.debug_assemble_general(0, "张飞");
    e_yw.debug_assemble_general(1, "关羽");
    e_yw.debug_assemble_general(2, "曹操");
    advance_to_play(&mut e_yw, 0);
    for p in 0..3 {
        e_yw.debug_clear_hand(p);
        e_yw.debug_set_max_hp(p, 5);
        e_yw.debug_set_hp(p, 5);
    }
    e_yw.debug_grant(0, "ywuhun");
    // Advance a full cycle so Seat 0 TurnBegin triggers ywuhun snapshot with off_court & pile cards!
    let pre_oc = find_non_offdeck_card(&e_yw, CardKind::Shan, |_| true).unwrap();
    let pre_pile = find_non_offdeck_card(&e_yw, CardKind::Tao, |_| true).unwrap();
    let extra_oc = find_non_offdeck_card(&e_yw, CardKind::Jiu, |_| true).unwrap();
    let fire_sha = find_non_offdeck_card(&e_yw, CardKind::FireSha, |_| true).unwrap();
    {
        let mut snap = e_yw.save();
        for cid in [pre_oc, pre_pile, extra_oc, fire_sha] {
            snap.core.deck.retain(|&c| c != cid);
            snap.core.discard.retain(|&c| c != cid);
        }
        snap.core.players[1].off_court.push(pre_oc);
        snap.core.vm_aux.piles.insert((0, "xj_probe_pile".to_string()), vec![pre_pile]);
        snap.core.players[0].hand.push(extra_oc);
        snap.core.players[0].hand.push(fire_sha);
        e_yw = content_native_loader::restore_locked_engine(&snap, &v2).unwrap();
    }
    step_full_turn_to(&mut e_yw, 0); // runs Seat 0 TurnBegin -> ywuhun snapshot!
    assert_eq!((e_yw.turn().current, e_yw.turn().phase), (0, Phase::Play));
    let (init_cnt, init_set) = total_card_set(&e_yw);
    assert_eq!(e_yw.state_core().vm_aux.flags.get(&(0, "ywuhun_active".to_string())).copied().unwrap_or(0), 1);

    let order = e_yw.state_core().seat_order_from(0);
    let (t1, t2) = (order[1], order[2]);
    // Mutate off_court mid-turn, link t1 & t2, and play FireSha against t1:
    // Hitting t1 sets xj_ywuhun_abort on damageEnd -> MUST abort IMMEDIATELY before chain damage conducts to t2,
    // ending Seat 0 turn cleanly and restoring pre_oc & pre_pile!
    {
        let mut snap = e_yw.save();
        for p in [t1, t2] {
            let moved_back = std::mem::take(&mut snap.core.players[p as usize].hand);
            snap.core.discard.extend(moved_back);
            snap.core.players[p as usize].linked = true;
        }
        if let Some(pos) = snap.core.players[0].hand.iter().position(|&c| c == extra_oc) {
            snap.core.players[0].hand.remove(pos);
            snap.core.players[2].off_court.push(extra_oc);
        }
        e_yw = content_native_loader::restore_locked_engine(&snap, &v2).unwrap();
    }
    let sha_out = submit(&mut e_yw, 0, GameAction::PlayCard { card: fire_sha, targets: vec![t1] });
    assert!(!matches!(sha_out, CommandOutcome::Rejected(_)), "fire_sha PlayCard accepted: {sha_out:?}");
    drain_pass_all(&mut e_yw);
    assert_eq!(e_yw.debug_hp(t1), 4, "primary target took 1 fire damage");
    assert_eq!(e_yw.debug_hp(t2), 5, "immediate damageEnd abort stopped chain conduction to linked 2nd target");
    assert!(e_yw.state_core().player(t2).linked, "linked 2nd target remained linked because chain step aborted");
    assert_ne!(e_yw.turn().current, 0, "ywuhun abort immediately wound down Seat 0 turn");
    assert_eq!(e_yw.state_core().player(1).off_court, vec![pre_oc], "restore_ywuhun preserved off_court");
    assert!(e_yw.state_core().player(2).off_court.is_empty(), "restore_ywuhun cleared mid-turn off_court");
    assert_eq!(
        e_yw.state_core().vm_aux.piles.get(&(0, "xj_probe_pile".to_string())),
        Some(&vec![pre_pile]),
        "restore_ywuhun preserved vm_aux.piles"
    );
    assert_eq!(total_card_set(&e_yw), (init_cnt, init_set), "strict conservation across ywuhun abort");

    // 4) tuoqiao uses actual carrier card `c` (not hardcoded `0`) in `eval_card_target_canceled`:
    // mark target seat 1 with `u05_mod:molicha = 1` (black-card ward) -> tuoqiao with black card `c` is suppressed against seat 1!
    let (v_tq, mut e_tq) = start_xianjian_only(84991, 2);
    advance_to_play(&mut e_tq, 0);
    e_tq.debug_clear_hand(0);
    e_tq.debug_grant(0, "tuoqiao");
    let blk_tq = find_non_offdeck_card(&e_tq, CardKind::Sha, |s| s == Suit::Club).unwrap();
    force_move_card_to_hand(&mut e_tq, &v_tq, 0, blk_tq);
    {
        let mut snap = e_tq.save();
        snap.core.vm_aux.flags.insert((1, "u05_mod:molicha".to_string()), 1);
        e_tq = content_native_loader::restore_locked_engine(&snap, &v_tq).unwrap();
    }
    step_full_turn_to(&mut e_tq, 1);
    assert!(
        e_tq.open_choice().is_none_or(|c| c.spec.prompt != "xj_u05_tuoqiao_card"),
        "tuoqiao must respect eval_card_target_canceled on actual black card c"
    );
}

fn step_full_turn_to_next_round(e: &mut Engine) {
    let start_round = e.turn().round;
    for _ in 0..40 {
        if let Some(c) = e.open_choice().cloned() {
            let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else if c.spec.allow_cancel {
                Vec::new()
            } else {
                c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
            };
            let _ = submit(e, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
            continue;
        }
        if e.turn().round > start_round {
            return;
        }
        let cur = e.turn().current;
        let _ = submit(e, cur, GameAction::NextPhase);
    }
}
