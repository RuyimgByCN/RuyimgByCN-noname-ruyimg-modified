#![allow(dead_code)]

use content_native_loader::{
    prepare_locked_content_with_bridge, restore_locked_engine, start_locked_engine,
    validate_and_freeze_raw_package, LockedStartMode, VerifiedLockedContent,
};
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardKind, Suit};
use engine_native::event::Phase;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(920_000);

pub(super) fn repo_extensions_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

pub(super) fn freeze_formal_package(dir_name: &str) -> content_native_loader::FrozenPackage {
    let pkg_dir = repo_extensions_root().join(dir_name);
    let raw_json = std::fs::read_to_string(pkg_dir.join("package.npack.json"))
        .unwrap_or_else(|e| panic!("read {dir_name}/package.npack.json: {e}"))
        .replace("\r\n", "\n");
    let parsed: serde_json::Value = serde_json::from_str(&raw_json).unwrap();
    let mut assets = BTreeMap::new();
    if let Some(paths) = parsed.get("asset_paths").and_then(|v| v.as_object()) {
        for rel_path in paths.values().filter_map(|v| v.as_str()) {
            let bytes = std::fs::read(pkg_dir.join(rel_path))
                .unwrap_or_else(|e| panic!("read {dir_name}/{rel_path}: {e}"))
                .into_iter()
                .filter(|&b| b != b'\r')
                .collect::<Vec<u8>>();
            assets.insert(rel_path.to_string(), bytes);
        }
    }
    validate_and_freeze_raw_package(raw_json, assets)
        .unwrap_or_else(|e| panic!("freeze {dir_name}: {e}"))
}

pub(super) fn start_shahai_swd_only(seed: u64, players: usize) -> (VerifiedLockedContent, Engine) {
    let frozen = freeze_formal_package("shahai-swd-upstream-noname");
    let verified = prepare_locked_content_with_bridge(vec![frozen]).expect("prepare shahai-swd");
    let engine = start_locked_engine(seed, players, LockedStartMode::Prototype, &verified)
        .expect("start locked engine with shahai-swd");
    (verified, engine)
}

pub(super) fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(10_000, Ordering::Relaxed),
        action,
    })
}

pub(super) fn pass_nullify(e: &mut Engine) {
    while let Some(c) = e.open_choice() {
        if c.spec.prompt != "nullify_window" {
            break;
        }
        let (cid, obs) = (c.id, c.observer);
        let _ = submit(
            e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
}

pub(super) fn drain_pass_all(e: &mut Engine) {
    for _ in 0..48 {
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

pub(super) fn advance_to_play(e: &mut Engine, actor: u8) {
    for _ in 0..96 {
        if let Some(c) = e.open_choice().cloned() {
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
            continue;
        }
        let t = e.turn();
        if t.current == actor && t.phase == Phase::Play {
            return;
        }
        let _ = submit(e, t.current, GameAction::NextPhase);
    }
    panic!("failed to reach player {actor} Play phase: {:?}", e.turn());
}

pub(super) fn step_full_turn_to(e: &mut Engine, actor: u8) {
    for _ in 0..96 {
        if let Some(c) = e.open_choice().cloned() {
            let sel = if c.spec.allow_cancel {
                Vec::new()
            } else if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
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
            continue;
        }
        let cur = e.turn().current;
        let _ = submit(e, cur, GameAction::NextPhase);
        if e.turn().current == actor && e.turn().phase == Phase::Play {
            return;
        }
    }
}

pub(super) fn answer_open(e: &mut Engine, sel: Vec<ChoiceOption>) -> CommandOutcome {
    let c = e.open_choice().expect("expected open choice").clone();
    submit(
        e,
        c.observer,
        GameAction::Answer {
            choice_id: c.id,
            selection: sel,
        },
    )
}

pub(super) fn roundtrip_choice(e: Engine, verified: &VerifiedLockedContent) -> Engine {
    let snap = e.save();
    let bytes = encode_archive(&snap);
    let decoded = decode_archive(&bytes).expect("decode archive");
    restore_locked_engine(&decoded, verified).expect("restore locked engine")
}

pub(super) fn find_kind(e: &Engine, kind: CardKind) -> u32 {
    (0..e.catalog().cards.len() as u32)
        .find(|&id| e.catalog().card_kind(id) == kind)
        .unwrap_or_else(|| panic!("card kind missing: {kind:?}"))
}

pub(super) fn find_kind_suit(
    e: &Engine,
    kind: CardKind,
    suit_filter: impl Fn(Suit) -> bool,
) -> Option<u32> {
    (0..e.catalog().cards.len() as u32).find(|&id| {
        let cd = &e.catalog().cards[id as usize];
        cd.kind == kind && suit_filter(cd.suit)
    })
}

pub(super) fn clear_hand_to_discard(
    e: &mut Engine,
    verified: &VerifiedLockedContent,
    seat: u8,
) {
    let mut snap = e.save();
    let drained: Vec<u32> = snap.core.players[seat as usize].hand.drain(..).collect();
    snap.core.discard.extend(drained);
    *e = restore_locked_engine(&snap, verified).expect("restore after hand clear");
}

pub(super) fn find_card_name(e: &Engine, keyword: &str) -> u32 {
    (0..e.catalog().cards.len() as u32)
        .find(|&id| {
            let name = e.catalog().cards[id as usize].name;
            name == keyword
                || name == format!("swd_{keyword}")
                || name.starts_with(&format!("swd_{keyword}"))
                || name.ends_with(&format!("_{keyword}"))
        })
        .unwrap_or_else(|| panic!("card keyword missing: {keyword}"))
}

pub(super) fn force_move_card_to_hand(
    e: &mut Engine,
    verified: &VerifiedLockedContent,
    seat: u8,
    cid: u32,
) {
    let mut snap = e.save();
    remove_card_everywhere(&mut snap.core, cid);
    snap.core.players[seat as usize].hand.push(cid);
    *e = restore_locked_engine(&snap, verified).expect("restore after card place");
}

pub(super) fn force_equip_card(
    e: &mut Engine,
    verified: &VerifiedLockedContent,
    seat: u8,
    cid: u32,
) {
    let mut snap = e.save();
    remove_card_everywhere(&mut snap.core, cid);
    snap.core.players[seat as usize].equips.push(cid);
    *e = restore_locked_engine(&snap, verified).expect("restore after force equip");
}

pub(super) fn place_top_of_deck(
    e: &mut Engine,
    verified: &VerifiedLockedContent,
    top_first: &[u32],
) {
    let mut snap = e.save();
    for &cid in top_first {
        remove_card_everywhere(&mut snap.core, cid);
    }
    for &cid in top_first.iter().rev() {
        snap.core.deck.push(cid);
    }
    *e = restore_locked_engine(&snap, verified).expect("restore deck top");
}

fn remove_card_everywhere(core: &mut engine_native::state::GameStateCore, cid: u32) {
    core.deck.retain(|&x| x != cid);
    core.discard.retain(|&x| x != cid);
    core.table.retain(|&x| x != cid);
    core.detached_cards.retain(|&(x, _)| x != cid);
    for pile in core.vm_aux.piles.values_mut() {
        pile.retain(|&x| x != cid);
    }
    core.declared_kinds.remove(&cid);
    core.declared_slots.remove(&cid);
    for pl in &mut core.players {
        pl.hand.retain(|&x| x != cid);
        pl.equips.retain(|&x| x != cid);
        pl.judge.retain(|&x| x != cid);
        pl.off_court.retain(|&x| x != cid);
        pl.jiguanyuan_cards.retain(|&x| x != cid);
    }
}

pub(super) fn total_card_set(e: &Engine) -> (usize, BTreeSet<u32>) {
    let st = e.state_core();
    let mut all = Vec::new();
    all.extend(st.deck.iter().copied());
    all.extend(st.discard.iter().copied());
    all.extend(st.table.iter().copied());
    all.extend(st.detached_cards.iter().map(|&(c, _)| c));
    for p in &st.players {
        all.extend(p.hand.iter().copied());
        all.extend(p.equips.iter().copied());
        all.extend(p.judge.iter().copied());
        all.extend(p.off_court.iter().copied());
    }
    for pile in st.vm_aux.piles.values() {
        all.extend(pile.iter().copied());
    }
    let count = all.len();
    let uniq: BTreeSet<u32> = all.into_iter().collect();
    (count, uniq)
}
