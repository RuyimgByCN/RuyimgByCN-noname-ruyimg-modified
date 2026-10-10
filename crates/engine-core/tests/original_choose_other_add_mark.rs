//! Golden coverage for the `choose_other_add_mark` primitive (ABI 652, tag 672,
//! Cont 271): mbzhenfu (mobile 镇抚) + cangchizhibi×4 (shahai 苍鸱之臂).

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_api::TurnPhaseTag;
use rule_api::EffectRequest;

const MBZHENFU: u32 = 4_040_497_433;
const CANGCHIZHIBI_E1: u32 = 3_833_037_504;
const MOBILE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");
const SHAHAI: &[u8] = include_bytes!("../../../content/original/ported/shahai/shahai.npack.json");
const MBZHENFU_SRC: &str =
    include_str!("../../../content/original/ported/mobile/mbzhenfu.skill");
const CANGCHIZHIBI_SRC_RAW: &str =
    include_str!("../../../content/original/ported/shahai/shahai_meta.skill");

fn bootstrap(source: &str, skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("coam", source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(skill_def), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|p| *p != owner)
        .unwrap();
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }],
    )
    .unwrap();
    (tx, owner, other)
}

fn cangchizhibi_block() -> String {
    let src = CANGCHIZHIBI_SRC_RAW;
    let start = src.find("// cangchizhibi_equip1").unwrap();
    let decl = src[start..].find("skill cangchizhibi_equip1").unwrap() + start;
    let open = src[decl..].find('{').unwrap() + decl;
    let bytes = src.as_bytes();
    let mut depth = 0;
    let mut i = open;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            depth += 1;
        } else if bytes[i] == b'}' {
            depth -= 1;
            if depth == 0 {
                break;
            }
        }
        i += 1;
    }
    src[start..=i].to_string()
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    state
}

fn hujia(state: &GameState, player: PlayerId) -> i32 {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(def, _)| *def == 3)
        .map_or(0, |(_, n)| *n)
}

fn chained(state: &GameState, player: PlayerId) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .any(|(def, n)| *def == 2 && *n > 0)
}

#[test]
fn choose_other_add_mark_npack_contracts() {
    let mobile = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    assert_eq!(mobile.manifest.rule_vm_abi_version, 734);
    assert!(mobile.trigger_index.iter().any(|t| {
        t.skill_def_id == MBZHENFU && t.event == "phase_end" && t.optional
    }));
    let shahai = content_schema::ContentPackage::from_json_bytes(SHAHAI).unwrap();
    assert_eq!(shahai.manifest.rule_vm_abi_version, 657);
    assert!(shahai.trigger_index.iter().any(|t| {
        t.skill_def_id == CANGCHIZHIBI_E1 && t.event == "phase_end" && t.optional
    }));
}

#[test]
fn mbzhenfu_end_phase_grants_other_one_armor() {
    let (mut tx, owner, other) = bootstrap(MBZHENFU_SRC, MBZHENFU);
    assert_eq!(hujia(tx.state(), other), 0);
    // 批 279 门槛：owner 本回合须失去过牌——先弃一张手牌。
    {
        let lost = tx.state().hand_cards(owner)[0];
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardCards {
                player: owner,
                cards: vec![lost],
            }],
        )
        .unwrap();
    }
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: TurnPhaseTag::End,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    );
    // optional confirm → 选目标（other）。
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    let state = resolve(state, owner, vec![u32::from(other.raw())], 3);
    assert_eq!(hujia(&state, other), 1, "other should gain one armor");
    assert_eq!(hujia(&state, owner), 0);
}

#[test]
fn cangchizhibi_end_phase_chains_other() {
    let (mut tx, owner, other) = bootstrap(&cangchizhibi_block(), CANGCHIZHIBI_E1);
    assert!(!chained(tx.state(), other));
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: TurnPhaseTag::End,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    let state = resolve(state, owner, vec![u32::from(other.raw())], 3);
    assert!(chained(&state, other), "other should be chained");
    assert!(!chained(&state, owner));
}

#[test]
fn mbzhenfu_without_card_loss_does_not_trigger() {
    let (mut tx, owner, _other) = bootstrap(MBZHENFU_SRC, MBZHENFU);
    // 未失去过牌：结束阶段不触发（批 279 门槛生效）。
    let outcome = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(outcome.is_ok(), "gate must suppress the trigger: {outcome:?}");
    let state = tx.commit().state;
    assert!(!state.has_pending_choice());
    assert_eq!(hujia(&state, _other), 0);
}
