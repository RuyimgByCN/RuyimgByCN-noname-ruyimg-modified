//! U08 英雄杀 独立复核 Round-2 定向回归测试（覆盖 Round-1 全部 12 项 FAIL）。
//!
//! 严格经真实对局事件流（`Engine::submit` + `PlayCard`/`ActivateSkill`/`Answer`/`NextPhase`）
//! 触发，预期逐条取自上游 `archive/upstream/noname/apps/core/` 并标注 `file:line`。

use content_native_loader::decision34_pack9_extension_pack;
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardDef, CardKind, Suit};
use engine_native::content::extension::ExtensionPack;
use engine_native::event::Phase;
use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, LogEvent,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(8_900_000);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn load_pack(rel_dir: &str) -> ExtensionPack {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join(rel_dir).join("assets/cards.json"))
        .expect("read extension cards.json")
        .replace("\r\n", "\n");
    let mut pack = content_native_loader::bridge_pack(&json)
        .expect("bridge pack")
        .pack;
    let p9 = decision34_pack9_extension_pack();
    let existing_skills: std::collections::BTreeSet<&str> = p9
        .host_skills
        .iter()
        .map(|s| s.name)
        .chain(p9.vm_skills.iter().map(|s| s.name.as_str()))
        .collect();
    let existing_generals: std::collections::BTreeSet<&str> =
        p9.generals.iter().map(|g| g.name).collect();
    pack.host_skills
        .retain(|s| !existing_skills.contains(s.name));
    pack.vm_skills
        .retain(|s| !existing_skills.contains(s.name.as_str()));
    pack.generals
        .retain(|g| !existing_generals.contains(g.name));
    pack
}

fn new_u08_engine(seed: u64, players: usize) -> Engine {
    let p9 = decision34_pack9_extension_pack();
    Engine::new_with_extensions(seed, players, &[&p9]).expect("U08 engine init")
}

fn new_u08_engine_with_extra(seed: u64, players: usize, extra: &ExtensionPack) -> Engine {
    let p9 = decision34_pack9_extension_pack();
    Engine::new_with_extensions(seed, players, &[&p9, extra]).expect("U08+extra engine init")
}

fn set_general(e: &mut Engine, p: u8, name: &str) {
    let g = (0..e.catalog().general_count() as u16)
        .find(|&idx| e.catalog().general_meta(idx).is_some_and(|m| m.0 == name))
        .unwrap_or_else(|| panic!("general {name} not found in catalog"));
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

fn equip_matching<F>(e: &mut Engine, p: u8, pred: F) -> u32
where
    F: Fn(u32, &CardDef) -> bool,
{
    let cid = find_card_matching(e, pred);
    e.state_core_mut().deck.retain(|&c| c != cid);
    e.state_core_mut().discard.retain(|&c| c != cid);
    e.state_core_mut().player_mut(p).equips.push(cid);
    cid
}

fn put_on_deck_top<F>(e: &mut Engine, pred: F) -> u32
where
    F: Fn(u32, &CardDef) -> bool,
{
    let cid = find_card_matching(e, pred);
    e.state_core_mut().deck.retain(|&c| c != cid);
    e.state_core_mut().discard.retain(|&c| c != cid);
    e.state_core_mut().deck.push(cid);
    cid
}

fn count_draws(e: &Engine, from_log: usize, player: u8) -> Vec<usize> {
    e.log()[from_log..]
        .iter()
        .filter_map(|ev| match ev {
            LogEvent::CardsDrawn { player: p, cards } if *p == player => Some(cards.len()),
            _ => None,
        })
        .collect()
}

fn assert_mid_choice_roundtrip_extra(
    e: &Engine,
    expected_prompt: &str,
    extra: Option<&ExtensionPack>,
) -> Engine {
    let c = e.open_choice().expect("choice must be open for roundtrip");
    assert_eq!(c.spec.prompt, expected_prompt);
    let p9 = decision34_pack9_extension_pack();
    let saved = e.save();
    let bytes = encode_archive(&saved);
    let decoded = decode_archive(&bytes).expect("decode_archive");
    let restored = match extra {
        Some(ext) => Engine::restore_with_extensions(&decoded, &[&p9, ext]),
        None => Engine::restore_with_extensions(&decoded, &[&p9]),
    }
    .expect("restore_with_extensions");
    assert_eq!(restored.state_hash(), e.state_hash());
    assert_eq!(
        restored.open_choice().map(|oc| oc.spec.prompt),
        Some(expected_prompt)
    );
    restored
}

#[test]
fn u08_r2_heqin2_requires_actual_skill() {
    // Upstream: `英雄杀/character/skill.js:1522-1529`
    // `heqin2`: `phaseDrawBegin` `forced`: `trigger.num++` only when player actually holds `heqin2`.
    let mut e = new_u08_engine(890001, 4);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    set_general(&mut e, 0, "yxs_wangzhaojun");
    set_general(&mut e, 1, "yxs_xiangyu");
    e.debug_grant(0, "heqin");
    e.debug_set_turn(0, Phase::Play, 1);

    let _ = activate(&mut e, 0, "heqin");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    assert_eq!(flag(&e, 1, "heqin2_active"), 1);

    // Strip `heqin2` from seat 1 even while `heqin2_active` flag stays `1` -> DrawPhase must draw 2 (NOT 3).
    let hq2 = e.catalog().skill_by_name("heqin2").expect("heqin2");
    e.state_core_mut().skills.remove(&(1, hq2));
    e.debug_clear_hand(1);
    e.debug_set_turn(1, Phase::Judge, 2);
    let mark = e.log().len();
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    drain_pass(&mut e);
    assert_eq!(
        count_draws(&e, mark, 1),
        vec![2],
        "stripped heqin2 must NOT get +1 draw via heqin2_active fallback"
    );
}

#[test]
fn u08_r2_bolehuiyan_hanbing_fengxueren_damage_order_youdishenru_nested() {
    // 2a) Hanbing Sword prevents damage (`card/standard.js:149-151, 3409-3416`, CDP `1g`):
    //     `shaDamage` STILL fires -> `bolehuiyan3` resolves HIT!
    let mut e_hb = new_u08_engine(890002, 4);
    for p in 0..4 {
        e_hb.debug_clear_hand(p);
    }
    e_hb.debug_grant(0, "bolehuiyan");
    equip_matching(&mut e_hb, 1, |_, m| m.kind == CardKind::Hanbing);
    give_card_matching(&mut e_hb, 2, |_, m| m.kind == CardKind::Tao);
    give_card_matching(&mut e_hb, 2, |_, m| m.kind == CardKind::Jiu);
    let sha_hb = give_card_matching(&mut e_hb, 1, |_, m| m.kind == CardKind::Sha);
    e_hb.debug_set_turn(1, Phase::Play, 1);

    let _ = submit(
        &mut e_hb,
        1,
        GameAction::PlayCard {
            card: sha_hb,
            targets: vec![2],
        },
    );
    assert_eq!(
        e_hb.open_choice().map(|c| c.spec.prompt),
        Some("bolehuiyan:predict")
    );
    // Branch A: predict HIT (Tag(1)) -> Hanbing prevents dmg -> still draws 1; Branch B: predict MISS (Tag(2)) -> forced discard!
    let mut e_hb_miss = assert_mid_choice_roundtrip_extra(&e_hb, "bolehuiyan:predict", None);
    let _ = answer(&mut e_hb, 0, vec![ChoiceOption::Tag(1)]);
    assert_eq!(
        e_hb.open_choice().map(|c| c.spec.prompt),
        Some("hanbing_prevent_confirm")
    );
    let _ = answer(&mut e_hb, 1, vec![ChoiceOption::Confirm]);
    for _ in 0..2 {
        assert_eq!(
            e_hb.open_choice().map(|c| c.spec.prompt),
            Some("hanbing_target_discard")
        );
        let first = e_hb.open_choice().unwrap().spec.options[0];
        let _ = answer(&mut e_hb, 2, vec![first]);
    }
    drain_pass(&mut e_hb);
    assert_eq!(e_hb.state_core().player(2).hp, 4, "Hanbing prevented HP loss");
    assert_eq!(e_hb.state_core().player(2).hand.len(), 0);
    assert_eq!(
        e_hb.state_core().player(0).hand.len(),
        1,
        "Hanbing-prevented Sha still triggers shaDamage so hit prediction draws 1"
    );

    give_card_matching(&mut e_hb_miss, 0, |_, m| m.kind == CardKind::Tao);
    let _ = answer(&mut e_hb_miss, 0, vec![ChoiceOption::Tag(2)]);
    let _ = answer(&mut e_hb_miss, 1, vec![ChoiceOption::Confirm]);
    for _ in 0..2 {
        let first = e_hb_miss.open_choice().unwrap().spec.options[0];
        let _ = answer(&mut e_hb_miss, 2, vec![first]);
    }
    drain_pass(&mut e_hb_miss);
    assert_eq!(
        e_hb_miss.state_core().player(0).hand.len(),
        0,
        "Hanbing-prevented Sha settles hit -> miss prediction discards 1"
    );

    // 2b) Fengxueren prevents damage at `shaHit` (`杀海拾遗/card/swd/skill.js:2284-2312` + `card/standard.js:246, 282-286`):
    //     `trigger.unhurt = true` skips `shaDamage` and settles `shaEnd` -> Miss prediction (`Tag(2)`) draws 1!
    let swd_pack = load_pack("shahai-swd-upstream-noname");
    let mut e_fx = new_u08_engine_with_extra(890003, 4, &swd_pack);
    for p in 0..4 {
        e_fx.debug_clear_hand(p);
    }
    e_fx.debug_grant(0, "bolehuiyan");
    equip_matching(&mut e_fx, 1, |_, m| m.kind == CardKind::Fengxueren);
    give_card_matching(&mut e_fx, 2, |_, m| m.kind == CardKind::Tao);
    let sha_fx = give_card_matching(&mut e_fx, 1, |_, m| m.kind == CardKind::Sha);
    e_fx.debug_set_turn(1, Phase::Play, 1);

    let _ = submit(
        &mut e_fx,
        1,
        GameAction::PlayCard {
            card: sha_fx,
            targets: vec![2],
        },
    );
    assert_eq!(
        e_fx.open_choice().map(|c| c.spec.prompt),
        Some("bolehuiyan:predict")
    );
    let _ = answer(&mut e_fx, 0, vec![ChoiceOption::Tag(2)]);
    assert_eq!(
        e_fx.open_choice().map(|c| c.spec.prompt),
        Some("fengxueren_flip_confirm")
    );
    let _ = answer(&mut e_fx, 1, vec![ChoiceOption::Confirm]);
    drain_pass(&mut e_fx);
    assert!(e_fx.state_core().player(2).face_down, "Fengxueren flipped target");
    assert_eq!(
        e_fx.state_core().player(2).hand.len(),
        2,
        "Fengxueren target drew 1 after TurnOverBefore"
    );
    assert_eq!(
        e_fx.state_core().player(0).hand.len(),
        1,
        "Fengxueren sets unhurt=true -> shaEnd -> miss prediction draws 1"
    );
    assert_eq!(flag(&e_fx, 0, "bolehuiyan_pred"), 0);

    // 2c) `shaDamage` ordering AFTER `Damage` settles (`card/standard.js:149-151`):
    //     Linked fire Sha kills predictor seat 0 -> dead seat 0 must NOT draw/trigger `bolehuiyan3` after dying,
    //     and surviving `DyingSave` pause mid-choice archive roundtrip must work cleanly!
    let mut e_ord = new_u08_engine(890004, 4);
    for p in 0..4 {
        e_ord.debug_clear_hand(p);
    }
    e_ord.debug_grant(0, "bolehuiyan");
    e_ord.state_core_mut().player_mut(0).hp = 1;
    e_ord.state_core_mut().player_mut(0).linked = true;
    e_ord.state_core_mut().player_mut(2).linked = true;
    give_card_matching(&mut e_ord, 2, |_, m| m.kind == CardKind::Jiu);
    let fsha = give_card_matching(&mut e_ord, 1, |_, m| m.kind == CardKind::FireSha);
    e_ord.debug_set_turn(1, Phase::Play, 1);

    let mark_ord = e_ord.log().len();
    let _ = submit(
        &mut e_ord,
        1,
        GameAction::PlayCard {
            card: fsha,
            targets: vec![2],
        },
    );
    let _ = answer(&mut e_ord, 0, vec![ChoiceOption::Tag(1)]);
    drain_pass(&mut e_ord);
    assert!(!e_ord.state_core().player(0).alive, "chained fire Sha killed seat 0");
    assert!(
        !e_ord.log()[mark_ord..]
            .iter()
            .any(|ev| matches!(ev, LogEvent::SkillTriggered { skill, .. } if *skill == "bolehuiyan3")),
        "dead predictor must NOT trigger bolehuiyan3 after Damage kills them"
    );

    // Surviving Damage with DyingSave mid-choice roundtrip: Log `bolehuiyan3` comes strictly AFTER DamageDealt!
    let mut e_live = new_u08_engine(890005, 4);
    for p in 0..4 {
        e_live.debug_clear_hand(p);
    }
    e_live.debug_grant(0, "bolehuiyan");
    e_live.state_core_mut().player_mut(2).hp = 1;
    give_card_matching(&mut e_live, 2, |_, m| m.kind == CardKind::Tao);
    let sha_lv = give_card_matching(&mut e_live, 1, |_, m| m.kind == CardKind::Sha);
    e_live.debug_set_turn(1, Phase::Play, 1);
    let mark_lv = e_live.log().len();
    let _ = submit(
        &mut e_live,
        1,
        GameAction::PlayCard {
            card: sha_lv,
            targets: vec![2],
        },
    );
    let _ = answer(&mut e_live, 0, vec![ChoiceOption::Tag(1)]);
    assert_eq!(e_live.open_choice().map(|c| c.spec.prompt), Some("dying_save"));
    let mut e_lv_snap = assert_mid_choice_roundtrip_extra(&e_live, "dying_save", None);
    let tao2 = e_live.state_core().player(2).hand[0];
    let _ = answer(&mut e_live, 2, vec![ChoiceOption::Card(tao2)]);
    let _ = answer(&mut e_lv_snap, 2, vec![ChoiceOption::Card(tao2)]);
    drain_pass(&mut e_live);
    drain_pass(&mut e_lv_snap);
    assert_eq!(e_live.state_hash(), e_lv_snap.state_hash());
    assert_eq!(e_live.state_core().player(0).hand.len(), 1);
    let dmg_pos = e_live.log()[mark_lv..]
        .iter()
        .position(|ev| matches!(ev, LogEvent::DamageDealt { target: 2, .. }))
        .expect("DamageDealt logged");
    let bhy_pos = e_live.log()[mark_lv..]
        .iter()
        .position(|ev| matches!(ev, LogEvent::SkillTriggered { skill, .. } if *skill == "bolehuiyan3"))
        .expect("bolehuiyan3 logged");
    assert!(
        dmg_pos < bhy_pos,
        "bolehuiyan3 (shaDamage) must settle AFTER DamageDealt ({dmg_pos} < {bhy_pos})"
    );

    // 2d-1) Upstream `g_youdishenru` (`玩点论杀/card/skill.js:16-36`) triggers at `shaBefore` BEFORE `bolehuiyan` (`shaBegin`):
    // canceling `sha_a` at `shaBefore` skips `shaBegin`/`shaEnd`; counter-`sha_b` enters its own `shaBegin` (`bolehuiyan`):
    let wd_pack = load_pack("wudian-upstream-noname");
    let mut e_yd = new_u08_engine_with_extra(890006, 4, &wd_pack);
    for p in 0..4 {
        e_yd.debug_clear_hand(p);
    }
    e_yd.debug_grant(0, "bolehuiyan");
    let sha_a = give_card_matching(&mut e_yd, 1, |_, m| m.kind == CardKind::Sha);
    let sha_b = give_card_matching(&mut e_yd, 1, |_, m| m.kind == CardKind::Sha);
    let yd = give_card_matching(&mut e_yd, 2, |_, m| m.kind == CardKind::Youdishenru);
    let _keep_2 = give_card_matching(&mut e_yd, 2, |_, m| m.kind == CardKind::Tao);
    e_yd.debug_set_turn(1, Phase::Play, 1);
    let _ = submit(
        &mut e_yd,
        1,
        GameAction::PlayCard {
            card: sha_a,
            targets: vec![2],
        },
    );
    assert_eq!(
        e_yd.open_choice().map(|c| c.spec.prompt),
        Some("wudian_youdishenru_use")
    );
    let _ = answer(&mut e_yd, 2, vec![ChoiceOption::Card(yd)]);
    assert_eq!(
        e_yd.open_choice().map(|c| c.spec.prompt),
        Some("wudian_youdishenru_sha")
    );
    // Case A: cancel `wudian_youdishenru_sha` -> seat 1 takes 1 dmg -> cancelled `sha_a` never fired `bolehuiyan`:
    let mut e_yd_play =
        assert_mid_choice_roundtrip_extra(&e_yd, "wudian_youdishenru_sha", Some(&wd_pack));
    let _ = answer(&mut e_yd, 1, Vec::new());
    drain_pass(&mut e_yd);
    assert_eq!(e_yd.state_core().player(1).hp, 3, "youdishenru counter-damaged source");
    assert_eq!(e_yd.state_core().player(0).hand.len(), 0);
    assert_eq!(flag(&e_yd, 0, "bolehuiyan_pred"), 0);

    // Case B: counter-`sha_b` enters `shaBegin` -> `bolehuiyan` predicts Hit (Tag(1)) -> `sha_b` hits -> `bolehuiyan3` draws 1!
    let _ = answer(&mut e_yd_play, 1, vec![ChoiceOption::Card(sha_b)]);
    assert_eq!(
        e_yd_play.open_choice().map(|c| c.spec.prompt),
        Some("bolehuiyan:predict")
    );
    let _ = answer(&mut e_yd_play, 0, vec![ChoiceOption::Tag(1)]);
    drain_pass(&mut e_yd_play);
    assert_eq!(e_yd_play.state_core().player(2).hp, 3, "counter-Sha#2 hit target");
    assert_eq!(
        e_yd_play.state_core().player(0).hand.len(),
        1,
        "counter-Sha#2 settles bolehuiyan3 (hit pred draws 1)"
    );
    assert_eq!(flag(&e_yd_play, 0, "bolehuiyan_pred"), 0);

    // 2d-2) Post-`shaBegin` nested `Sha#2` (via `yxs_fanji` during `Sha#1` Damage) settles `bolehuiyan2` before `Sha#1` ends (CDP 1h):
    let mut e_nest = new_u08_engine(890008, 4);
    for p in 0..4 {
        e_nest.debug_clear_hand(p);
    }
    e_nest.debug_grant(0, "bolehuiyan");
    e_nest.debug_grant(2, "yxs_fanji");
    let sha_outer = give_card_matching(&mut e_nest, 1, |_, m| m.kind == CardKind::Sha);
    let shan_p1 = give_card_matching(&mut e_nest, 1, |_, m| m.kind == CardKind::Shan);
    let sha_inner = give_card_matching(&mut e_nest, 2, |_, m| m.kind == CardKind::Sha);
    e_nest.debug_set_turn(1, Phase::Play, 1);
    let _ = submit(
        &mut e_nest,
        1,
        GameAction::PlayCard {
            card: sha_outer,
            targets: vec![2],
        },
    );
    assert_eq!(
        e_nest.open_choice().map(|c| c.spec.prompt),
        Some("bolehuiyan:predict")
    );
    let _ = answer(&mut e_nest, 0, vec![ChoiceOption::Tag(2)]); // Miss prediction on Sha#1; seat 2 holds no Shan -> straight to Damage -> yxs_fanji
    assert_eq!(
        e_nest.open_choice().map(|c| c.spec.prompt),
        Some("trigger_confirm")
    );
    let _ = answer(&mut e_nest, 2, vec![ChoiceOption::Confirm]);
    assert_eq!(e_nest.open_choice().map(|c| c.spec.prompt), Some("yxs_fanji_sha"));
    let _ = answer(&mut e_nest, 2, vec![ChoiceOption::Card(sha_inner)]);
    assert_eq!(e_nest.open_choice().map(|c| c.spec.prompt), Some("respond_shan"));
    let _ = answer(&mut e_nest, 1, vec![ChoiceOption::Card(shan_p1)]);
    drain_pass(&mut e_nest);
    assert_eq!(
        e_nest.state_core().player(0).hand.len(),
        1,
        "nested Sha#2 dodged -> settles bolehuiyan2 per CDP 1h (miss pred draws 1 BEFORE Sha#1 shaDamage)"
    );
    assert_eq!(flag(&e_nest, 0, "bolehuiyan_pred"), 0);
}

#[test]
fn u08_r2_cike_direct_hit_weapons_qinglong_and_stale_judge() {
    // Upstream: `英雄杀/character/skill.js:3001-3027` + `card/standard.js:244-286`
    // 3a) Red-judge `cike` sets `directHit` while target holds `Shan` -> Hanbing / Qilin window MUST still open!
    let mut e = new_u08_engine(890007, 4);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "cike");
    equip_matching(&mut e, 0, |_, m| m.kind == CardKind::Hanbing);
    give_card_matching(&mut e, 1, |_, m| m.kind == CardKind::Shan);
    give_card_matching(&mut e, 1, |_, m| m.kind == CardKind::Tao);
    let sha1 = give_card_matching(&mut e, 0, |_, m| m.kind == CardKind::Sha);
    // Pre-seed stale red suit to test stale `last_judge_suit` clearing later, and verify Heart judge triggers Hanbing!
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "last_judge_suit".to_string()), 1); // stale Spade
    let red_j = find_card_matching(&e, |_, m| matches!(m.suit, Suit::Heart | Suit::Diamond));
    e.state_core_mut().deck.retain(|&c| c != red_j);
    e.state_core_mut().discard.retain(|&c| c != red_j);
    e.state_core_mut().deck.push(red_j);

    e.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![1],
        },
    );
    assert_eq!(e.open_choice().map(|c| c.spec.prompt), Some("cike:confirm"));
    let _ = answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    assert_eq!(
        e.open_choice().map(|c| c.spec.prompt),
        Some("hanbing_prevent_confirm"),
        "cike directHit with target holding Shan MUST still open Hanbing window"
    );
    assert_eq!(flag(&e, 0, "last_judge_suit"), 0, "cike must clear last_judge_suit");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Pass]);
    drain_pass(&mut e);
    assert_eq!(e.state_core().player(1).hp, 3);

    // 3b) `cike` + `Qinglong`: 1st Sha dodged -> Qinglong replay Sha -> `cike:confirm` MUST re-open,
    //     and stale red `last_judge_suit` must NOT turn a black judge into `directHit`!
    let mut e2 = new_u08_engine(890008, 4);
    for p in 0..4 {
        e2.debug_clear_hand(p);
    }
    e2.debug_grant(0, "cike");
    equip_matching(&mut e2, 0, |_, m| m.kind == CardKind::Qinglong);
    let t_shan = give_card_matching(&mut e2, 1, |_, m| m.kind == CardKind::Shan);
    let s1 = give_card_matching(&mut e2, 0, |_, m| {
        m.kind == CardKind::Sha && matches!(m.suit, Suit::Heart)
    });
    let s2 = give_card_matching(&mut e2, 0, |_, m| {
        m.kind == CardKind::Sha && matches!(m.suit, Suit::Diamond)
    });
    let blk1 = put_on_deck_top(&mut e2, |_, m| matches!(m.suit, Suit::Spade | Suit::Club));
    let _ = blk1;
    e2.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "last_judge_suit".to_string()), 2); // stale Heart!
    e2.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(
        &mut e2,
        0,
        GameAction::PlayCard {
            card: s1,
            targets: vec![1],
        },
    );
    assert_eq!(e2.open_choice().map(|c| c.spec.prompt), Some("cike:confirm"));
    let _ = answer(&mut e2, 0, vec![ChoiceOption::Confirm]);
    assert_eq!(
        e2.open_choice().map(|c| c.spec.prompt),
        Some("cike:discard_card"),
        "black judge must open discard_card, NOT red directHit"
    );
    assert_eq!(flag(&e2, 0, "last_judge_suit"), 0);
    let _ = answer(&mut e2, 0, Vec::new()); // cancel discard
    assert_eq!(e2.open_choice().map(|c| c.spec.prompt), Some("respond_shan"));
    let _ = answer(&mut e2, 1, vec![ChoiceOption::Card(t_shan)]);
    assert_eq!(
        e2.open_choice().map(|c| c.spec.prompt),
        Some("qinglong_replay_sha")
    );
    put_on_deck_top(&mut e2, |_, m| matches!(m.suit, Suit::Heart | Suit::Diamond));
    let _ = answer(&mut e2, 0, vec![ChoiceOption::Card(s2)]);
    assert_eq!(
        e2.open_choice().map(|c| c.spec.prompt),
        Some("cike:confirm"),
        "Qinglong replayed Sha MUST trigger cike again"
    );
    let _ = answer(&mut e2, 0, vec![ChoiceOption::Confirm]);
    drain_pass(&mut e2);
    assert_eq!(e2.state_core().player(1).hp, 3);
}

#[test]
fn u08_r2_nichang_skip_and_nichang2_phase_end() {
    // Upstream: `英雄杀/character/skill.js:2144-2179` (`phaseDrawBefore`, `phaseEnd`)
    // + CDP `nichang_when_draw_skipped` & `nichang2_wange2_phase_timing`.
    // 4a) Skipped draw phase (`skip_next_draw = true`) MUST NOT prompt `nichang:confirm` or grant `nichang2`.
    let mut e_sk = new_u08_engine(890009, 3);
    for p in 0..3 {
        e_sk.debug_clear_hand(p);
    }
    e_sk.debug_grant(0, "nichang");
    e_sk.state_core_mut().player_mut(0).skip_next_draw = true;
    e_sk.debug_set_turn(0, Phase::Judge, 1);
    let _ = submit(&mut e_sk, 0, GameAction::NextPhase);
    assert!(
        e_sk.open_choice().is_none(),
        "skipped draw phase must NOT trigger nichang"
    );
    let nc2 = e_sk.catalog().skill_by_name("nichang2").expect("nichang2");
    assert!(!e_sk.state_core().has_skill(0, nc2));

    // 4b) Normal draw: cancel draws 2 (no `nichang2`); confirm cancels draw (0 drawn even with `yxsdili`) + mid-choice roundtrip!
    // 4c) Combined `nichang` + `xumou`: at `Phase::End` (`phaseJieshuBegin`), `xumou` draws 3 cards FIRST,
    //     `nichang2` MUST wait until `TurnEndWindow { after: false }` (`phaseEnd`) so it counts `xumou`'s 3 cards!
    let mut e = new_u08_engine(890010, 3);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "nichang");
    e.debug_grant(0, "yxsdili");
    e.debug_grant(0, "xumou");
    e.state_core_mut().player_mut(0).hp = 1; // yxsdili would give +2 if draw wasn't cancelled
    // Put 1 Spade, 1 Heart, 1 Club on top of deck for xumou's 3 draws -> only Diamond missing (1 card drawn by nichang2)!
    let d1 = put_on_deck_top(&mut e, |_, m| m.suit == Suit::Spade);
    let d2 = put_on_deck_top(&mut e, |_, m| m.suit == Suit::Heart);
    let d3 = put_on_deck_top(&mut e, |_, m| m.suit == Suit::Club);
    let _ = (d1, d2, d3);
    e.debug_set_turn(0, Phase::Judge, 1);

    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert_eq!(e.open_choice().map(|c| c.spec.prompt), Some("nichang:confirm"));
    let mut e_cancel = assert_mid_choice_roundtrip_extra(&e, "nichang:confirm", None);
    let _ = answer(&mut e_cancel, 0, vec![ChoiceOption::Pass]);
    drain_pass(&mut e_cancel);
    assert_eq!(
        e_cancel.state_core().player(0).hand.len(),
        4,
        "declining nichang lets DrawPhase proceed (+yxsdili)"
    );
    assert!(!e_cancel.state_core().has_skill(0, nc2));

    let _ = answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    assert!(
        e.state_core().player(0).hand.is_empty(),
        "confirming nichang cancels draw phase completely (0 cards even with yxsdili)"
    );
    assert!(e.state_core().has_skill(0, nc2));
    assert!(!e.state_core().turn.skip_draw_phase);

    // Advance Draw -> Play -> Discard -> End (`xumou:confirm` at Phase::End; `nichang2` NOT yet triggered!)
    let _ = submit(&mut e, 0, GameAction::NextPhase); // -> Play
    let _ = submit(&mut e, 0, GameAction::NextPhase); // -> Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // -> End
    assert_eq!(
        e.open_choice().map(|c| c.spec.prompt),
        Some("xumou:confirm"),
        "Phase::End runs xumou BEFORE phaseEnd nichang2"
    );
    let _ = answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    assert_eq!(
        e.state_core().player(0).hand.len(),
        3,
        "xumou drew 3; nichang2 waits for TurnEndWindow"
    );
    assert!(e.state_core().has_skill(0, nc2));
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End -> TurnEndWindow
    drain_pass(&mut e);
    // xumou drew 3 (Spade, Heart, Club); then TurnEndWindow { after: false } ran `nichang2` -> 4 - 3 = 1 card!
    assert_eq!(
        e.state_core().player(0).hand.len(),
        4,
        "nichang2 at phaseEnd after xumou counts xumou's 3 suits (Spade+Heart+Club) -> draws 1"
    );
    assert!(!e.state_core().has_skill(0, nc2));
}

#[test]
fn u08_r2_wange2_phase_end_and_interactive_card_pick() {
    // Upstream: `英雄杀/character/skill.js:2219-2260` (`phaseEnd`, `chooseTarget`, `gainMultiple`)
    use engine_native::ai::BoundedPlanner;

    let mut e = new_u08_engine(890011, 3);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "wange");
    e.debug_grant(0, "xumou"); // Phase::End `phaseJieshuBegin`
    e.state_core_mut().player_mut(0).max_hp = 4;
    e.state_core_mut().player_mut(0).hp = 2; // can pick up to 2 targets
    let p1_a = give_card_matching(&mut e, 1, |_, m| m.kind == CardKind::Sha);
    let p1_b = give_card_matching(&mut e, 1, |_, m| m.kind == CardKind::Tao);
    let p2_a = give_card_matching(&mut e, 2, |_, m| m.kind == CardKind::Jiu);
    e.debug_set_turn(0, Phase::Judge, 1);

    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert_eq!(e.open_choice().map(|c| c.spec.prompt), Some("wange:confirm"));
    let _ = answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    assert_eq!(e.state_core().player(0).hand.len(), 1, "wange reduces draw by 1");
    let wg2 = e.catalog().skill_by_name("wange2").expect("wange2");
    assert!(e.state_core().has_skill(0, wg2));

    let _ = submit(&mut e, 0, GameAction::NextPhase); // -> Play
    let _ = submit(&mut e, 0, GameAction::NextPhase); // -> Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // -> End
    assert_eq!(
        e.open_choice().map(|c| c.spec.prompt),
        Some("xumou:confirm"),
        "wange2 MUST NOT fire before Phase::End (xumou)"
    );
    let _ = answer(&mut e, 0, Vec::new()); // pass xumou
    assert!(
        e.open_choice().is_none(),
        "wange2 waits for TurnEndWindow after Phase::End"
    );
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End -> TurnEndWindow
    assert_eq!(
        e.open_choice().map(|c| c.spec.prompt),
        Some("wange:choose_target"),
        "wange2 fires at TurnEndWindow after Phase::End"
    );

    // Cancel path on `wange:choose_target`
    let mut e_cancel = assert_mid_choice_roundtrip_extra(&e, "wange:choose_target", None);
    let _ = answer(&mut e_cancel, 0, Vec::new());
    drain_pass(&mut e_cancel);
    assert_eq!(e_cancel.state_core().player(1).hand.len(), 2);

    // Pick both seat 1 and seat 2 -> opens interactive `wange:choose_card` for each!
    let _ = answer(
        &mut e,
        0,
        vec![ChoiceOption::Player(1), ChoiceOption::Player(2)],
    );
    assert_eq!(
        e.open_choice().map(|c| c.spec.prompt),
        Some("wange:choose_card")
    );
    // Test AI plan + A vs B pick + mid-choice roundtrip
    assert!(matches!(
        e.plan_action(0, &BoundedPlanner::new(4)).action,
        GameAction::Answer { .. }
    ));
    let mut e_b = assert_mid_choice_roundtrip_extra(&e, "wange:choose_card", None);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(p1_a)]);
    let _ = answer(&mut e_b, 0, vec![ChoiceOption::Card(p1_b)]);
    assert_eq!(
        e.open_choice().map(|c| c.spec.prompt),
        Some("wange:choose_card")
    );
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(p2_a)]);
    let _ = answer(&mut e_b, 0, vec![ChoiceOption::Card(p2_a)]);
    drain_pass(&mut e);
    drain_pass(&mut e_b);
    assert!(e.state_core().player(0).hand.contains(&p1_a));
    assert!(!e.state_core().player(0).hand.contains(&p1_b));
    assert!(e_b.state_core().player(0).hand.contains(&p1_b));
    assert!(!e_b.state_core().player(0).hand.contains(&p1_a));
    assert!(e.state_core().player(0).hand.contains(&p2_a));
    assert!(!e.state_core().has_skill(0, wg2));
}

#[test]
fn u08_r2_qiandu_excludes_next_alive() {
    // Upstream: `英雄杀/character/skill.js:2623-2655` (`target !== player && target !== player.next`),
    // CDP `qiandu_2p` / `qiandu_8p`.
    let mut e2 = new_u08_engine(890012, 2);
    for p in 0..2 {
        e2.debug_clear_hand(p);
    }
    e2.debug_grant(0, "qiandu");
    give_card_matching(&mut e2, 0, |_, m| m.suit == Suit::Spade);
    e2.debug_set_turn(0, Phase::Play, 1);
    assert!(
        matches!(activate(&mut e2, 0, "qiandu"), CommandOutcome::Rejected(_)),
        "2-player game has no valid qiandu target (other is next_alive)"
    );

    let mut e3 = new_u08_engine(890013, 3);
    for p in 0..3 {
        e3.debug_clear_hand(p);
    }
    e3.debug_grant(0, "qiandu");
    let blk = give_card_matching(&mut e3, 0, |_, m| m.suit == Suit::Club);
    e3.debug_set_turn(0, Phase::Play, 1);
    let _ = activate(&mut e3, 0, "qiandu");
    let _ = answer(&mut e3, 0, vec![ChoiceOption::Card(blk)]);
    let ask_t = e3.open_choice().expect("qiandu:choose_target").clone();
    assert_eq!(ask_t.spec.prompt, "qiandu:choose_target");
    assert!(ask_t.spec.options.contains(&ChoiceOption::Player(2)));
    assert!(
        !ask_t.spec.options.contains(&ChoiceOption::Player(1)),
        "qiandu must exclude player.next (seat 1)"
    );
    let mut e3_cancel = assert_mid_choice_roundtrip_extra(&e3, "qiandu:choose_target", None);
    let _ = answer(&mut e3_cancel, 0, Vec::new());
    assert!(
        matches!(activate(&mut e3_cancel, 0, "qiandu"), CommandOutcome::Pending { .. }),
        "cancelling qiandu refunds usable"
    );
    let _ = answer(&mut e3, 0, vec![ChoiceOption::Player(2)]);
    assert_eq!(flag(&e3, 0, "yxs_qiandu_seat_swapped"), 1);
}

#[test]
fn u08_r2_ducai3_blocks_respond_save_and_uses_effective_suit() {
    // Upstream: `英雄杀/character/skill.js:2012-2075`, CDP `ducai3`:
    // `cardEnabled`, `cardUsable`, `cardRespondable`, `cardSavable`.
    // 1) Heart `ducai` lock + `xinhongyan` effective suit:
    let mut eh = new_u08_engine(890014, 3);
    for p in 0..3 {
        eh.debug_clear_hand(p);
    }
    eh.debug_grant(0, "ducai");
    let d_heart = give_card_matching(&mut eh, 0, |_, m| m.suit == Suit::Heart);
    let p0_heart_sha1 =
        give_card_matching(&mut eh, 0, |_, m| m.kind == CardKind::Sha && m.suit == Suit::Heart);
    let p0_heart_sha2 =
        give_card_matching(&mut eh, 0, |_, m| m.kind == CardKind::Sha && m.suit == Suit::Heart);
    let p0_sha3 = give_card_matching(&mut eh, 0, |_, m| m.kind == CardKind::Sha);
    eh.debug_set_turn(0, Phase::Play, 1);

    let _ = activate(&mut eh, 0, "ducai");
    let _ = answer(&mut eh, 0, vec![ChoiceOption::Card(d_heart)]);

    // 1a) Seat 2 holds ONLY Heart Shan -> cannot respond to Seat 0's Heart Sha (`ducai2` owner CAN use Heart Sha):
    give_card_matching(&mut eh, 2, |_, m| m.kind == CardKind::Shan && m.suit == Suit::Heart);
    let _ = submit(
        &mut eh,
        0,
        GameAction::PlayCard {
            card: p0_heart_sha1,
            targets: vec![2],
        },
    );
    assert!(
        eh.open_choice().is_none(),
        "seat 2 holding only Heart Shan must NOT get respond_shan under Heart ducai3 lock"
    );
    assert_eq!(eh.state_core().player(2).hp, 3);

    // 1b) Seat 2 holds Heart Shan + Diamond Shan -> respond_shan contains ONLY Diamond Shan (+ roundtrip)
    //     AND when Seat 2 enters dying (HP 1), Seat 1 holds Heart Tao + Diamond Tao -> dying_save contains ONLY Diamond Tao!
    let p2_diamond_shan =
        give_card_matching(&mut eh, 2, |_, m| m.kind == CardKind::Shan && m.suit == Suit::Diamond);
    let p1_heart_tao =
        give_card_matching(&mut eh, 1, |_, m| m.kind == CardKind::Tao && m.suit == Suit::Heart);
    let p1_diamond_tao =
        give_card_matching(&mut eh, 1, |_, m| m.kind == CardKind::Tao && m.suit == Suit::Diamond);
    eh.state_core_mut().player_mut(2).hp = 1;
    eh.state_core_mut().turn.sha_used_play_phase = false;
    let _ = submit(
        &mut eh,
        0,
        GameAction::PlayCard {
            card: p0_heart_sha2,
            targets: vec![2],
        },
    );
    let ask_shan = eh.open_choice().expect("respond_shan").clone();
    assert_eq!(ask_shan.spec.prompt, "respond_shan");
    assert!(ask_shan.spec.options.contains(&ChoiceOption::Card(p2_diamond_shan)));
    assert!(
        !ask_shan
            .spec
            .options
            .iter()
            .any(|o| matches!(o, ChoiceOption::Card(c) if eh.catalog().cards[*c as usize].suit == Suit::Heart)),
        "Heart Shan must be filtered out of respond_shan under Heart ducai3 lock"
    );
    let mut eh_shan_snap = assert_mid_choice_roundtrip_extra(&eh, "respond_shan", None);
    let _ = answer(&mut eh_shan_snap, 2, vec![ChoiceOption::Pass]);
    let ask_save = eh_shan_snap.open_choice().expect("dying_save").clone();
    assert_eq!(ask_save.spec.prompt, "dying_save");
    assert_eq!(ask_save.observer, 1);
    assert!(ask_save.spec.options.contains(&ChoiceOption::Card(p1_diamond_tao)));
    assert!(
        !ask_save.spec.options.contains(&ChoiceOption::Card(p1_heart_tao)),
        "Heart Tao must be filtered out of dying_save under Heart ducai3 lock"
    );
    let mut eh_save_snap = assert_mid_choice_roundtrip_extra(&eh_shan_snap, "dying_save", None);
    let _ = answer(&mut eh_save_snap, 1, vec![ChoiceOption::Card(p1_diamond_tao)]);
    drain_pass(&mut eh_save_snap);
    assert!(eh_save_snap.state_core().player(2).alive);
    drain_pass(&mut eh);

    // 1c) Effective suit test (`xinhongyan` converts Spade -> Heart):
    //     Seat 1 holds Spade Sha + Club Sha, has `xinhongyan`, targeted by Seat 0's Duel under Heart ducai3:
    //     Spade Sha has effective suit Heart -> filtered OUT of `duel_sha` (only Club Sha remains; roundtrip passes!).
    eh.debug_grant(1, "xinhongyan");
    eh.debug_clear_hand(1);
    let p1_spade_sha_hy =
        give_card_matching(&mut eh, 1, |_, m| m.kind == CardKind::Sha && m.suit == Suit::Spade);
    let p1_club_sha_hy =
        give_card_matching(&mut eh, 1, |_, m| m.kind == CardKind::Sha && m.suit == Suit::Club);
    let p0_duel_hy =
        give_card_matching(&mut eh, 0, |_, m| m.kind == CardKind::Duel && m.suit == Suit::Club);
    let _ = p0_sha3;
    let _ = submit(
        &mut eh,
        0,
        GameAction::PlayCard {
            card: p0_duel_hy,
            targets: vec![1],
        },
    );
    let ask_duel_hy = eh.open_choice().expect("duel_sha").clone();
    assert_eq!(ask_duel_hy.spec.prompt, "duel_sha");
    assert!(ask_duel_hy.spec.options.contains(&ChoiceOption::Card(p1_club_sha_hy)));
    assert!(
        !ask_duel_hy.spec.options.contains(&ChoiceOption::Card(p1_spade_sha_hy)),
        "Seat 1's Spade Sha has effective suit Heart via xinhongyan -> forbidden under Heart ducai3!"
    );
    drain_pass(&mut eh);

    // 2) Spade `ducai` lock: Duel + AOE + Nullify blocked for other seats (with roundtrip):
    let mut e = new_u08_engine(890015, 3);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "ducai");
    let d_spade = give_card_matching(&mut e, 0, |_, m| m.suit == Suit::Spade);
    let p0_spade_aoe = give_card_matching(&mut e, 0, |_, m| {
        m.kind == CardKind::SavageAttack && m.suit == Suit::Spade
    });
    e.debug_set_turn(0, Phase::Play, 1);
    let _ = activate(&mut e, 0, "ducai");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(d_spade)]);

    // `ducai2` owner plays Spade SavageAttack; Seat 1 holds Spade Sha + Heart Sha -> `aoe_response` ONLY allows Heart Sha:
    let p1_spade_sha =
        give_card_matching(&mut e, 1, |_, m| m.kind == CardKind::Sha && m.suit == Suit::Spade);
    let p1_heart_sha =
        give_card_matching(&mut e, 1, |_, m| m.kind == CardKind::Sha && m.suit == Suit::Heart);
    let p1_spade_null = give_card_matching(&mut e, 1, |_, m| {
        m.kind == CardKind::Nullify && m.suit == Suit::Spade
    });
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: p0_spade_aoe,
            targets: Vec::new(),
        },
    );
    let ask_aoe = e.open_choice().expect("aoe_response").clone();
    assert_eq!(ask_aoe.spec.prompt, "aoe_response");
    assert_eq!(ask_aoe.observer, 1);
    assert!(ask_aoe.spec.options.contains(&ChoiceOption::Card(p1_heart_sha)));
    assert!(
        !ask_aoe.spec.options.contains(&ChoiceOption::Card(p1_spade_sha)),
        "ducai3 must block Spade Sha in aoe_response"
    );
    assert!(
        !ask_aoe.spec.options.contains(&ChoiceOption::Card(p1_spade_null)),
        "Spade Nullify must not open nullify_window"
    );
    let mut e_aoe_snap = assert_mid_choice_roundtrip_extra(&e, "aoe_response", None);
    let _ = answer(&mut e_aoe_snap, 1, vec![ChoiceOption::Pass]);
    drain_pass(&mut e_aoe_snap);
    drain_pass(&mut e);

    // Seat 2 plays Diamond Duel against Seat 1 holding Spade Sha + Heart Sha + Spade Nullify -> `duel_sha` ONLY allows Heart Sha!
    e.debug_set_turn(2, Phase::Play, 2);
    let p2_diamond_duel =
        give_card_matching(&mut e, 2, |_, m| m.kind == CardKind::Duel && m.suit == Suit::Diamond);
    let _ = submit(
        &mut e,
        2,
        GameAction::PlayCard {
            card: p2_diamond_duel,
            targets: vec![1],
        },
    );
    let ask_duel = e.open_choice().expect("duel_sha").clone();
    assert_eq!(ask_duel.spec.prompt, "duel_sha");
    assert!(ask_duel.spec.options.contains(&ChoiceOption::Card(p1_heart_sha)));
    assert!(
        !ask_duel.spec.options.contains(&ChoiceOption::Card(p1_spade_sha)),
        "ducai3 must block Spade Sha in duel_sha"
    );
    let mut e_duel_snap = assert_mid_choice_roundtrip_extra(&e, "duel_sha", None);
    let _ = answer(&mut e_duel_snap, 1, vec![ChoiceOption::Pass]);
    drain_pass(&mut e_duel_snap);
}

#[test]
fn u08_r2_tongyu_and_lguiyin_require_luobinhan_general() {
    // Upstream: `英雄杀/character/skill.js:683, 719`: `[player.name1, player.name2].includes("yxs_luobinhan")`.
    let mut e = new_u08_engine(890016, 3);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    set_general(&mut e, 0, "yxs_xiangyu");
    e.debug_grant(0, "tongyu");
    e.debug_grant(0, "lguiyin");
    let c0 = give_card_matching(&mut e, 0, |_, _| true);
    e.debug_set_turn(0, Phase::Play, 1);
    assert!(
        matches!(activate(&mut e, 0, "tongyu"), CommandOutcome::Rejected(_)),
        "non-luobinhan general cannot activate tongyu"
    );
    assert!(
        matches!(activate(&mut e, 0, "lguiyin"), CommandOutcome::Rejected(_)),
        "non-luobinhan general cannot activate lguiyin"
    );

    // Put `yxs_luobinhan` in sub-general slot `general_slots[1]` -> BOTH become legal!
    let lb = (0..e.catalog().general_count() as u16)
        .find(|&idx| e.catalog().general_meta(idx).is_some_and(|m| m.0 == "yxs_luobinhan"))
        .expect("yxs_luobinhan");
    e.state_core_mut().player_mut(0).general_slots[1] = Some(lb);
    let out = activate(&mut e, 0, "tongyu");
    assert!(matches!(out, CommandOutcome::Pending { .. }));
    let mut e_can = assert_mid_choice_roundtrip_extra(&e, "tongyu:discard_cards", None);
    let _ = answer(&mut e_can, 0, Vec::new());
    assert!(
        matches!(activate(&mut e_can, 0, "tongyu"), CommandOutcome::Pending { .. }),
        "cancelling tongyu refunds usable"
    );
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(c0)]);
    let xds = e.catalog().skill_by_name("xiadao").expect("xiadao");
    assert!(e.state_core().has_skill(0, xds));
}

#[test]
fn u08_r2_xiadao_random_hand_no_card_prompt() {
    // Upstream: `英雄杀/character/skill.js:1665-1719` + CDP `xiadao`:
    // prompts ONLY `chooseTarget`, then `trigger.player.getCards("h").randomGet()` with NO card prompt.
    let mut e = new_u08_engine(890017, 3);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "xiadao");
    give_card_matching(&mut e, 1, |_, m| m.kind == CardKind::Tao);
    give_card_matching(&mut e, 1, |_, m| m.kind == CardKind::Jiu);
    give_card_matching(&mut e, 1, |_, m| m.kind == CardKind::Wuzhong);
    give_card_matching(&mut e, 2, |_, m| m.kind == CardKind::Sha);
    let sha = give_card_matching(&mut e, 0, |_, m| m.kind == CardKind::Sha);
    e.debug_set_turn(0, Phase::Play, 1);

    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    assert_eq!(e.open_choice().map(|c| c.spec.prompt), Some("xiadao:confirm"));
    let mut e_cancel = assert_mid_choice_roundtrip_extra(&e, "xiadao:confirm", None);
    let _ = answer(&mut e_cancel, 0, Vec::new());
    drain_pass(&mut e_cancel);
    assert_eq!(e_cancel.state_core().player(1).hand.len(), 3);
    assert_eq!(e_cancel.state_core().player(2).hand.len(), 1);

    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
    assert!(
        e.open_choice().is_none(),
        "xiadao MUST NOT open xiadao:choose_card prompt after choosing target"
    );
    assert_eq!(e.state_core().player(1).hand.len(), 2);
    assert_eq!(e.state_core().player(2).hand.len(), 2);
}

#[test]
fn u08_r2_jieyong_use_card_after_all_kinds_and_jianxiong_order() {
    // Upstream: `英雄杀/character/skill.js:2797-2841`
    // 10a) `Sha` against `Jianxiong` resolves FIRST -> `Jianxiong` steals `Sha` out of discard
    //      -> `useCardAfter` sees `filterInD().length === 0` so `jieyong` MUST NOT trigger!
    let mut e_jx = new_u08_engine(890018, 3);
    for p in 0..3 {
        e_jx.debug_clear_hand(p);
    }
    e_jx.debug_grant(0, "jieyong");
    e_jx.debug_grant(1, "奸雄");
    give_card_matching(&mut e_jx, 0, |_, m| m.suit == Suit::Spade && m.kind != CardKind::Sha);
    let sha_jx = give_card_matching(&mut e_jx, 0, |_, m| m.kind == CardKind::Sha);
    e_jx.debug_set_turn(0, Phase::Play, 1);
    let _ = submit(
        &mut e_jx,
        0,
        GameAction::PlayCard {
            card: sha_jx,
            targets: vec![1],
        },
    );
    // Seat 1 confirms Jianxiong (`trigger_confirm`):
    assert_eq!(
        e_jx.open_choice().map(|c| c.spec.prompt),
        Some("trigger_confirm")
    );
    let _ = answer(&mut e_jx, 1, vec![ChoiceOption::Confirm]);
    assert!(
        e_jx.open_choice().is_none(),
        "Jianxiong took Sha before useCardAfter -> jieyong must NOT prompt"
    );
    assert!(e_jx.state_core().player(1).hand.contains(&sha_jx));

    // 10b) Delayed trick `Le`: un-nullified enters judge zone (NO jieyong); nullified enters discard (`jieyong` DOES reclaim Le!);
    //      plus black card A vs B choice, cancel leaves card in discard without granting `jieyong2`, roundtrip works!
    let mut e_le = new_u08_engine(890019, 3);
    for p in 0..3 {
        e_le.debug_clear_hand(p);
    }
    e_le.debug_grant(0, "jieyong");
    let le1 = give_card_matching(&mut e_le, 0, |_, m| m.kind == CardKind::Le);
    let le2 = give_card_matching(&mut e_le, 0, |_, m| m.kind == CardKind::Le);
    let blk_a = give_card_matching(&mut e_le, 0, |_, m| m.suit == Suit::Spade && m.kind == CardKind::Sha);
    let blk_b = give_card_matching(&mut e_le, 0, |_, m| m.suit == Suit::Club && m.kind == CardKind::Sha);
    let p1_null = give_card_matching(&mut e_le, 1, |_, m| m.kind == CardKind::Nullify);
    e_le.debug_set_turn(0, Phase::Play, 1);

    // Un-nullified Le on seat 2 -> enters judge zone -> NO jieyong!
    let _ = submit(
        &mut e_le,
        0,
        GameAction::PlayCard {
            card: le1,
            targets: vec![2],
        },
    );
    assert_eq!(e_le.open_choice().map(|c| c.spec.prompt), Some("nullify_window"));
    let _ = answer(&mut e_le, 1, vec![ChoiceOption::Pass]);
    assert!(e_le.open_choice().is_none(), "Le in judge zone -> no jieyong");

    // Nullified Le on seat 1 -> enters discard -> `jieyong` DOES prompt after CardUsed settles!
    let _ = submit(
        &mut e_le,
        0,
        GameAction::PlayCard {
            card: le2,
            targets: vec![1],
        },
    );
    let _ = answer(&mut e_le, 1, vec![ChoiceOption::Card(p1_null)]);
    assert_eq!(
        e_le.open_choice().map(|c| c.spec.prompt),
        Some("jieyong:discard_black"),
        "nullified Delayed Trick Le is in discard -> jieyong must trigger at useCardAfter!"
    );
    let mut e_cancel = assert_mid_choice_roundtrip_extra(&e_le, "jieyong:discard_black", None);
    let mut e_pick_b = assert_mid_choice_roundtrip_extra(&e_le, "jieyong:discard_black", None);

    // Cancel preserves Le in discard & does NOT grant jieyong2:
    let _ = answer(&mut e_cancel, 0, Vec::new());
    let jy2 = e_le.catalog().skill_by_name("jieyong2").expect("jieyong2");
    assert!(!e_cancel.state_core().player(0).hand.contains(&le2));
    assert!(!e_cancel.state_core().has_skill(0, jy2));

    // Choice A discards blk_a, Choice B discards blk_b; both regain le2 and grant jieyong2:
    let _ = answer(&mut e_le, 0, vec![ChoiceOption::Card(blk_a)]);
    let _ = answer(&mut e_pick_b, 0, vec![ChoiceOption::Card(blk_b)]);
    assert!(!e_le.state_core().player(0).hand.contains(&blk_a));
    assert!(e_le.state_core().player(0).hand.contains(&blk_b));
    assert!(e_le.state_core().player(0).hand.contains(&le2));
    assert!(!e_pick_b.state_core().player(0).hand.contains(&blk_b));
    assert!(e_pick_b.state_core().player(0).hand.contains(&blk_a));
    assert!(e_pick_b.state_core().player(0).hand.contains(&le2));
    assert!(e_le.state_core().has_skill(0, jy2));
    assert!(
        flag(&e_le, 0, "jieyong2_used") == 1,
        "2nd card in same turn blocked by jieyong2"
    );
    // 2nd card in same turn does not re-trigger jieyong:
    let sha_2nd = give_card_matching(&mut e_le, 0, |_, m| m.kind == CardKind::Sha);
    let _ = submit(
        &mut e_le,
        0,
        GameAction::PlayCard {
            card: sha_2nd,
            targets: vec![2],
        },
    );
    drain_pass(&mut e_le);
    assert!(!e_le.state_core().player(0).hand.contains(&sha_2nd));
}

#[test]
fn u08_r3_jieyong_no_dangling_flag_and_ignores_cancelled_use() {
    // Round-3 regressions for `U08:skills:jieyong` (`英雄杀/character/skill.js:2797-2841`):
    // 1) After `jieyong` reclaims `wz` from discard at `useCardAfter`, `shouye_pending_gain_card`
    //    MUST NOT stay set in `vm_aux.flags` — replaying `wz` and settling another `CardUsed`
    //    must leave `wz` in `discard` rather than pulling it back for free (`resolution.rs:799-816`).
    let mut e_no_dangle = new_u08_engine(890020, 3);
    for p in 0..3 {
        e_no_dangle.debug_clear_hand(p);
    }
    e_no_dangle.debug_grant(0, "jieyong");
    let wz = give_card_matching(&mut e_no_dangle, 0, |_, m| m.kind == CardKind::Wuzhong);
    let blk = give_card_matching(&mut e_no_dangle, 0, |_, m| {
        matches!(m.suit, Suit::Spade | Suit::Club) && m.kind != CardKind::Sha
    });
    let sha_after = give_card_matching(&mut e_no_dangle, 0, |_, m| m.kind == CardKind::Sha);
    e_no_dangle.debug_set_turn(0, Phase::Play, 1);

    let _ = submit(
        &mut e_no_dangle,
        0,
        GameAction::PlayCard {
            card: wz,
            targets: Vec::new(),
        },
    );
    assert_eq!(
        e_no_dangle.open_choice().map(|c| c.spec.prompt),
        Some("jieyong:discard_black")
    );
    let _ = answer(&mut e_no_dangle, 0, vec![ChoiceOption::Card(blk)]);
    assert_eq!(
        flag(&e_no_dangle, 0, "shouye_pending_gain_card"),
        0,
        "jieyong runs after CardUsed pop and must NOT set dangling shouye_pending_gain_card"
    );
    assert!(e_no_dangle.state_core().player(0).hand.contains(&wz));

    // Replay reclaimed `wz` (blocked from re-triggering by `jieyong2`) -> lands in discard,
    // then settle another `CardUsed` (`sha_after`) -> `wz` must stay in discard!
    let _ = submit(
        &mut e_no_dangle,
        0,
        GameAction::PlayCard {
            card: wz,
            targets: Vec::new(),
        },
    );
    drain_pass(&mut e_no_dangle);
    assert!(e_no_dangle.state_core().discard.contains(&wz));
    assert!(!e_no_dangle.state_core().player(0).hand.contains(&wz));

    let _ = submit(
        &mut e_no_dangle,
        0,
        GameAction::PlayCard {
            card: sha_after,
            targets: vec![2],
        },
    );
    drain_pass(&mut e_no_dangle);
    assert!(
        e_no_dangle.state_core().discard.contains(&wz),
        "subsequent CardUsed pop must NOT pull reclaimed wz out of discard for free"
    );
    assert!(!e_no_dangle.state_core().player(0).hand.contains(&wz));
    assert_eq!(flag(&e_no_dangle, 0, "shouye_pending_gain_card"), 0);

    // 2) Card use cancelled in `Before` (`feigong` sets `u.cancelled = true` and puts `sha_can`
    //    in `discard`): upstream dispatches `useCardOmitted`, NOT `useCardAfter`, so `jieyong`
    //    MUST NOT prompt (`CDP afterCancelledUse: { promptsOnCancelledUse: 0 }`).
    let mut e_can = new_u08_engine(890021, 3);
    for p in 0..3 {
        e_can.debug_clear_hand(p);
    }
    e_can.debug_grant(0, "jieyong");
    e_can.debug_grant(1, "feigong");
    give_card_matching(&mut e_can, 0, |_, m| {
        matches!(m.suit, Suit::Spade | Suit::Club) && m.kind != CardKind::Sha
    });
    let sha_can = give_card_matching(&mut e_can, 0, |_, m| m.kind == CardKind::Sha);
    let fg_sha = give_card_matching(&mut e_can, 1, |_, m| m.kind == CardKind::Sha);
    e_can.debug_set_turn(0, Phase::Play, 1);

    let _ = submit(
        &mut e_can,
        0,
        GameAction::PlayCard {
            card: sha_can,
            targets: vec![2],
        },
    );
    assert_eq!(
        e_can.open_choice().map(|c| c.spec.prompt),
        Some("feigong:discard_sha")
    );
    let _ = answer(&mut e_can, 1, vec![ChoiceOption::Card(fg_sha)]);
    assert!(
        e_can.open_choice().is_none(),
        "cancelled CardUsed (`u.cancelled = true`) must NOT trigger jieyong even though sha_can is in discard"
    );
    assert!(e_can.state_core().discard.contains(&sha_can));
    assert!(!e_can.state_core().player(0).hand.contains(&sha_can));
    assert_eq!(e_can.state_core().player(2).hp, 4, "feigong cancelled the Sha");
}
