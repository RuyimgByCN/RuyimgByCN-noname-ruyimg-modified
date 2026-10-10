//! Phase 12: load DSL package → override bootstrap jianxiong with RunVm → damage draws.

use content_compiler::compile_source;
use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::{Engine, BOOTSTRAP_JIANXIONG};
use rule_api::EffectRequest;

const JIANXIONG: &str = include_str!("../../../content/standard/jianxiong.skill");

#[test]
fn package_overrides_bootstrap_with_exact_vm_trigger() {
    let pkg = compile_source(JIANXIONG).unwrap();
    assert!(!pkg.modules.is_empty());
    assert!(!pkg.trigger_index.is_empty());

    let mut eng = Engine::new();
    eng.load_package(&pkg).unwrap();
    assert_eq!(eng.content().package_count(), 1);

    // Skill def 1 now resolves to RunVm
    let defs = eng
        .content()
        .defs_for_skill(SkillDefId::new(BOOTSTRAP_JIANXIONG));
    assert_eq!(defs.len(), 1);
    assert!(matches!(
        defs[0].handler,
        engine_core::trigger::TriggerHandler::RunVm { .. }
    ));

    let r = eng.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 7,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(matches!(r, CommandResult::Accepted { .. }));

    // Characters are shuffled — grant jianxiong explicitly so the test is stable.
    let mut tx = Transaction::begin(eng.state().clone());
    let owner = PlayerId::new(1);
    for player in [owner, PlayerId::new(2)] {
        for skill in tx.state().players.get(player).unwrap().skills.clone() {
            run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
        }
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(BOOTSTRAP_JIANXIONG),
        }],
    )
    .unwrap();

    let hand_before = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::new(2),
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();

    assert_eq!(tx.state().hand_count(owner), hand_before);
    assert!(!tx.state().has_pending_choice());
}

#[test]
fn raw_bootstrap_also_ignores_cardless_damage() {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 3, "bootstrap").unwrap();
    // grant skill manually if not on character — use player 1
    let p1 = PlayerId::new(1);
    // find any player with jianxiong or add skill
    let owner = tx
        .state()
        .players
        .iter_live()
        .find_map(|p| {
            p.skills.iter().find_map(|sid| {
                tx.state()
                    .skills
                    .get(*sid)
                    .filter(|s| s.def.raw() == BOOTSTRAP_JIANXIONG)
                    .map(|_| p.id)
            })
        })
        .unwrap_or(p1);

    if owner == p1
        && !tx
            .state()
            .players
            .get(p1)
            .map(|p| {
                p.skills.iter().any(|s| {
                    tx.state()
                        .skills
                        .get(*s)
                        .map(|sk| sk.def.raw() == BOOTSTRAP_JIANXIONG)
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
    {
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: p1,
                skill_def: SkillDefId::new(BOOTSTRAP_JIANXIONG),
            }],
        )
        .unwrap();
    }

    let owner = tx
        .state()
        .players
        .iter_live()
        .find_map(|p| {
            p.skills.iter().find_map(|sid| {
                tx.state()
                    .skills
                    .get(*sid)
                    .filter(|s| s.def.raw() == BOOTSTRAP_JIANXIONG)
                    .map(|_| p.id)
            })
        })
        .expect("jianxiong owner");

    let hand_before = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::new(2),
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand_before);
}
