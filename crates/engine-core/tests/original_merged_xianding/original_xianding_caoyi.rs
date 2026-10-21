//! Golden for XIANDING 曹轶【蜜饴/寅君】（ABI 606）。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCMIYI: u32 = 3_041_789_306;
const DCMIYI_EFFECT: u32 = 3_041_789_307;
const SOURCE: &str = include_str!("../../../../content/original/ported/xianding/caoyi.skill");

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
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
        "{result:?}"
    );
    state
}

#[test]
fn miyi_reverses_the_selected_effect_at_end_phase() {
    let package = content_compiler::compile_source_named("caoyi-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(std::iter::once(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCMIYI),
        }))
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    tx.state_mut().players.get_mut(target).unwrap().hp -= 1;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DCMIYI)
        .unwrap()
        .id;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveCaoyi {
                player: owner,
                card: CardId::NONE,
                helper_skill_def: SkillDefId::new(DCMIYI_EFFECT),
                action: 0,
                event_targets: Vec::new(),
                skill,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = choose(tx.commit().state, owner, vec![0], 6061);
    let state = choose(state, owner, vec![target.raw()], 6062);
    assert_eq!(
        state.players.get(target).unwrap().hp,
        state.players.get(target).unwrap().max_hp as i32
    );
    let helper = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == target && skill.def.raw() == DCMIYI_EFFECT)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::End;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveCaoyi {
            player: target,
            card: CardId::NONE,
            helper_skill_def: SkillDefId::new(DCMIYI_EFFECT),
            action: 10,
            event_targets: Vec::new(),
            skill: helper,
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state().players.get(target).unwrap().hp,
        tx.state().players.get(target).unwrap().max_hp as i32 - 1
    );
    assert!(tx.state().skills.get(helper).is_none());
}
