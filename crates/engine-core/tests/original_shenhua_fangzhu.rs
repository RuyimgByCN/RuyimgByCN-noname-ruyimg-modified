//! Golden coverage for 神话「放逐」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const FANGZHU: u32 = 2_335_755_764;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/fangzhu.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    source: PlayerId,
    target: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("fangzhu-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 20_260_722_85, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let source = seats[1];
    let target = seats[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(FANGZHU),
    });
    run_effects(&mut tx, effects).unwrap();
    {
        let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
        owner_state.max_hp = 6;
        owner_state.hp = 6;
    }
    Setup {
        state: tx.commit().state,
        owner,
        source,
        target,
    }
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_exposes_exact_damage_contract() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(committed.manifest.rule_vm_abi_version, 671);
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == FANGZHU && skill.name == "fangzhu" && !skill.locked));
    let trigger = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == FANGZHU)
        .unwrap();
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_is_target_other_alive");
    assert!(trigger.optional);

    let compiled = content_compiler::compile_source_named("fangzhu", SOURCE).unwrap();
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn target_draws_current_owner_lost_hp_before_turning_over() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: setup.source,
                target: setup.owner,
                amount: 2,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    let (mut state, confirmed) = resolve(state, setup.owner, vec![1], false, 1);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseOtherDrawOwnerLostHpTurnOver
    );
    let options: Vec<_> = confirmed
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert!(!options.contains(&setup.owner.raw()));
    assert!(options.contains(&setup.target.raw()));

    state.players.get_mut(setup.owner).unwrap().hp = 3;
    let (state, resolved) = resolve(state, setup.owner, vec![setup.target.raw()], false, 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.target), 3);
    assert!(state.players.get(setup.target).unwrap().turned_over);
    let draw_index = resolved
        .events
        .iter()
        .position(|event| matches!(event, DomainEvent::CardsDrawn { player, .. } if *player == setup.target))
        .unwrap();
    let turn_index = resolved
        .events
        .iter()
        .position(|event| matches!(event, DomainEvent::PlayerTurnedOver { player, .. } if *player == setup.target))
        .unwrap();
    assert!(draw_index < turn_index);
}

#[test]
fn no_other_player_suppresses_trigger_and_dead_choice_is_rejected() {
    let setup = setup();
    let mut no_other = Transaction::begin(setup.state.clone());
    for player in no_other.state().players.alive_by_seat() {
        if player != setup.owner {
            no_other.state_mut().players.get_mut(player).unwrap().alive = false;
        }
    }
    run_effects(
        &mut no_other,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: setup.owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert!(!no_other.state().has_pending_choice());

    let mut tx = Transaction::begin(setup.state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: setup.source,
                target: setup.owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (mut state, confirmed) = resolve(tx.commit().state, setup.owner, vec![1], false, 3);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    state.players.get_mut(setup.target).unwrap().alive = false;
    let (_, rejected) = resolve(state, setup.owner, vec![setup.target.raw()], false, 4);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
}
