//! LSP 悬停文档链：持有者条件（owner_*）说明。

pub(in super::super) fn documentation(word: &str) -> Option<&'static str> {
    Some(match word {
            "owner_debate_participant" => {
                "Trigger filter: the owner is among the participants of a debate that just opened (ADR 0644 P3, debateShowOpinion window)."
            }
            "owner_cards_moved_contain_ying" => {
                "Trigger filter: cards moved into the discard pile and at least one is a shadow card (jsrg funi③)."
            }
            "owner_is_lord" => {
                "Trigger filter: the skill owner has the Lord identity (identity mode)."
            }
            "owner_alive_play_end_used_categories_positive" => {
                "Play-phase-end trigger filter: the living owner's own play phase ends with at least one card used. Implements 连计 (offline zylianji)."
            }
            "owner_alive_other_targeted_by_sha" => {
                "Card-target trigger filter: a living owner observes a logical Sha targeting another player (daimou)."
            }
            "owner_alive_targeted_by_other_sha_judge_xumou" => {
                "Card-target trigger filter: a living owner is targeted by another player's Sha and their judge zone holds at least one xumou card (daimou)."
            }
            "owner_alive_event_player_lord" => {
                "Phase-end trigger filter: the living owner observes the ending turn of the Lord player (jsrgzhenglve)."
            }
            "owner_alive_event_player_lord_no_damage" => {
                "Phase-end trigger filter: the living owner observes the Lord's ending turn and the Lord dealt no damage this turn (jsrgzhenglve two-grant branch)."
            }
            "owner_alive_event_player_lord_damaged" => {
                "Phase-end trigger filter: the living owner observes the Lord's ending turn and the Lord dealt damage this turn (jsrgzhenglve single-grant branch)."
            }
            "owner_alive_turn_used_all_suits" => {
                "Turn-end trigger filter: the living owner's own turn ends with all four suits used during the turn. Implements 博图 (refresh botu)."
            }
            "owner_alive_other_source_damage" => {
                "伤害事件且来源为非拥有者的其他角色（旁观者视角）时触发。"
            }
            "owner_alive_event_player_self_or_clan_<clan>" => {
                "保族 (clanbaozu) 家族过滤器：owner 存活、事件角色存活，且事件角色是 owner 本人或 <clan>（CJK 族名，如 颍川钟氏）成员。配合 dying 事件使用；上游 hp<=0 由濒死事件本身保证，未横置门槛由 DSL body 的 mark 守卫表达。"
            }
            "owner_hand_compare_finished_participant" => {
                "The living skill owner is either participant (initiator or target) in a completed hand comparison (hand_compare_finished)."
            }
            "owner_hand_compare_participant_has_sha" => {
                "Completed hand-comparison trigger filter: the living owner participated and at least one compared physical card is an effective Sha in the discard pile."
            }
            "owner_hand_compare_participant_result_card_in_discard" => {
                "Completed hand-comparison trigger filter: the living owner participated and the exact result card selected by legacy zongshi semantics remains in the discard pile. Requires Rule VM ABI 66."
            }
            "owner_source_sha_damage_target_alive_has_discardable_equip" => {
                "Damage trigger filter: the living owner caused logical Sha damage to a living target that currently has equipment the owner may discard. Requires Rule VM ABI 282."
            }
            "owner_source_equip_used_skill_mark_below_seven" => {
                "Card-used trigger filter: the living owner used a logical equipment card and this skill definition's stable player mark is below seven. Requires Rule VM ABI 283."
            }
            "owner_source_outside_turn_card_used_or_responded" => {
                "Card-used/responded filter: the living owner is the event source and used or responded with a logical card outside their own turn. Requires Rule VM ABI 284."
            }
            "owner_is_target_any_alive_hand_nonempty" => {
                "Prepare-phase trigger filter: the living owner is the event player and at least one living player, including the owner, has a nonempty hand. Requires Rule VM ABI 62."
            }
            "owner_is_target_any_alive_hand_empty" => {
                "End-phase trigger filter: the living owner is the event player and at least one living player, including the owner, has no hand cards. Requires Rule VM ABI 162."
            }
            "owner_is_target_draw_reshuffle_zero_wounded" => {
                "Prepare-phase trigger filter: the living owner is the event player, is wounded, and the discard pile has never been reshuffled into the draw pile. Requires Rule VM ABI 163."
            }
            "owner_is_target_draw_reshuffle_positive" => {
                "Prepare-phase trigger filter: the living owner is the event player after at least one discard-pile reshuffle into the draw pile. Requires Rule VM ABI 163."
            }
            "owner_discarded_black_basic_to_discard" => {
                "Cards-moved trigger filter: the owner discarded at least one effective black basic card from hand/equipment into the discard pile. The full moved-card batch is preserved for the effect. Requires Rule VM ABI 51."
            }
            "owner_alive_other_judge_phase_in_range_has_judge_card" => {
                "Judgment-phase trigger filter: another living player is within the living owner's attack range and has a discardable judgment card. Requires Rule VM ABI 51."
            }
            "owner_source_single_target_sha_or_ordinary_trick_current_turn" => {
                "Card-used trigger filter: during the owner's turn, the owner used a logical Sha or ordinary trick with exactly one declared target. Requires Rule VM ABI 49."
            }
            "owner_source_event_card_policy_draw" => {
                "Damage trigger filter: the owner caused this damage with the exact card carrying the draw-on-damage bit in the current skill instance. Requires Rule VM ABI 49."
            }
            "owner_source_card_resolved_phase_policy" => {
                "Card-resolved filter: the owner finished the exact physical card armed in this skill instance's phase policy. Requires Rule VM ABI 519."
            }
            "owner_phase_play_end_potguansha_available_once_per_game" => {
                "Play-phase-end filter: the owner has hand/equipment cards and has not used 灌沙 this match. Requires Rule VM ABI 520."
            }
            "owner_discarded_sha_to_discard" => {
                "Cards-moved filter: the owner discarded at least one physical Sha that remains in the shared discard pile. Requires Rule VM ABI 522."
            }
            "owner_alive_other_turn_discarded_recorded_potfuan_suit" => {
                "End-phase filter: another player's turn ended after a suit recorded by 伏暗 entered the discard pile this turn. Requires Rule VM ABI 525."
            }
            "owner_source_sha_resolved_potfuan_dealt_damage" => {
                "Sha-resolved filter: the owner's first pending 伏暗 virtual Sha dealt damage. Requires Rule VM ABI 525."
            }
            "owner_source_first_target_hand_damage_card_discardable" => {
                "Card-targeted filter for 鸱张: the owner used a hand damage card, this is its first target, and the owner can discard a hand card. Requires Rule VM ABI 505."
            }
            "owner_lost_hand_ownerless_sha_not_use_once_per_turn" => {
                "Cards-moved filter for 断鞅: the owner lost an ownerless physical Sha from hand outside use/response, once per turn. Requires Rule VM ABI 505."
            }
            "owner_alive_phase_any_end_originating_skill_pile_nonempty" => {
                "Phase-any-end filter: the living owner's pile for the originating skill is nonempty. Requires Rule VM ABI 505."
            }
            "owner_source_damage_tracked_mbduanyang_target_recastable" => {
                "Damage filter for 断鞅: its tracked Sha dealt damage and the target has a hand/equipment/judgment card. Requires Rule VM ABI 505."
            }
            "owner_mbrunwei_loss_threshold" => {
                "Cards-moved filter for an armed 润微 loss threshold, excluding pure equipment-card use. Requires Rule VM ABI 506."
            }
            "owner_phase_play_end_mbrunwei_tracked_hand" => {
                "Play-phase-end filter for tracked 润微 self-gains still in hand. Requires Rule VM ABI 506."
            }
            "owner_alive_damage_before_target_distance_le_one_once_per_turn" => {
                "Damage-before filter: the target is within final distance one of the living owner, once per turn. Requires Rule VM ABI 506."
            }
            "owner_source_damage_mbxiongtu_armed" => {
                "Damage filter for an armed 凶图 reward, excluding damage caused by 凶图 itself. Requires Rule VM ABI 507."
            }
            "owner_phase_play_end_mbxiongtu_state" => {
                "Play-phase-end filter for clearing 凶图's transient activation state. Requires Rule VM ABI 507."
            }
            "owner_phase_end_mbganggeng_target_alive" => {
                "Phase-end filter for resolving the living target remembered by 刚鲠. Requires Rule VM ABI 508."
            }
            "owner_from_hand_empty_or_dying_mbsijian_twice_per_turn" => {
                "Cards-moved or dying filter for 死谏, shared across both event sources and limited to twice per turn. Requires Rule VM ABI 508."
            }
            "owner_is_target_can_move_field_card_mbyuanmo_twice_per_turn" => {
                "Prepare or post-damage filter for 远谟 when any legal field-card move exists, shared limit twice per turn. Requires Rule VM ABI 509."
            }
            "owner_source_first_single_target_sha_mbfeijing_twice_per_turn" => {
                "Card-targeted filter for the owner's first and sole Sha target, limited to twice per turn. Requires Rule VM ABI 510."
            }
            "owner_source_damage_before_mbfeijing_extra" => {
                "Damage-before filter for a target added to this exact Sha by 飞径. Requires Rule VM ABI 510."
            }
            "owner_source_sha_resolved_original_single_mbxiaoge" => {
                "Card-resolved filter for the owner's originally single-target Sha, including targets later added by 飞径. Requires Rule VM ABI 510."
            }
            "owner_alive_other_dying_mbchenshe_available" => {
                "Dying filter: another living player is dying and at least one of the owner, dying player, or living damage source has a card the owner can discard. Requires Rule VM ABI 503."
            }
            "owner_alive_other_phase_end_mobiledanshou" => {
                "Phase-end filter: the owner is alive, the phase player is another player, and Mobile Danshou can either draw for zero targetings or pay the exact current-turn targeting count. Requires Rule VM ABI 500."
            }
            "owner_alive_other_phase_end_not_damaged_owner_this_turn" => {
                "Phase-end filter: the owner is alive, the phase player is another player, and that player dealt no damage to the owner this turn."
            }
            "owner_alive_other_phase_end_used_card_on_owner_undamaged_this_turn" => {
                "Phase-end filter: the owner is alive, the phase player is another player who used at least one card targeting the owner this turn, and the owner took no damage this turn."
            }
            "owner_target_physical_sha_source_hp_gt_owner" => {
                "Card-targeted filter: the living owner is targeted by another living player's physical Sha and that source currently has strictly more HP. Requires Rule VM ABI 370."
            }
            "owner_source_other_target_sha_or_ordinary_trick_twguoyi_condition" => {
                "Card-targeted filter: the owner used a Sha or ordinary trick on another living player, that target ties for maximum HP/hand or the owner's hand is at most lost HP plus one, and the card was not replayed by the same skill. Requires Rule VM ABI 371."
            }
            "owner_alive_phase_end_caused_two_distinct_dying" => {
                "Phase-end filter: the living owner caused at least two distinct other players to enter dying during the current turn. Requires Rule VM ABI 372."
            }
            "owner_gained_event_cards_from_other_personal_zone" => {
                "Cards-moved filter: the owner gained at least one event card from another player's hand, equipment, or judge zone and at least one such card remains in hand. Requires Rule VM ABI 372."
            }
            "owner_alive_other_single_target_used_tracked_twyizhu" => {
                "Card-targeted filter: another player used a tracked TW Yizhu physical card with exactly one declared target. Requires Rule VM ABI 373."
            }
            "owner_tracked_twyizhu_entered_discard" => {
                "Cards-moved filter: at least one physical card tracked by TW Yizhu entered the discard pile. Requires Rule VM ABI 373."
            }
            "owner_alive_other_twgonghuan_damage_before_hp_lte_owner_unused" => {
                "Damage-before filter: another living holder of the same Gonghuan helper has HP no greater than the owner, the owner has not redirected damage this turn, and the segment was not itself redirected. Requires Rule VM ABI 374."
            }
            "owner_phase_begin_playable_skill_pile_nonempty" => {
                "Phase-begin filter: the owner has at least one nonempty skill pile exposed at hand-card use/response boundaries. Requires Rule VM ABI 375."
            }
            "owner_used_playable_skill_pile_card_unused" => {
                "Card-used/responded filter: the owner consumed a physical card from a playable skill pile and this trigger has not fired this turn. Requires Rule VM ABI 375."
            }
            "owner_target_damage_before_playable_skill_piles_empty" => {
                "Damage-before filter: the owner is the target and all playable skill piles they own are empty. Requires Rule VM ABI 375."
            }
            "owner_alive_damage_before_target_min_hp_unused" => {
                "Damage-before filter: a living player tied for minimum HP is the target and this skill has not fired this turn. Requires Rule VM ABI 376."
            }
            "owner_alive_global_phase_end" => {
                "Phase-end filter: the owner is alive at the beginning of any player's End phase. Requires Rule VM ABI 376."
            }
            "owner_alive_global_phase_end_lost_cards_target_other_alive" => {
                "Phase-end filter: the owner is alive, has lost at least one physical card this turn, and another living player's turn is ending. Requires Rule VM ABI 496."
            }
            "owner_alive_global_phase_end_discarded_cards_available" => {
                "Phase-end filter: the owner and phase player are alive, at least one card discarded this turn remains in the discard pile, and the owner has a fixed or selectable other target. Requires Rule VM ABI 583."
            }
            "owner_alive_phase_end_used_card_no_damage" => {
                "Owner End-phase filter: at least one card was used and the owner dealt no damage this turn. Requires Rule VM ABI 377."
            }
            "owner_cards_entered_discard_not_use_unused" => {
                "Cards-moved filter: the owner's personal cards entered discard for a cause other than use/response and the skill has not fired this turn. Requires Rule VM ABI 377."
            }
            "owner_alive_other_source_sha_resolved_not_target_gainable_once_per_turn" => {
                "Sha-resolved trigger filter: once per turn, a living non-target owner observes another player's fully resolved Sha while its physical carrier remains in discard. Requires Rule VM ABI 151."
            }
            "owner_alive_phase_play_end_zhengbi_pending" => {
                "Zhengbi Play-phase-end filter: the living owner has a pending target; the resolver consumes it and checks that phase's gain history. Requires Rule VM ABI 585."
            }
            "owner_is_target_source_hand_nonempty_per_damage_point" => {
                "Xicha damage filter: owner is the damaged player and the source currently has hand cards; repeats once per damage point. Requires Rule VM ABI 586."
            }
            "owner_source_qiaojian_extra_available" => {
                "Qiaojian helper filter: owner used a Sha or ordinary trick whose category has not yet been consumed this turn. Requires Rule VM ABI 586."
            }
            "owner_xicha_watched_source_card_used" => {
                "Xicha watch filter: a player recorded in this skill's private state used their next card. Requires Rule VM ABI 586."
            }
            "owner_qiaojian_tracked_card_left_hand" => {
                "Qiaojian tag cleanup filter: a tracked exempt card left the owner's hand. Requires Rule VM ABI 586."
            }
            "owner_alive_damage_target_self_or_in_range_other_source_numeric_card" => {
                "Zhantao damage filter: owner or a character in range took numeric card damage from another source. Requires Rule VM ABI 587."
            }
            "owner_source_wounded_exists_once_per_turn" => {
                "Anjing damage filter: owner dealt damage, a wounded living target exists, and the skill is unused this turn. Requires Rule VM ABI 587."
            }
            "owner_hand_transfer_other_dcsbmengmou_branch_available" => {
                "Mengmou transfer filter: the owner and another living character exchanged at least one hand card, and the current conversion branch remains unused this turn. Requires Rule VM ABI 603."
            }
            "owner_is_target_card_damage_before" => {
                "Card-damage-before filter: the living owner is the pending damage target and the damage carries a physical or logical card context. Requires Rule VM ABI 604."
            }
            "owner_source_basic" => {
                "Trigger filter: the owner used a logical basic card. Requires Rule VM ABI 99."
            }
            "owner_source_non_basic" => {
                "Trigger filter: the owner used a logical non-basic card. Requires Rule VM ABI 99."
            }
            "owner_source_wounded_nanman_wanjian_or_normal_spade_sha" => {
                "Card-use trigger filter: the wounded owner used logical Nanman, Wanjian, or an ordinary-nature spade Sha. Requires Rule VM ABI 104."
            }
            "owner_alive_other_play_phase_duplicate_card_category_has_hand_equip" => {
                "Play-phase-end trigger filter: the living owner is not the ending player, can discard from hand/equipment, and that player used at least two logical cards of the same basic/trick/equipment category in the exact Play phase. Requires Rule VM ABI 105."
            }
            "owner_alive_other_phase_end_damage_gt_one_has_hand_equip" => {
                "End-phase trigger filter: the living owner is not the phase player, can discard from hand/equipment, and that player dealt more than one total damage during the current turn. Requires Rule VM ABI 105."
            }
            "owner_alive_play_phase_sha_user_has_discardable_hand_equip" => {
                "Trigger filter: the living owner can discard a hand/equipment card and a living player is currently using a logical Sha in that player's play phase. Requires Rule VM ABI 43."
            }
            "owner_is_target_play_phase_card_category_count_positive" => {
                "Trigger filter: the owner is ending their play phase after using at least one logical basic, trick, or equipment card in that phase. Requires Rule VM ABI 42."
            }
            "owner_alive_play_phase_card_category_count_positive" => {
                "Trigger filter: a living owner used at least one logical basic, trick, or equipment card in the exact Play phase that is ending. Requires Rule VM ABI 293."
            }
            "owner_is_target_sha_source_has_discardable_hand_equip" => {
                "Trigger filter: the owner is the logical-Sha damage target, the damage has an identified source, and the owner can currently discard at least one hand/equipment card."
            }
            "owner_alive_current_turn_other_event_player" => {
                "Trigger filter: the living skill owner is the current turn player and differs from the event player."
            }
            "owner_alive_current_turn_other_event_player_from_hand_empty" => {
                "Trigger filter: the living current-turn skill owner observes another living player lose hand cards and become empty-handed."
            }
            "owner_is_target_current_turn" => {
                "Trigger filter: the living skill owner is both the event player and the current turn player. Requires Rule VM ABI 351."
            }
            "owner_gains_multiple_outside_own_draw" => {
                "Trigger filter: the living skill owner gains more than one card in one draw or move event outside their own draw phase."
            }
            "owner_gained_or_lost_at_least_two" => {
                "Trigger filter: the living skill owner gains or loses at least two cards in one draw or move event."
            }
            "owner_gains_outside_turn_mark_below_alive_once_2058376721" => {
                "Parameterized trigger filter: once per turn, the living owner gains cards outside their turn while the mark named by the numeric suffix is below the living-player count."
            }
            "owner_card_responded_to_by_player_in_range" => {
                "Trigger filter: a living player whose attack range includes the skill owner responded to a card used by that owner."
            }
            "owner_alive_first_cards_moved_to_hand" => {
                "Trigger filter: the living skill owner observes the first non-draw card move into any player's hand this turn."
            }
            "owner_draws_during_own_play_not_self" => {
                "Trigger filter: the living owner draws during their own play phase, excluding draws emitted by this skill instance."
            }
            "owner_source_non_equip_no_basic_hand_has_targets" => {
                "Trigger filter: the living owner used a non-equipment card with at least one target while holding no basic hand cards."
            }
            "owner_equip_zone_changed_alive_wounded_at_least_two" => {
                "Trigger filter: an equipment card entered or left the living owner's equipment zone while at least two living players were wounded."
            }
            "owner_alive_once_per_round" => {
                "Trigger filter: the skill owner is alive and this skill instance has not fired during the current round."
            }
            "owner_is_event_player_no_enabled_equip_slot_once_per_game" => {
                "Lifetime trigger filter: the living skill owner is the event player, has no enabled equipment slot, and this skill instance has never fired before. Requires Rule VM ABI 19."
            }
            "owner_alive_other_dying_hand_nonempty_once_per_game" => {
                "Lifetime dying trigger filter: the living owner observes another living player at non-positive HP, has at least one hand card, and this skill instance has never fired before. Requires Rule VM ABI 125."
            }
            "owner_alive_sha_or_duel_damage_source_target_alive_hand_equip_nonempty" => {
                "Damage trigger filter: the living owner observes damage from a logical Sha or Duel; the living source and target must collectively have at least one hand/equipment card. Requires Rule VM ABI 126."
            }
            "owner_alive_sha_damage_source_target_alive_owner_hand_equip_nonempty" => {
                "Damage trigger filter: the living owner observes sourced logical-Sha damage to a living player and can discard one of their own hand/equipment cards. The source may already be dead when damage-end triggers settle. Requires Rule VM ABI 219."
            }
            "owner_is_target_any_alive_self_discardable_hand_equip" => {
                "End-phase trigger filter: the living owner is the phase player and at least one living player can discard one of their own hand/equipment cards. Requires Rule VM ABI 127."
            }
            "owner_is_target_any_alive_discardable_by_owner_hand_equip" => {
                "Prepare/End-phase trigger filter: the living owner is the phase player and may discard at least one hand/equipment card from some living player, including themselves. Requires Rule VM ABI 298."
            }
            "owner_is_target_any_alive_self_discardable_hand" => {
                "Phase trigger filter: the living owner is the phase player and at least one living player can discard one of their own hand cards. Requires Rule VM ABI 194."
            }
            "owner_is_target_any_alive_wounded" => {
                "Phase trigger filter: the living owner is the phase player and at least one living player is below maximum HP. Requires Rule VM ABI 195."
            }
            "owner_current_turn_other_equipment_lost" => {
                "Trigger filter: during the living skill owner's turn, another player loses one or more cards from their equipment zone."
            }
            "owner_alive_other_discarded_basic_outside_turn" => {
                "Trigger filter: outside the living owner's turn, another player discards at least one basic card from hand or equipment."
            }
            "owner_alive_other_discarded_basic_outside_turn_once" => {
                "Trigger filter: once during each character's turn, outside the living owner's turn, another player discards at least one basic card from hand or equipment."
            }
            "owner_alive_other_discarded_black_to_discard" => {
                "Cards-moved trigger filter: another player discarded at least one raw black physical card into the discard pile. Only matching black cards are exposed through `event_cards`; requires Rule VM ABI 112."
            }
            "owner_from_equip_per_card" => {
                "Cards-moved trigger filter: the skill owner lost one or more equipment cards; offers the trigger independently once per moved equipment card. Requires Rule VM ABI 175."
            }
            "owner_outside_turn_red" => {
                "View-as filter: the source card is red and the skill owner is not the current turn player. Requires Rule VM ABI 176."
            }
            "owner_alive_dealt_or_received_damage_this_turn" => {
                "Turn-end trigger filter: the living owner dealt or received damage during the turn that just ended."
            }
            "owner_received_gift_from_other" => {
                "Cards-moved trigger filter: the living skill owner received cards voluntarily given by another player."
            }
            "owner_initiated_duel_target_alive_loser_has_hand_or_equip" => {
                "Duel-resolved trigger filter: the living owner initiated the Duel, its original target survived, and the loser can give a hand/equipment card to the winner."
            }
            "owner_is_target" => {
                "Trigger filter: fires only when the owner is the target of the event."
            }
            "owner_alive_is_target_current_play_phase" => {
                "Trigger filter: the living owner is the event player, the owner is the current turn player, and the current phase is Play. Requires Rule VM ABI 106."
            }
            "owner_source" => "Trigger filter: fires only when the owner is the event source.",
            "owner_is_target_not_unique_max_current_hp" => {
                "Trigger filter: fires only when the owner is the event player and does not have strictly more current HP than every other living player."
            }
            "owner_is_target_once" => {
                "Trigger filter: fires only when the owner is the event player and this skill has not fired during the current turn."
            }
            "owner_source_target_alive_has_equip" => {
                "Damage trigger filter: the living owner is the source and the living damaged player has at least one equipment card."
            }
            "owner_source_black_sha_target_alive_has_discardable_equip" => {
                "Damage trigger filter: the living owner caused damage with a physical black logical Sha, and the living damaged player has at least one equipment card the owner may discard. Requires Rule VM ABI 33."
            }
            "owner_source_sha_target_alive_has_discardable_region_card" => {
                "Damage trigger filter: the living owner caused damage with a logical Sha, and the living damaged player has at least one discardable card in hand, equipment, or judgment zones. Requires Rule VM ABI 34."
            }
            "owner_source_sha_target_alive_has_discardable_hand_equip" => {
                "Card-target trigger filter: the living owner used a logical Sha, and the current living target has at least one hand/equipment card the owner may discard. Requires Rule VM ABI 94."
            }
            "owner_source_tiesuo_target_alive_unlinked_hand_equip_nonempty" => {
                "Card-target trigger filter: the living owner used a logical Tiesuo, and the current living target is unlinked with a nonempty hand/equipment region. Requires Rule VM ABI 305."
            }
            "owner_alive_source_target_was_linked" => {
                "Damage trigger filter: the living owner was the source and the target was linked immediately before that damage segment committed. Requires Rule VM ABI 306."
            }
            "owner_source_sha_target_alive_hand_nonempty" => {
                "Card-target trigger filter: the living owner used a logical Sha, and the current living target has at least one hand card. Requires Rule VM ABI 154."
            }
            "owner_source_sha_target_hand_gte_hp_or_lte_attack_range" => {
                "Card-target trigger filter: the living owner used a logical Sha, and the current living target's hand count is at least the owner's HP or at most the owner's current attack range. Requires Rule VM ABI 35."
            }
            "owner_source_sha_target_hand_lte_owner_or_hp_gte_owner" => {
                "Card-target trigger filter: the living owner used a logical Sha, and the current living target has no more hand cards than the owner or at least as much HP as the owner. Requires Rule VM ABI 223."
            }
            "owner_source_direct_sha_damage_target_armor_or_mount_card_gainable" => {
                "Damage trigger filter: the living owner dealt direct logical Sha damage to a living target with armor or a mount, and the physical Sha remains in processing or discard. Requires Rule VM ABI 318."
            }
            "owner_is_source_or_target_xinyaoming_available" => {
                "Damage trigger filter: the living owner is the damage source or target and at least one legal target/action pair remains for 界邀名 this turn. Requires Rule VM ABI 319."
            }
            "owner_alive_event_player_lost_armor_or_mount_has_originating_skill_pile" => {
                "Cards-moved trigger filter: a living event player lost armor or a mount from equipment while holding cards in the public pile keyed by the observer's originating skill. Requires Rule VM ABI 318."
            }
            "owner_is_target_judge_nonempty" => {
                "Prepare-phase trigger filter: the living owner is the event player and currently has at least one judgment-zone card. Requires Rule VM ABI 224."
            }
            "owner_is_target_draw_not_replaced" => {
                "Draw-phase trigger filter: the living owner is the event player and the pending normal draw has not already been replaced. Requires Rule VM ABI 225."
            }
            "owner_is_target_draw_not_fixed_hp_one_or_hand_empty" => {
                "Draw-phase trigger filter: the living owner is the event player, the pending normal draw count is not fixed, and the owner has exactly 1 HP or no hand cards. Requires Rule VM ABI 300."
            }
            "owner_source_sha_target_alive_owner_outside_target_attack_range" => {
                "Card-target trigger filter: the living owner used a logical Sha on a living target whose current attack range does not include the owner. Requires Rule VM ABI 119."
            }
            "owner_source_sha_dodged_target_alive_has_discardable_hand_equip" => {
                "Sha-dodged trigger filter: the living owner used the fully dodged Sha, and the living target has at least one discardable hand or equipment card. Requires Rule VM ABI 36."
            }
            "owner_source_sha_dodged_target_recorded_by_skill_has_discardable_hand_equip" => {
                "Sha-dodged trigger filter: the living owner armed this skill for the exact dodged Sha target, and that target can discard one owner hand/equipment card. Requires Rule VM ABI 152."
            }
            "owner_source_sha_target_alive_distance_le_one" => {
                "Damage trigger filter: the living owner is the source of logical Sha damage, the living target remains alive, and the final distance from owner to target is at most one. Requires Rule VM ABI 37."
            }
            "owner_alive_sha_target_distance_le_one" => {
                "Per-target trigger filter: a living owner observes a living logical Sha target whose final distance from the owner is at most one. Requires Rule VM ABI 227."
            }
            "owner_alive_sha_target_distance_le_one_has_equip_card" => {
                "Per-target trigger filter: a living owner observes a living logical Sha target within distance one and can discard an equipment card from hand/equipment. Requires Rule VM ABI 613."
            }
            "owner_source_target_alive_has_hand_or_equip" => {
                "Damage trigger filter: the living owner is the source and the living damaged player has at least one hand or equipment card."
            }
            "owner_source_other_target_alive_has_discardable_hand_equip" => {
                "Damage trigger filter: the living owner damaged another living player and may currently discard at least one card from that target's hand or equipment zone. Requires Rule VM ABI 118."
            }
            "owner_targeted_by_other_sha_once" => {
                "Card-target trigger filter: the living owner is targeted by another player's Sha and this skill has not fired during the current turn."
            }
            "owner_targeted_by_other_sha" => {
                "Card-target trigger filter: the living owner is targeted by another player's Sha."
            }
            "owner_is_target_wounded" => {
                "Trigger filter: the skill owner is the event player and is wounded."
            }
            "owner_is_target_linked" => {
                "Trigger filter: the living skill owner is the event player and is currently linked. Requires Rule VM ABI 189."
            }
            "owner_is_target_chain_damage_linked" => {
                "Damage-before trigger filter: the skill owner is the damage target and the damage is iron-chain-propagated. Implements 红棉百花袍. Requires Rule VM ABI 575."
            }
            "owner_is_target_wounded_sha" => {
                "Damage-before trigger filter: the skill owner is wounded and the damage comes from a Sha card. Implements 得势 (dcdeshi). Requires Rule VM ABI 575."
            }
            "owner_is_target_healthy_draw_not_fixed" => {
                "Draw-phase trigger filter: the skill owner is at full health and the draw phase amount is not fixed. Implements 螭纹玉佩 (chiwenyupei) draw branch. Appended for ABI-stable serialization."
            }
            "owner_is_target_alive_wounded_or_guiming_wu_exists" => {
                "Draw-phase trigger filter: the living owner is the event player and at least one living wounded player, or eligible other living Wu player under enabled Lord skill Guiming, exists. Requires Rule VM ABI 190."
            }
            "owner_is_target_hand_nonempty" => {
                "Trigger filter: the skill owner is the event player and has at least one hand card."
            }
            "owner_is_target_hand_below_max_hp" => {
                "Trigger filter: the skill owner is the event player and has fewer hand cards than maximum HP."
            }
            "owner_is_target_hand_above_hp" => {
                "Trigger filter: the living skill owner is the event player and has more hand cards than current HP."
            }
            "owner_first_transformed_sha_used_this_turn" => {
                "Card-used trigger filter: the living owner used a logical Sha converted by a skill from at least one physical card, limited to the first matching use each turn. Requires Rule VM ABI 357."
            }
            "owner_target_hand_empty_elemental_or_trick_damage_before" => {
                "Damage-before trigger filter: the living owner has no hand cards and is about to take elemental or ordinary-trick damage. Requires Rule VM ABI 358."
            }
            "owner_is_target_hand_at_least_hp_other_alive" => {
                "Trigger filter: the living skill owner is the event player, has at least as many hand cards as current HP, and another living player exists. Requires Rule VM ABI 69."
            }
            "owner_is_target_lost_cards_at_most_hp_gainable" => {
                "Trigger filter: the living skill owner is the event player, lost between one and current-HP cards from hand/equipment this turn, and at least one unique lost physical card remains in discard. Loss history is tracked only when loaded content needs this filter. Requires Rule VM ABI 201."
            }
            "owner_is_target_source_movable_field_card" => {
                "Damage trigger filter: the living owner is the damaged event player, the source is living, and at least one equipment or delayed-trick card can be moved from another living player's field to the source. Requires Rule VM ABI 202."
            }
            "owner_from_hand_empty_other_discardable_hand_equip" => {
                "Cards-moved trigger filter: the living skill owner lost at least one card from hand in this movement batch, is now empty-handed, and another living player has at least one hand/equipment card the owner may discard. Requires Rule VM ABI 70."
            }
            "owner_is_source_or_target" => {
                "Trigger filter: the skill owner is either the damage source or the damaged player."
            }
            "owner_is_source_or_target_other_alive_per_damage_point" => {
                "Damage trigger filter: the owner is one endpoint of sourced damage between two distinct living players; the trigger is offered independently once per damage point."
            }
            "owner_target_fire" => "Trigger filter: the skill owner is receiving fire damage.",
            "owner_target_thunder" => {
                "Trigger filter: the skill owner is receiving thunder damage."
            }
            _ => return super::owner_is_target_no_turn_damage::documentation(word),
    })
}
