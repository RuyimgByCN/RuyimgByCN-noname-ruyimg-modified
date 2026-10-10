//! Golden for MOBILE rewritten armor link protection.

use engine_api::{
    Command, CommandKind, ExpectedRevision, IdempotencyKey, MarkDefId, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const RW_TENGJIA3: u32 = 4_247_798_542;
const RW_MINGUANGKAI_LINK: u32 = 2_481_253_205;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn loaded(skill_def: Option<u32>) -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(RW_TENGJIA3 as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: RW_TENGJIA3 as u64,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let removals = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, removals).unwrap();
    if let Some(skill_def) = skill_def {
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(skill_def),
            }],
        )
        .unwrap();
    }
    (tx, owner)
}

fn is_linked(tx: &Transaction, player: PlayerId) -> bool {
    tx.state()
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0)
}

fn add_chain(tx: &mut Transaction, player: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::AddMark {
            player,
            mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
            delta: 1,
        }],
    )
    .unwrap();
}

#[test]
fn package_exposes_locked_armor_link_modifiers() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    for skill_def in [RW_MINGUANGKAI_LINK, RW_TENGJIA3] {
        assert!(package
            .skill_index
            .iter()
            .any(|skill| skill.skill_def_id == skill_def && skill.locked));
        let modifiers: Vec<_> = package
            .modifier_index
            .iter()
            .filter(|modifier| modifier.skill_def_id == skill_def)
            .collect();
        assert_eq!(modifiers.len(), 1);
        assert_eq!(modifiers[0].target, "armor_link_gain_cancel");
        assert_eq!(modifiers[0].value, 1);
    }
}

#[test]
fn active_armor_blocks_only_unlinked_to_linked_transition() {
    for skill_def in [RW_MINGUANGKAI_LINK, RW_TENGJIA3] {
        let (mut protected, owner) = loaded(Some(skill_def));
        assert!(engine_core::query::armor_link_gain_cancelled_for(
            protected.state(),
            owner
        ));
        add_chain(&mut protected, owner);
        assert!(!is_linked(&protected, owner));
    }

    let (mut ordinary, owner) = loaded(None);
    add_chain(&mut ordinary, owner);
    assert!(is_linked(&ordinary, owner));
    run_effects(
        &mut ordinary,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(RW_TENGJIA3),
        }],
    )
    .unwrap();
    run_effects(
        &mut ordinary,
        vec![EffectRequest::RemoveMark {
            player: owner,
            mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
            amount: 0,
        }],
    )
    .unwrap();
    assert!(!is_linked(&ordinary, owner));
}
