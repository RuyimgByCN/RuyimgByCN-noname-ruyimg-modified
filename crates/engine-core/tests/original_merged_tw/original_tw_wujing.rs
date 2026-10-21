//! Golden coverage for TW 吴景「锋捍/从击」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWFENGHAN: u32 = 3_542_733_092;
const TWCONGJI: u32 = 3_604_610_940;
const SOURCE: &str = include_str!("../../../../content/original/ported/tw/twwujing.skill");

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
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

fn inject_discard(tx: &mut Transaction, suit: Suit, def: u32) -> CardId {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit,
        rank: 8,
        weapon_range: 0,
        zone: discard,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(discard)
        .unwrap()
        .cards
        .push(card);
    card
}

#[test]
fn fenghan_draws_selected_targets_and_congji_gives_selected_red_discards() {
    let package = content_compiler::compile_source_named("tw-wujing-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 384);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_125, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWFENGHAN),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWCONGJI),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let skill = |tx: &Transaction, def| {
        tx.state()
            .skills
            .sorted_live()
            .into_iter()
            .find(|skill| skill.owner == owner && skill.def.raw() == def)
            .unwrap()
            .id
    };
    let fenghan = skill(&tx, TWFENGHAN);
    let first_hand = tx.state().hand_count(first);
    let second_hand = tx.state().hand_count(second);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwwujing {
                player: owner,
                target_count: 2,
                action: 0,
                event_cards: vec![],
                skill: fenghan,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, owner, vec![first.raw(), second.raw()], 1);
    assert_eq!(state.hand_count(first), first_hand + 1);
    assert_eq!(state.hand_count(second), second_hand + 1);

    let mut tx = Transaction::begin(state);
    let red = inject_discard(&mut tx, Suit::Heart, 93_850);
    let red_two = inject_discard(&mut tx, Suit::Diamond, 93_851);
    let black = inject_discard(&mut tx, Suit::Spade, 93_852);
    let congji = skill(&tx, TWCONGJI);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwwujing {
                player: owner,
                target_count: 0,
                action: 1,
                event_cards: vec![red, red_two, black],
                skill: congji,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, owner, vec![red.raw(), red_two.raw()], 2);
    let state = choose(state, owner, vec![first.raw()], 3);
    let hand = state.zones.find(ZoneKind::Hand, first).unwrap();
    assert_eq!(state.cards.get(red).unwrap().zone, hand);
    assert_eq!(state.cards.get(red_two).unwrap().zone, hand);
    assert_eq!(
        state.cards.get(black).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}
