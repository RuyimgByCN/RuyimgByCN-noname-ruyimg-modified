//! Coverage for the per-skill keyed storage map primitive: `storage_set` effect and
//! `storage_get` / `storage_has` queries. Backed by `SkillInstance.content_storage`.
//! Foundation for clan skills that record per-key state (clanbaichu's suit+type→trick map,
//! clanyuzhi's mark, etc.).

use engine_api::{
    Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

/// A skill that reads its own stored value and returns it via an if-branch observable through
/// storage_get. def id chosen to fit u32.
const STORE: u32 = 3_111_111_111;
const SOURCE: &str = concat!(
    "skill store_tagger(",
    "3111111111",
    ") locked {\n",
    "  trigger card_used always {\n",
    "    effect storage_set(self_skill, 7, 42);\n",
    "  }\n",
    "}\n",
);

#[test]
fn storage_set_get_has_roundtrip() {
    let pkg = content_compiler::compile_source(SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&pkg).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_809,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;

    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STORE),
    });
    run_effects(&mut tx, effects).unwrap();

    let skill = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == STORE)
        .unwrap()
        .id;

    // Before any write: storage_has(key=7) is false, storage_get returns 0.
    let has0 = query(
        tx.state(),
        &QueryRequest::StorageHas {
            skill,
            key: 7,
        },
        &mut QueryContext::new(),
    );
    assert!(matches!(has0, QueryResponse::Bool(false)));
    let get0 = query(
        tx.state(),
        &QueryRequest::StorageGet {
            skill,
            key: 7,
        },
        &mut QueryContext::new(),
    );
    assert!(matches!(get0, QueryResponse::Int(0)));

    // Write value=42 under key=7 via the primitive.
    run_effects(
        &mut tx,
        vec![EffectRequest::StorageSet {
            skill,
            key: 7,
            value: 42,
        }],
    )
    .unwrap();

    // After write: has=true, get=42. A different key is still absent.
    let has1 = query(
        tx.state(),
        &QueryRequest::StorageHas {
            skill,
            key: 7,
        },
        &mut QueryContext::new(),
    );
    assert!(matches!(has1, QueryResponse::Bool(true)));
    let get1 = query(
        tx.state(),
        &QueryRequest::StorageGet {
            skill,
            key: 7,
        },
        &mut QueryContext::new(),
    );
    assert!(matches!(get1, QueryResponse::Int(42)));
    let get_other = query(
        tx.state(),
        &QueryRequest::StorageGet {
            skill,
            key: 99,
        },
        &mut QueryContext::new(),
    );
    assert!(matches!(get_other, QueryResponse::Int(0)));

    // Overwrite the same key with a new value.
    run_effects(
        &mut tx,
        vec![EffectRequest::StorageSet {
            skill,
            key: 7,
            value: 100,
        }],
    )
    .unwrap();
    let get2 = query(
        tx.state(),
        &QueryRequest::StorageGet {
            skill,
            key: 7,
        },
        &mut QueryContext::new(),
    );
    assert!(matches!(get2, QueryResponse::Int(100)));
}
