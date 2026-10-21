//! Golden coverage for SHENHUA「往烈」and OLD「旧陈到」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{
    card_distance_unlimited_for, card_use_cancelled_for, card_use_unrespondable_for_kind,
};
use engine_core::state::{turn_flag, Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const DRLT_WANGLIE: u32 = 3_599_509_027;
const DRLT_WANGLIE2: u32 = 2_154_023_320;
const OLD_CHENDAO: u32 = 3_212_321_683;
const SHENHUA: &[u8] =
    include_bytes!("../../../../content/original/ported/shenhua/shenhua.npack.json");
const OLD: &[u8] = include_bytes!("../../../../content/original/ported/old/old.npack.json");

fn setup() -> (Transaction, PlayerId, CardId) {
    let package = content_schema::ContentPackage::from_json_bytes(SHENHUA).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(u64::from(DRLT_WANGLIE)),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: u64::from(DRLT_WANGLIE),
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DRLT_WANGLIE),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();

    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(95_001),
        kind: CardKind::TrickWuzhong,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    (tx, owner, card)
}

fn has_skill(tx: &Transaction, owner: PlayerId, skill_def: u32) -> bool {
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == skill_def)
        })
}

#[test]
fn packages_publish_abi115_skill_and_cross_package_character() {
    let shenhua = content_schema::ContentPackage::from_json_bytes(SHENHUA).unwrap();
    assert_eq!(shenhua.manifest.rule_vm_abi_version, 671);
    assert_eq!(shenhua.skill_index.len(), 201);
    let trigger = shenhua
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DRLT_WANGLIE)
        .unwrap();
    assert_eq!(trigger.event, "card_used");
    assert_eq!(
        trigger.filter,
        "owner_source_sha_or_trick_current_play_phase"
    );
    assert!(trigger.optional);
    let modifier = shenhua
        .modifier_index
        .iter()
        .find(|modifier| {
            modifier.skill_def_id == DRLT_WANGLIE && modifier.target == "card_distance_unlimited"
        })
        .unwrap();
    assert_eq!(
        modifier.value_source.as_deref(),
        Some("owner_play_phase_card_use_count_zero")
    );
    assert!(shenhua.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == DRLT_WANGLIE2
            && trigger.event == "phase_play_end"
            && !trigger.optional
    }));

    let old = content_schema::ContentPackage::from_json_bytes(OLD).unwrap();
    assert!(old
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "shenhua"));
    let character = old
        .character_index
        .iter()
        .find(|character| character.id == OLD_CHENDAO)
        .unwrap();
    assert_eq!(character.name_key, "old_chendao");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [DRLT_WANGLIE]);
}

#[test]
fn only_the_first_card_in_the_owners_play_phase_ignores_distance() {
    let (mut tx, owner, _) = setup();
    assert!(tx.state().content.tracks_per_player_play_phase_card_count());
    assert!(card_distance_unlimited_for(tx.state(), owner));

    tx.state_mut()
        .turn
        .set_flag(turn_flag::play_phase_card_used_by_player(owner), 1);
    assert!(!card_distance_unlimited_for(tx.state(), owner));

    tx.state_mut()
        .turn
        .set_flag(turn_flag::play_phase_card_used_by_player(owner), 0);
    tx.state_mut().turn.phase = Phase::End;
    assert!(!card_distance_unlimited_for(tx.state(), owner));
}

#[test]
fn accepting_sha_or_trick_makes_it_unrespondable_then_cleans_up() {
    let (mut tx, owner, card) = setup();
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::CardUsed {
                player: owner,
                card,
                targets: vec![],
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = tx.state().clone();
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let mut tx = Transaction::begin(state);
    assert!(has_skill(&tx, owner, DRLT_WANGLIE2));
    assert!(card_use_cancelled_for(tx.state(), owner));
    assert!(card_use_unrespondable_for_kind(
        tx.state(),
        owner,
        CardKind::TrickWuzhong,
        Suit::Spade,
    ));

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: owner,
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
    assert!(!has_skill(&tx, owner, DRLT_WANGLIE2));
    assert!(!card_use_cancelled_for(tx.state(), owner));
}

#[test]
fn non_sha_basic_and_wrong_phase_do_not_offer_wanglie() {
    let (mut tx, owner, card) = setup();
    tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Shan;
    assert!(dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());

    tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::TrickWuzhong;
    tx.state_mut().turn.phase = Phase::End;
    assert!(dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());
}
