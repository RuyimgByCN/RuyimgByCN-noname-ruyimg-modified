//! Golden coverage for old 虎啸 rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLDHUXIAO: u32 = 3_615_190_640;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/oldhuxiao.skill");

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(93_278 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn play(state: GameState, owner: PlayerId, target: PlayerId, sha: CardId, key: u64) -> GameState {
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    state
}

#[test]
fn dodged_sha_refunds_its_use_count_but_a_hit_still_counts() {
    let package = content_compiler::compile_source_named("oldhuxiao-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 278);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_072_939, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
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
    for player in &players {
        effects.push(EffectRequest::MoveCards {
            cards: tx.state().hand_cards(*player),
            to: discard,
            from: tx.state().zones.find(ZoneKind::Hand, *player),
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLDHUXIAO),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let first_sha = inject(&mut tx, owner, CardKind::Sha, Suit::Spade);
    let shan = inject(&mut tx, target, CardKind::Shan, Suit::Heart);
    let state = play(tx.commit().state, owner, target, first_sha, 1);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 1);
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor: target,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![shan.raw()],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 0);

    let mut tx = Transaction::begin(state);
    let second_sha = inject(&mut tx, owner, CardKind::Sha, Suit::Club);
    let state = play(tx.commit().state, owner, target, second_sha, 3);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 1);
    assert_eq!(state.players.get(target).unwrap().hp, 3);
}
