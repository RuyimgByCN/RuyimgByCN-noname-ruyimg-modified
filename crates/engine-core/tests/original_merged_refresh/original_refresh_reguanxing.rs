//! 第一百八十四批：refresh「观星」（界诸葛亮 reguanxing）零新原语重写。
//! 上游：准备阶段（frequent）观看牌堆顶 X 张（存活<4 → 3，否则 5），任意重排
//! （拖拽顶/底）；取消后有结束阶段补偿臂（reguanxing_on 门控，未移植——登记债）。
//! 实现：phase_prepare + owner_is_target（forced）→ guanxing_top(self, 3|5)
//! （逐张二选顶/底近似拖拽——pxlangya 同债；循环拒绝 cancel 属原语契约）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const REGUANXING: u32 = 4_220_147_649;
const RE_ZHUGELIANG: u32 = 2_446_214_915;
const SOURCE: &str = include_str!("../../../../content/original/ported/refresh/reguanxing.skill");
const REFRESH: &[u8] =
    include_bytes!("../../../../content/original/ported/refresh/refresh.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("reguanxing-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
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
        skill_def: SkillDefId::new(REGUANXING),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn fire_prepare(state: GameState, owner: PlayerId) -> (GameState, Result<(), EffectError>) {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let outcome = run_effects(&mut tx, effects);
    (tx.commit().state, outcome.map(|_| ()))
}

fn resolve(state: GameState, owner: PlayerId, pick: u32, key: u64) -> GameState {
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
                selections: vec![pick],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "guanxing round rejected: {:?}",
        result.result
    );
    next
}

#[test]
fn package_publishes_reguanxing_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 681);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == REGUANXING)
        .unwrap();
    assert_eq!(trigger.event, "phase_prepare");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_ZHUGELIANG)
        .expect("re_zhugeliang in roster");
    assert!(character.skills.contains(&REGUANXING));
}

#[test]
fn prepare_phase_opens_three_card_guanxing_loop_in_small_game() {
    // 2 人局存活数 < 4 → 观星 3 张（countPlayer()<4 ? 3 : 5 的存活数口径）。
    let (state, owner, _other) = setup(20_260_909_01);
    let (state, outcome) = fire_prepare(state, owner);
    assert!(
        matches!(outcome, Err(EffectError::ChoiceRequested)),
        "guanxing loop must open, got {outcome:?}"
    );
    assert!(state.has_pending_choice());

    // 三轮逐张二选：前两张置底（token 2），最后一张置顶（token 1）。
    let mut next = state;
    let mut key = 20_260_909_11;
    for round in 0..3 {
        assert!(next.has_pending_choice(), "round {round} must stay open");
        next = resolve(next, owner, if round == 2 { 1 } else { 2 }, key);
        key += 1;
    }
    assert!(!next.has_pending_choice(), "loop must close after 3 rounds");
    let processing = next
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .unwrap();
    assert!(
        next.zones.get(processing).unwrap().cards.is_empty(),
        "processing must drain after guanxing settles"
    );
    let draw = next.zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    assert!(
        next.zones.get(draw).unwrap().cards.len() >= 3,
        "draw pile must hold the reordered cards"
    );
}
