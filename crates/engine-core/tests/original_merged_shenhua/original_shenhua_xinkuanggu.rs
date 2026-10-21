//! Golden for 神话再临「新狂骨」rewritten from the legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const XINKUANGGU: u32 = 4_089_972_415;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/shenhua/shenhua.npack.json");

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let seats = engine.state().players.alive_by_seat();
    let source = seats[0];
    let near = seats[1];
    let far = seats[2];
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: source,
        skill_def: SkillDefId::new(XINKUANGGU),
    });
    run_effects(&mut tx, effects).unwrap();
    for target in [near, far] {
        let player = tx.state_mut().players.get_mut(target).unwrap();
        player.max_hp = 10;
        player.hp = 10;
    }
    (tx, source, near, far)
}

fn resolve(
    state: GameState,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor: PlayerId::new(state.resolution.pending_observer),
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_contains_cancellable_per_damage_point_trigger() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == XINKUANGGU && !skill.locked));
    assert_eq!(
        package
            .trigger_index
            .iter()
            .filter(|trigger| trigger.skill_def_id == XINKUANGGU)
            .map(|trigger| trigger.filter.as_str())
            .collect::<Vec<_>>(),
        ["owner_source_distance_le_one_per_damage_point"]
    );
}

#[test]
fn near_multi_point_damage_offers_one_cancellable_choice_per_point() {
    let (mut tx, source, near, _) = setup(20_260_716_30);
    {
        let player = tx.state_mut().players.get_mut(source).unwrap();
        player.max_hp = 4;
        player.hp = 2;
    }
    let source_hand = tx.state().hand_count(source);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target: near,
                amount: 2,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    let choice = committed.output.choice.unwrap();
    assert!(choice.allow_cancel);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2]
    );

    let (state, pipeline) = resolve(committed.state, vec![2], false, 1);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
    assert_eq!(state.players.get(source).unwrap().hp, 3);

    let (state, pipeline) = resolve(state, vec![1], false, 2);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(source).unwrap().hp, 3);
    assert_eq!(state.hand_count(source), source_hand + 1);
    assert_eq!(state.players.get(near).unwrap().hp, 8);
}

#[test]
fn full_hp_can_cancel_the_draw_and_far_damage_does_not_trigger() {
    let (mut full, source, near, _) = setup(20_260_716_31);
    let hand = full.state().hand_count(source);
    assert_eq!(
        run_effects(
            &mut full,
            vec![EffectRequest::DealDamage {
                source,
                target: near,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let committed = full.commit();
    let choice = committed.output.choice.unwrap();
    assert!(choice.allow_cancel);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1]
    );
    let (state, pipeline) = resolve(committed.state, vec![], true, 3);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(source), hand);

    let (mut far_tx, source, _, far) = setup(20_260_716_32);
    let hand = far_tx.state().hand_count(source);
    run_effects(
        &mut far_tx,
        vec![EffectRequest::DealDamage {
            source,
            target: far,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert!(!far_tx.state().has_pending_choice());
    assert_eq!(far_tx.state().hand_count(source), hand);
}
