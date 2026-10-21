//! Golden coverage for HUICUI「守成」and its once-per-turn last-hand-loss filter.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const DCSHOUCHENG: u32 = 4_099_961_254;
const RESHENGXI: u32 = 2_846_213_948;
const JIANGFEI: u32 = 4_198_602_555;
const HUICUI: &[u8] =
    include_bytes!("../../../../content/original/ported/huicui/huicui.npack.json");
const SHIJI: &[u8] = include_bytes!("../../../../content/original/ported/shiji/shiji.npack.json");

fn loaded() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let huicui = content_schema::ContentPackage::from_json_bytes(HUICUI).unwrap();
    let shiji = content_schema::ContentPackage::from_json_bytes(SHIJI).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&shiji).unwrap();
    engine.load_package(&huicui).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(DCSHOUCHENG as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: DCSHOUCHENG as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let players = engine.state().players.alive_by_seat();
    let (owner, target, current) = (players[0], players[1], players[2]);
    let mut tx = Transaction::begin(engine.state().clone());
    let mut setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    setup.extend([
        EffectRequest::DiscardFromHand {
            player: target,
            count: u32::MAX,
        },
        EffectRequest::DiscardFromHand {
            player: current,
            count: u32::MAX,
        },
        EffectRequest::SetTurnState {
            current,
            phase: Phase::Play.to_u8(),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCSHOUCHENG),
        },
        EffectRequest::DrawCards {
            player: target,
            count: 1,
        },
        EffectRequest::DrawCards {
            player: current,
            count: 1,
        },
    ]);
    run_effects(&mut tx, setup).unwrap();
    (tx, owner, target, current)
}

fn move_all_hand(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<engine_api::DomainEvent>, EffectError> {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let cards = tx.state().hand_cards(player);
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards,
            to: discard,
            from: Some(hand),
        }],
    )
}

fn accept(tx: Transaction, owner: PlayerId, key: u64) -> engine_core::state::GameState {
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn committed_package_declares_dcshoucheng_and_closes_jiangfei() {
    let package = content_schema::ContentPackage::from_json_bytes(HUICUI).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 26);
    assert!(package.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "shiji"
            && dependency.version == "0.1.0"
    }));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCSHOUCHENG)
        .unwrap();
    assert_eq!(trigger.event, "cards_moved");
    assert_eq!(
        trigger.filter,
        "owner_alive_event_player_outside_turn_from_hand_empty_once"
    );
    assert!(trigger.optional);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == JIANGFEI)
        .unwrap();
    assert_eq!(character.name_key, "jiangfei");
    assert_eq!(character.skills, vec![RESHENGXI, DCSHOUCHENG]);
}

#[test]
fn dcshoucheng_draws_two_only_once_per_turn_and_resets_next_turn() {
    let (mut tx, owner, target, current) = loaded();
    assert_eq!(
        move_all_hand(&mut tx, target).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept(tx, owner, 1);
    assert_eq!(state.hand_count(target), 2);

    let mut tx = Transaction::begin(state);
    move_all_hand(&mut tx, target).unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().hand_count(target), 0);

    tx.state_mut().turn.phase = Phase::End;
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: current,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(20_260_718_03),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.current, owner);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: target,
            count: 1,
        }],
    )
    .unwrap();
    assert_eq!(
        move_all_hand(&mut tx, target).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept(tx, owner, 2);
    assert_eq!(state.hand_count(target), 2);
}

#[test]
fn current_turn_player_losing_their_last_hand_does_not_trigger() {
    let (mut tx, _, _, current) = loaded();
    move_all_hand(&mut tx, current).unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().hand_count(current), 0);
}
