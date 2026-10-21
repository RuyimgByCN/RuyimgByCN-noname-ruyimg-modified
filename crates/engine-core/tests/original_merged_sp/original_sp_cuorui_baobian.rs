//! Golden for sp 崔锐 (cuorui) + 豹变 (baobian) rewrites.
//!
//! cuorui: locked — draws min(5, maxHp) at game start; once per game, skips the
//! judge phase when the judge area is non-empty.
//! baobian: locked — treats HP ≤3/≤2/≤1 as having 挑衅/咆哮/神速 respectively
//! (approximated by re-syncing the granted skills at each prepare phase).

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, MarkDefId, PlayerId, SkillDefId, Suit, TurnPhaseTag, ZoneId,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Zone, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const CUORUI: u32 = 4_002_191_084;
const BAOBIAN: u32 = 3_727_333_388;
const OL_TIAOXIN: u32 = 3_936_161_008;
const OL_PAOXIAO: u32 = 2_829_854_661;
const XIN_SHENSU: u32 = 2_561_255_694;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");
const REFRESH: &[u8] =
    include_bytes!("../../../../content/original/ported/refresh/refresh.npack.json");
const SHENHUA: &[u8] =
    include_bytes!("../../../../content/original/ported/shenhua/shenhua.npack.json");

fn start_with(packages: &[&[u8]]) -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    for bytes in packages {
        let pkg = content_schema::ContentPackage::from_json_bytes(bytes).unwrap();
        engine.load_package(&pkg).unwrap();
    }
    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_261_101,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(
        matches!(result, CommandResult::Accepted { .. }),
        "start must be accepted: {result:?}"
    );
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine, owner, other)
}

fn strip_and_add(tx: &mut Transaction, owner: PlayerId, def: u32) {
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(tx, effects).unwrap();
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, kind: CardKind) -> engine_api::CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_100 + kind.tag() as u32),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn inject_judge_zone(tx: &mut Transaction, player: PlayerId) -> CardId {
    // Reuse the canonical Judge zone (created by the rules bootstrap); only
    // allocate a fresh one when the game has not created it yet.
    let zone_id = match tx.state().zones.find(ZoneKind::Judge, player) {
        Some(existing) => existing,
        None => tx.state_mut().zones.alloc(Zone {
            id: ZoneId::NONE,
            kind: ZoneKind::Judge,
            owner: player,
            cards: vec![],
        }),
    };
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_200),
        kind: CardKind::TrickLe,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: zone_id,
        owner: player,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(zone_id)
        .unwrap()
        .cards
        .push(card);
    card
}

fn fire_game_started(tx: &mut Transaction, count: u8) {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::GameStarted {
            player_count: count,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("game-started dispatch error: {e:?}"));
    run_effects(tx, effects).unwrap();
}

fn fire_phase(tx: &mut Transaction, player: PlayerId, phase: TurnPhaseTag) {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: player,
            phase,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("phase dispatch error: {e:?}"));
    run_effects(tx, effects).unwrap();
}

fn has_skill_def(tx: &Transaction, player: PlayerId, def: u32) -> bool {
    tx.state()
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == player && skill.def.raw() == def)
}

#[test]
fn committed_package_publishes_cuorui_and_baobian_triggers() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let cuorui: Vec<&str> = pkg
        .trigger_index
        .iter()
        .filter(|t| t.skill_def_id == CUORUI)
        .map(|t| t.event.as_str())
        .collect();
    assert!(cuorui.contains(&"game_started"));
    assert!(cuorui.contains(&"phase_prepare"));
    assert!(pkg
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == BAOBIAN && t.event == "phase_prepare" }));
}

#[test]
fn cuorui_draws_min_five_max_hp_at_game_start() {
    let (engine, owner, _other) = start_with(&[PACKAGE]);
    let mut tx = Transaction::begin(engine.state().clone());
    strip_and_add(&mut tx, owner, CUORUI);
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    let hand_before = tx.state().hand_count(owner);

    fire_game_started(&mut tx, 2);

    let expected = max_hp.min(5);
    assert_eq!(
        tx.state().hand_count(owner),
        hand_before + expected,
        "game start draws min(5, maxHp)"
    );
}

#[test]
fn cuorui_skips_judge_phase_once_when_judge_area_nonempty() {
    let (engine, owner, _other) = start_with(&[PACKAGE]);
    let mut tx = Transaction::begin(engine.state().clone());
    strip_and_add(&mut tx, owner, CUORUI);
    inject_judge_zone(&mut tx, owner);
    let marks = |tx: &Transaction| {
        tx.state()
            .players
            .get(owner)
            .and_then(|p| p.marks.iter().find(|(id, _)| *id == CUORUI))
            .map(|(_, c)| *c)
            .unwrap_or(0)
    };

    fire_phase(&mut tx, owner, TurnPhaseTag::Prepare);
    assert_eq!(
        marks(&tx),
        1,
        "first prepare with judge cards consumes the once-gate"
    );

    fire_phase(&mut tx, owner, TurnPhaseTag::Prepare);
    assert_eq!(marks(&tx), 1, "second prepare does not re-fire (限定技)");
}

#[test]
fn baobian_grants_and_removes_skills_by_hp() {
    let (engine, owner, _other) = start_with(&[PACKAGE, REFRESH, SHENHUA]);
    let mut tx = Transaction::begin(engine.state().clone());
    strip_and_add(&mut tx, owner, BAOBIAN);

    // HP 1 → all three derived skills present.
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    fire_phase(&mut tx, owner, TurnPhaseTag::Prepare);
    assert!(has_skill_def(&tx, owner, OL_TIAOXIN));
    assert!(has_skill_def(&tx, owner, OL_PAOXIAO));
    assert!(has_skill_def(&tx, owner, XIN_SHENSU));

    // HP 4 → all three removed.
    tx.state_mut().players.get_mut(owner).unwrap().hp = 4;
    fire_phase(&mut tx, owner, TurnPhaseTag::Prepare);
    assert!(!has_skill_def(&tx, owner, OL_TIAOXIN));
    assert!(!has_skill_def(&tx, owner, OL_PAOXIAO));
    assert!(!has_skill_def(&tx, owner, XIN_SHENSU));

    // HP 3 → only 挑衅.
    tx.state_mut().players.get_mut(owner).unwrap().hp = 3;
    fire_phase(&mut tx, owner, TurnPhaseTag::Prepare);
    assert!(has_skill_def(&tx, owner, OL_TIAOXIN));
    assert!(!has_skill_def(&tx, owner, OL_PAOXIAO));
    assert!(!has_skill_def(&tx, owner, XIN_SHENSU));
}

#[test]
fn baobian_does_not_leak_derived_skills_to_other_players() {
    let (engine, owner, other) = start_with(&[PACKAGE, REFRESH, SHENHUA]);
    let mut tx = Transaction::begin(engine.state().clone());
    strip_and_add(&mut tx, owner, BAOBIAN);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    fire_phase(&mut tx, owner, TurnPhaseTag::Prepare);
    assert!(!has_skill_def(&tx, other, OL_TIAOXIN));
    assert!(!has_skill_def(&tx, other, OL_PAOXIAO));
}
