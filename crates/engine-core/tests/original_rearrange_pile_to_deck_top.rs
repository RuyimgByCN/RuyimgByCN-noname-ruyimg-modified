//! 第二百一十四批：chooseToMove 堆重排原语 `rearrange_pile_to_deck_top` 行为测试
//! （tag 713 / ABI 690 / Cont 296，A1 直构模式）。
//! 语义：从自身 (player, pile_def) 技能堆选 min(count, 堆数) 张，按所选顺序依次
//! 压入牌堆顶（最后所选最靠顶，与 place_hand_cards_to_deck_top 同序）。
//! §8.8 修订后 41 件 chooseToMove 消费者均为复合面（登记债），本测试以合成
//! probe 技能直构验证原语真实可执行（dormant 契约）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Zone, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const PILE_DEF: u32 = 3_241_592_654;
const PILE_DEF_WIDE: u32 = 3_241_592_655;

fn setup(seed: u64, probe_def: u32, pile_def: u32, count: u32) -> (GameState, PlayerId, PlayerId) {
    let source = format!(
        "skill probe_rptd({}) {{\n  trigger phase_prepare owner_is_target {{\n    effect rearrange_pile_to_deck_top(self, self_skill, {}, {});\n  }}\n}}\n",
        probe_def, pile_def, count
    );
    let compiled = content_compiler::compile_source_named("rptd-test", &source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(probe_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn inject_hand_card(state: GameState, owner: PlayerId, def: u32) -> (GameState, CardId) {
    let mut tx = Transaction::begin(state);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    (tx.commit().state, card)
}

/// 直接构造 (owner, pile_def) 技能堆并移入 n 张牌，返回堆内卡序。
fn fill_pile(
    state: GameState,
    owner: PlayerId,
    pile_def: u32,
    n: usize,
) -> (GameState, Vec<CardId>) {
    let mut state = state;
    let mut cards = Vec::new();
    for i in 0..n {
        let (s2, card) = inject_hand_card(state, owner, 9_800 + i as u32);
        state = s2;
        let mut tx = Transaction::begin(state);
        let existing = tx
            .state()
            .skill_piles
            .iter()
            .find(|p| p.owner == owner && p.skill_def == pile_def)
            .map(|p| p.zone);
        let zone = match existing {
            Some(zone) => zone,
            None => {
                let zone = tx.state_mut().zones.alloc(Zone {
                    id: engine_api::ZoneId::NONE,
                    kind: ZoneKind::Expansion,
                    owner,
                    cards: Vec::new(),
                });
                tx.state_mut()
                    .skill_piles
                    .push(engine_core::state::SkillPileRef {
                        owner,
                        skill_def: pile_def,
                        zone,
                        usable_as_hand: false,
                        wants_lost_count: false,
                    });
                zone
            }
        };
        let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
        tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
        tx.state_mut()
            .zones
            .get_mut(hand)
            .unwrap()
            .cards
            .retain(|c| *c != card);
        if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
            card_state.zone = zone;
        }
        cards.push(card);
        state = tx.commit().state;
    }
    (state, cards)
}

fn fire_phase(
    state: GameState,
    owner: PlayerId,
) -> (GameState, Result<(), EffectError>) {
    let mut tx = Transaction::begin(state);
    let outcome = match dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Ok(effects) => engine_core::effect::run_effects(&mut tx, effects).map(|_| ()),
        Err(other) => panic!("unexpected dispatch error: {other:?}"),
    };
    (tx.commit().state, outcome)
}

fn resolve(
    state: GameState,
    owner: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (
    GameState,
    engine_core::command::PipelineResult,
) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (next, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    (next, result)
}

fn draw_zone_cards(state: &GameState) -> Vec<CardId> {
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .expect("draw pile");
    state.zones.get(draw).unwrap().cards.clone()
}

#[test]
fn rearrange_moves_selection_to_deck_top_in_order() {
    let (state, owner, _other) = setup(20_260_919_01, 900_1006, PILE_DEF, 2);
    let (state, cards) = fill_pile(state, owner, PILE_DEF, 3);
    assert_eq!(state.skill_pile_cards(owner, PILE_DEF).len(), 3);

    let (state, outcome) = fire_phase(state, owner);
    assert!(
        matches!(outcome, Err(EffectError::ChoiceRequested)),
        "SelectCards choice must open, got {outcome:?}"
    );
    assert!(state.has_pending_choice());
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::state::ContinuationKind::RearrangePileToDeckTop,
        "Cont 296 routing"
    );

    // 选 [c2, c0]：依次压顶 → 牌堆顶自顶向下为 c0、c2（最后所选最靠顶）。
    let (state, result) = resolve(
        state,
        owner,
        vec![cards[2].raw(), cards[0].raw()],
        20_260_919_11,
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "resolve rejected: {:?}",
        result.result
    );
    assert!(!state.has_pending_choice());

    // 堆内剩余 c1。
    assert_eq!(
        state.skill_pile_cards(owner, PILE_DEF),
        vec![cards[1]],
        "pile keeps unselected card"
    );
    // 牌堆顶两张按所选顺序落位。
    let draw = draw_zone_cards(&state);
    assert_eq!(&draw[draw.len() - 2..], &[cards[2], cards[0]], "top order");
    // 卡区与归属更新为公共牌堆。
    for card in [cards[2], cards[0]] {
        let card_state = state.cards.get(card).unwrap();
        let draw_zone = state.zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
        assert_eq!(card_state.zone, draw_zone, "zone updated");
        assert!(card_state.owner.is_none(), "owner cleared to field");
    }
}

#[test]
fn rearrange_count_clamped_to_pile_size() {
    // count=5 但堆仅 2 张 → 选择量钳位为 2，两张全部置顶。
    let (state, owner, _other) = setup(20_260_919_02, 900_1007, PILE_DEF_WIDE, 5);
    let (state, cards) = fill_pile(state, owner, PILE_DEF_WIDE, 2);

    let (state, outcome) = fire_phase(state, owner);
    assert!(
        matches!(outcome, Err(EffectError::ChoiceRequested)),
        "clamped choice still opens, got {outcome:?}"
    );
    let (state, result) = resolve(
        state,
        owner,
        vec![cards[0].raw(), cards[1].raw()],
        20_260_919_21,
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "2 selections accepted under clamp: {:?}",
        result.result
    );
    assert!(!state.has_pending_choice());
    assert_eq!(
        state.skill_pile_cards(owner, PILE_DEF_WIDE).len(),
        0,
        "pile drained"
    );
    let draw = draw_zone_cards(&state);
    assert_eq!(&draw[draw.len() - 2..], &[cards[0], cards[1]], "both on top");
}

#[test]
fn rearrange_empty_pile_is_noop() {
    let (state, owner, _other) = setup(20_260_919_03, 900_1008, PILE_DEF, 2);
    let (state, outcome) = fire_phase(state, owner);
    assert!(outcome.is_ok(), "empty pile must not open choice");
    assert!(!state.has_pending_choice());
}

#[test]
fn rearrange_rejects_foreign_selection() {
    let (state, owner, _other) = setup(20_260_919_04, 900_1009, PILE_DEF, 2);
    let (state, cards) = fill_pile(state, owner, PILE_DEF, 2);
    // 一张手牌（不在堆内）混入选择 → 拒绝。
    let (state, foreign) = inject_hand_card(state, owner, 9_899);

    let (state, _) = fire_phase(state, owner);
    let (_, result) = resolve(
        state,
        owner,
        vec![cards[0].raw(), foreign.raw()],
        20_260_919_41,
    );
    assert!(
        !matches!(result.result, CommandResult::Accepted { .. }),
        "foreign card must be rejected: {:?}",
        result.result
    );
}
