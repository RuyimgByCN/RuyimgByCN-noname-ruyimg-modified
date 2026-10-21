//! Golden for 联动「虚构」：红色【杀】免疫、无距离限制且伤害 +1。

use engine_api::{
    Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::state::{CardKind, Phase};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const OLXUGOU: u32 = 2_803_220_890;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/collab/collab.npack.json");

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
        other => panic!("unexpected target query: {other:?}"),
    }
}

#[test]
fn red_sha_is_blocked_incoming_and_boosted_outgoing() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let skill = package
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == OLXUGOU)
        .expect("olxugou must be packaged");
    assert!(skill.locked);

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(2_803_220_890),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: 2_803_220_890,
            ruleset: "identity-standard".into(),
        },
    });

    let seats = engine.state().players.alive_by_seat();
    let owner = seats[0];
    let attacker = seats[1];
    let distant = seats[2];
    let mut tx = Transaction::begin(engine.state().clone());
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(OLXUGOU),
        }],
    )
    .unwrap();

    let outgoing = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(outgoing).unwrap().kind = CardKind::Sha;
    tx.state_mut().cards.get_mut(outgoing).unwrap().suit = Suit::Heart;
    assert!(engine_core::query::card_target_within_distance(
        tx.state(),
        owner,
        outgoing,
        distant,
        1,
    ));
    assert_eq!(
        engine_core::query::sha_damage_for_card(tx.state(), owner, outgoing, 1),
        2
    );
    tx.state_mut().cards.get_mut(outgoing).unwrap().suit = Suit::Spade;
    assert!(!engine_core::query::card_target_within_distance(
        tx.state(),
        owner,
        outgoing,
        distant,
        1,
    ));
    assert_eq!(
        engine_core::query::sha_damage_for_card(tx.state(), owner, outgoing, 1),
        1
    );

    tx.state_mut().turn.current = attacker;
    tx.state_mut().turn.phase = Phase::Play;
    let incoming = tx.state().hand_cards(attacker)[0];
    tx.state_mut().cards.get_mut(incoming).unwrap().kind = CardKind::Sha;
    tx.state_mut().cards.get_mut(incoming).unwrap().suit = Suit::Diamond;
    assert!(!target_legal(&tx, attacker, incoming, owner));
    tx.state_mut().cards.get_mut(incoming).unwrap().suit = Suit::Club;
    assert!(target_legal(&tx, attacker, incoming, owner));
}
