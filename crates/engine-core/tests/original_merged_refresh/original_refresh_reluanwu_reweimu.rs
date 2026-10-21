//! Golden for REFRESH 界贾诩「界乱武」「界帷幕」及 OFFLINE 复用闭包。

use engine_api::{DomainEvent, SkillDefId, RULE_VM_ABI_VERSION};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RE_JIAXU: u32 = 3_147_472_843;
const SCL_JIAXU: u32 = 4_227_787_579;
const REWANSHA: u32 = 3_382_466_194;
const SCLWANSHA: u32 = 3_972_375_066;
const RELUANWU: u32 = 2_626_196_598;
const REWEIMU: u32 = 2_589_015_751;
const RELUANWU_SOURCE: &str =
    include_str!("../../../../content/original/ported/refresh/reluanwu.skill");
const REWEIMU_SOURCE: &str =
    include_str!("../../../../content/original/ported/refresh/reweimu.skill");
const REFRESH: &[u8] =
    include_bytes!("../../../../content/original/ported/refresh/refresh.npack.json");
const OFFLINE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

#[test]
fn packages_close_both_jiaxu_and_reweimu_only_prevents_damage_in_owners_turn() {
    let refresh = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    let offline = content_schema::ContentPackage::from_json_bytes(OFFLINE).unwrap();
    assert_eq!(refresh.manifest.rule_vm_abi_version, 723);
    assert!(refresh.character_index.iter().any(|character| {
        character.id == RE_JIAXU && character.skills == [REWANSHA, RELUANWU, REWEIMU]
    }));
    assert!(offline.character_index.iter().any(|character| {
        character.id == SCL_JIAXU && character.skills == [SCLWANSHA, RELUANWU, REWEIMU]
    }));
    assert!(offline
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "refresh"));

    let package = content_compiler::compile_source_named(
        "reluanwu-reweimu-test",
        &format!("{RELUANWU_SOURCE}\n{REWEIMU_SOURCE}"),
    )
    .unwrap();
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == REWEIMU
            && modifier.target == "card_target_cancel"
            && modifier.selector.as_deref() == Some("black_trick")
    }));

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 2_026_080_101, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    let players = tx.state().players.alive_by_seat();
    let source = players[0];
    let owner = players[1];
    let old_skills = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old_skills).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(REWEIMU),
        }],
    )
    .unwrap();

    let hp = tx.state().players.get(owner).unwrap().hp;
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
    assert_eq!(tx.state().hand_count(owner), hand);

    tx.state_mut().turn.current = owner;
    let hand = tx.state().hand_count(owner);
    let events = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 2,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
    assert_eq!(tx.state().hand_count(owner), hand + 4);
    assert!(events.iter().any(|event| matches!(
        event,
        DomainEvent::DamagePrevented {
            source: event_source,
            target,
            current,
        } if *event_source == source && *target == owner && *current == owner
    )));
}
