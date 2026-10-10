//! Golden coverage for 标司马昭「昭心」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STD_SIMAZHAO: u32 = 2_365_843_011;
const STDZHAOXIN: u32 = 2_955_343_028;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdzhaoxin.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn phase_prepare(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::Prepare,
    }
}

fn setup() -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("stdzhaoxin-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_722, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
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
        skill_def: SkillDefId::new(STDZHAOXIN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    (tx, owner, target)
}

fn inject(tx: &mut Transaction, owner: PlayerId, def: u32, suit: Suit) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(state: GameState, actor: PlayerId, selection: PlayerId) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_publishes_exact_skill_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == STDZHAOXIN && skill.name == "stdzhaoxin" && skill.locked
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_SIMAZHAO)
        .unwrap();
    assert_eq!(character.name_key, "std_simazhao");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wei");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [STDZHAOXIN]);
}

#[test]
fn non_uniform_hand_is_revealed_without_damage_choice() {
    let (mut tx, owner, target) = setup();
    let red = inject(&mut tx, owner, 96_001, Suit::Heart);
    let black = inject(&mut tx, owner, 96_002, Suit::Spade);
    let target_hp = tx.state().players.get(target).unwrap().hp;
    let effects = dispatch_for_events(
        &mut tx,
        &[phase_prepare(owner)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let events = run_effects(&mut tx, effects).unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().players.get(target).unwrap().hp, target_hp);
    assert!(events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsRevealed { cards, to }
            if cards == &vec![red, black] && to.is_empty()
    )));
}

#[test]
fn uniform_hand_forces_any_living_target_and_allows_self_damage() {
    let (mut tx, owner, target) = setup();
    let red_a = inject(&mut tx, owner, 96_011, Suit::Heart);
    let red_b = inject(&mut tx, owner, 96_012, Suit::Diamond);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let effects = dispatch_for_events(
        &mut tx,
        &[phase_prepare(owner)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseOtherDamage
    );
    let committed = tx.commit();
    assert!(committed.output.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsRevealed { cards, to }
            if cards == &vec![red_a, red_b] && to.is_empty()
    )));
    let choice = committed.output.choice.unwrap();
    assert!(!choice.allow_cancel);
    assert_eq!((choice.min_count, choice.max_count), (1, 1));
    let options: Vec<_> = choice
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert!(options.contains(&owner.raw()));
    assert!(options.contains(&target.raw()));

    let state = resolve(committed.state, owner, owner);
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);
}
