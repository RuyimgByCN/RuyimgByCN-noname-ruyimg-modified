//! Golden coverage for REFRESH「强袭」and MOBILE 界典韦 rewritten from legacy JavaScript.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
    SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const REQIANGXI: u32 = 2_185_312_161;
const QIANGXIX: u32 = 3_151_452_629;
const RE_DIANWEI: u32 = 598_273_341;
const SOURCE: &str = include_str!("../../../../content/original/ported/refresh/reqiangxi.skill");
const QIANGXIX_SOURCE: &str =
    include_str!("../../../../content/original/ported/shenhua/qiangxix.skill");
const REFRESH: &[u8] =
    include_bytes!("../../../../content/original/ported/refresh/refresh.npack.json");
const MOBILE: &[u8] =
    include_bytes!("../../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    reqiangxi: SkillId,
    qiangxix: SkillId,
}

fn setup() -> Setup {
    let source = format!("{SOURCE}\n{QIANGXIX_SOURCE}");
    let package = content_compiler::compile_source_named("reqiangxi-test", &source).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 292);
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, u64::from(REQIANGXI), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(REQIANGXI),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(QIANGXIX),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.max_hp = 12;
        participant.hp = 12;
    }
    let skill = |definition| {
        tx.state()
            .players
            .get(owner)
            .unwrap()
            .skills
            .iter()
            .copied()
            .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == definition)
            .unwrap()
    };
    let reqiangxi = skill(REQIANGXI);
    let qiangxix = skill(QIANGXIX);
    Setup {
        state: tx.commit().state,
        owner,
        target: players[1],
        reqiangxi,
        qiangxix,
    }
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    target: PlayerId,
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
                card_targets: Vec::new(),
            },
        },
    )
}

fn enter_new_play_phase(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Play.to_u8(),
            },
        ],
    )
    .unwrap();
    tx.commit().state
}

#[test]
fn packages_publish_play_phase_target_scope_and_close_re_dianwei() {
    let refresh = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    let mobile = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    assert!(refresh.manifest.rule_vm_abi_version >= 298);
    let targets = refresh
        .active_index
        .iter()
        .find(|active| active.skill_def_id == REQIANGXI)
        .and_then(|active| active.targets.as_ref())
        .unwrap();
    assert!(!targets.allow_self);
    assert!(targets.in_attack_range);
    assert!(targets.unused_by_skill_this_play_phase);
    assert!(!targets.unused_by_skill_this_turn);
    assert!(mobile
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "refresh"));
    assert_eq!(
        mobile
            .character_index
            .iter()
            .find(|character| character.id == RE_DIANWEI)
            .unwrap()
            .skills,
        [REQIANGXI]
    );
    assert!(!serde_json::to_string(&(refresh, mobile))
        .unwrap()
        .contains(".js"));
}

#[test]
fn phase_scope_resets_without_clearing_existing_turn_scope() {
    let setup = setup();
    let (state, first) = activate(setup.state, setup.owner, setup.reqiangxi, setup.target, 1);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    let (same, repeated) = activate(state.clone(), setup.owner, setup.reqiangxi, setup.target, 2);
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
    assert_eq!(same, state);

    let state = enter_new_play_phase(state, setup.owner);
    let (state, next_phase) = activate(state, setup.owner, setup.reqiangxi, setup.target, 3);
    assert!(matches!(next_phase.result, CommandResult::Accepted { .. }));

    let (state, turn_scoped) = activate(state, setup.owner, setup.qiangxix, setup.target, 4);
    assert!(matches!(turn_scoped.result, CommandResult::Accepted { .. }));
    let state = enter_new_play_phase(state, setup.owner);
    let (_, still_used) = activate(state, setup.owner, setup.qiangxix, setup.target, 5);
    assert!(matches!(still_used.result, CommandResult::Rejected { .. }));
}
