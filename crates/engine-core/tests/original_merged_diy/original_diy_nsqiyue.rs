//! Golden coverage for DIY「骑钺」and the generic character-state trigger.

use engine_api::{
    Command, CommandKind, ExpectedRevision, IdempotencyKey, MarkDefId, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const NSQIYUE: u32 = 2_381_587_208;
const NSXUEZHU: u32 = 2_233_994_945;
const NS_ZHANGWEI: u32 = 2_299_280_554;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/diy/diy.npack.json");

fn loaded() -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(NSQIYUE as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: NSQIYUE as u64,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    let removals = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, removals).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NSQIYUE),
        }],
    )
    .unwrap();
    (tx, owner, target)
}

#[test]
fn committed_package_declares_nsqiyue_and_closes_zhangwei() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 684);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == NSQIYUE && skill.locked));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == NSQIYUE)
        .unwrap();
    assert_eq!(trigger.event, "character_state_changed");
    assert_eq!(trigger.filter, "owner_alive");
    assert!(!trigger.optional);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == NS_ZHANGWEI)
        .unwrap();
    assert_eq!(character.name_key, "ns_zhangwei");
    assert_eq!(character.skills, vec![NSQIYUE, NSXUEZHU]);
}

#[test]
fn turning_over_any_character_draws_one_card() {
    let (mut tx, owner, target) = loaded();
    let before = tx.state().hand_count(owner);
    run_effects(&mut tx, vec![EffectRequest::TurnOver { player: target }]).unwrap();
    assert!(tx.state().players.get(target).unwrap().turned_over);
    assert_eq!(tx.state().hand_count(owner), before + 1);
}

#[test]
fn only_actual_link_state_transitions_draw() {
    let (mut tx, owner, target) = loaded();
    let chain = MarkDefId::new(engine_core::content::MARK_CHAIN);
    let unrelated = MarkDefId::new(97_531_246);
    let before = tx.state().hand_count(owner);

    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: target,
            mark: chain,
            delta: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 1);

    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: target,
            mark: chain,
            delta: 1,
        }],
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::RemoveMark {
            player: target,
            mark: chain,
            amount: 1,
        }],
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: target,
            mark: unrelated,
            delta: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 1);

    run_effects(
        &mut tx,
        vec![EffectRequest::RemoveMark {
            player: target,
            mark: chain,
            amount: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 2);
}
