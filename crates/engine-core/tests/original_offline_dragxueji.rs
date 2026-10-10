//! offline/dragxueji 血祭（第一百二十七批·近似晋升）：active 出牌阶段限一次，
//! choose_other_damage 选一名其他角色造成 1 点伤害（上游为火伤，普伤近似）。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DRAGXUEJI: u32 = 3_444_463_445;

const SOURCES_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../content/original/ported/offline"
);

fn sources() -> String {
    let dir = std::path::Path::new(SOURCES_DIR);
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "skill"))
        .map(|e| e.path())
        .collect();
    files.sort();
    files
        .iter()
        .map(|p| {
            let mut s = std::fs::read_to_string(p).unwrap();
            s.push('\n');
            s
        })
        .collect()
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

fn started() -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named("offline-dragxueji", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_840, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
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
            skill_def: SkillDefId::new(DRAGXUEJI),
        }],
    )
    .unwrap();
    (tx.commit().state, owner)
}

#[test]
fn dragxueji_publishes_contract() {
    let compiled = content_compiler::compile_source_named("offline-dragxueji-contract", &sources())
        .unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 691);
    assert!(compiled
        .active_index
        .iter()
        .any(|a| a.skill_def_id == DRAGXUEJI));
}

#[test]
fn dragxueji_deals_one_damage_to_chosen_target() {
    let (state, holder) = started();
    let victim = *state
        .players
        .alive_by_seat()
        .iter()
        .find(|&&p| p != holder)
        .unwrap();

    let mut tx = Transaction::begin(state);
    // 出牌阶段内发动(active 门控需要 Play 阶段)。
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().turn.current = holder;
    let sid = {
        let s = tx.state();
        s.skills
            .sorted_live()
            .iter()
            .find(|s| s.def.raw() == DRAGXUEJI)
            .unwrap()
            .id
    };
    let victim_hp = tx.state().players.get(victim).unwrap().hp;

    // ActivateSkill 命令驱动主动技:效果内 open SelectTargets 选目标。
    let (state_after, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor: holder,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(3001),
            kind: CommandKind::ActivateSkill {
                skill: sid,
                targets: vec![victim],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    let mid = state_after;

    // 选目标续体:resolve [victim] → victim 受 1 点伤害。
    let final_state = resolve(mid, holder, vec![victim.raw()], 3101);
    assert_eq!(
        final_state.players.get(victim).unwrap().hp,
        victim_hp - 1,
        "choose_other_damage deals 1 damage to the chosen target"
    );
}
