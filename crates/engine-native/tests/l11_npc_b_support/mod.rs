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
use engine_native::catalog::{CardKind, Suit};
use engine_native::event::Phase;
use engine_native::ids::{CardId, PlayerId};
use engine_native::state::Role;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};

static TAIXU_NPC_B_VERIFIED: OnceLock<VerifiedLockedContent> = OnceLock::new();
static NPC_B_DEDUP: AtomicU64 = AtomicU64::new(83_000_000);

pub fn next_dedup() -> u64 {
    NPC_B_DEDUP.fetch_add(1, Ordering::Relaxed)
}

pub fn taixu_verified() -> &'static VerifiedLockedContent {
    TAIXU_NPC_B_VERIFIED.get_or_init(|| {
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

pub fn start_2p(seed: u64) -> Engine {
    let mut e = start_locked_engine(seed, 2, LockedStartMode::Prototype, taixu_verified())
        .expect("start 2p locked taixu");
    e.debug_sweep_judge_zones();
    e
}

pub fn start_4p_identity(seed: u64) -> Engine {
    let mut e = start_locked_engine(seed, 4, LockedStartMode::Identity, taixu_verified())
        .expect("start 4p identity locked taixu");
    e.debug_set_identity_roles(vec![Role::Lord, Role::Rebel, Role::Traitor, Role::Loyalist]);
    e.debug_sweep_judge_zones();
    e
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
    let ch = e
        .open_choice()
        .unwrap_or_else(|| panic!("expected open choice for seat {actor}"))
        .clone();
    assert_eq!(
        ch.observer, actor,
        "choice observer mismatch for prompt {}",
        ch.spec.prompt
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

pub fn drain_pass(e: &mut Engine, max_steps: usize) {
    for _ in 0..max_steps {
        let Some(ch) = e.open_choice().cloned() else {
            break;
        };
        let sel = if ch.spec.options.contains(&ChoiceOption::Pass) {
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
    }
}

pub fn drain_with_ai(e: &mut Engine, max_steps: usize) {
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

pub fn advance_to_phase(
    e: &mut Engine,
    seat: PlayerId,
    phase: Phase,
    stop_prefix: Option<&str>,
) {
    for _ in 0..80 {
        if let Some(sp) = stop_prefix {
            if e.open_choice().is_some_and(|c| c.spec.prompt.starts_with(sp)) {
                return;
            }
        }
        drain_pass(e, 32);
        if let Some(sp) = stop_prefix {
            if e.open_choice().is_some_and(|c| c.spec.prompt.starts_with(sp)) {
                return;
            }
        }
        if e.state_core().turn.current == seat && e.state_core().turn.phase == phase {
            return;
        }
        let cur = e.state_core().turn.current;
        let _ = submit(e, cur, GameAction::NextPhase);
    }
    if let Some(sp) = stop_prefix {
        if e.open_choice().is_some_and(|c| c.spec.prompt.starts_with(sp)) {
            return;
        }
    }
    assert_eq!(e.state_core().turn.current, seat);
    assert_eq!(e.state_core().turn.phase, phase);
}

pub fn activate(e: &mut Engine, owner: PlayerId, skill_name: &str) -> CommandOutcome {
    let sid = e
        .catalog()
        .skill_by_name(skill_name)
        .unwrap_or_else(|| panic!("skill {skill_name} missing"));
    submit(e, owner, GameAction::ActivateSkill { skill: sid })
}

pub fn take_deck_or_discard_by<F>(e: &mut Engine, mut pred: F) -> CardId
where
    F: FnMut(&engine_native::catalog::CardDef) -> bool,
{
    let cat = e.catalog().clone();
    if let Some(pos) = e
        .state_core()
        .deck
        .iter()
        .position(|&c| pred(&cat.cards[c as usize]))
    {
        return e.state_core_mut().deck.remove(pos);
    }
    if let Some(pos) = e
        .state_core()
        .discard
        .iter()
        .position(|&c| pred(&cat.cards[c as usize]))
    {
        return e.state_core_mut().discard.remove(pos);
    }
    panic!("no matching card available in deck/discard");
}

pub fn place_hand_suit_kind(
    e: &mut Engine,
    seat: PlayerId,
    suit: Option<Suit>,
    kind: CardKind,
) -> CardId {
    let c = take_deck_or_discard_by(e, |d| {
        d.kind == kind && suit.is_none_or(|s| d.suit == s)
    });
    e.state_core_mut().player_mut(seat).hand.push(c);
    c
}

pub fn place_hand_kind_any(e: &mut Engine, seat: PlayerId, kind: CardKind) -> CardId {
    place_hand_suit_kind(e, seat, None, kind)
}

pub fn flag(e: &Engine, p: PlayerId, key: &str) -> u32 {
    e.state_core()
        .vm_aux
        .flags
        .get(&(p, key.to_string()))
        .copied()
        .unwrap_or(0)
}

pub fn roundtrip_restore(e: &Engine) -> Engine {
    let bytes = encode_archive(&e.save());
    let snap = decode_archive(&bytes).expect("decode archive");
    let restored = restore_locked_engine(&snap, taixu_verified()).expect("restore engine");
    assert_eq!(restored.state_hash(), e.state_hash());
    restored
}
