//! Batch U05-1 (`U05 杀海拾遗` 5 sub-packages = 94 rule-bearing items) TDD parity suite.
//!
//! Covers upstream `archive/upstream/noname/apps/core/extension/杀海拾遗/`:
//! - `main/` (`stone_chenshui`, `wuxing`, `rand`, `mtg`, `zhenfa`, `gwent`, `yunchou`)
//! - `play/wuxing/` (`wuxingpan`, `_shengke`, `wuxingpan_skill`)
//! - `card/zhenfa/` (all 11 zhenfa cards + `recastable`)
//! - `card/mtg/` + `character/mtg/` (12 land cards + 12 land skills + 7 char skills + 3 generals)
//! - `card/yunchou/` + `character/yunchou/` (10 cards + 2 card skills + 17 char skills + 10 generals)
//!
//! Uses only real `GameAction` paths (PlayCard / ActivateSkill / NextPhase / Answer) —
//! zero `debug_execute_effect` — plus public-info AI checks and mid-choice NGSA 107 save/restore.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use content_native_loader::{
    prepare_locked_content_with_bridge, restore_locked_engine, start_locked_engine_with_shahai_config,
    validate_and_freeze_raw_package, FrozenPackage, LockedStartMode, ShahaiMainConfig,
    VerifiedLockedContent,
};
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardKind, Faction, Suit};
use engine_native::content::standard::Gender;
use engine_native::event::DamageNature;
use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand,
    GameMode, Phase, Role,
};

static DEDUP: AtomicU64 = AtomicU64::new(9_200_000);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn freeze_pkg(dir_name: &str) -> FrozenPackage {
    let pkg_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions")
        .join(dir_name);
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

fn load_u05_b1_verified(pkg_dirs: &[&str]) -> VerifiedLockedContent {
    let frozen: Vec<FrozenPackage> = pkg_dirs.iter().map(|d| freeze_pkg(d)).collect();
    prepare_locked_content_with_bridge(frozen).expect("prepare locked U05-1 packs")
}

fn start_u05_b1_engine(
    seed: u64,
    seats: usize,
    mode: LockedStartMode,
    cfg: &ShahaiMainConfig,
    pkg_dirs: &[&str],
) -> (VerifiedLockedContent, Engine) {
    let verified = load_u05_b1_verified(pkg_dirs);
    let engine = start_locked_engine_with_shahai_config(seed, seats, mode, &verified, cfg)
        .expect("start locked U05-1 engine");
    (verified, engine)
}

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action,
    })
}

fn answer(e: &mut Engine, expected_prompt: &str, selection: Vec<ChoiceOption>) {
    let c = e
        .open_choice()
        .unwrap_or_else(|| panic!("expected open_choice {expected_prompt:?}, got None"))
        .clone();
    assert_eq!(
        c.spec.prompt, expected_prompt,
        "unexpected prompt: got {}, expected {expected_prompt}",
        c.spec.prompt
    );
    let out = submit(
        e,
        c.observer,
        GameAction::Answer {
            choice_id: c.id,
            selection,
        },
    );
    assert!(
        !matches!(out, CommandOutcome::Rejected(_)),
        "Answer({expected_prompt}) rejected: {out:?}"
    );
}

fn snapshot_roundtrip(e: &Engine, verified: &VerifiedLockedContent) -> Engine {
    let bytes = encode_archive(&e.save());
    let snap = decode_archive(&bytes).expect("decode NGSA 107");
    let restored = restore_locked_engine(&snap, verified).expect("restore NGSA 107");
    assert_eq!(restored.state_hash(), e.state_hash(), "state hash mismatch on restore");
    assert_eq!(restored.open_choice(), e.open_choice(), "open_choice mismatch on restore");
    restored
}

fn drain(e: &mut Engine) {
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

fn advance_to_phase(e: &mut Engine, seat: u8, target: Phase) {
    for _ in 0..80 {
        if e.turn().current == seat && e.turn().phase == target && e.open_choice().is_none() {
            return;
        }
        if e.open_choice().is_some() {
            drain(e);
            continue;
        }
        let actor = e.turn().current;
        let _ = submit(e, actor, GameAction::NextPhase);
    }
    panic!("failed to advance to seat {seat} {target:?}");
}

fn find_card(e: &Engine, pred: impl Fn(&engine_native::catalog::CardDef) -> bool) -> u32 {
    e.catalog()
        .cards
        .iter()
        .position(pred)
        .expect("matching card in catalog") as u32
}

fn find_card_excl(
    e: &Engine,
    excl: &[u32],
    pred: impl Fn(&engine_native::catalog::CardDef) -> bool,
) -> u32 {
    e.catalog()
        .cards
        .iter()
        .enumerate()
        .position(|(i, c)| !excl.contains(&(i as u32)) && pred(c))
        .expect("matching card in catalog") as u32
}

fn move_card_to_hand(e: &mut Engine, seat: u8, cid: u32) {
    let core = e.state_core_mut();
    for p in 0..core.players.len() as u8 {
        core.player_mut(p).hand.retain(|&c| c != cid);
        core.player_mut(p).equips.retain(|&c| c != cid);
        core.player_mut(p).judge.retain(|&c| c != cid);
    }
    core.deck.retain(|&c| c != cid);
    core.discard.retain(|&c| c != cid);
    core.table.retain(|&c| c != cid);
    for pile in core.vm_aux.piles.values_mut() {
        pile.retain(|&c| c != cid);
    }
    core.player_mut(seat).hand.push(cid);
}

fn put_deck_top(e: &mut Engine, cid: u32) {
    let core = e.state_core_mut();
    for p in 0..core.players.len() as u8 {
        core.player_mut(p).hand.retain(|&c| c != cid);
        core.player_mut(p).equips.retain(|&c| c != cid);
        core.player_mut(p).judge.retain(|&c| c != cid);
    }
    core.deck.retain(|&c| c != cid);
    core.discard.retain(|&c| c != cid);
    core.table.retain(|&c| c != cid);
    for pile in core.vm_aux.piles.values_mut() {
        pile.retain(|&c| c != cid);
    }
    core.deck.push(cid);
}

fn activate(e: &mut Engine, seat: u8, skill_name: &str) -> CommandOutcome {
    let id = e
        .catalog()
        .skill_by_name(skill_name)
        .unwrap_or_else(|| panic!("skill {skill_name} missing in catalog"));
    submit(e, seat, GameAction::ActivateSkill { skill: id })
}

fn total_tracked_cards(e: &Engine) -> usize {
    let core = e.state_core();
    let mut n = core.deck.len() + core.discard.len() + core.table.len();
    for p in &core.players {
        n += p.hand.len() + p.equips.len() + p.judge.len();
    }
    for pile in core.vm_aux.piles.values() {
        n += pile.len();
    }
    n
}

fn flag(e: &Engine, p: u8, key: &str) -> u32 {
    e.state_core()
        .vm_aux
        .flags
        .get(&(p, key.to_string()))
        .copied()
        .unwrap_or(0)
}

#[path = "u05_b1_fixes/part1_config_stone_wuxing_zhenfa.rs"]
mod part1_config_stone_wuxing_zhenfa;
#[path = "u05_b1_fixes/part2_mtg.rs"]
mod part2_mtg;
#[path = "u05_b1_fixes/part3_yunchou_cards.rs"]
mod part3_yunchou_cards;
#[path = "u05_b1_fixes/part4_yunchou_chars.rs"]
mod part4_yunchou_chars;
#[path = "u05_b1_fixes/part5_round2_fixes.rs"]
mod part5_round2_fixes;
#[path = "u05_b1_fixes/part6_round3_fixes.rs"]
mod part6_round3_fixes;
