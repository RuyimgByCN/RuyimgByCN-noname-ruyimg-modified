//! Golden coverage for SP2「狼灭」and the cross-package DIY 牢段煨 closure.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const LANGMIE: u32 = 3_392_096_402;
const JUNK_DUANWEI: u32 = 3_934_400_008;
const SOURCE: &str = include_str!("../../../content/original/ported/sp2/langmie.skill");
const SP2_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");
const DIY_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/diy/diy.npack.json");

fn started() -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("langmie-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, LANGMIE as u64, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(LANGMIE),
    });
    effects.push(EffectRequest::SetTurnState {
        current: other,
        phase: Phase::Play.to_u8(),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
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

fn record_used(tx: &mut Transaction, player: PlayerId, card: CardId, target: PlayerId) {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::CardUsed {
            player,
            card,
            targets: vec![target],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(tx, effects).unwrap();
}

fn phase_play_ended(tx: &mut Transaction, player: PlayerId) -> Result<(), EffectError> {
    let effects = match dispatch_for_events(
        tx,
        &[DomainEvent::PhaseEnded {
            current: player,
            phase: TurnPhaseTag::Play,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: true,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Ok(effects) => effects,
        Err(err) => return Err(err),
    };
    run_effects(tx, effects).map(|_| ())
}

fn phase_end_started(tx: &mut Transaction, player: PlayerId) -> Result<(), EffectError> {
    let effects = match dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: player,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Ok(effects) => effects,
        Err(err) => return Err(err),
    };
    run_effects(tx, effects).map(|_| ())
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    )
}

#[test]
fn packages_publish_langmie_and_cross_package_character_closure() {
    let sp2 = content_schema::ContentPackage::from_json_bytes(SP2_PACKAGE).unwrap();
    let diy = content_schema::ContentPackage::from_json_bytes(DIY_PACKAGE).unwrap();
    let compiled = content_compiler::compile_source_named("langmie", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 105);
    assert_eq!(sp2.manifest.rule_vm_abi_version, 753);
    assert!(sp2
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == LANGMIE && skill.name == "langmie"));
    let triggers: Vec<_> = sp2
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == LANGMIE)
        .collect();
    assert_eq!(triggers.len(), 2);
    assert!(triggers.iter().all(|trigger| trigger.optional));
    assert!(diy
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "sp2"));
    let character = diy
        .character_index
        .iter()
        .find(|character| character.id == JUNK_DUANWEI)
        .unwrap();
    assert_eq!(character.name_key, "junk_duanwei");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [LANGMIE]);
}

#[test]
fn duplicate_logical_category_in_exact_play_phase_offers_optional_draw() {
    let (mut tx, owner, other) = started();
    let basic = inject(&mut tx, other, CardKind::Sha, 91_001);
    let trick = inject(&mut tx, other, CardKind::TrickWuzhong, 91_002);
    record_used(&mut tx, other, basic, owner);
    record_used(&mut tx, other, trick, owner);
    assert!(!tx.state().turn.play_card_category_repeated(other));
    assert_eq!(phase_play_ended(&mut tx, other), Ok(()));

    tx.state_mut().turn.clear_play_card_category_history();
    let second_basic = inject(&mut tx, other, CardKind::Shan, 91_003);
    record_used(&mut tx, other, basic, owner);
    record_used(&mut tx, other, second_basic, owner);
    assert!(tx.state().turn.play_card_category_repeated(other));
    let before = tx.state().hand_count(owner);
    assert_eq!(
        phase_play_ended(&mut tx, other),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let (state, accepted) = resolve(tx.commit().state, owner, vec![1], 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before + 1);
}

#[test]
fn damage_branch_declines_cleanly_or_discards_before_owner_sourced_damage() {
    let (mut tx, owner, other) = started();
    tx.state_mut().turn.phase = Phase::End;
    tx.state_mut().turn.set_flag(turn_flag::DAMAGE_DEALT, 2);
    let owner_hand = tx.state().hand_count(owner);
    let other_hp = tx.state().players.get(other).unwrap().hp;
    assert_eq!(
        phase_end_started(&mut tx, other),
        Err(EffectError::ChoiceRequested)
    );
    let (state, declined) = resolve(tx.commit().state, owner, vec![0], 2);
    assert!(matches!(declined.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), owner_hand);
    assert_eq!(state.players.get(other).unwrap().hp, other_hp);

    let (mut tx, owner, other) = started();
    tx.state_mut().turn.phase = Phase::End;
    tx.state_mut().turn.set_flag(turn_flag::DAMAGE_DEALT, 2);
    let discard = tx.state().hand_cards(owner)[0];
    let owner_hand = tx.state().hand_count(owner);
    let other_hp = tx.state().players.get(other).unwrap().hp;
    assert_eq!(
        phase_end_started(&mut tx, other),
        Err(EffectError::ChoiceRequested)
    );
    let (state, accepted) = resolve(tx.commit().state, owner, vec![1], 3);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::DiscardOneHandOrEquip
    );
    let offered: Vec<_> = accepted
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert!(offered.contains(&discard.raw()));

    let (state, discarded) = resolve(state, owner, vec![discard.raw()], 4);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), owner_hand - 1);
    assert_eq!(state.players.get(other).unwrap().hp, other_hp - 1);
    assert!(discarded.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            source,
            target,
            amount: 1,
            ..
        } if *source == owner && *target == other
    )));
}
