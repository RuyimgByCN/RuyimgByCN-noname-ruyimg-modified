//! Golden for 四象「忠事」比较伤害来源与目标横置状态的加伤语义。

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::intent::{resolve_damage_intent, DamageIntent};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const STDZHONGSHI: u32 = 3_843_853_491;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/sixiang/sixiang.npack.json");

fn isolated() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDZHONGSHI && skill.locked));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(STDZHONGSHI as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: STDZHONGSHI as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let players = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(STDZHONGSHI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, players[0], players[1], players[2])
}

fn set_linked(tx: &mut Transaction, player: PlayerId, linked: bool) {
    let marks = &mut tx.state_mut().players.get_mut(player).unwrap().marks;
    marks.retain(|(mark, _)| *mark != engine_core::content::MARK_CHAIN);
    if linked {
        marks.push((engine_core::content::MARK_CHAIN, 1));
        marks.sort_by_key(|(mark, _)| *mark);
    }
}

fn resolved_amount(tx: &Transaction, source: PlayerId, target: PlayerId) -> u32 {
    resolve_damage_intent(
        tx.state(),
        DamageIntent {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
            cancelled: false,
        },
    )
    .amount
}

#[test]
fn bonus_applies_only_while_source_and_target_linked_states_differ() {
    let (mut tx, source, target, _) = isolated();

    assert_eq!(resolved_amount(&tx, source, target), 1);
    set_linked(&mut tx, target, true);
    assert_eq!(resolved_amount(&tx, source, target), 2);
    set_linked(&mut tx, source, true);
    assert_eq!(resolved_amount(&tx, source, target), 1);
    set_linked(&mut tx, target, false);
    assert_eq!(resolved_amount(&tx, source, target), 2);
}

#[test]
fn source_skill_does_not_modify_unrelated_or_sourceless_damage() {
    let (mut tx, source, target, third) = isolated();
    set_linked(&mut tx, target, true);
    set_linked(&mut tx, third, true);

    assert_eq!(resolved_amount(&tx, source, target), 2);
    assert_eq!(resolved_amount(&tx, third, target), 1);
    assert_eq!(resolved_amount(&tx, PlayerId::NONE, target), 1);
}

#[test]
fn owning_the_skill_on_the_damage_target_does_not_add_damage() {
    let (mut tx, source, target, _) = isolated();
    let source_skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == source && skill.def.raw() == STDZHONGSHI)
        .unwrap()
        .id;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::RemoveSkill {
                skill: source_skill,
            },
            EffectRequest::AddSkill {
                player: target,
                skill_def: SkillDefId::new(STDZHONGSHI),
            },
        ],
    )
    .unwrap();
    set_linked(&mut tx, target, true);

    assert_eq!(resolved_amount(&tx, source, target), 1);
}
