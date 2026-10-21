//! Golden coverage for 得宠（sixiang 标毛皇后）赠牌 + 后续伤害（ABI 459）.
//!
//! 其他角色准备阶段：选任意张可弃置手牌/装备交给该角色；其下回合弃牌阶段开始时
//! 若手牌不小于体力值，拥有者可对其造成 1 点伤害。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STDDECHONG: u32 = 2_987_704_809;
const SOURCE: &str = include_str!("../../../../content/original/ported/sixiang/stddechong.skill");

fn drive_choice(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selection: u32,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted"
    );
    next
}

fn fire_phase(
    mut tx: Transaction,
    player: PlayerId,
    phase: TurnPhaseTag,
) -> Result<Transaction, EffectError> {
    let fired = dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: player,
            phase,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => Ok(tx),
            Err(EffectError::ChoiceRequested) => Ok(tx),
            Err(e) => Err(e),
        },
        Err(EffectError::ChoiceRequested) => Ok(tx),
        Err(e) => Err(e),
    }
}

#[test]
fn gives_cards_then_damages_at_target_discard_phase() {
    let package = content_compiler::compile_source_named("stddechong-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 459);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_115, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target, _third) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDDECHONG),
    });
    run_effects(&mut tx, effects).unwrap();
    // Give the owner one discardable hand card.
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, owner)
        .expect("owner hand zone");
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_700),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Heart,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(hand)
        .expect("hand zone")
        .cards
        .push(card);
    let target_hand_before = tx.state().hand_count(target);
    let target_hp_before = tx.state().players.get(target).unwrap().hp;

    // Target's prepare phase: card gift choice.
    let tx = fire_phase(tx, target, TurnPhaseTag::Prepare).unwrap();
    let mut state = tx.commit().state;
    assert!(
        state.has_pending_choice(),
        "gift card choice must be pending"
    );
    state = drive_choice(state, owner, card.raw());
    assert_eq!(
        state.hand_count(target),
        target_hand_before + 1,
        "target gains the gifted card"
    );
    assert!(!state.hand_cards(owner).contains(&card));

    // Target's discard phase: owner optionally deals 1 damage.
    let mut tx = Transaction::begin(state);
    let tx = fire_phase(tx, target, TurnPhaseTag::Discard).unwrap();
    let mut state = tx.commit().state;
    assert!(
        state.has_pending_choice(),
        "optional damage confirm must be pending"
    );
    state = drive_choice(state, owner, 1);
    assert_eq!(
        state.players.get(target).unwrap().hp,
        target_hp_before - 1,
        "target takes 1 damage at its discard phase"
    );
    assert!(!state.has_pending_choice());
}
