//! Golden coverage for SP OL 秦朗「贤膺」(olxianying).
//! Optional draw 2 after taking damage, and again at the owner's prepare.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OLXIANYING: u32 = 2_869_635_574;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/olxianying.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("olxianying-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_14, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLXIANYING),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    )
}

#[test]
fn compiled_source_and_npack_publish_both_optional_draw_windows() {
    let compiled = content_compiler::compile_source_named("olxianying", SOURCE).unwrap();
    assert!(compiled
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLXIANYING && skill.name == "olxianying" && !skill.locked));
    let mut filters: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == OLXIANYING)
        .map(|trigger| (trigger.event.as_str(), trigger.filter.as_str(), trigger.optional))
        .collect();
    filters.sort();
    assert_eq!(
        filters,
        vec![
            ("damage", "owner_is_target", true),
            ("phase_prepare", "owner_is_target", true),
        ]
    );

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let pack: Vec<_> = committed
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == OLXIANYING)
        .map(|trigger| trigger.event.as_str())
        .collect();
    assert!(pack.contains(&"damage"));
    assert!(pack.contains(&"phase_prepare"));
}

#[test]
fn damage_then_prepare_each_draw_two_when_confirmed() {
    let (state, owner, other) = setup();
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: other,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, confirmed) = resolve(tx.commit().state, owner, vec![1], 1);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before + 2);

    let mut tx = Transaction::begin(state);
    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    let after_damage = tx.state().hand_count(owner);
    let (state, confirmed) = resolve(tx.commit().state, owner, vec![1], 2);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), after_damage + 2);
}
