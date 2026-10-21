//! Golden for SHIJI/SIXIANG 吕范「调度」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const MBDIAODU: u32 = 2_440_438_931;
const STD_LVFAN: u32 = 3_319_229_677;
const SHIJI: &[u8] = include_bytes!("../../../../content/original/ported/shiji/shiji.npack.json");
const SIXIANG: &[u8] =
    include_bytes!("../../../../content/original/ported/sixiang/sixiang.npack.json");

fn setup(player_count: u8) -> (GameState, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(SHIJI).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(MBDIAODU as u64),
            kind: CommandKind::StartGame {
                player_count,
                rng_seed: 20_260_718,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let players = engine.state().players.alive_by_seat();
    let owner = players[0];
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(MBDIAODU),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.state().clone(), players)
}

fn equip(state: &mut GameState, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Equip, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn offer(
    state: GameState,
    owner: PlayerId,
) -> (GameState, Result<Vec<EffectRequest>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    );
    (tx.state().clone(), result)
}

fn resolve(state: GameState, selection: u32, key: u64) -> (GameState, PipelineResult) {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections: vec![selection],
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_contains_mbdiaodu_effect_and_dependent_complete_character() {
    let shiji = content_schema::ContentPackage::from_json_bytes(SHIJI).unwrap();
    assert!(shiji.manifest.rule_vm_abi_version >= 198);
    let trigger = shiji
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == MBDIAODU)
        .unwrap();
    assert_eq!(
        (
            trigger.event.as_str(),
            trigger.filter.as_str(),
            trigger.optional,
        ),
        (
            "phase_prepare",
            "owner_is_target_has_movable_equipment",
            true,
        )
    );

    let sixiang = content_schema::ContentPackage::from_json_bytes(SIXIANG).unwrap();
    assert!(sixiang
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "shiji"));
    let lvfan = sixiang
        .character_index
        .iter()
        .find(|character| character.id == STD_LVFAN)
        .unwrap();
    assert_eq!(lvfan.skills, vec![MBDIAODU, 3_414_143_685]);
}

#[test]
fn chooser_moves_equipment_to_only_legal_target_and_source_draws() {
    let (mut state, players) = setup(3);
    let (owner, source, destination) = (players[0], players[1], players[2]);
    let moved = equip(&mut state, source, CardKind::Weapon, 91_001);
    let source_hand = state.hand_count(source);

    let (state, offered) = offer(state, owner);
    assert_eq!(offered, Err(EffectError::ChoiceRequested));
    let (state, accepted) = resolve(state, 1, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::MoveEquipmentThenDraw
    );
    assert_eq!(
        accepted.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    assert_eq!(
        accepted
            .choice
            .as_ref()
            .unwrap()
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        vec![moved.raw()]
    );

    let (state, selected) = resolve(state, moved.raw(), 2);
    assert!(matches!(selected.result, CommandResult::Accepted { .. }));
    assert_eq!(
        selected.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectTargets
    );
    assert_eq!(
        selected.choice.as_ref().unwrap().options[0].token,
        destination.raw()
    );
    let (state, completed) = resolve(state, destination.raw(), 3);
    assert!(matches!(completed.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.cards.get(moved).unwrap().owner, destination);
    assert_eq!(
        state.cards.get(moved).unwrap().zone,
        state.zones.find(ZoneKind::Equip, destination).unwrap()
    );
    assert_eq!(state.hand_count(source), source_hand + 1);
}

#[test]
fn destination_same_slot_is_replaced_before_move() {
    let (mut state, players) = setup(3);
    let (owner, source, destination) = (players[0], players[1], players[2]);
    let moved = equip(&mut state, source, CardKind::Armor, 91_002);
    let replaced = equip(&mut state, destination, CardKind::Armor, 91_003);

    let (state, offered) = offer(state, owner);
    assert_eq!(offered, Err(EffectError::ChoiceRequested));
    let (state, _) = resolve(state, 1, 4);
    let (state, _) = resolve(state, moved.raw(), 5);
    let (state, completed) = resolve(state, destination.raw(), 6);
    assert!(matches!(completed.result, CommandResult::Accepted { .. }));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(replaced).unwrap().zone, discard);
    assert_eq!(state.cards.get(moved).unwrap().owner, destination);
}

#[test]
fn no_third_legal_recipient_means_prepare_trigger_is_not_offered() {
    let (mut two_players, players) = setup(2);
    equip(&mut two_players, players[1], CardKind::Weapon, 91_004);
    let (_, offered) = offer(two_players, players[0]);
    assert_eq!(offered, Ok(Vec::new()));

    let (mut disabled, players) = setup(3);
    equip(&mut disabled, players[1], CardKind::Weapon, 91_005);
    disabled
        .players
        .get_mut(players[2])
        .unwrap()
        .disabled_equip_slots[0] = 1;
    let (_, offered) = offer(disabled, players[0]);
    assert_eq!(offered, Ok(Vec::new()));
}
