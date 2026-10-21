pub(in super::super) fn documentation(word: &str) -> Option<&'static str> {
    Some(match word {
            "owner_is_target_no_turn_damage" => {
                "Trigger filter: the skill owner is the event player and has dealt no damage during the current turn."
            }
            "owner_is_target_turn_damage" => {
                "Trigger filter: the skill owner is the event player and has dealt damage during the current turn."
            }
            "owner_is_target_turn_damage_at_least_three" => {
                "Trigger filter: the skill owner is the event player and has dealt at least three damage during the current turn."
            }
            "owner_is_target_card_used_at_least_hp" => {
                "Trigger filter: the skill owner is the event player and has proactively used at least as many cards as their current HP during this turn."
            }
            "owner_is_target_use_response_discarded_at_least_hp" => {
                "Trigger filter: the skill owner is the current turn's event player and at least current-HP physical cards used or played as responses by any player entered the discard pile this turn."
            }
            "owner_is_target_damaged_other_qun" => {
                "Trigger filter: the skill owner is the event player and damaged another Qun character during the current turn."
            }
            "owner_is_target_source_alive" => {
                "Trigger filter: owner is the event target and the event source is still alive."
            }
            "owner_is_target_source_alive_once" => {
                "Trigger filter: owner is the event target, the event source is still alive, and this skill has not fired during the current turn. Requires Rule VM ABI 165."
            }
            "owner_is_target_source_has_weapon" => {
                "Damage trigger filter: owner is the damaged player and an identified damage source currently has at least one weapon equipped. Requires Rule VM ABI 117."
            }
            "owner_is_target_source_alive_disabled_weapon" => {
                "Damage trigger filter: owner is the damaged player and the living damage source has at least one disabled weapon slot. Requires Rule VM ABI 161."
            }
            "owner_is_target_other_source_alive" => {
                "Trigger filter: owner is the event target and a different event source is still alive."
            }
            "owner_is_target_other_source_alive_per_damage_point" => {
                "Damage trigger filter: owner is the damaged player and a different source remains alive; the trigger is emitted once per damage point."
            }
            "owner_is_target_per_damage_point" => {
                "Damage trigger filter: owner is the damaged player; the trigger is emitted once per damage point, including self-sourced or source-free damage. Requires Rule VM ABI 84."
            }
            "owner_is_target_per_damage_point_other_hand_equip_judge_nonempty" => {
                "Damage trigger filter: the living owner is the damaged player and at least one other living player has a hand, equipment, or judgment card; the trigger is emitted once per damage point. Requires Rule VM ABI 206."
            }
            "owner_is_target_other_alive" => {
                "Trigger filter: owner is the event player and at least one other player remains alive. Requires Rule VM ABI 85."
            }
            "owner_is_target_hand_equip_nonempty_other_alive" => {
                "Damage trigger filter: the living owner is the damaged player of a positive damage event, owns at least one hand/equipment card, and at least one other player remains alive. Requires Rule VM ABI 138."
            }
            "owner_is_target_phase_end_hand_equip_nonempty_other_alive" => {
                "End-phase trigger filter: the living owner is the event player, owns at least one hand/equipment card, and at least one other player remains alive. Requires Rule VM ABI 280."
            }
            "owner_alive_gained_three_this_turn_other_alive" => {
                "Turn-end trigger filter: the living owner gained at least three physical cards during the current turn and another living player exists. Card gains are tracked only when loaded content needs this filter. Requires Rule VM ABI 139."
            }
            "owner_alive_event_player_hand_nonempty_once_per_round" => {
                "Play-phase-start trigger filter: the owner and phase player are alive, the phase player has at least one hand card, and this skill instance has not fired during the current round. Requires Rule VM ABI 140."
            }
            "owner_source_bound_category_first_three_play_phase" => {
                "Card-used trigger filter: the current Play-phase player matches this skill instance's bound target and the used card's effective basic/trick/equipment category matches the bound category; fires at most three times in that Play phase. Requires Rule VM ABI 140."
            }
            "owner_source_damage_card_once_per_turn" => {
                "Card-used trigger filter: the living owner used a logical Sha or damage trick and this skill instance has not fired during the current turn. Requires Rule VM ABI 141."
            }
            value if value.starts_with("owner_alive_other_judgment_black_event_player_group_") => {
                "Trigger filter: owner and another judged player are alive, the judged player has the requested group tag, and the final judgment result is black. Requires Rule VM ABI 86."
            }
            value if value.starts_with("owner_alive_other_damage_source_group_") => {
                "Trigger filter: a living owner observes damage dealt by another living player whose character group equals the suffix. Requires Rule VM ABI 88."
            }
            "owner_is_target_source_alive_gainable_card_per_damage_point" => {
                "Damage trigger filter: owner is the damaged player and the living source has a card the owner may take; emits once per damage point. Self-source exposes equipment only, another source exposes hand and equipment. Requires Rule VM ABI 22."
            }
            "owner_is_target_source_alive_gainable_card" => {
                "Damage trigger filter: owner is the damaged player and the living source has a card the owner may take; emits once for the damage event. Self-source exposes equipment only, another source exposes hand and equipment. Requires Rule VM ABI 169."
            }
            "owner_is_target_card_in_processing" => {
                "Damage trigger filter: owner is the damaged player and the current physical damage-card carrier is still in the processing zone. Requires Rule VM ABI 170."
            }
            "owner_is_target_red_sha_source_alive" => {
                "Trigger filter: owner is receiving card-linked damage from a red logical Sha whose source is alive."
            }
            "owner_is_target_nonred_sha" => {
                "Trigger filter: owner is receiving card-linked damage from a non-red logical Sha."
            }
            "owner_is_target_red_or_jiu_sha" => {
                "Trigger filter: owner received card-linked damage from a red logical Sha or a logical Sha boosted by Jiu."
            }
            "owner_is_target_red_card_source_alive" => {
                "Trigger filter: owner is receiving card-linked damage from a red card whose source is alive."
            }
            "owner_is_target_nonred_card" => {
                "Trigger filter: owner is receiving card-linked damage from a non-red card."
            }
            "owner_is_target_first_damage" => {
                "Trigger filter: owner is the damaged player and this skill has not handled damage earlier this turn."
            }
            "owner_is_target_second_damage" => {
                "Trigger filter: owner is the damaged player and this skill has handled exactly one damage event earlier this turn."
            }
            "owner_is_target_after_first_damage" => {
                "Trigger filter: owner is the damaged player and this skill already handled damage earlier this turn."
            }
            "owner_alive_source_target_same_sex" => {
                "Trigger filter: the skill owner is alive and the event source and target share a sex tag; `double` matches either known sex."
            }
            "owner_alive_event_player_same_group" => {
                "Trigger filter: the living skill owner and event player have the same non-empty character group tag."
            }
            "owner_alive_event_player_same_group_hand_nonempty" => {
                "Trigger filter: the living skill owner and event player have the same non-empty group tag, and the owner has at least one discardable hand card. Requires Rule VM ABI 596."
            }
            "owner_tracked_juedou_original_target_damage_recoverable_in_range" => {
                "Damage trigger filter: the owner dealt damage to the original target of this skill instance's exact tracked virtual Duel, and that target's attack range contains a wounded living character other than the owner. Requires Rule VM ABI 597."
            }
            "owner_is_target_phase_end_hand_nonempty_other_hand_nonempty" => {
                "End-phase trigger filter: the living owner is the phase owner, has hand cards, and another living character also has hand cards. Requires Rule VM ABI 598."
            }
            "owner_is_target_ordinary_trick_cilv_available" => {
                "Card-targeted filter: the owner is targeted by an ordinary trick and still has at least one Cilv option. Requires Rule VM ABI 600."
            }
            "cilv_tracked_card_damage_before" => {
                "Damage-before filter: the exact source card is tracked by this Cilv instance's damage-prevention option. Requires Rule VM ABI 600."
            }
            "cilv_tracked_card_resolved" => {
                "Card-resolved filter: the exact card has pending Cilv damage-prevention cleanup or material gain. Requires Rule VM ABI 600."
            }
            "owner_dying_zhupeilan_once_per_game" => {
                "Dying filter: the owner is dying and this Tongdao instance has not fired during the match. Requires Rule VM ABI 600."
            }
            "owner_alive_other_play_phase_tied_max_hand_dcsbfuxi_available" => {
                "Play-phase filter: another tied-most-hand player began their Play phase and Fuxi has a legal branch. Requires Rule VM ABI 601."
            }
            "owner_phase_end_dcsbhaoyi_available" => {
                "End-phase filter: the owner can gain a damage card that entered discard this turn without dealing damage. Requires Rule VM ABI 601."
            }
            "owner_alive_global_fire_damage_dcsbronghuo" => {
                "Damage filter: a living Ronghuo owner observes completed fire damage and arms one non-stacking permission. Requires Rule VM ABI 602."
            }
            "owner_source_fire_sha_or_huogong_dcsbronghuo" => {
                "Card-used filter: the owner uses a fire Sha or Fire Attack while Ronghuo's one-shot permission is armed. Requires Rule VM ABI 602."
            }
            "owner_source_card_resolved_has_other_target_dcsbyingmou_once_per_turn" => {
                "Card-resolved filter: once per turn, the living owner finished a card with at least one other original target and the current Yingmou branch has a legal selection. Requires Rule VM ABI 602."
            }
            "dcsbyingmou_tracked_card_resolved" => {
                "Card-resolved filter: the exact physical damage card currently sequenced by Yingmou finished resolving. Requires Rule VM ABI 602."
            }
            "owner_wounded_not_event_source" => {
                "Trigger filter: the living skill owner is wounded and is not the event source."
            }
            "owner_alive_event_player_group_wei" => {
                "Parameterized trigger filter: both players are alive and the event player's character group equals the suffix (`wei` here). Other canonical group tags may replace the suffix."
            }
            "owner_wounded_event_player_group_qun" => {
                "Parameterized trigger filter: the living skill owner is wounded and the event player's character group equals the suffix (`qun` here)."
            }
            "owner_wounded_event_player_qun" => {
                "Legacy spelling of `owner_wounded_event_player_group_qun`."
            }
            "owner_source_amount_gt_one" => {
                "Trigger filter: owner dealt damage whose amount is greater than one."
            }
            "owner_source_wounded_distance_le_one" => {
                "Trigger filter: owner is the wounded damage source and is within distance 1 of the target."
            }
            "owner_source_distance_le_one_per_damage_point" => {
                "Damage trigger filter: owner is the source and within distance 1 of the target; the trigger is offered once per damage point."
            }
            "owner_source_target_distance_gt_one" => {
                "Per-target card trigger filter: owner is the card source and the current declared target's final distance from the owner is greater than one."
            }
            "owner_source_target_not_group_wei" => {
                "Parameterized damage trigger filter: owner is the source and the damage target's character group differs from the suffix (`wei` here). Other canonical group tags may replace the suffix. Requires Rule VM ABI 498."
            }
            "owner_source_target_distance_le_one_first_damage_to_target_this_turn" => {
                "Damage trigger filter: the living owner is the source, the damaged player is within final distance one, and this is the owner's first applied damage event to that player this turn. Requires Rule VM ABI 353."
            }
            "owner_alive" => {
                "Trigger filter: fires only while the skill owner remains alive after the event."
            }
            "owner_alive_event_cards_nonempty" => {
                "Trigger filter: the skill owner is alive and the triggering event carries at least one captured physical card. Death events capture the victim's hand and equipment before cleanup. Requires Rule VM ABI 82."
            }
            "owner_alive_neighbor_gained_cards_outside_own_draw_once_per_turn" => {
                "Cards-moved filter: once per turn, a living previous/next-seat neighbor gained cards into hand outside their Draw phase and has a hand/equipment card the owner may gain. Requires Rule VM ABI 487."
            }
            "owner_first_hand_discard_has_sha_in_discard" => {
                "Cards-moved trigger filter: the living owner is discarding hand cards for the first time this turn, and at least one physical Sha from this event still remains in the shared discard pile. Requires Rule VM ABI 332."
            }
            "owner_alive_hand_below_alive_count_capped_seven" => {
                "Trigger filter: the living owner has fewer hand cards than the living-player count capped at seven."
            }
            "owner_alive_hand_below_alive_count_capped_eight" => {
                "Trigger filter: the living owner has fewer hand cards than the living-player count capped at eight. Requires Rule VM ABI 228."
            }
            "owner_is_target_first_hp_change_turn_unique_trick_majority" => {
                "HP-change trigger filter: the living owner is the event player on their first actual HP change this turn, and trick cards are the sole most numerous basic/trick/equipment category in their current hand. Supports damage, hp_lost, hp_recovered, and max_hp_changed. Requires Rule VM ABI 336."
            }
            "owner_alive_delayed_trick_target_without_pending_judge_skip" => {
                "Per-target trigger filter: the living owner is targeted by a delayed trick and has no pending judgment-phase skip. Requires Rule VM ABI 228."
            }
            "owner_source_sha_or_juedou_single_target_first_same_name_this_turn" => {
                "Per-target trigger filter: the living owner used their first logical Sha or first Duel of the current turn and specified exactly one target. Requires Rule VM ABI 229."
            }
            "owner_source_sha_damage_before_endpoint_discardable_hand_equip" => {
                "Damage-before trigger filter: the living owner is the source of logical-Sha damage to a living target and may currently discard a hand/equipment card from either endpoint. Chain-propagated Sha damage is included. Requires Rule VM ABI 230."
            }
            "owner_longdan_view_as_first_counterpart_hand_nonempty" => {
                "Card-targeted/CardResponded trigger filter: the living owner used or responded with a Sha/Shan converted by a Longdan/Fanghun-named skill, the current player is the first counterpart, and that counterpart has hand cards. Requires Rule VM ABI 231."
            }
            "owner_is_source_hand_below_alive_count_capped_seven" => {
                "Trigger filter: the owner is the event source and has fewer hand cards than the living-player count capped at seven."
            }
            "owner_alive_event_player_alive" => {
                "Trigger filter: the skill owner and event player are both alive."
            }
            "owner_alive_event_player_equip_empty" => {
                "Trigger filter: the skill owner and event player are both alive, and the event player's equipment zone is empty."
            }
            "owner_alive_event_player_from_equip_empty" => {
                "Cards-moved trigger filter: while the owner remains alive, a living event player actually lost one or more cards from their equipment zone and now has no equipment. Requires Rule VM ABI 28."
            }
            "owner_alive_sha_target_no_other_red_card_this_turn" => {
                "Per-target Sha trigger filter: while the owner and event target remain alive, no distinct red physical card other than the current card targeted this player earlier in the current turn. Requires Rule VM ABI 29."
            }
            "owner_alive_event_player_hand_empty" => {
                "Trigger filter: the skill owner and event player are both alive, and the event player's hand is empty."
            }
            "owner_alive_event_player_outside_turn_from_hand_empty" => {
                "Trigger filter: a living non-current event player lost hand cards and now has an empty hand while the skill owner remains alive."
            }
            "owner_alive_event_player_outside_turn_from_hand_empty_once" => {
                "Trigger filter: a living non-current event player lost hand cards and now has an empty hand while the skill owner remains alive; fires at most once during each current character's turn. Requires Rule VM ABI 26."
            }
            "owner_alive_event_player_turn_changed_or_became_linked" => {
                "Character-state trigger filter: while the owner is alive, fires after either turn-over direction or after the event player becomes linked; becoming unlinked does not fire. Requires Rule VM ABI 27."
            }
            "owner_alive_event_player_became_face_up_or_linked" => {
                "Character-state trigger filter: while the owner is alive, fires after the event player becomes face-up or linked; becoming face-down or unlinked does not fire. Requires Rule VM ABI 27."
            }
            "owner_turned_over" => {
                "Character-state trigger filter: the owner (event player) just completed a turn-over in either direction (face-up or face-down). Matches upstream `player: \"turnOverEnd\"`."
            }
            "owner_alive_other_wei_recovery_own_phase_once_per_turn" => {
                "Recovery trigger filter: while the owner is alive, another living wei-group player recovers HP during their own turn's phases (once per turn). Matches upstream zhushi's wei-recovery watch."
            }
            "owner_alive_global_juedou_or_red_sha_used" => {
                "Card-used trigger filter: while the owner is alive, any living player uses a duel or a red sha (owner involvement not required). Matches upstream global juedou/red-sha watch."
            }
            "owner_alive_other_global_juedou_or_red_sha_used" => {
                "Card-used trigger filter: while the owner is alive, another player (user != owner) uses a duel or a red sha."
            }
            "owner_alive_event_player_distance_le_owner_hp" => {
                "Trigger filter: the living event player is within the living owner's current HP distance from the owner."
            }
            "owner_alive_other_normal_damage_target_unlinked" => {
                "Damage-before trigger filter: a living owner observes another living, currently unlinked player about to receive normal damage. Requires Rule VM ABI 73."
            }
            "owner_alive_not_target" => {
                "Trigger filter: fires for each living owner other than the event player."
            }
            "owner_alive_not_target_hand_equip_nonempty" => {
                "Trigger filter: fires for a living owner other than the living event player while the owner has at least one hand/equipment card. Requires Rule VM ABI 144."
            }
            "owner_source_sha_or_ordinary_trick_target_wounded_has_discardable_hand_equip" => {
                "Per-target trigger filter: the living owner used a logical Sha or ordinary trick on a wounded living target and can discard one hand/equipment card. Requires Rule VM ABI 145."
            }
            "owner_source_sha_or_ordinary_trick_has_discardable_hand_equip" => {
                "Card-used trigger filter: the living owner used a logical Sha or ordinary trick and can discard one hand/equipment card. Requires Rule VM ABI 190."
            }
            "owner_source_jiu_boosted_sha_damage_skill_3365741831_enabled" => {
                "Parameterized damage trigger filter: the living owner dealt logical-Sha damage boosted by Jiu and still has the numeric-suffix skill definition effectively enabled. Requires Rule VM ABI 191."
            }
            "owner_alive_not_target_neither_unique_max_hand" => {
                "Trigger filter: the living owner and another living event player are both not the unique maximum-hand player."
            }
            "owner_alive_not_target_source_same_group" => {
                "Trigger filter: the living owner is not the event player and has the same non-empty group tag as the event source."
            }
            "owner_alive_not_target_equal_hand" => {
                "Trigger filter: fires when the living owner and another living event player have equal hand-card counts."
            }
            "owner_alive_not_target_hand_below_hp" => {
                "Trigger filter: the living owner is not the event player and has fewer hand cards than current HP."
            }
            "owner_alive_not_target_once" => {
                "Trigger filter: fires at most once per turn for each living owner other than the event player."
            }
            "owner_alive_current_or_turn_fires_below_hp" => {
                "Trigger filter: the living owner may fire up to their current HP times per turn, or without limit during their own turn."
            }
            "owner_alive_other_source_sha_twice" => {
                "Damage trigger filter: a living owner observes another living player deal logical Sha damage, at most twice per turn."
            }
            "owner_alive_card_is_sha" => {
                "Trigger filter: fires for any logical 杀 response while the skill owner remains alive."
            }
            "owner_alive_other_responded_sha_gainable" => {
                "Card-responded trigger filter: a living owner observes another player respond with logical 杀 whose physical cards are in the discard pile and can be gained. Requires Rule VM ABI 276."
            }
            "owner_targeted_by_trick_equip_empty" => {
                "Trigger filter: the skill owner was a declared trick target and their equipment zone is empty."
            }
            "owner_targeted_by_sha_hand_below_hp" => {
                "Trigger filter: the skill owner was a declared Sha target and their hand count is below their current HP."
            }
            "owner_targeted_by_sha_or_juedou_damaged_hp_le_source_once" => {
                "Trigger filter: once per turn, the damaged skill owner was a declared Sha or Juedou target and their HP is no greater than the card user's HP."
            }
            "owner_targeted_outside_turn_by_sha_or_trick_hand_empty" => {
                "Trigger filter: outside the skill owner's turn, they were a declared Sha or trick target while holding no hand cards."
            }
            "owner_targeted_hand_below_max_hp" => {
                "Trigger filter: the skill owner was a declared target and their hand count is below maximum HP."
            }
            "owner_targeted_by_other_single_target_trick" => {
                "Trigger filter: another player used a single-target trick on the skill owner."
            }
            "owner_source_or_targeted_by_juedou" => {
                "Trigger filter: the skill owner used or was a declared target of Juedou."
            }
            "owner_source_or_targeted_by_juedou_or_red_sha" => {
                "Trigger filter: the skill owner used or was a declared target of Juedou or a red Sha."
            }
            "owner_source_or_targeted_by_sha_equal_hp" => {
                "Trigger filter: in a per-target event, the skill owner is the Sha source or target and both players have equal HP."
            }
            "owner_source_or_targeted_by_other_diamond" => {
                "Trigger filter: the skill owner used/responded with a diamond card, or another player targeted them with one."
            }
            "owner_source_red" => {
                "Trigger filter: the skill owner used a red card, using the owner's effective suit."
            }
            "owner_source_alternates_color" => {
                "Trigger filter: during the owner's turn, the used card has a different color from the owner's previous used card this turn."
            }
            "owner_source_repeats_color_has_discardable_hand_equip" => {
                "Card-used trigger filter: during the owner's current turn, the used red/black card matches the immediately previous used card's effective color and the owner can discard a hand/equipment card. Requires Rule VM ABI 131."
            }
            "owner_alive_skill_bound_target_is_event_player" => {
                "Trigger filter: the living owner observes an event whose player equals this skill instance's bound target. Requires Rule VM ABI 132."
            }
            "owner_source_matches_previous_suit_or_rank" => {
                "Trigger filter: during the owner's play phase, the used card matches the previous card from that play phase by effective suit or rank."
            }
            "owner_source_ordinary_trick" => {
                "Trigger filter: the skill owner used an ordinary (non-delayed) trick card."
            }
            "owner_source_basic_or_trick" => {
                "Trigger filter: the skill owner used a basic or trick card. Requires Rule VM ABI 67."
            }
            "owner_is_target_can_compare_other" => {
                "Play-phase trigger filter: the owner is entering their Play phase and both they and at least one other living player have a hand card. Requires Rule VM ABI 68."
            }
            "owner_source_physical_ordinary_trick" => {
                "Trigger filter: the skill owner used a physical ordinary trick, excluding cards converted by view-as rules."
            }
            "owner_hand_below_lost_hp_after_relevant_change" => {
                "Trigger filter: the owner's hand is below lost HP after their HP/max HP changes or a move touches one of their hand, equipment, or judgment zones."
            }
            "owner_alive_source_first_card_ordinary_trick" => {
                "Trigger filter: a living skill owner observes a player's first used card this turn, and that card is an ordinary trick."
            }
            "owner_card_action_count_eq_attack_range" => {
                "Trigger filter: the skill owner's current-turn used-or-responded card count reaches their attack range on this event."
            }
            "owner_source_matches_last_used_suit_or_rank" => {
                "Trigger filter: the owner used a card matching their previous used card across turn boundaries by effective suit or rank."
            }
            "owner_alive_equip_source_max_equip" => {
                "Trigger filter: a living owner observes an equipment user tied for the most cards in the equipment zone after use."
            }
            "owner_is_target_event_card_sha" => {
                "Trigger filter: the skill owner is `event_player` and `event_card` is an effective Sha. For `hand_compare_finished`, this identifies the initiator's comparison card. Requires Rule VM ABI 339."
            }
            "owner_source_target_earlier_seat_hand_below_target_max_hp_once_per_turn" => {
                "Damage-before trigger filter: once per turn, the living owner is the damage source, the living target has an earlier seat, and the owner's hand is below the target's max HP. Requires Rule VM ABI 340."
            }
            "owner_alive_equip_used" => {
                "Card-used trigger filter: a living owner observes any logical equipment-card use. Requires Rule VM ABI 158."
            }
            "owner_alive_equip_used_skill_mark_below_three" => {
                "Trigger filter: a living owner observes an equipment-card use while the stable mark matching this skill definition is below three."
            }
            "owner_alive_source_alive_once_per_round" => {
                "Trigger filter: a living owner observes a sourced event at most once in the current round."
            }
            "owner_alive_source_alive_event_player_dying_once_per_round" => {
                "Trigger filter: a living owner observes a living sourced dying event while the event player remains alive and at 0 HP or below, at most once in the current round. Requires Rule VM ABI 129."
            }
            "owner_is_target_source_group_qun_discardable_hand_equip_gte_two" => {
                "Damage trigger filter: the living owner is the damaged event player, the damage source's character group is Qun, and the owner can currently discard at least two hand/equipment cards. Requires Rule VM ABI 134."
            }
            "owner_alive_other_current_source_repeats_card_name" => {
                "Trigger filter: a living owner observes another current-turn player use the same logical card name as their immediately previous card."
            }
            "owner_source_hand_basic_equals_non_basic" => {
                "Trigger filter: the skill owner used a card while their remaining hand contains equal numbers of basic and non-basic cards."
            }
            "owner_source_sha" => "Trigger filter: the skill owner used a logical Sha.",
            "owner_target_sha" => "Trigger filter: the owner took Sha-card damage (mirror of owner_source_sha).",
            "owner_source_heart_sha" => {
                "Card-use trigger filter: the living skill owner used a logical Sha whose effective suit is heart. Requires Rule VM ABI 580."
            }
            "owner_source_normal_sha" => {
                "Card-use trigger filter: the skill owner used an ordinary-nature logical Sha. Requires Rule VM ABI 45."
            }
            "owner_source_fire_sha" => {
                "Card-use trigger filter: the skill owner used a fire-nature logical Sha. Requires Rule VM ABI 45."
            }
            "owner_is_target_hand_sha_skill_pile_empty" => {
                "End-phase trigger filter: the living owner has at least one logical hand Sha and this skill definition's public expansion pile is empty. Requires Rule VM ABI 46."
            }
            "owner_alive_dying_has_skill_pile_cards" => {
                "Dying trigger filter: the living owner observes a living player at non-positive HP while this skill definition's public expansion pile is nonempty. Requires Rule VM ABI 46."
            }
            "owner_source_play_sha_other_alive_once_per_play_phase" => {
                "Damage trigger filter: during the owner's play phase, their logical Sha damaged another living player; fires at most once in that play phase."
            }
            "owner_source_jiu" => "Trigger filter: the skill owner used a logical Jiu.",
            "owner_is_target_hp_odd" => {
                "Trigger filter: owner is the living event player and has odd current HP."
            }
            "owner_is_target_hp_even_wounded" => {
                "Trigger filter: owner is the living event player, has even current HP, and is wounded."
            }
            "owner_alive_card_targets_gt_one" => {
                "Trigger filter: owner is alive and the used card declared more than one target."
            }
            "owner_alive_non_multitarget_trick_targets_gt_one" => {
                "Card-used trigger filter: a living owner observes a non-fixed-multitarget ordinary trick with at least two declared targets. Requires Rule VM ABI 308."
            }
            "owner_alive_wounded_card_targets_gt_one" => {
                "Trigger filter: owner is alive, wounded, and the used card declared more than one target. Requires Rule VM ABI 114."
            }
            "owner_source_sha_or_trick_current_play_phase" => {
                "Card-used trigger filter: the owner used a logical Sha or trick during their current Play phase. Requires Rule VM ABI 115."
            }
            "owner_source_card_has_other_target_current_play_phase_first_two" => {
                "Card-used trigger filter: fires for the first two cards the owner uses during their Play phase that declare at least one other player as a target. Requires Rule VM ABI 220."
            }
            "owner_alive_other_phase_end_has_discardable_basic_hand" => {
                "End-phase trigger filter: a living owner observes another living player's End phase and has at least one effective basic card in hand for the optional cost. Requires Rule VM ABI 178."
            }
            "owner_alive_other_phase_end_hand_nonempty" => {
                "End-phase trigger filter: a living owner observes another living player's End phase and has at least one hand card. Requires Rule VM ABI 299."
            }
            "owner_alive_other_phase_begin_both_hand_nonempty" => {
                "Turn-begin filter: owner and another living current-turn player both have hand cards. Requires Rule VM ABI 625."
            }
            "owner_alive_ddd_dongcha_pile_nonempty" => {
                "End-phase filter: owner is alive and the 洞察 skill pile is nonempty. Requires Rule VM ABI 625."
            }
            "owner_alive_prepare_player_hp_gt_owner" => {
                "Prepare-phase trigger filter: a living owner observes a living player's Prepare phase and that player's current HP is greater than the owner's. Requires Rule VM ABI 179."
            }
            "owner_alive_prepare_lord_hp_gt_owner" => {
                "Prepare-phase trigger filter: the HP-greater comparison also requires the event player to hold the Lord identity. Requires Rule VM ABI 179."
            }
            "owner_alive_other_sha_target_can_redirect_to_owner" => {
                "Per-target Sha filter: the living owner is not already targeted; another current target can discard a hand/equipment card and has the owner in that target's attack range. Requires Rule VM ABI 180."
            }
            "owner_sha_target_can_discard_redirect_to_other_in_attack_range" => {
                "Per-target Sha filter: the living owner is the current target, has a discardable hand/equipment card, and has another legal untargeted player other than the Sha user in the owner's attack range. Requires Rule VM ABI 186."
            }
            "owner_alive_gave_cards_to_other_alive" => {
                "Cards-given filter: the living owner is the attributed giver and the living recipient is another player. Requires Rule VM ABI 181."
            }
            "owner_alive_first_target_virtual_or_converted_card" => {
                "Trigger filter: a living owner observes only the first declared target of a virtual card or a card whose logical kind differs from its physical kind."
            }
            "owner_alive_targeted_by_other_single_card_discardable" => {
                "Trigger filter: a living owner is the sole target of another player's card and has at least one hand or equipment card to discard."
            }
            "owner_alive_targeted_by_trick_single_or_discardable" => {
                "Trigger filter: a living owner is targeted by a trick; multi-target tricks require at least one hand or equipment card the owner can discard."
            }
            "owner_alive_targeted_by_other_trick" => {
                "Trigger filter: a living owner is targeted by another player's logical trick card. Requires Rule VM ABI 111."
            }
            "owner_alive_targeted_by_other_matching_equip_suit" => {
                "Trigger filter: a living owner is targeted by another player's card whose raw suit matches an equipped card's effective suit."
            }
            "owner_alive_targeted_by_other" => {
                "Trigger filter: a living owner is targeted by another player's card."
            }
            "owner_alive_targeted_by_other_black_card" => {
                "Trigger filter: a living owner is targeted by another player's black card. Implements 协穆·指定 (xiemu2)."
            }
            "owner_alive_targeted_by_other_red_card" => {
                "Trigger filter: a living owner is targeted by another player's red card (mirror of the black variant). Implements the per-color targeted family (郊夏/芳华)."
            }
            "owner_alive_other_source_death" => {
                "Trigger filter: a living owner observes another player being killed and is not the killer. Implements 起乱 draw-on-death (qiluan)."
            }
            "owner_is_target_play_phase_hand_no_sha" => {
                "Trigger filter: the owner is the event player during their own play phase and has no Sha in hand. Implements 摧怀/咄咄 (cihuai/oldcihuai)."
            }
            "owner_alive_other_phase_end_player_outside_owner_range" => {
                "Trigger filter: a living owner observes another player's end phase and that event player is not in the owner's attack range. Implements 礼下·OL (ollixia)."
            }
            "owner_source_sha_target_alive_has_discardable_equip" => {
                "Trigger filter: the living owner used a Sha and the event player (a living target) has a discardable equipment card. Implements 风石 (zfengshi)."
            }
            "owner_source_card_used_targets_self" => {
                "Trigger filter: the living owner used a card and is among its targets. Implements 成都 (pxchengdu)."
            }
            "owner_source_current_phase_card_targeted_other_healthy" => {
                "Trigger filter: the living owner (current phase) targets a healthy other player with a card. Implements 廷机·OL (oltingji)."
            }
            "owner_source_first_card_suit_this_turn" => {
                "Trigger filter: the living owner used a card whose effective suit is the first of that suit this turn. Implements 文喜 (pxwenxi)."
            }
            "owner_lost_hand_now_empty" => {
                "Trigger filter: the living owner lost hand cards and their hand is now empty. Implements 平原 (pxpingyuan)."
            }
            "owner_lost_spade_two_to_nine" => {
                "Trigger filter: the living owner lost a physical spade 2-9 card from their hand. Implements 巨鹿 (pxjulu)."
            }
            "owner_min_hp" => {
                "Trigger filter: skill owner is tied for the lowest HP among living players."
            }
            "owner_is_target_not_min_hp" => {
                "Trigger filter: the living owner is the event player and is not tied for the lowest current HP. Requires Rule VM ABI 89."
            }
            "owner_is_target_has_movable_equipment" => {
                "Trigger filter: the owner is the event player and at least one equipment card can move to a living recipient other than its holder and the owner. Requires Rule VM ABI 30."
            }
            "owner_alive_used_sha_this_turn" => {
                "Trigger filter: the owner is alive and has actually used at least one logical Sha during the current character's turn. Requires Rule VM ABI 31."
            }
            "owner_target_min_hand" => {
                "Trigger filter: owner is the event player and tied for the fewest hand cards."
            }
            "owner_target_not_min_hand" => {
                "Trigger filter: owner is the event player but not tied for the fewest hand cards."
            }
            _ => return None,
    })
}
