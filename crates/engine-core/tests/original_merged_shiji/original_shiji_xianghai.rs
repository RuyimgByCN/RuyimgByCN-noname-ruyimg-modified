//! Golden for SHIJI「乡害」的装备牌名与全局手牌上限光环语义。

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::query::{effective_card_kind, query, QueryContext};
use engine_core::state::CardKind;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const XIANGHAI: u32 = 3_356_961_773;
const XIANGHAI_G: u32 = 3_747_474_489;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/shiji/shiji.npack.json");

fn isolated() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == XIANGHAI));
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == XIANGHAI_G));
    assert!(package.modifier_index.iter().any(
        |modifier| modifier.skill_def_id == XIANGHAI && modifier.target == "global_hand_limit"
    ));
    assert!(!package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == XIANGHAI_G));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(XIANGHAI as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: XIANGHAI as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let players = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(XIANGHAI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, players[0], players[1], players[2])
}

fn hand_limit(tx: &Transaction, player: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::HandLimit { player },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("expected hand-limit integer, got {other:?}"),
    }
}

#[test]
fn owner_equipment_is_jiu_without_mutating_or_affecting_other_players() {
    let (mut tx, owner, other, _) = isolated();
    let card = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Armor;
    let physical = tx.state().cards.get(card).unwrap();

    assert_eq!(
        effective_card_kind(tx.state(), owner, physical),
        CardKind::Jiu
    );
    assert_eq!(
        effective_card_kind(tx.state(), other, physical),
        CardKind::Armor
    );
    assert_eq!(tx.state().cards.get(card).unwrap().kind, CardKind::Armor);
}

#[test]
fn living_owners_reduce_only_other_players_and_stack() {
    let (mut tx, first_owner, second_owner, target) = isolated();
    for player in [first_owner, second_owner, target] {
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.hp = 4;
        state.max_hp = 4;
    }

    assert_eq!(hand_limit(&tx, first_owner), 4);
    assert_eq!(hand_limit(&tx, second_owner), 3);
    assert_eq!(hand_limit(&tx, target), 3);

    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: second_owner,
            skill_def: SkillDefId::new(XIANGHAI),
        }],
    )
    .unwrap();
    assert_eq!(hand_limit(&tx, first_owner), 3);
    assert_eq!(hand_limit(&tx, second_owner), 3);
    assert_eq!(hand_limit(&tx, target), 2);

    tx.state_mut().players.get_mut(second_owner).unwrap().alive = false;
    assert_eq!(hand_limit(&tx, first_owner), 4);
    assert_eq!(hand_limit(&tx, target), 3);
    tx.state_mut().players.get_mut(first_owner).unwrap().alive = false;
    assert_eq!(hand_limit(&tx, target), 4);
}
