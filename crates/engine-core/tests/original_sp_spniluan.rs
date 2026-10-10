//! Golden coverage for SP 韩遂「逆乱」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SPNILUAN: u32 = 4_063_477_590;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/spniluan.skill");

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_000 + kind.tag() as u32),
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

#[test]
fn dodged_converted_sha_refunds_use_count_but_hit_still_counts() {
    let package = content_compiler::compile_source_named("spniluan-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 354);
    assert!(package.view_as_index[0].refund_use_count_if_no_damage);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
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
    effects.extend(players.iter().map(|player| EffectRequest::MoveCards {
        cards: tx.state().hand_cards(*player),
        to: discard,
        from: tx.state().zones.find(ZoneKind::Hand, *player),
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SPNILUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|id| tx.state().skills.get(*id).unwrap().def.raw() == SPNILUAN)
        .unwrap();
    let first = inject(&mut tx, owner, CardKind::Shan, Suit::Club);
    let shan = inject(&mut tx, target, CardKind::Shan, Suit::Heart);
    let (state, result) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: vec![first],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 1);
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: target,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![shan.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 0);

    let mut tx = Transaction::begin(state);
    let second = inject(&mut tx, owner, CardKind::Tao, Suit::Spade);
    let hp = tx.state().players.get(target).unwrap().hp;
    let (state, result) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: vec![second],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 1);
}
