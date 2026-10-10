//! Golden for OFFLINE「惧鞭」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::ContinuationKind;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const JUBIAN: u32 = 3_008_850_817;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn isolated(hand: u32) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(JUBIAN as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: JUBIAN as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
        EffectRequest::DrawCards {
            player: owner,
            count: hand,
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JUBIAN),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 4;
    (tx, owner, other)
}

fn damage(tx: &mut Transaction, source: PlayerId, target: PlayerId) -> Result<(), EffectError> {
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .map(|_| ())
}

#[test]
fn committed_package_exposes_pre_damage_hand_above_hp_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == JUBIAN)
        .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert_eq!(
        (
            trigger.event.as_str(),
            trigger.filter.as_str(),
            trigger.optional
        ),
        ("damage_before", "owner_is_target_hand_above_hp", false)
    );
}

#[test]
fn excess_hand_choice_resumes_then_prevents_damage() {
    let (mut tx, owner, other) = isolated(6);
    let original = tx.state().hand_cards(owner);
    assert_eq!(
        damage(&mut tx, other, owner),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::AdjustHandTo
    );
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 4);
    let discarded = vec![original[2].raw(), original[3].raw()];
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(JUBIAN as u64 + 1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: discarded,
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 4);
    assert_eq!(state.players.get(owner).unwrap().hp, 4);
}

#[test]
fn hand_not_above_hp_takes_damage_and_source_is_not_restricted() {
    let (mut ordinary, owner, other) = isolated(4);
    damage(&mut ordinary, other, owner).unwrap();
    assert_eq!(ordinary.state().players.get(owner).unwrap().hp, 3);

    let (mut self_damage, owner, _) = isolated(5);
    assert_eq!(
        damage(&mut self_damage, owner, owner),
        Err(EffectError::ChoiceRequested)
    );
}
