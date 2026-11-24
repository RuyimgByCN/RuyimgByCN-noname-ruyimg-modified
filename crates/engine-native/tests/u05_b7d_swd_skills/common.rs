use content_native_loader::{
    prepare_locked_content_with_bridge, start_locked_engine, validate_and_freeze_raw_package,
    LockedStartMode, VerifiedLockedContent,
};
pub use engine_native::ai::BoundedPlanner;
pub use engine_native::archive::{decode_archive, encode_archive};
pub use engine_native::catalog::{CardKind, Suit};
pub use engine_native::state::Role;
pub use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase, RejectReason,
};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(7_050_000);

pub fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

pub fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action,
    })
}

pub fn load_formal_swd_only(seed: u64, seats: u8) -> (VerifiedLockedContent, Engine) {
    load_formal_swd_only_mode(seed, seats, LockedStartMode::Prototype)
}

pub fn load_formal_swd_only_mode(
    seed: u64,
    seats: u8,
    mode: LockedStartMode,
) -> (VerifiedLockedContent, Engine) {
    let ext_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let pkg_dir = ext_root.join("shahai-swd-upstream-noname");
    let raw_json = std::fs::read_to_string(pkg_dir.join("package.npack.json"))
        .expect("read package.npack.json")
        .replace("\r\n", "\n");
    let parsed: serde_json::Value = serde_json::from_str(&raw_json).unwrap();
    let mut assets = BTreeMap::new();
    if let Some(paths) = parsed.get("asset_paths").and_then(|v| v.as_object()) {
        for rel_path in paths.values().filter_map(|v| v.as_str()) {
            let bytes = std::fs::read(pkg_dir.join(rel_path))
                .unwrap()
                .into_iter()
                .filter(|&b| b != b'\r')
                .collect::<Vec<u8>>();
            assets.insert(rel_path.to_string(), bytes);
        }
    }
    let frozen = validate_and_freeze_raw_package(raw_json, assets).expect("freeze shahai-swd");
    let verified = prepare_locked_content_with_bridge(vec![frozen]).expect("prepare shahai-swd");
    let engine = start_locked_engine(seed, usize::from(seats), mode, &verified)
        .expect("start swd");
    (verified, engine)
}

pub fn place_matching_card(
    e: &mut Engine,
    seat: u8,
    pred: impl Fn(u32, &engine_native::catalog::CardDef) -> bool,
) -> u32 {
    let cid = (0..e.catalog().cards.len() as u32)
        .find(|&id| pred(id, &e.catalog().cards[id as usize]))
        .expect("matching card in catalog");
    let core = e.state_core_mut();
    core.deck.retain(|&x| x != cid);
    core.discard.retain(|&x| x != cid);
    core.table.retain(|&x| x != cid);
    for p in core.players.iter_mut() {
        p.hand.retain(|&x| x != cid);
        p.equips.retain(|&x| x != cid);
        p.judge.retain(|&x| x != cid);
    }
    core.players[seat as usize].hand.push(cid);
    cid
}

pub fn set_deck_top_matching(
    e: &mut Engine,
    pred: impl Fn(&engine_native::catalog::CardDef) -> bool,
) -> u32 {
    e.debug_set_deck_top_by(pred)
}

pub fn settle_confirm(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.options.contains(&ChoiceOption::Confirm) {
            vec![ChoiceOption::Confirm]
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

pub fn advance_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..128 {
        if e.turn().current == seat && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            return;
        }
        if e.open_choice().is_some() {
            settle_confirm(e);
            continue;
        }
        let cur = e.turn().current;
        let _ = submit(e, cur, GameAction::NextPhase);
    }
    panic!("failed to reach Phase::Play for seat {seat}");
}

pub fn roundtrip_bytes(verified: &VerifiedLockedContent, e: &Engine) -> Engine {
    let bytes = encode_archive(&e.save());
    let decoded = decode_archive(&bytes).expect("decode archive");
    content_native_loader::restore_locked_engine(&decoded, verified).expect("restore archive")
}
