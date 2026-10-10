//! 第二百一十六批：A0 置入面散批——store 置入腿（批 215 修复）模板晋升两件。
//! twxingwu 星舞（tw）：弃牌阶段开始时置入一张手牌（消耗面登记债）。
//! biaozhao 表召（sp2）：结束阶段堆空门控置入一张手牌（同点数消耗面登记债）。
//! 模板：phase 触发 optional → store_selected_in_skill_pile(self) →
//! SelectCards 1/1 续体 Cont 297 落堆。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const TWXINGWU: u32 = 2_550_622_152;
const BIAOZHAO: u32 = 3_640_810_944;

const TWXINGWU_SRC: &str = include_str!("../../../content/original/ported/tw/twxingwu.skill");
const TW: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");
const BIAOZHAO_SRC: &str =
    include_str!("../../../content/original/ported/sp2/biaozhao.skill");
const SP2: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn setup_with(source: &str, tag: &str, skill_def: u32, seed: u64) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named(tag, source).unwrap();
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

fn fire_phase(
    state: GameState,
    owner: PlayerId,
    phase: TurnPhaseTag,
) -> (GameState, Result<(), EffectError>) {
    let mut tx = Transaction::begin(state);
    let outcome = match dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Ok(effects) => engine_core::effect::run_effects(&mut tx, effects).map(|_| ()),
        Err(other) => panic!("unexpected dispatch error: {other:?}"),
    };
    (tx.commit().state, outcome)
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
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
        "resolve rejected {key}: {:?}",
        result.result
    );
    next
}

/// 置入全链：phase 触发 → confirm [1] → SelectCards 1/1 → 选牌落堆。
fn drive_placement(
    state: GameState,
    owner: PlayerId,
    skill_def: u32,
    phase: TurnPhaseTag,
    key: u64,
) -> GameState {
    let hand_before = state.hand_count(owner);
    let pile_before = state.skill_pile_cards(owner, skill_def).len();
    let (state, outcome) = fire_phase(state, owner, phase);
    assert!(
        matches!(outcome, Err(EffectError::ChoiceRequested)),
        "optional confirm must open: {outcome:?}"
    );
    let state = resolve(state, owner, vec![1], key);
    assert!(
        state.has_pending_choice(),
        "store SelectCards must follow confirm"
    );
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::state::ContinuationKind::StoreSelectedInSkillPile,
        "Cont 297 routing"
    );
    let pick = state
        .hand_cards(owner)
        .first()
        .copied()
        .expect("hand card to store");
    let state = resolve(state, owner, vec![pick.raw()], key + 1);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), hand_before - 1, "hand -1");
    assert_eq!(
        state.skill_pile_cards(owner, skill_def).len(),
        pile_before + 1,
        "pile +1"
    );
    state
}

#[test]
fn twxingwu_contract_and_placement() {
    // 契约：包内触发存在（phase_discard、optional）。
    let package = content_schema::ContentPackage::from_json_bytes(TW).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == TWXINGWU)
        .expect("twxingwu trigger");
    assert_eq!(trigger.event, "phase_discard");
    assert!(trigger.optional);

    // 行为：弃牌阶段开始 → 置入一张手牌（无堆空门控，可连续置入）。
    let (state, owner, _other) =
        setup_with(TWXINGWU_SRC, "twxingwu-b216", TWXINGWU, 20_260_922_01);
    let state = drive_placement(
        state,
        owner,
        TWXINGWU,
        TurnPhaseTag::Discard,
        20_260_922_01,
    );
    // 同回合再触发（测试直驱）：无门控 → 再次可置入，堆 +1。
    let state = drive_placement(
        state,
        owner,
        TWXINGWU,
        TurnPhaseTag::Discard,
        20_260_922_05,
    );
    assert_eq!(state.skill_pile_cards(owner, TWXINGWU).len(), 2);
}

#[test]
fn biaozhao_contract_and_pile_empty_gate() {
    // 契约：包内触发存在（phase_end、optional）。
    let package = content_schema::ContentPackage::from_json_bytes(SP2).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == BIAOZHAO)
        .expect("biaozhao trigger");
    assert_eq!(trigger.event, "phase_end");
    assert!(trigger.optional);

    // 行为：第一次结束阶段 → 置入（堆空门控通过）。
    let (state, owner, _other) =
        setup_with(BIAOZHAO_SRC, "biaozhao-b216", BIAOZHAO, 20_260_922_02);
    let state = drive_placement(state, owner, BIAOZHAO, TurnPhaseTag::End, 20_260_922_11);

    // 第二次结束阶段：堆非空 → body 级门控拦截，confirm 后无 store 选择。
    let (state, outcome) = fire_phase(state, owner, TurnPhaseTag::End);
    assert!(
        matches!(outcome, Err(EffectError::ChoiceRequested)),
        "confirm still opens (filter has no pile gate)"
    );
    let state = resolve(state, owner, vec![1], 20_260_922_21);
    assert!(
        !state.has_pending_choice(),
        "pile-empty gate blocks second placement"
    );
    assert_eq!(
        state.skill_pile_cards(owner, BIAOZHAO).len(),
        1,
        "pile unchanged"
    );
}
