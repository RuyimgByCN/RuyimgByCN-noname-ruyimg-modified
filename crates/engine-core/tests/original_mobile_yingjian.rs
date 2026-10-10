//! Golden for MOBILE 孙茹「影箭」的无实体【杀】语义。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{turn_flag, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SUNRU: u32 = 3_566_559_635;
const YINGJIAN: u32 = 3_382_160_794;
const SHIXIN: u32 = 3_071_284_185;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn start(seed: u64) -> (Transaction, PlayerId, PlayerId) {
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
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let owner_index = seats.iter().position(|player| *player == owner).unwrap();
    let far = seats[(owner_index + 2) % seats.len()];
    let mut tx = Transaction::begin(engine.state().clone());
    let existing = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, existing).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    (tx, owner, far)
}

fn resolve(
    state: engine_core::state::GameState,
    actor: PlayerId,
    choice_id: ChoiceId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (
    engine_core::state::GameState,
    engine_core::command::PipelineResult,
) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn mobile_package_contains_complete_sunru() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let sunru = package
        .character_index
        .iter()
        .find(|character| character.id == SUNRU)
        .unwrap();
    assert_eq!(sunru.name_key, "sunru");
    assert_eq!(sunru.max_hp, 3);
    assert_eq!(sunru.skills, vec![YINGJIAN, SHIXIN]);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == YINGJIAN));
}

#[test]
fn virtual_sha_distance_policy_controls_target_options_and_cancel_cleans_up() {
    let (mut normal, owner, far) = start(20_260_716);
    assert_eq!(
        run_effects(
            &mut normal,
            vec![EffectRequest::ChooseUseVirtualSha {
                player: owner,
                ignore_distance: false,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let virtual_card = normal
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .and_then(|zone| normal.state().zones.get(zone))
        .and_then(|zone| zone.cards.last())
        .copied()
        .unwrap();
    let committed = normal.commit();
    let choice = committed.output.choice.unwrap();
    assert!(!choice
        .options
        .iter()
        .any(|option| option.token == far.raw()));
    let (state, result) = resolve(committed.state, owner, choice.choice_id, vec![], true, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let card = state.cards.get(virtual_card).unwrap();
    assert!(card.def.is_none());
    assert_eq!(card.suit, Suit::None);
    assert_eq!(state.zones.get(card.zone).unwrap().kind, ZoneKind::Removed);

    let (mut unlimited, owner, far) = start(20_260_717);
    assert_eq!(
        run_effects(
            &mut unlimited,
            vec![EffectRequest::ChooseUseVirtualSha {
                player: owner,
                ignore_distance: true,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let committed = unlimited.commit();
    let choice = committed.output.choice.as_ref().unwrap();
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == far.raw()));
}

#[test]
fn yingjian_uses_the_normal_sha_response_damage_and_history_pipeline() {
    let (mut tx, owner, far) = start(20_260_718);
    let response_card = tx.state().hand_cards(far)[0];
    tx.state_mut().cards.get_mut(response_card).unwrap().kind = CardKind::Shan;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(YINGJIAN),
        }],
    )
    .unwrap();
    let hp = tx.state().players.get(far).unwrap().hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Prepare.to_u8(),
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let committed = tx.commit();
    let target_choice = committed.output.choice.unwrap();
    assert!(target_choice
        .options
        .iter()
        .any(|option| option.token == far.raw()));

    let (state, target_result) = resolve(
        committed.state,
        owner,
        target_choice.choice_id,
        vec![far.raw()],
        false,
        2,
    );
    assert!(matches!(
        target_result.result,
        CommandResult::Accepted { .. }
    ));
    let response = target_result.choice.unwrap();
    assert_eq!(response.observer, far);
    let (state, response_result) = resolve(state, far, response.choice_id, vec![], true, 3);
    assert!(matches!(
        response_result.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(state.players.get(far).unwrap().hp, hp - 1);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 1);

    let removed = state.zones.find(ZoneKind::Removed, PlayerId::NONE).unwrap();
    let virtual_sha = state
        .zones
        .get(removed)
        .unwrap()
        .cards
        .iter()
        .copied()
        .find(|card| {
            state.cards.get(*card).is_some_and(|card| {
                card.def.is_none() && card.kind == CardKind::Sha && card.suit == Suit::None
            })
        })
        .unwrap();
    assert_eq!(state.cards.get(virtual_sha).unwrap().owner, PlayerId::NONE);
}
