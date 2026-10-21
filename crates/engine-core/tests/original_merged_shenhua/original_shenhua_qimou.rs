//! Golden coverage for 界魏延「奇谋」rewritten from legacy JavaScript semantics.

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

const RE_WEIYAN: u32 = 2_771_589_662;
const XINKUANGGU: u32 = 4_089_972_415;
const QIMOU: u32 = 3_084_798_293;
const QIMOU2: u32 = 2_727_616_630;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/shenhua/shenhua.npack.json");

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
        .find_map(|(mark, count)| (*mark == QIMOU2).then_some(*count))
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

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let far = seats[2];
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
            skill_def: SkillDefId::new(QIMOU),
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
    (tx.commit().state, owner, far)
}

#[test]
fn package_publishes_complete_re_weiyan_and_qimou_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 671);
    assert_eq!(package.skill_index.len(), 201);
    assert_eq!(package.character_index.len(), 48);
    assert!(package.character_index.iter().any(|character| {
        character.id == RE_WEIYAN
            && character.name_key == "re_weiyan"
            && character.max_hp == 4
            && character.skills == [XINKUANGGU, QIMOU]
    }));
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == QIMOU)
        .unwrap();
    assert_eq!(active.window, "play_phase");
    assert!(active.once_per_game);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == QIMOU2
            && trigger.event == "turn_ended"
            && trigger.filter == "owner_is_target"
    }));
}

#[test]
fn choosing_three_loses_three_hp_grants_three_bonuses_once_and_expires() {
    let (state, owner, far) = setup(2_026_072_301);
    let active = skill_id(&state, owner, QIMOU).unwrap();
    let (state, opened) = command(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill: active,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        1,
    );
    assert!(matches!(opened.result, CommandResult::Accepted { .. }));
    let choice = opened.choice.unwrap();
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2, 3, 4]
    );
    assert_eq!(
        (choice.min_count, choice.max_count, choice.allow_cancel),
        (1, 1, false)
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
    assert_eq!(mark_count(&state, owner), 3);
    assert!(skill_id(&state, owner, QIMOU2).is_some());
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
    assert!(skill_id(tx.state(), owner, QIMOU2).is_none());
    assert_eq!(mark_count(tx.state(), owner), 0);
    assert_eq!(sha_use_limit_for(tx.state(), owner), 1);
}
