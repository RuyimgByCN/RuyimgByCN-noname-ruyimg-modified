//! Golden for REFRESH「同心」的攻击范围修正。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{query, QueryContext};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const RETONGXIN: u32 = 2_855_175_601;
const REWEIZHONG: u32 = 4_262_017_394;
const REBIYUE: u32 = 4_285_757_447;
const REZONGSHI: u32 = 3_490_528_973;
const REZONGSHI_PAOXIAO: u32 = 2_831_909_821;
const NEW_REPAOXIAO2: u32 = 2_901_743_430;
const NEW_REPAOXIAO: u32 = 2_733_157_850;
const REFANGQUAN2: u32 = 2_792_995_788;
const REYINGZI: u32 = 2_412_932_912;
const REDANXIN: u32 = 2_528_063_581;
const REDANXIN_MARK: u32 = 380_579_933;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/refresh/refresh.npack.json");

fn has_skill(tx: &Transaction, player: PlayerId, skill_def: u32) -> bool {
    tx.state()
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == skill_def)
}

#[test]
fn attack_range_adds_two() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();

    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_714,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let base = match query(
        tx.state(),
        &QueryRequest::AttackRange { player: owner },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    };
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(RETONGXIN),
        }],
    )
    .unwrap();
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::AttackRange { player: owner },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(value) if value == base + 2
    ));
}

#[test]
fn reweizhong_draws_two_after_max_hp_changes() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_715,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(REWEIZHONG),
        }],
    )
    .unwrap();
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::ChangeMaxHp {
            player: owner,
            delta: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 2);
}

#[test]
fn rebiyue_draws_two_when_empty_and_one_otherwise() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_716,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;

    for empty in [false, true] {
        let mut tx = Transaction::begin(engine.state().clone());
        let mut setup = vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(REBIYUE),
        }];
        if empty {
            setup.push(EffectRequest::DiscardFromHand {
                player: owner,
                count: u32::MAX,
            });
        }
        run_effects(&mut tx, setup).unwrap();
        let hand = tx.state().hand_count(owner);
        assert_eq!(
            run_effects(
                &mut tx,
                vec![EffectRequest::SetTurnState {
                    current: owner,
                    phase: engine_core::state::Phase::End.to_u8(),
                }],
            )
            .unwrap_err(),
            EffectError::ChoiceRequested
        );
        let (state, result) = apply_command(
            tx.state().clone(),
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(1),
                kind: CommandKind::ResolveChoice {
                    choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                    selections: vec![1],
                    cancelled: false,
                },
            },
        );
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        assert_eq!(state.hand_count(owner), hand + if empty { 2 } else { 1 });
    }
}

#[test]
fn paoxiao_temp_skills_remove_sha_count_and_distance_limits() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_717,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(REZONGSHI_PAOXIAO),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(NEW_REPAOXIAO2),
            },
        ],
    )
    .unwrap();
    assert_eq!(
        engine_core::query::sha_use_limit_for(tx.state(), owner),
        i32::MAX as u32
    );
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::AttackRange { player: owner },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(i32::MAX)
    ));
}

#[test]
fn rezongshi_adds_group_hand_limit_and_grants_paoxiao_only_for_the_qualifying_turn() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: 20_260_717_2,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    for skill in tx.state().players.get(owner).unwrap().skills.clone() {
        run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
    }

    let base_limit = match query(
        tx.state(),
        &QueryRequest::HandLimit { player: owner },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    };
    let group_count = tx
        .state()
        .players
        .iter_live()
        .filter(|player| player.alive)
        .filter_map(|player| tx.state().content.character(player.character_def))
        .map(|character| character.group.as_str())
        .filter(|group| !group.is_empty())
        .collect::<std::collections::BTreeSet<_>>()
        .len() as i32;
    assert!(group_count > 0);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(REZONGSHI),
        }],
    )
    .unwrap();
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::HandLimit { player: owner },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(value) if value == base_limit + group_count
    ));

    let hand = tx.state().hand_count(owner);
    assert!(hand > 0);
    tx.state_mut().players.get_mut(owner).unwrap().hp = hand as i32 - 1;
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: engine_core::state::Phase::Prepare.to_u8(),
        }],
    )
    .unwrap();
    assert!(has_skill(&tx, owner, REZONGSHI_PAOXIAO));
    assert_eq!(
        engine_core::query::sha_use_limit_for(tx.state(), owner),
        i32::MAX as u32
    );

    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: owner,
                phase: engine_core::state::Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: other,
                phase: engine_core::state::Phase::Prepare.to_u8(),
            },
        ],
    )
    .unwrap();
    assert!(!has_skill(&tx, owner, REZONGSHI_PAOXIAO));
    assert_ne!(
        engine_core::query::sha_use_limit_for(tx.state(), owner),
        i32::MAX as u32
    );

    tx.state_mut().players.get_mut(owner).unwrap().hp = tx.state().hand_count(owner) as i32;
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: engine_core::state::Phase::Prepare.to_u8(),
        }],
    )
    .unwrap();
    assert!(!has_skill(&tx, owner, REZONGSHI_PAOXIAO));
}

#[test]
fn new_repaoxiao_removes_the_sha_limit_then_range_after_the_first_sha() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_717_1,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    for skill in tx.state().players.get(owner).unwrap().skills.clone() {
        run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NEW_REPAOXIAO),
        }],
    )
    .unwrap();

    assert_eq!(
        engine_core::query::sha_use_limit_for(tx.state(), owner),
        i32::MAX as u32
    );
    assert!(!engine_core::query::sha_distance_unlimited_for(
        tx.state(),
        owner
    ));
    tx.state_mut()
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED, 1);
    assert!(engine_core::query::sha_distance_unlimited_for(
        tx.state(),
        owner
    ));
}

#[test]
fn max_hp_hand_limit_skills_share_the_same_public_query_surface() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_718,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;

    for skill_def in [REFANGQUAN2, REYINGZI] {
        let mut tx = Transaction::begin(engine.state().clone());
        tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
        // Roster-independent: the booted standard/refresh roster includes
        // hand-limit and choice skills (e.g. 袁术 庸肆, 旧界李典 询询); drop every
        // booted skill so the queries below measure only the granted skills.
        {
            let booted: Vec<_> = tx
                .state()
                .skills
                .sorted_live()
                .into_iter()
                .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
                .collect();
            if !booted.is_empty() {
                run_effects(&mut tx, booted).unwrap();
            }
        }

        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(skill_def),
            }],
        )
        .unwrap();
        let max_hp = tx.state().players.get(owner).unwrap().max_hp as i32;
        assert!(matches!(
            query(
                tx.state(),
                &QueryRequest::HandLimit { player: owner },
                &mut QueryContext::new(),
            ),
            QueryResponse::Int(value) if value == max_hp
        ));
        if skill_def == REYINGZI {
            assert_eq!(engine_core::query::draw_count_for(tx.state(), owner), 3);
        }
    }
}

#[test]
fn redanxin_draws_each_time_and_caps_its_upgrade_mark_at_two() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_719),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_719,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.hp = 10;
        player.max_hp = 10;
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(REDANXIN),
        }],
    )
    .unwrap();
    let initial_hand = tx.state().hand_count(owner);
    let mut state = tx.state().clone();

    for activation in 1..=3 {
        let mut damage = Transaction::begin(state);
        assert_eq!(
            run_effects(
                &mut damage,
                vec![EffectRequest::DealDamage {
                    source,
                    target: owner,
                    amount: 1,
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )
            .unwrap_err(),
            EffectError::ChoiceRequested
        );
        let (next, result) = apply_command(
            damage.state().clone(),
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(activation),
                kind: CommandKind::ResolveChoice {
                    choice_id: ChoiceId::new(damage.state().resolution.pending_choice_id),
                    selections: vec![1],
                    cancelled: false,
                },
            },
        );
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        assert_eq!(next.hand_count(owner), initial_hand + activation as u32);
        assert_eq!(
            next.players
                .get(owner)
                .unwrap()
                .marks
                .iter()
                .find(|(mark, _)| *mark == REDANXIN_MARK)
                .map_or(0, |(_, count)| *count),
            activation.min(2) as i32
        );
        state = next;
    }
}
