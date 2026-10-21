//! Golden coverage for sp 炙日 (zhiri): awakening skill that, at the owner's
//! prepare phase, if the 「焚」 (fentian) skill pile holds at least 3 cards,
//! reduces the owner's max HP by 1 and grants 心惔 (xintan). Fires at most once.
//!
//! Exercises the awakening approximation (`mark_count` once-gate), the
//! `skill_pile_count` query, and `change_max_hp` + `add_skill` effects. The
//! upstream `awakenSkill` once-per-game guard is approximated with the
//! awakening mark (see the .skill comment).

use engine_api::{
    CardDefId, CardId, Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit, TurnPhaseTag, ZoneId,
};
use engine_core::state::{Card, CardKind, SkillPileRef, Zone, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;

const ZHIRI: u32 = 2_217_683_305;
const XINTAN: u32 = 2_185_342_847;
const FENTIAN: u32 = 3_669_648_107;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

fn start() -> (Engine, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_817,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&pkg).unwrap();
    let owner = engine.state().turn.current;
    (engine, owner)
}

fn add_zhiri(tx: &mut Transaction, owner: PlayerId) {
    use engine_core::effect::run_effects;
    use rule_api::EffectRequest;
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
        skill_def: SkillDefId::new(ZHIRI),
    });
    run_effects(tx, effects).unwrap();
}

/// Seeds `count` face-up cards into the owner's fentian (焚) skill pile.
fn seed_fentian(tx: &mut Transaction, owner: PlayerId, count: usize) {
    let zone_id = tx.state_mut().zones.alloc(Zone {
        id: ZoneId::NONE,
        kind: ZoneKind::Expansion,
        owner,
        cards: vec![],
    });
    let mut cards = Vec::new();
    for index in 0..count {
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(170_100 + index as u32),
            kind: CardKind::Sha,
            suit: Suit::Spade,
            rank: 7,
            weapon_range: 0,
            zone: zone_id,
            owner,
            face_up: true,
        });
        cards.push(card);
    }
    tx.state_mut().zones.get_mut(zone_id).unwrap().cards = cards;
    tx.state_mut().skill_piles.push(SkillPileRef {
        owner,
        skill_def: FENTIAN,
        zone: zone_id,
        usable_as_hand: false,
        wants_lost_count: false,
    });
}

fn fire_prepare(tx: &mut Transaction, owner: PlayerId) {
    use engine_core::effect::run_effects;
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("prepare dispatch error: {e:?}"));
    run_effects(tx, effects).unwrap();
}

fn once_mark(tx: &Transaction, owner: PlayerId) -> i32 {
    tx.state()
        .players
        .get(owner)
        .and_then(|p| p.marks.iter().find(|(id, _)| *id == ZHIRI))
        .map(|(_, count)| *count)
        .unwrap_or(0)
}

#[test]
fn committed_package_publishes_zhiri_prepare_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == ZHIRI
            && trigger.event == "phase_prepare"
            && trigger.filter == "owner_is_target"
    }));
}

#[test]
fn fentian_at_three_awakens_once() {
    let (engine, owner) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    add_zhiri(&mut tx, owner);
    seed_fentian(&mut tx, owner, 3);
    let max_hp_before = tx.state().players.get(owner).unwrap().max_hp;
    assert!(
        !tx.state()
            .skills
            .sorted_live()
            .iter()
            .any(|skill| skill.def.raw() == XINTAN),
        "心惔 not granted before awakening"
    );

    fire_prepare(&mut tx, owner);

    let owner_state = tx.state().players.get(owner).unwrap();
    assert_eq!(
        owner_state.max_hp,
        max_hp_before - 1,
        "awakening loses 1 max HP"
    );
    assert_eq!(once_mark(&tx, owner), 1, "awakening mark set");
    assert!(
        tx.state()
            .skills
            .sorted_live()
            .iter()
            .any(|skill| skill.def.raw() == XINTAN),
        "心惔 granted after awakening"
    );

    // A later prepare phase must not re-fire the awakening.
    let max_hp_after = tx.state().players.get(owner).unwrap().max_hp;
    fire_prepare(&mut tx, owner);
    assert_eq!(
        tx.state().players.get(owner).unwrap().max_hp,
        max_hp_after,
        "awakening fires at most once"
    );
}

#[test]
fn fentian_below_three_does_not_awaken() {
    let (engine, owner) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    add_zhiri(&mut tx, owner);
    seed_fentian(&mut tx, owner, 2);
    let max_hp_before = tx.state().players.get(owner).unwrap().max_hp;

    fire_prepare(&mut tx, owner);

    assert_eq!(
        tx.state().players.get(owner).unwrap().max_hp,
        max_hp_before,
        "2 cards in 焚 → no awakening"
    );
    assert_eq!(once_mark(&tx, owner), 0);
    assert!(!tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.def.raw() == XINTAN));
}
