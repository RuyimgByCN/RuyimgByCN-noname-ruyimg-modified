//! Golden coverage for 势斩's reversed virtual-Duel active skill.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
    SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{CardKind, GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SHIZHAN: u32 = 3_879_999_347;
const REYAOWU: u32 = 2_805_206_003;
const RE_HUAXIONG: u32 = 3_874_151_406;
const JUE_DOU_LOCK: u32 = 2_000_000_113;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/shizhan.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn setup(lock_target: bool) -> (GameState, PlayerId, PlayerId, SkillId) {
    let source = if lock_target {
        format!(
            "{SOURCE}\nskill juedou_lock({JUE_DOU_LOCK}) locked {{ modifier card_use_cancel(juedou) override 1; }}"
        )
    } else {
        SOURCE.to_owned()
    };
    let package = content_compiler::compile_source_named("shizhan-test", &source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_718, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
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
        skill_def: SkillDefId::new(SHIZHAN),
    });
    if lock_target {
        effects.push(EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(JUE_DOU_LOCK),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    for player in [owner, target] {
        for card in tx.state().hand_cards(player) {
            tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
        }
    }
    tx.state_mut().players.get_mut(owner).unwrap().hp = 6;
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 6;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == SHIZHAN)
        .unwrap();
    (tx.state().clone(), owner, target, skill)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    )
}

#[test]
fn committed_package_declares_target_legality_and_huaxiong() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 23);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == SHIZHAN)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(2));
    let targets = active.targets.as_ref().unwrap();
    assert!(!targets.allow_self);
    assert_eq!(targets.can_use_on_owner.as_deref(), Some("juedou"));
    assert!(package.character_index.iter().any(|character| {
        character.id == RE_HUAXIONG
            && character.max_hp == 6
            && character.skills == vec![REYAOWU, SHIZHAN]
    }));
}

#[test]
fn selected_target_uses_duel_on_owner_and_limit_is_two() {
    let (state, owner, target, skill) = setup(false);
    let owner_hp = state.players.get(owner).unwrap().hp;
    let target_hp = state.players.get(target).unwrap().hp;

    let (state, first) = activate(state, owner, target, skill, 1);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp);

    let (state, second) = activate(state, owner, target, skill, 2);
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 2);
    assert_eq!(state.skills.get(skill).unwrap().use_count, 2);

    let (same, third) = activate(state.clone(), owner, target, skill, 3);
    assert!(matches!(third.result, CommandResult::Rejected { .. }));
    assert_eq!(same, state);
}

#[test]
fn target_that_cannot_use_duel_is_rejected_without_consuming_the_skill() {
    let (state, owner, target, skill) = setup(true);
    let (same, result) = activate(state.clone(), owner, target, skill, 10);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(same, state);
    assert_eq!(same.skills.get(skill).unwrap().use_count, 0);
}
