//! Golden coverage for MOBILE 界朱桓「平寇」rewritten from legacy JavaScript.

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

const XINPINGKOU: u32 = 3_441_378_011;
const XIN_ZHUHUAN: u32 = 4_000_463_828;
const FENLI: u32 = 2_848_697_859;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/xinpingkou.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    others: [PlayerId; 3],
}

fn setup(seed: u64) -> Setup {
    let package = content_compiler::compile_source_named("xinpingkou", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
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
        skill_def: SkillDefId::new(XINPINGKOU),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 8;
        participant.max_hp = 8;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    tx.state_mut().turn.record_skipped_phase();
    tx.state_mut().turn.record_skipped_phase();
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        others: [players[1], players[2], players[3]],
    }
}

fn inject_shared(state: &mut GameState, zone_kind: ZoneKind, kind: CardKind, def: u32) -> CardId {
    let zone = state.zones.find(zone_kind, PlayerId::NONE).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: if kind == CardKind::Weapon { 2 } else { 0 },
        zone,
        owner: PlayerId::NONE,
        face_up: zone_kind == ZoneKind::DiscardPile,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    )
}

fn open_phase_end(state: GameState, owner: PlayerId) -> (GameState, PipelineResult) {
    let mut tx = Transaction::begin(state);
    let event = DomainEvent::TurnStateChanged {
        current: owner,
        phase: TurnPhaseTag::End,
    };
    match dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new()) {
        Ok(effects) => assert_eq!(
            run_effects(&mut tx, effects).unwrap_err(),
            EffectError::ChoiceRequested
        ),
        Err(error) => assert_eq!(error, EffectError::ChoiceRequested),
    }
    let committed = tx.commit();
    let new_revision = committed.state.revision;
    let state_hash = committed.state_hash;
    let events = committed.output.events;
    let choice = committed.output.choice;
    (
        committed.state,
        PipelineResult {
            result: CommandResult::Accepted {
                new_revision,
                state_hash,
            },
            events,
            choice,
        },
    )
}

#[test]
fn package_contains_abi124_skill_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 252);
    assert_eq!(package.skill_index.len(), 440);
    assert_eq!(package.character_index.len(), 204);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == XIN_ZHUHUAN)
        .unwrap();
    assert_eq!(character.name_key, "xin_zhuhuan");
    assert_eq!(character.skills, [FENLI, XINPINGKOU]);

    let compiled = content_compiler::compile_source_named("xinpingkou", SOURCE).unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn damages_selected_targets_then_gains_draw_pile_equipment_without_discard_fallback() {
    let mut initial = setup(2_026_072_401);
    let draw_equipment = inject_shared(
        &mut initial.state,
        ZoneKind::DrawPile,
        CardKind::Weapon,
        XINPINGKOU + 1,
    );
    let discard_equipment = inject_shared(
        &mut initial.state,
        ZoneKind::DiscardPile,
        CardKind::Armor,
        XINPINGKOU + 2,
    );
    let (state, confirm) = open_phase_end(initial.state, initial.owner);
    assert_eq!(confirm.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);

    let (state, target_offer) = resolve(state, initial.owner, vec![1], 1);
    let choice = target_offer.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert_eq!((choice.min_count, choice.max_count), (1, 2));

    let first = initial.others[1];
    let second = initial.others[0];
    let first_hp = state.players.get(first).unwrap().hp;
    let second_hp = state.players.get(second).unwrap().hp;
    let (state, resolved) = resolve(state, initial.owner, vec![first.raw(), second.raw()], 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(first).unwrap().hp, first_hp - 1);
    assert_eq!(state.players.get(second).unwrap().hp, second_hp - 1);
    let damage_order: Vec<_> = resolved
        .events
        .iter()
        .filter_map(|event| match event {
            DomainEvent::DamageDealt { source, target, .. } if *source == initial.owner => {
                Some(*target)
            }
            _ => None,
        })
        .collect();
    assert_eq!(damage_order, [first, second]);
    let hand = state.zones.find(ZoneKind::Hand, initial.owner).unwrap();
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(draw_equipment).unwrap().zone, hand);
    assert_eq!(state.cards.get(discard_equipment).unwrap().zone, discard);

    let mut no_draw_equipment = setup(2_026_072_402);
    let draw = no_draw_equipment
        .state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let draw_cards = no_draw_equipment
        .state
        .zones
        .get(draw)
        .unwrap()
        .cards
        .clone();
    for card in draw_cards {
        let state = no_draw_equipment.state.cards.get_mut(card).unwrap();
        if state.kind.is_equip() {
            state.kind = CardKind::Sha;
            state.weapon_range = 0;
        }
    }
    let discard_only = inject_shared(
        &mut no_draw_equipment.state,
        ZoneKind::DiscardPile,
        CardKind::Weapon,
        XINPINGKOU + 3,
    );
    let before = no_draw_equipment.state.hand_count(no_draw_equipment.owner);
    let mut tx = Transaction::begin(no_draw_equipment.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::GainEquipmentFromDrawPile {
            player: no_draw_equipment.owner,
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    assert_eq!(state.hand_count(no_draw_equipment.owner), before);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(discard_only).unwrap().zone, discard);
}
