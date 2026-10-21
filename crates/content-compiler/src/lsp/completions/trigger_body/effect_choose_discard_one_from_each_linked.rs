use super::*;

pub(super) fn items() -> Vec<LspCompletionItem> {
    vec![
            LspCompletionItem {
                label: "effect choose_discard_one_from_each_linked".into(),
                kind: 15,
                detail: "Discard one hand/equipment card from each linked player in seat order"
                    .into(),
                insert_text:
                    "effect choose_discard_one_from_each_linked(${1:self}, ${2:self_skill});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect judge_set_skill_mark_to_rank".into(),
                kind: 15,
                detail: "Judge and replace this skill's same-id mark with the result rank".into(),
                insert_text:
                    "effect judge_set_skill_mark_to_rank(${1:self}, ${2:self_skill});".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_redirect_current_sha".into(),
                kind: 15,
                detail: "Discard one card, then redirect the saved Sha target slot".into(),
                insert_text:
                    "effect choose_discard_redirect_current_sha(${1:self}, ${2:event_source}, ${3:event_card});"
                        .into(),
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
                label: "effect choose_prevent_damage_or_lose_max_hp_increase_damage".into(),
                kind: 15,
                detail: "Forced choice: prevent bound damage, or lose 1 max HP and increase it"
                    .into(),
                insert_text:
                    "effect choose_prevent_damage_or_lose_max_hp_increase_damage(self, 2);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_draw_or_increase_current_card_damage_or_both_mark".into(),
                kind: 15,
                detail: "Choose draw, current-card damage +1, or both plus a mark".into(),
                insert_text:
                    "effect choose_draw_or_increase_current_card_damage_or_both_mark(self, event_card, 2, 1, ${1:mark_def}, 1);"
                        .into(),
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
                label: "effect choose_add_or_remove_current_card_target".into(),
                kind: 15,
                detail: "Optionally add or remove one target from the current card use".into(),
                insert_text:
                    "effect choose_add_or_remove_current_card_target(self, event_card);".into(),
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
                label: "effect choose_gain_top_cards_by_suit".into(),
                kind: 15,
                detail: "Privately view top cards and optionally gain matching-suit cards"
                    .into(),
                insert_text:
                    "effect choose_gain_top_cards_by_suit(${1:self}, ${2:3}, ${3:heart});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_other_draw_by_hand_count".into(),
                kind: 15,
                detail: "Repeatedly choose another player; empty hands use a different draw count"
                    .into(),
                insert_text:
                    "effect choose_other_draw_by_hand_count(${1:self}, ${2:1}, ${3:2}, ${4:event_amount});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_target_draw".into(),
                kind: 15,
                detail: "Choose a living target from a typed scope and make them draw".into(),
                insert_text:
                    "effect choose_target_draw(${1:self}, ${2:1}, ${3:min_hand});".into(),
            },
            LspCompletionItem {
                label: "effect choose_targets_draw".into(),
                kind: 15,
                detail: "Choose distinct living targets from a typed scope and make each draw"
                    .into(),
                insert_text:
                    "effect choose_targets_draw(${1:self}, ${2:2}, ${3:2}, ${4:any});".into(),
            },
            LspCompletionItem {
                label: "effect choose_optional_targets_draw".into(),
                kind: 15,
                detail: "Optionally choose a bounded number of targets and make each draw"
                    .into(),
                insert_text:
                    "effect choose_optional_targets_draw(${1:self}, ${2:1}, ${3:1}, ${4:255}, ${5:wounded});"
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
                label: "effect choose_targets_damage".into(),
                kind: 15,
                detail: "Choose a bounded number of targets and damage each in seat order".into(),
                insert_text:
                    "effect choose_targets_damage(${1:self}, ${2:1}, ${3:1}, ${4:hand_empty}, ${5:1});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_target_draw_to_max_hp_cap".into(),
                kind: 15,
                detail: "Choose any living target and draw them up to capped maximum HP".into(),
                insert_text:
                    "effect choose_target_draw_to_max_hp_cap(${1:self}, ${2:5});".into(),
            },
            LspCompletionItem {
                label: "effect choose_target_draw_then_owner_if_below_max_hp".into(),
                kind: 15,
                detail:
                    "Choose a living target to draw, then draw one for the owner if that hand remains below max HP"
                        .into(),
                insert_text:
                    "effect choose_target_draw_then_owner_if_below_max_hp(${1:self}, ${2:2});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_target_draw_max_hp_cap_then_discard_excess".into(),
                kind: 15,
                detail:
                    "Choose a target to draw by capped max HP, then make that target discard excess hand cards"
                        .into(),
                insert_text:
                    "effect choose_target_draw_max_hp_cap_then_discard_excess(${1:self}, ${2:5});"
                        .into(),
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
                label: "effect choose_other_hand_targets_gain_one_replace_draw".into(),
                kind: 15,
                detail:
                    "Choose 1–2 other hand-card targets, gain one from each, and replace the draw"
                        .into(),
                insert_text:
                    "effect choose_other_hand_targets_gain_one_replace_draw(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect choose_other_hand_targets_gain_one_reduce_draw".into(),
                kind: 15,
                detail:
                    "Gain one hand card from selected players and reduce the draw by that count"
                        .into(),
                insert_text:
                    "effect choose_other_hand_targets_gain_one_reduce_draw(${1:self});".into(),
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
                label:
                    "effect choose_target_self_discard_then_owner_lose_hp_if_hand_ne_hp".into(),
                kind: 15,
                detail:
                    "Choose a target to self-discard, then lose HP if their hand differs from HP"
                        .into(),
                insert_text:
                    "effect choose_target_self_discard_then_owner_lose_hp_if_hand_ne_hp(${1:self});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_give_one_hand_equip_or_recover".into(),
                kind: 15,
                detail: "Let another player give one hand/equipment card or recover the target"
                    .into(),
                insert_text:
                    "effect choose_give_one_hand_equip_or_recover(${1:self}, ${2:event_source}, ${3:event_player});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_gzjinfa".into(),
                kind: 15,
                detail: "Discard the selected card, then resolve equipment gift or target-card gain"
                    .into(),
                insert_text: "effect resolve_gzjinfa(${1:self}, ${2:event_source});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_wn_qifeng".into(),
                kind: 15,
                detail: "Use a virtual Sha whose damage equals cards lost this turn".into(),
                insert_text: "effect resolve_wn_qifeng(${1:self}, ${2:event_player});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_dcjue".into(),
                kind: 15,
                detail: "Resolve Jue's discarded-card-capped random virtual-card loop".into(),
                insert_text: "effect resolve_dcjue(${1:self}, ${2:event_player});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_ruanji".into(),
                kind: 15,
                detail: "Resolve Ruanji's Zhaowen or Jiudun branch".into(),
                insert_text:
                    "effect resolve_ruanji(${1:self}, ${2:event_source}, ${3:event_card}, ${4:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_zhengbi".into(),
                kind: 15,
                detail: "Resolve Cuimao's Zhengbi branch".into(),
                insert_text:
                    "effect resolve_zhengbi(${1:self}, ${2:event_source}, ${3:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_fengying".into(),
                kind: 15,
                detail: "Resolve Cuimao's Fengying extra turn".into(),
                insert_text:
                    "effect resolve_fengying(${1:self}, ${2:597375372}, ${3:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_qiaojian".into(),
                kind: 15,
                detail: "Resolve Fugan's Qiaojian simultaneous discard and next-turn branch".into(),
                insert_text:
                    "effect resolve_qiaojian(${1:self}, ${2:event_source}, ${3:0}, ${4:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_xicha".into(),
                kind: 15,
                detail: "Resolve Fugan's Xicha hidden card watch".into(),
                insert_text:
                    "effect resolve_xicha(${1:self}, ${2:event_source}, ${3:event_card}, ${4:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_huzun".into(),
                kind: 15,
                detail: "Resolve Huzun's Zhantao or Anjing branch".into(),
                insert_text:
                    "effect resolve_huzun(${1:self}, ${2:event_source}, ${3:event_card}, ${4:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_zhugemengxue".into(),
                kind: 15,
                detail: "Resolve Zhugemengxue's Jichun or Hanying branch".into(),
                insert_text: "effect resolve_zhugemengxue(${1:self}, ${2:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_bailingyun".into(),
                kind: 15,
                detail: "Resolve Bailingyun's Linghui or Xiace branch".into(),
                insert_text: "effect resolve_bailingyun(${1:self}, ${2:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_dcshaowei".into(),
                kind: 15,
                detail: "Resolve Wei Guanyinping's Shaowei branches".into(),
                insert_text: "effect resolve_dcshaowei(${1:self}, ${2:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_dcdichou".into(),
                kind: 15,
                detail: "Resolve Wei Guanyinping's Dichou turn branches".into(),
                insert_text:
                    "effect resolve_dcdichou(${1:self}, ${2:0}, ${3:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_dcdufeng".into(),
                kind: 15,
                detail: "Resolve Ling Cao's Dufeng costs and turn modifiers".into(),
                insert_text:
                    "effect resolve_dcdufeng(${1:self}, ${2:0}, ${3:0});".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_hand_range".into(),
                kind: 15,
                detail: "Force a player to choose a range of hand cards to discard".into(),
                insert_text:
                    "effect choose_discard_hand_range(${1:self}, ${2:1}, ${3:2147483647});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_dcctjiuxian".into(),
                kind: 15,
                detail: "Resolve Chen Tai's Jiu Xian recast, Duel, and recovery".into(),
                insert_text:
                    "effect resolve_dcctjiuxian(${1:self}, ${2:self}, ${3:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_bianyue".into(),
                kind: 15,
                detail: "Resolve Bian Yue's Bizu or Wuxie branch".into(),
                insert_text: "effect resolve_bianyue(${1:self}, ${2:self}, ${3:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_zhupeilan".into(),
                kind: 15,
                detail: "Resolve Zhu Peilan's Cilv or Tongdao branch".into(),
                insert_text:
                    "effect resolve_zhupeilan(${1:self}, ${2:event_source}, ${3:event_card}, ${4:helper_skill_def}, ${5:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_dc_sb_zhangxiu".into(),
                kind: 15,
                detail: "Resolve DC SB Zhang Xiu's Fuxi or Haoyi branch".into(),
                insert_text:
                    "effect resolve_dc_sb_zhangxiu(${1:self}, ${2:event_player}, ${3:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_dc_sb_zhouyu".into(),
                kind: 15,
                detail: "Resolve DC SB Zhou Yu's Ronghuo or Yingmou branch".into(),
                insert_text:
                    "effect resolve_dc_sb_zhouyu(${1:self}, ${2:event_source}, ${3:event_card}, ${4:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_dc_sb_lusu".into(),
                kind: 15,
                detail: "Resolve DC SB Lu Su's Mingshi or Mengmou branch".into(),
                insert_text:
                    "effect resolve_dc_sb_lusu(${1:self}, ${2:event_source}, ${3:event_card}, ${4:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_zhangjian".into(),
                kind: 15,
                detail: "Resolve Zhangjian's damage rewrite or temporary skill grant".into(),
                insert_text:
                    "effect resolve_zhangjian(${1:self}, ${2:event_card}, ${3:grant_skill_def}, ${4:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_zhugeruoxue".into(),
                kind: 15,
                detail: "Resolve Zhugeruoxue's Qiongying or Nuanhui sequence".into(),
                insert_text:
                    "effect resolve_zhugeruoxue(${1:self}, ${2:self}, ${3:event_card}, ${4:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_give_two_different_colors_or_recover".into(),
                kind: 15,
                detail:
                    "Let another player give two differently colored hand/equipment cards or recover the target"
                        .into(),
                insert_text:
                    "effect choose_give_two_different_colors_or_recover(${1:self}, ${2:event_source}, ${3:event_player});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_mbzhuguo".into(),
                kind: 15,
                detail: "Resolve capped hand adjustment and the optional follow-up Sha".into(),
                insert_text: "effect resolve_mbzhuguo(${1:self}, ${2:event_source});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_mbanxian".into(),
                kind: 15,
                detail: "Gain and use a Sha from the first hand-discard event".into(),
                insert_text: "effect resolve_mbanxian(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_mbjinzu".into(),
                kind: 15,
                detail: "Reveal selected hand cards and install the next-Sha helper".into(),
                insert_text:
                    "effect resolve_mbjinzu(${1:self}, ${2:event_source}, ${3:helper_skill_def});"
                        .into(),
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
                label:
                    "effect choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp".into(),
                kind: 15,
                detail:
                    "Choose a nonempty card zone, discard it, draw by discarded cards plus lost HP, then lose 1 HP"
                        .into(),
                insert_text:
                    "effect choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp(${1:self});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_other_draw_owner_lost_hp_turn_over".into(),
                kind: 15,
                detail: "Choose another player, draw by owner lost HP, then turn them over".into(),
                insert_text:
                    "effect choose_other_draw_owner_lost_hp_turn_over(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect draw_by_play_phase_card_category_count".into(),
                kind: 15,
                detail: "Draw by distinct logical card categories used this play phase".into(),
                insert_text:
                    "effect draw_by_play_phase_card_category_count(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_one_for_sha_refund_and_draw_if_red".into(),
                kind: 15,
                detail: "Discard one card, refund the current Sha use once, and draw if red"
                    .into(),
                insert_text:
                    "effect choose_discard_one_for_sha_refund_and_draw_if_red(${1:self}, ${2:event_source}, ${3:event_card});"
                        .into(),
            },
            LspCompletionItem {
                label:
                    "effect choose_discard_one_for_sha_refund_and_draw_if_red_restore_if_equal_rank"
                        .into(),
                kind: 15,
                detail:
                    "Discard one card, resolve Longyin, and restore a skill on equal effective rank"
                        .into(),
                insert_text:
                    "effect choose_discard_one_for_sha_refund_and_draw_if_red_restore_if_equal_rank(${1:self}, ${2:event_source}, ${3:event_card}, ${4:skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_beige_configured".into(),
                kind: 15,
                detail:
                    "Resolve Beige with configurable Heart recovery and Diamond draw counts"
                        .into(),
                insert_text:
                    "effect resolve_beige_configured(${1:self}, ${2:event_player}, ${3:event_source}, ${4:event_amount}, ${5:3});"
                        .into(),
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
                label: "effect choose_add_current_sha_target".into(),
                kind: 15,
                detail: "Optionally add one legal target to the current Sha".into(),
                insert_text:
                    "effect choose_add_current_sha_target(${1:self}, ${2:event_card});".into(),
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
                label: "effect choose_store_hand_sha_in_skill_pile".into(),
                kind: 15,
                detail: "Store selected hand Sha in this skill's public pile".into(),
                insert_text: "effect choose_store_hand_sha_in_skill_pile(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect choose_spend_skill_pile_for_dying_jiu".into(),
                kind: 15,
                detail: "Spend one skill-pile card so a dying player uses virtual Jiu".into(),
                insert_text:
                    "effect choose_spend_skill_pile_for_dying_jiu(${1:self}, ${2:event_player});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_compare_selected_hand_card".into(),
                kind: 15,
                detail: "Compare the selected hand card and grant a win/lose helper skill".into(),
                insert_text:
                    "effect choose_compare_selected_hand_card(${1:self}, ${2:event_source}, ${3:win_skill_def}, ${4:lose_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_hand_compare".into(),
                kind: 15,
                detail: "Choose a hand-comparison target and both cards".into(),
                insert_text:
                    "effect choose_hand_compare(${1:self}, ${2:win_skill_def}, ${3:lose_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_other_damage".into(),
                kind: 15,
                detail: "Choose another living player and deal normal damage".into(),
                insert_text: "effect choose_other_damage(${1:self}, ${2:1});".into(),
            },
            LspCompletionItem {
                label: "effect choose_bound_source_attack_range_damage".into(),
                kind: 15,
                detail: "Choose a target in a bound source's attack range and deal damage from that source".into(),
                insert_text:
                    "effect choose_bound_source_attack_range_damage(${1:self}, ${2:event_source}, ${3:1});"
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
                label: "effect choose_gain_discard_card_by_kind".into(),
                kind: 15,
                detail: "Choose and gain one matching card from the discard pile".into(),
                insert_text:
                    "effect choose_gain_discard_card_by_kind(${1:self}, ${2:sha});".into(),
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
                label: "effect choose_gain_own_lost_discard_card_this_turn".into(),
                kind: 15,
                detail: "Choose one own card lost this turn that remains in discard".into(),
                insert_text:
                    "effect choose_gain_own_lost_discard_card_this_turn(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect choose_move_field_card_to".into(),
                kind: 15,
                detail: "Move one compatible field card to a fixed player".into(),
                insert_text:
                    "effect choose_move_field_card_to(${1:self}, ${2:event_source});".into(),
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
                label: "effect resolve_dingpan".into(),
                kind: 15,
                detail: "Draw, then resolve Dingpan's equipment branch and delayed damage".into(),
                insert_text: "effect resolve_dingpan(${1:self}, ${2:event_source});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_tanlu".into(),
                kind: 15,
                detail: "Resolve Tanlu's frozen HP-gap gift or damage branch".into(),
                insert_text: "effect resolve_tanlu(${1:self}, ${2:event_player});".into(),
            },
            LspCompletionItem {
                label: "effect choose_max_mark_other_judge_kill_unless_tao_or_taoyuan".into(),
                kind: 15,
                detail: "Choose a highest-marked other player, judge, and conditionally kill"
                    .into(),
                insert_text:
                    "effect choose_max_mark_other_judge_kill_unless_tao_or_taoyuan(${1:self}, ${2:mark_def});"
                        .into(),
            },
            LspCompletionItem {
                label:
                    "effect choose_zone_priority_gain_random_card_from_each_other_then_turn_over"
                        .into(),
                kind: 15,
                detail: "Choose a zone priority, gain one random card from each other player, then turn over"
                    .into(),
                insert_text:
                    "effect choose_zone_priority_gain_random_card_from_each_other_then_turn_over(${1:self});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_add_missing_skill".into(),
                kind: 15,
                detail: "Choose and gain one candidate skill not already owned".into(),
                insert_text:
                    "effect choose_add_missing_skill(${1:self}, ${2:first_skill_def}, ${3:second_skill_def}, ${4:third_skill_def});"
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
                label: "effect resolve_dccunsi".into(),
                kind: 15,
                detail: "Grant Yongjue, draw for another target, then resolve Guixiu recovery"
                    .into(),
                insert_text:
                    "effect resolve_dccunsi(${1:self}, ${2:event_source}, ${3:guixiu_def}, ${4:yongjue_def}, 0);"
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
                label: "effect choose_other_discard_target_card".into(),
                kind: 15,
                detail: "Choose another player, then discard one of their scoped cards".into(),
                insert_text:
                    "effect choose_other_discard_target_card(${1:self}, ${2:hand_equip});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_add_skill_bound_card_target".into(),
                kind: 15,
                detail: "Optionally append this skill instance's bound target".into(),
                insert_text:
                    "effect choose_add_skill_bound_card_target(${1:self}, ${2:event_card});"
                        .into(),
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
                label: "effect resolve_retieji".into(),
                kind: 15,
                detail: "Resolve 界铁骑's judgment, skill suppression, and gift branch".into(),
                insert_text: "effect resolve_retieji(self, event_card, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_new_reluoyi".into(),
                kind: 15,
                detail: "Resolve 界裸衣's top-three optional draw replacement".into(),
                insert_text: "effect resolve_new_reluoyi(self, ${1:buff_skill_def});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_remoshi".into(),
                kind: 15,
                detail: "Place or retrieve 没矢 cards".into(),
                insert_text:
                    "effect resolve_remoshi(self, event_player, event_card, ${1:helper_skill_def}, ${2:action});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_xinyaoming".into(),
                kind: 15,
                detail: "Resolve 界邀名's per-turn target and action choices".into(),
                insert_text: "effect resolve_xinyaoming(self);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_rehuaiyi".into(),
                kind: 15,
                detail: "Resolve 界怀异's reveal, color discard, and target gains".into(),
                insert_text: "effect resolve_rehuaiyi(self);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_reanguo".into(),
                kind: 15,
                detail: "Resolve 界安国's minimum-metric branches and optional recast".into(),
                insert_text: "effect resolve_reanguo(self);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_beizhu".into(),
                kind: 15,
                detail: "Resolve 备诛's hand reveal, discard-or-Sha sequence".into(),
                insert_text: "effect resolve_beizhu(self, event_source);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_mbshanjia".into(),
                kind: 15,
                detail: "Resolve mobile 缮甲's draw, dynamic discard, and category rewards"
                    .into(),
                insert_text:
                    "effect resolve_mbshanjia(self, mark_count(self, ${1:main_skill_def}), ${2:distance_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_daoji".into(),
                kind: 15,
                detail: "Resolve mobile 盗戟's equipment gain, forced use, and weapon damage"
                    .into(),
                insert_text: "effect resolve_daoji(self, event_source);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_tw_daoji".into(),
                kind: 15,
                detail: "Resolve TW 盗戟's hand/equipment gain and category reward".into(),
                insert_text: "effect resolve_tw_daoji(self, event_source);".into(),
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
                label: "effect choose_category_then_target_give_matching_or_sha_unrespondable"
                    .into(),
                kind: 15,
                detail: "Choose a card category; Sha target gives one or cannot respond".into(),
                insert_text: "effect choose_category_then_target_give_matching_or_sha_unrespondable(self, event_card, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect choose_category_then_target_give_matching_or_damage_plus_one"
                    .into(),
                kind: 15,
                detail: "Choose a category; damage target gives one or damage increases".into(),
                insert_text: "effect choose_category_then_target_give_matching_or_damage_plus_one(self, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect choose_other_excluding_source_random_gain_hand_equip_then_use_if_equip"
                    .into(),
                kind: 15,
                detail: "Choose a non-source role, randomly gain hand/equipment, use equipment"
                    .into(),
                insert_text: "effect choose_other_excluding_source_random_gain_hand_equip_then_use_if_equip(self, event_source);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_mobiledanshou".into(),
                kind: 15,
                detail: "Draw at zero targetings, otherwise discard that many to damage".into(),
                insert_text: "effect resolve_mobiledanshou(self, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_jsrgchiying".into(),
                kind: 15,
                detail: "Force in-range roles to discard; target conditionally gains basics"
                    .into(),
                insert_text: "effect resolve_jsrgchiying(self, event_source);".into(),
            },
            LspCompletionItem {
                label: "effect choose_lose_hp_use_virtual_sha_by_basic_variants".into(),
                kind: 15,
                detail: "Choose an ignore-distance virtual Sha after losing HP; distinct basic variants add damage".into(),
                insert_text:
                    "effect choose_lose_hp_use_virtual_sha_by_basic_variants(self);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twfupan".into(),
                kind: 15,
                detail: "Draw by damage, give a card, then apply per-target Fupan state".into(),
                insert_text: "effect resolve_twfupan(self, event_amount);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twjiaohua".into(),
                kind: 15,
                detail: "Give the draw recipient an unused category from draw/discard".into(),
                insert_text: "effect resolve_twjiaohua(self, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twkujian".into(),
                kind: 15,
                detail: "Gift or settle tracked TW Kujian physical cards".into(),
                insert_text: "effect resolve_twkujian(self, event_source, ${1:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twruilian".into(),
                kind: 15,
                detail: "Track a role's round discard categories and gain matching cards".into(),
                insert_text: "effect resolve_twruilian(self, event_player, ${1:0});".into(),
            },
            LspCompletionItem {
                label: "effect equip_weapon_from_draw_pile".into(),
                kind: 15,
                detail: "Randomly equip one draw-pile weapon if the slot is empty".into(),
                insert_text: "effect equip_weapon_from_draw_pile(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twjianwei".into(),
                kind: 15,
                detail: "Start either direction of TW Jianwei's prepare-phase comparison".into(),
                insert_text: "effect resolve_twjianwei(self, event_player, ${1:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twtanfeng".into(),
                kind: 15,
                detail: "Resolve TW Tanfeng's discard, Sha, damage, and phase-skip branches"
                    .into(),
                insert_text: "effect resolve_twtanfeng(self);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twzhibian".into(),
                kind: 15,
                detail: "Resolve TW Zhibian's comparison and win/loss branches".into(),
                insert_text: "effect resolve_twzhibian(self);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twyuyan".into(),
                kind: 15,
                detail: "Resolve TW Yuyan's gift-or-cancel current Sha target".into(),
                insert_text: "effect resolve_twyuyan(self, event_source, event_card);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twguoyi".into(),
                kind: 15,
                detail: "Resolve TW Guoyi's target branch and post-use replay".into(),
                insert_text:
                    "effect resolve_twguoyi(self, event_player, event_card, ${1:helper_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_twchuhai".into(),
                kind: 15,
                detail: "Resolve TW Chuhai's mission success or gained-card discard".into(),
                insert_text: "effect resolve_twchuhai(self, ${1:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twyizhu".into(),
                kind: 15,
                detail: "Resolve TW Yizhu planting, use, or discard tracking".into(),
                insert_text:
                    "effect resolve_twyizhu(self, ${1:event_source}, event_card, ${2:1});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_twluanchou".into(),
                kind: 15,
                detail: "Reset Gonghuan holders or redirect current damage".into(),
                insert_text:
                    "effect resolve_twluanchou(self, ${1:helper_skill_def}, ${2:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twxiawei".into(),
                kind: 15,
                detail: "Maintain Wei pile, Wangxing cost, or Qiongji damage".into(),
                insert_text:
                    "effect resolve_twxiawei(self, ${1:wangxing_skill_def}, ${2:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twbianfuren".into(),
                kind: 15,
                detail: "Resolve TW Wanwei damage/end phase or Yuejian pile placement".into(),
                insert_text: "effect resolve_twbianfuren(self, ${1:event_player}, ${2:helper_skill_def}, ${3:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twfeiyi".into(),
                kind: 15,
                detail: "Resolve TW Shengxi prepare/end or Kuanji transfer".into(),
                insert_text: "effect resolve_twfeiyi(self, ${1:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twchenzhen".into(),
                kind: 15,
                detail: "Resolve TW Muyue or Chayi state machines".into(),
                insert_text: "effect resolve_twchenzhen(self, ${1:event_player}, ${2:helper_skill_def}, ${3:record_skill_def}, ${4:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twqianju".into(),
                kind: 15,
                detail: "Equip a compatible shared-pile equipment card".into(),
                insert_text: "effect resolve_twqianju(self, ${1:event_player});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twqingxi".into(),
                kind: 15,
                detail: "Resolve TW Qingxi's first-Sha target choice".into(),
                insert_text: "effect resolve_twqingxi(self, ${1:event_player}, ${2:event_card});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twzaoli".into(),
                kind: 15,
                detail: "Resolve TW Zaoli discard, draw, and equipment refill".into(),
                insert_text: "effect resolve_twzaoli(self);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twzhengjian".into(),
                kind: 15,
                detail: "Resolve TW Zhengjian setup, tribute, switching, and Zhongchi".into(),
                insert_text: "effect resolve_twzhengjian(self, ${1:event_player}, ${2:helper_skill_def}, ${3:0});".into(),
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
                label: "effect resolve_jiaohua".into(),
                kind: 15,
                detail: "Resolve mobile 教化's persistent category cycle and recipient gain"
                    .into(),
                insert_text: "effect resolve_jiaohua(self);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_luanqun".into(),
                kind: 15,
                detail: "Resolve mobile 乱群's private reveal and temporary Sha restrictions"
                    .into(),
                insert_text:
                    "effect resolve_luanqun(self, ${1:effect_skill_def}, ${2:direct_hit_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_basic_then_target_discard_equip_or_damage".into(),
                kind: 15,
                detail: "Pay a basic hand card, then make the target discard equipment or take damage".into(),
                insert_text: "effect choose_discard_basic_then_target_discard_equip_or_damage(self, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect choose_current_sha_target_discard_redirect_to_owner".into(),
                kind: 15,
                detail: "Let the current target discard, then redirect the Sha to the skill owner".into(),
                insert_text: "effect choose_current_sha_target_discard_redirect_to_owner(self, event_source, event_card, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect choose_target_gain_bottom_card".into(),
                kind: 15,
                detail: "Reveal the bottom card, then give it to a chosen living player".into(),
                insert_text: "effect choose_target_gain_bottom_card(self);".into(),
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
                label: "effect choose_discard_target_card_resolve_current_sha".into(),
                kind: 15,
                detail: "Discard a target card, then resolve the equipment/non-equipment Sha branch".into(),
                insert_text: "effect choose_discard_target_card_resolve_current_sha(self, event_card, event_player);".into(),
            },
            LspCompletionItem {
                label:
                    "effect choose_discard_target_card_resolve_current_sha_nonbasic_bonus".into(),
                kind: 15,
                detail:
                    "Discard a target card, then resolve Rejianchu's basic/non-basic Sha branch"
                        .into(),
                insert_text: "effect choose_discard_target_card_resolve_current_sha_nonbasic_bonus(self, event_card, event_player, ${1:bonus_skill_def});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_dcqiaomeng".into(),
                kind: 15,
                detail:
                    "Choose a target card, discard it, then resolve 趫猛's equipment/trick branch"
                        .into(),
                insert_text:
                    "effect resolve_dcqiaomeng(self, event_card, ${1:helper_skill_def});".into(),
            },
            LspCompletionItem {
                label: "effect choose_draw_or_recover".into(),
                kind: 15,
                detail: "Let a specified player draw or recover".into(),
                insert_text:
                    "effect choose_draw_or_recover(${1:event_source}, ${2:1}, ${3:1}, ${4:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_recover_target".into(),
                kind: 15,
                detail: "Let one player optionally recover a specified target".into(),
                insert_text:
                    "effect choose_recover_target(${1:self}, ${2:event_source}, ${3:1});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_use_virtual_sha".into(),
                kind: 15,
                detail: "Choose legal targets and use a suitless virtual Sha".into(),
                insert_text:
                    "effect choose_use_virtual_sha(${1:self}, ${2:ignore_distance});".into(),
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
                label: "effect choose_use_virtual_sha_or_draw".into(),
                kind: 15,
                detail: "Use a normal-distance virtual Sha without counting it, or draw one"
                    .into(),
                insert_text: "effect choose_use_virtual_sha_or_draw(${1:self});".into(),
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
                label: "effect choose_target_lose_hp_discard_hand_sha_if_basic".into(),
                kind: 15,
                detail: "Choose a target, lose HP, then conditionally use a virtual Sha".into(),
                insert_text:
                    "effect choose_target_lose_hp_discard_hand_sha_if_basic(${1:self});".into(),
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
                label: "effect choose_target_reclaim_equipment_draw_if_range_reduced".into(),
                kind: 15,
                detail: "Reclaim target equipment and draw if its attack range shrinks".into(),
                insert_text:
                    "effect choose_target_reclaim_equipment_draw_if_range_reduced(${1:self}, ${2:event_source}, self_skill);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_use_virtual_card".into(),
                kind: 15,
                detail: "Use a source-free Wanjian or choose a Duel target".into(),
                insert_text:
                    "effect choose_use_virtual_card(${1:self}, ${2:wanjian}, self_skill);".into(),
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
                label: "effect choose_target_reveal_hand_damage_if_sha".into(),
                kind: 15,
                detail: "Choose and reveal a nonempty hand, then damage it when it contains Sha"
                    .into(),
                insert_text:
                    "effect choose_target_reveal_hand_damage_if_sha(${1:self}, ${2:1});".into(),
            },
            LspCompletionItem {
                label: "effect choose_other_give_one_hand_equip".into(),
                kind: 15,
                detail: "Choose another living player, then give them one hand/equipment card"
                    .into(),
                insert_text: "effect choose_other_give_one_hand_equip(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect choose_reveal_hand_card_bind_category_target".into(),
                kind: 15,
                detail: "Reveal a Play-phase player's hand card and bind its category".into(),
                insert_text:
                    "effect choose_reveal_hand_card_bind_category_target(${1:self}, ${2:event_player});"
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
                label: "effect choose_discard_target_card".into(),
                kind: 15,
                detail: "Choose and discard one scoped card from a target".into(),
                insert_text: "effect choose_discard_target_card(${1:self}, ${2:event_player}, ${3:hand_equip});".into(),
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
                label: "effect choose_optional_discard_target_card".into(),
                kind: 15,
                detail: "Optionally choose and discard one scoped target card".into(),
                insert_text: "effect choose_optional_discard_target_card(${1:self}, ${2:event_source}, ${3:hand});".into(),
            },
            LspCompletionItem {
                label: "effect reveal_hand_choose_discard_color".into(),
                kind: 15,
                detail: "Reveal a hand, then let its owner choose and discard one color".into(),
                insert_text:
                    "effect reveal_hand_choose_discard_color(${1:event_player});".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_target_cards_recover_same_category".into(),
                kind: 15,
                detail: "Discard target cards and recover when their categories match".into(),
                insert_text: "effect choose_discard_target_cards_recover_same_category(${1:self}, ${2:event_source}, ${3:2}, ${4:1});".into(),
            },
            LspCompletionItem {
                label: "effect choose_scskuiji_discard".into(),
                kind: 15,
                detail: "Discard up to count distinct-suit cards from owner+target hands".into(),
                insert_text: "effect choose_scskuiji_discard(${1:self}, ${2:event_source}, ${3:4});".into(),
            },
            LspCompletionItem {
                label: "effect choose_pskuiji_discard".into(),
                kind: 15,
                detail: "Like scskuiji; under-discarder loses 1 HP on asymmetric counts".into(),
                insert_text: "effect choose_pskuiji_discard(${1:self}, ${2:event_source}, ${3:4});".into(),
            },
            LspCompletionItem {
                label:
                    "effect choose_compare_selected_hand_card_second_failure_seat_damage".into(),
                kind: 15,
                detail: "Compare, branch on a strict win, and resolve a repeated-failure seat path"
                    .into(),
                insert_text: "effect choose_compare_selected_hand_card_second_failure_seat_damage(${1:self}, ${2:event_source}, ${3:1}, ${4:2}, ${5:1}, ${6:1});".into(),
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
