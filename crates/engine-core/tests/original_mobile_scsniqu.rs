//! Golden tests for MOBILE「逆取」rewritten from the legacy JavaScript semantics.

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SCSNIQU: u32 = 2_407_745_929;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn started_state(seed: u64) -> (GameState, PlayerId, PlayerId, SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let target = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    for player in [owner, target] {
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.hp = 10;
        state.max_hp = 10;
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SCSNIQU),
        }],
    )
    .unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == SCSNIQU)
        .unwrap();
    (tx.state().clone(), owner, target, skill)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets,
                card_targets: vec![],
            },
        },
    )
}

#[test]
fn package_contains_the_active_contract_without_a_fake_selectable_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == SCSNIQU)
        .expect("逆取 active entry");
    assert_eq!(active.uses_per_turn, Some(1));
    assert_eq!(
        active
            .targets
            .as_ref()
            .map(|targets| (targets.min, targets.max, targets.allow_self)),
        Some((1, 1, true))
    );
    // scs_guosheng was added to the mobile pack by the upstream source update.
}

#[test]
fn deals_fire_damage_and_rejects_a_second_use_without_mutation() {
    let (state, owner, target, skill) = started_state(2_026_071_512);
    let hp = state.players.get(target).unwrap().hp;
    let (state, pipe) = activate(state, owner, skill, vec![target], 1);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert!(pipe.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            source,
            target: damaged,
            amount: 1,
            is_lose_hp: false,
            nature: 1,
        } if *source == owner && *damaged == target
    )));

    let before = canonical_state_hash(&state);
    let (same, pipe) = activate(state, owner, skill, vec![target], 2);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}

#[test]
fn missing_or_dead_target_is_rejected_without_mutation() {
    let (state, owner, target, skill) = started_state(2_026_071_513);
    let before = canonical_state_hash(&state);
    let (same, pipe) = activate(state, owner, skill, vec![], 1);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let mut dead = same;
    dead.players.get_mut(target).unwrap().alive = false;
    let before = canonical_state_hash(&dead);
    let (same, pipe) = activate(dead, owner, skill, vec![target], 2);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
