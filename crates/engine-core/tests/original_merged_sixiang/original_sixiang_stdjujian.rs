//! Golden coverage for 举荐（sixiang 标徐庶）无懈获牌（ABI 462）.
//!
//! 使用【无懈可击】后（每回合限一次），可令一名其他角色获得该牌。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STDJUJIAN: u32 = 2_772_808_467;
const SOURCE: &str = include_str!("../../../../content/original/ported/sixiang/stdjujian.skill");

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

fn fire_card_used(
    tx: Transaction,
    player: PlayerId,
    card: CardId,
) -> Result<Transaction, EffectError> {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::CardUsed {
            player,
            card,
            targets: Vec::new(),
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
fn gives_wuxie_to_chosen_target_once_per_turn() {
    let package = content_compiler::compile_source_named("stdjujian-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 462);
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
        skill_def: SkillDefId::new(STDJUJIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    // Create a wuxie card and place it in the discard pile (post-use state).
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .expect("discard pile");
    let wuxie = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_700),
        kind: CardKind::TrickWuxie,
        suit: engine_api::Suit::Spade,
        rank: 11,
        weapon_range: 0,
        zone: discard,
        owner: PlayerId::NONE,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(discard)
        .expect("discard zone")
        .cards
        .push(wuxie);

    // First wuxie use: optional confirm + target choice.
    let tx = fire_card_used(tx, owner, wuxie).unwrap();
    let mut state = tx.commit().state;
    assert!(
        state.has_pending_choice(),
        "optional confirm must be pending"
    );
    state = drive_choice(state, owner, 1); // owner confirms
    state = drive_choice(state, owner, target.raw()); // choose the recipient
    assert!(
        state.hand_cards(target).contains(&wuxie),
        "target gains the used wuxie"
    );

    // Second wuxie use in the same turn: once-per-turn, no choice.
    let mut tx = Transaction::begin(state);
    let wuxie2 = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_701),
        kind: CardKind::TrickWuxie,
        suit: engine_api::Suit::Club,
        rank: 12,
        weapon_range: 0,
        zone: discard,
        owner: PlayerId::NONE,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(discard)
        .expect("discard zone")
        .cards
        .push(wuxie2);
    let tx = fire_card_used(tx, owner, wuxie2).unwrap();
    let state = tx.commit().state;
    assert!(
        !state.has_pending_choice(),
        "second use in the same turn does not fire (once per turn)"
    );
}
