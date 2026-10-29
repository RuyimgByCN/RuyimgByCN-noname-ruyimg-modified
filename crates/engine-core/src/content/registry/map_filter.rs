//! Content registry operations for `map_filter` and related helpers.

use super::*;

pub(crate) fn map_filter(s: &str) -> Option<TriggerFilter> {
    // Skill-provenance family (Phase 3): `event_view_as_skill_<DEF>` — event card originated from
    // view-as skill with the given definition id (analogue of upstream `event.skill == "X"`).
    // Name-matched variant `event_view_as_skill_name_<NAME>` covers dynamic _backup helpers without
    // a static def id; checked first so it isn't shadowed by the DEF branch.
    if let Some(name) = s.strip_prefix("event_view_as_skill_name_") {
        if !name.is_empty()
            && name
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        {
            return Some(TriggerFilter::EventViewAsSkillName(name.to_string()));
        }
    }
    if let Some(def) = s.strip_prefix("event_view_as_skill_") {
        if let Ok(skill_def) = def.parse::<u32>() {
            if skill_def > 0 {
                return Some(TriggerFilter::EventViewAsSkill(skill_def));
            }
        }
    }
    // Card-name-scoped family (Phase 3): `owner_source_card_<selector>`,
    // `owner_is_target_card_<selector>`, `owner_is_target_card_damage_<selector>` where
    // `<selector>` is a card-selector name (sha, nanman, juedou, trick, ...) mapped by
    // map_card_selector. Unlocks card_name_scoped placeholders.
    // NOTE: do NOT `return None` here when the selector is unknown — fall through so that
    // pre-existing filters sharing the prefix (e.g. owner_is_target_card_in_processing) still
    // resolve via the later branches.
    if let Some(selector) = s.strip_prefix("owner_source_card_") {
        if let Some(card) = map_card_selector(selector) {
            return Some(TriggerFilter::OwnerSourceCardName(card));
        }
    }
    if let Some(selector) = s.strip_prefix("owner_is_target_card_damage_") {
        if let Some(card) = map_card_selector(selector) {
            return Some(TriggerFilter::OwnerIsTargetCardDamage(card));
        }
    }
    if let Some(selector) = s.strip_prefix("owner_is_target_card_") {
        if let Some(card) = map_card_selector(selector) {
            return Some(TriggerFilter::OwnerIsTargetCardName(card));
        }
    }
    if let Some(skill_def) = s.strip_prefix("owner_is_target_sha_skill_pile_nonempty_") {
        return skill_def
            .parse::<u32>()
            .ok()
            .filter(|skill_def| *skill_def > 0)
            .map(TriggerFilter::OwnerIsTargetShaSkillPileNonempty);
    }
    if let Some(skill_def) = s.strip_prefix("owner_alive_other_play_discard_skill_pile_nonempty_") {
        return skill_def
            .parse::<u32>()
            .ok()
            .filter(|skill_def| *skill_def > 0)
            .map(TriggerFilter::OwnerAliveOtherPlayDiscardSkillPileNonempty);
    }
    if let Some(skill_def) =
        s.strip_prefix("owner_source_own_view_as_card_targeted_target_skill_pile_")
    {
        return skill_def
            .parse::<u32>()
            .ok()
            .filter(|skill_def| *skill_def > 0)
            .map(TriggerFilter::OwnerSourceOwnViewAsCardTargetedTargetSkillPile);
    }
    if let Some(mark) = s.strip_prefix("owner_alive_card_targeted_marked_peer_") {
        return mark
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerAliveCardTargetedMarkedPeer);
    }
    if let Some(mark) = s.strip_prefix("owner_source_target_mark_positive_") {
        return mark
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerSourceTargetMarkPositive);
    }
    if let Some(mark) = s.strip_prefix("owner_source_play_skill_mark_mode_") {
        return mark
            .parse::<u32>()
            .ok()
            .filter(|mark| *mark > 0)
            .map(TriggerFilter::OwnerSourcePlaySkillMarkMode);
    }
    if let Some(skill_def) = s.strip_prefix("owner_alive_targeted_by_other_hidden_skill_pile_") {
        return skill_def
            .parse::<u32>()
            .ok()
            .filter(|skill_def| *skill_def > 0)
            .map(TriggerFilter::OwnerAliveTargetedByOtherHiddenSkillPile);
    }
    if let Some(skill_def) = s.strip_prefix("owner_is_target_hand_nonempty_hidden_skill_pile_") {
        return skill_def
            .parse::<u32>()
            .ok()
            .filter(|skill_def| *skill_def > 0)
            .map(TriggerFilter::OwnerIsEventPlayerHandNonemptyHiddenSkillPile);
    }
    if let Some(skill_def) = s.strip_prefix("owner_phase_begin_skill_pile_gte_three_") {
        return skill_def
            .parse::<u32>()
            .ok()
            .filter(|skill_def| *skill_def > 0)
            .map(TriggerFilter::OwnerPhaseBeginSkillPileGteThree);
    }
    if let Some(skill_def) = s.strip_prefix("owner_lost_matching_revealed_skill_pile_suit_") {
        return skill_def
            .parse::<u32>()
            .ok()
            .filter(|skill_def| *skill_def > 0)
            .map(TriggerFilter::OwnerLostMatchingRevealedSkillPileSuit);
    }
    if s == "owner_source_play_card_count_eq_hp" {
        return Some(TriggerFilter::OwnerSourcePlayCardCountEqualsHp);
    }
    if let Some(count) = s.strip_prefix("owner_source_play_card_count_eq_") {
        return count
            .parse::<u32>()
            .ok()
            .filter(|count| *count > 0)
            .map(TriggerFilter::OwnerSourcePlayCardCountEquals);
    }
    if let Some(mark) = s.strip_prefix("owner_is_target_source_alive_mark_positive_") {
        return mark
            .parse::<u32>()
            .ok()
            .filter(|mark| *mark > 0)
            .map(TriggerFilter::OwnerIsEventPlayerSourceAliveMarkPositive);
    }
    if let Some(skill_def) = s.strip_prefix("owner_is_target_any_field_card_no_skill_pile_") {
        return skill_def
            .parse::<u32>()
            .ok()
            .filter(|skill_def| *skill_def > 0)
            .map(TriggerFilter::OwnerIsEventPlayerAnyFieldCardNoSkillPile);
    }
    if let Some(skill_def) = s.strip_prefix("owner_alive_source_sha_hand_equip_skill_pile_") {
        return skill_def
            .parse::<u32>()
            .ok()
            .filter(|skill_def| *skill_def > 0)
            .map(TriggerFilter::OwnerAliveSourceShaHandEquipSkillPile);
    }
    if let Some(skill_def) = s.strip_prefix("owner_alive_dying_skill_pile_") {
        return skill_def
            .parse::<u32>()
            .ok()
            .filter(|skill_def| *skill_def > 0)
            .map(TriggerFilter::OwnerAliveDyingSkillPile);
    }
    if let Some(threshold) = s
        .strip_prefix("owner_is_target_active_used_at_least_")
        .and_then(|value| value.strip_suffix("_this_play_phase"))
    {
        return threshold
            .parse::<u32>()
            .ok()
            .filter(|threshold| *threshold > 0)
            .map(TriggerFilter::OwnerIsEventPlayerActiveUsedAtLeastThisPlayPhase);
    }
    if let Some(threshold) = s.strip_prefix("owner_is_target_hp_below_") {
        return threshold
            .parse::<u32>()
            .ok()
            .filter(|threshold| *threshold > 0)
            .map(TriggerFilter::OwnerIsEventPlayerHpBelow);
    }
    if let Some(mark) = s.strip_prefix("owner_is_target_mark_positive_") {
        return mark
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerIsEventPlayerMarkPositive);
    }
    if let Some(mark) = s.strip_prefix("owner_mark_positive_") {
        return mark
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerMarkPositive);
    }
    if let Some(skill_def) = s.strip_prefix("owner_skill_added_self_") {
        return skill_def
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerSkillAddedSelf);
    }
    if s == "owner_source_first_card_suit_this_turn" {
        return Some(TriggerFilter::OwnerSourceFirstCardSuitThisTurn);
    }
    if s == "owner_lost_hand_now_empty" {
        return Some(TriggerFilter::OwnerLostHandNowEmpty);
    }
    if s == "owner_lost_spade_two_to_nine" {
        return Some(TriggerFilter::OwnerLostSpadeTwoToNine);
    }
    if let Some(mark) = s.strip_prefix("owner_source_ordinary_trick_mark_positive_") {
        return mark
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerSourceOrdinaryTrickMarkPositive);
    }
    if let Some(mark) = s.strip_prefix("owner_source_sha_or_black_trick_mark_positive_") {
        return mark
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerSourceShaOrBlackTrickMarkPositive);
    }
    if let Some(value) = s
        .strip_prefix("owner_source_damage_target_mark_positive_")
        .and_then(|v| v.strip_suffix("_once"))
    {
        return value
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerSourceDamageTargetMarkPositiveOnce);
    }
    if let Some(mark) = s.strip_prefix("owner_alive_marked_alive_above_two_") {
        return mark
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerAliveMarkedAliveAboveTwo);
    }
    if let Some(mark) = s.strip_prefix("owner_alive_event_target_mark_positive_") {
        if let Ok(value) = mark.parse::<u32>() {
            return Some(TriggerFilter::OwnerAliveEventTargetMarkPositive(value));
        }
        // non-numeric remainder (e.g. "..._no_damage") falls through to the
        // suffix-aware arm registered below.
    }
    if let Some(value) = s.strip_prefix("owner_source_hand_gaintag_absent_") {
        return value
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerSourceHandGaintagAbsent);
    }
    if s == "owner_alive_play_end_used_categories_positive" {
        return Some(TriggerFilter::OwnerAlivePlayEndUsedCategoriesPositive);
    }
    if let Some(value) = s
        .strip_prefix("owner_alive_event_target_mark_positive_")
        .and_then(|value| value.strip_suffix("_no_damage"))
    {
        eprintln!("DBG B no_damage matched");
        return value
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerAliveEventTargetMarkPositiveNoDamage);
    }
    if let Some(mark) = s.strip_prefix("owner_alive_someone_mark_positive_") {
        return mark
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerAliveSomeoneMarkPositive);
    }
    if let Some(pile_def) = s.strip_prefix("owner_alive_discard_matches_own_pile_card_") {
        return pile_def
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerAliveDiscardMatchesOwnPileCard);
    }
    if s == "cards_moved_into_expansion" {
        return Some(TriggerFilter::CardsMovedIntoExpansion);
    }
    if s == "cards_moved_out_of_expansion" {
        return Some(TriggerFilter::CardsMovedOutOfExpansion);
    }
    if let Some(mark) = s.strip_prefix("owner_alive_damage_before_event_target_mark_positive_") {
        return mark
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerAliveDamageBeforeEventTargetMarkPositive);
    }
    if let Some(mark) = s.strip_prefix("owner_alive_phase_begin_event_target_mark_positive_") {
        return mark
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerAlivePhaseBeginEventTargetMarkPositive);
    }
    if let Some(skill_def) = s.strip_prefix("owner_alive_sha_resolved_city_") {
        return skill_def
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerAliveShaResolvedCity);
    }
    if let Some(group) = s
        .strip_prefix("owner_alive_other_shan_outside_turn_group_")
        .and_then(|value| value.strip_suffix("_once_per_turn"))
    {
        return valid_group_filter_tag(group).then(|| {
            TriggerFilter::OwnerAliveOtherShanOutsideTurnGroupOncePerTurn(group.to_owned())
        });
    }
    if let Some(group) = s.strip_prefix("owner_alive_event_player_group_") {
        return valid_group_filter_tag(group)
            .then(|| TriggerFilter::OwnerAliveEventPlayerGroup(group.to_owned()));
    }
    // clan 中流 family: owner is the event player and no OTHER living member of <clan>
    // lost hand cards this turn. <clan> is an arbitrary (CJK) clan name, so the suffix is
    // stripped rather than validated as a group tag.
    if let Some(rest) = s.strip_prefix("owner_is_event_player_no_") {
        if let Some(clan) = rest.strip_suffix("_lost_hand_this_turn") {
            if !clan.is_empty() {
                return Some(TriggerFilter::OwnerIsEventPlayerNoClanLostHandThisTurn(
                    clan.to_owned(),
                ));
            }
        }
    }
    // 保族 (clanbaozu) family: event player is the owner or a member of <clan>.
    // <clan> is an arbitrary (CJK) clan name.
    if let Some(clan) = s.strip_prefix("owner_alive_event_player_self_or_clan_") {
        if !clan.is_empty() {
            return Some(TriggerFilter::OwnerAliveEventPlayerSelfOrClan(
                clan.to_owned(),
            ));
        }
    }
    // 神君 (clanshenjun) family: same-kind hand reveal gate and tagged-card
    // phase-end use gate.
    if s == "owner_alive_hand_same_kind_event_card" {
        return Some(TriggerFilter::OwnerAliveHandHasSameKindAsEventCard);
    }
    if s == "owner_alive_has_skill_tagged_cards" {
        return Some(TriggerFilter::OwnerAliveHasSkillTaggedCards);
    }
    if s == "owner_alive_tagged_card_lost_this_turn" {
        return Some(TriggerFilter::OwnerAliveTaggedCardLostThisTurn);
    }
    if let Some(group) = s.strip_prefix("owner_source_target_not_group_") {
        return valid_group_filter_tag(group)
            .then(|| TriggerFilter::OwnerSourceTargetNotGroup(group.to_owned()));
    }
    if let Some(group) = s.strip_prefix("owner_alive_other_damage_source_group_") {
        return valid_group_filter_tag(group)
            .then(|| TriggerFilter::OwnerAliveOtherDamageSourceGroup(group.to_owned()));
    }
    if let Some(group) = s.strip_prefix("owner_dying_other_tao_responder_group_") {
        return valid_group_filter_tag(group)
            .then(|| TriggerFilter::OwnerDyingOtherTaoResponderGroup(group.to_owned()));
    }
    if let Some(group) = s.strip_prefix("owner_alive_other_judgment_black_event_player_group_") {
        return valid_group_filter_tag(group).then(|| {
            TriggerFilter::OwnerAliveOtherJudgmentBlackEventPlayerGroup(group.to_owned())
        });
    }
    if let Some(group) = s.strip_prefix("owner_wounded_event_player_group_") {
        return valid_group_filter_tag(group)
            .then(|| TriggerFilter::OwnerWoundedEventPlayerGroup(group.to_owned()));
    }
    if let Some(mark) = s.strip_prefix("owner_gains_outside_turn_mark_below_alive_once_") {
        return mark
            .parse::<u32>()
            .ok()
            .map(TriggerFilter::OwnerGainsOutsideTurnMarkBelowAliveOncePerTurn);
    }
    if let Some(skill_def) = s
        .strip_prefix("owner_source_jiu_boosted_sha_damage_skill_")
        .and_then(|value| value.strip_suffix("_enabled"))
    {
        return skill_def
            .parse::<u32>()
            .ok()
            .map(SkillDefId::new)
            .map(TriggerFilter::OwnerSourceJiuBoostedShaDamageSkillEnabled);
    }
    Some(match s {
        "owner_alive_play_phase_card_category_count_positive" => {
            TriggerFilter::OwnerAlivePlayPhaseCardCategoryCountPositive
        }
        "owner_alive_play_phase_actor_hand_nonempty_potjiejie_unused" => {
            TriggerFilter::OwnerAlivePlayPhaseActorHandNonemptyPotjiejieUnused
        }
        "owner_gained_cards_potzhouyu" => TriggerFilter::OwnerGainedCardsPotzhouyu,
        "owner_alive_fire_damage_potfentao" => TriggerFilter::OwnerAliveFireDamagePotfentao,
        "owner_alive_potfentao_pending_damage" => TriggerFilter::OwnerAlivePotfentaoPendingDamage,
        "owner_alive_turn_ended_rehengjiang_pending" => {
            TriggerFilter::OwnerAliveTurnEndedRehengjiangPending
        }
        "owner_xicha_watched_source_card_used" => TriggerFilter::OwnerXichaWatchedSourceCardUsed,
        "owner_qiaojian_tracked_card_left_hand" => TriggerFilter::OwnerQiaojianTrackedCardLeftHand,
        "owner_alive_damage_target_self_or_in_range_other_source_numeric_card" => {
            TriggerFilter::OwnerAliveDamageTargetSelfOrInRangeOtherSourceNumericCard
        }
        "owner_discarded_has_hand_once_per_phase" => {
            TriggerFilter::OwnerDiscardedHasHandOncePerPhase
        }
        "owner_alive_lost_hand_this_turn" => TriggerFilter::OwnerAliveLostHandThisTurn,
        "owner_lost_hand_ownerless_sha_not_use_once_per_turn" => {
            TriggerFilter::OwnerLostHandOwnerlessShaNotUseOncePerTurn
        }
        "owner_mbrunwei_loss_threshold" => TriggerFilter::OwnerMbrunweiLossThreshold,
        "owner_alive_damage_before_target_distance_le_one_once_per_turn" => {
            TriggerFilter::OwnerAliveDamageBeforeTargetDistanceLeOneOncePerTurn
        }
        "owner_from_hand_empty_or_dying_mbsijian_twice_per_turn" => {
            TriggerFilter::OwnerFromHandEmptyOrDyingMbsijianTwicePerTurn
        }
        "owner_alive_damage_target_min_hand_mbxuye_once_per_turn" => {
            TriggerFilter::OwnerAliveDamageTargetMinHandMbxuyeOncePerTurn
        }
        "owner_mbkuangxiang_received_cards_exhausted" => {
            TriggerFilter::OwnerMbkuangxiangReceivedCardsExhausted
        }
        "owner_gained_cards_potduzuo" => TriggerFilter::OwnerGainedCardsPotduzuo,
        "owner_gained_cards_potduzuo_once_per_turn" => {
            TriggerFilter::OwnerGainedCardsPotduzuoOncePerTurn
        }
        "owner_sha_damage_before_potbihan_available" => {
            TriggerFilter::OwnerShaDamageBeforePotbihanAvailable
        }
        "owner_alive_card_targeted_healthy" => TriggerFilter::OwnerAliveCardTargetedHealthy,
        "owner_alive_card_resolved_source_current_potsifeng_pile" => {
            TriggerFilter::OwnerAliveCardResolvedSourceCurrentPotsifengPile
        }
        "owner_alive_potdingpin_available_once_per_round" => {
            TriggerFilter::OwnerAlivePotdingpinAvailableOncePerRound
        }
        "owner_outside_turn_discarded_red_cards_in_discard" => {
            TriggerFilter::OwnerOutsideTurnDiscardedRedCardsInDiscard
        }
        "owner_alive_twruilian_round_unprompted" => {
            TriggerFilter::OwnerAliveTwruilianRoundUnprompted
        }
        "owner_alive_twruilian_selected_target_discarded" => {
            TriggerFilter::OwnerAliveTwruilianSelectedTargetDiscarded
        }
        "owner_alive_twruilian_selected_target_turn_ended" => {
            TriggerFilter::OwnerAliveTwruilianSelectedTargetTurnEnded
        }
        "owner_alive_lost_weapon_outside_turn" => TriggerFilter::OwnerAliveLostWeaponOutsideTurn,
        "owner_prepare_group_outside_attack_range_exists" => {
            TriggerFilter::OwnerPrepareGroupOutsideAttackRangeExists
        }
        "owner_lost_one_hand_equip_judge_other_current_phase_once" => {
            TriggerFilter::OwnerLostOneHandEquipJudgeOtherCurrentPhaseOnce
        }
        "owner_alive_skill_bound_target_is_event_source" => {
            TriggerFilter::OwnerAliveSkillBoundTargetIsEventSource
        }
        "owner_lost_cards_outside_turn" => TriggerFilter::OwnerLostCardsOutsideTurn,
        "owner_gained_cards_during_play_phase_wounded" => {
            TriggerFilter::OwnerGainedCardsDuringPlayPhaseWounded
        }
        "owner_discarded_outside_play_phase" => TriggerFilter::OwnerDiscardedOutsidePlayPhase,
        "owner_in_multitarget_sha_or_trick_targets" => {
            TriggerFilter::OwnerInMultitargetShaOrTrickTargets
        }
        "owner_alive_damage_card_nanman" => TriggerFilter::OwnerAliveDamageCardNanman,
        "owner_discard_suit_no_own_mark_spade_falu" => {
            TriggerFilter::OwnerDiscardSuitNoOwnFaluMark(1, 3_540_560_359)
        }
        "owner_discard_suit_no_own_mark_heart_falu" => {
            TriggerFilter::OwnerDiscardSuitNoOwnFaluMark(2, 3_540_560_360)
        }
        "owner_discard_suit_no_own_mark_club_falu" => {
            TriggerFilter::OwnerDiscardSuitNoOwnFaluMark(3, 3_540_560_361)
        }
        "owner_discard_suit_no_own_mark_diamond_falu" => {
            TriggerFilter::OwnerDiscardSuitNoOwnFaluMark(4, 3_540_560_362)
        }
        "owner_black_card_moved_equip_judge" => TriggerFilter::OwnerBlackCardMovedEquipJudge,
        "owner_lost_non_damage_card_pottuntian_charge_not_full" => {
            TriggerFilter::OwnerLostNonDamageCardPottuntianChargeNotFull
        }
        "owner_alive_sha_entered_discard_potzhanlie_available" => {
            TriggerFilter::OwnerAliveShaEnteredDiscardPotzhanlieAvailable
        }
        "owner_responded_shan" => TriggerFilter::OwnerRespondedShan,
        "owner_alive_nanman_resolved_target_dead_or_damaged_gt_three" => {
            TriggerFilter::OwnerAliveNanmanResolvedTargetDeadOrDamagedGtThree
        }
        "owner_alive_gained_two_this_turn_can_compare_other" => {
            TriggerFilter::OwnerAliveGainedTwoThisTurnCanCompareOther
        }
        "owner_hand_compare_participant" => TriggerFilter::OwnerHandCompareParticipant,
        "owner_hand_compare_finished_participant" => {
            TriggerFilter::OwnerHandCompareFinishedParticipant
        }
        "owner_hand_compare_participant_has_sha" => {
            TriggerFilter::OwnerHandCompareParticipantHasSha
        }
        "owner_hand_compare_participant_result_card_in_discard" => {
            TriggerFilter::OwnerHandCompareParticipantResultCardInDiscard
        }
        "owner_alive_prepare_player_hp_gt_owner" => TriggerFilter::OwnerAlivePreparePlayerHpGtOwner,
        "owner_alive_prepare_lord_hp_gt_owner" => TriggerFilter::OwnerAlivePrepareLordHpGtOwner,
        "owner_sha_target_can_discard_redirect_to_other_in_attack_range" => {
            TriggerFilter::OwnerShaTargetCanDiscardRedirectToOtherInAttackRange
        }
        "owner_sha_target_can_discard_redirect_to_zhengjian_marked" => {
            TriggerFilter::OwnerShaTargetCanDiscardRedirectToZhengjianMarked
        }
        "owner_is_responder_sha" => TriggerFilter::OwnerIsResponderSha,
        "owner_alive_second_global_dying" => TriggerFilter::OwnerAliveSecondGlobalDying,
        "owner_used_and_was_targeted_this_turn_in_range" => {
            TriggerFilter::OwnerUsedAndWasTargetedThisTurnInRange
        }
        "owner_alive_gave_cards_to_other_alive" => TriggerFilter::OwnerAliveGaveCardsToOtherAlive,
        "owner_alive_play_phase_sha_user_has_discardable_hand_equip" => {
            TriggerFilter::OwnerAlivePlayPhaseShaUserHasDiscardableHandEquip
        }
        "owner_tracked_skill_card_left_hand_not_use_or_respond" => {
            TriggerFilter::OwnerTrackedSkillCardLeftHandNotUseOrRespond
        }
        "owner_alive_sha_used_by_self_or_in_range_discardable_hand_equip" => {
            TriggerFilter::OwnerAliveShaUsedBySelfOrInRangeDiscardableHandEquip
        }
        "owner_alive_tracked_cuijin_sha_resolved_no_damage" => {
            TriggerFilter::OwnerAliveTrackedCuijinShaResolvedNoDamage
        }
        "owner_alive_tracked_cuijin_sha_resolved_dealt_damage" => {
            TriggerFilter::OwnerAliveTrackedCuijinShaResolvedDealtDamage
        }
        "owner_alive_tracked_twqingkou_duel_resolved" => {
            TriggerFilter::OwnerAliveTrackedTwqingkouDuelResolved
        }
        "owner_alive_twchenxun_round_unprompted" => {
            TriggerFilter::OwnerAliveTwchenxunRoundUnprompted
        }
        "owner_alive_tracked_twchenxun_duel_target_damaged" => {
            TriggerFilter::OwnerAliveTrackedTwchenxunDuelTargetDamaged
        }
        "owner_alive_tracked_twchenxun_duel_target_not_damaged" => {
            TriggerFilter::OwnerAliveTrackedTwchenxunDuelTargetNotDamaged
        }
        "owner_alive_tracked_twchenxun_card_left_processing" => {
            TriggerFilter::OwnerAliveTrackedTwchenxunCardLeftProcessing
        }
        "owner_is_respond_to" => TriggerFilter::OwnerIsRespondTo,
        "owner_alive" => TriggerFilter::OwnerAlive,
        "owner_alive_killed_this_turn" => TriggerFilter::OwnerAliveKilledThisTurn,
        "owner_alive_hand_below_alive_count_capped_seven" => {
            TriggerFilter::OwnerAliveHandBelowAliveCountCappedSeven
        }
        "owner_alive_hand_below_alive_count_capped_eight" => {
            TriggerFilter::OwnerAliveHandBelowAliveCountCappedEight
        }
        "owner_alive_delayed_trick_target_without_pending_judge_skip" => {
            TriggerFilter::OwnerAliveDelayedTrickTargetWithoutPendingJudgeSkip
        }
        "owner_longdan_view_as_first_counterpart_hand_nonempty" => {
            TriggerFilter::OwnerLongdanViewAsFirstCounterpartHandNonempty
        }
        "owner_alive_sha_target_no_other_red_card_this_turn" => {
            TriggerFilter::OwnerAliveShaTargetNoOtherRedCardThisTurn
        }
        "owner_alive_neighbor_gained_cards_outside_own_draw_once_per_turn" => {
            TriggerFilter::OwnerAliveNeighborGainedCardsOutsideOwnDrawOncePerTurn
        }
        "owner_alive_damaged_event_player" => TriggerFilter::OwnerAliveDamagedEventPlayer,
        "owner_tracked_juedou_original_target_damage_recoverable_in_range" => {
            TriggerFilter::OwnerTrackedJuedouOriginalTargetDamageRecoverableInRange
        }
        "cilv_tracked_card_damage_before" => TriggerFilter::CilvTrackedCardDamageBefore,
        "cilv_tracked_card_resolved" => TriggerFilter::CilvTrackedCardResolved,
        "owner_dying_zhupeilan_once_per_game" => TriggerFilter::OwnerDyingZhupeilanOncePerGame,
        "owner_alive_global_fire_damage_dcsbronghuo" => {
            TriggerFilter::OwnerAliveGlobalFireDamageDcsbronghuo
        }
        "dcsbyingmou_tracked_card_resolved" => TriggerFilter::DcsbyingmouTrackedCardResolved,
        "owner_hand_transfer_other_dcsbmengmou_branch_available" => {
            TriggerFilter::OwnerHandTransferOtherDcsbmengmouBranchAvailable
        }
        "dcsbmengmou_tracked_sha_damage" => TriggerFilter::DcsbmengmouTrackedShaDamage,
        "dcsbmengmou_tracked_sha_resolved" => TriggerFilter::DcsbmengmouTrackedShaResolved,
        "dcyinjun_tracked_sha_resolved" => TriggerFilter::DcyinjunTrackedShaResolved,
        "owner_wounded_not_event_source" => TriggerFilter::OwnerWoundedNotEventSource,
        "owner_wounded_event_player_qun" => {
            TriggerFilter::OwnerWoundedEventPlayerGroup("qun".into())
        }
        "owner_alive_not_target" => TriggerFilter::OwnerAliveNotEventPlayer,
        "owner_alive_not_target_hand_equip_nonempty" => {
            TriggerFilter::OwnerAliveNotEventPlayerHandEquipNonempty
        }
        "owner_equip_zone_changed" => TriggerFilter::OwnerEquipZoneChanged,
        "owner_alive_not_target_neither_unique_max_hand" => {
            TriggerFilter::OwnerAliveNotEventPlayerNeitherUniqueMaxHand
        }
        "owner_alive_not_target_source_same_group" => {
            TriggerFilter::OwnerAliveNotEventPlayerSourceSameGroup
        }
        "owner_alive_not_target_equal_hand" => TriggerFilter::OwnerAliveNotEventPlayerEqualHand,
        "owner_alive_not_target_hand_below_hp" => {
            TriggerFilter::OwnerAliveNotEventPlayerHandBelowHp
        }
        "owner_alive_not_target_nonnegative_hp_gt_target" => {
            TriggerFilter::OwnerAliveNotEventPlayerNonnegativeHpGtEventPlayer
        }
        "owner_alive_current_turn_other_event_player" => {
            TriggerFilter::OwnerAliveCurrentTurnOtherEventPlayer
        }
        "owner_alive_current_turn" => TriggerFilter::OwnerAliveCurrentTurn,
        "owner_alive_group_count_at_most_2" => TriggerFilter::OwnerAliveGroupCountAtMostTwo,
        "debate_owner_is_initiator" => TriggerFilter::DebateOwnerIsInitiator,
        "debate_owner_participated" => TriggerFilter::DebateOwnerParticipated,
        "owner_debate_participant" => TriggerFilter::OwnerDebateParticipant,
        "owner_cards_moved_contain_ying" => TriggerFilter::OwnerCardsMovedContainYing,
        "owner_is_lord" => TriggerFilter::OwnerIsLord,
        "owner_alive_turn_used_all_suits" => TriggerFilter::OwnerAliveTurnUsedAllSuits,
        "cards_moved_discard_at_least_three" => TriggerFilter::CardsMovedDiscardAtLeastThree,
        "debate_owner_opinion_differs_initiator" => {
            TriggerFilter::DebateOwnerOpinionDiffersInitiator
        }
        "debate_owner_is_initiator_result_red" => TriggerFilter::DebateOwnerInitiatorResultRed,
        "debate_owner_is_initiator_result_black" => TriggerFilter::DebateOwnerInitiatorResultBlack,
        "owner_lost_package_card_from_equip" => TriggerFilter::OwnerLostPackageCardFromEquip,
        "owner_alive_current_turn_other_event_player_from_hand_empty" => {
            TriggerFilter::OwnerAliveCurrentTurnOtherEventPlayerFromHandEmpty
        }
        "owner_gains_multiple_outside_own_draw" => TriggerFilter::OwnerGainsMultipleOutsideOwnDraw,
        "owner_gained_or_lost_at_least_two" => TriggerFilter::OwnerGainedOrLostAtLeastTwo,
        "owner_card_responded_to_by_player_in_range" => {
            TriggerFilter::OwnerCardRespondedToByPlayerInRange
        }
        "owner_alive_first_cards_moved_to_hand" => TriggerFilter::OwnerAliveFirstCardsMovedToHand,
        "owner_draws_during_own_play_not_self" => TriggerFilter::OwnerDrawsDuringOwnPlayNotSelf,
        "owner_equip_zone_changed_alive_wounded_at_least_two" => {
            TriggerFilter::OwnerEquipZoneChangedAtLeastTwoWounded
        }
        "owner_current_turn_other_equipment_lost" => {
            TriggerFilter::OwnerCurrentTurnOtherEquipmentLost
        }
        "owner_first_hand_discard_has_sha_in_discard" => {
            TriggerFilter::OwnerFirstHandDiscardHasShaInDiscard
        }
        "owner_alive_dealt_or_received_damage_this_turn" => {
            TriggerFilter::OwnerAliveDealtOrReceivedDamageThisTurn
        }
        "owner_received_gift_from_other" => TriggerFilter::OwnerReceivedGiftFromOther,
        "owner_initiated_duel_target_alive_loser_has_hand_or_equip" => {
            TriggerFilter::OwnerInitiatedDuelTargetAliveLoserHasHandOrEquip
        }
        "owner_alive_equip_used" => TriggerFilter::OwnerAliveEquipUsed,
        "owner_alive_equip_used_skill_mark_below_three" => {
            TriggerFilter::OwnerAliveEquipUsedSkillMarkBelowThree
        }
        "owner_alive_once_per_round" => TriggerFilter::OwnerAliveOncePerRound,
        "owner_alive_not_target_once" => TriggerFilter::OwnerAliveNotEventPlayerOncePerTurn,
        "owner_alive_current_or_turn_fires_below_hp" => {
            TriggerFilter::OwnerAliveCurrentOrTurnFiresBelowHp
        }
        "owner_alive_card_is_sha" => TriggerFilter::OwnerAliveCardIsSha,
        "owner_alive_card_targets_gt_one" => TriggerFilter::OwnerAliveCardTargetsGtOne,
        "owner_alive_non_multitarget_trick_targets_gt_one" => {
            TriggerFilter::OwnerAliveNonMultitargetTrickTargetsGtOne
        }
        "owner_alive_wounded_card_targets_gt_one" => {
            TriggerFilter::OwnerAliveWoundedCardTargetsGtOne
        }
        "owner_alive_first_target_virtual_or_converted_card" => {
            TriggerFilter::OwnerAliveFirstTargetVirtualOrConvertedCard
        }
        "owner_min_hp" => TriggerFilter::OwnerMinHp,
        "owner_sha_used_at_least_two" => TriggerFilter::OwnerShaUsedAtLeastTwo,
        "owner_alive_used_sha_this_turn" => TriggerFilter::OwnerAliveUsedShaThisTurn,
        "owner_max_hp_gt_one_hand_or_equip_empty" => TriggerFilter::OwnerMaxHpGtOneHandOrEquipEmpty,
        "owner_hand_empty" | "hand_empty" => TriggerFilter::OwnerHandEmpty,
        "owner_turned_over" => TriggerFilter::OwnerTurnedOver,
        "owner_alive_global_juedou_or_red_sha_used" => {
            TriggerFilter::OwnerAliveGlobalJuedouOrRedShaUsed
        }
        "owner_skill_pile_nonempty" => TriggerFilter::OwnerSkillPileNonempty(u32::MAX),
        s_token
            if s_token
                .strip_prefix("owner_skill_pile_nonempty_")
                .and_then(|rest| rest.parse::<u32>().ok())
                .is_some() =>
        {
            TriggerFilter::OwnerSkillPileNonempty(
                s_token
                    .strip_prefix("owner_skill_pile_nonempty_")
                    .and_then(|rest| rest.parse::<u32>().ok())
                    .unwrap_or(0),
            )
        }
        "owner_from_hand" | "from_hand" => TriggerFilter::OwnerFromHand,
        "owner_from_equip" | "from_equip" => TriggerFilter::OwnerFromEquip,
        "owner_from_equip_per_card" => TriggerFilter::OwnerFromEquipPerCard,
        "owner_from_hand_empty" | "from_hand_empty" => TriggerFilter::OwnerFromHandEmpty,
        "owner_from_hand_empty_other_discardable_hand_equip" => {
            TriggerFilter::OwnerFromHandEmptyOtherDiscardableHandEquip
        }
        "owner_from_field_empty" | "from_field_empty" => TriggerFilter::OwnerFromFieldEmpty,
        "owner_gained_event_cards_from_other_personal_zone" => {
            TriggerFilter::OwnerGainedEventCardsFromOtherPersonalZone
        }
        "owner_tracked_twyizhu_entered_discard" => TriggerFilter::OwnerTrackedTwyizhuEnteredDiscard,
        "owner_used_playable_skill_pile_card_unused" => {
            TriggerFilter::OwnerUsedPlayableSkillPileCardUnused
        }
        "owner_alive_damage_before_target_min_hp_unused" => {
            TriggerFilter::OwnerAliveDamageBeforeTargetMinHpUnused
        }
        "owner_alive_global_phase_end" => TriggerFilter::OwnerAliveGlobalPhaseEnd,
        "owner_alive_first_non_discard_phase_discard_mbrenxing_available" => {
            TriggerFilter::OwnerAliveFirstNonDiscardPhaseDiscardMbrenxingAvailable
        }
        "owner_alive_global_phase_end_lost_cards_target_other_alive" => {
            TriggerFilter::OwnerAliveGlobalPhaseEndLostCardsTargetOtherAlive
        }
        "owner_alive_global_phase_end_discarded_cards_available" => {
            TriggerFilter::OwnerAliveGlobalPhaseEndDiscardedCardsAvailable
        }
        "owner_cards_entered_discard_not_use_unused" => {
            TriggerFilter::OwnerCardsEnteredDiscardNotUseUnused
        }
        "owner_alive_skill_bound_target_is_event_player" => {
            TriggerFilter::OwnerAliveSkillBoundTargetIsEventPlayer
        }
        "owner_hand_below_lost_hp_after_relevant_change" => {
            TriggerFilter::OwnerHandBelowLostHpAfterRelevantChange
        }
        "owner_alive_equip_source_max_equip" => TriggerFilter::OwnerAliveEquipSourceMaxEquip,
        "owner_first_transformed_sha_used_this_turn" => {
            TriggerFilter::OwnerFirstTransformedShaUsedThisTurn
        }
        "owner_alive_draw_player_owner_or_min_hp_twjiaohua_available" => {
            TriggerFilter::OwnerAliveDrawPlayerOwnerOrMinHpTwjiaohuaAvailable
        }
        "owner_alive_hand_gained_this_turn_can_compare_other" => {
            TriggerFilter::OwnerAliveGainedThisTurnCanCompareOther
        }
        "owner_alive_dying_has_skill_pile_cards" => TriggerFilter::OwnerAliveDyingHasSkillPileCards,
        "owner_discarded_sha_to_discard" => TriggerFilter::OwnerDiscardedShaToDiscard,
        "owner_discarded_black_basic_to_discard" => {
            TriggerFilter::OwnerDiscardedBlackBasicToDiscard
        }
        "owner_alive_discarded_black_basic_to_discard" => {
            TriggerFilter::OwnerAliveDiscardedBlackBasicToDiscard
        }
        "owner_alive_hp_changed_this_turn_entered_discard_available" => {
            TriggerFilter::OwnerAliveHpChangedThisTurnEnteredDiscardAvailable
        }
        "owner_play_dddzhengjun_hp_available" => TriggerFilter::OwnerPlayDddZhengjunHpAvailable,
        "owner_play_dddzhengjun_cards_moved_available" => {
            TriggerFilter::OwnerPlayDddZhengjunCardsMovedAvailable
        }
        "dddchashi_matching_card" => TriggerFilter::DddChashiMatchingCard,
        "owner_alive_dddqice_ready" => TriggerFilter::OwnerAliveDddQiceReady,
        "dddchashi_pending_phase_play_end" => TriggerFilter::DddChashiPendingPhasePlayEnd,
        "owner_alive_dddzhilian_available" => TriggerFilter::OwnerAliveDddZhilianAvailable,
        "once_per_turn" => TriggerFilter::OncePerTurn,
        "owner_alive_shan_used_by_owner_or_during_owner_turn" => {
            TriggerFilter::OwnerAliveShanUsedByOwnerOrDuringOwnerTurn
        }
        "owner_alive_sha_target_distance_le_one" => TriggerFilter::OwnerAliveShaTargetDistanceLeOne,
        "owner_alive_sha_target_distance_le_one_has_equip_card" => {
            TriggerFilter::OwnerAliveShaTargetDistanceLeOneHasEquipCard
        }
        "owner_alive_wdb_fuchou_pending_living" => TriggerFilter::OwnerAliveWdbFuchouPendingLiving,
        "owner_alive_wugu_remained_in_discard" => TriggerFilter::OwnerAliveWuguRemainedInDiscard,
        "owner_dying_other_hand_equip_nonempty_once_per_game" => {
            TriggerFilter::OwnerDyingOtherHandEquipNonemptyOncePerGame
        }
        "owner_alive_sha_or_duel_damage_source_target_alive_hand_equip_nonempty" => {
            TriggerFilter::OwnerAliveShaOrDuelDamageSourceTargetAliveHandEquipNonempty
        }
        "owner_alive_sha_damage_source_target_alive_owner_hand_equip_nonempty" => {
            TriggerFilter::OwnerAliveShaDamageSourceTargetAliveOwnerHandEquipNonempty
        }
        "owner_alive_gained_three_this_turn_other_alive" => {
            TriggerFilter::OwnerAliveGainedThreeThisTurnOtherAlive
        }
        "owner_twfeifu_single_sha_party_target_has_hand_equip" => {
            TriggerFilter::OwnerTwfeifuSingleShaPartyTargetHasHandEquip
        }
        "owner_card_action_count_eq_attack_range" => {
            TriggerFilter::OwnerCardActionCountEqualsAttackRange
        }
        "owner_lifetime_card_action" => TriggerFilter::OwnerLifetimeCardAction,
        "owner_has_enabled_equip_slot" => TriggerFilter::OwnerHasEnabledEquipSlot,
        "owner_alive_discard_batch" => TriggerFilter::OwnerAliveDiscardBatch,
        "owner_alive_dying_potjianzhuan_available" => {
            TriggerFilter::OwnerAliveDyingPotjianzhuanAvailable
        }
        "owner_alive_dying_potcaoshuang_assigned_untriggered" => {
            TriggerFilter::OwnerAliveDyingPotcaoshuangAssignedUntriggered
        }
        "owner_used_potnizun_pile_card" => TriggerFilter::OwnerUsedPotnizunPileCard,
        "owner_alive_turn_ended_used_card_tagged_discard" => {
            TriggerFilter::OwnerAliveTurnEndedUsedCardTaggedDiscard
        }
        "always" => TriggerFilter::Always,
        _ => return super::map_filter_source_target::map_filter(s),
    })
}
