//! Golden for OFFLINE 袁绍「肃敌」的 respondTo 与攻击范围语义。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const YYSUDI: u32 = 3_521_654_010;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

fn isolated(player_count: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count,
                rng_seed: 20_260_715,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let others = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
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
        skill_def: SkillDefId::new(YYSUDI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, others)
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_900 + rank as u32),
        kind,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn draws_when_an_in_range_player_responds_to_the_owners_sha() {
    let (mut tx, owner, others) = isolated(2);
    let responder = others[0];
    tx.state_mut().turn.phase = Phase::Play;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::DiscardFromHand {
                player: owner,
                count: u32::MAX,
            },
            EffectRequest::DiscardFromHand {
                player: responder,
                count: u32::MAX,
            },
        ],
    )
    .unwrap();
    let sha = inject(&mut tx, owner, CardKind::Sha, 7);
    let shan = inject(&mut tx, responder, CardKind::Shan, 8);

    let (state, pending) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![responder],
                card_targets: vec![],
            },
        },
    );
    let choice = pending.choice.expect("Shan response choice");
    let hand_before = state.hand_count(owner);
    let (state, result) = apply_command(
        state,
        Command {
            actor: responder,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![shan.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before + 1);
}

#[test]
fn ignores_the_wrong_response_origin_and_an_out_of_range_responder() {
    let (mut tx, owner, others) = isolated(4);
    let far = others
        .iter()
        .copied()
        .max_by_key(|player| {
            match query(
                tx.state(),
                &QueryRequest::Distance {
                    from: *player,
                    to: owner,
                },
                &mut QueryContext::new(),
            ) {
                QueryResponse::Int(distance) => distance,
                _ => 0,
            }
        })
        .unwrap();
    let near = others
        .iter()
        .copied()
        .find(|player| *player != far)
        .unwrap();
    let before = tx.state().hand_count(owner);

    run_effects(
        &mut tx,
        vec![EffectRequest::CardResponded {
            player: near,
            respond_to: far,
            card: CardId::NONE,
            response_kind: CardKind::Shan.tag(),
        }],
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::CardResponded {
            player: far,
            respond_to: owner,
            card: CardId::NONE,
            response_kind: CardKind::Shan.tag(),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before);
}
