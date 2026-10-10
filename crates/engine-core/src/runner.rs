//! Headless identity match runner with deterministic auto-player.

use crate::query::{query, QueryContext};
use crate::state::{CardKind, ContinuationKind, Phase};
use crate::Engine;
use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    MatchOutcome, PlayerId,
};
use rule_api::{QueryRequest, QueryResponse};

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

fn resolve_pending_choice(eng: &Engine) -> Command {
    let st = eng.state();
    let choice_id = ChoiceId::new(st.resolution.pending_choice_id);
    let observer = PlayerId::new(st.resolution.pending_observer);
    let kind = st.resolution.continuation_kind;
    let payload = &st.resolution.continuation_payload;
    let offered_tokens = || -> Vec<u32> {
        eng.last_choice()
            .map(|choice| choice.options.iter().map(|option| option.token).collect())
            .unwrap_or_default()
    };

    let (selections, cancelled) = match kind {
        ContinuationKind::DyingSave => {
            // Auto: current responder saves with the first legal 桃 (or self-save 酒).
            let dying = PlayerId::new(*payload.first().unwrap_or(&0));
            if crate::query::dying_save_card_for(st, observer, dying).is_some() {
                (vec![1u32], false)
            } else if let Some(token) = offered_tokens().into_iter().find(|token| *token != 0) {
                (vec![token], false)
            } else {
                (vec![0u32], false)
            }
        }
        ContinuationKind::DiscardToLimit => {
            let need = *payload.get(1).unwrap_or(&0) as usize;
            let hand = crate::query::discard_to_limit_tokens(st, observer);
            let sel: Vec<u32> = hand.iter().take(need).map(|c| c.raw()).collect();
            (sel, false)
        }
        ContinuationKind::AdjustHandTo => {
            let need = *payload.get(1).unwrap_or(&0) as usize;
            (offered_tokens().into_iter().take(need).collect(), false)
        }
        ContinuationKind::JudgeDamageOtherOnSuit => {
            (offered_tokens().into_iter().take(1).collect(), false)
        }
        ContinuationKind::GanglieSourceDiscardOrDamage => {
            (offered_tokens().into_iter().take(2).collect(), false)
        }
        ContinuationKind::ResolveGanglieThree => {
            (offered_tokens().into_iter().take(1).collect(), false)
        }
        ContinuationKind::ResolveSpXiaoguo => (Vec::new(), true),
        ContinuationKind::ChooseDingpanBranch => {
            (offered_tokens().into_iter().take(1).collect(), false)
        }
        ContinuationKind::ResolveTanlu => {
            let count = eng.last_choice().map_or(1, |choice| choice.min_count) as usize;
            (offered_tokens().into_iter().take(count).collect(), false)
        }
        ContinuationKind::TuxiGainOtherHandsReplaceDraw => {
            let count = if payload.first().copied() == Some(0) {
                eng.last_choice().map_or(1, |choice| choice.max_count) as usize
            } else {
                1
            };
            (offered_tokens().into_iter().take(count).collect(), false)
        }
        ContinuationKind::YijiDistributeTopCards => {
            let count = if payload.first().copied() == Some(0) {
                eng.last_choice().map_or(1, |choice| choice.max_count) as usize
            } else {
                1
            };
            (offered_tokens().into_iter().take(count).collect(), false)
        }
        ContinuationKind::DrawThenOptionallyDistributeHandCards => {
            if payload.first().copied() == Some(0) {
                (Vec::new(), true)
            } else {
                (offered_tokens().into_iter().take(1).collect(), false)
            }
        }
        ContinuationKind::GiveSelectedThenContinueDistinctRecoverAtTwo => {
            if payload.first().copied() == Some(0) {
                (Vec::new(), true)
            } else {
                (offered_tokens().into_iter().take(1).collect(), false)
            }
        }
        ContinuationKind::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan => {
            (offered_tokens().into_iter().take(1).collect(), false)
        }
        ContinuationKind::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver => {
            (offered_tokens().into_iter().take(1).collect(), false)
        }
        ContinuationKind::ChooseGainCardFromEachOtherThenTurnOver => {
            (offered_tokens().into_iter().take(1).collect(), false)
        }
        ContinuationKind::ChooseHiddenSkillPileHandCards => {
            (offered_tokens().into_iter().take(1).collect(), false)
        }
        ContinuationKind::ChooseHiddenSkillPileCards => {
            let count = payload.get(5).copied().unwrap_or(1) as usize;
            (offered_tokens().into_iter().take(count).collect(), false)
        }
        ContinuationKind::DiscardHandForMassDamage => {
            let count = payload.get(1).copied().unwrap_or(0) as usize;
            (offered_tokens().into_iter().take(count).collect(), false)
        }
        ContinuationKind::ChooseMarkParityControlTarget
        | ContinuationKind::DiscardMarkParityControlTargetCard
        | ContinuationKind::ConfirmMarkParityMassDamage
        | ContinuationKind::ChooseSelectedTargetDamage
        | ContinuationKind::ChooseInspectedHeartCard
        | ContinuationKind::ChooseInspectedHeartDestination
        | ContinuationKind::ChooseYeyanMode
        | ContinuationKind::ChooseYeyanTargets
        | ContinuationKind::ChooseYeyanSingleDamage
        | ContinuationKind::ChooseYeyanDoubleTarget
        | ContinuationKind::ChooseAllLoseHpOrRecover
        | ContinuationKind::ChooseQixingHandCards
        | ContinuationKind::ChooseQixingWeatherCards => {
            (offered_tokens().into_iter().take(1).collect(), false)
        }
        ContinuationKind::ChooseQixingPileCards | ContinuationKind::ChooseQixingWeatherTargets => {
            let count = eng.last_choice().map_or(1, |choice| choice.min_count) as usize;
            (offered_tokens().into_iter().take(count).collect(), false)
        }
        ContinuationKind::ChooseDistinctSuitCards => {
            let player = PlayerId::new(payload.first().copied().unwrap_or(0));
            let mut suits = std::collections::BTreeSet::new();
            let selected = offered_tokens()
                .into_iter()
                .filter(|token| {
                    st.cards.get(CardId::new(*token)).is_some_and(|card| {
                        suits.insert(crate::command::judge::suit_tag(
                            crate::query::effective_card_suit(st, player, card),
                        ))
                    })
                })
                .collect();
            (selected, false)
        }
        ContinuationKind::ChooseYeyanBigCards => {
            let player = PlayerId::new(payload.first().copied().unwrap_or(0));
            let mut suits = std::collections::BTreeSet::new();
            let selected = offered_tokens()
                .into_iter()
                .filter(|token| {
                    st.cards.get(CardId::new(*token)).is_some_and(|card| {
                        suits.insert(crate::command::judge::suit_tag(
                            crate::query::effective_card_suit(st, player, card),
                        ))
                    })
                })
                .take(4)
                .collect();
            (selected, false)
        }
        ContinuationKind::TongjiDiscardRedirectSha => (Vec::new(), true),
        ContinuationKind::LiuliDiscardRedirectSha => (Vec::new(), true),
        ContinuationKind::JijieChooseBottomCardRecipient => {
            (offered_tokens().into_iter().take(1).collect(), false)
        }
        ContinuationKind::LuoshenRepeatBlackJudgment => (Vec::new(), true),
        ContinuationKind::JudgeSameColorGainRepeat => (Vec::new(), true),
        ContinuationKind::MoveEquipmentThenDraw => {
            (offered_tokens().into_iter().take(1).collect(), false)
        }
        ContinuationKind::ChooseMoveTargetEquipmentOrDraw => {
            (offered_tokens().into_iter().take(1).collect(), false)
        }
        ContinuationKind::ChooseRecoverOwnerOrDamageByGivenEquipment => (Vec::new(), true),
        ContinuationKind::ChooseTargetDiscardWeaponRangeHandOrIncreaseDamage => (Vec::new(), true),
        ContinuationKind::ChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth => {
            let count = eng.last_choice().map_or(1, |choice| choice.min_count) as usize;
            (offered_tokens().into_iter().take(count).collect(), false)
        }
        ContinuationKind::ChooseHandCountDifferentTargetDrawOrRandomDiscard => {
            (offered_tokens().into_iter().take(1).collect(), false)
        }
        ContinuationKind::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw => {
            match payload.first().copied().unwrap_or(0) {
                2 | 5 => (Vec::new(), true),
                1 | 4 => {
                    let required = payload.get(4).copied().unwrap_or(0) as usize;
                    (offered_tokens().into_iter().take(required).collect(), false)
                }
                _ => (offered_tokens().into_iter().take(1).collect(), false),
            }
        }
        ContinuationKind::ChooseGainOneFromDamageTargetAndNeighbors => {
            (offered_tokens().into_iter().take(1).collect(), false)
        }
        ContinuationKind::ChoosePreventDamageOrLoseMaxHpIncreaseDamage => {
            (offered_tokens().into_iter().take(1).collect(), false)
        }
        ContinuationKind::DiscardEquipmentPairWithArmor => {
            (offered_tokens().into_iter().take(1).collect(), false)
        }
        ContinuationKind::DiscardTargetEquipmentGainMount
        | ContinuationKind::DiscardTargetEquipmentDrawWeaponRange => {
            (offered_tokens().into_iter().take(1).collect(), false)
        }
        ContinuationKind::ChooseTargetRevealHandDamageIfSha => {
            (offered_tokens().into_iter().take(1).collect(), false)
        }
        ContinuationKind::ChooseOtherGiveOneHandEquip => {
            (offered_tokens().into_iter().take(1).collect(), false)
        }
        ContinuationKind::DiscardTargetCardGainMount => {
            (offered_tokens().into_iter().take(1).collect(), false)
        }
        ContinuationKind::ChooseDiscardTargetCard => {
            (offered_tokens().into_iter().take(1).collect(), false)
        }
        ContinuationKind::ChooseDiscardTargetCardResolveCurrentSha
        | ContinuationKind::ResolveDcqiaomeng
        | ContinuationKind::ResolveRetieji
        | ContinuationKind::ResolveNewReluoyi => {
            (offered_tokens().into_iter().take(1).collect(), false)
        }
        ContinuationKind::ChooseGainTopCardsBySuit => (offered_tokens(), false),
        ContinuationKind::ChooseOtherDrawByHandCount => {
            (offered_tokens().into_iter().take(1).collect(), false)
        }
        ContinuationKind::ChooseDiscardOneThenGainTargetEquipment => {
            (offered_tokens().into_iter().take(1).collect(), false)
        }
        ContinuationKind::ChooseTargetDraw
        | ContinuationKind::ChooseTargetsDamage
        | ContinuationKind::ChooseDiscardTargetCardsRecoverSameCategory
        | ContinuationKind::ChooseScskuijiDiscard
        | ContinuationKind::ChoosePskuijiDiscard => {
            let count = eng.last_choice().map_or(1, |choice| choice.min_count) as usize;
            (offered_tokens().into_iter().take(count).collect(), false)
        }
        ContinuationKind::ChooseCurrentCardUseEnhancements => {
            (offered_tokens().into_iter().take(1).collect(), false)
        }
        ContinuationKind::ChooseAddCurrentCardTarget => {
            (offered_tokens().into_iter().take(1).collect(), false)
        }
        ContinuationKind::ChooseAddOrRemoveCurrentCardTarget => {
            (offered_tokens().into_iter().take(1).collect(), false)
        }
        ContinuationKind::ChooseHandCompareTarget
        | ContinuationKind::ChooseHandComparePlayerCard
        | ContinuationKind::ChooseCompareTargetHandCardSecondFailureSeatDamage
        | ContinuationKind::ChooseDrawOrDiscardTargetCard
        | ContinuationKind::ChooseSeatPathDamage
        | ContinuationKind::ChooseOtherDamage
        | ContinuationKind::ChooseOtherDiscardTargetCard
        | ContinuationKind::ChooseGainDiscardCardByKind
        | ContinuationKind::ChooseGainOwnLostDiscardCardThisTurn
        | ContinuationKind::ChooseMoveFieldCardTo
        | ContinuationKind::ChooseTakeLargerHandUntilMax
        | ContinuationKind::ChooseDiscardDamageEndpointCardIncreaseIfSuit
        | ContinuationKind::RevealHandChooseDiscardColor
        | ContinuationKind::ChooseDrawOrIncreaseCurrentCardDamageOrBothMark => {
            (offered_tokens().into_iter().take(1).collect(), false)
        }
        ContinuationKind::PickTargetCard => {
            // payload: [target, actor, mode]; mode 0=过河(手/装/判) 1=顺手(手/装)
            let target = PlayerId::new(*payload.first().unwrap_or(&0));
            let actor = PlayerId::new(*payload.get(1).unwrap_or(&0));
            let mode = *payload.get(2).unwrap_or(&0);
            let include_judge = mode == 0;
            let sel = pick_zone_card_token(st, actor, target, true, true, include_judge, mode == 0)
                .map(|c| vec![c])
                .unwrap_or_else(|| vec![0]);
            (sel, false)
        }
        ContinuationKind::HuogongShow => {
            // Target shows first hand card
            let target = PlayerId::new(*payload.get(1).unwrap_or(&0));
            let hand = st.hand_cards(target);
            let sel = hand
                .first()
                .map(|c| vec![c.raw()])
                .unwrap_or_else(|| vec![0]);
            (sel, false)
        }
        ContinuationKind::HuogongDiscard => {
            // Actor discards first same-suit hand card, else cancel (fire fails)
            let actor = PlayerId::new(*payload.first().unwrap_or(&0));
            let suit = crate::command::judge::suit_from_tag(*payload.get(2).unwrap_or(&1));
            let match_card = st.hand_cards(actor).into_iter().find(|cid| {
                st.cards
                    .get(*cid)
                    .map(|c| crate::query::effective_card_suit(st, actor, c) == suit)
                    .unwrap_or(false)
            });
            if let Some(c) = match_card {
                (vec![c.raw()], false)
            } else {
                (vec![], true)
            }
        }
        ContinuationKind::WuxieWindow => {
            // Headless: always pass (do not spend 无懈) for match stability
            (vec![], true)
        }
        ContinuationKind::RespondToSha => {
            // Prefer real 闪; if only 八卦 available try it, else cancel
            let target = PlayerId::new(*payload.get(1).unwrap_or(&0));
            let shan = crate::command::play::response_shan_tokens(st, target);
            if let Some(t) = shan.first() {
                (vec![*t], false)
            } else if crate::content::player_has_bagua(st, target) {
                (vec![crate::content::BAGUA_RESPONSE_TOKEN], false)
            } else {
                (vec![], true)
            }
        }
        ContinuationKind::ActiveSkillSelection => {
            let Some(choice) = eng.last_choice() else {
                return end_phase(observer);
            };
            if choice.min_count == 0 && choice.allow_cancel {
                (Vec::new(), true)
            } else {
                let selection = payload
                    .first()
                    .map(|skill| engine_api::SkillId::new(*skill))
                    .and_then(|skill| st.skills.get(skill))
                    .and_then(|skill| st.content.first_active_for_skill(skill.def))
                    .and_then(|active| active.selection);
                let cards: Vec<_> = choice
                    .options
                    .iter()
                    .map(|option| engine_api::CardId::new(option.token))
                    .collect();
                let selected = if selection
                    .as_ref()
                    .is_some_and(|selection| selection.same_suit)
                {
                    crate::query::effective_suit_card_groups(st, observer, &cards)
                        .into_iter()
                        .find_map(|group| {
                            if let Some(sum) = selection
                                .as_ref()
                                .and_then(|selection| selection.sum_effective_number_exact)
                            {
                                crate::query::effective_number_sum_subsets(
                                    st,
                                    observer,
                                    &group,
                                    usize::from(choice.min_count),
                                    usize::from(choice.max_count),
                                    sum,
                                    1,
                                )
                                .into_iter()
                                .next()
                            } else if group.len() >= usize::from(choice.min_count) {
                                Some(
                                    group
                                        .into_iter()
                                        .take(usize::from(choice.min_count))
                                        .map(engine_api::CardId::raw)
                                        .collect(),
                                )
                            } else {
                                None
                            }
                        })
                } else if let Some(sum) = selection
                    .as_ref()
                    .and_then(|selection| selection.sum_effective_number_exact)
                {
                    crate::query::effective_number_sum_subsets(
                        st,
                        observer,
                        &cards,
                        usize::from(choice.min_count),
                        usize::from(choice.max_count),
                        sum,
                        1,
                    )
                    .into_iter()
                    .next()
                } else {
                    None
                };
                if let Some(selected) = selected {
                    return Command {
                        actor: observer,
                        expected_revision: ExpectedRevision(0),
                        idempotency_key: IdempotencyKey(0),
                        kind: CommandKind::ResolveChoice {
                            choice_id,
                            selections: selected,
                            cancelled: false,
                        },
                    };
                }
                (
                    choice
                        .options
                        .iter()
                        .take(choice.min_count as usize)
                        .map(|option| option.token)
                        .collect(),
                    false,
                )
            }
        }
        ContinuationKind::RespondAoeQueue => {
            // Auto: play first legal response for current target, else cancel
            let kind = *payload.first().unwrap_or(&0);
            let idx = *payload.get(3).unwrap_or(&0) as usize;
            let target = PlayerId::new(*payload.get(5 + idx).unwrap_or(&0));
            match kind {
                1 => {
                    let tokens = crate::command::play::response_sha_tokens(st, target);
                    if let Some(t) = tokens.first() {
                        (vec![*t], false)
                    } else {
                        (vec![], true)
                    }
                }
                2 => {
                    let shan = crate::command::play::response_shan_tokens(st, target);
                    if let Some(t) = shan.first() {
                        (vec![*t], false)
                    } else if crate::content::player_has_bagua(st, target) {
                        (vec![crate::content::BAGUA_RESPONSE_TOKEN], false)
                    } else {
                        (vec![], true)
                    }
                }
                _ => (vec![], true),
            }
        }
        ContinuationKind::JuedouDuel => {
            let next = PlayerId::new(*payload.get(2).unwrap_or(&0));
            let tokens = crate::command::play::response_sha_tokens(st, next);
            if let Some(t) = tokens.first() {
                (vec![*t], false)
            } else {
                (vec![], true)
            }
        }
        ContinuationKind::RewriteJudgment | ContinuationKind::RewriteBeigeJudgment => {
            // Auto: cancel (keep original judgment) for headless stability
            (vec![], true)
        }
        ContinuationKind::JiedaoChoice => {
            // Prefer give weapon (0) for simplicity
            (vec![0], false)
        }
        ContinuationKind::ReorderDrawPile => {
            // Headless: keep all cards on top in current order (cancel).
            (vec![], true)
        }
        ContinuationKind::ReorderDrawPileFourSplitTwoThenDraw => {
            // Headless: decline the optional draw-phase reorder.
            (vec![], true)
        }
        ContinuationKind::GiveOneHandOrEquip => {
            (offered_tokens().into_iter().take(1).collect(), false)
        }
        ContinuationKind::ChooseGiveOneHandEquipOrRecover => {
            (offered_tokens().into_iter().take(1).collect(), false)
        }
        ContinuationKind::ChooseGiveTwoDifferentColorsOrRecover => {
            let offered = offered_tokens();
            let pair = offered.iter().enumerate().find_map(|(index, first)| {
                let first = engine_api::CardId::new(*first);
                let color = crate::query::effective_card_color_mask(st, observer, first);
                offered[index + 1..].iter().find_map(|second| {
                    let second_card = engine_api::CardId::new(*second);
                    (crate::query::effective_card_color_mask(st, observer, second_card) != color)
                        .then_some(vec![first.raw(), second_card.raw()])
                })
            });
            pair.map_or((Vec::new(), true), |cards| (cards, false))
        }
        ContinuationKind::ResolveMbzhuguo => match payload.first().copied() {
            Some(3) => (vec![*payload.get(3).unwrap_or(&0)], false),
            Some(1 | 2) => (offered_tokens().into_iter().take(1).collect(), false),
            _ => {
                let count = eng.last_choice().map_or(0, |choice| choice.min_count) as usize;
                (offered_tokens().into_iter().take(count).collect(), false)
            }
        },
        ContinuationKind::ResolveMbanxian => {
            let count = eng.last_choice().map_or(1, |choice| choice.min_count) as usize;
            (offered_tokens().into_iter().take(count).collect(), false)
        }
        ContinuationKind::ResolveMbjinzu => (offered_tokens().into_iter().take(2).collect(), false),
        ContinuationKind::DrawReplacement => {
            // Headless: keep the ordinary draw for match stability.
            (vec![], true)
        }
        ContinuationKind::RevealDiscardSuitGainOrUse => {
            let stage = *payload.first().unwrap_or(&0);
            let len = *payload.get(6).unwrap_or(&0) as usize;
            let extra = payload.get(7 + len..).unwrap_or_default();
            match stage {
                1 => {
                    let options = offered_tokens();
                    if options.contains(&2) {
                        (vec![2], false)
                    } else {
                        (options.into_iter().take(1).collect(), false)
                    }
                }
                2 | 4 => (offered_tokens().into_iter().take(1).collect(), false),
                3 => {
                    let recipient = offered_tokens()
                        .into_iter()
                        .find(|token| *token == observer.raw())
                        .or_else(|| offered_tokens().into_iter().next());
                    (recipient.into_iter().collect(), false)
                }
                5 => {
                    let min = eng.last_choice().map_or(0, |choice| choice.min_count) as usize;
                    (offered_tokens().into_iter().take(min).collect(), false)
                }
                6 => (vec![1], false),
                _ => {
                    let _ = extra;
                    (vec![], true)
                }
            }
        }
        ContinuationKind::ChooseRevealOtherHandCardRememberCategory => {
            (offered_tokens().into_iter().take(1).collect(), false)
        }
        ContinuationKind::GiveExactRecordPhaseTarget => {
            let count = eng.last_choice().map_or(0, |choice| choice.min_count) as usize;
            (offered_tokens().into_iter().take(count).collect(), false)
        }
        ContinuationKind::ChooseGainResolvedShaOrShanThenOptionalUseSha => {
            let stage = eng
                .state()
                .resolution
                .continuation_payload
                .first()
                .copied()
                .unwrap_or(u32::MAX);
            match stage {
                0 | 1 => (offered_tokens().into_iter().take(1).collect(), false),
                2 => (Vec::new(), true),
                3 => {
                    let min = eng.last_choice().map_or(0, |choice| choice.min_count) as usize;
                    (offered_tokens().into_iter().take(min).collect(), false)
                }
                _ => (Vec::new(), true),
            }
        }
        _ => {
            let choice = eng.last_choice();
            let count = choice.map_or(1, |c| c.min_count) as usize;
            let offered = offered_tokens();
            if count == 0 {
                // Optional/min-0 choices (e.g. 骁果 cost): decline via cancel
                // instead of submitting empty selections, which resolvers
                // reject as "requires exactly one ...".
                (Vec::new(), true)
            } else if offered.is_empty() {
                (vec![0u32], false)
            } else {
                (offered.into_iter().take(count).collect(), false)
            }
        }
    };

    Command {
        actor: observer,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
    }
}

fn try_play_action(eng: &Engine, actor: PlayerId) -> Option<Command> {
    let st = eng.state();
    let hand = st.hand_cards(actor);
    let me = st.players.get(actor)?;
    let kind_of = |card| {
        st.cards
            .get(card)
            .map(|value| crate::query::effective_card_kind(st, actor, value))
    };

    // 1) Tao if wounded
    if me.hp < me.max_hp as i32 {
        if let Some(card) = hand.iter().find(|c| kind_of(**c) == Some(CardKind::Tao)) {
            return Some(play(actor, *card, vec![]));
        }
    }

    // 2) Equip first available equipment
    if let Some(card) = hand
        .iter()
        .find(|c| kind_of(**c).is_some_and(CardKind::is_equip))
    {
        return Some(play(actor, *card, vec![]));
    }

    // 3) Wuzhong
    if let Some(card) = hand
        .iter()
        .find(|c| kind_of(**c) == Some(CardKind::TrickWuzhong))
    {
        return Some(play(actor, *card, vec![]));
    }

    // 4) Sha nearest enemy (by distance, then seat id)
    let sha_used = crate::query::sha_play_phase_use_count(st);
    let card = hand.iter().find(|card| {
        kind_of(**card).is_some_and(CardKind::is_sha)
            && sha_used < crate::query::sha_use_limit_for_card(st, actor, **card)
    });
    if let Some(card) = card {
        for target in enemy_targets(eng, actor) {
            if sha_legal(eng, actor, *card, target) {
                return Some(play(actor, *card, vec![target]));
            }
        }
    }

    // 5) Guohe / Shunshou / Huogong
    if let Some(card) = hand
        .iter()
        .find(|c| kind_of(**c) == Some(CardKind::TrickGuohe))
    {
        for target in enemy_targets(eng, actor) {
            if pick_zone_card_token(st, actor, target, true, true, true, true).is_some() {
                return Some(play(actor, *card, vec![target]));
            }
        }
    }
    if let Some(card) = hand
        .iter()
        .find(|c| kind_of(**c) == Some(CardKind::TrickShunshou))
    {
        for target in enemy_targets(eng, actor) {
            if crate::query::card_target_within_distance(st, actor, *card, target, 1)
                && pick_zone_card_token(st, actor, target, true, true, false, false).is_some()
            {
                return Some(play(actor, *card, vec![target]));
            }
        }
    }
    if let Some(card) = hand
        .iter()
        .find(|c| kind_of(**c) == Some(CardKind::TrickHuogong))
    {
        for target in enemy_targets(eng, actor) {
            if !st.hand_cards(target).is_empty() {
                return Some(play(actor, *card, vec![target]));
            }
        }
    }

    // 6) AOE / party tricks (no target)
    if let Some(card) = hand.iter().find(|c| {
        kind_of(**c).is_some_and(|kind| {
            matches!(
                kind,
                CardKind::TrickNanman
                    | CardKind::TrickWanjian
                    | CardKind::TrickTaoyuan
                    | CardKind::TrickWugu
            )
        })
    }) {
        return Some(play(actor, *card, vec![]));
    }

    // 7) Juedou
    if let Some(card) = hand
        .iter()
        .find(|c| kind_of(**c) == Some(CardKind::TrickJuedou))
    {
        if let Some(target) = enemy_targets(eng, actor).into_iter().next() {
            return Some(play(actor, *card, vec![target]));
        }
    }

    // 8b) 铁索重铸
    if let Some(card) = hand
        .iter()
        .find(|c| kind_of(**c) == Some(CardKind::TrickTiesuo))
    {
        return Some(play(actor, *card, vec![]));
    }

    // 8) Delayed tricks (乐/兵粮 → enemy, 闪电 → self)
    if let Some(card) = hand.iter().find(|c| {
        kind_of(**c)
            .is_some_and(|kind| matches!(kind, CardKind::TrickLe | CardKind::TrickBingliang))
    }) {
        let kind = kind_of(*card);
        if let Some(target) = enemy_targets(eng, actor).into_iter().find(|target| {
            kind != Some(CardKind::TrickBingliang)
                || crate::query::card_target_within_distance(st, actor, *card, *target, 1)
        }) {
            return Some(play(actor, *card, vec![target]));
        }
    }
    if let Some(card) = hand
        .iter()
        .find(|c| kind_of(**c) == Some(CardKind::TrickShandian))
    {
        return Some(play(actor, *card, vec![]));
    }

    try_activate_skill(eng, actor)
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
