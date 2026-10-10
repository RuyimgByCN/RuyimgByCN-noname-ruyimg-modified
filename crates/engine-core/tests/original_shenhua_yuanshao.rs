//! Golden semantics for SHENHUA 袁绍「乱击」rewritten from legacy JavaScript.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
    SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const LUANJI: u32 = 2_717_757_870;
const RE_YUANSHAO: u32 = 4_036_141_875;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/luanji.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

fn setup() -> (
    GameState,
    PlayerId,
    PlayerId,
    SkillId,
    Vec<engine_api::CardId>,
) {
    let package = content_compiler::compile_source_named("luanji-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(LUANJI), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, other) = (players[0], players[1]);
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
            skill_def: SkillDefId::new(LUANJI),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    for player in [owner, other] {
        let participant = tx.state_mut().players.get_mut(player).unwrap();
        participant.max_hp = 8;
        participant.hp = 8;
        for card in tx.state().hand_cards(player) {
            let card = tx.state_mut().cards.get_mut(card).unwrap();
            card.kind = CardKind::Tao;
            card.suit = Suit::Heart;
        }
    }
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == LUANJI)
        .unwrap();
    let cards = tx.state().hand_cards(owner);
    (tx.commit().state, owner, other, skill, cards)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    cards: Vec<engine_api::CardId>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: Vec::new(),
                card_targets: cards,
            },
        },
    )
}

#[test]
fn package_publishes_yuanshao_with_exact_two_card_rule() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_YUANSHAO)
        .unwrap();
    assert_eq!(character.skills, [LUANJI, 3_178_128_297]);
    let rule = package
        .view_as_index
        .iter()
        .find(|rule| rule.skill_def_id == LUANJI)
        .unwrap();
    assert_eq!(rule.as_kind, "wanjian");
    assert_eq!((rule.min_cards, rule.max_cards), (2, 2));
    assert!(rule.same_suit);
}

#[test]
fn exactly_two_same_suit_hand_cards_become_wanjian() {
    let (mut state, owner, other, skill, cards) = setup();
    let before = state.players.get(other).unwrap().hp;

    let (next, one) = activate(state, owner, skill, vec![cards[0]], 1);
    assert!(matches!(one.result, CommandResult::Rejected { .. }));
    state = next;

    state.cards.get_mut(cards[1]).unwrap().suit = Suit::Club;
    let (next, mixed) = activate(state, owner, skill, vec![cards[0], cards[1]], 2);
    assert!(matches!(mixed.result, CommandResult::Rejected { .. }));
    state = next;

    state.cards.get_mut(cards[1]).unwrap().suit = Suit::Heart;
    let (state, accepted) = activate(state, owner, skill, vec![cards[0], cards[1]], 3);
    assert!(
        matches!(accepted.result, CommandResult::Accepted { .. }),
        "{:?}",
        accepted.result
    );
    assert_eq!(state.players.get(other).unwrap().hp, before - 1);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(
        cards[..2]
            .iter()
            .all(|card| state.cards.get(*card).unwrap().zone == discard),
        "zones={:?} pending={} continuation={:?}",
        cards[..2]
            .iter()
            .map(|card| state.cards.get(*card).unwrap().zone)
            .collect::<Vec<_>>(),
        state.resolution.pending_choice_id,
        state.resolution.continuation_kind
    );
}
