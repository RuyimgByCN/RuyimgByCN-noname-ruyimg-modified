use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

use content_native_loader::{
    prepare_locked_content_with_bridge, restore_locked_engine, start_locked_engine,
    validate_and_freeze_raw_package, LockedStartMode, VerifiedLockedContent,
};
use engine_native::ai::BoundedPlanner;
use engine_native::catalog::{CardKind, Faction, Suit};
use engine_native::choice::OpenChoice;
use engine_native::state::GameMode;
use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, LogEvent, Phase,
};

static DEDUP: AtomicU64 = AtomicU64::new(22_000_000);
static TAIXU_VERIFIED: OnceLock<VerifiedLockedContent> = OnceLock::new();

pub fn taixu_verified() -> &'static VerifiedLockedContent {
    TAIXU_VERIFIED.get_or_init(|| {
        let pkg_dir =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions/taixu-upstream-liuli");
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

pub fn start_taixu(seed: u64, players: u8) -> Engine {
    let v = taixu_verified();
    let mut e = start_locked_engine(seed, players as usize, LockedStartMode::Prototype, v)
        .expect("start taixu engine");
    for s in 0..players {
        e.state_core_mut().player_mut(s).max_hp = 6;
        e.state_core_mut().player_mut(s).hp = 6;
        e.debug_clear_hand(s);
    }
    e.debug_set_turn(0, Phase::Play, 1);
    e
}

pub fn set_team_factions(e: &mut Engine, teams: &[&[u8]], factions: &[Faction]) {
    assert_eq!(teams.len(), factions.len());
    let total: usize = teams.iter().map(|t| t.len()).sum();
    assert_eq!(total, e.state_core().players.len());
    let mut vec_teams = Vec::new();
    let mut subs = Vec::new();
    for (ti, &seats) in teams.iter().enumerate() {
        vec_teams.push(seats.to_vec());
        subs.push(Vec::new());
        let fac = factions[ti];
        let gdef = (0..e.catalog().general_count() as u16)
            .find(|&idx| e.catalog().general_meta(idx).is_some_and(|m| m.1 == fac))
            .unwrap_or_else(|| panic!("general for faction {fac:?}"));
        for &s in seats {
            e.state_core_mut().player_mut(s).general_slots[0] = Some(gdef);
        }
    }
    e.state_core_mut().mode = GameMode::Versus {
        teams: vec_teams,
        substitutes: subs,
    };
}

pub fn start_versus(seed: u64, teams: &[&[u8]], factions: &[Faction]) -> Engine {
    let n: u8 = teams.iter().map(|t| t.len() as u8).sum();
    let mut e = start_taixu(seed, n);
    set_team_factions(&mut e, teams, factions);
    e
}

pub fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action,
    })
}

pub fn play_card(e: &mut Engine, actor: u8, card: u32, targets: Vec<u8>) -> CommandOutcome {
    submit(e, actor, GameAction::PlayCard { card, targets })
}

pub fn equip_named(e: &mut Engine, actor: u8, name: &str) -> u32 {
    let prev = (e.turn().current, e.turn().phase, e.turn().round);
    if prev.0 != actor || prev.1 != Phase::Play {
        e.debug_set_turn(actor, Phase::Play, prev.2);
    }
    let cid = e.debug_move_named_card_to_hand(actor, name);
    let out = play_card(e, actor, cid, Vec::new());
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "equip {name} rejected: {out:?}"
    );
    drain(e);
    if e.turn().current != prev.0 || e.turn().phase != prev.1 || e.turn().round != prev.2 {
        e.debug_set_turn(prev.0, prev.1, prev.2);
    }
    cid
}

pub fn activate_skill(e: &mut Engine, actor: u8, name: &str) -> CommandOutcome {
    let skill = e
        .catalog()
        .skill_by_name(name)
        .unwrap_or_else(|| panic!("missing skill {name}"));
    submit(e, actor, GameAction::ActivateSkill { skill })
}

pub fn next_phase(e: &mut Engine) -> CommandOutcome {
    let actor = e.turn().current;
    submit(e, actor, GameAction::NextPhase)
}

pub fn answer(e: &mut Engine, c: &OpenChoice, selection: Vec<ChoiceOption>) -> CommandOutcome {
    submit(
        e,
        c.observer,
        GameAction::Answer {
            choice_id: c.id,
            selection,
        },
    )
}

pub fn expect_pending(e: &Engine, prompt: &str) -> OpenChoice {
    let c = e
        .open_choice()
        .unwrap_or_else(|| panic!("expected pending choice {prompt}, got None"));
    assert_eq!(c.spec.prompt, prompt, "unexpected choice prompt");
    c.clone()
}

pub fn answer_pass_until(e: &mut Engine, prompt: &str) -> OpenChoice {
    for _ in 0..24 {
        let c = e
            .open_choice()
            .unwrap_or_else(|| panic!("choice ended before prompt {prompt}"))
            .clone();
        if c.spec.prompt == prompt {
            return c;
        }
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
        let _ = answer(e, &c, sel);
    }
    panic!("never reached prompt {prompt}");
}

pub fn drain(e: &mut Engine) {
    for _ in 0..64 {
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
        let _ = answer(e, &c, sel);
    }
}

pub fn equip_kind(e: &mut Engine, actor: u8, kind: CardKind) -> u32 {
    let prev = (e.turn().current, e.turn().phase, e.turn().round);
    if prev.0 != actor || prev.1 != Phase::Play {
        e.debug_set_turn(actor, Phase::Play, prev.2);
    }
    let cid = place_kind(e, actor, kind);
    let out = play_card(e, actor, cid, Vec::new());
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "equip kind {kind:?} rejected: {out:?}"
    );
    drain(e);
    if e.turn().current != prev.0 || e.turn().phase != prev.1 || e.turn().round != prev.2 {
        e.debug_set_turn(prev.0, prev.1, prev.2);
    }
    cid
}

pub fn place_kind(e: &mut Engine, player: u8, kind: CardKind) -> u32 {
    let cid = e
        .state_core()
        .deck
        .iter()
        .chain(e.state_core().discard.iter())
        .copied()
        .find(|&c| e.catalog().cards[c as usize].kind == kind)
        .unwrap_or_else(|| panic!("no card ({kind:?}) in deck/discard"));
    if let Some(p) = e.state_core().deck.iter().position(|&c| c == cid) {
        e.state_core_mut().deck.remove(p);
    } else if let Some(p) = e.state_core().discard.iter().position(|&c| c == cid) {
        e.state_core_mut().discard.remove(p);
    }
    e.state_core_mut().player_mut(player).hand.push(cid);
    cid
}

pub fn place_kind_suit(e: &mut Engine, player: u8, kind: CardKind, suit: Suit) -> u32 {
    let cid = e
        .state_core()
        .deck
        .iter()
        .chain(e.state_core().discard.iter())
        .copied()
        .find(|&c| {
            let cd = &e.catalog().cards[c as usize];
            cd.kind == kind && cd.suit == suit
        })
        .unwrap_or_else(|| panic!("no card ({kind:?}, {suit:?}) in deck/discard"));
    let pos_deck = e.state_core().deck.iter().position(|&c| c == cid);
    if let Some(p) = pos_deck {
        e.state_core_mut().deck.remove(p);
    } else if let Some(p) = e.state_core().discard.iter().position(|&c| c == cid) {
        e.state_core_mut().discard.remove(p);
    }
    e.state_core_mut().player_mut(player).hand.push(cid);
    cid
}

pub fn top_deck_suit(e: &mut Engine, suit: Suit) -> u32 {
    e.debug_set_deck_top_by(|cd| cd.suit == suit)
}

pub fn save_restore_and_compare_pending(e: &Engine, c: &OpenChoice, sel: Vec<ChoiceOption>) -> Engine {
    let snap = e.save();
    let mut live = e.clone();
    let mut restored = restore_locked_engine(&snap, taixu_verified()).expect("restore pending");
    assert_eq!(live.state_hash(), restored.state_hash());
    let _ = answer(&mut live, c, sel.clone());
    let _ = answer(&mut restored, c, sel);
    drain(&mut live);
    drain(&mut restored);
    assert_eq!(
        live.state_hash(),
        restored.state_hash(),
        "state hash must match after restore -> Answer on {}",
        c.spec.prompt
    );
    live
}

pub fn ai_plan(e: &Engine, seat: u8) -> GameAction {
    e.plan_action(seat, &BoundedPlanner::new(8)).action
}

pub fn log_has_reveal(log: &[LogEvent], to_seat: u8) -> bool {
    log.iter().any(|ev| match ev {
        LogEvent::CardsRevealed { to, .. } => to.is_empty() || to.contains(&to_seat),
        _ => false,
    })
}

pub fn flag(e: &Engine, p: u8, key: &str) -> u32 {
    e.state_core()
        .vm_aux
        .flags
        .get(&(p, key.to_string()))
        .copied()
        .unwrap_or(0)
}
