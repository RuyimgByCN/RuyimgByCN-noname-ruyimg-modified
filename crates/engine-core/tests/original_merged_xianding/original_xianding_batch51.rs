//! Golden for XIANDING（限定包）批次五十一：
//! 佩娘（dcpeiniang）。

use engine_api::{PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCPEINIANG: u32 = 3_794_638_462;
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
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

#[test]
fn package_exposes_peiniang_jiu_view_as_and_unlimited() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let view_as = package
        .view_as_index
        .iter()
        .find(|v| v.skill_def_id == DCPEINIANG)
        .expect("dcpeiniang view_as");
    assert_eq!(view_as.as_kind, "jiu");
    assert_eq!(view_as.window, "play_phase");
    assert_eq!(view_as.source_zone.as_deref(), Some("hand_equip"));

    let unlimited = package
        .modifier_index
        .iter()
        .find(|m| m.skill_def_id == DCPEINIANG && m.target == "card_use_unlimited")
        .expect("dcpeiniang card_use_unlimited");
    assert_eq!(unlimited.selector.as_deref(), Some("jiu"));
}

#[test]
fn peiniang_can_be_attached() {
    let (mut tx, owner, _first, _second) = setup(20_260_908);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCPEINIANG),
        }],
    )
    .unwrap();
    let has = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == DCPEINIANG);
    assert!(has);
}
