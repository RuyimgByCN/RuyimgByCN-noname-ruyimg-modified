//! Shared test harness for Batch U05-6b (`shahai-hearth` Part 2: 111 skills + 110 generals).
//!
//! Zero `debug_execute_effect`; real formal `packages/extensions/shahai-hearth-upstream-noname`
//! engine + real `GameAction::{PlayCard, ActivateSkill, Answer, NextPhase}` + card conservation +
//! `BoundedPlanner` public-info checks + `NGSA 107` save/restore roundtrips.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

use content_native_loader::{
    prepare_locked_content_with_bridge, restore_locked_engine, start_locked_engine,
    validate_and_freeze_raw_package, LockedStartMode, VerifiedLockedContent,
};
use engine_native::ai::BoundedPlanner;
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::CardKind;
use engine_native::event::Phase;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, OpenChoice};

static DEDUP_B6B: AtomicU64 = AtomicU64::new(7_050_000);
static HEARTH_VERIFIED_B6B: OnceLock<VerifiedLockedContent> = OnceLock::new();

pub fn next_dedup() -> u64 {
    DEDUP_B6B.fetch_add(1, Ordering::Relaxed)
}

pub fn hearth_verified() -> &'static VerifiedLockedContent {
    HEARTH_VERIFIED_B6B.get_or_init(|| {
        let pkg_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/extensions/shahai-hearth-upstream-noname");
        let raw_json = std::fs::read_to_string(pkg_dir.join("package.npack.json"))
            .expect("read shahai-hearth package.npack.json")
            .replace("\r\n", "\n");
        let parsed: serde_json::Value = serde_json::from_str(&raw_json).unwrap();
        let mut assets = BTreeMap::new();
        if let Some(paths) = parsed.get("asset_paths").and_then(|v| v.as_object()) {
            for rel_path in paths.values().filter_map(|v| v.as_str()) {
                let bytes = std::fs::read(pkg_dir.join(rel_path))
                    .expect("read shahai-hearth asset")
                    .into_iter()
                    .filter(|&b| b != b'\r')
                    .collect::<Vec<u8>>();
                assets.insert(rel_path.to_string(), bytes);
            }
        }
        let frozen = validate_and_freeze_raw_package(raw_json, assets)
            .expect("freeze shahai-hearth formal package");
        prepare_locked_content_with_bridge(vec![frozen])
            .expect("prepare shahai-hearth verified content")
    })
}

pub fn fresh_proto(seed: u64, generals: &[&str]) -> Engine {
    let mut e = start_locked_engine(
        seed,
        generals.len(),
        LockedStartMode::Prototype,
        hearth_verified(),
    )
    .expect("start hearth prototype engine");
    for (i, g) in generals.iter().enumerate() {
        if !g.is_empty() {
            e.assign_general_named(i as u8, g)
                .unwrap_or_else(|err| panic!("assign_general_named({i}, {g}): {err:?}"));
        }
    }
    e
}

pub fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action,
    })
}

pub fn play_card(e: &mut Engine, actor: u8, card: u32, targets: Vec<u8>) -> CommandOutcome {
    submit(e, actor, GameAction::PlayCard { card, targets })
}

pub fn activate(e: &mut Engine, actor: u8, skill_name: &str) -> CommandOutcome {
    let skill = e
        .catalog()
        .skill_by_name(skill_name)
        .unwrap_or_else(|| panic!("skill `{skill_name}` missing in catalog"));
    submit(e, actor, GameAction::ActivateSkill { skill })
}

pub fn answer(e: &mut Engine, actor: u8, selection: Vec<ChoiceOption>) -> CommandOutcome {
    let ch = e
        .open_choice()
        .cloned()
        .unwrap_or_else(|| panic!("expected open_choice for actor {actor}"));
    assert_eq!(
        ch.observer, actor,
        "expected choice observer {actor}, got {} (prompt={})",
        ch.observer, ch.spec.prompt
    );
    submit(
        e,
        actor,
        GameAction::Answer {
            choice_id: ch.id,
            selection,
        },
    )
}

pub fn cancel_open_choice(e: &mut Engine) -> CommandOutcome {
    let ch = e.open_choice().cloned().expect("expected open_choice to cancel");
    assert!(
        ch.spec.allow_cancel || ch.spec.options.contains(&ChoiceOption::Pass),
        "choice {} is not cancelable",
        ch.spec.prompt
    );
    let sel = if ch.spec.options.contains(&ChoiceOption::Pass) && !ch.spec.allow_cancel {
        vec![ChoiceOption::Pass]
    } else {
        Vec::new()
    };
    submit(
        e,
        ch.observer,
        GameAction::Answer {
            choice_id: ch.id,
            selection: sel,
        },
    )
}

pub fn drain_auto(e: &mut Engine) {
    for _ in 0..48 {
        let Some(ch) = e.open_choice().cloned() else {
            break;
        };
        let sel = if ch.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if ch.spec.allow_cancel {
            Vec::new()
        } else {
            let take_n = ch.spec.min.max(1).min(ch.spec.options.len());
            ch.spec.options.iter().take(take_n).copied().collect()
        };
        let _ = submit(
            e,
            ch.observer,
            GameAction::Answer {
                choice_id: ch.id,
                selection: sel,
            },
        );
    }
}

pub fn advance_to_phase(e: &mut Engine, target_seat: u8, target_phase: Phase) {
    for _ in 0..48 {
        if e.turn().current == target_seat
            && e.turn().phase == target_phase
            && e.open_choice().is_none()
        {
            return;
        }
        if e.open_choice().is_some() {
            drain_auto(e);
            continue;
        }
        let cur = e.turn().current;
        let out = submit(e, cur, GameAction::NextPhase);
        if matches!(out, CommandOutcome::Rejected(_)) {
            panic!(
                "NextPhase rejected at seat {cur} {:?}: {out:?}, open={:?}",
                e.turn().phase,
                e.open_choice().map(|c| (c.observer, c.spec.prompt))
            );
        }
        drain_auto(e);
    }
    panic!(
        "failed to reach seat {target_seat} phase {target_phase:?}, stuck at seat {} {:?}, open={:?}",
        e.turn().current,
        e.turn().phase,
        e.open_choice().map(|c| (c.observer, c.spec.prompt, c.spec.allow_cancel, c.spec.options.len()))
    );
}

pub fn advance_until_open_prompt(
    e: &mut Engine,
    prompt: &str,
    max_steps: usize,
) -> Option<OpenChoice> {
    for _ in 0..max_steps {
        if let Some(ch) = e.open_choice().cloned() {
            if ch.spec.prompt == prompt {
                return Some(ch);
            }
            let sel = if ch.spec.prompt == "trigger_confirm" {
                vec![ChoiceOption::Confirm]
            } else if ch.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else if ch.spec.allow_cancel {
                Vec::new()
            } else {
                ch.spec
                    .options
                    .iter()
                    .take(ch.spec.min.max(1))
                    .copied()
                    .collect()
            };
            let _ = submit(
                e,
                ch.observer,
                GameAction::Answer {
                    choice_id: ch.id,
                    selection: sel,
                },
            );
            continue;
        }
        let cur = e.turn().current;
        let _ = submit(e, cur, GameAction::NextPhase);
    }
    None
}

pub fn clear_hand(e: &mut Engine, seat: u8) {
    let cards: Vec<u32> = e.state_core_mut().player_mut(seat).hand.drain(..).collect();
    e.state_core_mut().discard.extend(cards);
}

#[allow(dead_code)]
pub fn clear_equips(e: &mut Engine, seat: u8) {
    let cards: Vec<u32> = e.state_core_mut().player_mut(seat).equips.drain(..).collect();
    e.state_core_mut().discard.extend(cards);
}

pub fn total_tracked_cards(e: &Engine) -> usize {
    let core = e.state_core();
    let mut n = core.deck.len() + core.discard.len() + core.table.len() + core.detached_cards.len();
    for p in &core.players {
        n += p.hand.len() + p.equips.len() + p.judge.len() + p.off_court.len();
    }
    for pile in core.vm_aux.piles.values() {
        n += pile.len();
    }
    n
}

pub fn remove_cid_everywhere(e: &mut Engine, cid: u32) {
    let core = e.state_core_mut();
    for p in 0..core.players.len() as u8 {
        core.player_mut(p).hand.retain(|&x| x != cid);
        core.player_mut(p).equips.retain(|&x| x != cid);
        core.player_mut(p).judge.retain(|&x| x != cid);
        core.player_mut(p).off_court.retain(|&x| x != cid);
    }
    core.deck.retain(|&x| x != cid);
    core.discard.retain(|&x| x != cid);
    core.table.retain(|&x| x != cid);
    core.detached_cards.retain(|&(x, _)| x != cid);
    for pile in core.vm_aux.piles.values_mut() {
        pile.retain(|&x| x != cid);
    }
}

#[allow(dead_code)]
pub fn find_card_matching(e: &Engine, base_or_exact: &str) -> u32 {
    let cat = e.catalog();
    if let Some(id) = cat.cards.iter().position(|c| c.name == base_or_exact) {
        return id as u32;
    }
    let stem_eq = |name: &str, target: &str| -> bool {
        let s = name
            .strip_prefix("ht_")
            .or_else(|| name.strip_prefix("hs_"))
            .unwrap_or(name);
        if s == target {
            return true;
        }
        if let Some((head, tail)) = s.rsplit_once('_') {
            if !tail.is_empty() && tail.bytes().all(|b| b.is_ascii_digit()) {
                return head == target;
            }
        }
        false
    };
    if let Some(id) = cat.cards.iter().position(|c| stem_eq(c.name, base_or_exact)) {
        return id as u32;
    }
    panic!("card matching `{base_or_exact}` not found in catalog");
}

#[allow(dead_code)]
pub fn give_card_named(e: &mut Engine, seat: u8, name: &str) -> u32 {
    let cid = find_card_matching(e, name);
    remove_cid_everywhere(e, cid);
    e.state_core_mut().player_mut(seat).hand.push(cid);
    cid
}

pub fn give_free_base_named(e: &mut Engine, seat: u8, base: &str) -> u32 {
    let cat = e.catalog();
    let core = e.state_core();
    let matches_base = |c: u32| {
        let raw = cat.cards[c as usize].name;
        let s = raw.strip_prefix("hs_").unwrap_or(raw);
        s == base
            || s.rsplit_once('_')
                .is_some_and(|(h, t)| h == base && !t.is_empty() && t.bytes().all(|b| b.is_ascii_digit()))
    };
    let cid = core
        .deck
        .iter()
        .chain(core.discard.iter())
        .chain(core.detached_cards.iter().map(|(c, _)| c))
        .copied()
        .find(|&c| matches_base(c))
        .unwrap_or_else(|| find_card_matching(e, base));
    remove_cid_everywhere(e, cid);
    e.state_core_mut().player_mut(seat).hand.push(cid);
    cid
}

pub fn give_free_card_kind_suit(
    e: &mut Engine,
    seat: u8,
    kind: CardKind,
    suit: Option<engine_native::catalog::Suit>,
) -> u32 {
    let cat = e.catalog();
    let core = e.state_core();
    let matches_pred = |c: u32| {
        let d = &cat.cards[c as usize];
        d.kind == kind && suit.is_none_or(|s| d.suit == s)
    };
    let cid = core
        .deck
        .iter()
        .chain(core.discard.iter())
        .chain(core.detached_cards.iter().map(|(c, _)| c))
        .copied()
        .find(|&c| matches_pred(c))
        .unwrap_or_else(|| {
            (0..cat.cards.len() as u32)
                .find(|&c| matches_pred(c))
                .unwrap_or_else(|| panic!("no card matching {kind:?} {suit:?}"))
        });
    remove_cid_everywhere(e, cid);
    e.state_core_mut().player_mut(seat).hand.push(cid);
    cid
}

pub fn put_card_on_deck_top(e: &mut Engine, cid: u32) {
    remove_cid_everywhere(e, cid);
    e.state_core_mut().deck.push(cid);
}

pub fn grant_host_skill(e: &mut Engine, seat: u8, name: &str) {
    let def = e
        .catalog()
        .skill_by_name(name)
        .unwrap_or_else(|| panic!("missing skill `{name}`"));
    e.state_core_mut().skills.insert(
        (seat, def),
        engine_native::state::SkillInstance {
            owner: seat,
            enabled: true,
            once_used: false,
            expires_end_of_turn: false,
            rounds: 0,
        },
    );
}

pub fn has_skill(e: &Engine, p: u8, name: &str) -> bool {
    let Some(def) = e.catalog().skill_by_name(name) else {
        return false;
    };
    e.state_core().has_skill(p, def)
}

pub fn flag(e: &Engine, p: u8, key: &str) -> u32 {
    e.state_core()
        .vm_aux
        .flags
        .get(&(p, key.to_string()))
        .copied()
        .unwrap_or(0)
}

pub fn save_restore_and_answer(e: &mut Engine, expected_prompt: &str, sel: Vec<ChoiceOption>) {
    let ch_before = e
        .open_choice()
        .cloned()
        .unwrap_or_else(|| panic!("expected open_choice `{expected_prompt}`"));
    assert_eq!(ch_before.spec.prompt, expected_prompt);
    let saved = e.save();
    let bytes = encode_archive(&saved);
    let decoded = decode_archive(&bytes).expect("decode NGSA 107 archive");
    let mut restored =
        restore_locked_engine(&decoded, hearth_verified()).expect("restore_locked_engine");
    assert_eq!(restored.state_hash(), e.state_hash());
    let ch_after = restored
        .open_choice()
        .cloned()
        .expect("restored engine must preserve pending choice");
    assert_eq!(ch_after.spec.prompt, ch_before.spec.prompt);
    assert_eq!(ch_after.observer, ch_before.observer);
    let out = submit(
        &mut restored,
        ch_after.observer,
        GameAction::Answer {
            choice_id: ch_after.id,
            selection: sel,
        },
    );
    assert!(
        !matches!(out, CommandOutcome::Rejected(_)),
        "restored Answer({expected_prompt}) rejected: {out:?}"
    );
    *e = restored;
}

pub fn ai_choose_current(e: &Engine) -> Vec<ChoiceOption> {
    let ch = e.open_choice().expect("open_choice for AI");
    let actor = ch.observer;
    let plan = e.plan_action(actor, &BoundedPlanner::new(4));
    match plan.action {
        GameAction::Answer { selection, .. } => selection,
        other => panic!("expected Answer from AI while choice open, got {other:?}"),
    }
}
