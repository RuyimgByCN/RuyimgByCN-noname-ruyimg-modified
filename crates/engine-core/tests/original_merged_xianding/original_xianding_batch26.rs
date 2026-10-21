//! Golden for XIANDING（限定包）批次二十六：
//! 溃击（kuiji）、膂力·闪（lvli4）、膂力·无懈（lvli5）。

use engine_api::PlayerId;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const KUIJI: u32 = 2_248_505_393;
const LVLI4: u32 = 3_495_185_261;
const LVLI5: u32 = 2_862_723_587;
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
fn package_exposes_batch26_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let kuiji = package
        .view_as_index
        .iter()
        .find(|view_as| view_as.skill_def_id == KUIJI)
        .expect("kuiji view_as");
    assert_eq!(kuiji.as_kind, "bingliang");
    assert_eq!(kuiji.window, "play_phase");
    assert_eq!(kuiji.filter, "black_basic_equip");
    assert_eq!(kuiji.uses_per_turn, Some(1));

    let lvli4 = package
        .view_as_index
        .iter()
        .find(|view_as| view_as.skill_def_id == LVLI4)
        .expect("lvli4 view_as");
    assert_eq!(lvli4.as_kind, "shan");
    assert_eq!(lvli4.window, "any");
    assert_eq!(lvli4.filter, "any");

    let lvli5 = package
        .view_as_index
        .iter()
        .find(|view_as| view_as.skill_def_id == LVLI5)
        .expect("lvli5 view_as");
    assert_eq!(lvli5.as_kind, "wuxie");
    assert_eq!(lvli5.window, "any");
    assert_eq!(lvli5.filter, "any");
}

#[test]
fn batch26_skills_can_be_attached() {
    let (mut tx, owner, _first, _second) = setup(20_260_855);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: engine_api::SkillDefId::new(KUIJI),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: engine_api::SkillDefId::new(LVLI4),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: engine_api::SkillDefId::new(LVLI5),
            },
        ],
    )
    .unwrap();
    let defs: Vec<u32> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| skill.def.raw())
        .collect();
    assert!(defs.contains(&KUIJI));
    assert!(defs.contains(&LVLI4));
    assert!(defs.contains(&LVLI5));
}
