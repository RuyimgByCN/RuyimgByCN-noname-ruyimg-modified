//! 效果原语分派（apply_effect）：通用效果原语。

use super::*;

pub(super) fn apply_effect(
    tx: &mut Transaction,
    req: EffectRequest,
) -> Result<Vec<DomainEvent>, EffectError> {
    match req {
        EffectRequest::PinDebateOpinions { player, skill } => {
            pin_debate_opinions(tx, player, skill)
        }

        EffectRequest::AlignDebateOpinions { player, skill } => {
            align_debate_opinions(tx, player, skill)
        }

        EffectRequest::GainShadowCards {
            player,
            count,
            skill,
        } => gain_shadow_cards(tx, player, count, skill),

        EffectRequest::ChangeGroup {
            player,
            group,
            skill,
        } => change_group(tx, player, group, skill),

        EffectRequest::DistributeShadowCards {
            player,
            count,
            skill,
        } => distribute_shadow_cards(tx, player, count, skill),

        EffectRequest::RegisterTurnEndDebate { player, skill } => {
            register_turn_end_debate(tx, player, skill)
        }

        EffectRequest::LoseGainedSkill { player, skill } => lose_gained_skill(tx, player, skill),

        EffectRequest::PlaceXumou {
            player,
            card,
            skill,
        } => place_xumou(tx, player, card, skill),

        EffectRequest::PlaceDeckTopXumou { player, skill } => {
            place_deck_top_xumou(tx, player, skill)
        }

        EffectRequest::DiscardOneOwnXumou { player, skill } => {
            discard_one_own_xumou(tx, player, skill)
        }

        EffectRequest::PlaceDeckTopExpansion {
            player,
            count,
            skill,
        } => place_deck_top_expansion(tx, player, count, skill),

        EffectRequest::CityConsume { player, skill } => city_consume(tx, player, skill),

        EffectRequest::QueueExtraTurn {
            player,
            check,
            skill,
        } => queue_extra_turn(tx, player, check, skill),

        EffectRequest::QueueExtraPhases {
            player,
            phases_mask,
            skill,
        } => queue_extra_phases(tx, player, phases_mask, skill),

        EffectRequest::SkipRemainingPhases {
            player,
            phases_mask,
            skill,
        } => skip_remaining_phases(tx, player, phases_mask, skill),

        EffectRequest::QueueOpeningExtraPlayPhase { player, skill } => {
            queue_opening_extra_play_phase(tx, player, skill)
        }

        EffectRequest::LoseHpDoubleCurrentDamageThenReplace {
            player,
            damage_id,
            amount,
            skill,
            replacement_skill_def,
            ready_to_replace,
        } => lose_hp_double_current_damage_then_replace(
            tx,
            player,
            damage_id,
            amount,
            skill,
            replacement_skill_def,
            ready_to_replace,
        ),

        EffectRequest::BeginTurn { player } => begin_turn(tx, player),

        EffectRequest::FinishTurnBegin { player } => finish_turn_begin(tx, player),

        EffectRequest::AdvanceOpeningExtraPlay { player } => advance_opening_extra_play(tx, player),

        EffectRequest::ConvertCurrentShaToFireLoseHpIfDamage { player, card, .. } => {
            crate::command::play::convert_saved_sha_to_fire(tx, player, card)?;
            Ok(Vec::new())
        }

        EffectRequest::IncreaseCurrentShaBaseDamage {
            player,
            card,
            amount,
            ..
        } => {
            crate::command::play::increase_saved_sha_base_damage(tx, player, card, amount)?;
            Ok(Vec::new())
        }

        EffectRequest::IncreaseCurrentCardBaseDamage {
            player,
            card,
            amount,
            ..
        } => {
            crate::command::play::increase_saved_card_base_damage(tx, player, card, amount)?;
            Ok(Vec::new())
        }

        EffectRequest::RefundCurrentShaUse {
            player,
            card,
            skill: _,
        } => {
            crate::command::play::refund_resolving_sha_use(tx, player, card)?;
            Ok(Vec::new())
        }

        EffectRequest::GuanxingTop {
            player,
            count,
            round: _,
            flipped: _,
            top: _,
            bottom: _,
            skill,
        } => guanxing_top(tx, player, count, skill),

        EffectRequest::GuanxingTopByFaluMarks {
            player,
            marks,
            skill,
        } => guanxing_top_by_falu_marks(tx, player, marks, skill),

        EffectRequest::GiveSelectedThenRecoverSelf {
            player,
            target,
            cards,
            amount,
            skill,
        } => give_selected_then_recover_self(tx, player, target, cards, amount, skill),

        EffectRequest::ContinueChooseCompareTargetHandCard {
            player,
            target,
            card,
            win_skill_def,
            lose_skill_def,
            skill,
        } => continue_choose_compare_target_hand_card(
            tx,
            player,
            target,
            card,
            win_skill_def,
            lose_skill_def,
            skill,
        ),

        EffectRequest::ContinueChooseCompareTargetHandCardSecondFailureSeatDamage {
            player,
            target,
            card,
            draw_count,
            failure_count,
            lose_hp,
            damage,
            skill,
        } => continue_choose_compare_target_hand_card_second_failure_seat_damage(
            tx,
            player,
            target,
            card,
            draw_count,
            failure_count,
            lose_hp,
            damage,
            skill,
        ),

        EffectRequest::RecordCompareFailureSeatDamage {
            player,
            target,
            failure_count,
            lose_hp,
            damage,
            skill,
        } => {
            let effects = record_compare_failure_seat_damage(
                tx,
                player,
                target,
                failure_count,
                lose_hp,
                damage,
                skill,
            )?;
            super::super::run_effects(tx, effects).map(|_| Vec::new())
        }

        EffectRequest::DealDamageEachInOrder {
            source,
            targets,
            amount,
            skill,
        } => {
            let effects = continue_damage_each_in_order(tx, source, targets, amount, skill)?;
            super::super::run_effects(tx, effects).map(|_| Vec::new())
        }

        EffectRequest::RandomizeCurrentHandCompareOpponent { player, skill } => {
            randomize_current_hand_compare_opponent(tx, player, skill)
        }

        EffectRequest::EmitHandCompareFinished {
            player,
            target,
            player_card,
            target_card,
            winner,
        } => Ok(vec![DomainEvent::HandCompareFinished {
            player,
            target,
            player_card,
            target_card,
            winner,
        }]),

        EffectRequest::GainHighestRankShaFromEventCards {
            player,
            cards,
            skill,
        } => gain_highest_rank_sha_from_event_cards(tx, player, &cards, skill),

        EffectRequest::GainHandCompareResultCard {
            player,
            initiator,
            target,
            winner,
            cards,
            skill,
        } => gain_hand_compare_result_card(tx, player, initiator, target, winner, &cards, skill),

        EffectRequest::AddCardColorLockSkill {
            player,
            source,
            color_mask,
            skill_def,
        } => add_card_color_lock_skill(tx, player, source, color_mask, skill_def),

        EffectRequest::UseVirtualShaFromCardColorLockSourceIfNoSha { player, skill } => {
            use_virtual_sha_from_card_color_lock_source_if_no_sha(tx, player, skill)
        }

        EffectRequest::UseVirtualShaDrawIfNoDamage {
            player,
            target,
            skill,
        } => crate::command::play::use_virtual_sha_on_draw_if_no_damage(tx, player, target, skill)
            .map(|_| Vec::new()),

        EffectRequest::DiscardSelectedRecordUniqueCategoryTarget {
            player,
            target,
            card,
            skill,
        } => discard_selected_record_unique_category_target(tx, player, target, card, skill),

        EffectRequest::FinishLinkIfTargetWounded {
            player,
            target,
            toggle_link,
            skill: _,
        } => finish_link_if_target_wounded(tx, player, target, toggle_link),

        EffectRequest::DrawThenGiveExactRecordPhaseTarget {
            player,
            target,
            count,
            skill,
            ready_to_give,
        } => draw_then_give_exact_record_phase_target(
            tx,
            player,
            target,
            count,
            skill,
            ready_to_give,
        ),

        EffectRequest::ClearPhaseGiftTarget { player, skill } => {
            clear_phase_gift_target(tx, player, skill)
        }

        EffectRequest::EmitShaResolved {
            source,
            card,
            targets,
            response_cards,
            card_flags,
        } => Ok(vec![DomainEvent::ShaResolved {
            source,
            card,
            targets,
            response_cards,
            card_flags,
        }]),

        EffectRequest::EmitCardResolved {
            source,
            card,
            kind,
            targets,
            damaged_targets,
            source_cards,
            all_sources_were_hand,
        } => Ok(vec![DomainEvent::CardResolved {
            source,
            card,
            kind,
            targets,
            damaged_targets,
            source_cards,
            all_sources_were_hand,
        }]),

        EffectRequest::SkipCurrentPhase { player } => skip_current_phase(tx, player),

        EffectRequest::RevealUniformHandDrawSelfChooseOthers { player, skill } => {
            reveal_uniform_hand_draw_self_choose_others(tx, player, skill)
        }

        EffectRequest::ModifyCurrentDamage {
            player,
            delta,
            skill: _,
        } => modify_current_damage(tx, player, delta),

        EffectRequest::GiveAllEquipmentChooseRecoverOrDamage {
            player,
            target,
            skill,
        } => give_all_equipment_choose_recover_or_damage(tx, player, target, skill),

        EffectRequest::FinishOpeningExtraPlayShaGain { player, skill } => {
            finish_opening_extra_play_sha_gain(tx, player, skill)
        }

        EffectRequest::RevealTargetHandGainShanArmorOrDrawTwo {
            player,
            target,
            skill,
        } => reveal_target_hand_gain_shan_armor_or_draw_two(tx, player, target, skill),

        EffectRequest::IncreaseCurrentDrawCount { player, count } => {
            increase_current_draw_count(tx, player, count)
        }

        EffectRequest::SkipJudgeAndDrawPhases { player, skill } => {
            skip_judge_and_draw_phases(tx, player, skill)
        }

        EffectRequest::SkipNextPlayPhase { player } => skip_next_play_phase(tx, player),

        EffectRequest::EnableEquipSlot {
            player,
            slot,
            source,
        } => enable_equip_slot(tx, player, slot, source),

        EffectRequest::DrawSelectedTargets {
            targets,
            count,
            skill: _,
        } => draw_selected_targets(tx, &targets, count),

        EffectRequest::DamageThenDrawSelectedTargets {
            source,
            targets,
            damage,
            draw,
            skill: _,
        } => damage_then_draw_selected_targets(tx, source, &targets, damage, draw),

        EffectRequest::FinishCurrentPhaseAbortResolution { player, stage } => {
            finish_current_phase_abort_resolution(tx, player, stage)
        }

        EffectRequest::RecoverSelectedTargets {
            source,
            targets,
            amount,
            skill: _,
        } => recover_selected_targets(tx, source, &targets, amount),

        EffectRequest::RecordSelectedPlayPhaseCategory { card, skill } => {
            record_selected_play_phase_category(tx, card, skill)
        }

        EffectRequest::RecoverIfMinCurrentHp { player, amount } => {
            recover_if_min_current_hp(tx, player, amount)
        }

        EffectRequest::DrawThenChooseGainOneFromDamageTargetAndNeighbors {
            player,
            target,
            skill,
            ready_to_choose,
        } => draw_then_choose_gain_one_from_damage_target_and_neighbors(
            tx,
            player,
            target,
            skill,
            ready_to_choose,
        ),

        EffectRequest::ContinueChooseGainOneFromDamageTargetAndNeighbors {
            player,
            targets,
            index,
            selected,
            skill,
        } => continue_choose_gain_one_from_damage_target_and_neighbors(
            tx, player, targets, index, selected, skill,
        ),

        EffectRequest::ContinueIncreaseBoundDamage {
            player,
            damage_id,
            amount,
            ..
        } => {
            if !bound_damage_source_matches(tx, damage_id, player) {
                return Err(EffectError::Illegal(
                    "bound damage increase context changed".into(),
                ));
            }
            increase_bound_damage(tx, damage_id, amount)?;
            Ok(Vec::new())
        }

        EffectRequest::AddCardUsePolicySkill {
            player,
            card,
            policy_mask,
            skill_def,
        } => add_card_use_policy_skill(tx, player, card, policy_mask, skill_def),

        EffectRequest::DiscardTargetCards {
            actor,
            target,
            cards,
            scope,
        } => discard_target_cards(tx, actor, target, cards, scope),

        EffectRequest::RemoveSkillByDef { player, skill_def } => {
            remove_skill_by_def(tx, player, skill_def)
        }

        EffectRequest::RevealTopGainNonBasicOrSwapBasic { player } => {
            reveal_top_gain_non_basic_or_swap_basic(tx, player)
        }

        EffectRequest::PreventDamage { damage_id } => {
            if damage_id == 0 {
                return Err(EffectError::ContentFault(
                    "prevent_current_damage used outside damage_before".into(),
                ));
            }
            if !tx
                .state()
                .resolution
                .prevented_damage_ids
                .contains(&damage_id)
            {
                tx.state_mut()
                    .resolution
                    .prevented_damage_ids
                    .push(damage_id);
            }
            Ok(Vec::new())
        }

        EffectRequest::MoveCards { cards, to, from } => move_cards(tx, cards, to, from),

        EffectRequest::MoveCardsWithReason {
            cards,
            to,
            from,
            reason,
        } => move_cards_with_reason(tx, cards, to, from, reason),

        EffectRequest::GiveCards { cards, to, from } => give_cards(tx, cards, to, from),

        EffectRequest::UseCard {
            player,
            card,
            targets,
        } => use_card(tx, player, card, targets),

        EffectRequest::UseFieldCard {
            player,
            card,
            targets,
        } => use_field_card(tx, player, card, targets),

        EffectRequest::DealDamage {
            source,
            target,
            amount,
            is_lose_hp,
            nature,
        } => deal_damage(
            tx,
            source,
            target,
            amount,
            is_lose_hp,
            nature,
            false,
            CardId::NONE,
            0,
            0,
        ),

        EffectRequest::DealDamageBanActiveIfDying {
            source,
            target,
            amount,
            skill,
        } => {
            let Some(EffectRequest::CommitDamage {
                damage_id,
                source,
                target,
                amount,
                base_amount,
                ..
            }) = prepare_damage(
                tx.state(),
                source,
                target,
                amount,
                false,
                rule_api::damage_nature::NORMAL,
                false,
                CardId::NONE,
            )?
            else {
                return Ok(Vec::new());
            };
            let (mut events, followups) = commit_damage_ban_active_if_dying(
                tx,
                damage_id,
                source,
                target,
                amount,
                base_amount,
                rule_api::damage_nature::NORMAL,
                skill,
            )?;
            for followup in followups {
                events.extend(apply_effect(tx, followup)?);
            }
            Ok(events)
        }

        EffectRequest::SpjungongDiscardOrLoseThenSha {
            player,
            target,
            skill,
        } => spjungong_discard_or_lose_then_sha(tx, player, target, skill),

        EffectRequest::BanActiveUntilTurnEnd { player, skill } => {
            ban_active_until_turn_end(tx, player, skill)
        }

        EffectRequest::SpwanweiActive { player, target } => spwanwei_active(tx, player, target),

        EffectRequest::SpwanweiDying { player, target } => spwanwei_dying(tx, player, target),

        EffectRequest::LoseHpThenDamageOtherLoseMaxHpIfDying {
            player,
            target,
            amount,
            skill,
        } => {
            let Some(EffectRequest::CommitDamage { amount, .. }) = prepare_damage(
                tx.state(),
                PlayerId::NONE,
                player,
                amount,
                true,
                rule_api::damage_nature::NORMAL,
                false,
                CardId::NONE,
            )?
            else {
                return Ok(Vec::new());
            };
            let (events, followups) = commit_lose_hp_then_damage_other_lose_max_hp_if_dying(
                tx, player, target, amount, skill,
            )?;
            debug_assert!(followups.is_empty());
            Ok(events)
        }

        EffectRequest::DealShaDamage {
            source,
            target,
            card,
            amount,
            nature,
            card_flags,
        } => deal_damage(
            tx,
            source,
            target,
            amount,
            false,
            nature,
            true,
            card,
            CardKind::Sha.tag(),
            card_flags,
        ),

        EffectRequest::DealCardDamage {
            source,
            target,
            amount,
            is_lose_hp,
            nature,
            card,
            card_kind,
            card_flags,
        } => deal_damage(
            tx, source, target, amount, is_lose_hp, nature, false, card, card_kind, card_flags,
        ),

        EffectRequest::RecoverHp {
            source,
            player,
            amount,
        } => recover_hp(tx, source, player, amount),

        EffectRequest::PreventRecovery { recovery_id } => {
            if recovery_id == 0 || tx.state().resolution.current_recovery_id != recovery_id {
                return Err(EffectError::ContentFault(
                    "prevent recovery requested outside recover_before".into(),
                ));
            }
            if !tx
                .state()
                .resolution
                .prevented_recovery_ids
                .contains(&recovery_id)
            {
                tx.state_mut()
                    .resolution
                    .prevented_recovery_ids
                    .push(recovery_id);
            }
            Ok(Vec::new())
        }

        EffectRequest::ChangeMaxHp { player, delta } => change_max_hp(tx, player, delta),

        EffectRequest::AddSkill { player, skill_def } => add_skill(tx, player, skill_def),

        EffectRequest::AddBoundSkill {
            player,
            skill_def,
            bound_target,
        } => add_bound_skill(tx, player, skill_def, bound_target),

        EffectRequest::RemoveSkill { skill } => remove_skill(tx, skill),

        EffectRequest::RefreshSkill { skill } => refresh_skill(tx, skill),

        EffectRequest::TagCard { cards, tag } => tag_cards(tx, cards, tag),

        EffectRequest::UntagCard { cards, tag } => untag_cards(tx, cards, tag),

        EffectRequest::ClanshenjunReveal {
            player,
            event_card,
            skill,
        } => clanshenjun_reveal(tx, player, event_card, skill),

        EffectRequest::ClanshenjunUse { player, skill } => clanshenjun_use(tx, player, skill),

        EffectRequest::StorageSet { skill, key, value } => storage_set(tx, skill, key, value),

        EffectRequest::DrawAllEventTargets { players, count } => {
            draw_all_event_targets(tx, players, count)
        }

        EffectRequest::RoundEnded { round, current } => {
            Ok(vec![DomainEvent::RoundEnded { round, current }])
        }

        EffectRequest::RoundStarted { round, current } => {
            Ok(vec![DomainEvent::RoundStarted { round, current }])
        }

        EffectRequest::ClanyuzhiReveal { player, skill } => clanyuzhi_reveal(tx, player, skill),

        EffectRequest::ClanyuzhiRoundEnd { player, skill } => {
            clanyuzhi_round_end(tx, player, skill)
        }

        EffectRequest::QieyiPhaseStart { player, skill } => qieyi_phase_start(tx, player, skill),

        EffectRequest::QieyiUseAfter {
            player,
            card,
            skill,
        } => qieyi_use_after(tx, player, card, skill),

        EffectRequest::JianzhiPhaseEnd { player, skill } => jianzhi_phase_end(tx, player, skill),

        EffectRequest::AddMark {
            player,
            mark,
            delta,
        } => add_mark(tx, player, mark.raw(), delta),

        EffectRequest::RemoveMark {
            player,
            mark,
            amount,
        } => remove_mark(tx, player, mark.raw(), amount),

        EffectRequest::RevealCards { cards, to } => {
            Ok(vec![DomainEvent::CardsRevealed { cards, to }])
        }

        EffectRequest::RevealHandDiscardKind { player, kind } => {
            reveal_hand_discard_kind(tx, player, kind)
        }

        EffectRequest::RequestChoice {
            observer,
            min_count,
            max_count,
            allow_cancel,
            option_tokens,
        } => request_choice(
            tx,
            observer,
            min_count,
            max_count,
            allow_cancel,
            option_tokens,
            ChoiceKind::Custom,
        ),

        EffectRequest::RequestDyingSave { player, source } => {
            request_dying_save(tx, player, source)
        }

        EffectRequest::ContinueDyingSave {
            player,
            source,
            responders,
        } => request_dying_save_from_queue(tx, player, source, responders),

        EffectRequest::EnterDying { player } => enter_dying(tx, player),

        EffectRequest::KillPlayer { player, killer } => kill_player(tx, player, killer),

        EffectRequest::SetTurnState { current, phase } => set_turn(tx, current, phase),

        EffectRequest::AdvanceTurn => {
            crate::command::turn::finish_turn(tx)?;
            Ok(Vec::new())
        }

        EffectRequest::EmitRuleEvent { code, payload } => {
            Ok(vec![DomainEvent::RuleNote { code, payload }])
        }

        EffectRequest::EmitDebateResolved {
            initiator,
            red,
            black,
            result,
        } => Ok(vec![DomainEvent::DebateResolved {
            initiator,
            red,
            black,
            result,
        }]),

        EffectRequest::DiscardFromHand { player, count } => discard_from_hand(tx, player, count),

        EffectRequest::DiscardFromHandEquip { player, count } => {
            discard_from_hand_equip(tx, player, count)
        }

        EffectRequest::AdjustHandTo { player, count } => adjust_hand_to(tx, player, count),

        EffectRequest::DiscardCards { player, cards } => discard_cards(tx, player, cards),

        EffectRequest::DiscardAllHandAndEquip { player } => discard_all_hand_and_equip(tx, player),

        EffectRequest::RemoveAllSkills { player } => remove_all_skills(tx, player),

        EffectRequest::JudgeGainEquipmentByColor { player } => {
            judge_gain_equipment_by_color(tx, player)
        }

        EffectRequest::JudgeCancelCurrentShaTargetOnRed {
            player,
            card,
            target,
        } => judge_cancel_current_sha_target_on_red(tx, player, card, target),

        EffectRequest::MakeCurrentShaTargetUnrespondable {
            source,
            card,
            target,
        } => make_current_sha_target_unrespondable(tx, source, card, target),

        EffectRequest::JudgeMakeCurrentShaTargetUnrespondableOnRed {
            player,
            card,
            target,
            retieji,
            skill,
            ready_to_choose,
            judgment_suit,
        } => judge_make_current_sha_target_unrespondable_on_red(
            tx,
            player,
            card,
            target,
            retieji,
            skill,
            ready_to_choose,
            judgment_suit,
        ),

        EffectRequest::TargetDiscardThenDuelIfNonShaAndHpGteOwner {
            player,
            target,
            skill,
        } => target_discard_then_duel_if_non_sha_and_hp_gte_owner(tx, player, target, skill),

        _ => unreachable!("effect routed to a different primitive group"),
    }
}
