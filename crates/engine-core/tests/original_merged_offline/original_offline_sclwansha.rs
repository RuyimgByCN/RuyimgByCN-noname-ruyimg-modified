//! Golden for OFFLINE「完杀」的回合、目标与死亡归因语义。

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const SCLWANSHA: u32 = 3_972_375_066;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

fn isolated_state() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 20_260_715,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let owner = engine.state().turn.current;
    let others: Vec<_> = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SCLWANSHA),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, others[0], others[1])
}

fn dying_effects(tx: &mut Transaction, player: PlayerId, source: PlayerId) -> Vec<EffectRequest> {
    dispatch_for_events(
        tx,
        &[DomainEvent::EnteredDying { player, source }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
}

#[test]
fn kills_only_another_player_during_the_owners_turn_and_preserves_the_killer() {
    let (mut tx, _, victim, killer) = isolated_state();
    {
        let player = tx.state_mut().players.get_mut(victim).unwrap();
        player.hp = 0;
        player.dying = true;
    }
    let effects = dying_effects(&mut tx, victim, killer);
    let events = run_effects(&mut tx, effects).unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        DomainEvent::PlayerKilled { player, killer: credited, .. }
            if *player == victim && *credited == killer
    )));
    let dead = tx.state().players.get(victim).unwrap();
    assert!(!dead.alive);
    assert!(!dead.dying);

    let (mut self_event, owner, _, killer) = isolated_state();
    assert!(dying_effects(&mut self_event, owner, killer).is_empty());

    let (mut other_turn, _, victim, current) = isolated_state();
    other_turn.state_mut().turn.current = current;
    assert!(dying_effects(&mut other_turn, victim, current).is_empty());
}
