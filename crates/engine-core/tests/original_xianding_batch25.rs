//! Golden for XIANDING（限定包）批次二十五：
//! 义指（dcsbyizhi，界刘璋主公技）。

use engine_api::{PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCSBYIZHI: u32 = 3_294_916_313;
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
fn package_exposes_batch25_skill_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let view_as = package
        .view_as_index
        .iter()
        .find(|view_as| view_as.skill_def_id == DCSBYIZHI)
        .expect("dcsbyizhi view_as");
    assert_eq!(view_as.as_kind, "wugu");
    assert_eq!(view_as.window, "play_phase");
    assert_eq!(view_as.filter, "any");
    assert_eq!(view_as.source_zone, None); // from hand 为默认，序列化省略
    assert_eq!(view_as.uses_per_turn, Some(1));

    let skill = package
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == DCSBYIZHI)
        .expect("dcsbyizhi skill");
    assert!(skill.lord_only, "义指必须是主公技");
}

#[test]
fn dcsbyizhi_skill_can_be_attached() {
    let (mut tx, owner, _first, _second) = setup(20_260_854);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCSBYIZHI),
        }],
    )
    .unwrap();
    let has = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.def == SkillDefId::new(DCSBYIZHI));
    assert!(has);
}
