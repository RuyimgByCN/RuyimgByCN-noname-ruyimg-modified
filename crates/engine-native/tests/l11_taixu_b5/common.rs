#![allow(dead_code)]

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

use content_native_loader::{
    prepare_locked_content_with_bridge, restore_locked_engine, start_locked_engine,
    validate_and_freeze_raw_package, LockedStartMode, VerifiedLockedContent,
};
use engine_native::ai::BoundedPlanner;
use engine_native::catalog::{CardKind, Suit};
use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, GameMode, OpenChoice, Phase,
};

static TAIXU_B5_VERIFIED: OnceLock<VerifiedLockedContent> = OnceLock::new();
static TAIXU_L27_B5_VERIFIED: OnceLock<VerifiedLockedContent> = OnceLock::new();
static TAIXU_U08_B5_VERIFIED: OnceLock<VerifiedLockedContent> = OnceLock::new();
static TAIXU_L28_B5_VERIFIED: OnceLock<VerifiedLockedContent> = OnceLock::new();
static TAIXU_XJ_B5_VERIFIED: OnceLock<VerifiedLockedContent> = OnceLock::new();
static B5_DEDUP: AtomicU64 = AtomicU64::new(55_000_000);

pub fn next_dedup() -> u64 {
    B5_DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn load_frozen_extension(rel_dir: &str) -> content_native_loader::FrozenPackage {
    let pkg_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel_dir);
    let raw_json = std::fs::read_to_string(pkg_dir.join("package.npack.json"))
        .unwrap_or_else(|e| panic!("read {rel_dir}/package.npack.json: {e}"))
        .replace("\r\n", "\n");
    let parsed: serde_json::Value = serde_json::from_str(&raw_json).unwrap();
    let mut assets = BTreeMap::new();
    if let Some(paths) = parsed.get("asset_paths").and_then(|v| v.as_object()) {
        for rel_path in paths.values().filter_map(|v| v.as_str()) {
            let bytes = std::fs::read(pkg_dir.join(rel_path))
                .unwrap_or_else(|e| panic!("read {rel_dir}/{rel_path}: {e}"))
                .into_iter()
                .filter(|&b| b != b'\r')
                .collect::<Vec<u8>>();
            assets.insert(rel_path.to_string(), bytes);
        }
    }
    validate_and_freeze_raw_package(raw_json, assets).expect("freeze extension")
}

pub fn taixu_verified() -> &'static VerifiedLockedContent {
    TAIXU_B5_VERIFIED.get_or_init(|| {
        let frozen = load_frozen_extension("../../packages/extensions/taixu-upstream-liuli");
        prepare_locked_content_with_bridge(vec![frozen]).expect("prepare locked taixu")
    })
}

pub fn taixu_with_l27_verified() -> &'static VerifiedLockedContent {
    TAIXU_L27_B5_VERIFIED.get_or_init(|| {
        let taixu = load_frozen_extension("../../packages/extensions/taixu-upstream-liuli");
        let l27 = load_frozen_extension("../../packages/extensions/select-beautify-upstream-liuli");
        prepare_locked_content_with_bridge(vec![taixu, l27]).expect("prepare locked taixu + l27")
    })
}

pub fn taixu_with_u08_verified() -> &'static VerifiedLockedContent {
    TAIXU_U08_B5_VERIFIED.get_or_init(|| {
        let taixu = load_frozen_extension("../../packages/extensions/taixu-upstream-liuli");
        let u08 = load_frozen_extension("../../packages/extensions/hero-kill-upstream-noname");
        prepare_locked_content_with_bridge(vec![taixu, u08]).expect("prepare locked taixu + u08")
    })
}

pub fn taixu_with_l28_verified() -> &'static VerifiedLockedContent {
    TAIXU_L28_B5_VERIFIED.get_or_init(|| {
        let thunder = load_frozen_extension("../../packages/extensions/thunder-upstream-liuli");
        let taixu = load_frozen_extension("../../packages/extensions/taixu-upstream-liuli");
        prepare_locked_content_with_bridge(vec![thunder, taixu]).expect("prepare locked l28 + taixu")
    })
}

pub fn taixu_with_xianjian_verified() -> &'static VerifiedLockedContent {
    TAIXU_XJ_B5_VERIFIED.get_or_init(|| {
        let taixu = load_frozen_extension("../../packages/extensions/taixu-upstream-liuli");
        let xianjian = load_frozen_extension("../../packages/extensions/shaai-xianjian-upstream-noname");
        prepare_locked_content_with_bridge(vec![taixu, xianjian])
            .expect("prepare locked taixu + xianjian")
    })
}

pub fn start_versus_with(
    seed: u64,
    teams: &[&[u8]],
    verified: &'static VerifiedLockedContent,
) -> Engine {
    let total: u8 = teams
        .iter()
        .flat_map(|t| t.iter().copied())
        .max()
        .map_or(2, |m| m + 1);
    let mut e = start_locked_engine(seed, total as usize, LockedStartMode::Prototype, verified)
        .expect("start locked taixu engine");
    for s in 0..total {
        e.debug_set_max_hp(s, 6);
        e.debug_set_hp(s, 6);
        e.debug_clear_hand(s);
    }
    let team_vecs: Vec<Vec<u8>> = teams.iter().map(|t| t.to_vec()).collect();
    let subs = vec![Vec::new(); team_vecs.len()];
    e.state_core_mut().mode = GameMode::Versus {
        teams: team_vecs,
        substitutes: subs,
    };
    e
}

pub fn start_versus(seed: u64, teams: &[&[u8]]) -> Engine {
    start_versus_with(seed, teams, taixu_verified())
}

pub fn start_versus_with_l27(seed: u64, teams: &[&[u8]]) -> Engine {
    start_versus_with(seed, teams, taixu_with_l27_verified())
}

pub fn start_versus_with_u08(seed: u64, teams: &[&[u8]]) -> Engine {
    start_versus_with(seed, teams, taixu_with_u08_verified())
}

pub fn start_versus_with_l28(seed: u64, teams: &[&[u8]]) -> Engine {
    start_versus_with(seed, teams, taixu_with_l28_verified())
}

pub fn start_versus_with_xianjian(seed: u64, teams: &[&[u8]]) -> Engine {
    start_versus_with(seed, teams, taixu_with_xianjian_verified())
}

pub fn equip_std_kind(e: &mut Engine, actor: u8, kind: CardKind) -> u32 {
    let prev = (e.turn().current, e.turn().phase, e.turn().round);
    if prev.0 != actor || prev.1 != Phase::Play {
        e.debug_set_turn(actor, Phase::Play, prev.2);
    }
    let cid = place_std_kind(e, actor, kind);
    let out = play_card(e, actor, cid, Vec::new());
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "equip std kind {kind:?} rejected: {out:?}"
    );
    drain(e);
    if e.turn().current != prev.0 || e.turn().phase != prev.1 || e.turn().round != prev.2 {
        e.debug_set_turn(prev.0, prev.1, prev.2);
    }
    cid
}

pub fn grant_skill(e: &mut Engine, player: u8, name: &str) {
    assert!(
        e.catalog().skill_by_name(name).is_some(),
        "skill {name} missing in catalog"
    );
    e.debug_grant(player, name);
}

pub fn has_skill(e: &Engine, player: u8, name: &str) -> bool {
    e.catalog()
        .skill_by_name(name)
        .is_some_and(|def| e.state_core().has_skill(player, def))
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

pub fn activate_skill(e: &mut Engine, actor: u8, name: &str) -> CommandOutcome {
    let skill = e
        .catalog()
        .skill_by_name(name)
        .unwrap_or_else(|| panic!("skill {name} missing"));
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
        .unwrap_or_else(|| panic!("expected open_choice '{prompt}', got None"));
    assert_eq!(c.spec.prompt, prompt, "unexpected open choice prompt");
    c.clone()
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

pub fn remove_from_piles_to_hand(e: &mut Engine, player: u8, cid: u32) {
    let core = e.state_core_mut();
    core.deck.retain(|&c| c != cid);
    core.discard.retain(|&c| c != cid);
    core.detached_cards.retain(|&(c, _)| c != cid);
    core.player_mut(player).hand.push(cid);
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
    remove_from_piles_to_hand(e, player, cid);
    cid
}

pub fn place_std_kind(e: &mut Engine, player: u8, kind: CardKind) -> u32 {
    let cid = e
        .state_core()
        .deck
        .iter()
        .chain(e.state_core().discard.iter())
        .copied()
        .find(|&c| {
            let cd = &e.catalog().cards[c as usize];
            cd.kind == kind && !cd.name.starts_with("txhj_")
        })
        .unwrap_or_else(|| panic!("no standard card ({kind:?}) in deck/discard"));
    remove_from_piles_to_hand(e, player, cid);
    cid
}

pub fn place_kind_suit_rank(
    e: &mut Engine,
    player: u8,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> u32 {
    let cid = e
        .state_core()
        .deck
        .iter()
        .chain(e.state_core().discard.iter())
        .copied()
        .find(|&c| {
            let cd = &e.catalog().cards[c as usize];
            cd.kind == kind && cd.suit == suit && cd.rank == rank
        })
        .unwrap_or_else(|| {
            let any = e
                .state_core()
                .deck
                .iter()
                .chain(e.state_core().discard.iter())
                .copied()
                .find(|&c| {
                    let cd = &e.catalog().cards[c as usize];
                    cd.kind == kind && cd.rank == rank
                })
                .unwrap_or_else(|| panic!("no card ({kind:?}, rank {rank}) in deck/discard"));
            any
        });
    remove_from_piles_to_hand(e, player, cid);
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
    remove_from_piles_to_hand(e, player, cid);
    cid
}

pub fn place_named(e: &mut Engine, player: u8, name: &str) -> u32 {
    let cid = e
        .state_core()
        .deck
        .iter()
        .chain(e.state_core().discard.iter())
        .copied()
        .find(|&c| e.catalog().cards[c as usize].name == name)
        .unwrap_or_else(|| panic!("no named card {name} in deck/discard"));
    remove_from_piles_to_hand(e, player, cid);
    cid
}

pub fn equip_named(e: &mut Engine, actor: u8, name: &str) -> u32 {
    let prev = (e.turn().current, e.turn().phase, e.turn().round);
    if prev.0 != actor || prev.1 != Phase::Play {
        e.debug_set_turn(actor, Phase::Play, prev.2);
    }
    let cid = place_named(e, actor, name);
    let out = play_card(e, actor, cid, Vec::new());
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "equip named {name} rejected: {out:?}"
    );
    drain(e);
    if e.turn().current != prev.0 || e.turn().phase != prev.1 || e.turn().round != prev.2 {
        e.debug_set_turn(prev.0, prev.1, prev.2);
    }
    cid
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

pub fn top_deck_suit(e: &mut Engine, suit: Suit) -> u32 {
    e.debug_set_deck_top_by(|cd| cd.suit == suit)
}

pub fn top_deck_suit_rank(e: &mut Engine, suit: Suit, rank: u8) -> u32 {
    e.debug_set_deck_top_by(|cd| cd.suit == suit && cd.rank == rank)
}

pub fn advance_until_turn_phase(e: &mut Engine, actor: u8, phase: Phase) {
    for _ in 0..40 {
        if e.turn().current == actor && e.turn().phase == phase {
            return;
        }
        drain(e);
        if e.turn().current == actor && e.turn().phase == phase {
            return;
        }
        let _ = next_phase(e);
    }
    panic!(
        "failed to reach actor {actor} phase {phase:?}, stuck at {:?}",
        e.turn()
    );
}

pub fn save_restore_and_compare_pending_with(
    e: &Engine,
    c: &OpenChoice,
    sel: Vec<ChoiceOption>,
    verified: &'static VerifiedLockedContent,
) -> Engine {
    let snap = e.save();
    let mut live = e.clone();
    let mut restored = restore_locked_engine(&snap, verified).expect("restore pending");
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

pub fn save_restore_and_compare_pending(
    e: &Engine,
    c: &OpenChoice,
    sel: Vec<ChoiceOption>,
) -> Engine {
    save_restore_and_compare_pending_with(e, c, sel, taixu_verified())
}

pub fn ai_plan(e: &Engine, seat: u8) -> GameAction {
    e.plan_action(seat, &BoundedPlanner::new(8)).action
}

pub fn flag(e: &Engine, p: u8, key: &str) -> u32 {
    e.state_core()
        .vm_aux
        .flags
        .get(&(p, key.to_string()))
        .copied()
        .unwrap_or(0)
}
