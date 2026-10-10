//! Golden coverage for KEY 井之原真人「暴拳」rewritten from legacy JavaScript.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::{CommitResult, Transaction};
use rule_api::EffectRequest;

const KEY_MASATO: u32 = 2_270_763_259;
const MASATO_BAOQUAN: u32 = 3_870_573_152;
const SOURCE: &str = include_str!("../../../content/original/ported/key/masato_baoquan.skill");
const KEY: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn prepared(seed: u64) -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("masato-baoquan-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let owner = tx.state().turn.current;
    let target = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(MASATO_BAOQUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
    owner_state.max_hp = 8;
    owner_state.hp = 4;
    let target_state = tx.state_mut().players.get_mut(target).unwrap();
    target_state.max_hp = 6;
    target_state.hp = 6;
    (tx, owner, target)
}

fn damage_and_pause(mut tx: Transaction, source: PlayerId, target: PlayerId) -> CommitResult {
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChoosePreventDamageOrLoseMaxHpIncreaseDamage
    );
    tx.commit()
}

fn resolve(
    state: GameState,
    owner: PlayerId,
    choice_id: ChoiceId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
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

#[test]
fn package_closes_exact_character_and_abi64_skill() {
    let package = content_schema::ContentPackage::from_json_bytes(KEY).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert!(package.manifest.capabilities.request_choice);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == KEY_MASATO)
        .unwrap();
    assert_eq!(character.name_key, "key_masato");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "key");
    assert_eq!(character.initial_hp, 4);
    assert_eq!(character.max_hp, 8);
    assert_eq!(character.skills, [MASATO_BAOQUAN]);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == MASATO_BAOQUAN)
        .unwrap();
    assert_eq!(trigger.event, "damage_before");
    assert_eq!(trigger.filter, "owner_source");
    assert!(!trigger.optional);
    let compiled = content_compiler::compile_source_named("masato_baoquan", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 64);
    assert_eq!(
        compiled.modules[0].logic_hash,
        package
            .modules
            .iter()
            .find(|module| module.name == compiled.modules[0].name)
            .unwrap()
            .logic_hash
    );
}

#[test]
fn choice_is_forced_and_prevent_branch_cancels_only_bound_damage() {
    let (tx, owner, target) = prepared(2_026_072_201);
    let committed = damage_and_pause(tx, owner, target);
    let choice = committed.output.choice.unwrap();
    assert_eq!(choice.observer, owner);
    assert_eq!((choice.min_count, choice.max_count), (1, 1));
    assert!(!choice.allow_cancel);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    let (state, result) = resolve(committed.state, owner, choice.choice_id, vec![1], false, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, 6);
    let owner_state = state.players.get(owner).unwrap();
    assert_eq!((owner_state.hp, owner_state.max_hp), (4, 8));
    assert!(!state.has_pending_choice());
    assert_eq!(state.resolution.current_damage_id, 0);
}

#[test]
fn increase_branch_loses_max_hp_then_changes_same_damage_from_one_to_three() {
    let (tx, owner, target) = prepared(2_026_072_202);
    let committed = damage_and_pause(tx, owner, target);
    let choice_id = committed.output.choice.as_ref().unwrap().choice_id;
    let (state, result) = resolve(committed.state, owner, choice_id, vec![2], false, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let owner_state = state.players.get(owner).unwrap();
    assert_eq!((owner_state.hp, owner_state.max_hp), (4, 7));
    assert_eq!(state.players.get(target).unwrap().hp, 3);
    assert!(!state.has_pending_choice());
    assert_eq!(state.resolution.current_damage_id, 0);
}

#[test]
fn cancellation_and_forged_option_are_rejected_atomically() {
    for (cancelled, selections, key) in [(true, Vec::new(), 3), (false, vec![3], 4)] {
        let (tx, owner, target) = prepared(2_026_072_203 + key);
        let committed = damage_and_pause(tx, owner, target);
        let choice_id = committed.output.choice.as_ref().unwrap().choice_id;
        let before = canonical_state_hash(&committed.state);
        let (same, result) = resolve(
            committed.state,
            owner,
            choice_id,
            selections,
            cancelled,
            key,
        );
        assert!(matches!(result.result, CommandResult::Rejected { .. }));
        assert_eq!(canonical_state_hash(&same), before);
        assert!(same.has_pending_choice());
    }
}

#[test]
fn damage_not_sourced_by_owner_does_not_trigger_baoquan() {
    let (mut tx, owner, target) = prepared(2_026_072_210);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: target,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 3);
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 8);
}
