//! Headless identity match runner with deterministic auto-player.

use crate::query::{query, QueryContext};
use crate::state::{CardKind, ContinuationKind, Phase};
use crate::Engine;
use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    MatchOutcome, PlayerId,
};
use rule_api::{QueryRequest, QueryResponse};
mod choices;
use choices::*;

/// Configuration for a headless run.
#[derive(Debug, Clone)]
pub struct MatchConfig {
    /// Player count 2..=8.
    pub player_count: u8,
    /// Rule RNG seed.
    pub rng_seed: u64,
    /// Max accepted commands before abort (safety).
    pub max_steps: u32,
    /// Ruleset name.
    pub ruleset: String,
}

impl Default for MatchConfig {
    fn default() -> Self {
        Self {
            player_count: 4,
            rng_seed: 42,
            max_steps: 5_000,
            ruleset: "identity-standard".into(),
        }
    }
}

/// Result of a finished (or capped) run.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MatchReport {
    /// Final outcome.
    pub outcome: MatchOutcome,
    /// Revisions committed.
    pub revisions: u64,
    /// Accepted commands count.
    pub steps: u32,
    /// Final state hash.
    pub state_hash: [u8; 32],
    /// Whether stopped by max_steps.
    pub capped: bool,
}

/// Run a full identity match with auto-players until terminal or cap.
pub fn run_identity_match(cfg: MatchConfig) -> Result<MatchReport, String> {
    let mut eng = Engine::new();
    let start = Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: cfg.player_count,
            rng_seed: cfg.rng_seed,
            ruleset: cfg.ruleset.clone(),
        },
    };
    match eng.submit(start) {
        CommandResult::Accepted { .. } => {}
        CommandResult::Rejected { reason } => {
            return Err(format!("start rejected: {reason:?}"));
        }
    }

    let mut steps = 1u32;
    while !eng.state().outcome.is_finished() && steps < cfg.max_steps {
        let cmd = choose_command(&eng);
        match eng.submit(cmd.clone()) {
            CommandResult::Accepted { .. } => {
                steps += 1;
            }
            CommandResult::Rejected { reason } => {
                // Try EndPhase as fallback once
                let actor = eng.state().turn.current;
                let fallback = Command {
                    actor,
                    expected_revision: ExpectedRevision(0),
                    idempotency_key: IdempotencyKey(0),
                    kind: CommandKind::EndPhase,
                };
                match eng.submit(fallback) {
                    CommandResult::Accepted { .. } => steps += 1,
                    CommandResult::Rejected { reason: r2 } => {
                        let roster: Vec<_> = eng
                            .state()
                            .players
                            .iter_live()
                            .map(|player| {
                                (player.id, player.character_def, player.hp, player.alive)
                            })
                            .collect();
                        return Err(format!(
                            "auto-player stuck at rev {} phase {:?} roster {roster:?} after {cmd:?}: {reason:?} / {r2:?}",
                            eng.state().revision,
                            eng.state().turn.phase,
                        ));
                    }
                }
            }
        }
    }

    let capped = !eng.state().outcome.is_finished();
    Ok(MatchReport {
        outcome: eng.state().outcome.clone(),
        revisions: eng.state().revision,
        steps,
        state_hash: eng.state_hash(),
        capped,
    })
}

/// Run a full guozhan (国战) match with auto-players until terminal or cap.
///
/// ADR 0598 §6 phase B. The auto-player reuses the identity heuristic's main loop; faction
/// enmity (instead of identity roles) is resolved inside `enemy_targets` via `player_group`.
pub fn run_guozhan_match(cfg: MatchConfig) -> Result<MatchReport, String> {
    let mut eng = Engine::new();
    let ruleset = if cfg.ruleset.starts_with("guozhan") {
        cfg.ruleset.clone()
    } else {
        "guozhan-standard".to_string()
    };
    let start = Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: cfg.player_count,
            rng_seed: cfg.rng_seed,
            ruleset,
        },
    };
    match eng.submit(start) {
        CommandResult::Accepted { .. } => {}
        CommandResult::Rejected { reason } => {
            return Err(format!("guozhan start rejected: {reason:?}"));
        }
    }

    let mut steps = 1u32;
    while !eng.state().outcome.is_finished() && steps < cfg.max_steps {
        let cmd = choose_command(&eng);
        match eng.submit(cmd.clone()) {
            CommandResult::Accepted { .. } => {
                steps += 1;
            }
            CommandResult::Rejected { reason } => {
                let actor = eng.state().turn.current;
                let fallback = Command {
                    actor,
                    expected_revision: ExpectedRevision(0),
                    idempotency_key: IdempotencyKey(0),
                    kind: CommandKind::EndPhase,
                };
                match eng.submit(fallback) {
                    CommandResult::Accepted { .. } => steps += 1,
                    CommandResult::Rejected { reason: r2 } => {
                        return Err(format!(
                            "guozhan auto-player stuck at rev {} phase {:?} after {cmd:?}: {reason:?} / {r2:?}",
                            eng.state().revision,
                            eng.state().turn.phase,
                        ));
                    }
                }
            }
        }
    }

    let capped = !eng.state().outcome.is_finished();
    Ok(MatchReport {
        outcome: eng.state().outcome.clone(),
        revisions: eng.state().revision,
        steps,
        state_hash: eng.state_hash(),
        capped,
    })
}

/// Pick next command for the current state (deterministic heuristic AI).
pub fn choose_command(eng: &Engine) -> Command {
    let st = eng.state();

    // Resolve open choice first
    if st.has_pending_choice() {
        return resolve_pending_choice(eng);
    }

    let actor = st.turn.current;
    let phase = st.turn.phase;

    // ADR 0598 phase C: in guozhan, the auto-player reveals any face-down generals during
    // its own play phase before taking other actions. Identity mode never enters here.
    if st.ruleset.name.starts_with("guozhan")
        && phase == Phase::Play
        && !actor.is_none()
    {
        if let Some(slot) = st.players.get(actor).and_then(|p| {
            p.unseen.iter().position(|&u| u).map(|i| i as u8)
        }) {
            return Command {
                actor,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(0),
                kind: CommandKind::RevealGeneral { slot },
            };
        }
    }

    match phase {
        Phase::Play => {
            if let Some(cmd) = try_play_action(eng, actor) {
                return cmd;
            }
            end_phase(actor)
        }
        Phase::Draw | Phase::Prepare | Phase::Judge | Phase::Discard | Phase::End | Phase::Idle => {
            end_phase(actor)
        }
    }
}

fn end_phase(actor: PlayerId) -> Command {
    Command {
        actor,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::EndPhase,
    }
}

fn try_activate_skill(eng: &Engine, actor: PlayerId) -> Option<Command> {
    let state = eng.state();
    let player = state.players.get(actor)?;
    let targets: Vec<PlayerId> = state
        .players
        .iter_live()
        .filter(|candidate| candidate.alive && candidate.id != actor)
        .map(|candidate| candidate.id)
        .collect();
    let hand = state.hand_cards(actor);

    for skill_id in &player.skills {
        let Some(skill) = state.skills.get(*skill_id) else {
            continue;
        };
        if !state.skill_effectively_enabled(skill) {
            continue;
        }

        let view_as = state.content.view_as_defs_for_skill(skill.def);
        if !view_as.is_empty() {
            if skill.use_count > 0 {
                continue;
            }
            for card in &hand {
                for target in std::iter::once(None).chain(targets.iter().copied().map(Some)) {
                    let cmd = activate(actor, *skill_id, target.into_iter().collect(), vec![*card]);
                    let mut trial = eng.clone();
                    if matches!(trial.submit(cmd.clone()), CommandResult::Accepted { .. }) {
                        return Some(cmd);
                    }
                }
            }
        } else if let Some(active) = state.content.first_active_for_skill(skill.def) {
            // The deterministic auto-player voluntarily tries an unlimited
            // active skill at most once per turn; this is not a rule limit.
            let limit_reached = active
                .uses_per_turn
                .is_some_and(|limit| skill.use_count >= u32::from(limit))
                || active
                    .uses_per_play_phase
                    .is_some_and(|limit| skill.play_phase_use_count >= u32::from(limit))
                || (active.uses_per_turn.is_none()
                    && active.uses_per_turn_source.is_none()
                    && active.uses_per_play_phase.is_none()
                    && skill.use_count >= 1);
            if limit_reached {
                continue;
            }
            for selected_targets in active_target_sets(state, actor, &active) {
                let cmd = activate(actor, *skill_id, selected_targets, Vec::new());
                let mut trial = eng.clone();
                if matches!(trial.submit(cmd.clone()), CommandResult::Accepted { .. }) {
                    return Some(cmd);
                }
            }
        }
    }
    None
}

fn play(actor: PlayerId, card: CardId, targets: Vec<PlayerId>) -> Command {
    Command {
        actor,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::PlayCard {
            card,
            targets,
            card_targets: vec![],
        },
    }
}

fn activate(
    actor: PlayerId,
    skill: engine_api::SkillId,
    targets: Vec<PlayerId>,
    card_targets: Vec<CardId>,
) -> Command {
    Command {
        actor,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::ActivateSkill {
            skill,
            targets,
            card_targets,
        },
    }
}

fn active_target_sets(
    state: &crate::state::GameState,
    actor: PlayerId,
    active: &crate::content::ActiveSkillDef,
) -> Vec<Vec<PlayerId>> {
    let Some(spec) = &active.targets else {
        return vec![Vec::new()];
    };
    let players: Vec<PlayerId> = state
        .players
        .iter_live()
        .filter(|candidate| {
            candidate.alive
                && (spec.allow_self || candidate.id != actor)
                && (!spec.male
                    || state
                        .content
                        .character(candidate.character_def)
                        .is_some_and(|character| {
                            matches!(character.sex.as_str(), "male" | "double")
                        }))
                && (!spec.hand_gt_owner || state.hand_count(candidate.id) > state.hand_count(actor))
                && (!spec.hand_lte_owner
                    || state.hand_count(candidate.id) <= state.hand_count(actor))
                && (!spec.hand_nonempty || state.hand_count(candidate.id) > 0)
        })
        .map(|candidate| candidate.id)
        .collect();
    let mut out = Vec::new();
    let max = if spec.max_by_owner_hp {
        usize::from(spec.max).min(
            state
                .players
                .get(actor)
                .map(|player| player.hp.max(0) as usize)
                .unwrap_or(0),
        )
    } else {
        usize::from(spec.max)
    };
    for count in usize::from(spec.min)..=max.min(players.len()) {
        collect_player_combinations(&players, count, 0, &mut Vec::new(), &mut out);
    }
    if spec.pair_hand_difference_lte_three {
        out.retain(|targets| {
            let [left, right] = targets.as_slice() else {
                return false;
            };
            state.hand_count(*left).abs_diff(state.hand_count(*right)) <= 3
        });
    }
    out
}

fn collect_player_combinations(
    players: &[PlayerId],
    needed: usize,
    index: usize,
    current: &mut Vec<PlayerId>,
    out: &mut Vec<Vec<PlayerId>>,
) {
    if current.len() == needed {
        out.push(current.clone());
        return;
    }
    let remaining = needed.saturating_sub(current.len());
    if players.len().saturating_sub(index) < remaining {
        return;
    }
    for next in index..players.len() {
        current.push(players[next]);
        collect_player_combinations(players, needed, next + 1, current, out);
        current.pop();
    }
}

/// First card id in target zones: hand → equip → judge.
fn pick_zone_card_token(
    st: &crate::state::GameState,
    actor: PlayerId,
    target: PlayerId,
    hand: bool,
    equip: bool,
    judge: bool,
    discard: bool,
) -> Option<u32> {
    use crate::state::ZoneKind;
    if hand {
        if let Some(c) = st.hand_cards(target).first() {
            return Some(c.raw());
        }
    }
    if equip {
        if let Some(eq) = st.zones.find(ZoneKind::Equip, target) {
            if let Some(z) = st.zones.get(eq) {
                if let Some(c) = z.cards.iter().find(|card| {
                    !discard
                        || !crate::query::card_discard_protected_from(st, actor, target, **card)
                }) {
                    return Some(c.raw());
                }
            }
        }
    }
    if judge {
        if let Some(jz) = st.zones.find(ZoneKind::Judge, target) {
            if let Some(z) = st.zones.get(jz) {
                if let Some(c) = z.cards.first() {
                    return Some(c.raw());
                }
            }
        }
    }
    None
}

fn enemy_targets(eng: &Engine, actor: PlayerId) -> Vec<PlayerId> {
    use crate::query::QueryContext;
    use crate::state::Identity;
    use rule_api::RelationTag;

    let st = eng.state();
    let mut ctx = QueryContext::new();
    let mut enemies = Vec::new();

    // ADR 0598: guozhan enmity is by faction (同势力 = ally, 不同势力 = enemy); identity
    // mode keeps the role-based判定.
    let is_guozhan = st.ruleset.name.starts_with("guozhan");
    let my_group = if is_guozhan {
        st.player_group(actor).map(str::to_string)
    } else {
        None
    };

    for p in st.players.iter_live() {
        if !p.alive || p.id == actor {
            continue;
        }
        let is_enemy = if is_guozhan {
            // Guozhan: different faction (or either side unresolved) => enemy.
            let their_group = st.player_group(p.id);
            match (&my_group, their_group) {
                (Some(mine), Some(theirs)) => mine != theirs,
                _ => true,
            }
        } else {
            // Identity role-based判定 (unchanged).
            let me = st.players.get(actor).map(|x| x.identity);
            match me {
                Some(Identity::Renegade) => true,
                Some(Identity::Lord) | Some(Identity::Loyalist) => {
                    matches!(p.identity, Identity::Rebel | Identity::Renegade)
                }
                Some(Identity::Rebel) => {
                    matches!(p.identity, Identity::Lord | Identity::Loyalist)
                }
                None => true,
            }
        };
        let _ = (
            query(st, &QueryRequest::Relation { a: actor, b: p.id }, &mut ctx),
            RelationTag::Enemy,
        );
        if is_enemy {
            enemies.push(p.id);
        }
    }
    // Stable order by distance then id
    enemies.sort_by_key(|t| {
        let mut ctx = QueryContext::new();
        let d = match query(
            st,
            &QueryRequest::Distance {
                from: actor,
                to: *t,
            },
            &mut ctx,
        ) {
            QueryResponse::Int(v) => v,
            _ => 99,
        };
        (d, t.raw())
    });
    enemies
}

fn sha_legal(eng: &Engine, actor: PlayerId, card: CardId, target: PlayerId) -> bool {
    if crate::query::card_distance_unlimited_for_card(eng.state(), actor, card) {
        return true;
    }
    let mut ctx = QueryContext::new();
    let dist = query(
        eng.state(),
        &QueryRequest::Distance {
            from: actor,
            to: target,
        },
        &mut ctx,
    );
    let range = query(
        eng.state(),
        &QueryRequest::AttackRange { player: actor },
        &mut ctx,
    );
    matches!(
        (dist, range),
        (QueryResponse::Int(d), QueryResponse::Int(r)) if d <= r
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_match_terminates_or_caps_cleanly() {
        let report = run_identity_match(MatchConfig {
            player_count: 2,
            rng_seed: 7,
            max_steps: 200,
            ruleset: "identity-standard".into(),
        })
        .expect("run");
        assert!(report.steps >= 1);
        assert!(report.revisions >= 1);
        // Either finished or capped without panic
        if !report.capped {
            assert!(report.outcome.is_finished());
        }
    }

    #[test]
    fn four_player_deterministic() {
        let a = run_identity_match(MatchConfig {
            player_count: 4,
            rng_seed: 99,
            max_steps: 800,
            ..Default::default()
        })
        .unwrap();
        let b = run_identity_match(MatchConfig {
            player_count: 4,
            rng_seed: 99,
            max_steps: 800,
            ..Default::default()
        })
        .unwrap();
        assert_eq!(a.state_hash, b.state_hash);
        assert_eq!(a.outcome, b.outcome);
        assert_eq!(a.steps, b.steps);
    }

    #[test]
    fn guozhan_match_runs_and_assigns_factions() {
        // ADR 0598 phase B: a guozhan match must bootstrap (double-general dealing +
        // faction assignment), run the auto-player loop without panicking, and reach a
        // terminal outcome or the step cap.
        let report = run_guozhan_match(MatchConfig {
            player_count: 4,
            rng_seed: 42,
            max_steps: 2000,
            ruleset: "guozhan-standard".into(),
        })
        .expect("guozhan run");
        assert!(report.steps >= 1);
        assert!(report.revisions >= 1);
        if !report.capped {
            assert!(report.outcome.is_finished());
        }
    }

    #[test]
    fn guozhan_match_is_deterministic() {
        let a = run_guozhan_match(MatchConfig {
            player_count: 4,
            rng_seed: 42,
            max_steps: 2000,
            ruleset: "guozhan-standard".into(),
        })
        .unwrap();
        let b = run_guozhan_match(MatchConfig {
            player_count: 4,
            rng_seed: 42,
            max_steps: 2000,
            ruleset: "guozhan-standard".into(),
        })
        .unwrap();
        assert_eq!(a.state_hash, b.state_hash);
        assert_eq!(a.outcome, b.outcome);
        assert_eq!(a.steps, b.steps);
    }
}
