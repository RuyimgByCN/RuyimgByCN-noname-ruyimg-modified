//! LSP 悬停文档链：关键字与效果说明（结算器见 `skill_resolvers`，owner_* 条件见 `skill_owner_filters`）。

pub(in super::super) fn documentation(word: &str) -> Option<&'static str> {
    Some(match word {
            "skill" => {
                "Define a skill.\n\n```\nskill name(def_id) [locked] [lord_only] [non_gainable] [conversion] {\n  trigger ... { effect ...; }\n  active play_phase [uses_per_turn N|owner_hp|owner_lost_hp_min_one] [once_per_game] [owner_missing_skill def_id] [hand_gt_hp|hand_gt_max_hp|hand_limit_positive|equip_nonempty|hand_nonempty|face_up] [targets min max [other] [lowest_hp|highest_hp] [unlinked] [wounded] [male] [max_by_owner_lost_hp|max_by_owner_hp] [hp_at_least_owner|hp_gt_owner|hp_lte_owner] [hp_gt N] [hand_gt_owner|hand_lte_owner] [hand_lt N] [hand_nonempty] [can_compare] [can_use_on_owner card_kind] [second_can_use_juedou_on_first] [equipment_pair_with_armor] [has_equip] [has_discardable_equip] [has_gainable_equip] [in_attack_range] [hand_equip_nonempty] [has_discardable_hand_equip] [unused_by_skill_this_turn|unused_by_skill_this_play_phase] [match_selected_card_count] [selected_cards_match_target_equip_count]] [select_hand|select_he min max [allow_cancel] [same_suit|same_color|distinct_suits] [distinct_categories] [card_equip|card_armor|card_sha|card_basic|card_nonbasic|card_non_equip|card_red] [unused_category_by_skill_this_turn|unused_category_by_skill_this_play_phase|unused_category_by_owner_history_this_turn]] { effect ...; }\n  modifier target mode value;\n}\n```\n`lord_only` keeps every trigger, active, view-as, and modifier inactive unless the owner has the Lord identity. `conversion` exposes the skill's branch to generic conversion rules."
            }
            "trigger" => {
                "Define a triggered ability.\n\nEvents include game_started, damage_before, damage_prevented, damage, hp_lost, card_used, phase_prepare, phase_play_end, phase_draw_skipped, phase_end, death, dying, dying_after, max_hp_changed, cards_drawn.\nFilters include owner_is_target, owner_is_source, owner_is_source_or_target, owner_alive, owner_alive_card_targets_gt_one, and the documented source/rate filters."
            }
            "active" => {
                "Define a player-activated skill body. Optional `uses_per_turn N|owner_hp|owner_lost_hp_min_one|living_rebel_count`, `uses_per_play_phase N`, `once_per_game`, `owner_missing_skill def_id`, `hand_gt_hp`, `hand_gt_max_hp`, `hand_limit_positive`, `equip_nonempty`, `hand_nonempty`, `face_up`, `targets min max`, and `select_hand`/`select_he` clauses are authoritative contracts enforced by the engine."
            }
            "view_as" => {
                "Treat matching physical source cards as another card kind. Optional `from hand_equip`, `from hand_expansion`, `up_to 2` or `exactly 2`, `same_suit`, `nature fire|thunder`, `multi_card_bonus damage_1|recover_1|discard_current_hand_equip`, `uses_per_turn N`, `ignore_distance`, `red_unrespondable`, `black_no_use_count`, `no_use_count`, `target_lacks_source_suit_unrespondable`, and `refund_use_count_if_no_damage` clauses are authoritative. Source-free dynamic families use `basic|ordinary_trick any any from none branch N once_per_round next_branch N cost ...`; `cost swap_lower_hand uses_per_turn 1` swaps whole hands with a lower-hand target before continuing; delegated `shan response` and `sha any` rules use `from group_<tag>` and ask matching other players in seat order."
            }
            "exactly" => {
                "View-as clause: require exactly the following number of physical source cards."
            }
            "choose_discard_weapon_or_lose_hp" => {
                "Effect: pay one of two active-skill costs: discard one current physical weapon from hand/equipment, or lose 1 HP. If only one cost is legal it is forced. Requires Rule VM ABI 222.\n\n```\neffect choose_discard_weapon_or_lose_hp(self);\n```"
            }
            "uses_per_turn" => {
                "Active/view-as clause: positive fixed per-turn activation limit; active skills may use `owner_hp`, `owner_lost_hp_min_one`, or `living_rebel_count` to read a dynamic limit at each activation. Checked by the engine and reset at turn start."
            }
            "uses_per_play_phase" => {
                "Active-only clause: positive activation limit reset on every entry to a Play phase, including extra Play phases. Requires Rule VM ABI 301."
            }
            "choose_range" => {
                "Expression: `choose_range(min, max)` offers every non-negative integer in the inclusive dynamic range and evaluates to the selected token. At most 255 options; requires Rule VM ABI 92."
            }
            "branch" => {
                "Source-free view-as clause: require the skill instance's current conversion branch (0..31)."
            }
            "once_per_round" => {
                "Source-free view-as clause: the declared branch may resolve only once in the current round."
            }
            "next_branch" => {
                "Source-free view-as clause: select this conversion branch after a successful cost payment."
            }
            "cost" => {
                "Source-free view-as clause: authoritative cost `gain_max_hp_1`, `disable_equip_slot`, deterministic `guhuo`, or a tagged `face_down_hand` card."
            }
            "ignore_distance" => {
                "View-as clause: skip only the converted card's ordinary distance check; all other target, use-count, response, and effect rules remain active. Requires Rule VM ABI 4."
            }
            "red_unrespondable" => {
                "View-as Sha clause: when the selected source card is effectively red, only this converted Sha becomes unrespondable. Requires Rule VM ABI 14."
            }
            "black_no_use_count" => {
                "View-as Sha clause: when the selected source card is effectively black, only this converted Sha skips the ordinary use-limit check and count increment. Requires Rule VM ABI 14."
            }
            "no_use_count" => {
                "View-as Sha clause: this converted Sha skips the ordinary use-limit check and count increment. Requires Rule VM ABI 521."
            }
            "target_lacks_source_suit_unrespondable" => {
                "View-as Sha clause: each target without a hand card matching the physical source card's effective suit cannot respond to this Sha. Requires Rule VM ABI 521."
            }
            "refund_use_count_if_no_damage" => {
                "View-as Sha clause: after this exact converted Sha finishes, refund its ordinary use count if it dealt no damage. Requires Rule VM ABI 354."
            }
            "non_basic" => "Card filter: match cards whose effective kind is not a basic card.",
            "hand_gt_hp" => {
                "Active-skill clause: activation is legal only while the owner's hand count exceeds current HP."
            }
            "hand_gt_max_hp" => {
                "Active-skill clause: activation is legal only while the owner's hand count exceeds maximum HP."
            }
            "hand_limit_positive" => {
                "Active-skill clause: activation is legal only while the owner's current hand limit is positive. Requires Rule VM ABI 143."
            }
            "equip_nonempty" => {
                "Active-skill clause: activation is legal only while the owner's equipment zone is nonempty. Requires Rule VM ABI 58."
            }
            "face_up" => {
                "Active-skill clause: activation is legal only while the owner is face up. Requires Rule VM ABI 151."
            }
            "once_per_game" => {
                "Active-skill clause: the skill may resolve at most once during the match. The lifetime counter is not reset at turn boundaries. Requires Rule VM ABI 58."
            }
            "targets" => {
                "Active-skill clause: inclusive minimum and maximum number of distinct living player targets. Append `other` to exclude the owner, `lowest_hp`/`highest_hp` to require a global HP extremum, `unlinked` to require an unlinked target, `wounded` to require current HP below maximum HP, `male` to require male/double-sex targets, `hp_at_least_owner`/`hp_gt_owner`/`hp_lte_owner` to compare target HP with the owner's current HP, `hp_gt N` to require current HP above a literal threshold, `hand_gt_owner` to require more hand cards than the owner, `hand_lt N` to require fewer than N hand cards, `hand_nonempty` to require at least one hand card, `can_compare` to require both participants to have hand cards, `can_use_on_owner card_kind` to require a legal source-free use against the owner, `second_can_use_juedou_on_first` to require the second of exactly two targets to be able to use Duel on the first, `equipment_pair_with_armor` for an exact two-target equipment pair containing discardable armor, `has_equip` to require a nonempty equipment zone, `has_discardable_equip` to require equipment the skill owner may discard, `has_gainable_equip` to require equipment the skill owner may gain, `in_attack_range` to require the owner's current attack range, `hand_equip_nonempty` to require a nonempty hand/equipment region, `has_discardable_hand_equip` to require a discardable hand/equipment card, `unused_by_skill_this_turn` or `unused_by_skill_this_play_phase` to reject targets this skill already recorded in that scope, `match_selected_card_count` to bind target count to card count, or `selected_cards_match_target_equip_count` to bind the selected-card count to the sole target's equipment count."
            }
            "other" => "Active-target constraint: the skill owner cannot be selected as a target.",
            "lowest_hp" => {
                "Active-target constraint: every selected target must tie for the lowest current HP among living players."
            }
            "highest_hp" => {
                "Active-target constraint: every selected target must tie for the highest current HP among living players."
            }
            "unlinked" => {
                "Active-target constraint: every selected target must currently be unlinked."
            }
            "wounded" => {
                "Active-target constraint: every selected target must currently be alive and below maximum HP. Requires Rule VM ABI 109."
            }
            "male" => {
                "Active-target constraint: every selected target must be male or double-sex. Requires Rule VM ABI 151."
            }
            "hp_at_least_owner" => {
                "Active-target constraint: every selected target's current HP must be at least the skill owner's current HP."
            }
            "hp_gt_owner" => {
                "Active-target constraint: every selected target's current HP must be greater than the skill owner's current HP. Requires Rule VM ABI 121."
            }
            "hp_lte_owner" => {
                "Active-target constraint: every selected target's current HP must be less than or equal to the skill owner's current HP. Requires Rule VM ABI 402."
            }
            "hp_gt" => {
                "Active-target constraint: `hp_gt N` requires every selected target to have more than N current HP. Revalidated by the engine and requires Rule VM ABI 132."
            }
            "hand_gt_owner" => {
                "Active-target constraint: every selected target must currently have more hand cards than the skill owner. Revalidated by the engine and requires Rule VM ABI 113."
            }
            "hand_lte_owner" => {
                "Active-target constraint: every selected target must currently have no more hand cards than the skill owner. Revalidated by the engine and requires Rule VM ABI 514."
            }
            "pair_hand_difference_lte_three" => {
                "Active-target constraint: exactly two selected targets must have hand-card counts differing by at most three. Revalidated by the engine and requires Rule VM ABI 535."
            }
            "potqingyan" => {
                "View-as cost: reveal exactly min(round uses + 1, 5) hand cards without consuming them; unavailable while any card shown by the previous activation remains in hand. Requires Rule VM ABI 544."
            }
            "hand_lt" => {
                "Active-target constraint: `hand_lt N` requires every selected target to have fewer than N hand cards. Revalidated by the engine and requires Rule VM ABI 128."
            }
            "hand_nonempty" => {
                "Active-skill/target constraint: outside `targets`, the owner must have at least one hand card (ABI 151); inside `targets`, every selected target must have at least one hand card (ABI 150)."
            }
            "can_compare" => {
                "Active-target constraint: the skill owner and every selected target must each currently have at least one hand card. Revalidated before opening the owner's card selection; requires Rule VM ABI 47."
            }
            "can_use_on_owner" => {
                "Active-target constraint: every selected target must currently be able to use the following source-free card kind on the skill owner. The engine checks user state, use locks, target locks, self-target rules, and target-count modifiers. Currently supports `juedou`; requires Rule VM ABI 23."
            }
            "second_can_use_juedou_on_first" => {
                "Active-target constraint: requires exactly two other targets and checks that the second can currently use a source-free Duel on the first. Target order is authoritative and legality is checked before active-skill card costs. Requires Rule VM ABI 183."
            }
            "equipment_pair_with_armor" => {
                "Active-target constraint: requires exactly two distinct living targets, each with discardable equipment, and at least one with discardable armor. Requires Rule VM ABI 32."
            }
            "has_equip" => {
                "Active-target constraint: every selected target must currently have at least one card in its equipment zone. Requires Rule VM ABI 97."
            }
            "has_discardable_equip" => {
                "Active-target constraint: every selected target must currently have at least one equipment card the skill owner may discard. Equipment-zone ownership and discard protection are authoritatively revalidated. Requires Rule VM ABI 136."
            }
            "has_gainable_equip" => {
                "Active-target constraint: every selected target must currently have at least one equipment card the skill owner may gain. Equipment-zone ownership and gain protection are authoritatively revalidated. Requires Rule VM ABI 327."
            }
            "has_gainable_hand_equip" => {
                "Active-target constraint: every selected target must currently have at least one hand/equipment card the skill owner may gain. Zone ownership and gain protection are authoritatively revalidated. Requires Rule VM ABI 355."
            }
            "in_attack_range" => {
                "Active-target constraint: every selected target must currently be inside the skill owner's attack range. Distance and attack-range modifiers are authoritatively revalidated. Requires Rule VM ABI 155."
            }
            "hand_equip_nonempty" => {
                "Active-target constraint: every selected target must currently have at least one card in hand or equipment. Requires Rule VM ABI 192."
            }
            "has_discardable_hand_equip" => {
                "Active-target constraint: every selected target must currently have at least one hand/equipment card the skill owner may discard. Zone ownership and discard protection are authoritatively revalidated. Requires Rule VM ABI 155."
            }
            "unused_by_skill_this_turn" => {
                "Active-target constraint: rejects any target already recorded in this skill instance's canonical turn-scoped uniqueness state. Requires Rule VM ABI 52."
            }
            "unused_by_skill_this_play_phase" => {
                "Active-target constraint: rejects any target already recorded by this skill in the current Play phase. The target history resets on every entry to a Play phase, including extra Play phases. Requires Rule VM ABI 292."
            }
            "match_selected_card_count" => {
                "Active-target constraint: requires selected-player and selected-card counts to be equal; both declarations must use the same min/max range. Requires Rule VM ABI 109."
            }
            "selected_cards_match_target_equip_count" => {
                "Active-target constraint: requires exactly one equipped target and binds the selected-card count to that target's current equipment-card count. The count is revalidated before the activation resolves. Requires Rule VM ABI 157."
            }
            "select_hand" => {
                "Active-skill clause: inclusive minimum and maximum number of distinct cards selected from the owner's hand. Append `allow_cancel` to allow cancellation at the choice barrier, `same_suit` to require one non-virtual effective suit, `same_color` to require one effective color, `distinct_categories` to require unique basic/trick/equipment categories, `card_sha` to offer only logical Sha cards, `card_basic` to offer only effective basic cards, `card_nonbasic` to exclude effective basic cards, `card_non_equip` to exclude effective equipment cards, or `card_heart` to require an effective heart suit."
            }
            "select_he" => {
                "Active-skill clause: inclusive minimum and maximum number of distinct cards selected from the owner's hand or equipment zone. Append `allow_cancel` to allow cancellation at the choice barrier, `same_suit` to require one non-virtual effective suit, `distinct_categories` to require unique basic/trick/equipment categories, `card_equip` to offer only effective equipment cards, `card_armor` to offer only physical armor cards, `card_sha` to offer only logical Sha cards, `card_basic` to offer only effective basic cards, or `card_nonbasic` to exclude effective basic cards."
            }
            "card_nonbasic" => {
                "Active-card selection filter: offer only cards whose effective kind is not basic. Requires Rule VM ABI 327."
            }
            "card_non_equip" => {
                "Active-card selection filter: offer only cards whose effective kind is not equipment. Requires Rule VM ABI 497."
            }
            "card_heart" => {
                "Active-card selection filter: offer only cards whose effective suit is heart. Requires Rule VM ABI 494."
            }
            "allow_cancel" => {
                "Active-card selection constraint: the selection barrier may be cancelled without paying cards or consuming the active-skill use. Cancellation remains invalid for non-cancellable choices."
            }
            "same_suit" => {
                "Active-card constraint: all selected physical cards must have the same non-virtual effective suit. Requires Rule VM ABI 304."
            }
            "same_color" => {
                "Active-card constraint: all selected cards must have the same effective red/black color."
            }
            "distinct_categories" => {
                "Active-card constraint: selected cards must have pairwise-distinct effective basic/trick/equipment categories. Requires Rule VM ABI 109."
            }
            "sum_effective_number_exact" => {
                "Active hand-card constraint: selected distinct cards must have effective numbers totaling exactly the positive configured value. Requires Rule VM ABI 301."
            }
            "unused_category_by_skill_this_turn" => {
                "Active-card constraint: offers and accepts only cards whose effective basic/trick/equipment category is not yet recorded by this skill in the current turn. Requires Rule VM ABI 52."
            }
            "unused_category_by_skill_this_play_phase" => {
                "Active-card constraint: offers and accepts only cards whose effective basic/trick/equipment category is not yet recorded by this skill in the current play phase. The history resets on every entry to a Play phase. Requires Rule VM ABI 110."
            }
            "unused_category_by_owner_history_this_turn" => {
                "Active-card constraint: offers and accepts only cards whose effective basic/trick/equipment category has not been used or discarded by the owner in the current turn. Requires Rule VM ABI 285."
            }
            "modifier" => {
                "Define a passive modifier.\n\n```\nmodifier target mode value;\n```\nA value may be an integer or a documented query-time source such as owner_hp, owner_lost_hp, owner_skill_mark, owner_skill_use_count, alive_group_count, alive_female_count, alive_other_count_capped_four, alive_wounded_count_zero_or_at_least_four, alive_wounded_count_at_least_N, alive_other_qun_count, alive_field_rank_thirteen_player_count, field_mount_card_count_ceil_half, owner_unique_max_hand, owner_hand_gt_other, owner_hand_lt_other, owner_qun_other_shu, owner_turned_over_diff_other, owner_linked_diff_other, owner_hand_equip_gte_other, owner_sha_damage_dealt_zero_this_turn, queried_player_hand_gt_hp_signed, queried_player_judge_nonempty_other_than_owner, queried_player_linked_other_than_owner, queried_player_other_than_owner, queried_source_enemy_owner_ally_target_other_than_owner, owner_max_hand_before_skill_fires_this_turn, owner_no_mount, owner_armor_empty, owner_healthy, owner_has_weapon, owner_has_treasure, owner_has_heart_equip, alive_mounted_player_count_min_one, owner_equip_count, owner_equip_count_at_least_N, owner_equip_count_odd, owner_equip_count_even, owner_equip_count_half_floor, owner_equip_count_exactly_one, owner_hp_gt_two, owner_hp_le_two, owner_lost_hp_at_least_two, owner_all_others_in_attack_range, owner_sha_used_zero, owner_sha_used_nonzero, owner_sha_used_exactly_one, owner_sha_used_exactly_two, owner_non_damage_hand_type_count, owner_sha_hand_count, owner_distinct_hand_card_suits_used_this_turn, owner_current_turn_card_use_count, owner_play_phase_card_use_count_zero, or owner_turn_damage_dealt; prefix a dynamic value with `-` to subtract it.\nTargets include distance_from, distance_to, global_distance_to, attack_range, global_attack_range, global_hand_limit, global_damage_taken_amount(nature), global_damage_taken_final_amount(nature), hand_limit_base, hand_limit, draw_count, optional_draw_count, sha_use_limit, sha_target_limit, judgment_rewrite, judgment_rewrite_hand_equip, judgment_rewrite_red_hand_equip_expansion, judgment_best_from_draw_pile, card_target_limit(selector), observer_foreign_hand_faces_visible, observer_hidden_identities_visible, damage_amount(selector), card_suit(selector), card_name(selector), card_nature(selector), card_targets_all(selector), card_targets_self_only.\nModes: add, override, multiply, min, max."
            }
            "optional_draw_count" => {
                "Modifier target: offers one draw-phase confirmation per owning skill, then adds that skill's grouped positive contribution."
            }
            "effect" => {
                "Request an effect from the trigger body.\n\n```\neffect draw_cards(self, N);\n```"
            }
            "damage" => "Trigger event: fires after damage is dealt to the owner.",
            "hp_lost" => "Trigger event: fires after HP is lost without damage.",
            "phase_prepare" => "Trigger event: fires at the start of the preparation phase.",
            "phase_begin" => {
                "Trigger event: fires once at the start of a turn, immediately before phase_prepare triggers."
            }
            "phase_judge" => "Trigger event: fires at the start of the judgment phase.",
            "phase_draw" => "Trigger event: fires at the start of the draw phase.",
            "phase_play" => "Trigger event: fires at the start of the play phase.",
            "phase_discard" => "Trigger event: fires at the start of the discard phase.",
            "phase_end" => "Trigger event: fires at the end of a phase.",
            "phase_draw_end" => {
                "Trigger event: fires when the draw phase ends. Requires Rule VM ABI 50."
            }
            "phase_play_end" => "Trigger event: fires when the play phase ends.",
            "phase_any_end" => {
                "Trigger event: fires after any concrete Prepare, Judge, Draw, Play, Discard, or End phase. It carries bounded target and actual-HP-change histories for that one phase. Requires Rule VM ABI 98."
            }
            "phase_draw_skipped" => {
                "Trigger event: fires when a player's draw phase is skipped or cancelled."
            }
            "death" => "Trigger event: fires when a player dies.",
            "card_targeted" => {
                "Trigger event: fires once for each declared target, in target order."
            }
            "card_responded" => "Trigger event: fires when a logical response card is committed.",
            "card_resolved" => {
                "Trigger event: fires once after one logical card finishes all targets, responses, damage, dying, and nested choices. `event_source` is the user, `event_card` is the physical carrier, and `event_targets` preserves declared target order. Requires Rule VM ABI 503."
            }
            "sha_dodged" => {
                "Trigger event: fires once after a target supplies every Shan required to fully dodge one logical Sha. `event_source` is the Sha user, `event_player` is the dodging target, and `event_card` is the physical Sha."
            }
            "sha_resolved" => {
                "Trigger event: fires once after one logical Sha finishes every target, response, damage, dying, and nested choice. `event_source` and `event_player` are the Sha user; `event_card` is its physical carrier. Requires Rule VM ABI 54."
            }
            "hand_compare_started" => {
                "Trigger event: fires after the initiator has selected a valid hand card and before the other participant chooses. `event_player` is the initiator, `event_source` is the target, and the initiator's selected card is captured privately for follow-up effects. Requires Rule VM ABI 65 for comparison effects."
            }
            "hand_compare_finished" => {
                "Trigger event: fires after both comparison cards are revealed and moved to the discard pile. `event_player` is the initiator, `event_source` is the target, and both physical cards are captured in participant order. Requires Rule VM ABI 65 for comparison effects."
            }
            "judgment_resolved" => {
                "Trigger event: fires once after a judgment and every rewrite opportunity complete. `event_player` is the judged player, `event_card` is the delayed trick/equipment cause when present, and the final physical result card is captured as the first event card. Requires Rule VM ABI 86."
            }
            "cards_given" => {
                "Trigger event: fires after an attributed player-to-player gift. `event_source` is the giver, `event_player` is the recipient, and `event_cards` contains the transferred physical cards. Requires Rule VM ABI 181."
            }
            "dying" => "Trigger event: fires when a player enters dying state.",
            "dying_after" => "Trigger event: fires when a saved player leaves dying state.",
            "max_hp_changed" => "Trigger event: fires after a player's maximum HP changes.",
            "distance_from" => "Modifier target: adjusts distance from owner to other players.",
            "distance_from_if_hp_gte_target" => {
                "Modifier target: sets the owner's final distance to a target to one when the owner's HP is at least that target's HP. Requires `override 1`."
            }
            "distance_from_skill_bound_target" => {
                "Modifier target: sets the owner's final distance to this skill instance's bound target to one. Requires `override 1` and Rule VM ABI 196."
            }
            "attack_range" => "Modifier target: adjusts owner's attack range.",
            "hand_limit_base" => {
                "Modifier target: adjusts the base hand limit before ordinary hand-limit bonuses."
            }
            "hand_limit_base_max_hp" => {
                "Modifier target: uses maximum HP instead of current HP as the hand-limit base when non-zero."
            }
            "skip_judge_phase" => {
                "Modifier target: skips the owner's judgment phase while non-zero."
            }
            "skip_discard_phase" => {
                "Modifier target: skips the owner's discard phase while non-zero."
            }
            "restore_skipped_phase" => {
                "Modifier target: after the engine records a skipped/cancelled phase audit event, restore that exact owner phase instead of advancing past it. Covers Judge, Draw, Play, and Discard phase skip paths. Requires `override 1` and Rule VM ABI 111."
            }
            "turn_face_down_cancel" => {
                "Modifier target: prevents a face-up owner from being turned face-down. An owner who is already face-down may still turn face-up. Requires `override 1` and Rule VM ABI 111."
            }
            "turn_over_cancel" => {
                "Modifier target: prevents the owner from changing either face-up/face-down state. Requires `override 1` and Rule VM ABI 164."
            }
            "discard_phase_includes_equip" => {
                "Modifier target: counts the owner's equipment together with hand cards during the ordinary discard-to-limit phase and permits discarding from either zone. Requires `override 1`."
            }
            "draw_phase_scry_four_split_two" => {
                "Modifier target: offers an optional draw-phase reorder of four cards with exactly two left on top and two sent to the bottom."
            }
            "initial_hand_count" => {
                "Modifier target: adjusts the owner's initial hand size before the first turn."
            }
            "card_suit" => {
                "Card-suit modifier: `card_suit(spade) override heart` makes the owner's physical spade cards count as hearts in rule checks."
            }
            "card_name" => {
                "Card-name modifier: `card_name(red) override sha` makes the owner's red cards behave as Sha without mutating the physical cards."
            }
            "card_nature" => {
                "Card-nature modifier: `card_nature(sha) override kami` changes the damage nature of matching cards without mutating their physical identity."
            }
            "sha_unrespondable" => {
                "Modifier target: prevents targets from answering matching cards when they resolve as Sha."
            }
            "card_use_unrespondable" => {
                "Modifier target: prevents other players from answering matching cards used by the skill owner."
            }
            "sha_unrespondable_within_distance" => {
                "Modifier target: prevents a target from answering the owner's Sha when their final distance is at most the positive override value."
            }
            "sha_use_unrespondable_within_distance" => {
                "Modifier target: prevents every target from answering a Sha use when its source or one target owns the modifier and the other participant's final distance to that owner is at most the positive override value."
            }
            "sha_target_unrespondable" => {
                "Target-side modifier: prevents an incoming Sha from being answered when the target owns this skill."
            }
            "damage_taken_source_range_threshold" => {
                "Target-side damage policy: source attack range below the positive threshold caps damage at one; above it adds one."
            }
            "sha_damage_source_range_threshold" => {
                "Source-side logical Sha damage policy: owner attack range below the positive threshold caps final damage at one; above it adds one before later damage modifiers."
            }
            "sha_damage" => {
                "Modifier target: adjusts damage dealt by the owner's Sha/Juedou, or only matching Sha when given a card selector."
            }
            "sha_damage_taken" => {
                "Target-side modifier: adjusts damage received by the owner specifically from a logical Sha."
            }
            "sha_damage_if_outside_target_range" => {
                "Source-side modifier: adjusts logical Sha damage only when the source is outside the target's attack range."
            }
            "card_distance_unlimited" => {
                "Modifier target: ignores ordinary target-distance limits for all cards used by the owner, or only matching cards when given a selector."
            }
            "card_discard_protected" => {
                "Target-side modifier: matching physical equipment in the owner's equipment zone cannot be discarded by another player. Requires `override 1`."
            }
            "card_gain_protected" => {
                "Target-side modifier: matching physical cards in the owner's hand/equipment/judgment zones cannot be gained by another player. Requires `override 1` and Rule VM ABI 300."
            }
            "card_distance_unlimited_if_target_hand_gte_owner" => {
                "Target-aware modifier: matching cards ignore ordinary distance only when the target has at least as many hand cards as the owner."
            }
            "card_distance_limit" => {
                "Card-aware modifier target: overrides the ordinary target-distance limit for matching cards. Requires `override` with a non-negative integer."
            }
            "virtual_bagua_if_armor_slot_empty" => {
                "Provide the owner with a virtual Bagua response only while their armor slot is enabled and empty. Requires `override 1` and Rule VM ABI 87."
            }
            "sha_response_count_if_other_female" => {
                "Sha participant modifier: when the owner is the source or current target and the other participant has female or double sex, add to that target's required Shan count. Requires `add` and Rule VM ABI 89."
            }
            "sha_response_count_if_source_owner_target_distance_le_one" => {
                "Sha source modifier: when the owner uses Sha against a target within final distance one, add to that target's required Shan count. Requires `add` and Rule VM ABI 353."
            }
            "duel_opponent_response_count" => {
                "Duel participant modifier: add to the Sha count required from the modifier owner's opponent in each exchange. Requires `add` and Rule VM ABI 167."
            }
            "tao_use_save_cancel_for_non_dying_others_during_owner_turn" => {
                "Global Tao restriction: during the modifier owner's current turn, other players who are not themselves dying cannot proactively use Tao or use Tao to save a dying player. The turn owner and a dying player remain allowed. Use `override 1`, or `override owner_skill_mark` for a turn-scoped activation mark (ABI 218)."
            }
            "card_target_limit" => {
                "Card-aware modifier target: adjusts the maximum number of distinct targets for matching independently-targeted cards. It requires a selector and Rule VM ABI 19."
            }
            "judgment_rewrite_hand_equip" => {
                "Permit one optional judgment rewrite from the owner's hand or equipment zone. Every eligible rewriter is offered in seat order; requires Rule VM ABI 24."
            }
            "judgment_rewrite_red_hand_equip_expansion" => {
                "Permit one optional judgment rewrite using an effective-red card from the owner's hand, equipment, or expansion zone. Every eligible rewriter is offered in seat order; requires Rule VM ABI 342."
            }
            "judgment_best_from_draw_pile" => {
                "Use the highest-scoring card in the draw pile for each judgment owned by this player; requires Rule VM ABI 72."
            }
            "observer_foreign_hand_faces_visible" => {
                "Observer-projection modifier: reveals every other player's hand-card faces only to the owning seated player or that player's private replay export. Requires `override 1` and Rule VM ABI 20."
            }
            "observer_hidden_identities_visible" => {
                "Observer-projection modifier: reveals identities that would otherwise be hidden only to the owning seated player or that player's private replay export. Requires `override 1` and Rule VM ABI 20."
            }
            "card_targets_all" => {
                "Card-aware modifier target: matching cards must target every otherwise-legal player. Selectors: sha, basic, non_basic, trick, equip, suits, red, black."
            }
            "card_target_cancel" => {
                "Target-side modifier: matching cards cannot select the skill owner. Selectors include names, categories, suits/colors, ordinary_trick, black_trick, and damage_trick; a query-time value such as owner_hand_empty may make the restriction conditional."
            }
            "armor_card_target_cancel" => {
                "Armor modifier: matching cards cannot select the skill owner unless the source ignores armor or the owner's armor effects are disabled."
            }
            "armor_damage_taken_amount" => {
                "Armor modifier: adjusts incoming normal, fire, or thunder damage unless the source ignores armor or the owner's armor effects are disabled."
            }
            "armor_link_gain_cancel" => {
                "Armor modifier: prevents the unlinked owner from gaining linked state while armor is active; removing existing linked state remains allowed."
            }
            "link_loss_cancel" => {
                "Modifier: prevents a living linked owner from losing the chain mark. Requires `override 1` and Rule VM ABI 213."
            }
            "ignore_armor" => {
                "Source-side modifier: suppresses armor-owned targeting and damage modifiers."
            }
            "armor_effect_disabled" => {
                "Target-side modifier: suppresses armor-owned targeting and damage modifiers for the owner."
            }
            "damage_amount" => {
                "Source-side modifier: adjusts non-lose-HP damage dealt by the owner. An optional (normal), (fire), or (thunder) selector limits the original nature."
            }
            "damage_as_hp_loss" => {
                "Source-side replacement: damage dealt by the owner becomes equal HP loss. Add `(sha)` to limit the replacement to logical Sha damage. Requires `override 1`."
            }
            "damage_between_owner_and_other_as_hp_loss" => {
                "Relationship replacement: damage dealt by the owner to another player, or received by the owner from another living player, becomes equal HP loss. Requires `override 1`."
            }
            "damage_taken_cancel" => {
                "Target-side modifier: cancels damage received by the owner. With (normal), (fire), or (thunder), it only matches that resolved nature."
            }
            "damage_taken_amount" => {
                "Target-side modifier: adjusts non-lose-HP damage received by the owner. An optional (normal), (fire), or (thunder) selector limits the resolved nature."
            }
            "global_damage_taken_amount" => {
                "Living-owner aura: adjusts ordinary incoming damage for every queried player. An optional nature selector limits the resolved nature."
            }
            "global_damage_taken_final_amount" => {
                "Living-owner aura: adjusts final incoming damage after ordinary and armor modifiers for every queried player. An optional nature selector limits the resolved nature."
            }
            "global_draw_count" => {
                "Living-owner aura: adjusts every queried player's draw-phase count. Requires Rule VM ABI 215."
            }
            "global_hand_limit" => {
                "Living-owner aura: adjusts every queried player's hand limit while the skill owner is alive."
            }
            "global_sha_use_limit" => {
                "Living-owner aura: adjusts every queried player's ordinary Sha use limit. Requires Rule VM ABI 215."
            }
            "global_distance_to" => {
                "Living-owner aura: adjusts distance from a queried source to a queried target."
            }
            "damage_taken_final_amount" => {
                "Late target-side modifier: adjusts final non-lose-HP damage after equipment and ordinary amount modifiers. An optional (normal), (fire), or (thunder) selector limits the resolved nature."
            }
            "damage_taken_nature" => {
                "Target-side modifier: overrides the resolved nature of incoming non-lose-HP damage. Use 0 for normal, 1 for fire, or 2 for thunder."
            }
            "card_target_cancel_from_current" => {
                "Target-side modifier: cards from the current turn owner cannot select the skill owner. The currently supported selector is any."
            }
            "card_target_cancel_if_target_lacks_matching_suit" => {
                "Target-side modifier: a card cannot select the skill owner unless their hand contains that card's suit."
            }
            "card_target_cancel_if_other_user_hp_gte_target" => {
                "Target-side modifier: cards from another player whose current HP is at least the skill owner's current HP cannot select the owner. Requires Rule VM ABI 107."
            }
            "card_target_cancel_if_other_user_distance_gt_one" => {
                "Target-side modifier: matching cards from another player whose final distance to the skill owner is greater than one cannot select the owner. Requires Rule VM ABI 166."
            }
            "card_target_cancel_if_other_user_hand_gte_target_and_armor_empty" => {
                "Target-side modifier: matching cards from another player with at least as many hand cards cannot select an armorless skill owner."
            }
            "card_targets_self_only" => {
                "Modifier target: cards used by the owner may not target another player."
            }
            "card_use_cancel" => {
                "Modifier target: prevents the owner from proactively using any card, or only cards matching an optional selector."
            }
            "card_action_cancel_for_skill_color_mask" => {
                "Modifier target: prevents the owner from using, responding with, or saving with a physical card whose effective red/black color appears in this skill instance's color-lock state. Requires `override 1` and Rule VM ABI 48."
            }
            "card_use_unrespondable_within_responder_to_owner_distance" => {
                "Source-side modifier: cards used by the owner cannot be answered by another player whose final distance to the owner is within the configured threshold."
            }
            "card_use_unrespondable_within_owner_to_responder_distance" => {
                "Source-side modifier: cards used by the owner cannot be answered by another player within the configured final distance from the owner."
            }
            "card_response_cancel" => {
                "Modifier target: prevents the owner from responding with any card, or only cards matching an optional selector."
            }
            "hand_card_use_response_cancel" => {
                "Modifier target: prevents the owner from using or responding with physical cards in hand while leaving non-hand responses available."
            }
            "hand_card_gained_this_turn_only" => {
                "Modifier target: during the owner's Play phase, only physical hand cards gained this turn may be used or played as responses. Requires Rule VM ABI 380."
            }
            "card_save_cancel" => {
                "Modifier target: prevents the owner from using any card to save a dying player, or specifically 桃/酒 with the tao or jiu selector."
            }
            "sha_response_cancel" => {
                "Modifier target: prevents the owner from responding with 杀 when non-zero."
            }
            "sha_ignored_hand_limit" => {
                "Modifier target: excludes the owner's 杀 from hand-limit count and discard-phase choices when non-zero."
            }
            "card_ignored_hand_limit" => {
                "Card-aware modifier target: excludes matching cards from hand-limit count and discard-phase choices. Selectors include card names, sha/sha_fire/sha_red/sha_diamond, categories, suits, and colors."
            }
            "queried_player_other_than_owner" => {
                "Query-time modifier value: one when the queried living player is not the skill owner, otherwise zero."
            }
            "queried_player_wounded_in_owner_attack_range_other" => {
                "Query-time modifier value: one when the queried living player is wounded, differs from the owner, and is in the owner's attack range; otherwise zero."
            }
            "queried_source_enemy_owner_ally_target_other_than_owner" => {
                "Global distance aura value: one when the target is the owner's living ally other than the owner and the source is the target's enemy."
            }
            "queried_player_qun_owner_lord_alive" => {
                "Query-time modifier value: one for a living Qun card user while the living modifier owner is the Lord. ABI 553."
            }
            "alive_group_count" => {
                "Read-only `alive_group_count()` query and query-time modifier value: the number of distinct non-empty groups among living players' characters. The expression query requires Rule VM ABI 6."
            }
            "alive_wounded_or_guiming_wu_count" => {
                "Read-only `alive_wounded_or_guiming_wu_count(player)` query: living wounded-player count, plus other living Wu players while that player is the Lord and has enabled Guiming. Requires Rule VM ABI 190."
            }
            "unique_max_current_hp" => {
                "Read-only `unique_max_current_hp(player)` query: one only when the living player has strictly more current HP than every other living player. Requires Rule VM ABI 7."
            }
            "unique_max_current_hp_at" => {
                "Read-only `unique_max_current_hp_at(player, value)` query: one only when the living player would be the unique current-HP leader at the supplied value, leaving every other player unchanged. Requires Rule VM ABI 7."
            }
            "lifetime_damage_dealt" => {
                "Read-only `lifetime_damage_dealt(player)` query: total non-HP-loss damage points dealt by that player during the current match. Requires Rule VM ABI 8."
            }
            "public_card_kind_count" => {
                "Read-only `public_card_kind_count(card_kind)` query: number of physical matching cards in the shared draw and discard piles. Requires Rule VM ABI 11."
            }
            "field_equipment_kind_count" => {
                "Read-only `field_equipment_kind_count(card_kind)` query: number of physical matching cards in living players' equipment zones. Requires Rule VM ABI 102."
            }
            "hand_equip_suit_count" => {
                "Read-only `hand_equip_suit_count(player, suit)` query: number of cards in that player's hand/equipment region with the requested effective suit. Requires Rule VM ABI 229."
            }
            "round_number_capped" => {
                "Read-only `round_number_capped(cap)` query: the current round number limited to the supplied non-negative cap. Requires Rule VM ABI 75."
            }
            "alive_female_count" => {
                "Query-time modifier value: the number of alive players whose character sex is female."
            }
            "alive_wounded_count_zero_or_at_least_four" => {
                "Query-time modifier value: one while no alive player is wounded or at least four alive players are wounded, otherwise zero."
            }
            "alive_other_count_capped_four" => {
                "Query-time modifier value: the number of other alive players, capped at four."
            }
            "alive_other_qun_count" => {
                "Query-time modifier value: the number of other alive players whose character group is qun."
            }
            "round_number_capped_five" => {
                "Query-time modifier value: the current round number, capped at five."
            }
            "round_at_least_alive_qun_count" => {
                "Query-time modifier value: one while the round number is at least the number of alive qun characters, otherwise zero."
            }
            "round_below_alive_qun_count" => {
                "Query-time modifier value: one while the round number is below the number of alive qun characters, otherwise zero."
            }
            "alive_field_rank_thirteen_player_count" => {
                "Query-time modifier value: number of alive players with at least one rank-K card in their equip or judge zone."
            }
            "field_mount_card_count_ceil_half" => {
                "Query-time modifier value: half the mount cards on living players' fields, rounded up."
            }
            "queried_player_hand_gt_hp_signed" => {
                "Global attack-range modifier value: positive one when the queried player's hand count exceeds HP, otherwise negative one."
            }
            "queried_player_judge_nonempty_other_than_owner" => {
                "Global incoming-damage aura value: one when the queried living player differs from the owner and has a judge-zone card, otherwise zero."
            }
            "queried_player_linked_other_than_owner" => {
                "Global incoming-damage aura value: one when the queried living player differs from the owner and is linked, otherwise zero."
            }
            "queried_player_linked_first_living_skill_owner" => {
                "Global aura value: one when the queried living player is linked and this modifier belongs to the first living enabled owner of its skill definition; otherwise zero. Requires Rule VM ABI 213."
            }
            "queried_player_mark_positive_2172383355" => {
                "Parameterized global-aura value: one while the queried living player has the stable mark id in the numeric suffix; otherwise zero. Requires Rule VM ABI 215."
            }
            "choose_link_other" => {
                "Effect: forces the owner to choose another living unlinked player and link them. Requires Rule VM ABI 213."
            }
            "lord_only" => {
                "Skill declaration marker: the skill is active only while its owner has the Lord identity."
            }
            "non_gainable" => {
                "Skill declaration marker: the skill cannot be copied or gained by another skill."
            }
            "conversion" => {
                "Skill declaration marker: the skill has a generic togglable conversion branch."
            }
            "alive_mounted_player_count_min_one" => {
                "Query-time modifier value: the number of alive players with at least one offensive or defensive mount equipped, with a minimum of one."
            }
            "alive_wounded_or_guiming_wu_count_min_one" => {
                "Query-time modifier value: living wounded-player count, plus other living Wu players while the owner is the Lord and has enabled Guiming; minimum one. Requires Rule VM ABI 145."
            }
            "sha_use_limit" => {
                "Modifier target: adjusts proactive 杀 uses allowed during the owner's turn."
            }
            "sha_use_cancel" => {
                "Modifier target: prevents the owner from proactively using 杀 when non-zero."
            }
            "card_use_unlimited" => {
                "Modifier target: removes ordinary use limits for all cards, or for cards matching an optional name/category/suit/color selector."
            }
            "first_hand_card_suit_use_unlimited" => {
                "Modifier target: during the owner's turn, the first physical hand card of each effective suit ignores and does not consume ordinary use limits. Requires Rule VM ABI 507."
            }
            "add" => {
                "Modifier mode: adds to the base value.\n\n```\nmodifier distance_from add -1;\n```"
            }
            "override" => "Modifier mode: overrides the base value entirely.",
            "draw_cards" => {
                "Effect: draws N cards for the specified player.\n\n```\neffect draw_cards(self, 1);\n```"
            }
            "draw_each_by_seat" => {
                "Effect: both distinct living players draw the same count in seat order.\n\n```\neffect draw_each_by_seat(event_player, event_source, 1);\n```"
            }
            "draw_then_give_one" => {
                "Effect: draws first, resolves draw triggers, then forces one card from hand/equipment to the target. Use `drawn_only` to restrict the gift to cards drawn by this activation, `any_hand_equip` for any current card, or `any_hand_equip_use_equipment` to let the recipient optionally use gifted equipment after movement triggers. The last mode requires Rule VM ABI 227.\n\n```\neffect draw_then_give_one(self, event_source, 2, drawn_only);\n```"
            }
            "draw_then_take_one" => {
                "Effect: draws first, resolves draw triggers, then lets a living taker choose one of the drawer's hand/equipment cards to gain.\n\n```\neffect draw_then_take_one(self, event_source, 3);\n```"
            }
            "draw_then_choose_give_or_discard_hand" => {
                "Active-only staged effect: draws `draw_count` cards, waits for every nested draw trigger, then forces the owner to give exactly `payment_count` hand cards to another living player or discard that many hand cards. If fewer cards remain, every currently discardable hand card is discarded. Skill ownership, living players, target legality, and hand-card ownership are revalidated at every choice barrier. Requires Rule VM ABI 116.\n\n```\neffect draw_then_choose_give_or_discard_hand(self, 2, 2);\n```"
            }
            "choose_give_or_discard_hand" => {
                "Effect: if the owner has at least `payment_count` hand cards, forces a choice to give exactly that many to one currently hand-empty living player or discard that many. Recipient scope and card ownership are revalidated at every choice barrier. Requires Rule VM ABI 581.\n\n```\neffect choose_give_or_discard_hand(self, 2, hand_empty);\n```"
            }
            "choose_discard_hand_equip_then_recover_target" => {
                "Effect: lets the chooser cancel or discard exactly `count` current hand/equipment cards, then recovers the specified living wounded target by `amount` after movement triggers finish. Skill ownership, target state, and selected cards are revalidated when the choice resolves. Requires Rule VM ABI 582.\n\n```\neffect choose_discard_hand_equip_then_recover_target(self, event_player, 2, 1);\n```"
            }
            "take_one_card" => {
                "Effect: lets the recipient choose and gain one card from the source. Scope is `hand_equip`, `equip`, or `hand_equip_judge`; the same scope and gain protection are revalidated when resolving the choice. `hand_equip_judge` requires Rule VM ABI 334.\n\n```\neffect take_one_card(event_player, self, hand_equip_judge);\n```"
            }
            "take_one_event_target_hand" => {
                "Effect: lets the player choose and gain one hand card from the first counterpart carried by the triggering event. Requires Rule VM ABI 231.\n\n```\neffect take_one_event_target_hand(self);\n```"
            }
            "choose_end_extra_draw_or_play_by_used_suits" => {
                "Effect: after the current End phase, choose a complete extra Draw or Play phase; when distinct effective suits among cards used this turn reach current HP, execute both in Draw-then-Play order. Requires Rule VM ABI 232.\n\n```\neffect choose_end_extra_draw_or_play_by_used_suits(self);\n```"
            }
            "lose_hp_double_current_damage_then_replace" => {
                "Damage-before effect: lose HP equal to the bound current damage, wait for that HP-loss resolution, double the bound damage, then permanently replace the originating skill. Requires Rule VM ABI 233.\n\n```\neffect lose_hp_double_current_damage_then_replace(self, 3481582878);\n```"
            }
            "take_target_hand_then_repeat_until_max" => {
                "Active-only effect: take one hand card from the mandatory selected target, then after each complete movement-trigger chain optionally choose another player whose hand is still larger. Stops once the owner ties for the largest living hand or cancels. Requires `targets 1 1 other hand_gt_owner` and Rule VM ABI 113.\n\n```\neffect take_target_hand_then_repeat_until_max(self, event_source, self_skill);\n```"
            }
            "gain_event_cards" => {
                "Effect: gains every physical card captured by the triggering event that is still in the discard pile. Requires Rule VM ABI 82.\n\n```\neffect gain_event_cards(self);\n```"
            }
            "gain_card_if_discard" => {
                "Effect: gains one physical card only if it is still in the shared discard pile. Requires Rule VM ABI 151.\n\n```\neffect gain_card_if_discard(self, event_card);\n```"
            }
            "choose_draw_or_discard_target_card_record_sha" => {
                "Effect: lets the current Sha user cancel, draw one card, or discard one current target hand/equipment card. Accepting either branch binds this skill and target to the exact resolving Sha. Requires Rule VM ABI 152.\n\n```\neffect choose_draw_or_discard_target_card_record_sha(self, event_card, event_player);\n```"
            }
            "gain_target_equipment" => {
                "Effect: gains every card currently equipped by `target` in the requested slot. The target zone and slot membership are revalidated at effect resolution. Slots are `weapon`, `armor`, `horse_def`, `horse_off`, and `treasure`. Requires Rule VM ABI 117.\n\n```\neffect gain_target_equipment(self, event_source, weapon);\n```"
            }
            "discard_all_target_equipment" => {
                "Effect: discards every equipment card that `actor` may currently discard from `target`. Target living state, equipment-zone ownership, and discard protection are revalidated at resolution; no eligible equipment is a deterministic no-op. Requires Rule VM ABI 118.\n\n```\neffect discard_all_target_equipment(self, event_player);\n```"
            }
            "increase_current_sha_target_damage" => {
                "Effect: revalidates the suspended proactive Sha source/card/current-target tuple and increases only that target's saved damage. Other targets of the same Sha are unchanged. Available only in `card_targeted` triggers and requires Rule VM ABI 119.\n\n```\neffect increase_current_sha_target_damage(self, event_card, event_player, 1);\n```"
            }
            "turn_over" => {
                "Effect: toggles a living player's character card. A face-down player skips their next whole turn and returns face-up.\n\n```\neffect turn_over(self);\n```"
            }
            "restore_character_state" => {
                "Effect: restores a living character card to its normal state without toggling an already-normal state: remove linked state and turn face-up if needed. Requires Rule VM ABI 93.\n\n```\neffect restore_character_state(self);\n```"
            }
            "discard_all_hand_equip_judge" => {
                "Effect: forcibly discards every card in a living player's hand, equipment, and judgment zones. Requires Rule VM ABI 93.\n\n```\neffect discard_all_hand_equip_judge(self);\n```"
            }
            "damage_then_discard_all_judge" => {
                "Effect: deals one ordinary self-damage, waits for the complete damage, dying, and save chain, then discards every card currently in the player's judgment zone. Requires Rule VM ABI 224.\n\n```\neffect damage_then_discard_all_judge(self);\n```"
            }
            "replace_draw_gain_top_two_add_skills_if_different_colors" => {
                "Effect: replaces the pending normal draw, publicly reveals and gains the top two cards, waits for gain movement triggers, then grants both configured skills until turn end only when the two effective colors differ. Requires Rule VM ABI 225.\n\n```\neffect replace_draw_gain_top_two_add_skills_if_different_colors(self, first_skill_def, second_skill_def);\n```"
            }
            "choose_gain_target_card_alternate_zone_then_target_draw" => {
                "Active-only effect: choose and gain one current hand/equipment card from the selected other target, excluding the zone already used by this skill in the current Play phase, then let that target draw one card after movement triggers finish. Requires `targets 1 1 other hand_equip_nonempty` and Rule VM ABI 226.\n\n```\neffect choose_gain_target_card_alternate_zone_then_target_draw(self, event_source);\n```"
            }
            "choose_discard_target_card_resolve_current_sha" => {
                "Effect: forces the current Sha user to discard one hand/equipment card from the current target. Equipment makes only that target unable to respond; a non-equipment card makes the target gain the physical Sha if it remains in the discard pile. Available only in `card_targeted` triggers and requires Rule VM ABI 94.\n\n```\neffect choose_discard_target_card_resolve_current_sha(self, event_card, event_player);\n```"
            }
            "choose_discard_target_card_resolve_current_sha_nonbasic_bonus" => {
                "Effect: forces the current Sha user to discard one hand/equipment card from the current target. A non-basic card makes only that target unable to respond, grants the supplied modifier skill, and adds one matching mark; a basic card makes the target gain the physical Sha if it remains in the discard pile. Available only in `card_targeted` triggers and requires Rule VM ABI 312.\n\n```\neffect choose_discard_target_card_resolve_current_sha_nonbasic_bonus(self, event_card, event_player, bonus_skill_def);\n```"
            }
            "remove_skill_by_def" => {
                "Effect: removes every instance of one stable skill definition from the specified player.\n\n```\neffect remove_skill_by_def(self, skill_def);\n```"
            }
            "add_bound_skill" => {
                "Effect: grants the specified skill definition to a living player and binds that skill instance to one living target. Repeated grants update the same instance's bound target. Requires Rule VM ABI 130.\n\n```\neffect add_bound_skill(self, skill_def, event_source);\n```"
            }
            "replace_skill_with_bound_after_resolution_if_target_alive" => {
                "Effect: after the current effect, trigger, and dying/save resolution finishes, replaces one skill instance with a helper bound to `target` only if that target remains alive. Requires Rule VM ABI 132.\n\n```\neffect replace_skill_with_bound_after_resolution_if_target_alive(self, self_skill, helper_skill_def, event_source);\n```"
            }
            "choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp" => {
                "Effect: forces the owner to choose a nonempty hand or equipment zone, discards every discardable card in that zone, waits for movement triggers, draws the discarded count plus current lost HP, waits for draw triggers, then loses 1 HP. Requires Rule VM ABI 133.\n\n```\neffect choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp(self);\n```"
            }
            "choose_discard_hand_add_skill_by_color" => {
                "Effect: optionally discards one hand card, then grants the configured red- or black-discard skill using the card's effective color.\n\n```\neffect choose_discard_hand_add_skill_by_color(self, red_skill_def, black_skill_def);\n```"
            }
            "choose_discard_matching_equip_color_add_skill" => {
                "Effect: optionally discards one non-basic hand/equipment card whose effective color matches at least one of the player's equipped cards, then grants or updates a color-lock skill on the target. Repeated grants merge colors and replace the recorded source. Requires Rule VM ABI 48.\n\n```\neffect choose_discard_matching_equip_color_add_skill(self, event_player, skill_def);\n```"
            }
            "use_virtual_sha_from_card_color_lock_source_if_no_sha" => {
                "Effect: if the owner used no Sha in the current play phase, the source recorded in this skill instance uses a suitless virtual Sha on the owner while ignoring distance and ordinary Sha-use count. Requires Rule VM ABI 48.\n\n```\neffect use_virtual_sha_from_card_color_lock_source_if_no_sha(self);\n```"
            }
            "choose_current_card_use_enhancements" => {
                "Effect: for the suspended one-target Sha or ordinary trick, forces the user to choose one or two of: add one legal target, ignore armor, become unrespondable, or draw after each damage event. Exact-card policies are stored in the supplied temporary helper and revalidated at each choice barrier. Requires Rule VM ABI 49.\n\n```\neffect choose_current_card_use_enhancements(self, event_card, helper_skill_def);\n```"
            }
            "make_current_card_unrespondable" => {
                "Effect: records the suspended physical card in the supplied temporary helper so only that exact use skips Shan and Wuxie response windows. The helper is created or merged without affecting later cards. Requires Rule VM ABI 220.\n\n```\neffect make_current_card_unrespondable(self, event_card, helper_skill_def);\n```"
            }
            "choose_draw_or_discard_add_skill" => {
                "Effect: forces the skill owner to choose between gaining the draw helper then drawing one card, or gaining the discard helper then discarding exactly one hand/equipment card. The discard branch is omitted when it has no legal cost, and both stages are revalidated. Requires Rule VM ABI 50.\n\n```\neffect choose_draw_or_discard_add_skill(self, draw_skill_def, discard_skill_def);\n```"
            }
            "choose_three_way_draw_or_discard_add_skill" => {
                "Effect: lets the skill owner cancel or choose between drawing one card, gaining the draw helper then drawing two cards, or discarding exactly one hand/equipment card then gaining the discard helper. The discard branch is omitted when it has no legal cost, and both stages are revalidated. Requires Rule VM ABI 315.\n\n```\neffect choose_three_way_draw_or_discard_add_skill(self, draw_skill_def, discard_skill_def);\n```"
            }
            "choose_discarded_black_basic_as_bingliang" => {
                "Effect: captures the full triggering discard batch, repeatedly offers its still-discarded effective black basic cards, and uses each selected physical card as a distance-unlimited Bingliang through ordinary target, CardUsed, Wuxie, and delayed-trick settlement. Each iteration is cancellable and revalidated. Requires Rule VM ABI 51.\n\n```\neffect choose_discarded_black_basic_as_bingliang(self);\n```"
            }
            "choose_discard_judge_use_virtual_sha_draw_if_no_damage" => {
                "Effect: optionally discards one judgment card from the target, then uses a fixed suitless virtual Sha without consuming the ordinary use count. If that exact Sha is illegal or commits no damage, the owner draws one card with skill provenance. Requires Rule VM ABI 51.\n\n```\neffect choose_discard_judge_use_virtual_sha_draw_if_no_damage(self, event_player);\n```"
            }
            "discard_selected_record_unique_category_target" => {
                "Effect: atomically records the selected card's effective basic/trick/equipment category and the selected target in the originating skill's turn- or play-phase-scoped state, then discards that selected hand/equipment card. Requires the matching uniqueness clauses.\n\n```\neffect discard_selected_record_unique_category_target(self, event_source);\n```"
            }
            "choose_target_draw_or_discard_by_skill_use_count_link_if_wounded" => {
                "Effect: lets the owner choose whether the target draws or discards the originating active skill's current turn use count. The target chooses forced discard cards; after nested effects finish, the owner becomes linked if the target is wounded. Requires Rule VM ABI 52.\n\n```\neffect choose_target_draw_or_discard_by_skill_use_count_link_if_wounded(self, event_source);\n```"
            }
            "choose_target_draw_or_discard_by_skill_use_count_toggle_link_if_wounded" => {
                "Effect: lets the owner choose whether the target draws or discards the originating active skill's current turn use count. The target chooses forced discard cards; after nested effects finish, the owner's linked state toggles if the target is wounded. Requires the exact play-phase uniqueness contract and Rule VM ABI 314.\n\n```\neffect choose_target_draw_or_discard_by_skill_use_count_toggle_link_if_wounded(self, event_source);\n```"
            }
            "choose_reveal_other_hand_card_remember_category" => {
                "Effect: forces the owner to choose another living player with hand cards, choose and publicly reveal one of those cards, and remember its effective basic/trick/equipment category for the current Play phase. Requires Rule VM ABI 53.\n\n```\neffect choose_reveal_other_hand_card_remember_category(self);\n```"
            }
            "draw_then_give_exact_record_phase_target" => {
                "Effect: draws `count` cards, then forces the owner to give exactly that many current hand/equipment cards to the other current Play-phase player and records that target for delayed phase logic. Requires Rule VM ABI 53.\n\n```\neffect draw_then_give_exact_record_phase_target(self, event_player, 2);\n```"
            }
            "clear_phase_gift_target" => {
                "Effect: clears the originating skill instance's delayed Play-phase gift target. Requires Rule VM ABI 53.\n\n```\neffect clear_phase_gift_target(self);\n```"
            }
            "choose_gain_resolved_sha_or_shan_then_optional_use_sha" => {
                "Effect: gives a legal non-owner/non-original-target recipient either the resolved physical Sha or all captured physical Shan cards still in discard. A red gift lets the recipient optionally use a physical or physical view-as Sha without ordinary use count; targets must be in the original owner's attack range and cannot include that owner. Requires Rule VM ABI 54.\n\n```\neffect choose_gain_resolved_sha_or_shan_then_optional_use_sha(self, event_card);\n```"
            }
            "choose_gain_all_resolved_sha_shan_then_color_rewards" => {
                "Effect: gives one legal non-owner/non-original-target recipient every resolved physical Sha and captured physical Shan source still in discard. A red gift lets the recipient optionally use Sha without ordinary use count; a black gift makes that recipient draw one card. Follow-up Sha targets use the original owner's attack range and cannot include that owner. Requires Rule VM ABI 310.\n\n```\neffect choose_gain_all_resolved_sha_shan_then_color_rewards(self, event_card);\n```"
            }
            "skip_current_phase" => {
                "Effect: skips the current player's entered non-idle phase, records the skipped-phase audit event/count, and advances through the ordinary phase lifecycle. Prepare/Judge/End support requires Rule VM ABI 388.\n\n```\neffect skip_current_phase(self);\n```"
            }
            "finish_current_phase_abort_resolution" => {
                "Effect: discards the shared processing area, aborts the remaining current resolution, and finishes the current phase without recording it as skipped. Requires Rule VM ABI 286.\n\n```\neffect finish_current_phase_abort_resolution(self);\n```"
            }
            "gain_top_card_to_skill_pile" => {
                "Effect: moves the draw-pile top card into the originating skill's public skill pile. Requires Rule VM ABI 287.\n\n```\neffect gain_top_card_to_skill_pile(self);\n```"
            }
            "choose_discard_skill_pile_card_add_marked_skill" => {
                "Effect: chooses and discards one card from the originating skill's public pile, then grants the target a helper skill and adds one same-id mark after movement triggers. Requires Rule VM ABI 287.\n\n```\neffect choose_discard_skill_pile_card_add_marked_skill(self, event_player, helper_skill_def);\n```"
            }
            "gain_one_hand_from_selected_targets" => {
                "Effect: gains one current hand card from each active-skill target in submitted order. Requires `targets ... other max_by_owner_hp hand_nonempty` and Rule VM ABI 288.\n\n```\neffect gain_one_hand_from_selected_targets(self);\n```"
            }
            "choose_repolu_targets_draw" => {
                "Death-trigger effect: the owner, including a dead owner, may cancel or choose one or more living recipients. A successful choice increments the originating skill's same-id mark once, then each selected recipient draws the previous mark plus one card in seat order. Requires Rule VM ABI 291.\n\n```\neffect choose_repolu_targets_draw(self);\n```"
            }
            "choose_other_targets_by_skipped_phase_count_damage" => {
                "Effect: forces the owner to choose one to X distinct living other players, where X is this turn's current skipped-phase count, then deals the supplied amount to each in submitted order. Requires Rule VM ABI 55.\n\n```\neffect choose_other_targets_by_skipped_phase_count_damage(self, 1);\n```"
            }
            "reveal_uniform_hand_draw_self_choose_others" => {
                "Effect: publicly reveals the owner's current hand. If every card has one uniform effective color, the owner chooses zero to the current hand count of other living players; the chosen players and owner each draw one in canonical seat order. Requires Rule VM ABI 56.\n\n```\neffect reveal_uniform_hand_draw_self_choose_others(self);\n```"
            }
            "choose_card_user_or_add_target" => {
                "Effect: chooses another legal target of the suspended single-target Sha/ordinary trick. That player may give one hand card and become the user, or become an extra target. Requires Rule VM ABI 57.\n\n```\neffect choose_card_user_or_add_target(self, event_card);\n```"
            }
            "choose_discard_equipment_reduce_current_damage" => {
                "Effect: forces one effective equipment card to be discarded from hand/equipment, then reduces the bound pre-resolution damage by one. Requires a damage_before trigger and Rule VM ABI 57.\n\n```\neffect choose_discard_equipment_reduce_current_damage(self);\n```"
            }
            "choose_discard_equipment_prevent_current_damage" => {
                "Effect: forces one effective equipment card to be discarded from hand/equipment, then prevents the entire bound pre-resolution damage segment. Requires a damage_before trigger and Rule VM ABI 398.\n\n```\neffect choose_discard_equipment_prevent_current_damage(self);\n```"
            }
            "choose_move_target_equipment_or_draw" => {
                "Effect: lets the owner cancel, draw one, or move one compatible equipment card from the other player into the owner's matching slot, replacing an occupied slot normally. Requires Rule VM ABI 58.\n\n```\neffect choose_move_target_equipment_or_draw(self, event_player);\n```"
            }
            "choose_move_target_equipment_and_or_draw" => {
                "Effect: lets the owner cancel, draw one, move one compatible equipment card from the other player into the owner's matching slot, or choose both available actions in move-then-draw order. Requires Rule VM ABI 146.\n\n```\neffect choose_move_target_equipment_and_or_draw(self, event_player);\n```"
            }
            "choose_target_discard_attack_range_cards_or_increase_damage_weapon_forces_both" => {
                "Effect: the bound direct-Sha damage target discards up to the source's current attack range from hand/equipment or increases the damage and chain base by one. A physical source weapon forces discard followed by the increase, then is discarded. Requires Rule VM ABI 147.\n\n```\neffect choose_target_discard_attack_range_cards_or_increase_damage_weapon_forces_both(self, event_player);\n```"
            }
            "choose_draw_or_recast_add_skill" => {
                "Effect: lets the owner cancel, draw one card and gain `draw_skill_def`, or choose one current hand/equipment card to recast and gain `recast_skill_def`. The branch and card are revalidated. Requires Rule VM ABI 148.\n\n```\neffect choose_draw_or_recast_add_skill(self, draw_skill_def, recast_skill_def);\n```"
            }
            "choose_gain_sha_for_opening_extra_play" => {
                "Effect: only during the turn-opening extra Play phase created by this skill, lets the owner cancel or gain the first physical Sha from the draw pile, then discard pile. The gained card alone ignores distance and accepting arms the phase-end consequence. Requires Rule VM ABI 149.\n\n```\neffect choose_gain_sha_for_opening_extra_play(self);\n```"
            }
            "finish_opening_extra_play_sha_gain" => {
                "Effect: clears this skill's armed exact-card policy and deals 1 self-damage if the owner dealt no damage after accepting the opening-phase Sha. Requires Rule VM ABI 149.\n\n```\neffect finish_opening_extra_play_sha_gain(self);\n```"
            }
            "reveal_target_hand_gain_shan_armor_or_draw_two" => {
                "Effect: publicly reveals one nonempty target hand, gives every effective Shan and physical armor to the skill owner in hand order, or draws two when none match. Requires `targets 1 1 other hand_nonempty` and Rule VM ABI 150.\n\n```\neffect reveal_target_hand_gain_shan_armor_or_draw_two(self, event_source);\n```"
            }
            "give_all_equipment_choose_recover_or_damage" => {
                "Effect: gives all current equipment to one recipient. After movement triggers, that recipient may cancel target selection to recover the owner by the captured count, or damage one to that many distinct players in current attack range. Requires Rule VM ABI 58.\n\n```\neffect give_all_equipment_choose_recover_or_damage(self, event_source);\n```"
            }
            "choose_target_discard_weapon_range_hand_or_increase_damage" => {
                "Effect: the bound Sha damage target may discard exactly the source's physical weapon range in hand cards. If the target cannot or cancels, the damage and its chain base increase by one; if paid, the selected hand cards and the source's current physical weapon are discarded in that order. Requires Rule VM ABI 59.\n\n```\neffect choose_target_discard_weapon_range_hand_or_increase_damage(self, event_player);\n```"
            }
            "choose_target_discard_distance_hand_or_increase_damage" => {
                "Effect: the bound damage target may discard exactly `max(1, 4-distance from source)` current hand cards. If the target has fewer cards or cancels, the current damage and chain base increase by one. Requires Rule VM ABI 322.\n\n```\neffect choose_target_discard_distance_hand_or_increase_damage(self, event_player);\n```"
            }
            "choose_draw_view_two_gain_or_move_equipment" => {
                "Effect: lets the owner cancel, draw one card, privately inspect up to two deterministically sampled current hand cards from the other player and gain one, or move one compatible equipment card into the owner's matching slot. Requires Rule VM ABI 323.\n\n```\neffect choose_draw_view_two_gain_or_move_equipment(self, event_player);\n```"
            }
            "gain_from_skill_pile" => {
                "取用（expansion P1，ADR 0641）：从自己的技能牌堆（武将牌上）选一张牌加入手牌。参数 (player, self_skill)。"
            }
            "gain_whole_skill_pile" => {
                "Effect: moves every card from your own skill pile (the cards placed on your general card) into your hand, creating the pile if needed. No-op when the pile is empty.\n\n```\neffect gain_whole_skill_pile(self, self_skill, 1234567890);\n```"
            }
            "place_hand_cards_to_deck_top" => {
                "Effect: select cards from your hand and place them on top of the draw pile.\n\n```\neffect place_hand_cards_to_deck_top(self, self_skill, 2);\n```"
            }
            "rearrange_pile_to_deck_top" => {
                "Effect: select count cards from the specified skill pile and place them on top of the draw pile.\n\n```\neffect rearrange_pile_to_deck_top(self, self_skill, 1234567890, 3);\n```"
            }
            "reset_linked_selected_targets" => {
                "Effect: resets every selected linked target, then deals 1 fire damage to each (duansuo).\n\n```\neffect reset_linked_selected_targets(self);\n```"
            }
            "choose_hand_cards_to_skill_pile" => {
                "Effect: select between min and max cards from your hand and place them into your own skill pile.\n\n```\neffect choose_hand_cards_to_skill_pile(self, self_skill, 1, hand_count(self));\n```"
            }
            "choose_target_hand_card_to_pile" => {
                "Effect: pick another player with hand cards, then one of their hand cards, and move it into your own skill pile (the cards on your general card).\n\n```\neffect choose_target_hand_card_to_pile(self, self_skill, 1234567890);\n```"
            }
            "choose_one_from_skill_pile_discard" => {
                "Effect: lets you pick one card from your own skill pile and discard it. No-op when the pile is empty.\n\n```\neffect choose_one_from_skill_pile_discard(self, self_skill, 1234567890);\n```"
            }
            "place_top_deck_to_expansion" => {
                "Effect: places the top N cards of the draw pile into your own skill pile (the cards placed on your general card), creating the pile if needed.\n\n```\neffect place_top_deck_to_expansion(self, self_skill, 1234567890, 2);\n```"
            }
            _ => return super::skill_resolvers::documentation(word),
    })
}
