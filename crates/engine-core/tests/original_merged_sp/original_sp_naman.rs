//! Golden coverage for 纳蛮 rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const NAMAN: u32 = 4_147_868_153;
const SOURCE: &str = include_str!("../../../../content/original/ported/sp/naman.skill");

fn add_discard_card(tx: &mut Transaction, kind: CardKind) -> CardId {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(92_276),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: discard,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(discard)
        .unwrap()
        .cards
        .push(card);
    card
}

#[test]
fn gains_only_another_players_physical_sha_response() {
    let package = content_compiler::compile_source_named("naman-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 276);
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: 2_026_072_936,
            ruleset: "identity-standard".into(),
        },
    });
    let players = engine.state().players.alive_by_seat();
    let owner = players[0];
    let responder = players[1];
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NAMAN),
        }],
    )
    .unwrap();

    let sha = add_discard_card(&mut tx, CardKind::Sha);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::CardResponded {
                player: responder,
                respond_to: owner,
                card: sha,
                response_kind: CardKind::Sha.tag(),
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(sha).unwrap().owner, owner);
    assert!(state.hand_cards(owner).contains(&sha));

    let mut tx = Transaction::begin(state);
    let own_sha = add_discard_card(&mut tx, CardKind::Sha);
    let shan = add_discard_card(&mut tx, CardKind::Shan);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::CardResponded {
                player: owner,
                respond_to: responder,
                card: own_sha,
                response_kind: CardKind::Sha.tag(),
            },
            EffectRequest::CardResponded {
                player: responder,
                respond_to: owner,
                card: shan,
                response_kind: CardKind::Shan.tag(),
            },
        ],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(
        tx.state().cards.get(own_sha).unwrap().zone,
        tx.state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
    assert_eq!(
        tx.state().cards.get(shan).unwrap().zone,
        tx.state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}
