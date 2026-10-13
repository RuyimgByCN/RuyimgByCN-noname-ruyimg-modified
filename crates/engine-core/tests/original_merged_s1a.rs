//! O2 全量合并第二批（2026-10-14）：家族 s1a 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_s1a_expansion_awaken {
//! 第一百八十九批：ADR 0641 §8 S1-A 首批——expansion 计数门控觉醒三件零新原语复活。
//! moucuan（谋篡，mobile）：「兵」≥势力数 → 减上限+获兵火；
//! xinzili（自立，refresh）：「权」>2 → 回复+摸2-减上限+获排溢；
//! tianxing（天行，onlyOL）：「蛊」≥3 → 减上限+移除雏囤+四选一授技。
//! 供给：直接构造 (owner, def) 技能堆（等价 P2 置入终态；P2 选牌续体为 ollihuo
//! 形态限手杀——泛化债记入 ADR 0641 §8）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Zone, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const MOUCUAN: u32 = 2_965_767_294;
const JIBING: u32 = 3_194_681_807;
const BINGHUO: u32 = 4_199_054_322;

fn setup(seed: u64, skill_source: &str, skill_def: u32, pile_skill: Option<u32>) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("s1a-test", skill_source).unwrap();
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
        skill_def: SkillDefId::new(skill_def),
    });
    if let Some(pile) = pile_skill {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(pile),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

/// 注入一张手牌【杀】，返回（新 state, card id）。
fn inject_hand_card(state: GameState, owner: PlayerId, def: u32) -> (GameState, CardId) {
    let mut tx = Transaction::begin(state);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
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

/// 直接构造 (owner, pile_def) 技能堆并移入 n 张牌（等价 P2 置入终态）。
fn fill_pile(state: GameState, owner: PlayerId, pile_def: u32, n: usize) -> GameState {
    let mut state = state;
    for i in 0..n {
        let (s2, card) = inject_hand_card(state, owner, 9_000 + i as u32);
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
                tx.state_mut().skill_piles.push(engine_core::state::SkillPileRef {
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
        state = tx.commit().state;
    }
    state
}

/// forced 触发的模块在 dispatch 期内联执行：体内的 choose 会让 dispatch 本身
/// 返回 Err(ChoiceRequested)（选择已进 tx）；无选择体则走 Ok(effects) → run_effects。
fn fire_prepare(state: GameState, owner: PlayerId) -> (GameState, Result<(), EffectError>) {
    let mut tx = Transaction::begin(state);
    let outcome = match dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Ok(effects) => engine_core::effect::run_effects(&mut tx, effects).map(|_| ()),
        Err(other) => panic!("unexpected dispatch error: {other:?}"),
    };
    (tx.commit().state, outcome)
}

fn has_skill(state: &GameState, owner: PlayerId, skill_def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == skill_def)
}

#[test]
fn moucuan_awakens_when_pile_meets_group_count() {
    let source = include_str!("../../../content/original/ported/mobile/moucuan.skill");
    let (state, owner, _other) = setup(20_260_913_01, source, MOUCUAN, Some(JIBING));
    let max_before = state.players.get(owner).unwrap().max_hp;

    // 0 张「兵」：不触发。
    let (state, outcome) = fire_prepare(state, owner);
    assert!(outcome.is_ok() && !state.has_pending_choice());
    assert_eq!(state.players.get(owner).unwrap().max_hp, max_before);

    // 填 2 张：势力数口径下 2 人局至多 2 组，必达门。
    let state = fill_pile(state, owner, JIBING, 2);
    let (state, outcome) = fire_prepare(state, owner);
    assert!(outcome.is_ok(), "forced awakening, got {outcome:?}");
    assert_eq!(state.players.get(owner).unwrap().max_hp, max_before - 1);
    assert!(has_skill(&state, owner, BINGHUO), "兵火 granted");

    // once 门控：再次准备阶段不重复觉醒。
    let (state, outcome) = fire_prepare(state, owner);
    assert!(outcome.is_ok());
    assert_eq!(
        state.players.get(owner).unwrap().max_hp,
        max_before - 1,
        "max hp must not drop twice"
    );
}

#[test]
fn xinzili_awakens_at_three_tokens() {
    let source = include_str!("../../../content/original/ported/refresh/xinzili.skill");
    let (state, owner, _other) = setup(20_260_913_02, source, 4_072_748_511, Some(3_444_777_027));
    let state = {
        let mut tx = Transaction::begin(state);
        tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
        tx.commit().state
    };
    let state = fill_pile(state, owner, 3_444_777_027, 3);
    let (state, outcome) = fire_prepare(state, owner);
    assert!(outcome.is_ok(), "forced awakening, got {outcome:?}");
    assert!(has_skill(&state, owner, 3_639_231_394), "排溢 granted");
    assert!(!state.has_pending_choice());
}

#[test]
fn tianxing_awakens_and_chooses_one_of_four_grants() {
    let source = include_str!("../../../content/original/ported/onlyOL/tianxing.skill");
    let (state, owner, _other) = setup(20_260_913_03, source, 2_406_413_222, Some(3_324_653_503));
    let state = fill_pile(state, owner, 3_324_653_503, 3);
    let (state, outcome) = fire_prepare(state, owner);
    assert!(
        matches!(outcome, Err(EffectError::ChoiceRequested)),
        "awakening runs to the grant choice, got {outcome:?}"
    );
    assert!(state.has_pending_choice(), "four-way grant choice pending");

    // 选 1 → 仁德。
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (next, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(20_260_913_11),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(has_skill(&next, owner, 2_246_702_546), "仁德 granted");
    assert!(!has_skill(&next, owner, 2_455_708_599), "制衡 not granted");
}
}
