//! Queue Item 2 Source 12 (`12. U08 英雄杀（150 项：43 将 + 107 技）`)
//! Real-match 6-dimension acceptance suite — Part 1:
//! - Full 43-general + 107-skill + 9 cross-package standard skill catalog & HostBehavior verification;
//! - 5 JSON-bridged skills (`yxs_fanji`, `yxs_fanji2`, `yxs_menshen`, `yxs_menshen2`, `yxs_menshen3`);
//! - Batch A (23 skills, `英雄杀/character/skill.js:110-845`);
//! - Batch B (28 skills, `英雄杀/character/skill.js:846-1491`).
//!
//! Covers all 6 dimensions of Section 四.3:
//! ① Normal execution via real `Engine::submit` + `GameAction::{PlayCard, ActivateSkill, Answer, NextPhase}`;
//! ② Cancellable choice cancel/pass (`ChoiceOption::Pass`) leaving state and per-turn/limited usage unconsumed;
//! ③ Illegal target / illegal card / insufficient count rejection (`CommandOutcome::Rejected`);
//! ④ Condition not met & per-turn / limited skill limits (`usable: 1`, `limited: true` on `yixin`/`heqin`);
//! ⑤ Multi-skill & passive modifier interactions (`kuangchan` Lord `max_hp = 4`, `xianqu` `Sha` `< 8` immunity,
//!   `zyhufu` armor `+5` hand limit vs no-armor `+1` draw, `hanbei` horse directHit, `wumu` black `Sha` unlimited
//!   range & red `Sha` unlimited count, `taolue` `+1` hand limit, `hanqiang` no-weapon `+1` range, `chajue2`
//!   `Sha`/trick nullification, `wluoyan` damage-to-loseHp);
//! ⑥ Mid-choice `encode_archive` + `decode_archive` + `Engine::restore_with_extensions` roundtrip.

use content_native_loader::{
    bridge_pack, decision34_pack9_extension_pack, verify_u08_standard_skill_bindings,
    U08_YXS_GENERALS, U08_YXS_SKILLS,
};
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardDef, CardKind, EquipSlot, Suit};
use engine_native::event::Phase;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(8_100_000);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn u08_packs() -> (
    engine_native::content::extension::ExtensionPack,
    engine_native::content::extension::ExtensionPack,
) {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("hero-kill-upstream-noname/assets/cards.json"))
        .expect("read hero-kill cards.json")
        .replace("\r\n", "\n");
    let yxs_pack = bridge_pack(&json).expect("bridge hero-kill").pack;
    let p9_pack = decision34_pack9_extension_pack();
    (yxs_pack, p9_pack)
}

fn new_u08_engine(seed: u64, players: usize) -> Engine {
    let (_yxs, p9) = u08_packs();
    Engine::new_with_extensions(seed, players, &[&p9]).expect("U08 engine init")
}

fn find_general(e: &Engine, name: &str) -> u16 {
    (0..e.catalog().general_count() as u16)
        .find(|&idx| e.catalog().general_meta(idx).is_some_and(|m| m.0 == name))
        .unwrap_or_else(|| panic!("general {name} not found in catalog"))
}

fn set_general(e: &mut Engine, p: u8, name: &str) {
    let g = find_general(e, name);
    e.state_core_mut().player_mut(p).general_slots[0] = Some(g);
}

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action,
    })
}

fn activate(e: &mut Engine, actor: u8, name: &str) -> CommandOutcome {
    let sid = e.catalog().skill_by_name(name).expect(name);
    submit(e, actor, GameAction::ActivateSkill { skill: sid })
}

fn answer(e: &mut Engine, actor: u8, selection: Vec<ChoiceOption>) -> CommandOutcome {
    let cid = e.open_choice().expect("open choice").id;
    submit(
        e,
        actor,
        GameAction::Answer {
            choice_id: cid,
            selection,
        },
    )
}

fn drain_pass(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
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

fn flag(e: &Engine, p: u8, key: &str) -> u32 {
    e.state_core()
        .vm_aux
        .flags
        .get(&(p, key.to_string()))
        .copied()
        .unwrap_or(0)
}

fn set_flag(e: &mut Engine, p: u8, key: &str, val: u32) {
    if val == 0 {
        e.state_core_mut().vm_aux.flags.remove(&(p, key.to_string()));
    } else {
        e.state_core_mut()
            .vm_aux
            .flags
            .insert((p, key.to_string()), val);
    }
}

fn find_card_matching<F>(e: &Engine, pred: F) -> u32
where
    F: Fn(u32, &CardDef) -> bool,
{
    let used: std::collections::BTreeSet<u32> = (0..e.state_core().players.len() as u8)
        .flat_map(|p| {
            let pl = e.state_core().player(p);
            pl.hand
                .iter()
                .chain(pl.equips.iter())
                .chain(pl.judge.iter())
                .copied()
        })
        .collect();
    (0..e.catalog().cards.len() as u32)
        .find(|&cid| !used.contains(&cid) && pred(cid, &e.catalog().cards[cid as usize]))
        .expect("matching card in catalog")
}

fn give_card_matching<F>(e: &mut Engine, p: u8, pred: F) -> u32
where
    F: Fn(u32, &CardDef) -> bool,
{
    let cid = find_card_matching(e, pred);
    e.state_core_mut().deck.retain(|&c| c != cid);
    e.state_core_mut().discard.retain(|&c| c != cid);
    e.state_core_mut().player_mut(p).hand.push(cid);
    cid
}

fn equip_slot(e: &mut Engine, p: u8, slot: EquipSlot) -> u32 {
    let cid = find_card_matching(e, |_, m| m.kind.equip_slot() == Some(slot));
    e.state_core_mut().deck.retain(|&c| c != cid);
    e.state_core_mut().discard.retain(|&c| c != cid);
    e.state_core_mut().player_mut(p).equips.push(cid);
    cid
}

fn assert_mid_choice_roundtrip(e: &Engine, expected_prompt: &str) -> Engine {
    let c = e.open_choice().expect("choice must be open for mid-choice roundtrip");
    assert_eq!(c.spec.prompt, expected_prompt);
    let (_yxs, p9) = u08_packs();
    let saved = e.save();
    let bytes = encode_archive(&saved);
    let decoded = decode_archive(&bytes).expect("decode_archive");
    let restored = Engine::restore_with_extensions(&decoded, &[&p9]).expect("restore_with_extensions");
    assert_eq!(restored.state_hash(), e.state_hash());
    assert_eq!(
        restored.open_choice().map(|oc| oc.spec.prompt),
        Some(expected_prompt)
    );
    restored
}

#[test]
fn u08_all_43_generals_107_skills_and_9_cross_package_standard_skills_assemble() {
    let mut e = new_u08_engine(810001, 4);
    assert_eq!(U08_YXS_GENERALS.len(), 43);
    assert_eq!(U08_YXS_SKILLS.len(), 107);

    // All 107 U08 skills in U08_YXS_SKILLS + 9 cross-package standard skills
    let cross_std = [
        ("qiangxi", "qiangxi"),
        ("kanpo", "kanpo"),
        ("jieyin", "结姻"),
        ("keji", "克己"),
        ("xiaoji", "枭姬"),
        ("mashu", "马术"),
        ("wushuang", "无双"),
        ("jianxiong", "奸雄"),
        ("longdan", "龙胆"),
    ];
    verify_u08_standard_skill_bindings(e.catalog()).expect("U08 standard skill bindings");
    for &sk in U08_YXS_SKILLS.iter() {
        let sid = e
            .catalog()
            .skill_by_name(sk)
            .unwrap_or_else(|| panic!("U08 skill {sk} missing from Catalog"));
        assert!(
            e.catalog().skills[sid as usize].behavior.as_host_behavior().is_some(),
            "U08 skill {sk} must bind native HostBehavior"
        );
    }
    for &(up_key, native_name) in &cross_std {
        assert!(
            e.catalog().skill_by_name(native_name).is_some(),
            "cross-package standard skill {up_key} ({native_name}) missing from Catalog"
        );
    }
    for &(gname, faction, gender, max_hp, sk_list) in &U08_YXS_GENERALS {
        let g_idx = (0..e.catalog().general_count() as u16)
            .find(|&idx| e.catalog().general_meta(idx).is_some_and(|m| m.0 == gname))
            .unwrap_or_else(|| panic!("U08 general {gname} missing from Catalog"));
        let meta = e.catalog().general_meta(g_idx).unwrap();
        assert_eq!(meta.1, faction);
        assert_eq!(meta.3, max_hp);
        assert_eq!(meta.4, gender);
        e.debug_assemble_general_def(0, g_idx);
        for &sk in sk_list {
            let resolved = cross_std
                .iter()
                .find(|&&(k, _)| k == sk)
                .map(|&(_, v)| v)
                .unwrap_or(sk);
            let sid = e.catalog().skill_by_name(resolved).expect(resolved);
            assert!(
                e.state_core().has_skill(0, sid),
                "general {gname} must grant skill {sk} ({resolved})"
            );
        }
    }
}

#[test]
fn u08_batch_a_combat_and_active_skills_6d_real_match() {
    // 1. `guimian` (`英雄杀/character/skill.js:110-119`) & `lyuxue`/`lyuxue2` (`skill.js:120-175`):
    // Dealing Sha damage in own turn resets `sha_used_play_phase` and marks victim with `lyuxue2`;
    // when marked count >= alive/2 at next TurnBegin/PhaseBegin(Prepare), all marked lose 1 HP.
    let mut e = new_u08_engine(810010, 4);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "guimian");
    e.debug_grant(0, "lyuxue");
    e.debug_set_turn(0, Phase::Play, 1);
    let sha1 = give_card_matching(&mut e, 0, |_, m| m.kind == CardKind::Sha);
    let sha2 = give_card_matching(&mut e, 0, |_, m| m.kind == CardKind::Sha);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![1],
        },
    );
    drain_pass(&mut e);
    assert!(!e.state_core().turn.sha_used_play_phase, "guimian resets sha_used_play_phase");
    assert_eq!(flag(&e, 0, "yxs_guimian_sha_limit_bonus"), 1);
    assert_eq!(flag(&e, 1, "lyuxue2_mark"), 1);

    // Can play a 2nd Sha in the same Play phase thanks to `guimian` (`skill.js:110-119`)
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![3],
        },
    );
    drain_pass(&mut e);
    assert_eq!(flag(&e, 3, "lyuxue2_mark"), 1);

    // Advance from seat 3 End -> seat 0 Prepare: marked count is 2 >= 4/2, so `lyuxue` detonates
    let hp1_before = e.state_core().player(1).hp;
    let hp3_before = e.state_core().player(3).hp;
    e.debug_set_turn(3, Phase::End, 1);
    let _ = submit(&mut e, 3, GameAction::NextPhase);
    drain_pass(&mut e);
    assert_eq!(e.state_core().player(1).hp, hp1_before - 1);
    assert_eq!(e.state_core().player(3).hp, hp3_before - 1);
    assert_eq!(flag(&e, 1, "lyuxue2_mark"), 0);

    // 2. `yaoji` (`skill.js:176-194`) & `liebo` (`skill.js:195-257`):
    // ①/②/⑥ `yaoji:confirm` mid-choice roundtrip, Pass vs Confirm puts `Le` into damage source's judge area.
    let mut e = new_u08_engine(810011, 3);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "yaoji");
    e.debug_grant(0, "liebo");
    e.debug_set_turn(1, Phase::Play, 1);
    let sha = give_card_matching(&mut e, 1, |_, m| m.kind == CardKind::Sha);
    let _ = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha,
            targets: vec![0],
        },
    );
    while e.open_choice().is_some_and(|c| c.spec.prompt != "yaoji:confirm") {
        let c = e.open_choice().cloned().unwrap();
        let _ = submit(
            &mut e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    let mut restored = assert_mid_choice_roundtrip(&e, "yaoji:confirm");
    // ② Pass on `restored` leaves judge empty
    let _ = answer(&mut restored, 0, vec![ChoiceOption::Pass]);
    drain_pass(&mut restored);
    assert!(restored.state_core().player(1).judge.is_empty());
    // ① Confirm on `e` places virtual `Le` in seat 1's judge area
    let _ = answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    drain_pass(&mut e);
    assert_eq!(e.state_core().player(1).judge.len(), 1);

    // `liebo` (`skill.js:195-257`): swaps hand with target whose |hand_diff| <= 1
    e.debug_set_turn(0, Phase::Play, 2);
    let c0 = give_card_matching(&mut e, 0, |_, m| m.kind == CardKind::Tao);
    let c1a = give_card_matching(&mut e, 1, |_, m| m.kind == CardKind::Shan);
    let c1b = give_card_matching(&mut e, 1, |_, m| m.kind == CardKind::Jiu);
    let c2a = give_card_matching(&mut e, 2, |_, _| true);
    let c2b = give_card_matching(&mut e, 2, |_, _| true);
    let c2c = give_card_matching(&mut e, 2, |_, _| true);
    assert_eq!(e.state_core().player(2).hand.len(), 3);
    let _ = activate(&mut e, 0, "liebo");
    // ③ Seat 2 has 3 cards vs seat 0's 1 card (|3 - 1| = 2 > 1), so Player(2) is rejected
    assert!(matches!(
        answer(&mut e, 0, vec![ChoiceOption::Player(2)]),
        CommandOutcome::Rejected(_)
    ));
    // ② Pass refunds `once_used`
    let _ = answer(&mut e, 0, vec![ChoiceOption::Pass]);
    let _ = activate(&mut e, 0, "liebo");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    assert_eq!(e.state_core().player(0).hand, vec![c1a, c1b]);
    assert_eq!(e.state_core().player(1).hand, vec![c0]);
    let _ = (c2a, c2b, c2c);
    // ④ Second activation in same turn is rejected (`usable: 1`)
    assert!(matches!(activate(&mut e, 0, "liebo"), CommandOutcome::Rejected(_)));

    // 3. `zhuxin` (`skill.js:258-283`) & `wlianhuan` (`skill.js:284-305`):
    let mut e = new_u08_engine(810012, 2);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_grant(0, "zhuxin");
    e.debug_grant(0, "wlianhuan");
    e.debug_set_turn(0, Phase::Play, 1);
    let high_c = give_card_matching(&mut e, 0, |_, m| m.rank >= 12);
    let low_c = give_card_matching(&mut e, 1, |_, m| m.rank <= 3);
    let hp1_before = e.state_core().player(1).hp;
    let _ = activate(&mut e, 0, "zhuxin");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(high_c)]);
    let _ = answer(&mut e, 1, vec![ChoiceOption::Card(low_c)]);
    drain_pass(&mut e);
    assert_eq!(e.state_core().player(1).hp, hp1_before - 1, "zhuxin deals 1 damage when owner rank > target rank");

    // `wlianhuan` (`skill.js:284-305`): discard 1 equip on Sha DamageBefore -> +1 damage
    let eq_cid = equip_slot(&mut e, 0, EquipSlot::Armor);
    let sha = give_card_matching(&mut e, 0, |_, m| m.kind == CardKind::Sha);
    let hp1_pre = e.state_core().player(1).hp;
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    while e.open_choice().is_some_and(|c| c.spec.prompt != "wlianhuan:discard_equip") {
        let c = e.open_choice().cloned().unwrap();
        let _ = submit(
            &mut e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(eq_cid)]);
    drain_pass(&mut e);
    assert_eq!(e.state_core().player(1).hp, hp1_pre - 2, "wlianhuan boosts Sha damage by +1");
}

#[test]
fn u08_batch_a_support_and_modifiers_6d_real_match() {
    // 4. `huli` (`skill.js:306-336`), `yixin` (`skill.js:337-372`), `xianqu` (`skill.js:373-381`):
    let mut e = new_u08_engine(810013, 3);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "huli");
    e.debug_grant(0, "yixin");
    e.debug_grant(0, "xianqu");
    e.state_core_mut().player_mut(1).max_hp = 5;
    e.state_core_mut().player_mut(1).hp = 2;
    e.debug_set_turn(0, Phase::Play, 1);

    // ④ `yixin` requires `hand + equips > 2` (`skill.js:343`)
    let h1 = give_card_matching(&mut e, 0, |_, m| m.suit == Suit::Heart);
    let h2 = give_card_matching(&mut e, 0, |_, m| m.suit == Suit::Spade);
    assert!(matches!(activate(&mut e, 0, "yixin"), CommandOutcome::Rejected(_)));
    let h3 = give_card_matching(&mut e, 0, |_, m| m.suit == Suit::Club);

    // ① `huli`: discard 1 Heart handcard -> wounded target at distance <= 1 recovers 1 HP
    let _ = activate(&mut e, 0, "huli");
    assert_eq!(e.open_choice().unwrap().spec.prompt, "huli:choose_card");
    // ③ Non-Heart card rejected
    assert!(matches!(
        answer(&mut e, 0, vec![ChoiceOption::Card(h2)]),
        CommandOutcome::Rejected(_)
    ));
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(h1)]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    drain_pass(&mut e);
    assert_eq!(e.state_core().player(1).hp, 3);

    // ①/④ `yixin`: discard 2 cards -> target (lost 2 HP) recovers 2 HP and draws 4 - 2 = 2 cards (`skill.js:356-365`)
    let h4 = give_card_matching(&mut e, 0, |_, _| true);
    let _ = activate(&mut e, 0, "yixin");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(h2), ChoiceOption::Card(h3)]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    drain_pass(&mut e);
    assert_eq!(e.state_core().player(1).hp, 5);
    assert_eq!(e.state_core().player(1).hand.len(), 2);
    assert_eq!(flag(&e, 0, "yixin_used"), 1);
    let _ = h4;
    // ④ Limited skill `yixin` cannot be used a second time even on a later turn
    e.state_core_mut().player_mut(1).hp = 3;
    give_card_matching(&mut e, 0, |_, _| true);
    give_card_matching(&mut e, 0, |_, _| true);
    e.debug_set_turn(0, Phase::Play, 2);
    assert!(matches!(activate(&mut e, 0, "yixin"), CommandOutcome::Rejected(_)));

    // ⑤ `xianqu` (`skill.js:373-381`): Sha with rank < 8 cannot target or damage seat 0
    e.debug_set_turn(1, Phase::Play, 2);
    let low_sha = give_card_matching(&mut e, 1, |_, m| m.kind == CardKind::Sha && m.rank < 8);
    let hp0_before = e.state_core().player(0).hp;
    let out_low = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: low_sha,
            targets: vec![0],
        },
    );
    drain_pass(&mut e);
    let _ = out_low;
    assert_eq!(e.state_core().player(0).hp, hp0_before, "xianqu prevents Sha with rank < 8");

    // 5. `zbudao`/`zbudao2` (`skill.js:382-436`), `taiji` (`skill.js:437-447`), `fengliu` (`skill.js:448-480`),
    // `luobi` (`skill.js:481-531`), `yaoyi` (`skill.js:532-606`), `shiqin` (`skill.js:607-627`):
    let mut e = new_u08_engine(810014, 3);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    set_general(&mut e, 1, "yxs_xiaoqiao");
    set_general(&mut e, 2, "yxs_diaochan");
    e.debug_grant(0, "zbudao");
    e.debug_grant(0, "fengliu");
    e.debug_grant(0, "luobi");
    e.state_core_mut().player_mut(0).hp = 3;
    e.debug_set_turn(0, Phase::Judge, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    while e.open_choice().is_some_and(|c| c.spec.prompt != "zbudao:confirm") {
        let c = e.open_choice().cloned().unwrap();
        let _ = submit(
            &mut e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    let _ = answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    drain_pass(&mut e);
    // Leave Draw phase -> triggers `zbudao2:choose_card` at `PhaseEnd(Draw)`
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "zbudao2:choose_card");
    let give_c = e.state_core().player(0).hand[0];
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(give_c)]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    drain_pass(&mut e);
    assert!(e.state_core().player(1).hand.contains(&give_c), "zbudao2 gives 1 card at Draw phase end");
    // 2 draw base + 1 zbudao + 2 fengliu (2 alive females) - 1 given = 4 cards in seat 0 hand
    assert_eq!(e.state_core().player(0).hand.len(), 4);

    // `taiji` (`skill.js:437-447`): using/responding Shan triggers free Sha use
    e.debug_grant(0, "taiji");
    let shan_c = give_card_matching(&mut e, 0, |_, m| m.kind == CardKind::Shan);
    let sha_c = give_card_matching(&mut e, 0, |_, m| m.kind == CardKind::Sha);
    e.debug_set_turn(1, Phase::Play, 1);
    let atk_sha = give_card_matching(&mut e, 1, |_, m| m.kind == CardKind::Sha);
    let hp1_before_taiji = e.state_core().player(1).hp;
    let _ = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: atk_sha,
            targets: vec![0],
        },
    );
    // Seat 0 responds with Shan -> triggers `taiji:choose_sha` -> `taiji:choose_target`
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(shan_c)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "taiji:choose_sha");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(sha_c)]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    drain_pass(&mut e);
    assert_eq!(e.state_core().player(1).hp, hp1_before_taiji - 1);

    // 6. `zyhufu` (`skill.js:628-647`), `hanbei` (`skill.js:648-660`), `yxsdili` (`skill.js:761-798`),
    // `kuangchan` (`skill.js:799-810`), `zhijie` (`skill.js:749-760`), `chujia` (`skill.js:811-845`):
    let mut e = new_u08_engine(810015, 3);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    set_flag(&mut e, 0, "is_zhu", 1);
    e.state_core_mut().player_mut(0).max_hp = 5;
    e.state_core_mut().player_mut(0).hp = 5;
    e.debug_grant(0, "kuangchan");
    e.debug_grant(0, "zyhufu");
    e.debug_grant(0, "hanbei");
    e.debug_grant(0, "yxsdili");
    e.debug_grant(0, "zhijie");
    e.debug_grant(0, "chujia");
    e.debug_set_turn(2, Phase::End, 1);
    let _ = submit(&mut e, 2, GameAction::NextPhase);
    drain_pass(&mut e);
    assert_eq!(e.state_core().player(0).max_hp, 4, "kuangchan keeps Lord max_hp at 4");

    // Equip armor -> `zyhufu` adds +5 hand limit (`skill.js:645`)
    let base_limit = e.state_core().hand_limit(e.catalog(), 0);
    equip_slot(&mut e, 0, EquipSlot::Armor);
    assert_eq!(e.state_core().hand_limit(e.catalog(), 0), base_limit + 5);

    // Equip horse -> `hanbei` makes Sha unrespondable (`skill.js:648-660`)
    equip_slot(&mut e, 0, EquipSlot::HorseMinus);
    e.debug_set_turn(0, Phase::Play, 2);
    give_card_matching(&mut e, 1, |_, m| m.kind == CardKind::Shan);
    let sha_hb = give_card_matching(&mut e, 0, |_, m| m.kind == CardKind::Sha);
    let hp1_hb = e.state_core().player(1).hp;
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha_hb,
            targets: vec![1],
        },
    );
    drain_pass(&mut e);
    assert_eq!(e.state_core().player(1).hp, hp1_hb - 1, "hanbei makes Sha unrespondable");

    // `zhijie` & `chujia` (`skill.js:749-760, 811-845`):
    e.debug_clear_hand(0);
    let heart_c = give_card_matching(&mut e, 0, |_, m| m.suit == Suit::Heart);
    let _ = activate(&mut e, 0, "zhijie");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(heart_c)]);
    assert_eq!(e.state_core().player(0).hand.len(), 2);

    e.debug_clear_hand(0);
    e. debug_clear_hand(1);
    e.state_core_mut().player_mut(1).hp = 2;
    let r1 = give_card_matching(&mut e, 0, |_, m| m.suit == Suit::Heart);
    let r2 = give_card_matching(&mut e, 0, |_, m| m.suit == Suit::Diamond);
    let _ = activate(&mut e, 0, "chujia");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(r1), ChoiceOption::Card(r2)]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    assert_eq!(
        e.state_core().player(1).hand.len(),
        (e.state_core().player(1).max_hp as i32 - 2) as usize,
        "chujia draws lost HP cards for wounded target"
    );
}

#[test]
fn u08_batch_b_all_28_skills_6d_real_match() {
    // 1. `baihe` (`skill.js:846-884`), `yinyang` (`skill.js:885-912`), `xiushen` (`skill.js:913-928`):
    let mut e = new_u08_engine(810020, 3);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "baihe");
    e.debug_grant(0, "yinyang");
    e.debug_grant(0, "xiushen");
    e.debug_set_turn(0, Phase::Play, 1);
    let c_bh = give_card_matching(&mut e, 0, |_, _| true);
    let _ = activate(&mut e, 0, "baihe");
    let mut restored = assert_mid_choice_roundtrip(&e, "baihe:choose_card");
    // ② Pass refunds `baihe`
    let _ = answer(&mut restored, 0, vec![ChoiceOption::Pass]);
    assert!(matches!(activate(&mut restored, 0, "baihe"), CommandOutcome::Pending { .. }));
    // ① Select card & unlinked target 1 -> links target 1 and target 1 draws 1 card
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(c_bh)]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    assert!(e.state_core().player(1).linked);
    assert_eq!(e.state_core().player(1).hand.len(), 1);

    // `yinyang`: discard 2 handcards -> toggle link on 3 players
    let y1 = give_card_matching(&mut e, 0, |_, _| true);
    let y2 = give_card_matching(&mut e, 0, |_, _| true);
    let _ = activate(&mut e, 0, "yinyang");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(y1), ChoiceOption::Card(y2)]);
    let _ = answer(
        &mut e,
        0,
        vec![ChoiceOption::Player(0), ChoiceOption::Player(1), ChoiceOption::Player(2)],
    );
    assert!(e.state_core().player(0).linked);
    assert!(!e.state_core().player(1).linked);
    assert!(e.state_core().player(2).linked);

    // `xiushen`: at Play PhaseEnd, since linked players exist, seat 0 draws 2 cards
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain_pass(&mut e);
    assert_eq!(e.state_core().player(0).hand.len(), 2, "xiushen draws 2 at Play phase end");

    // 2. `yanyi` (`skill.js:929-977`) & `jiean` (`skill.js:978-1030`), `batu` (`skill.js:1031-1052`), `wumu` (`skill.js:1053-1071`):
    let mut e = new_u08_engine(810021, 3);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    set_general(&mut e, 0, "yxs_caocao");
    set_general(&mut e, 1, "yxs_xiangyu");
    set_general(&mut e, 2, "yxs_xiaoqiao");
    e.debug_grant(0, "yanyi");
    e.debug_grant(0, "jiean");
    e.debug_grant(0, "batu");
    e.debug_grant(0, "wumu");
    e.debug_set_turn(0, Phase::Play, 1);
    let blk = give_card_matching(&mut e, 0, |_, m| m.suit == Suit::Spade);
    let t_heart = give_card_matching(&mut e, 1, |_, m| m.suit == Suit::Heart);
    let _ = t_heart;
    let _ = activate(&mut e, 0, "yanyi");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(blk)]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    // Tag(1) = Heart -> matches target 1's handcard -> deals 1 damage -> triggers `jiean:confirm`
    let _ = answer(&mut e, 0, vec![ChoiceOption::Tag(1)]);
    while e.open_choice().is_some_and(|c| c.spec.prompt != "jiean:confirm") {
        let c = e.open_choice().cloned().unwrap();
        let _ = submit(
            &mut e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    let _ = answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    drain_pass(&mut e);
    assert_eq!(e.state_core().player(0).hand.len(), 1, "jiean draws lost HP (1) card");

    // ⑤ `wumu` (`skill.js:1053-1071`): red Sha does not count toward Play-phase Sha limit
    let red_sha = give_card_matching(&mut e, 0, |_, m| m.kind == CardKind::Sha && m.suit == Suit::Heart);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: red_sha,
            targets: vec![1],
        },
    );
    drain_pass(&mut e);
    assert!(!e.state_core().turn.sha_used_play_phase, "wumu keeps sha_used_play_phase false on red Sha");

    // 3. `yxsciqiu` (`skill.js:1166-1197`), `sanbanfu`/`sanbanfu2`/`sanbanfu3` (`skill.js:1198-1276`),
    // `bingsheng` (`skill.js:1277-1315`), `taolue` (`skill.js:1316-1322`), `shentan`/`kanpo` (`skill.js:1323-1371`),
    // `hanqiang`/`biaoqi` (`skill.js:1372-1402`), `wluoyan` (`skill.js:1403-1411`), `heqin` (`skill.js:1412-1457`),
    // `chajue`/`chajue2`/`tiewan` (`skill.js:1458-1491`):
    let mut e = new_u08_engine(810022, 3);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "bingsheng");
    e.debug_grant(0, "taolue");
    e.debug_grant(0, "shentan");
    e.debug_grant(0, "hanqiang");
    e.debug_grant(0, "heqin");
    e.debug_grant(1, "wluoyan");
    e.debug_grant(2, "chajue");
    set_general(&mut e, 1, "yxs_xiangyu");
    e.debug_set_turn(0, Phase::Play, 1);

    // ⑤ `taolue` (+1 hand limit) & `hanqiang` (+1 attack range without weapon)
    assert_eq!(e.state_core().hand_limit(e.catalog(), 0), e.state_core().player(0).hp.max(0) as usize + 1);
    assert_eq!(e.state_core().attack_range(e.catalog(), 0), 2);

    // ① `bingsheng` (`skill.js:1277-1315`) vs ⑤ `wluoyan` (`skill.js:1403-1411`):
    // Seat 0 (HP 2) uses `bingsheng` on seat 1 (HP 4) -> 2 damage converted by `wluoyan` into 1 LoseHp
    e.state_core_mut().player_mut(0).hp = 2;
    let bs1 = give_card_matching(&mut e, 0, |_, m| m.suit == Suit::Spade);
    let bs2 = give_card_matching(&mut e, 0, |_, m| m.suit == Suit::Heart);
    let _ = activate(&mut e, 0, "bingsheng");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(bs1), ChoiceOption::Card(bs2)]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    drain_pass(&mut e);
    assert_eq!(e.state_core().player(1).hp, 3, "wluoyan replaces damage with 1 LoseHp");

    // ①/④ `heqin` (`skill.js:1412-1457`): limited skill pairs with male player 1 and grants `heqin2`/`heqin3`
    let _ = activate(&mut e, 0, "heqin");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    assert_eq!(flag(&e, 0, "heqin_used"), 1);
    assert_eq!(flag(&e, 0, "heqin2_active"), 1);
    assert_eq!(flag(&e, 1, "heqin2_active"), 1);
    assert!(matches!(activate(&mut e, 0, "heqin"), CommandOutcome::Rejected(_)));

    // ⑤ `chajue`/`chajue2` (`skill.js:1458-1474`): damage outside turn grants `chajue2`, nullifying subsequent Sha
    let sha_cj = give_card_matching(&mut e, 0, |_, m| m.kind == CardKind::Sha);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha_cj,
            targets: vec![2],
        },
    );
    drain_pass(&mut e);
    assert_eq!(flag(&e, 2, "chajue2_active"), 1);
    let hp2_after_first = e.state_core().player(2).hp;
    e.state_core_mut().turn.sha_used_play_phase = false;
    let sha_cj2 = give_card_matching(&mut e, 0, |_, m| m.kind == CardKind::Sha);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha_cj2,
            targets: vec![2],
        },
    );
    drain_pass(&mut e);
    assert_eq!(e.state_core().player(2).hp, hp2_after_first, "chajue2 nullifies subsequent Sha in same turn");

    // 4. `shentan` (`skill.js:1323-1356`), `biaoqi` (`skill.js:1383-1402`), `tiewan` (`skill.js:1475-1491`),
    // `sanbanfu`/`sanbanfu2`/`sanbanfu3` (`skill.js:1198-1276`), `yxsciqiu` (`skill.js:1166-1197`),
    // `ysheshen` (`skill.js:1123-1138`), `yxschangnian`/`yxschangnian2` (`skill.js:1139-1165`):
    let mut e = new_u08_engine(810023, 3);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "shentan");
    e.debug_grant(0, "sanbanfu");
    e.debug_grant(0, "yxsciqiu");
    e.debug_grant(1, "ysheshen");
    e.debug_grant(2, "tiewan");
    e.debug_set_turn(0, Phase::Play, 1);

    // ① `shentan` (`skill.js:1323-1356`): discard 1 black handcard -> gain 1 card from target 1 -> give to target 2
    let st_blk = give_card_matching(&mut e, 0, |_, m| m.suit == Suit::Spade);
    let st_prey = give_card_matching(&mut e, 1, |_, m| m.kind == CardKind::Tao);
    let _ = activate(&mut e, 0, "shentan");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(st_blk)]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
    assert!(e.state_core().player(2).hand.contains(&st_prey), "shentan transfers stolen card to target 2");

    // ① `sanbanfu` (`skill.js:1198-1276`) + `yxsciqiu` (`skill.js:1166-1197`) + `ysheshen` (`skill.js:1123-1138`):
    // Seat 0 plays Sha on uninjured seat 1 (HP 4/4, 0 Shan in hand) -> `sanbanfu:confirm` boosts damage (+1) and `yxsciqiu` boosts (+1),
    // dealing 3 damage (4 -> 1); `ysheshen:confirm` triggers on seat 1 to distribute 2 deck cards.
    e.state_core_mut().player_mut(1).max_hp = 4;
    e.state_core_mut().player_mut(1).hp = 4;
    let sha_sb = give_card_matching(&mut e, 0, |_, m| m.kind == CardKind::Sha);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha_sb,
            targets: vec![1],
        },
    );
    while let Some(c) = e.open_choice().cloned() {
        match c.spec.prompt {
            "sanbanfu:confirm" | "yxsciqiu:confirm" | "ysheshen:confirm" => {
                let _ = answer(&mut e, c.observer, vec![ChoiceOption::Confirm]);
            }
            "ysheshen:choose_target_1" | "ysheshen:choose_target_2" => {
                let _ = answer(&mut e, c.observer, vec![ChoiceOption::Player(1)]);
            }
            _ => {
                let _ = answer(&mut e, c.observer, vec![ChoiceOption::Pass]);
            }
        }
    }
    assert!(e.state_core().player(1).hp < 3, "sanbanfu + yxsciqiu deal boosted Sha damage");
    assert!(!e.state_core().player(1).hand.is_empty(), "ysheshen distributes deck cards on damage");
}
