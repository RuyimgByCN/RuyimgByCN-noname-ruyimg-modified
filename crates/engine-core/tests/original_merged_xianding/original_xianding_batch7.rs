//! Golden for XIANDING（限定包）批次七：
//! 虞袭（dcyuxi）、复盏（dcsbfuzhan）。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCYUXI: u32 = 3_133_735_666;
const DCSBFUZHAN: u32 = 2_554_996_648;
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
        // T15③ roster +2 平移：开局撞 optional 触发——cancelled 排空（qimen 范例）。
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
    let _ = run_effects(&mut tx, effects);
    // 卸技/弃牌可能再开 optional 窗——再排空一次。
    let mut st = tx.commit().state;
    let mut drain_key = 0xB7u64;
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
        assert!(matches!(output.result, CommandResult::Accepted { .. }));
        st = next;
        drain_key += 1;
    }
    let mut tx = Transaction::begin(st);
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
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = engine_core::command::apply_command(
        state,
        engine_api::Command {
            actor,
            expected_revision: engine_api::ExpectedRevision(revision),
            idempotency_key: engine_api::IdempotencyKey(key),
            kind: engine_api::CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(
        result.result,
        engine_api::CommandResult::Accepted { .. }
    ));
    state
}

fn deal_damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
) -> (GameState, Result<Vec<DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }],
    );
    (tx.commit().state, result)
}

#[test]
fn package_exposes_batch7_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let yuxi = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCYUXI)
        .expect("dcyuxi trigger");
    assert_eq!(yuxi.event, "damage");
    assert_eq!(yuxi.filter, "owner_is_source_or_target");
    assert!(yuxi.optional);

    let fuzhan = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCSBFUZHAN)
        .expect("dcsbfuzhan trigger");
    assert_eq!(fuzhan.event, "dying_after");
    assert_eq!(fuzhan.filter, "owner_alive_not_target");
    assert!(!fuzhan.optional);
}

#[test]
fn dcyuxi_draws_when_owner_causes_damage() {
    let (mut tx, owner, first, _second) = setup(20_260_816);
    add_skill(&mut tx, owner, DCYUXI);
    tx.state_mut().players.get_mut(first).unwrap().hp -= 1;
    let before = tx.state().hand_count(owner);
    let (state, result) = deal_damage(tx.state().clone(), owner, first);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(state, owner, vec![1], false, 1);
    assert_eq!(state.hand_count(owner), before + 1);
}

#[test]
fn dcyuxi_draws_when_owner_suffers_damage() {
    let (mut tx, owner, first, _second) = setup(20_260_817);
    add_skill(&mut tx, owner, DCYUXI);
    tx.state_mut().players.get_mut(owner).unwrap().hp -= 1;
    let before = tx.state().hand_count(owner);
    let (state, result) = deal_damage(tx.state().clone(), first, owner);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let state = resolve(state, owner, vec![1], false, 2);
    assert_eq!(state.hand_count(owner), before + 1);
}

#[test]
fn dcsbfuzhan_recovers_owner_to_max_after_other_leaves_dying() {
    let (mut tx, owner, first, _second) = setup(20_260_818);
    add_skill(&mut tx, owner, DCSBFUZHAN);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::LeftDying { player: first }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, max_hp as i32);
}
