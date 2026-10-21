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
        "owner_is_target" | "OwnerIsEventPlayer" | "owner_is_event_player" => {
            TriggerFilter::OwnerIsEventPlayer
        }
        "owner_source" => TriggerFilter::OwnerIsEventSource,
        "owner_is_target_not_unique_max_current_hp" => {
            TriggerFilter::OwnerIsEventPlayerNotUniqueMaxCurrentHp
        }
        "owner_is_target_wounded" => TriggerFilter::OwnerIsEventPlayerWounded,
        "owner_is_target_linked" => TriggerFilter::OwnerIsEventPlayerLinked,
        "owner_is_target_chain_damage_linked" => TriggerFilter::OwnerIsTargetChainDamageLinked,
        "owner_is_target_wounded_sha" => TriggerFilter::OwnerIsTargetWoundedSha,
        "owner_is_target_healthy_draw_not_fixed" => TriggerFilter::OwnerIsTargetHealthyDrawNotFixed,
        "owner_is_target_alive_wounded_or_guiming_wu_exists" => {
            TriggerFilter::OwnerIsEventPlayerAliveWoundedOrGuimingWuExists
        }
        "owner_is_target_hand_nonempty" => TriggerFilter::OwnerIsEventPlayerHandNonempty,
        "owner_is_target_hand_below_max_hp" => TriggerFilter::OwnerIsEventPlayerHandBelowMaxHp,
        "owner_is_target_first_hp_change_turn_unique_trick_majority" => {
            TriggerFilter::OwnerIsEventPlayerFirstHpChangeTurnUniqueTrickMajority
        }
        "owner_is_target_hand_above_hp" => TriggerFilter::OwnerIsEventPlayerHandAboveHp,
        "owner_is_target_hand_at_least_hp_other_alive" => {
            TriggerFilter::OwnerIsEventPlayerHandAtLeastHpOtherAlive
        }
        "owner_is_target_lost_cards_at_most_hp_gainable" => {
            TriggerFilter::OwnerIsEventPlayerLostCardsAtMostHpGainable
        }
        "owner_is_target_source_movable_field_card" => {
            TriggerFilter::OwnerIsEventPlayerSourceMovableFieldCard
        }
        "owner_is_target_no_turn_damage" => TriggerFilter::OwnerIsEventPlayerNoTurnDamage,
        "owner_is_target_turn_damage" => TriggerFilter::OwnerIsEventPlayerTurnDamage,
        "owner_is_target_turn_damage_at_least_three" => {
            TriggerFilter::OwnerIsEventPlayerTurnDamageAtLeastThree
        }
        "owner_source_target_distance_gt_one" => TriggerFilter::OwnerSourceTargetDistanceGtOne,
        "owner_is_target_card_used_at_least_hp" => {
            TriggerFilter::OwnerIsEventPlayerCardUsedAtLeastHp
        }
        "owner_is_target_play_phase_card_category_count_positive" => {
            TriggerFilter::OwnerIsEventPlayerPlayPhaseCardCategoryCountPositive
        }
        "owner_alive_play_phase_card_category_count_positive" => {
            TriggerFilter::OwnerAlivePlayPhaseCardCategoryCountPositive
        }
        "owner_is_target_other_hand_nonempty" => TriggerFilter::OwnerIsEventPlayerOtherHandNonempty,
        "owner_is_target_positive_draw_other_hand_nonempty" => {
            TriggerFilter::OwnerIsEventPlayerPositiveDrawOtherHandNonempty
        }
        "owner_source_first_target_black_card_any_target_other_discardable_hand_equip" => {
            TriggerFilter::OwnerSourceFirstTargetBlackCardAnyTargetOtherDiscardableHandEquip
        }
        "owner_source_direct_sha_damage_target_armor_or_mount_card_gainable" => {
            TriggerFilter::OwnerSourceDirectShaDamageTargetArmorOrMountCardGainable
        }
        "owner_alive_event_player_lost_armor_or_mount_has_originating_skill_pile" => {
            TriggerFilter::OwnerAliveEventPlayerLostArmorOrMountHasOriginatingSkillPile
        }
        "owner_is_source_or_target_xinyaoming_available" => {
            TriggerFilter::OwnerIsEventSourceOrPlayerXinyaomingAvailable
        }
        "owner_source_play_card_matches_remembered_category" => {
            TriggerFilter::OwnerSourcePlayCardMatchesRememberedCategory
        }
        "owner_source_damage_card_once_per_turn" => TriggerFilter::OwnerSourceDamageCardOncePerTurn,
        "owner_alive_event_player_hand_nonempty_once_per_round" => {
            TriggerFilter::OwnerAliveEventPlayerHandNonemptyOncePerRound
        }
        "owner_alive_play_phase_actor_hand_nonempty_potjiejie_unused" => {
            TriggerFilter::OwnerAlivePlayPhaseActorHandNonemptyPotjiejieUnused
        }
        "owner_source_sha_damage_before_potyinzhan_condition" => {
            TriggerFilter::OwnerSourceShaDamageBeforePotyinzhanCondition
        }
        "owner_source_damage_within_one" => TriggerFilter::OwnerSourceDamageWithinOne,
        "owner_gained_cards_potzhouyu" => TriggerFilter::OwnerGainedCardsPotzhouyu,
        "owner_source_first_target_has_hand_potyanhui" => {
            TriggerFilter::OwnerSourceFirstTargetHasHandPotyanhui
        }
        "owner_alive_fire_damage_potfentao" => TriggerFilter::OwnerAliveFireDamagePotfentao,
        "owner_alive_potfentao_pending_damage" => TriggerFilter::OwnerAlivePotfentaoPendingDamage,
        "owner_alive_turn_ended_rehengjiang_pending" => {
            TriggerFilter::OwnerAliveTurnEndedRehengjiangPending
        }
        "owner_alive_phase_play_end_zhengbi_pending" => {
            TriggerFilter::OwnerAlivePhasePlayEndZhengbiPending
        }
        "owner_is_target_source_hand_nonempty_per_damage_point" => {
            TriggerFilter::OwnerIsTargetSourceHandNonemptyPerDamagePoint
        }
        "owner_source_qiaojian_extra_available" => TriggerFilter::OwnerSourceQiaojianExtraAvailable,
        "owner_xicha_watched_source_card_used" => TriggerFilter::OwnerXichaWatchedSourceCardUsed,
        "owner_qiaojian_tracked_card_left_hand" => TriggerFilter::OwnerQiaojianTrackedCardLeftHand,
        "owner_alive_damage_target_self_or_in_range_other_source_numeric_card" => {
            TriggerFilter::OwnerAliveDamageTargetSelfOrInRangeOtherSourceNumericCard
        }
        "owner_source_wounded_exists_once_per_turn" => {
            TriggerFilter::OwnerSourceWoundedExistsOncePerTurn
        }
        "owner_is_target_discardable_hand_equip_gte_two_once_per_round" => {
            TriggerFilter::OwnerIsEventPlayerDiscardableHandEquipGteTwoOncePerRound
        }
        "owner_is_target_other_source_alive_hand_equip_nonempty_oldhuisheng_unused" => {
            TriggerFilter::OwnerIsEventPlayerOtherSourceAliveHandEquipNonemptyOldHuishengUnused
        }
        "owner_alive_other_dying_outside_turn_can_sha_current" => {
            TriggerFilter::OwnerAliveOtherDyingOutsideTurnCanShaCurrent
        }
        "owner_source_oldjiefan_sha" => TriggerFilter::OwnerSourceOldJiefanSha,
        "owner_source_current_turn_first_target_non_equip" => {
            TriggerFilter::OwnerSourceCurrentTurnFirstTargetNonEquip
        }
        "owner_source_face_down_view_as_chenwei_available" => {
            TriggerFilter::OwnerSourceFaceDownViewAsChenweiAvailable
        }
        "owner_source_bound_category_first_three_play_phase" => {
            TriggerFilter::OwnerSourceBoundCategoryFirstThreePlayPhase
        }
        "owner_phase_gift_target_is_source" => TriggerFilter::OwnerPhaseGiftTargetIsEventSource,
        "owner_phase_gift_target_is_target" => TriggerFilter::OwnerPhaseGiftTargetIsEventPlayer,
        "owner_source_sha_resolved_has_gainable_cards_and_recipient" => {
            TriggerFilter::OwnerSourceShaResolvedHasGainableCardsAndRecipient
        }
        "owner_alive_other_source_sha_resolved_not_target_gainable_once_per_turn" => {
            TriggerFilter::OwnerAliveOtherSourceShaResolvedNotTargetGainableOncePerTurn
        }
        "owner_is_target_tied_max_hand" => TriggerFilter::OwnerIsEventPlayerTiedMaxHand,
        "owner_is_target_tied_max_hp" => TriggerFilter::OwnerIsEventPlayerTiedMaxHp,
        "owner_is_target_tied_max_equip_nonempty" => {
            TriggerFilter::OwnerIsEventPlayerTiedMaxEquipNonempty
        }
        "owner_is_target_skipped_phase_count_positive" => {
            TriggerFilter::OwnerIsEventPlayerSkippedPhaseCountPositive
        }
        "owner_discarded_has_hand_once_per_phase" => {
            TriggerFilter::OwnerDiscardedHasHandOncePerPhase
        }
        "owner_alive_lost_hand_this_turn" => TriggerFilter::OwnerAliveLostHandThisTurn,
        "owner_alive_phase_end_lost_cards_at_least_two" => {
            TriggerFilter::OwnerAlivePhaseEndLostCardsAtLeastTwo
        }
        "owner_source_first_sha_this_play_phase" => TriggerFilter::OwnerSourceFirstShaThisPlayPhase,
        "owner_source_damage_target_distance_at_most_one_empty_equip_slot_unused" => {
            TriggerFilter::OwnerSourceDamageTargetDistanceAtMostOneEmptyEquipSlotUnused
        }
        "owner_source_first_sha_target_this_turn" => {
            TriggerFilter::OwnerSourceFirstShaTargetThisTurn
        }
        "owner_alive_other_phase_play_end_twzhengjian_condition" => {
            TriggerFilter::OwnerAliveOtherPhasePlayEndTwzhengjianCondition
        }
        "owner_source_sha_target_twzhenxi_available_once_per_turn" => {
            TriggerFilter::OwnerSourceShaTargetTwzhenxiAvailableOncePerTurn
        }
        "owner_source_first_target_damage_card_once_per_turn" => {
            TriggerFilter::OwnerSourceFirstTargetDamageCardOncePerTurn
        }
        "owner_source_first_target_hand_damage_card_discardable" => {
            TriggerFilter::OwnerSourceFirstTargetHandDamageCardDiscardable
        }
        "owner_lost_hand_ownerless_sha_not_use_once_per_turn" => {
            TriggerFilter::OwnerLostHandOwnerlessShaNotUseOncePerTurn
        }
        "owner_alive_phase_any_end_originating_skill_pile_nonempty" => {
            TriggerFilter::OwnerAlivePhaseAnyEndOriginatingSkillPileNonempty
        }
        "owner_source_damage_tracked_mbduanyang_target_recastable" => {
            TriggerFilter::OwnerSourceDamageTrackedMbduanyangTargetRecastable
        }
        "owner_mbrunwei_loss_threshold" => TriggerFilter::OwnerMbrunweiLossThreshold,
        "owner_phase_play_end_mbrunwei_tracked_hand" => {
            TriggerFilter::OwnerPhasePlayEndMbrunweiTrackedHand
        }
        "owner_alive_damage_before_target_distance_le_one_once_per_turn" => {
            TriggerFilter::OwnerAliveDamageBeforeTargetDistanceLeOneOncePerTurn
        }
        "owner_source_damage_mbxiongtu_armed" => TriggerFilter::OwnerSourceDamageMbxiongtuArmed,
        "owner_phase_play_end_mbxiongtu_state" => TriggerFilter::OwnerPhasePlayEndMbxiongtuState,
        "owner_phase_end_mbganggeng_target_alive" => {
            TriggerFilter::OwnerPhaseEndMbganggengTargetAlive
        }
        "owner_from_hand_empty_or_dying_mbsijian_twice_per_turn" => {
            TriggerFilter::OwnerFromHandEmptyOrDyingMbsijianTwicePerTurn
        }
        "owner_is_target_can_move_field_card_mbyuanmo_twice_per_turn" => {
            TriggerFilter::OwnerIsTargetCanMoveFieldCardMbyuanmoTwicePerTurn
        }
        "owner_source_first_single_target_sha_mbfeijing_twice_per_turn" => {
            TriggerFilter::OwnerSourceFirstSingleTargetShaMbfeijingTwicePerTurn
        }
        "owner_source_damage_before_mbfeijing_extra" => {
            TriggerFilter::OwnerSourceDamageBeforeMbfeijingExtra
        }
        "owner_source_sha_resolved_original_single_mbxiaoge" => {
            TriggerFilter::OwnerSourceShaResolvedOriginalSingleMbxiaoge
        }
        "owner_alive_damage_target_min_hand_mbxuye_once_per_turn" => {
            TriggerFilter::OwnerAliveDamageTargetMinHandMbxuyeOncePerTurn
        }
        "owner_mbkuangxiang_received_cards_exhausted" => {
            TriggerFilter::OwnerMbkuangxiangReceivedCardsExhausted
        }
        "owner_phase_play_mbkuangxiang_active" => TriggerFilter::OwnerPhasePlayMbkuangxiangActive,
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
        "owner_alive_phase_end_player_potsifeng_pile" => {
            TriggerFilter::OwnerAlivePhaseEndPlayerPotsifengPile
        }
        "owner_alive_potdingpin_available_once_per_round" => {
            TriggerFilter::OwnerAlivePotdingpinAvailableOncePerRound
        }
        "owner_outside_turn_discarded_red_cards_in_discard" => {
            TriggerFilter::OwnerOutsideTurnDiscardedRedCardsInDiscard
        }
        "owner_alive_other_twkujian_used_or_responded" => {
            TriggerFilter::OwnerAliveOtherTwkujianUsedOrResponded
        }
        "owner_alive_other_twkujian_lost" => TriggerFilter::OwnerAliveOtherTwkujianLost,
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
        "owner_alive_other_phase_prepare_can_compare_owner" => {
            TriggerFilter::OwnerAliveOtherPhasePrepareCanCompareOwner
        }
        "owner_is_target_discarded_at_least_two_in_discard_phase" => {
            TriggerFilter::OwnerIsEventPlayerDiscardedAtLeastTwoInDiscardPhase
        }
        "owner_alive_other_discarded_in_discard_phase" => {
            TriggerFilter::OwnerAliveOtherDiscardedInDiscardPhase
        }
        "owner_alive_other_wu_play_phase_can_compare_owner" => {
            TriggerFilter::OwnerAliveOtherWuPlayPhaseCanCompareOwner
        }
        "owner_alive_other_play_phase_zhengsu_missing_in_range" => {
            TriggerFilter::OwnerAliveOtherPlayPhaseZhengsuMissingInRange
        }
        "owner_is_target_missing_zhengsu" => TriggerFilter::OwnerIsTargetMissingZhengsu,
        "owner_source_club" => TriggerFilter::OwnerSourceClub,
        "owner_alive_other_play_phase_discardable_gte_mark" => {
            TriggerFilter::OwnerAliveOtherPlayPhaseDiscardableGteMark
        }
        "owner_alive_other_trick_first_target_source_max_hand" => {
            TriggerFilter::OwnerAliveOtherTrickFirstTargetSourceMaxHand
        }
        "owner_alive_other_marked_xi" => TriggerFilter::OwnerAliveOtherMarkedXi,
        "owner_alive_other_recovered_outside_owner_turn" => {
            TriggerFilter::OwnerAliveOtherRecoveredOutsideOwnerTurn
        }
        "owner_is_target_discarded_sha_in_discard_phase" => {
            TriggerFilter::OwnerIsTargetDiscardedShaInDiscardPhase
        }
        "owner_source_sha_damage_before_target_healthy" => {
            TriggerFilter::OwnerSourceShaDamageBeforeTargetHealthy
        }
        "owner_phase_end_originating_skill_pile_empty" => {
            TriggerFilter::OwnerPhaseEndOriginatingSkillPileEmpty
        }
        "owner_prepare_group_outside_attack_range_exists" => {
            TriggerFilter::OwnerPrepareGroupOutsideAttackRangeExists
        }
        "owner_is_target_single_target_other_source_once_per_turn" => {
            TriggerFilter::OwnerIsTargetSingleTargetOtherSourceOncePerTurn
        }
        "owner_lost_one_hand_equip_judge_other_current_phase_once" => {
            TriggerFilter::OwnerLostOneHandEquipJudgeOtherCurrentPhaseOnce
        }
        "owner_is_target_multi_target_trick_has_hand_equip" => {
            TriggerFilter::OwnerIsTargetMultiTargetTrickHasHandEquip
        }
        "owner_is_target_feijun_new_target" => TriggerFilter::OwnerIsTargetFeijunNewTarget,
        "owner_is_target_discard_phase_owe_limit_other_qun" => {
            TriggerFilter::OwnerIsTargetDiscardPhaseOweLimitOtherQun
        }
        "owner_source_sha_damage_target_alive_both_hand" => {
            TriggerFilter::OwnerSourceShaDamageTargetAliveBothHand
        }
        "owner_alive_other_nanman_cards_discard" => {
            TriggerFilter::OwnerAliveOtherNanmanCardsDiscard
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
        "owner_targeted_by_nanman" => TriggerFilter::OwnerTargetedByNanman,
        "owner_targeted_source_hp_hand_differ_once_per_turn" => {
            TriggerFilter::OwnerTargetedSourceHpHandDifferOncePerTurn
        }
        "owner_source_trick_resolved_own_play_phase_once_per_turn" => {
            TriggerFilter::OwnerSourceTrickResolvedOwnPlayPhaseOncePerTurn
        }
        "owner_alive_other_dying_has_equip_card" => TriggerFilter::OwnerAliveOtherDyingHasEquipCard,
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
        "owner_source_sha_resolved_tracked_potceduan_dealt_damage" => {
            TriggerFilter::OwnerSourceShaResolvedTrackedPotceduanDealtDamage
        }
        "owner_alive_sha_entered_discard_potzhanlie_available" => {
            TriggerFilter::OwnerAliveShaEnteredDiscardPotzhanlieAvailable
        }
        "owner_source_sha_resolved_tracked_potzhanlie_draw" => {
            TriggerFilter::OwnerSourceShaResolvedTrackedPotzhanlieDraw
        }
        "owner_phase_begin_skill_pile_gte_three" => {
            TriggerFilter::OwnerPhaseBeginSkillPileGteThree(0)
        }
        "owner_phase_begin_hand_empty" => TriggerFilter::OwnerPhaseBeginHandEmpty,
        "owner_responded_shan" => TriggerFilter::OwnerRespondedShan,
        "owner_alive_other_qun_play_phase_can_give_owner" => {
            TriggerFilter::OwnerAliveOtherQunPlayPhaseCanGiveOwner
        }
        "owner_source_first_damage_card_target_hand_gte_owner" => {
            TriggerFilter::OwnerSourceFirstDamageCardTargetHandGteOwner
        }
        "owner_alive_nanman_resolved_target_dead_or_damaged_gt_three" => {
            TriggerFilter::OwnerAliveNanmanResolvedTargetDeadOrDamagedGtThree
        }
        "owner_source_single_target_sha_or_black_ordinary_trick_current_play_phase_unused" => {
            TriggerFilter::OwnerSourceSingleTargetShaOrBlackOrdinaryTrickCurrentPlayPhaseUnused
        }
        "owner_is_target_male_source_has_discardable_equipment" => {
            TriggerFilter::OwnerIsEventPlayerMaleSourceHasDiscardableEquipment
        }
        "owner_alive_other_turn_ended_no_card_targeted_other" => {
            TriggerFilter::OwnerAliveOtherTurnEndedNoCardTargetedOther
        }
        "owner_alive_other_turn_ended_no_damage_other" => {
            TriggerFilter::OwnerAliveOtherTurnEndedNoDamageOther
        }
        "owner_alive_gained_two_this_turn_can_compare_other" => {
            TriggerFilter::OwnerAliveGainedTwoThisTurnCanCompareOther
        }
        "owner_alive_other_turn_ended_card_targeted_other_no_damage_other" => {
            TriggerFilter::OwnerAliveOtherTurnEndedCardTargetedOtherNoDamageOther
        }
        "owner_alive_phase_targeted_hp_changed_can_use_wanjian" => {
            TriggerFilter::OwnerAlivePhaseTargetedHpChangedCanUseWanjian
        }
        "owner_alive_phase_targeted_hp_unchanged_can_use_juedou" => {
            TriggerFilter::OwnerAlivePhaseTargetedHpUnchangedCanUseJuedou
        }
        "owner_source_sha_has_weapon" => TriggerFilter::OwnerSourceShaHasWeapon,
        "owner_source_sha_direct_damage_before" => TriggerFilter::OwnerSourceShaDirectDamageBefore,
        "owner_source_other_damage_before" => TriggerFilter::OwnerSourceOtherDamageBefore,
        "owner_source_other_damage_before_once_per_turn" => {
            TriggerFilter::OwnerSourceOtherDamageBeforeOncePerTurn
        }
        "owner_source_other_direct_damage_before" => {
            TriggerFilter::OwnerSourceOtherDirectDamageBefore
        }
        "owner_is_target_discardable_hand_equip" => {
            TriggerFilter::OwnerIsTargetDiscardableHandEquip
        }
        "owner_is_target_discardable_hand_equip_gte_two" => {
            TriggerFilter::OwnerIsTargetDiscardableHandEquipGteTwo
        }
        "owner_source_or_target_hand_count_differs_once_per_turn" => {
            TriggerFilter::OwnerSourceOrTargetHandCountDiffersOncePerTurn
        }
        "owner_is_target_any_alive_hand_nonempty" => {
            TriggerFilter::OwnerIsEventPlayerAnyAliveHandNonempty
        }
        "owner_is_target_any_alive_hand_empty" => {
            TriggerFilter::OwnerIsEventPlayerAnyAliveHandEmpty
        }
        "owner_is_target_other_hand_lte_owner" => {
            TriggerFilter::OwnerIsEventPlayerOtherHandLteOwner
        }
        "owner_is_target_draw_reshuffle_zero_wounded" => {
            TriggerFilter::OwnerIsEventPlayerDrawReshuffleZeroWounded
        }
        "owner_is_target_draw_reshuffle_positive" => {
            TriggerFilter::OwnerIsEventPlayerDrawReshufflePositive
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
        "owner_source_basic" => TriggerFilter::OwnerSourceBasic,
        "owner_source_non_basic" => TriggerFilter::OwnerSourceNonBasic,
        "owner_source_basic_or_trick" => TriggerFilter::OwnerSourceBasicOrTrick,
        "owner_source_wounded_nanman_wanjian_or_normal_spade_sha" => {
            TriggerFilter::OwnerSourceWoundedNanmanWanjianOrNormalSpadeSha
        }
        "owner_alive_other_play_phase_duplicate_card_category_has_hand_equip" => {
            TriggerFilter::OwnerAliveOtherPlayPhaseDuplicateCardCategoryHasHandEquip
        }
        "owner_alive_other_phase_end_damage_gt_one_has_hand_equip" => {
            TriggerFilter::OwnerAliveOtherPhaseEndDamageGtOneHasHandEquip
        }
        "owner_alive_other_phase_end_has_discardable_basic_hand" => {
            TriggerFilter::OwnerAliveOtherPhaseEndHasDiscardableBasicHand
        }
        "owner_alive_other_phase_end_hand_nonempty" => {
            TriggerFilter::OwnerAliveOtherPhaseEndHandNonempty
        }
        "owner_alive_prepare_player_hp_gt_owner" => TriggerFilter::OwnerAlivePreparePlayerHpGtOwner,
        "owner_alive_prepare_lord_hp_gt_owner" => TriggerFilter::OwnerAlivePrepareLordHpGtOwner,
        "owner_alive_other_sha_target_can_redirect_to_owner" => {
            TriggerFilter::OwnerAliveOtherShaTargetCanRedirectToOwner
        }
        "owner_sha_target_can_discard_redirect_to_other_in_attack_range" => {
            TriggerFilter::OwnerShaTargetCanDiscardRedirectToOtherInAttackRange
        }
        "owner_sha_target_can_discard_redirect_to_zhengjian_marked" => {
            TriggerFilter::OwnerShaTargetCanDiscardRedirectToZhengjianMarked
        }
        "owner_is_target_sha_source_alive_hand_equip_nonempty_once_per_turn" => {
            TriggerFilter::OwnerIsTargetShaSourceAliveHandEquipNonemptyOncePerTurn
        }
        "owner_is_target_trick_targets_gt_source_hp_hand_equip_nonempty" => {
            TriggerFilter::OwnerIsTargetTrickTargetsGtSourceHpHandEquipNonempty
        }
        "owner_is_source_sha_used" => TriggerFilter::OwnerIsSourceShaUsed,
        "owner_is_responder_sha" => TriggerFilter::OwnerIsResponderSha,
        "owner_is_source_sha_card_damage_before" => TriggerFilter::OwnerIsSourceShaCardDamageBefore,
        "owner_is_target_source_hp_gt_owner_per_role_turn" => {
            TriggerFilter::OwnerIsTargetSourceHpGtOwnerPerRoleTurn
        }
        "owner_phase_end_lost_cards" => TriggerFilter::OwnerPhaseEndLostCards,
        "owner_alive_second_global_dying" => TriggerFilter::OwnerAliveSecondGlobalDying,
        "owner_used_and_was_targeted_this_turn_in_range" => {
            TriggerFilter::OwnerUsedAndWasTargetedThisTurnInRange
        }
        "owner_source_in_turn_target_unharmed_card_actions_ge_target_hp" => {
            TriggerFilter::OwnerSourceInTurnTargetUnharmedCardActionsGeTargetHp
        }
        "owner_is_target_source_hp_gt_owner_or_source_damaged_owner" => {
            TriggerFilter::OwnerIsTargetSourceHpGtOwnerOrSourceDamagedOwner
        }
        "owner_is_target_sanshou_type_tracked" => TriggerFilter::OwnerIsTargetSanshouTypeTracked,
        "owner_is_target_owner_hp_gt_source_or_source_damaged_owner" => {
            TriggerFilter::OwnerIsTargetOwnerHpGtSourceOrSourceDamagedOwner
        }
        "owner_is_target_damage_amount_gt_one" => TriggerFilter::OwnerIsTargetDamageAmountGtOne,
        "owner_alive_gave_cards_to_other_alive" => TriggerFilter::OwnerAliveGaveCardsToOtherAlive,
        "owner_is_target_can_compare_other" => TriggerFilter::OwnerIsEventPlayerCanCompareOther,
        "owner_alive_play_phase_sha_user_has_discardable_hand_equip" => {
            TriggerFilter::OwnerAlivePlayPhaseShaUserHasDiscardableHandEquip
        }
        "owner_is_target_use_response_discarded_at_least_hp" => {
            TriggerFilter::OwnerIsEventPlayerUseResponseDiscardedAtLeastHp
        }
        "owner_is_target_damaged_other_qun" => TriggerFilter::OwnerIsEventPlayerDamagedOtherQun,
        "owner_is_target_source_alive" => TriggerFilter::OwnerIsEventPlayerSourceAlive,
        "owner_alive_other_source_single_sha_target_other" => {
            TriggerFilter::OwnerAliveOtherSourceSingleShaTargetOther
        }
        "owner_alive_other_source_single_sha_target_other_owner_in_attack_range" => {
            TriggerFilter::OwnerAliveOtherSourceSingleShaTargetOtherOwnerInAttackRange
        }
        "owner_alive_other_source_sha_resolved_twxiaolian_redirect_has_hand_equip" => {
            TriggerFilter::OwnerAliveOtherSourceShaResolvedTwxiaolianRedirectHasHandEquip
        }
        "owner_alive_other_sha_resolved_tracked_skill_card_dealt_damage" => {
            TriggerFilter::OwnerAliveOtherShaResolvedTrackedSkillCardDealtDamage
        }
        "owner_alive_other_sha_resolved_tracked_skill_card_no_damage" => {
            TriggerFilter::OwnerAliveOtherShaResolvedTrackedSkillCardNoDamage
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
        "owner_is_target_twxiaolian_redirected_damage" => {
            TriggerFilter::OwnerIsEventPlayerTwxiaolianRedirectedDamage
        }
        "owner_is_target_other_higher_hand_once_per_game" => {
            TriggerFilter::OwnerIsEventPlayerOtherHigherHandOncePerGame
        }
        "owner_is_target_source_alive_once" => {
            TriggerFilter::OwnerIsEventPlayerSourceAliveOncePerTurn
        }
        "owner_is_target_source_has_weapon" => TriggerFilter::OwnerIsEventPlayerSourceHasWeapon,
        "owner_is_target_source_alive_disabled_weapon" => {
            TriggerFilter::OwnerIsEventPlayerSourceAliveDisabledWeapon
        }
        "owner_is_target_other_source_alive" => TriggerFilter::OwnerIsEventPlayerOtherSourceAlive,
        "owner_is_target_other_source_alive_per_damage_point" => {
            TriggerFilter::OwnerIsEventPlayerOtherSourceAlivePerDamagePoint
        }
        "owner_is_target_per_damage_point" => TriggerFilter::OwnerIsEventPlayerPerDamagePoint,
        "owner_is_target_per_damage_point_other_hand_equip_judge_nonempty" => {
            TriggerFilter::OwnerIsEventPlayerPerDamagePointOtherHandEquipJudgeNonempty
        }
        "owner_is_target_other_alive" => TriggerFilter::OwnerIsEventPlayerOtherAlive,
        "owner_is_target_source_alive_gainable_card_per_damage_point" => {
            TriggerFilter::OwnerIsEventPlayerSourceAliveGainableCardPerDamagePoint
        }
        "owner_is_target_source_alive_gainable_card" => {
            TriggerFilter::OwnerIsEventPlayerSourceAliveGainableCard
        }
        "owner_is_target_card_in_processing" => TriggerFilter::OwnerIsEventPlayerCardInProcessing,
        "owner_is_target_red_sha_source_alive" => {
            TriggerFilter::OwnerIsEventPlayerRedShaSourceAlive
        }
        "owner_is_target_nonred_sha" => TriggerFilter::OwnerIsEventPlayerNonRedSha,
        "owner_is_target_red_or_jiu_sha" => TriggerFilter::OwnerIsEventPlayerRedOrJiuSha,
        "owner_is_target_sha_source_has_discardable_hand_equip" => {
            TriggerFilter::OwnerIsEventPlayerShaSourceHasDiscardableHandEquip
        }
        "owner_is_target_red_card_source_alive" => {
            TriggerFilter::OwnerIsEventPlayerRedCardSourceAlive
        }
        "owner_is_target_nonred_card" => TriggerFilter::OwnerIsEventPlayerNonRedCard,
        "owner_is_target_first_damage" => TriggerFilter::OwnerIsEventPlayerFirstDamagePerTurn,
        "owner_is_target_second_damage" => TriggerFilter::OwnerIsEventPlayerSecondDamagePerTurn,
        "owner_is_target_after_first_damage" => {
            TriggerFilter::OwnerIsEventPlayerAfterFirstDamagePerTurn
        }
        "owner_source_distance_le_one_per_damage_point" => {
            TriggerFilter::OwnerSourceDistanceLeOnePerDamagePoint
        }
        "owner_is_respond_to" => TriggerFilter::OwnerIsRespondTo,
        "owner_is_source" | "OwnerIsEventSource" | "owner_is_event_source" => {
            TriggerFilter::OwnerIsEventSource
        }
        "owner_alive_is_source" => TriggerFilter::OwnerAliveIsEventSource,
        "owner_source_target_alive_has_hand_or_equip" => {
            TriggerFilter::OwnerSourceTargetAliveHasHandOrEquip
        }
        "owner_source_other_target_alive_has_discardable_hand_equip" => {
            TriggerFilter::OwnerSourceOtherTargetAliveHasDiscardableHandEquip
        }
        "owner_source_sha_target_alive_owner_outside_target_attack_range" => {
            TriggerFilter::OwnerSourceShaTargetAliveOwnerOutsideTargetAttackRange
        }
        "owner_is_source_or_target" => TriggerFilter::OwnerIsEventSourceOrPlayer,
        "owner_is_source_or_target_other_alive_per_damage_point" => {
            TriggerFilter::OwnerSourceOrTargetOtherAlivePerDamagePoint
        }
        "owner_source_fire" => TriggerFilter::OwnerSourceFire,
        "owner_target_fire" => TriggerFilter::OwnerTargetFire,
        "owner_target_thunder" => TriggerFilter::OwnerTargetThunder,
        "owner_source_thunder" => TriggerFilter::OwnerSourceThunder,
        "owner_source_wounded_distance_le_one" => TriggerFilter::OwnerSourceWoundedDistanceLeOne,
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
        "owner_source_sha_or_juedou_single_target_first_same_name_this_turn" => {
            TriggerFilter::OwnerSourceShaOrJuedouSingleTargetFirstSameNameThisTurn
        }
        "owner_source_sha_or_juedou_single_target_hand_nonempty" => {
            TriggerFilter::OwnerSourceShaOrJuedouSingleTargetHandNonempty
        }
        "owner_source_sha_damage_before_endpoint_discardable_hand_equip" => {
            TriggerFilter::OwnerSourceShaDamageBeforeEndpointDiscardableHandEquip
        }
        "owner_longdan_view_as_first_counterpart_hand_nonempty" => {
            TriggerFilter::OwnerLongdanViewAsFirstCounterpartHandNonempty
        }
        "owner_is_source_hand_below_alive_count_capped_seven" => {
            TriggerFilter::OwnerSourceHandBelowAliveCountCappedSeven
        }
        "owner_alive_event_player_alive" => TriggerFilter::OwnerAliveEventPlayerAlive,
        "owner_alive_event_player_equip_empty" => TriggerFilter::OwnerAliveEventPlayerEquipEmpty,
        "owner_alive_event_player_from_equip_empty" => {
            TriggerFilter::OwnerAliveEventPlayerFromEquipEmpty
        }
        "owner_alive_sha_target_no_other_red_card_this_turn" => {
            TriggerFilter::OwnerAliveShaTargetNoOtherRedCardThisTurn
        }
        "owner_alive_event_player_hand_empty" => TriggerFilter::OwnerAliveEventPlayerHandEmpty,
        "owner_alive_event_cards_nonempty" => TriggerFilter::OwnerAliveEventCardsNonempty,
        "owner_alive_neighbor_gained_cards_outside_own_draw_once_per_turn" => {
            TriggerFilter::OwnerAliveNeighborGainedCardsOutsideOwnDrawOncePerTurn
        }
        "owner_alive_event_player_outside_turn_from_hand_empty" => {
            TriggerFilter::OwnerAliveEventPlayerOutsideTurnFromHandEmpty
        }
        "owner_alive_event_player_outside_turn_from_hand_empty_once" => {
            TriggerFilter::OwnerAliveEventPlayerOutsideTurnFromHandEmptyOncePerTurn
        }
        "owner_alive_event_player_turn_changed_or_became_linked" => {
            TriggerFilter::OwnerAliveEventPlayerTurnChangedOrBecameLinked
        }
        "owner_alive_event_player_became_face_up_or_linked" => {
            TriggerFilter::OwnerAliveEventPlayerBecameFaceUpOrLinked
        }
        "owner_alive_event_player_distance_le_owner_hp" => {
            TriggerFilter::OwnerAliveEventPlayerDistanceLeOwnerHp
        }
        "owner_alive_other_normal_damage_target_unlinked" => {
            TriggerFilter::OwnerAliveOtherNormalDamageTargetUnlinked
        }
        "owner_alive_damaged_event_player" => TriggerFilter::OwnerAliveDamagedEventPlayer,
        "owner_alive_event_player_owner_or_female" => {
            TriggerFilter::OwnerAliveEventPlayerOwnerOrFemale
        }
        "owner_alive_source_target_same_sex" => TriggerFilter::OwnerAliveSourceTargetSameSex,
        "owner_alive_event_player_same_group" => TriggerFilter::OwnerAliveEventPlayerSameGroup,
        "owner_alive_event_player_same_group_hand_nonempty" => {
            TriggerFilter::OwnerAliveEventPlayerSameGroupHandNonempty
        }
        "owner_tracked_juedou_original_target_damage_recoverable_in_range" => {
            TriggerFilter::OwnerTrackedJuedouOriginalTargetDamageRecoverableInRange
        }
        "owner_is_target_phase_end_hand_nonempty_other_hand_nonempty" => {
            TriggerFilter::OwnerIsTargetPhaseEndHandNonemptyOtherHandNonempty
        }
        "owner_is_target_ordinary_trick_cilv_available" => {
            TriggerFilter::OwnerIsTargetOrdinaryTrickCilvAvailable
        }
        "cilv_tracked_card_damage_before" => TriggerFilter::CilvTrackedCardDamageBefore,
        "cilv_tracked_card_resolved" => TriggerFilter::CilvTrackedCardResolved,
        "owner_dying_zhupeilan_once_per_game" => TriggerFilter::OwnerDyingZhupeilanOncePerGame,
        "owner_alive_other_play_phase_tied_max_hand_dcsbfuxi_available" => {
            TriggerFilter::OwnerAliveOtherPlayPhaseTiedMaxHandDcsbfuxiAvailable
        }
        "owner_phase_end_dcsbhaoyi_available" => TriggerFilter::OwnerPhaseEndDcsbhaoyiAvailable,
        "owner_alive_global_fire_damage_dcsbronghuo" => {
            TriggerFilter::OwnerAliveGlobalFireDamageDcsbronghuo
        }
        "owner_source_fire_sha_or_huogong_dcsbronghuo" => {
            TriggerFilter::OwnerSourceFireShaOrHuogongDcsbronghuo
        }
        "owner_source_card_resolved_has_other_target_dcsbyingmou_once_per_turn" => {
            TriggerFilter::OwnerSourceCardResolvedHasOtherTargetDcsbyingmouOncePerTurn
        }
        "dcsbyingmou_tracked_card_resolved" => TriggerFilter::DcsbyingmouTrackedCardResolved,
        "owner_hand_transfer_other_dcsbmengmou_branch_available" => {
            TriggerFilter::OwnerHandTransferOtherDcsbmengmouBranchAvailable
        }
        "dcsbmengmou_tracked_sha_damage" => TriggerFilter::DcsbmengmouTrackedShaDamage,
        "dcsbmengmou_tracked_sha_resolved" => TriggerFilter::DcsbmengmouTrackedShaResolved,
        "owner_is_target_card_damage_before" => TriggerFilter::OwnerIsTargetCardDamageBefore,
        "owner_source_single_target_sha_or_trick_all_materials_from_hand_can_sha" => {
            TriggerFilter::OwnerSourceSingleTargetShaOrTrickAllMaterialsFromHandCanSha
        }
        "dcyinjun_tracked_sha_resolved" => TriggerFilter::DcyinjunTrackedShaResolved,
        "owner_wounded_not_event_source" => TriggerFilter::OwnerWoundedNotEventSource,
        "owner_wounded_event_player_qun" => {
            TriggerFilter::OwnerWoundedEventPlayerGroup("qun".into())
        }
        "owner_alive_not_target" => TriggerFilter::OwnerAliveNotEventPlayer,
        "owner_alive_not_target_hand_equip_nonempty" => {
            TriggerFilter::OwnerAliveNotEventPlayerHandEquipNonempty
        }
        "owner_source_sha_or_ordinary_trick_target_wounded_has_discardable_hand_equip" => {
            TriggerFilter::OwnerSourceShaOrOrdinaryTrickTargetWoundedHasDiscardableHandEquip
        }
        "owner_source_sha_or_ordinary_trick_has_discardable_hand_equip" => {
            TriggerFilter::OwnerSourceShaOrOrdinaryTrickHasDiscardableHandEquip
        }
        "owner_source_sha_or_ordinary_trick" => TriggerFilter::OwnerSourceShaOrOrdinaryTrick,
        "owner_source_wuxie_used" => TriggerFilter::OwnerSourceWuxieUsed,
        "owner_source_sha_once_per_turn" => TriggerFilter::OwnerSourceShaOncePerTurn,
        "owner_is_target_hej_above_hp" => TriggerFilter::OwnerIsEventPlayerHandEquipJudgeAboveHp,
        "owner_source_play_card_repeats_previous_targets" => {
            TriggerFilter::OwnerSourcePlayCardRepeatsPreviousTargets
        }
        "owner_source_sha_repeats_previous_sha_targets" => {
            TriggerFilter::OwnerSourceShaRepeatsPreviousShaTargets
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
        "owner_is_target_current_turn" => TriggerFilter::OwnerIsEventPlayerCurrentTurn,
        "owner_alive_group_count_at_most_2" => TriggerFilter::OwnerAliveGroupCountAtMostTwo,
        "debate_owner_is_initiator" => TriggerFilter::DebateOwnerIsInitiator,
        "debate_owner_participated" => TriggerFilter::DebateOwnerParticipated,
        "owner_debate_participant" => TriggerFilter::OwnerDebateParticipant,
        "owner_cards_moved_contain_ying" => TriggerFilter::OwnerCardsMovedContainYing,
        "owner_is_lord" => TriggerFilter::OwnerIsLord,
        "owner_alive_other_targeted_by_sha" => TriggerFilter::OwnerAliveOtherTargetedBySha,
        "owner_alive_event_player_lord" => TriggerFilter::OwnerAliveEventPlayerLord,
        "owner_alive_event_player_lord_no_damage" => {
            TriggerFilter::OwnerAliveEventPlayerLordNoDamage
        }
        "owner_alive_event_player_lord_damaged" => TriggerFilter::OwnerAliveEventPlayerLordDamaged,
        "owner_alive_turn_used_all_suits" => TriggerFilter::OwnerAliveTurnUsedAllSuits,
        "cards_moved_discard_at_least_three" => TriggerFilter::CardsMovedDiscardAtLeastThree,
        "owner_alive_targeted_by_other_sha_judge_xumou" => {
            TriggerFilter::OwnerAliveTargetedByOtherShaJudgeXumou
        }
        "debate_owner_opinion_differs_initiator" => {
            TriggerFilter::DebateOwnerOpinionDiffersInitiator
        }
        "debate_owner_is_initiator_result_red" => TriggerFilter::DebateOwnerInitiatorResultRed,
        "owner_alive_other_source_damage" => TriggerFilter::OwnerAliveOtherSourceDamage,
        "debate_owner_is_initiator_result_black" => TriggerFilter::DebateOwnerInitiatorResultBlack,
        "owner_lost_package_card_from_equip" => TriggerFilter::OwnerLostPackageCardFromEquip,
        "owner_alive_current_turn_other_event_player_from_hand_empty" => {
            TriggerFilter::OwnerAliveCurrentTurnOtherEventPlayerFromHandEmpty
        }
        "owner_gains_multiple_outside_own_draw" => TriggerFilter::OwnerGainsMultipleOutsideOwnDraw,
        "owner_gained_or_lost_at_least_two" => TriggerFilter::OwnerGainedOrLostAtLeastTwo,
        "owner_alive_event_player_play_phase_same_suit_at_least_two_once_per_turn" => {
            TriggerFilter::OwnerAliveEventPlayerPlayPhaseSameSuitAtLeastTwoOncePerTurn
        }
        "owner_is_event_player_discard_pile_four_suits_below_alive_cap" => {
            TriggerFilter::OwnerIsEventPlayerDiscardPileFourSuitsBelowAliveCap
        }
        "owner_card_responded_to_by_player_in_range" => {
            TriggerFilter::OwnerCardRespondedToByPlayerInRange
        }
        "owner_alive_first_cards_moved_to_hand" => TriggerFilter::OwnerAliveFirstCardsMovedToHand,
        "owner_draws_during_own_play_not_self" => TriggerFilter::OwnerDrawsDuringOwnPlayNotSelf,
        "owner_source_non_equip_no_basic_hand_has_targets" => {
            TriggerFilter::OwnerSourceNonEquipNoBasicHandHasTargets
        }
        "owner_equip_zone_changed_alive_wounded_at_least_two" => {
            TriggerFilter::OwnerEquipZoneChangedAtLeastTwoWounded
        }
        "owner_current_turn_other_equipment_lost" => {
            TriggerFilter::OwnerCurrentTurnOtherEquipmentLost
        }
        "owner_alive_other_discarded_basic_outside_turn" => {
            TriggerFilter::OwnerAliveOtherDiscardedBasicOutsideTurn
        }
        "owner_alive_other_discarded_basic_outside_turn_once" => {
            TriggerFilter::OwnerAliveOtherDiscardedBasicOutsideTurnOncePerTurn
        }
        "owner_first_hand_discard_has_sha_in_discard" => {
            TriggerFilter::OwnerFirstHandDiscardHasShaInDiscard
        }
        "owner_alive_other_discarded_black_to_discard" => {
            TriggerFilter::OwnerAliveOtherDiscardedBlackToDiscard
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
        "owner_alive_source_alive_once_per_round" => {
            TriggerFilter::OwnerAliveSourceAliveOncePerRound
        }
        "owner_alive_source_alive_event_player_dying_once_per_round" => {
            TriggerFilter::OwnerAliveSourceAliveEventPlayerDyingOncePerRound
        }
        "owner_alive_once_per_round" => TriggerFilter::OwnerAliveOncePerRound,
        "owner_alive_not_target_once" => TriggerFilter::OwnerAliveNotEventPlayerOncePerTurn,
        "owner_alive_current_or_turn_fires_below_hp" => {
            TriggerFilter::OwnerAliveCurrentOrTurnFiresBelowHp
        }
        "owner_alive_card_is_sha" => TriggerFilter::OwnerAliveCardIsSha,
        "owner_alive_other_responded_sha_gainable" => {
            TriggerFilter::OwnerAliveOtherRespondedShaGainable
        }
        "owner_alive_card_targets_gt_one" => TriggerFilter::OwnerAliveCardTargetsGtOne,
        "owner_alive_non_multitarget_trick_targets_gt_one" => {
            TriggerFilter::OwnerAliveNonMultitargetTrickTargetsGtOne
        }
        "owner_alive_wounded_card_targets_gt_one" => {
            TriggerFilter::OwnerAliveWoundedCardTargetsGtOne
        }
        "owner_source_sha_or_trick_current_play_phase" => {
            TriggerFilter::OwnerSourceShaOrTrickCurrentPlayPhase
        }
        "owner_source_sha_current_play_phase" => TriggerFilter::OwnerSourceShaCurrentPlayPhase,
        "owner_source_card_has_other_target_current_play_phase_first_two" => {
            TriggerFilter::OwnerSourceCardHasOtherTargetCurrentPlayPhaseFirstTwo
        }
        "owner_alive_first_target_virtual_or_converted_card" => {
            TriggerFilter::OwnerAliveFirstTargetVirtualOrConvertedCard
        }
        "owner_alive_targeted_by_other_single_card_discardable" => {
            TriggerFilter::OwnerAliveTargetedByOtherSingleCardDiscardable
        }
        "owner_alive_targeted_by_trick_single_or_discardable" => {
            TriggerFilter::OwnerAliveTargetedByTrickSingleOrDiscardable
        }
        "owner_alive_targeted_by_other_trick" => TriggerFilter::OwnerAliveTargetedByOtherTrick,
        "owner_alive_targeted_by_other_matching_equip_suit" => {
            TriggerFilter::OwnerAliveTargetedByOtherMatchingEquipSuit
        }
        "owner_alive_targeted_by_other" => TriggerFilter::OwnerAliveTargetedByOther,
        "owner_alive_targeted_by_other_black_card" => {
            TriggerFilter::OwnerAliveTargetedByOtherBlackCard
        }
        "owner_alive_targeted_by_other_red_card" => TriggerFilter::OwnerAliveTargetedByOtherRedCard,
        "owner_alive_other_source_death" => TriggerFilter::OwnerAliveOtherSourceDeath,
        "owner_is_target_play_phase_hand_no_sha" => TriggerFilter::OwnerIsTargetPlayPhaseHandNoSha,
        "owner_alive_other_phase_end_player_outside_owner_range" => {
            TriggerFilter::OwnerAliveOtherPhaseEndPlayerOutsideOwnerRange
        }
        "owner_source_sha_target_alive_has_discardable_equip" => {
            TriggerFilter::OwnerSourceShaTargetAliveHasDiscardableEquip
        }
        "owner_source_card_used_targets_self" => TriggerFilter::OwnerSourceCardUsedTargetsSelf,
        "owner_source_current_phase_card_targeted_other_healthy" => {
            TriggerFilter::OwnerSourceCurrentPhaseCardTargetedOtherHealthy
        }
        "owner_min_hp" => TriggerFilter::OwnerMinHp,
        "owner_is_target_not_min_hp" => TriggerFilter::OwnerIsEventPlayerNotMinHp,
        "owner_target_min_hand" => TriggerFilter::OwnerIsEventPlayerMinHand,
        "owner_target_not_min_hand" => TriggerFilter::OwnerIsEventPlayerNotMinHand,
        "owner_sha_used_at_least_two" => TriggerFilter::OwnerShaUsedAtLeastTwo,
        "owner_alive_used_sha_this_turn" => TriggerFilter::OwnerAliveUsedShaThisTurn,
        "owner_max_hp_gt_one_hand_or_equip_empty" => TriggerFilter::OwnerMaxHpGtOneHandOrEquipEmpty,
        "owner_hand_empty" | "hand_empty" => TriggerFilter::OwnerHandEmpty,
        "owner_turned_over" => TriggerFilter::OwnerTurnedOver,
        "owner_alive_other_wei_recovery_own_phase_once_per_turn" => {
            TriggerFilter::OwnerAliveOtherWeiRecoveryOwnPhaseOncePerTurn
        }
        "owner_alive_global_juedou_or_red_sha_used" => {
            TriggerFilter::OwnerAliveGlobalJuedouOrRedShaUsed
        }
        "owner_alive_other_global_juedou_or_red_sha_used" => {
            TriggerFilter::OwnerAliveOtherGlobalJuedouOrRedShaUsed
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
        "owner_targeted_by_trick_equip_empty" => TriggerFilter::OwnerTargetedByTrickEquipEmpty,
        "owner_targeted_by_sha_hand_below_hp" => TriggerFilter::OwnerTargetedByShaHandBelowHp,
        "owner_target_physical_sha_source_hp_gt_owner" => {
            TriggerFilter::OwnerTargetPhysicalShaSourceHpGtOwner
        }
        "owner_source_other_target_sha_or_ordinary_trick_twguoyi_condition" => {
            TriggerFilter::OwnerSourceOtherTargetShaOrOrdinaryTrickTwguoyiCondition
        }
        "owner_alive_phase_end_caused_two_distinct_dying" => {
            TriggerFilter::OwnerAlivePhaseEndCausedTwoDistinctDying
        }
        "owner_gained_event_cards_from_other_personal_zone" => {
            TriggerFilter::OwnerGainedEventCardsFromOtherPersonalZone
        }
        "owner_alive_other_single_target_used_tracked_twyizhu" => {
            TriggerFilter::OwnerAliveOtherSingleTargetUsedTrackedTwyizhu
        }
        "owner_tracked_twyizhu_entered_discard" => TriggerFilter::OwnerTrackedTwyizhuEnteredDiscard,
        "owner_alive_other_twgonghuan_damage_before_hp_lte_owner_unused" => {
            TriggerFilter::OwnerAliveOtherTwgonghuanDamageBeforeHpLteOwnerUnused
        }
        "owner_phase_begin_playable_skill_pile_nonempty" => {
            TriggerFilter::OwnerPhaseBeginPlayableSkillPileNonempty
        }
        "owner_used_playable_skill_pile_card_unused" => {
            TriggerFilter::OwnerUsedPlayableSkillPileCardUnused
        }
        "owner_target_damage_before_playable_skill_piles_empty" => {
            TriggerFilter::OwnerTargetDamageBeforePlayableSkillPilesEmpty
        }
        "owner_alive_damage_before_target_min_hp_unused" => {
            TriggerFilter::OwnerAliveDamageBeforeTargetMinHpUnused
        }
        "owner_alive_global_phase_end" => TriggerFilter::OwnerAliveGlobalPhaseEnd,
        "owner_alive_other_phase_end_mobiledanshou" => {
            TriggerFilter::OwnerAliveOtherPhaseEndMobiledanshou
        }
        "owner_alive_other_phase_end_not_damaged_owner_this_turn" => {
            TriggerFilter::OwnerAliveOtherPhaseEndNotDamagedOwnerThisTurn
        }
        "owner_alive_other_phase_end_used_card_on_owner_undamaged_this_turn" => {
            TriggerFilter::OwnerAliveOtherPhaseEndUsedCardOnOwnerUndamagedThisTurn
        }
        "owner_alive_other_source_damage_card_resolved_targeted_damaged" => {
            TriggerFilter::OwnerAliveOtherSourceDamageCardResolvedTargetedDamaged
        }
        "owner_alive_other_source_damage_card_resolved_targeted_not_damaged" => {
            TriggerFilter::OwnerAliveOtherSourceDamageCardResolvedTargetedNotDamaged
        }
        "owner_alive_other_dying_mbchenshe_available" => {
            TriggerFilter::OwnerAliveOtherDyingMbchensheAvailable
        }
        "owner_is_target_discardable_hand_equip_once_per_round" => {
            TriggerFilter::OwnerIsEventPlayerDiscardableHandEquipOncePerRound
        }
        "owner_alive_first_non_discard_phase_discard_mbrenxing_available" => {
            TriggerFilter::OwnerAliveFirstNonDiscardPhaseDiscardMbrenxingAvailable
        }
        "owner_alive_global_phase_end_lost_cards_target_other_alive" => {
            TriggerFilter::OwnerAliveGlobalPhaseEndLostCardsTargetOtherAlive
        }
        "owner_alive_global_phase_end_discarded_cards_available" => {
            TriggerFilter::OwnerAliveGlobalPhaseEndDiscardedCardsAvailable
        }
        "owner_source_active_dczhaowen_tracked_red" => {
            TriggerFilter::OwnerSourceActiveDczhaowenTrackedRed
        }
        "owner_is_target_event_card_jiu" => TriggerFilter::OwnerIsTargetEventCardJiu,
        "owner_is_target_other_source_black_dcjiudun_available" => {
            TriggerFilter::OwnerIsTargetOtherSourceBlackDcjiudunAvailable
        }
        "owner_alive_phase_end_used_card_no_damage" => {
            TriggerFilter::OwnerAlivePhaseEndUsedCardNoDamage
        }
        "owner_cards_entered_discard_not_use_unused" => {
            TriggerFilter::OwnerCardsEnteredDiscardNotUseUnused
        }
        "owner_targeted_by_sha_or_juedou_damaged_hp_le_source_once" => {
            TriggerFilter::OwnerTargetedByShaOrJuedouDamagedHpLeSourceOnce
        }
        "owner_targeted_outside_turn_by_sha_or_trick_hand_empty" => {
            TriggerFilter::OwnerTargetedOutsideTurnByShaOrTrickHandEmpty
        }
        "owner_targeted_hand_below_max_hp" => TriggerFilter::OwnerTargetedHandBelowMaxHp,
        "owner_targeted_by_other_single_target_trick" => {
            TriggerFilter::OwnerTargetedByOtherSingleTargetTrick
        }
        "owner_source_or_targeted_by_juedou_or_red_sha" => {
            TriggerFilter::OwnerSourceOrTargetedByJuedouOrRedSha
        }
        "owner_source_or_targeted_by_juedou" => TriggerFilter::OwnerSourceOrTargetedByJuedou,
        "owner_source_or_targeted_by_sha_equal_hp" => {
            TriggerFilter::OwnerSourceOrTargetedByShaEqualHp
        }
        "owner_source_or_targeted_by_other_diamond" => {
            TriggerFilter::OwnerSourceOrTargetedByOtherDiamond
        }
        "owner_source_red" => TriggerFilter::OwnerSourceRed,
        "owner_source_alternates_color" => TriggerFilter::OwnerSourceAlternatesColor,
        "owner_source_repeats_color_has_discardable_hand_equip" => {
            TriggerFilter::OwnerSourceRepeatsColorHasDiscardableHandEquip
        }
        "owner_alive_skill_bound_target_is_event_player" => {
            TriggerFilter::OwnerAliveSkillBoundTargetIsEventPlayer
        }
        "owner_is_target_source_group_qun_discardable_hand_equip_gte_two" => {
            TriggerFilter::OwnerIsEventPlayerSourceGroupQunDiscardableHandEquipGteTwo
        }
        "owner_source_matches_previous_suit_or_rank" => {
            TriggerFilter::OwnerSourceMatchesPreviousSuitOrRank
        }
        "owner_source_ordinary_trick" => TriggerFilter::OwnerSourceOrdinaryTrick,
        "owner_source_physical_ordinary_trick" => TriggerFilter::OwnerSourcePhysicalOrdinaryTrick,
        "owner_hand_below_lost_hp_after_relevant_change" => {
            TriggerFilter::OwnerHandBelowLostHpAfterRelevantChange
        }
        "owner_alive_source_first_card_ordinary_trick" => {
            TriggerFilter::OwnerAliveSourceFirstCardOrdinaryTrick
        }
        "owner_source_matches_last_used_suit_or_rank" => {
            TriggerFilter::OwnerSourceMatchesLastUsedSuitOrRank
        }
        "owner_alive_equip_source_max_equip" => TriggerFilter::OwnerAliveEquipSourceMaxEquip,
        "owner_alive_other_current_source_repeats_card_name" => {
            TriggerFilter::OwnerAliveOtherCurrentSourceRepeatsCardName
        }
        "owner_source_hand_basic_equals_non_basic" => {
            TriggerFilter::OwnerSourceHandBasicEqualsNonBasic
        }
        "owner_is_target_event_card_sha" => TriggerFilter::OwnerIsEventPlayerEventCardSha,
        "owner_is_target_sha_or_trick" => TriggerFilter::OwnerIsEventPlayerEventCardShaOrTrick,
        "owner_source_or_within_distance_one_sha_or_juedou" => {
            TriggerFilter::OwnerSourceOrWithinDistanceOneShaOrJuedou
        }
        "owner_source_target_earlier_seat_hand_below_target_max_hp_once_per_turn" => {
            TriggerFilter::OwnerSourceTargetEarlierSeatHandBelowTargetMaxHpOncePerTurn
        }
        "owner_source_or_targeted_club_linked_exists_once_per_turn" => {
            TriggerFilter::OwnerSourceOrTargetedClubLinkedExistsOncePerTurn
        }
        "owner_source_sha_single_trick_source_card" => {
            TriggerFilter::OwnerSourceShaSingleTrickSourceCard
        }
        "owner_source_sha_single_equip_source_card" => {
            TriggerFilter::OwnerSourceShaSingleEquipSourceCard
        }
        "owner_is_target_hand_above_hp_no_liubei_lord" => {
            TriggerFilter::OwnerIsEventPlayerHandAboveHpNoLiubeiLord
        }
        "owner_alive_other_dying_first_per_target" => {
            TriggerFilter::OwnerAliveOtherDyingFirstPerTarget
        }
        "owner_is_target_trick_used_zero" => TriggerFilter::OwnerIsEventPlayerTrickUsedZero,
        "owner_is_target_hp_le_one" => TriggerFilter::OwnerIsEventPlayerHpLeOne,
        "owner_is_target_hp_le_two" => TriggerFilter::OwnerIsEventPlayerHpLeTwo,
        "owner_source_or_target_second_damage_this_turn" => {
            TriggerFilter::OwnerSourceOrTargetEventPlayerSecondDamageThisTurn
        }
        "owner_source_or_target_first_damage_this_turn" => {
            TriggerFilter::OwnerSourceOrTargetEventPlayerFirstDamageThisTurn
        }
        "owner_source_target_distance_le_one_first_damage_to_target_this_turn" => {
            TriggerFilter::OwnerSourceTargetDistanceLeOneFirstDamageToTargetThisTurn
        }
        "owner_first_transformed_sha_used_this_turn" => {
            TriggerFilter::OwnerFirstTransformedShaUsedThisTurn
        }
        "owner_target_hand_empty_elemental_or_trick_damage_before" => {
            TriggerFilter::OwnerTargetHandEmptyElementalOrTrickDamageBefore
        }
        "owner_alive_draw_player_owner_or_min_hp_twjiaohua_available" => {
            TriggerFilter::OwnerAliveDrawPlayerOwnerOrMinHpTwjiaohuaAvailable
        }
        "owner_source_sha" => TriggerFilter::OwnerSourceSha,
        "owner_target_sha" => TriggerFilter::OwnerTargetSha,
        "owner_source_heart_sha" => TriggerFilter::OwnerSourceHeartSha,
        "owner_source_sha_first_target" => TriggerFilter::OwnerSourceShaFirstTarget,
        "owner_alive_other_single_target_juedou_or_red_sha" => {
            TriggerFilter::OwnerAliveOtherSingleTargetJuedouOrRedSha
        }
        "owner_alive_hand_gained_this_turn_can_compare_other" => {
            TriggerFilter::OwnerAliveGainedThisTurnCanCompareOther
        }
        "owner_source_sha_or_juedou_single_target" => {
            TriggerFilter::OwnerSourceShaOrJuedouSingleTarget
        }
        "owner_source_sha_damage_recorded_by_skill" => {
            TriggerFilter::OwnerSourceShaDamageRecordedBySkill
        }
        "owner_source_normal_sha" => TriggerFilter::OwnerSourceNormalSha,
        "owner_source_fire_sha" => TriggerFilter::OwnerSourceFireSha,
        "owner_is_target_hand_sha_skill_pile_empty" => {
            TriggerFilter::OwnerIsEventPlayerHandShaSkillPileEmpty
        }
        "owner_alive_dying_has_skill_pile_cards" => TriggerFilter::OwnerAliveDyingHasSkillPileCards,
        "owner_source_single_target_sha_or_ordinary_trick_current_turn" => {
            TriggerFilter::OwnerSourceSingleTargetShaOrOrdinaryTrickCurrentTurn
        }
        "owner_source_event_card_policy_draw" => TriggerFilter::OwnerSourceEventCardPolicyDraw,
        "owner_source_card_resolved_phase_policy" => {
            TriggerFilter::OwnerSourceCardResolvedPhasePolicy
        }
        "owner_phase_play_end_potguansha_available_once_per_game" => {
            TriggerFilter::OwnerPhasePlayEndPotguanshaAvailableOncePerGame
        }
        "owner_discarded_sha_to_discard" => TriggerFilter::OwnerDiscardedShaToDiscard,
        "owner_alive_other_turn_discarded_recorded_potfuan_suit" => {
            TriggerFilter::OwnerAliveOtherTurnDiscardedRecordedPotfuanSuit
        }
        "owner_source_sha_resolved_potfuan_dealt_damage" => {
            TriggerFilter::OwnerSourceShaResolvedPotfuanDealtDamage
        }
        "owner_discarded_black_basic_to_discard" => {
            TriggerFilter::OwnerDiscardedBlackBasicToDiscard
        }
        "owner_alive_discarded_black_basic_to_discard" => {
            TriggerFilter::OwnerAliveDiscardedBlackBasicToDiscard
        }
        "owner_source_card_resolved_ddd_xuyu_pending" => {
            TriggerFilter::OwnerSourceCardResolvedDddXuyuPending
        }
        "ddd_xuyu_pending" => TriggerFilter::DddXuyuPending,
        "owner_alive_other_source_second_play_card_resolved_hand_equip_nonempty" => {
            TriggerFilter::OwnerAliveOtherSourceSecondPlayCardResolvedHandEquipNonempty
        }
        "ddd_shijian_watched_source_card_used_targeted" => {
            TriggerFilter::DddShijianWatchedSourceCardUsedTargeted
        }
        "ddd_shijian_pending" => TriggerFilter::DddShijianPending,
        "owner_alive_hp_changed_this_turn_entered_discard_available" => {
            TriggerFilter::OwnerAliveHpChangedThisTurnEnteredDiscardAvailable
        }
        "owner_alive_other_phase_begin_both_hand_nonempty" => {
            TriggerFilter::OwnerAliveOtherPhaseBeginBothHandNonempty
        }
        "owner_alive_ddd_dongcha_pile_nonempty" => TriggerFilter::OwnerAliveDddDongchaPileNonempty,
        "owner_alive_other_phase_begin_unshown_hand_gte_hp" => {
            TriggerFilter::OwnerAliveOtherPhaseBeginUnshownHandGteHp
        }
        "owner_play_dddzhengjun_hp_available" => TriggerFilter::OwnerPlayDddZhengjunHpAvailable,
        "owner_play_dddzhengjun_cards_moved_available" => {
            TriggerFilter::OwnerPlayDddZhengjunCardsMovedAvailable
        }
        "owner_alive_other_phase_play_dddchashi_available" => {
            TriggerFilter::OwnerAliveOtherPhasePlayDddChashiAvailable
        }
        "dddchashi_matching_card" => TriggerFilter::DddChashiMatchingCard,
        "owner_alive_dddqice_ready" => TriggerFilter::OwnerAliveDddQiceReady,
        "owner_alive_ddd_fenji_available" => TriggerFilter::OwnerAliveDddFenjiAvailable,
        "owner_alive_ddd_fengzheng_round_pending" => {
            TriggerFilter::OwnerAliveDddFengzhengRoundPending
        }
        "owner_alive_ddd_yulv_discard_matches" => TriggerFilter::OwnerAliveDddYulvDiscardMatches,
        "owner_alive_ddd_youxue_round_pending" => TriggerFilter::OwnerAliveDddYouxueRoundPending,
        "owner_alive_ddd_chengjing_available" => TriggerFilter::OwnerAliveDddChengjingAvailable,
        "owner_alive_ddd_zhuanshe_gift" => TriggerFilter::OwnerAliveDddZhuansheGift,
        "owner_alive_ddd_zhuanshe_card_used" => TriggerFilter::OwnerAliveDddZhuansheCardUsed,
        "owner_alive_ddd_zhuanshe_phase_end" => TriggerFilter::OwnerAliveDddZhuanshePhaseEnd,
        "owner_alive_ddd_weiqiu" => TriggerFilter::OwnerAliveDddWeiqiu,
        "owner_alive_ddd_lianer_available" => TriggerFilter::OwnerAliveDddLianerAvailable,
        "owner_alive_ddd_lianer_ceiling" => TriggerFilter::OwnerAliveDddLianerCeiling,
        "owner_alive_ddd_anzhi_available" => TriggerFilter::OwnerAliveDddAnzhiAvailable,
        "owner_alive_ddd_anzhi_tracking" => TriggerFilter::OwnerAliveDddAnzhiTracking,
        "owner_alive_ddd_anzhi_target_black" => TriggerFilter::OwnerAliveDddAnzhiTargetBlack,
        "owner_alive_ddd_anzhi_turn_end" => TriggerFilter::OwnerAliveDddAnzhiTurnEnd,
        "owner_alive_ddd_shixing_turn_refresh" => TriggerFilter::OwnerAliveDddShixingTurnRefresh,
        "owner_alive_ddd_shixing_female_changed" => {
            TriggerFilter::OwnerAliveDddShixingFemaleChanged
        }
        "owner_alive_ddd_shixing_first_trick" => TriggerFilter::OwnerAliveDddShixingFirstTrick,
        "owner_alive_ddd_danggu_available" => TriggerFilter::OwnerAliveDddDangguAvailable,
        "owner_alive_ddd_fuzong_death" => TriggerFilter::OwnerAliveDddFuzongDeath,
        "owner_is_target_face_down" => TriggerFilter::OwnerIsTargetFaceDown,
        "owner_alive_ddd_xuanlun_pending" => TriggerFilter::OwnerAliveDddXuanlunPending,
        "owner_alive_ddd_jiexing_recover_once_per_game" => {
            TriggerFilter::OwnerAliveDddJiexingRecoverOncePerGame
        }
        "owner_alive_ddd_jiexing_equip_once_per_game" => {
            TriggerFilter::OwnerAliveDddJiexingEquipOncePerGame
        }
        "owner_alive_ddd_jiexing_recovery_reset" => {
            TriggerFilter::OwnerAliveDddJiexingRecoveryReset
        }
        "owner_alive_ddd_jiexing_equip_reset" => TriggerFilter::OwnerAliveDddJiexingEquipReset,
        "owner_alive_ddd_bailei_available" => TriggerFilter::OwnerAliveDddBaileiAvailable,
        "owner_source_own_view_as_ddd_tongyu_used" => {
            TriggerFilter::OwnerSourceOwnViewAsDddTongyuUsed
        }
        "owner_source_own_view_as_ddd_tongyu_targeted" => {
            TriggerFilter::OwnerSourceOwnViewAsDddTongyuTargeted
        }
        "owner_alive_ddd_tongyu_remained" => TriggerFilter::OwnerAliveDddTongyuRemained,
        "dddchashi_pending_phase_play_end" => TriggerFilter::DddChashiPendingPhasePlayEnd,
        "owner_alive_dddzhilian_available" => TriggerFilter::OwnerAliveDddZhilianAvailable,
        "owner_alive_source_basic_dddfushi_available" => {
            TriggerFilter::OwnerAliveSourceBasicDddFushiAvailable
        }
        "owner_alive_other_judge_phase_in_range_has_judge_card" => {
            TriggerFilter::OwnerAliveOtherJudgePhaseInRangeHasJudgeCard
        }
        "owner_source_play_sha_other_alive_once_per_play_phase" => {
            TriggerFilter::OwnerSourcePlayShaOtherAliveOncePerPlayPhase
        }
        "owner_source_jiu" => TriggerFilter::OwnerSourceJiu,
        "owner_is_target_hp_odd" => TriggerFilter::OwnerIsEventPlayerHpOdd,
        "owner_is_target_hp_even_wounded" => TriggerFilter::OwnerIsEventPlayerHpEvenWounded,
        "owner_source_trick" | "card_is_trick" => TriggerFilter::OwnerSourceTrick,
        "once_per_turn" => TriggerFilter::OncePerTurn,
        "owner_source_once" => TriggerFilter::OwnerSourceOncePerTurn,
        "owner_source_once_per_turn" => TriggerFilter::OwnerSourceOncePerTurn,
        "owner_source_target_alive_has_equip" => TriggerFilter::OwnerSourceTargetAliveHasEquip,
        "owner_source_black_sha_target_alive_has_discardable_equip" => {
            TriggerFilter::OwnerSourceBlackShaTargetAliveHasDiscardableEquip
        }
        "owner_source_sha_damage_target_alive_has_discardable_equip" => {
            TriggerFilter::OwnerSourceShaDamageTargetAliveHasDiscardableEquip
        }
        "owner_source_equip_used_skill_mark_below_seven" => {
            TriggerFilter::OwnerSourceEquipUsedSkillMarkBelowSeven
        }
        "owner_source_outside_turn_card_used_or_responded" => {
            TriggerFilter::OwnerSourceOutsideTurnCardUsedOrResponded
        }
        "owner_alive_shan_used_by_owner_or_during_owner_turn" => {
            TriggerFilter::OwnerAliveShanUsedByOwnerOrDuringOwnerTurn
        }
        "owner_alive_other_phase_play_skill_pile_nonempty" => {
            TriggerFilter::OwnerAliveOtherPhasePlaySkillPileNonempty
        }
        "owner_source_sha_target_alive_has_discardable_region_card" => {
            TriggerFilter::OwnerSourceShaTargetAliveHasDiscardableRegionCard
        }
        "owner_source_sha_target_alive_has_discardable_hand_equip" => {
            TriggerFilter::OwnerSourceShaTargetAliveHasDiscardableHandEquip
        }
        "owner_source_tiesuo_target_alive_unlinked_hand_equip_nonempty" => {
            TriggerFilter::OwnerSourceTiesuoTargetAliveUnlinkedHandEquipNonempty
        }
        "owner_alive_source_target_was_linked" => TriggerFilter::OwnerAliveSourceTargetWasLinked,
        "owner_source_sha_target_alive_hand_nonempty" => {
            TriggerFilter::OwnerSourceShaTargetAliveHandNonempty
        }
        "owner_source_sha_target_hand_gte_hp_or_lte_attack_range" => {
            TriggerFilter::OwnerSourceShaTargetHandGteHpOrLteAttackRange
        }
        "owner_source_sha_target_hand_lte_owner_or_hp_gte_owner" => {
            TriggerFilter::OwnerSourceShaTargetHandLteOwnerOrHpGteOwner
        }
        "owner_is_target_judge_nonempty" => TriggerFilter::OwnerIsEventPlayerJudgeNonempty,
        "owner_is_target_draw_not_replaced" => TriggerFilter::OwnerIsEventPlayerDrawNotReplaced,
        "owner_is_target_draw_not_fixed_hp_one_or_hand_empty" => {
            TriggerFilter::OwnerIsEventPlayerDrawNotFixedHpOneOrHandEmpty
        }
        "owner_source_sha_dodged_target_alive_has_discardable_hand_equip" => {
            TriggerFilter::OwnerSourceShaDodgedTargetAliveHasDiscardableHandEquip
        }
        "owner_source_sha_dodged_target_recorded_by_skill_has_discardable_hand_equip" => {
            TriggerFilter::OwnerSourceShaDodgedTargetRecordedBySkillHasDiscardableHandEquip
        }
        "owner_source_sha_dodged" => TriggerFilter::OwnerSourceShaDodged,
        "owner_source_sha_target_alive_distance_le_one" => {
            TriggerFilter::OwnerSourceShaTargetAliveDistanceLeOne
        }
        "owner_alive_sha_target_distance_le_one" => TriggerFilter::OwnerAliveShaTargetDistanceLeOne,
        "owner_alive_sha_target_distance_le_one_has_equip_card" => {
            TriggerFilter::OwnerAliveShaTargetDistanceLeOneHasEquipCard
        }
        "owner_is_target_sha_source_alive_hand_equip_nonempty" => {
            TriggerFilter::OwnerIsEventPlayerShaSourceAliveHandEquipNonempty
        }
        "owner_alive_wdb_fuchou_pending_living" => TriggerFilter::OwnerAliveWdbFuchouPendingLiving,
        "owner_alive_other_gained_single_equip_to_hand_outside_turn_usable" => {
            TriggerFilter::OwnerAliveOtherGainedSingleEquipToHandOutsideTurnUsable
        }
        "owner_alive_wugu_remained_in_discard" => TriggerFilter::OwnerAliveWuguRemainedInDiscard,
        "owner_is_target_non_multitarget_trick_multiple_targets" => {
            TriggerFilter::OwnerIsTargetNonMultitargetTrickMultipleTargets
        }
        "owner_source_play_phase_sha_has_additional_target_ignoring_distance" => {
            TriggerFilter::OwnerSourcePlayPhaseShaHasAdditionalTargetIgnoringDistance
        }
        "owner_source_card_resolved_tracked_dddxianxi" => {
            TriggerFilter::OwnerSourceCardResolvedTrackedDddXianxi
        }
        "owner_source_card_damage_target_can_receive_owner_bingliang" => {
            TriggerFilter::OwnerSourceCardDamageTargetCanReceiveOwnerBingliang
        }
        "owner_alive_other_dying_hand_nonempty_once_per_game" => {
            TriggerFilter::OwnerAliveOtherDyingHandNonemptyOncePerGame
        }
        "owner_dying_other_hand_equip_nonempty_once_per_game" => {
            TriggerFilter::OwnerDyingOtherHandEquipNonemptyOncePerGame
        }
        "owner_alive_other_dying_once_per_round" => TriggerFilter::OwnerAliveOtherDyingOncePerRound,
        "owner_alive_sha_or_duel_damage_source_target_alive_hand_equip_nonempty" => {
            TriggerFilter::OwnerAliveShaOrDuelDamageSourceTargetAliveHandEquipNonempty
        }
        "owner_alive_sha_damage_source_target_alive_owner_hand_equip_nonempty" => {
            TriggerFilter::OwnerAliveShaDamageSourceTargetAliveOwnerHandEquipNonempty
        }
        "owner_alive_source_ally_sha_damage_before" => {
            TriggerFilter::OwnerAliveSourceAllyShaDamageBefore
        }
        "owner_is_target_wounded_dead_ally" => TriggerFilter::OwnerIsEventPlayerWoundedDeadAlly,
        "owner_is_target_any_alive_self_discardable_hand_equip" => {
            TriggerFilter::OwnerIsEventPlayerAnyAliveSelfDiscardableHandEquip
        }
        "owner_is_target_any_alive_discardable_by_owner_hand_equip" => {
            TriggerFilter::OwnerIsEventPlayerAnyAliveDiscardableByOwnerHandEquip
        }
        "owner_is_target_any_alive_self_discardable_hand" => {
            TriggerFilter::OwnerIsEventPlayerAnyAliveSelfDiscardableHand
        }
        "owner_is_target_any_alive_wounded" => TriggerFilter::OwnerIsEventPlayerAnyAliveWounded,
        "owner_is_target_hand_equip_nonempty_other_alive" => {
            TriggerFilter::OwnerIsEventPlayerHandEquipNonemptyOtherAlive
        }
        "owner_is_target_phase_end_hand_equip_nonempty_other_alive" => {
            TriggerFilter::OwnerIsEventPlayerPhaseEndHandEquipNonemptyOtherAlive
        }
        "owner_alive_gained_three_this_turn_other_alive" => {
            TriggerFilter::OwnerAliveGainedThreeThisTurnOtherAlive
        }
        "owner_targeted_by_other_sha_once" => TriggerFilter::OwnerTargetedByOtherShaOncePerTurn,
        "owner_targeted_by_other_card_once_per_turn" => {
            TriggerFilter::OwnerTargetedByOtherCardOncePerTurn
        }
        "owner_twfeifu_single_sha_party_target_has_hand_equip" => {
            TriggerFilter::OwnerTwfeifuSingleShaPartyTargetHasHandEquip
        }
        "owner_targeted_by_other_sha" => TriggerFilter::OwnerTargetedByOtherSha,
        "owner_card_action_count_eq_attack_range" => {
            TriggerFilter::OwnerCardActionCountEqualsAttackRange
        }
        "owner_lifetime_card_action" => TriggerFilter::OwnerLifetimeCardAction,
        "owner_is_target_once" => TriggerFilter::OwnerIsEventPlayerOncePerTurn,
        "owner_source_healthy" => TriggerFilter::OwnerSourceHealthy,
        "owner_source_amount_gt_one" => TriggerFilter::OwnerSourceAmountGtOne,
        "owner_source_other_twice" => TriggerFilter::OwnerSourceOtherTwicePerTurn,
        "owner_alive_other_source_sha_twice" => TriggerFilter::OwnerAliveOtherSourceShaTwicePerTurn,
        "owner_has_enabled_equip_slot" => TriggerFilter::OwnerHasEnabledEquipSlot,
        "owner_is_event_player_no_enabled_equip_slot_once_per_game" => {
            TriggerFilter::OwnerIsEventPlayerNoEnabledEquipSlotOncePerGame
        }
        "owner_is_target_has_movable_equipment" => {
            TriggerFilter::OwnerIsEventPlayerHasMovableEquipment
        }
        "owner_alive_is_target_current_play_phase" => {
            TriggerFilter::OwnerAliveIsEventPlayerCurrentPlayPhase
        }
        "owner_alive_other_play_phase_pothuilv_available" => {
            TriggerFilter::OwnerAliveOtherPlayPhasePothuilvAvailable
        }
        "owner_is_target_can_use_wuzhong" => TriggerFilter::OwnerIsTargetCanUseWuzhong,
        "owner_alive_discard_batch" => TriggerFilter::OwnerAliveDiscardBatch,
        "owner_alive_other_responded_to_owner" => TriggerFilter::OwnerAliveOtherRespondedToOwner,
        "owner_source_damage_potjianzhuan_available" => {
            TriggerFilter::OwnerSourceDamagePotjianzhuanAvailable
        }
        "owner_target_damage_potjianzhuan_available" => {
            TriggerFilter::OwnerTargetDamagePotjianzhuanAvailable
        }
        "owner_alive_dying_potjianzhuan_available" => {
            TriggerFilter::OwnerAliveDyingPotjianzhuanAvailable
        }
        "owner_source_damage_potcaoshuang_assigned_untriggered" => {
            TriggerFilter::OwnerSourceDamagePotcaoshuangAssignedUntriggered
        }
        "owner_target_damage_potcaoshuang_assigned_untriggered" => {
            TriggerFilter::OwnerTargetDamagePotcaoshuangAssignedUntriggered
        }
        "owner_alive_dying_potcaoshuang_assigned_untriggered" => {
            TriggerFilter::OwnerAliveDyingPotcaoshuangAssignedUntriggered
        }
        "owner_used_potnizun_pile_card" => TriggerFilter::OwnerUsedPotnizunPileCard,
        "owner_target_damage_before_potnizun_used" => {
            TriggerFilter::OwnerTargetDamageBeforePotnizunUsed
        }
        "owner_alive_turn_ended_used_card_tagged_discard" => {
            TriggerFilter::OwnerAliveTurnEndedUsedCardTaggedDiscard
        }
        "always" => TriggerFilter::Always,
        _ => return None,
    })
}
