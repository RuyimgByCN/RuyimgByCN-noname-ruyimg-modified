use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use engine_native::archive::{decode_archive, encode_archive};
use engine_native::content::extension::ExtensionPack;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};

static Q4_DEDUP: AtomicU64 = AtomicU64::new(90_000_000);

pub fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: Q4_DEDUP.fetch_add(1, Ordering::Relaxed),
        action,
    })
}

pub fn repo_asset(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("failed to read {path:?}: {err}"))
}

pub fn drain_one_default(e: &mut Engine) {
    let Some(c) = e.open_choice().cloned() else {
        return;
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

pub fn drain_default(e: &mut Engine) {
    for _ in 0..64 {
        if e.open_choice().is_none() {
            break;
        }
        drain_one_default(e);
    }
}

pub fn advance_phases(e: &mut Engine, steps: usize) {
    for _ in 0..steps {
        drain_default(e);
        if e.outcome().is_some() {
            break;
        }
        let cur = e.turn().current;
        let _ = submit(e, cur, GameAction::NextPhase);
        drain_default(e);
    }
}

pub fn step_to_phase(e: &mut Engine, actor: u8, target: Phase) {
    for _ in 0..32 {
        drain_default(e);
        if e.turn().current == actor && e.turn().phase == target {
            return;
        }
        let cur = e.turn().current;
        let _ = submit(e, cur, GameAction::NextPhase);
    }
    panic!(
        "failed to reach actor {actor} phase {target:?}, stuck at {} {:?}",
        e.turn().current,
        e.turn().phase
    );
}

pub fn assert_accepted_or_pending(res: &CommandOutcome) {
    assert!(
        matches!(
            res,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "expected Accepted/Pending, got {res:?}"
    );
}

pub fn answer_choice(e: &mut Engine, selection: Vec<ChoiceOption>) {
    let c = e.open_choice().expect("expected open choice").clone();
    let res = submit(
        e,
        c.observer,
        GameAction::Answer {
            choice_id: c.id,
            selection,
        },
    );
    assert_accepted_or_pending(&res);
}

pub fn assert_archive_roundtrip(e: &Engine, packs: &[&ExtensionPack]) -> Engine {
    let snap = e.save();
    let bytes = encode_archive(&snap);
    let decoded = decode_archive(&bytes).expect("decode archive");
    let restored = Engine::restore_with_extensions(&decoded, packs).expect("restore archive");
    assert_eq!(restored.state_hash(), e.state_hash(), "state hash parity after save/restore");
    restored
}

pub fn remove_card_from_all_zones(e: &mut Engine, card: u32) {
    let core = e.state_core_mut();
    core.deck.retain(|&c| c != card);
    core.discard.retain(|&c| c != card);
    for p in &mut core.players {
        p.hand.retain(|&c| c != card);
        p.equips.retain(|&c| c != card);
        p.judge.retain(|&c| c != card);
    }
    for pile in core.vm_aux.piles.values_mut() {
        pile.retain(|&c| c != card);
    }
}

pub fn move_card_to_hand(e: &mut Engine, seat: u8, card: u32) {
    remove_card_from_all_zones(e, card);
    e.state_core_mut().player_mut(seat).hand.push(card);
}

pub fn move_card_to_pile(e: &mut Engine, seat: u8, pile_key: &str, card: u32) {
    remove_card_from_all_zones(e, card);
    e.state_core_mut()
        .vm_aux
        .piles
        .entry((seat, pile_key.to_string()))
        .or_default()
        .push(card);
}

pub fn move_card_to_deck_top(e: &mut Engine, card: u32) {
    remove_card_from_all_zones(e, card);
    e.state_core_mut().deck.push(card);
}

pub fn clear_equips_to_discard(e: &mut Engine, seat: u8) {
    let equips = std::mem::take(&mut e.state_core_mut().player_mut(seat).equips);
    e.state_core_mut().discard.extend(equips);
}
