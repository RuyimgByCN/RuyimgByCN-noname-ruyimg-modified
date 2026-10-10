//! Golden coverage for 应图（huicui 陈珪，ABI 487）.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit, ZoneId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCYINGTU: u32 = 3_170_147_690;
const DCCONGSHI: u32 = 3_668_194_449;
const CHENGUI: u32 = 3_537_583_790;
const SOURCE: &str = include_str!("../../../content/original/ported/huicui/dcyingtu.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/huicui/huicui.npack.json");

fn resolve_choice(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
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
        "choice must be accepted: {:?}",
        result.result
    );
    next
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    kind: CardKind,
    def: u32,
    suit: Suit,
    zone: ZoneId,
) -> CardId {
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 5,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn neighbor_gain_triggers_take_give_chain() {
    let packaged = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(packaged.manifest.rule_vm_abi_version, 487);
    assert_eq!(
        packaged
            .character_index
            .iter()
            .find(|character| character.id == CHENGUI)
            .unwrap()
            .skills,
        [DCYINGTU, DCCONGSHI]
    );
    assert!(packaged.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == DCYINGTU
            && trigger.filter == "owner_alive_neighbor_gained_cards_outside_own_draw_once_per_turn"
            && trigger.optional
    }));
    let package = content_compiler::compile_source_named("dcyingtu-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 487);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_801, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, next, prev) = (seats[0], seats[1], seats[2]);
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    // Empty all hands; seed the "next" neighbor with a Sha, the owner with a Tao.
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for &player in &[owner, next, prev] {
        let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
        let cards: Vec<CardId> = tx.state().zones.get(hand).unwrap().cards.clone();
        run_effects(
            &mut tx,
            cards
                .into_iter()
                .map(|card| EffectRequest::MoveCards {
                    cards: vec![card],
                    to: discard,
                    from: Some(hand),
                })
                .collect(),
        )
        .unwrap();
    }
    let next_hand = tx.state().zones.find(ZoneKind::Hand, next).unwrap();
    let sha = alloc_card(&mut tx, next, CardKind::Sha, 96_100, Suit::Spade, next_hand);
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let tao = alloc_card(
        &mut tx,
        owner,
        CardKind::Tao,
        96_000,
        Suit::Heart,
        owner_hand,
    );
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCYINGTU),
        }],
    )
    .unwrap();

    // The "next" neighbor gains a card (a move into its hand) -> the trigger fires.
    let mut tx = Transaction::begin(tx.commit().state);
    let moved = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_200),
        kind: CardKind::Sha,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: discard,
        owner: PlayerId::NONE,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(discard)
        .unwrap()
        .cards
        .push(moved);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![moved],
            to: next_hand,
            from: Some(discard),
        }],
    );
    assert!(matches!(
        result,
        Err(engine_core::effect::EffectError::ChoiceRequested)
    ));
    let state = tx.commit().state;
    assert!(
        state.resolution.pending_choice_id != 0,
        "gain choice pending after the neighbor gained"
    );
    let state = resolve_choice(state, owner, vec![1], 1); // confirm 应图
    assert_eq!(state.resolution.continuation_payload[3], next.raw());
    assert!(engine_core::query::gainable_target_cards(
        &state,
        owner,
        next,
        rule_api::TargetCardScope::HandEquip,
    )
    .contains(&sha));
    let mut state = resolve_choice(state, owner, vec![sha.raw()], 2);
    assert!(state.has_pending_choice(), "give choice pending");
    state = resolve_choice(state, owner, vec![tao.raw()], 3);

    assert!(
        state.hand_cards(owner).contains(&sha),
        "the owner gained the Sha from the next neighbor"
    );
    assert!(
        state.hand_cards(prev).contains(&tao),
        "the other neighbor received the Tao"
    );
    assert!(
        !state.has_pending_choice(),
        "once-per-turn prevents re-entry"
    );
}
