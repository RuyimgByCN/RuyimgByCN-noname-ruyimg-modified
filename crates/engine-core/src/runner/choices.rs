//! 对局驱动的待决选择应答与出牌动作选择。

use super::*;

pub(super) fn resolve_pending_choice(eng: &Engine) -> Command {
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
                                    choice.min_count as usize,
                                    choice.max_count as usize,
                                    sum,
                                    1,
                                )
                                .into_iter()
                                .next()
                            } else if group.len() >= choice.min_count as usize {
                                Some(
                                    group
                                        .into_iter()
                                        .take(choice.min_count as usize)
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
                        choice.min_count as usize,
                        choice.max_count as usize,
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

pub(super) fn try_play_action(eng: &Engine, actor: PlayerId) -> Option<Command> {
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
