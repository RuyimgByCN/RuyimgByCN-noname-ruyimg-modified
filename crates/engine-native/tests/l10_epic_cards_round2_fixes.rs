//! L10 史诗卡牌 Round 2 对偶/伴生技与 AI/限制回归测试。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use content_native_loader::bridge_pack;
use engine_native::ai::BoundedPlanner;
use engine_native::catalog::{CardKind, Suit};
use engine_native::event::DamageNature;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};

static DEDUP: AtomicU64 = AtomicU64::new(500_000);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn repo_extensions_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn epic_pack() -> engine_native::content::extension::ExtensionPack {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("epic-cards-upstream-liuli/assets/cards.json"),
    )
    .expect("read epic cards.json");
    bridge_pack(&json).expect("bridge epic").pack
}

fn epic_engine(seed: u64) -> Engine {
    Engine::new_with_extensions(seed, 4, &[&epic_pack()]).expect("new epic engine")
}

fn ans(e: &mut Engine, actor: u8, selection: Vec<ChoiceOption>) -> CommandOutcome {
    let id = e.open_choice().expect("open choice for answer").id;
    submit(e, actor, GameAction::Answer { choice_id: id, selection })
}

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action,
    })
}

fn drain(e: &mut Engine) {
    for _ in 0..96 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.prompt == "reguhuo_guess_pick" {
            vec![ChoiceOption::Tag(0)]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = submit(
            e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        );
    }
}

fn advance_to_play(e: &mut Engine, player: u8) {
    for _ in 0..96 {
        if e.turn().current == player && e.turn().phase == Phase::Play && e.open_choice().is_none()
        {
            return;
        }
        if e.open_choice().is_some() {
            drain(e);
            continue;
        }
        let cur = e.turn().current;
        let _ = submit(e, cur, GameAction::NextPhase);
    }
    panic!("failed to reach player {player} Play phase");
}

fn roundtrip_and_answer(
    e: &mut Engine,
    pack: &[&engine_native::content::extension::ExtensionPack],
    selection: Vec<ChoiceOption>,
) {
    let c = e.open_choice().cloned().expect("pending choice for roundtrip");
    let raw = engine_native::archive::encode_archive(&e.save());
    let decoded = engine_native::archive::decode_archive(&raw).expect("decode archive");
    let mut restored =
        Engine::restore_with_extensions(&decoded, pack).expect("restore archive with extensions");
    let out_a = submit(
        e,
        c.observer,
        GameAction::Answer {
            choice_id: c.id,
            selection: selection.clone(),
        },
    );
    let out_b = submit(
        &mut restored,
        c.observer,
        GameAction::Answer {
            choice_id: c.id,
            selection,
        },
    );
    assert!(
        !matches!(out_a, CommandOutcome::Rejected(_)),
        "expected answer accepted on {}: {out_a:?}",
        c.spec.prompt
    );
    assert_eq!(
        std::mem::discriminant(&out_a),
        std::mem::discriminant(&out_b)
    );
    assert_eq!(e.state_hash(), restored.state_hash());
}

fn assert_ai_no_leak(
    e: &Engine,
    pack: &[&engine_native::content::extension::ExtensionPack],
) -> Vec<ChoiceOption> {
    let c = e.open_choice().expect("open choice for AI");
    let obs = c.observer;
    let planner = BoundedPlanner::new(16);
    let a = e.plan_action(obs, &planner);
    let GameAction::Answer {
        choice_id,
        selection: sel_a,
    } = a.action
    else {
        panic!("expected AI Answer");
    };
    assert_eq!(choice_id, c.id);

    let mut snap_b = e.save();
    snap_b.core.deck.reverse();
    let e_b = Engine::restore_with_extensions(&snap_b, pack).expect("restore hidden flip");
    let b = e_b.plan_action(obs, &planner);
    let GameAction::Answer {
        selection: sel_b, ..
    } = b.action
    else {
        panic!("expected AI Answer after hidden flip");
    };
    assert_eq!(
        sel_a, sel_b,
        "AI choice on {} must not depend on hidden deck/hand info",
        c.spec.prompt
    );
    sel_a
}

fn drain_guess_skip_non_owner(e: &mut Engine) {
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if c.spec.prompt != "reguhuo_guess_pick" {
            break;
        }
        let _ = submit(
            e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: vec![ChoiceOption::Tag(0)],
            },
        );
    }
}


#[test]
fn l10_rangjie_direct_and_ai_move_benefit() {
    let pack = epic_pack();
    // 1. Direct prompt (`optional: false`) & AI prefers Tag(0) when no beneficial field move exists
    let mut e = epic_engine(10_4001);
    e.debug_grant(0, "rangjie");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    advance_to_play(&mut e, 0);
    let own_wp = e.debug_place(0, "zhuge");
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: own_wp,
            targets: Vec::new(),
        },
    );
    drain(&mut e);
    let _ = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    let ch = e.open_choice().cloned().expect("direct rangjie_choice");
    assert_eq!(ch.spec.prompt, "rangjie_choice", "direct: true must skip trigger_confirm");
    assert!(ch.spec.options.contains(&ChoiceOption::Tag(1)));
    let ai_sel = assert_ai_no_leak(&e, &[&pack]);
    assert_eq!(ai_sel, vec![ChoiceOption::Tag(0)], "AI must not blindly choose Tag(1) when only self owns equip");
    drain(&mut e);

    // 2. When enemy (seat 1) holds equip, AI picks Tag(1) -> rangjie_move_card -> rangjie_move_target cleanly
    let mut e2 = epic_engine(10_4002);
    e2.debug_grant(0, "rangjie");
    advance_to_play(&mut e2, 1);
    let enemy_wp = e2.debug_place(1, "zhuge");
    let _ = submit(
        &mut e2,
        1,
        GameAction::PlayCard {
            card: enemy_wp,
            targets: Vec::new(),
        },
    );
    drain(&mut e2);
    let _ = e2.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    let sel_choice = assert_ai_no_leak(&e2, &[&pack]);
    assert_eq!(sel_choice, vec![ChoiceOption::Tag(1)]);
    roundtrip_and_answer(&mut e2, &[&pack], sel_choice);
    let mc = e2.open_choice().cloned().expect("rangjie_move_card");
    assert_eq!(mc.spec.prompt, "rangjie_move_card");
    let sel_card = assert_ai_no_leak(&e2, &[&pack]);
    assert_eq!(sel_card, vec![ChoiceOption::Card(enemy_wp)]);
    roundtrip_and_answer(&mut e2, &[&pack], sel_card);
    let mt = e2.open_choice().cloned().expect("rangjie_move_target");
    assert_eq!(mt.spec.prompt, "rangjie_move_target");
    let sel_tgt = assert_ai_no_leak(&e2, &[&pack]);
    assert_eq!(sel_tgt, vec![ChoiceOption::Player(0)]);
    roundtrip_and_answer(&mut e2, &[&pack], sel_tgt);
    assert!(e2.state_core().player(0).equips.contains(&enemy_wp));
}

#[test]
fn l10_reguhuo_scstaoluan_jiedao_non20_tricks_and_responses() {
    let pack = epic_pack();
    // 1. `Jiedao` 2-target (`holder -> victim`) + cancel victim preserves once, on both `reguhuo` & `scstaoluan`
    let mut e = epic_engine(10_4101);
    e.debug_grant(0, "reguhuo");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let cost = e.debug_place(0, "tao");
    let wp = e.debug_place(1, "zhuge");
    advance_to_play(&mut e, 1);
    let _ = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: wp,
            targets: Vec::new(),
        },
    );
    advance_to_play(&mut e, 0);
    let guhuo = e.catalog().skill_by_name("reguhuo").unwrap();
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: guhuo });
    let _ = assert_ai_no_leak(&e, &[&pack]);
    let _ = ans(&mut e, 0, vec![ChoiceOption::Card(cost)]);
    let _ = assert_ai_no_leak(&e, &[&pack]);
    let _ = ans(&mut e, 0, vec![ChoiceOption::Tag(18)]); // Jiedao
    assert_eq!(e.open_choice().unwrap().spec.prompt, "reguhuo_targets");
    let _ = assert_ai_no_leak(&e, &[&pack]);
    let _ = ans(&mut e, 0, vec![ChoiceOption::Player(1)]);
    let jv = e.open_choice().cloned().expect("reguhuo_jiedao_victim");
    assert_eq!(jv.spec.prompt, "reguhuo_jiedao_victim");
    let _ = assert_ai_no_leak(&e, &[&pack]);
    // Cancel victim -> once NOT consumed!
    roundtrip_and_answer(&mut e, &[&pack], Vec::new());
    assert!(!matches!(
        submit(&mut e, 0, GameAction::ActivateSkill { skill: guhuo }),
        CommandOutcome::Rejected(_)
    ));
    let _ = ans(&mut e, 0, vec![ChoiceOption::Card(cost)]);
    let _ = ans(&mut e, 0, vec![ChoiceOption::Tag(18)]);
    let _ = ans(&mut e, 0, vec![ChoiceOption::Player(1)]);
    roundtrip_and_answer(&mut e, &[&pack], vec![ChoiceOption::Player(2)]);
    drain_guess_skip_non_owner(&mut e);
    // JiedaoMachine asks seat 1 (`jiedao_sha_or_give`) -> Tag(1) surrenders weapon (`JiedaoMachine` moves to victim equips)
    let jd = e.open_choice().cloned().expect("jiedao_sha_or_give");
    assert_eq!((jd.observer, jd.spec.prompt), (1, "jiedao_sha_or_give"));
    let _ = ans(&mut e, 1, vec![ChoiceOption::Tag(1)]);
    drain(&mut e);
    assert!(
        !e.state_core().player(1).equips.contains(&wp)
            && e.state_core().player(2).equips.contains(&wp),
        "Jiedao surrender weapon moves wp out of holder"
    );

    // 2. Same Jiedao 2-target on `scstaoluan` + cancel victim preserves once
    let mut e_tl = epic_engine(10_4102);
    e_tl.debug_grant(0, "scstaoluan");
    let wp2 = e_tl.debug_place(1, "zhuge");
    advance_to_play(&mut e_tl, 1);
    let _ = submit(
        &mut e_tl,
        1,
        GameAction::PlayCard {
            card: wp2,
            targets: Vec::new(),
        },
    );
    advance_to_play(&mut e_tl, 0);
    e_tl.debug_clear_hand(0);
    let c_tl = e_tl.debug_place(0, "shan");
    let tl_def = e_tl.catalog().skill_by_name("scstaoluan").unwrap();
    let _ = submit(&mut e_tl, 0, GameAction::ActivateSkill { skill: tl_def });
    let _ = ans(&mut e_tl, 0, vec![ChoiceOption::Card(c_tl)]);
    let _ = ans(&mut e_tl, 0, vec![ChoiceOption::Tag(18)]);
    let _ = ans(&mut e_tl, 0, vec![ChoiceOption::Player(1)]);
    assert_eq!(e_tl.open_choice().unwrap().spec.prompt, "scstaoluan_jiedao_victim");
    let _ = assert_ai_no_leak(&e_tl, &[&pack]);
    roundtrip_and_answer(&mut e_tl, &[&pack], Vec::new());
    // Reactivate & finish 2-target Jiedao
    let _ = submit(&mut e_tl, 0, GameAction::ActivateSkill { skill: tl_def });
    let _ = ans(&mut e_tl, 0, vec![ChoiceOption::Card(c_tl)]);
    let _ = ans(&mut e_tl, 0, vec![ChoiceOption::Tag(18)]);
    let _ = ans(&mut e_tl, 0, vec![ChoiceOption::Player(1)]);
    roundtrip_and_answer(&mut e_tl, &[&pack], vec![ChoiceOption::Player(2)]);
    assert_eq!(e_tl.open_choice().unwrap().observer, 1);
    let hp2_before_tl = e_tl.debug_hp(2);
    let _ = ans(&mut e_tl, 1, vec![ChoiceOption::Tag(0)]); // holder chooses Sha against victim 2
    drain(&mut e_tl);
    assert_eq!(e_tl.debug_hp(2), hp2_before_tl - 1);

    // 3. Non-20 catalog trick (`gujian` `feibiao` requires 1 target, NOT 0-target panic; `swd` `guiyoujie` excluded)
    let gujian_json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-gujian-upstream-noname/assets/cards.json"),
    )
    .unwrap();
    let swd_json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .unwrap();
    let gujian = bridge_pack(&gujian_json).unwrap().pack;
    let swd = bridge_pack(&swd_json).unwrap().pack;
    let mut e_n20 = Engine::new_with_extensions(10_4103, 4, &[&pack, &gujian, &swd]).unwrap();
    e_n20.debug_grant(0, "reguhuo");
    advance_to_play(&mut e_n20, 0);
    e_n20.debug_clear_hand(0);
    let card_n20 = e_n20.debug_place(0, "shan");
    let guhuo2 = e_n20.catalog().skill_by_name("reguhuo").unwrap();
    let _ = submit(&mut e_n20, 0, GameAction::ActivateSkill { skill: guhuo2 });
    let _ = ans(&mut e_n20, 0, vec![ChoiceOption::Card(card_n20)]);
    let name_ch = e_n20.open_choice().cloned().expect("reguhuo_name with ext");
    let feibiao_tag = name_ch
        .spec
        .options
        .iter()
        .filter_map(|o| match *o {
            ChoiceOption::Tag(t) if t >= 20 => Some(t),
            _ => None,
        })
        .next()
        .expect("non-20 catalog trick offered");
    let _ = submit(
        &mut e_n20,
        0,
        GameAction::Answer {
            choice_id: name_ch.id,
            selection: vec![ChoiceOption::Tag(feibiao_tag)],
        },
    );
    let t_non20 = e_n20.open_choice().cloned().expect("non-20 trick MUST open target prompt");
    assert_eq!(t_non20.spec.prompt, "reguhuo_targets");
    let _ = submit(
        &mut e_n20,
        0,
        GameAction::Answer {
            choice_id: t_non20.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    drain(&mut e_n20);

    // 4. Real `reguhuo` responses on `duel_sha`, `aoe_response` (ArcheryAttack), and `dying_save`
    let mut e_resp = epic_engine(10_4104);
    e_resp.debug_grant(0, "reguhuo");
    let gh = e_resp.catalog().skill_by_name("reguhuo").unwrap();
    // 4a. `duel_sha`
    advance_to_play(&mut e_resp, 1);
    e_resp.debug_clear_hand(0);
    let r_sha = e_resp.debug_place(0, "sha");
    let duel = e_resp.debug_place(1, "duel");
    let _ = submit(
        &mut e_resp,
        1,
        GameAction::PlayCard {
            card: duel,
            targets: vec![0],
        },
    );
    drain(&mut e_resp);
    // Reset once via next round then test `aoe_response`
    advance_to_play(&mut e_resp, 2);
    e_resp.debug_clear_hand(0);
    let r_shan = e_resp.debug_place(0, "shan");
    let wanjian = e_resp.debug_place(2, "archery");
    let _ = submit(
        &mut e_resp,
        2,
        GameAction::PlayCard {
            card: wanjian,
            targets: Vec::new(),
        },
    );
    // Pass nullify windows (and any other seat before 0) to reach seat 0's `aoe_response`
    while let Some(c) = e_resp.open_choice().cloned() {
        if c.spec.prompt == "nullify_window"
            || (c.spec.prompt == "aoe_response" && c.observer != 0)
        {
            let _ = submit(
                &mut e_resp,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Pass],
                },
            );
        } else {
            break;
        }
    }
    // Seat 0 responds to ArcheryAttack via reguhuo
    let aoe_c = e_resp.open_choice().cloned().expect("aoe respond with reguhuo");
    assert_eq!((aoe_c.observer, &*aoe_c.spec.prompt), (0, "aoe_response"));
    assert!(aoe_c.spec.options.contains(&ChoiceOption::Skill(gh)));
    let o1 = submit(
        &mut e_resp,
        0,
        GameAction::Answer {
            choice_id: aoe_c.id,
            selection: vec![ChoiceOption::Skill(gh)],
        },
    );
    assert!(!matches!(o1, CommandOutcome::Rejected(_)), "aoe reguhuo: {o1:?}");
    assert_eq!(e_resp.open_choice().unwrap().spec.prompt, "reguhuo_card");
    let o2 = ans(&mut e_resp, 0, vec![ChoiceOption::Card(r_shan)]);
    assert!(!matches!(o2, CommandOutcome::Rejected(_)), "reguhuo_card: {o2:?}");
    let nm = e_resp.open_choice().cloned().expect("reguhuo_name");
    assert_eq!(nm.spec.prompt, "reguhuo_name");
    let o3 = submit(
        &mut e_resp,
        0,
        GameAction::Answer {
            choice_id: nm.id,
            selection: vec![nm.spec.options[0]],
        },
    );
    assert!(!matches!(o3, CommandOutcome::Rejected(_)), "reguhuo_name: {o3:?}");
    assert_eq!(e_resp.open_choice().unwrap().spec.prompt, "reguhuo_guess_pick");
    let hp0_before = e_resp.debug_hp(0);
    drain(&mut e_resp);
    assert_eq!(e_resp.debug_hp(0), hp0_before, "reguhuo real Shan in AOE avoids damage");
    // 4b. `duel_sha` + `dying_save` in fresh round
    let _ = (r_sha,);
    let mut e_dd = epic_engine(10_4105);
    e_dd.debug_grant(0, "reguhuo");
    advance_to_play(&mut e_dd, 1);
    e_dd.debug_clear_hand(0);
    let r_sha2 = e_dd.debug_place(0, "sha");
    let duel2 = e_dd.debug_place(1, "duel");
    let _ = submit(
        &mut e_dd,
        1,
        GameAction::PlayCard {
            card: duel2,
            targets: vec![0],
        },
    );
    while let Some(c) = e_dd.open_choice().cloned() {
        if c.spec.prompt == "nullify_window" {
            let _ = submit(
                &mut e_dd,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Pass],
                },
            );
        } else {
            break;
        }
    }
    let dc = e_dd.open_choice().cloned().expect("duel_sha");
    assert_eq!(dc.spec.prompt, "duel_sha");
    assert!(dc.spec.options.contains(&ChoiceOption::Skill(gh)));
    let _ = submit(
        &mut e_dd,
        0,
        GameAction::Answer {
            choice_id: dc.id,
            selection: vec![ChoiceOption::Skill(gh)],
        },
    );
    let _ = ans(&mut e_dd, 0, vec![ChoiceOption::Card(r_sha2)]);
    let dn = e_dd.open_choice().cloned().expect("duel reguhuo_name");
    assert_eq!(dn.spec.prompt, "reguhuo_name");
    let _ = submit(
        &mut e_dd,
        0,
        GameAction::Answer {
            choice_id: dn.id,
            selection: vec![dn.spec.options[0]],
        },
    );
    drain_guess_skip_non_owner(&mut e_dd);
    let hp1_before = e_dd.debug_hp(1);
    drain(&mut e_dd);
    assert_eq!(e_dd.debug_hp(1), hp1_before - 1, "reguhuo truthful Sha bounces Duel back onto challenger");
    // 4c. `dying_save` saving self (`Tag(4)` Tao)
    advance_to_play(&mut e_dd, 2);
    e_dd.debug_clear_hand(0);
    let r_tao = e_dd.debug_place(0, "tao");
    e_dd.debug_set_hp(0, 1);
    let _ = e_dd.debug_deal_damage(Some(2), 0, 1, DamageNature::Normal);
    while let Some(c) = e_dd.open_choice().cloned() {
        if c.spec.prompt == "dying_save" && c.observer != 0 {
            let _ = submit(
                &mut e_dd,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Pass],
                },
            );
        } else {
            break;
        }
    }
    let dy = e_dd.open_choice().cloned().expect("dying_save");
    assert_eq!((dy.observer, &*dy.spec.prompt), (0, "dying_save"));
    assert!(dy.spec.options.contains(&ChoiceOption::Skill(gh)));
    let d1 = submit(
        &mut e_dd,
        0,
        GameAction::Answer {
            choice_id: dy.id,
            selection: vec![ChoiceOption::Skill(gh)],
        },
    );
    assert!(!matches!(d1, CommandOutcome::Rejected(_)), "dying reguhuo: {d1:?}");
    assert_eq!(e_dd.open_choice().unwrap().spec.prompt, "reguhuo_card");
    let d2 = ans(&mut e_dd, 0, vec![ChoiceOption::Card(r_tao)]);
    assert!(!matches!(d2, CommandOutcome::Rejected(_)), "dying reguhuo_card: {d2:?}");
    let dn2 = e_dd.open_choice().cloned().expect("dying reguhuo_name");
    assert_eq!(dn2.spec.prompt, "reguhuo_name");
    let d3 = ans(&mut e_dd, 0, vec![ChoiceOption::Tag(4)]);
    assert!(!matches!(d3, CommandOutcome::Rejected(_)), "dying reguhuo_name: {d3:?}");
    assert_eq!(e_dd.open_choice().unwrap().spec.prompt, "reguhuo_guess_pick");
    drain(&mut e_dd);
    assert_eq!(e_dd.debug_hp(0), 1, "reguhuo dying_save recovers 1 HP");
}

#[test]
fn l10_scsanruo_four_suits_unlimited_sha_and_steal_effects() {
    let pack = epic_pack();
    // 1. `Club -> Shan` responding to Sha + `scsanruo_steal_confirm` steals from attacker
    let mut e = epic_engine(10_4201);
    e.debug_grant(0, "scsanruo");
    let anruo = e.catalog().skill_by_name("scsanruo").unwrap();
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    let club = e.debug_place_suit(0, Suit::Club);
    let s1_loot = e.debug_place(1, "tao");
    let sha1 = e.debug_place(1, "sha");
    let _ = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![0],
        },
    );
    let resp = e.open_choice().cloned().expect("respond_shan with scsanruo");
    assert!(resp.spec.options.contains(&ChoiceOption::Skill(anruo)));
    roundtrip_and_answer(&mut e, &[&pack], vec![ChoiceOption::Skill(anruo)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "scsanruo_card");
    roundtrip_and_answer(&mut e, &[&pack], vec![ChoiceOption::Card(club)]);
    let cf = e.open_choice().cloned().expect("scsanruo_steal_confirm");
    assert_eq!(cf.spec.prompt, "scsanruo_steal_confirm");
    let _ = assert_ai_no_leak(&e, &[&pack]);
    roundtrip_and_answer(&mut e, &[&pack], vec![ChoiceOption::Confirm]);
    let sc = e.open_choice().cloned().expect("scsanruo_steal_card");
    assert_eq!(sc.spec.prompt, "scsanruo_steal_card");
    assert!(sc.spec.options.contains(&ChoiceOption::Card(s1_loot)));
    let _ = assert_ai_no_leak(&e, &[&pack]);
    roundtrip_and_answer(&mut e, &[&pack], vec![ChoiceOption::Card(s1_loot)]);
    drain(&mut e);
    assert!(e.debug_hand(0).contains(&s1_loot), "scsanruo steals card from Sha source");

    // 2. `Spade -> Nullify` in NullifyWindow: enemy trick steals `he`; self trick steals `e` only!
    advance_to_play(&mut e, 2);
    e.debug_clear_hand(0);
    let spade = e.debug_place_suit(0, Suit::Spade);
    let s2_loot = e.debug_place(2, "shan");
    let wuzhong2 = e.debug_place(2, "wuzhong");
    let _ = submit(
        &mut e,
        2,
        GameAction::PlayCard {
            card: wuzhong2,
            targets: Vec::new(),
        },
    );
    let nw = e.open_choice().cloned().expect("nullify_window");
    assert!(nw.spec.options.contains(&ChoiceOption::Skill(anruo)));
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: nw.id,
            selection: vec![ChoiceOption::Skill(anruo)],
        },
    );
    let _ = ans(&mut e, 0, vec![ChoiceOption::Card(spade)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "scsanruo_steal_confirm");
    let _ = ans(&mut e, 0, vec![ChoiceOption::Confirm]);
    let _ = ans(&mut e, 0, vec![ChoiceOption::Card(s2_loot)]);
    drain(&mut e);
    assert!(e.debug_hand(0).contains(&s2_loot));

    // Self-nullify (`respondTo[0] == owner`): only `'e'` allowed, hand card must NOT steal
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    let sp0 = e.debug_place_suit(0, Suit::Spade);
    let wz0 = e.debug_place(0, "wuzhong");
    let keep0 = e.debug_place(0, "sha");
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: wz0,
            targets: Vec::new(),
        },
    );
    let nw0 = e.open_choice().cloned().expect("self nullify_window");
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: nw0.id,
            selection: vec![ChoiceOption::Skill(anruo)],
        },
    );
    let _ = ans(&mut e, 0, vec![ChoiceOption::Card(sp0)]);
    assert!(
        e.open_choice().is_none_or(|c| c.spec.prompt != "scsanruo_steal_confirm"),
        "self-nullify with empty equips must not steal own hand ({keep0})"
    );
    drain(&mut e);

    // 3. `Diamond -> FireSha`: unlimited with Paoxiao in Phase::Play + `Duel` response + `scsanruo_steal_confirm`
    e.debug_grant(0, "咆哮");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    let sha0 = e.debug_place(0, "sha");
    let dia = e.debug_place_suit(0, Suit::Diamond);
    let s3_loot = e.debug_place(3, "tao");
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha0,
            targets: vec![1],
        },
    );
    drain(&mut e);
    assert!(
        !matches!(
            submit(&mut e, 0, GameAction::ActivateSkill { skill: anruo }),
            CommandOutcome::Rejected(_)
        ),
        "scsanruo Diamond FireSha must obey has_unlimited_sha (paoxiao)"
    );
    let _ = ans(&mut e, 0, vec![ChoiceOption::Card(dia)]);
    let _ = ans(&mut e, 0, vec![ChoiceOption::Player(3)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "scsanruo_steal_confirm");
    let _ = ans(&mut e, 0, vec![ChoiceOption::Confirm]);
    let _ = ans(&mut e, 0, vec![ChoiceOption::Card(s3_loot)]);
    drain(&mut e);
    assert!(e.debug_hand(0).contains(&s3_loot));

    // 4. `Heart -> Tao` in `DyingSave` opens `scsanruo_steal_target` -> steals 1 from other
    let mut e_dy = epic_engine(10_4202);
    e_dy.debug_grant(0, "scsanruo");
    let anruo_d = e_dy.catalog().skill_by_name("scsanruo").unwrap();
    advance_to_play(&mut e_dy, 1);
    e_dy.debug_clear_hand(0);
    let heart = e_dy.debug_place_suit(0, Suit::Heart);
    let loot_d = e_dy.debug_place(1, "shan");
    e_dy.debug_set_hp(0, 1);
    let _ = e_dy.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    let dy = e_dy.open_choice().cloned().expect("dying_save offers scsanruo");
    assert!(dy.spec.options.contains(&ChoiceOption::Skill(anruo_d)));
    let _ = submit(
        &mut e_dy,
        0,
        GameAction::Answer {
            choice_id: dy.id,
            selection: vec![ChoiceOption::Skill(anruo_d)],
        },
    );
    let _ = ans(&mut e_dy, 0, vec![ChoiceOption::Card(heart)]);
    let st = e_dy.open_choice().cloned().expect("scsanruo_steal_target");
    assert_eq!(st.spec.prompt, "scsanruo_steal_target");
    let _ = assert_ai_no_leak(&e_dy, &[&pack]);
    roundtrip_and_answer(&mut e_dy, &[&pack], vec![ChoiceOption::Player(1)]);
    roundtrip_and_answer(&mut e_dy, &[&pack], vec![ChoiceOption::Card(loot_d)]);
    drain(&mut e_dy);
    assert_eq!(e_dy.debug_hp(0), 1);
    assert!(e_dy.debug_hand(0).contains(&loot_d));
}

#[test]
fn l10_scschiyan_scschihe_virtual_sha_raw_suit_and_order() {
    let pack = epic_pack();
    // 1. Converted FireSha triggers `scschiyan` (incl. CardsLost -> yishan + +1 dmg), while chain damage does not!
    let swd_json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .unwrap();
    let swd = bridge_pack(&swd_json).unwrap().pack;
    let mut e = Engine::new_with_extensions(10_4301, 4, &[&pack, &swd]).unwrap();
    e.debug_grant(0, "scsanruo");
    e.debug_grant(0, "scschiyan");
    e.debug_grant(1, "yishan");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let dia = e.debug_place_suit(0, Suit::Diamond);
    let t_card = e.debug_place(1, "sha");
    let hp1 = e.debug_hp(1);
    let anruo = e.catalog().skill_by_name("scsanruo").unwrap();
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: anruo });
    let _ = ans(&mut e, 0, vec![ChoiceOption::Card(dia)]);
    let _ = ans(&mut e, 0, vec![ChoiceOption::Player(1)]);
    // Cancel anruo steal so target keeps `t_card` for `scschiyan_pick`!
    let _ = ans(&mut e, 0, Vec::new());
    let cy = e.open_choice().cloned().expect("converted FireSha triggers scschiyan_pick");
    assert_eq!(cy.spec.prompt, "scschiyan_pick");
    roundtrip_and_answer(&mut e, &[&pack, &swd], vec![ChoiceOption::Card(t_card)]);
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1 - 2, "converted FireSha triggers scschiyan +1 dmg (`mobile.js:4798`)");
    assert!(!e.state_core().player(1).yishan_lost.is_empty(), "scschiyan_pick must record CardsLost");

    // 2. Virtual `Sha` (`dddxiaxing`, card == None) + selectable `l10_scschiyan_scschihe_order`
    let d3_json = std::fs::read_to_string(
        repo_extensions_root().join("d3-upstream-noname/assets/character.json"),
    )
    .unwrap();
    let d3 = bridge_pack(&d3_json).unwrap().pack;
    let p9 = content_native_loader::decision34_pack9_for_coexistence(&[&d3, &pack]);
    let mut ev = Engine::new_with_extensions(10_4302, 4, &[&pack, &d3, &p9]).unwrap();
    let zm = (0..ev.catalog().general_count() as u16)
        .find(|&idx| ev.catalog().general_meta(idx).is_some_and(|m| m.0 == "ddd_zhangmiao"))
        .expect("find ddd_zhangmiao");
    ev.assign_general(0, zm).expect("assign zhangmiao");
    ev.debug_grant(0, "scschiyan");
    ev.debug_grant(0, "scschihe");
    let dd_cost = ev.debug_place(0, "shan");
    ev.debug_set_turn(3, Phase::End, 1);
    let _ = submit(&mut ev, 3, GameAction::NextPhase);
    assert_eq!(ev.open_choice().unwrap().spec.prompt, "dddxiaxing_card");
    let _ = ans(&mut ev, 0, vec![ChoiceOption::Card(dd_cost)]);
    advance_to_play(&mut ev, 0);
    ev.debug_clear_hand(0);
    ev.debug_clear_hand(1);
    let t1_card = ev.debug_place(1, "tao");
    let xia_def = ev.catalog().skill_by_name("dddxiaxing").unwrap();
    let _ = submit(&mut ev, 0, GameAction::ActivateSkill { skill: xia_def });
    let _ = ans(&mut ev, 0, vec![ChoiceOption::Player(1)]);
    let ord = ev.open_choice().cloned().expect("order choice between scschiyan and scschihe");
    assert_eq!(ord.spec.prompt, "l10_scschiyan_scschihe_order");
    let _ = assert_ai_no_leak(&ev, &[&pack, &d3, &p9]);
    // Pick Tag(1) -> scschihe runs FIRST on virtual Sha (card == None)
    roundtrip_and_answer(&mut ev, &[&pack, &d3, &p9], vec![ChoiceOption::Tag(1)]);
    assert_eq!(ev.open_choice().unwrap().spec.prompt, "scschihe_confirm");
    roundtrip_and_answer(&mut ev, &[&pack, &d3, &p9], vec![ChoiceOption::Confirm]);
    assert_eq!(ev.open_choice().unwrap().spec.prompt, "scschiyan_pick");
    let _ = ans(&mut ev, 0, vec![ChoiceOption::Card(t1_card)]);
    drain(&mut ev);

    // 3. `scschihe` uses RAW `catalog.cards[judged].suit` (`mobile.js:5111`), immune to `xinhongyan`!
    let mut eh = Engine::new_with_extensions(10_4303, 4, &[&pack, &d3, &p9]).unwrap();
    eh.debug_grant(0, "scschihe");
    eh.debug_grant(0, "xinhongyan");
    advance_to_play(&mut eh, 0);
    eh.debug_clear_hand(0);
    let heart_sha = eh
        .state_core()
        .deck
        .iter()
        .copied()
        .find(|&c| eh.catalog().card_kind(c).is_sha_like() && eh.catalog().cards[c as usize].suit == Suit::Heart)
        .expect("heart sha");
    eh.state_core_mut().deck.retain(|&c| c != heart_sha);
    eh.state_core_mut().player_mut(0).hand.push(heart_sha);
    let sp1 = eh
        .state_core()
        .deck
        .iter()
        .copied()
        .find(|&c| eh.catalog().cards[c as usize].suit == Suit::Spade)
        .unwrap();
    let sp2 = eh
        .state_core()
        .deck
        .iter()
        .copied()
        .find(|&c| c != sp1 && eh.catalog().cards[c as usize].suit == Suit::Spade)
        .unwrap();
    eh.state_core_mut().deck.retain(|&c| c != sp1 && c != sp2);
    eh.state_core_mut().deck.push(sp1);
    eh.state_core_mut().deck.push(sp2);
    let hp1_h = eh.debug_hp(1);
    let _ = submit(
        &mut eh,
        0,
        GameAction::PlayCard {
            card: heart_sha,
            targets: vec![1],
        },
    );
    let _ = ans(&mut eh, 0, vec![ChoiceOption::Confirm]);
    drain(&mut eh);
    assert_eq!(eh.debug_hp(1), hp1_h - 1, "scschihe must read raw Spade on deck (NOT hongyan Heart), so 0 bonus dmg");
}

#[test]
fn l10_scskuiji_scsyaozhuo_scspicai_scszimou_scsxiaolu() {
    let pack = epic_pack();
    // 1. `scskuiji` enforces `[4, 4]` distinct suits (reject <4 & duplicate suit; accepts 4 distinct)
    let mut e = epic_engine(10_4401);
    e.debug_grant(0, "scskuiji");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let c0 = e.debug_place_suit(0, Suit::Spade);
    let c1 = e.debug_place_suit(0, Suit::Heart);
    let c2 = e.debug_place_suit(1, Suit::Club);
    let c3 = e.debug_place_suit(1, Suit::Diamond);
    let c_dup = e.debug_place_suit(1, Suit::Spade);
    let kj = e.catalog().skill_by_name("scskuiji").unwrap();
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: kj });
    let _ = ans(&mut e, 0, vec![ChoiceOption::Player(1)]);
    let kc = e.open_choice().cloned().expect("scskuiji_cards");
    assert_eq!((kc.spec.min, kc.spec.max, kc.spec.allow_cancel), (4, 4, true));
    assert!(matches!(
        submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: kc.id,
                selection: vec![ChoiceOption::Card(c0), ChoiceOption::Card(c1)],
            },
        ),
        CommandOutcome::Rejected(_)
    ));
    assert!(matches!(
        submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: kc.id,
                selection: vec![
                    ChoiceOption::Card(c0),
                    ChoiceOption::Card(c1),
                    ChoiceOption::Card(c2),
                    ChoiceOption::Card(c_dup),
                ],
            },
        ),
        CommandOutcome::Rejected(_)
    ));
    let ai_kj = assert_ai_no_leak(&e, &[&pack]);
    assert_eq!(ai_kj.len(), 4);
    roundtrip_and_answer(
        &mut e,
        &[&pack],
        vec![
            ChoiceOption::Card(c0),
            ChoiceOption::Card(c1),
            ChoiceOption::Card(c2),
            ChoiceOption::Card(c3),
        ],
    );
    drain(&mut e);
    assert!(e.debug_hand(0).is_empty() && e.debug_hand(1) == vec![c_dup]);

    // 2. `scsyaozhuo` winner skips target next draw WITHOUT permanently reducing target hand limit (-1)
    let mut ey = epic_engine(10_4402);
    ey.debug_grant(0, "scsyaozhuo");
    advance_to_play(&mut ey, 0);
    ey.debug_clear_hand(0);
    ey.debug_clear_hand(1);
    let high = ey
        .state_core()
        .deck
        .iter()
        .copied()
        .max_by_key(|&c| ey.catalog().cards[c as usize].rank)
        .unwrap();
    let low = ey
        .state_core()
        .deck
        .iter()
        .copied()
        .min_by_key(|&c| ey.catalog().cards[c as usize].rank)
        .unwrap();
    ey.state_core_mut().deck.retain(|&c| c != high && c != low);
    ey.state_core_mut().player_mut(0).hand.push(high);
    ey.state_core_mut().player_mut(1).hand.push(low);
    let yz = ey.catalog().skill_by_name("scsyaozhuo").unwrap();
    let _ = submit(&mut ey, 0, GameAction::ActivateSkill { skill: yz });
    let _ = ans(&mut ey, 0, vec![ChoiceOption::Player(1)]);
    let _ = ans(&mut ey, 0, vec![ChoiceOption::Card(high)]);
    let _ = ans(&mut ey, 1, vec![ChoiceOption::Card(low)]);
    drain(&mut ey);
    assert!(ey.state_core().player(1).skip_next_draw);
    // Advance to seat 1 Play phase, verify seat 1 still holds hp==4 cards through Phase::Discard!
    advance_to_play(&mut ey, 1);
    ey.debug_clear_hand(1);
    for _ in 0..4 {
        let _ = ey.debug_place(1, "shan");
    }
    let _ = submit(&mut ey, 1, GameAction::NextPhase); // Play -> Discard
    drain(&mut ey);
    assert_eq!(
        ey.debug_hand(1).len(),
        4,
        "scsyaozhuo must NOT apply permanent -1 hand limit (`d34_flags.rs:209`)"
    );

    // 3. `scspicai` emits real `OrdinaryJudge` (with `鬼才` rewrite & `filterInD` via `天妒`)
    let mut ep = epic_engine(10_4403);
    ep.debug_grant(0, "scspicai");
    ep.debug_grant(0, "鬼才");
    ep.debug_grant(0, "天妒");
    advance_to_play(&mut ep, 0);
    ep.debug_clear_hand(0);
    let rewrite = ep.debug_place_suit(0, Suit::Club);
    let j1 = ep
        .state_core()
        .deck
        .iter()
        .copied()
        .find(|&c| ep.catalog().cards[c as usize].suit == Suit::Spade)
        .unwrap();
    let j2 = ep
        .state_core()
        .deck
        .iter()
        .copied()
        .find(|&c| c != j1 && ep.catalog().cards[c as usize].suit == Suit::Spade)
        .unwrap();
    ep.state_core_mut().deck.retain(|&c| c != j1 && c != j2);
    ep.state_core_mut().deck.push(j2); // 2nd judge -> duplicate Spade unless rewritten to Club!
    ep.state_core_mut().deck.push(j1); // 1st judge -> Spade
    let pc = ep.catalog().skill_by_name("scspicai").unwrap();
    let _ = submit(&mut ep, 0, GameAction::ActivateSkill { skill: pc });
    // Judge 1 asks `鬼才` (`gui_dao_rewrite`) -> pass -> asks `天妒` (`tian_du_gain`, seat 0 declines so j1 remains in discard!)
    let g1 = ep.open_choice().unwrap();
    assert_eq!((g1.observer, &*g1.spec.prompt), (0, "gui_dao_rewrite"));
    let _ = ans(&mut ep, 0, vec![ChoiceOption::Pass]);
    let t1 = ep.open_choice().unwrap();
    assert_eq!((t1.observer, &*t1.spec.prompt), (0, "tian_du_gain"));
    let _ = ans(&mut ep, 0, Vec::new());
    assert_eq!(ep.open_choice().unwrap().spec.prompt, "scspicai_continue");
    roundtrip_and_answer(&mut ep, &[&pack], vec![ChoiceOption::Confirm]);
    // Judge 2 (Spade -> rewrite to Club via `鬼才`!) -> asks `天妒` -> seat 0 TAKES Club (`filterInD` excludes it)!
    let g2 = ep.open_choice().unwrap();
    assert_eq!((g2.observer, &*g2.spec.prompt), (0, "gui_dao_rewrite"));
    let _ = ans(&mut ep, 0, vec![ChoiceOption::Card(rewrite)]);
    let t2 = ep.open_choice().unwrap();
    assert_eq!((t2.observer, &*t2.spec.prompt), (0, "tian_du_gain"));
    let _ = ans(&mut ep, 0, vec![ChoiceOption::Confirm]);
    // Different suit (Club != Spade) so scspicai_continue reopens! Stop -> gives ONLY j1 (still in discard)
    assert_eq!(ep.open_choice().unwrap().spec.prompt, "scspicai_continue");
    let _ = ans(&mut ep, 0, Vec::new());
    let pt = ep.open_choice().cloned().expect("scspicai_target");
    assert_eq!(pt.spec.prompt, "scspicai_target");
    roundtrip_and_answer(&mut ep, &[&pack], vec![ChoiceOption::Player(2)]);
    drain(&mut ep);
    assert!(ep.debug_hand(0).contains(&rewrite), "tiandu claimed rewritten Club");
    assert!(ep.debug_hand(2).contains(&j1) && !ep.debug_hand(2).contains(&rewrite), "filterInD gives only cards still in discard");

    // 4. `scszimou` reads `cards_used_play_phase` (cards used BEFORE gaining `scszimou` count!) + `scsxiaolu` CardsLost
    let swd_json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .unwrap();
    let swd = bridge_pack(&swd_json).unwrap().pack;
    let mut ez = Engine::new_with_extensions(10_4404, 4, &[&pack, &swd]).unwrap();
    advance_to_play(&mut ez, 0);
    ez.debug_clear_hand(0);
    let wz1 = ez.debug_place(0, "wuzhong");
    let _ = submit(
        &mut ez,
        0,
        GameAction::PlayCard {
            card: wz1,
            targets: Vec::new(),
        },
    );
    drain(&mut ez);
    ez.debug_grant(0, "scszimou");
    let wz2 = ez.debug_place(0, "wuzhong");
    let _ = submit(
        &mut ez,
        0,
        GameAction::PlayCard {
            card: wz2,
            targets: Vec::new(),
        },
    );
    drain(&mut ez);
    assert!(
        ez.debug_hand(0).iter().any(|&c| ez.catalog().card_kind(c) == CardKind::Jiu),
        "2nd play-phase card after late scszimou grant must trigger Jiu gain"
    );

    ez.debug_grant(0, "scsxiaolu");
    ez.debug_grant(0, "yishan");
    let xl = ez.catalog().skill_by_name("scsxiaolu").unwrap();
    let eq0 = ez.debug_place(0, "zhuge");
    let spare = ez.debug_place(0, "shan");
    let _ = submit(&mut ez, 0, GameAction::ActivateSkill { skill: xl });
    let _ = ans(&mut ez, 0, vec![ChoiceOption::Tag(0)]);
    let _ = ans(&mut ez, 0, vec![ChoiceOption::Card(eq0), ChoiceOption::Card(spare)]);
    let _ = ans(&mut ez, 0, vec![ChoiceOption::Player(1)]);
    drain(&mut ez);
    assert!(ez.debug_hand(1).contains(&eq0) && !ez.state_core().player(0).yishan_lost.is_empty(), "scsxiaolu give-2 must emit CardsLost");
}

#[test]
fn l10_round3_kdanggu_scschihe_declined_nomatch_and_multitarget_no_phantom_boost() {
    let pack = epic_pack();
    // 1. Grant `scschihe` through real `k_danggu` product flow (`k_danggu_partner` -> Tag(7)).
    let mut e = (10_5000..10_5080)
        .find_map(|seed| {
            let mut cand = epic_engine(seed);
            cand.debug_grant(0, "k_danggu");
            let _ = submit(&mut cand, 0, GameAction::NextPhase);
            let c = cand.open_choice().cloned()?;
            if c.spec.prompt == "k_danggu_partner" && c.spec.options.contains(&ChoiceOption::Tag(7))
            {
                let out = submit(
                    &mut cand,
                    0,
                    GameAction::Answer {
                        choice_id: c.id,
                        selection: vec![ChoiceOption::Tag(7)],
                    },
                );
                assert!(!matches!(out, CommandOutcome::Rejected(_)));
                Some(cand)
            } else {
                None
            }
        })
        .expect("find k_danggu seed offering scschihe (Tag 7)");
    advance_to_play(&mut e, 0);
    let chihe_def = e.catalog().skill_by_name("scschihe").expect("scschihe def");
    assert!(e.state_core().skills.contains_key(&(0, chihe_def)));
    assert!(
        !e.state_core()
            .vm_aux
            .flags
            .contains_key(&(0, "l10_scschihe_sha_boost".to_string())),
        "k_danggu granting scschihe must NOT set phantom l10_scschihe_sha_boost"
    );

    for seat in 0..4 {
        e.debug_clear_hand(seat);
    }
    let find_deck_suit = |eng: &Engine, sha: bool, suit: Suit, excl: &[u32]| -> u32 {
        eng.state_core()
            .deck
            .iter()
            .copied()
            .find(|&c| {
                !excl.contains(&c)
                    && eng.catalog().cards[c as usize].suit == suit
                    && (!sha || eng.catalog().card_kind(c).is_sha_like())
            })
            .expect("matching card in deck")
    };

    // 1a. Declined `scschihe_confirm` deals base 1 damage (NOT 2!)
    let sha_h1 = find_deck_suit(&e, true, Suit::Heart, &[]);
    e.state_core_mut().deck.retain(|&c| c != sha_h1);
    e.state_core_mut().player_mut(0).hand.push(sha_h1);
    let hp1_before = e.debug_hp(1);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha_h1,
            targets: vec![1],
        },
    );
    assert_eq!(e.open_choice().unwrap().spec.prompt, "scschihe_confirm");
    roundtrip_and_answer(&mut e, &[&pack], Vec::new());
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1_before - 1, "declined scschihe must deal 1 dmg, not 2");

    // 1b. Confirmed `scschihe` flipping 2 non-matching Spades vs Heart Sha deals 1 damage (NOT 2!)
    e.debug_grant(0, "咆哮");
    let sha_h2 = find_deck_suit(&e, true, Suit::Heart, &[]);
    let sp1 = find_deck_suit(&e, false, Suit::Spade, &[sha_h2]);
    let sp2 = find_deck_suit(&e, false, Suit::Spade, &[sha_h2, sp1]);
    e.state_core_mut()
        .deck
        .retain(|&c| c != sha_h2 && c != sp1 && c != sp2);
    e.state_core_mut().player_mut(0).hand.push(sha_h2);
    e.state_core_mut().deck.push(sp1);
    e.state_core_mut().deck.push(sp2);
    let hp1_b = e.debug_hp(1);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha_h2,
            targets: vec![1],
        },
    );
    assert_eq!(e.open_choice().unwrap().spec.prompt, "scschihe_confirm");
    let _ = ans(&mut e, 0, vec![ChoiceOption::Confirm]);
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1_b - 1, "0-match scschihe must deal 1 dmg, not 2");

    // 1c. Confirmed `scschihe` flipping 2 matching Hearts vs Heart Sha deals 1 + 2 = 3 damage (NOT 4!)
    let sha_h3 = find_deck_suit(&e, true, Suit::Heart, &[]);
    let h1 = find_deck_suit(&e, false, Suit::Heart, &[sha_h3]);
    let h2 = find_deck_suit(&e, false, Suit::Heart, &[sha_h3, h1]);
    e.state_core_mut()
        .deck
        .retain(|&c| c != sha_h3 && c != h1 && c != h2);
    e.state_core_mut().player_mut(0).hand.push(sha_h3);
    e.state_core_mut().deck.push(h1);
    e.state_core_mut().deck.push(h2);
    let hp3_before = e.debug_hp(3);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha_h3,
            targets: vec![3],
        },
    );
    assert_eq!(e.open_choice().unwrap().spec.prompt, "scschihe_confirm");
    let _ = ans(&mut e, 0, vec![ChoiceOption::Confirm]);
    drain(&mut e);
    assert_eq!(e.debug_hp(3), hp3_before - 3, "2-match scschihe deals 1 + 2 = 3 dmg, not 4");

    // 1d. Multi-target Sha (`fangtian` last hand Sha targeting [1, 3]) skips `scschihe` and deals 1 dmg each!
    let ft = e.debug_place(0, "fangtian");
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: ft,
            targets: Vec::new(),
        },
    );
    drain(&mut e);
    e.debug_clear_hand(0);
    let sha_multi = e.debug_place(0, "sha");
    let hp1_m = e.debug_hp(1);
    let hp3_m = e.debug_hp(3);
    let out_m = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha_multi,
            targets: vec![1, 3],
        },
    );
    assert!(!matches!(out_m, CommandOutcome::Rejected(_)));
    assert!(
        e.open_choice().is_none_or(|c| c.spec.prompt != "scschihe_confirm"),
        "multi-target Sha must NOT prompt scschihe"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1_m - 1, "multi-target Sha deals 1 dmg to target 1");
    assert_eq!(e.debug_hp(3), hp3_m - 1, "multi-target Sha deals 1 dmg to target 3");
}

#[test]
fn l10_round3_scsanruo_counter_wuxie_steals_stateplayer_and_self() {
    let pack = epic_pack();
    // Case A: P0 plays Wuzhong, P1 plays Nullify, P2 counters P1's Nullify via `scsanruo` -> steals P1 (stateplayer), NOT P0!
    let mut e = epic_engine(10_5201);
    e.debug_grant(2, "scsanruo");
    let anruo = e.catalog().skill_by_name("scsanruo").unwrap();
    advance_to_play(&mut e, 0);
    for seat in 0..4 {
        e.debug_clear_hand(seat);
    }
    let eq0 = e.debug_place(0, "zhuge");
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: eq0, targets: Vec::new() });
    drain(&mut e);
    let wz0 = e.debug_place(0, "wuzhong");
    let p0_hand = e.debug_place(0, "shan");
    let p1_null = e.debug_place(1, "nullify");
    let p1_loot = e.debug_place(1, "tao");
    let sp2 = e.debug_place_suit(2, Suit::Spade);

    let _ = submit(&mut e, 0, GameAction::PlayCard { card: wz0, targets: Vec::new() });
    let nw_p1 = e.open_choice().cloned().expect("P1 nullify_window");
    assert_eq!((nw_p1.observer, &*nw_p1.spec.prompt), (1, "nullify_window"));
    let _ = submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: nw_p1.id,
            selection: vec![ChoiceOption::Card(p1_null)],
        },
    );
    let nw_p2 = e.open_choice().cloned().expect("P2 counter nullify_window");
    assert_eq!((nw_p2.observer, &*nw_p2.spec.prompt), (2, "nullify_window"));
    assert!(nw_p2.spec.options.contains(&ChoiceOption::Skill(anruo)));
    roundtrip_and_answer(&mut e, &[&pack], vec![ChoiceOption::Skill(anruo)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "scsanruo_card");
    roundtrip_and_answer(&mut e, &[&pack], vec![ChoiceOption::Card(sp2)]);
    // Confirm `scsanruo_steal_confirm` (may appear before or after P3/P0 nullify_window pass; drain only nullify_window).
    while let Some(c) = e.open_choice().cloned() {
        if c.spec.prompt == "nullify_window" {
            let _ = submit(
                &mut e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Pass],
                },
            );
        } else {
            break;
        }
    }
    assert_eq!(e.open_choice().unwrap().spec.prompt, "scsanruo_steal_confirm");
    roundtrip_and_answer(&mut e, &[&pack], vec![ChoiceOption::Confirm]);
    let pick = e.open_choice().cloned().expect("scsanruo_steal_card");
    assert_eq!((pick.observer, &*pick.spec.prompt), (2, "scsanruo_steal_card"));
    assert!(
        pick.spec.options.contains(&ChoiceOption::Card(p1_loot)),
        "counter-wuxie must steal from P1 (stateplayer)"
    );
    assert!(
        !pick.spec.options.contains(&ChoiceOption::Card(p0_hand))
            && !pick.spec.options.contains(&ChoiceOption::Card(eq0)),
        "counter-wuxie must NOT steal from original trick user P0"
    );
    let _ = assert_ai_no_leak(&e, &[&pack]);
    roundtrip_and_answer(&mut e, &[&pack], vec![ChoiceOption::Card(p1_loot)]);
    drain(&mut e);
    assert!(e.debug_hand(2).contains(&p1_loot));
    assert_eq!(e.debug_hand(0).len(), 3, "countered P1 nullify allows P0 Wuzhong to draw 2");

    // Case B: Original trick user (P0 == source) counters P1's Nullify via `scsanruo` -> steals P1's HAND (`!= equip_only`!).
    let mut e2 = epic_engine(10_5202);
    e2.debug_grant(0, "scsanruo");
    let anruo2 = e2.catalog().skill_by_name("scsanruo").unwrap();
    advance_to_play(&mut e2, 0);
    for seat in 0..4 {
        e2.debug_clear_hand(seat);
    }
    let wz_a = e2.debug_place(0, "wuzhong");
    let sp0 = e2.debug_place_suit(0, Suit::Spade);
    let p1_null_b = e2.debug_place(1, "nullify");
    let p1_hand_loot = e2.debug_place(1, "tao");

    let _ = submit(&mut e2, 0, GameAction::PlayCard { card: wz_a, targets: Vec::new() });
    // Queue starts from P0: P0 passes initial window -> P1 plays Nullify -> P0 counters P1!
    if e2.open_choice().is_some_and(|c| c.observer == 0 && c.spec.prompt == "nullify_window") {
        let _ = ans(&mut e2, 0, vec![ChoiceOption::Pass]);
    }
    let nw1 = e2.open_choice().cloned().expect("P1 nullify_window");
    assert_eq!((nw1.observer, &*nw1.spec.prompt), (1, "nullify_window"));
    let _ = submit(
        &mut e2,
        1,
        GameAction::Answer {
            choice_id: nw1.id,
            selection: vec![ChoiceOption::Card(p1_null_b)],
        },
    );
    let nw0_counter = e2.open_choice().cloned().expect("P0 counter nullify_window");
    assert_eq!((nw0_counter.observer, &*nw0_counter.spec.prompt), (0, "nullify_window"));
    let _ = submit(
        &mut e2,
        0,
        GameAction::Answer {
            choice_id: nw0_counter.id,
            selection: vec![ChoiceOption::Skill(anruo2)],
        },
    );
    let _ = ans(&mut e2, 0, vec![ChoiceOption::Card(sp0)]);
    while let Some(c) = e2.open_choice().cloned() {
        if c.spec.prompt == "nullify_window" {
            let _ = submit(
                &mut e2,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Pass],
                },
            );
        } else {
            break;
        }
    }
    assert_eq!(e2.open_choice().unwrap().spec.prompt, "scsanruo_steal_confirm");
    let _ = ans(&mut e2, 0, vec![ChoiceOption::Confirm]);
    let pick2 = e2.open_choice().cloned().expect("P0 steals P1 hand");
    assert_eq!(pick2.spec.prompt, "scsanruo_steal_card");
    assert!(
        pick2.spec.options.contains(&ChoiceOption::Card(p1_hand_loot)),
        "P0 countering P1 Nullify must NOT be restricted to self-equips; must offer P1's hand card"
    );
    let _ = ans(&mut e2, 0, vec![ChoiceOption::Card(p1_hand_loot)]);
    drain(&mut e2);
    assert!(e2.debug_hand(0).contains(&p1_hand_loot));
}

#[test]
fn l10_round3_blocks_all_cards_new_zhixi_bingfeng2_blocks_play_activation() {
    use engine_native::content::host::{HostBehavior, HostSkillDecl};
    let pack = epic_pack();

    // Part A: `new_zhixi` locks after playing a trick -> `reguhuo`, `scstaoluan`, `scsanruo` all blocked in `Phase::Play`!
    let mut p9 = content_native_loader::decision34_pack9_for_coexistence(&[&pack]);
    if !p9.host_skills.iter().any(|s| s.name == "new_zhixi_mark2") {
        p9.host_skills.push(HostSkillDecl {
            name: "new_zhixi_mark2",
            behavior: HostBehavior::NewZhixi,
            presentation_binding: None,
        });
    }
    let mut ez = Engine::new_with_extensions(10_5301, 4, &[&pack, &p9]).unwrap();
    ez.debug_grant(0, "new_zhixi");
    ez.debug_grant(0, "reguhuo");
    ez.debug_grant(0, "scstaoluan");
    ez.debug_grant(0, "scsanruo");
    advance_to_play(&mut ez, 0);
    ez.debug_set_hp(0, 3);
    ez.debug_clear_hand(0);
    let wz = ez.debug_place(0, "wuzhong");
    let _ht = ez.debug_place_suit(0, Suit::Heart);
    let _dia = ez.debug_place_suit(0, Suit::Diamond);

    let gh = ez.catalog().skill_by_name("reguhuo").unwrap();
    let tl = ez.catalog().skill_by_name("scstaoluan").unwrap();
    let ar = ez.catalog().skill_by_name("scsanruo").unwrap();
    let before = ez.legal_actions(0);
    for sk in [gh, tl, ar] {
        assert!(
            before.contains(&GameAction::ActivateSkill { skill: sk }),
            "before new_zhixi triggers, skill {sk} must be legal in Phase::Play"
        );
    }

    let _ = submit(&mut ez, 0, GameAction::PlayCard { card: wz, targets: Vec::new() });
    drain(&mut ez);
    assert!(
        ez.catalog().blocks_all_cards(ez.state_core(), 0),
        "playing Wuzhong triggers new_zhixi trick lock"
    );
    let after = ez.legal_actions(0);
    for sk in [gh, tl, ar] {
        assert!(
            !after.contains(&GameAction::ActivateSkill { skill: sk }),
            "under new_zhixi lock, skill {sk} must NOT appear in legal_actions"
        );
        assert!(
            matches!(
                submit(&mut ez, 0, GameAction::ActivateSkill { skill: sk }),
                CommandOutcome::Rejected(engine_native::RejectReason::InvalidChoice)
            ),
            "under new_zhixi lock, ActivateSkill({sk}) must be rejected"
        );
    }

    // Part B: `bingfeng2` blocks `reguhuo`, `scstaoluan`, and `scsanruo` in `Phase::Play`!
    let swd_json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .unwrap();
    let swd = bridge_pack(&swd_json).unwrap().pack;
    let mut eb = Engine::new_with_extensions(10_5302, 4, &[&pack, &swd]).unwrap();
    eb.debug_grant(0, "bingfeng2");
    eb.debug_grant(0, "reguhuo");
    eb.debug_grant(0, "scstaoluan");
    eb.debug_grant(0, "scsanruo");
    advance_to_play(&mut eb, 0);
    eb.debug_set_hp(0, 3);
    eb.debug_clear_hand(0);
    let _ht_b = eb.debug_place_suit(0, Suit::Heart);
    let _dia_b = eb.debug_place_suit(0, Suit::Diamond);
    assert!(eb.catalog().blocks_all_cards(eb.state_core(), 0));
    let gh_b = eb.catalog().skill_by_name("reguhuo").unwrap();
    let tl_b = eb.catalog().skill_by_name("scstaoluan").unwrap();
    let ar_b = eb.catalog().skill_by_name("scsanruo").unwrap();
    let la_b = eb.legal_actions(0);
    for sk in [gh_b, tl_b, ar_b] {
        assert!(
            !la_b.contains(&GameAction::ActivateSkill { skill: sk }),
            "under bingfeng2, skill {sk} must NOT appear in legal_actions"
        );
        assert!(
            matches!(
                submit(&mut eb, 0, GameAction::ActivateSkill { skill: sk }),
                CommandOutcome::Rejected(engine_native::RejectReason::InvalidChoice)
            ),
            "under bingfeng2, ActivateSkill({sk}) must be rejected"
        );
    }
}
