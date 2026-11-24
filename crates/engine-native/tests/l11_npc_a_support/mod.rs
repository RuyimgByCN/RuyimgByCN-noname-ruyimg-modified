//! Shared real-match helpers for P3-C1a `L11` Cross NPC Part A test suites (`0` `debug_execute_effect`/`debug_resolve`).

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

use content_native_loader::{
    prepare_locked_content_with_bridge, start_locked_engine, validate_and_freeze_raw_package,
    LockedStartMode, VerifiedLockedContent,
};
use engine_native::catalog::Suit;
use engine_native::ids::{CardId, PlayerId};
use engine_native::state::Role;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};

static TAIXU_NPC_A_VERIFIED: OnceLock<VerifiedLockedContent> = OnceLock::new();
static NPC_A_DEDUP: AtomicU64 = AtomicU64::new(91_000_000);

pub fn next_dedup() -> u64 {
    NPC_A_DEDUP.fetch_add(1, Ordering::Relaxed)
}

pub fn taixu_verified() -> &'static VerifiedLockedContent {
    TAIXU_NPC_A_VERIFIED.get_or_init(|| {
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

pub fn new_4p(seed: u64) -> Engine {
    let mut e = start_locked_engine(seed, 4, LockedStartMode::Identity, taixu_verified())
        .expect("start 4p identity");
    e.debug_set_identity_roles(vec![Role::Lord, Role::Rebel, Role::Traitor, Role::Loyalist]);
    for s in 0..4u8 {
        e.debug_set_max_hp(s, 6);
        e.debug_set_hp(s, 6);
        e.debug_set_hujia(s, 0);
    }
    e
}

pub fn new_2p(seed: u64) -> Engine {
    let mut e = start_locked_engine(seed, 2, LockedStartMode::Prototype, taixu_verified())
        .expect("start 2p prototype");
    for s in 0..2u8 {
        e.debug_set_max_hp(s, 6);
        e.debug_set_hp(s, 6);
        e.debug_set_hujia(s, 0);
    }
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

pub fn answer_checked(e: &mut Engine, actor: PlayerId, sel: Vec<ChoiceOption>) {
    let ch = e.open_choice().expect("open choice expected");
    assert_eq!(
        ch.observer, actor,
        "observer mismatch on `{}`",
        ch.spec.prompt
    );
    let prompt = ch.spec.prompt;
    let opts = ch.spec.options.clone();
    let out = submit(
        e,
        actor,
        GameAction::Answer {
            choice_id: ch.id,
            selection: sel.clone(),
        },
    );
    assert!(
        !matches!(out, CommandOutcome::Rejected(_)),
        "answer rejected on `{prompt}` (sel={sel:?}, opts={opts:?}): {out:?}"
    );
}

pub fn answer_prompt(
    e: &mut Engine,
    expected_prompt: &str,
    actor: PlayerId,
    sel: Vec<ChoiceOption>,
) {
    let ch = e.open_choice().expect("expected open choice");
    assert_eq!(
        ch.spec.prompt, expected_prompt,
        "prompt mismatch: got `{}`, expected `{expected_prompt}` (opts={:?})",
        ch.spec.prompt, ch.spec.options
    );
    answer_checked(e, actor, sel);
}

pub fn activate_skill(e: &mut Engine, actor: PlayerId, skill_name: &str) {
    let def = e
        .catalog()
        .skill_by_name(skill_name)
        .unwrap_or_else(|| panic!("skill `{skill_name}` missing"));
    let out = submit(e, actor, GameAction::ActivateSkill { skill: def });
    assert!(
        !matches!(out, CommandOutcome::Rejected(_)),
        "activate `{skill_name}` rejected: {out:?}"
    );
}

pub fn drain_pass(e: &mut Engine) {
    for _ in 0..48 {
        if e.outcome().is_some() {
            return;
        }
        let Some(ch) = e.open_choice().cloned() else {
            return;
        };
        let sel = if ch.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if ch.spec.allow_cancel {
            Vec::new()
        } else {
            ch.spec.options.iter().take(ch.spec.min.max(1)).copied().collect()
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

pub fn goto_play(e: &mut Engine, seat: PlayerId) {
    for _ in 0..48 {
        if e.state_core().turn.current == seat
            && e.state_core().turn.phase == Phase::Play
            && e.open_choice().is_none()
        {
            return;
        }
        if e.open_choice().is_some() {
            drain_pass(e);
            continue;
        }
        let cur = e.state_core().turn.current;
        let _ = submit(e, cur, GameAction::NextPhase);
    }
    assert_eq!(e.state_core().turn.current, seat);
    assert_eq!(e.state_core().turn.phase, Phase::Play);
    assert!(e.open_choice().is_none(), "unhandled choice at goto_play");
}

pub fn flag(e: &Engine, p: PlayerId, key: &str) -> u32 {
    e.state_core()
        .vm_aux
        .flags
        .get(&(p, key.to_string()))
        .copied()
        .unwrap_or(0)
}

pub fn pile(e: &Engine, p: PlayerId, key: &str) -> Vec<CardId> {
    e.state_core()
        .vm_aux
        .piles
        .get(&(p, key.to_string()))
        .cloned()
        .unwrap_or_default()
}

pub fn place_with_rank(e: &mut Engine, seat: PlayerId, min_rank: u8, max_rank: u8) -> CardId {
    for (idx, c) in e.catalog().cards.iter().enumerate() {
        let cid = idx as CardId;
        if (min_rank..=max_rank).contains(&c.rank)
            && !e.state_core().players.iter().any(|pl| pl.hand.contains(&cid) || pl.equips.contains(&cid) || pl.judge.contains(&cid))
            && (e.state_core().deck.contains(&cid) || e.state_core().discard.contains(&cid))
        {
            e.state_core_mut().deck.retain(|&x| x != cid);
            e.state_core_mut().discard.retain(|&x| x != cid);
            e.state_core_mut().player_mut(seat).hand.push(cid);
            return cid;
        }
    }
    e.debug_place(seat, "sha")
}

pub fn place_by_suit_pred(
    e: &mut Engine,
    seat: PlayerId,
    suit: Suit,
    want_name: Option<&str>,
) -> CardId {
    for (idx, c) in e.catalog().cards.iter().enumerate() {
        let cid = idx as CardId;
        if c.suit == suit
            && want_name.is_none_or(|n| c.name == n)
            && (e.state_core().deck.contains(&cid) || e.state_core().discard.contains(&cid))
        {
            e.state_core_mut().deck.retain(|&x| x != cid);
            e.state_core_mut().discard.retain(|&x| x != cid);
            e.state_core_mut().player_mut(seat).hand.push(cid);
            return cid;
        }
    }
    e.debug_place_suit(seat, suit)
}

pub fn place_named_suit(
    e: &mut Engine,
    seat: PlayerId,
    name: &str,
    suit: Option<Suit>,
) -> CardId {
    let cid = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find_map(|(i, c)| {
            (c.name == name
                && suit.is_none_or(|s| c.suit == s)
                && (e.state_core().deck.contains(&(i as u32))
                    || e.state_core().discard.contains(&(i as u32))))
            .then_some(i as u32)
        })
        .unwrap_or_else(|| panic!("named card `{name}` (suit={suit:?}) missing from deck/discard"));
    e.state_core_mut().deck.retain(|&x| x != cid);
    e.state_core_mut().discard.retain(|&x| x != cid);
    e.state_core_mut().player_mut(seat).hand.push(cid);
    cid
}

pub fn equip_card_now(e: &mut Engine, seat: PlayerId, name: &str) -> CardId {
    let cid = e.debug_place(seat, name);
    let out = submit(
        e,
        seat,
        GameAction::PlayCard {
            card: cid,
            targets: vec![],
        },
    );
    assert!(
        !matches!(out, CommandOutcome::Rejected(_)),
        "equip `{name}` rejected: {out:?}"
    );
    drain_pass(e);
    cid
}
