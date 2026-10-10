//! Golden for standard「空城」target legality.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::state::{CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const KONGCHENG: u32 = 3_681_607_931;
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let cards = tx.state().hand_cards(player);
    if !cards.is_empty() {
        run_effects(
            tx,
            vec![EffectRequest::MoveCards {
                cards,
                to: discard,
                from: Some(hand),
            }],
        )
        .unwrap();
    }
}

fn strip_skills(tx: &mut Transaction, player: PlayerId) {
    let skills = tx.state().players.get(player).unwrap().skills.clone();
    for skill in skills {
        run_effects(tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
    }
}

fn target_legal(
    tx: &Transaction,
    user: PlayerId,
    card: engine_api::CardId,
    target: PlayerId,
) -> bool {
    match query(
        tx.state(),
        &QueryRequest::TargetLegal { user, card, target },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Bool(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    }
}

#[test]
fn kongcheng_blocks_sha_and_juedou_only_while_hand_is_empty() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == KONGCHENG && skill.locked));
    assert!(package.character_index.iter().any(|character| {
        character.name_key == "zhugeliang" && character.skills.contains(&KONGCHENG)
    }));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_786),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_786,
            ruleset: "identity-standard".into(),
        },
    });
    let user = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != user)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    strip_skills(&mut tx, user);
    strip_skills(&mut tx, target);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(KONGCHENG),
        }],
    )
    .unwrap();
    tx.state_mut().turn.current = user;
    tx.state_mut().turn.phase = Phase::Play;

    let cards = tx.state().hand_cards(user);
    assert!(cards.len() >= 2);
    let sha = cards[0];
    let juedou = cards[1];
    tx.state_mut().cards.get_mut(sha).unwrap().kind = CardKind::Sha;
    tx.state_mut().cards.get_mut(juedou).unwrap().kind = CardKind::TrickJuedou;
    assert!(target_legal(&tx, user, sha, target));
    assert!(target_legal(&tx, user, juedou, target));

    clear_hand(&mut tx, target);
    assert!(!target_legal(&tx, user, sha, target));
    assert!(!target_legal(&tx, user, juedou, target));

    for (key, card) in [(20_260_787, sha), (20_260_788, juedou)] {
        let before = tx.state().clone();
        let (after, pipe) = apply_command(
            before.clone(),
            Command {
                actor: user,
                expected_revision: ExpectedRevision(before.revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::PlayCard {
                    card,
                    targets: vec![target],
                    card_targets: vec![],
                },
            },
        );
        assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
        assert_eq!(after, before);
    }

    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: target,
            count: 1,
        }],
    )
    .unwrap();
    assert!(target_legal(&tx, user, sha, target));
    assert!(target_legal(&tx, user, juedou, target));
}
