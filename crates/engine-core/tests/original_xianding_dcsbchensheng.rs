//! Golden for XIANDING「沉声」的整回合结束与唯一最多手牌语义。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const SKILL: u32 = 2_769_678_482;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

#[test]
fn chensheng_only_offers_draw_after_another_turn_when_neither_is_unique_max_hand() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_714_5),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: 20_260_714_5,
            ruleset: "identity-standard".into(),
        },
    });
    let players = engine.state().players.alive_by_seat();
    let (owner, ending_player, leader) = (players[0], players[1], players[2]);
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SKILL),
        },
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
        EffectRequest::DiscardFromHand {
            player: ending_player,
            count: u32::MAX,
        },
        EffectRequest::DiscardFromHand {
            player: leader,
            count: u32::MAX,
        },
        EffectRequest::DrawCards {
            player: owner,
            count: 1,
        },
        EffectRequest::DrawCards {
            player: ending_player,
            count: 1,
        },
        EffectRequest::DrawCards {
            player: leader,
            count: 2,
        },
    ]);
    run_effects(&mut tx, effects).unwrap();

    let before = tx.state().hand_count(owner);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::PhaseEnded {
                current: ending_player,
                phase: TurnPhaseTag::End,
                damage_sources: 0,
                damaged_players: 0,
                card_targeted_other: false,
                current_dealt_damage_other: false,
                phase_targeted_players: 0,
                phase_hp_changed_players: 0,
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before + 1);

    let mut tx = Transaction::begin(state);
    assert!(dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: ending_player,
            phase: TurnPhaseTag::Play,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());

    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 1,
        }],
    )
    .unwrap();
    assert!(dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: ending_player,
            phase: TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());

    run_effects(
        &mut tx,
        vec![
            EffectRequest::DiscardFromHand {
                player: owner,
                count: u32::MAX,
            },
            EffectRequest::DrawCards {
                player: ending_player,
                count: 3,
            },
        ],
    )
    .unwrap();
    assert!(dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: ending_player,
            phase: TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());
}
