/// draw_cards(player_u32, count_u32)
pub const DRAW_CARDS: u16 = 1;
/// deal_damage(source, target, amount)
pub const DEAL_DAMAGE: u16 = 2;
/// recover_hp(player, amount)
pub const RECOVER_HP: u16 = 3;
/// emit_note(code)
pub const EMIT_NOTE: u16 = 4;
/// change_max_hp(player, delta_i32)
pub const CHANGE_MAX_HP: u16 = 5;
/// add_mark(player, mark_def, delta_i32)
pub const ADD_MARK: u16 = 6;
/// remove_mark(player, mark_def, amount_u32)
pub const REMOVE_MARK: u16 = 7;
/// discard_hand(player, count) — deterministic first-N from hand
pub const DISCARD_HAND: u16 = 8;
/// discard_hand_equip(player, count)
pub const DISCARD_HAND_EQUIP: u16 = 322;
/// take_hand(from, to) — move first hand card from→to
pub const TAKE_HAND: u16 = 9;
/// lose_hp(player, amount) — DealDamage is_lose_hp
pub const LOSE_HP: u16 = 10;
/// add_skill(player, skill_def_u32)
pub const ADD_SKILL: u16 = 11;
/// enter_dying(player)
pub const ENTER_DYING: u16 = 12;
/// reveal_card(card_u32) — reveal a single card publicly
pub const REVEAL_CARD: u16 = 13;
/// use_card(player, card_u32) — play-as-use with no explicit targets
pub const USE_CARD: u16 = 14;
/// remove_skill(skill_instance_u32)
pub const REMOVE_SKILL: u16 = 15;
/// move_card(card, to_zone, from_zone_or_zero)
pub const MOVE_CARD: u16 = 16;
/// set_turn_state(current, phase_tag)
pub const SET_TURN_STATE: u16 = 17;
/// use_card_on(player, card, target)
pub const USE_CARD_ON: u16 = 18;
/// discard_selected(player) — discard active invocation card inputs.
pub const DISCARD_SELECTED: u16 = 19;
/// draw_selected(player) — draw as many cards as the invocation selected.
pub const DRAW_SELECTED: u16 = 20;
/// remove_cards_by_kind(kind_tag) — move all matching live-zone cards out of game.
pub const REMOVE_CARDS_BY_KIND: u16 = 21;
/// kill_player(player, killer)
pub const KILL_PLAYER: u16 = 22;
/// discard_one_hand_or_equip(player) — forced dynamic card choice.
pub const DISCARD_ONE_HAND_OR_EQUIP: u16 = 23;
/// judge_gain_equipment_by_color(player) — red draws from deck, black from discard.
pub const JUDGE_GAIN_EQUIPMENT_BY_COLOR: u16 = 24;
/// reveal_discard_suit_gain_or_use(player, mark, base_count, max_mark)
pub const REVEAL_DISCARD_SUIT_GAIN_OR_USE: u16 = 25;
/// choose_discard_or_fire_damage_skip_phases(player, max_discard, damage)
pub const CHOOSE_DISCARD_OR_FIRE_DAMAGE_SKIP_PHASES: u16 = 26;
/// deal_damage(source, target, amount, nature)
pub const DEAL_NATURE_DAMAGE: u16 = 27;
/// discard_all_hand_and_equip(player) — forced discard of both zones.
pub const DISCARD_ALL_HAND_AND_EQUIP: u16 = 28;
/// draw_each_by_seat(first, second, count)
pub const DRAW_EACH_BY_SEAT: u16 = 29;
/// draw_then_give_one(player, target, count, drawn_only)
pub const DRAW_THEN_GIVE_ONE: u16 = 30;
/// draw_then_take_one(player, taker, count)
pub const DRAW_THEN_TAKE_ONE: u16 = 31;
/// take_one_card(from, to, equip_only)
pub const TAKE_ONE_CARD: u16 = 32;
/// turn_over(player) — toggle face-down state.
pub const TURN_OVER: u16 = 33;
/// remove_skill_by_def(player, skill_def)
pub const REMOVE_SKILL_BY_DEF: u16 = 34;
/// choose_discard_hand_add_skill_by_color(player, red_skill_def, black_skill_def)
pub const CHOOSE_DISCARD_HAND_ADD_SKILL_BY_COLOR: u16 = 35;
/// reveal_top_gain_non_basic_or_swap_basic(player)
pub const REVEAL_TOP_GAIN_NON_BASIC_OR_SWAP_BASIC: u16 = 36;
/// prevent_current_damage()
pub const PREVENT_CURRENT_DAMAGE: u16 = 37;
/// judge_cancel_current_sha_target_on_red(player, card, target)
pub const JUDGE_CANCEL_CURRENT_SHA_TARGET_ON_RED: u16 = 38;
/// choose_draw_or_recover(player, draw_count, recover_count, allow_cancel)
pub const CHOOSE_DRAW_OR_RECOVER: u16 = 39;
/// choose_use_virtual_sha(player, ignore_distance)
pub const CHOOSE_USE_VIRTUAL_SHA: u16 = 40;
/// adjust_hand_to(player, count) — draw or force an exact hand-card discard choice.
pub const ADJUST_HAND_TO: u16 = 41;
/// gain_card_by_kind(player, kind) — draw pile first, then discard pile.
pub const GAIN_CARD_BY_KIND: u16 = 42;
/// judge_recover_on_suit(player, suit, amount)
pub const JUDGE_RECOVER_ON_SUIT: u16 = 43;
/// use_virtual_card_on(player, kind, target)
pub const USE_VIRTUAL_CARD_ON: u16 = 44;
/// judge_damage_other_on_suit(player, suit, amount, nature)
pub const JUDGE_DAMAGE_OTHER_ON_SUIT: u16 = 45;
/// recast_selected(player)
pub const RECAST_SELECTED: u16 = 46;
/// disable_all_equip_slots(player)
pub const DISABLE_ALL_EQUIP_SLOTS: u16 = 47;
/// move_equipment_then_draw(player)
pub const MOVE_EQUIPMENT_THEN_DRAW: u16 = 48;
/// choose_use_virtual_sha_or_draw(player)
pub const CHOOSE_USE_VIRTUAL_SHA_OR_DRAW: u16 = 49;
/// discard_equipment_pair_with_armor(player)
pub const DISCARD_EQUIPMENT_PAIR_WITH_ARMOR: u16 = 50;
/// discard_target_equipment_gain_mount(player, target)
pub const DISCARD_TARGET_EQUIPMENT_GAIN_MOUNT: u16 = 51;
/// discard_target_card_gain_mount(player, target, scope)
pub const DISCARD_TARGET_CARD_GAIN_MOUNT: u16 = 52;
/// make_current_sha_target_unrespondable(source, card, target)
pub const MAKE_CURRENT_SHA_TARGET_UNRESPONDABLE: u16 = 53;
/// choose_discard_target_card(player, target, scope)
pub const CHOOSE_DISCARD_TARGET_CARD: u16 = 54;
/// judge_prevent_damage_lose_max_hp_on_nonheart(player, target)
pub const JUDGE_PREVENT_DAMAGE_LOSE_MAX_HP_ON_NONHEART: u16 = 55;
/// choose_gain_top_cards_by_suit(player, count, suit)
pub const CHOOSE_GAIN_TOP_CARDS_BY_SUIT: u16 = 56;
/// choose_other_draw_by_hand_count(player, normal_count, empty_count, times)
pub const CHOOSE_OTHER_DRAW_BY_HAND_COUNT: u16 = 57;
/// choose_discard_one_then_gain_target_equipment(player, target, slot)
pub const CHOOSE_DISCARD_ONE_THEN_GAIN_TARGET_EQUIPMENT: u16 = 58;
/// choose_target_draw(player, count, scope)
pub const CHOOSE_TARGET_DRAW: u16 = 59;
/// Draw by distinct logical basic/trick/equipment categories used this play phase.
pub const DRAW_BY_PLAY_PHASE_CARD_CATEGORY_COUNT: u16 = 60;
/// Refund current Sha use count once and draw for its effective red color.
pub const CHOOSE_DISCARD_ONE_FOR_SHA_REFUND_AND_DRAW_IF_RED: u16 = 61;
/// Queue an independent extra Play phase at the opening of the player's turn.
pub const QUEUE_OPENING_EXTRA_PLAY_PHASE: u16 = 62;
/// Convert the current ordinary Sha to fire and lose HP after it deals damage.
pub const CONVERT_CURRENT_SHA_TO_FIRE_LOSE_HP_IF_DAMAGE: u16 = 63;
/// Optionally add one legal target to the current Sha.
pub const CHOOSE_ADD_CURRENT_SHA_TARGET: u16 = 64;
/// Choose hand Sha and store them in the current skill's public pile.
pub const CHOOSE_STORE_HAND_SHA_IN_SKILL_PILE: u16 = 65;
/// Spend one current-skill pile card to make a dying player use virtual Jiu.
pub const CHOOSE_SPEND_SKILL_PILE_FOR_DYING_JIU: u16 = 66;
/// Compare the selected hand card with a target hand card and grant a helper skill.
pub const CHOOSE_COMPARE_SELECTED_HAND_CARD: u16 = 67;
/// Optionally append the current skill's bound target to a suspended card use.
pub const CHOOSE_ADD_SKILL_BOUND_CARD_TARGET: u16 = 68;
/// Optionally discard a matching-color non-basic card and grant a color-lock helper.
pub const CHOOSE_DISCARD_MATCHING_EQUIP_COLOR_ADD_SKILL: u16 = 69;
/// Retaliate from the current color-lock instance's source when no Sha was used.
pub const USE_VIRTUAL_SHA_FROM_CARD_COLOR_LOCK_SOURCE_IF_NO_SHA: u16 = 70;
/// Choose one or two enhancements for the currently suspended card use.
pub const CHOOSE_CURRENT_CARD_USE_ENHANCEMENTS: u16 = 71;
/// Choose whether to draw or discard, then grant the corresponding temporary skill.
pub const CHOOSE_DRAW_OR_DISCARD_ADD_SKILL: u16 = 72;
/// Repeatedly use an eligible triggering discard card as Bingliang.
pub const CHOOSE_DISCARDED_BLACK_BASIC_AS_BINGLIANG: u16 = 73;
/// Discard a judgment card, use virtual Sha, and draw if it deals no damage.
pub const CHOOSE_DISCARD_JUDGE_USE_VIRTUAL_SHA_DRAW_IF_NO_DAMAGE: u16 = 74;
/// Record a selected card category and target, then discard the selected card.
pub const DISCARD_SELECTED_RECORD_UNIQUE_CATEGORY_TARGET: u16 = 75;
/// Choose target draw/discard by active-skill use count, then conditionally link the owner.
pub const CHOOSE_TARGET_DRAW_OR_DISCARD_BY_SKILL_USE_COUNT_LINK_IF_WOUNDED: u16 = 76;
/// choose_reveal_other_hand_card_remember_category(player)
pub const CHOOSE_REVEAL_OTHER_HAND_CARD_REMEMBER_CATEGORY: u16 = 77;
/// draw_then_give_exact_record_phase_target(player, target, count)
pub const DRAW_THEN_GIVE_EXACT_RECORD_PHASE_TARGET: u16 = 78;
/// clear_phase_gift_target(player)
pub const CLEAR_PHASE_GIFT_TARGET: u16 = 79;
/// choose_gain_resolved_sha_or_shan_then_optional_use_sha(player, sha_card)
pub const CHOOSE_GAIN_RESOLVED_SHA_OR_SHAN_THEN_OPTIONAL_USE_SHA: u16 = 80;
/// skip_current_phase(player)
pub const SKIP_CURRENT_PHASE: u16 = 81;
/// choose_other_targets_by_skipped_phase_count_damage(player, amount)
pub const CHOOSE_OTHER_TARGETS_BY_SKIPPED_PHASE_COUNT_DAMAGE: u16 = 82;
/// reveal_uniform_hand_draw_self_choose_others(player)
pub const REVEAL_UNIFORM_HAND_DRAW_SELF_CHOOSE_OTHERS: u16 = 83;
/// choose_card_user_or_add_target(player, card)
pub const CHOOSE_CARD_USER_OR_ADD_TARGET: u16 = 84;
/// choose_discard_equipment_reduce_current_damage(player)
pub const CHOOSE_DISCARD_EQUIPMENT_REDUCE_CURRENT_DAMAGE: u16 = 85;
/// choose_move_target_equipment_or_draw(player, target)
pub const CHOOSE_MOVE_TARGET_EQUIPMENT_OR_DRAW: u16 = 86;
/// give_all_equipment_choose_recover_or_damage(player, target)
pub const GIVE_ALL_EQUIPMENT_CHOOSE_RECOVER_OR_DAMAGE: u16 = 87;
/// choose_target_discard_weapon_range_hand_or_increase_damage(player, target)
pub const CHOOSE_TARGET_DISCARD_WEAPON_RANGE_HAND_OR_INCREASE_DAMAGE: u16 = 88;
/// choose_hand_count_different_target_draw_or_random_discard(player)
pub const CHOOSE_HAND_COUNT_DIFFERENT_TARGET_DRAW_OR_RANDOM_DISCARD: u16 = 89;
/// increase_current_draw_count(player, count)
pub const INCREASE_CURRENT_DRAW_COUNT: u16 = 90;
/// choose_target_discard_excess_then_self_discard_or_target_draw(player)
pub const CHOOSE_TARGET_DISCARD_EXCESS_THEN_SELF_DISCARD_OR_TARGET_DRAW: u16 = 91;
/// draw_then_choose_gain_one_from_damage_target_and_neighbors(player, target)
pub const DRAW_THEN_CHOOSE_GAIN_ONE_FROM_DAMAGE_TARGET_AND_NEIGHBORS: u16 = 92;
/// choose_prevent_damage_or_lose_max_hp_increase_damage(player, amount)
pub const CHOOSE_PREVENT_DAMAGE_OR_LOSE_MAX_HP_INCREASE_DAMAGE: u16 = 93;
/// randomize_current_hand_compare_opponent(player)
pub const RANDOMIZE_CURRENT_HAND_COMPARE_OPPONENT: u16 = 94;
/// gain_highest_rank_sha_from_event_cards(player)
pub const GAIN_HIGHEST_RANK_SHA_FROM_EVENT_CARDS: u16 = 95;
/// gain_hand_compare_result_card(player)
pub const GAIN_HAND_COMPARE_RESULT_CARD: u16 = 96;
/// choose_add_or_remove_current_card_target(player, card)
pub const CHOOSE_ADD_OR_REMOVE_CURRENT_CARD_TARGET: u16 = 97;
/// choose_hand_compare(player, win_skill_def, lose_skill_def)
pub const CHOOSE_HAND_COMPARE: u16 = 98;
/// choose_other_damage(player, amount)
pub const CHOOSE_OTHER_DAMAGE: u16 = 99;
/// choose_other_discard_target_card(player, scope)
pub const CHOOSE_OTHER_DISCARD_TARGET_CARD: u16 = 100;
/// gain_discard_card_by_kind(player, kind)
pub const GAIN_DISCARD_CARD_BY_KIND: u16 = 101;
/// reveal_uniform_hand_choose_damage(player, amount)
pub const REVEAL_UNIFORM_HAND_CHOOSE_DAMAGE: u16 = 102;
/// choose_gain_discard_card_by_kind(player, kind)
pub const CHOOSE_GAIN_DISCARD_CARD_BY_KIND: u16 = 103;
/// draw_for_lifetime_card_action_divisors(player, first, second, third)
pub const DRAW_FOR_LIFETIME_CARD_ACTION_DIVISORS: u16 = 104;
/// choose_add_missing_skill(player, first, second, third)
pub const CHOOSE_ADD_MISSING_SKILL: u16 = 105;
/// choose_targets_draw(player, count, target_count, scope)
pub const CHOOSE_TARGETS_DRAW: u16 = 106;
/// gain_event_cards(player)
pub const GAIN_EVENT_CARDS: u16 = 107;
/// choose_target_draw_to_max_hp_cap(player, cap)
pub const CHOOSE_TARGET_DRAW_TO_MAX_HP_CAP: u16 = 108;
/// choose_other_draw_owner_lost_hp_turn_over(player)
pub const CHOOSE_OTHER_DRAW_OWNER_LOST_HP_TURN_OVER: u16 = 109;
/// judge_recover_other_on_suit(judge_player, recover_player, suit, amount)
pub const JUDGE_RECOVER_OTHER_ON_SUIT: u16 = 110;
/// force_others_use_sha_on_nearest_or_lose_hp(player)
pub const FORCE_OTHERS_USE_SHA_ON_NEAREST_OR_LOSE_HP: u16 = 111;
/// restore_character_state(player) — unlink and turn face-up without toggling normal state.
pub const RESTORE_CHARACTER_STATE: u16 = 112;
/// discard_all_hand_equip_judge(player)
pub const DISCARD_ALL_HAND_EQUIP_JUDGE: u16 = 113;
/// choose_discard_target_card_resolve_current_sha(source, card, target)
pub const CHOOSE_DISCARD_TARGET_CARD_RESOLVE_CURRENT_SHA: u16 = 114;
/// use_virtual_card_on_selected_targets(player, card_kind)
pub const USE_VIRTUAL_CARD_ON_SELECTED_TARGETS: u16 = 115;
/// deal_damage_ban_active_if_dying(source, target, amount, skill)
pub const DEAL_DAMAGE_BAN_ACTIVE_IF_DYING: u16 = 116;
/// choose_target_reclaim_equipment_draw_if_range_reduced(player, target, skill)
pub const CHOOSE_TARGET_RECLAIM_EQUIPMENT_DRAW_IF_RANGE_REDUCED: u16 = 117;
/// choose_use_virtual_card(player, card_kind, skill)
pub const CHOOSE_USE_VIRTUAL_CARD: u16 = 118;
/// increase_current_sha_base_damage(player, card, amount)
pub const INCREASE_CURRENT_SHA_BASE_DAMAGE: u16 = 119;
/// skip_next_play_phase(player)
pub const SKIP_NEXT_PLAY_PHASE: u16 = 120;
/// choose_other_draw_or_recover(player, draw_count, recover_count, times)
pub const CHOOSE_OTHER_DRAW_OR_RECOVER: u16 = 121;
/// choose_discard_hand_equip_then_remove_self_skill(player, count)
pub const CHOOSE_DISCARD_HAND_EQUIP_THEN_REMOVE_SELF_SKILL: u16 = 122;
/// draw_selected_targets(count)
pub const DRAW_SELECTED_TARGETS: u16 = 123;
/// recover_if_min_current_hp(player, amount)
pub const RECOVER_IF_MIN_CURRENT_HP: u16 = 124;
/// increase_current_card_base_damage(player, card, amount)
pub const INCREASE_CURRENT_CARD_BASE_DAMAGE: u16 = 125;
/// lose_hp_then_damage_other_lose_max_hp_if_dying(player, target, amount, skill)
pub const LOSE_HP_THEN_DAMAGE_OTHER_LOSE_MAX_HP_IF_DYING: u16 = 126;
/// recover_selected_targets(amount)
pub const RECOVER_SELECTED_TARGETS: u16 = 127;
/// record_selected_play_phase_category(skill)
pub const RECORD_SELECTED_PLAY_PHASE_CATEGORY: u16 = 128;
/// take_target_hand_then_repeat_until_max(player, target, skill)
pub const TAKE_TARGET_HAND_THEN_REPEAT_UNTIL_MAX: u16 = 129;
/// choose_draw_or_increase_current_card_damage_or_both_mark(
/// player, card, draw_count, damage_amount, mark_def, mark_count)
pub const CHOOSE_DRAW_OR_INCREASE_CURRENT_CARD_DAMAGE_OR_BOTH_MARK: u16 = 130;
/// draw_then_choose_give_or_discard_hand(player, draw_count, payment_count)
pub const DRAW_THEN_CHOOSE_GIVE_OR_DISCARD_HAND: u16 = 131;
/// gain_target_equipment(player, target, slot)
pub const GAIN_TARGET_EQUIPMENT: u16 = 132;
/// discard_all_target_equipment(actor, target)
pub const DISCARD_ALL_TARGET_EQUIPMENT: u16 = 133;
/// increase_current_sha_target_damage(source, card, target, amount)
pub const INCREASE_CURRENT_SHA_TARGET_DAMAGE: u16 = 134;
/// choose_target_draw_then_owner_if_below_max_hp(player, count)
pub const CHOOSE_TARGET_DRAW_THEN_OWNER_IF_BELOW_MAX_HP: u16 = 135;
/// choose_bound_source_attack_range_damage(player, source, amount)
pub const CHOOSE_BOUND_SOURCE_ATTACK_RANGE_DAMAGE: u16 = 136;
/// choose_target_draw_max_hp_cap_then_discard_excess(player, cap)
pub const CHOOSE_TARGET_DRAW_MAX_HP_CAP_THEN_DISCARD_EXCESS: u16 = 137;
/// give_selected_then_target_choose_discard_same_suit_or_lose_hp(player, target)
pub const GIVE_SELECTED_THEN_TARGET_CHOOSE_DISCARD_SAME_SUIT_OR_LOSE_HP: u16 = 138;
/// gain_equipment_from_draw_pile(player)
pub const GAIN_EQUIPMENT_FROM_DRAW_PILE: u16 = 139;
/// give_all_hand_recover_to_one(player, target)
pub const GIVE_ALL_HAND_RECOVER_TO_ONE: u16 = 140;
/// choose_target_self_discard_then_owner_lose_hp_if_hand_ne_hp(player)
pub const CHOOSE_TARGET_SELF_DISCARD_THEN_OWNER_LOSE_HP_IF_HAND_NE_HP: u16 = 141;
/// choose_give_one_hand_equip_or_recover(owner, giver, target)
pub const CHOOSE_GIVE_ONE_HAND_EQUIP_OR_RECOVER: u16 = 142;
/// add_bound_skill(player, skill_def, target)
pub const ADD_BOUND_SKILL: u16 = 143;
/// replace_skill_with_bound_after_resolution_if_target_alive(
/// player, skill, skill_def, target)
pub const REPLACE_SKILL_WITH_BOUND_AFTER_RESOLUTION_IF_TARGET_ALIVE: u16 = 144;
/// choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp(player)
pub const CHOOSE_DISCARD_ALL_HAND_OR_EQUIP_DRAW_LOST_HP_THEN_LOSE_HP: u16 = 145;
/// choose_discard_hand_equip_then_recover(player, count, amount)
pub const CHOOSE_DISCARD_HAND_EQUIP_THEN_RECOVER: u16 = 146;
/// choose_targets_damage(player, min_count, max_count, scope, amount)
pub const CHOOSE_TARGETS_DAMAGE: u16 = 147;
/// discard_target_equipment_draw_weapon_range(player, target)
pub const DISCARD_TARGET_EQUIPMENT_DRAW_WEAPON_RANGE: u16 = 148;
/// choose_target_reveal_hand_damage_if_sha(player, amount)
pub const CHOOSE_TARGET_REVEAL_HAND_DAMAGE_IF_SHA: u16 = 149;
/// choose_other_give_one_hand_equip(player)
pub const CHOOSE_OTHER_GIVE_ONE_HAND_EQUIP: u16 = 150;
/// choose_reveal_hand_card_bind_category_target(player, target)
pub const CHOOSE_REVEAL_HAND_CARD_BIND_CATEGORY_TARGET: u16 = 151;
/// recover_to_max_then_optional_exchange_skill(player, removable_skill_def)
pub const RECOVER_TO_MAX_THEN_OPTIONAL_EXCHANGE_SKILL: u16 = 152;
/// reveal_hand_discard_kind(player, card_kind)
pub const REVEAL_HAND_DISCARD_KIND: u16 = 153;
/// choose_move_target_equipment_and_or_draw(player, target)
pub const CHOOSE_MOVE_TARGET_EQUIPMENT_AND_OR_DRAW: u16 = 154;
/// choose_target_discard_attack_range_cards_or_increase_damage_weapon_forces_both(
/// player, target)
pub const CHOOSE_TARGET_DISCARD_ATTACK_RANGE_CARDS_OR_INCREASE_DAMAGE_WEAPON_FORCES_BOTH: u16 = 155;
/// choose_draw_or_recast_add_skill(player, draw_skill_def, recast_skill_def)
pub const CHOOSE_DRAW_OR_RECAST_ADD_SKILL: u16 = 156;
/// choose_gain_sha_for_opening_extra_play(player)
pub const CHOOSE_GAIN_SHA_FOR_OPENING_EXTRA_PLAY: u16 = 157;
/// finish_opening_extra_play_sha_gain(player)
pub const FINISH_OPENING_EXTRA_PLAY_SHA_GAIN: u16 = 158;
/// reveal_target_hand_gain_shan_armor_or_draw_two(player, target)
pub const REVEAL_TARGET_HAND_GAIN_SHAN_ARMOR_OR_DRAW_TWO: u16 = 159;
/// give_all_hand(player, target)
pub const GIVE_ALL_HAND: u16 = 160;
/// gain_card_if_discard(player, card)
pub const GAIN_CARD_IF_DISCARD: u16 = 161;
/// choose_draw_or_discard_target_card_record_sha(player, card, target)
pub const CHOOSE_DRAW_OR_DISCARD_TARGET_CARD_RECORD_SHA: u16 = 162;
/// reveal_hand_to_player(player, observer)
pub const REVEAL_HAND_TO_PLAYER: u16 = 163;
/// choose_optional_discard_target_card(player, target, scope)
pub const CHOOSE_OPTIONAL_DISCARD_TARGET_CARD: u16 = 164;
/// reveal_hand_choose_discard_color(player)
pub const REVEAL_HAND_CHOOSE_DISCARD_COLOR: u16 = 165;
/// choose_discard_target_cards_recover_same_category(player, target, count, amount)
pub const CHOOSE_DISCARD_TARGET_CARDS_RECOVER_SAME_CATEGORY: u16 = 166;
/// choose_compare_selected_hand_card_second_failure_seat_damage(
///     player, target, draw_count, failure_count, lose_hp, damage
/// )
pub const CHOOSE_COMPARE_SELECTED_HAND_CARD_SECOND_FAILURE_SEAT_DAMAGE: u16 = 167;
/// remove_all_skills(player)
pub const REMOVE_ALL_SKILLS: u16 = 168;
/// choose_other_recover(player, amount)
pub const CHOOSE_OTHER_RECOVER: u16 = 169;
/// enable_equip_slot(player, slot, source)
pub const ENABLE_EQUIP_SLOT: u16 = 170;
/// target_choose_suit_gain_owner_hand_damage_if_mismatch(player, target)
pub const TARGET_CHOOSE_SUIT_GAIN_OWNER_HAND_DAMAGE_IF_MISMATCH: u16 = 171;
/// judge_source_discard_two_hand_or_damage(player, source)
pub const JUDGE_SOURCE_DISCARD_TWO_HAND_OR_DAMAGE: u16 = 172;
/// give_selected_recover_after_phase_total_two(player, target)
pub const GIVE_SELECTED_RECOVER_AFTER_PHASE_TOTAL_TWO: u16 = 173;
/// choose_other_hand_targets_gain_one_replace_draw(player)
pub const CHOOSE_OTHER_HAND_TARGETS_GAIN_ONE_REPLACE_DRAW: u16 = 174;
/// distribute_top_two_cards(player)
pub const DISTRIBUTE_TOP_TWO_CARDS: u16 = 175;
/// judge_make_current_sha_target_unrespondable_on_red(player, card, target)
pub const JUDGE_MAKE_CURRENT_SHA_TARGET_UNRESPONDABLE_ON_RED: u16 = 176;
/// choose_discard_basic_then_target_discard_equip_or_damage(player, target)
pub const CHOOSE_DISCARD_BASIC_THEN_TARGET_DISCARD_EQUIP_OR_DAMAGE: u16 = 177;
/// choose_current_sha_target_discard_redirect_to_owner(owner, source, card, target)
pub const CHOOSE_CURRENT_SHA_TARGET_DISCARD_REDIRECT_TO_OWNER: u16 = 178;
/// choose_target_gain_bottom_card(player)
pub const CHOOSE_TARGET_GAIN_BOTTOM_CARD: u16 = 179;
/// judge_black_gain_repeat(player)
pub const JUDGE_BLACK_GAIN_REPEAT: u16 = 180;
/// use_virtual_juedou_second_on_first()
pub const USE_VIRTUAL_JUEDOU_SECOND_ON_FIRST: u16 = 181;
/// recover_hp_as(source, player, amount)
pub const RECOVER_HP_AS: u16 = 182;
/// choose_discard_redirect_current_sha(player, source, card)
pub const CHOOSE_DISCARD_REDIRECT_CURRENT_SHA: u16 = 183;
/// judge_set_skill_mark_to_rank(player, skill)
pub const JUDGE_SET_SKILL_MARK_TO_RANK: u16 = 184;
/// choose_discard_one_from_each_linked(player, skill)
pub const CHOOSE_DISCARD_ONE_FROM_EACH_LINKED: u16 = 185;
/// disable_skill_by_def_until_turn_end(player, skill_def)
pub const DISABLE_SKILL_BY_DEF_UNTIL_TURN_END: u16 = 186;
/// target_discard_then_duel_if_non_sha_and_hp_gte_owner(player, target)
pub const TARGET_DISCARD_THEN_DUEL_IF_NON_SHA_AND_HP_GTE_OWNER: u16 = 187;
/// choose_target_lose_hp_discard_hand_sha_if_basic(player)
pub const CHOOSE_TARGET_LOSE_HP_DISCARD_HAND_SHA_IF_BASIC: u16 = 188;
/// choose_optional_targets_draw(player, count, min_count, max_count, scope)
pub const CHOOSE_OPTIONAL_TARGETS_DRAW: u16 = 189;
/// reveal_target_hand_sha_or_discard_bind(player, target, helper_skill_def)
pub const REVEAL_TARGET_HAND_SHA_OR_DISCARD_BIND: u16 = 190;
/// draw_remembered_hand_cards(player, count)
pub const DRAW_REMEMBERED_HAND_CARDS: u16 = 191;
/// discard_remembered_hand_cards(player)
pub const DISCARD_REMEMBERED_HAND_CARDS: u16 = 192;
/// gain_draw_cards_for_lifetime_card_action_divisors(player, divisor/group pairs)
pub const GAIN_DRAW_CARDS_FOR_LIFETIME_CARD_ACTION_DIVISORS: u16 = 193;
/// lose_hp_gain_discard_card_for_opening_extra_play(
///     player, kind, mandatory_while_skill_def_owned
/// )
pub const LOSE_HP_GAIN_DISCARD_CARD_FOR_OPENING_EXTRA_PLAY: u16 = 194;
/// choose_gain_own_lost_discard_card_this_turn(player)
pub const CHOOSE_GAIN_OWN_LOST_DISCARD_CARD_THIS_TURN: u16 = 195;
/// choose_move_field_card_to(player, target)
pub const CHOOSE_MOVE_FIELD_CARD_TO: u16 = 196;
/// draw_then_optionally_distribute_hand_cards(player, draw_count, give_cap)
pub const DRAW_THEN_OPTIONALLY_DISTRIBUTE_HAND_CARDS: u16 = 197;
/// give_selected_then_continue_distinct_recover_at_two(player, target)
pub const GIVE_SELECTED_THEN_CONTINUE_DISTINCT_RECOVER_AT_TWO: u16 = 198;
/// choose_max_mark_other_judge_kill_unless_tao_or_taoyuan(player, mark_def)
pub const CHOOSE_MAX_MARK_OTHER_JUDGE_KILL_UNLESS_TAO_OR_TAOYUAN: u16 = 199;
/// choose_zone_priority_gain_random_card_from_each_other_then_turn_over(player)
pub const CHOOSE_ZONE_PRIORITY_GAIN_RANDOM_CARD_FROM_EACH_OTHER_THEN_TURN_OVER: u16 = 200;
/// add_skill_until_turn_end(player, skill_def)
pub const ADD_SKILL_UNTIL_TURN_END: u16 = 201;
/// resolve_mass_damage_discard_turn_over(player, mark, mark_count, damage, hand_count)
pub const RESOLVE_MASS_DAMAGE_DISCARD_TURN_OVER: u16 = 202;
/// record_active_target(player, target)
pub const RECORD_ACTIVE_TARGET: u16 = 203;
/// resolve_mark_parity_control(player, mark, mass_threshold, damage_amount)
pub const RESOLVE_MARK_PARITY_CONTROL: u16 = 204;
/// resolve_selected_equipment_discard_then_choose_damage(player, damage_amount, nature)
pub const RESOLVE_SELECTED_EQUIPMENT_DISCARD_THEN_CHOOSE_DAMAGE: u16 = 205;
/// replace_draw_with_distinct_suit_selection(player, count)
pub const REPLACE_DRAW_WITH_DISTINCT_SUIT_SELECTION: u16 = 206;
/// inspect_target_hand_choose_heart_discard_or_top(player, target)
pub const INSPECT_TARGET_HAND_CHOOSE_HEART_DISCARD_OR_TOP: u16 = 207;
/// resolve_yeyan(player)
pub const RESOLVE_YEYAN: u16 = 208;
/// choose_all_lose_hp_or_recover(player)
pub const CHOOSE_ALL_LOSE_HP_OR_RECOVER: u16 = 209;
/// resolve_qixing(player, initialize)
pub const RESOLVE_QIXING: u16 = 210;
/// resolve_qixing_weather(player, qixing_skill_def, helper_skill_def, max_targets)
pub const RESOLVE_QIXING_WEATHER: u16 = 211;
/// resolve_nzry_longnu(player, fire_skill_def, thunder_skill_def)
pub const RESOLVE_NZRY_LONGNU: u16 = 212;
/// choose_link_other(player)
pub const CHOOSE_LINK_OTHER: u16 = 213;
/// resolve_drlt_poxi(player, target, hand_limit_skill_def)
pub const RESOLVE_DRLT_POXI: u16 = 214;
/// resolve_drlt_jieying(player, event_player, mark_def, mode)
pub const RESOLVE_DRLT_JIEYING: u16 = 215;
/// resolve_drlt_duorui(player, target, helper_skill_def, stage)
pub const RESOLVE_DRLT_DUORUI: u16 = 216;
/// resolve_drlt_zhiti(player, event_player, event_source, event_amount, mode)
pub const RESOLVE_DRLT_ZHITI: u16 = 217;
/// resolve_shen_simayi(player, event_player, event_amount, mark, mode)
pub const RESOLVE_SHEN_SIMAYI: u16 = 218;
/// resolve_beige(player, target, source)
pub const RESOLVE_BEIGE: u16 = 219;
/// make_current_card_unrespondable(player, card, helper_skill_def)
pub const MAKE_CURRENT_CARD_UNRESPONDABLE: u16 = 220;
/// choose_discard_weapon_or_lose_hp(player)
pub const CHOOSE_DISCARD_WEAPON_OR_LOSE_HP: u16 = 221;
/// damage_then_discard_all_judge(player)
pub const DAMAGE_THEN_DISCARD_ALL_JUDGE: u16 = 222;
/// replace_draw_gain_top_two_add_skills_if_different_colors(player, first_skill_def, second_skill_def)
pub const REPLACE_DRAW_GAIN_TOP_TWO_ADD_SKILLS_IF_DIFFERENT_COLORS: u16 = 223;
/// choose_gain_target_card_alternate_zone_then_target_draw(player, target)
pub const CHOOSE_GAIN_TARGET_CARD_ALTERNATE_ZONE_THEN_TARGET_DRAW: u16 = 224;
/// skip_next_judge_phase(player)
pub const SKIP_NEXT_JUDGE_PHASE: u16 = 225;
/// choose_discard_damage_endpoint_card_increase_if_suit(player, target, suit, amount)
pub const CHOOSE_DISCARD_DAMAGE_ENDPOINT_CARD_INCREASE_IF_SUIT: u16 = 226;
/// take_one_event_target_hand(player)
pub const TAKE_ONE_EVENT_TARGET_HAND: u16 = 227;
/// choose_other_hand_store_skill_pile(player, target) — ABI 567
pub const CHOOSE_OTHER_HAND_STORE_SKILL_PILE: u16 = 529;
/// gain_all_skill_pile_cards(player) — ABI 569
pub const GAIN_ALL_SKILL_PILE_CARDS: u16 = 530;
/// gain_other_skill_pile_cards(player, target, skill_def) — ABI 600 (英逝 yingshi_die)
pub const GAIN_OTHER_SKILL_PILE_CARDS: u16 = 656;
/// choose_discard_one_he_or_lose_hp(player, lose_amount) — ABI 601 (庸肆 xinyongsi2)
pub const CHOOSE_DISCARD_ONE_HE_OR_LOSE_HP: u16 = 657;
/// judge_spade_two_to_nine_draw_two_or_thunder_sha_twice(source, target) — ABI 602 (雷劫 releijie)
pub const JUDGE_SPADE_TWO_TO_NINE_DRAW_TWO_OR_THUNDER_SHA_TWICE: u16 = 658;
/// choose_source_confirm_owner_lose_hp(owner, source, amount) — ABI 603 (随势 suishi2)
pub const CHOOSE_SOURCE_CONFIRM_OWNER_LOSE_HP: u16 = 659;
/// choose_use_virtual_nanman(player, skill) — ABI 604 (新镇南 newzhennan):
/// use a suitless, source-free virtual 南蛮入侵 through the ordinary card pipeline
/// (phase-agnostic, no phase-end history suppression).
pub const CHOOSE_USE_VIRTUAL_NANMAN: u16 = 660;
/// resolve_qiluan(player, skill) — ABI 605 (起乱 qiluan): draw Σ alive players'
/// current-turn kill counts × (3 if the killer is the owner else 1).
pub const RESOLVE_QILUAN: u16 = 661;
/// discard_all_own_judge(player, skill) — ABI 606 (飞杨 olfeiyang):
/// discard every card in the player's judgment zone.
pub const DISCARD_ALL_OWN_JUDGE: u16 = 662;
/// store_event_card_in_skill_pile(player, card, skill) — ABI 607 (灵杉 huamu):
/// store the triggering event's used card onto the owner's skill pile.
pub const STORE_EVENT_CARD_IN_SKILL_PILE: u16 = 663;
/// resolve_ollinjie(player, source, skill) — ABI 608 (临界 ollinjie):
/// the damage source discards one hand card; owner draws one if it is a Sha.
pub const RESOLVE_OLLINJIE: u16 = 664;
/// choose_target_gain_max_hp(player, skill) — ABI 646 (寿春 pxshouchun):
/// the owner selects any living player; that player gains 1 max hp.
pub const CHOOSE_TARGET_GAIN_MAX_HP: u16 = 665;
/// gain_target_hand_card(player, skill) — ABI 647 (肃州 pxsizhou):
/// the owner selects another player, then gains one of their hand cards.
pub const GAIN_TARGET_HAND_CARD: u16 = 666;
/// choose_clan_not_max_hand_limit_gain(player, skill) — ABI 648 (穆荫 clanmuyin):
/// select a same-clan-or-self player whose hand limit is not the field max;
/// that player gains +1 hand limit (persistent).
pub const CHOOSE_CLAN_NOT_MAX_HAND_LIMIT_GAIN: u16 = 667;
/// extra_resolve_current_trick(player, card, skill) — ABI 649 (额外结算 family):
/// queue one extra effect pass for the suspended trick (upstream effectCount++).
pub const EXTRA_RESOLVE_CURRENT_TRICK: u16 = 668;
/// place_hand_card_to_expansion(player, skill) — ABI 650 (expansion P2, ADR 0641):
/// select one hand card onto the skill pile (置入武将牌上).
pub const PLACE_HAND_CARD_TO_EXPANSION: u16 = 669;
/// gain_from_skill_pile(player, skill) — ABI 651 (expansion P1, ADR 0641):
/// select one card from own skill pile into hand (取用).
pub const GAIN_FROM_SKILL_PILE: u16 = 671;
/// discard_skill_pile(player, skill) — ABI 684 (expansion S1-B, ADR 0641 §8.2):
/// moves every card of the owner's own skill pile into the discard pile.
pub const DISCARD_SKILL_PILE: u16 = 707;
/// place_top_deck_to_expansion(player, skill, pile_def, count) — ABI 685
/// (expansion P2-var, ADR 0641 §8.7): moves the top `count` draw-pile cards
/// into the owner's own (player, pile_def) skill pile.
pub const PLACE_TOP_DECK_TO_EXPANSION: u16 = 708;
/// choose_one_from_skill_pile_discard(player, skill, pile_def) — ABI 686
/// (expansion S1-B, ADR 0641 §8.7): pick one card from the own (player,
/// pile_def) skill pile and discard it.
pub const CHOOSE_ONE_FROM_SKILL_PILE_DISCARD: u16 = 709;
/// gain_whole_skill_pile(player, skill, pile_def) — ABI 687
/// (expansion S1-B 深化, ADR 0641 §8.7): moves every card of the owner's
/// own (player, pile_def) skill pile into the owner's hand.
pub const GAIN_WHOLE_SKILL_PILE: u16 = 710;
/// choose_target_hand_card_to_pile(player, skill, pile_def) — ABI 688
/// (expansion A1, ADR 0641 §8.7): pick another player's hand card into the
/// (player, pile_def) skill pile.
pub const CHOOSE_TARGET_HAND_CARD_TO_PILE: u16 = 711;
/// rearrange_pile_to_deck_top(player, skill, pile_def, count) — ABI 690
/// (chooseToMove family, ADR 0641 §8.8): select count cards from the
/// (player, pile_def) skill pile and place them on top of the draw pile.
pub const REARRANGE_PILE_TO_DECK_TOP: u16 = 713;
/// choose_hand_cards_to_skill_pile(player, skill, min_count, max_count) —
/// ABI 691 (A0 placement face, ADR 0641 §8.12): select [min,max] hand
/// cards and place them into the own originating skill pile.
pub const CHOOSE_HAND_CARDS_TO_SKILL_PILE: u16 = 714;
/// resolve_dcbingji(player, skill, stage, kind) — ABI 692 (authoritative
/// machine for 秉纪, ADR 0641 §8.30).
pub const RESOLVE_DCBINGJI: u16 = 715;
/// resolve_pxqixian(player, skill, stage, kind) — ABI 693 (authoritative
/// machine for 祁县, ADR 0641 §8.31).
pub const RESOLVE_PXQIXIAN: u16 = 716;
/// resolve_quanjia(player, skill, source, target) — ABI 694 (authoritative
/// machine for 拳甲, ADR 0641 §8.32).
pub const RESOLVE_QUANJIA: u16 = 717;
/// resolve_duwang(player, skill) — ABI 695 (authoritative machine for
/// 度望, ADR 0641 §8.34).
pub const RESOLVE_DUWANG: u16 = 718;
/// resolve_jsrgdaoren(player, skill) — ABI 696 (authoritative machine for
/// 蹈仁, ADR 0641 §8.35).
pub const RESOLVE_JSRGDAOREN: u16 = 719;
/// resolve_qianlong(player, skill) — ABI 697 (authoritative machine for
/// 潜龙, ADR 0641 §8.36).
pub const RESOLVE_QIANLONG: u16 = 720;
/// resolve_jsrgzuozhan(player, skill) — ABI 698 (authoritative machine for
/// 坐瞻, ADR 0641 §8.37).
pub const RESOLVE_JSRGZUOZHAN: u16 = 721;
/// resolve_jsrgzuozhan_gain(player, skill, event_player) — ABI 699
/// (authoritative machine for the 坐瞻 group face, ADR 0641 §8.38).
pub const RESOLVE_JSRGZUOZHAN_GAIN: u16 = 722;
/// resolve_twmouzhi(player, skill, mode, event_card) — ABI 700
/// (authoritative machine for 谋制, ADR 0641 §8.39).
pub const RESOLVE_TWMOUZHI: u16 = 723;
/// resolve_twdaigui(player, skill, stage, count) — ABI 701 (authoritative
/// machine for 待归 slice 1, ADR 0641 §8.40).
pub const RESOLVE_TWDAIGUI: u16 = 724;
/// resolve_twmitu(player, skill, stage, count) — ABI 702 (authoritative
/// machine for 迷途 slice 1, ADR 0641 §8.41).
pub const RESOLVE_TWMITU: u16 = 725;
/// resolve_qiantun(player, skill) — ABI 703 (authoritative machine for
/// 谦吞, ADR 0641 §8.42).
pub const RESOLVE_QIANTUN: u16 = 726;
/// resolve_yangming(player, skill) — ABI 704 (authoritative machine for
/// 养名, ADR 0641 §8.43).
pub const RESOLVE_YANGMING: u16 = 727;
/// resolve_canyun(player, skill) — ABI 705 (authoritative machine for
/// 残殒, ADR 0641 §8.44).
pub const RESOLVE_CANYUN: u16 = 728;
/// resolve_hengwu(player, skill, event_card) — ABI 706 (authoritative
/// machine for 横骛, ADR 0641 §8.45).
pub const RESOLVE_HENGWU: u16 = 729;
/// resolve_biaozhao3(player, skill) — ABI 707 (authoritative machine for
/// 表召·归, ADR 0641 §8.46).
pub const RESOLVE_BIAOZHAO3: u16 = 730;
/// resolve_chuanyun(player, skill, event_player, event_card) — ABI 708
/// (authoritative machine for 穿云, ADR 0641 §8.47).
pub const RESOLVE_CHUANYUN: u16 = 731;
/// resolve_biaozhao2(player, skill, event_player) — ABI 709 (authoritative
/// machine for 表召·应, ADR 0641 §8.48).
pub const RESOLVE_BIAOZHAO2: u16 = 732;
/// resolve_dcniji(player, skill, mode, event_card) — ABI 710 (authoritative
/// machine for 逆击, ADR 0641 §8.49).
pub const RESOLVE_DCNIJI: u16 = 733;
/// resolve_dcmoukui(player, skill, stage, target) — ABI 711 (authoritative
/// machine for 谋溃, ADR 0641 §8.50).
pub const RESOLVE_DCMOUKUI: u16 = 734;
/// resolve_zhongyan(player, skill, stage) — ABI 712 (authoritative machine
/// for 忠言, ADR 0641 §8.51).
pub const RESOLVE_ZHONGYAN: u16 = 735;
/// resolve_chumei(player, skill, stage) — ABI 713 (authoritative machine
/// for 狐魅, ADR 0641 §8.52).
pub const RESOLVE_CHUMEI: u16 = 736;
/// resolve_shuaiyan(player, skill, stage) — ABI 715 (authoritative machine
/// for 率言, ADR 0641 §8.54).
pub const RESOLVE_SHUAIYAN: u16 = 738;
/// resolve_mbbeiming(player, skill, stage) — ABI 718 (authoritative machine
/// for 孛明, ADR 0641 §8.57).
pub const RESOLVE_MBBEIMING: u16 = 742;
/// reset_linked_selected_targets(source) — ABI 719 (duansuo 断索 active
/// primitive, ADR 0641 §8.58).
pub const RESET_LINKED_SELECTED_TARGETS: u16 = 743;
/// resolve_gaoyuan(player, source, card) — ABI 720 (告援 Sha-redirect
/// machine, ADR 0641 §8.59).
pub const RESOLVE_GAOYUAN: u16 = 744;
/// resolve_duoduan(player, source, card) — ABI 721 (度断 recast-then-
/// choose machine, ADR 0641 §8.60).
pub const RESOLVE_DUODUAN: u16 = 745;
/// resolve_dcanxu(player) — ABI 722 (暗蓄 active machine, ADR 0641 §8.61).
pub const RESOLVE_DCANXU: u16 = 746;
/// resolve_chulao(player) — ABI 723 (黜老 active machine, ADR 0641 §8.62).
pub const RESOLVE_CHULAO: u16 = 747;
/// resolve_friendbiwei(player, target) — ABI 724 (鄙位 rank-compare
/// machine, ADR 0641 §8.63).
pub const RESOLVE_FRENDBIWEI: u16 = 748;
/// resolve_mbzhenann(player) — ABI 725 (镇南 target-count-threshold
/// machine, ADR 0641 §8.64).
pub const RESOLVE_MBZHENANN: u16 = 749;
/// resolve_meiyong(player) — ABI 726 (姝勇 Sha-follow-up machine,
/// ADR 0641 §8.65).
pub const RESOLVE_MEIYONG: u16 = 750;
/// resolve_qiangyong(player, target) — ABI 727 (羌勇 damage-attached
/// machine, ADR 0641 §8.66).
pub const RESOLVE_QIANGYONG: u16 = 751;
/// resolve_dingce(player, source) — ABI 730 (定策 damage-follow-up
/// machine, ADR 0641 §8.69).
pub const RESOLVE_DINGCE: u16 = 752;
/// resolve_dcqiaolve(player) — ABI 732 (巧略 cost-added machine,
/// ADR 0641 §8.71).
pub const RESOLVE_DCQIAOLVE: u16 = 753;
/// resolve_yixiang(player) — ABI 734 (义襄 random basic gain machine,
/// ADR 0641 §8.73).
pub const RESOLVE_YIXIANG: u16 = 755;
/// resolve_xinfu_qiai(player) — ABI 735 (求援 dying hand-around machine,
/// ADR 0641 §8.75).
pub const RESOLVE_XINFU_QIAI: u16 = 756;
/// resolve_olwenren(player) — ABI 736 (温仁 conditional multi-target draw
/// machine, ADR 0641 §8.76).
pub const RESOLVE_OLWENREN: u16 = 757;
/// resolve_spolaige(player) — ABI 737 (spolaige second-dying awakening
/// machine, ADR 0641 §8.77).
pub const RESOLVE_SPOLAIGE: u16 = 758;
/// resolve_olzhijie(player) — ABI 738 (直谏 end-phase in-range damage
/// machine, ADR 0641 §8.78).
pub const RESOLVE_OLZHIJIE: u16 = 759;
/// resolve_olsuyi(player, target) — ABI 739 (俗义 deck-equipment gift
/// machine, ADR 0641 §8.79).
pub const RESOLVE_OLSUYI: u16 = 760;
/// resolve_olqiaozhi(player, stage) — ABI 740 (巧织 weave-gain machine,
/// ADR 0641 §8.80).
pub const RESOLVE_OLQIAOZHI: u16 = 761;
/// resolve_dczhee(player, mode) — ABI 746 (闪电挂判定 machine,
/// ADR 0641 §8.86).
pub const RESOLVE_DCZHEE: u16 = 763;
/// resolve_dchaoxian(player) — ABI 747 (朝献 rank-3 collection machine,
/// ADR 0641 §8.87).
pub const RESOLVE_DCHAOXIAN: u16 = 764;
/// resolve_zhongjie(player) — ABI 748 (张介 death-gift machine,
/// ADR 0641 §8.88).
pub const RESOLVE_ZHONGJIE: u16 = 765;
/// resolve_starcuxia_weiming(player, source, variant) — ABI 750
/// (孪生 random-discard machine, ADR 0641 §8.90).
pub const RESOLVE_STARCUXIA_WEIMING: u16 = 766;
/// resolve_sanshou(player) — ABI 751 (三首 type-reveal machine,
/// ADR 0641 §8.91).
pub const RESOLVE_SANSHOU: u16 = 767;
/// resolve_nhlundao(player, source) — ABI 752 (论道 hand-compare
/// machine, ADR 0641 §8.93).
pub const RESOLVE_NHLUNDAO: u16 = 768;
/// resolve_xiangshu(player, target) — ABI 753 (襄戍 limited support
/// machine, ADR 0641 §8.94).
pub const RESOLVE_XIANGSHU: u16 = 769;
/// resolve_twxiongjun(player) — ABI 733 (雄军 batch-draw machine,
/// ADR 0641 §8.72).
pub const RESOLVE_TW_XIONGJUN: u16 = 754;
/// resolve_zhengjian(player, skill, stage, event_player) — ABI 716
/// (authoritative machine for 诤荐, ADR 0641 §8.55).
pub const RESOLVE_ZHENGJIAN: u16 = 739;
/// resolve_reliangyin(player, skill, mode) — ABI 717 (authoritative machine
/// for 良姻 mobile 重印版, ADR 0641 §8.56).
pub const RESOLVE_RELIANGYIN: u16 = 741;
/// resolve_qinmian(player, skill, mode, amount) — ABI 714 (authoritative
/// machine for 勤勉, ADR 0641 §8.53).
pub const RESOLVE_QINMIAN: u16 = 737;
/// place_hand_cards_to_deck_top(player, skill, count) — ABI 689
/// (chooseToMove family, ADR 0641 §8.8): select `count` cards from hand,
/// place them on top of the draw pile.
pub const PLACE_HAND_CARDS_TO_DECK_TOP: u16 = 712;
/// ABI 652: choose one other alive player and adjust one of their marks.
pub const CHOOSE_OTHER_ADD_MARK: u16 = 672;
/// ABI 653: resolve the jsrgchaozheng debate.
pub const RESOLVE_JSRGCHAOZHENG: u16 = 673;
/// ABI 654: resolve the jsrgsheju two-player debate.
pub const RESOLVE_JSRGSHEJU: u16 = 674;
/// ABI 655: start a debate among all other players.
pub const START_DEBATE_ALL_OTHERS: u16 = 675;
/// ABI 656: start a debate with selected targets.
pub const START_DEBATE_TARGETS: u16 = 676;
/// ABI 657: discard any number of basic cards and draw equal.
pub const DISCARD_BASIC_DRAW_EQUAL: u16 = 677;
/// ABI 658: pin debate opinions (self picks two, others random).
pub const PIN_DEBATE_OPINIONS: u16 = 678;
/// ABI 659: align debate opinions of players damaged by the owner this turn.
pub const ALIGN_DEBATE_OPINIONS: u16 = 679;
/// ABI 660: gain N shadow (ying) cards into hand.
pub const GAIN_SHADOW_CARDS: u16 = 680;
/// ABI 661: change a player's faction/group.
pub const CHANGE_GROUP: u16 = 681;
/// ABI 662: distribute N shadow cards among chosen players.
pub const DISTRIBUTE_SHADOW_CARDS: u16 = 682;
/// ABI 663: register turn-end debate participants.
pub const REGISTER_TURN_END_DEBATE: u16 = 683;
/// ABI 663: resolve the registered turn-end debate.
pub const RESOLVE_TURN_END_DEBATES: u16 = 684;
/// ABI 664: choose a faction and change to it.
pub const CHOOSE_GROUP: u16 = 685;
/// choose_gain_other_skill(player, self_skill) — ABI 675
pub const CHOOSE_GAIN_OTHER_SKILL: u16 = 698;
/// lose_gained_skill(player, self_skill) — ABI 676
pub const LOSE_GAINED_SKILL: u16 = 699;
/// resolve_potheyun(player, self_skill) — ABI 677
pub const RESOLVE_POTHEYUN: u16 = 700;
/// choose_hand_compare_grant_target_on_win(player, win_grant_def, lose_skill_def) — ABI 678
pub const CHOOSE_HAND_COMPARE_GRANT_TARGET_ON_WIN: u16 = 701;
/// give_selected_then_recover_self(player, target, amount) — ABI 679
pub const GIVE_SELECTED_THEN_RECOVER_SELF: u16 = 702;
/// choose_hand_compare_between(player, first, skill) — ABI 680
pub const CHOOSE_HAND_COMPARE_BETWEEN: u16 = 703;
/// guanxing_top(player, count, skill) — ABI 681
pub const GUANXING_TOP: u16 = 704;
/// choose_other_give_then_recover(player, skill) — ABI 682
pub const CHOOSE_OTHER_GIVE_THEN_RECOVER: u16 = 705;
/// guanxing_top_by_falu_marks(player, m1, m2, m3, m4, skill) — ABI 683
pub const GUANXING_TOP_BY_FALU_MARKS: u16 = 706;
/// ABI 665: place a card as a xumou (scheme) card in the judge zone.
pub const PLACE_XUMOU: u16 = 686;
/// ABI 666: move the draw-pile top card into the owner's judge zone as a xumou.
pub const PLACE_DECK_TOP_XUMOU: u16 = 687;
/// ABI 666: choose one hand card and move it into the judge zone as a xumou.
pub const CHOOSE_PLACE_HAND_XUMOU: u16 = 688;
/// ABI 666: deterministically discard one tagged xumou card from own judge zone.
pub const DISCARD_ONE_OWN_XUMOU: u16 = 689;
/// ABI 667: move the draw-pile top N cards face-up into the owner's expansion pile.
pub const PLACE_DECK_TOP_EXPANSION: u16 = 690;
/// ABI 668: remove one City card and draw X (lost-count, cap 5) for a target.
pub const CITY_CONSUME: u16 = 691;
/// ABI 669: enqueue one extra turn for the player with a post-check.
pub const QUEUE_EXTRA_TURN: u16 = 692;
/// ABI 670: choose 1..N other players to each gain a mark.
pub const CHOOSE_OTHERS_ADD_MARK: u16 = 693;
/// ABI 671: choose one other player to enqueue an extra turn.
pub const CHOOSE_OTHERS_INSERT_EXTRA_TURN: u16 = 694;
/// ABI 672: enqueue a cropped extra turn limited to the masked phases.
pub const QUEUE_EXTRA_PHASES: u16 = 695;
/// ABI 673: twjieqiu multi-action (disable slots / judge leftover slots).
pub const RESOLVE_TWJIEQIU: u16 = 696;
/// ABI 674: skip the masked remaining phases of the current turn.
pub const SKIP_REMAINING_PHASES: u16 = 697;
/// choose_end_extra_draw_or_play_by_used_suits(player)
pub const CHOOSE_END_EXTRA_DRAW_OR_PLAY_BY_USED_SUITS: u16 = 228;
/// lose_hp_double_current_damage_then_replace(player, replacement_skill_def)
pub const LOSE_HP_DOUBLE_CURRENT_DAMAGE_THEN_REPLACE: u16 = 229;
/// resolve_mass_damage_discard_hand_equip(player, mark, mark_count, damage, discard_count)
pub const RESOLVE_MASS_DAMAGE_DISCARD_HAND_EQUIP: u16 = 230;
/// choose_gain_card_from_each_other_then_turn_over(player)
pub const CHOOSE_GAIN_CARD_FROM_EACH_OTHER_THEN_TURN_OVER: u16 = 231;
/// resolve_hidden_skill_pile(player, pile_skill_def, action, card)
pub const RESOLVE_HIDDEN_SKILL_PILE: u16 = 232;
/// set_non_locked_skill_suppression(player, enabled)
pub const SET_NON_LOCKED_SKILL_SUPPRESSION: u16 = 233;
/// resolve_dili_yuheng(player, action, card)
pub const RESOLVE_DILI_YUHENG: u16 = 234;
/// discard_selected_draw_all_hand_bonus(player)
pub const DISCARD_SELECTED_DRAW_ALL_HAND_BONUS: u16 = 235;
/// choose_other_yinghun(player)
pub const CHOOSE_OTHER_YINGHUN: u16 = 236;
/// choose_other_bizheng(player)
pub const CHOOSE_OTHER_BIZHENG: u16 = 237;
/// choose_other_xiashu(player)
pub const CHOOSE_OTHER_XIASHU: u16 = 238;
/// resolve_guanchao_rank(player, card)
pub const RESOLVE_GUANCHAO_RANK: u16 = 239;
/// resolve_guanwei(player, event_player)
pub const RESOLVE_GUANWEI: u16 = 240;
/// resolve_lanjiang(player)
pub const RESOLVE_LANJIANG: u16 = 241;
/// resolve_rebingyi(player)
pub const RESOLVE_REBINGYI: u16 = 242;
/// resolve_rebotu(player)
pub const RESOLVE_REBOTU: u16 = 243;
/// resolve_oldimeng(player, action)
pub const RESOLVE_OLDIMENG: u16 = 244;
/// resolve_rejieyin(player)
pub const RESOLVE_REJIEYIN: u16 = 245;
/// resolve_reanxu(player)
pub const RESOLVE_REANXU: u16 = 246;
/// resolve_drlt_jueyan(player, sha_skill, weapon_skill, mount_skill, armor_skill, treasure_skill)
pub const RESOLVE_DRLT_JUEYAN: u16 = 247;
/// resolve_rejizhi(player)
pub const RESOLVE_REJIZHI: u16 = 248;
/// resolve_jiexun(player, funan_helper_skill)
pub const RESOLVE_JIEXUN: u16 = 249;
/// resolve_wengua_setup(player, helper_skill)
pub const RESOLVE_WENGUA_SETUP: u16 = 250;
/// resolve_wengua(player, main_skill, marker_skill)
pub const RESOLVE_WENGUA: u16 = 251;
/// resolve_xinanguo(player)
pub const RESOLVE_XINANGUO: u16 = 252;
/// resolve_ganglie_three(player)
pub const RESOLVE_GANGLIE_THREE: u16 = 253;
/// resolve_zhongyi(player, action, helper_skill_def)
pub const RESOLVE_ZHONGYI: u16 = 254;
/// discard_equipment_slot(player, slot)
pub const DISCARD_EQUIPMENT_SLOT: u16 = 255;
/// resolve_oldanxu(player)
pub const RESOLVE_OLDANXU: u16 = 256;
/// store_selected_in_skill_pile(player)
pub const STORE_SELECTED_IN_SKILL_PILE: u16 = 257;
/// end_match_for_player_camp_if_skill_pile_at_least(player, pile_skill_def, count)
pub const END_MATCH_FOR_CAMP_IF_SKILL_PILE_AT_LEAST: u16 = 258;
/// resolve_oldqinqing(player)
pub const RESOLVE_OLDQINQING: u16 = 259;
/// resolve_oldmingjian(player)
pub const RESOLVE_OLDMINGJIAN: u16 = 260;
/// resolve_oldxuanfeng(player)
pub const RESOLVE_OLDXUANFENG: u16 = 261;
/// resolve_jianrou(player)
pub const RESOLVE_JIANROU: u16 = 262;
/// resolve_oldhuisheng(player)
pub const RESOLVE_OLDHUISHENG: u16 = 263;
/// resolve_oldjiefan(player, stage)
pub const RESOLVE_OLDJIEFAN: u16 = 264;
/// resolve_chegu(player)
pub const RESOLVE_CHEGU: u16 = 265;
/// resolve_junkguixin(player)
pub const RESOLVE_JUNKGUIXIN: u16 = 266;
/// resolve_yao_chenwei(player)
pub const RESOLVE_YAO_CHENWEI: u16 = 267;
/// choose_add_current_sha_target_within_distance(player, card, max_distance)
pub const CHOOSE_ADD_CURRENT_SHA_TARGET_WITHIN_DISTANCE: u16 = 268;
/// refund_current_sha_use(player, card)
pub const REFUND_CURRENT_SHA_USE: u16 = 269;
/// damage_then_draw_selected_targets(source, damage, draw)
pub const DAMAGE_THEN_DRAW_SELECTED_TARGETS: u16 = 270;
/// resolve_youdi(player)
pub const RESOLVE_YOUDI: u16 = 271;
/// resolve_kuangfu(player, target)
pub const RESOLVE_KUANGFU: u16 = 272;
/// resolve_shanjia(player, count)
pub const RESOLVE_SHANJIA: u16 = 273;
/// resolve_new_yajiao(player, card)
pub const RESOLVE_NEW_YAJIAO: u16 = 274;
/// resolve_dingpin(player, target)
pub const RESOLVE_DINGPIN: u16 = 275;
/// finish_current_phase_abort_resolution(player)
pub const FINISH_CURRENT_PHASE_ABORT_RESOLUTION: u16 = 276;
/// gain_top_card_to_skill_pile(player)
pub const GAIN_TOP_CARD_TO_SKILL_PILE: u16 = 277;
/// choose_discard_skill_pile_card_add_marked_skill(player, target, helper_skill_def)
pub const CHOOSE_DISCARD_SKILL_PILE_CARD_ADD_MARKED_SKILL: u16 = 278;
/// gain_one_hand_from_selected_targets(player)
pub const GAIN_ONE_HAND_FROM_SELECTED_TARGETS: u16 = 279;
/// choose_repolu_targets_draw(player)
pub const CHOOSE_REPOLU_TARGETS_DRAW: u16 = 280;
/// resolve_xinfu_guolun(player, target)
pub const RESOLVE_XINFU_GUOLUN: u16 = 281;
/// choose_discard_one_for_sha_refund_and_draw_if_red_restore_if_equal_rank(
///     player, sha_user, card, restore_skill_def
/// )
pub const CHOOSE_DISCARD_ONE_FOR_SHA_REFUND_AND_DRAW_IF_RED_RESTORE_IF_EQUAL_RANK: u16 = 282;
/// resolve_beige_configured(
///     player, target, source, heart_recover_amount, diamond_draw_count
/// )
pub const RESOLVE_BEIGE_CONFIGURED: u16 = 283;
/// resolve_zhuiyi(player, killer)
pub const RESOLVE_ZHUIYI: u16 = 284;
/// resolve_decadezhenjun(player)
pub const RESOLVE_DECADEZHENJUN: u16 = 285;
/// resolve_sp_xiaoguo(player, target)
pub const RESOLVE_SP_XIAOGUO: u16 = 286;
/// choose_gain_card_from_each_other_replace_draw(player)
pub const CHOOSE_GAIN_CARD_FROM_EACH_OTHER_REPLACE_DRAW: u16 = 287;
/// give_selected_then_recover_target(player, target, amount)
pub const GIVE_SELECTED_THEN_RECOVER_TARGET: u16 = 288;
/// resolve_dingpan(player, target)
pub const RESOLVE_DINGPAN: u16 = 289;
/// resolve_tanlu(player, target)
pub const RESOLVE_TANLU: u16 = 290;
/// judge_same_color_gain_repeat(player)
pub const JUDGE_SAME_COLOR_GAIN_REPEAT: u16 = 291;
/// choose_current_trick_targets_to_nullify(player, source, card)
pub const CHOOSE_CURRENT_TRICK_TARGETS_TO_NULLIFY: u16 = 292;
/// judge_black_gain_repeat_protect_hand_limit(player)
pub const JUDGE_BLACK_GAIN_REPEAT_PROTECT_HAND_LIMIT: u16 = 293;
/// choose_gain_all_resolved_sha_shan_then_color_rewards(player, sha_card)
pub const CHOOSE_GAIN_ALL_RESOLVED_SHA_SHAN_THEN_COLOR_REWARDS: u16 = 294;
/// choose_other_hand_targets_gain_one_reduce_draw(player)
pub const CHOOSE_OTHER_HAND_TARGETS_GAIN_ONE_REDUCE_DRAW: u16 = 295;
/// choose_discard_target_card_resolve_current_sha_nonbasic_bonus(
///     source, card, target, bonus_skill_def
/// )
pub const CHOOSE_DISCARD_TARGET_CARD_RESOLVE_CURRENT_SHA_NONBASIC_BONUS: u16 = 296;
/// resolve_dcqiaomeng(player, card, helper_skill_def)
pub const RESOLVE_DCQIAOMENG: u16 = 297;
/// choose_target_draw_or_discard_by_skill_use_count_toggle_link_if_wounded(player, target)
pub const CHOOSE_TARGET_DRAW_OR_DISCARD_BY_SKILL_USE_COUNT_TOGGLE_LINK_IF_WOUNDED: u16 = 298;
/// choose_three_way_draw_or_discard_add_skill(player, draw_skill_def, discard_skill_def)
pub const CHOOSE_THREE_WAY_DRAW_OR_DISCARD_ADD_SKILL: u16 = 299;
/// resolve_retieji(player, card, target)
pub const RESOLVE_RETIEJI: u16 = 300;
/// resolve_new_reluoyi(player, buff_skill_def)
pub const RESOLVE_NEW_RELUOYI: u16 = 301;
/// resolve_remoshi(player, target, card, helper_skill_def, action)
pub const RESOLVE_REMOSHI: u16 = 302;
/// resolve_xinyaoming(player)
pub const RESOLVE_XINYAOMING: u16 = 303;
/// resolve_rehuaiyi(player)
pub const RESOLVE_REHUAIYI: u16 = 304;
/// resolve_reanguo(player)
pub const RESOLVE_REANGUO: u16 = 305;
/// choose_target_discard_distance_hand_or_increase_damage(player, target)
pub const CHOOSE_TARGET_DISCARD_DISTANCE_HAND_OR_INCREASE_DAMAGE: u16 = 306;
/// choose_draw_view_two_gain_or_move_equipment(player, target)
pub const CHOOSE_DRAW_VIEW_TWO_GAIN_OR_MOVE_EQUIPMENT: u16 = 307;
/// resolve_beizhu(player, target)
pub const RESOLVE_BEIZHU: u16 = 308;
/// resolve_mbshanjia(player, lost_equipment_count, distance_skill_def)
pub const RESOLVE_MBSHANJIA: u16 = 309;
/// resolve_daoji(player, target)
pub const RESOLVE_DAOJI: u16 = 310;
/// resolve_jiaohua(player)
pub const RESOLVE_JIAOHUA: u16 = 311;
/// resolve_luanqun(player, effect_skill_def, direct_hit_skill_def)
pub const RESOLVE_LUANQUN: u16 = 312;
/// choose_give_two_different_colors_or_recover(owner, giver, target)
pub const CHOOSE_GIVE_TWO_DIFFERENT_COLORS_OR_RECOVER: u16 = 313;
/// resolve_mbzhuguo(player, target)
pub const RESOLVE_MBZHUGUO: u16 = 314;
/// resolve_mbanxian(player)
pub const RESOLVE_MBANXIAN: u16 = 315;
/// resolve_mbjinzu / apply_mbjinzu_next_sha
pub const RESOLVE_MBJINZU: u16 = 316;
/// take_one_card(from, to, hand_equip_judge)
pub const TAKE_ONE_SCOPED_CARD: u16 = 317;
/// reveal_hand(player)
pub const REVEAL_HAND: u16 = 318;
/// draw_linked_players(count)
pub const DRAW_LINKED_PLAYERS: u16 = 319;
/// choose_scskuiji_discard(player, target, count)
pub const CHOOSE_SCSKUIJI_DISCARD: u16 = 320;
/// choose_pskuiji_discard(player, target, count)
pub const CHOOSE_PSKKUIJI_DISCARD: u16 = 321;
/// reveal_top_gain_if_color(player, color_mask)
pub const REVEAL_TOP_GAIN_IF_COLOR: u16 = 323;
/// spjungong_discard_or_lose_then_sha(player, target, skill)
pub const SPJUNGONG_DISCARD_OR_LOSE_THEN_SHA: u16 = 324;
/// ban_active_until_turn_end(player, skill)
pub const BAN_ACTIVE_UNTIL_TURN_END: u16 = 325;
/// resolve_tw_daoji(player, target)
pub const RESOLVE_TW_DAOJI: u16 = 326;
/// spwanwei_active(player, target)
pub const SPWANWEI_ACTIVE: u16 = 327;
/// spwanwei_dying(player, target)
pub const SPWANWEI_DYING: u16 = 328;
/// suppress_non_locked_skills_until_turn_end(player)
pub const SUPPRESS_NON_LOCKED_SKILLS_UNTIL_TURN_END: u16 = 329;
/// choose_recover_target(chooser, target, amount)
pub const CHOOSE_RECOVER_TARGET: u16 = 330;
/// give_all_hand_then_choose_use_one_or_damage(player, target)
pub const GIVE_ALL_HAND_THEN_CHOOSE_USE_ONE_OR_DAMAGE: u16 = 331;
/// choose_category_then_target_give_matching_or_sha_unrespondable(player, card, target)
pub const CHOOSE_CATEGORY_THEN_TARGET_GIVE_MATCHING_OR_SHA_UNRESPONDABLE: u16 = 332;
/// choose_lose_hp_use_virtual_sha_by_basic_variants(player)
pub const CHOOSE_LOSE_HP_USE_VIRTUAL_SHA_BY_BASIC_VARIANTS: u16 = 333;
/// resolve_spxizhan(player, phase_player, fangzong_skill_def)
pub const RESOLVE_SPXIZHAN: u16 = 334;
/// resolve_twfupan(player, damage_amount)
pub const RESOLVE_TWFUPAN: u16 = 335;
/// resolve_spsongshu(player, phase_player, renku_skill_def, block_skill_def)
pub const RESOLVE_SPSONGSHU: u16 = 336;
/// resolve_twjiaohua(player, target)
pub const RESOLVE_TWJIAOHUA: u16 = 337;
/// resolve_twkujian(player, target, action)
pub const RESOLVE_TWKUJIAN: u16 = 338;
/// resolve_twruilian(player, target, action)
pub const RESOLVE_TWRUILIAN: u16 = 339;
/// skip_judge_and_draw_phases(player)
pub const SKIP_JUDGE_AND_DRAW_PHASES: u16 = 340;
/// equip_weapon_from_draw_pile(player)
pub const EQUIP_WEAPON_FROM_DRAW_PILE: u16 = 341;
/// resolve_twjianwei(player, actor, stage)
pub const RESOLVE_TWJIANWEI: u16 = 342;
/// resolve_spcunsi(player, target)
pub const RESOLVE_SPCUNSI: u16 = 343;
/// resolve_twtanfeng(player)
pub const RESOLVE_TWTANFENG: u16 = 344;
/// resolve_twzhibian(player)
pub const RESOLVE_TWZHIBIAN: u16 = 345;
/// resolve_twyuyan(player, source, card)
pub const RESOLVE_TWYUYAN: u16 = 346;
/// resolve_twguoyi(player, target, card, hand_lock_skill_def)
pub const RESOLVE_TWGUOYI: u16 = 347;
/// resolve_twchuhai(player, action)
pub const RESOLVE_TWCHUHAI: u16 = 348;
/// resolve_twyizhu(player, source, card, action)
pub const RESOLVE_TWYIZHU: u16 = 349;
/// resolve_twluanchou(player, helper_skill_def, action)
pub const RESOLVE_TWLUANCHOU: u16 = 350;
/// resolve_twxiawei(player, wangxing_skill_def, action)
pub const RESOLVE_TWXIAWEI: u16 = 351;
/// resolve_twbianfuren(player, target, helper_skill_def, action)
pub const RESOLVE_TWBIANFUREN: u16 = 352;
/// resolve_twfeiyi(player, action)
pub const RESOLVE_TWFEIYI: u16 = 353;
/// resolve_twchenzhen(player, target, helper_skill_def, record_skill_def, action)
pub const RESOLVE_TWCHENZHEN: u16 = 354;
/// resolve_twqianju(player, target)
pub const RESOLVE_TWQIANJU: u16 = 355;
/// resolve_twqingxi(player, target, card)
pub const RESOLVE_TWQINGXI: u16 = 356;
/// resolve_twzaoli(player)
pub const RESOLVE_TWZAOLI: u16 = 357;
/// resolve_twzhengjian(player, target, helper_skill_def, action)
pub const RESOLVE_TWZHENGJIAN: u16 = 358;
/// resolve_twtianyu(player, target, helper_skill_def, action)
pub const RESOLVE_TWTIANYU: u16 = 359;
/// resolve_twwujing(player, target_count, action)
pub const RESOLVE_TWWUJING: u16 = 360;
/// resolve_twshanghe(player)
pub const RESOLVE_TWSHANGHE: u16 = 361;
/// resolve_twwangchang(player, target, card, action)
pub const RESOLVE_TWWANGCHANG: u16 = 362;
/// resolve_twcaozhao(player, target, source, card, twfuzuan_def, twfeifu_def, action)
pub const RESOLVE_TWCAOZHAO: u16 = 363;
/// resolve_twguohuai(player, target, source, card, mark_def, helper_def, action)
pub const RESOLVE_TWGUOHUAI: u16 = 364;
/// resolve_twchengpu(player, target, source, card, chunlao_def, action)
pub const RESOLVE_TWCHENGPU: u16 = 365;
/// resolve_twcaoang(player, target, source, card, helper_def, action)
pub const RESOLVE_TWCAOANG: u16 = 367;
/// choose_target_draw_to_max_hp_cap_then_add_skill(player, cap, skill_def)
pub const CHOOSE_TARGET_DRAW_MAX_HP_CAP_THEN_ADD_SKILL: u16 = 366;
/// choose_discard_heart_redirect_current_damage(player, draw_skill_def)
pub const RESOLVE_TIANXIANG: u16 = 368;
/// resolve_twniujin(player, limited_skill_def, action, stage)
pub const RESOLVE_TWNIUJIN: u16 = 369;
/// choose_discard_heart_cancel_damage_branch(player)
pub const RESOLVE_RETIANXIANG: u16 = 370;
/// choose_link_other_same_hp(player)
pub const RESOLVE_LIANZHOU: u16 = 371;
/// choose_give_hand_equip_to_event_player(player)
pub const RESOLVE_STDDECHONG: u16 = 372;
/// resolve_twmoukui(player, target, card)
pub const RESOLVE_TWMOUKUI: u16 = 373;
/// resolve_twfuman(player, target)
pub const RESOLVE_TWFUMAN: u16 = 374;
/// discard_hand_equip_count(player, count)
pub const DISCARD_HAND_EQUIP_COUNT: u16 = 375;
/// resolve_jieliang_gain(player)
pub const RESOLVE_JIELIANG_GAIN: u16 = 376;
/// choose_other_gain_event_cards(player)
pub const RESOLVE_STDJUJIAN: u16 = 377;
/// resolve_cuijin(player, source, card, action)
pub const RESOLVE_CUIJIN: u16 = 378;
/// resolve_lulve(player)
pub const RESOLVE_LULVE: u16 = 379;
/// resolve_twqingkou(player, winner, card, action)
pub const RESOLVE_TWQINGKOU: u16 = 380;
/// resolve_dctingxian(player)
pub const RESOLVE_DCTINGXIAN: u16 = 381;
/// choose_other_add_skill(player, skill_def)
pub const RESOLVE_HFJIEYING: u16 = 382;
/// resolve_twchenxun(player, target, card, action)
pub const RESOLVE_TWCHENXUN: u16 = 383;
/// resolve_dcnuchen(player)
pub const RESOLVE_DCNUCHEN: u16 = 384;
/// resolve_dcdanji(player, skill)
pub const RESOLVE_DCDANJI: u16 = 385;
/// choose_discard_equipment_prevent_current_damage(player)
pub const CHOOSE_DISCARD_EQUIPMENT_PREVENT_CURRENT_DAMAGE: u16 = 386;
/// resolve_dczhiheng(player)
pub const RESOLVE_DCZHIHENG: u16 = 387;
/// resolve_twfenxian(player, target, card, action)
pub const RESOLVE_TWFENXIAN: u16 = 388;
/// resolve_stdgaobian(player)
pub const RESOLVE_STDGAOBIAN: u16 = 389;
/// resolve_spjiedao(player)
pub const RESOLVE_SPJIEDAO: u16 = 390;
/// resolve_sibian(player)
pub const RESOLVE_SIBIAN: u16 = 391;
/// skip_next_discard_phase(player)
pub const SKIP_NEXT_DISCARD_PHASE: u16 = 392;
/// resolve_spfengyin(player, target)
pub const RESOLVE_SPFENGYIN: u16 = 393;
/// set_skill_content_flag(player, bit, value)
pub const SET_SKILL_CONTENT_FLAG: u16 = 394;
/// resolve_xinfu_lianpian(player)
pub const RESOLVE_XINFU_LIANPIAN: u16 = 395;
/// resolve_gongjian(player)
pub const RESOLVE_GONGJIAN: u16 = 396;
/// resolve_xinfu_qinguo(player)
pub const RESOLVE_XINFU_QINGUO: u16 = 397;
/// reveal_choose_category_distribute_self_threshold_hp_loss(player, reveal_count, recipient_cap)
pub const REVEAL_CHOOSE_CATEGORY_DISTRIBUTE_SELF_THRESHOLD_HP_LOSS: u16 = 398;
/// resolve_dcmanzhi(player)
pub const RESOLVE_DCMANZHI: u16 = 399;
/// resolve_xinxingluan(player)
pub const RESOLVE_XINXINGLUAN: u16 = 400;
/// choose_other_random_gain_region_then_draw_compare_color_lose_hp(player)
pub const CHOOSE_OTHER_RANDOM_GAIN_REGION_THEN_DRAW_COMPARE_COLOR_LOSE_HP: u16 = 401;
/// resolve_dckuizhen(player)
pub const RESOLVE_DCKUIZHEN: u16 = 402;
/// resolve_dckuizhen_resolve(player, winner, loser)
pub const RESOLVE_DCKUIZHEN_RESOLVE: u16 = 403;
/// resolve_nsdingzhou(player)
pub const RESOLVE_NSDINGZHOU: u16 = 404;
/// resolve_sxrmlanjiao(player)
pub const RESOLVE_SXRMLANJIAO: u16 = 405;
/// resolve_refuhai(player)
pub const RESOLVE_REFUHAI: u16 = 406;
/// resolve_rekuanshi(player)
pub const RESOLVE_REKUANSHI: u16 = 407;
/// resolve_rekuanshi_resolve(player, damaged)
pub const RESOLVE_REKUANSHI_RESOLVE: u16 = 408;
/// resolve_dcmoyu(player)
pub const RESOLVE_DCMOYU: u16 = 409;
/// resolve_xizhen(player)
pub const RESOLVE_XIZHEN: u16 = 410;
/// choose_gain_target_hand_ceil_half_difference(player, target)
pub const CHOOSE_GAIN_TARGET_HAND_CEIL_HALF_DIFFERENCE: u16 = 411;
/// resolve_xuxie(player)
pub const RESOLVE_XUXIE: u16 = 412;
/// resolve_xuxie_add(player)
pub const RESOLVE_XUXIE_ADD: u16 = 413;
/// resolve_gzjinfa(player, target)
pub const RESOLVE_GZJINFA: u16 = 414;
/// resolve_dcchiying(player)
pub const RESOLVE_DCCHIYING: u16 = 415;
/// resolve_wn_qifeng(player, event_player)
pub const RESOLVE_WN_QIFENG: u16 = 416;
/// resolve_xinkuangfu(player)
pub const RESOLVE_XINKUANGFU: u16 = 417;
/// resolve_dcxiangshu(player)
pub const RESOLVE_DCXIANG_SHU: u16 = 418;
/// resolve_dcxiangshu_lottery(player, target)
pub const RESOLVE_DCXIANG_SHU_LOTTERY: u16 = 419;
/// give_selected_then_target_may_use_draw_owner(player, target)
pub const GIVE_SELECTED_THEN_TARGET_MAY_USE_DRAW_OWNER: u16 = 420;
/// Reserved by coordinated ABI 487 (`resolve_dcyingtu`).
pub const RESOLVE_DCYINGTU: u16 = 421;
/// choose_category_then_target_give_matching_or_damage_plus_one(player, target)
pub const CHOOSE_CATEGORY_THEN_TARGET_GIVE_MATCHING_OR_DAMAGE_PLUS_ONE: u16 = 422;
/// ABI 499: choose a non-source target, random-gain one he card, use it if equipment.
pub const CHOOSE_OTHER_EXCLUDING_SOURCE_RANDOM_GAIN_HAND_EQUIP_THEN_USE_IF_EQUIP: u16 = 423;
/// ABI 500: resolve Mobile Danshou from current-turn card-target history.
pub const RESOLVE_MOBILEDANSHOU: u16 = 424;
/// ABI 501: resolve JSRG Chiying's ordered forced discards and conditional basic-card gain.
pub const RESOLVE_JSRGCHIYING: u16 = 425;
/// ABI 503: resolve 清蹈 post-card branch.
pub const RESOLVE_MBQINGDAO: u16 = 426;
/// ABI 503: resolve 修耕 recording or draw-phase settlement.
pub const RESOLVE_MBXIUGENG: u16 = 427;
/// ABI 503: resolve 陈赦 ordered discards and rescue.
pub const RESOLVE_MBCHENSHE: u16 = 428;
/// ABI 504: resolve 权宠 extra-turn sequence.
pub const RESOLVE_MBQUNCHONG: u16 = 429;
/// ABI 504: resolve 任行 branch choice.
pub const RESOLVE_MBRENXING: u16 = 430;
/// ABI 505: resolve 鸱张 response-color lock.
pub const RESOLVE_MBCHIZHANG: u16 = 431;
/// ABI 505: resolve 断鞅 pile/use/recast sequence.
pub const RESOLVE_MBDUANYANG: u16 = 432;
/// 势陆郁生「润微」复合。ABI 506.
pub const RESOLVE_MBRUNWEI: u16 = 433;
/// 势陆郁生「霜怀」复合。ABI 506.
pub const RESOLVE_MBSHUANGHUAI: u16 = 434;
/// 势孙峻「凶图」复合。ABI 507.
pub const RESOLVE_MBXIONGTU: u16 = 435;
/// 势田丰「刚鲠」复合。ABI 508.
pub const RESOLVE_MBGANGGENG: u16 = 436;
/// 势田丰「死谏」复合。ABI 508.
pub const RESOLVE_MBSIJIAN: u16 = 437;
/// resolve_mbjianji(player)
pub const RESOLVE_MBJIANJI: u16 = 438;
/// resolve_mbyuanmo(player)
pub const RESOLVE_MBYUANMO: u16 = 439;
/// resolve_mbfeijing(player, target, card)
pub const RESOLVE_MBFEIJING: u16 = 440;
/// resolve_mbxiaoge(player, target, card, stage)
pub const RESOLVE_MBXIAOGE: u16 = 441;
/// resolve_shuangxiong(player)
pub const RESOLVE_SHUANGXIONG: u16 = 442;
/// resolve_zhijian(player)
pub const RESOLVE_ZHIJIAN: u16 = 443;
/// resolve_guzheng(player, event_player)
pub const RESOLVE_GUZHENG: u16 = 444;
/// resolve_mbxuye(player, target)
pub const RESOLVE_MBXUYE: u16 = 445;
/// resolve_mbkuangxiang(player, target, stage)
pub const RESOLVE_MBKUANGXIANG: u16 = 446;
/// resolve_potchengpu(player, target, stage)
pub const RESOLVE_POTCHENGPU: u16 = 447;
/// resolve_potjugu(player)
pub const RESOLVE_POTJUGU: u16 = 448;
/// resolve_potsifeng(player, target, card, stage)
pub const RESOLVE_POTSIFENG: u16 = 449;
/// resolve_potchenqun(player, target, stage)
pub const RESOLVE_POTCHENQUN: u16 = 450;
/// resolve_potwanglie(player, card, helper_skill_def, stage)
pub const RESOLVE_POTWANGLIE: u16 = 451;
/// resolve_pothongyi(player, stage)
pub const RESOLVE_POTHONGYI: u16 = 452;
/// resolve_potguansha(player, helper_skill_def)
pub const RESOLVE_POTGUANSHA: u16 = 453;
/// resolve_potjiyu(player, stage)
pub const RESOLVE_POTJIYU: u16 = 454;
/// resolve_mbzhuji(player, stage)
pub const RESOLVE_MBZHUJI: u16 = 455;
/// resolve_potlibing(player, card, stage)
pub const RESOLVE_POTLIBING: u16 = 456;
/// resolve_potpoxi(player)
pub const RESOLVE_POTPOXI: u16 = 457;
/// resolve_yuli(player, mode) — ABI 409
pub const RESOLVE_YULI: u16 = 458;
/// resolve_tingwei(player, mode) — ABI 410
pub const RESOLVE_TINGWEI: u16 = 459;
/// resolve_zhiba(player, event_player) — ABI 523 (omp, 制霸)
pub const RESOLVE_ZHIBA: u16 = 460;
/// resolve_zhiba_settle(...) — ABI 523 (omp, 制霸结算)
pub const RESOLVE_ZHIBA_SETTLE: u16 = 461;
/// resolve_drlt_congjian(player, targets) — ABI 524 (omp, 从俭)
pub const RESOLVE_DRLT_CONGJIAN: u16 = 462;
/// resolve_potfuan(player, card, stage, range_helper_def, yinxian_def) — ABI 525
pub const RESOLVE_POTFUAN: u16 = 463;
/// resolve_potyinxian(player, card, stage) — ABI 525
pub const RESOLVE_POTYINXIAN: u16 = 464;
/// resolve_nzry_feijun(player) — ABI 526 (omp, 飞军)
pub const RESOLVE_NZRY_FEIJUN: u16 = 465;
/// resolve_drlt_yongsi(player, stage) — ABI 527 (omp, 庸肆)
pub const RESOLVE_DRLT_YONGSI: u16 = 466;
/// resolve_drlt_weidi(player) — ABI 528 (omp, 伪帝)
pub const RESOLVE_DRLT_WEIDI: u16 = 467;
/// resolve_lieren(player, target, win_skill_def, lose_skill_def) — ABI 531 (omp, 烈刃)
pub const RESOLVE_LIEREN: u16 = 469;
/// resolve_xinmingshi(player, source) — ABI 411 (Claude, 新名士)
pub const RESOLVE_XINMINGSHI: u16 = 468;
/// resolve_pothuanjie(player, target, stage, qice_def, kanpo_def) — ABI 529
pub const RESOLVE_POTHUANJIE: u16 = 473;
/// resolve_xunyi(player, mode) — ABI 412 (Claude, 殉义)
pub const RESOLVE_XUNYI: u16 = 470;
/// resolve_boming(player, target) — ABI 413 (Claude, 博明)
pub const RESOLVE_BOMING: u16 = 471;
/// resolve_ejian(player, target) — ABI 414 (Claude, 恶荐)
pub const RESOLVE_EJIAN: u16 = 472;
/// resolve_potnigu(player, stage) — ABI 532
pub const RESOLVE_POTNIGU: u16 = 474;
/// resolve_potlulian(player, card, stage) — ABI 532
pub const RESOLVE_POTLULIAN: u16 = 475;
/// resolve_yiyong(player) — ABI 415 (Claude, 异勇)
pub const RESOLVE_YIYONG: u16 = 476;
/// resolve_shanxie(player) — ABI 416 (Claude, 善械)
pub const RESOLVE_SHANXIE: u16 = 477;
/// resolve_tuntian(player) — ABI 533 (omp, 屯田)
pub const RESOLVE_TUNTIAN: u16 = 478;
/// resolve_zaoxian(player, jixi_skill_def, tuntian_skill_def) — ABI 534 (omp, 凿险)
pub const RESOLVE_ZAOXIAN: u16 = 479;
/// resolve_potdimeng(player) — ABI 535 (势鲁肃「缔盟」)
pub const RESOLVE_POTDIMENG: u16 = 480;
/// resolve_sheyi(player, target) — ABI 417 (Claude, 舍医)
pub const RESOLVE_SHEYI: u16 = 481;
/// resolve_tianyin(player) — ABI 418 (Claude, 天荫)
pub const RESOLVE_TIANYIN: u16 = 482;
/// resolve_pothaoshi(player, action) — ABI 536 (势鲁肃「好施」)
pub const RESOLVE_POTHAOSHI: u16 = 483;
/// resolve_tiaoxin(player, target, stage) — ABI 537 (omp, 挑衅)
pub const RESOLVE_TIAOXIN: u16 = 484;
/// resolve_zhiji(player, guanxing_def) — ABI 538 (omp, 志继)
pub const RESOLVE_ZHIJI: u16 = 485;
/// resolve_spmiaolve(player, action) — ABI 539 (势董昭「妙略」)
pub const RESOLVE_SPMIAOLVE: u16 = 486;
/// resolve_spyingjia(player, action, helper_def) — ABI 539 (势董昭「迎驾」)
pub const RESOLVE_SPYINGJIA: u16 = 487;
/// resolve_dbzhuifeng(player) — ABI 419 (Claude, 追锋)
pub const RESOLVE_DBZHUIFENG: u16 = 488;
/// resolve_dbquedi(player, target, card) — ABI 420 (Claude, 曲迪)
pub const RESOLVE_DBQUEDI: u16 = 489;
/// resolve_zaiqi(player, extra) — ABI 540 (omp, 再起)
pub const RESOLVE_ZAIQI: u16 = 490;
/// resolve_twqiushou(player) — ABI 542 (omp, 求首)
pub const RESOLVE_TWQIUSHOU: u16 = 491;
/// resolve_pottuntian(player, action, zaoxian_def) — ABI 543 (势邓艾)
pub const RESOLVE_POTTUNTIAN: u16 = 492;
/// resolve_potjixi(player, target, action) — ABI 543 (势邓艾)
pub const RESOLVE_POTJIXI: u16 = 493;
/// resolve_zhengrong(player, target, stage) — ABI 545 (omp, 征荣)
pub const RESOLVE_ZHENGRONG: u16 = 494;
/// resolve_hongju(player, qingjian_def, stage) — ABI 546 (omp, 鸿举)
pub const RESOLVE_HONGJU: u16 = 495;
/// resolve_qingjian(player, stage) — ABI 547 (omp, 清俭)
pub const RESOLVE_QINGJIAN: u16 = 496;
/// resolve_potceduan(player, target, action) — ABI 544 (势陈矫)
pub const RESOLVE_POTCEDUAN: u16 = 499;
/// resolve_spyinju(player, target) — ABI 421 (Claude, 隐据)
pub const RESOLVE_SPYINJU: u16 = 497;
/// resolve_spchijie(player, source, card) — ABI 422 (Claude, 持节)
pub const RESOLVE_SPCHIJIE: u16 = 498;
/// resolve_spshangyi(player, target) — ABI 423 (Claude, 上议)
pub const RESOLVE_SPSHANGYI: u16 = 500;
/// resolve_spjianyi(player) — ABI 424 (Claude, 简议)
pub const RESOLVE_SPJIANYI: u16 = 501;
/// resolve_pottaishici(player, target, action, hanzhan_def, zhanlie_def) — ABI 548
pub const RESOLVE_POTTAISHICI: u16 = 502;
/// resolve_releiji(player, target, stage) — ABI 549 (omp, 雷击)
pub const RESOLVE_RELEIJI: u16 = 503;
/// resolve_huangtian(player, actor, stage) — ABI 551 (omp, 黄天)
pub const RESOLVE_HUANGTIAN: u16 = 504;
/// resolve_fyjianyu(player, a, b) — ABI 425 (Claude, 谏喻)
pub const RESOLVE_FYJIANYU: u16 = 505;
/// resolve_liubing(player, card) — ABI 426 (Claude, 流冰)
pub const RESOLVE_LIUBING: u16 = 506;
/// resolve_heji(player, target, card) — ABI 427 (Claude, 合击)
pub const RESOLVE_HEJI: u16 = 507;
/// resolve_mbzhonghui(player, target, action, amount, mbsizi_def, mbkechang_def) — ABI 553
pub const RESOLVE_MBZHONGHUI: u16 = 508;
/// resolve_haoshi(player, stage) — ABI 555 (omp, 好施)
pub const RESOLVE_HAOSHI: u16 = 509;
/// resolve_dimeng(player, targets, stage) — ABI 556 (omp, 缔盟)
pub const RESOLVE_DIMENG: u16 = 510;
/// resolve_potjiejie(player, actor, used_def, spade_def, heart_def, club_def, diamond_def, qingshi_def) — ABI 554
pub const RESOLVE_POTJIEJIE: u16 = 511;
/// resolve_potqingshi(player) — ABI 554
pub const RESOLVE_POTQINGSHI: u16 = 512;
/// resolve_spyajun(player, less_def, stage) — ABI 428 (Claude, 压军)
pub const RESOLVE_SPYAJUN: u16 = 513;
/// resolve_spzundi(player, target) — ABI 429 (Claude, 尊帝)
pub const RESOLVE_SPZUNDI: u16 = 514;
/// resolve_xinyangjie(player, target) — ABI 430 (Claude, 佯解)
pub const RESOLVE_XINYANGJIE: u16 = 515;
/// resolve_qingyu(player, action, xuancun_def, mbyongjue_def) — ABI 431 (Claude, 清雨)
pub const RESOLVE_QINGYU: u16 = 516;
/// resolve_zhengsu(player, mode, stage) — ABI 432 (Claude, 整肃)
pub const RESOLVE_ZHENGSU: u16 = 517;
/// resolve_houfeng(player, target, mode, stage) — ABI 433 (Claude, 侯封/燕集)
pub const RESOLVE_HOUFENG: u16 = 518;
/// resolve_mbdiancai(player, actor) — ABI 434 (Claude, 点才)
pub const RESOLVE_MBDIANCAI: u16 = 519;
/// resolve_rechuhai(player, target, card, target_card, stage) — ABI 435 (Claude, 战海)
pub const RESOLVE_RECHUHAI: u16 = 545;
/// resolve_xiongshu(player, target, card, action, stage) — ABI 436 (Claude, 凶竖)
pub const RESOLVE_XIONGSHU: u16 = 546;
/// resolve_jianhui(player, source, target) — ABI 437 (Claude, 奸回)
pub const RESOLVE_JIANHUI: u16 = 531;
/// resolve_spqiai(player, target) — ABI 439 (Claude, 弃哀)
pub const RESOLVE_SPQIAI: u16 = 532;
/// resolve_spshanxi(player, target, stage) — ABI 440 (Claude, 擅袭)
pub const RESOLVE_SPSHANXI: u16 = 533;
/// resolve_spshanxi_suoming(player, actor, stage) — ABI 441 (Claude, 索命)
pub const RESOLVE_SPSHANXI_SUOMING: u16 = 534;
/// resolve_tongduo(player, target, card, stage) — ABI 442 (Claude, 通夺)
pub const RESOLVE_TONGDUO: u16 = 535;
/// resolve_yuanqing(player) — ABI 443 (Claude, 怨清)
pub const RESOLVE_YUANQING: u16 = 536;
/// resolve_shuchen(player, dying, renku_def) — ABI 444 (Claude, 疏陈)
pub const RESOLVE_SHUCHEN: u16 = 537;
/// resolve_duanbi(player, targets, stage) — ABI 445 (Claude, 断笔)
pub const RESOLVE_DUANBI: u16 = 538;
/// resolve_zaoli(player, mark) — ABI 446 (Claude, 躁厉)
pub const RESOLVE_ZAOLI: u16 = 539;
/// resolve_guying(player, actor, card, stage) — ABI 447 (Claude, 孤影)
pub const RESOLVE_GUYING: u16 = 540;
/// resolve_potweiyan(player, target, card, action, kuanggu_def, zhuangshi_def, kunfen_def, direct_def, limit_def, kuanggu_limit_def) — ABI 558
pub const RESOLVE_POTWEIYAN: u16 = 520;
/// resolve_qiaobian(player, phase) — ABI 559 (omp, 巧变)
pub const RESOLVE_QIAOBIAN: u16 = 521;
/// exclude_current_sha_target(card, target) — ABI 560 (omp, 享乐)
pub const EXCLUDE_SAVED_SHA_TARGET: u16 = 522;
/// resolve_xiangle(player, source, card) — ABI 560 (omp, 享乐)
pub const RESOLVE_XIANGLE: u16 = 523;
/// resolve_fangquan_skip(player) — ABI 560 (omp, 放权)
pub const RESOLVE_FANGQUAN_SKIP: u16 = 524;
/// resolve_fangquan_grant(player) — ABI 560 (omp, 放权)
pub const RESOLVE_FANGQUAN_GRANT: u16 = 525;
/// resolve_ruoyu(player, jijiang_def) — ABI 560 (omp, 若愚)
pub const RESOLVE_RUOYU: u16 = 526;
/// 势周瑜【炽沄/焰洄/焚涛/雄姿】复合结算（ABI 561）。
pub const RESOLVE_POTZHOUYU: u16 = 527;
/// modify_current_damage(player, delta) — ABI 562 (Hermes, 通用当前伤害修改)
pub const MODIFY_CURRENT_DAMAGE: u16 = 528;
/// resolve_zuilun(player) — ABI 563 (omp, 罪论)
pub const RESOLVE_ZUILUN: u16 = 549;
/// resolve_fuyin(player, source, card) — ABI 563 (omp, 父荫)
pub const RESOLVE_FUYIN: u16 = 541;
/// resolve_potlvyi(player, target, card, action) — ABI 564
pub const RESOLVE_POTLVYI: u16 = 542;
/// resolve_potcaoshuang(player, target, card, action) — ABI 568
pub const RESOLVE_POTCAOSHUANG: u16 = 544;
/// resolve_potyuji(player, target, card, action) — ABI 570
pub const RESOLVE_POTYUJI: u16 = 543;
/// ban_use_against(user, target) — ABI 571 (omp, 拒战)
pub const BAN_USE_AGAINST: u16 = 547;
/// resolve_juzhan(player, source, card) — ABI 571 (omp, 拒战)
pub const RESOLVE_JUZHAN: u16 = 548;
/// resolve_qizhi(player) — ABI 573 (omp, 奇制)
pub const RESOLVE_QIZHI: u16 = 551;
/// resolve_jinqu(player) — ABI 573 (omp, 进趋)
pub const RESOLVE_JINQU: u16 = 552;
/// resolve_jianxiang(player) — ABI 574 (omp, 荐相)
pub const RESOLVE_JIANXIANG: u16 = 553;
/// resolve_shenshi(player, source) — ABI 574 (omp, 审时)
pub const RESOLVE_SHENSHI: u16 = 554;
/// resolve_mingren(player) — ABI 576 (omp, 明任)
pub const RESOLVE_MINGREN: u16 = 555;
/// resolve_zhenliang(player, source) — ABI 576 (omp, 贞良)
pub const RESOLVE_ZHENLIANG: u16 = 556;
/// resolve_kuizhu(player) — ABI 577 (omp, 溃诛)
pub const RESOLVE_KUIZHU: u16 = 560;
/// resolve_zhizheng(player) — ABI 577 (omp, 掣政)
pub const RESOLVE_ZHIZHENG: u16 = 561;
/// resolve_lijun(player, source, card) — ABI 577 (omp, 立军)
pub const RESOLVE_LIJUN: u16 = 562;
/// resolve_liangyin(player) — ABI 579 (omp, 良姻)
pub const RESOLVE_LIANGYIN: u16 = 565;
/// resolve_kongsheng(player) — ABI 579 (omp, 箜声)
pub const RESOLVE_KONGSHENG: u16 = 566;
/// resolve_dccunsi(player, target, guixiu_def, yongjue_def, stage) — ABI 578
pub const RESOLVE_DCCUNSI: u16 = 563;
/// record_current_sha_all_targets(player, card) — ABI 580
pub const RECORD_CURRENT_SHA_ALL_TARGETS: u16 = 567;
/// choose_give_or_discard_hand(player, payment_count, scope) — ABI 581
pub const CHOOSE_GIVE_OR_DISCARD_HAND: u16 = 568;
/// choose_discard_hand_equip_then_recover_target(chooser, target, count, amount) — ABI 582
pub const CHOOSE_DISCARD_HAND_EQUIP_THEN_RECOVER_TARGET: u16 = 569;
/// resolve_dcjue(player, event_player) — ABI 583
pub const RESOLVE_DCJUE: u16 = 570;
/// resolve_ruanji(player, source, card, action) — ABI 584
pub const RESOLVE_RUANJI: u16 = 571;
/// resolve_zhengbi(player, target, action) — ABI 585
pub const RESOLVE_ZHENGBI: u16 = 572;
/// resolve_fengying(player, helper_def, action) — ABI 585
pub const RESOLVE_FENGYING: u16 = 573;
/// resolve_qiaojian(player, target, helper_def, action) — ABI 586
pub const RESOLVE_QIAOJIAN: u16 = 574;
/// resolve_xicha(player, source, card, action) — ABI 586
pub const RESOLVE_XICHA: u16 = 575;
/// resolve_huzun(player, source, card, action) — ABI 587
pub const RESOLVE_HUZUN: u16 = 576;
/// resolve_zhugemengxue(player, action) — ABI 588
pub const RESOLVE_ZHUGEMENGXUE: u16 = 577;
/// draw_cards_from_bottom(player, count) — ABI 589.
pub const DRAW_CARDS_FROM_BOTTOM: u16 = 578;
/// 寸目（许攸）：当前摸牌阶段从牌堆底摸牌。
pub const SET_DRAW_PHASE_FROM_BOTTOM: u16 = 581;
/// resolve_xiongluan(player, stage) — ABI 590.
pub const RESOLVE_XIONGLUAN: u16 = 582;
/// resolve_twjuxiang(player, event_player, stage) — ABI 590.
pub const RESOLVE_TWJUXIANG: u16 = 583;
/// resolve_gzbuqu(player, delta, stage) — ABI 591.
pub const RESOLVE_GZBUQU: u16 = 584;
/// resolve_gzbuqu_recover(player, stage) — ABI 591.
pub const RESOLVE_GZBUQU_RECOVER: u16 = 585;
/// resolve_sbbiyue(player) — ABI 599.
pub const RESOLVE_SBBIYUE: u16 = 588;
/// resolve_sxrmqiewang(player, event_player) — ABI 596.
pub const RESOLVE_SXRMQIEWANG: u16 = 586;
/// resolve_sxrmqianliu(player, stage) — ABI 596.
pub const RESOLVE_SXRMQIANLIU: u16 = 587;
/// resolve_chenglve(player, stage, suits) — ABI 589.
pub const RESOLVE_CHENGLVE: u16 = 579;
/// resolve_shicai(player, card, stage) — ABI 589.
pub const RESOLVE_SHICAI: u16 = 580;
/// resolve_bailingyun(player, action) — ABI 593.
pub const RESOLVE_BAILINGYUN: u16 = 593;
/// resolve_dcshaowei(player, action) — ABI 594.
pub const RESOLVE_DCSHAOWEI: u16 = 594;
/// resolve_dcdichou(player, action, helper_skill_def) — ABI 594.
pub const RESOLVE_DCDICHOU: u16 = 592;
/// resolve_dcdufeng(player, action, helper_skill_def) — ABI 595.
pub const RESOLVE_DCDUFENG: u16 = 595;
/// choose_discard_hand_range(player, min, max) — ABI 596.
pub const CHOOSE_DISCARD_HAND_RANGE: u16 = 596;
/// resolve_dcctjiuxian(player, event_player, action) — ABI 597.
pub const RESOLVE_DCCTJIUXIAN: u16 = 597;
/// resolve_bianyue(player, target, action) — ABI 598.
pub const RESOLVE_BIANYUE: u16 = 598;
/// resolve_zhupeilan(player, target, card, helper_def, action) — ABI 600.
pub const RESOLVE_ZHUPEILAN: u16 = 599;
/// resolve_dc_sb_zhangxiu(player, target, action) — ABI 601.
pub const RESOLVE_DC_SB_ZHANGXIU: u16 = 600;
/// resolve_dc_sb_zhouyu(player, source, card, action) — ABI 602.
pub const RESOLVE_DC_SB_ZHOUYU: u16 = 601;
/// resolve_dc_sb_lusu(player, source, card, action) — ABI 603.
pub const RESOLVE_DC_SB_LUSU: u16 = 602;
/// resolve_zhangjian(player, card, grant_skill_def, action) — ABI 604.
pub const RESOLVE_ZHANGJIAN: u16 = 603;
/// resolve_zhugeruoxue(player, target, card, action) — ABI 605.
pub const RESOLVE_ZHUGERUOXUE: u16 = 604;
/// resolve_caoyi(player, card, helper_skill_def, action) — ABI 606.
pub const RESOLVE_CAOYI: u16 = 605;
/// resolve_malingli(player, card, helper_skill_def, action) — ABI 607.
pub const RESOLVE_MALINGLI: u16 = 606;
/// resolve_nsshuaiyan(player, recovered_player) — ABI 608.
pub const RESOLVE_NSSHUAIYAN: u16 = 607;
/// resolve_wdb_hanlong(player, target, card, helper_skill_def, action) — ABI 609.
pub const RESOLVE_WDB_HANLONG: u16 = 608;
/// resolve_wdb_liufu(player, target, card, zhucheng_skill_def, action) — ABI 610.
pub const RESOLVE_WDB_LIUFU: u16 = 609;
/// resolve_wdb_liuyan(player, card, action) — ABI 611.
pub const RESOLVE_WDB_LIUYAN: u16 = 610;
/// resolve_wdb_liuzan(player, target, helper_skill_def, action) — ABI 612.
pub const RESOLVE_WDB_LIUZAN: u16 = 611;
/// resolve_wdb_tianyu(player, target, card, action) — ABI 613.
pub const RESOLVE_WDB_TIANYU: u16 = 612;
/// resolve_wdb_xizhenxihong(player, target, card, action) — ABI 614.
pub const RESOLVE_WDB_XIZHENXIHONG: u16 = 613;
/// resolve_wdb_yangyi(player, target, card, action) — ABI 615.
pub const RESOLVE_WDB_YANGYI: u16 = 614;
/// resolve_wdb_zaozhirenjun(player, target, card, action) — ABI 616.
pub const RESOLVE_WDB_ZAOZHIRENJUN: u16 = 615;
/// resolve_dddxianxi(player, card, action) — ABI 617.
pub const RESOLVE_DDD_XIANXI: u16 = 616;
/// resolve_dddxiaheng(player, action) — ABI 618.
pub const RESOLVE_DDD_XIAHENG: u16 = 617;
/// resolve_ddd_liuchong(player, actor, card, action) — ABI 619.
pub const RESOLVE_DDD_LIUCHONG: u16 = 618;
/// resolve_ddd_duanbing(player, target, card, action) — ABI 621.
pub const RESOLVE_DDD_DUANBING: u16 = 619;
/// resolve_ddd_wuzhi(player, card, action) — ABI 622.
pub const RESOLVE_DDD_WUZHI: u16 = 620;
/// resolve_ddd_xujing(player, source, card, action) — ABI 623.
pub const RESOLVE_DDD_XUJING: u16 = 621;
/// resolve_ddd_caomao(player, action) — ABI 624.
pub const RESOLVE_DDD_CAOMAO: u16 = 622;
/// resolve_ddd_xinxianying(player, actor, action) — ABI 625.
pub const RESOLVE_DDD_XINXIANYING: u16 = 623;
/// resolve_ddd_xianglang(player, actor, action) — ABI 626.
pub const RESOLVE_DDD_XIANGLANG: u16 = 624;
/// resolve_ddd_yujin(player, actor, action) — ABI 627.
pub const RESOLVE_DDD_YUJIN: u16 = 625;
/// resolve_ddd_liuye(player, actor, card, action) — ABI 628.
pub const RESOLVE_DDD_LIUYE: u16 = 626;
/// resolve_ddd_baosanniang(player, action) — ABI 629.
pub const RESOLVE_DDD_BAOSANNIANG: u16 = 627;
/// resolve_ddd_zhenji(player, action) — ABI 630.
pub const RESOLVE_DDD_ZHENJI: u16 = 628;
/// resolve_ddd_zhaoang(player, action) — ABI 631.
pub const RESOLVE_DDD_ZHAOANG: u16 = 629;
/// resolve_ddd_liuba(player, actor, action) — ABI 632.
pub const RESOLVE_DDD_LIUBA: u16 = 630;
/// resolve_ddd_jianshuo(player, target, action) — ABI 633.
pub const RESOLVE_DDD_JIANSHUO: u16 = 631;
/// resolve_ddd_guanning(player, actor, action) — ABI 634.
pub const RESOLVE_DDD_GUANNING: u16 = 632;
/// resolve_ddd_caoshuang(player, actor, card, action) — ABI 635.
pub const RESOLVE_DDD_CAOSHUANG: u16 = 633;
/// resolve_ddd_xuelingyun(player, actor, card, action) — ABI 636.
pub const RESOLVE_DDD_XUELINGYUN: u16 = 634;
/// resolve_ddd_liuhong(player, actor, action) — ABI 637.
pub const RESOLVE_DDD_LIUHONG: u16 = 635;
/// resolve_ddd_xiahouxuan(player, action) — ABI 638.
pub const RESOLVE_DDD_XIAHOUXUAN: u16 = 636;
/// resolve_ddd_zhangkai(player, actor, card, action) — ABI 639.
pub const RESOLVE_DDD_ZHANGKAI: u16 = 637;
/// resolve_ddd_liangxi(player, actor, card, action) — ABI 640.
pub const RESOLVE_DDD_LIANGXI: u16 = 638;
/// guozhan_use_zhulian_bihe(player) — ABI 641.
pub const GUOZHAN_USE_ZHULIAN_BIHE: u16 = 639;
/// resolve_shoulijian(player, target, skill) — ABI 642.
pub const RESOLVE_SHOULIJIAN: u16 = 640;
/// cancel_incoming_card_use(player, source, card, skill) — ABI 643.
pub const CANCEL_INCOMING_CARD_USE: u16 = 641;
/// choose_other_gain_one_of_two_skills(player, target, first_def, second_def) — ABI 644.
pub const CHOOSE_OTHER_GAIN_ONE_OF_TWO_SKILLS: u16 = 642;
/// gwyewu_random_discard_chain(player, wuyao_def, lang_def) — ABI 645.
pub const GWYEWU_RANDOM_DISCARD_CHAIN: u16 = 643;
/// refresh_skill(skill) — reset one skill instance's per-turn/per-phase counters.
pub const REFRESH_SKILL: u16 = 644;
/// tag_card(cards, tag) — attach a content tag to each card (gaintag).
pub const TAG_CARD: u16 = 645;
/// untag_card(cards, tag) — remove a content tag from each card.
pub const UNTAG_CARD: u16 = 646;
/// clanshenjun_reveal(player, event_card, skill) — reveal + gaintag same-kind hand cards.
pub const CLANSHENJUN_REVEAL: u16 = 647;
/// clanshenjun_use(player, skill) — phase-end tagged-card use.
pub const CLANSHENJUN_USE: u16 = 648;
/// storage_set(skill, key, value) — write to keyed content storage.
pub const STORAGE_SET: u16 = 649;
/// draw_all_event_targets(count) — draw `count` cards to each event target + the event player.
pub const DRAW_ALL_EVENT_TARGETS: u16 = 650;
/// clanyuzhi_reveal(player, skill) — round-start hand-card reveal + gaintagged draw.
pub const CLANYUZHI_REVEAL: u16 = 651;
/// clanyuzhi_round_end(player, skill) — round-end tagged discard + ambition check.
pub const CLANYUZHI_ROUND_END: u16 = 652;
/// qieyi_phase_start(player, skill) — play-phase suit-key reset + top-2 private view.
pub const QIEYI_PHASE_START: u16 = 653;
/// qieyi_use_after(player, event_card, skill) — per-suit first use reveal + gain/put-top.
pub const QIEYI_USE_AFTER: u16 = 654;
/// jianzhi_phase_end(player, skill) — end-phase judgment count choice + assign/damage.
pub const JIANZHI_PHASE_END: u16 = 655;
/// add_first_missing_skill(player, first, second, third) — ABI 575
pub const ADD_FIRST_MISSING_SKILL: u16 = 557;
/// resolve_rehengjiang(player, actor, action, hand_limit_def, draw_def) — ABI 572
pub const RESOLVE_REHENGJIANG: u16 = 550;
