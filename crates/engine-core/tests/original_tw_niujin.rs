//! Golden coverage for TW 牛金「挫锐」「裂围」.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWCUORUI: u32 = 3_369_253_433;
const TWLIEWEI: u32 = 2_387_969_096;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twniujin.skill");

fn choose(state: GameState, actor: PlayerId, selection: u32) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn cuorui_disables_judge_and_liewei_can_reset_it() {
    let package = content_compiler::compile_source_named("tw-niujin-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 391);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_210, "identity-standard").unwrap();
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
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWCUORUI),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWLIEWEI),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let cuorui = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == TWCUORUI)
        .unwrap()
        .id;
    let liewei = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == TWLIEWEI)
        .unwrap()
        .id;
    let target_hand = tx.state().hand_count(other);
    // 挫锐当前实现与上游一致：自动补牌（至最高手牌、最多 5 张）并废除判定区，无确认 choice。
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Prepare.to_u8(),
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    assert_eq!(state.hand_count(owner), target_hand.min(5));
    assert!(state.disabled_judge_zones.contains(&owner));
    assert!(state.zones.find(ZoneKind::Judge, owner).is_some());
    assert_eq!(state.skills.get(cuorui).unwrap().lifetime_fire_count, 1);

    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwniujin {
                player: owner,
                limited_skill_def: SkillDefId::new(TWCUORUI),
                action: 1,
                stage: 0,
                skill: liewei,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = choose(tx.commit().state, owner, 1);
    assert_eq!(state.skills.get(cuorui).unwrap().lifetime_fire_count, 0);
}
