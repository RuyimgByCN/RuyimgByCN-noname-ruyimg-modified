//! Golden for 标文鸳「殇决」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STDKENGQIANG: u32 = 3_956_037_825;
const STDSHANGJUE: u32 = 3_835_885_608;
const STD_WENYUAN: u32 = 4_166_759_034;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdshangjue.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn setup(with_kengqiang: bool) -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 2_026_072_402, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let owner = tx.state().players.alive_by_seat()[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    if with_kengqiang {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDKENGQIANG),
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDSHANGJUE),
    });
    run_effects(&mut tx, effects).unwrap();
    let participant = tx.state_mut().players.get_mut(owner).unwrap();
    participant.max_hp = 3;
    participant.hp = 0;
    participant.dying = true;
    (tx, owner)
}

fn offer(tx: &mut Transaction, owner: PlayerId) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::EnteredDying {
            player: owner,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    )
}

fn resolve(
    state: GameState,
    owner: PlayerId,
    selection: u32,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    )
}

fn has_skill(state: &GameState, owner: PlayerId, skill_def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == skill_def)
}

#[test]
fn package_publishes_abi142_and_complete_std_wenyuan() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STDSHANGJUE)
        .unwrap();
    assert_eq!(
        (
            trigger.event.as_str(),
            trigger.filter.as_str(),
            trigger.optional
        ),
        ("dying", "owner_is_target", true)
    );
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_WENYUAN)
        .unwrap();
    assert_eq!(character.name_key, "std_wenyuan");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [STDKENGQIANG, STDSHANGJUE]);
    assert_eq!(
        content_compiler::compile_source_named("stdshangjue", SOURCE)
            .unwrap()
            .manifest
            .rule_vm_abi_version,
        142
    );
}

#[test]
fn accepting_recovers_before_exchange_and_accepting_exchange_restores_the_limit() {
    let (mut tx, owner) = setup(true);
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, result) = resolve(tx.commit().state, owner, 1, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert!(!state.players.get(owner).unwrap().dying);
    assert!(!has_skill(&state, owner, STDSHANGJUE));
    assert!(has_skill(&state, owner, STDKENGQIANG));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::RecoverToMaxThenOptionalExchangeSkill
    );
    assert_eq!(result.choice.unwrap().kind, ChoiceKind::Confirm);

    let (state, result) = resolve(state, owner, 1, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert!(!has_skill(&state, owner, STDKENGQIANG));
    assert!(has_skill(&state, owner, STDSHANGJUE));

    let mut tx = Transaction::begin(state);
    let participant = tx.state_mut().players.get_mut(owner).unwrap();
    participant.hp = 0;
    participant.dying = true;
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
}

#[test]
fn declining_or_missing_kengqiang_consumes_no_extra_skill_and_never_offers_exchange() {
    let (mut tx, owner) = setup(true);
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, result) = resolve(tx.commit().state, owner, 0, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, 0);
    assert!(has_skill(&state, owner, STDKENGQIANG));
    assert!(has_skill(&state, owner, STDSHANGJUE));

    let (mut tx, owner) = setup(false);
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, result) = resolve(tx.commit().state, owner, 1, 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert!(!state.has_pending_choice());
    assert!(!has_skill(&state, owner, STDKENGQIANG));
    assert!(!has_skill(&state, owner, STDSHANGJUE));
}

#[test]
fn declining_post_recovery_exchange_keeps_kengqiang_and_spends_shangjue() {
    let (mut tx, owner) = setup(true);
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, _) = resolve(tx.commit().state, owner, 1, 5);
    let (state, result) = resolve(state, owner, 0, 6);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert!(!state.has_pending_choice());
    assert!(has_skill(&state, owner, STDKENGQIANG));
    assert!(!has_skill(&state, owner, STDSHANGJUE));
}
