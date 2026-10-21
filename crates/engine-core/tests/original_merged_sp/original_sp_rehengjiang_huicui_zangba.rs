//! Golden coverage for SP 臧霸「横江」rewritten from legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{PlayerId, SkillDefId, SkillId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{query, QueryContext};
use engine_core::state::{turn_flag, GameState};
use engine_core::transaction::Transaction;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const REHENGJIANG: u32 = 2_552_200_673;
const HAND_HELPER: u32 = 4_200_000_582;
const DRAW_HELPER: u32 = 4_200_000_583;
const ZANGBA: u32 = 1_112_200_336;
const SOURCE: &str = include_str!("../../../../content/original/ported/sp/rehengjiang.skill");
const SP_PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");
const HUICUI_PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/huicui/huicui.npack.json");

fn setup() -> (Transaction, PlayerId, PlayerId, SkillId) {
    let package = compile_source_named("sp-rehengjiang-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803_01, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let actor = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(REHENGJIANG),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = actor;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == REHENGJIANG)
        .unwrap()
        .id;
    (tx, owner, actor, skill)
}

fn activate(tx: &mut Transaction, owner: PlayerId, skill: SkillId) {
    run_effects(
        tx,
        vec![EffectRequest::ResolveRehengjiang {
            player: owner,
            actor: PlayerId::NONE,
            action: 0,
            hand_limit_def: SkillDefId::new(HAND_HELPER),
            draw_def: SkillDefId::new(DRAW_HELPER),
            skill,
        }],
    )
    .unwrap();
}

fn hand_limit(tx: &Transaction, player: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::HandLimit { player },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected hand-limit response: {other:?}"),
    }
}

fn draw_helper(tx: &Transaction, owner: PlayerId) -> SkillId {
    tx.state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DRAW_HELPER)
        .unwrap()
        .id
}

fn settle(tx: &mut Transaction, owner: PlayerId, actor: PlayerId) {
    let helper = draw_helper(tx, owner);
    run_effects(
        tx,
        vec![EffectRequest::ResolveRehengjiang {
            player: owner,
            actor,
            action: 1,
            hand_limit_def: SkillDefId::new(HAND_HELPER),
            draw_def: SkillDefId::new(DRAW_HELPER),
            skill: helper,
        }],
    )
    .unwrap();
}

#[test]
fn each_damage_point_stacks_and_no_discard_draws_activation_count() {
    let (mut tx, owner, actor, skill) = setup();
    let base_limit = hand_limit(&tx, actor);
    activate(&mut tx, owner, skill);
    activate(&mut tx, owner, skill);
    assert_eq!(hand_limit(&tx, actor), base_limit - 2);

    let hand_helper = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|instance| instance.owner == actor && instance.def.raw() == HAND_HELPER)
        .unwrap();
    assert_eq!(hand_helper.temporary_until_turn_end_of, actor);
    let before = tx.state().hand_count(owner);
    settle(&mut tx, owner, actor);
    assert_eq!(tx.state().hand_count(owner), before + 2);
    assert!(tx
        .state()
        .skills
        .get(draw_helper(&tx, owner))
        .unwrap()
        .state_blob
        .is_empty());
}

#[test]
fn discarding_in_discard_phase_reduces_reward_to_one() {
    let (mut tx, owner, actor, skill) = setup();
    activate(&mut tx, owner, skill);
    activate(&mut tx, owner, skill);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::DISCARD_PHASE_DISCARDED, 1);
    let before = tx.state().hand_count(owner);
    settle(&mut tx, owner, actor);
    assert_eq!(tx.state().hand_count(owner), before + 1);
}

#[test]
fn huicui_zangba_reuses_the_canonical_sp_skill() {
    let sp = content_schema::ContentPackage::from_json_bytes(SP_PACKAGE).unwrap();
    let huicui = content_schema::ContentPackage::from_json_bytes(HUICUI_PACKAGE).unwrap();
    assert!(sp.manifest.rule_vm_abi_version >= 647);
    assert!(sp
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == REHENGJIANG));
    assert!(huicui
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "sp"));
    let zangba = huicui
        .character_index
        .iter()
        .find(|character| character.id == ZANGBA)
        .unwrap();
    assert_eq!(zangba.name_key, "zangba");
    assert_eq!(zangba.skills, [REHENGJIANG]);
}
