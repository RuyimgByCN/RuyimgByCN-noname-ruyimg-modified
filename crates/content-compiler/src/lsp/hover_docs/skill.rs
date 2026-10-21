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
            "owner_missing_skill" => {
                "Active-skill clause: activation is legal only while the owner does not have the given stable skill definition. Requires Rule VM ABI 234."
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
            "resolve_pothaoshi" => {
                "Effect: action 0 optionally authorizes another living player to use/respond with the owner's current hand until action 1 clears all grants at the owner's next turn start. The first two borrowed uses that empty the owner's hand draw it to three. Requires Rule VM ABI 536.\n\n```\neffect resolve_pothaoshi(self, 0);\n```"
            }
            "resolve_spmiaolve" => {
                "Effect: action 0 creates up to two globally suit-distinct Mantianguohai cards in the owner's hand; action 1 lets the owner cancel, draw two, or gain one canonical zhinang. Requires Rule VM ABI 539.\n\n```\neffect resolve_spmiaolve(self, 0);\n```"
            }
            "resolve_pottuntian" => {
                "势屯田 effect: gain charge after a qualifying loss, grow the maximum and draw when full at turn start, or spend charge to distribute heart cards and trigger 势凿险 threshold rewards. Requires Rule VM ABI 543."
            }
            "resolve_potjixi" => {
                "势急袭 effect: reset/record current-turn card targets, then discard one card from the turn player and use an ignore-distance virtual Shunshou on selected recorded targets. Requires Rule VM ABI 543."
            }
            "resolve_potceduan" => {
                "势策断 effect: collect one hidden hand-card choice from every player in the selected target's attack range, reveal them together, and use all owner hand cards of every tied majority color as an unlimited physical Sha. Requires Rule VM ABI 544."
            }
            "resolve_pottaishici" => {
                "势太史慈 effect: reset/collect 战烈 marks, resolve its enhanced virtual Sha, draw-to-X then duel for 酣战, and choose the limited 振锋 recovery or X-source rewrite. Requires Rule VM ABI 548."
            }
            "resolve_mbzhonghui" => {
                "势钟会 effect: spend/settle 俟资 charge, grow 挟志 limits, awaken 狱难, and upgrade 克昌. Requires Rule VM ABI 553."
            }
            "resolve_potjiejie" => {
                "势辛宪英诫节 effect: privately reveal the current actor hand, choose a suit, apply the matching discard/unlimited-use or draw-pile gain branch, and track the twice-per-round unique-maximum 清识 bonus. Requires Rule VM ABI 554."
            }
            "resolve_potqingshi" => {
                "势辛宪英清识 effect: choose any living player; allied participants each draw one, otherwise both discard one hand/equipment card. Requires Rule VM ABI 554."
            }
            "resolve_potweiyan" => {
                "势魏延 effect: resolve 忠傲 mission state, 壮誓 first-N-card helpers, 饮战 Sha follow-up, upgraded 狂骨, or 困奋. Requires Rule VM ABI 558."
            }
            "resolve_potzhouyu" => {
                "势周瑜 effect: resolve 炽沄 gifts, 焰洄 reveal history, 焚涛 linked fire-damage choices, or 雄姿 branch locking. Requires Rule VM ABI 561."
            }
            "resolve_potlvyi" => {
                "势吕壹 effect: resolve 隳律 tracked-card settlement, 颂谳 Wuzhong response conversion, or 势施 exact-card response locks. Requires Rule VM ABI 564."
            }
            "resolve_potcaoshuang" => {
                "势曹爽 effect: resolve 典易/奢权 timing assignment, persistent card tags, 渐专 awakening, and 溺尊 pile use. Requires Rule VM ABI 568."
            }
            "resolve_potyuji" => {
                "势于吉 effect: resolve 道转 basic-card conversion costs and 符济 tagged-card rewards. Requires Rule VM ABI 570."
            }
            "resolve_rehengjiang" => {
                "SP 横江 effect: stack the current turn player's temporary hand-limit penalty and settle the owner's draw at turn end. Requires Rule VM ABI 572."
            }
            "owner_alive_turn_ended_rehengjiang_pending" => {
                "横江 turn-end filter: the living helper owner has pending state for the ended turn. Requires Rule VM ABI 572."
            }
            "owner_gained_cards_potzhouyu" => {
                "炽沄 gain filter: the living owner gained cards and still has hand cards. Requires Rule VM ABI 561."
            }
            "owner_source_first_target_has_hand_potyanhui" => {
                "焰洄 target filter: the owner used a card, this is its first target event, and some target has hand cards. Requires Rule VM ABI 561."
            }
            "owner_alive_fire_damage_potfentao" => {
                "焚涛 fire-damage filter; target and chain eligibility are revalidated by the resolver. Requires Rule VM ABI 561."
            }
            "owner_alive_potfentao_pending_damage" => {
                "焚涛 post-damage filter for its pending relink target. Requires Rule VM ABI 561."
            }
            "owner_source_sha_damage_before_potyinzhan_condition" => {
                "饮战 Damage-before filter: the owner dealt Sha damage and has no more HP or hand cards than the target. Requires Rule VM ABI 558."
            }
            "owner_source_damage_within_one" => {
                "狂骨 Damage filter: the owner dealt positive damage to a living target at distance 1 or less. Requires Rule VM ABI 558."
            }
            "owner_alive_play_phase_actor_hand_nonempty_potjiejie_unused" => {
                "诫节 Play-phase filter: owner and current actor are alive, the actor has hand cards, and no 诫节 has been invoked by that actor this phase. Requires Rule VM ABI 554."
            }
            "potqingyan" => {
                "View-as cost: reveal exactly min(round uses + 1, 5) hand cards without consuming them; unavailable while any card shown by the previous activation remains in hand. Requires Rule VM ABI 544."
            }
            "owner_lost_non_damage_card_pottuntian_charge_not_full" => {
                "Cards-moved filter: the owner lost at least one non-damage card from a personal zone, excluding an equipment card consumed by using it, while 势屯田 charge is below its stored maximum. Requires Rule VM ABI 543."
            }
            "resolve_spyingjia" => {
                "Effect: actions 0/1 reset and record same-name trick uses; action 2 may discard one hand card and queue an extra turn for any living target, granting the supplied draw helper. Requires Rule VM ABI 539.\n\n```\neffect resolve_spyingjia(self, 2, 3612952432);\n```"
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
            "owner_in_target_attack_range" => {
                "Active-target constraint: the skill owner must currently be inside every selected target's attack range. Requires Rule VM ABI 544."
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
            "owner_hp" => {
                "Query-time modifier value: the skill owner's current HP, clamped to zero."
            }
            "owner_max_hp" => "Query-time modifier value: the skill owner's maximum HP.",
            "owner_hp_minus_one" => {
                "Query-time modifier value: the skill owner's current HP minus one, clamped to zero."
            }
            "owner_lost_hp" => {
                "Query-time modifier value: the skill owner's current lost HP, clamped to zero."
            }
            "owner_lost_hp_minus_one" => {
                "Query-time modifier value: the skill owner's current lost HP minus one, clamped to zero."
            }
            "owner_lost_hp_min_one" => {
                "Query-time modifier value: the skill owner's current lost HP, with a minimum of one."
            }
            "owner_skill_mark" => {
                "Query-time modifier value: the owner's mark count whose id equals this skill definition id."
            }
            "owner_skill_state_u32" => {
                "Query-time modifier value: the first little-endian u32 in this skill instance's state blob. ABI 522."
            }
            "owner_skill_state_u32_negative" => {
                "Query-time modifier value: the negative first little-endian u32 in this skill instance's state blob. ABI 612."
            }
            "queried_player_qun_owner_lord_alive" => {
                "Query-time modifier value: one for a living Qun card user while the living modifier owner is the Lord. ABI 553."
            }
            "owner_mark_2058376721" => {
                "Parameterized query-time modifier value: the owner's mark count whose stable definition id is the numeric suffix."
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
            "owner_has_weapon" => {
                "Query-time modifier value: one while the skill owner's equipment zone contains a weapon, otherwise zero."
            }
            "owner_no_weapon" => {
                "Query-time modifier value: one while the skill owner's equipment zone lacks a weapon, otherwise zero."
            }
            "owner_has_armor_equip" => {
                "Query-time modifier value: one while the skill owner's equipment zone contains an armor, otherwise zero."
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
            "owner_unique_max_hand" => {
                "Query-time modifier value: one while the skill owner uniquely has the most hand cards among living players, otherwise zero."
            }
            "owner_hand_gt_other" => {
                "Contextual damage modifier value: one while the skill owner's hand count is greater than the other damage participant's, otherwise zero."
            }
            "owner_hand_lt_other" => {
                "Contextual damage modifier value: one while the skill owner's hand count is less than the other damage participant's, otherwise zero."
            }
            "owner_qun_other_shu" => {
                "Contextual damage modifier value: one while the skill owner is a living qun character and the other damage participant is a living shu character, otherwise zero. Requires Rule VM ABI 81."
            }
            "owner_turned_over_diff_other" => {
                "Contextual damage modifier value: one while the skill owner's face-up/face-down state differs from the other damage participant's, otherwise zero."
            }
            "owner_linked_diff_other" => {
                "Contextual damage modifier value: one while the skill owner and the other damage participant have different linked states, otherwise zero."
            }
            "owner_hand_equip_gte_other" => {
                "Contextual damage modifier value: one while the skill owner's hand and equipment counts are each no smaller than the other damage participant's corresponding count, otherwise zero."
            }
            "owner_sha_damage_dealt_zero_this_turn" => {
                "Turn-scoped damage modifier value: one before the skill owner has dealt any logical-Sha damage event in the current turn, otherwise zero."
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
            "resolve_nzry_longnu" => {
                "Effect: toggles Longnu, pays the matching HP/max-HP cost, draws one card, and grants the matching helper. Requires Rule VM ABI 213."
            }
            "choose_link_other" => {
                "Effect: forces the owner to choose another living unlinked player and link them. Requires Rule VM ABI 213."
            }
            "resolve_drlt_poxi" => {
                "Effect: privately exposes the selected target hand, chooses exactly four distinct-suit cards across both hands, discards them, and resolves Poxi by the owner's selected-card count. Requires Rule VM ABI 214."
            }
            "resolve_drlt_jieying" => {
                "Effect: resolves Jieying's ensure-mark, optional transfer, or other-player turn-end hand collection mode. Requires Rule VM ABI 215."
            }
            "resolve_shen_simayi" => {
                "Effect: resolves Shen Simayi's discard-phase Renjie count, Lianpo extra-turn queue, or a Jilue Wansha/Zhiheng/Jizhi branch selected by mode. Requires Rule VM ABI 218."
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
            "owner_no_mount" => {
                "Query-time modifier value: one while the skill owner's equipment zone contains no mount, otherwise zero."
            }
            "owner_armor_empty" => {
                "Query-time modifier value: one while the skill owner's equipment zone contains no armor, otherwise zero."
            }
            "owner_healthy" => {
                "Query-time modifier value: one while the skill owner is at full health, otherwise zero."
            }
            "owner_hand_empty" => {
                "Trigger filter or query-time modifier value: matches while the skill owner's hand is empty."
            }
            "owner_has_treasure" => {
                "Query-time modifier value: one while the skill owner's equipment zone contains a treasure, otherwise zero."
            }
            "alive_mounted_player_count_min_one" => {
                "Query-time modifier value: the number of alive players with at least one offensive or defensive mount equipped, with a minimum of one."
            }
            "owner_equip_count_odd" => {
                "Query-time modifier value: one while the skill owner's equipment-card count is odd, otherwise zero."
            }
            "owner_equip_count_even" => {
                "Query-time modifier value: one while the skill owner's equipment-card count is even (including zero), otherwise zero."
            }
            "owner_equip_count_half_floor" => {
                "Query-time modifier value: half the skill owner's equipment-card count, rounded down."
            }
            "owner_equip_count_exactly_one" => {
                "Query-time modifier value: one while the skill owner has exactly one equipment card, otherwise zero."
            }
            "owner_equip_count" => {
                "Query-time modifier value: the number of cards in the skill owner's equipment zone."
            }
            "owner_hp_gt_two" => {
                "Query-time modifier value: one while the skill owner's current HP is greater than two, otherwise zero."
            }
            "owner_hp_le_two" => {
                "Query-time modifier value: one while the skill owner's current HP is at most two, otherwise zero."
            }
            "owner_lost_hp_at_least_two" => {
                "Query-time modifier value: one while the skill owner's lost HP is at least two, otherwise zero."
            }
            "owner_all_others_in_attack_range" => {
                "Query-time modifier value: one while every other alive player is within the skill owner's attack range, otherwise zero."
            }
            "owner_current_turn_all_others_distance_at_most_one" => {
                "Query-time modifier value: one during the owner's current turn while every other living player's final distance from the owner is at most one; otherwise zero. Requires Rule VM ABI 281."
            }
            "owner_sha_used_zero" => {
                "Query-time modifier value: one before the first Sha is used in the current turn, otherwise zero."
            }
            "owner_sha_used_nonzero" => {
                "Query-time modifier value: one after the skill owner uses at least one Sha in the current turn, otherwise zero."
            }
            "owner_sha_used_exactly_one" => {
                "Query-time modifier value: one after the skill owner uses exactly one Sha in their current turn, otherwise zero."
            }
            "owner_sha_used_exactly_two" => {
                "Query-time modifier value: one after the skill owner uses exactly two Sha in their current turn, otherwise zero."
            }
            "owner_max_hand_before_skill_fires_this_turn" => {
                "Query-time modifier value: one while the skill owner ties for maximum hand size and this skill has not fired in the current turn, otherwise zero."
            }
            "owner_non_damage_hand_type_count" => {
                "Query-time modifier value: number of distinct basic, trick, and equipment categories among the owner's non-damage hand cards."
            }
            "owner_sha_hand_count" => {
                "Query-time modifier value: number of Sha cards in the skill owner's hand (任城 pxrencheng)."
            }
            "owner_turn_damage_dealt" => {
                "Query-time modifier value: total damage dealt by the skill owner during their current turn."
            }
            "owner_current_turn_card_use_count" => {
                "Query-time modifier value: cards proactively used by the skill owner during their current turn; zero outside that owner's turn. Requires Rule VM ABI 49."
            }
            "owner_play_phase_card_use_count_zero" => {
                "Query-time modifier value: one before the skill owner proactively uses a card in their current Play phase; zero otherwise. Requires Rule VM ABI 115."
            }
            "owner_skill_play_phase_fire_count_below_2" => {
                "Parameterized query-time modifier value: one during the owner's Play phase while this skill instance has fired fewer than the positive numeric suffix; zero otherwise. Requires Rule VM ABI 220."
            }
            "owner_skill_use_count" => {
                "Query-time modifier value: originating active skill invocations in the current turn. Requires Rule VM ABI 143."
            }
            "alive_wounded_or_guiming_wu_count_min_one" => {
                "Query-time modifier value: living wounded-player count, plus other living Wu players while the owner is the Lord and has enabled Guiming; minimum one. Requires Rule VM ABI 145."
            }
            "owner_has_heart_equip" => {
                "Query-time modifier value: one while the owner's equipment zone contains a card whose effective suit is heart."
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
            "resolve_dcqiaomeng" => {
                "Effect: choose one other target of the current black card, then discard one of that target's hand/equipment cards. Gain discarded equipment; discarded trick cards make the current card use unrespondable to all players. Available only in the matching `card_targeted` trigger and requires Rule VM ABI 313.\n\n```\neffect resolve_dcqiaomeng(self, event_card, helper_skill_def);\n```"
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
            "owner_outside_turn" => {
                "View-as filter: matches any legal source card only while the skill owner is not the current turn player. Requires Rule VM ABI 289."
            }
            "owner_alive_is_source" => {
                "Trigger filter: the skill owner is the event source and remains alive after the event. Requires Rule VM ABI 291."
            }
            "choose_repolu_targets_draw" => {
                "Death-trigger effect: the owner, including a dead owner, may cancel or choose one or more living recipients. A successful choice increments the originating skill's same-id mark once, then each selected recipient draws the previous mark plus one card in seat order. Requires Rule VM ABI 291.\n\n```\neffect choose_repolu_targets_draw(self);\n```"
            }
            "resolve_xinfu_guolun" => {
                "Active effect: the owner privately selects and publicly reveals one target hand card, then may cancel or publicly reveal one own hand/equipment card. A successful choice atomically exchanges both cards into the opposite hands; the original owner of the lower physical rank draws one card, while a tie draws none. Requires Rule VM ABI 294.\n\n```\neffect resolve_xinfu_guolun(self, event_source);\n```"
            }
            "owner_alive_shan_used_by_owner_or_during_owner_turn" => {
                "Card-used trigger filter: the owner is alive, the logical card is Shan, and either the owner used it or it was used during the owner's current turn. Requires Rule VM ABI 287."
            }
            "owner_alive_other_phase_play_skill_pile_nonempty" => {
                "Play-phase trigger filter: the owner and entering player are distinct and alive, and the originating skill's public pile is nonempty. Requires Rule VM ABI 287."
            }
            "choose_other_targets_by_skipped_phase_count_damage" => {
                "Effect: forces the owner to choose one to X distinct living other players, where X is this turn's current skipped-phase count, then deals the supplied amount to each in submitted order. Requires Rule VM ABI 55.\n\n```\neffect choose_other_targets_by_skipped_phase_count_damage(self, 1);\n```"
            }
            "owner_is_target_other_hand_nonempty" => {
                "Phase trigger filter: the owner is the entering Play- or Draw-phase player and at least one other living player has a hand card. Requires Rule VM ABI 53."
            }
            "owner_is_target_positive_draw_other_hand_nonempty" => {
                "Draw-phase trigger filter: the living owner is entering an unfixed Draw phase with a positive pending draw count and at least one other living player has a hand card. Requires Rule VM ABI 311."
            }
            "owner_source_first_target_black_card_any_target_other_discardable_hand_equip" => {
                "Card-target trigger filter: the living owner used a black card, this is its first target event, and at least one other target has a discardable hand/equipment card. Requires Rule VM ABI 313."
            }
            "owner_source_play_card_matches_remembered_category" => {
                "Card-used trigger filter: during the owner's Play phase, the used card's effective basic/trick/equipment category matches the category remembered by this skill instance. Requires Rule VM ABI 53."
            }
            "owner_phase_gift_target_is_source" => {
                "Death trigger filter: the killer matches this skill instance's delayed Play-phase gift target. Requires Rule VM ABI 53."
            }
            "owner_phase_gift_target_is_target" => {
                "Play-phase-end trigger filter: the ending phase player matches this skill instance's delayed gift target. Requires Rule VM ABI 53."
            }
            "owner_source_sha_resolved_has_gainable_cards_and_recipient" => {
                "Sha-resolved trigger filter: the living owner used this Sha, at least one physical Sha/Shan group remains in discard, and at least one living recipient is neither the owner nor an original target. Requires Rule VM ABI 54."
            }
            "owner_is_target_tied_max_hand" => {
                "Phase trigger filter: the living owner is the event player and ties for the largest hand among living players. Requires Rule VM ABI 55."
            }
            "owner_is_target_tied_max_hp" => {
                "Phase trigger filter: the living owner is the event player and ties for the highest current HP among living players. Requires Rule VM ABI 55."
            }
            "owner_is_target_tied_max_equip_nonempty" => {
                "Phase trigger filter: the living owner is the event player, has at least one equipment card, and ties for the largest equipment count among living players. Requires Rule VM ABI 55."
            }
            "owner_is_target_skipped_phase_count_positive" => {
                "Phase trigger filter: the living owner is the event player and at least one phase has been skipped this turn. Requires Rule VM ABI 55."
            }
            "owner_discarded_has_hand_once_per_phase" => {
                "Cards-moved trigger filter: the living owner discarded at least one hand/equipment card, still has hand cards, and has not accepted this trigger in the current phase. Requires Rule VM ABI 56."
            }
            "owner_alive_lost_hand_this_turn" => {
                "Phase-end trigger filter: the living owner lost at least one hand/equipment card during the current turn. Requires Rule VM ABI 405."
            }
            "owner_source_first_sha_this_play_phase" => {
                "Card-used trigger filter: the living owner used the first logical Sha of their current Play phase. Requires Rule VM ABI 407."
            }
            "owner_source_damage_target_distance_at_most_one_empty_equip_slot_unused" => {
                "Damage trigger filter: the owner dealt damage to a target within distance 1, has an enabled empty equipment slot, and has not fired this turn. Requires Rule VM ABI 379."
            }
            "owner_source_first_sha_target_this_turn" => {
                "Card-targeted trigger filter: this target belongs to the owner's first logical Sha of the current turn. Requires Rule VM ABI 379."
            }
            "owner_alive_other_phase_play_end_twzhengjian_condition" => {
                "Play-phase-end filter: another living player failed the current Zhengjian branch (no non-basic use or no gain), and can pay unless Zhengjian has awakened. Requires Rule VM ABI 381."
            }
            "owner_source_sha_target_twzhenxi_available_once_per_turn" => {
                "Card-targeted filter: once per turn, the owner's Sha target has discardable hand cards or a field card that can move to another player. Requires Rule VM ABI 382."
            }
            "owner_source_first_target_damage_card_once_per_turn" => {
                "Card-targeted filter: the first declared target of the owner's Sha or damage trick, once per turn. Requires Rule VM ABI 384."
            }
            "owner_outside_turn_discarded_red_cards_in_discard" => {
                "Cards-moved filter: outside the owner's turn, at least one red card discarded by the owner remains in the shared discard pile. Requires Rule VM ABI 384."
            }
            "resolve_twtianyu" => {
                "TW Tianyu resolver: Zhenxi discards up to distance hand cards and/or moves one target field card; Yangshi adds a permanent attack-range mark unless all others are already in range, then gains a Sha. Requires Rule VM ABI 382."
            }
            "resolve_twwujing" => {
                "TW Wujing resolver: Fenghan chooses up to the triggering card's target count to draw one each; Congji chooses red event cards still in discard and gives them to one other player. Requires Rule VM ABI 384."
            }
            "reveal_uniform_hand_draw_self_choose_others" => {
                "Effect: publicly reveals the owner's current hand. If every card has one uniform effective color, the owner chooses zero to the current hand count of other living players; the chosen players and owner each draw one in canonical seat order. Requires Rule VM ABI 56.\n\n```\neffect reveal_uniform_hand_draw_self_choose_others(self);\n```"
            }
            "owner_source_single_target_sha_or_black_ordinary_trick_current_play_phase_unused" => {
                "Card-used trigger filter: during the owner's Play phase, the owner used a single-target Sha or black ordinary trick and this skill has not completed in the current phase. Requires Rule VM ABI 57."
            }
            "owner_is_target_male_source_has_discardable_equipment" => {
                "Damage-before trigger filter: the living owner is the damage target, the living source is male/double-sex, and the owner can discard an effective equipment card from hand/equipment. Requires Rule VM ABI 57."
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
            "owner_alive_other_turn_ended_no_card_targeted_other" => {
                "Turn-ended trigger filter: the living owner is not the living ending player, and that player used no card targeting another player during the ended turn. Requires Rule VM ABI 58."
            }
            "owner_alive_other_turn_ended_card_targeted_other_no_damage_other" => {
                "Turn-ended trigger filter: the living owner is not the living ending player, and that player targeted another player with a card but dealt no damage to another player during the ended turn. Requires Rule VM ABI 146."
            }
            "owner_alive_phase_targeted_hp_changed_can_use_wanjian" => {
                "Phase-any-end filter: the living owner was named as a card target and their actual HP changed during the ending phase, and a source-free Wanjian is currently legal. Requires Rule VM ABI 98."
            }
            "owner_alive_phase_targeted_hp_unchanged_can_use_juedou" => {
                "Phase-any-end filter: the living owner was named as a card target and their actual HP did not change during the ending phase, and a source-free Juedou is currently legal. Requires Rule VM ABI 98."
            }
            "choose_move_target_equipment_or_draw" => {
                "Effect: lets the owner cancel, draw one, or move one compatible equipment card from the other player into the owner's matching slot, replacing an occupied slot normally. Requires Rule VM ABI 58.\n\n```\neffect choose_move_target_equipment_or_draw(self, event_player);\n```"
            }
            "choose_move_target_equipment_and_or_draw" => {
                "Effect: lets the owner cancel, draw one, move one compatible equipment card from the other player into the owner's matching slot, or choose both available actions in move-then-draw order. Requires Rule VM ABI 146.\n\n```\neffect choose_move_target_equipment_and_or_draw(self, event_player);\n```"
            }
            "owner_source_sha_direct_damage_before" => {
                "Damage-before trigger filter: the living owner is the source of direct logical-Sha damage and the event player is its living target. Chain-propagated damage is excluded. Requires Rule VM ABI 147."
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
            "owner_source_sha_has_weapon" => {
                "Damage-before trigger filter: the living owner is the source of Sha damage, the event player is its living target, and the owner has a physical weapon equipped. Requires Rule VM ABI 59."
            }
            "choose_target_discard_weapon_range_hand_or_increase_damage" => {
                "Effect: the bound Sha damage target may discard exactly the source's physical weapon range in hand cards. If the target cannot or cancels, the damage and its chain base increase by one; if paid, the selected hand cards and the source's current physical weapon are discarded in that order. Requires Rule VM ABI 59.\n\n```\neffect choose_target_discard_weapon_range_hand_or_increase_damage(self, event_player);\n```"
            }
            "owner_source_other_damage_before_once_per_turn" => {
                "Damage-before trigger filter: once per current turn, the living owner is the damage source and the living event player is another player. Requires Rule VM ABI 322."
            }
            "choose_target_discard_distance_hand_or_increase_damage" => {
                "Effect: the bound damage target may discard exactly `max(1, 4-distance from source)` current hand cards. If the target has fewer cards or cancels, the current damage and chain base increase by one. Requires Rule VM ABI 322.\n\n```\neffect choose_target_discard_distance_hand_or_increase_damage(self, event_player);\n```"
            }
            "owner_alive_other_turn_ended_no_damage_other" => {
                "Turn-ended trigger filter: the living owner is not the living ending player, and that player dealt no damage to any other player during the ended turn. Requires Rule VM ABI 323."
            }
            "owner_alive_gained_two_this_turn_can_compare_other" => {
                "End-phase trigger filter: the living owner gained at least two physical cards during the current turn, has a hand card, and another living player has a hand card for comparison. Card gains are tracked only when loaded content needs this filter. Requires Rule VM ABI 325."
            }
            "choose_draw_view_two_gain_or_move_equipment" => {
                "Effect: lets the owner cancel, draw one card, privately inspect up to two deterministically sampled current hand cards from the other player and gain one, or move one compatible equipment card into the owner's matching slot. Requires Rule VM ABI 323.\n\n```\neffect choose_draw_view_two_gain_or_move_equipment(self, event_player);\n```"
            }
            "owner_source_or_target_hand_count_differs_once_per_turn" => {
                "Damage trigger filter: the living owner is the damage source or target, some living player's hand count differs from the owner's, and this skill has not fired in the current turn. Requires Rule VM ABI 60."
            }
            "owner_hand_compare_participant" => {
                "Hand-comparison trigger filter: the living owner is either the initiator or target of the current comparison."
            }
            "owner_skill_pile_nonempty_<def>" => {
                "Trigger filter: the owner's named skill pile (keyed by the skill definition id literal) holds at least one card. Matches upstream getExpansions(...).length > 0 gates."
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
            "resolve_qianlong" => {
                "Effect: authoritative machine for 潜龙 — after damage, view the top three deck cards and gain up to lost-HP count, the rest go to deck bottom.\n\n```\neffect resolve_qianlong(self, self_skill);\n```"
            }
            "resolve_jsrgzuozhan" => {
                "Effect: authoritative machine for 坐瞻 — at game start choose up to two other players; gain an attack-range mark equal to min(their highest HP, 5) and mark them as 坐瞻 allies.\n\n```\neffect resolve_jsrgzuozhan(self, self_skill);\n```"
            }
            "resolve_jsrgzuozhan_gain" => {
                "Effect: authoritative machine for the 坐瞻 group face — after a marked ally dies, have one surviving 坐瞻 ally gain up to X basic cards of distinct names from the discard pile.\n\n```\neffect resolve_jsrgzuozhan_gain(self, self_skill, event_player);\n```"
            }
            "resolve_twmouzhi" => {
                "Effect: authoritative machine for 谋制 — mode 0 (damage_before): if the damage card's color matches the recorded last damage color, prevent the damage; mode 1 (damage): record the event card's color (red/black, cleared when colorless).\n\n```\neffect resolve_twmouzhi(self, self_skill, mode, event_card);\n```"
            }
            "resolve_twdaigui" => {
                "Effect: authoritative machine for 待归 (slice 1) — at play-phase end with a uniform-colored hand, choose up to hand-count players and reveal that many cards from the deck bottom (per-target picks follow in a later slice).\n\n```\neffect resolve_twdaigui(self, self_skill, stage, count);\n```"
            }
            "resolve_twmitu" => {
                "Effect: authoritative machine for 迷途 (slice 1) — at your prepare phase choose up to three players; each draws two cards tagged 迷途 and shows them publicly (compare rounds follow in a later slice).\n\n```\neffect resolve_twmitu(self, self_skill, stage, count);\n```"
            }
            "resolve_qiantun" => {
                "Effect: authoritative machine for 谦吞 — once per play phase, have another player show any number of hand cards (their only compare cards), then compare: if you win, take the shown cards; otherwise take their other hand cards.\n\n```\neffect resolve_qiantun(self, self_skill);\n```"
            }
            "resolve_yangming" => {
                "Effect: authoritative machine for 养名 — once per play phase, compare with one player; if the target wins, they draw once per prior loss and you recover 1; otherwise count a loss and you may repeat.\n\n```\neffect resolve_yangming(self, self_skill);\n```"
            }
            "resolve_canyun" => {
                "Effect: authoritative machine for 残殒 — discard one card and pick 1-2 other players; compare equipment counts: if you have more they recover 1, if equal they draw 1, if they have more they lose 1 HP; you draw 1 when a target is at 1 HP.\n\n```\neffect resolve_canyun(self, self_skill);\n```"
            }
            "resolve_hengwu" => {
                "Effect: authoritative machine for 横骛 — after using or playing a card whose suit is absent from your hand, draw one card per equipment of that suit on the field.\n\n```\neffect resolve_hengwu(self, self_skill, event_card);\n```"
            }
            "resolve_biaozhao3" => {
                "Effect: authoritative machine for 表召·归 — at your prepare phase with a 表 card present: discard it, then optionally have one player draw up to the field's largest hand count (max 5) and recover 1.\n\n```\neffect resolve_biaozhao3(self, self_skill);\n```"
            }
            "resolve_chuanyun" => {
                "Effect: authoritative machine for 穿云 — after your Sha targets a player with equipment, randomly discard one of their equipped cards.\n\n```\neffect resolve_chuanyun(self, self_skill, event_player, event_card);\n```"
            }
            "resolve_biaozhao2" => {
                "Effect: authoritative machine for 表召·应 — when a discard matches the 表 card's suit and rank, give the 表 card to the discarder; you lose 1 HP.\n\n```\neffect resolve_biaozhao2(self, self_skill, event_player);\n```"
            }
            "resolve_dcniji" => {
                "Effect: authoritative machine for 逆击 — mode 0 (targeted by a non-equipment card): draw a card and tag it 逆击; mode 1 (your end phase): discard all tagged cards from your hand.\n\n```\neffect resolve_dcniji(self, self_skill, mode, event_card);\n```"
            }
            "resolve_dcmoukui" => {
                "Effect: authoritative machine for 谋溃 — after your Sha targets a player, choose one or both: draw a card, or discard one of the target's hand cards.\n\n```\neffect resolve_dcmoukui(self, self_skill, stage, target);\n```"
            }
            "resolve_zhongyan" => {
                "Effect: authoritative machine for 忠言 — once per play phase, reveal the top three deck cards; a chosen player with hand cards exchanges one of them for one revealed card.\n\n```\neffect resolve_zhongyan(self, self_skill, stage);\n```"
            }
            "resolve_chumei" => {
                "Effect: authoritative machine for 狐魅 — during your play phase, choose a player whose HP is at most the damage you dealt this turn; they draw a card, give you a card, or recover 1 HP.\n\n```\neffect resolve_chumei(self, self_skill, stage);\n```"
            }
            "owner_is_target_damage_amount_gt_one" => {
                "Trigger filter: the owner is the damage target and the incoming damage amount is greater than one (照月狮子盔 gate)."
            }
            "owner_phase_end_lost_cards" => {
                "Trigger filter: at the owner\'s own end phase, if they have lost at least one card this turn (镇抚 gate)."
            }
            "owner_alive_second_global_dying" => {
                "Trigger filter: after the second dying window of the whole match resolves (觉醒 gate; fires exactly once)."
            }
            "owner_is_target_source_hp_gt_owner_or_source_damaged_owner" => {
                "Trigger filter: you are targeted by another player\'s card, and their hp is higher than yours, or they have damaged you this match (粗遐 gate)."
            }
            "owner_is_target_owner_hp_gt_source_or_source_damaged_owner" => {
                "Trigger filter: you are targeted by another player\'s card, and your hp is higher than theirs, or they have damaged you this match (未命 gate)."
            }
            "owner_is_target_sanshou_type_tracked" => {
                "Trigger filter: the owner is the damage target (carries the used-type bitmap track for 三首)."
            }
            "owner_source_in_turn_target_unharmed_card_actions_ge_target_hp" => {
                "Trigger filter: the owner is the damage source during their own turn, the target has never taken damage this match, and the owner\'s lifetime card actions reach the target\'s hp (威临 gate)."
            }
            "owner_used_and_was_targeted_this_turn_in_range" => {
                "Trigger filter: at any player\'s end phase, if this turn you have used or played a card, been named a card target, and someone is within your attack range (直谏 gate)."
            }
            "resolve_qiangyong" => {
                "Effect: authoritative machine for 羌勇 — when your Sha deals damage, discard X cards from the wounded player (X = Sha uses this turn); if they then have no hand cards this damage is increased by one.\n\n```\neffect resolve_qiangyong(self, event_player);\n```"
            }
            "resolve_twxiongjun" => {
                "Effect: authoritative machine for 雄军 — after the owner deals damage, every living player owning 雄军 draws one card.\n\n```\neffect resolve_twxiongjun(self);\n```"
            }
            "owner_is_target_source_hp_gt_owner_per_role_turn" => {
                "Trigger filter: once per role turn, when the owner is targeted by a card and the source has more HP than the owner."
            }
            "resolve_dcqiaolve" => {
                "Effect: authoritative machine for 巧略 — discard one of your cards first, then take a random hand card from the event target (cost-added completion).\n\n```\neffect resolve_dcqiaolve(self);\n```"
            }
            "resolve_yixiang" => {
                "Effect: authoritative machine for 义襄 — randomly gain a basic card from the deck that is not already in your hand (once per role turn when targeted).\n\n```\neffect resolve_yixiang(self);\n```"
            }
            "resolve_xinfu_qiai" => {
                "Effect: authoritative machine for 求援 — once per game, while dying, every other player in seat order hands you one hand/equip card (players with none are skipped).\n\n```\neffect resolve_xinfu_qiai(self);\n```"
            }
            "resolve_olwenren" => {
                "Effect: authoritative machine for 温仁 — once per play phase, pick any number of targets; each draws 1 if they have no hand cards and 1 more if their hand count is not greater than yours.\n\n```\neffect resolve_olwenren(self);\n```"
            }
            "resolve_spolaige" => {
                "Effect: authoritative machine for 霸业 — after the second dying window of the match, exchange 西向 for 朔北, draw up to your max HP, and recover up to your max HP.\n\n```\neffect resolve_spolaige(self);\n```"
            }
            "resolve_olzhijie" => {
                "Effect: authoritative machine for 直谏 — at any player\'s end phase, choose a player within your attack range and deal 1 damage to them.\n\n```\neffect resolve_olzhijie(self);\n```"
            }
            "resolve_olsuyi" => {
                "Effect: authoritative machine for 俗义 — once per play phase, search the deck for equipment the target can use, have them equip it, then use a virtual Sha against them.\n\n```\neffect resolve_olsuyi(self, event_source);\n```"
            }
            "resolve_olqiaozhi" => {
                "Effect: authoritative machine for 巧织 — discard one card, reveal the top two deck cards, and gain one of them (tagged 巧织; while you hold one you cannot activate again).\n\n```\neffect resolve_olqiaozhi(self, stage);\n```"
            }
            "resolve_dczhee" => {
                "Effect: authoritative machine for 拓跋? — on entry/game start search the draw pile, and at every end phase search the discard pile, for a Lightning to hang as your judge card.\n\n```\neffect resolve_dczhee(self, mode);\n```"
            }
            "resolve_dchaoxian" => {
                "Effect: authoritative machine for 朝献 — once per game, shuffle every rank-3 card from the discard pile back into the deck bottom and collect all rank-3 hand cards from other players.\n\n```\neffect resolve_dchaoxian(self);\n```"
            }
            "resolve_xiangshu" => {
                "Effect: authoritative machine for 襄戍 — once per game, a wounded target recovers X hp and draws X cards (X = min(5, damage you dealt this turn)).\n\n```\neffect resolve_xiangshu(self, event_target);\n```"
            }
            "resolve_nhlundao" => {
                "Effect: authoritative machine for 论道 — after taking damage, if the source is alive with a different hand count: draw one when your hand is larger, otherwise discard one of the source\'s cards.\n\n```\neffect resolve_nhlundao(self, event_source);\n```"
            }
            "resolve_sanshou" => {
                "Effect: authoritative machine for 三首 — reveal the top three deck cards; if any revealed card is of a type never used this match, the incoming damage is cancelled.\n\n```\neffect resolve_sanshou(self);\n```"
            }
            "resolve_starcuxia_weiming" => {
                "Effect: authoritative machine for 粗遐/未命 — the card user randomly discards a hand card; the owner draws one if the dominance condition still holds.\n\n```\neffect resolve_starcuxia_weiming(self, event_source, variant);\n```"
            }
            "resolve_zhongjie" => {
                "Effect: authoritative machine for 张介 — when you die, choose another player: they gain 1 max hp, recover 1 hp, and draw a card.\n\n```\neffect resolve_zhongjie(self);\n```"
            }
            "resolve_dingce" => {
                "Effect: authoritative machine for 定策 — after taking damage, discard one of your hand cards, then the damage source discards one (you choose it); same-color bonus is deferred (card absent from the pool).\n\n```\neffect resolve_dingce(self, event_source);\n```"
            }
            "resolve_meiyong" => {
                "Effect: authoritative machine for 姝勇 — when you use or play a Sha, take one card from another player\'s hand/equip/judge area, then they draw one.\n\n```\neffect resolve_meiyong(self);\n```"
            }
            "resolve_mbzhenann" => {
                "Effect: authoritative machine for 镇南 — when targeted by a trick whose target count exceeds max(1, the user\'s hp), discard one card and deal 1 damage to any character.\n\n```\neffect resolve_mbzhenann(self);\n```"
            }
            "resolve_friendbiwei" => {
                "Effect: authoritative machine for 鄙位 — discard the highest-rank hand card, then the target discards every hand card of rank >= it; if they discarded nothing the skill\'s use is refunded.\n\n```\neffect resolve_friendbiwei(self, event_source);\n```"
            }
            "resolve_chulao" => {
                "Effect: authoritative machine for 黜老 — discard one card, then for each selected target of a distinct group the owner picks one of their cards to discard; spade discards draw.\n\n```\neffect resolve_chulao(self);\n```"
            }
            "resolve_dcanxu" => {
                "Effect: authoritative machine for 暗蓄 — the player with fewer hand cards takes one card from the other; non-spade gain draws owner one, equalized hands recover owner one.\n\n```\neffect resolve_dcanxu(self);\n```"
            }
            "resolve_duoduan" => {
                "Effect: authoritative machine for 度断 — once per turn when targeted by a Sha, recast one card, then choose: the Sha user draws two and the Sha has no effect on you, or the user discards a card and you cannot respond.\n\n```\neffect resolve_duoduan(self, event_source, event_card);\n```"
            }
            "resolve_gaoyuan" => {
                "Effect: authoritative machine for 告援 — when targeted by a Sha, discard one card to redirect the Sha to a legal target holding a 诤 mark.\n\n```\neffect resolve_gaoyuan(self, event_source, event_card);\n```"
            }
            "reset_linked_selected_targets" => {
                "Effect: resets every selected linked target, then deals 1 fire damage to each (duansuo).\n\n```\neffect reset_linked_selected_targets(self);\n```"
            }
            "resolve_mbbeiming" => {
                "Effect: authoritative machine for 孛明 — at game start choose up to two players; each gains a weapon from the deck whose attack range equals the number of suits in their hand.\n\n```\neffect resolve_mbbeiming(self, self_skill, stage);\n```"
            }
            "resolve_shuaiyan" => {
                "Effect: authoritative machine for 率言 — at your discard phase with more than one hand card, choose another player; they give you one card from their hand or equipment.\n\n```\neffect resolve_shuaiyan(self, self_skill, stage);\n```"
            }
            "resolve_zhengjian" => {
                "Effect: authoritative machine for 诤荐 — mode 0: at your end phase, grant a 诤 mark to one player; mode 1/2: a marked player's card use or play increments their mark; mode 3: at your turn begin, each marked player draws min(5, max HP, marks) and marks clear.\n\n```\neffect resolve_zhengjian(self, self_skill, stage, event_player);\n```"
            }
            "resolve_qinmian" => {
                "Effect: authoritative machine for 勤勉 — mode 0: at your prepare phase or after damage, draw that many cards tagged 勤勉; mode 1: at an end phase, optionally discard all tagged cards to damage the current turn player equal to their count (or 1 if your whole hand is tagged).\n\n```\neffect resolve_qinmian(self, self_skill, mode, amount);\n```"
            }
            "resolve_jsrgdaoren" => {
                "Effect: authoritative machine for 蹈仁 — give one hand card to another player, then deal 1 damage to every player inside both attack ranges.\n\n```\neffect resolve_jsrgdaoren(self, self_skill);\n```"
            }
            "resolve_duwang" => {
                "Effect: authoritative machine for 度望 — at game start, place the first five non-Sha cards from the deck top onto the own skill pile.\n\n```\neffect resolve_duwang(self, self_skill);\n```"
            }
            "resolve_quanjia" => {
                "Effect: authoritative machine for 拳甲 — randomly view (2+marks) cards of the damage source's hand; use a virtual Sha on them if any, else gain a mark.\n\n```\neffect resolve_quanjia(self, self_skill, event_source, event_player);\n```"
            }
            "resolve_pxqixian" => {
                "Effect: authoritative machine for 祁县 — recast one equip card as a virtual Sha or Guohe used on another player.\n\n```\neffect resolve_pxqixian(self, self_skill, 0, 0);\n```"
            }
            "resolve_dcbingji" => {
                "Effect: authoritative machine for 秉纪 — validates hand suit uniformity and per-turn suit history, then uses a virtual Sha or Tao.\n\n```\neffect resolve_dcbingji(self, self_skill, 0, 0);\n```"
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
            _ => return super::discard_skill_pile::documentation(word),
    })
}
