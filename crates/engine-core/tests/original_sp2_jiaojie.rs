//! Golden for SP2「狡黠」按颜色生效的手牌、距离与次数 modifier。

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
    Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::state::{CardKind, Phase};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const JIAOJIE: u32 = 3_299_853_333;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn target_legal(
    tx: &Transaction,
    owner: PlayerId,
    card: engine_api::CardId,
    target: PlayerId,
) -> bool {
    matches!(
        query(
            tx.state(),
            &QueryRequest::TargetLegal {
                user: owner,
                card,
                target,
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Bool(true)
    )
}

#[test]
fn red_cards_are_protected_and_only_black_cards_ignore_range_and_use_count() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == JIAOJIE && skill.locked));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 20_260_714,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let owner_index = seats.iter().position(|player| *player == owner).unwrap();
    let near = seats[(owner_index + 1) % seats.len()];
    let far = seats[(owner_index + 2) % seats.len()];

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
        skill_def: SkillDefId::new(JIAOJIE),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut()
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED, 1);
    tx.state_mut()
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 1);

    let cards = tx.state().hand_cards(owner);
    assert!(cards.len() >= 2);
    let black = cards[0];
    let red = cards[1];
    {
        let card = tx.state_mut().cards.get_mut(black).unwrap();
        card.kind = CardKind::Sha;
        card.suit = Suit::Spade;
    }
    {
        let card = tx.state_mut().cards.get_mut(red).unwrap();
        card.kind = CardKind::Sha;
        card.suit = Suit::Heart;
    }

    assert!(engine_core::query::card_ignored_for_hand_limit(
        tx.state(),
        owner,
        red
    ));
    assert!(!engine_core::query::card_ignored_for_hand_limit(
        tx.state(),
        owner,
        black
    ));
    let discardable = engine_core::query::discard_to_limit_tokens(tx.state(), owner);
    assert!(!discardable.contains(&red));
    assert!(discardable.contains(&black));

    assert_eq!(engine_core::query::sha_use_limit_for(tx.state(), owner), 1);
    assert_eq!(
        engine_core::query::sha_use_limit_for_card(tx.state(), owner, black),
        u32::MAX
    );
    assert_eq!(
        engine_core::query::sha_use_limit_for_card(tx.state(), owner, red),
        1
    );
    assert!(target_legal(&tx, owner, black, far));
    assert!(!target_legal(&tx, owner, red, far));

    let (unchanged, rejected) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: red,
                targets: vec![near],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(unchanged, *tx.state());

    let (_, accepted) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::PlayCard {
                card: black,
                targets: vec![far],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
}
