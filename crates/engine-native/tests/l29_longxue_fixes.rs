//! Batch 1.5 regression tests for L29 龙血玄黄 upstream semantics.
//!
//! Upstream references (`archive/upstream/noname-liuli/noname/`):
//! - `mbqianlong` + `mbweitong`: `character/mobile.js:16998-17116, 17635-17658`, `game/game.js:29451-29467`
//! - `mbcmqingzheng`: `character/mobile.js:17117-17250` (`direct: true`, step 2 discard before step 3 control, `target.damage()` source)
//! - `mbcmjiushi`: `character/mobile.js:17251-17409` (`turnOver()` before virtual `jiu`, dying self-save `enable:'chooseToUse'`, `damageBegin3`->`damageEnd`, `get.cardPile2` deck-only)
//! - `mbcmfangzhu`: `character/mobile.js:17410-17509` (`skillBlocker` uses `!forever && !charlotte`, `cardEnabled`/`cardSavable` without `cardRespondable`)
//! - `mbjuejin` + `mbcmxscw`: `character/mobile.js:17510-17634` (`hp<1` hujia compensation, hand-only exile, `mbcmxscw` discard sweep)

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use engine_native::ai::BoundedPlanner;
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::Suit;
use engine_native::content::extension::ExtensionPack;
use engine_native::event::DamageNature;
use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, LogEvent, Phase,
};

static DEDUP: AtomicU64 = AtomicU64::new(9_000_000);

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action,
    })
}

fn load_pack(rel: &str) -> ExtensionPack {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    let json = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
    content_native_loader::bridge_pack(&json)
        .unwrap_or_else(|e| panic!("bridge {path:?}: {e}"))
        .pack
}

fn place_kind(e: &mut Engine, player: u8, kind: engine_native::catalog::CardKind) -> u32 {
    let name = e
        .catalog()
        .cards
        .iter()
        .find(|c| c.kind == kind)
        .unwrap_or_else(|| panic!("no card of kind {kind:?}"))
        .name;
    e.debug_place(player, name)
}

fn longxue_pack() -> ExtensionPack {
    load_pack("../../packages/extensions/longxue-upstream-liuli/assets/cards.json")
}

fn general_def_of(e: &Engine, name: &str) -> u16 {
    for def in 0..=u16::MAX {
        if e.catalog().general_meta(def).map(|g| g.0) == Some(name) {
            return def;
        }
        if e.catalog().general_meta(def).is_none() && def > 64 {
            break;
        }
    }
    panic!("general not found: {name}");
}

fn drain(e: &mut Engine) {
    for _ in 0..48 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec
                .options
                .iter()
                .take(c.spec.min.max(1))
                .copied()
                .collect()
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

fn next_phase(e: &mut Engine) {
    let actor = e.turn().current;
    let out = submit(e, actor, GameAction::NextPhase);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(e);
}

fn advance_to_phase(e: &mut Engine, seat: u8, phase: Phase) {
    for _ in 0..64 {
        if e.turn().current == seat && e.turn().phase == phase && e.open_choice().is_none() {
            return;
        }
        if e.open_choice().is_some() {
            drain(e);
            continue;
        }
        let actor = e.turn().current;
        let _ = submit(e, actor, GameAction::NextPhase);
    }
    panic!("failed to reach seat={seat} phase={phase:?}");
}

fn snapshot_roundtrip(e: &Engine, pack: &ExtensionPack) -> Engine {
    let bytes = encode_archive(&e.save());
    let snap = decode_archive(&bytes).expect("decode archive");
    let restored = Engine::restore_with_extensions(&snap, &[pack]).expect("restore archive");
    assert_eq!(e.state_hash(), restored.state_hash());
    assert_eq!(e.open_choice(), restored.open_choice());
    restored
}

/// Upstream `mobile.js:16998-17116, 17635-17658`, `game.js:29451-29467`:
/// - `hasZhuSkill('mbweitong')` checks whether *owner* is Lord with `mbweitong` and any other alive Wei exists (other Wei does not need `mbweitong`).
/// - Non-lord `mb_caomao` only gains +20 even with another Wei `mb_caomao`.
/// - Reaching 99 keeps `mbweitong`.
/// - Self-damage triggers both `DamageSource` (+15) and `DamageEnd` (+10) -> +25.
/// - `forever: true` is set on all 6 L29 skills.
#[test]
fn l29_qianlong_weitong_lord_self_damage_forever_and_keep_at_99() {
    let pack = longxue_pack();
    // Case A: Lord (seat 0) mb_caomao + ordinary Wei (曹操 on seat 2, NO mbweitong) -> +60 opening + 5 draw = 65
    {
        let mut e = Engine::try_new_identity_with_extensions(15001, 4, &[&pack]).unwrap();
        let caomao = general_def_of(&e, "mb_caomao");
        e.debug_assemble_general_def(0, caomao);
        e.debug_assemble_general(2, "曹操");
        next_phase(&mut e);
        assert_eq!(e.state_core().player(0).mbqianlong_marks, 65, "Lord mb_caomao with Wei ally starts at 60 + 5 draw");
        let wt = e.catalog().skill_by_name("mbweitong").unwrap();
        let juejin = e.catalog().skill_by_name("mbjuejin").unwrap();
        let _ = e.debug_deal_damage(Some(0), 1, 3, DamageNature::Normal);
        drain(&mut e);
        assert_eq!(e.state_core().player(0).mbqianlong_marks, 99);
        assert!(e.state_core().skills.contains_key(&(0, juejin)));
        assert!(
            e.state_core().skills.contains_key(&(0, wt)),
            "mbweitong must remain installed when marks reach 99"
        );
    }

    // Case B: Non-lord (seat 1) mb_caomao + Wei ally (seat 2 mb_caomao) -> +20 (NOT 60)
    {
        let mut e = Engine::try_new_identity_with_extensions(15002, 4, &[&pack]).unwrap();
        let caomao = general_def_of(&e, "mb_caomao");
        e.debug_assemble_general_def(1, caomao);
        e.debug_assemble_general_def(2, caomao);
        next_phase(&mut e);
        assert_eq!(
            e.state_core().player(1).mbqianlong_marks,
            20,
            "Non-lord mb_caomao must NOT activate Zhu skill mbweitong (+20 only)"
        );
        assert_eq!(
            e.state_core().player(2).mbqianlong_marks,
            20,
            "Non-lord seat 2 mb_caomao also starts at +20"
        );

        // Self-damage (+15 DamageSource + 10 DamageEnd = +25 -> 45)
        e.debug_set_hp(1, 3);
        let _ = e.debug_deal_damage(Some(1), 1, 1, DamageNature::Normal);
        drain(&mut e);
        assert_eq!(
            e.state_core().player(1).mbqianlong_marks,
            45,
            "Self-damage of 1 must add +15 (DamageSource) + 10 (DamageEnd) = +25"
        );
    }

    // All 6 L29 skills must have is_forever_skill == true
    let e = Engine::new_with_extensions(15003, 4, &[&pack]).unwrap();
    for name in [
        "mbqianlong",
        "mbweitong",
        "mbcmqingzheng",
        "mbcmjiushi",
        "mbcmfangzhu",
        "mbjuejin",
    ] {
        let def = e.catalog().skill_by_name(name).expect(name);
        assert!(
            e.catalog().is_forever_skill(def),
            "L29 skill {name} must report is_forever_skill == true"
        );
    }
}

/// Upstream `mobile.js:17117-17250` (`L29-R01`, `L29-R02`):
/// - `direct: true` opens `mbcmqingzheng_suit` with `allow_cancel: true` (no `trigger_confirm`).
/// - Owner discard executes BEFORE `mbcmqingzheng_target_suit` opens (triggering `CardsLost` skills like `连营`).
/// - Damage has `source: Some(owner)`.
/// - Save/restore preserves all suspended prompts cleanly.
#[test]
fn l29_qingzheng_direct_order_damage_source_and_restore() {
    let pack = longxue_pack();

    // 1. Direct suit prompt + cancel at suit & cancel at target
    {
        let mut e = Engine::new_with_extensions(15010, 4, &[&pack]).unwrap();
        e.state_core_mut().skills.clear();
        e.debug_grant(0, "mbcmqingzheng");
        advance_to_phase(&mut e, 0, Phase::Draw);
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        e.debug_place_suit(0, Suit::Spade);
        e.debug_place_suit(1, Suit::Club);
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        let c1 = e.open_choice().expect("direct suit prompt").clone();
        assert_eq!(
            c1.spec.prompt, "mbcmqingzheng_suit",
            "mbcmqingzheng must open mbcmqingzheng_suit directly without trigger_confirm"
        );
        assert!(c1.spec.allow_cancel, "mbcmqingzheng_suit must allow cancel");
        let mut restored = snapshot_roundtrip(&e, &pack);
        let hand_before = restored.debug_hand(0).len();
        let out = submit(
            &mut restored,
            0,
            GameAction::Answer {
                choice_id: c1.id,
                selection: vec![],
            },
        );
        assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
        assert_eq!(restored.debug_hand(0).len(), hand_before, "cancel suit keeps hand intact");
        assert_eq!(restored.turn().phase, Phase::Play);
    }

    // 2. Owner discards BEFORE target_suit opens + damage source == Some(owner) + Choice A vs Choice B
    {
        let mut e = Engine::new_with_extensions(15011, 4, &[&pack]).unwrap();
        e.state_core_mut().skills.clear();
        e.debug_grant(0, "mbcmqingzheng");
        e.debug_grant(0, "连营");
        e.debug_grant(0, "mbqianlong");
        e.state_core_mut().player_mut(0).mbqianlong_marks = 30;
        advance_to_phase(&mut e, 0, Phase::Draw);
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        // Owner holds 2 Spades + 1 Heart; target holds 1 Club.
        let s1 = e.debug_place_suit(0, Suit::Spade);
        let s2 = e.debug_place_suit(0, Suit::Spade);
        let h1 = e.debug_place_suit(0, Suit::Heart);
        e.debug_place_suit(1, Suit::Club);
        let _ = submit(&mut e, 0, GameAction::NextPhase);

        let c_suit = e.open_choice().expect("suit").clone();
        assert_eq!(c_suit.spec.prompt, "mbcmqingzheng_suit");
        // Choice A (Heart = 1 card <= 1 Club): target takes 0 damage
        {
            let mut e_a = e.clone();
            let _ = submit(&mut e_a, 0, GameAction::Answer { choice_id: c_suit.id, selection: vec![ChoiceOption::Tag(1)] });
            let t_id = e_a.open_choice().unwrap().id;
            let _ = submit(&mut e_a, 0, GameAction::Answer { choice_id: t_id, selection: vec![ChoiceOption::Player(1)] });
            let hp1_before = e_a.debug_hp(1);
            let ts_id = e_a.open_choice().unwrap().id;
            let _ = submit(&mut e_a, 0, GameAction::Answer { choice_id: ts_id, selection: vec![ChoiceOption::Tag(2)] });
            drain(&mut e_a);
            assert_eq!(e_a.debug_hp(1), hp1_before, "Choice A (1 vs 1) deals no damage");
        }
        let _ = submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: c_suit.id,
                selection: vec![ChoiceOption::Tag(0)],
            },
        );
        let c_tgt = e.open_choice().expect("target").clone();
        assert_eq!(c_tgt.spec.prompt, "mbcmqingzheng_target");
        let mut e_cancel = e.clone();
        let _ = submit(
            &mut e_cancel,
            0,
            GameAction::Answer {
                choice_id: c_tgt.id,
                selection: vec![],
            },
        );
        assert_eq!(e_cancel.debug_hand(0).len(), 3, "canceling target must not discard owner's cards");

        let _ = submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: c_tgt.id,
                selection: vec![ChoiceOption::Player(1)],
            },
        );
        assert!(
            !e.debug_hand(0).contains(&s1) && !e.debug_hand(0).contains(&s2) && e.debug_hand(0).contains(&h1),
            "L29-R02: owner cards must be discarded BEFORE mbcmqingzheng_target_suit opens"
        );
        let c_tsuit = e.open_choice().expect("target_suit").clone();
        assert_eq!(c_tsuit.spec.prompt, "mbcmqingzheng_target_suit");
        assert!(!c_tsuit.spec.allow_cancel);
        let mut restored = snapshot_roundtrip(&e, &pack);
        let marks_before = restored.state_core().player(0).mbqianlong_marks;
        let log_before = restored.log().len();
        let _ = submit(
            &mut restored,
            0,
            GameAction::Answer {
                choice_id: c_tsuit.id,
                selection: vec![ChoiceOption::Tag(2)],
            },
        );
        drain(&mut restored);
        let dmg_log = restored.log()[log_before..].iter().find_map(|ev| match ev {
            LogEvent::DamageDealt { source, target, amount, .. } => Some((*source, *target, *amount)),
            _ => None,
        });
        assert_eq!(
            dmg_log,
            Some((Some(0), 1, 1)),
            "L29-R01: mbcmqingzheng damage must attribute source = Some(owner)"
        );
        assert_eq!(
            restored.state_core().player(0).mbqianlong_marks,
            marks_before + 15,
            "DamageSource from owner's mbcmqingzheng grants +15 mbqianlong marks"
        );
    }

    // 3. Negative: 0 hand cards -> does not trigger; no other player with hand -> suit prompt then finishes without discarding
    {
        let mut e = Engine::new_with_extensions(15012, 4, &[&pack]).unwrap();
        e.state_core_mut().skills.clear();
        e.debug_grant(0, "mbcmqingzheng");
        advance_to_phase(&mut e, 0, Phase::Draw);
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        let mut e_empty = e.clone();
        let _ = submit(&mut e_empty, 0, GameAction::NextPhase);
        assert!(
            e_empty.open_choice().is_none(),
            "must not open mbcmqingzheng when owner has 0 hand cards"
        );

        let keep = e.debug_place_suit(0, Suit::Heart);
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        let c_suit = e.open_choice().expect("suit opens when owner has hand").clone();
        let _ = submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: c_suit.id,
                selection: vec![ChoiceOption::Tag(1)],
            },
        );
        assert!(e.open_choice().is_none(), "finishes cleanly without targets");
        assert!(e.debug_hand(0).contains(&keep), "owner card preserved when no target exists");
    }
}

/// Upstream `mobile.js:17251-17409` (`L29-R03`):
/// - `turnOver()` runs BEFORE virtual `jiu` commits (`jiu_active`/`jiu_used` are false while `mbcmjiushi_gain_confirm` is open).
/// - `chooseToUse` during `Dying` allows face-up owner to self-save via `ChoiceOption::Skill(jiushi_def)` with 0 hand cards.
/// - `damageBegin3` records `face_down`; `damageEnd` offers turnback even for chain / boosted Sha or even if flipped before `damageEnd`.
/// - `get.cardPile2` reads `deck` ONLY (never `discard`) and triggers `mbqianlong` `+5` via `CardsMoved`.
#[test]
fn l29_jiushi_order_dying_save_damage_begin3_and_deck_only() {
    let pack = longxue_pack();

    // 1. L29-R03 order + deck-only cardPile2 + mbqianlong +5 on gain
    {
        let mut e = Engine::new_with_extensions(15020, 4, &[&pack]).unwrap();
        e.state_core_mut().skills.clear();
        advance_to_phase(&mut e, 0, Phase::Play);
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        e.debug_grant(0, "mbcmjiushi");
        e.debug_grant(0, "mbqianlong");
        e.state_core_mut().player_mut(0).mbqianlong_marks = 50;
        // Empty deck of tricks so only discard holds a trick -> confirm gain must get 0 cards!
        let mut non_tricks = Vec::new();
        let mut discard_tricks = Vec::new();
        for cid in e.state_core().deck.iter().copied() {
            let k = e.catalog().card_kind(cid);
            if k.is_basic_kind() || k.equip_slot().is_some() {
                non_tricks.push(cid);
            } else {
                discard_tricks.push(cid);
            }
        }
        e.state_core_mut().deck = non_tricks;
        e.state_core_mut().discard.extend(discard_tricks);

        let jiushi = e.catalog().skill_by_name("mbcmjiushi").unwrap();
        let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: jiushi });
        assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
        assert!(e.state_core().player(0).face_down, "flipped face-down first");
        assert!(
            !e.turn().jiu_active && !e.turn().jiu_used,
            "L29-R03: virtual jiu must NOT commit until after turnOver/gain confirm closes"
        );
        let c_gain = e.open_choice().expect("gain confirm").clone();
        assert_eq!(c_gain.spec.prompt, "mbcmjiushi_gain_confirm");
        let mut restored = snapshot_roundtrip(&e, &pack);
        let _ = submit(
            &mut restored,
            0,
            GameAction::Answer {
                choice_id: c_gain.id,
                selection: vec![ChoiceOption::Confirm],
            },
        );
        drain(&mut restored);
        assert!(
            restored.turn().jiu_active && restored.turn().jiu_used,
            "virtual jiu commits AFTER turnOver completes"
        );
        assert_eq!(
            restored.debug_hand(0).len(),
            0,
            "get.cardPile2 must NOT fall back to discardPile"
        );
    }

    // 2. Deck has trick + mbqianlong +5 via CardsMoved
    {
        let mut e = Engine::new_with_extensions(15021, 4, &[&pack]).unwrap();
        e.state_core_mut().skills.clear();
        advance_to_phase(&mut e, 0, Phase::Play);
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        e.debug_grant(0, "mbcmjiushi");
        e.debug_grant(0, "mbqianlong");
        e.state_core_mut().player_mut(0).mbqianlong_marks = 50;
        let jiushi = e.catalog().skill_by_name("mbcmjiushi").unwrap();
        let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: jiushi });
        let c_gain = e.open_choice().expect("gain confirm").clone();
        let _ = submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: c_gain.id,
                selection: vec![ChoiceOption::Confirm],
            },
        );
        drain(&mut e);
        assert_eq!(e.debug_hand(0).len(), 1);
        assert_eq!(
            e.state_core().player(0).mbqianlong_marks,
            55,
            "mbcmjiushi trick gain must trigger mbqianlong +5"
        );
    }

    // 3. Dying self-save with 0 hand cards + save/restore + AI choice + negative (face_down / fangzhu_ban)
    {
        let mut e = Engine::new_with_extensions(15022, 4, &[&pack]).unwrap();
        e.state_core_mut().skills.clear();
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        e.debug_grant(1, "mbcmjiushi");
        let jiushi = e.catalog().skill_by_name("mbcmjiushi").unwrap();
        e.debug_set_hp(1, 1);
        let out = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
        assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
        let c_dying = e.open_choice().expect("dying_save").clone();
        assert_eq!(c_dying.spec.prompt, "dying_save");
        assert_eq!(c_dying.observer, 1);
        assert!(
            c_dying.spec.options.contains(&ChoiceOption::Skill(jiushi)),
            "dying_save must offer ChoiceOption::Skill(mbcmjiushi) to dying owner"
        );
        // AI must choose mbcmjiushi rather than Pass
        let plan = e.plan_action(1, &BoundedPlanner::new(8));
        assert_eq!(
            plan.action,
            GameAction::Answer {
                choice_id: c_dying.id,
                selection: vec![ChoiceOption::Skill(jiushi)],
            }
        );

        let mut restored = snapshot_roundtrip(&e, &pack);
        let _ = submit(
            &mut restored,
            1,
            GameAction::Answer {
                choice_id: c_dying.id,
                selection: vec![ChoiceOption::Skill(jiushi)],
            },
        );
        let c_gain = restored.open_choice().expect("turnOver gain confirm inside dying").clone();
        assert_eq!(c_gain.spec.prompt, "mbcmjiushi_gain_confirm");
        assert!(restored.state_core().player(1).face_down);
        assert_eq!(restored.debug_hp(1), 0, "virtual jiu recovers HP AFTER turnOver closes");
        let _ = submit(
            &mut restored,
            1,
            GameAction::Answer {
                choice_id: c_gain.id,
                selection: vec![],
            },
        );
        // DamageEnd opens turnback confirm because seat 1 was NOT face_down at DamageBegin3? Wait: seat 1 was face-UP at DamageBegin3!
        drain(&mut restored);
        assert_eq!(restored.debug_hp(1), 1, "virtual jiu must recover 1 HP and save dying owner");
        assert!(!restored.state_core().player(1).dying);
        assert!(!restored.turn().jiu_active && !restored.turn().jiu_used);

        // Negative: Pass on dying_save kills dying seat 1; face_down == true skips dying_save mbcmjiushi directly
        let mut e_pass = e.clone();
        let _ = submit(&mut e_pass, 1, GameAction::Answer { choice_id: c_dying.id, selection: vec![ChoiceOption::Pass] });
        drain(&mut e_pass);
        assert!(!e_pass.state_core().player(1).alive, "passing dying_save leaves seat 1 dead");

        let mut e_fd = Engine::new_with_extensions(15024, 4, &[&pack]).unwrap();
        e_fd.state_core_mut().skills.clear();
        for p in 0..4 {
            e_fd.debug_clear_hand(p);
        }
        e_fd.debug_grant(1, "mbcmjiushi");
        e_fd.state_core_mut().player_mut(1).face_down = true;
        e_fd.debug_set_hp(1, 1);
        let _ = e_fd.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
        drain(&mut e_fd);
        assert!(!e_fd.state_core().player(1).alive, "face_down seat 1 cannot self-save with mbcmjiushi");
    }

    // 4. DamageBegin3 records face_down -> DamageEnd triggers even for boosted Sha / chain / flipped before DamageEnd; AI confirms iff face_down
    {
        let mut e = Engine::new_with_extensions(15023, 4, &[&pack]).unwrap();
        e.state_core_mut().skills.clear();
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        e.debug_grant(0, "mbcmjiushi");
        e.debug_grant(1, "mbcmjiushi");
        advance_to_phase(&mut e, 0, Phase::Play);
        let jiushi = e.catalog().skill_by_name("mbcmjiushi").unwrap();
        let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: jiushi });
        let g0 = e.open_choice().unwrap().id;
        let _ = submit(&mut e, 0, GameAction::Answer { choice_id: g0, selection: vec![] });
        // Link 0 and 1, deal fire damage from 0 to 1 (chaining back to face-down 0 with from_chain=true and source=Some(0))!
        e.state_core_mut().player_mut(0).linked = true;
        e.state_core_mut().player_mut(1).linked = true;
        e.debug_set_hp(0, 3);
        e.debug_set_hp(1, 3);
        let _ = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Fire);
        let c_tb = e.open_choice().expect("turnback confirm on chained self-damage").clone();
        assert_eq!(c_tb.spec.prompt, "mbcmjiushi_turnback_confirm");
        assert_eq!(c_tb.observer, 0);
        // Flip face-up artificially before answering to prove AI respects face_down == false
        let mut e_face_up = e.clone();
        e_face_up.state_core_mut().player_mut(0).face_down = false;
        let ai_up = e_face_up.plan_action(0, &BoundedPlanner::new(8));
        assert_eq!(
            ai_up.action,
            GameAction::Answer {
                choice_id: c_tb.id,
                selection: vec![],
            },
            "AI must decline turnback when already face-up"
        );
        // Confirming still flips regardless of current orientation
        let _ = submit(&mut e_face_up, 0, GameAction::Answer { choice_id: c_tb.id, selection: vec![ChoiceOption::Confirm] });
        drain(&mut e_face_up);
        assert!(e_face_up.state_core().player(0).face_down, "DamageEnd triggers whenever DamageBegin3 recorded face_down");
    }
}

/// Upstream `mobile.js:17410-17509`:
/// - `baiban` (`Tag(1)`): disables all `!forever && !charlotte` skills INCLUDING locked skills (`马术`, `咆哮`), while preserving `forever: true` (`mbcmjiushi`, `mbqianlong`). Re-enables after `Phase::End` at `phaseEnd`.
/// - `jin` (`Tag(0)`): blocks non-trick cardEnabled/cardSavable (including equip/dying saves/`mbcmjiushi`), does NOT block `chooseToRespond` (`Duel`/`Nanman`/`Wanjian`).
#[test]
fn l29_fangzhu_baiban_blocks_locked_and_jin_allows_choose_to_respond() {
    let pack = longxue_pack();

    // 1. Tag(1) baiban: disables 马术 & 咆哮 (locked!), preserves mbcmjiushi (forever!)
    {
        let mut e = Engine::new_with_extensions(15030, 4, &[&pack]).unwrap();
        e.state_core_mut().skills.clear();
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        e.debug_grant(0, "mbcmfangzhu");
        e.debug_grant(1, "马术");
        e.debug_grant(1, "咆哮");
        e.debug_grant(1, "闭月");
        e.debug_grant(1, "mbcmjiushi");
        advance_to_phase(&mut e, 0, Phase::Play);
        let fangzhu = e.catalog().skill_by_name("mbcmfangzhu").unwrap();
        let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: fangzhu });
        let t_id = e.open_choice().unwrap().id;
        let _ = snapshot_roundtrip(&e, &pack);
        let _ = submit(&mut e, 0, GameAction::Answer { choice_id: t_id, selection: vec![ChoiceOption::Player(1)] });
        let o_id = e.open_choice().unwrap().id;
        let _ = snapshot_roundtrip(&e, &pack);
        let _ = submit(&mut e, 0, GameAction::Answer { choice_id: o_id, selection: vec![ChoiceOption::Tag(1)] });

        let mashu = e.catalog().skill_by_name("马术").unwrap();
        let paoxiao = e.catalog().skill_by_name("咆哮").unwrap();
        let biyue = e.catalog().skill_by_name("闭月").unwrap();
        let jiushi = e.catalog().skill_by_name("mbcmjiushi").unwrap();
        assert!(!e.state_core().skills[&(1, mashu)].enabled, "baiban must disable locked skill 马术");
        assert!(!e.state_core().skills[&(1, paoxiao)].enabled, "baiban must disable locked skill 咆哮");
        assert!(!e.state_core().skills[&(1, biyue)].enabled, "baiban must disable normal skill 闭月");
        assert!(e.state_core().skills[&(1, jiushi)].enabled, "baiban must NOT disable forever skill mbcmjiushi");

        // Advance to seat 1 Play: verify 2nd Sha rejected while 咆哮 disabled
        advance_to_phase(&mut e, 1, Phase::Play);
        e.debug_clear_hand(1);
        e.debug_clear_hand(2);
        let sha1 = e.debug_place(1, "sha");
        let sha2 = e.debug_place(1, "sha");
        assert!(matches!(
            submit(&mut e, 1, GameAction::PlayCard { card: sha1, targets: vec![2] }),
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ));
        drain(&mut e);
        assert!(
            matches!(
                submit(&mut e, 1, GameAction::PlayCard { card: sha2, targets: vec![2] }),
                CommandOutcome::Rejected(_)
            ),
            "2nd Sha must be rejected while 咆哮 is disabled by baiban"
        );

        // Advance past seat 1 End phase into seat 2: 闭月 must NOT have triggered, then skills re-enable
        advance_to_phase(&mut e, 2, Phase::Draw);
        assert!(e.state_core().skills[&(1, mashu)].enabled, "马术 re-enables at phaseEnd");
        assert!(e.state_core().skills[&(1, paoxiao)].enabled, "咆哮 re-enables at phaseEnd");
        assert!(e.state_core().skills[&(1, biyue)].enabled, "闭月 re-enables at phaseEnd");
    }

    // 2. Tag(0) jin: allows chooseToRespond (Duel Sha / Nanman Sha / Wanjian Shan), blocks dying_save Tao & mbcmjiushi
    {
        let mut e = Engine::new_with_extensions(15031, 4, &[&pack]).unwrap();
        e.state_core_mut().skills.clear();
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        e.debug_grant(0, "mbcmfangzhu");
        e.debug_grant(1, "mbcmjiushi");
        advance_to_phase(&mut e, 0, Phase::Play);
        let fangzhu = e.catalog().skill_by_name("mbcmfangzhu").unwrap();
        let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: fangzhu });
        let t_id = e.open_choice().unwrap().id;
        let _ = submit(&mut e, 0, GameAction::Answer { choice_id: t_id, selection: vec![ChoiceOption::Player(1)] });
        let o_id = e.open_choice().unwrap().id;
        let _ = submit(&mut e, 0, GameAction::Answer { choice_id: o_id, selection: vec![ChoiceOption::Tag(0)] });

        // Duel against seat 1 holding Sha -> seat 1 CAN play Sha via chooseToRespond!
        let shan_1 = e.debug_place(1, "shan");
        let sha_1 = e.debug_place(1, "sha");
        let tao_1 = e.debug_place(1, "tao");
        let duel = e.debug_place(0, "duel");
        e.debug_set_hp(1, 2);
        let out = submit(&mut e, 0, GameAction::PlayCard { card: duel, targets: vec![1] });
        assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
        let c_duel = e.open_choice().expect("duel respond sha").clone();
        assert_eq!(c_duel.observer, 1);
        assert!(
            c_duel.spec.options.contains(&ChoiceOption::Card(sha_1)),
            "mbcmfangzhu_ban must NOT block Duel Sha (chooseToRespond)"
        );
        let _ = submit(&mut e, 1, GameAction::Answer { choice_id: c_duel.id, selection: vec![ChoiceOption::Card(sha_1)] });
        drain(&mut e);
        assert_eq!(e.debug_hp(1), 2, "seat 1 responded Sha cleanly in Duel");

        // Wanjian against seat 1 holding Shan -> seat 1 CAN respond Shan!
        let archery = e.debug_place(0, "archery");
        let out = submit(&mut e, 0, GameAction::PlayCard { card: archery, targets: vec![] });
        assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
        let c_wj = e.open_choice().expect("wanjian respond shan").clone();
        assert_eq!(c_wj.observer, 1);
        assert!(
            c_wj.spec.options.contains(&ChoiceOption::Card(shan_1)),
            "mbcmfangzhu_ban must NOT block Wanjian Shan (chooseToRespond)"
        );
        let _ = submit(&mut e, 1, GameAction::Answer { choice_id: c_wj.id, selection: vec![ChoiceOption::Card(shan_1)] });
        drain(&mut e);

        // Dying save on seat 1 holding Tao + mbcmjiushi while seat 2 holds a Tao -> skips seat 1 completely and asks seat 2!
        let _jiushi = e.catalog().skill_by_name("mbcmjiushi").unwrap();
        let tao_2 = e.debug_place(2, "tao");
        let _ = tao_1;
        e.debug_set_hp(1, 1);
        let _ = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
        let c_die = e.open_choice().expect("dying_save asks seat 2 instead of banned seat 1").clone();
        assert_eq!(
            c_die.observer, 2,
            "mbcmfangzhu_ban must block both Tao and mbcmjiushi on dying seat 1, skipping directly to seat 2"
        );
        assert!(c_die.spec.options.contains(&ChoiceOption::Card(tao_2)));
        drain(&mut e);
    }
}

/// Upstream `mobile.js:17510-17634`:
/// - `hp < 1` gains `(1 - hp) + plus` hujia.
/// - `mbjuejin` exiles `shan`/`tao`/`jiu` ONLY from alive players' hands (NOT deck, discard, equips, judge).
/// - Subsequent discard sweeps both incoming and pre-existing `shan`/`tao`/`jiu` from discard pile via `mbcmxscw`.
#[test]
fn l29_juejin_hp_below_one_hand_only_exile_and_mbcmxscw_discard_sweep() {
    let pack = longxue_pack();
    let mut e = Engine::new_with_extensions(15040, 4, &[&pack]).unwrap();
    e.state_core_mut().skills.clear();
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "mbjuejin");
    advance_to_phase(&mut e, 0, Phase::Play);

    e.debug_set_hp(0, 1);
    e.debug_set_hp(1, 3);
    e.state_core_mut().player_mut(2).hp = -1;
    let hand_tao = e.debug_place(1, "tao");
    let discard_shan = e.debug_place(3, "shan");
    e.state_core_mut().player_mut(3).hand.retain(|&c| c != discard_shan);
    e.state_core_mut().discard.push(discard_shan);
    let deck_shan_count_before = e
        .state_core()
        .deck
        .iter()
        .filter(|&&c| e.catalog().card_kind(c) == engine_native::catalog::CardKind::Shan)
        .count();
    assert!(deck_shan_count_before > 0);

    let juejin = e.catalog().skill_by_name("mbjuejin").unwrap();
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: juejin });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    drain(&mut e);

    assert_eq!(e.debug_hp(0), 1);
    assert_eq!(e.state_core().player(0).hujia, 2, "hp==1 owner gets +2 hujia");
    assert_eq!(e.debug_hp(1), 1);
    assert_eq!(e.state_core().player(1).hujia, 2, "hp==3 other gets +2 hujia");
    assert_eq!(e.debug_hp(2), 1);
    assert_eq!(
        e.state_core().player(2).hujia,
        2,
        "hp==-1 other must get (1 - (-1)) = 2 hujia"
    );

    assert!(
        e.state_core().detached_cards.iter().any(|&(c, _)| c == hand_tao),
        "hand tao exiled by mbjuejin"
    );
    let deck_shan_count_after = e
        .state_core()
        .deck
        .iter()
        .filter(|&&c| e.catalog().card_kind(c) == engine_native::catalog::CardKind::Shan)
        .count();
    assert_eq!(
        deck_shan_count_after, deck_shan_count_before,
        "mbjuejin must NOT exile cards from deck"
    );
    assert!(
        e.state_core().discard.contains(&discard_shan),
        "mbjuejin step 2 must NOT exile pre-existing discard pile cards before next discard event"
    );

    // Limited: 2nd activation rejected
    assert!(matches!(
        submit(&mut e, 0, GameAction::ActivateSkill { skill: juejin }),
        CommandOutcome::Rejected(_)
    ));

    // Play -> Discard with 2 cards while hp == 1 opens "discard" -> discard triggers mbcmxscw sweep!
    e.debug_clear_hand(0);
    let _hand_sha = e.debug_place(0, "sha");
    let late_shan = e.debug_place(0, "shan");
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let c_disc = e.open_choice().expect("discard choice").clone();
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_disc.id,
            selection: vec![ChoiceOption::Card(late_shan)],
        },
    );
    drain(&mut e);
    assert!(
        !e.state_core().discard.contains(&discard_shan) && !e.state_core().discard.contains(&late_shan),
        "mbcmxscw must sweep pre-existing and newly discarded shan/tao/jiu into detached_cards"
    );
    assert!(e.state_core().detached_cards.iter().any(|&(c, _)| c == discard_shan));
    assert!(e.state_core().detached_cards.iter().any(|&(c, _)| c == late_shan));
}

/// Round 2 Finding 1 (`character/mobile.js:17035-17045, 17635-17658`, `game/game.js:29451-29467`):
/// - `mbqianlong` sub1 triggers on `CardsDrawn`/`CardsMoved` at `Timing::Normal, priority: 0`
///   (NOT `Timing::First, priority: 6`), so turn-owner `Timing::Normal` triggers run first.
/// - `mbweitong` (`hasZhuSkill`) boosts `mbqianlong` only for `Identity` Lord (+15 vs +10),
///   NEVER in `Identity` non-Lord, `Doudizhu`, `Boss`, `Kof`, `Kingdoms`, or `Versus`.
#[test]
fn l29_round2_qianlong_timing_and_weitong_mode_gate() {
    let pack = longxue_pack();

    // 1. TriggerMeta for CardsDrawn/CardsMoved must be Normal, priority 0; real CardsMoved ordering check
    let mut e = Engine::new_with_extensions(11, 3, &[&pack]).expect("engine");
    let qianlong = e.catalog().skill_by_name("mbqianlong").expect("mbqianlong");
    for ev in [engine_native::EventKind::CardsDrawn, engine_native::EventKind::CardsMoved] {
        let meta = e
            .catalog()
            .skills[qianlong as usize]
            .behavior
            .kind_trigger_meta(ev)
            .expect("kind_trigger_meta for mbqianlong draw/move");
        assert_eq!(meta.timing, engine_native::Timing::Normal, "{ev:?} must use Timing::Normal");
        assert_eq!(meta.priority, 0, "{ev:?} must use priority 0");
    }

    // Real match ordering: turn-owner seat 0 has `连营` (Normal, pri 0), seat 1 has `mbqianlong`.
    advance_to_phase(&mut e, 0, Phase::Play);
    e.debug_clear_hand(0);
    e.debug_grant(0, "连营");
    e.debug_grant(0, "仁德");
    e.debug_grant(1, "mbqianlong");
    e.state_core_mut().player_mut(1).mbqianlong_marks = 40;
    let gift = e.debug_place(0, "sha");
    let rende = e.catalog().skill_by_name("仁德").expect("rende");
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: rende });
    let c_cards = e.open_choice().expect("rende cards").clone();
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_cards.id,
            selection: vec![ChoiceOption::Card(gift)],
        },
    );
    let c_tgt = e.open_choice().expect("rende target").clone();
    let out = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_tgt.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let ch = e.open_choice().expect("turn-owner lianying must pause before seat-1 mbqianlong");
    assert_eq!(ch.observer, 0);
    assert_eq!(
        e.state_core().player(1).mbqianlong_marks,
        40,
        "at lianying pause point (Normal pri 0 on turn owner 0), seat 1 mbqianlong must NOT have resolved yet!"
    );
    drain(&mut e);
    assert_eq!(
        e.state_core().player(1).mbqianlong_marks,
        45,
        "after lianying resolves, seat 1 mbqianlong gains +5"
    );

    // 2. mbweitong mode eligibility (`Identity` Lord = +60 opening (+5 draw = 65); `Identity` Rebel / `Doudizhu`/`Boss`/`Kof`/`Kingdoms`/`Versus` = +20 opening (+5 draw = 25))
    let make_opening_test = |mode: engine_native::GameMode| -> u32 {
        let mut m = Engine::new_with_extensions(17, 3, &[&pack]).expect("engine");
        let caomao = general_def_of(&m, "mb_caomao");
        m.debug_assemble_general_def(0, caomao);
        m.debug_assemble_general(1, "关羽");
        m.debug_assemble_general(2, "曹操");
        m.state_core_mut().mode = mode;
        advance_to_phase(&mut m, 0, Phase::Play);
        m.state_core_mut().player(0).mbqianlong_marks.into()
    };

    assert_eq!(
        make_opening_test(engine_native::GameMode::Identity {
            roles: vec![engine_native::Role::Lord, engine_native::Role::Rebel, engine_native::Role::Loyalist],
        }),
        65,
        "Identity Lord mbweitong boosts +60 opening (+5 draw)"
    );
    assert_eq!(
        make_opening_test(engine_native::GameMode::Identity {
            roles: vec![engine_native::Role::Rebel, engine_native::Role::Lord, engine_native::Role::Loyalist],
        }),
        25,
        "Identity non-Lord must NOT boost (+20 opening + 5 draw)"
    );
    assert_eq!(
        make_opening_test(engine_native::GameMode::Doudizhu { lord: 0 }),
        25,
        "Doudizhu lord must NOT trigger hasZhuSkill mbweitong (+20 opening + 5 draw)"
    );
    assert_eq!(
        make_opening_test(engine_native::GameMode::Kingdoms),
        25,
        "Kingdoms must NOT trigger hasZhuSkill mbweitong (+20 opening + 5 draw)"
    );
    assert_eq!(
        make_opening_test(engine_native::GameMode::Versus {
            teams: vec![vec![0, 2], vec![1]],
            substitutes: vec![vec![], vec![]],
        }),
        25,
        "Versus 2v2 must NOT trigger hasZhuSkill mbweitong (+20 opening + 5 draw)"
    );
    assert_eq!(
        make_opening_test(Engine::new_boss(17, 3, 0, 0).state_core().mode.clone()),
        25,
        "Boss must NOT trigger hasZhuSkill mbweitong (+20 opening + 5 draw)"
    );
    assert_eq!(
        make_opening_test(engine_native::GameMode::Kof {
            pools: [vec![], vec![]],
            wins: [0, 0],
            games: 1,
        }),
        25,
        "Kof must NOT trigger hasZhuSkill mbweitong (+20 opening + 5 draw)"
    );
}

/// Round 2 Finding 5 (`character/mobile.js:17117-17250`):
/// `mbcmqingzheng` suit collection and discard matching must use `catalog.effective_card_suit`
/// (e.g. `xinhongyan` Spade -> Heart) rather than raw catalog card suit.
#[test]
fn l29_round2_qingzheng_effective_suit_with_xinhongyan() {
    let d34 = content_native_loader::decision34_pack9_extension_pack();
    let mut e = Engine::new_with_extensions(23, 3, &[&d34]).expect("engine");
    advance_to_phase(&mut e, 0, Phase::Draw);
    e.debug_grant(0, "mbcmqingzheng");
    e.debug_grant(0, "xinhongyan");
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);

    let sp1 = e.debug_place_suit(0, Suit::Spade);
    let sp2 = e.debug_place_suit(0, Suit::Spade);
    let tgt_heart = e.debug_place_suit(1, Suit::Heart);
    let tgt_diamond = e.debug_place_suit(1, Suit::Diamond);
    let hp1 = e.debug_hp(1);

    // Draw -> Play (triggers mbcmqingzheng with only Spade cards in hand)
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let ch_suit = e.open_choice().expect("mbcmqingzheng_suit").clone();
    assert_eq!(ch_suit.spec.prompt, "mbcmqingzheng_suit");
    // Only Spade cards in hand, turned Heart (Tag(1)) by xinhongyan!
    assert!(
        ch_suit.spec.options.contains(&ChoiceOption::Tag(1)),
        "xinhongyan Spade->Heart must offer Heart Tag(1), got {:?}",
        ch_suit.spec.options
    );
    assert!(
        !ch_suit.spec.options.contains(&ChoiceOption::Tag(0)),
        "Spade Tag(0) must NOT be offered when xinhongyan turns Spade to Heart, got {:?}",
        ch_suit.spec.options
    );
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: ch_suit.id,
            selection: vec![ChoiceOption::Tag(1)],
        },
    );
    let ch_tgt = e.open_choice().expect("mbcmqingzheng_target").clone();
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: ch_tgt.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    let ch_tsuit = e.open_choice().expect("mbcmqingzheng_target_suit").clone();
    assert_eq!(ch_tsuit.spec.prompt, "mbcmqingzheng_target_suit");
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: ch_tsuit.id,
            selection: vec![ChoiceOption::Tag(1)],
        },
    );
    drain(&mut e);

    assert!(!e.debug_hand(0).contains(&sp1) && !e.debug_hand(0).contains(&sp2), "owner Spade cards discarded as Heart");
    assert!(!e.debug_hand(1).contains(&tgt_heart), "target Heart card discarded");
    assert!(e.debug_hand(1).contains(&tgt_diamond), "target Diamond kept");
    assert_eq!(e.debug_hp(1), hp1 - 1, "target discarded 1 < owner 2 so takes 1 damage");
}

/// Round 2 Finding 2 (`character/mobile.js:17410-17509`, `mbcmfangzhu` `jin`):
/// `mbcmfangzhu_ban` blocks ViewAs/response-use active skills (`zhangba`, `dddxiaxing`, `jijiang`),
/// `Qinglong` follow-up `Sha`, and extension basic cards (`swd_caoyao1` = `CardKind::Caoyao`).
#[test]
fn l29_round2_fangzhu_jin_blocks_viewas_qinglong_jijiang_and_ext_basics() {
    let pack = longxue_pack();
    let swd = load_pack("../../packages/extensions/shahai-swd-upstream-noname/assets/cards.json");
    let mut e = Engine::new_with_extensions(31, 3, &[&pack, &swd]).expect("engine");

    // 1. ActivateSkill ViewAs: zhangba, jijiang under mbcmfangzhu_ban
    let caomao = general_def_of(&e, "mb_caomao");
    e.debug_assemble_general_def(0, caomao);
    e.debug_assemble_general(1, "刘备");
    e.debug_assemble_general(2, "关羽");
    e.state_core_mut().mode = engine_native::GameMode::Identity {
        roles: vec![
            engine_native::Role::Rebel,
            engine_native::Role::Lord,
            engine_native::Role::Loyalist,
        ],
    };
    advance_to_phase(&mut e, 0, Phase::Play);
    e.debug_grant(0, "mbcmfangzhu");
    let fz = e.catalog().skill_by_name("mbcmfangzhu").unwrap();
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: fz });
    let ct = e.open_choice().unwrap().clone();
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: ct.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    let co = e.open_choice().unwrap().clone();
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: co.id,
            selection: vec![ChoiceOption::Tag(0)], // jin
        },
    );
    drain(&mut e);

    // Seat 1 equips zhangba + jijiang + swd_caoyao1 + wuzhong before seat 1 Play
    e.debug_grant(1, "激将");
    advance_to_phase(&mut e, 1, Phase::Play);
    let zhangba_card = e.debug_place(1, "zhangba");
    assert!(matches!(
        submit(&mut e, 1, GameAction::PlayCard { card: zhangba_card, targets: vec![] }),
        CommandOutcome::Rejected(_),
    ), "jin must also block equipping zhangba!");
    // Place zhangba directly in equips (simulating equipped prior to jin) -> ActivateSkill MUST be rejected!
    e.debug_clear_hand(1);
    let _ = e.debug_place(1, "sha");
    let _ = e.debug_place(1, "shan");
    let eq_zb = e.debug_place(1, "zhangba");
    e.debug_take_owned_card_quiet(1, eq_zb);
    e.state_core_mut().player_mut(1).equips.push(eq_zb);
    e.debug_grant(1, "丈八蛇矛");

    let zhangba_sk = e.catalog().skill_by_name("丈八蛇矛").unwrap();
    let jijiang_sk = e.catalog().skill_by_name("激将").unwrap();
    assert!(
        matches!(submit(&mut e, 1, GameAction::ActivateSkill { skill: zhangba_sk }), CommandOutcome::Rejected(_)),
        "mbcmfangzhu jin must reject ActivateSkill(zhangba)"
    );
    assert!(
        matches!(submit(&mut e, 1, GameAction::ActivateSkill { skill: jijiang_sk }), CommandOutcome::Rejected(_)),
        "mbcmfangzhu jin must reject ActivateSkill(jijiang)"
    );

    // ActivateSkill(dddxiaxing) under mbcmfangzhu_ban (loaded via decision34 pack9)
    {
        let d34 = content_native_loader::decision34_pack9_extension_pack();
        let mut ed = Engine::new_with_extensions(32, 3, &[&d34]).expect("d34");
        advance_to_phase(&mut ed, 1, Phase::Play);
        ed.debug_grant(1, "dddxiaxing");
        ed.state_core_mut().vm_aux.flags.insert((1, "dddxiaxing_xia".to_string()), 1);
        ed.state_core_mut().vm_aux.flags.insert((1, "mbcmfangzhu_ban".to_string()), 1);
        let xiaxing_sk = ed.catalog().skill_by_name("dddxiaxing").unwrap();
        assert!(
            matches!(submit(&mut ed, 1, GameAction::ActivateSkill { skill: xiaxing_sk }), CommandOutcome::Rejected(_)),
            "mbcmfangzhu jin must reject ActivateSkill(dddxiaxing)"
        );
    }

    // Extension basic card (Caoyao) MUST be rejected under jin, while trick (wuzhong) is allowed
    e.debug_set_hp(2, 2);
    let caoyao = place_kind(&mut e, 1, engine_native::catalog::CardKind::Caoyao);
    let wuzhong = e.debug_place(1, "wuzhong");
    assert!(
        matches!(submit(&mut e, 1, GameAction::PlayCard { card: caoyao, targets: vec![2] }), CommandOutcome::Rejected(_)),
        "mbcmfangzhu jin must reject extension basic card swd_caoyao1"
    );
    assert!(
        matches!(
            submit(&mut e, 1, GameAction::PlayCard { card: wuzhong, targets: vec![] }),
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "mbcmfangzhu jin must still allow trick card wuzhong"
    );
    drain(&mut e);

    // 2. Qinglong follow-up Sha blocked if mbcmfangzhu_ban applies to attacker
    advance_to_phase(&mut e, 0, Phase::Play);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let ql = e.debug_place(0, "qinglong");
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: ql, targets: vec![] });
    let sha1 = e.debug_place(0, "sha");
    let _sha2 = e.debug_place(0, "sha");
    let shan1 = e.debug_place(1, "shan");
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha1, targets: vec![1] });
    let ask_shan = e.open_choice().expect("seat 1 shan ask").clone();
    // Apply jin to seat 0 before seat 1 dodges
    e.state_core_mut().vm_aux.flags.insert((0, "mbcmfangzhu_ban".to_string()), 1);
    let _ = submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: ask_shan.id,
            selection: vec![ChoiceOption::Card(shan1)],
        },
    );
    assert!(
        e.open_choice().is_none(),
        "mbcmfangzhu jin must NOT open qinglong follow-up Sha choice: {:?}",
        e.open_choice()
    );
}

/// Round 2 Findings 3 & 4 (`character/mobile.js:17410-17509`, `mbcmfangzhu` `baiban`, `CardKind::is_trick_kind`):
/// - Equipping `zhuge`, `zhangba`, `renwang` AFTER `baiban` is active still keeps their skills disabled until `phaseEnd`,
///   and they become active again after `baiban` expires.
/// - `CardKind::is_trick_kind` covers exact upstream `get.type2 == 'trick'` (excludes `Caoyao`/`Hufu`/`Jiguan`/`Haidao`/`Food`/`Spell` etc.),
///   so `mbcmjiushi` `get.cardPile2` skips non-tricks and draws real tricks.
#[test]
fn l29_round2_fangzhu_baiban_blocks_late_equips_skills_and_cardpile2_non_tricks() {
    let pack = longxue_pack();
    let swd = load_pack("../../packages/extensions/shahai-swd-upstream-noname/assets/cards.json");
    let mtg = load_pack("../../packages/extensions/shahai-mtg-upstream-noname/assets/cards.json");
    let mut e = Engine::new_with_extensions(41, 3, &[&pack, &swd, &mtg]).expect("engine");
    use engine_native::catalog::CardKind;

    assert!(!CardKind::Caoyao.is_trick_kind(), "caoyao is basic, not trick");
    assert!(!CardKind::Hufu.is_trick_kind(), "hufu is basic, not trick");
    assert!(!CardKind::Ziyangdan.is_trick_kind(), "ziyangdan is basic, not trick");
    assert!(!CardKind::Jiguan.is_trick_kind(), "jiguan is not trick");
    assert!(!CardKind::Haidao.is_trick_kind(), "haidao is land, not trick");
    assert!(!CardKind::Dizaizhen.is_trick_kind(), "dizaizhen is zhenfa, not trick");
    assert!(!CardKind::Tanhuadong.is_trick_kind(), "food is not trick");
    assert!(!CardKind::Yanziyaoshui.is_trick_kind(), "spell is not trick");
    assert!(!CardKind::Hslingjian.is_trick_kind(), "hslingjian is not trick");
    assert!(!CardKind::Yuchanqian.is_trick_kind(), "yuchan jiqi is not trick");
    assert!(CardKind::Wuzhong.is_trick_kind(), "wuzhong is trick");
    assert!(CardKind::Le.is_trick_kind(), "lebu (delay) has type2 == trick");
    assert!(CardKind::Heilonglinpian.is_trick_kind(), "heilonglinpian is trick");
    assert!(CardKind::Jingleishan.is_trick_kind(), "jingleishan is trick");

    // 1. Apply baiban to seat 1
    advance_to_phase(&mut e, 0, Phase::Play);
    e.debug_grant(0, "mbcmfangzhu");
    let fz = e.catalog().skill_by_name("mbcmfangzhu").unwrap();
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: fz });
    let ct = e.open_choice().unwrap().clone();
    let _ = submit(&mut e, 0, GameAction::Answer { choice_id: ct.id, selection: vec![ChoiceOption::Player(1)] });
    let co = e.open_choice().unwrap().clone();
    let _ = submit(&mut e, 0, GameAction::Answer { choice_id: co.id, selection: vec![ChoiceOption::Tag(1)] }); // baiban
    drain(&mut e);

    // Seat 1 Play phase (still under baiban):
    advance_to_phase(&mut e, 1, Phase::Play);
    e.debug_clear_hand(1);
    let zhuge = e.debug_place(1, "zhuge");
    let sha1 = e.debug_place(1, "sha");
    let sha2 = e.debug_place(1, "sha");
    assert!(matches!(
        submit(&mut e, 1, GameAction::PlayCard { card: zhuge, targets: vec![] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    drain(&mut e);
    assert!(matches!(
        submit(&mut e, 1, GameAction::PlayCard { card: sha1, targets: vec![2] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    drain(&mut e);
    assert!(
        matches!(submit(&mut e, 1, GameAction::PlayCard { card: sha2, targets: vec![2] }), CommandOutcome::Rejected(_)),
        "zhuge equipped AFTER baiban must NOT grant unlimited Sha while baiban is active"
    );

    // Equip zhangba AFTER baiban -> zhangba skill must be disabled!
    let zhangba_c = e.debug_place(1, "zhangba");
    assert!(matches!(
        submit(&mut e, 1, GameAction::PlayCard { card: zhangba_c, targets: vec![] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    drain(&mut e);
    let zhangba_sk = e.catalog().skill_by_name("丈八蛇矛").unwrap();
    assert!(!e.state_core().skills[&(1, zhangba_sk)].enabled, "zhangba equipped under baiban must start disabled");
    assert!(
        matches!(submit(&mut e, 1, GameAction::ActivateSkill { skill: zhangba_sk }), CommandOutcome::Rejected(_)),
        "zhangba equipped under baiban must not activate"
    );

    // After baiban expires at seat 1 phaseEnd -> seat 2 equips renwang, then back to seat 1 Play checks zhangba re-enabled!
    advance_to_phase(&mut e, 2, Phase::Play);
    let renwang_c = e.debug_place(2, "renwang");
    let _ = submit(&mut e, 2, GameAction::PlayCard { card: renwang_c, targets: vec![] });
    drain(&mut e);

    advance_to_phase(&mut e, 1, Phase::Play);
    assert!(e.state_core().skills[&(1, zhangba_sk)].enabled, "zhangba re-enables after baiban expires");

    advance_to_phase(&mut e, 0, Phase::Play);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: fz });
    let ct2 = e.open_choice().unwrap().clone();
    let _ = submit(&mut e, 0, GameAction::Answer { choice_id: ct2.id, selection: vec![ChoiceOption::Player(2)] });
    let co2 = e.open_choice().unwrap().clone();
    let _ = submit(&mut e, 0, GameAction::Answer { choice_id: co2.id, selection: vec![ChoiceOption::Tag(1)] }); // baiban
    drain(&mut e);

    // Clear seat 2 hand so no shan, then seat 0 attacks seat 2 with black Sha while seat 2 is baiban!
    e.debug_clear_hand(2);
    let take_black_sha = |eng: &mut Engine, to: u8| -> u32 {
        let mut found = None;
        for id in 0..eng.catalog().cards.len() as u32 {
            if eng.catalog().card_kind(id) == CardKind::Sha
                && matches!(eng.catalog().cards[id as usize].suit, Suit::Spade | Suit::Club)
            {
                found = Some(id);
                break;
            }
        }
        let id = found.expect("spade/club sha");
        eng.state_core_mut().deck.retain(|&c| c != id);
        eng.state_core_mut().discard.retain(|&c| c != id);
        for pid in 0..3u8 {
            eng.state_core_mut().player_mut(pid).hand.retain(|&c| c != id);
        }
        eng.state_core_mut().player_mut(to).hand.push(id);
        id
    };
    let bs1 = take_black_sha(&mut e, 0);
    let hp2_before = e.debug_hp(2);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: bs1, targets: vec![2] });
    drain(&mut e);
    assert_eq!(e.debug_hp(2), hp2_before - 1, "renwang on seat 2 under baiban must NOT block black Sha");

    // Advance past seat 2's phaseEnd -> baiban expires -> seat 0 black Sha on next turn IS blocked by renwang!
    advance_to_phase(&mut e, 0, Phase::Play);
    e.debug_clear_hand(2);
    let bs2 = take_black_sha(&mut e, 0);
    let hp2_after_exp = e.debug_hp(2);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: bs2, targets: vec![2] });
    drain(&mut e);
    assert_eq!(e.debug_hp(2), hp2_after_exp, "renwang must block black Sha again once baiban expires");

    // 2. MbcmJiushi cardPile2 skips Caoyao (basic), Jiguan (jiguan), Haidao (land) and draws real trick (wuzhong)
    advance_to_phase(&mut e, 2, Phase::Play);
    e.debug_grant(1, "mbcmjiushi");
    e.debug_set_face_down(1, true);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    let real_trick = e.debug_place(0, "wuzhong");
    e.debug_take_owned_card_quiet(0, real_trick);
    let non1 = place_kind(&mut e, 0, CardKind::Caoyao);
    e.debug_take_owned_card_quiet(0, non1);
    let non2 = place_kind(&mut e, 0, CardKind::Jiguan);
    e.debug_take_owned_card_quiet(0, non2);
    let non3 = place_kind(&mut e, 0, CardKind::Haidao);
    e.debug_take_owned_card_quiet(0, non3);
    e.debug_drain_deck_to_discard();
    e.state_core_mut().deck.push(real_trick);
    e.state_core_mut().deck.push(non1);
    e.state_core_mut().deck.push(non2);
    e.state_core_mut().deck.push(non3);
    let sha_to_1 = e.debug_place(2, "sha");
    let _ = submit(&mut e, 2, GameAction::PlayCard { card: sha_to_1, targets: vec![1] });
    for _ in 0..4 {
        let Some(ch) = e.open_choice().cloned() else { break };
        assert!(ch.spec.options.contains(&ChoiceOption::Confirm), "unexpected choice {:?}", ch.spec.prompt);
        let _ = submit(
            &mut e,
            ch.observer,
            GameAction::Answer {
                choice_id: ch.id,
                selection: vec![ChoiceOption::Confirm],
            },
        );
    }
    drain(&mut e);
    assert!(
        e.debug_hand(1).contains(&real_trick),
        "mbcmjiushi get.cardPile2 must skip Caoyao/Jiguan/Haidao and draw real trick {real_trick}, got {:?}",
        e.debug_hand(1)
    );
    assert!(!e.debug_hand(1).contains(&non1) && !e.debug_hand(1).contains(&non2) && !e.debug_hand(1).contains(&non3));
}
