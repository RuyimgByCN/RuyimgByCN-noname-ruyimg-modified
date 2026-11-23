use content_native_loader::{
    prepare_locked_content_with_bridge, restore_locked_engine, start_locked_engine,
    validate_and_freeze_raw_package, LockedStartMode, VerifiedLockedContent,
};
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardKind, Suit};
use engine_native::event::DamageNature;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(9_100_000);

pub(super) fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

pub(super) trait CatalogExtCompat {
    fn generals_count_compat(&self) -> usize;
}

impl CatalogExtCompat for engine_native::catalog::Catalog {
    fn generals_count_compat(&self) -> usize {
        self.standard_generals.len() + self.extension_generals.len() + self.boss_generals.len()
    }
}

pub(super) fn repo_extensions_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("packages")
        .join("extensions")
}

pub(super) fn prepare_formal_packages(dirs: &[&str]) -> VerifiedLockedContent {
    let ext_root = repo_extensions_root();
    let mut frozen_list = Vec::with_capacity(dirs.len());
    for dir_name in dirs {
        let pkg_dir = ext_root.join(dir_name);
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
        let frozen = validate_and_freeze_raw_package(raw_json, assets)
            .unwrap_or_else(|e| panic!("freeze {dir_name}: {e}"));
        frozen_list.push(frozen);
    }
    prepare_locked_content_with_bridge(frozen_list).expect("prepare formal packages")
}

pub(super) fn start_xianjian_only(seed: u64, seats: u8) -> (VerifiedLockedContent, Engine) {
    let verified = prepare_formal_packages(&["shaai-xianjian-upstream-noname"]);
    let e = start_locked_engine(seed, seats.into(), LockedStartMode::Prototype, &verified).expect("start xianjian");
    (verified, e)
}

pub(super) fn start_xianjian_full(seed: u64, seats: u8) -> (VerifiedLockedContent, Engine) {
    let verified = prepare_formal_packages(&[
        "shahai-swd-upstream-noname",
        "shahai-gujian-upstream-noname",
        "shaai-xianjian-upstream-noname",
    ]);
    let e = start_locked_engine(seed, seats.into(), LockedStartMode::Prototype, &verified)
        .expect("start xianjian + swd + gujian");
    (verified, e)
}

pub(super) fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action,
    })
}

pub(super) fn advance_to_play(e: &mut Engine, actor: u8) {
    for _ in 0..40 {
        if let Some(c) = e.open_choice().cloned() {
            let sel = if c.spec.allow_cancel {
                Vec::new()
            } else if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else {
                c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
            };
            let _ = submit(e, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
            continue;
        }
        let t = e.turn();
        if t.current == actor && t.phase == engine_native::event::Phase::Play {
            return;
        }
        let _ = submit(e, t.current, GameAction::NextPhase);
    }
    panic!("failed to reach player {actor} Play phase: {:?}", e.turn());
}

pub(super) fn assemble_general_by_name(e: &mut Engine, seat: u8, name: &str) {
    let cat = e.catalog();
    let def = (0..cat.generals_count_compat() as u16)
        .find(|&id| cat.general_meta(id).is_some_and(|(n, ..)| n == name))
        .unwrap_or_else(|| panic!("missing general {name}"));
    e.debug_assemble_general_def(seat, def);
}

pub(super) fn step_full_turn_to(e: &mut Engine, actor: u8) {
    for _ in 0..64 {
        if let Some(c) = e.open_choice().cloned() {
            let sel = if c.spec.allow_cancel {
                Vec::new()
            } else if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else {
                c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
            };
            let _ = submit(e, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
            continue;
        }
        let cur = e.turn().current;
        let _ = submit(e, cur, GameAction::NextPhase);
        if e.turn().current == actor && e.turn().phase == engine_native::event::Phase::Play {
            return;
        }
    }
}

pub(super) fn drain_pass_all(e: &mut Engine) {
    for _ in 0..40 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = submit(e, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
    }
}

pub(super) fn answer_open(e: &mut Engine, sel: Vec<ChoiceOption>) -> CommandOutcome {
    let c = e.open_choice().expect("expected open choice").clone();
    submit(e, c.observer, GameAction::Answer { choice_id: c.id, selection: sel })
}

pub(super) fn roundtrip_choice(e: Engine, verified: &VerifiedLockedContent) -> Engine {
    let snap = e.save();
    let bytes = encode_archive(&snap);
    let decoded = decode_archive(&bytes).expect("decode archive");
    restore_locked_engine(&decoded, verified).expect("restore locked engine")
}

pub(super) fn place_kind_suit(e: &mut Engine, _seat: u8, kind: CardKind, suit_filter: impl Fn(Suit) -> bool) -> Option<u32> {
    (0..e.catalog().cards.len() as u32).find(|&id| {
        let cd = &e.catalog().cards[id as usize];
        cd.kind == kind && suit_filter(cd.suit)
    })
}

pub(super) fn force_move_card_to_hand(e: &mut Engine, verified: &VerifiedLockedContent, seat: u8, cid: u32) {
    let mut snap = e.save();
    snap.core.deck.retain(|&x| x != cid);
    snap.core.discard.retain(|&x| x != cid);
    snap.core.table.retain(|&x| x != cid);
    snap.core.detached_cards.retain(|&(x, _)| x != cid);
    for pile in snap.core.vm_aux.piles.values_mut() {
        pile.retain(|&x| x != cid);
    }
    for pl in &mut snap.core.players {
        pl.hand.retain(|&x| x != cid);
        pl.equips.retain(|&x| x != cid);
        pl.judge.retain(|&x| x != cid);
        pl.off_court.retain(|&x| x != cid);
    }
    snap.core.players[seat as usize].hand.push(cid);
    *e = restore_locked_engine(&snap, verified).expect("restore after card place");
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

pub(super) fn deal_damage(e: &mut Engine, src: Option<u8>, target: u8, amt: i32, nat: DamageNature) {
    e.debug_deal_damage(src, target, amt as u32, nat);
}

pub(super) fn place_top_of_deck(e: &mut Engine, verified: &VerifiedLockedContent, top_first: &[u32]) {
    let mut snap = e.save();
    for &cid in top_first {
        snap.core.deck.retain(|&x| x != cid);
        snap.core.discard.retain(|&x| x != cid);
        snap.core.table.retain(|&x| x != cid);
        snap.core.detached_cards.retain(|&(x, _)| x != cid);
        for pile in snap.core.vm_aux.piles.values_mut() {
            pile.retain(|&x| x != cid);
        }
        for pl in &mut snap.core.players {
            pl.hand.retain(|&x| x != cid);
            pl.equips.retain(|&x| x != cid);
            pl.judge.retain(|&x| x != cid);
            pl.off_court.retain(|&x| x != cid);
        }
    }
    for &cid in top_first.iter().rev() {
        snap.core.deck.push(cid);
    }
    *e = restore_locked_engine(&snap, verified).expect("restore deck top");
}
