//! Golden coverage for OFFLINE 杨秋「齐锋」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const WN_QIFENG: u32 = 3_930_492_591;

fn inject(tx: &mut Transaction, owner: PlayerId, def: u32) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn losing_the_same_card_twice_makes_the_phase_end_virtual_sha_deal_two_damage() {
    let package = content_compiler::compile_source_named(
        "wn-qifeng-test",
        include_str!("../../../../content/original/ported/offline/wn_qifeng.skill"),
    )
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 496);

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 2_026_080_202, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
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
        skill_def: SkillDefId::new(WN_QIFENG),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.lost_cards.clear();
    tx.state_mut()
        .turn
        .set_flag(turn_flag::card_lost_by_player(owner), 0);
    tx.state_mut().turn.current = target;
    tx.state_mut().turn.phase = Phase::Discard;
    let card = inject(&mut tx, owner, 98_400);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardCards {
            player: owner,
            cards: vec![card],
        }],
    )
    .unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::MoveCards {
                cards: vec![card],
                to: hand,
                from: Some(discard),
            },
            EffectRequest::DiscardCards {
                player: owner,
                cards: vec![card],
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().turn.card_loss_count(owner), 2);
    assert!(tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == owner && skill.def == SkillDefId::new(WN_QIFENG)));
    let hp = tx.state().players.get(target).unwrap().hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::SetTurnState {
                current: target,
                phase: Phase::End.to_u8(),
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 2);
}
