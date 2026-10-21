//! Golden coverage for 新涯角 and old 赵云 rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NEW_YAJIAO: u32 = 3_751_106_523;
const SOURCE: &str = include_str!("../../../../content/original/ported/refresh/new_yajiao.skill");

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    def: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Diamond,
        rank: 7,
        weapon_range: kind.is_equip().then_some(2).unwrap_or(0),
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(top_kind: CardKind, seed: u64) -> (GameState, PlayerId, PlayerId, CardId, CardId) {
    let package = content_compiler::compile_source_named("new-yajiao-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 284);
    assert_eq!(package.trigger_index.len(), 2);
    assert!(package
        .trigger_index
        .iter()
        .all(|trigger| { trigger.filter == "owner_source_outside_turn_card_used_or_responded" }));
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let current = players[0];
    let owner = players[1];
    let recipient = players[2];
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NEW_YAJIAO),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = current;
    tx.state_mut().turn.phase = Phase::Play;
    let source_card = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 98_301);
    let discard_card = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Shan, 98_302);
    let revealed = inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        top_kind,
        98_303,
    );
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def == SkillDefId::new(NEW_YAJIAO))
        .unwrap()
        .id;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveNewYajiao {
                player: owner,
                source_card,
                revealed: CardId::NONE,
                target: PlayerId::NONE,
                source_category: 0,
                stage: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .unwrap();
    assert_eq!(tx.state().cards.get(revealed).unwrap().zone, processing);
    (tx.commit().state, owner, recipient, revealed, discard_card)
}

fn choose(state: GameState, actor: PlayerId, selection: u32, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    state
}

#[test]
fn revealed_card_is_given_and_only_category_mismatch_forces_owner_discard() {
    let (state, owner, recipient, revealed, discard_card) = setup(CardKind::Weapon, 2_026_072_904);
    let state = choose(state, owner, recipient.raw(), 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveNewYajiao
    );
    assert_eq!(state.resolution.continuation_payload, vec![1, owner.raw()]);
    let state = choose(state, owner, discard_card.raw(), 2);
    let recipient_hand = state.zones.find(ZoneKind::Hand, recipient).unwrap();
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(revealed).unwrap().zone, recipient_hand);
    assert_eq!(state.cards.get(discard_card).unwrap().zone, discard);

    let (state, owner, recipient, revealed, discard_card) = setup(CardKind::Tao, 2_026_072_905);
    let state = choose(state, owner, recipient.raw(), 3);
    assert_eq!(state.resolution.pending_choice_id, 0);
    let recipient_hand = state.zones.find(ZoneKind::Hand, recipient).unwrap();
    let owner_hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    assert_eq!(state.cards.get(revealed).unwrap().zone, recipient_hand);
    assert_eq!(state.cards.get(discard_card).unwrap().zone, owner_hand);
}
