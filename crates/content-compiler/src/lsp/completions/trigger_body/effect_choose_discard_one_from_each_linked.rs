//! 触发体补全：通用效果（选择类见 `effect_choices`，结算器见 `effect_resolvers`）。

use super::*;

pub(super) fn items() -> Vec<LspCompletionItem> {
    vec![
            LspCompletionItem {
                label: "effect judge_set_skill_mark_to_rank".into(),
                kind: 15,
                detail: "Judge and replace this skill's same-id mark with the result rank".into(),
                insert_text:
                    "effect judge_set_skill_mark_to_rank(${1:self}, ${2:self_skill});".into(),
            },
            LspCompletionItem {
                label: "effect recover_hp_as".into(),
                kind: 15,
                detail: "Recover HP while preserving an explicit recovery source".into(),
                insert_text:
                    "effect recover_hp_as(${1:event_source}, ${2:self}, ${3:1});".into(),
            },
            LspCompletionItem {
                label: "effect prevent_current_damage".into(),
                kind: 15,
                detail: "Prevent the damage bound to a damage_before trigger".into(),
                insert_text: "effect prevent_current_damage();".into(),
            },
            LspCompletionItem {
                label: "effect randomize_current_hand_compare_opponent".into(),
                kind: 15,
                detail: "Replace the other comparison participant's choice with deterministic RNG"
                    .into(),
                insert_text: "effect randomize_current_hand_compare_opponent(self);".into(),
            },
            LspCompletionItem {
                label: "effect gain_highest_rank_sha_from_event_cards".into(),
                kind: 15,
                detail: "Gain every tied highest-rank Sha from the completed comparison".into(),
                insert_text: "effect gain_highest_rank_sha_from_event_cards(self);".into(),
            },
            LspCompletionItem {
                label: "effect gain_hand_compare_result_card".into(),
                kind: 15,
                detail: "Gain the comparison result card selected by legacy zongshi semantics"
                    .into(),
                insert_text: "effect gain_hand_compare_result_card(self);".into(),
            },
            LspCompletionItem {
                label: "effect judge_prevent_damage_lose_max_hp_on_nonheart".into(),
                kind: 15,
                detail: "Judge; on non-heart prevent current damage and reduce target max HP"
                    .into(),
                insert_text:
                    "effect judge_prevent_damage_lose_max_hp_on_nonheart(self, event_player);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect reveal_target_hand_sha_or_discard_bind".into(),
                kind: 15,
                detail: "Resolve Weikui's Shan branch and bind its temporary helper".into(),
                insert_text:
                    "effect reveal_target_hand_sha_or_discard_bind(${1:self}, ${2:event_source}, ${3:helper_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect draw_remembered_hand_cards".into(),
                kind: 15,
                detail: "Draw cards and remember those that remain continuously in hand".into(),
                insert_text:
                    "effect draw_remembered_hand_cards(${1:self}, ${2:2});".into(),
            },
            LspCompletionItem {
                label: "effect discard_remembered_hand_cards".into(),
                kind: 15,
                detail: "Discard remembered cards still continuously in hand".into(),
                insert_text:
                    "effect discard_remembered_hand_cards(${1:self});".into(),
            },
            LspCompletionItem {
                label:
                    "effect give_selected_then_target_choose_discard_same_suit_or_lose_hp".into(),
                kind: 15,
                detail:
                    "Give the selected hand card, then let its recipient discard matching-suit cards or lose HP"
                        .into(),
                insert_text:
                    "effect give_selected_then_target_choose_discard_same_suit_or_lose_hp(${1:self}, ${2:event_source});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect gain_equipment_from_draw_pile".into(),
                kind: 15,
                detail: "Gain the topmost equipment card from the draw pile".into(),
                insert_text: "effect gain_equipment_from_draw_pile(${1:self});".into(),
            },
            LspCompletionItem {
                label:
                    "effect target_choose_suit_gain_owner_hand_damage_if_mismatch".into(),
                kind: 15,
                detail:
                    "Target guesses a suit, gains one owner hand card, and takes damage on mismatch"
                        .into(),
                insert_text:
                    "effect target_choose_suit_gain_owner_hand_damage_if_mismatch(${1:self}, ${2:event_source});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect judge_source_discard_two_hand_or_damage".into(),
                kind: 15,
                detail:
                    "Judge; on non-heart the damage source discards two hand cards or takes 1 damage"
                        .into(),
                insert_text:
                    "effect judge_source_discard_two_hand_or_damage(${1:self}, ${2:event_source});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect give_selected_recover_after_phase_total_two".into(),
                kind: 15,
                detail:
                    "Give selected hand cards; recover once when the Play-phase total reaches two"
                        .into(),
                insert_text:
                    "effect give_selected_recover_after_phase_total_two(${1:self}, ${2:event_source});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect distribute_top_two_cards".into(),
                kind: 15,
                detail:
                    "Move the top two cards to processing and distribute all of them".into(),
                insert_text: "effect distribute_top_two_cards(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect give_all_hand_recover_to_one".into(),
                kind: 15,
                detail: "Give every hand card, then recover the dying recipient to one HP".into(),
                insert_text:
                    "effect give_all_hand_recover_to_one(${1:self}, ${2:event_player});".into(),
            },
            LspCompletionItem {
                label: "effect give_all_hand".into(),
                kind: 15,
                detail: "Give every current hand card to one recipient".into(),
                insert_text: "effect give_all_hand(${1:self}, ${2:event_source});".into(),
            },
            LspCompletionItem {
                label: "effect apply_mbjinzu_next_sha".into(),
                kind: 15,
                detail: "Consume the Jinzu helper on its owner's next Sha".into(),
                insert_text:
                    "effect apply_mbjinzu_next_sha(${1:self}, ${2:event_card});".into(),
            },
            LspCompletionItem {
                label: "effect add_bound_skill".into(),
                kind: 15,
                detail: "Grant a skill instance bound to one living target".into(),
                insert_text:
                    "effect add_bound_skill(${1:self}, ${2:skill_def}, ${3:event_source});"
                        .into(),
            },
            LspCompletionItem {
                label:
                    "effect replace_skill_with_bound_after_resolution_if_target_alive".into(),
                kind: 15,
                detail:
                    "After the resolution barrier, replace a skill with a target-bound helper when the target survived"
                        .into(),
                insert_text:
                    "effect replace_skill_with_bound_after_resolution_if_target_alive(${1:self}, ${2:self_skill}, ${3:skill_def}, ${4:event_source});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect draw_by_play_phase_card_category_count".into(),
                kind: 15,
                detail: "Draw by distinct logical card categories used this play phase".into(),
                insert_text:
                    "effect draw_by_play_phase_card_category_count(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect queue_opening_extra_play_phase".into(),
                kind: 15,
                detail: "Queue an extra Play phase before the current turn's Prepare phase"
                    .into(),
                insert_text: "effect queue_opening_extra_play_phase(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect convert_current_sha_to_fire_lose_hp_if_damage".into(),
                kind: 15,
                detail: "Convert the current ordinary Sha to fire with its post-use HP cost"
                    .into(),
                insert_text:
                    "effect convert_current_sha_to_fire_lose_hp_if_damage(${1:self}, ${2:event_card});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect increase_current_sha_base_damage".into(),
                kind: 15,
                detail: "Increase the suspended Sha use's base damage".into(),
                insert_text:
                    "effect increase_current_sha_base_damage(${1:self}, ${2:event_card}, ${3:1});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect increase_current_card_base_damage".into(),
                kind: 15,
                detail: "Increase the suspended Sha/Nanman/Wanjian use's base damage".into(),
                insert_text:
                    "effect increase_current_card_base_damage(${1:self}, ${2:event_card}, ${3:1});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect lose_hp_then_damage_other_lose_max_hp_if_dying".into(),
                kind: 15,
                detail:
                    "Lose HP, conditionally lose max HP on exact dying, then damage another"
                        .into(),
                insert_text:
                    "effect lose_hp_then_damage_other_lose_max_hp_if_dying(${1:self}, ${2:event_source}, ${3:1}, self_skill);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect reveal_uniform_hand_choose_damage".into(),
                kind: 15,
                detail: "Reveal a uniform-color hand, then choose any living player to damage"
                    .into(),
                insert_text:
                    "effect reveal_uniform_hand_choose_damage(${1:self}, ${2:1});".into(),
            },
            LspCompletionItem {
                label: "effect draw_for_lifetime_card_action_divisors".into(),
                kind: 15,
                detail: "Draw once for each divisor of the player's lifetime card-action count"
                    .into(),
                insert_text:
                    "effect draw_for_lifetime_card_action_divisors(${1:self}, ${2:3}, ${3:5}, ${4:8});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect gain_draw_cards_for_lifetime_card_action_divisors".into(),
                kind: 15,
                detail: "Gain one topmost matching draw-pile card for each matching divisor"
                    .into(),
                insert_text:
                    "effect gain_draw_cards_for_lifetime_card_action_divisors(${1:self}, ${2:3}, ${3:sha_or_shan}, ${4:5}, ${5:tao_or_jiu}, ${6:8}, ${7:juedou_or_wuzhong});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect lose_hp_gain_discard_card_for_opening_extra_play".into(),
                kind: 15,
                detail: "Lose HP and gain a discard card in this skill's opening extra Play phase"
                    .into(),
                insert_text:
                    "effect lose_hp_gain_discard_card_for_opening_extra_play(${1:self}, ${2:sha}, ${3:mandatory_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect draw_then_optionally_distribute_hand_cards".into(),
                kind: 15,
                detail: "Draw, then optionally distribute current hand cards".into(),
                insert_text:
                    "effect draw_then_optionally_distribute_hand_cards(${1:self}, ${2:2}, ${3:2});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect give_selected_then_continue_distinct_recover_at_two".into(),
                kind: 15,
                detail: "Give selected cards, then optionally repeat for distinct recipients"
                    .into(),
                insert_text:
                    "effect give_selected_then_continue_distinct_recover_at_two(${1:self}, ${2:event_source});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect give_selected_then_recover_target".into(),
                kind: 15,
                detail: "Give selected cards, then recover the target after movement triggers"
                    .into(),
                insert_text:
                    "effect give_selected_then_recover_target(${1:self}, ${2:event_source}, ${3:1});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect give_selected_then_target_may_use_draw_owner".into(),
                kind: 15,
                detail: "Give one non-equipment card; recipient may use it; owner draws"
                    .into(),
                insert_text:
                    "effect give_selected_then_target_may_use_draw_owner(${1:self}, ${2:event_source});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect add_first_missing_skill".into(),
                kind: 15,
                detail: "Gain the first candidate skill not already owned".into(),
                insert_text:
                    "effect add_first_missing_skill(${1:self}, ${2:first_skill_def}, ${3:second_skill_def}, ${4:third_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect record_current_sha_all_targets".into(),
                kind: 15,
                detail: "Bind every current Sha target to the originating skill".into(),
                insert_text:
                    "effect record_current_sha_all_targets(${1:self}, ${2:event_card});".into(),
            },
            LspCompletionItem {
                label: "effect judge_cancel_current_sha_target_on_red".into(),
                kind: 15,
                detail: "Judge and exclude the current Sha target on red".into(),
                insert_text: "effect judge_cancel_current_sha_target_on_red(self, event_card, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect make_current_sha_target_unrespondable".into(),
                kind: 15,
                detail: "Disable only the current Sha target's response window".into(),
                insert_text: "effect make_current_sha_target_unrespondable(self, event_card, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect judge_make_current_sha_target_unrespondable_on_red".into(),
                kind: 15,
                detail: "Judge and disable the current Sha target's response window on red".into(),
                insert_text: "effect judge_make_current_sha_target_unrespondable_on_red(self, event_card, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect give_all_hand_then_choose_use_one_or_damage".into(),
                kind: 15,
                detail: "Give all hand cards; recipient uses one or takes damage".into(),
                insert_text:
                    "effect give_all_hand_then_choose_use_one_or_damage(self, event_source);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect equip_weapon_from_draw_pile".into(),
                kind: 15,
                detail: "Randomly equip one draw-pile weapon if the slot is empty".into(),
                insert_text: "effect equip_weapon_from_draw_pile(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect suppress_non_locked_skills_until_turn_end".into(),
                kind: 15,
                detail: "Suppress one target's non-locked skills for this turn".into(),
                insert_text:
                    "effect suppress_non_locked_skills_until_turn_end(${1:event_source});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect judge_black_gain_repeat".into(),
                kind: 15,
                detail: "Judge, gain a black result, and optionally repeat".into(),
                insert_text: "effect judge_black_gain_repeat(self);".into(),
            },
            LspCompletionItem {
                label: "effect judge_black_gain_repeat_protect_hand_limit".into(),
                kind: 15,
                detail: "Judge, gain black results, and protect those cards from this turn's hand limit".into(),
                insert_text: "effect judge_black_gain_repeat_protect_hand_limit(self);".into(),
            },
            LspCompletionItem {
                label: "effect judge_same_color_gain_repeat".into(),
                kind: 15,
                detail: "Judge, gain results matching the first color, and optionally repeat".into(),
                insert_text: "effect judge_same_color_gain_repeat(self);".into(),
            },
            LspCompletionItem {
                label: "effect use_virtual_juedou_second_on_first".into(),
                kind: 15,
                detail: "Make the second active target use Duel on the first".into(),
                insert_text: "effect use_virtual_juedou_second_on_first();".into(),
            },
            LspCompletionItem {
                label: "effect adjust_hand_to".into(),
                kind: 15,
                detail: "Draw or force-discard to an exact hand count".into(),
                insert_text: "effect adjust_hand_to(${1:self}, ${2:4});".into(),
            },
            LspCompletionItem {
                label: "effect gain_card_by_kind".into(),
                kind: 15,
                detail: "Gain a card kind from draw pile, then discard pile".into(),
                insert_text: "effect gain_card_by_kind(${1:self}, ${2:tiesuo});".into(),
            },
            LspCompletionItem {
                label: "effect gain_discard_card_by_kind".into(),
                kind: 15,
                detail: "Gain a card kind from the discard pile only".into(),
                insert_text: "effect gain_discard_card_by_kind(${1:self}, ${2:sha});".into(),
            },
            LspCompletionItem {
                label: "effect disable_all_equip_slots".into(),
                kind: 15,
                detail: "Disable every currently enabled equipment slot".into(),
                insert_text: "effect disable_all_equip_slots(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect enable_equip_slot".into(),
                kind: 15,
                detail: "Restore one disabled copy of an equipment slot".into(),
                insert_text:
                    "effect enable_equip_slot(${1:event_source}, ${2:1}, ${3:self});".into(),
            },
            LspCompletionItem {
                label: "effect move_equipment_then_draw".into(),
                kind: 15,
                detail: "Move equipment to another legal player, then draw for its owner".into(),
                insert_text: "effect move_equipment_then_draw(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect use_virtual_card_on_selected_targets".into(),
                kind: 15,
                detail: "Use one virtual Duel or Tiesuo on active-skill targets".into(),
                insert_text:
                    "effect use_virtual_card_on_selected_targets(${1:self}, juedou);".into(),
            },
            LspCompletionItem {
                label: "effect target_discard_then_duel_if_non_sha_and_hp_gte_owner".into(),
                kind: 15,
                detail: "Target discards, then receives Duel when its non-Sha discard and HP allow"
                    .into(),
                insert_text:
                    "effect target_discard_then_duel_if_non_sha_and_hp_gte_owner(${1:self}, ${2:event_source});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect deal_damage_ban_active_if_dying".into(),
                kind: 15,
                detail: "Deal damage and ban this active skill if it creates dying".into(),
                insert_text:
                    "effect deal_damage_ban_active_if_dying(${1:self}, ${2:event_source}, ${3:1}, self_skill);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect record_selected_play_phase_category".into(),
                kind: 15,
                detail: "Record the selected card category for this play phase".into(),
                insert_text: "effect record_selected_play_phase_category(self_skill);".into(),
            },
            LspCompletionItem {
                label: "effect discard_equipment_pair_with_armor".into(),
                kind: 15,
                detail: "Discard equipment from two selected targets, including armor".into(),
                insert_text: "effect discard_equipment_pair_with_armor(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect discard_target_equipment_gain_mount".into(),
                kind: 15,
                detail: "Discard target equipment, then gain it when it is a mount".into(),
                insert_text:
                    "effect discard_target_equipment_gain_mount(${1:self}, ${2:event_player});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect discard_target_equipment_draw_weapon_range".into(),
                kind: 15,
                detail: "Discard target equipment; a weapon makes its former owner draw its range"
                    .into(),
                insert_text:
                    "effect discard_target_equipment_draw_weapon_range(${1:self}, ${2:event_source});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect recover_to_max_then_optional_exchange_skill".into(),
                kind: 15,
                detail:
                    "Recover to max, then optionally exchange another skill to restore this one"
                        .into(),
                insert_text:
                    "effect recover_to_max_then_optional_exchange_skill(${1:self}, ${2:skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect reveal_hand_discard_kind".into(),
                kind: 15,
                detail: "Publicly reveal a hand, then discard every matching effective card kind"
                    .into(),
                insert_text:
                    "effect reveal_hand_discard_kind(${1:event_player}, ${2:shan});".into(),
            },
            LspCompletionItem {
                label: "effect discard_target_card_gain_mount".into(),
                kind: 15,
                detail: "Discard a scoped target card, then gain it when it is a mount".into(),
                insert_text: "effect discard_target_card_gain_mount(${1:self}, ${2:event_player}, ${3:hand_equip_judge});".into(),
            },
            LspCompletionItem {
                label: "effect reveal_hand_to_player".into(),
                kind: 15,
                detail: "Reveal a complete hand only to one observer".into(),
                insert_text:
                    "effect reveal_hand_to_player(${1:self}, ${2:event_source});".into(),
            },
            LspCompletionItem {
                label: "effect reveal_hand".into(),
                kind: 15,
                detail: "Publicly reveal a complete hand".into(),
                insert_text: "effect reveal_hand(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect reveal_hand_choose_discard_color".into(),
                kind: 15,
                detail: "Reveal a hand, then let its owner choose and discard one color".into(),
                insert_text:
                    "effect reveal_hand_choose_discard_color(${1:event_player});".into(),
            },
            LspCompletionItem {
                label: "effect deal_damage".into(),
                kind: 15,
                detail: "Deal normal, fire, or thunder damage".into(),
                insert_text:
                    "effect deal_damage(self, ${1:event_source}, ${2:1}, ${3:fire});".into(),
            },
    ]
}
