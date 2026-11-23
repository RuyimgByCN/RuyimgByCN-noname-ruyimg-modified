use super::common::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::event::{DamageNature, Phase};
use engine_native::{ChoiceOption, CommandOutcome, GameAction, RejectReason};

#[test]
fn u05_b4_xiaoyue_danqing_zhangmu_ai_leiyu_chengxin2() {
    // 1. xiaoyue: RoundStart uses random hand Sha on enemy +1 hujia; non-Heart Shan unrespondable
    let (vx, mut ex) = start_xianjian_only(8440, 2);
    advance_to_play(&mut ex, 1);
    ex.debug_clear_hand(0);
    ex.debug_clear_hand(1);
    ex.debug_set_hp(1, 5);
    ex.debug_grant(0, "xiaoyue");
    let sha_cid = (0..ex.catalog().cards.len() as u32).find(|&c| ex.catalog().card_kind(c) == CardKind::Sha).unwrap();
    let dm_shan = place_kind_suit(&mut ex, 1, CardKind::Shan, |s| s == Suit::Diamond).unwrap();
    let ht_shan = place_kind_suit(&mut ex, 1, CardKind::Shan, |s| s == Suit::Heart).unwrap();
    force_move_card_to_hand(&mut ex, &vx, 0, sha_cid);
    force_move_card_to_hand(&mut ex, &vx, 1, dm_shan);
    force_move_card_to_hand(&mut ex, &vx, 1, ht_shan);
    // Advancing player 1 turn -> TurnPhaseAfter -> Round 2 RoundStart triggers xiaoyue!
    let _ = submit(&mut ex, 1, GameAction::NextPhase); // Play -> Discard
    let _ = submit(&mut ex, 1, GameAction::NextPhase); // Discard -> End
    let _ = submit(&mut ex, 1, GameAction::NextPhase); // End -> TurnEndWindow -> RoundStart!
    assert_eq!(ex.state_core().player(0).hujia, 1, "xiaoyue +1 armor");
    let c = ex.open_choice().expect("target asked for Heart Shan").clone();
    assert!(c.spec.options.contains(&ChoiceOption::Card(ht_shan)), "Heart Shan allowed");
    assert!(!c.spec.options.contains(&ChoiceOption::Card(dm_shan)), "Diamond Shan rejected by xiaoyue2");
    answer_open(&mut ex, vec![ChoiceOption::Card(ht_shan)]);

    // 2. danqing + danqing_count: real assembly of pal_xiahoujinxuan (no debug_grant("danqing_count"))
    let (vd, mut ed) = start_xianjian_only(8441, 2);
    advance_to_play(&mut ed, 0);
    ed.debug_clear_hand(0);
    assemble_general_by_name(&mut ed, 0, "pal_xiahoujinxuan");
    let c1 = place_kind_suit(&mut ed, 0, CardKind::Wuzhong, |s| s == Suit::Heart).unwrap();
    let c2 = place_kind_suit(&mut ed, 0, CardKind::Dismantle, |s| s == Suit::Club).unwrap();
    let c3 = place_kind_suit(&mut ed, 0, CardKind::Dismantle, |s| s == Suit::Spade).unwrap();
    let c4 = place_kind_suit(&mut ed, 0, CardKind::Snatch, |s| s == Suit::Diamond).unwrap();
    for &c in &[c1, c2, c3, c4] {
        force_move_card_to_hand(&mut ed, &vd, 0, c);
        let t = if matches!(ed.catalog().card_kind(c), CardKind::Dismantle | CardKind::Snatch) {
            if ed.debug_hand(1).is_empty() {
                let _ = ed.debug_place(1, "tao");
            }
            vec![1]
        } else {
            vec![]
        };
        let _ = submit(&mut ed, 0, GameAction::PlayCard { card: c, targets: t });
        drain_pass_all(&mut ed);
    }
    ed.debug_clear_hand(0);
    let _ = submit(&mut ed, 0, GameAction::NextPhase); // Play -> Discard
    let _ = submit(&mut ed, 0, GameAction::NextPhase); // Discard -> End
    let _ = submit(&mut ed, 0, GameAction::NextPhase); // End -> danqing!
    let dc = ed.open_choice().expect("danqing target ask").clone();
    assert_eq!(dc.spec.prompt, "xj_u05_danqing_targets");
    assert_eq!(dc.spec.max, 2);
    let _ = answer_open(&mut ed, vec![ChoiceOption::Player(0)]);

    // 3. zhangmu_ai + chengxin2
    let (_, mut ez) = start_xianjian_only(8442, 2);
    advance_to_play(&mut ez, 0);
    ez.debug_clear_hand(0);
    ez.debug_grant(0, "zhangmu_ai");
    ez.debug_grant(0, "chengxin2");
    ez.debug_set_hp(0, 4);
    deal_damage(&mut ez, Some(1), 0, 2, DamageNature::Normal);
    assert_eq!(ez.debug_hp(0), 4, "chengxin2 prevented all damage");
    let shan = ez.debug_place(0, "shan");
    ez.debug_set_hp(0, 0);
    ez.debug_set_hp(0, 1);
    let _ = shan;
    // Trigger Discard of Shan to remove zhangmu_ai cleanly
    let tao_k = ez.debug_place(0, "tao");
    let _ = submit(&mut ez, 0, GameAction::NextPhase); // Play -> Discard (hp=1 -> must discard 1)
    answer_open(&mut ez, vec![ChoiceOption::Card(shan)]);
    assert!(!ez.state_core().skills.contains_key(&(0, ez.catalog().skill_by_name("zhangmu_ai").unwrap())), "zhangmu_ai removed when 0 Shan");
    let _ = tao_k;

    // 4. leiyu + leiyu2 + leiyu3 + leiyu4
    let (vl, mut el) = start_xianjian_only(8443, 3);
    advance_to_play(&mut el, 0);
    assemble_general_by_name(&mut el, 0, "pal_xingxuan");
    for p in 0..3 {
        el.debug_clear_hand(p);
        el.debug_set_max_hp(p, 6);
        el.debug_set_hp(p, 6);
    }
    el.debug_grant(0, "leiyu3");
    let sha1 = el.debug_place(0, "sha");
    let blk = place_kind_suit(&mut el, 0, CardKind::Sha, |s| matches!(s, Suit::Spade | Suit::Club)).unwrap();
    force_move_card_to_hand(&mut el, &vl, 0, blk);
    // Only target player 1 during Play; leave player 2 untargeted
    submit(&mut el, 0, GameAction::PlayCard { card: sha1, targets: vec![1] });
    drain_pass_all(&mut el);
    el.debug_set_hp(0, 6); // avoid discard
    let _ = submit(&mut el, 0, GameAction::NextPhase); // Play -> Discard
    let _ = submit(&mut el, 0, GameAction::NextPhase); // Discard -> End
    let _ = submit(&mut el, 0, GameAction::NextPhase); // End -> leiyu!
    let lc = el.open_choice().expect("leiyu black card ask").clone();
    assert_eq!(lc.spec.prompt, "xj_u05_leiyu_card");
    answer_open(&mut el, vec![ChoiceOption::Card(blk)]);
    assert_eq!(el.debug_hp(1), 4, "player 1 hit by sha (1) + leiyu Jingleishan (1)");
    assert_eq!(el.debug_hp(2), 6, "player 2 NOT targeted during Play, untouched by leiyu");
}

#[test]
fn u05_b4_batch_d_all_9_active_skills_and_subskills() {
    // 1. lingdi: Play distance=1 suitA -> both draw 1 -> distance=2 suitB -> cancel safe
    let (vd, mut e) = start_xianjian_only(8450, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 5);
    }
    e.debug_grant(0, "lingdi");
    let c_heart = place_kind_suit(&mut e, 0, CardKind::Tao, |s| s == Suit::Heart).unwrap();
    let c_club = place_kind_suit(&mut e, 0, CardKind::Sha, |s| s == Suit::Club).unwrap();
    force_move_card_to_hand(&mut e, &vd, 0, c_heart);
    force_move_card_to_hand(&mut e, &vd, 0, c_club);
    let ld_def = e.catalog().skill_by_name("lingdi").unwrap();
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: ld_def });
    assert!(matches!(out, CommandOutcome::Pending { .. }));
    answer_open(&mut e, vec![ChoiceOption::Card(c_heart)]);
    let tc = e.open_choice().expect("lingdi distance=1 targets").clone();
    assert_eq!(tc.spec.prompt, "xj_u05_lingdi_target");
    let mut e2 = roundtrip_choice(e, &vd);
    answer_open(&mut e2, vec![ChoiceOption::Player(1)]);
    assert!(e2.debug_hand(1).len() == 1 && e2.debug_hand(0).len() == 2, "both drew 1 (0 lost c_heart, +1)");

    // 2. xjzongyu + dujiang (with poison LoseHp conservation)
    let (vz, mut ez) = start_xianjian_only(8451, 3);
    advance_to_play(&mut ez, 0);
    let (init_cnt, init_set) = total_card_set(&ez);
    for p in 0..3 {
        ez.debug_clear_hand(p);
        ez.debug_set_hp(p, 5);
    }
    ez.debug_grant(0, "xjzongyu");
    ez.debug_grant(0, "dujiang");
    let blk1 = place_kind_suit(&mut ez, 0, CardKind::Sha, |s| matches!(s, Suit::Spade | Suit::Club)).unwrap();
    let blk2 = place_kind_suit(&mut ez, 0, CardKind::SavageAttack, |s| matches!(s, Suit::Spade | Suit::Club)).unwrap();
    let dism_poison = place_kind_suit(&mut ez, 0, CardKind::Dismantle, |s| s == Suit::Heart).unwrap();
    force_move_card_to_hand(&mut ez, &vz, 0, blk1);
    force_move_card_to_hand(&mut ez, &vz, 0, blk2);
    let zy_def = ez.catalog().skill_by_name("xjzongyu").unwrap();
    // Cancel check first!
    submit(&mut ez, 0, GameAction::ActivateSkill { skill: zy_def });
    answer_open(&mut ez, vec![]);
    assert!(matches!(submit(&mut ez, 0, GameAction::ActivateSkill { skill: zy_def }), CommandOutcome::Pending { .. }), "cancel refunded usable");
    answer_open(&mut ez, vec![ChoiceOption::Card(blk1)]);
    drain_pass_all(&mut ez);
    assert!(ez.debug_hp(1) < 5 || ez.debug_hp(2) < 5, "xjzongyu Feibiao dealt LoseHp");

    let dj_def = ez.catalog().skill_by_name("dujiang").unwrap();
    submit(&mut ez, 0, GameAction::ActivateSkill { skill: dj_def });
    answer_open(&mut ez, vec![ChoiceOption::Card(blk2)]);
    let poisoned_seat = [1u8, 2u8].into_iter().find(|&p| !ez.debug_hand(p).is_empty()).expect("enemy got poison");
    let poison_cid = ez.debug_hand(poisoned_seat)[0];
    assert_eq!(ez.catalog().cards[poison_cid as usize].name, "du", "dujiang creates true du poison card");
    step_full_turn_to(&mut ez, poisoned_seat);
    let play_du = submit(&mut ez, poisoned_seat, GameAction::PlayCard { card: poison_cid, targets: vec![] });
    assert!(matches!(play_du, CommandOutcome::Rejected(RejectReason::CardBlocked)), "du poison cannot be played directly: {play_du:?}");
    step_full_turn_to(&mut ez, 0);
    let hp_before_poison = ez.debug_hp(poisoned_seat);
    force_move_card_to_hand(&mut ez, &vz, 0, dism_poison);
    submit(&mut ez, 0, GameAction::PlayCard { card: dism_poison, targets: vec![poisoned_seat] });
    drain_pass_all(&mut ez);
    assert!(!ez.debug_hand(poisoned_seat).contains(&poison_cid), "poison card discarded from hand");
    assert_eq!(ez.debug_hp(poisoned_seat), hp_before_poison - 1, "poison card leaving hand causes LoseHp(1)");
    let (end_cnt, end_set) = total_card_set(&ez);
    assert_eq!((end_cnt, end_set), (init_cnt, init_set), "strict conservation across xjzongyu+dujiang");

    // 3. yanzhan + yanzhan2 + yanzhan3 + yanzhan4 (via real pal_jiangcheng assembly, no debug_grant("yanzhan2"))
    let (vy, mut ey) = start_xianjian_only(8452, 2);
    advance_to_play(&mut ey, 0);
    assemble_general_by_name(&mut ey, 0, "pal_jiangcheng");
    ey.debug_clear_hand(0);
    ey.debug_clear_hand(1);
    ey.debug_set_hp(0, 5);
    ey.debug_set_hp(1, 5);
    let red_c = place_kind_suit(&mut ey, 0, CardKind::Wuzhong, |s| s == Suit::Heart).unwrap();
    let dm_shan = place_kind_suit(&mut ey, 1, CardKind::Shan, |s| s == Suit::Diamond).unwrap();
    let ht_shan = place_kind_suit(&mut ey, 1, CardKind::Shan, |s| s == Suit::Heart).unwrap();
    force_move_card_to_hand(&mut ey, &vy, 0, red_c);
    force_move_card_to_hand(&mut ey, &vy, 1, dm_shan);
    force_move_card_to_hand(&mut ey, &vy, 1, ht_shan);
    let yz_def = ey.catalog().skill_by_name("yanzhan").unwrap();
    submit(&mut ey, 0, GameAction::ActivateSkill { skill: yz_def });
    answer_open(&mut ey, vec![ChoiceOption::Card(red_c)]);
    answer_open(&mut ey, vec![ChoiceOption::Player(1)]);
    let shan_ask = ey.open_choice().expect("yanzhan3 Shan suit restriction").clone();
    assert!(shan_ask.spec.options.contains(&ChoiceOption::Card(ht_shan)), "Heart matches FireSha suit");
    assert!(!shan_ask.spec.options.contains(&ChoiceOption::Card(dm_shan)), "Diamond Shan filtered by yanzhan3");
    answer_open(&mut ey, vec![ChoiceOption::Pass]);
    assert_eq!(ey.debug_hp(1), 4, "FireSha hit");
    assert!(ey.state_core().skills.contains_key(&(0, ey.catalog().skill_by_name("yanzhan4").unwrap())), "yanzhan2 granted yanzhan4");
    // yanzhan4 grants +1 Sha usable in same Play phase!
    let extra_sha = ey.debug_place(0, "sha");
    let out2 = submit(&mut ey, 0, GameAction::PlayCard { card: extra_sha, targets: vec![1] });
    assert!(!matches!(out2, CommandOutcome::Rejected(_)), "yanzhan4 allowed extra Sha: {out2:?}");
    drain_pass_all(&mut ey);

    // 4. feixia + huxi + guijiang + guijiang2/3 + duxinshu + tuoqiao
    let (vm, mut em) = start_xianjian_only(8453, 3);
    advance_to_play(&mut em, 0);
    for p in 0..3 {
        em.debug_clear_hand(p);
        em.debug_set_hp(p, 5);
    }
    em.debug_grant(0, "feixia");
    em.debug_grant(0, "huxi");
    em.debug_grant(0, "guijiang");
    em.debug_grant(0, "duxinshu");
    em.debug_grant(0, "tuoqiao");
    let r_fx = place_kind_suit(&mut em, 0, CardKind::Tao, |s| s == Suit::Heart).unwrap();
    let b_gj = place_kind_suit(&mut em, 0, CardKind::Sha, |s| s == Suit::Spade).unwrap();
    let b_tq = place_kind_suit(&mut em, 0, CardKind::Sha, |s| s == Suit::Club).unwrap();
    let p1_card = em.debug_place(1, "tao");
    force_move_card_to_hand(&mut em, &vm, 0, r_fx);
    force_move_card_to_hand(&mut em, &vm, 0, b_gj);
    force_move_card_to_hand(&mut em, &vm, 0, b_tq);

    // feixia
    let fx_def = em.catalog().skill_by_name("feixia").unwrap();
    submit(&mut em, 0, GameAction::ActivateSkill { skill: fx_def });
    answer_open(&mut em, vec![ChoiceOption::Card(r_fx)]);
    drain_pass_all(&mut em);
    assert!(em.debug_hp(1) < 5 || em.debug_hp(2) < 5, "feixia virtual Sha hit an enemy");

    // huxi: target -> LoseHp(1) +1 hujia + virtual Sha
    let hx_def = em.catalog().skill_by_name("huxi").unwrap();
    submit(&mut em, 0, GameAction::ActivateSkill { skill: hx_def });
    answer_open(&mut em, vec![ChoiceOption::Player(1)]);
    drain_pass_all(&mut em);
    assert_eq!(em.debug_hp(0), 4, "huxi lost 1 HP");
    assert_eq!(em.state_core().player(0).hujia, 1, "huxi +1 armor");

    // guijiang: blocks Tao/recover on player 2 until player 0 next TurnBegin
    let gj_def = em.catalog().skill_by_name("guijiang").unwrap();
    submit(&mut em, 0, GameAction::ActivateSkill { skill: gj_def });
    answer_open(&mut em, vec![ChoiceOption::Card(b_gj)]);
    answer_open(&mut em, vec![ChoiceOption::Player(2)]);
    assert!(em.state_core().skills.contains_key(&(2, em.catalog().skill_by_name("guijiang2").unwrap())));
    assert!(em.catalog().skill_by_name("guijiang3").is_some() && em.catalog().skill_by_name("guijin2").is_some());
    em.debug_set_hp(2, 2);
    let p2_tao = em.debug_place(2, "tao");
    let rej_tao = submit(&mut em, 2, GameAction::PlayCard { card: p2_tao, targets: vec![] });
    assert!(matches!(rej_tao, CommandOutcome::Rejected(_)), "guijiang2 blocks Peach play");

    // duxinshu (mid-choice save/restore)
    let p0_give = em.debug_place(0, "wuzhong");
    let dx_def = em.catalog().skill_by_name("duxinshu").unwrap();
    submit(&mut em, 0, GameAction::ActivateSkill { skill: dx_def });
    answer_open(&mut em, vec![ChoiceOption::Player(1)]);
    let steal_c = em.open_choice().expect("duxinshu steal card").clone();
    assert_eq!(steal_c.spec.prompt, "xj_u05_duxinshu_steal");
    let mut em_r = roundtrip_choice(em, &vm);
    answer_open(&mut em_r, vec![ChoiceOption::Card(p1_card)]);
    answer_open(&mut em_r, vec![ChoiceOption::Card(p0_give)]);
    assert!(em_r.debug_hand(0).contains(&p1_card));
    assert!(em_r.debug_hand(1).contains(&p0_give));

    // tuoqiao: triggers out of turn at another player's Phase::Play begin (never phaseUse active, never self-target)
    let tq_def = em_r.catalog().skill_by_name("tuoqiao").unwrap();
    assert!(matches!(submit(&mut em_r, 0, GameAction::ActivateSkill { skill: tq_def }), CommandOutcome::Rejected(_)), "tuoqiao is phaseUseBegin trigger, not phaseUse active");
    em_r.debug_set_hp(0, 5);
    let p1_shan_tq = em_r.debug_place(1, "shan");
    let _ = p1_shan_tq;
    while em_r.open_choice().is_none_or(|c| c.spec.prompt != "xj_u05_tuoqiao_card") {
        if let Some(c) = em_r.open_choice().cloned() {
            let sel = if c.spec.options.contains(&ChoiceOption::Pass) { vec![ChoiceOption::Pass] } else { Vec::new() };
            let _ = submit(&mut em_r, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
        } else {
            let cur = em_r.turn().current;
            let _ = submit(&mut em_r, cur, GameAction::NextPhase);
        }
    }
    let tq_ask = em_r.open_choice().expect("tuoqiao phaseUseBegin ask").clone();
    assert_eq!(tq_ask.observer, 0);
    assert_eq!(tq_ask.spec.prompt, "xj_u05_tuoqiao_card");
    assert!(tq_ask.spec.allow_cancel);
    assert_eq!(em_r.turn().current, 1);
    assert_eq!(em_r.turn().phase, Phase::Play);
    answer_open(&mut em_r, vec![ChoiceOption::Card(b_tq)]);
    // Player 1 gets respond_shan -> refuses -> gj_food_shihuifen_ban applied unconditionally!
    let shan_resp = em_r.open_choice().expect("shihuifen respond_shan").clone();
    assert_eq!(shan_resp.observer, 1);
    assert_eq!(shan_resp.spec.prompt, "respond_shan");
    answer_open(&mut em_r, vec![ChoiceOption::Pass]);
    drain_pass_all(&mut em_r);
    let shf_def = em_r.catalog().skill_by_name(engine_native::content::host::GJ_FOOD_SHIHUIFEN_BAN_KEY).expect("shihuifen ban in xianjian catalog");
    assert!(em_r.state_core().skills.contains_key(&(1, shf_def)), "tuoqiao inflicted shihuifen ban");
    let p1_sha = em_r.debug_place(1, "sha");
    let p1_wz = em_r.debug_place(1, "wuzhong");
    assert!(matches!(submit(&mut em_r, 1, GameAction::PlayCard { card: p1_sha, targets: vec![0] }), CommandOutcome::Rejected(RejectReason::IllegalTargets)), "shihuifen_ban blocks targeting others");
    assert!(!matches!(submit(&mut em_r, 1, GameAction::PlayCard { card: p1_wz, targets: vec![] }), CommandOutcome::Rejected(_)), "shihuifen_ban still allows self tricks");
    drain_pass_all(&mut em_r);

    // guijiang2/guijiang3 dying Peach-save refusal probe + TurnBegin expiry
    let p2_dying_tao = em_r.debug_place(2, "tao");
    let p1_save_tao = em_r.debug_place(1, "tao");
    let _ = (p2_dying_tao, p1_save_tao);
    em_r.debug_set_hp(2, 1);
    deal_damage(&mut em_r, Some(1), 2, 1, DamageNature::Normal);
    drain_pass_all(&mut em_r);
    assert!(!em_r.debug_alive(2), "guijiang2/guijiang3 blocks Dying Peach rescue");

    // Fresh guijiang expiry check at owner's next TurnBegin
    let (vgj, mut egj) = start_xianjian_only(8459, 2);
    advance_to_play(&mut egj, 0);
    for p in 0..2 {
        egj.debug_clear_hand(p);
        egj.debug_set_hp(p, 4);
    }
    egj.debug_grant(0, "guijiang");
    let gj_b = place_kind_suit(&mut egj, 0, CardKind::Sha, |s| s == Suit::Spade).unwrap();
    force_move_card_to_hand(&mut egj, &vgj, 0, gj_b);
    let gj_d = egj.catalog().skill_by_name("guijiang").unwrap();
    submit(&mut egj, 0, GameAction::ActivateSkill { skill: gj_d });
    answer_open(&mut egj, vec![ChoiceOption::Card(gj_b)]);
    answer_open(&mut egj, vec![ChoiceOption::Player(1)]);
    assert!(egj.state_core().skills.contains_key(&(1, egj.catalog().skill_by_name("guijiang2").unwrap())));
    step_full_turn_to(&mut egj, 0);
    assert!(!egj.state_core().skills.contains_key(&(1, egj.catalog().skill_by_name("guijiang2").unwrap())), "guijiang2 cleared at owner TurnBegin");
}
