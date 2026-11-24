use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

use content_native_loader::{
    prepare_locked_content_with_bridge, restore_locked_engine, start_locked_engine,
    validate_and_freeze_raw_package, LockedStartMode, VerifiedLockedContent,
};
use engine_native::ai::BoundedPlanner;
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardDef, Suit};
use engine_native::ids::{CardId, PlayerId};
use engine_native::state::Role;
use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase,
};

static TAIXU_E2_VERIFIED: OnceLock<VerifiedLockedContent> = OnceLock::new();
static E2_DEDUP: AtomicU64 = AtomicU64::new(82_000_000);

pub fn next_dedup() -> u64 {
    E2_DEDUP.fetch_add(1, Ordering::Relaxed)
}

pub fn taixu_verified() -> &'static VerifiedLockedContent {
    TAIXU_E2_VERIFIED.get_or_init(|| {
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
    assert_eq!(
        ch.observer, actor,
        "choice observer mismatch for {}",
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

pub fn expect_prompt<'a>(e: &'a Engine, prompt: &str) -> (u64, PlayerId, Vec<ChoiceOption>) {
    let ch = e
        .open_choice()
        .unwrap_or_else(|| panic!("expected open choice {prompt}, got None"));
    assert_eq!(
        ch.spec.prompt, prompt,
        "expected prompt {prompt}, got {} (observer {})",
        ch.spec.prompt, ch.observer
    );
    (ch.id, ch.observer, ch.spec.options.clone())
}

pub fn activate_skill(e: &mut Engine, actor: PlayerId, name: &str) -> CommandOutcome {
    let skill = e
        .catalog()
        .skill_by_name(name)
        .unwrap_or_else(|| panic!("skill {name} missing from catalog"));
    submit(e, actor, GameAction::ActivateSkill { skill })
}

pub fn play_card(
    e: &mut Engine,
    actor: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
) -> CommandOutcome {
    submit(e, actor, GameAction::PlayCard { card, targets })
}

pub fn drain_open_with_ai(e: &mut Engine, max_steps: usize) {
    let ai = BoundedPlanner::new(16);
    for _ in 0..max_steps {
        if e.outcome().is_some() {
            break;
        }
        let Some(ch) = e.open_choice() else {
            break;
        };
        let obs = ch.observer;
        let prompt = ch.spec.prompt;
        let act = e.plan_action(obs, &ai).action;
        let out = submit(e, obs, act);
        assert!(
            !matches!(out, CommandOutcome::Rejected(_)),
            "AI rejected on prompt {prompt}: {out:?}"
        );
    }
}

pub fn advance_to_play(e: &mut Engine, seat: PlayerId) {
    let ai = BoundedPlanner::new(16);
    {
        let st = e.state_core();
        if e.open_choice().is_none() && st.turn.current == seat && st.turn.phase == Phase::Play {
            let _ = submit(e, seat, GameAction::NextPhase);
        }
    }
    for _ in 0..160 {
        if e.outcome().is_some() {
            break;
        }
        let st = e.state_core();
        if st.turn.current == seat && st.turn.phase == Phase::Play {
            return;
        }
        if let Some(ch) = e.open_choice() {
            let obs = ch.observer;
            let act = e.plan_action(obs, &ai).action;
            let _ = submit(e, obs, act);
            continue;
        }
        if st.turn.current == seat {
            let _ = submit(e, seat, GameAction::NextPhase);
        } else if st.turn.phase == Phase::Play {
            let _ = submit(e, st.turn.current, GameAction::NextPhase);
        } else {
            let act = e.plan_action(st.turn.current, &ai).action;
            let _ = submit(e, st.turn.current, act);
        }
    }
    assert_eq!(e.state_core().turn.current, seat);
    assert_eq!(e.state_core().turn.phase, Phase::Play);
}

pub fn start_4p_taixu(seed: u64) -> Engine {
    let verified = taixu_verified();
    let mut e = start_locked_engine(seed, 4, LockedStartMode::Identity, verified)
        .expect("start 4p identity");
    e.debug_set_identity_roles(vec![Role::Lord, Role::Rebel, Role::Traitor, Role::Loyalist]);
    advance_to_play(&mut e, 0);
    for s in 0..4u8 {
        e.debug_set_max_hp(s, 5);
        e.debug_set_hp(s, 4);
        e.debug_clear_hand(s);
    }
    e.debug_sweep_judge_zones();
    e
}

pub fn place_matching(e: &mut Engine, seat: PlayerId, pred: impl Fn(&CardDef) -> bool) -> CardId {
    let c = e.debug_set_deck_top_by(pred);
    let popped = e.state_core_mut().deck.pop().expect("top deck");
    assert_eq!(popped, c);
    e.state_core_mut().player_mut(seat).hand.push(c);
    c
}

pub fn place_suit_rank(e: &mut Engine, seat: PlayerId, suit: Suit, rank: u8) -> CardId {
    let c = e.debug_set_deck_top_by(|d| !d.off_deck && d.suit == suit && d.rank == rank);
    let popped = e.state_core_mut().deck.pop().unwrap();
    assert_eq!(popped, c);
    e.state_core_mut().player_mut(seat).hand.push(c);
    c
}

pub fn roundtrip_archive(e: &Engine, verified: &VerifiedLockedContent) -> Engine {
    let bytes = encode_archive(&e.save());
    let decoded = decode_archive(&bytes).expect("decode NGSA v107");
    restore_locked_engine(&decoded, verified).expect("restore locked engine")
}
