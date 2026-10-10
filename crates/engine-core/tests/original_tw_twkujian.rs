//! Golden coverage for TW 阎象「苦谏」.

use engine_api::{
    CardDefId, CardId, CardMoveReason, ChoiceId, Command, CommandKind, CommandResult,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const TWKUJIAN: u32 = 2_903_587_563;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twkujian.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn inject(tx: &mut Transaction, owner: PlayerId, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_364_000 + u32::from(rank)),
        kind: CardKind::Sha,
        suit: Suit::Heart,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn submit(state: GameState, actor: PlayerId, kind: CommandKind, key: u64) -> GameState {
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn tracked_cards_reward_use_response_and_force_discards_for_other_losses() {
    let package = content_compiler::compile_source_named("twkujian-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 364);
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == TWKUJIAN));

    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_080_104, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(TWKUJIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let owner_cost = inject(&mut tx, owner, 1);
    let target_cost = inject(&mut tx, target, 2);
    let tracked = [
        inject(&mut tx, owner, 3),
        inject(&mut tx, owner, 4),
        inject(&mut tx, owner, 5),
    ];
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == TWKUJIAN)
        .unwrap()
        .id;
    let mut state = tx.commit().state;

    state = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: tracked.to_vec(),
        },
        1,
    );
    assert!(tracked
        .iter()
        .all(|card| state.hand_cards(target).contains(card)));

    let owner_before = state.hand_count(owner);
    let target_before = state.hand_count(target);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let target_hand = state.zones.find(ZoneKind::Hand, target).unwrap();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCardsWithReason {
            cards: vec![tracked[0]],
            to: discard,
            from: Some(target_hand),
            reason: CardMoveReason::UseOrRespond,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), owner_before + 1);
    assert_eq!(tx.state().hand_count(target), target_before);

    let owner_after_reward = tx.state().hand_count(owner);
    let target_after_reward = tx.state().hand_count(target);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[engine_api::DomainEvent::CardResponded {
                player: target,
                respond_to: owner,
                card: tracked[0],
                response_kind: CardKind::Sha.tag(),
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap(),
        Vec::<EffectRequest>::new(),
        "the movement and CardResponded events must reward the same card only once",
    );
    assert_eq!(tx.state().hand_count(owner), owner_after_reward);
    assert_eq!(tx.state().hand_count(target), target_after_reward);

    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::MoveCards {
                cards: vec![tracked[1]],
                to: discard,
                from: Some(target_hand),
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = tx.commit().state;
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    state = submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id: choice,
            selections: vec![owner_cost.raw()],
            cancelled: false,
        },
        2,
    );
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    state = submit(
        state,
        target,
        CommandKind::ResolveChoice {
            choice_id: choice,
            selections: vec![target_cost.raw()],
            cancelled: false,
        },
        3,
    );
    assert!(!state.hand_cards(owner).contains(&owner_cost));
    assert!(!state.hand_cards(target).contains(&target_cost));
    assert!(state.hand_cards(target).contains(&tracked[2]));
}
