//! Golden coverage for 标留赞「奋音」rewritten from legacy JavaScript semantics.

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

const STD_LIUZAN: u32 = 4_100_887_402;
const STDFENYIN: u32 = 2_225_023_788;
const STDFENYIN_DISCARD: u32 = 3_965_200_877;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdfenyin.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
}

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> Option<engine_api::SkillId> {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
}

fn setup(seed: u64) -> Setup {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
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
        skill_def: SkillDefId::new(STDFENYIN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Judge;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
    }
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    owner: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled: false,
        },
        key,
    )
}

fn enter_draw(state: GameState, owner: PlayerId, accept: bool) -> (GameState, PipelineResult) {
    let (state, offered) = submit(state, owner, CommandKind::EndPhase, 1);
    let choice = offered.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::Confirm);
    resolve(state, owner, vec![u32::from(accept)], 2)
}

fn inject_event_card(state: &mut GameState, def: u32, suit: Suit, rank: u8) -> CardId {
    let zone = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner: PlayerId::NONE,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn dispatch_use(
    state: GameState,
    player: PlayerId,
    card: CardId,
) -> (GameState, Option<engine_api::ChoiceRequest>) {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player,
            card,
            targets: Vec::new(),
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let result = run_effects(&mut tx, effects);
    assert!(result.is_ok() || result == Err(EffectError::ChoiceRequested));
    let committed = tx.commit();
    (committed.state, committed.output.choice)
}

fn end_owner_turn(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: owner,
            phase: TurnPhaseTag::End,
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
    tx.commit().state
}

#[test]
fn package_publishes_abi131_skills_and_complete_std_liuzan() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    let draw = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STDFENYIN && trigger.event == "phase_draw")
        .unwrap();
    assert!(draw.optional);
    let discard = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STDFENYIN_DISCARD && trigger.event == "card_used")
        .unwrap();
    assert_eq!(
        discard.filter,
        "owner_source_repeats_color_has_discardable_hand_equip"
    );
    assert!(!discard.optional);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == STDFENYIN_DISCARD
            && trigger.event == "turn_ended"
            && trigger.filter == "owner_is_target"
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_LIUZAN)
        .unwrap();
    assert_eq!(character.name_key, "std_liuzan");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wu");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [STDFENYIN]);

    let compiled = content_compiler::compile_source_named("stdfenyin", SOURCE).unwrap();
    for expected in &compiled.modules {
        let actual = package
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn accepting_draws_two_extra_then_repeated_color_forces_one_discard() {
    let setup = setup(2_026_072_408);
    let before = setup.state.hand_count(setup.owner);
    let (mut state, accepted) = enter_draw(setup.state, setup.owner, true);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), before + 4);
    assert!(skill_id(&state, setup.owner, STDFENYIN_DISCARD).is_some());
    state.turn.phase = Phase::Play;

    let red_one = inject_event_card(&mut state, 131_001, Suit::Heart, 2);
    let red_two = inject_event_card(&mut state, 131_002, Suit::Diamond, 3);
    let black_one = inject_event_card(&mut state, 131_003, Suit::Spade, 4);
    let black_two = inject_event_card(&mut state, 131_004, Suit::Club, 5);
    let colorless_one = inject_event_card(&mut state, 131_005, Suit::None, 6);
    let colorless_two = inject_event_card(&mut state, 131_006, Suit::None, 7);

    let (state_after_first, choice) = dispatch_use(state, setup.owner, red_one);
    assert!(choice.is_none());
    let (pending, choice) = dispatch_use(state_after_first, setup.owner, red_two);
    assert_eq!(pending.resolution.pending_observer, setup.owner.raw());
    let choice = choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(choice.min_count, 1);
    assert_eq!(choice.max_count, 1);
    let cost = CardId::new(choice.options[0].token);
    let hand_before = pending.hand_count(setup.owner);
    let (resolved, result) = resolve(pending, setup.owner, vec![cost.raw()], 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(resolved.hand_count(setup.owner), hand_before - 1);
    assert!(!resolved.has_pending_choice());

    let (state_after_black, choice) = dispatch_use(resolved, setup.owner, black_one);
    assert!(choice.is_none());
    let (pending, choice) = dispatch_use(state_after_black, setup.owner, black_two);
    let cost = choice.unwrap().options[0].token;
    let (resolved, result) = resolve(pending, setup.owner, vec![cost], 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));

    let (state_after_colorless, choice) = dispatch_use(resolved, setup.owner, colorless_one);
    assert!(choice.is_none());
    let (state, choice) = dispatch_use(state_after_colorless, setup.owner, colorless_two);
    assert!(choice.is_none());

    let expired = end_owner_turn(state, setup.owner);
    assert!(skill_id(&expired, setup.owner, STDFENYIN_DISCARD).is_none());
}

#[test]
fn declining_draws_normally_and_outside_owner_turn_never_discards() {
    let declined = setup(2_026_072_409);
    let before = declined.state.hand_count(declined.owner);
    let (state, result) = enter_draw(declined.state, declined.owner, false);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(declined.owner), before + 2);
    assert!(skill_id(&state, declined.owner, STDFENYIN_DISCARD).is_none());

    let accepted = setup(2_026_072_410);
    let (mut state, result) = enter_draw(accepted.state, accepted.owner, true);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let other = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != accepted.owner)
        .unwrap();
    state.turn.current = other;
    state.turn.phase = Phase::Play;
    let red_one = inject_event_card(&mut state, 131_011, Suit::Heart, 8);
    let red_two = inject_event_card(&mut state, 131_012, Suit::Diamond, 9);
    let (state, choice) = dispatch_use(state, accepted.owner, red_one);
    assert!(choice.is_none());
    let (_, choice) = dispatch_use(state, accepted.owner, red_two);
    assert!(choice.is_none());
}
