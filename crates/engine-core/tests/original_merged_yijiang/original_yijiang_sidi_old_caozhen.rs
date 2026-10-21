//! Golden coverage for 旧曹真「司敌」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::query::sha_use_limit_for;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OLD_CAOZHEN: u32 = 2_623_285_537;
const SIDI: u32 = 3_456_311_904;
const SIDI3: u32 = 3_364_392_152;
const SOURCE: &str = concat!(
    include_str!("../../../../content/original/ported/yijiang/sidi.skill"),
    "\n",
    include_str!("../../../../content/original/ported/yijiang/sidi3.skill")
);
const YIJIANG: &[u8] =
    include_bytes!("../../../../content/original/ported/yijiang/yijiang.npack.json");
const OLD: &[u8] = include_bytes!("../../../../content/original/ported/old/old.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId, CardId) {
    let package = content_compiler::compile_source_named("sidi-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 287);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_729_21, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SIDI),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let shan = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_701),
        kind: CardKind::Shan,
        suit: Suit::Heart,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(shan);
    (tx.commit().state, owner, target, shan)
}

fn dispatch(state: GameState, events: &[DomainEvent]) -> GameState {
    let mut tx = Transaction::begin(state);
    assert_eq!(
        dispatch_for_events(&mut tx, events, &mut TriggerFireRecord::new()),
        Err(EffectError::ChoiceRequested)
    );
    tx.commit().state
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
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
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    state
}

#[test]
fn shan_builds_the_public_pile_and_one_chosen_card_reduces_sha_until_play_ends() {
    let (mut state, owner, target, shan) = setup();
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let first = *state.zones.get(draw).unwrap().cards.last().unwrap();
    for key in 1..=2 {
        state = dispatch(
            state,
            &[DomainEvent::CardUsed {
                player: owner,
                card: shan,
                targets: Vec::new(),
            }],
        );
        state = resolve(state, owner, vec![1], key);
    }
    let pile = state.skill_pile_cards(owner, SIDI);
    assert_eq!(pile.len(), 2);
    assert!(pile.contains(&first));

    let base = sha_use_limit_for(&state, target);
    state.turn.current = target;
    state.turn.phase = Phase::Play;
    state = dispatch(
        state,
        &[DomainEvent::TurnStateChanged {
            current: target,
            phase: TurnPhaseTag::Play,
        }],
    );
    state = resolve(state, owner, vec![1], 3);
    assert!(state.has_pending_choice());
    state = resolve(state, owner, vec![pile[0].raw()], 4);

    assert_eq!(state.skill_pile_cards(owner, SIDI).len(), 1);
    assert!(state
        .players
        .get(target)
        .unwrap()
        .marks
        .contains(&(SIDI3, 1)));
    assert_eq!(sha_use_limit_for(&state, target), base.saturating_sub(1));

    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: target,
            phase: TurnPhaseTag::Play,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(sha_use_limit_for(tx.state(), target), base);
    assert!(!tx
        .state()
        .players
        .get(target)
        .unwrap()
        .marks
        .contains(&(SIDI3, 1)));
}

#[test]
fn packages_publish_sidi_and_old_caozhen() {
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    assert_eq!(yijiang.manifest.rule_vm_abi_version, 637);
    assert!(yijiang
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == SIDI && skill.name == "sidi"));
    let old = content_schema::ContentPackage::from_json_bytes(OLD).unwrap();
    assert_eq!(
        old.character_index
            .iter()
            .find(|character| character.id == OLD_CAOZHEN)
            .unwrap()
            .skills,
        [SIDI]
    );
}
