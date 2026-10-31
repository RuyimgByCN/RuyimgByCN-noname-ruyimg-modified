//! EndPhase and turn advancement (aligned with noname phase list).

use super::judge::resolve_judge_zone;
use crate::effect::{open_choice, run_effects, EffectError};
use crate::query::{draw_count_for, optional_draw_bonuses_for};
use crate::state::{turn_flag, ContinuationKind, Phase};
use crate::transaction::Transaction;
use engine_api::{ChoiceKind, PlayerId};
use rule_api::EffectRequest;

/// Reset state scoped to one turn and the new player's active-skill counters.
fn reset_turn_state(tx: &mut Transaction, player: PlayerId) {
    tx.state_mut().turn.clear_per_player_card_counts();
    tx.state_mut().turn.hand_limit_exempt_cards.clear();
    tx.state_mut().turn.set_flag(turn_flag::SKIP_DRAW, 0);
    tx.state_mut().turn.set_flag(turn_flag::SKIP_PLAY, 0);
    tx.state_mut().turn.set_flag(turn_flag::SKIP_DISCARD, 0);
    tx.state_mut().turn.set_flag(turn_flag::SHA_PLAYED, 0);
    tx.state_mut().turn.set_flag(turn_flag::DRAW_REPLACED, 0);
    tx.state_mut().turn.clear_draw_phase_fixed_count();
    tx.state_mut().turn.set_flag(turn_flag::DAMAGE_DEALT, 0);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::DAMAGE_DEALT_OTHER, 0);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::DAMAGE_DEALT_OTHER_QUN, 0);
    tx.state_mut().turn.set_flag(turn_flag::CARD_USED, 0);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::USE_RESPONSE_DISCARDED, 0);
    tx.state_mut().turn.set_flag(turn_flag::DAMAGED_PLAYERS, 0);
    tx.state_mut().turn.set_flag(turn_flag::DAMAGE_SOURCES, 0);
    tx.state_mut().turn.set_flag(turn_flag::LAST_CARD_META, 0);
    tx.state_mut().turn.set_flag(turn_flag::LAST_CARD_KIND, 0);
    tx.state_mut().turn.set_flag(turn_flag::LAST_CARD_DEF, 0);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::LAST_PLAY_CARD_META, 0);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::OPENING_EXTRA_PLAY_PENDING, 0);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::OPENING_EXTRA_PLAY_ACTIVE, 0);
    tx.state_mut().turn.opening_extra_play_queue.clear();
    tx.state_mut().turn.opening_extra_play_active_skill = engine_api::SkillId::NONE;
    tx.state_mut().turn.opening_extra_play_damage_baseline = 0;
    tx.state_mut()
        .turn
        .set_flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 0);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::EXPLICIT_TURN_BEGIN_EVENT, 0);
    tx.state_mut().turn.set_flag(turn_flag::END_EXTRA_PHASES, 0);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::SKIPPED_PHASE_COUNT, 0);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::CARD_TARGETED_OTHER, 0);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::CURRENT_DRAW_COUNT_BONUS, 0);
    tx.state_mut().turn.clear_phase_histories();
    tx.state_mut()
        .turn
        .set_flag(turn_flag::PHASE_HISTORY_SUPPRESSED, 0);
    let sids: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| skill.id)
        .collect();
    let cleared_suppression = tx.state().turn.non_locked_skill_suppression.is_some()
        || !tx
            .state()
            .turn
            .non_locked_skill_suppressed_players
            .is_empty();
    tx.state_mut().turn.non_locked_skill_suppression = engine_api::SkillId::NONE;
    tx.state_mut()
        .turn
        .non_locked_skill_suppressed_players
        .clear();
    let mut restored_skill = false;
    for sid in sids {
        if let Some(s) = tx.state_mut().skills.get_mut(sid) {
            s.turn_fire_count = 0;
            s.play_phase_fire_count = 0;
            s.play_phase_use_count = 0;
            s.active_turn_banned = false;
            restored_skill |= std::mem::take(&mut s.turn_disabled);
            s.clear_xinyaoming_turn_state();
            s.clear_twguoyi_turn_state();
            if s.owner == player {
                s.use_count = 0;
                s.clear_active_unique_state();
            }
        }
    }
    if restored_skill || cleared_suppression {
        tx.state_mut().rebuild_derived_indexes();
    }
}

fn expire_temporary_skills(tx: &Transaction, turn_owner: PlayerId) -> Vec<EffectRequest> {
    tx.state()
        .skills
        .sorted_live()
        .into_iter()
        .filter(|skill| skill.temporary_until_turn_end_of == turn_owner)
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect()
}

fn expire_source_turn_bound_skills(
    tx: &mut Transaction,
    turn_owner: PlayerId,
) -> Vec<EffectRequest> {
    let skill_ids: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .filter(|skill| {
            skill
                .source_turn_bounds()
                .is_some_and(|sources| sources.contains(&turn_owner))
        })
        .map(|skill| skill.id)
        .collect();
    let mut effects = Vec::new();
    for skill in skill_ids {
        if tx
            .state_mut()
            .skills
            .get_mut(skill)
            .and_then(|instance| instance.remove_source_turn_bound(turn_owner))
            == Some(false)
        {
            effects.push(EffectRequest::RemoveSkill { skill });
        }
    }
    effects
}

/// Advance clockwise to the next face-up living player. Face-down players consume
/// one whole turn, return face-up, and are skipped without entering any phase.
pub(crate) fn continue_turn_advance(tx: &mut Transaction) -> Result<(), EffectError> {
    // 猎族 S3（jsrgpingrong）：刚结束的回合若是受益人的额外回合，则在此收尾
    // 尾随 check——该回合 DAMAGE_DEALT==0 时失去 1 点体力。
    let finished = tx.state().turn.current;
    if finished.is_some()
        && tx
            .state()
            .turn
            .flag(crate::state::turn_flag::extra_turn_active_by_player(
                finished,
            ))
            == 1
    {
        let dealt = tx.state().turn.flag(turn_flag::DAMAGE_DEALT);
        let key = crate::state::turn_flag::extra_turn_active_by_player(finished);
        tx.state_mut().turn.set_flag(key, 0);
        if dealt == 0 {
            let effects = vec![EffectRequest::DealDamage {
                source: PlayerId::NONE,
                target: finished,
                amount: 1,
                is_lose_hp: true,
                nature: 0,
            }];
            run_effects(tx, effects)?;
        }
    }
    let order = tx.state().players.alive_by_seat();
    if order.is_empty() {
        return advance_to(tx, Phase::Idle);
    }

    let current = tx.state().turn.current;
    let mut effects = expire_temporary_skills(tx, current);
    while tx
        .state()
        .turn
        .extra_draw_phase_queue
        .first()
        .is_some_and(|player| !order.contains(player))
    {
        tx.state_mut().turn.extra_draw_phase_queue.remove(0);
    }
    if let Some(next) = tx.state().turn.extra_draw_phase_queue.first().copied() {
        tx.state_mut().turn.extra_draw_phase_queue.remove(0);
        if tx.state().turn.extra_turn_anchor.is_none() {
            tx.state_mut().turn.extra_turn_anchor = current;
        }
        reset_turn_state(tx, next);
        tx.state_mut().turn.extra_draw_phase_active = true;
        effects.extend(expire_source_turn_bound_skills(tx, next));
        run_effects(tx, effects)?;
        return enter_draw_phase(tx, next);
    }
    while tx
        .state()
        .turn
        .extra_play_phase_queue
        .first()
        .is_some_and(|player| !order.contains(player))
    {
        tx.state_mut().turn.extra_play_phase_queue.remove(0);
    }
    if let Some(next) = tx.state().turn.extra_play_phase_queue.first().copied() {
        tx.state_mut().turn.extra_play_phase_queue.remove(0);
        if tx.state().turn.extra_turn_anchor.is_none() {
            tx.state_mut().turn.extra_turn_anchor = current;
        }
        reset_turn_state(tx, next);
        tx.state_mut().turn.extra_play_phase_active = true;
        effects.extend(expire_source_turn_bound_skills(tx, next));
        effects.push(EffectRequest::SetTurnState {
            current: next,
            phase: Phase::Play.to_u8(),
        });
        return run_effects(tx, effects).map(|_| ());
    }
    while tx
        .state()
        .turn
        .extra_turn_queue
        .first()
        .is_some_and(|player| !order.contains(player))
    {
        tx.state_mut().turn.extra_turn_queue.remove(0);
    }
    if let Some(next) = tx.state_mut().turn.extra_turn_queue.first().copied() {
        tx.state_mut().turn.extra_turn_queue.remove(0);
        if tx.state().turn.extra_turn_anchor.is_none() {
            tx.state_mut().turn.extra_turn_anchor = current;
        }
        reset_turn_state(tx, next);
        // 进入受益人的额外回合：消费 pending 标记为 active（尾随 check 于其
        // 结束后的下一次 continue_turn_advance 头部结算）。
        tx.state_mut().turn.set_flag(
            crate::state::turn_flag::extra_turn_check_pending_by_player(next),
            0,
        );
        tx.state_mut().turn.set_flag(
            crate::state::turn_flag::extra_turn_active_by_player(next),
            1,
        );
        let face_down = tx
            .state()
            .players
            .get(next)
            .is_some_and(|player| player.turned_over);
        if face_down {
            effects.push(EffectRequest::TurnOver { player: next });
        } else {
            effects.extend(expire_source_turn_bound_skills(tx, next));
            if tx.state().content.tracks_opening_extra_play_lifecycle() {
                effects.push(EffectRequest::BeginTurn { player: next });
                effects.push(EffectRequest::FinishTurnBegin { player: next });
            } else {
                effects.push(EffectRequest::SetTurnState {
                    current: next,
                    phase: Phase::Prepare.to_u8(),
                });
            }
            return run_effects(tx, effects).map(|_| ());
        }
    }
    // §6 相位裁剪队列（第九十八批 S0）：在整回合额外队列之后、常规座次推进
    // 之前消费；仅允许列表内阶段，未列阶段以既有 SKIP_* / 判定跳过机制武装。
    while !tx.state().turn.extra_phase_queue.is_empty() {
        let head_alive = tx
            .state()
            .players
            .get(tx.state().turn.extra_phase_queue[0].player)
            .is_some_and(|p| p.alive);
        if head_alive {
            break;
        }
        tx.state_mut().turn.extra_phase_queue.remove(0);
    }
    if let Some(entry) = tx.state().turn.extra_phase_queue.first().cloned() {
        tx.state_mut().turn.extra_phase_queue.remove(0);
        if tx.state().turn.extra_turn_anchor.is_none() {
            tx.state_mut().turn.extra_turn_anchor = current;
        }
        reset_turn_state(tx, entry.player);
        // Arm skips for every phase NOT in the allowed list. Prepare always
        // runs (engine has no prepare-skip); Judge is skipped through the
        // per-player pending list; Draw/Play/Discard via their SKIP flags.
        let allowed: std::collections::BTreeSet<u8> = entry.phases.iter().copied().collect();
        for tag in 1u8..=5u8 {
            if !allowed.contains(&tag) {
                match tag {
                    1 => {
                        tx.state_mut().pending_judge_skips.push(entry.player);
                    }
                    2 => tx.state_mut().turn.set_flag(turn_flag::SKIP_DRAW, 1),
                    3 => tx.state_mut().turn.set_flag(turn_flag::SKIP_PLAY, 1),
                    4 => tx.state_mut().turn.set_flag(turn_flag::SKIP_DISCARD, 1),
                    _ => {}
                }
            }
        }
        let first = entry
            .phases
            .iter()
            .copied()
            .min()
            .unwrap_or_else(|| engine_api::TurnPhaseTag::End as u8);
        effects.extend(expire_source_turn_bound_skills(tx, entry.player));
        effects.push(EffectRequest::SetTurnState {
            current: entry.player,
            phase: first,
        });
        return run_effects(tx, effects).map(|_| ());
    }

    let base = if tx.state().turn.extra_turn_anchor.is_some() {
        std::mem::replace(&mut tx.state_mut().turn.extra_turn_anchor, PlayerId::NONE)
    } else {
        current
    };
    let mut index = order
        .iter()
        .position(|player| *player == base)
        .unwrap_or_else(|| {
            let seat = tx
                .state()
                .players
                .get(base)
                .map(|player| player.seat.raw())
                .unwrap_or(0);
            order
                .iter()
                .rposition(|player| {
                    tx.state()
                        .players
                        .get(*player)
                        .is_some_and(|player| player.seat.raw() <= seat)
                })
                .unwrap_or(order.len() - 1)
        });
    let mut skipped = std::collections::BTreeSet::new();

    loop {
        index = (index + 1) % order.len();
        if index == 0 {
            let previous_round = tx.state().turn.round;
            let round_ended_last = order
                .iter()
                .rev()
                .find(|player| {
                    tx.state()
                        .players
                        .get(**player)
                        .is_some_and(|player| player.alive)
                })
                .copied()
                .unwrap_or_else(|| {
                    tx.state()
                        .players
                        .iter_live()
                        .next()
                        .map(|player| player.id)
                        .unwrap_or(engine_api::PlayerId::NONE)
                });
            effects.push(EffectRequest::RoundEnded {
                round: previous_round,
                current: round_ended_last,
            });
            tx.state_mut().turn.round = tx.state().turn.round.saturating_add(1);
            tx.state_mut().turn.clear_round_card_use_counts();
        }
        let next = order[index];
        if index == 0 {
            effects.push(EffectRequest::RoundStarted {
                round: tx.state().turn.round,
                current: next,
            });
        }
        reset_turn_state(tx, next);

        let face_down = tx
            .state()
            .players
            .get(next)
            .is_some_and(|player| player.turned_over)
            && !skipped.contains(&next);
        if face_down {
            skipped.insert(next);
            effects.push(EffectRequest::TurnOver { player: next });
            continue;
        }

        effects.extend(expire_source_turn_bound_skills(tx, next));
        if tx.state().content.tracks_opening_extra_play_lifecycle() {
            effects.push(EffectRequest::BeginTurn { player: next });
            effects.push(EffectRequest::FinishTurnBegin { player: next });
        } else {
            effects.push(EffectRequest::SetTurnState {
                current: next,
                phase: Phase::Prepare.to_u8(),
            });
        }
        return run_effects(tx, effects).map(|_| ());
    }
}

pub(crate) fn finish_turn(tx: &mut Transaction) -> Result<(), EffectError> {
    if tx.state().turn.flag(turn_flag::TURN_END_DISPATCHED) != 0 {
        return continue_turn_advance(tx);
    }
    let current = tx.state().turn.current;
    let event = engine_api::DomainEvent::PhaseEnded {
        current,
        phase: engine_api::TurnPhaseTag::End,
        damage_sources: tx.state().turn.flag(turn_flag::DAMAGE_SOURCES) as u32,
        damaged_players: tx.state().turn.flag(turn_flag::DAMAGED_PLAYERS) as u32,
        card_targeted_other: tx.state().turn.flag(turn_flag::CARD_TARGETED_OTHER) != 0,
        current_dealt_damage_other: tx.state().turn.flag(turn_flag::DAMAGE_DEALT_OTHER) != 0,
        phase_targeted_players: tx.state().turn.flag(turn_flag::PHASE_TARGETED_PLAYERS) as u32,
        phase_hp_changed_players: tx.state().turn.flag(turn_flag::PHASE_HP_CHANGED_PLAYERS) as u32,
    };
    tx.state_mut()
        .turn
        .set_flag(turn_flag::TURN_END_DISPATCHED, 1);
    tx.push_event(event.clone());
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::AdvanceTurn);
    let mut fires = crate::trigger::TriggerFireRecord::new();
    let effects = crate::trigger::dispatch_for_events(tx, &[event], &mut fires)?;
    tx.state_mut().resolution.deferred_effects.extend(effects);
    crate::effect::resume_deferred_resolution(tx)
}

pub(super) fn handle_end_phase(tx: &mut Transaction, actor: PlayerId) -> Result<(), EffectError> {
    if tx.state().revision == 0 {
        return Err(EffectError::Illegal("not started".into()));
    }
    if tx.state().has_pending_choice() {
        return Err(EffectError::Illegal("choice pending".into()));
    }

    let current = tx.state().turn.current;
    // Only current player (or NONE system) may end phase
    if actor.is_some() && actor != current {
        return Err(EffectError::Illegal("not current player".into()));
    }
    // A mid-turn death still closes the turn so TurnEnded triggers can resolve.
    if let Some(p) = tx.state().players.get(current) {
        if !p.alive {
            if tx.state().turn.extra_play_phase_active {
                tx.state_mut().turn.extra_play_phase_active = false;
                return continue_turn_advance(tx);
            }
            if tx.state().turn.flag(turn_flag::END_EXTRA_PHASES) != 0 {
                tx.state_mut().turn.set_flag(turn_flag::END_EXTRA_PHASES, 0);
                return continue_turn_advance(tx);
            }
            return finish_turn(tx);
        }
    }

    let phase = tx.state().turn.phase;
    match phase {
        Phase::Prepare => {
            if maybe_open_prepare_scry(tx)? {
                return Err(EffectError::ChoiceRequested);
            }
            // Clear previous skip flags when a new turn's prepare ends → judge.
            advance_to(tx, Phase::Judge)
        }
        Phase::Judge => {
            let cur = tx.state().turn.current;
            let pending_judge_skip = tx
                .state()
                .pending_judge_skips
                .iter()
                .position(|player| *player == cur);
            if let Some(index) = pending_judge_skip {
                tx.state_mut().pending_judge_skips.remove(index);
            }
            if pending_judge_skip.is_some()
                || tx.state().disabled_judge_zones.contains(&cur)
                || crate::query::skip_judge_phase(tx.state(), cur)
            {
                let event = record_phase_skipped(tx, cur, Phase::Judge);
                tx.push_event(event.clone());
                let mut fires = crate::trigger::TriggerFireRecord::new();
                let effects = crate::trigger::dispatch_for_events(tx, &[event], &mut fires)?;
                if !effects.is_empty() {
                    run_effects(tx, effects)?;
                }
                if !tx.state().disabled_judge_zones.contains(&cur)
                    && crate::query::restore_skipped_phase(tx.state(), cur)
                {
                    resolve_judge_zone(tx, cur)?;
                }
            } else {
                resolve_judge_zone(tx, cur)?;
            }
            if tx.state().turn.flag(turn_flag::PRE_NORMAL_EXTRA_DRAW) != 0 {
                enter_draw_phase(tx, cur)
            } else {
                enter_normal_draw_or_skip(tx, cur)
            }
        }
        Phase::Draw => {
            if tx.state().turn.extra_draw_phase_active {
                tx.state_mut().turn.extra_draw_phase_active = false;
                return continue_turn_advance(tx);
            }
            if tx.state().turn.flag(turn_flag::PRE_NORMAL_EXTRA_DRAW) != 0 {
                tx.state_mut()
                    .turn
                    .set_flag(turn_flag::PRE_NORMAL_EXTRA_DRAW, 0);
                return enter_normal_draw_or_skip(tx, current);
            }
            let extra = tx.state().turn.flag(turn_flag::END_EXTRA_PHASES);
            if extra != 0 {
                if extra & 2 != 0 {
                    tx.state_mut().turn.set_flag(turn_flag::END_EXTRA_PHASES, 2);
                    return advance_to(tx, Phase::Play);
                }
                tx.state_mut().turn.set_flag(turn_flag::END_EXTRA_PHASES, 0);
                return continue_turn_advance(tx);
            }
            if tx.state().turn.skip_play() {
                tx.state_mut().turn.set_flag(turn_flag::SKIP_PLAY, 0);
                let current = tx.state().turn.current;
                let event = record_phase_skipped(tx, current, Phase::Play);
                tx.push_event(event);
                if crate::query::restore_skipped_phase(tx.state(), current) {
                    advance_to(tx, Phase::Play)
                } else {
                    advance_to(tx, Phase::Discard)
                }
            } else {
                advance_to(tx, Phase::Play)
            }
        }
        Phase::Play => {
            if tx.state().turn.extra_play_phase_active {
                tx.state_mut().turn.extra_play_phase_active = false;
                continue_turn_advance(tx)
            } else if tx.state().turn.flag(turn_flag::END_EXTRA_PHASES) != 0 {
                tx.state_mut().turn.set_flag(turn_flag::END_EXTRA_PHASES, 0);
                continue_turn_advance(tx)
            } else if tx.state().turn.flag(turn_flag::OPENING_EXTRA_PLAY_ACTIVE) != 0 {
                run_effects(
                    tx,
                    vec![EffectRequest::AdvanceOpeningExtraPlay { player: current }],
                )
                .map(|_| ())
            } else {
                advance_to(tx, Phase::Discard)
            }
        }
        Phase::Discard => {
            let cur = tx.state().turn.current;
            let skip_discard = tx.state().turn.skip_discard();
            tx.state_mut().turn.set_flag(turn_flag::SKIP_DISCARD, 0);
            if skip_discard
                || crate::query::skip_discard_phase(tx.state(), cur)
                || crate::query::skip_discard_if_no_sha(tx.state(), cur)
            {
                let event = record_phase_skipped(tx, cur, Phase::Discard);
                tx.push_event(event);
                if !crate::query::restore_skipped_phase(tx.state(), cur) {
                    return advance_to(tx, Phase::End);
                }
            }
            let limit = match crate::query::query(
                tx.state(),
                &rule_api::QueryRequest::HandLimit { player: cur },
                &mut crate::query::QueryContext::new(),
            ) {
                rule_api::QueryResponse::Int(value) => value.max(0) as u32,
                _ => return Err(EffectError::Illegal("hand-limit query failed".into())),
            };
            let hand = crate::query::discard_to_limit_tokens(tx.state(), cur);
            let hand_n = hand.len() as u32;
            if hand_n > limit {
                let need = hand_n - limit;
                let tokens: Vec<u32> = hand.iter().map(|c| c.raw()).collect();
                open_choice(
                    tx,
                    cur,
                    need as u8,
                    need as u8,
                    false,
                    tokens,
                    ChoiceKind::DiscardToLimit,
                    ContinuationKind::DiscardToLimit,
                    vec![cur.raw(), need],
                )?;
                return Err(EffectError::ChoiceRequested);
            }
            advance_to(tx, Phase::End)
        }
        Phase::End => {
            let extra = tx.state().turn.flag(turn_flag::END_EXTRA_PHASES).max(0) as u8;
            if extra == 0 {
                finish_turn(tx)
            } else {
                crate::effect::start_end_extra_phases(tx, current, extra)
            }
        }
        Phase::Idle => continue_turn_advance(tx),
    }
}

/// Finish the entered phase immediately without running its remaining default work or recording a
/// skipped phase. Used by legacy effects that terminate the enclosing phase event.
pub(crate) fn finish_current_phase_abruptly(tx: &mut Transaction) -> Result<(), EffectError> {
    let current = tx.state().turn.current;
    match tx.state().turn.phase {
        Phase::Prepare => advance_to(tx, Phase::Judge),
        Phase::Judge => enter_normal_draw_or_skip(tx, current),
        Phase::Discard => advance_to(tx, Phase::End),
        Phase::Draw | Phase::Play | Phase::End | Phase::Idle => {
            handle_end_phase(tx, PlayerId::NONE)
        }
    }
}

fn advance_to(tx: &mut Transaction, phase: Phase) -> Result<(), EffectError> {
    let current = tx.state().turn.current;
    transition_to(tx, current, phase)
}

fn enter_normal_draw_or_skip(tx: &mut Transaction, current: PlayerId) -> Result<(), EffectError> {
    let skip_draw = tx.state().turn.skip_draw();
    let skip_play = tx.state().turn.skip_play();
    tx.state_mut().turn.set_flag(turn_flag::SKIP_DRAW, 0);
    if !skip_draw {
        return enter_draw_phase(tx, current);
    }
    let event = record_phase_skipped(tx, current, Phase::Draw);
    tx.push_event(event.clone());
    let mut fires = crate::trigger::TriggerFireRecord::new();
    let effects = crate::trigger::dispatch_for_events(tx, &[event], &mut fires)?;
    if !effects.is_empty() {
        run_effects(tx, effects)?;
    }
    if crate::query::restore_skipped_phase(tx.state(), current) {
        return enter_draw_phase(tx, current);
    }
    if !skip_play {
        return advance_to(tx, Phase::Play);
    }
    tx.state_mut().turn.set_flag(turn_flag::SKIP_PLAY, 0);
    let event = record_phase_skipped(tx, current, Phase::Play);
    tx.push_event(event);
    if crate::query::restore_skipped_phase(tx.state(), current) {
        advance_to(tx, Phase::Play)
    } else {
        advance_to(tx, Phase::Discard)
    }
}

fn transition_to(tx: &mut Transaction, current: PlayerId, phase: Phase) -> Result<(), EffectError> {
    run_effects(
        tx,
        vec![EffectRequest::SetTurnState {
            current,
            phase: phase.to_u8(),
        }],
    )
    .map(|_| ())
}

pub(crate) fn enter_draw_phase(tx: &mut Transaction, player: PlayerId) -> Result<(), EffectError> {
    match transition_to(tx, player, Phase::Draw) {
        Ok(()) => continue_entered_draw_phase(tx, player),
        Err(EffectError::ChoiceRequested) => {
            let resolution = &mut tx.state_mut().resolution;
            resolution.resume_kind = ContinuationKind::ContinueEnteredDrawPhase;
            resolution.resume_payload = vec![player.raw()];
            Err(EffectError::ChoiceRequested)
        }
        Err(error) => Err(error),
    }
}

/// Resume the phase-local automatic work after every phase-entry trigger has completed.
/// A skill may have skipped the Draw phase while choices were open, in which case there is
/// intentionally nothing left to do.
pub(super) fn continue_entered_draw_phase(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<(), EffectError> {
    if tx.state().turn.current != player || tx.state().turn.phase != Phase::Draw {
        return Ok(());
    }
    if maybe_open_draw_phase_scry_four_split_two(tx)? {
        return Err(EffectError::ChoiceRequested);
    }
    continue_draw_phase(tx, player)
}

/// Record one skipped phase and build its public audit event.
pub(crate) fn record_phase_skipped(
    tx: &mut Transaction,
    player: PlayerId,
    phase: Phase,
) -> engine_api::DomainEvent {
    tx.state_mut().turn.record_skipped_phase();
    engine_api::DomainEvent::PhaseSkipped {
        current: player,
        phase: phase.to_tag(),
    }
}

/// Offer a package-defined draw replacement. The draw is deferred to the
/// generic continuation so content modifiers decide the eventual benefit.
fn maybe_open_draw_replacement(tx: &mut Transaction) -> Result<bool, EffectError> {
    let cur = tx.state().turn.current;
    if !crate::query::has_draw_replacement(tx.state(), cur) {
        return Ok(false);
    }
    let n = take_draw_count(tx, cur);
    open_choice(
        tx,
        cur,
        0,
        1,
        true, // cancel = draw normally
        vec![1],
        ChoiceKind::Confirm,
        ContinuationKind::DrawReplacement,
        vec![cur.raw(), n],
    )?;
    Ok(true)
}

/// Continue a draw phase after all phase-entry reorder hooks have resolved.
pub(super) fn continue_draw_phase(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<(), EffectError> {
    if let Some(count) = tx.state().turn.draw_phase_fixed_count() {
        tx.state_mut()
            .turn
            .set_flag(turn_flag::CURRENT_DRAW_COUNT_BONUS, 0);
        if count == 0 {
            return Ok(());
        }
        return run_effects(tx, vec![EffectRequest::DrawCards { player, count }]).map(|_| ());
    }
    if maybe_open_draw_replacement(tx)? {
        return Err(EffectError::ChoiceRequested);
    }
    let count = take_draw_count(tx, player);
    if open_optional_draw_bonus(tx, player, count, false)? {
        return Err(EffectError::ChoiceRequested);
    }
    // 寸目（许攸）：当前摸牌阶段从牌堆底摸牌。
    if tx
        .state()
        .turn
        .flag(crate::state::turn_flag::DRAW_PHASE_FROM_BOTTOM)
        != 0
    {
        tx.state_mut()
            .turn
            .set_flag(crate::state::turn_flag::DRAW_PHASE_FROM_BOTTOM, 0);
        return run_effects(
            tx,
            vec![EffectRequest::DrawCardsFromBottom { player, count }],
        )
        .map(|_| ());
    }
    run_effects(tx, vec![EffectRequest::DrawCards { player, count }]).map(|_| ())
}

fn take_draw_count(tx: &mut Transaction, player: PlayerId) -> u32 {
    let bonus = tx
        .state()
        .turn
        .flag(turn_flag::CURRENT_DRAW_COUNT_BONUS)
        .max(0) as u32;
    let count = draw_count_for(tx.state(), player).saturating_add(bonus);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::CURRENT_DRAW_COUNT_BONUS, 0);
    count
}

/// Offer the reusable draw-phase "four cards, exactly two to the bottom" reorder.
fn maybe_open_draw_phase_scry_four_split_two(tx: &mut Transaction) -> Result<bool, EffectError> {
    use crate::state::ZoneKind;

    let player = tx.state().turn.current;
    if !crate::query::has_draw_phase_scry_four_split_two(tx.state(), player) {
        return Ok(false);
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let available = tx
        .state()
        .zones
        .get(draw)
        .map_or(0, |zone| zone.cards.len());
    if available < 4 {
        crate::effect::reshuffle_discard_into_draw(tx)?;
    }
    let zone = tx
        .state()
        .zones
        .get(draw)
        .ok_or_else(|| EffectError::EngineFault("draw zone missing".into()))?;
    let take = zone.cards.len().min(4);
    if take < 2 {
        return Ok(false);
    }
    let peeked: Vec<u32> = zone.cards[zone.cards.len() - take..]
        .iter()
        .map(|card| card.raw())
        .collect();
    let mut payload = vec![player.raw(), take as u32];
    payload.extend_from_slice(&peeked);
    open_choice(
        tx,
        player,
        take as u8,
        take as u8,
        true,
        peeked,
        ChoiceKind::SelectCards,
        ContinuationKind::ReorderDrawPileFourSplitTwoThenDraw,
        payload,
    )?;
    Ok(true)
}

pub(super) fn open_optional_draw_bonus(
    tx: &mut Transaction,
    player: PlayerId,
    base_draw: u32,
    advance_after: bool,
) -> Result<bool, EffectError> {
    let bonuses = optional_draw_bonuses_for(tx.state(), player);
    if bonuses.is_empty() {
        return Ok(false);
    }
    let mut payload = vec![player.raw(), base_draw, u32::from(advance_after)];
    payload.extend(bonuses);
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        vec![0, 1],
        ChoiceKind::Confirm,
        ContinuationKind::OptionalDrawBonus,
        payload,
    )?;
    Ok(true)
}

pub(super) fn finish_replaced_draw_phase(tx: &mut Transaction) {
    let skip_play = tx.state().turn.skip_play();
    let next = if skip_play {
        tx.state_mut().turn.set_flag(turn_flag::SKIP_PLAY, 0);
        let event = record_phase_skipped(tx, tx.state().turn.current, Phase::Play);
        tx.push_event(event);
        if crate::query::restore_skipped_phase(tx.state(), tx.state().turn.current) {
            Phase::Play
        } else {
            Phase::Discard
        }
    } else {
        Phase::Play
    };
    tx.state_mut().turn.phase = next;
    let current = tx.state().turn.current;
    tx.push_event(engine_api::DomainEvent::TurnStateChanged {
        current,
        phase: next.to_tag(),
    });
}

/// Run a package-defined prepare-phase draw-pile scry. Selected cards go to the
/// bottom and the rest stay on top.
///
/// The draw pile stores top-of-deck as the LAST vec element. The payload lists
/// the peeked cards bottom→top so the resolver can rebuild the top slice.
fn maybe_open_prepare_scry(tx: &mut Transaction) -> Result<bool, EffectError> {
    use crate::state::ZoneKind;
    let cur = tx.state().turn.current;
    let n = crate::query::prepare_scry_count(tx.state(), cur);
    if n == 0 {
        return Ok(false);
    }
    // Ensure enough cards (reshuffle if the pile is short).
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let avail = tx
        .state()
        .zones
        .get(draw)
        .map(|z| z.cards.len())
        .unwrap_or(0);
    if (avail as u32) < n {
        crate::effect::reshuffle_discard_into_draw(tx)?;
    }
    let z = tx
        .state()
        .zones
        .get(draw)
        .ok_or_else(|| EffectError::EngineFault("draw zone missing".into()))?;
    let total = z.cards.len();
    if total == 0 {
        return Ok(false);
    }
    let take = (n as usize).min(total);
    // Top `take` cards = last `take` elements (bottom→top order as stored).
    let top: Vec<u32> = z.cards[total - take..].iter().map(|c| c.raw()).collect();
    let mut payload = vec![cur.raw(), take as u32];
    payload.extend_from_slice(&top);
    open_choice(
        tx,
        cur,
        0,
        take as u8,
        true, // cancel = keep all on top in current order
        top.clone(),
        ChoiceKind::SelectCards,
        ContinuationKind::ReorderDrawPile,
        payload,
    )?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::bootstrap_match;
    use crate::state::GameState;

    #[test]
    fn pre_normal_extra_draw_runs_as_a_real_second_draw_phase() {
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, 1, "extra-draw").unwrap();
        let player = tx.state().players.alive_by_seat()[0];
        tx.state_mut().turn.current = player;
        tx.state_mut().turn.phase = Phase::Judge;
        tx.state_mut().revision = 1;
        tx.state_mut()
            .turn
            .set_flag(turn_flag::PRE_NORMAL_EXTRA_DRAW, 1);
        let hand = tx.state().hand_count(player);

        handle_end_phase(&mut tx, player).unwrap();
        assert_eq!(tx.state().turn.phase, Phase::Draw);
        assert_eq!(tx.state().hand_count(player), hand + 2);
        handle_end_phase(&mut tx, player).unwrap();
        assert_eq!(tx.state().turn.phase, Phase::Draw);
        assert_eq!(tx.state().hand_count(player), hand + 4);
        handle_end_phase(&mut tx, player).unwrap();
        assert_eq!(tx.state().turn.phase, Phase::Play);
    }

    #[test]
    fn nonzero_fixed_draw_count_is_exact_and_ignores_pending_bonus() {
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, 2, "fixed-draw").unwrap();
        let player = tx.state().players.alive_by_seat()[0];
        tx.state_mut().turn.current = player;
        tx.state_mut().turn.phase = Phase::Draw;
        tx.state_mut().turn.set_draw_phase_fixed_count(3);
        tx.state_mut()
            .turn
            .set_flag(turn_flag::CURRENT_DRAW_COUNT_BONUS, 7);
        let hand = tx.state().hand_count(player);

        continue_draw_phase(&mut tx, player).unwrap();

        assert_eq!(tx.state().hand_count(player), hand + 3);
        assert_eq!(tx.state().turn.draw_phase_fixed_count(), Some(3));
        assert_eq!(tx.state().turn.flag(turn_flag::CURRENT_DRAW_COUNT_BONUS), 0);
    }
}
