//! N01 第一片：OL界朱桓 `ol_zhuhuan` 的「奋励」三阶段与「平寇」伤害入口。
//! 平寇的「每造成一次伤害摸一张」尚未实现，本测试只锁定已交付的伤害触发。

const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/onlyOL/onlyOL.npack.json");
const OL_ZHUHUAN: u32 = 3_125_343_915;
const OL_FENLI: u32 = 2_744_558_996;
const OL_PINGKOU: u32 = 2_376_678_765;

#[test]
fn ol_zhuhuan_publishes_fenli_three_windows_and_pingkou_damage_entry() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == OL_ZHUHUAN)
        .expect("ol_zhuhuan");
    assert_eq!(character.name_key, "ol_zhuhuan");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.group, "wu");
    assert_eq!(character.skills, [OL_FENLI, OL_PINGKOU]);

    let fenli_events: Vec<&str> = package
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == OL_FENLI)
        .map(|trigger| trigger.event.as_str())
        .collect();
    assert_eq!(fenli_events, ["phase_draw", "phase_play", "phase_discard"]);

    let pingkou = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OL_PINGKOU)
        .expect("ol_pingkou");
    assert_eq!(pingkou.event, "phase_end");
    assert_eq!(
        pingkou.filter,
        "owner_is_target_skipped_phase_count_positive"
    );
}

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const FENLI_SRC: &str = include_str!("../../../../content/original/ported/onlyOL/ol_fenli.skill");
const PINGKOU_SRC: &str =
    include_str!("../../../../content/original/ported/onlyOL/ol_pingkou.skill");

fn started() -> (GameState, PlayerId) {
    let source = format!("{FENLI_SRC}\n{PINGKOU_SRC}");
    let compiled = content_compiler::compile_source_named("ol-zhuhuan-n01", &source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, 2_026_073_601, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.extend([OL_FENLI, OL_PINGKOU].map(|def| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    }));
    // 让 owner 手牌全場最大，并留足体力防濒死。
    effects.push(EffectRequest::DrawCards {
        player: owner,
        count: 3,
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let p = tx.state_mut().players.get_mut(*player).unwrap();
        p.hp = 8;
        p.max_hp = 8;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Judge;
    tx.state_mut().revision = 1;
    (tx.commit().state, owner)
}

#[test]
fn ol_fenli_draw_skip_advances_to_play_without_drawing() {
    let (state, owner) = started();
    let before = state.hand_count(owner);
    let revision = state.revision;
    let (state, offered) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, resolved) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Play);
    assert_eq!(state.hand_count(owner), before);
    assert_eq!(state.turn.skipped_phase_count(), 1);
    assert!(resolved.events.iter().any(|event| matches!(
        event,
        engine_api::DomainEvent::PhaseSkipped {
            current,
            phase: TurnPhaseTag::Draw,
        } if *current == owner
    )));
}

#[test]
fn ol_pingkou_draws_once_per_target_that_actually_took_damage() {
    let (state, owner) = started();
    let mut tx = Transaction::begin(state);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.owner == owner && s.def.raw() == OL_PINGKOU)
        .unwrap()
        .id;
    let others: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|p| *p != owner)
        .collect();
    tx.state_mut().turn.phase = Phase::End;
    tx.state_mut().turn.record_skipped_phase();
    tx.state_mut().turn.record_skipped_phase();
    let hand_before = tx.state().hand_count(owner);
    let hp_before: Vec<_> = others
        .iter()
        .map(|p| tx.state().players.get(*p).unwrap().hp)
        .collect();
    let err = run_effects(
        &mut tx,
        vec![
            EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamageDraw {
                player: owner,
                amount: 1,
                skill,
            },
        ],
    )
    .unwrap_err();
    assert_eq!(err, engine_core::effect::EffectError::ChoiceRequested);
    let mut state = tx.commit().state;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (next, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(9),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![others[0].raw(), others[1].raw()],
                cancelled: false,
            },
        },
    );
    state = next;
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before + 2);
    for (target, before) in others.iter().take(2).zip(hp_before) {
        assert_eq!(state.players.get(*target).unwrap().hp, before - 1);
    }
}

#[test]
fn ol_pingkou_does_not_draw_when_damage_is_absorbed() {
    let (state, owner) = started();
    let mut tx = Transaction::begin(state);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.owner == owner && s.def.raw() == OL_PINGKOU)
        .unwrap()
        .id;
    let target = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|p| *p != owner)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: target,
            mark: engine_api::MarkDefId::new(engine_core::content::MARK_HUJIA),
            delta: 1,
        }],
    )
    .unwrap();
    tx.state_mut().turn.phase = Phase::End;
    tx.state_mut().turn.record_skipped_phase();
    let hand_before = tx.state().hand_count(owner);
    let hp_before = tx.state().players.get(target).unwrap().hp;
    let err = run_effects(
        &mut tx,
        vec![
            EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamageDraw {
                player: owner,
                amount: 1,
                skill,
            },
        ],
    )
    .unwrap_err();
    assert_eq!(err, engine_core::effect::EffectError::ChoiceRequested);
    let mut state = tx.commit().state;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (next, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(10),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![target.raw()],
                cancelled: false,
            },
        },
    );
    state = next;
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.players.get(target).unwrap().hp,
        hp_before,
        "护驾吸收伤害"
    );
    assert_eq!(state.hand_count(owner), hand_before, "未实际受伤不摸牌");
}

#[test]
fn ol_pingkou_without_skipped_phases_does_not_open_choice() {
    let (state, owner) = started();
    let mut tx = Transaction::begin(state);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.owner == owner && s.def.raw() == OL_PINGKOU)
        .unwrap()
        .id;
    tx.state_mut().turn.phase = Phase::End;
    let effects = run_effects(
        &mut tx,
        vec![
            EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamageDraw {
                player: owner,
                amount: 1,
                skill,
            },
        ],
    )
    .unwrap();
    assert!(effects.is_empty());
    assert!(!tx.state().has_pending_choice());
}

#[test]
fn ol_pingkou_rejects_owner_as_target_without_mutation() {
    let (state, owner) = started();
    let mut tx = Transaction::begin(state);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.owner == owner && s.def.raw() == OL_PINGKOU)
        .unwrap()
        .id;
    tx.state_mut().turn.phase = Phase::End;
    tx.state_mut().turn.record_skipped_phase();
    let err = run_effects(
        &mut tx,
        vec![
            EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamageDraw {
                player: owner,
                amount: 1,
                skill,
            },
        ],
    )
    .unwrap_err();
    assert_eq!(err, engine_core::effect::EffectError::ChoiceRequested);
    let mut state = tx.commit().state;
    let before = engine_core::canonical_state_hash(&state);
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (next, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(11),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![owner.raw()],
                cancelled: false,
            },
        },
    );
    state = next;
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(engine_core::canonical_state_hash(&state), before);
}

#[test]
fn ol_fenli_play_window_skips_to_discard() {
    let (mut state, owner) = started();
    assert!(
        !state.has_pending_choice(),
        "setup must not leave a pending choice"
    );
    state.turn.phase = engine_core::state::Phase::Play;
    let mut tx = Transaction::begin(state);
    match engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Play,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    ) {
        Ok(effects) => {
            let err = run_effects(&mut tx, effects).unwrap_err();
            assert_eq!(err, engine_core::effect::EffectError::ChoiceRequested);
        }
        Err(err) => assert_eq!(err, engine_core::effect::EffectError::ChoiceRequested),
    }
    let state = tx.commit().state;
    assert!(state.has_pending_choice());
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(31),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, engine_core::state::Phase::Discard);
    assert_eq!(state.turn.skipped_phase_count(), 1);

    // 装备数全场唯一非空，弃牌窗口应可跳过。
    let mut tx = Transaction::begin(state);
    let zone = tx
        .state()
        .zones
        .find(engine_core::state::ZoneKind::Equip, owner)
        .unwrap();
    let card = tx.state_mut().cards.alloc(engine_core::state::Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(99_502),
        kind: engine_core::state::CardKind::Weapon,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 2,
        zone,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    let state = tx.commit().state;
    let mut tx = Transaction::begin(state);
    match engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Discard,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    ) {
        Ok(effects) => {
            let err = run_effects(&mut tx, effects).unwrap_err();
            assert_eq!(err, engine_core::effect::EffectError::ChoiceRequested);
        }
        Err(err) => assert_eq!(err, engine_core::effect::EffectError::ChoiceRequested),
    }
    let state = tx.commit().state;
    assert!(state.has_pending_choice());
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(32),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, engine_core::state::Phase::End);
    assert_eq!(state.turn.skipped_phase_count(), 2);
}

#[test]
fn ol_fenli_draw_window_not_offered_when_not_tied_max_hand() {
    let (mut state, owner) = started();
    let other = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: other,
            count: 5,
        }],
    )
    .unwrap();
    let mut state = tx.commit().state;
    state.turn.phase = engine_core::state::Phase::Draw;
    let mut tx = Transaction::begin(state);
    match engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Draw,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    ) {
        Ok(effects) => {
            let _ = run_effects(&mut tx, effects);
        }
        Err(err) => assert_eq!(err, engine_core::effect::EffectError::ChoiceRequested),
    }
    let state = tx.commit().state;
    assert!(
        !state.has_pending_choice(),
        "手牌不是并列最大时不应询问奋励"
    );
}

#[test]
fn ol_fenli_draw_window_is_offered_when_hands_tie_for_max() {
    let (state, owner) = started();
    let other = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    let target = tx.state().hand_count(owner);
    let have = tx.state().hand_count(other);
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: other,
            count: target.saturating_sub(have),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(other), target);
    let mut state = tx.commit().state;
    state.turn.phase = engine_core::state::Phase::Draw;
    let mut tx = Transaction::begin(state);
    match engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Draw,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    ) {
        Ok(effects) => {
            let err = run_effects(&mut tx, effects).unwrap_err();
            assert_eq!(err, engine_core::effect::EffectError::ChoiceRequested);
        }
        Err(err) => assert_eq!(err, engine_core::effect::EffectError::ChoiceRequested),
    }
    let state = tx.commit().state;
    assert!(state.has_pending_choice(), "并列最大手牌应询问奋励");
}

#[test]
fn ol_fenli_discard_window_not_offered_without_equipment() {
    let (mut state, owner) = started();
    state.turn.phase = engine_core::state::Phase::Discard;
    let mut tx = Transaction::begin(state);
    match engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Discard,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    ) {
        Ok(effects) => {
            let _ = run_effects(&mut tx, effects);
        }
        Err(err) => assert_eq!(err, engine_core::effect::EffectError::ChoiceRequested),
    }
    let state = tx.commit().state;
    assert!(!state.has_pending_choice(), "装备区为空时不应询问奋励弃牌");
}
