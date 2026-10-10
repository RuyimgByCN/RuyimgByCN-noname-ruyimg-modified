//! Golden coverage for OFFLINE 崔廉「贪赂」rewritten from legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, ChoiceRequest, Command, CommandKind, CommandResult,
    DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::{choose_command, Engine, Snapshot};
use rule_api::EffectRequest;

const CUILIAN: u32 = 4_231_642_947;
const TANLU: u32 = 4_005_608_784;
const JUBIAN: u32 = 3_008_850_817;
const PROTECT_EQUIP_GAIN: u32 = 4_200_003_031;
const GIVER_DIES: u32 = 4_200_003_032;
const SOURCE_DIES: u32 = 4_200_003_033;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/xktanlu.skill");
const JUBIAN_SOURCE: &str = include_str!("../../../content/original/ported/offline/xkjubian.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");
const HELPERS: &str = r#"
skill protect_equip_gain(4200003031) locked {
  modifier card_gain_protected(equip) override 1;
}

skill giver_dies(4200003032) locked {
  trigger cards_given owner_alive_gave_cards_to_other_alive {
    effect lose_hp(self, 99);
  }
}

skill source_dies(4200003033) locked {
  trigger damage owner_is_source {
    effect lose_hp(self, 99);
  }
}
"#;

#[derive(Clone)]
struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    target_cards: Vec<CardId>,
    owner_cards: Vec<CardId>,
}

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_500 + u32::from(rank)),
        kind,
        suit: Suit::Club,
        rank,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(
    owner_hp: i32,
    target_hp: i32,
    target_kinds: &[CardKind],
    owner_kinds: &[CardKind],
    owner_helpers: &[u32],
    target_helpers: &[u32],
) -> Setup {
    let package = compile_source_named(
        "offline-xktanlu-test",
        &format!("{SOURCE}\n{JUBIAN_SOURCE}\n{HELPERS}"),
    )
    .unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_073_003, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
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
        skill_def: SkillDefId::new(TANLU),
    });
    effects.extend(
        owner_helpers
            .iter()
            .map(|definition| EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(*definition),
            }),
    );
    effects.extend(
        target_helpers
            .iter()
            .map(|definition| EffectRequest::AddSkill {
                player: target,
                skill_def: SkillDefId::new(*definition),
            }),
    );
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = owner_hp;
    tx.state_mut().players.get_mut(target).unwrap().hp = target_hp;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == TANLU)
        .unwrap();
    let mut state = tx.commit().state;
    let target_cards = target_kinds
        .iter()
        .enumerate()
        .map(|(index, kind)| inject(&mut state, target, *kind, index as u8 + 1))
        .collect();
    let owner_cards = owner_kinds
        .iter()
        .enumerate()
        .map(|(index, kind)| inject(&mut state, owner, *kind, index as u8 + 8))
        .collect();
    Setup {
        state,
        owner,
        target,
        skill,
        target_cards,
        owner_cards,
    }
}

fn begin_turn(setup: &Setup) -> (GameState, ChoiceRequest, Vec<DomainEvent>) {
    let mut tx = Transaction::begin(setup.state.clone());
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::BeginTurn {
                player: setup.target
            }]
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let committed = tx.commit();
    (
        committed.state,
        committed.output.choice.unwrap(),
        committed.output.events,
    )
}

fn command(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    command(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn accept(setup: &Setup, key: u64) -> (GameState, PipelineResult) {
    let (state, choice, _) = begin_turn(setup);
    assert_eq!(choice.observer, setup.owner);
    assert_eq!(choice.kind, ChoiceKind::Confirm);
    resolve(state, setup.owner, vec![1], false, key)
}

fn pass_dying(
    mut state: GameState,
    mut result: PipelineResult,
    mut key: u64,
) -> (GameState, Vec<DomainEvent>) {
    let mut events = result.events;
    while state.has_pending_choice()
        && state.resolution.continuation_kind == engine_core::ContinuationKind::DyingSave
    {
        let actor = PlayerId::new(state.resolution.pending_observer);
        (state, result) = resolve(state, actor, vec![0], false, key);
        key += 1;
        events.extend(result.events);
    }
    (state, events)
}

#[test]
fn package_module_character_and_ids_are_stable() {
    let compiled = compile_source_named("offline-xktanlu", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 303);
    let trigger = &compiled.trigger_index[0];
    assert_eq!(trigger.skill_def_id, TANLU);
    assert_eq!(trigger.event, "phase_begin");
    assert_eq!(trigger.filter, "owner_alive_not_target");
    assert!(trigger.optional);
    assert_eq!(compiled.modules[0].code, [83, 84, 90, 34, 1, 1]);
    assert_eq!(rule_vm::bytecode::effect_tag::RESOLVE_TANLU, 290);
    assert_eq!(engine_core::ContinuationKind::ResolveTanlu.tag(), 208);

    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert_eq!(package.manifest.logic_hash, package.compute_logic_hash());
    assert_eq!(package.skill_index.len(), 838);
    assert_eq!(package.character_index.len(), 382);
    assert_eq!(package.modules.len(), 724);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == CUILIAN)
        .unwrap();
    assert_eq!(character.name_key, "xk_cuilian");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "qun");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [TANLU, JUBIAN]);
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}

#[test]
fn zero_or_gap_above_original_hand_forces_damage_then_discard() {
    for (owner_hp, target_hp, target_cards) in [(2, 2, 2), (1, 4, 2)] {
        let fixture = setup(
            owner_hp,
            target_hp,
            &vec![CardKind::Other; target_cards],
            &[CardKind::Other],
            &[],
            &[],
        );
        let target = fixture.target;
        let owner_card = fixture.owner_cards[0];
        let hp = fixture.state.players.get(target).unwrap().hp;
        let (state, result) = accept(&fixture, 10 + target_cards as u64);
        assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
        assert_eq!(
            state.resolution.continuation_kind,
            engine_core::ContinuationKind::ResolveTanlu
        );
        assert_eq!(result.choice.as_ref().unwrap().observer, target);
        let (state, result) = resolve(state, target, vec![owner_card.raw()], false, 20);
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        assert!(!state.has_pending_choice());
        let discard = state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        assert_eq!(state.cards.get(owner_card).unwrap().zone, discard);
    }
}

#[test]
fn gift_branch_rejects_forgery_then_moves_exactly_x_as_one_give_batch() {
    let fixture = setup(
        3,
        1,
        &[CardKind::Other, CardKind::Other, CardKind::Other],
        &[],
        &[],
        &[],
    );
    let owner = fixture.owner;
    let target = fixture.target;
    let cards = fixture.target_cards.clone();
    let (state, result) = accept(&fixture, 30);
    assert_eq!(result.choice.as_ref().unwrap().kind, ChoiceKind::Custom);

    let (state, result) = resolve(state, owner, vec![0], false, 31);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    let (state, result) = resolve(state, target, Vec::new(), true, 32);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    let (state, result) = resolve(state, target, vec![2], false, 33);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    let (state, result) = resolve(state, target, vec![0], false, 34);
    assert_eq!(
        result.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );

    let (state, result) = resolve(
        state,
        target,
        vec![cards[0].raw(), cards[0].raw()],
        false,
        35,
    );
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    let selected = vec![cards[0].raw(), cards[1].raw()];
    let (state, result) = resolve(state, target, selected, false, 36);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert!(cards[..2]
        .iter()
        .all(|card| state.cards.get(*card).unwrap().owner == owner));
    let moved: Vec<_> = result
        .events
        .iter()
        .filter_map(|event| match event {
            DomainEvent::CardsMoved { cards, .. } if cards == &fixture.target_cards[..2] => {
                Some(cards.clone())
            }
            _ => None,
        })
        .collect();
    assert_eq!(moved, [fixture.target_cards[..2].to_vec()]);
}

#[test]
fn protected_cards_force_a_smaller_gift_and_nested_giver_death_finishes() {
    let fixture = setup(
        3,
        1,
        &[CardKind::Weapon, CardKind::Weapon, CardKind::Other],
        &[],
        &[],
        &[PROTECT_EQUIP_GAIN, GIVER_DIES],
    );
    let owner = fixture.owner;
    let target = fixture.target;
    let given = fixture.target_cards[2];
    let (state, _) = accept(&fixture, 40);
    let (state, result) = resolve(state, target, vec![0], false, 41);
    let (state, events) = pass_dying(state, result, 42);
    assert_eq!(state.cards.get(given).unwrap().owner, owner);
    assert!(!state.players.get(target).unwrap().alive);
    let moved = events
        .iter()
        .position(
            |event| matches!(event, DomainEvent::CardsMoved { cards, .. } if cards == &[given]),
        )
        .unwrap();
    let killed = events
        .iter()
        .position(
            |event| matches!(event, DomainEvent::PlayerKilled { player, .. } if *player == target),
        )
        .unwrap();
    assert!(moved < killed);
}

#[test]
fn jubian_prevents_damage_before_tanlu_reloads_the_discard_choice() {
    let fixture = setup(
        3,
        1,
        &[CardKind::Other, CardKind::Other],
        &[CardKind::Other],
        &[],
        &[JUBIAN],
    );
    let target = fixture.target;
    let target_card = fixture.target_cards[0];
    let owner_card = fixture.owner_cards[0];
    let hp = fixture.state.players.get(target).unwrap().hp;
    let (state, _) = accept(&fixture, 50);
    let (state, result) = resolve(state, target, vec![1], false, 51);
    assert_eq!(result.choice.as_ref().unwrap().observer, target);
    assert_ne!(
        state.resolution.continuation_kind,
        engine_core::ContinuationKind::ResolveTanlu
    );

    let (state, result) = resolve(state, target, vec![target_card.raw()], false, 52);
    assert_eq!(state.players.get(target).unwrap().hp, hp);
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::ContinuationKind::ResolveTanlu
    );
    assert_eq!(
        result.choice.as_ref().unwrap().options[0].token,
        owner_card.raw()
    );
    let (state, result) = resolve(state, target, vec![owner_card.raw()], false, 53);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
}

#[test]
fn target_death_skips_discard_and_source_death_does_not_cancel_the_started_chain() {
    let target_dies = setup(2, 1, &[CardKind::Other], &[CardKind::Other], &[], &[]);
    let target = target_dies.target;
    let owner_card = target_dies.owner_cards[0];
    let (state, _) = accept(&target_dies, 60);
    let (state, result) = resolve(state, target, vec![1], false, 61);
    let (state, events) = pass_dying(state, result, 62);
    assert!(!state.players.get(target).unwrap().alive);
    assert_ne!(
        state.resolution.continuation_kind,
        engine_core::ContinuationKind::ResolveTanlu
    );
    assert_ne!(
        state.cards.get(owner_card).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
    assert!(!events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsMoved { cards, .. } if cards == &[owner_card]
    )));

    let source_dies = setup(2, 2, &[], &[CardKind::Other], &[SOURCE_DIES], &[]);
    let owner = source_dies.owner;
    let target = source_dies.target;
    let target_hp = source_dies.state.players.get(target).unwrap().hp;
    let (state, result) = accept(&source_dies, 70);
    let (state, events) = pass_dying(state, result, 71);
    assert!(!state.players.get(owner).unwrap().alive);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    assert_eq!(state.hand_count(owner), 0);
    assert!(!state.has_pending_choice());
    assert!(events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt { source, target: damaged, .. }
            if *source == owner && *damaged == target
    )));
}

#[test]
fn optional_cancel_bad_initial_context_and_runner_choices_are_authoritative() {
    let fixture = setup(
        3,
        1,
        &[CardKind::Other, CardKind::Other, CardKind::Other],
        &[],
        &[],
        &[],
    );
    let (state, _, _) = begin_turn(&fixture);
    let (state, result) = resolve(state, fixture.owner, Vec::new(), true, 80);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(fixture.target).unwrap().hp, 1);

    let mut tx = Transaction::begin(fixture.state.clone());
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTanlu {
                player: fixture.owner,
                target: fixture.target,
                amount: 0,
                stage: 0,
                skill: fixture.skill,
            }]
        ),
        Err(EffectError::Illegal(_))
    ));

    let (state, branch) = accept(&fixture, 81);
    let mut snapshot = Snapshot::capture(&state);
    snapshot.pending_choice = branch.choice;
    let mut engine = Engine::new();
    engine.restore_snapshot(&snapshot).unwrap();
    let branch_command = choose_command(&engine);
    assert!(matches!(
        &branch_command.kind,
        CommandKind::ResolveChoice {
            selections,
            cancelled: false,
            ..
        } if selections == &vec![0]
    ));
    assert!(matches!(
        engine.submit(branch_command),
        CommandResult::Accepted { .. }
    ));
    let gift_command = choose_command(&engine);
    assert!(matches!(
        &gift_command.kind,
        CommandKind::ResolveChoice {
            selections,
            cancelled: false,
            ..
        } if selections.len() == 2
    ));
    assert!(matches!(
        engine.submit(gift_command),
        CommandResult::Accepted { .. }
    ));
    assert_eq!(engine.state().hand_count(fixture.owner), 2);
}
