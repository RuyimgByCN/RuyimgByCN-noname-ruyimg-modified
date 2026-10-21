//! Golden coverage for SP「豹变」and XIANDING 新杀夏侯霸 (ABI 575).

use engine_api::{DomainEvent, PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const REBAOBIAN: u32 = 4_258_572_312;
const TIAOXIN: u32 = 695_232_805;
const OLPAOXIAO: u32 = 2_829_854_661;
const XINSHENSU: u32 = 2_561_255_694;
const DC_XIAHOUBA: u32 = 931_835_034;
const SOURCE: &str = concat!(
    include_str!("../../../../content/original/ported/sp/rebaobian.skill"),
    "\n",
    include_str!("../../../../content/original/ported/shenhua/jiangwei.skill"),
    "\n",
    include_str!("../../../../content/original/ported/refresh/olpaoxiao.skill"),
    "\n",
    include_str!("../../../../content/original/ported/shenhua/xinshensu.skill"),
);
const SP_PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");
const XIANDING_PACKAGE: &[u8] =
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

fn damage(state: GameState, source: PlayerId, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source,
            target: owner,
            amount: 1,
            nature: 0,
            is_lose_hp: false,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn setup() -> (GameState, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("rebaobian-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 2, REBAOBIAN as u64, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let source = seats[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(REBAOBIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, source)
}

#[test]
fn packages_publish_the_skill_and_complete_character_closure() {
    let compiled = content_compiler::compile_source_named("rebaobian", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 575);
    let sp = content_schema::ContentPackage::from_json_bytes(SP_PACKAGE).unwrap();
    let skill = sp
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == REBAOBIAN)
        .unwrap();
    assert_eq!(skill.name, "rebaobian");
    assert!(skill.locked);
    let expected = compiled
        .modules
        .iter()
        .find(|module| module.name == "rebaobian_damage")
        .unwrap();
    let actual = sp
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);

    let xianding = content_schema::ContentPackage::from_json_bytes(XIANDING_PACKAGE).unwrap();
    let character = xianding
        .character_index
        .iter()
        .find(|character| character.id == DC_XIAHOUBA)
        .unwrap();
    assert_eq!(character.skills, [REBAOBIAN]);
}

#[test]
fn each_damage_grants_the_first_missing_derivation_in_fixed_order() {
    let (state, owner, source) = setup();
    let state = damage(state, source, owner);
    assert!(has_skill(&state, owner, TIAOXIN));
    assert!(!has_skill(&state, owner, OLPAOXIAO));

    let state = damage(state, source, owner);
    assert!(has_skill(&state, owner, OLPAOXIAO));
    assert!(!has_skill(&state, owner, XINSHENSU));

    let state = damage(state, source, owner);
    assert!(has_skill(&state, owner, XINSHENSU));
    let skill_count = state.players.get(owner).unwrap().skills.len();

    let state = damage(state, source, owner);
    assert_eq!(state.players.get(owner).unwrap().skills.len(), skill_count);
}

#[test]
fn existing_earlier_skill_is_skipped_without_player_choice() {
    let (state, owner, source) = setup();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TIAOXIN),
        }],
    )
    .unwrap();
    let state = damage(tx.commit().state, source, owner);
    assert!(has_skill(&state, owner, OLPAOXIAO));
    assert!(!state.has_pending_choice());
}
