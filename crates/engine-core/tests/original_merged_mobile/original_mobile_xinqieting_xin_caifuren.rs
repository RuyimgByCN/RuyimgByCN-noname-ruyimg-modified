//! Golden coverage for MOBILE 界蔡夫人「窃听」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, ChoiceRequest, Command, CommandKind, CommandResult,
    DomainEvent, ExpectedRevision, IdempotencyKey, ObserverKind, PlayerId, SkillDefId, Suit,
    TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::{canonical_state_hash, project_observer};
use rule_api::EffectRequest;

const XIN_CAIFUREN: u32 = 3_572_525_059;
const XINQIETING: u32 = 3_722_360_517;
const XIANZHOU: u32 = 4_217_889_451;
const SOURCE: &str = include_str!("../../../../content/original/ported/mobile/xinqieting.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    ending: PlayerId,
}

fn setup(seed: u64) -> Setup {
    let package = content_compiler::compile_source_named("xinqieting-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let ending = players[1];
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
        skill_def: SkillDefId::new(XINQIETING),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = ending;
    tx.state_mut().turn.phase = Phase::End;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        ending,
    }
}

fn inject_card(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    def: u32,
    kind: CardKind,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: if kind == CardKind::Weapon { 2 } else { 0 },
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn trigger(
    state: GameState,
    ending: PlayerId,
    current_dealt_damage_other: bool,
) -> (GameState, Option<ChoiceRequest>) {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: ending,
            phase: TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: true,
            current_dealt_damage_other,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let result = run_effects(&mut tx, effects);
    assert!(matches!(result, Ok(_) | Err(EffectError::ChoiceRequested)));
    let committed = tx.commit();
    (committed.state, committed.output.choice)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
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
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_abi323_skill_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 323);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == XINQIETING && skill.name == "xinqieting"));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == XINQIETING)
        .unwrap();
    assert_eq!(trigger.event, "turn_ended");
    assert_eq!(
        trigger.filter,
        "owner_alive_other_turn_ended_no_damage_other"
    );
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == XIN_CAIFUREN)
        .unwrap();
    assert_eq!(character.name_key, "xin_caifuren");
    assert_eq!(character.sex, "female");
    assert_eq!(character.group, "qun");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [XINQIETING, XIANZHOU]);

    let compiled = content_compiler::compile_source_named("xinqieting", SOURCE).unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn damage_history_blocks_while_draw_and_cancel_remain_available() {
    let blocked = setup(2_026_073_101);
    let (state, choice) = trigger(blocked.state, blocked.ending, true);
    assert!(choice.is_none());
    assert!(!state.has_pending_choice());

    let draw = setup(2_026_073_102);
    let hand = draw.state.hand_count(draw.owner);
    let (state, choice) = trigger(draw.state, draw.ending, false);
    let choice = choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::Custom);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1]
    );
    let (state, result) = resolve(state, draw.owner, vec![1], false, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(draw.owner), hand + 1);

    let cancel = setup(2_026_073_103);
    let (state, choice) = trigger(cancel.state, cancel.ending, false);
    assert!(choice.unwrap().allow_cancel);
    let (state, result) = resolve(state, cancel.owner, Vec::new(), true, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
}

#[test]
fn hand_branch_reveals_only_two_and_rejects_an_uninspected_card_atomically() {
    let mut setup = setup(2_026_073_104);
    let cards: Vec<_> = [99_101, 99_102, 99_103]
        .into_iter()
        .map(|def| {
            inject_card(
                &mut setup.state,
                setup.ending,
                ZoneKind::Hand,
                def,
                CardKind::Other,
            )
        })
        .collect();
    let (state, choice) = trigger(setup.state, setup.ending, false);
    assert_eq!(
        choice
            .unwrap()
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    let (state, branch) = resolve(state, setup.owner, vec![2], false, 3);
    assert!(matches!(branch.result, CommandResult::Accepted { .. }));
    let choice = branch.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(choice.observer, setup.owner);
    assert_eq!((choice.min_count, choice.max_count), (1, 1));
    assert!(!choice.allow_cancel);
    let inspected: Vec<_> = choice.options.iter().map(|option| option.token).collect();
    assert_eq!(inspected.len(), 2);

    let view = project_observer(&state, ObserverKind::Player(setup.owner));
    assert!(inspected.iter().all(|card| {
        view.known_cards
            .iter()
            .any(|known| known.id.raw() == *card && known.def.is_some())
    }));
    let hidden = cards
        .iter()
        .find(|card| !inspected.contains(&card.raw()))
        .unwrap();
    assert!(view.known_cards.iter().all(|known| known.id != *hidden));

    let before = canonical_state_hash(&state);
    let (unchanged, rejected) = resolve(state.clone(), setup.owner, vec![hidden.raw()], false, 4);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&unchanged), before);

    let gained = CardId::new(inspected[0]);
    let (state, result) = resolve(state, setup.owner, vec![gained.raw()], false, 5);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(gained).unwrap().owner, setup.owner);
    assert_eq!(state.hand_count(setup.ending), 2);
}

#[test]
fn compatible_equipment_can_move_to_the_owner() {
    let mut setup = setup(2_026_073_105);
    let weapon = inject_card(
        &mut setup.state,
        setup.ending,
        ZoneKind::Equip,
        99_111,
        CardKind::Weapon,
    );
    let (state, choice) = trigger(setup.state, setup.ending, false);
    assert_eq!(
        choice
            .unwrap()
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    let (state, branch) = resolve(state, setup.owner, vec![0], false, 6);
    assert!(matches!(branch.result, CommandResult::Accepted { .. }));
    assert_eq!(branch.choice.unwrap().kind, ChoiceKind::SelectCards);
    let (state, result) = resolve(state, setup.owner, vec![weapon.raw()], false, 7);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let owner_equip = state.zones.find(ZoneKind::Equip, setup.owner).unwrap();
    assert_eq!(state.cards.get(weapon).unwrap().zone, owner_equip);
    assert_eq!(state.cards.get(weapon).unwrap().owner, setup.owner);
}
