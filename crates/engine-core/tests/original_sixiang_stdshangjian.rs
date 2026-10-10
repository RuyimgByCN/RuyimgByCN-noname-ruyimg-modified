//! Golden coverage for 标王元姬「尚俭」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STD_WANGYUANJI: u32 = 3_012_016_618;
const STDQIANCHONG: u32 = 2_607_738_803;
const STDSHANGJIAN: u32 = 4_047_696_367;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdshangjian.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn phase_end(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::End,
    }
}

fn setup(hp: i32) -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("stdshangjian-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 201);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_726, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, other) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDSHANGJIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.clear_per_player_card_counts();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    tx.state_mut().players.get_mut(owner).unwrap().hp = hp;
    (tx, owner, other)
}

fn inject(tx: &mut Transaction, owner: PlayerId, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn move_card(tx: &mut Transaction, card: CardId, from: u32, to: u32) {
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            from: Some(engine_api::ZoneId::new(from)),
            to: engine_api::ZoneId::new(to),
        }],
    )
    .unwrap();
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    (state, result)
}

#[test]
fn committed_package_publishes_complete_character_and_abi201_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == STDSHANGJIAN
            && trigger.event == "phase_end"
            && trigger.filter == "owner_is_target_lost_cards_at_most_hp_gainable"
            && trigger.optional
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_WANGYUANJI)
        .unwrap();
    assert_eq!(character.skills, [STDQIANCHONG, STDSHANGJIAN]);
}

#[test]
fn total_loss_threshold_and_discard_provenance_offer_only_gainable_cards() {
    let (mut tx, owner, other) = setup(3);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let other_hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let gainable = inject(&mut tx, owner, 98_001);
    let transferred = inject(&mut tx, owner, 98_002);
    move_card(&mut tx, gainable, owner_hand.raw(), discard.raw());
    move_card(&mut tx, transferred, owner_hand.raw(), other_hand.raw());
    assert_eq!(tx.state().turn.card_loss_count(owner), 2);

    assert_eq!(
        dispatch_for_events(&mut tx, &[phase_end(owner)], &mut TriggerFireRecord::new(),),
        Err(EffectError::ChoiceRequested)
    );
    let (state, offered) = resolve(tx.commit().state, owner, vec![1], 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseGainOwnLostDiscardCardThisTurn
    );
    let choice = offered.choice.unwrap();
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        vec![gainable.raw()]
    );
    let (state, _) = resolve(state, owner, vec![gainable.raw()], 2);
    assert!(!state.has_pending_choice());
    assert_eq!(state.cards.get(gainable).unwrap().zone, owner_hand);
    assert_eq!(state.cards.get(transferred).unwrap().zone, other_hand);
}

#[test]
fn repeat_loss_counts_twice_but_offers_one_card_and_excess_loss_suppresses_trigger() {
    let (mut tx, owner, _) = setup(2);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = inject(&mut tx, owner, 98_003);
    move_card(&mut tx, card, hand.raw(), discard.raw());
    move_card(&mut tx, card, discard.raw(), hand.raw());
    move_card(&mut tx, card, hand.raw(), discard.raw());
    assert_eq!(tx.state().turn.card_loss_count(owner), 2);
    assert_eq!(
        tx.state().turn.lost_cards_for(owner).collect::<Vec<_>>(),
        vec![card]
    );
    assert_eq!(
        dispatch_for_events(&mut tx, &[phase_end(owner)], &mut TriggerFireRecord::new(),),
        Err(EffectError::ChoiceRequested)
    );

    let (mut excessive, owner, _) = setup(1);
    let discard = excessive
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let hand = excessive.state().zones.find(ZoneKind::Hand, owner).unwrap();
    for def in [98_004, 98_005] {
        let card = inject(&mut excessive, owner, def);
        move_card(&mut excessive, card, hand.raw(), discard.raw());
    }
    assert!(dispatch_for_events(
        &mut excessive,
        &[phase_end(owner)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());
}
