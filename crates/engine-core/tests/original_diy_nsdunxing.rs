//! Golden for DIY 左慈「遁形」的伤害前可选替代。

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

const NSDUNXING: u32 = 4_203_825_879;
const NSXINSHENG: u32 = 2_604_261_106;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/diy/diy.npack.json");

fn started() -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.character_index.iter().any(|character| {
        character.name_key == "ns_zuoci" && character.skills == [NSXINSHENG, NSDUNXING]
    }));
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_715),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_715,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
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
        skill_def: SkillDefId::new(NSDUNXING),
    });
    run_effects(&mut tx, effects).unwrap();
    let player = tx.state_mut().players.get_mut(owner).unwrap();
    player.max_hp = 3;
    player.hp = 3;
    (tx, owner, source)
}

fn resolve(
    tx: Transaction,
    owner: PlayerId,
    selection: u32,
    key: u64,
) -> engine_core::state::GameState {
    let committed = tx.commit();
    let choice_id = ChoiceId::new(committed.state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        committed.state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn healthy_damage_is_normal_and_wounded_damage_can_be_declined_or_replaced() {
    let (mut tx, owner, source) = started();
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 2);

    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 2);
    let state = resolve(tx, owner, 0, 1);
    assert_eq!(state.players.get(owner).unwrap().hp, 1);

    let mut tx = Transaction::begin(state);
    let hand_before = tx.state().hand_count(owner);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target: owner,
                amount: 2,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx, owner, 1, 2);
    let player = state.players.get(owner).unwrap();
    assert_eq!(player.hp, 1);
    assert_eq!(player.max_hp, 1);
    assert_eq!(state.hand_count(owner), hand_before + 2);
    assert!(state.resolution.prevented_damage_ids.is_empty());
    assert_eq!(state.resolution.current_damage_id, 0);
}
