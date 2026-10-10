//! Golden coverage for 标夏侯霸「豹变」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STDBAOBIAN: u32 = 3_406_574_661;
const STD_XIAHOUBA: u32 = 2_845_782_138;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdbaobian.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn setup(card_kind: Option<CardKind>) -> (Transaction, PlayerId, PlayerId, Option<CardId>) {
    let package = content_compiler::compile_source_named("stdbaobian-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 20_260_726, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(
        players
            .iter()
            .map(|player| EffectRequest::DiscardAllHandEquipJudge { player: *player }),
    );
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDBAOBIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in players {
        let participant = tx.state_mut().players.get_mut(player).unwrap();
        participant.hp = 4;
        participant.max_hp = 4;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let card = card_kind.map(|kind| {
        let hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(194_001),
            kind,
            suit: Suit::Heart,
            rank: 7,
            weapon_range: 0,
            zone: hand,
            owner: target,
            face_up: false,
        });
        tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
        card
    });
    (tx, owner, target, card)
}

fn phase_play(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::Play,
    }
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
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
    )
}

fn offer(mut tx: Transaction, owner: PlayerId) -> (GameState, PipelineResult) {
    assert_eq!(
        dispatch_for_events(&mut tx, &[phase_play(owner)], &mut TriggerFireRecord::new(),),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    assert_eq!(committed.output.choice.unwrap().kind, ChoiceKind::Confirm);
    resolve(committed.state, owner, vec![1], 1)
}

#[test]
fn package_publishes_abi194_skill_and_complete_std_xiahouba() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STDBAOBIAN)
        .unwrap();
    assert_eq!(trigger.event, "phase_play");
    assert_eq!(
        trigger.filter,
        "owner_is_target_any_alive_self_discardable_hand"
    );
    assert!(trigger.optional);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_XIAHOUBA)
        .unwrap();
    assert_eq!(character.name_key, "std_xiahouba");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [STDBAOBIAN]);
}

#[test]
fn basic_discard_follows_hp_loss_with_distance_ignoring_virtual_sha() {
    let (tx, owner, target, card) = setup(Some(CardKind::Shan));
    let card = card.unwrap();
    let (state, target_choice) = offer(tx, owner);
    assert!(matches!(
        target_choice.result,
        CommandResult::Accepted { .. }
    ));
    let choice = target_choice.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert_eq!(choice.observer, owner);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [target.raw()]
    );

    let (state, discard_choice) = resolve(state, owner, vec![target.raw()], 2);
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    let choice = discard_choice.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(choice.observer, target);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [card.raw()]
    );

    let (state, resolved) = resolve(state, target, vec![card.raw()], 3);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, 3);
    assert_eq!(
        state.cards.get(card).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
    assert!(resolved.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            source,
            target: damaged,
            amount: 1,
            ..
        } if *source == owner && *damaged == target
    )));
}

#[test]
fn non_basic_discard_suppresses_sha_and_empty_table_suppresses_trigger() {
    let (tx, owner, target, card) = setup(Some(CardKind::Other));
    let card = card.unwrap();
    let (state, _) = offer(tx, owner);
    let (state, _) = resolve(state, owner, vec![target.raw()], 2);
    let (state, resolved) = resolve(state, target, vec![card.raw()], 3);
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert_eq!(state.players.get(target).unwrap().hp, 4);
    assert!(!resolved.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            source,
            target: damaged,
            ..
        } if *source == owner && *damaged == target
    )));

    let (mut empty, owner, _, _) = setup(None);
    assert!(dispatch_for_events(
        &mut empty,
        &[phase_play(owner)],
        &mut TriggerFireRecord::new(),
    )
    .is_ok());
    assert_eq!(empty.state().resolution.pending_choice_id, 0);
}
