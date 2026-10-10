//! Golden for 神话再临蔡文姬「断肠」rewritten from legacy JS.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const DUANCHANG: u32 = 2_554_836_650;
const KUANGGU: u32 = 3_700_078_390;
const WEIMU: u32 = 4_145_129_024;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/duanchang.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

fn started(seed: u64) -> Transaction {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let mut tx = Transaction::begin(engine.state().clone());
    let existing = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, existing).unwrap();
    tx
}

fn grant(tx: &mut Transaction, player: PlayerId, skill_def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(skill_def),
        }],
    )
    .unwrap();
}

#[test]
fn package_exposes_exact_forced_death_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 671);
    assert_eq!(package.skill_index.len(), 201);
    assert_eq!(package.character_index.len(), 48);
    assert_eq!(package.trigger_index.len(), 158);
    assert_eq!(package.modifier_index.len(), 56);
    assert_eq!(package.active_index.len(), 25);
    assert_eq!(package.modules.len(), 183);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == DUANCHANG && skill.locked));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DUANCHANG)
        .unwrap();
    assert_eq!(trigger.event, "death");
    assert_eq!(trigger.filter, "owner_is_target_source_alive");
    assert!(!trigger.optional);

    let compiled = content_compiler::compile_source_named("shenhua", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 159);
    assert_eq!(
        package
            .modules
            .iter()
            .find(|module| module.name == "duanchang_death"),
        compiled.modules.first()
    );
}

#[test]
fn death_removes_every_current_skill_only_from_a_living_killer() {
    let mut tx = started(DUANCHANG as u64);
    let killer = tx.state().turn.current;
    let victim = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != killer)
        .unwrap();
    grant(&mut tx, victim, DUANCHANG);
    grant(&mut tx, killer, KUANGGU);
    grant(&mut tx, killer, WEIMU);

    run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: victim,
            killer,
        }],
    )
    .unwrap();
    assert!(tx.state().players.get(killer).unwrap().skills.is_empty());

    let mut tx = started(DUANCHANG as u64 + 1);
    let other = tx.state().turn.current;
    let victim = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != other)
        .unwrap();
    grant(&mut tx, victim, DUANCHANG);
    grant(&mut tx, other, KUANGGU);
    run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: victim,
            killer: PlayerId::NONE,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(other).unwrap().skills.len(), 1);
}
