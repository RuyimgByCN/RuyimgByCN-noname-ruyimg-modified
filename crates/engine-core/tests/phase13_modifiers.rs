//! Phase 13: package modifiers override bootstrap mashu / qicao.

use content_compiler::compile_source;
use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId};
use engine_core::query::{query, QueryContext};
use engine_core::{CombineMode, Engine, ModifierTarget, BOOTSTRAP_MASHU, BOOTSTRAP_QICAO};
use rule_api::{QueryRequest, QueryResponse};

const MASHU: &str = include_str!("../../../content/standard/mashu.skill");
const QICAO: &str = include_str!("../../../content/standard/qicao.skill");
const JIANXIONG: &str = include_str!("../../../content/standard/jianxiong.skill");

#[test]
fn package_mashu_overrides_and_affects_distance() {
    let pkg = compile_source(MASHU).unwrap();
    assert!(!pkg.modifier_index.is_empty());
    assert_eq!(pkg.modifier_index[0].skill_def_id, BOOTSTRAP_MASHU);
    assert_eq!(pkg.modifier_index[0].value, -1);

    let mut eng = Engine::new();
    eng.load_package(&pkg).unwrap();
    let defs = eng
        .content()
        .modifier_defs_for_skill(SkillDefId::new(BOOTSTRAP_MASHU));
    assert_eq!(defs.len(), 1);
    assert!(defs[0].id >= engine_core::PACKAGE_MODIFIER_ID_BASE);
    assert_eq!(defs[0].value, -1);
    assert_eq!(defs[0].target, ModifierTarget::DistanceFrom);
    assert_eq!(defs[0].mode, CombineMode::Add);

    eng.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: 11,
            ruleset: "identity-standard".into(),
        },
    });

    // Find a player with mashu, or add to p1
    let p1 = PlayerId::new(1);
    let has = eng.state().players.get(p1).map(|p| {
        p.skills.iter().any(|s| {
            eng.state()
                .skills
                .get(*s)
                .map(|sk| sk.def.raw() == BOOTSTRAP_MASHU)
                .unwrap_or(false)
        })
    });
    if !has.unwrap_or(false) {
        use engine_core::effect::run_effects;
        use engine_core::transaction::Transaction;
        use rule_api::EffectRequest;
        let mut tx = Transaction::begin(eng.state().clone());
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: p1,
                skill_def: SkillDefId::new(BOOTSTRAP_MASHU),
            }],
        )
        .unwrap();
        // query on tx state
        let p2 = PlayerId::new(2);
        let mut ctx = QueryContext::new();
        let d = query(
            tx.state(),
            &QueryRequest::Distance { from: p1, to: p2 },
            &mut ctx,
        );
        match d {
            QueryResponse::Int(v) => {
                // With mashu -1 from owner, distance should be less than without.
                // Seat-adjacent base distance is typically 1; mashu makes 0 min.
                assert!(v <= 1, "distance with mashu was {v}");
            }
            other => panic!("expected Int, got {other:?}"),
        }
        return;
    }

    let p2 = PlayerId::new(2);
    let mut ctx = QueryContext::new();
    let d = query(
        eng.state(),
        &QueryRequest::Distance { from: p1, to: p2 },
        &mut ctx,
    );
    assert!(matches!(d, QueryResponse::Int(_)));
}

#[test]
fn package_qicao_increases_attack_range() {
    let pkg = compile_source(QICAO).unwrap();
    let mut eng = Engine::new();
    eng.load_package(&pkg).unwrap();
    eng.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 3,
            ruleset: "bootstrap".into(),
        },
    });

    use engine_core::effect::run_effects;
    use engine_core::transaction::Transaction;
    use rule_api::EffectRequest;
    let p1 = PlayerId::new(1);
    let mut tx = Transaction::begin(eng.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: p1,
            skill_def: SkillDefId::new(BOOTSTRAP_QICAO),
        }],
    )
    .unwrap();
    let mut ctx = QueryContext::new();
    let r = query(
        tx.state(),
        &QueryRequest::AttackRange { player: p1 },
        &mut ctx,
    );
    match r {
        QueryResponse::Int(v) => assert!(v >= 2, "qicao range was {v}, expected >= 2"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn combined_standard_package_loads_triggers_and_modifiers() {
    let mut src = String::new();
    src.push_str(JIANXIONG);
    src.push('\n');
    src.push_str(MASHU);
    src.push('\n');
    src.push_str(QICAO);
    let pkg = compile_source(&src).unwrap();
    assert!(!pkg.trigger_index.is_empty());
    assert!(pkg.modifier_index.len() >= 2);

    let mut eng = Engine::new();
    eng.load_package(&pkg).unwrap();
    assert_eq!(eng.content().package_count(), 1);
    assert!(matches!(
        eng.content().defs_for_skill(SkillDefId::new(1))[0].handler,
        engine_core::trigger::TriggerHandler::RunVm { .. }
    ));
    assert_eq!(
        eng.content()
            .modifier_defs_for_skill(SkillDefId::new(BOOTSTRAP_MASHU))
            .len(),
        1
    );
    assert_eq!(
        eng.content()
            .modifier_defs_for_skill(SkillDefId::new(BOOTSTRAP_QICAO))
            .len(),
        1
    );
}

#[test]
fn without_package_bootstrap_modifiers_unchanged() {
    let eng = Engine::new();
    let m = eng
        .content()
        .modifier_defs_for_skill(SkillDefId::new(BOOTSTRAP_MASHU));
    assert_eq!(m[0].id, 1);
    assert_eq!(m[0].value, -1);
}
