//! Golden coverage for TW 呼厨泉「复叛」.

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const TWFUPAN: u32 = 2_667_455_987;
const TW_HUCHUQUAN: u32 = 3_374_283_771;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twfupan.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn setup() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("twfupan-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 362);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_080_103, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target, other) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWFUPAN),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, target, other)
}

fn fire(tx: &mut Transaction, owner: PlayerId, target: PlayerId, amount: u32) {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::DamageDealt {
            source: owner,
            target,
            amount,
            is_lose_hp: false,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(
        run_effects(tx, effects),
        Err(EffectError::ChoiceRequested)
    );
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
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

fn accept_and_give(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    // 当前实现：触发后直接是交牌 SelectCards（无独立确认段）。
    let card = state.hand_cards(owner)[0];
    let (state, selected) = resolve(state, owner, vec![card.raw()], false, key);
    let target_choice = selected.choice.unwrap();
    assert_eq!(target_choice.kind, ChoiceKind::SelectTargets);
    assert!(target_choice
        .options
        .iter()
        .any(|option| option.token == target.raw()));
    resolve(state, owner, vec![target.raw()], false, key + 1)
}

#[test]
fn twfupan_draws_by_damage_and_tracks_each_recipient_through_the_damage_lock() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed.manifest.rule_vm_abi_version >= 362);
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == TW_HUCHUQUAN)
            .unwrap()
            .skills,
        [TWFUPAN]
    );

    let (mut tx, owner, target, other) = setup();
    let owner_hand = tx.state().hand_count(owner);
    let target_hand = tx.state().hand_count(target);
    fire(&mut tx, owner, target, 2);
    let optional = tx.commit();
    let (state, first) = accept_and_give(optional.state, owner, target, 1);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), owner_hand + 3);
    assert_eq!(state.hand_count(target), target_hand + 1);

    let mut tx = Transaction::begin(state);
    fire(&mut tx, owner, other, 1);
    let optional = tx.commit();
    let hp = optional.state.players.get(target).unwrap().hp;
    let (state, second) = accept_and_give(optional.state, owner, target, 10);
    let confirm = second.choice.unwrap();
    assert_eq!(confirm.kind, ChoiceKind::Confirm);
    assert_eq!(state.players.get(target).unwrap().hp, hp);
    let (state, damaged) = resolve(state, owner, vec![1], false, 12);
    assert!(matches!(damaged.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);

    // 复叛造成的伤害会再次触发复叛；交牌为 forced（不可取消），完成它。
    let (state, nested) = accept_and_give(state, owner, other, 14);
    assert!(matches!(nested.result, CommandResult::Accepted { .. }));
    let mut tx = Transaction::begin(state);
    fire(&mut tx, owner, other, 1);
    let optional = tx.commit();
    let card = optional.state.hand_cards(owner)[0];
    let (_, selected) = resolve(optional.state, owner, vec![card.raw()], false, 20);
    let targets = selected.choice.unwrap();
    assert!(!targets
        .options
        .iter()
        .any(|option| option.token == target.raw()));
}
