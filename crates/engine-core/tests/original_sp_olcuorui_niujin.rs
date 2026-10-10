//! Golden coverage for SP Niu Jin's 挫锐 rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OLCUORUI: u32 = 2_241_852_934;
const LIEWEI: u32 = 3_656_060_804;
const NIUJIN: u32 = 2_469_157_690;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/olcuorui.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup(player_count: u8) -> (Transaction, PlayerId, PlayerId, CardId) {
    let package = content_compiler::compile_source_named("sp-olcuorui-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, player_count, 2_026_072_705, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let source = players[1];
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let cards = tx.state().zones.get(hand).unwrap().cards.clone();
    if !cards.is_empty() {
        effects.push(EffectRequest::MoveCards {
            cards,
            to: discard,
            from: Some(hand),
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLCUORUI),
    });
    run_effects(&mut tx, effects).unwrap();
    let source_hand = tx.state().zones.find(ZoneKind::Hand, source).unwrap();
    let delayed = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_030),
        kind: CardKind::TrickLe,
        suit: Suit::Spade,
        rank: 6,
        weapon_range: 0,
        zone: source_hand,
        owner: source,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(source_hand)
        .unwrap()
        .cards
        .push(delayed);
    (tx, owner, source, delayed)
}

#[test]
fn package_publishes_olcuorui_and_niujin_without_javascript() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLCUORUI && skill.name == "olcuorui" && skill.locked));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == NIUJIN)
        .unwrap();
    assert_eq!(character.skills, [OLCUORUI, LIEWEI]);
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}

#[test]
fn fills_to_eight_then_consumes_one_pending_judge_skip() {
    let (mut tx, owner, source, delayed) = setup(8);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 8 }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), 8);

    let targeted = DomainEvent::CardUsed {
        player: source,
        card: delayed,
        targets: vec![owner],
    };
    let effects =
        dispatch_for_events(&mut tx, &[targeted.clone()], &mut TriggerFireRecord::new()).unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().pending_judge_skips, [owner]);

    let repeated =
        dispatch_for_events(&mut tx, &[targeted], &mut TriggerFireRecord::new()).unwrap();
    assert!(repeated.is_empty());

    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Judge.to_u8(),
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.pending_judge_skips.is_empty());
    assert!(result.events.iter().any(|event| matches!(
        event,
        DomainEvent::PhaseSkipped {
            current,
            phase: TurnPhaseTag::Judge,
        } if *current == owner
    )));
}
