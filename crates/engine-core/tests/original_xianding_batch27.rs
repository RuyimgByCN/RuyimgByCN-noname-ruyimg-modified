//! Golden for XIANDING（限定包）批次二十七：
//! 施仇（twodcspshichou）、壮魄（zhuangpo）、剑出（sejianchu）。

use engine_api::{PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWODCSP_SHICHOU: u32 = 3_276_628_911;
const ZHUANGPO: u32 = 2_849_685_408;
const SEJIANCHU: u32 = 3_620_279_118;
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
fn package_exposes_batch27_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let shichou = package
        .view_as_index
        .iter()
        .find(|view_as| view_as.skill_def_id == TWODCSP_SHICHOU)
        .expect("twodcspshichou view_as");
    assert_eq!(shichou.as_kind, "sha");
    assert_eq!(shichou.window, "play_phase");
    assert_eq!(shichou.filter, "any");
    assert!(shichou.no_use_count);

    let zhuangpo = package
        .view_as_index
        .iter()
        .find(|view_as| view_as.skill_def_id == ZHUANGPO)
        .expect("zhuangpo view_as");
    assert_eq!(zhuangpo.as_kind, "juedou");
    assert_eq!(zhuangpo.window, "play_phase");

    let sejianchu = package
        .view_as_index
        .iter()
        .find(|view_as| view_as.skill_def_id == SEJIANCHU)
        .expect("sejianchu view_as");
    assert_eq!(sejianchu.as_kind, "sha");
    assert_eq!(sejianchu.window, "play_phase");
    assert_eq!(sejianchu.filter, "card_shan");
}

#[test]
fn batch27_skills_can_be_attached() {
    let (mut tx, owner, _first, _second) = setup(20_260_856);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(TWODCSP_SHICHOU),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(ZHUANGPO),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(SEJIANCHU),
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
    assert!(defs.contains(&TWODCSP_SHICHOU));
    assert!(defs.contains(&ZHUANGPO));
    assert!(defs.contains(&SEJIANCHU));
}
