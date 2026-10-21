//! Golden for 旧包魏延「狂骨」：近距离造成伤害后回复等量体力。

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{turn_flag, Phase};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const WEIYAN: u32 = 2_400_833_780;
const KUANGGU: u32 = 3_700_078_390;
const JUEYAN_WEAPON: u32 = 3_194_432_315;
const JUEYAN_ARMOR: u32 = 4_232_768_410;
const JIEZI: u32 = 2_177_948_990;
const XUHUANG: u32 = 4_181_254_980;
const GZDUANLIANG: u32 = 2_148_140_249;
const SHENHUA_PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/shenhua/shenhua.npack.json");
const OLD_PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/old/old.npack.json");

fn start(seed: u64) -> Command {
    Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    }
}

#[test]
fn wounded_source_recovers_only_within_distance_one() {
    let shenhua = content_schema::ContentPackage::from_json_bytes(SHENHUA_PACKAGE).unwrap();
    assert!(shenhua
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == KUANGGU && skill.locked));

    let old = content_schema::ContentPackage::from_json_bytes(OLD_PACKAGE).unwrap();
    assert!(old.character_index.iter().any(|character| {
        character.id == WEIYAN && character.max_hp == 4 && character.skills == [KUANGGU]
    }));
    assert!(old.character_index.iter().any(|character| {
        character.id == XUHUANG && character.max_hp == 4 && character.skills == [GZDUANLIANG]
    }));

    let mut engine = Engine::new();
    // Character and skill packages are independent; either load order is valid.
    engine.load_package(&old).unwrap();
    engine.load_package(&shenhua).unwrap();
    assert!(matches!(
        engine.submit(start(4)),
        CommandResult::Accepted { .. }
    ));

    let seats = engine.state().players.alive_by_seat();
    assert_eq!(seats.len(), 4);
    // Pin the subject explicitly so adding another fully migrated old-pack character does not
    // make this skill Golden depend on character-pool sampling.
    let source_index = 0;
    let source = seats[source_index];
    let near = seats[(source_index + 1) % 4];
    let far = seats[(source_index + 2) % 4];
    let other_near = seats[(source_index + 3) % 4];
    let mut tx = Transaction::begin(engine.state().clone());
    for player in &seats {
        let existing = tx.state().players.get(*player).unwrap().skills.clone();
        for skill in existing {
            run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
        }
    }
    tx.state_mut()
        .players
        .get_mut(source)
        .unwrap()
        .character_def = WEIYAN;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: source,
            skill_def: SkillDefId::new(KUANGGU),
        }],
    )
    .unwrap();
    let max_hp = tx.state().players.get(source).unwrap().max_hp as u32;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: source,
            amount: max_hp - 1,
            is_lose_hp: true,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(source).unwrap().hp, 1);

    let budget = tx.state().resolution.trigger_budget;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: near,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(source).unwrap().hp, 3);
    assert_eq!(tx.state().resolution.trigger_budget, budget - 1);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: source,
            amount: 2,
            is_lose_hp: true,
            nature: 0,
        }],
    )
    .unwrap();
    let budget = tx.state().resolution.trigger_budget;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: far,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(source).unwrap().hp, 1);
    assert_eq!(tx.state().resolution.trigger_budget, budget);

    run_effects(
        &mut tx,
        vec![EffectRequest::RecoverHp {
            source,
            player: source,
            amount: max_hp,
        }],
    )
    .unwrap();
    let budget = tx.state().resolution.trigger_budget;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: other_near,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(source).unwrap().hp, max_hp as i32);
    assert_eq!(tx.state().resolution.trigger_budget, budget);
}

#[test]
fn jueyan_subskills_add_three_sha_and_three_hand_limit() {
    let shenhua = content_schema::ContentPackage::from_json_bytes(SHENHUA_PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&shenhua).unwrap();
    assert!(matches!(
        engine.submit(start(5)),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let base_hand = match engine_core::query::query(
        tx.state(),
        &rule_api::QueryRequest::HandLimit { player: owner },
        &mut engine_core::query::QueryContext::new(),
    ) {
        rule_api::QueryResponse::Int(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    };
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(JUEYAN_WEAPON),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(JUEYAN_ARMOR),
            },
        ],
    )
    .unwrap();
    assert_eq!(engine_core::query::sha_use_limit_for(tx.state(), owner), 4);
    assert!(matches!(
        engine_core::query::query(
            tx.state(),
            &rule_api::QueryRequest::HandLimit { player: owner },
            &mut engine_core::query::QueryContext::new(),
        ),
        rule_api::QueryResponse::Int(value) if value == base_hand + 3
    ));
}

#[test]
fn jiezi_draws_only_when_another_players_draw_phase_is_skipped() {
    let package = content_schema::ContentPackage::from_json_bytes(SHENHUA_PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(start(221)),
        CommandResult::Accepted { .. }
    ));
    let current = engine.state().turn.current;
    let owner = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != current)
        .unwrap();

    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JIEZI),
        }],
    )
    .unwrap();
    tx.state_mut().turn.phase = Phase::Judge;
    tx.state_mut().turn.set_flag(turn_flag::SKIP_DRAW, 1);
    let before = tx.state().hand_count(owner);
    let state = tx.commit().state;
    let revision = state.revision;
    let (after, result) = apply_command(
        state,
        Command {
            actor: current,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(221),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(after.hand_count(owner), before + 1);
    assert_eq!(after.turn.phase, Phase::Play);

    let mut self_skip = Transaction::begin(engine.state().clone());
    run_effects(
        &mut self_skip,
        vec![EffectRequest::AddSkill {
            player: current,
            skill_def: SkillDefId::new(JIEZI),
        }],
    )
    .unwrap();
    self_skip.state_mut().turn.phase = Phase::Judge;
    self_skip.state_mut().turn.set_flag(turn_flag::SKIP_DRAW, 1);
    let before = self_skip.state().hand_count(current);
    let state = self_skip.commit().state;
    let revision = state.revision;
    let (after, result) = apply_command(
        state,
        Command {
            actor: current,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(222),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(after.hand_count(current), before);
}
