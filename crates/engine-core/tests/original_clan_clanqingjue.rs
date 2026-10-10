//! Golden coverage for clan 清绝 (clanqingjue): locked clan skill that, after the owner takes
//! damage, draws 1 (a verifiable subset of upstream's first-HP-change → discard non-unique-suit
//! hand cards → chooseButton give/gain behavior).
//!
//! Verifies the damage trigger + draw effect. The full upstream semantics (first-HP-change
//! history gate, suit-uniqueness analysis, chooseToDiscard, chooseButton give/gain, hand-limit
//! exemption modifier, discard-phase exemption) need additional primitives not yet ported;
//! see the .skill comment.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::{damage_nature, EffectRequest};

const CLANQINGJUE: u32 = 3_860_800_298;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/clan/clan.npack.json");

fn start() -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_814,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&pkg).unwrap();
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

fn owner_with_skill(tx: &mut Transaction, owner: PlayerId) {
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(CLANQINGJUE),
    });
    run_effects(tx, effects).unwrap();
}

#[test]
fn committed_package_publishes_clanqingjue_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == CLANQINGJUE
            && trigger.event == "damage"
            && trigger.filter == "owner_is_target"
    }));
}

#[test]
fn damage_to_owner_draws_one() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    owner_with_skill(&mut tx, owner);

    let hand_before = tx.state().hand_count(owner);
    // A Damage event with the owner as target fires clanqingjue → draw 1.
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: other,
            target: owner,
            amount: 1,
            nature: damage_nature::NORMAL,
            is_lose_hp: false,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(
        tx.state().hand_count(owner),
        hand_before + 1,
        "damage to owner → draw 1"
    );
}

#[test]
fn damage_to_other_does_not_draw_for_owner() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    owner_with_skill(&mut tx, owner);

    let hand_before = tx.state().hand_count(owner);
    // Damage to the other player: owner is not the target → no draw.
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: owner,
            target: other,
            amount: 1,
            nature: damage_nature::NORMAL,
            is_lose_hp: false,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(
        tx.state().hand_count(owner),
        hand_before,
        "damage to other → owner does not draw"
    );
}
