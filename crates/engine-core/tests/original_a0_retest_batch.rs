//! 第二百一十九批：A0 池变张/单张置入复测——rebiaozhao 表召（mobile）与
//! twmingren 明任（tw）晋升行为测试（quanji/store 模板，批 215/216 同款）。
//! 复测维持债：xinqingjian（gainAfter 非额定摸牌分类触发）、twyouye/mbyouye
//! （按对手本回合伤害历史过滤）、twmingren 置换面。

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

const REBIAOZHAO: u32 = 4_210_066_284;
const REBIAOZHAO_SRC: &str =
    include_str!("../../../content/original/ported/mobile/rebiaozhao.skill");
const MOBILE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");
const TWMINGREN: u32 = 2_729_616_872;
const TWMINGREN_SRC: &str = include_str!("../../../content/original/ported/tw/twmingren.skill");
const TW: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn setup_with(
    source: &str,
    tag: &str,
    skill_def: u32,
    seed: u64,
) -> (GameState, PlayerId, PlayerId) {
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

/// 可选触发的完整链路：触发 → 确认 [1] → Cont 297 SelectCards 1/1 → 入堆。
fn drive_optional_store(
    state: GameState,
    owner: PlayerId,
    skill_def: u32,
    phase: TurnPhaseTag,
    key: u64,
) -> GameState {
    let hand_before = state.hand_count(owner);
    let pile_before = state.skill_pile_cards(owner, skill_def).len();
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
        Err(other) => panic!("unexpected: {other:?}"),
    };
    assert!(
        matches!(outcome, Err(EffectError::ChoiceRequested)),
        "confirm must open: {outcome:?}"
    );
    let state = tx.commit().state;
    let state = resolve(state, owner, vec![1], key);
    assert!(state.has_pending_choice(), "store choice must open");
    let pick = state.hand_cards(owner).first().copied().unwrap();
    let state = resolve(state, owner, vec![pick.raw()], key + 1);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), hand_before - 1);
    assert_eq!(
        state.skill_pile_cards(owner, skill_def).len(),
        pile_before + 1
    );
    state
}

#[test]
fn rebiaozhao_contract_and_placement_with_pile_gate() {
    let package = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == REBIAOZHAO)
        .expect("rebiaozhao trigger");
    assert_eq!(trigger.event, "phase_end");
    assert!(trigger.optional);

    // 首个结束阶段：置入。
    let (state, owner, _other) =
        setup_with(REBIAOZHAO_SRC, "rebiaozhao-b219", REBIAOZHAO, 20_260_925_01);
    let state = drive_optional_store(
        state,
        owner,
        REBIAOZHAO,
        TurnPhaseTag::End,
        20_260_925_11,
    );
    // 第二个结束阶段：堆空门控拦截（确认后无选卡）。
    let mut tx = Transaction::begin(state);
    let outcome = match dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Ok(effects) => engine_core::effect::run_effects(&mut tx, effects).map(|_| ()),
        Err(other) => panic!("unexpected: {other:?}"),
    };
    assert!(matches!(outcome, Err(EffectError::ChoiceRequested)));
    let state = tx.commit().state;
    let state = resolve(state, owner, vec![1], 20_260_925_21);
    assert!(!state.has_pending_choice(), "pile gate blocks second");
    assert_eq!(state.skill_pile_cards(owner, REBIAOZHAO).len(), 1);
}

#[test]
fn twmingren_contract_and_game_start_placement() {
    let package = content_schema::ContentPackage::from_json_bytes(TW).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == TWMINGREN)
        .expect("twmingren trigger");
    assert_eq!(trigger.event, "game_started");
    assert!(!trigger.optional, "forced");

    // 行为：开局 → 摸 1 + 必选置入一张。
    let (state, owner, _other) =
        setup_with(TWMINGREN_SRC, "twmingren-b219", TWMINGREN, 20_260_925_02);
    let hand_before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 2 }],
        &mut TriggerFireRecord::new(),
    )
    .expect("forced dispatch");
    let outcome = engine_core::effect::run_effects(&mut tx, effects);
    assert!(
        matches!(outcome, Err(EffectError::ChoiceRequested)),
        "forced store opens SelectCards: {outcome:?}"
    );
    let state = tx.commit().state;
    assert!(state.has_pending_choice());
    let pick = state.hand_cards(owner).first().copied().unwrap();
    let state = resolve(state, owner, vec![pick.raw()], 20_260_925_31);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), hand_before, "draw 1, store 1");
    assert_eq!(state.skill_pile_cards(owner, TWMINGREN).len(), 1);
}
