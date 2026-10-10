//! Golden for XIANDING（限定包）批次三十八：
//! 孔武（dcsbkongwu，界呼厨儿）。

use engine_api::PlayerId;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCSBKONGWU: u32 = 3_614_891_392;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
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
    (tx, owner, first, second)
}

#[test]
fn package_exposes_batch38_skill_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let modifier = package
        .modifier_index
        .iter()
        .find(|m| m.skill_def_id == DCSBKONGWU && m.target == "draw_count")
        .expect("dcsbkongwu draw_count modifier");
    assert_eq!(modifier.mode, "add");
    assert_eq!(modifier.value, -1);
}

#[test]
fn dcsbkongwu_skill_can_be_attached() {
    let (mut tx, owner, _first, _second) = setup(20_260_874);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: engine_api::SkillDefId::new(DCSBKONGWU),
        }],
    )
    .unwrap();
    let has = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.def == engine_api::SkillDefId::new(DCSBKONGWU));
    assert!(has);
}
