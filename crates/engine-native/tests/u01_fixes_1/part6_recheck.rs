use super::common::*;

fn find_tengjia(e: &Engine) -> u32 {
    e.catalog()
        .cards
        .iter()
        .enumerate()
        .skip(1)
        .rfind(|(_, def)| def.name == "tengjia")
        .map(|(id, _)| id as u32)
        .expect("tengjia in catalog")
}

fn find_huogong(e: &Engine) -> u32 {
    e.catalog()
        .cards
        .iter()
        .enumerate()
        .skip(1)
        .rfind(|(_, def)| def.name == "huogong")
        .map(|(id, _)| id as u32)
        .expect("huogong in catalog")
}

#[test]
fn recheck_19_dddjijian() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:1676-1806
    // Verify: FireSha+Sha canonical dedup; Huogong (Linghunzhihuo + 1 Fire dmg + xinhongyan effective_card_suit);
    // Jiedao (holder -> victim); BOSS_REMEN blocks unarmored FireSha target; Flow::Yield after give before 1055.
    let (mut e, d3, p9, aux) = new_recheck_engine(92019, 4);
    let bao = find_general(&e, "ddd_baosanniang");
    let male_g = find_general(&e, "ddd_liuchong");
    e.assign_general(0, bao).unwrap();
    e.assign_general(1, male_g).unwrap();
    retain_only_skill(&mut e, 0, Some("dddjijian"));
    retain_only_skill(&mut e, 1, None);
    let jj = e.catalog().skill_by_name("dddjijian").unwrap();

    let sha_give = find_card_by(&e, |_, d| d.kind == CardKind::Sha && d.suit == Suit::Heart);
    let sha_hand = find_card_by(&e, |id, d| id != sha_give && d.kind == CardKind::Sha && d.suit == Suit::Spade);
    let huosha = find_card_by(&e, |_, d| d.kind == CardKind::FireSha);
    let huosha_owner = find_card_by(&e, |id, d| id != huosha && d.kind == CardKind::FireSha);
    let sha_owner2 = find_card_by(&e, |id, d| id != sha_give && id != sha_hand && d.kind == CardKind::Sha);
    let sha_owner3 = find_card_by(&e, |id, d| id != sha_give && id != sha_hand && id != sha_owner2 && d.kind == CardKind::Sha);
    let huogong = find_huogong(&e);
    let jiedao = find_card_by(&e, |_, d| d.kind == CardKind::Jiedao);
    let spade1 = find_card_by(&e, |id, d| {
        id != sha_give && id != sha_hand && id != sha_owner2 && id != sha_owner3 && id != huosha && id != huosha_owner && d.suit == Suit::Spade
    });
    let heart_disc = find_card_by(&e, |id, d| {
        id != sha_give && id != sha_hand && id != sha_owner2 && id != sha_owner3 && id != huosha && id != huosha_owner && id != spade1 && d.suit == Suit::Heart
    });
    let weapon2 = find_card_by(&e, |_, d| d.kind == CardKind::Zhuge);

    put_in_hand(&mut e, 0, sha_give);
    put_in_hand(&mut e, 0, sha_owner2);
    put_in_hand(&mut e, 0, sha_owner3);
    put_in_hand(&mut e, 0, huosha_owner);
    put_in_hand(&mut e, 1, sha_hand); // Spade (via xinhongyan => Red!)
    put_in_hand(&mut e, 1, huosha);
    e.debug_grant(1, "xinhongyan"); // turns seat 1's Spade show card into Heart!
    put_in_equip(&mut e, 2, weapon2);
    e.debug_grant(2, "热闷"); // BOSS_REMEN with no armor

    // 1) FireSha + Sha shown together share canonical_jijian_key ("sha") dedup:
    // using Sha consumes BOTH Sha and FireSha so neither remains usable for the 2nd pick!
    e.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: jj });
    answer(&mut e, 0, vec![ChoiceOption::Card(sha_give)]);
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    let show_c = e.open_choice().expect("dddjijian_show").clone();
    assert_eq!((show_c.observer, show_c.spec.prompt), (1, "dddjijian_show"));
    answer(&mut e, 1, vec![ChoiceOption::Card(sha_hand), ChoiceOption::Card(huosha)]);
    let u0 = e.open_choice().expect("dddjijian_use_card seat 0 canonical dedup").clone();
    assert!(u0.spec.options.contains(&ChoiceOption::Kind(CardKind::Sha as u16)));
    assert!(u0.spec.options.contains(&ChoiceOption::Kind(CardKind::FireSha as u16)));
    answer(&mut e, 0, vec![ChoiceOption::Kind(CardKind::Sha as u16)]);
    answer(&mut e, 0, vec![ChoiceOption::Player(3)]);
    drain(&mut e);
    // Once seat 0 used Sha, canonical_jijian_key(Sha) consumes FireSha too -> seat 1 cannot use either Sha or FireSha!
    assert!(
        e.open_choice().as_ref().is_none_or(|c| {
            !c.spec.options.contains(&ChoiceOption::Kind(CardKind::FireSha as u16))
        }),
        "using Sha must also consume FireSha via canonical_jijian_key"
    );

    // 2) Huogong (Linghunzhihuo + xinhongyan Spade->Heart + 1 Fire dmg):
    let (mut eh, d3h, p9h, auxh) = new_recheck_engine(92119, 4);
    eh.assign_general(0, bao).unwrap();
    eh.assign_general(1, male_g).unwrap();
    retain_only_skill(&mut eh, 0, Some("dddjijian"));
    retain_only_skill(&mut eh, 1, None);
    eh.debug_grant(1, "xinhongyan");
    put_in_hand(&mut eh, 0, sha_owner2);
    put_in_hand(&mut eh, 0, heart_disc);
    put_in_hand(&mut eh, 1, huogong);
    put_in_hand(&mut eh, 1, spade1);
    eh.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(&mut eh, 0, GameAction::ActivateSkill { skill: jj });
    answer(&mut eh, 0, vec![ChoiceOption::Card(sha_owner2)]);
    answer(&mut eh, 0, vec![ChoiceOption::Player(1)]);
    let _show_hg = eh.open_choice().expect("dddjijian_show huogong").clone();
    answer(&mut eh, 1, vec![ChoiceOption::Card(huogong)]);
    // Seat 0 uses virtual huogong (`Linghunzhihuo`) on seat 1 (xinhongyan Spade -> Heart -> seat 0 discards Heart -> 1 Fire dmg)!
    let _u0_hg = eh.open_choice().expect("dddjijian_use_card huogong").clone();
    answer(&mut eh, 0, vec![ChoiceOption::Kind(CardKind::Linghunzhihuo as u16)]);
    answer(&mut eh, 0, vec![ChoiceOption::Player(1)]);
    let hg_show = eh.open_choice().expect("dddjijian_huogong_show").clone();
    assert_eq!((hg_show.observer, hg_show.spec.prompt), (1, "dddjijian_huogong_show"));
    answer(&mut eh, 1, vec![ChoiceOption::Card(spade1)]);
    let hg_disc = eh.open_choice().expect("dddjijian_huogong_discard").clone();
    assert_eq!((hg_disc.observer, hg_disc.spec.prompt), (0, "dddjijian_huogong_discard"));
    assert!(hg_disc.spec.options.contains(&ChoiceOption::Card(heart_disc)), "xinhongyan Spade->Heart matches Heart");
    let hp1_before = eh.debug_hp(1);
    assert_mid_choice_packs(&eh, &[&d3h, &p9h, &auxh], vec![ChoiceOption::Card(heart_disc)]);
    answer(&mut eh, 0, vec![ChoiceOption::Card(heart_disc)]);
    drain(&mut eh);
    assert_eq!(eh.debug_hp(1), hp1_before - 1, "Huogong must deal 1 Fire damage");

    // 3) Jiedao (2-step holder -> victim) AND BOSS_REMEN on FireSha:
    let (mut ej, _d3j, _p9j, _auxj) = new_recheck_engine(92219, 4);
    ej.assign_general(0, bao).unwrap();
    ej.assign_general(1, male_g).unwrap();
    retain_only_skill(&mut ej, 0, Some("dddjijian"));
    retain_only_skill(&mut ej, 1, None);
    put_in_hand(&mut ej, 0, sha_owner3);
    put_in_hand(&mut ej, 1, jiedao);
    put_in_equip(&mut ej, 2, weapon2);
    ej.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(&mut ej, 0, GameAction::ActivateSkill { skill: jj });
    answer(&mut ej, 0, vec![ChoiceOption::Card(sha_owner3)]);
    answer(&mut ej, 0, vec![ChoiceOption::Player(1)]);
    let _show_jd = ej.open_choice().expect("dddjijian_show jiedao").clone();
    answer(&mut ej, 1, vec![ChoiceOption::Card(jiedao)]);
    let _uj = ej.open_choice().expect("dddjijian_use_card jiedao").clone();
    answer(&mut ej, 0, vec![ChoiceOption::Kind(CardKind::Jiedao as u16)]);
    let jh = ej.open_choice().expect("jiedao holder").clone();
    assert!(jh.spec.options.contains(&ChoiceOption::Player(2)));
    answer(&mut ej, 0, vec![ChoiceOption::Player(2)]);
    let jv = ej.open_choice().expect("jiedao victim").clone();
    assert!(!jv.spec.options.contains(&ChoiceOption::Player(0)), "Zhuge range 1 cannot reach seat 0 at dist 2");
    assert!(jv.spec.options.contains(&ChoiceOption::Player(3)));
    answer(&mut ej, 0, vec![ChoiceOption::Player(3)]);
    drain(&mut ej);

    // 4) BOSS_REMEN excludes seat 2 from FireSha targets:
    let (mut ef, _d3f, _p9f, _auxf) = new_recheck_engine(92319, 4);
    ef.assign_general(0, bao).unwrap();
    ef.assign_general(1, male_g).unwrap();
    retain_only_skill(&mut ef, 0, Some("dddjijian"));
    retain_only_skill(&mut ef, 1, None);
    ef.debug_grant(2, "热闷");
    put_in_hand(&mut ef, 0, huosha_owner);
    ef.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(&mut ef, 0, GameAction::ActivateSkill { skill: jj });
    answer(&mut ef, 0, vec![ChoiceOption::Card(huosha_owner)]);
    answer(&mut ef, 0, vec![ChoiceOption::Player(1)]);
    // Seat 1 has only 1 valid card (`huosha_owner`), so dddjijian auto-shows it and enters dddjijian_use_card directly!
    let _uf = ef.open_choice().expect("dddjijian_use_card huosha").clone();
    answer(&mut ef, 0, vec![ChoiceOption::Kind(CardKind::FireSha as u16)]);
    let tf = ef.open_choice().expect("dddjijian_use_target huosha").clone();
    assert!(!tf.spec.options.contains(&ChoiceOption::Player(2)), "BOSS_REMEN without armor blocks FireSha target");
    let _ = (&d3, &p9, &aux);
}

#[test]
fn recheck_20_dddjijian_old() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:1807-2004
    // Verify: Jiu usable outside target's turn; frozen step-4 rem_names multiset ignores newly drawn Wuzhong cards
    // and only consumes ONE copy per use (2 unshown Sha permit 2 uses); private CardsGiven (!CardsGivenPublic);
    // Huogong & Jiedao work cleanly.
    let (mut e, d3, p9, aux) = new_recheck_engine(92020, 4);
    let bao = find_general(&e, "ddd_baosanniang");
    e.assign_general(0, bao).unwrap();
    retain_only_skill(&mut e, 0, Some("dddjijian_old"));
    e.debug_grant(0, "dddjijian_old");
    let jjo = e.catalog().skill_by_name("dddjijian_old").unwrap();

    let sha1 = find_card_by(&e, |_, d| d.kind == CardKind::Sha);
    let sha2 = find_card_by(&e, |id, d| id != sha1 && d.kind == CardKind::Sha);
    let jiu = find_card_by(&e, |_, d| d.kind == CardKind::Jiu);
    let wuzhong = find_card_by(&e, |_, d| d.kind == CardKind::Wuzhong);
    let huogong = find_huogong(&e);
    let jiedao = find_card_by(&e, |_, d| d.kind == CardKind::Jiedao);
    let tao = find_card_by(&e, |_, d| d.kind == CardKind::Tao);
    let weapon2 = find_card_by(&e, |_, d| d.kind == CardKind::Zhuge);
    put_in_equip(&mut e, 2, weapon2);

    // Put 6 cards in male target seat 1's hand (required by `dddjijian_old` activate filter!):
    put_in_hand(&mut e, 1, sha1);
    put_in_hand(&mut e, 1, sha2);
    put_in_hand(&mut e, 1, jiu);
    put_in_hand(&mut e, 1, wuzhong);
    put_in_hand(&mut e, 1, huogong);
    put_in_hand(&mut e, 1, jiedao);
    let c0_give = find_card_by(&e, |id, d| id != sha1 && id != sha2 && id != jiu && id != wuzhong && id != huogong && id != jiedao && d.kind == CardKind::Sha);
    put_in_hand(&mut e, 0, c0_give);
    put_deck_top(&mut e, tao);
    put_deck_top(&mut e, tao);

    // Seat 0's turn (`turn.current == 0`, NOT target seat 1's turn!), so target seat 1 using virtual Jiu is off-turn!
    e.debug_set_turn(0, Phase::Play, 1);
    let log_start = e.log().len();
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: jjo });
    let _tgt = e.open_choice().expect("dddjijian_old_target").clone();
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    let show = e.open_choice().expect("dddjijian_old_show").clone();
    assert_eq!(show.spec.prompt, "dddjijian_old_show");
    // Seat 1 picks 2 cards to show (`huogong`, `jiedao` stay unshown? Wait show `[sha1, jiu]` vs give from 0 -> check all unshown rem_names!):
    // Have seat 1 show `[sha1, sha2]` or give `[c0_give]` from 0 so 1 shows 1 card and keeps others unshown in step 4:
    let show0 = show.spec.options[0];
    answer(&mut e, 1, vec![show0]);
    let g_c = e.open_choice().expect("dddjijian_old_give_sha").clone();
    answer(&mut e, 0, vec![g_c.spec.options[0]]);
    assert!(e.log()[log_start..].iter().any(|ev| matches!(ev, LogEvent::CardsGiven { from: 0, to: 1, .. })));
    assert!(!e.log()[log_start..].iter().any(|ev| matches!(ev, LogEvent::CardsGivenPublic { .. })));

    // Step 4 frozen multiset contains all seat 1 unshown cards (`sha1`, `sha2`, `jiu`, `wuzhong`, `huogong`, `jiedao` minus `show0`)!
    let u_jiu = e.open_choice().expect("dddjijian_old_use_card off-turn").clone();
    assert_eq!((u_jiu.observer, u_jiu.spec.prompt), (1, "dddjijian_old_use_card"));
    if u_jiu.spec.options.contains(&ChoiceOption::Card(jiu)) {
        assert_mid_choice_packs(&e, &[&d3, &p9, &aux], vec![ChoiceOption::Card(jiu)]);
        answer(&mut e, 1, vec![ChoiceOption::Card(jiu)]);
    } else {
        answer(&mut e, 1, vec![u_jiu.spec.options[0]]);
    }

    if let Some(u_wz) = e.open_choice().cloned() {
        if u_wz.spec.options.contains(&ChoiceOption::Card(wuzhong)) {
            answer(&mut e, 1, vec![ChoiceOption::Card(wuzhong)]);
            let u_after = e.open_choice().expect("dddjijian_old_use_card after wuzhong").clone();
            assert!(!u_after.spec.options.contains(&ChoiceOption::Card(tao)), "newly drawn Tao must NOT enter frozen rem_names");
        }
    }
    drain(&mut e);
}

#[test]
fn recheck_01_dddjuxian() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:244-383
    let (mut e, d3, p9, aux) = new_recheck_engine(92001, 4);
    let zhangcheng = find_general(&e, "ddd_zhangcheng");
    e.assign_general(0, zhangcheng).unwrap();
    retain_only_skill(&mut e, 0, Some("dddjuxian"));
    e.debug_grant(0, "连营");
    e.debug_grant(0, "xinhongyan");
    let juxian = e.catalog().skill_by_name("dddjuxian").unwrap();

    // `hand` branch yields after give (`CardsLost` -> 连营 draws 1 BEFORE pre_hand snapshot + start_draw)
    let eq0 = find_tengjia(&e);
    let c1 = find_card_by(&e, |id, _| id != eq0);
    put_in_hand(&mut e, 0, c1);
    e.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: juxian });
    let mode = e.open_choice().expect("dddjuxian_mode").clone();
    assert_eq!(mode.spec.prompt, "dddjuxian_mode");
    answer(&mut e, 0, vec![ChoiceOption::Tag(1)]);
    let c = e.open_choice().expect("dddjuxian_card").clone();
    assert_eq!(c.spec.prompt, "dddjuxian_card");
    answer(&mut e, 0, vec![ChoiceOption::Card(c1)]);
    let _tg = e.open_choice().expect("dddjuxian_target").clone();
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    // Yield after CardsLost/give fires 连营 (`trigger_confirm`) BEFORE `pre_hand` snapshot + start_draw!
    if let Some(ly) = e.open_choice().cloned() {
        if ly.spec.prompt == "trigger_confirm" {
            answer(&mut e, 0, vec![ChoiceOption::Confirm]);
        }
    }
    drain(&mut e);

    // `top` branch when owner has 1 `he` card (Spade under xinhongyan -> effective Red) + seat 1 has 1 `he` card:
    let (mut e2, _d3, _p9, _aux) = new_recheck_engine(92101, 4);
    e2.assign_general(0, zhangcheng).unwrap();
    retain_only_skill(&mut e2, 0, Some("dddjuxian"));
    e2.debug_grant(0, "xinhongyan");
    let sp = find_card_by(&e2, |_, d| d.suit == Suit::Spade);
    let sp2 = find_card_by(&e2, |id, d| id != sp && d.suit == Suit::Spade);
    let cl = find_card_by(&e2, |id, d| id != sp && id != sp2 && d.suit == Suit::Club);
    put_in_hand(&mut e2, 0, sp);
    put_in_hand(&mut e2, 0, sp2);
    put_in_hand(&mut e2, 1, cl);
    e2.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(&mut e2, 0, GameAction::ActivateSkill { skill: juxian });
    answer(&mut e2, 0, vec![ChoiceOption::Tag(0)]);
    let _pc = e2.open_choice().expect("dddjuxian_card top").clone();
    answer(&mut e2, 0, vec![ChoiceOption::Card(sp)]);
    let gt = e2.open_choice().expect("dddjuxian_target when owner+1 other have he").clone();
    assert_eq!(gt.spec.prompt, "dddjuxian_target");
    assert_eq!(gt.spec.options, vec![ChoiceOption::Player(1)], "only non-owner seat 1 is valid target");
    assert_mid_choice_packs(&e2, &[&d3, &p9, &aux], vec![ChoiceOption::Player(1)]);
}

#[test]
fn recheck_07_dddzhishu() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:37-58
    // Verify: equip move preserves declared_slots; Flow::Yield after equip move before virtual Dismantle;
    // respects blocks_all_cards / is_target_protected ("谦逊").
    let (mut e, d3, p9, aux) = new_recheck_engine(92007, 4);
    let qianzhao = find_general(&e, "ddd_qianzhao");
    e.assign_general(0, qianzhao).unwrap();

    let armor = find_tengjia(&e);
    put_in_equip(&mut e, 0, armor);
    e.state_core_mut().declared_slots.insert(armor, engine_native::catalog::EquipSlot::Weapon);
    let c1 = find_card_by(&e, |id, _| id != armor);
    let c2 = find_card_by(&e, |id, _| id != armor && id != c1);
    put_in_hand(&mut e, 1, c1);
    put_in_hand(&mut e, 2, c2);
    e.debug_set_turn(0, Phase::Draw, 1);
    let log_start = e.log().len();
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let eq_p = e.open_choice().expect("dddzhishu_card").clone();
    assert_eq!(eq_p.spec.prompt, "dddzhishu_card");
    answer(&mut e, 0, vec![ChoiceOption::Card(armor)]);
    let rcv = e.open_choice().expect("dddzhishu_target").clone();
    assert_eq!(rcv.spec.prompt, "dddzhishu_target");
    assert_mid_choice_packs(&e, &[&d3, &p9, &aux], vec![ChoiceOption::Player(1)]);
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    drain(&mut e);
    assert_eq!(
        e.state_core().declared_slots.get(&armor).copied(),
        Some(engine_native::catalog::EquipSlot::Weapon),
        "declared_slots must be preserved when equip moves via dddzhishu"
    );
    assert!(e.log()[log_start..].iter().any(|ev| matches!(ev, LogEvent::VirtualCardPlayed { kind: CardKind::Dismantle, player: 0, .. })));
}

#[test]
fn recheck_13_dddqiaoduan() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:747-801
    let (mut e, d3, p9, aux) = new_recheck_engine(92013, 3);
    e.debug_grant(0, "dddqiaoduan");
    e.debug_grant(0, "连营");

    let h0 = find_card_by(&e, |_, _| true);
    let h0b = find_card_by(&e, |id, _| id != h0);
    put_in_hand(&mut e, 0, h0);
    put_in_hand(&mut e, 0, h0b);
    let _ = e.debug_deal_damage(Some(2), 0, 1, DamageNature::Normal);
    drain(&mut e);
    let hp0_before = e.debug_hp(0);
    let chain = find_card_by(&e, |id, d| id != h0 && id != h0b && d.kind == CardKind::Chain);
    e.state_core_mut().player_mut(1).linked = true; // pre-link seat 1 so linking seat 2 makes 2 linked total!
    put_in_hand(&mut e, 2, chain);
    e.debug_set_turn(2, Phase::Play, 1);
    let _ = submit(&mut e, 2, GameAction::PlayCard { card: chain, targets: vec![2] });
    let qc = e.open_choice().expect("dddqiaoduan_cards").clone();
    assert_eq!((qc.observer, qc.spec.prompt, qc.spec.min, qc.spec.max), (0, "dddqiaoduan_cards", 2, 2));
    answer(&mut e, 0, vec![ChoiceOption::Card(h0), ChoiceOption::Card(h0b)]);
    let rc = e.open_choice().expect("dddqiaoduan_recover_target").clone();
    assert_eq!(rc.spec.prompt, "dddqiaoduan_recover_target");
    assert_mid_choice_packs(&e, &[&d3, &p9, &aux], Vec::new());
    answer(&mut e, 0, Vec::new());
    assert!(e.state_core().player(0).hand.contains(&h0), "cancel recover target must NOT discard cards");
    assert_eq!(e.debug_hp(0), hp0_before);
    drain(&mut e);
}

#[test]
fn recheck_14_ddddiedang() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:802-860
    let (mut e, d3, p9, aux) = new_recheck_engine(92014, 3);
    let sunliang = find_general(&e, "ddd_sunliang");
    e.assign_general(0, sunliang).unwrap();
    retain_only_skill(&mut e, 0, Some("ddddiedang"));
    e.debug_grant(0, "连营");
    let diedang = e.catalog().skill_by_name("ddddiedang").unwrap();

    let eq0 = find_tengjia(&e);
    let c1 = find_card_by(&e, |id, _| id != eq0);
    let c2 = find_card_by(&e, |id, _| id != eq0 && id != c1);
    put_in_equip(&mut e, 0, eq0);
    put_in_hand(&mut e, 1, c1);
    put_in_hand(&mut e, 2, c2);
    e.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: diedang });
    let _ch = e.open_choice().expect("ddddiedang_discard st=0").clone();
    answer(&mut e, 0, vec![ChoiceOption::Card(eq0)]);
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hand.len(), 3);
    assert_eq!(flag(&e, 0, "ddddiedang_state"), 1);

    let a = find_card_by(&e, |id, _| id != eq0 && id != c1 && id != c2);
    let b = find_card_by(&e, |id, _| id != eq0 && id != c1 && id != c2 && id != a);
    let d = find_card_by(&e, |id, _| id != eq0 && id != c1 && id != c2 && id != a && id != b);
    clear_hand(&mut e, 0);
    put_in_hand(&mut e, 0, a);
    put_in_hand(&mut e, 0, b);
    put_in_hand(&mut e, 0, d);
    e.debug_set_turn(0, Phase::Draw, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: diedang });
    let ch3 = e.open_choice().expect("ddddiedang_discard st=1").clone();
    assert_eq!((ch3.spec.min, ch3.spec.max), (3, 3));
    assert_mid_choice_packs(&e, &[&d3, &p9, &aux], vec![ChoiceOption::Card(a), ChoiceOption::Card(b), ChoiceOption::Card(d)]);
    let log_start = e.log().len();
    answer(&mut e, 0, vec![ChoiceOption::Card(a), ChoiceOption::Card(b), ChoiceOption::Card(d)]);
    // Confirm optional 连营 (`trigger_confirm`) during batch-discard yield BEFORE start_draw(1)!
    let ly = e.open_choice().expect("连营 trigger_confirm on ddddiedang st=1 yield").clone();
    assert_eq!((ly.observer, ly.spec.prompt), (0, "trigger_confirm"));
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hand.len(), 2, "连营 drew 1 + ddddiedang drew 1 = 2");
    assert!(e.log()[log_start..].iter().any(|ev| matches!(ev, LogEvent::CardDiscarded { player: 0, cards } if cards.len() == 3)));
    assert!(e.log()[log_start..].iter().any(|ev| matches!(ev, LogEvent::CardsDrawn { player: 0, .. })));
}

#[test]
fn recheck_15_dddanliu() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:861-967
    // Simultaneous 1-card swap; yields before start_draw(2)/recover; effective_is_red (xinhongyan Spade == Red)!
    let (mut e, d3, p9, aux) = new_recheck_engine(92015, 3);
    let sunliang = find_general(&e, "ddd_sunliang");
    e.assign_general(0, sunliang).unwrap();
    retain_only_skill(&mut e, 0, Some("dddanliu"));
    e.debug_grant(0, "xinhongyan"); // Spade on seat 0 counts as Heart/Red!

    let sp0 = find_card_by(&e, |_, d| d.suit == Suit::Spade);
    let heart1 = find_card_by(&e, |id, d| id != sp0 && d.suit == Suit::Heart);
    put_in_hand(&mut e, 0, sp0);
    put_in_hand(&mut e, 1, heart1);
    let _ = e.debug_deal_damage(Some(2), 0, 1, DamageNature::Normal);
    drain(&mut e);
    let hp0_before = e.debug_hp(0);

    e.debug_set_turn(0, Phase::Discard, 1);
    let log_start = e.log().len();
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let al = e.open_choice().expect("dddanliu_target").clone();
    assert_eq!(al.spec.prompt, "dddanliu_target");
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    let pick1 = e.open_choice().expect("dddanliu_pick_target").clone();
    assert_eq!((pick1.observer, pick1.spec.prompt), (0, "dddanliu_pick_target"));
    answer(&mut e, 0, vec![ChoiceOption::Card(heart1)]);
    let pick0 = e.open_choice().expect("dddanliu_pick_owner").clone();
    assert_eq!(pick0.observer, 1);
    answer(&mut e, 1, vec![ChoiceOption::Card(sp0)]);
    let sw = e.open_choice().expect("dddanliu_swap_confirm").clone();
    assert_eq!(sw.spec.prompt, "dddanliu_swap_confirm");
    assert_mid_choice_packs(&e, &[&d3, &p9, &aux], vec![ChoiceOption::Confirm]);
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    drain(&mut e);
    assert!(e.log()[log_start..].iter().any(|ev| matches!(ev, LogEvent::CardsGiven { from: 1, to: 0, .. })));
    assert!(e.log()[log_start..].iter().any(|ev| matches!(ev, LogEvent::CardsGiven { from: 0, to: 1, .. })));
    assert_eq!(e.debug_hp(0), hp0_before + 1, "xinhongyan Spade+Heart matching Red must recover HP");
}

#[test]
fn recheck_03_dddguiying() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:968-1203
    let (d3, p9, aux) = (build_u01_packs().0, build_u01_packs().1, aux_recheck_pack());
    let modes = [
        GameMode::Prototype,
        GameMode::Kingdoms,
        GameMode::Doudizhu { lord: 0 },
        GameMode::Boss {
            boss: 0,
            chongzheng: 0,
            fellows: Vec::new(),
            stage: BossStageState::default_none(),
        },
        GameMode::Versus {
            teams: vec![vec![0, 2], vec![1]],
            substitutes: vec![Vec::new(), Vec::new()],
        },
        GameMode::Kof {
            pools: [vec![0], vec![1]],
            wins: [0, 0],
            games: 0,
        },
    ];
    for (idx, mode) in modes.into_iter().enumerate() {
        let seats = if matches!(mode, GameMode::Kof { .. }) { 2 } else { 3 };
        let mut em = Engine::new_with_extensions(92003 + idx as u64, seats, &[&d3, &p9, &aux]).unwrap();
        em.state_core_mut().skills.clear();
        em.state_core_mut().mode = mode.clone();
        let sunliang = find_general(&em, "ddd_sunliang");
        let wu_g = (0..em.catalog().general_count() as u16)
            .find(|&idx| {
                idx != sunliang
                    && em.catalog()
                        .general_meta(idx)
                        .is_some_and(|m| m.1 == engine_native::catalog::Faction::Wu)
            })
            .unwrap();
        em.assign_general(0, sunliang).unwrap();
        em.assign_general(1, wu_g).unwrap();
        let c = find_card_by(&em, |_, _| true);
        put_in_hand(&mut em, 1, c);
        em.debug_set_turn((seats - 1) as u8, Phase::End, 1);
        let _ = submit(&mut em, (seats - 1) as u8, GameAction::NextPhase);
        assert!(em.open_choice().is_none(), "dddguiying MUST NOT fire in {mode:?}");
    }

    // Identity Lord triggers, yields after equip move, and dynamically picks up Wu player gaining a hand card mid-skill via xingchong!
    let (mut e, d3, p9, aux) = new_identity_recheck_engine(92103, 4);
    let sunliang = find_general(&e, "ddd_sunliang");
    let wu_g = (0..e.catalog().general_count() as u16)
        .find(|&idx| {
            idx != sunliang
                && e.catalog()
                    .general_meta(idx)
                    .is_some_and(|m| m.1 == engine_native::catalog::Faction::Wu)
        })
        .unwrap();
    let wk = find_general(&e, "ddd_wangkanglvkai");
    e.assign_general(0, sunliang).unwrap();
    retain_only_skill(&mut e, 0, Some("dddguiying"));
    e.assign_general(1, wu_g).unwrap();
    retain_only_skill(&mut e, 1, None);
    e.assign_general(2, sunliang).unwrap();
    retain_only_skill(&mut e, 2, None);
    let king1 = find_card_by(&e, |_, d| d.rank == 13);
    let eq2 = find_tengjia(&e); // Club 2 -> rank 2
    // Q2-L27 aligned `xingchong` to clear marks at `roundStart` (Seat 3 -> Seat 0), so use
    // `ssr_bailun` (`CardsLost` with empty hand & `hp <= 1`) to refill Seat 2's hand mid-skill.
    e.debug_grant(2, "ssr_bailun");
    e.debug_set_hp(2, 1);
    e.assign_general(3, wk).unwrap();
    put_in_hand(&mut e, 1, king1);
    put_in_equip(&mut e, 2, eq2);

    e.debug_set_turn(3, Phase::End, 1);
    let _ = submit(&mut e, 3, GameAction::NextPhase);
    let c1 = e.open_choice().expect("seat 1 dddguiying_confirm").clone();
    assert_eq!((c1.observer, c1.spec.prompt), (1, "dddguiying_confirm"));
    answer(&mut e, 1, vec![ChoiceOption::Confirm]);
    let _ej_p = e.open_choice().expect("dddguiying_ej_player").clone();
    assert_mid_choice_packs(&e, &[&d3, &p9, &aux], vec![ChoiceOption::Player(2)]);
    answer(&mut e, 1, vec![ChoiceOption::Player(2)]);
    // Seat 2 lost equip eq2 -> xingchong drew 2 into seat 2's hand -> guiying dynamically prompts seat 2!
    let c2 = e.open_choice().expect("seat 2 dynamically picked up by dddguiying!").clone();
    assert_eq!((c2.observer, c2.spec.prompt), (2, "dddguiying_confirm"));
}

#[test]
fn recheck_04_dddfushi() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3727-3865
    let (mut e, d3, p9, aux) = new_recheck_engine(92004, 3);
    let zhenji = find_general(&e, "ddd_zhenji");
    e.assign_general(0, zhenji).unwrap();
    e.debug_grant(0, "倾国");
    e.debug_grant(0, "kanpo");
    e.debug_grant(0, "xingchong");

    let black_eq = find_tengjia(&e);
    let black_sha = find_card_by(&e, |_, d| d.kind == CardKind::Sha && d.suit == Suit::Spade);
    let red_sha = find_card_by(&e, |_, d| d.kind == CardKind::Sha && d.suit == Suit::Heart);
    let trick = find_card_by(&e, |_, d| d.kind == CardKind::Wuzhong);
    let hand0 = find_card_by(&e, |id, _| id != black_eq && id != black_sha && id != red_sha && id != trick);
    let pile1 = find_card_by(&e, |id, _| id != black_eq && id != black_sha && id != red_sha && id != trick && id != hand0);
    let male = find_general(&e, "ddd_zhangmiao");
    e.assign_general(2, male).unwrap();
    remove_card_from_all_zones(&mut e, pile1);
    e.state_core_mut()
        .vm_aux
        .piles
        .insert((0, "dddmiaoxing_1".to_string()), vec![pile1]);
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "xingchong_round_used".to_string()), 1);
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, format!("xingchong_card:{hand0}")), 1);

    // Kanpo -> Nullify MUST NOT trigger dddfushi:
    put_in_hand(&mut e, 0, black_sha);
    put_in_hand(&mut e, 0, hand0);
    put_in_hand(&mut e, 1, trick);
    e.debug_set_turn(1, Phase::Play, 1);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: trick, targets: Vec::new() });
    let nw = e.open_choice().expect("nullify_window with kanpo").clone();
    if let Some(&kp_opt) = nw.spec.options.iter().find(|o| matches!(o, ChoiceOption::Skill(_))) {
        answer(&mut e, 0, vec![kp_opt]);
        if let Some(_kc) = e.open_choice().cloned() {
            answer(&mut e, 0, vec![ChoiceOption::Card(black_sha)]);
        }
    }
    assert!(
        e.open_choice().as_ref().is_none_or(|c| c.spec.prompt != "dddfushi_pile"),
        "Kanpo -> Nullify must NOT trigger dddfushi"
    );
    drain(&mut e);

    // 倾国 using black card directly in respond_shan MUST trigger dddfushi AND fushi_swap_pile emits CardsLost + yields BEFORE dddfushi_male_target!
    clear_hand(&mut e, 0);
    put_in_hand(&mut e, 0, black_eq);
    put_in_hand(&mut e, 0, hand0);
    put_in_hand(&mut e, 1, red_sha);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: red_sha, targets: vec![0] });
    let qg_resp = e.open_choice().expect("respond_shan with qingguo").clone();
    assert!(qg_resp.spec.options.contains(&ChoiceOption::Card(black_eq)), "倾国 offers black_eq directly in respond_shan");
    answer(&mut e, 0, vec![ChoiceOption::Card(black_eq)]);
    let fp = e.open_choice().expect("dddfushi_pile").clone();
    assert_eq!((fp.observer, fp.spec.prompt), (0, "dddfushi_pile"));
    assert_mid_choice_packs(&e, &[&d3, &p9, &aux], vec![ChoiceOption::Tag(0)]);
    answer(&mut e, 0, vec![ChoiceOption::Tag(0)]);
    let mt = e.open_choice().expect("dddfushi_male_target AFTER swap yield").clone();
    assert_eq!((mt.observer, mt.spec.prompt), (0, "dddfushi_male_target"));
    assert_eq!(e.state_core().player(0).hand.len(), 3, "pile1 (1) + xingchong CardsLost draw (2) = 3 BEFORE dddfushi_male_target");
    drain(&mut e);

    // Virtual basic Shan response via dddxiaxing ALSO triggers dddfushi on next turn!
    let mut e_virt = Engine::restore_with_extensions(&e.save(), &[&d3, &p9, &aux]).unwrap();
    e_virt.debug_grant(0, "dddxiaxing");
    e_virt.state_core_mut().vm_aux.flags.insert((0, "dddxiaxing_xia".to_string()), 1);
    let xx_def = e_virt.catalog().skill_by_name("dddxiaxing").unwrap();
    let sha_v = find_card_by(&e_virt, |id, d| id != red_sha && d.kind == CardKind::Sha);
    put_in_hand(&mut e_virt, 0, hand0);
    put_in_hand(&mut e_virt, 1, sha_v);
    e_virt.debug_set_turn(1, Phase::Play, 2);
    e_virt.state_core_mut().turn.sha_used_play_phase = false;
    let _ = submit(&mut e_virt, 1, GameAction::PlayCard { card: sha_v, targets: vec![0] });
    answer(&mut e_virt, 0, vec![ChoiceOption::Skill(xx_def)]);
    assert_eq!(
        e_virt.open_choice().expect("dddfushi_pile from virtual dddxiaxing Shan").spec.prompt,
        "dddfushi_pile"
    );
}

#[test]
fn recheck_06_dddfuzong() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:5948-5988
    // Identity Lord ONLY: NEVER triggers outside Identity, and draws via start_draw + SkillTriggered!
    let (d3, p9, aux) = (build_u01_packs().0, build_u01_packs().1, aux_recheck_pack());
    for (idx, mode) in [
        GameMode::Doudizhu { lord: 0 },
        GameMode::Boss {
            boss: 0,
            chongzheng: 0,
            fellows: Vec::new(),
            stage: BossStageState::default_none(),
        },
        GameMode::Kingdoms,
    ]
    .into_iter()
    .enumerate()
    {
        let mut em = Engine::new_with_extensions(92006 + idx as u64, 3, &[&d3, &p9, &aux]).unwrap();
        em.state_core_mut().skills.clear();
        em.state_core_mut().mode = mode;
        let liuhong = find_general(&em, "ddd_liuhong");
        em.assign_general(0, liuhong).unwrap();
        let log_start = em.log().len();
        em.debug_set_turn(2, Phase::End, 0);
        let _ = submit(&mut em, 2, GameAction::NextPhase);
        assert!(
            !em.log()[log_start..].iter().any(|ev| matches!(ev, LogEvent::SkillTriggered { skill, .. } if *skill == "dddfuzong")),
            "dddfuzong MUST NOT trigger outside Identity"
        );
    }

    let (mut e, _d3, _p9, _aux) = new_identity_recheck_engine(92106, 4);
    let liuhong = find_general(&e, "ddd_liuhong");
    e.assign_general(0, liuhong).unwrap();
    e.assign_general(1, liuhong).unwrap();
    let log_start = e.log().len();
    e.debug_set_turn(3, Phase::End, 0);
    let _ = submit(&mut e, 3, GameAction::NextPhase);
    assert!(e.log()[log_start..].iter().any(|ev| matches!(ev, LogEvent::SkillTriggered { skill, owner: 0, .. } if *skill == "dddfuzong")));
    assert!(e.log()[log_start..].iter().any(|ev| matches!(ev, LogEvent::CardsDrawn { player: 0, cards } if cards.len() == 2)));
}

#[test]
fn recheck_r3_01_dddjuxian_top_self_he_gain() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:302-339
    // When only `owner` has remaining `he` cards (`targets.length === 1`), `top` gains 1 from `owner`
    // (accepting both hand ChoiceOption::Card and equip ChoiceOption::Card) and checks color mismatch.
    let (mut e, d3, p9, aux) = new_recheck_engine(93001, 3);
    e.debug_grant(0, "dddjuxian");
    let juxian = e.catalog().skill_by_name("dddjuxian").unwrap();
    let red_pay = find_card_by(&e, |_, d| matches!(d.suit, Suit::Heart | Suit::Diamond));
    let blk_hand = find_card_by(&e, |_, d| matches!(d.suit, Suit::Spade | Suit::Club) && d.kind.equip_slot().is_none());
    let blk_eq = find_card_by(&e, |_, d| matches!(d.suit, Suit::Spade | Suit::Club) && d.kind.equip_slot().is_some());

    // Branch A: owner has 0 remaining `he` anywhere (others 0 too) -> finishes cleanly after payment.
    let mut e_zero = Engine::restore_with_extensions(&e.save(), &[&d3, &p9, &aux]).unwrap();
    put_in_hand(&mut e_zero, 0, red_pay);
    e_zero.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(&mut e_zero, 0, GameAction::ActivateSkill { skill: juxian });
    answer(&mut e_zero, 0, vec![ChoiceOption::Tag(0)]);
    answer(&mut e_zero, 0, vec![ChoiceOption::Card(red_pay)]);
    assert!(e_zero.open_choice().is_none(), "0 total `he` across all players finishes after pay");
    assert_eq!(e_zero.state_core().deck.last().copied(), Some(red_pay));

    // Branch B: only owner has remaining `he` (both black hand card & black equip).
    // Include `xingchong` + `dddshilie_shown_card` markers to verify upstream `content.ts:11251`:
    // picking own hand card emits ZERO `CardsLost`/`CardsMoved`/`CardsGiven` events (keeps
    // `xingchong_card` and `dddshilie_shown_card`, no draw), while picking own equip DOES emit
    // `CardsLost(Equip)` + `CardsGiven` (`xingchong` draws 2).
    e.debug_grant(0, "xingchong");
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, format!("xingchong_card:{blk_hand}")), 1);
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, format!("xingchong_card:{blk_eq}")), 1);
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, format!("dddshilie_shown_card:{blk_hand}")), 1);
    put_in_hand(&mut e, 0, red_pay);
    put_in_hand(&mut e, 0, blk_hand);
    put_in_equip(&mut e, 0, blk_eq);
    e.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: juxian });
    answer(&mut e, 0, vec![ChoiceOption::Tag(0)]);
    answer(&mut e, 0, vec![ChoiceOption::Card(red_pay)]);

    let g = e.open_choice().expect("dddjuxian_gain on self").clone();
    assert_eq!((g.observer, g.spec.prompt), (0, "dddjuxian_gain"));
    assert!(g.spec.options.contains(&ChoiceOption::Card(blk_hand)));
    assert!(g.spec.options.contains(&ChoiceOption::Card(blk_eq)));
    assert_mid_choice_packs(&e, &[&d3, &p9, &aux], vec![ChoiceOption::Card(blk_eq)]);

    // Pick own hand card on one fork: stays in hand, ZERO CardsLost/CardsMoved/CardsGiven, color mismatch bans dddjuxian.
    let mut e_hand = Engine::restore_with_extensions(&e.save(), &[&d3, &p9, &aux]).unwrap();
    let log_before_hand = e_hand.log().len();
    answer(&mut e_hand, 0, vec![ChoiceOption::Card(blk_hand)]);
    assert!(e_hand.open_choice().is_none());
    assert_eq!(e_hand.state_core().player(0).hand, vec![blk_hand]);
    assert_eq!(flag(&e_hand, 0, &format!("xingchong_card:{blk_hand}")), 1, "no CardsLost on self hand gain");
    assert_eq!(flag(&e_hand, 0, &format!("dddshilie_shown_card:{blk_hand}")), 1, "shown tag stays on self hand gain");
    assert!(
        !e_hand.log()[log_before_hand..]
            .iter()
            .any(|ev| matches!(ev, LogEvent::CardsGiven { .. } | LogEvent::CardsGivenPublic { .. } | LogEvent::CardsTaken { .. })),
        "no CardsGiven/CardsTaken when picking own hand card"
    );
    assert!(
        matches!(submit(&mut e_hand, 0, GameAction::ActivateSkill { skill: juxian }), CommandOutcome::Rejected(_)),
        "red paid vs black gained hand card triggers dddjuxian_ban"
    );

    // Pick own equip on live engine: moves equip into hand, emits CardsLost(Equip) + CardsGiven (`xingchong` draws 2), triggers dddjuxian_ban.
    let log_before_eq = e.log().len();
    answer(&mut e, 0, vec![ChoiceOption::Card(blk_eq)]);
    assert!(e.open_choice().is_none());
    assert!(e.state_core().player(0).equips.is_empty());
    assert!(e.state_core().player(0).hand.contains(&blk_eq));
    assert_eq!(e.state_core().player(0).hand.len(), 4, "blk_hand + blk_eq + xingchong draw 2");
    assert_eq!(flag(&e, 0, &format!("xingchong_card:{blk_eq}")), 0, "CardsLost(Equip) fired xingchong");
    assert!(
        e.log()[log_before_eq..]
            .iter()
            .any(|ev| matches!(ev, LogEvent::CardsGiven { from: 0, to: 0, cards } if cards == &vec![blk_eq]))
    );
    assert!(matches!(submit(&mut e, 0, GameAction::ActivateSkill { skill: juxian }), CommandOutcome::Rejected(_)));
}

#[test]
fn recheck_r3_19_dddjijian_add_count_and_in_turn_jiu_stacking() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:1705, 1757-1788
    // 1) `dddjijian` show filter allows ONLY `Sha` or normal trick cards (NEVER `Jiu`, basic non-Sha).
    // 2) `dddjijian` virtual `Sha` uses `addCount: false` (does not consume `sha_used_play_phase`,
    //    and remains offered after `sha_used_play_phase == true`).
    // 3) Multiple `Jiu` drinks in-turn (`jiu_active` + `jiu_extra_stacks`) stack onto `Sha` damage (`1 + 3 = 4`).
    let (d3, p9) = build_u01_packs();
    let aux = ExtensionPack {
        def: ExtensionDef {
            name: "u01_r3_jijian_jiu",
            version: 1,
            description: "Extra red Jiu cards for Jiu stacking test",
        },
        generals: Vec::new(),
        cards: vec![
            CardDef::basic("jiu_r3_1", CardKind::Jiu).with_suit_rank(Suit::Heart, 3),
            CardDef::basic("jiu_r3_2", CardKind::Jiu).with_suit_rank(Suit::Diamond, 9),
        ],
        host_skills: Vec::new(),
        vm_skills: Vec::new(),
        init_hook: InitHook::None,
        enabled: true,
    };
    let mut e = Engine::new_with_extensions(93019, 3, &[&d3, &p9, &aux]).unwrap();
    e.state_core_mut().skills.clear();
    for p in 0..3 {
        clear_hand(&mut e, p);
        clear_equips(&mut e, p);
        clear_judge(&mut e, p);
    }
    e.debug_grant(0, "dddjijian");
    let jijian = e.catalog().skill_by_name("dddjijian").unwrap();
    let p0 = find_general(&e, "ddd_liuchong");
    e.assign_general(0, p0).unwrap();
    e.assign_general(1, p0).unwrap();

    let give_sha1 = find_card_by(&e, |_, d| matches!(d.kind, CardKind::Sha) && matches!(d.suit, Suit::Spade));
    let blk_sha1 = find_card_by(&e, |id, d| id != give_sha1 && matches!(d.kind, CardKind::Sha) && matches!(d.suit, Suit::Spade | Suit::Club));
    let blk_jiu = find_card_by(&e, |_, d| matches!(d.kind, CardKind::Jiu) && matches!(d.suit, Suit::Spade | Suit::Club));
    let blk_wz = find_card_by(&e, |_, d| matches!(d.kind, CardKind::Wuzhong) && matches!(d.suit, Suit::Spade | Suit::Club));
    let play_sha = find_card_by(&e, |id, d| id != give_sha1 && id != blk_sha1 && matches!(d.kind, CardKind::Sha));
    let red_jiu1 = find_card_by(&e, |_, d| matches!(d.kind, CardKind::Jiu) && matches!(d.suit, Suit::Heart));
    let red_jiu2 = find_card_by(&e, |_, d| matches!(d.kind, CardKind::Jiu) && matches!(d.suit, Suit::Diamond));
    let any_jiu3 = find_card_by(&e, |id, d| id != red_jiu1 && id != red_jiu2 && matches!(d.kind, CardKind::Jiu));

    put_in_hand(&mut e, 0, give_sha1);
    put_in_hand(&mut e, 0, play_sha);
    put_in_hand(&mut e, 1, blk_sha1);
    put_in_hand(&mut e, 1, blk_wz);
    put_in_hand(&mut e, 1, blk_jiu);
    e.debug_set_hp(2, 5);
    e.debug_set_turn(0, Phase::Play, 1);

    // Save pre-activation snapshot so we can test BOTH orders (virtual Sha before physical Sha, and
    // physical Sha setting `sha_used_play_phase = true` BEFORE `dddjijian`'s usable-1 activation).
    let pre_activate = e.save();
    let mut e_after_sha = Engine::restore_with_extensions(&pre_activate, &[&d3, &p9, &aux]).unwrap();

    // P1 holds blk_sha1, blk_wz, blk_jiu -> `dddjijian_show` MUST exclude blk_jiu!
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: jijian });
    answer(&mut e, 0, vec![ChoiceOption::Card(give_sha1)]);
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    let show1 = e.open_choice().expect("dddjijian_show").clone();
    assert!(!show1.spec.options.contains(&ChoiceOption::Card(blk_jiu)), "dddjijian_show excludes basic Jiu");
    answer(&mut e, 1, vec![ChoiceOption::Card(blk_sha1), ChoiceOption::Card(blk_wz)]);
    // P0 uses virtual Sha on P2 (`addCount: false` -> `sha_used_play_phase` remains false!)
    answer(&mut e, 0, vec![ChoiceOption::Kind(CardKind::Sha as u16)]);
    answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
    answer(&mut e, 1, vec![ChoiceOption::Kind(CardKind::Wuzhong as u16)]);
    assert!(!e.state_core().turn.sha_used_play_phase);
    assert_eq!(e.debug_hp(2), 4);

    // Now P0 uses physical `play_sha` (still legal!), setting `sha_used_play_phase = true`
    assert!(!matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: play_sha, targets: vec![2] }),
        CommandOutcome::Rejected(_)
    ));
    assert!(e.state_core().turn.sha_used_play_phase);
    assert_eq!(e.debug_hp(2), 3);

    // `dddjijian` is `usable: 1` per Play phase -> 2nd activation in the same Play phase is rejected.
    assert!(matches!(
        submit(&mut e, 0, GameAction::ActivateSkill { skill: jijian }),
        CommandOutcome::Rejected(_)
    ));

    // Forked order: P0 plays physical `play_sha` FIRST (`sha_used_play_phase == true`),
    // THEN activates `dddjijian` -> virtual `Sha` is STILL offered and usable!
    assert!(!matches!(
        submit(&mut e_after_sha, 0, GameAction::PlayCard { card: play_sha, targets: vec![2] }),
        CommandOutcome::Rejected(_)
    ));
    assert!(e_after_sha.state_core().turn.sha_used_play_phase);
    assert_eq!(e_after_sha.debug_hp(2), 4);
    let _ = submit(&mut e_after_sha, 0, GameAction::ActivateSkill { skill: jijian });
    answer(&mut e_after_sha, 0, vec![ChoiceOption::Card(give_sha1)]);
    answer(&mut e_after_sha, 0, vec![ChoiceOption::Player(1)]);
    answer(&mut e_after_sha, 1, vec![ChoiceOption::Card(blk_sha1), ChoiceOption::Card(blk_wz)]);
    let use_after_sha = e_after_sha.open_choice().expect("dddjijian_use_card after physical Sha").clone();
    assert!(
        use_after_sha.spec.options.contains(&ChoiceOption::Kind(CardKind::Sha as u16)),
        "virtual Sha must bypass sha_used_play_phase limit"
    );
    assert_mid_choice_packs(&e_after_sha, &[&d3, &p9, &aux], vec![ChoiceOption::Kind(CardKind::Sha as u16)]);
    answer(&mut e_after_sha, 0, vec![ChoiceOption::Kind(CardKind::Sha as u16)]);
    answer(&mut e_after_sha, 0, vec![ChoiceOption::Player(2)]);
    answer(&mut e_after_sha, 1, vec![ChoiceOption::Kind(CardKind::Wuzhong as u16)]);
    assert!(e_after_sha.open_choice().is_none());
    assert_eq!(e_after_sha.debug_hp(2), 3);

    // Part B: 3 in-turn Jiu stacks (`fentian` unlimited red Jiu: 1st + 2nd + 3rd) deal `1 + 3 = 4` Sha damage!
    let mut e_jiu = Engine::new_with_extensions(93119, 3, &[&d3, &p9, &aux]).unwrap();
    e_jiu.state_core_mut().skills.clear();
    for p in 0..3 {
        clear_hand(&mut e_jiu, p);
        clear_equips(&mut e_jiu, p);
        clear_judge(&mut e_jiu, p);
    }
    e_jiu.debug_grant(0, engine_native::content::standard::BOSS_FENTIAN);
    let final_sha = find_card_by(&e_jiu, |_, d| matches!(d.kind, CardKind::Sha));
    put_in_hand(&mut e_jiu, 0, any_jiu3);
    put_in_hand(&mut e_jiu, 0, red_jiu1);
    put_in_hand(&mut e_jiu, 0, red_jiu2);
    put_in_hand(&mut e_jiu, 0, final_sha);
    e_jiu.state_core_mut().player_mut(1).max_hp = 5;
    e_jiu.state_core_mut().player_mut(1).hp = 5;
    e_jiu.debug_set_turn(0, Phase::Play, 1);
    assert!(!matches!(submit(&mut e_jiu, 0, GameAction::PlayCard { card: any_jiu3, targets: Vec::new() }), CommandOutcome::Rejected(_)));
    assert!(!matches!(submit(&mut e_jiu, 0, GameAction::PlayCard { card: red_jiu1, targets: Vec::new() }), CommandOutcome::Rejected(_)));
    assert!(!matches!(submit(&mut e_jiu, 0, GameAction::PlayCard { card: red_jiu2, targets: Vec::new() }), CommandOutcome::Rejected(_)));
    let _ = submit(&mut e_jiu, 0, GameAction::PlayCard { card: final_sha, targets: vec![1] });
    drain(&mut e_jiu);
    assert_eq!(e_jiu.debug_hp(1), 1, "3 stacked Jius in-turn must deal 4 damage (5->1)");
}

#[test]
fn recheck_r3_20_dddjijian_old_jiu_no_count_and_stacking() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:1955, 1973
    // `dddjijian_old` also uses `hasUseTarget(card)` + `chooseUseTarget(card, true, false)` (`addCount: false`):
    // 2 unshown Jius in P1's hand let P1 use BOTH virtual Jius off-turn in one turn and stack +2 onto a Sha!
    let (mut e, d3, p9, aux) = new_recheck_engine(93020, 3);
    e.debug_grant(0, "dddjijian_old");
    e.debug_grant(0, "dddjijian");
    let jijian_old = e.catalog().skill_by_name("dddjijian_old").unwrap();
    let jijian = e.catalog().skill_by_name("dddjijian").unwrap();
    let p0 = find_general(&e, "ddd_liuchong");
    e.assign_general(0, p0).unwrap();
    e.assign_general(1, p0).unwrap();

    let red_sha = find_card_by(&e, |_, d| matches!(d.kind, CardKind::Sha) && matches!(d.suit, Suit::Heart | Suit::Diamond));
    let s1 = find_card_by(&e, |_, d| matches!(d.suit, Suit::Heart | Suit::Diamond) && matches!(d.kind, CardKind::Shan));
    let s2 = find_card_by(&e, |id, d| id != s1 && matches!(d.suit, Suit::Heart | Suit::Diamond) && matches!(d.kind, CardKind::Shan));
    let j1 = find_card_by(&e, |_, d| matches!(d.kind, CardKind::Jiu));
    let j2 = find_card_by(&e, |id, d| id != j1 && matches!(d.kind, CardKind::Jiu));
    let give_blk_sha = find_card_by(&e, |_, d| matches!(d.kind, CardKind::Sha) && matches!(d.suit, Suit::Spade | Suit::Club));
    let show_blk_sha = find_card_by(&e, |id, d| id != give_blk_sha && matches!(d.kind, CardKind::Sha) && matches!(d.suit, Suit::Spade | Suit::Club));
    let show_blk_wz = find_card_by(&e, |_, d| matches!(d.kind, CardKind::Wuzhong) && matches!(d.suit, Suit::Spade | Suit::Club));

    put_in_hand(&mut e, 0, red_sha);
    put_in_hand(&mut e, 1, s1);
    put_in_hand(&mut e, 1, s2);
    put_in_hand(&mut e, 1, j1);
    put_in_hand(&mut e, 1, j2);
    e.debug_set_hp(2, 4);
    e.debug_set_turn(0, Phase::Play, 1);

    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: jijian_old });
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    answer(&mut e, 1, vec![ChoiceOption::Card(s1), ChoiceOption::Card(s2)]);
    answer(&mut e, 0, vec![ChoiceOption::Card(red_sha)]);

    // rem == 2, unshown names contain [Jiu, Jiu]
    let u1 = e.open_choice().expect("1st dddjijian_old_use_card").clone();
    assert_eq!((u1.observer, u1.spec.prompt), (1, "dddjijian_old_use_card"));
    assert!(u1.spec.options.contains(&ChoiceOption::Kind(CardKind::Jiu as u16)));
    answer(&mut e, 1, vec![ChoiceOption::Kind(CardKind::Jiu as u16)]);

    let u2 = e.open_choice().expect("2nd dddjijian_old_use_card must STILL offer Jiu").clone();
    assert_eq!((u2.observer, u2.spec.prompt), (1, "dddjijian_old_use_card"));
    assert!(u2.spec.options.contains(&ChoiceOption::Kind(CardKind::Jiu as u16)));
    assert_mid_choice_packs(&e, &[&d3, &p9, &aux], vec![ChoiceOption::Kind(CardKind::Jiu as u16)]);
    answer(&mut e, 1, vec![ChoiceOption::Kind(CardKind::Jiu as u16)]);
    assert!(e.open_choice().is_none());

    // P1 off-turn uses virtual Sha via dddjijian in same turn -> deals 1 + 2 = 3 damage to P2!
    clear_hand(&mut e, 1);
    put_in_hand(&mut e, 0, give_blk_sha);
    put_in_hand(&mut e, 1, show_blk_sha);
    put_in_hand(&mut e, 1, show_blk_wz);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: jijian });
    answer(&mut e, 0, vec![ChoiceOption::Card(give_blk_sha)]);
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    answer(&mut e, 1, vec![ChoiceOption::Card(show_blk_sha), ChoiceOption::Card(show_blk_wz)]);
    answer(&mut e, 0, vec![ChoiceOption::Kind(CardKind::Wuzhong as u16)]);
    answer(&mut e, 1, vec![ChoiceOption::Kind(CardKind::Sha as u16)]);
    answer(&mut e, 1, vec![ChoiceOption::Player(2)]);
    assert_eq!(e.debug_hp(2), 1, "dddjijian_old 2 off-turn virtual Jius must stack +2 dmg (4->1)");
}

#[test]
fn recheck_r3_04_dddfushi_sadeng_hujia_and_kanpo_stale_guard() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3727-3865
    // 1) `sadengjinhuan_extra_shan` (incl. Qingguo black hand card) must call `emit_card_responded_as(..., Shan)`.
    // 2) `hujia` Wei ally `Shan` (physical OR Qingguo) must call `emit_card_responded_as(..., Shan)` and NOT
    //    inherit stale `dddfushi_resp_eff_kind = Nullify+1` from an earlier `kanpo` in the same turn.
    let (d3, p9) = build_u01_packs();
    let sd_card = CardDef::basic("sadengjinhuan", CardKind::Sadengjinhuan).with_suit_rank(Suit::Club, 6);
    let sd_pack = ExtensionPack {
        def: ExtensionDef {
            name: "u01_r3_sadeng",
            version: 1,
            description: "Sadengjinhuan treasure test card",
        },
        generals: Vec::new(),
        cards: vec![sd_card],
        host_skills: Vec::new(),
        vm_skills: Vec::new(),
        init_hook: InitHook::None,
        enabled: true,
    };

    // Part A: `sadengjinhuan_extra_shan` answered with Qingguo black non-basic card triggers `dddfushi`.
    {
        let mut e = Engine::new_with_extensions(93004, 3, &[&d3, &p9, &sd_pack]).unwrap();
        e.state_core_mut().skills.clear();
        for p in 0..3 {
            clear_hand(&mut e, p);
            clear_equips(&mut e, p);
            clear_judge(&mut e, p);
        }
        e.debug_grant(0, "dddfushi");
        e.debug_grant(0, "倾国");
        let sd = find_card_by(&e, |_, d| d.kind == CardKind::Sadengjinhuan);
        let shan = find_card_by(&e, |_, d| matches!(d.kind, CardKind::Shan));
        let sha = find_card_by(&e, |_, d| matches!(d.kind, CardKind::Sha));
        let qg_black = find_card_by(&e, |_, d| matches!(d.kind, CardKind::Wuzhong) && matches!(d.suit, Suit::Spade | Suit::Club));
        let red_sadeng = find_card_by(&e, |id, d| id != sha && id != shan && matches!(d.suit, Suit::Heart));
        let p1_c = find_card_by(&e, |id, _| {
            id != sd && id != shan && id != sha && id != qg_black && id != red_sadeng
        });
        e.state_core_mut().vm_aux.piles.insert((0, "dddmiaoxing_1".to_string()), vec![p1_c]);
        e.state_core_mut().deck.retain(|&c| c != p1_c);
        put_in_equip(&mut e, 1, sd);
        put_in_hand(&mut e, 1, sha);
        put_in_hand(&mut e, 0, shan);
        put_in_hand(&mut e, 0, qg_black);
        put_deck_top(&mut e, red_sadeng);
        e.debug_set_turn(1, Phase::Play, 1);

        let _ = submit(&mut e, 1, GameAction::PlayCard { card: sha, targets: vec![0] });
        answer(&mut e, 0, vec![ChoiceOption::Card(shan)]);
        assert_eq!(e.open_choice().unwrap().spec.prompt, "dddfushi_pile");
        answer(&mut e, 0, Vec::new()); // decline 1st so sadengjinhuan_extra_shan triggers dddfushi cleanly in isolation
        assert_eq!(e.open_choice().unwrap().spec.prompt, "sadengjinhuan_judge_confirm");
        answer(&mut e, 1, vec![ChoiceOption::Confirm]);
        let ex = e.open_choice().expect("sadengjinhuan_extra_shan").clone();
        assert_eq!((ex.observer, ex.spec.prompt), (0, "sadengjinhuan_extra_shan"));
        assert!(ex.spec.options.contains(&ChoiceOption::Card(qg_black)));
        answer(&mut e, 0, vec![ChoiceOption::Card(qg_black)]);
        let fp = e.open_choice().expect("dddfushi_pile after Qingguo on sadengjinhuan_extra_shan").clone();
        assert_eq!((fp.observer, fp.spec.prompt), (0, "dddfushi_pile"));
        assert_mid_choice_packs(&e, &[&d3, &p9, &sd_pack], vec![ChoiceOption::Tag(0)]);
    }

    // Part B: Wei ally `dddfushi` + `kanpo` + `qingguo` uses `kanpo` (sets Nullify+1), then answers `hujia_respond_shan`
    // with BOTH (i) physical `Shan` (must NOT inherit stale Nullify+1!) and (ii) Qingguo black non-basic card (`dddfushi_pile` opens!).
    {
        let mut e = Engine::try_new_identity_with_extensions(93104, 4, &[&d3, &p9]).expect("identity");
        e.state_core_mut().skills.clear();
        for p in 0..4 {
            clear_hand(&mut e, p);
            clear_equips(&mut e, p);
            clear_judge(&mut e, p);
        }
        let wei = find_general(&e, "ddd_caomao");
        e.assign_general(0, wei).unwrap();
        e.assign_general(2, wei).unwrap();
        e.state_core_mut().skills.clear();
        e.debug_grant(0, "护驾");
        e.debug_grant(2, "dddfushi");
        e.debug_grant(2, "倾国");

        let wz = find_card_by(&e, |_, d| matches!(d.kind, CardKind::Wuzhong));
        let sha = find_card_by(&e, |id, d| id != wz && matches!(d.kind, CardKind::Sha));
        let wuxie_c = find_card_by(&e, |id, d| id != wz && id != sha && matches!(d.kind, CardKind::Nullify));
        let qg_blk = find_card_by(&e, |id, d| id != wz && id != sha && id != wuxie_c && matches!(d.kind, CardKind::Snatch) && matches!(d.suit, Suit::Spade | Suit::Club));
        let phys_shan = find_card_by(&e, |id, d| id != wz && id != sha && id != wuxie_c && id != qg_blk && matches!(d.kind, CardKind::Shan));
        let pile_c = find_card_by(&e, |id, _| id != wz && id != sha && id != wuxie_c && id != qg_blk && id != phys_shan);

        e.state_core_mut().vm_aux.piles.insert((2, "dddmiaoxing_1".to_string()), vec![pile_c]);
        e.state_core_mut().deck.retain(|&c| c != pile_c);
        put_in_hand(&mut e, 1, wz);
        put_in_hand(&mut e, 1, sha);
        put_in_hand(&mut e, 2, wuxie_c);
        put_in_hand(&mut e, 2, qg_blk);
        put_in_hand(&mut e, 2, phys_shan);
        e.debug_set_turn(1, Phase::Play, 1);

        // Seat 1 plays Wuzhong -> Seat 2 plays Nullify (Nullify+1 written!)
        let _ = submit(&mut e, 1, GameAction::PlayCard { card: wz, targets: Vec::new() });
        while let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt == "nullify_window" && c.observer == 2 {
                answer(&mut e, 2, vec![ChoiceOption::Card(wuxie_c)]);
            } else {
                answer(&mut e, c.observer, vec![ChoiceOption::Pass]);
            }
        }

        // Now Seat 1 plays Sha targeting Wei Lord Seat 0 -> Hujia directly asks Wei ally Seat 2!
        let _ = submit(&mut e, 1, GameAction::PlayCard { card: sha, targets: vec![0] });
        let hj = e.open_choice().expect("hujia_respond on seat 2").clone();
        assert_eq!((hj.observer, hj.spec.prompt), (2, "hujia_respond"));

        // Fork 1: Seat 2 answers Hujia with physical Shan -> MUST NOT see stale Nullify+1!
        let mut e_phys = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
        answer(&mut e_phys, 2, vec![ChoiceOption::Card(phys_shan)]);
        assert_eq!(
            e_phys.open_choice().expect("dddfushi_pile on Hujia physical Shan after kanpo").spec.prompt,
            "dddfushi_pile"
        );

        // Live: Seat 2 answers Hujia with Qingguo black non-basic -> MUST trigger dddfushi_pile!
        answer(&mut e, 2, vec![ChoiceOption::Card(qg_blk)]);
        assert_eq!(
            e.open_choice().expect("dddfushi_pile on Hujia Qingguo").spec.prompt,
            "dddfushi_pile"
        );
        assert_mid_choice_packs(&e, &[&d3, &p9], vec![ChoiceOption::Tag(0)]);
    }
}

#[test]
fn recheck_r3_04b_hujia_dddfushi_settles_before_shaddodged_qianglue() {
    // upstream archive/upstream/noname/apps/core/character/standard/skill.js:287-305
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:3727-3865
    // upstream archive/upstream/noname/apps/core/extension/英雄杀/character/skill.js:1610-1628
    // Wei ally's `CardResponded` triggers (`dddfushi`) MUST settle BEFORE `ShaDodged` (`qianglue`):
    // if P0 Cao Cao had 0 `he` cards and gains P2's `dddmiaoxing` pile via `dddfushi`, `qianglue`
    // sees P0's new hand card and triggers AFTER `dddfushi` resolves!
    let (d3, p9) = build_u01_packs();
    let mut e = Engine::try_new_identity_with_extensions(93105, 4, &[&d3, &p9]).expect("identity");
    e.state_core_mut().skills.clear();
    for p in 0..4 {
        clear_hand(&mut e, p);
        clear_equips(&mut e, p);
        clear_judge(&mut e, p);
    }
    let wei_male = find_general(&e, "ddd_caomao");
    e.assign_general(0, wei_male).unwrap();
    e.assign_general(2, wei_male).unwrap();
    e.state_core_mut().skills.clear();
    e.debug_grant(0, "护驾");
    e.debug_grant(1, "qianglue");
    e.debug_grant(2, "dddfushi");

    let sha = find_card_by(&e, |_, d| matches!(d.kind, CardKind::Sha));
    let shan = find_card_by(&e, |_, d| matches!(d.kind, CardKind::Shan));
    let pile_c1 = find_card_by(&e, |id, _| id != sha && id != shan);
    let pile_c2 = find_card_by(&e, |id, _| id != sha && id != shan && id != pile_c1);
    let p0_extra = find_card_by(&e, |id, _| id != sha && id != shan && id != pile_c1 && id != pile_c2);

    e.state_core_mut().vm_aux.piles.insert((2, "dddmiaoxing_1".to_string()), vec![pile_c1]);
    e.state_core_mut().vm_aux.piles.insert((2, "dddmiaoxing_2".to_string()), vec![pile_c2]);
    e.state_core_mut().deck.retain(|&c| c != pile_c1 && c != pile_c2);
    put_in_hand(&mut e, 1, sha);
    put_in_hand(&mut e, 2, shan);
    e.debug_set_turn(1, Phase::Play, 1);

    // Fork A: P0 already holds `p0_extra` -> even so, `(2, "dddfushi_pile")` opens BEFORE `(1, "qianglue:confirm")`.
    let mut e_with_hand = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
    put_in_hand(&mut e_with_hand, 0, p0_extra);
    let _ = submit(&mut e_with_hand, 1, GameAction::PlayCard { card: sha, targets: vec![0] });
    assert_eq!((e_with_hand.open_choice().unwrap().observer, e_with_hand.open_choice().unwrap().spec.prompt), (2, "hujia_respond"));
    answer(&mut e_with_hand, 2, vec![ChoiceOption::Card(shan)]);
    assert_eq!((e_with_hand.open_choice().unwrap().observer, e_with_hand.open_choice().unwrap().spec.prompt), (2, "dddfushi_pile"));
    answer(&mut e_with_hand, 2, Vec::new());
    assert_eq!((e_with_hand.open_choice().unwrap().observer, e_with_hand.open_choice().unwrap().spec.prompt), (1, "qianglue:confirm"));

    // Live: P0 starts with 0 `he` cards. P2 answers `hujia_respond` -> `dddfushi_pile` opens FIRST!
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: sha, targets: vec![0] });
    answer(&mut e, 2, vec![ChoiceOption::Card(shan)]);
    let c_pile = e.open_choice().expect("dddfushi_pile must open first").clone();
    assert_eq!((c_pile.observer, c_pile.spec.prompt), (2, "dddfushi_pile"));

    // Fork B: P2 declines `dddfushi_pile` while P0 still has 0 `he` -> `qianglue` does NOT trigger!
    let mut e_decline = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
    answer(&mut e_decline, 2, Vec::new());
    assert!(e_decline.open_choice().is_none(), "qianglue must skip when P0 stays at 0 he");

    // Live swap: P2 swaps `Tag(0)`, chooses male P0, and P0 swaps empty hand with `dddmiaoxing_2` (`Tag(1)`),
    // giving P0 `[pile_c2]` BEFORE `ShaDodged` so `qianglue:confirm` opens SECOND!
    answer(&mut e, 2, vec![ChoiceOption::Tag(0)]);
    let c_male = e.open_choice().expect("dddfushi_male_target").clone();
    assert_eq!((c_male.observer, c_male.spec.prompt), (2, "dddfushi_male_target"));
    answer(&mut e, 2, vec![ChoiceOption::Player(0)]);
    let c_p0_pile = e.open_choice().expect("P0 dddfushi_target_pile").clone();
    assert_eq!((c_p0_pile.observer, c_p0_pile.spec.prompt), (0, "dddfushi_target_pile"));
    answer(&mut e, 0, vec![ChoiceOption::Tag(1)]);
    assert_eq!(e.state_core().player(0).hand, vec![pile_c2]);

    let c_ql = e.open_choice().expect("qianglue:confirm must open AFTER dddfushi settles").clone();
    assert_eq!((c_ql.observer, c_ql.spec.prompt), (1, "qianglue:confirm"));
    assert_mid_choice_packs(&e, &[&d3, &p9], vec![ChoiceOption::Pass]);
    answer(&mut e, 1, vec![ChoiceOption::Pass]);
    assert!(e.open_choice().is_none());
    assert_eq!(e.debug_hp(0), 4, "Hujia ShaDodged prevents damage to Wei Lord P0");
}

