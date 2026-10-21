//! Primitive dispatch for `ChooseCurrentShaTargetDiscardRedirectToOwner` and related requests.

use super::*;

pub(super) fn apply_effect(
    tx: &mut Transaction,
    req: EffectRequest,
) -> Result<Vec<DomainEvent>, EffectError> {
    match req {
        EffectRequest::ChooseCurrentShaTargetDiscardRedirectToOwner {
            owner,
            source,
            card,
            target,
            skill,
            ready_to_redirect,
        } => choose_current_sha_target_discard_redirect_to_owner(
            tx,
            owner,
            source,
            card,
            target,
            skill,
            ready_to_redirect,
        ),

        EffectRequest::ChooseDiscardRedirectCurrentSha {
            player,
            source,
            card,
            target,
            skill,
        } => choose_discard_redirect_current_sha(tx, player, source, card, target, skill),

        EffectRequest::ChooseTargetGainBottomCard {
            player,
            card,
            target,
            skill,
        } => choose_target_gain_bottom_card(tx, player, card, target, skill),

        EffectRequest::JudgeBlackGainRepeat {
            player,
            card,
            skill,
            ready_to_choose,
        } => judge_black_gain_repeat(tx, player, card, skill, ready_to_choose, false),

        EffectRequest::JudgeBlackGainRepeatProtectHandLimit {
            player,
            card,
            skill,
            ready_to_choose,
        } => judge_black_gain_repeat(tx, player, card, skill, ready_to_choose, true),

        EffectRequest::JudgeSameColorGainRepeat {
            player,
            card,
            skill,
            ready_to_choose,
            color,
        } => judge_same_color_gain_repeat(tx, player, card, skill, ready_to_choose, color),

        EffectRequest::ChooseCurrentTrickTargetsToNullify {
            player,
            source,
            card,
            skill,
        } => choose_current_trick_targets_to_nullify(tx, player, source, card, skill),

        EffectRequest::UseVirtualJuedouSecondOnFirst { targets, card } => {
            use_virtual_juedou_second_on_first(tx, targets, card)
        }

        EffectRequest::JudgeSetSkillMarkToRank { player, skill } => {
            judge_set_skill_mark_to_rank(tx, player, skill)
        }

        EffectRequest::ChooseDiscardOneFromEachLinked { player, skill } => {
            choose_discard_one_from_each_linked(tx, player, skill)
        }

        EffectRequest::DisableSkillByDefUntilTurnEnd { player, skill_def } => {
            disable_skill_by_def_until_turn_end(tx, player, skill_def)
        }

        EffectRequest::ChooseDiscardTargetCardResolveCurrentSha {
            source,
            card,
            target,
        } => choose_discard_target_card_resolve_current_sha(tx, source, card, target, None),

        EffectRequest::ChooseDiscardTargetCardResolveCurrentShaNonbasicBonus {
            source,
            card,
            target,
            bonus_skill_def,
        } => choose_discard_target_card_resolve_current_sha(
            tx,
            source,
            card,
            target,
            Some(bonus_skill_def),
        ),

        EffectRequest::ResolveDcqiaomeng {
            player,
            card,
            targets,
            target,
            selected,
            helper_skill_def,
            stage,
            skill,
        } => resolve_dcqiaomeng(
            tx,
            player,
            card,
            targets,
            target,
            selected,
            helper_skill_def,
            stage,
            skill,
        ),

        EffectRequest::ResolveNewReluoyi {
            player,
            buff_skill_def,
            stage,
            cards,
            accepted,
            skill,
        } => resolve_new_reluoyi(tx, player, buff_skill_def, stage, cards, accepted, skill),

        EffectRequest::ResolveRemoshi {
            player,
            target,
            card,
            helper_skill_def,
            action,
            skill,
        } => resolve_remoshi(tx, player, target, card, helper_skill_def, action, skill),

        EffectRequest::ResolveXinyaoming {
            player,
            target,
            action,
            skill,
        } => resolve_xinyaoming(tx, player, target, action, skill),

        EffectRequest::ResolveRehuaiyi {
            player,
            stage,
            color,
            discarded_count,
            targets,
            index,
            gained_count,
            skill,
        } => resolve_rehuaiyi(
            tx,
            player,
            stage,
            color,
            discarded_count,
            targets,
            index,
            gained_count,
            skill,
        ),

        EffectRequest::ChooseDrawOrRecover {
            player,
            draw_count,
            recover_count,
            allow_cancel,
        } => choose_draw_or_recover(tx, player, draw_count, recover_count, allow_cancel),

        EffectRequest::ChooseUseVirtualSha {
            player,
            ignore_distance,
        } => choose_use_virtual_sha(
            tx,
            player,
            ignore_distance,
            false,
            false,
            engine_api::SkillId::NONE,
            0,
            0,
        ),

        EffectRequest::ChooseUseVirtualShaWithBonus {
            player,
            ignore_distance,
            damage_bonus,
        } => choose_use_virtual_sha(
            tx,
            player,
            ignore_distance,
            false,
            false,
            engine_api::SkillId::NONE,
            0,
            damage_bonus,
        ),

        EffectRequest::ChooseUseVirtualShaOrDraw { player, skill } => {
            choose_use_virtual_sha(tx, player, false, true, true, skill, 0, 0)
        }

        EffectRequest::UseVirtualCardOn {
            player,
            kind,
            target,
        } => use_virtual_card_on(tx, player, kind, target),

        EffectRequest::UseVirtualCardOnSelectedTargets {
            player,
            kind,
            targets,
        } => use_virtual_card_on_selected_targets(tx, player, kind, targets),

        EffectRequest::ChooseTargetReclaimEquipmentDrawIfRangeReduced {
            player,
            target,
            skill,
        } => choose_target_reclaim_equipment_draw_if_range_reduced(tx, player, target, skill),

        EffectRequest::CommitTargetReclaimEquipmentDrawIfRangeReduced {
            player,
            target,
            card,
            before,
            skill,
        } => commit_target_reclaim_equipment_draw_if_range_reduced(
            tx, player, target, card, before, skill,
        ),

        EffectRequest::FinishTargetReclaimEquipmentDrawIfRangeReduced {
            player,
            target,
            before,
            skill: _,
        } => finish_target_reclaim_equipment_draw_if_range_reduced(tx, player, target, before),

        EffectRequest::ChooseUseVirtualCard {
            player,
            kind,
            skill,
        } => choose_use_virtual_card(tx, player, kind, skill),

        EffectRequest::ChooseUseVirtualNanman { player, skill } => choose_use_virtual_card_impl(
            tx,
            player,
            CardKind::TrickNanman.tag(),
            skill,
            VirtualCardTracking::None,
            false,
        ),

        EffectRequest::ResolveQiluan { player, skill } => resolve_qiluan(tx, player, skill),

        EffectRequest::DiscardAllOwnJudge { player, skill } => {
            let _ = skill;
            discard_all_judge(tx, player)
        }

        EffectRequest::StoreEventCardInSkillPile {
            player,
            card,
            skill,
        } => store_event_card_in_skill_pile(tx, player, card, skill),

        EffectRequest::FinishPhaseEndVirtualCard => {
            tx.state_mut().turn.clear_phase_histories();
            tx.state_mut()
                .turn
                .set_flag(turn_flag::PHASE_HISTORY_SUPPRESSED, 0);
            Ok(Vec::new())
        }

        EffectRequest::EmitDuelResolved {
            card,
            initiator,
            target,
            winner,
            loser,
        } => {
            let card_flags = tx
                .state_mut()
                .resolution
                .take_duel_damage(card)
                .then_some(rule_api::damage_card_flag::CARD_USE_DEALT_DAMAGE)
                .unwrap_or(0);
            Ok(vec![DomainEvent::DuelResolved {
                card,
                card_flags,
                initiator,
                target,
                winner,
                loser,
            }])
        }

        EffectRequest::EmitJudgmentResolved {
            player,
            result_card,
            suit,
            rank,
            cause_card,
        } => Ok(vec![DomainEvent::JudgmentResolved {
            player,
            result_card,
            suit,
            rank,
            cause_card,
        }]),

        EffectRequest::RevealDiscardSuitGainOrUse {
            player,
            mark,
            base_count,
            max_mark,
            revealed,
            completed,
            step,
            data,
        } => crate::command::reveal_options::apply(
            tx, player, mark, base_count, max_mark, revealed, completed, step, data,
        ),

        EffectRequest::ChooseDiscardOrFireDamageSkipPhases {
            player,
            max_discard,
            damage,
            selected,
            step,
            target,
        } => crate::command::discard_damage_options::apply(
            tx,
            player,
            max_discard,
            damage,
            selected,
            step,
            target,
        ),

        EffectRequest::TakeFromHand { from, to } => take_from_hand(tx, from, to),

        EffectRequest::CardResponded {
            player,
            respond_to,
            card,
            response_kind,
        } => {
            if tx.state().players.get(player).is_none() {
                return Err(EffectError::Illegal("bad responding player".into()));
            }
            if card.is_some() && tx.state().cards.get(card).is_none() {
                return Err(EffectError::Illegal("bad response card".into()));
            }
            if respond_to.is_some() && tx.state().players.get(respond_to).is_none() {
                return Err(EffectError::Illegal("bad response origin player".into()));
            }
            Ok(vec![DomainEvent::CardResponded {
                player,
                respond_to,
                card,
                response_kind,
            }])
        }

        EffectRequest::ShaDodged {
            source,
            target,
            card,
        } => {
            if ![source, target]
                .into_iter()
                .all(|player| tx.state().players.get(player).is_some())
            {
                return Err(EffectError::Illegal(
                    "sha-dodged participants must exist".into(),
                ));
            }
            if card.is_some()
                && (!tx.state().cards.get(card).is_some()
                    || !tx
                        .state()
                        .resolution
                        .sha_target_resolving(source, card, target))
            {
                return Err(EffectError::Illegal(
                    "sha-dodged card is not the current logical Sha carrier".into(),
                ));
            }
            Ok(vec![DomainEvent::ShaDodged {
                source,
                target,
                card,
            }])
        }

        EffectRequest::RemoveCardsByKind { kind } => remove_cards_by_kind(tx, kind),
        _ => unreachable!("effect routed to a different primitive group"),
    }
}
