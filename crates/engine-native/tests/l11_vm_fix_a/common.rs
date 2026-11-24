#![allow(dead_code)]

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

use content_native_loader::{
    prepare_locked_content_with_bridge, restore_locked_engine, start_locked_engine,
    validate_and_freeze_raw_package, LockedStartMode, VerifiedLockedContent,
};
use engine_native::ai::BoundedPlanner;
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardKind, EquipSlot, Suit};
use engine_native::ids::{CardId, PlayerId};
use engine_native::state::Role;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};

static TAIXU_VERIFIED: OnceLock<VerifiedLockedContent> = OnceLock::new();
static DEDUP: AtomicU64 = AtomicU64::new(91_000_000);

pub fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

pub fn taixu_verified() -> &'static VerifiedLockedContent {
    TAIXU_VERIFIED.get_or_init(|| {
        let pkg_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/extensions/taixu-upstream-liuli");
        let raw_json = std::fs::read_to_string(pkg_dir.join("package.npack.json"))
            .expect("read taixu package.npack.json")
            .replace("\r\n", "\n");
        let parsed: serde_json::Value = serde_json::from_str(&raw_json).unwrap();
        let mut assets = BTreeMap::new();
        if let Some(paths) = parsed.get("asset_paths").and_then(|v| v.as_object()) {
            for rel_path in paths.values().filter_map(|v| v.as_str()) {
                let bytes = std::fs::read(pkg_dir.join(rel_path))
                    .expect("read taixu asset")
                    .into_iter()
                    .filter(|&b| b != b'\r')
                    .collect::<Vec<u8>>();
                assets.insert(rel_path.to_string(), bytes);
            }
        }
        let frozen = validate_and_freeze_raw_package(raw_json, assets).expect("freeze taixu");
        prepare_locked_content_with_bridge(vec![frozen]).expect("prepare locked taixu")
    })
}

pub fn submit(e: &mut Engine, actor: PlayerId, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action,
    })
}

pub fn answer(e: &mut Engine, actor: PlayerId, selection: Vec<ChoiceOption>) -> CommandOutcome {
    let ch = e.open_choice().expect("open choice");
    assert_eq!(ch.observer, actor, "observer mismatch on {}", ch.spec.prompt);
    submit(
        e,
        actor,
        GameAction::Answer {
            choice_id: ch.id,
            selection,
        },
    )
}

pub fn prompt_is(e: &Engine, expected: &str) -> bool {
    e.open_choice().is_some_and(|c| c.spec.prompt == expected)
}

pub fn place_by_suit(e: &mut Engine, seat: PlayerId, name: &str, suit: Suit) -> CardId {
    let cid = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find_map(|(i, cd)| {
            (cd.name == name
                && cd.suit == suit
                && !e
                    .state_core()
                    .players
                    .iter()
                    .any(|p| p.hand.contains(&(i as u32)) || p.equips.contains(&(i as u32)) || p.judge.contains(&(i as u32)))
                && !e.state_core().table.contains(&(i as u32)))
            .then_some(i as u32)
        })
        .unwrap_or_else(|| e.debug_place(seat, name));
    if !e.debug_hand(seat).contains(&cid) {
        e.state_core_mut().deck.retain(|&c| c != cid);
        e.state_core_mut().discard.retain(|&c| c != cid);
        e.state_core_mut().player_mut(seat).hand.push(cid);
    }
    cid
}

pub fn place_by_rank(
    e: &mut Engine,
    seat: PlayerId,
    name: &str,
    min_rank: u8,
    max_rank: u8,
) -> CardId {
    let cid = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find_map(|(i, cd)| {
            let id = i as u32;
            let free = !e
                .state_core()
                .players
                .iter()
                .any(|p| p.hand.contains(&id) || p.equips.contains(&id) || p.judge.contains(&id))
                && !e.state_core().table.contains(&id);
            (cd.name == name && cd.rank >= min_rank && cd.rank <= max_rank && free).then_some(id)
        })
        .unwrap_or_else(|| panic!("no free {name} in rank {min_rank}..={max_rank}"));
    e.state_core_mut().deck.retain(|&c| c != cid);
    e.state_core_mut().discard.retain(|&c| c != cid);
    e.state_core_mut().player_mut(seat).hand.push(cid);
    cid
}

pub fn place_kind(e: &mut Engine, seat: PlayerId, kind: CardKind) -> CardId {
    let cid = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find_map(|(i, cd)| {
            let id = i as u32;
            let free = !e
                .state_core()
                .players
                .iter()
                .any(|p| p.hand.contains(&id) || p.equips.contains(&id) || p.judge.contains(&id))
                && !e.state_core().table.contains(&id);
            (cd.kind == kind && free).then_some(id)
        })
        .unwrap_or_else(|| panic!("no free card of kind {kind:?}"));
    e.state_core_mut().deck.retain(|&c| c != cid);
    e.state_core_mut().discard.retain(|&c| c != cid);
    e.state_core_mut().player_mut(seat).hand.push(cid);
    cid
}

pub fn drain_ai(e: &mut Engine, max_steps: usize) {
    let ai = BoundedPlanner::new(16);
    for _ in 0..max_steps {
        if e.outcome().is_some() {
            break;
        }
        let Some(ch) = e.open_choice() else {
            break;
        };
        let obs = ch.observer;
        let act = e.plan_action(obs, &ai).action;
        let _ = submit(e, obs, act);
    }
}

pub fn advance_to_phase(e: &mut Engine, seat: PlayerId, phase: Phase) {
    let ai = BoundedPlanner::new(16);
    for _ in 0..200 {
        if e.outcome().is_some() {
            break;
        }
        if let Some(ch) = e.open_choice() {
            let obs = ch.observer;
            let act = e.plan_action(obs, &ai).action;
            let _ = submit(e, obs, act);
            continue;
        }
        let st = e.state_core();
        if st.turn.current == seat && st.turn.phase == phase {
            return;
        }
        if st.turn.phase == Phase::Play {
            let _ = submit(e, st.turn.current, GameAction::NextPhase);
        } else {
            let act = e.plan_action(st.turn.current, &ai).action;
            let _ = submit(e, st.turn.current, act);
        }
    }
    assert_eq!(e.state_core().turn.current, seat, "failed to reach seat {seat}");
    assert_eq!(e.state_core().turn.phase, phase, "failed to reach phase {phase:?}");
}

pub fn advance_to_choice_on_seat(e: &mut Engine, seat: PlayerId, phase: Phase) {
    let ai = BoundedPlanner::new(16);
    for _ in 0..200 {
        if e.outcome().is_some() {
            break;
        }
        if let Some(ch) = e.open_choice() {
            if e.state_core().turn.current == seat && e.state_core().turn.phase == phase {
                return;
            }
            let obs = ch.observer;
            let act = e.plan_action(obs, &ai).action;
            let _ = submit(e, obs, act);
            continue;
        }
        let st = e.state_core();
        if st.turn.current == seat && st.turn.phase == phase {
            return;
        }
        if st.turn.phase == Phase::Play {
            let _ = submit(e, st.turn.current, GameAction::NextPhase);
        } else {
            let act = e.plan_action(st.turn.current, &ai).action;
            let _ = submit(e, st.turn.current, act);
        }
    }
    assert_eq!(e.state_core().turn.current, seat, "failed to reach seat {seat}");
    assert_eq!(e.state_core().turn.phase, phase, "failed to reach phase {phase:?}");
}

pub fn start_clean_4p_play(seed: u64) -> Engine {
    let verified = taixu_verified();
    let mut e = start_locked_engine(seed, 4, LockedStartMode::Identity, verified)
        .expect("start 4p identity");
    e.debug_set_identity_roles(vec![Role::Lord, Role::Rebel, Role::Traitor, Role::Loyalist]);
    for s in 0..4 {
        e.debug_remove_general(s);
        e.debug_set_max_hp(s, 5);
        e.debug_set_hp(s, 5);
        e.debug_clear_hand(s);
    }
    e.debug_sweep_judge_zones();
    advance_to_phase(&mut e, 0, Phase::Play);
    for s in 0..4 {
        e.debug_clear_hand(s);
    }
    e
}

pub fn start_clean_2p_play(seed: u64) -> Engine {
    let verified = taixu_verified();
    let mut e = start_locked_engine(seed, 2, LockedStartMode::Prototype, verified)
        .expect("start 2p prototype");
    for s in 0..2 {
        e.debug_remove_general(s);
        e.debug_set_max_hp(s, 5);
        e.debug_set_hp(s, 5);
        e.debug_clear_hand(s);
    }
    e.debug_sweep_judge_zones();
    advance_to_phase(&mut e, 0, Phase::Play);
    for s in 0..2 {
        e.debug_clear_hand(s);
    }
    e
}

pub fn roundtrip_ngsa(e: &Engine, verified: &VerifiedLockedContent) -> Engine {
    let bytes = encode_archive(&e.save());
    let decoded = decode_archive(&bytes).expect("decode NGSA archive");
    restore_locked_engine(&decoded, verified).expect("restore locked engine")
}

pub fn place_slot(e: &mut Engine, seat: PlayerId, slot: EquipSlot) -> CardId {
    let cid = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find_map(|(i, cd)| {
            let id = i as u32;
            let free = !e
                .state_core()
                .players
                .iter()
                .any(|p| p.hand.contains(&id) || p.equips.contains(&id) || p.judge.contains(&id))
                && !e.state_core().table.contains(&id);
            (cd.kind.equip_slot() == Some(slot) && free).then_some(id)
        })
        .unwrap_or_else(|| panic!("no free card in slot {slot:?}"));
    e.state_core_mut().deck.retain(|&c| c != cid);
    e.state_core_mut().discard.retain(|&c| c != cid);
    e.state_core_mut().player_mut(seat).hand.push(cid);
    cid
}

pub fn place_vanilla_dismantle(e: &mut Engine, seat: PlayerId) -> CardId {
    let cid = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find_map(|(i, cd)| {
            let id = i as u32;
            let free = !e
                .state_core()
                .players
                .iter()
                .any(|p| p.hand.contains(&id) || p.equips.contains(&id) || p.judge.contains(&id))
                && !e.state_core().table.contains(&id);
            (cd.kind == CardKind::Dismantle && cd.name != "txhj_caomu" && free).then_some(id)
        })
        .expect("free vanilla dismantle card");
    e.state_core_mut().deck.retain(|&c| c != cid);
    e.state_core_mut().discard.retain(|&c| c != cid);
    e.state_core_mut().player_mut(seat).hand.push(cid);
    cid
}
