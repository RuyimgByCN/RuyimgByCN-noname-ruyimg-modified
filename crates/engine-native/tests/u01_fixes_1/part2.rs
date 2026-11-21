use super::common::*;

#[test]
fn u01_fix_06_dddfuzong() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:5948-5987
    let (mut e, d3, p9) = new_clean_engine(91006, 4);
    let liuhong = find_general(&e, "ddd_liuhong"); // zhuSkill: true `dddfuzong` (Qun lord)
    let qun_g = (0..e.catalog().general_count() as u16)
        .find(|&idx| {
            idx != liuhong
                && e.catalog()
                    .general_meta(idx)
                    .is_some_and(|m| m.1 == engine_native::catalog::Faction::Qun)
        })
        .unwrap();
    e.assign_general(0, liuhong).unwrap();
    e.assign_general(1, qun_g).unwrap();

    // Negative guard: Rebel at seat 1 in Identity mode must NOT trigger either Qun draw or death punish.
    {
        let mut neg_e = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
        neg_e.state_core_mut().skills.clear();
        neg_e.assign_general(1, liuhong).unwrap();
        neg_e.assign_general(0, qun_g).unwrap();
        let mut snap = neg_e.save();
        snap.core.mode = GameMode::Identity {
            roles: vec![Role::Lord, Role::Rebel, Role::Loyalist, Role::Traitor],
        };
        let mut neg = Engine::restore_with_extensions(&snap, &[&d3, &p9]).unwrap();
        let c1 = find_card_by(&neg, |_, _| true);
        put_in_hand(&mut neg, 1, c1);
        let _ = neg.debug_deal_damage(None, 0, 10, DamageNature::Normal);
        drain(&mut neg);
        assert!(neg.state_core().player(1).hand.contains(&c1));
    }

    // Lord with 0 Qun alive: draws EXACTLY 0 cards (NOT `.max(1)`!).
    {
        let mut snap = e.save();
        snap.core.mode = GameMode::Identity {
            roles: vec![Role::Lord, Role::Rebel, Role::Loyalist, Role::Traitor],
        };
        snap.core.players[0].general_slots[0] = None;
        snap.core.players[0].revealed_generals[0] = false;
        snap.core.players[1].general_slots[0] = None;
        snap.core.players[1].revealed_generals[0] = false;
        snap.core.skills.clear();
        let mut zero_qun = Engine::restore_with_extensions(&snap, &[&d3, &p9]).unwrap();
        zero_qun.debug_grant(0, "dddfuzong");
        zero_qun.debug_set_turn(3, Phase::End, 0);
        let _ = submit(&mut zero_qun, 3, GameAction::NextPhase);
        assert_eq!(zero_qun.state_core().player(0).hand.len(), 0, "0 Qun alive must draw 0 cards");
    }

    // Lord with 2 Qun alive (liuhong + qun_g): draws 2 on round 1 start (`TurnBegin` -> `start_draw(0, 2)`), and dying Qun seat 1 chooses 1 owner `he` card!
    let mut snap = e.save();
    snap.core.mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Rebel, Role::Loyalist, Role::Traitor],
    };
    e = Engine::restore_with_extensions(&snap, &[&d3, &p9]).unwrap();
    {
        let mut draw_probe = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
        draw_probe.debug_set_turn(3, Phase::End, 0);
        let h0_before = draw_probe.state_core().player(0).hand.len();
        let _ = submit(&mut draw_probe, 3, GameAction::NextPhase);
        drain(&mut draw_probe);
        assert_eq!(
            draw_probe.state_core().player(0).hand.len(),
            h0_before + 2,
            "dddfuzong Lord with 2 Qun alive must draw 2 on round 1 start"
        );
    }
    let owner_eq = find_card_by(&e, |_, d| d.kind.is_equip());
    let owner_hand = find_card_by(&e, |id, _| id != owner_eq);
    put_in_hand(&mut e, 0, owner_hand);
    put_in_equip(&mut e, 0, owner_eq);
    let _ = e.debug_deal_damage(None, 1, 10, DamageNature::Normal);
    let c = e.open_choice().expect("dddfuzong_discard").clone();
    assert_eq!((c.observer, c.spec.prompt), (1, "dddfuzong_discard"));
    assert!(!c.spec.allow_cancel);
    assert_eq!(
        c.spec.options,
        vec![ChoiceOption::HiddenCard(0), ChoiceOption::Card(owner_eq)]
    );
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::HiddenCard(0)]);
    answer(&mut e, 1, vec![ChoiceOption::HiddenCard(0)]);
    assert!(!e.state_core().player(0).hand.contains(&owner_hand));
}

#[test]
fn u01_fix_07_dddzhishu() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:37-58
    let (mut e, d3, p9) = new_clean_engine(91007, 4);
    let qianzhao = find_general(&e, "ddd_qianzhao");
    e.assign_general(0, qianzhao).unwrap();
    e.debug_set_turn(0, Phase::Play, 1);
    let _ = qianzhao;

    // Only a delayed trick (`Le`) is in seat 1's judge zone; 0 equips anywhere!
    let le = find_card_by(&e, |_, d| d.kind == CardKind::Le);
    let victim_card = find_card_by(&e, |id, _| id != le);
    put_in_judge(&mut e, 1, le);
    put_in_hand(&mut e, 2, victim_card);

    e.debug_set_turn(0, Phase::Draw, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let c_card = e.open_choice().expect("dddzhishu_card must include judge card").clone();
    assert_eq!(c_card.spec.prompt, "dddzhishu_card");
    assert!(c_card.spec.options.contains(&ChoiceOption::Card(le)));
    assert!(c_card.spec.allow_cancel);
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Card(le)]);
    answer(&mut e, 0, vec![ChoiceOption::Card(le)]);

    let c_tgt = e.open_choice().expect("dddzhishu_target").clone();
    assert_eq!(c_tgt.spec.prompt, "dddzhishu_target");
    assert!(!c_tgt.spec.options.contains(&ChoiceOption::Player(1)));
    assert!(c_tgt.spec.options.contains(&ChoiceOption::Player(2)));
    answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
    drain(&mut e);

    // Moved judge `Le` to seat 2 AND seat 1 virtually dismantled seat 2 (discarding victim_card or Le)!
    assert!(e.state_core().player(1).judge.is_empty());
    assert!(
        e.log().iter().any(|ev| matches!(
            ev,
            LogEvent::VirtualCardPlayed { kind: CardKind::Dismantle, player: 1, targets, .. } if targets == &vec![2]
        )),
        "dddzhishu must emit virtual Dismantle VirtualCardPlayed from src to dst"
    );
}

#[test]
fn u01_fix_08_dddxiaxing() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:59-243
    let (mut e, d3, p9) = new_clean_engine(91008, 4);
    let zhangmiao = find_general(&e, "ddd_zhangmiao");
    let zhuge = find_general(&e, "诸葛亮");
    e.assign_general(0, zhangmiao).unwrap();
    e.assign_general(1, zhuge).unwrap(); // seat 1 has Kongcheng & 0 hand cards
    let xx_def = e.catalog().skill_by_name("dddxiaxing").unwrap();

    // Grant `dddxiaxing_xia = 1` cleanly via Prepare trigger first!
    let top_card = find_card_by(&e, |_, d| d.kind == CardKind::Tao);
    put_in_hand(&mut e, 0, top_card);
    e.debug_set_turn(3, Phase::End, 1);
    let _ = submit(&mut e, 3, GameAction::NextPhase);
    let xx_c = e.open_choice().expect("dddxiaxing_card").clone();
    assert_eq!(xx_c.spec.prompt, "dddxiaxing_card");
    answer(&mut e, 0, vec![ChoiceOption::Card(top_card)]);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "dddxiaxing_xia"), 1);

    // Defensive `Shan` (`dddxiaxing_xia == 1`, 0 physical `Shan` in hand):
    {
        let mut def_e = Engine::restore_with_extensions(&e.save(), &[&d3, &p9]).unwrap();
        def_e.debug_set_turn(3, Phase::Play, 2);
        let sha3 = find_card_by(&def_e, |id, d| id != top_card && d.kind == CardKind::Sha);
        put_in_hand(&mut def_e, 3, sha3);
        let hp0 = def_e.debug_hp(0);
        let _ = submit(
            &mut def_e,
            3,
            GameAction::PlayCard {
                card: sha3,
                targets: vec![0],
            },
        );
        let resp = def_e.open_choice().expect("respond_shan with dddxiaxing").clone();
        assert_eq!((resp.observer, resp.spec.prompt), (0, "respond_shan"));
        assert!(
            resp.spec.options.contains(&ChoiceOption::Skill(xx_def)),
            "dddxiaxing_xia must offer Skill(dddxiaxing) as Shan in respond_shan"
        );
        assert_mid_choice_archive_and_ai(&def_e, &d3, &p9, vec![ChoiceOption::Skill(xx_def)]);
        answer(&mut def_e, 0, vec![ChoiceOption::Skill(xx_def)]);
        drain(&mut def_e);
        assert_eq!(def_e.debug_hp(0), hp0);
        assert_eq!(flag(&def_e, 0, "dddxiaxing_xia"), 0);
    }

    // Play Sha via `ActivateSkill(dddxiaxing)`: must respect range & Kongcheng (seat 2 dist 2 & seat 1 Kongcheng excluded; only seat 3 in range!).
    e.debug_set_turn(0, Phase::Play, 2);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: xx_def });
    let tgt_c = e.open_choice().expect("zhangba_sha_target").clone();
    assert!(!tgt_c.spec.options.contains(&ChoiceOption::Player(2)), "dist 2 out of range 1");
    assert!(!tgt_c.spec.options.contains(&ChoiceOption::Player(1)), "Kongcheng must be excluded");
    assert!(tgt_c.spec.options.contains(&ChoiceOption::Player(3)));
    answer(&mut e, 0, vec![ChoiceOption::Player(3)]);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "dddxiaxing_xia"), 0);
}

#[test]
fn u01_fix_09_dddjungui() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:384-441
    let (mut e, d3, p9) = new_clean_engine(91009, 3);
    let zhangcheng = find_general(&e, "ddd_zhangcheng");
    e.assign_general(0, zhangcheng).unwrap();

    let heart_a = find_card_by(&e, |_, d| d.suit == Suit::Heart && d.kind == CardKind::Tao);
    let heart_b = find_card_by(&e, |id, d| id != heart_a && d.suit == Suit::Heart);
    let heart_c = find_card_by(&e, |id, d| id != heart_a && id != heart_b && d.suit == Suit::Heart);
    let spade_a = find_card_by(&e, |_, d| d.suit == Suit::Spade);
    let club_a = find_card_by(&e, |_, d| d.suit == Suit::Club);
    let club_b = find_card_by(&e, |id, d| id != club_a && d.suit == Suit::Club);
    put_in_hand(&mut e, 0, club_a);
    put_in_hand(&mut e, 1, heart_a);
    put_in_hand(&mut e, 1, heart_b);
    put_in_hand(&mut e, 1, spade_a);
    put_deck_top(&mut e, club_b);
    put_deck_top(&mut e, heart_c);

    // Owner uses 0 cards this turn -> usedCount == 0 -> target draws 2 (heart_c, club_b) + reveals hand -> skips `dddjungui_reduce` -> 1 batch discard of 3 distinct suits!
    e.debug_set_turn(0, Phase::Discard, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let tgt = e.open_choice().expect("dddjungui_target").clone();
    assert!(tgt.spec.options.contains(&ChoiceOption::Player(0)));
    assert!(tgt.spec.options.contains(&ChoiceOption::Player(1)));
    assert!(tgt.spec.options.contains(&ChoiceOption::Player(2)));
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);

    let d1 = e.open_choice().expect("dddjungui_discard batch").clone();
    assert_eq!((d1.observer, d1.spec.prompt), (1, "dddjungui_discard"));
    assert_eq!((d1.spec.min, d1.spec.max), (3, 3));
    assert!(!d1.spec.allow_cancel);
    // Reject picking duplicate suit (heart_a + heart_b + spade_a)!
    assert!(matches!(
        submit(
            &mut e,
            1,
            GameAction::Answer {
                choice_id: d1.id,
                selection: vec![
                    ChoiceOption::Card(heart_a),
                    ChoiceOption::Card(heart_b),
                    ChoiceOption::Card(spade_a),
                ],
            },
        ),
        CommandOutcome::Rejected(_)
    ));
    assert_mid_choice_archive_and_ai(
        &e,
        &d3,
        &p9,
        vec![
            ChoiceOption::Card(heart_a),
            ChoiceOption::Card(spade_a),
            ChoiceOption::Card(club_b),
        ],
    );
    answer(
        &mut e,
        1,
        vec![
            ChoiceOption::Card(heart_a),
            ChoiceOption::Card(spade_a),
            ChoiceOption::Card(club_b),
        ],
    );
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hand, vec![heart_b, heart_c]);
}

#[test]
fn u01_fix_10_dddjinggou() {
    // upstream archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:443-471
    let (mut e, d3, p9) = new_clean_engine(91010, 3);
    let liuchong = find_general(&e, "ddd_liuchong");
    e.assign_general(0, liuchong).unwrap();
    let w3 = find_card_by(&e, |_, d| d.kind == CardKind::Weapon && d.range == 3);
    let w2 = find_card_by(&e, |_, d| d.kind == CardKind::Weapon && d.range == 2);

    // Negative guard: putting/moving weapon directly into equips (not via PlayCard useCardAfter) MUST NOT trigger dddjinggou!
    e.debug_set_turn(1, Phase::Play, 1);
    put_in_equip(&mut e, 0, w3);
    assert!(e.open_choice().is_none(), "direct equip must not trigger dddjinggou");
    clear_equips(&mut e, 0);

    // Negative guard: playing w2 via PlayCard when seat 1 has w3 (< max range) does NOT trigger!
    e.debug_set_turn(0, Phase::Play, 2);
    put_in_equip(&mut e, 1, w3);
    put_in_hand(&mut e, 0, w2);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: w2,
            targets: Vec::new(),
        },
    );
    assert!(e.open_choice().is_none());

    // Positive: playing Qilin (CardKind::Qilin, EquipSlot::Weapon, range 5 >= max range 5) triggers `d3_jinggou_target` and deals 1 normal damage!
    let qilin = find_card_by(&e, |_, d| d.kind == CardKind::Qilin);
    clear_equips(&mut e, 1);
    put_in_hand(&mut e, 0, qilin);
    let log_start = e.log().len();
    let hp1 = e.debug_hp(1);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: qilin,
            targets: Vec::new(),
        },
    );
    let c = e.open_choice().expect("d3_jinggou_target").clone();
    assert_eq!(c.spec.prompt, "d3_jinggou_target");
    assert!(!c.spec.allow_cancel);
    assert_mid_choice_archive_and_ai(&e, &d3, &p9, vec![ChoiceOption::Player(1)]);
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1 - 1);
    assert!(
        e.log()[log_start..].iter().any(|ev| matches!(
            ev,
            LogEvent::SkillTriggered { skill, owner: 0, .. } if *skill == "dddjinggou"
        )),
        "dddjinggou must emit SkillTriggered log"
    );
}
