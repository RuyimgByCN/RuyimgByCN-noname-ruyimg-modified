//! Golden coverage for XIANDING DC 糜夫人【闺秀/存嗣】(ABI 578).

use engine_api::{PlayerId, SkillDefId, SkillId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DC_MIFUREN: u32 = 3_357_817_038;
const DCGUIXIU: u32 = 2_370_422_642;
const DCCUNSI: u32 = 4_233_990_625;
const DCYONGJUE: u32 = 2_297_054_445;
const DRAW_HURTS: u32 = 4_000_000_078;
const SOURCE: &str = concat!(
    include_str!("../../../../content/original/ported/xianding/dcguixiu.skill"),
    "\n",
    include_str!("../../../../content/original/ported/xianding/dccunsi.skill"),
    "\n",
    include_str!("../../../../content/original/ported/xianding/dcyongjue.skill"),
    "\n",
    "skill draw_hurts(4000000078) locked { trigger cards_drawn owner_is_target { effect lose_hp(self, 1); } }",
);
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/xianding/xianding.npack.json");

fn has_skill(state: &GameState, owner: PlayerId, def: u32) -> bool {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
        .unwrap()
}

fn setup(with_guixiu: bool, draw_hurts: bool) -> (Transaction, PlayerId, PlayerId, SkillId) {
    let package = content_compiler::compile_source_named("dc-mifuren-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 2, DC_MIFUREN as u64, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let target = seats[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DCCUNSI),
    });
    if with_guixiu {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCGUIXIU),
        });
    }
    if draw_hurts {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DRAW_HURTS),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    let cunsi = skill_id(tx.state(), owner, DCCUNSI);
    (tx, owner, target, cunsi)
}

fn activate(tx: &mut Transaction, owner: PlayerId, target: PlayerId, skill: SkillId) {
    run_effects(
        tx,
        vec![EffectRequest::ResolveDccunsi {
            player: owner,
            target,
            guixiu_def: SkillDefId::new(DCGUIXIU),
            yongjue_def: SkillDefId::new(DCYONGJUE),
            stage: 0,
            skill,
        }],
    )
    .unwrap();
}

#[test]
fn package_publishes_exact_dc_mifuren_closure() {
    let compiled = content_compiler::compile_source_named("dc-mifuren", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 578);
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 587);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == DC_MIFUREN)
        .unwrap();
    assert_eq!(character.name_key, "dc_mifuren");
    assert_eq!(character.sex, "female");
    assert_eq!(character.group, "shu");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [DCGUIXIU, DCCUNSI]);
    let expected = compiled
        .modules
        .iter()
        .find(|module| module.name == "dccunsi_active_play_phase_0")
        .unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn targeting_self_grants_yongjue_without_draw_and_recovers_if_wounded() {
    let (mut tx, owner, _target, skill) = setup(true, false);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    let hand = tx.state().hand_count(owner);
    activate(&mut tx, owner, owner, skill);
    assert!(has_skill(tx.state(), owner, DCYONGJUE));
    assert_eq!(tx.state().hand_count(owner), hand);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 3);
}

#[test]
fn other_target_draw_chain_finishes_before_guixiu_recovery() {
    let (mut tx, owner, target, skill) = setup(true, true);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    let hand = tx.state().hand_count(owner);
    activate(&mut tx, owner, target, skill);
    assert!(has_skill(tx.state(), target, DCYONGJUE));
    assert_eq!(tx.state().hand_count(owner), hand + 2);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 2);
}

#[test]
fn missing_guixiu_does_not_recover() {
    let (mut tx, owner, target, skill) = setup(false, false);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    activate(&mut tx, owner, target, skill);
    assert!(has_skill(tx.state(), target, DCYONGJUE));
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 2);
}
