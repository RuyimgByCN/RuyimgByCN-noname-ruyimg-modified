//! Golden coverage for BINGSHI 势陈到「往烈」「弘毅」.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::query;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const POTWANGLIE: u32 = 3_694_841_232;
const POTWANGLIE_DEBUFF: u32 = 3_694_841_233;
const POTHONGYI: u32 = 3_733_552_053;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(skill_def: u32) -> (engine_core::state::GameState, PlayerId, engine_api::SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(519),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 519,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
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
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == skill_def)
        .unwrap()
        .id;
    (tx.commit().state, owner, skill)
}

fn choose(
    state: engine_core::state::GameState,
    selections: Vec<u32>,
) -> engine_core::state::GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(5_190),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

fn mark_count(state: &engine_core::state::GameState, player: PlayerId) -> i32 {
    state
        .players
        .get(player)
        .and_then(|player| player.marks.iter().find(|(mark, _)| *mark == POTHONGYI))
        .map(|(_, count)| *count)
        .unwrap_or(0)
}

#[test]
fn package_publishes_complete_pot_chendao_at_abi519() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    let character = package
        .character_index
        .iter()
        .find(|character| character.name_key == "pot_chendao")
        .unwrap();
    assert_eq!(character.skills, vec![POTWANGLIE, POTHONGYI]);
}

#[test]
fn wanglie_arms_only_the_chosen_card_then_enables_self_target_lock() {
    let (state, owner, skill) = setup(POTWANGLIE);
    let card = state.hand_cards(owner)[0];
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotwanglie {
                player: owner,
                card: engine_api::CardId::NONE,
                helper_skill_def: SkillDefId::new(POTWANGLIE_DEBUFF),
                stage: 0,
                skill,
            }],
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, vec![card.raw()]);
    assert!(query::card_distance_unlimited_for_card(&state, owner, card));
    let physical = state.cards.get(card).unwrap();
    assert!(query::card_use_unrespondable_for_card(
        &state,
        owner,
        card,
        query::effective_card_kind(&state, owner, physical),
        query::effective_card_suit(&state, owner, physical),
    ));

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePotwanglie {
            player: owner,
            card,
            helper_skill_def: SkillDefId::new(POTWANGLIE_DEBUFF),
            stage: 1,
            skill,
        }],
    )
    .unwrap();
    assert!(query::card_targets_self_only_for(tx.state(), owner));
    assert!(!query::card_distance_unlimited_for_card(
        tx.state(),
        owner,
        card
    ));
}

#[test]
fn hongyi_caps_at_five_and_runs_the_opposite_end_phase_branch() {
    let (state, owner, skill) = setup(POTHONGYI);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePothongyi {
            player: owner,
            stage: 0,
            skill,
        }],
    )
    .unwrap();
    for _ in 0..3 {
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePothongyi {
                player: owner,
                stage: 1,
                skill,
            }],
        )
        .unwrap();
    }
    assert_eq!(mark_count(tx.state(), owner), 5);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePothongyi {
                player: owner,
                stage: 2,
                skill,
            }],
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    );
    let before = tx.state().hand_count(owner);
    let state = choose(tx.commit().state, vec![0]);
    assert_eq!(state.hand_count(owner), before + 5);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePothongyi {
            player: owner,
            stage: 3,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(mark_count(tx.state(), owner), 0);

    let (state, owner, skill) = setup(POTHONGYI);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePothongyi {
            player: owner,
            stage: 0,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePothongyi {
                player: owner,
                stage: 2,
                skill,
            }],
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, vec![1]);
    assert_eq!(mark_count(&state, owner), 0);
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePothongyi {
            player: owner,
            stage: 3,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 4);
}
