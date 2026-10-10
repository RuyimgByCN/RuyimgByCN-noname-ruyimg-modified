//! Golden coverage for SP「溃围」and the OFFLINE SP 曹仁 cross-package closure.

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

const JSP_CAOREN: u32 = 1_393_689_295;
const KUIWEI: u32 = 2_977_201_770;
const KUIWEI2: u32 = 4_087_547_393;
const YANZHENG: u32 = 2_899_287_996;
const NESTED_EMPTY_HAND: u32 = 9_900_001;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/kuiwei.skill");
const SP_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const OFFLINE_PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn setup(source: &str, skill_defs: &[u32]) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("kuiwei-test", source).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_723, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(skill_defs.iter().map(|skill_def| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(*skill_def),
    }));
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, first, second)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
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
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn inject_weapon(tx: &mut Transaction, owner: PlayerId, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_000 + u32::from(rank)),
        kind: CardKind::Weapon,
        suit: Suit::Spade,
        rank,
        weapon_range: 2,
        zone,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn has_skill_def(state: &GameState, player: PlayerId, skill_def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == skill_def)
        })
}

fn dispatch_optional_phase(
    tx: &mut Transaction,
    player: PlayerId,
    phase: Phase,
    tag: TurnPhaseTag,
) {
    tx.state_mut().turn.current = player;
    tx.state_mut().turn.phase = phase;
    let result = dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: player,
            phase: tag,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(result, Err(EffectError::ChoiceRequested));
}

fn dispatch_forced_phase(tx: &mut Transaction, player: PlayerId, phase: Phase, tag: TurnPhaseTag) {
    tx.state_mut().turn.current = player;
    tx.state_mut().turn.phase = phase;
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: player,
            phase: tag,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(run_effects(tx, effects), Err(EffectError::ChoiceRequested));
}

#[test]
fn committed_packages_expose_kuiwei_and_complete_jsp_caoren_closure() {
    let sp = content_schema::ContentPackage::from_json_bytes(SP_PACKAGE).unwrap();
    assert!(sp.manifest.rule_vm_abi_version >= 647);
    assert!(sp.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == KUIWEI
            && trigger.event == "phase_end"
            && trigger.filter == "owner_is_target"
            && trigger.optional
    }));
    assert!(sp.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == KUIWEI2
            && trigger.event == "phase_draw"
            && trigger.filter == "owner_is_target"
            && !trigger.optional
    }));

    let offline = content_schema::ContentPackage::from_json_bytes(OFFLINE_PACKAGE).unwrap();
    let character = offline
        .character_index
        .iter()
        .find(|character| character.id == JSP_CAOREN)
        .unwrap();
    assert_eq!(character.name_key, "jsp_caoren");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wei");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, vec![KUIWEI, YANZHENG]);

    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&sp).unwrap();
    std::sync::Arc::make_mut(&mut state.content).load_package(&offline).unwrap();
}

#[test]
fn end_phase_draw_uses_then_current_weapon_count_and_next_draw_recounts_for_discard() {
    let (mut tx, owner, first, second) = setup(SOURCE, &[KUIWEI]);
    let first_weapon = inject_weapon(&mut tx, first, 7);
    inject_weapon(&mut tx, second, 8);
    let hand_before = tx.state().hand_count(owner);

    dispatch_optional_phase(&mut tx, owner, Phase::End, TurnPhaseTag::End);
    let mut state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert!(state.players.get(owner).unwrap().turned_over);
    assert_eq!(state.hand_count(owner), hand_before + 4);
    assert!(has_skill_def(&state, owner, KUIWEI2));

    let mut tx = Transaction::begin(state);
    let equip = tx.state().zones.find(ZoneKind::Equip, first).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![first_weapon],
            to: discard,
            from: Some(equip),
        }],
    )
    .unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().turned_over = false;
    dispatch_forced_phase(&mut tx, owner, Phase::Draw, TurnPhaseTag::Draw);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseDiscardHandEquipThenRemoveSelfSkill
    );
    let selected = tx.state().resolution.continuation_payload[4];
    let hand_before_discard = tx.state().hand_count(owner);
    state = resolve(tx.state().clone(), owner, vec![selected], false, 2);
    assert_eq!(state.hand_count(owner), hand_before_discard - 1);
    assert!(!has_skill_def(&state, owner, KUIWEI2));
    assert!(has_skill_def(&state, owner, KUIWEI));
}

#[test]
fn insufficient_cards_are_all_discarded_without_choice_and_helper_is_removed() {
    let (mut tx, owner, first, second) = setup(SOURCE, &[KUIWEI2]);
    inject_weapon(&mut tx, first, 7);
    inject_weapon(&mut tx, second, 8);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let cards = tx.state().hand_cards(owner);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: cards[1..].to_vec(),
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Draw;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Draw,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), 0);
    assert!(!tx.state().has_pending_choice());
    assert!(!has_skill_def(tx.state(), owner, KUIWEI2));
}

#[test]
fn one_shot_cleanup_waits_for_nested_discard_triggers() {
    let nested_source = format!(
        "{SOURCE}\nskill nested({NESTED_EMPTY_HAND}) {{\n  trigger cards_moved owner_from_hand_empty_other_discardable_hand_equip optional {{\n    effect emit_note(9900001);\n  }}\n}}\n"
    );
    let (mut tx, owner, first, _) = setup(&nested_source, &[KUIWEI2, NESTED_EMPTY_HAND]);
    inject_weapon(&mut tx, first, 7);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let cards = tx.state().hand_cards(owner);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: cards[1..].to_vec(),
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();

    dispatch_forced_phase(&mut tx, owner, Phase::Draw, TurnPhaseTag::Draw);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    assert!(has_skill_def(tx.state(), owner, KUIWEI2));
    let state = resolve(tx.state().clone(), owner, Vec::new(), true, 3);
    assert!(!state.has_pending_choice());
    assert!(!has_skill_def(&state, owner, KUIWEI2));
}
