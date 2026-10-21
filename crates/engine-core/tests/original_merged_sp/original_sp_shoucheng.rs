//! Golden for SP「守成」：非当前回合角色失去最后的手牌后可摸一张牌。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SHOUCHENG: u32 = 3_263_722_944;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

fn resolve(tx: Transaction, owner: PlayerId) -> engine_core::state::GameState {
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(SHOUCHENG as u64),
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
fn only_non_current_hand_loss_to_empty_offers_the_draw() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == SHOUCHENG
            && trigger.event == "cards_moved"
            && trigger.filter == "owner_alive_event_player_outside_turn_from_hand_empty"
            && trigger.optional
    }));
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_715),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 20_260_715,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let players = engine.state().players.alive_by_seat();
    let (owner, other, current) = (players[0], players[1], players[2]);
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
            player: other,
            count: u32::MAX,
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SHOUCHENG),
        },
        EffectRequest::DrawCards {
            player: other,
            count: 1,
        },
    ]);
    run_effects(&mut tx, setup).unwrap();

    let other_hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let other_card = tx.state().hand_cards(other)[0];
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::MoveCards {
                cards: vec![other_card],
                to: discard,
                from: Some(other_hand),
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = resolve(tx, owner);
    assert_eq!(state.hand_count(other), 1);

    let mut tx = Transaction::begin(state);
    let other_hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let other_equip = tx.state().zones.find(ZoneKind::Equip, other).unwrap();
    let equipment = tx.state().hand_cards(other)[0];
    tx.state_mut().cards.get_mut(equipment).unwrap().kind = CardKind::Weapon;
    let shoucheng = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == SHOUCHENG)
        .unwrap()
        .id;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::RemoveSkill { skill: shoucheng },
            EffectRequest::MoveCards {
                cards: vec![equipment],
                to: other_equip,
                from: Some(other_hand),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(SHOUCHENG),
            },
            EffectRequest::MoveCards {
                cards: vec![equipment],
                to: discard,
                from: Some(other_equip),
            },
        ],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().hand_count(other), 0);

    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current,
                phase: Phase::Play.to_u8(),
            },
            EffectRequest::DiscardFromHand {
                player: current,
                count: u32::MAX,
            },
            EffectRequest::DrawCards {
                player: current,
                count: 1,
            },
        ],
    )
    .unwrap();
    let current_hand = tx.state().zones.find(ZoneKind::Hand, current).unwrap();
    let current_card = tx.state().hand_cards(current)[0];
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![current_card],
            to: discard,
            from: Some(current_hand),
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().hand_count(current), 0);
}
