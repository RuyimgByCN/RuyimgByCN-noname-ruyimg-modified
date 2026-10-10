//! Golden coverage for 劫粮（sp2 韩猛）摸牌削减 + 弃牌截获（ABI 461）.
//!
//! 其他角色摸牌阶段：可弃一张牌令其摸牌数/手牌上限 -1；其弃牌阶段弃牌后可获一张弃牌。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const JIELIANG: u32 = 4_227_370_894;
const JIELIANG_LESS: u32 = 3_795_806_305;
const JIELIANG_GAIN: u32 = 2_999_325_050;
const SOURCE: &str = include_str!("../../../content/original/ported/sp2/jieliang.skill");

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
fn reduces_draw_and_intercepts_discarded_card() {
    let package = content_compiler::compile_source_named("jieliang-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 461);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
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
        skill_def: SkillDefId::new(JIELIANG),
    });
    run_effects(&mut tx, effects).unwrap();
    // Give the owner one discardable card for the cost.
    let owner_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, owner)
        .expect("owner hand zone");
    let cost = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_700),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Heart,
        rank: 3,
        weapon_range: 0,
        zone: owner_hand,
        owner,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(owner_hand)
        .expect("hand zone")
        .cards
        .push(cost);
    // Give the target one card that will be discarded later.
    let target_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .expect("target hand zone");
    let loot = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_701),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: target_hand,
        owner: target,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(target_hand)
        .expect("hand zone")
        .cards
        .push(loot);

    // Target's draw phase: optional confirm + cost discard.
    let mut tx = Transaction::begin(tx.commit().state);
    tx.state_mut().turn.current = target;
    let tx = fire_phase(tx, target, TurnPhaseTag::Draw).unwrap();
    let mut state = tx.commit().state;
    assert!(
        state.has_pending_choice(),
        "optional confirm must be pending"
    );
    state = drive_choice(state, owner, 1); // owner confirms
    state = drive_choice(state, owner, cost.raw()); // owner discards the cost card
    assert!(
        state
            .skills
            .sorted_live()
            .iter()
            .any(|skill| skill.owner == target && skill.def.raw() == JIELIANG_LESS),
        "target gains jieliang_less (draw -1, hand limit -1)"
    );
    assert!(
        state
            .skills
            .sorted_live()
            .iter()
            .any(|skill| skill.owner == owner && skill.def.raw() == JIELIANG_GAIN),
        "owner gains jieliang_gain"
    );

    // Target's discard phase: move the loot card to the discard pile; owner intercepts it.
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::Discard;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .expect("discard pile");
    let moved = run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![loot],
            to: discard,
            from: Some(target_hand),
        }],
    );
    assert!(
        matches!(moved, Err(EffectError::ChoiceRequested)),
        "intercept choice must pause"
    );
    let mut state = tx.commit().state;
    assert!(
        state.has_pending_choice(),
        "intercept card choice must be pending"
    );
    state = drive_choice(state, owner, loot.raw());
    assert!(
        state.hand_cards(owner).contains(&loot),
        "owner intercepts the discarded card"
    );
    assert!(!state.has_pending_choice());
}
