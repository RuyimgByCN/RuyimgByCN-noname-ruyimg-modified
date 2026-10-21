//! 第一百六十八批：offline「探虎·距离子技」（tanhu3）新变体重写。
//! 上游：任何角色对「探虎」标记（tanhu2 技能）持有者无距离限制。
//! 实现：modifier card_distance_unlimited_skill_on_owner_<tanhu2def> override 1
//! （批 83 TargetMark 的方向反转变体：技能在目标侧、不限定使用方）。
//! 近似（登记债）：无懈禁响面未建模（批 167 立项书）。

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const TANHU3: u32 = 4_019_583_040;
const TANHU2: u32 = 2_642_618_323;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

fn loaded(grants_to_owner: &[u32], seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(
        matches!(result, CommandResult::Accepted { .. }),
        "start rejected: {result:?}"
    );
    let state = engine.state().clone();
    let owner = state.turn.current;
    let attacker = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    let removals: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    let _ = engine_core::effect::run_effects(&mut tx, removals);
    let grants: Vec<_> = grants_to_owner
        .iter()
        .map(|def| EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(*def),
        })
        .collect();
    engine_core::effect::run_effects(&mut tx, grants).unwrap();
    (tx.commit().state, owner, attacker)
}

#[test]
fn package_publishes_tanhu3_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    for target in [
        "card_distance_unlimited_skill_on_owner_2642618323",
        "card_use_unrespondable_skill_on_owner_2642618323",
    ] {
        assert!(
            package.modifier_index.iter().any(|m| {
                m.skill_def_id == TANHU3 && m.target == target && m.mode == "override"
            }),
            "tanhu3 must publish {target}"
        );
    }
}

#[test]
fn wuxie_blocked_by_target_skill_declarations() {
    // 2 人局 bootstrap 对个别 seed 敏感（"bad target"），用重试循环取首个可开局 seed。
    fn first_ok(grants: &[u32]) -> (GameState, PlayerId, PlayerId) {
        for seed in 2_026_091_601..2_026_091_631 {
            let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
            let mut engine = Engine::new();
            engine.load_package(&package).unwrap();
            if matches!(
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
            ) {
                return loaded(grants, seed);
            }
        }
        panic!("no usable seed in range");
    }

    // owner 带 tanhu3+tanhu2：目标集合含 owner 时无懈被声明阻断。
    let (state, owner, _attacker) = first_ok(&[TANHU3, TANHU2]);
    assert!(engine_core::query::wuxie_blocked_by_target_skill_declarations(&state, &[owner]));

    // owner 只带 tanhu3 不带 tanhu2：不阻断（def 不匹配）。
    let (state, owner, _attacker) = first_ok(&[TANHU3]);
    assert!(!engine_core::query::wuxie_blocked_by_target_skill_declarations(&state, &[owner]));

    // 目标集合不含 tanhu2 持有者：不阻断。
    let (state, owner, other) = first_ok(&[TANHU3, TANHU2]);
    assert!(!engine_core::query::wuxie_blocked_by_target_skill_declarations(&state, &[other]));
    let _ = owner;
}

#[test]
fn tanhu3_grants_distance_free_approach_to_tanhu2_holder_only() {
    // owner 同时带 tanhu3（声明变体）与 tanhu2（被声明技能）：任意使用者（attacker）
    // 对其无视距离。
    let (state, owner, attacker) = loaded(&[TANHU3, TANHU2], 2_026_091_401);
    assert!(engine_core::query::card_distance_unlimited_for_target(
        &state,
        attacker,
        engine_api::CardId::NONE,
        owner,
    ));
}

#[test]
fn tanhu3_requires_the_declared_skill_on_the_target() {
    // owner 只带 tanhu3（声明变体）不带 tanhu2（被声明技能）：不豁免。
    let (state, owner, attacker) = loaded(&[TANHU3], 2_026_091_402);
    assert!(!engine_core::query::card_distance_unlimited_for_target(
        &state,
        attacker,
        engine_api::CardId::NONE,
        owner,
    ));

    // owner 什么都不带：不豁免。
    let (state, owner, attacker) = loaded(&[], 2_026_091_403);
    assert!(!engine_core::query::card_distance_unlimited_for_target(
        &state,
        attacker,
        engine_api::CardId::NONE,
        owner,
    ));
}
