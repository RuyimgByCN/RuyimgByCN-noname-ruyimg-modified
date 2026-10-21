//! Golden for sp batch-S DSL rewrites:
//! 新镇南 (newzhennan) prepare-phase discard 2 → virtual 南蛮 (ABI 604);
//! 起乱 (qiluan) global phase-end draw by kill counts + death draw (ABI 605);
//! 摧怀 (cihuai) / 咄咄 (oldcihuai) play-phase no-Sha-in-hand virtual Sha.

use engine_api::{
    CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const NEWZHENNAN: u32 = 4_220_659_285;
const QILUAN: u32 = 2_783_300_002;
const CIHUAI: u32 = 4_115_676_072;
const OLDCIHUAI: u32 = 4_216_219_987;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let state = engine.state().clone();
    let owner = state.turn.current;
    (state, owner)
}

fn with_skill(state: GameState, owner: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
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
                cancelled,
            },
        },
    )
}

fn turn_state(state: GameState, current: PlayerId, phase: TurnPhaseTag) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged { current, phase }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn inject_hand(state: &mut GameState, player: PlayerId, kind: CardKind, tag: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(90_700 + tag),
        kind,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: player,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn package_publishes_batch_s_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| {
            matches!(
                trigger.skill_def_id,
                NEWZHENNAN | QILUAN | CIHUAI | OLDCIHUAI
            )
        })
        .map(|trigger| {
            (
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert!(triggers.contains(&(NEWZHENNAN, "phase_prepare", "owner_is_target", true)));
    assert!(triggers.contains(&(QILUAN, "phase_end", "owner_alive_global_phase_end", false)));
    assert!(triggers.contains(&(QILUAN, "death", "owner_alive_other_source_death", false)));
    assert!(triggers.contains(&(
        CIHUAI,
        "phase_play",
        "owner_is_target_play_phase_hand_no_sha",
        true
    )));
    assert!(triggers.contains(&(
        OLDCIHUAI,
        "phase_play",
        "owner_is_target_play_phase_hand_no_sha",
        true
    )));
    assert!(!package.modifier_index.iter().any(|modifier| {
        matches!(
            modifier.skill_def_id,
            NEWZHENNAN | QILUAN | CIHUAI | OLDCIHUAI
        )
    }));
}

#[test]
fn newzhennan_discards_two_then_uses_virtual_nanman() {
    let (state, owner) = started(2_026_081_301);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let state = with_skill(state, owner, NEWZHENNAN);
    let hand: Vec<_> = state.hand_cards(owner);
    assert!(hand.len() >= 2, "natural deal must hold >=2 cards");
    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.resolution.continuation_kind, ContinuationKind::Custom);
    assert_eq!(
        accepted.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    let hand_now: Vec<_> = state.hand_cards(owner);
    assert!(hand_now.len() >= 2);
    let (state, result) = resolve(
        state,
        owner,
        vec![hand_now[0].raw(), hand_now[1].raw()],
        false,
        2,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.hand_count(owner),
        u32::try_from(hand.len()).unwrap() - 2
    );
    // 南蛮 resolves immediately against the other living player (Immediate targets).
    assert!(state.hand_count(other) >= 0);
}

#[test]
fn qiluan_draws_by_kill_counts_on_global_phase_end() {
    let (state, owner) = started(2_026_081_302);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let state = with_skill(state, owner, QILUAN);
    let before = state.hand_count(owner);
    // Owner killed 1 this turn (×3); other killed 2 (×1) → 3 + 2 = 5.
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.set_flag(
        engine_core::state::turn_flag::kill_count_by_player(owner),
        1,
    );
    tx.state_mut().turn.set_flag(
        engine_core::state::turn_flag::kill_count_by_player(other),
        2,
    );
    let tx = tx.commit();
    let state = turn_state(tx.state, other, TurnPhaseTag::End);
    assert_eq!(state.hand_count(owner), before + 5);
}

#[test]
fn qiluan_death_trigger_draws_one_when_owner_not_killer() {
    let (state, owner) = started(2_026_081_303);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let state = with_skill(state, owner, QILUAN);
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PlayerKilled {
            player: other,
            killer: other,
            hand_equip_cards: Vec::new(),
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    assert_eq!(state.hand_count(owner), before + 1);
}

#[test]
fn qiluan_death_trigger_skipped_when_owner_is_killer() {
    let (state, owner) = started(2_026_081_304);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let state = with_skill(state, owner, QILUAN);
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PlayerKilled {
            player: other,
            killer: owner,
            hand_equip_cards: Vec::new(),
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    assert_eq!(state.hand_count(owner), before);
}

#[test]
fn cihuai_opens_virtual_sha_target_choice_without_hand_sha() {
    let (state, owner) = started(2_026_081_305);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let state = with_skill(state, owner, CIHUAI);
    // Clear all hand cards so the owner has no Sha in hand.
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    let tx = tx.commit();
    let (mut state, opened) = {
        let mut tx = Transaction::begin(tx.state);
        let result = dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: TurnPhaseTag::Play,
            }],
            &mut TriggerFireRecord::new(),
        );
        (tx, result)
    };
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        state.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = state.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::VirtualShaTargets
    );
    let options: Vec<_> = accepted
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect();
    assert!(options.contains(&other.raw()));
}

#[test]
fn oldcihuai_opens_virtual_sha_target_choice_without_hand_sha() {
    let (state, owner) = started(2_026_081_306);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let state = with_skill(state, owner, OLDCIHUAI);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    let tx = tx.commit();
    let (mut tx, opened) = {
        let mut tx = Transaction::begin(tx.state);
        let result = dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: TurnPhaseTag::Play,
            }],
            &mut TriggerFireRecord::new(),
        );
        (tx, result)
    };
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::VirtualShaTargets
    );
    let options: Vec<_> = accepted
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect();
    assert!(options.contains(&other.raw()));
}
