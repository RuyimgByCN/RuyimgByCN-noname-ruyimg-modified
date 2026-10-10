//! Golden for YIJIANG「制蛮」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const REZHIMAN: u32 = 3_521_391_741;
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn rezhiman_gains_judgment_card_then_prevents_damage() {
    let package = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    assert_eq!(package.skill_index.len(), 281);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == REZHIMAN
            && trigger.event == "damage_before"
            && trigger.filter == "owner_source_other_damage_before"
            && trigger.optional
    }));

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_731, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let target = seats[1];
    let remove: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(REZHIMAN),
        }],
    )
    .unwrap();

    let judge = tx.state().zones.find(ZoneKind::Judge, target).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(97_334),
        kind: CardKind::TrickLe,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: judge,
        owner: target,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(judge)
        .unwrap()
        .cards
        .push(card);
    let hp = tx.state().players.get(target).unwrap().hp;

    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: owner,
                target,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );

    let (state, accepted) = resolve(tx.commit().state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    let choice = accepted.choice.expect("制蛮应请求选择一张目标牌");
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(choice.observer, owner);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == card.raw()));

    let (state, selected) = resolve(state, owner, vec![card.raw()], false, 2);
    assert!(matches!(selected.result, CommandResult::Accepted { .. }));
    assert!(selected.choice.is_none());
    assert_eq!(state.players.get(target).unwrap().hp, hp);
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    assert_eq!(state.cards.get(card).unwrap().zone, hand);
    assert_eq!(state.cards.get(card).unwrap().owner, owner);
    assert_eq!(state.resolution.current_damage_id, 0);
}
