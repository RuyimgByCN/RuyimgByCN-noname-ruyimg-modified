//! Golden for XIANDING（限定包）批次四十二：
//! 夺锐（dcduorui，DC 神张辽）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCDUORUI: u32 = 2_823_548_533;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package())
        .unwrap();
    let mut tx = Transaction::begin(state);
    if bootstrap_match(&mut tx, 3, seed, "identity-standard").is_err() {
        // v1.11.6 增量 roster +7 平移种子：开局撞 optional 触发（ChoiceRequested）——
        // 以 cancelled=true 排空（original_sp_qimen_zniaoxiang 范例）。
        let mut st = tx.commit().state;
        let mut drain_key = 1u64;
        while st.has_pending_choice() {
            let choice = ChoiceId::new(st.resolution.pending_choice_id);
            let revision = st.revision;
            let (next, output) = apply_command(
                st,
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
            st = next;
            drain_key += 1;
        }
        tx = Transaction::begin(st);
    }
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
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
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_exposes_batch42_skill_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCDUORUI)
        .expect("dcduorui trigger");
    assert_eq!(trigger.event, "damage_before");
    assert_eq!(trigger.filter, "owner_is_source");
    assert!(trigger.optional);
}

#[test]
fn dcduorui_reduces_damage_by_one_when_optional_confirmed() {
    let (mut tx, owner, first, _second) = setup(20_260_879);
    add_skill(&mut tx, owner, DCDUORUI);
    let hp = tx.state().players.get(first).unwrap().hp;

    let err = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: first,
            amount: 3,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    assert!(matches!(err, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(state.players.get(first).unwrap().hp, hp - 2);
}
