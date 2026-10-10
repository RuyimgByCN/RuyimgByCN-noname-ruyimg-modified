//! Goldens for 神话再临旧曹仁「据守」and the generic turn-over contract.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    ObserverKind, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::Phase;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const OLD_CAOREN: u32 = 3_775_318_821;
const JUSHOU: u32 = 3_729_566_871;
const MOON_JUSHOU: u32 = 3_308_685_907;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

fn start(seed: u64) -> Command {
    Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    }
}

fn end_phase(actor: PlayerId, key: u64) -> Command {
    Command {
        actor,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(key),
        kind: CommandKind::EndPhase,
    }
}

#[test]
fn package_closes_old_caoren_and_jushou() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == OLD_CAOREN)
        .expect("old_caoren character");
    assert_eq!(character.skills, vec![JUSHOU]);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == JUSHOU));
}

#[test]
fn jushou_draws_three_turns_owner_over_and_skips_their_next_turn() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(start(20_260_715)),
        CommandResult::Accepted { .. }
    ));

    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let hand_before = engine.state().hand_count(owner);
    let mut tx = Transaction::begin(engine.state().clone());
    if !tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == JUSHOU)
        })
    {
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(JUSHOU),
            }],
        )
        .unwrap();
    }
    tx.state_mut().turn.phase = Phase::End;
    let phase_end = DomainEvent::TurnStateChanged {
        current: owner,
        phase: TurnPhaseTag::End,
    };
    // 出牌阶段结束的多个可选触发（刘禅放权等标准技 + 据守）逐段确认。
    let mut state = tx.state().clone();
    let mut dispatched = false;
    fn ok_result() -> PipelineResult {
        PipelineResult {
            result: CommandResult::Accepted {
                new_revision: 0,
                state_hash: [0; 32],
            },
            events: vec![],
            choice: None,
        }
    }
    let mut resolved = ok_result();
    loop {
        let pending = state.resolution.pending_choice_id;
        if pending == 0 && dispatched {
            break;
        }
        let mut tx = Transaction::begin(state.clone());
        let (state2, result) = if pending != 0 {
            let observer = PlayerId::new(tx.state().resolution.pending_observer);
            apply_command(
                tx.state().clone(),
                Command {
                    actor: observer,
                    expected_revision: ExpectedRevision(0),
                    idempotency_key: IdempotencyKey(1),
                    kind: CommandKind::ResolveChoice {
                        choice_id: ChoiceId::new(pending),
                        selections: vec![1],
                        cancelled: false,
                    },
                },
            )
        } else if !dispatched {
            dispatched = true;
            let fired =
                dispatch_for_events(&mut tx, &[phase_end.clone()], &mut TriggerFireRecord::new());
            match fired {
                Ok(effects) => match run_effects(&mut tx, effects) {
                    Ok(_) => (tx.commit().state, ok_result()),
                    Err(EffectError::ChoiceRequested) => (tx.commit().state, ok_result()),
                    Err(e) => panic!("run error: {e:?}"),
                },
                Err(EffectError::ChoiceRequested) => (tx.commit().state, ok_result()),
                Err(e) => panic!("dispatch error: {e:?}"),
            }
        } else {
            break;
        };
        assert!(
            matches!(result.result, CommandResult::Accepted { .. }),
            "choice must resolve: {result:?}"
        );
        resolved = result;
        state = state2;
    }
    assert_eq!(state.hand_count(owner), hand_before + 3);
    assert!(state.players.get(owner).unwrap().turned_over);
    assert!(resolved.events.iter().any(|event| matches!(
        event,
        DomainEvent::PlayerTurnedOver {
            player,
            turned_over: true
        } if *player == owner
    )));
    assert!(
        engine_core::project_observer(&state, ObserverKind::Player(owner))
            .seats
            .iter()
            .find(|seat| seat.player == owner)
            .unwrap()
            .turned_over
    );

    let (state, advanced) = apply_command(state, end_phase(owner, 2));
    assert!(matches!(advanced.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.current, other);
    assert_eq!(state.turn.phase, Phase::Prepare);

    // 座次角色随 shenhua 包角色池增长而变化：若下一家准备阶段有可选触发
    // （如甄姬洛神），先拒绝之，再推进测试。
    let mut state = state;
    if state.has_pending_choice() {
        // 若下一家准备阶段有可选触发（如甄姬洛神），拒绝之；若为强制摸牌置牌
        // 触发（卢植明任），以手牌结算之。
        let actor = PlayerId::new(state.resolution.pending_observer);
        let hand = state.hand_cards(actor);
        let picks = if hand.is_empty() {
            vec![0]
        } else {
            vec![hand[0].raw()]
        };
        let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
        let revision = state.revision;
        let (next, result) = apply_command(
            state,
            Command {
                actor,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(30),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: picks,
                    cancelled: false,
                },
            },
        );
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        state = next;
    }

    let round_before_skip = state.turn.round;
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::End;
    let (state, skipped) = apply_command(tx.state().clone(), end_phase(other, 3));
    assert!(matches!(skipped.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.current, other, "owner's whole turn was skipped");
    assert_eq!(state.turn.phase, Phase::Prepare);
    assert_eq!(state.turn.round, round_before_skip + 1);
    assert!(!state.players.get(owner).unwrap().turned_over);
    assert!(skipped.events.iter().any(|event| matches!(
        event,
        DomainEvent::PlayerTurnedOver {
            player,
            turned_over: false
        } if *player == owner
    )));
}

#[test]
fn all_face_down_players_are_flipped_once_before_play_resumes() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(start(20_260_716)),
        CommandResult::Accepted { .. }
    ));
    let players = engine.state().players.alive_by_seat();
    let first = players[0];
    let second = players[1];
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.current = first;
    tx.state_mut().turn.phase = Phase::End;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::TurnOver { player: first },
            EffectRequest::TurnOver { player: second },
        ],
    )
    .unwrap();
    let round_before = tx.state().turn.round;

    let (state, result) = apply_command(tx.state().clone(), end_phase(first, 9));
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.current, second);
    assert_eq!(state.turn.phase, Phase::Prepare);
    assert_eq!(state.turn.round, round_before + 1);
    assert!(players
        .iter()
        .all(|player| !state.players.get(*player).unwrap().turned_over));
    assert_eq!(
        result
            .events
            .iter()
            .filter(|event| matches!(
                event,
                DomainEvent::PlayerTurnedOver {
                    turned_over: false,
                    ..
                }
            ))
            .count(),
        2
    );
}

#[test]
fn moon_jushou_draws_one_then_turns_owner_over() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == MOON_JUSHOU));
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(start(20_260_717)),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .map(|skill| EffectRequest::RemoveSkill { skill })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(MOON_JUSHOU),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.phase = Phase::End;
    let hand_before = tx.state().hand_count(owner);
    let phase_end = DomainEvent::TurnStateChanged {
        current: owner,
        phase: TurnPhaseTag::End,
    };
    assert_eq!(
        dispatch_for_events(&mut tx, &[phase_end], &mut TriggerFireRecord::new()).unwrap_err(),
        EffectError::ChoiceRequested
    );

    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(10),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before + 1);
    assert!(state.players.get(owner).unwrap().turned_over);
}
