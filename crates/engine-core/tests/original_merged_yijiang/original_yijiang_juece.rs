//! Golden for 一将成名「绝策」：拥有者回合内对失去最后手牌的其他角色造成伤害。

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

const JUECE: u32 = 2_465_882_550;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/yijiang/yijiang.npack.json");

fn accept(tx: &Transaction, owner: PlayerId) -> engine_core::state::GameState {
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(JUECE as u64),
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
fn only_last_hand_loss_during_the_owners_turn_offers_damage() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == JUECE
            && trigger.event == "cards_moved"
            && trigger.filter == "owner_alive_current_turn_other_event_player_from_hand_empty"
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
    let (owner, other, third) = (players[0], players[1], players[2]);
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
            skill_def: SkillDefId::new(JUECE),
        },
        EffectRequest::DrawCards {
            player: other,
            count: 1,
        },
    ]);
    run_effects(&mut tx, setup).unwrap();

    let hand_zone = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let card = tx.state().hand_cards(other)[0];
    let hp = tx.state().players.get(other).unwrap().hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::MoveCards {
                cards: vec![card],
                to: discard,
                from: Some(hand_zone),
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept(&tx, owner);
    assert_eq!(state.players.get(other).unwrap().hp, hp - 1);

    let mut tx = Transaction::begin(state);
    let juece = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == JUECE)
        .unwrap()
        .id;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::RemoveSkill { skill: juece },
            EffectRequest::DrawCards {
                player: other,
                count: 1,
            },
        ],
    )
    .unwrap();
    let hand_zone = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let equip_zone = tx.state().zones.find(ZoneKind::Equip, other).unwrap();
    let equipment = tx.state().hand_cards(other)[0];
    tx.state_mut().cards.get_mut(equipment).unwrap().kind = CardKind::Weapon;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::MoveCards {
                cards: vec![equipment],
                to: equip_zone,
                from: Some(hand_zone),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(JUECE),
            },
            EffectRequest::MoveCards {
                cards: vec![equipment],
                to: discard,
                from: Some(equip_zone),
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(other).unwrap().hp, hp - 1);

    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: third,
                phase: Phase::Play.to_u8(),
            },
            EffectRequest::DrawCards {
                player: other,
                count: 1,
            },
        ],
    )
    .unwrap();
    let hand_zone = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let card = tx.state().hand_cards(other)[0];
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: discard,
            from: Some(hand_zone),
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().players.get(other).unwrap().hp, hp - 1);
}
