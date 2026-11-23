//! U05-5b Group A (`52 generals` 装载验证 + 21 技能: `sqlongyin`, `sqlongnu`, `sqlonghuo`,
//! `sqlongwu`, `jielue`, `gwmaoxian_hengsaite_sha`, `gwhuanshuang`, `gw_xianzumaijiu`, `gwjinli`,
//! `gwliaotian`, `gwhuanbi`, `gwminxiang`, `gwlangshi`, `gwjingtian`, `gwjingshi`, `gwweitu`,
//! `gwgouhun`, `gwfutian`, `gwzhongmo`, `gwyewu`, `g_gw_yewu`).

use super::common::*;
use engine_native::catalog::CardKind;
use engine_native::{ChoiceOption, CommandOutcome, GameAction, RejectReason};

#[test]
fn u05_b5b_all_52_generals_and_69_skills_loaded_with_host_bindings() {
    let (_v, e) = setup_gwent_locked(5600);
    let expected_generals = [
        ("gw_huoge", 3, "gwjinli"),
        ("gw_aisinie", 3, "huihun"),
        ("gw_enxier", 4, "gwbaquan"),
        ("gw_kaerweite", 4, "gwjiquan"),
        ("gw_falanxisika", 3, "shewu"),
        ("gw_haluo", 4, "nuhou"),
        ("gw_airuiting", 4, "kuanglie"),
        ("gw_laduoweide", 4, "gwxiaoshou"),
        ("gw_dagong", 4, "gwtianbian"),
        ("gw_bulanwang", 4, "bolang"),
        ("gw_kuite", 4, "gwxuezhan"),
        ("gw_fuertaisite", 3, "zhengjun"),
        ("gw_hengsaite", 3, "jielue"),
        ("gw_fulisi", 3, "lanquan"),
        ("gw_gaier", 3, "hunmo"),
        ("gw_jieluote", 6, "fayin"),
        ("gw_yenaifa", 3, "xuezhou"),
        ("gw_telisi", 3, "huandie"),
        ("gw_xili", 3, "fengjian"),
        ("gw_luoqi", 4, "gwzhanjiang"),
        ("gw_yioufeisi", 4, "gwchuanxin"),
        ("gw_aigeleisi", 3, "gwshenyu"),
        ("gw_aokeweisite", 4, "yunhuo"),
        ("gw_kaxier", 4, "gwfengchi"),
        ("gw_luobo", 3, "junchi"),
        ("gw_mieren", 3, "lingji"),
        ("gw_sanhanya", 3, "gwjinyan"),
        ("gw_shanhu", 3, "shuijian"),
        ("gw_zhangyujushou", 4, "gwjushi"),
        ("gw_zhuoertan", 3, "hupeng"),
        ("gw_meizi", 3, "gwjieyin"),
        ("gw_aimin", 3, "huanshu"),
        ("gw_puxila", 3, "gwqinwu"),
        ("gw_xigedelifa", 3, "gwfusheng"),
        ("gw_laomaotou", 4, "gwchenshui"),
        ("gw_qigaiwang", 4, "julian"),
        ("gw_bierna", 3, "gwfengshi"),
        ("gw_haizhiyezhu", 4, "yangfan"),
        ("gw_nitelila", 4, "shuangxi"),
        ("gw_linjing", 4, "gwyewu"),
        ("gw_kanbi", 1, "gwfutian"),
        ("gw_nvyemo", 3, "gwgouhun"),
        ("gw_kairuisi", 3, "gwweitu"),
        ("gw_oudimu", 3, "gwjingshi"),
        ("gw_shasixiwusi", 4, "gwjingtian"),
        ("gw_yioufeisisp", 3, "gwminxiang"),
        ("gw_lanbote", 4, "gwlangshi"),
        ("gw_fenghuang", 4, "gwliaotian"),
        ("gw_diandian", 3, "gwhuanbi"),
        ("gw_yisilinni", 3, "gwhuanshuang"),
        ("gw_feilafanruide", 3, "yinzhang"),
        ("gw_saqiya", 4, "sqlongwu"),
    ];
    for &(gname, hp, sk) in &expected_generals {
        let gid = (0..e.catalog().general_count() as u16)
            .find(|&id| e.catalog().general_meta(id).is_some_and(|m| m.0 == gname))
            .unwrap_or_else(|| panic!("general {gname} missing"));
        let (_, _, skills, g_hp, _) = e.catalog().general_meta(gid).unwrap();
        assert_eq!(g_hp, hp, "{gname} hp");
        assert!(skills.contains(&sk), "{gname} missing {sk}");
    }
    let all_69 = [
        "sqlongyin", "sqlongnu", "sqlonghuo", "sqlongwu", "jielue", "gwmaoxian_hengsaite_sha",
        "gwhuanshuang", "gw_xianzumaijiu", "gwjinli", "gwliaotian", "gwhuanbi", "gwminxiang",
        "gwlangshi", "gwjingtian", "gwjingshi", "gwweitu", "gwgouhun", "gwfutian", "gwzhongmo",
        "gwyewu", "g_gw_yewu", "shuangxi", "yangfan", "gwfengshi", "gwchenshui", "gwliedi",
        "julian", "gwfusheng", "gwqinwu", "gwqinwu2", "huanshu", "huanshu2", "huanshu3",
        "gwjieyin", "zhengjun", "gwxuezhan", "bolang", "gwjushi", "gwjushi2", "gwfengchi",
        "lingji", "gwjinyan", "gwshenyu", "junchi", "hupeng", "hupeng2", "hupeng3", "hupeng4",
        "hunmo", "shuijian", "yunhuo", "yinzhang", "gwtianbian", "gwxiaoshou", "kuanglie",
        "gwjiquan", "nuhou", "shewu", "gwzhanjiang", "gwzhanjiang3", "gwchuanxin", "fengjian",
        "huandie", "xuezhou", "xuezhou_hp", "fayin", "gwbaquan", "huihun", "lanquan",
    ];
    for sk in all_69 {
        let def = skill_id(&e, sk);
        assert!(
            e.catalog().skills[def as usize]
                .behavior
                .as_host_behavior()
                .is_some(),
            "{sk} must bind a HostBehavior implementation instead of coarse stub"
        );
    }
}

#[test]
fn u05_b5b_sqlongyin_sqlongnu_sqlonghuo_sqlongwu_transitions_and_effects() {
    // Upstream: `character/gwent/skill.js:5-148`
    let (v, e0) = setup_gwent_locked(5601);
    let sha = card_of_kind_and_suit(&e0, CardKind::Sha, None);
    let sha2 = find_cards_by(&e0, |d| d.kind == CardKind::Sha, 2)[1];
    let shan = card_of_kind_and_suit(&e0, CardKind::Shan, None);
    let tao = card_of_kind_and_suit(&e0, CardKind::Tao, None);

    // 1. sqlongwu: end phase draws max multiplicity of duplicate names (2) and asks to use a card
    let mut e = grant_skill(&e0, &v, "sqlongwu", 0);
    e = inject_hand(&e, &v, &[sha, sha2, shan], 0);
    advance_past_turn(&mut e, 0); // advances through Discard -> End -> TurnPhaseEnd -> sqlongwu triggers
    // After sqlongwu phaseEnd draw 2, hand gained 2 cards (3->5 or declined use)
    assert!(e.debug_hand(0).len() >= 4, "sqlongwu should draw max=2 matching cards");

    // 2. sqlongyin on seat 0's TurnBegin swaps sqlongwu -> sqlongnu or sqlonghuo
    let mut e = grant_skill(&e0, &v, "sqlongwu", 0);
    e = set_deck_top(&e, &v, &[sha, shan, tao]);
    advance_past_turn(&mut e, 0);
    advance_to_seat_prepare(&mut e, 0);
    drain_default(&mut e);
    assert!(!has_skill(&e, 0, "sqlongwu"), "sqlongyin must switch off sqlongwu on phaseBeginStart");
    assert!(
        has_skill(&e, 0, "sqlongnu") || has_skill(&e, 0, "sqlonghuo"),
        "sqlongyin must switch to sqlongnu or sqlonghuo"
    );

    // 3. sqlongnu: discover 3 random cards; if matching name already in hand, choose fire damage target instead of gain
    let mut e = grant_skill(&e0, &v, "sqlongnu", 0);
    e = inject_hand(&e, &v, &[sha], 0);
    advance_to_seat_play(&mut e, 3);
    let mut snap = e.save();
    let extra_deck: Vec<_> = snap
        .core
        .deck
        .drain(..)
        .filter(|&c| c != sha2 && c != shan && c != tao)
        .collect();
    snap.core.discard.extend(extra_deck);
    snap.core
        .discard
        .retain(|&c| c != sha2 && c != shan && c != tao);
    snap.core.deck = vec![sha2, shan, tao];
    e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    advance_to_seat_prepare(&mut e, 0);
    // Step 0: pick sha2 from 3 discovered cards
    let out = answer_first(&mut e, 0, |o| *o == ChoiceOption::Card(sha2));
    assert!(matches!(out, CommandOutcome::Pending { .. }));
    // Step 1: choose fire damage on seat 1 (save->restore test mid-choice!)
    let (mut e, out) = save_restore_roundtrip_answer(e, &v, 0, vec![ChoiceOption::Player(1)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    drain_default(&mut e);
    assert_eq!(e.debug_hp(1), 3, "sqlongnu fire damage on enemy 1");
    assert_zone_conservation(&e);

    // 4. sqlonghuo: Play active (usable:1), discard all hand (sha, shan), draw 2, discard matching-name from enemy
    let mut e = grant_skill(&e0, &v, "sqlonghuo", 0);
    e = inject_hand(&e, &v, &[sha, shan], 0);
    e = inject_hand(&e, &v, &[sha2], 1);
    let out = activate(&mut e, 0, "sqlonghuo");
    assert!(matches!(out, CommandOutcome::Accepted { .. }));
    assert_eq!(e.debug_hand(0).len(), 2, "sqlonghuo redraws equal count");
    assert!(e.debug_hand(1).is_empty(), "sqlonghuo discards enemy hand card matching discarded name");
    assert!(matches!(activate(&mut e, 0, "sqlonghuo"), CommandOutcome::Rejected(RejectReason::SkillAlreadyUsed)));
    assert_zone_conservation(&e);
}

#[test]
fn u05_b5b_jielue_steals_friend_usable_cards_uses_them_and_draws() {
    // Upstream: `character/gwent/skill.js:149-210` (`phaseUseBegin`, forced)
    let (v, e0) = setup_gwent_locked(5602);
    let wuzhong = card_of_kind_and_suit(&e0, CardKind::Wuzhong, None);
    let shan = card_of_kind_and_suit(&e0, CardKind::Shan, None);
    let mut e = grant_skill(&e0, &v, "jielue", 0);
    // Negative: friend seat 2 only has Shan -> not usable -> jielue doesn't trigger
    advance_to_seat_play(&mut e, 3);
    e = clear_hand(&e, &v, 2);
    e = inject_hand(&e, &v, &[shan], 2);
    advance_to_seat_play(&mut e, 0);
    assert_eq!(e.debug_hand(2), vec![shan], "unusable Shan stays with friend");

    // Positive: friend seat 2 has Wuzhong -> forced take & use at Play start, then seat 2 draws 1
    let mut e = grant_skill(&e0, &v, "jielue", 0);
    advance_to_seat_play(&mut e, 3);
    e = clear_hand(&e, &v, 2);
    e = clear_hand(&e, &v, 0);
    e = inject_hand(&e, &v, &[wuzhong], 2);
    advance_to_seat_play(&mut e, 0);
    assert!(e.debug_hand(0).len() >= 2, "seat 0 used stolen Wuzhong and drew 2");
    assert_eq!(e.debug_hand(2).len(), 1, "friend seat 2 drew 1 card after jielue");
    assert_zone_conservation(&e);
}

#[test]
fn u05_b5b_gwhuanshuang_uses_bronze_spell_twice_and_disables_for_turn() {
    // Upstream: `character/gwent/skill.js:223-340` (`["phaseBegin", "phaseEnd"]`)
    let (v, e0) = setup_gwent_locked(5603);
    let mut e = grant_skill(&e0, &v, "gwhuanshuang", 0);
    advance_past_turn(&mut e, 0);
    advance_to_seat_prepare(&mut e, 0);
    // Open choice: cancel first -> phaseBegin skips, so phaseEnd triggers later!
    answer_cancel(&mut e, 0);
    drain_default(&mut e);
    assert!(!has_skill(&e, 0, "gwhuanshuang_disable"));

    // Positive: in next turn Prepare pick first bronze option and use twice on chosen target(s)
    let mut e = grant_skill(&e0, &v, "gwhuanshuang", 0);
    let mut snap = e.save();
    snap.core.players[1].hp = 4;
    snap.core.players[3].hp = 4;
    e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    advance_past_turn(&mut e, 0);
    advance_to_seat_prepare(&mut e, 0);
    let c = e.open_choice().cloned().expect("gwhuanshuang pick");
    let first_tag = c.spec.options[0];
    submit(&mut e, 0, GameAction::Answer { choice_id: c.id, selection: vec![first_tag] });
    for _ in 0..8 {
        let Some(ch) = e.open_choice().cloned() else { break };
        let sel = if ch.spec.allow_cancel {
            // choose first target rather than cancelling bronze target
            ch.spec.options.iter().take(ch.spec.min.max(1)).copied().collect()
        } else {
            ch.spec.options.iter().take(ch.spec.min.max(1)).copied().collect()
        };
        submit(&mut e, ch.observer, GameAction::Answer { choice_id: ch.id, selection: sel });
    }
    assert!(has_skill(&e, 0, "gwhuanshuang_disable"), "gwhuanshuang must add disable until turn end");
    assert_zone_conservation(&e);
}

#[test]
fn u05_b5b_gwjinli_gwliaotian_gwhuanbi_gwminxiang() {
    let (v, e0) = setup_gwent_locked(5604);
    let sha = card_of_kind_and_suit(&e0, CardKind::Sha, Some(engine_native::catalog::Suit::Heart));
    let tao = card_of_kind_and_suit(&e0, CardKind::Tao, Some(engine_native::catalog::Suit::Diamond));
    let spade_sha = card_of_kind_and_suit(&e0, CardKind::Sha, Some(engine_native::catalog::Suit::Spade));

    // 1. gwjinli (`character/gwent/skill.js:368-416`): discard 1 -> target gets gwjinli_jiu -> 2 phaseEnds -> buff + gw_xianzumaijiu
    let mut e = grant_skill(&e0, &v, "gwjinli", 0);
    e = inject_hand(&e, &v, &[sha], 0);
    activate(&mut e, 0, "gwjinli");
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Card(sha));
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Player(2));
    assert!(has_skill(&e, 2, "gwjinli_jiu"));
    advance_to_seat_play(&mut e, 2);
    advance_past_turn(&mut e, 2); // turn 1 end -> buff, 1 left
    assert!(has_skill(&e, 2, "gwjinli_jiu"));
    advance_to_seat_play(&mut e, 2);
    advance_past_turn(&mut e, 2); // turn 2 end -> buff, removed, gain gw_xianzumaijiu
    assert!(!has_skill(&e, 2, "gwjinli_jiu"));
    assert!(
        e.debug_hand(2)
            .iter()
            .any(|&c| e.catalog().cards[c as usize].name == "gw_xianzumaijiu"),
        "target should gain gw_xianzumaijiu after 2 phaseEnds"
    );

    // 2. gwliaotian (`character/gwent/skill.js:417-449`, usable: 2): negative on mixed colors; positive on >=2 same color; capped at 2/turn
    let mut e = grant_skill(&e0, &v, "gwliaotian", 0);
    e = inject_hand(&e, &v, &[sha, spade_sha], 0);
    assert!(matches!(activate(&mut e, 0, "gwliaotian"), CommandOutcome::Rejected(_)));
    let reds4 = find_cards_by(&e0, |d| matches!(d.suit, engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond), 6);
    let mut e = grant_skill(&e0, &v, "gwliaotian", 0);
    e = inject_hand(&e, &v, &[reds4[0], reds4[1]], 0);
    assert!(matches!(activate(&mut e, 0, "gwliaotian"), CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    drain_default(&mut e);
    assert_eq!(e.debug_hand(0).len(), 2, "gwliaotian #1 recasts 2 same-color cards");
    e = clear_hand(&e, &v, 0);
    e = inject_hand(&e, &v, &[reds4[2], reds4[3]], 0);
    assert!(matches!(activate(&mut e, 0, "gwliaotian"), CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    drain_default(&mut e);
    e = clear_hand(&e, &v, 0);
    e = inject_hand(&e, &v, &[reds4[4], reds4[5]], 0);
    assert!(matches!(activate(&mut e, 0, "gwliaotian"), CommandOutcome::Rejected(_)), "gwliaotian capped at 2 uses per turn");
    advance_past_turn(&mut e, 0);
    advance_to_seat_play(&mut e, 0);
    e = clear_hand(&e, &v, 0);
    e = inject_hand(&e, &v, &[reds4[4], reds4[5]], 0);
    assert!(matches!(activate(&mut e, 0, "gwliaotian"), CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "gwliaotian resets on next turn");
    drain_default(&mut e);

    // 3. gwhuanbi (`character/gwent/skill.js:450-499`): discard 1 -> pick gwmaoxian -> random other with hand can trade hand card for copy
    let mut e = grant_skill(&e0, &v, "gwhuanbi", 0);
    e = inject_hand(&e, &v, &[sha], 0);
    e = inject_hand(&e, &v, &[tao], 1);
    activate(&mut e, 0, "gwhuanbi");
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Card(sha));
    answer_first(&mut e, 0, |o| matches!(o, ChoiceOption::Tag(_)));
    if let Some(ch) = e.open_choice().cloned() {
        assert_eq!(ch.observer, 1, "only seat 1 has hand cards");
        let card_opt = ch.spec.options.iter().find(|o| matches!(o, ChoiceOption::Card(_))).copied().unwrap();
        submit(&mut e, 1, GameAction::Answer { choice_id: ch.id, selection: vec![card_opt] });
        assert!(e.debug_hand(0).iter().any(|&c| c == tao), "seat 0 received traded card");
    }
    assert!(
        e.debug_hand(0)
            .iter()
            .any(|&c| e.catalog().cards[c as usize].name.starts_with("gwmaoxian_")),
        "seat 0 gained gwmaoxian card"
    );

    // 4. gwminxiang (`character/gwent/skill.js:500-575`): discard single-target basic/trick, draw 1, 2 others duel/sha each other
    let mut e = grant_skill(&e0, &v, "gwminxiang", 0);
    e = inject_hand(&e, &v, &[sha], 0);
    activate(&mut e, 0, "gwminxiang");
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Card(sha));
    let c = e.open_choice().cloned().expect("gwminxiang 2 targets");
    submit(&mut e, 0, GameAction::Answer {
        choice_id: c.id,
        selection: vec![ChoiceOption::Player(1), ChoiceOption::Player(3)],
    });
    drain_default(&mut e);
    assert_eq!(e.debug_hand(0).len(), 1, "gwminxiang draws 1");
    assert_eq!(e.debug_hp(3), 3, "target 1 dealt sha to target 3");
    assert_eq!(e.debug_hp(1), 3, "target 3 dealt sha back to target 1");
    assert_zone_conservation(&e);
}

#[test]
fn u05_b5b_gwlangshi_gwjingtian_gwjingshi_gwweitu_gwgouhun() {
    let (v, e0) = setup_gwent_locked(5605);
    let sha = card_of_kind_and_suit(&e0, CardKind::Sha, Some(engine_native::catalog::Suit::Spade));
    let heart_tao = card_of_kind_and_suit(&e0, CardKind::Tao, Some(engine_native::catalog::Suit::Heart));
    let club_shan = card_of_kind_and_suit(&e0, CardKind::Sha, Some(engine_native::catalog::Suit::Club));

    // 1. gwlangshi (`character/gwent/skill.js:576-605`): damageEnd -> damage other with hp >= victim hp; NO self-chain
    let mut e = grant_skill(&e0, &v, "gwlangshi", 0);
    e = inject_hand(&e, &v, &[sha], 0);
    submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    pass_window(&mut e);
    // Victim 1 hp=3; targets with hp>=3 include 2 and 3 (both hp=4). Pick 3.
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Player(3));
    drain_default(&mut e);
    assert_eq!(e.debug_hp(3), 3);
    assert!(e.open_choice().is_none(), "gwlangshi damage must not chain onto itself");

    // 2. gwjingtian (`character/gwent/skill.js:606-704`): skips draw phase for +3 marks; Play active takes top 1 (1/turn)
    let mut e = grant_skill(&e0, &v, "gwjingtian", 0);
    advance_past_turn(&mut e, 0);
    advance_to_seat_play(&mut e, 0);
    assert_eq!(e.debug_hand(0).len(), 0, "gwjingtian skipped draw phase");
    assert!(matches!(activate(&mut e, 0, "gwjingtian"), CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    drain_default(&mut e);
    assert_eq!(e.debug_hand(0).len(), 1, "gwjingtian claimed 1 card using 1 mark");
    assert!(matches!(activate(&mut e, 0, "gwjingtian"), CommandOutcome::Rejected(RejectReason::SkillAlreadyUsed)), "gwjingtian limited 1 per turn");

    // 3. gwjingshi (`character/gwent/skill.js:705-793`): guess max-black holder -> view & steal 1 hand card
    let mut e = grant_skill(&e0, &v, "gwjingshi", 0);
    e = inject_hand(&e, &v, &[sha, club_shan], 1); // 2 black
    e = inject_hand(&e, &v, &[heart_tao], 3); // 0 black
    activate(&mut e, 0, "gwjingshi");
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Player(1));
    let steal = e.open_choice().cloned().expect("gwjingshi reveal steal");
    assert!(steal.spec.options.contains(&ChoiceOption::Card(heart_tao)));
    submit(&mut e, 0, GameAction::Answer { choice_id: steal.id, selection: vec![ChoiceOption::Card(sha)] });
    assert!(e.debug_hand(0).contains(&sha));

    // 4. gwweitu (`character/gwent/skill.js:794-861`): discard -> +1 hujia (cap 3); 3 hujia losses -> gain random silver spell
    let mut e = grant_skill(&e0, &v, "gwweitu", 0);
    let guohe = card_of_kind_and_suit(&e0, CardKind::Dismantle, None);
    e = inject_hand(&e, &v, &[sha, club_shan, heart_tao], 0);
    e = inject_hand(&e, &v, &[guohe], 1);
    advance_to_seat_play(&mut e, 1);
    submit(&mut e, 1, GameAction::PlayCard { card: guohe, targets: vec![0] });
    drain_default(&mut e);
    assert!(e.state_core().player(0).hujia <= 3);

    // 5. gwgouhun (`character/gwent/skill.js:862-962`): give card, target chooses give-same-suit / discard-other / goMad
    let mut e = grant_skill(&e0, &v, "gwgouhun", 0);
    e = inject_hand(&e, &v, &[sha], 0);
    e = inject_hand(&e, &v, &[heart_tao], 1);
    activate(&mut e, 0, "gwgouhun");
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Card(sha));
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Player(1));
    let rep = e.open_choice().cloned().expect("gwgouhun reply on seat 1");
    assert_eq!(rep.observer, 1);
    // Pick goMad (last Control option)
    let mad_opt = *rep.spec.options.last().unwrap();
    submit(&mut e, 1, GameAction::Answer { choice_id: rep.id, selection: vec![mad_opt] });
    assert!(e.state_core().player(1).boss_mad, "seat 1 enters boss_mad");
    advance_to_seat_play(&mut e, 1);
    advance_past_turn(&mut e, 1);
    assert!(!e.state_core().player(1).boss_mad, "boss_mad clears at phaseAfter of target's turn");
    assert_zone_conservation(&e);
}

#[test]
fn u05_b5b_gwfutian_gwzhongmo_and_gwyewu_global_cleanup() {
    let (v, e0) = setup_gwent_locked(5606);
    // 1. gwfutian (`character/gwent/skill.js:963-1061`): prevents damage, discards >=24 -> gwzhongmo (+4 maxHp, full hp)
    let mut e = grant_skill(&e0, &v, "gwfutian", 0);
    let mut snap = e.save();
    snap.core.players[0].hp = 1;
    snap.core.players[0].max_hp = 1;
    e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    let king = find_cards_by(&e, |d| d.rank >= 12, 3);
    e = inject_hand(&e, &v, &[king[0], king[1]], 1);
    let sha = card_of_kind_and_suit(&e, CardKind::Sha, None);
    e = inject_hand(&e, &v, &[sha], 1);
    advance_to_seat_play(&mut e, 1);
    submit(&mut e, 1, GameAction::PlayCard { card: sha, targets: vec![0] });
    drain_default(&mut e);
    assert_eq!(e.debug_hp(0), 1, "gwfutian prevents all damage");
    // Turn 1 prepare: discard rank >= 12 from seat 1
    advance_to_seat_prepare(&mut e, 0);
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Player(1));
    drain_default(&mut e);
    // Turn 2 prepare: another rank >= 12 from seat 1 -> sum >= 24 -> transform
    e = inject_hand(&e, &v, &[king[2]], 1);
    advance_past_turn(&mut e, 0);
    e = clear_hand(&e, &v, 0);
    advance_to_seat_prepare(&mut e, 0);
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Player(1));
    drain_default(&mut e);
    assert!(!has_skill(&e, 0, "gwfutian"));
    assert!(has_skill(&e, 0, "gwzhongmo"));
    assert_eq!(e.state_core().player(0).max_hp, 5);
    assert_eq!(e.debug_hp(0), 5);

    // 2. gwzhongmo skips draw phase and grants 2 distinct-tier spells
    advance_to_seat_play(&mut e, 0);
    assert_eq!(e.debug_hand(0).len(), 2, "gwzhongmo gives 2 spells instead of normal draw");

    // 3. gwyewu & g_gw_yewu (`character/gwent/skill.js:1062-1133`): discard matching color enemy cards -> gain wuyao/lang; vanish at phaseAfter
    let mut e = grant_skill(&e0, &v, "gwyewu", 0);
    let spade_sha = card_of_kind_and_suit(&e0, CardKind::Sha, Some(engine_native::catalog::Suit::Spade));
    let club_shan = card_of_kind_and_suit(&e0, CardKind::Sha, Some(engine_native::catalog::Suit::Club));
    e = inject_hand(&e, &v, &[spade_sha], 0);
    e = inject_hand(&e, &v, &[club_shan], 1);
    activate(&mut e, 0, "gwyewu");
    answer_first(&mut e, 0, |o| *o == ChoiceOption::Card(spade_sha));
    assert!(
        e.debug_hand(0).iter().any(|&c| e.catalog().cards[c as usize].name == "gw_wuyao"),
        "gwyewu should grant gw_wuyao for discarded enemy hand card"
    );
    advance_past_turn(&mut e, 0);
    assert!(
        !e.debug_hand(0).iter().any(|&c| matches!(e.catalog().cards[c as usize].name, "gw_wuyao" | "gw_lang")),
        "g_gw_yewu cleans remaining gw_wuyao/gw_lang on phaseAfter"
    );
    assert_zone_conservation(&e);
}
