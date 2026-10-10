//! Golden coverage for REFRESH 界魏延「奇谋」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, RejectReason, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{query, sha_use_limit_for, QueryContext};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::TriggerFireRecord;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const OL_WEIYAN: u32 = 2_765_423_130;
const XINKUANGGU: u32 = 4_089_972_415;
const REQIMOU: u32 = 3_862_622_181;
const REQIMOU2: u32 = 2_749_452_350;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/refresh/reqimou.skill"),
    include_str!("../../../content/original/ported/refresh/reqimou2.skill"),
);
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> Option<engine_api::SkillId> {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .find_map(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
                .then_some(*skill)
        })
}

fn mark_count(state: &GameState, owner: PlayerId) -> i32 {
    state
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find_map(|(mark, count)| (*mark == REQIMOU2).then_some(*count))
        .unwrap_or(0)
}

fn command(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

#[test]
fn package_publishes_complete_ol_weiyan_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 251);
    assert!(package.skill_index.len() >= 76);
    assert!(package.character_index.iter().any(|character| {
        character.id == OL_WEIYAN
            && character.name_key == "ol_weiyan"
            && character.max_hp == 4
            && character.skills == [XINKUANGGU, REQIMOU]
    }));
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == REQIMOU)
        .unwrap();
    assert_eq!(active.window, "play_phase");
    assert!(active.once_per_game);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == REQIMOU2
            && trigger.event == "turn_ended"
            && trigger.filter == "owner_is_target"
    }));
}

#[test]
fn loses_and_draws_chosen_amount_grants_bonuses_once_then_expires() {
    let package = content_compiler::compile_source_named("reqimou-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, 20_260_724, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let far = players[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(REQIMOU),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let player = tx.state_mut().players.get_mut(owner).unwrap();
    player.hp = 4;
    player.max_hp = 4;
    let hand = tx.state().hand_count(owner);
    let active = skill_id(tx.state(), owner, REQIMOU).unwrap();

    let (state, opened) = command(
        tx.commit().state,
        owner,
        CommandKind::ActivateSkill {
            skill: active,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        1,
    );
    let choice = opened.choice.unwrap();
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2, 3, 4]
    );
    let (state, resolved) = command(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id: ChoiceId::new(choice.choice_id.raw()),
            selections: vec![3],
            cancelled: false,
        },
        2,
    );
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, 1);
    assert_eq!(state.hand_count(owner), hand + 3);
    assert_eq!(mark_count(&state, owner), 3);
    assert_eq!(sha_use_limit_for(&state, owner), 4);
    assert!(matches!(
        query(
            &state,
            &QueryRequest::Distance {
                from: owner,
                to: far,
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(1)
    ));
    let (_, repeated) = command(
        state.clone(),
        owner,
        CommandKind::ActivateSkill {
            skill: active,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        3,
    );
    assert!(matches!(
        repeated.result,
        CommandResult::Rejected {
            reason: RejectReason::IllegalAction { .. }
        }
    ));

    let mut tx = Transaction::begin(state);
    let effects = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: owner,
            phase: TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert!(skill_id(tx.state(), owner, REQIMOU2).is_none());
    assert_eq!(mark_count(tx.state(), owner), 0);
    assert_eq!(sha_use_limit_for(tx.state(), owner), 1);
}
