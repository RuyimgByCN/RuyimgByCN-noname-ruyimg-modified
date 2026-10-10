//! Golden for sp batch-AD DSL rewrite:
//! 武继 (wuji, 关银屏)：结束阶段开始时，若本回合累计造成至少 3 点伤害，失去「虎啸」、
//! 加 1 点体力上限并回复 1 点体力，然后觉醒。上游 (sp/skill.js:42961)。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{GameState, turn_flag};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const WUJI: u32 = 2_966_745_164;
const HUXIAO: u32 = 3_796_089_056;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let mut state = engine.state().clone();
    // v1.11.6 增量 sp roster +1（王皑）平移种子：开局 optional 触发可能挂起 choice，
    // 以 cancelled=true 排空（original_sp_qimen_zniaoxiang 范例）。
    let mut drain_key = 1u64;
    while state.has_pending_choice() {
        let choice = ChoiceId::new(state.resolution.pending_choice_id);
        let revision = state.revision;
        let (next, output) = apply_command(
            state,
            Command {
                actor: PlayerId::NONE,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(seed ^ drain_key),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        assert!(
            matches!(output.result, CommandResult::Accepted { .. }),
            "drain rejected: {:?}",
            output.result
        );
        state = next;
        drain_key += 1;
    }
    let owner = state.turn.current;
    (state, owner)
}

fn with_skill(state: GameState, owner: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(HUXIAO),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

#[test]
fn package_publishes_wuji_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == WUJI
            && trigger.event == "phase_end"
            && trigger.filter == "owner_is_target_turn_damage_at_least_three"
            && !trigger.optional));
    // 武继不再保留手牌上限 +0 的诚实 noop modifier
    assert!(!package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == WUJI && modifier.target == "hand_limit"
    }));
}

#[test]
fn wuji_awakens_at_three_dealt_damage() {
    let (state, owner) = started(2_026_081_701);
    let state = with_skill(state, owner, WUJI);
    let mut tx = Transaction::begin(state);
    // 本回合已造成 3 点伤害
    tx.state_mut().turn.set_flag(turn_flag::DAMAGE_DEALT, 3);
    let hp_before = tx.state().players.get(owner).unwrap().hp;
    let max_hp_before = tx.state().players.get(owner).unwrap().max_hp;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    let player = tx.state().players.get(owner).unwrap();
    assert_eq!(player.max_hp, max_hp_before + 1);
    assert_eq!(player.hp, hp_before + 1);
    // 失去「虎啸」并觉醒（移除武继自身）
    let defs: Vec<_> = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .filter_map(|skill| tx.state().skills.get(*skill))
        .map(|skill| skill.def.raw())
        .collect();
    assert!(!defs.contains(&HUXIAO), "虎啸 must be removed");
    assert!(!defs.contains(&WUJI), "武继 must remove itself after awakening");
}

#[test]
fn wuji_does_not_awaken_below_three_damage() {
    let (state, owner) = started(2_026_081_702);
    let state = with_skill(state, owner, WUJI);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.set_flag(turn_flag::DAMAGE_DEALT, 2);
    let max_hp_before = tx.state().players.get(owner).unwrap().max_hp;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty(), "below-3 damage must not trigger");
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, max_hp_before);
    let defs: Vec<_> = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .filter_map(|skill| tx.state().skills.get(*skill))
        .map(|skill| skill.def.raw())
        .collect();
    assert!(defs.contains(&WUJI), "武继 must stay until threshold met");
}
