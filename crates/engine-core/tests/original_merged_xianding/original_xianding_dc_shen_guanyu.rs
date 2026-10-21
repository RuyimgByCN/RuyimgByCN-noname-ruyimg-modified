//! Golden coverage for XIANDING 新杀神关羽【武神/武魂】(ABI 580).

use engine_api::{CardId, DomainEvent, PlayerId, SkillDefId, SkillId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{CardKind, GameState};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const CHARACTER: u32 = 2_564_129_342;
const DCWUSHEN: u32 = 3_292_885_385;
const DCWUHUN: u32 = 4_234_333_270;
const NIGHTMARE: u32 = 3_698_112_918;
const SOURCE: &str = concat!(
    include_str!("../../../../content/original/ported/xianding/dcwuhun.skill"),
    "\n",
    include_str!("../../../../content/original/ported/xianding/dcwushen.skill"),
);
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/xianding/xianding.npack.json");

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|id| {
            state
                .skills
                .get(*id)
                .is_some_and(|skill| skill.def.raw() == def)
        })
        .unwrap()
}

fn setup(def: u32) -> (Transaction, PlayerId, PlayerId, PlayerId, SkillId) {
    let package = content_compiler::compile_source_named("dc-shen-guanyu", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 3, CHARACTER as u64, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(std::iter::once(EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(def),
        }))
        .collect();
    run_effects(&mut tx, effects).unwrap();
    let skill = skill_id(tx.state(), players[0], def);
    (tx, players[0], players[1], players[2], skill)
}

fn mark_count(tx: &Transaction, player: PlayerId) -> i32 {
    tx.state()
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == NIGHTMARE)
        .map_or(0, |(_, count)| *count)
}

#[test]
fn package_publishes_exact_character_and_skill_contracts() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 587);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == CHARACTER)
        .unwrap();
    assert_eq!(character.name_key, "dc_shen_guanyu");
    assert_eq!(
        (
            character.sex.as_str(),
            character.group.as_str(),
            character.max_hp
        ),
        ("male", "shen", 5)
    );
    assert_eq!(character.skills, [DCWUSHEN, DCWUHUN]);

    let modifiers: Vec<_> = package
        .modifier_index
        .iter()
        .filter(|modifier| modifier.skill_def_id == DCWUSHEN)
        .map(|modifier| {
            (
                modifier.target.as_str(),
                modifier.selector.as_deref(),
                modifier.value,
            )
        })
        .collect();
    assert_eq!(
        modifiers,
        [
            ("card_name", Some("heart"), 1),
            ("card_nature", Some("heart"), 0),
            ("card_distance_unlimited", Some("heart"), 1),
            ("card_use_unlimited", Some("heart"), 1),
        ]
    );
    let mut contracts: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| matches!(trigger.skill_def_id, DCWUSHEN | DCWUHUN))
        .map(|trigger| {
            (
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
            )
        })
        .collect();
    contracts.sort_unstable();
    assert_eq!(
        contracts,
        [
            (DCWUSHEN, "card_used", "owner_source_heart_sha"),
            (
                DCWUSHEN,
                "damage",
                "owner_source_sha_damage_recorded_by_skill"
            ),
            (
                DCWUHUN,
                "damage",
                "owner_is_source_or_target_other_alive_per_damage_point"
            ),
            (DCWUHUN, "death", "owner_is_target"),
        ]
    );
}

#[test]
fn dcwushen_draws_only_for_targets_bound_to_the_exact_sha() {
    let (mut tx, owner, target, other, skill) = setup(DCWUSHEN);
    let armed = CardId::new(98_001);
    let unarmed = CardId::new(98_002);
    tx.state_mut()
        .resolution
        .register_sha_resolution(owner, armed, &[target, other], false);
    run_effects(
        &mut tx,
        vec![EffectRequest::RecordCurrentShaAllTargets {
            player: owner,
            card: armed,
            skill,
        }],
    )
    .unwrap();
    assert!(tx
        .state()
        .resolution
        .sha_skill_target_recorded(owner, armed, skill, target));
    assert!(!tx
        .state()
        .resolution
        .sha_skill_target_recorded(owner, unarmed, skill, target));

    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
            card: armed,
            card_kind: CardKind::Sha.tag(),
            card_flags: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 1);
}

#[test]
fn dcwuhun_marks_both_damage_directions_per_point_and_publishes_death_effect() {
    let (mut tx, owner, target, other, _) = setup(DCWUHUN);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: owner,
            target,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(mark_count(&tx, target), 2);

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: other,
            target: owner,
            amount: 3,
            is_lose_hp: false,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(mark_count(&tx, other), 3);

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PlayerKilled {
            player: owner,
            killer: PlayerId::NONE,
            hand_equip_cards: Vec::new(),
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.iter().any(|effect| matches!(
        effect,
        EffectRequest::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan {
            player,
            mark,
            ..
        } if *player == owner && mark.raw() == NIGHTMARE
    )));
}
