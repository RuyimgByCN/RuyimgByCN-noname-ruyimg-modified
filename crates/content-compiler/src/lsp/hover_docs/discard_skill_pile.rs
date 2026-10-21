pub(in super::super) fn documentation(word: &str) -> Option<&'static str> {
    Some(match word {
            "discard_skill_pile" => {
                "Effect: discards every card from your own skill pile (the cards placed on your general card by this skill) into the discard pile. No-op when the pile is empty.\n\n```\neffect discard_skill_pile(self, self);\n```"
            }
            "discard_basic_draw_equal" => {
                "玉蝉引擎（ABI 657）：出牌阶段限一次，弃置任意张（≥1）基本手牌并摸等量的牌。参数 (player, self_skill)。"
            }
            "start_debate_all_others" => {
                "发起议事·全员（ADR 0644 P2，ABI 655）：所有其他有手牌角色亮首张手牌分组，只发 DebateResolved 事件。参数 (player, self_skill)。"
            }
            "start_debate_targets" => {
                "发起议事·指定目标（ADR 0644 P2，ABI 656）：选 1..N 名其他角色议事并发 DebateResolved 事件。参数 (player, self_skill)。"
            }
            "resolve_jsrgchaozheng" => {
                "朝争议事（ADR 0644 P1，ABI 653）：所有其他角色亮首张手牌为意见，红黑分组多数决：红=红意见者各回复1；黑=红意见者各失去1；全同意见发起者摸参与者数张。参数 (player, self_skill)。"
            }
            "resolve_jsrgsheju" => {
                "设居双人议事（ADR 0644 P1，ABI 654）：与对方亮牌议事，黑多数则双方各减1点体力上限。参数 (player, counterpart, self_skill)。"
            }
            "choose_other_add_mark" => {
                "选他人授标记（ABI 652）：选一名其他存活角色，对其施加标记增减（护甲=3 / 横置=2 等）。参数 (player, mark_def, delta)。"
            }
            "place_hand_card_to_expansion" => {
                "置入（expansion P2，ADR 0641）：选择一张手牌置于自己的技能牌堆（武将牌上）。参数 (player, self_skill)。"
            }
            "extra_resolve_current_trick(player, card, self_skill)" => {
                "额外结算家族（clanfenshi 等，上游 effectCount++）：为当前挂起中的锦囊（WuxieWindow）追加一次效果结算。必须在 card_used 触发体内调用，card 传 event_card。"
            }
            "choose_clan_not_max_hand_limit_gain" => {
                "穆荫 (clanmuyin)：选择一名自己或与自己同族（共享任一族名）且手牌上限非全场最高的存活角色，其手牌上限永久 +1。参数 (player, self_skill)。"
            }
                        "debate_owner_is_initiator" => {
                "议事结算后触发：拥有者是发起者（ADR 0644 P2）。"
            }
            "debate_owner_participated" => {
                "议事结算后触发：拥有者亮牌参与（红或黑意见）。"
            }
            "owner_debate_participant" => {
                "Trigger filter: the owner is among the participants of a debate that just opened (ADR 0644 P3, debateShowOpinion window)."
            }
            "owner_cards_moved_contain_ying" => {
                "Trigger filter: cards moved into the discard pile and at least one is a shadow card (jsrg funi③)."
            }
            "owner_is_lord" => {
                "Trigger filter: the skill owner has the Lord identity (identity mode)."
            }
            "cards_moved_discard_at_least_three" => {
                "Card-move trigger filter: at least three cards moved into the discard pile in a single discard move. Implements 集计 (diy nsjihui)."
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
            "debate_owner_is_initiator_result_red" => {
                "议事结算为红色且拥有者为发起者时触发。"
            }
            "debate_owner_is_initiator_result_black" => {
                "议事结算为黑色且拥有者为发起者时触发。"
            }
            "debate_owner_opinion_differs_initiator" => {
                "议事结算后触发：拥有者与发起者均参与且意见颜色不同。"
            }
"owner_is_respond_to" => {
                "矢仄 (clanshize) 过滤器：CardResponded 事件中被响应的牌属于技能拥有者（ctx.respond_to == owner）。注意响应事件中 ctx.source 是响应者。"
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
            "choose_hand_count_different_target_draw_or_random_discard" => {
                "Effect: forces the owner to select one snapshotted living player with a currently different hand count. A lower-hand target draws one; a higher-hand target discards one hand card selected by authoritative deterministic RNG. Requires Rule VM ABI 60.\n\n```\neffect choose_hand_count_different_target_draw_or_random_discard(self);\n```"
            }
            "increase_current_draw_count" => {
                "Effect: adds a nonnegative amount to the current draw phase's pending draw count. The authoritative draw entry consumes the accumulated bonus exactly once, preserving one draw batch. Only valid in `phase_draw` triggers. Requires Rule VM ABI 61.\n\n```\neffect increase_current_draw_count(self, alive_group_count());\n```"
            }
            "skip_next_play_phase" => {
                "Effect: marks the current living player's upcoming Play phase to be skipped through the authoritative turn lifecycle. Valid only before Play in that player's current turn. Requires Rule VM ABI 100.\n\n```\neffect skip_next_play_phase(self);\n```"
            }
            "skip_next_judge_phase" => {
                "Effect: persistently marks a living player's next judgment phase to be skipped, then consumes the mark when that phase is skipped. Requires Rule VM ABI 228.\n\n```\neffect skip_next_judge_phase(self);\n```"
            }
            "choose_discard_damage_endpoint_card_increase_if_suit" => {
                "Damage-before effect: the source chooses and discards one currently discardable hand/equipment card from themselves or the damage target. After the discard and nested triggers resolve, the bound damage increases by `amount` when the card's owner-effective suit matches. Requires Rule VM ABI 230.\n\n```\neffect choose_discard_damage_endpoint_card_increase_if_suit(self, event_player, diamond, 1);\n```"
            }
            "choose_other_draw_or_recover" => {
                "Effect: opens `times` independently cancellable choices for `player` to select another living player. Each selected target must then choose to draw `draw_count` cards or recover `recover_count` HP; when recovery is unavailable, drawing resolves immediately. Requires Rule VM ABI 101.\n\n```\neffect choose_other_draw_or_recover(self, 2, 1, event_amount);\n```"
            }
            "choose_other_recover" => {
                "Effect: forces `player` to select another living player, then recovers that target by `amount` HP. Requires Rule VM ABI 160.\n\n```\neffect choose_other_recover(self, 2);\n```"
            }
            "enable_equip_slot" => {
                "Effect: restores one disabled copy of equipment `slot` (`1` weapon through `5` treasure) for `player`, attributed to `source`. Requires Rule VM ABI 161.\n\n```\neffect enable_equip_slot(event_source, 1, self);\n```"
            }
            "target_choose_suit_gain_owner_hand_damage_if_mismatch" => {
                "Effect: `target` chooses one suit, then one current hand card from `player` to gain. If the card's effective suit for `player` differs, `player` deals 1 normal damage to `target`. Active use requires `hand_nonempty targets 1 1 other`. Requires Rule VM ABI 168.\n\n```\neffect target_choose_suit_gain_owner_hand_damage_if_mismatch(self, event_source);\n```"
            }
            "judge_source_discard_two_hand_or_damage" => {
                "Effect: `player` judges. A heart ends the effect. Otherwise the living damage `source` may discard exactly two discardable hand cards; cancelling or having fewer than two makes `player` deal 1 normal damage to `source`. Requires Rule VM ABI 171.\n\n```\neffect judge_source_discard_two_hand_or_damage(self, event_source);\n```"
            }
            "give_selected_recover_after_phase_total_two" => {
                "Active-only effect: gives the invocation's selected hand cards from `player` to `target`. The skill tracks the number given in the current Play phase and recovers `player` by 1 after movement triggers only when the total first crosses two. Requires `targets 1 1 other select_hand 1 255` and Rule VM ABI 172.\n\n```\neffect give_selected_recover_after_phase_total_two(self, event_source);\n```"
            }
            "choose_other_hand_targets_gain_one_replace_draw" => {
                "Draw-phase effect: `player` chooses one or two other living players with hand cards, then gains one current hand card from each in selection order and replaces the normal draw. Requires Rule VM ABI 173.\n\n```\neffect choose_other_hand_targets_gain_one_replace_draw(self);\n```"
            }
            "choose_other_hand_targets_gain_one_reduce_draw" => {
                "Draw-phase effect: `player` chooses between one and the current pending draw count of other living players with hand cards, gains one current hand card from each in seat order, and reduces the pending draw by the selected-player count. Requires Rule VM ABI 311.\n\n```\neffect choose_other_hand_targets_gain_one_reduce_draw(self);\n```"
            }
            "distribute_top_two_cards" => {
                "Damage-trigger effect: move the draw-pile top two cards to public processing, then repeatedly choose a nonempty remaining subset and any living recipient until all cards are distributed. Pair with `owner_is_target_per_damage_point optional`. Requires Rule VM ABI 174.\n\n```\neffect distribute_top_two_cards(self);\n```"
            }
            "choose_discard_hand_equip_then_remove_self_skill" => {
                "Effect: forces the owner to discard `min(count, available hand/equipment cards)` cards, then removes the originating one-shot skill after movement triggers and nested choices finish. Requires Rule VM ABI 102.\n\n```\neffect choose_discard_hand_equip_then_remove_self_skill(self, field_equipment_kind_count(weapon));\n```"
            }
            "choose_discard_hand_equip_then_recover" => {
                "Effect: forces the owner to discard exactly `count` current hand/equipment cards, then recovers `amount` HP after movement triggers and nested choices finish. If the cost is no longer payable, neither discard nor recovery occurs. Requires Rule VM ABI 134.\n\n```\neffect choose_discard_hand_equip_then_recover(self, 2, 1);\n```"
            }
            "draw_selected_targets" => {
                "Active-only effect: snapshots the authoritative active-skill target list and draws `count` cards for each still-living distinct target in seat order. Requires at least one declared active target and Rule VM ABI 103.\n\n```\neffect draw_selected_targets(3);\n```"
            }
            "draw_linked_players" => {
                "Effect: draw `count` cards for every currently linked living player in seat order. Requires Rule VM ABI 344.\n\n```\neffect draw_linked_players(1);\n```"
            }
            "reveal_top_gain_if_color" => {
                "Effect: publicly reveal the draw-pile top card and gain it when its printed color is `red` or `black`; otherwise put it in the discard pile. Requires Rule VM ABI 352.\n\n```\neffect reveal_top_gain_if_color(self, red);\n```"
            }
            "recover_selected_targets" => {
                "Active-only effect: snapshots the authoritative active-skill target list and recovers `amount` HP for each still-living distinct target in seat order. Requires at least one declared active target and Rule VM ABI 109.\n\n```\neffect recover_selected_targets(1);\n```"
            }
            "damage_then_draw_selected_targets" => {
                "Active-only effect: snapshots distinct living active targets and, in seat order, deals damage from `source` then lets that target draw before continuing. Requires Rule VM ABI 279.\n\n```\neffect damage_then_draw_selected_targets(self, 1, 1);\n```"
            }
            "resolve_youdi" => {
                "End-phase effect: choose another player; that player discards one of the owner's hand/equipment cards; if it was not logical Sha, the owner gains one of that player's hand/equipment cards. Requires Rule VM ABI 280.\n\n```\neffect resolve_youdi(self);\n```"
            }
            "resolve_kuangfu" => {
                "Damage-source effect: choose one discardable equipment card from the damaged target, then discard it or move it into the owner's compatible equipment slot. Requires Rule VM ABI 282.\n\n```\neffect resolve_kuangfu(self, event_player);\n```"
            }
            "owner_source_sha_damage_target_alive_has_discardable_equip" => {
                "Damage trigger filter: the living owner caused logical Sha damage to a living target that currently has equipment the owner may discard. Requires Rule VM ABI 282."
            }
            "resolve_shanjia" => {
                "Play-phase effect: draw `min(count, 7)` cards, then discard that many current hand/equipment cards. Discarding equipment from the owner's equipment zone offers a normal-use-count, distance-unlimited virtual Sha after movement triggers finish. Requires Rule VM ABI 283.\n\n```\neffect resolve_shanjia(self, mark_count(self, self_skill));\n```"
            }
            "owner_source_equip_used_skill_mark_below_seven" => {
                "Card-used trigger filter: the living owner used a logical equipment card and this skill definition's stable player mark is below seven. Requires Rule VM ABI 283."
            }
            "resolve_new_yajiao" => {
                "Outside-turn card effect: reveal the draw-pile top card, force the owner to choose any living recipient, and give it after reveal triggers. If its logical basic/trick/equipment category differs from the triggering card, the owner then discards one hand/equipment card. Requires Rule VM ABI 284.\n\n```\neffect resolve_new_yajiao(self, event_card);\n```"
            }
            "resolve_dingpin" => {
                "Active effect: the selected target judges. Black makes that target draw their lost HP and become unavailable to this skill for the turn; red turns the owner over. Requires Rule VM ABI 285.\n\n```\neffect resolve_dingpin(self, event_player);\n```"
            }
            "owner_source_outside_turn_card_used_or_responded" => {
                "Card-used/responded filter: the living owner is the event source and used or responded with a logical card outside their own turn. Requires Rule VM ABI 284."
            }
            "record_selected_play_phase_category" => {
                "Active-only effect: record the exact selected card's effective basic/trick/equipment category in `self_skill` for the current Play phase. Requires exact one-card selection with `unused_category_by_skill_this_play_phase` and Rule VM ABI 110.\n\n```\neffect record_selected_play_phase_category(self_skill);\n```"
            }
            "recover_if_min_current_hp" => {
                "Effect: at resolution time, recovers `amount` HP for `player` only when that living player is tied for the lowest current HP. Requires Rule VM ABI 103.\n\n```\neffect recover_if_min_current_hp(self, 1);\n```"
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
            "choose_target_discard_excess_then_self_discard_or_target_draw" => {
                "Effect: during the owner's Prepare phase, choose any living player with discardable hand/equipment cards. Discard max(target hand minus HP, 1) cards, capped to what is available. If any discarded card is not equipment, the owner may discard that many cards; cancellation or inability makes the target draw that many instead. Requires Rule VM ABI 62.\n\n```\neffect choose_target_discard_excess_then_self_discard_or_target_draw(self);\n```"
            }
            "resolve_decadezhenjun" => {
                "Effect: during the owner's Prepare or End phase, choose any living player whose hand/equipment cards the owner may discard, including the owner. Discard max(target hand minus HP, 1), capped to currently available cards. Any equipment among the discarded cards ends the effect. If all are non-equipment, the owner may discard exactly one hand/equipment card; cancellation or inability makes the target draw the total number discarded. Uses independent payload stages so legacy Rezhenjun remains stable. Requires Rule VM ABI 298.\n\n```\neffect resolve_decadezhenjun(self);\n```"
            }
            "resolve_sp_xiaoguo" => {
                "Effect: during another living player's End phase, the owner may discard any one hand card. The phase player then may discard an effective equipment card from hand or equipment; success makes the owner draw one after movement triggers, while cancellation or inability deals one normal damage from the owner. Uses an independent state machine so Standard Xiaoguo remains stable. Requires Rule VM ABI 299.\n\n```\neffect resolve_sp_xiaoguo(self, event_player);\n```"
            }
            "choose_gain_card_from_each_other_replace_draw" => {
                "Effect: immediately fixes the pending normal Draw-phase count to zero, then snapshots every other living player in owner-relative clockwise order. For each target still alive, choose and gain exactly one currently gainable hand/equipment/judgment card, waiting for its movement triggers before continuing. Requires Rule VM ABI 300.\n\n```\neffect choose_gain_card_from_each_other_replace_draw(self);\n```"
            }
            "draw_then_choose_gain_one_from_damage_target_and_neighbors" => {
                "Effect: after damage caused by the owner, draw one card and wait for nested draw triggers. Then process the damaged player, their previous living seat, and their next living seat in that order, choosing and gaining one hand/equipment card from each. If the owner appears in the sequence, only equipment is eligible; duplicate previous/next seats are preserved. Requires Rule VM ABI 63.\n\n```\neffect draw_then_choose_gain_one_from_damage_target_and_neighbors(self, event_player);\n```"
            }
            "owner_discarded_black_basic_to_discard" => {
                "Cards-moved trigger filter: the owner discarded at least one effective black basic card from hand/equipment into the discard pile. The full moved-card batch is preserved for the effect. Requires Rule VM ABI 51."
            }
            "owner_alive_other_judge_phase_in_range_has_judge_card" => {
                "Judgment-phase trigger filter: another living player is within the living owner's attack range and has a discardable judgment card. Requires Rule VM ABI 51."
            }
            "ignore_armor_for_skill_card_policy" => {
                "Modifier target: ignores armor only while resolving a card carrying the ignore-armor bit in this skill instance's exact-card policy state. Requires `override 1` and Rule VM ABI 49."
            }
            "card_use_unrespondable_for_skill_card_policy" => {
                "Modifier target: prevents Shan and Wuxie responses only for a card carrying the unrespondable bit in this skill instance's exact-card policy state. Requires `override 1` and Rule VM ABI 49."
            }
            "card_distance_unlimited_for_skill_card_policy" => {
                "Modifier target: ignores ordinary target distance only for a card carrying the distance-unlimited bit in this skill instance's exact-card policy state. Requires `override 1` and Rule VM ABI 149."
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
            "reveal_top_gain_non_basic_or_swap_basic" => {
                "Effect: publicly reveals the draw-pile top card. A non-basic card is gained immediately; a basic card may be exchanged by placing one hand card on top of the draw pile, otherwise it is discarded.\n\n```\neffect reveal_top_gain_non_basic_or_swap_basic(self);\n```"
            }
            "prevent_current_damage" => {
                "Effect: prevents the damage event bound to the current `damage_before` trigger.\n\n```\neffect prevent_current_damage();\n```"
            }
            "choose_prevent_damage_or_lose_max_hp_increase_damage" => {
                "Effect: forces the damage source to choose between preventing the damage bound to the current `damage_before` trigger, or losing one maximum HP and increasing that same damage by the requested amount. Requires Rule VM ABI 64.\n\n```\neffect choose_prevent_damage_or_lose_max_hp_increase_damage(self, 2);\n```"
            }
            "choose_draw_or_increase_current_card_damage_or_both_mark" => {
                "Effect: during `card_used`, lets the owner choose to draw cards, increase the suspended damaging card's base damage, or perform both and then add a mark. A non-damaging multi-target card keeps the damage branch as a no-op. Requires Rule VM ABI 114.\n\n```\neffect choose_draw_or_increase_current_card_damage_or_both_mark(self, event_card, 2, 1, mark_def, 1);\n```"
            }
            "randomize_current_hand_compare_opponent" => {
                "Effect: during `hand_compare_started`, marks the other comparison participant's card selection for authoritative deterministic RNG. The comparison context and skill ownership are revalidated before selection. Requires Rule VM ABI 65.\n\n```\neffect randomize_current_hand_compare_opponent(self);\n```"
            }
            "gain_highest_rank_sha_from_event_cards" => {
                "Effect: during `hand_compare_finished`, gains every tied highest-rank effective Sha among the captured comparison cards that remains in the discard pile. Requires Rule VM ABI 65.\n\n```\neffect gain_highest_rank_sha_from_event_cards(self);\n```"
            }
            "gain_hand_compare_result_card" => {
                "Effect: during `hand_compare_finished`, gains the opponent's comparison card on a strict win, otherwise regains the owner's comparison card, if it remains in the discard pile. Requires Rule VM ABI 66.\n\n```\neffect gain_hand_compare_result_card(self);\n```"
            }
            "choose_add_or_remove_current_card_target" => {
                "Effect: during `card_used`, consumes the originating one-shot helper after optionally adding one legal target to or removing one target from the current basic/trick card. Requires Rule VM ABI 67.\n\n```\neffect choose_add_or_remove_current_card_target(self, event_card);\n```"
            }
            "choose_hand_compare" => {
                "Effect: chooses another living player with hand cards, then lets both participants choose one hand card for a comparison. Strict victory grants one helper; tie/loss grants the other. Requires Rule VM ABI 68.\n\n```\neffect choose_hand_compare(self, win_skill_def, lose_skill_def);\n```"
            }
            "resolve_lieren" => {
                "Effect: opens the generic hand-comparison card choices with a fixed compare target (the damaged player), then grants the win/lose helper skills. ABI 531.\n\n```\neffect resolve_lieren(self, event_player, win_skill_def, lose_skill_def);\n```"
            }
            "resolve_tuntian" => {
                "Effect: judges; if the judgment card is not a heart, moves it into the originating skill's public pile (田). ABI 533.\n\n```\neffect resolve_tuntian(self);\n```"
            }
            "resolve_zaoxian" => {
                "Effect: awakens when the 屯田 pile (田) has at least three cards — the owner loses 1 max HP and gains the supplied helper skill (急袭). ABI 534.\n\n```\neffect resolve_zaoxian(self, jixi_skill_def, tuntian_skill_def);\n```"
            }
            "resolve_tiaoxin" => {
                "Effect: once per play phase, choose another player whose attack range contains the owner and who has hand/equipment cards; that player either uses a Sha on the owner or the owner discards one of their cards. ABI 537.\n\n```\neffect resolve_tiaoxin(self);\n```"
            }
            "resolve_zhiji" => {
                "Effect: awakens at the owner's prepare phase with no hand cards — draws two, recovers 1 HP, loses 1 max HP and gains the supplied 观星 skill. ABI 538.\n\n```\neffect resolve_zhiji(self, guanxing_skill_def);\n```"
            }
            "choose_other_damage" => {
                "Effect: forces the player to choose one other living player, then deals the supplied amount of normal damage from the player to that target. Requires Rule VM ABI 69.\n\n```\neffect choose_other_damage(self, 1);\n```"
            }
            "choose_bound_source_attack_range_damage" => {
                "Effect: forces the skill owner to choose one living player other than the skill-bound source and inside that source's current attack range, then deals normal damage from the bound source. The skill binding, participants, range, and submitted target are revalidated at the choice barrier. Requires Rule VM ABI 121.\n\n```\neffect choose_bound_source_attack_range_damage(self, event_source, 1);\n```"
            }
            "choose_other_discard_target_card" => {
                "Effect: forces the player to choose another living player with at least one discardable card in the requested scope, then reuses the authoritative target-card discard choice. Scopes are `equip`, `hand_equip`, and `hand_equip_judge`. Requires Rule VM ABI 70.\n\n```\neffect choose_other_discard_target_card(self, hand_equip);\n```"
            }
            "judge_prevent_damage_lose_max_hp_on_nonheart" => {
                "Effect: performs a judgment for the skill owner. On a non-heart result, prevents the damage bound to the current `damage_before` trigger and reduces the target's maximum HP by one. Requires Rule VM ABI 37.\n\n```\neffect judge_prevent_damage_lose_max_hp_on_nonheart(self, event_player);\n```"
            }
            "choose_gain_top_cards_by_suit" => {
                "Effect: privately shows up to `count` draw-pile top cards to `player`, lets that player cancel or reveal and gain any nonempty subset of cards whose effective suit matches, and leaves every unselected card on top in original relative order. Requires Rule VM ABI 38.\n\n```\neffect choose_gain_top_cards_by_suit(self, 3, heart);\n```"
            }
            "choose_other_draw_by_hand_count" => {
                "Effect: opens `times` independently cancellable choices for `player`. Each time, that player may choose a different living player; the target draws `empty_count` cards while empty-handed, otherwise `normal_count`. The target and hand state are revalidated at every barrier, and draw-trigger provenance stays bound to the originating skill. Requires Rule VM ABI 39.\n\n```\neffect choose_other_draw_by_hand_count(self, 1, 2, event_amount);\n```"
            }
            "judge_cancel_current_sha_target_on_red" => {
                "Effect: performs a judgment for the specified player and, on a red result, excludes the current target from the currently resolving Sha. Available only in `card_targeted` triggers.\n\n```\neffect judge_cancel_current_sha_target_on_red(self, event_card, event_player);\n```"
            }
            "make_current_sha_target_unrespondable" => {
                "Effect: disables the response window for exactly one target of the currently resolving physical Sha. The source/card/target tuple is revalidated against the saved continuation; no persistent modifier is installed. Available only in `card_targeted` triggers and requires Rule VM ABI 35.\n\n```\neffect make_current_sha_target_unrespondable(self, event_card, event_player);\n```"
            }
            "judge_make_current_sha_target_unrespondable_on_red" => {
                "Effect: performs a judgment for the Sha user and, on a red result, disables the response window for exactly the current target of the currently resolving physical Sha. Available only in `card_targeted` triggers and requires Rule VM ABI 177.\n\n```\neffect judge_make_current_sha_target_unrespondable_on_red(self, event_card, event_player);\n```"
            }
            "resolve_retieji" => {
                "Effect: performs 界铁骑's neutral judgment, suppresses the current Sha target's non-locked skills until turn end, then lets that target give one hand/equipment card of the judgment suit to the skill owner. Declining or having no eligible card makes only this target unable to respond and increases only this target's Sha damage by one. Requires Rule VM ABI 316.\n\n```\neffect resolve_retieji(self, event_card, event_player);\n```"
            }
            "resolve_new_reluoyi" => {
                "Effect: reveals the draw-pile top three during an unfixed Draw phase, then asks the player whether to replace the normal draw. Accepting gains every basic card, Duel, and weapon, discards the rest, and grants the configured helper; declining discards all three and keeps the normal draw. Requires Rule VM ABI 317.\n\n```\neffect resolve_new_reluoyi(self, buff_skill_def);\n```"
            }
            "resolve_remoshi" => {
                "Effect: action 0 moves the current gainable physical Sha into the damaged player's public 没矢 pile and grants the configured presentation helper; action 1 gives that event player's complete pile to the skill owner. Requires Rule VM ABI 318.\n\n```\neffect resolve_remoshi(self, event_player, event_card, helper_skill_def, action);\n```"
            }
            "resolve_xinyaoming" => {
                "Effect: lets the owner choose a legal target and one unused 邀名 branch this turn: another player draws one; discard one hand card from another player; or make any target discard one or two hand/equipment cards and draw the same count. Requires Rule VM ABI 319.\n\n```\neffect resolve_xinyaoming(self);\n```"
            }
            "resolve_rehuaiyi" => {
                "Active-only effect: reveals the owner's hand. A uniform color draws one and permits a second activation this Play phase; mixed colors force one fully discardable color to be discarded, then optionally gain one hand/equipment card from up to that many other players. Gaining at least two cards loses 1 HP. Requires `active play_phase uses_per_play_phase 2 hand_nonempty` and Rule VM ABI 320.\n\n```\neffect resolve_rehuaiyi(self);\n```"
            }
            "resolve_reanguo" => {
                "Active-only effect: runs the selected other player's minimum-hand draw, wounded minimum-HP recovery, and minimum-equipment random-use branches in order; the owner then runs only branches not actually completed. If all three complete, the owner may recast any nonempty hand/equipment subset. Requires `active play_phase uses_per_turn 1 targets 1 1 other` and Rule VM ABI 321.\n\n```\neffect resolve_reanguo(self);\n```"
            }
            "resolve_beizhu" => {
                "Active-only effect: privately reveals the selected other player's hand. With no effective Sha, discards one of that player's hand/equipment cards and may give them the first physical Sha in the draw pile. Otherwise that player sequentially uses the snapshotted Sha on the owner, ignoring distance and ordinary Sha count; each point of direct resulting damage makes the owner draw one. Requires `active play_phase uses_per_turn 1 targets 1 1 other hand_nonempty` and Rule VM ABI 324.\n\n```\neffect resolve_beizhu(self, event_source);\n```"
            }
            "resolve_mbshanjia" => {
                "Active-only effect: draws three, then discards `max(0, 3-lost_equipment_count)` current hand/equipment cards. Discarding no basic card offers an optional virtual Sha that ignores ordinary use count; discarding no trick card grants the configured all-card distance helper until Play-phase end. Requires `active play_phase uses_per_turn 1` and Rule VM ABI 326.\n\n```\neffect resolve_mbshanjia(self, mark_count(self, main_skill_def), distance_skill_def);\n```"
            }
            "resolve_daoji" => {
                "Active-only effect: lets the owner gain one currently gainable equipment card from the selected other player, forcibly uses it when still legal, then deals 1 normal damage after the use attempt when the physical card is a weapon. Requires `active play_phase uses_per_turn 1 targets 1 1 other has_gainable_equip select_he 1 1 card_nonbasic` and Rule VM ABI 327.\n\n```\neffect resolve_daoji(self, event_source);\n```"
            }
            "resolve_tw_daoji" => {
                "Active-only effect: gains one hand/equipment card from the selected in-range player. A basic card still in hand draws one; an equipment card still in hand is used when legal, then the original owner takes 1 normal damage. Requires `active play_phase uses_per_turn 1 targets 1 1 other has_gainable_hand_equip in_attack_range select_he 1 1 card_nonbasic` and Rule VM ABI 355.\n\n```\neffect resolve_tw_daoji(self, event_source);\n```"
            }
            "give_all_hand_then_choose_use_one_or_damage" => {
                "Active-only effect: gives every hand card to the selected player. They may use one gifted card; the other gifted cards return to the owner. If they decline or cannot use one, they take 1 normal damage from the owner. Requires `active play_phase uses_per_turn 1 hand_nonempty targets 1 1 other` without card selection and Rule VM ABI 359.\n\n```\neffect give_all_hand_then_choose_use_one_or_damage(self, event_source);\n```"
            }
            "choose_category_then_target_give_matching_or_sha_unrespondable" => {
                "Trigger-only current-Sha effect: the skill owner chooses basic, trick, or equipment. The current Sha target may give one matching hand/equipment card; declining or having none makes that Sha unrespondable for the target. Requires a current Sha target context and Rule VM ABI 360.\n\n```\neffect choose_category_then_target_give_matching_or_sha_unrespondable(self, event_card, event_player);\n```"
            }
            "choose_category_then_target_give_matching_or_damage_plus_one" => {
                "Damage-before effect: the damage source chooses basic, trick, or equipment. The damage target may give one matching hand/equipment card; declining or having none increases the bound damage by 1. Requires Rule VM ABI 498.\n\n```\neffect choose_category_then_target_give_matching_or_damage_plus_one(self, event_player);\n```"
            }
            "choose_other_excluding_source_random_gain_hand_equip_then_use_if_equip" => {
                "Damage-end effect: the owner chooses a living role other than self and the damage source, deterministically gains one currently gainable hand/equipment card at random, and uses it when it is equipment and remains legal. Requires Rule VM ABI 499.\n\n```\neffect choose_other_excluding_source_random_gain_hand_equip_then_use_if_equip(self, event_source);\n```"
            }
            "resolve_mobiledanshou" => {
                "End-phase effect: count cards the phase player used targeting the owner this turn. At zero the owner draws one; otherwise the owner may discard exactly that many hand/equipment cards, then deals 1 damage to the phase player. Requires Rule VM ABI 500.\n\n```\neffect resolve_mobiledanshou(self, event_player);\n```"
            }
            "resolve_jsrgchiying" => {
                "Active effect: snapshot the living roles in the selected target's attack range except the owner, then make each discard exactly one hand/equipment card in seat order when possible. If the discarded basic-card count does not exceed the target's current HP, the target gains those basic cards still in the discard pile. Requires Rule VM ABI 501.\n\n```\neffect resolve_jsrgchiying(self, event_source);\n```"
            }
            "resolve_mbqingdao" => {
                "Post-card effect for 清蹈: after a foreign damage card targeting the owner fully resolves, choose the damaged or undamaged branch and reuse the normal gain, discard, and physical-card play pipelines. Requires Rule VM ABI 503.\n\n```\neffect resolve_mbqingdao(self, event_source, event_card, 1);\n```"
            }
            "resolve_mbxiugeng" => {
                "修耕 effect: record one or two players' current hand counts, then at each recorded player's next Draw phase apply the lower/equal draw and higher/equal permanent hand-limit branches. Requires Rule VM ABI 503."
            }
            "resolve_mbchenshe" => {
                "陈赦 effect: sequentially discard from owner, dying player, and damage source when possible; three cards of one suit recover the dying player to maximum HP and remove this skill. Requires Rule VM ABI 503."
            }
            "resolve_mbquanchong" => {
                "权宠 effect: discard all owner hand/equipment cards, queue an extra turn, and arm the supplied phase-begin HP-loss helper unless the owner is the unique current-HP leader. Requires Rule VM ABI 504."
            }
            "resolve_mbrenxing" => {
                "任行 effect: consume one of two round uses, then choose to draw for the owner/current turn player or discard one card from a role that has not used or responded with Sha this turn. Requires Rule VM ABI 504."
            }
            "resolve_mbchizhang" => {
                "鸱张 effect: after a hand damage card selects its first target, optionally discard hand cards and lock matching response colors for that exact card use. Requires Rule VM ABI 505."
            }
            "resolve_mbduanyang" => {
                "断鞅 effect: store a random ownerless physical Sha lost from hand, use stored Sha at phase ends ignoring distance, and resolve its optional target recast plus owner draw. Requires Rule VM ABI 505."
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
            "resolve_mbrunwei" => {
                "润微 effect: choose one to five draw-pile cards to reveal, choose a color and recipient, arm the qualifying-loss threshold, enable the second activation, and clean tracked self-gains at Play-phase end. Requires Rule VM ABI 506."
            }
            "resolve_mbshuanghuai" => {
                "霜怀 effect: choose to prevent the bound damage or let its target gain a Tao from discard, then apply the repeated/different previous-target result. Requires Rule VM ABI 506."
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
            "choose_hand_compare_grant_target_on_win" => {
                "Effect: choose one other player with hand cards for a hand comparison (both pick a card); on the chooser's strict victory the **target** is granted win_grant_def, on tie or loss nothing happens. Requires Rule VM ABI 678."
            }
            "choose_other_give_then_recover" => {
                "Effect: choose one other player with hand cards — they give one hand card to the chooser, then recover 1 HP themselves. Requires Rule VM ABI 682."
            }
            "guanxing_top_by_falu_marks" => {
                "Effect: guanxing with N = number of non-zero falu marks among the four declared defs. Requires Rule VM ABI 683."
            }
            "guanxing_top" => {
                "Effect: watch the top `count` cards of the draw pile and re-order them one by one — each round pick top-sequence or bottom-sequence for the current card. Approximation: free ordering within each sequence is not modeled. Requires Rule VM ABI 681."
            }
            "choose_hand_compare_between" => {
                "Effect: third-party hand comparison — `first` picks an opponent (excluding the skill owner), both stash one hand card, and on a non-tie the winner uses a distance-free virtual Sha against the loser (fixed to the loser). Requires Rule VM ABI 680."
            }
            "give_selected_then_recover_self" => {
                "Effect: give the active-selected hand/equip cards to the target, then the chooser recovers `amount` HP. Requires Rule VM ABI 679."
            }
            "resolve_mbxiongtu" => {
                "凶图 effect: reveal a selected target hand card, discard the missing-suit count or that revealed card, then arm the next external-damage draw and second activation. Requires Rule VM ABI 507."
            }
            "owner_source_damage_mbxiongtu_armed" => {
                "Damage filter for an armed 凶图 reward, excluding damage caused by 凶图 itself. Requires Rule VM ABI 507."
            }
            "owner_phase_play_end_mbxiongtu_state" => {
                "Play-phase-end filter for clearing 凶图's transient activation state. Requires Rule VM ABI 507."
            }
            "resolve_mbganggeng" => {
                "刚鲠 effect: give the selected hand cards to another player, then at turn end draw if that player ties for maximum hand size or discard one of their area cards otherwise. Requires Rule VM ABI 508."
            }
            "resolve_mbsijian" => {
                "死谏 effect: choose its discard, current-player draw, or combined branch; combined uses accumulate the later HP-loss cost. Requires Rule VM ABI 508."
            }
            "owner_phase_end_mbganggeng_target_alive" => {
                "Phase-end filter for resolving the living target remembered by 刚鲠. Requires Rule VM ABI 508."
            }
            "owner_from_hand_empty_or_dying_mbsijian_twice_per_turn" => {
                "Cards-moved or dying filter for 死谏, shared across both event sources and limited to twice per turn. Requires Rule VM ABI 508."
            }
            "resolve_mbjianji" => {
                "间计 effect: let two participants secretly compare using either their own hand card or the owner's shared selected card, then use a no-limit Sha from winner to loser and damage shared-Sha choosers. Requires Rule VM ABI 509."
            }
            "resolve_mbyuanmo" => {
                "远谟 effect: move one legal equipment or judgment card, compare its former owner's attack-range population before and after, then optionally draw the capped loss. Requires Rule VM ABI 509."
            }
            "owner_is_target_can_move_field_card_mbyuanmo_twice_per_turn" => {
                "Prepare or post-damage filter for 远谟 when any legal field-card move exists, shared limit twice per turn. Requires Rule VM ABI 509."
            }
            "resolve_mbfeijing" => {
                "飞径 effect: choose a seat path between a single-target Sha's endpoints, collect one secret hand discard per eligible role, then add legal targets matching the chosen color. Requires Rule VM ABI 510."
            }
            "resolve_mbxiaoge" => {
                "骁戈 effect: prevent damage to a 飞径-added target, recover and reclaim its discard, or follow the originally single-target Sha with a legal Duel. Requires Rule VM ABI 510."
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
            "resolve_mbxuye" => {
                "蓄业 effect: draw two for the selected role, then if they newly tie for maximum hand size put one of their HEJ cards on top of the draw pile. Requires Rule VM ABI 514."
            }
            "resolve_mbkuangxiang" => {
                "狂想 effect: swap all hands, track received cards until the owner's next Play phase, and invoke 蓄业 when either tracked set is exhausted. Requires Rule VM ABI 514."
            }
            "resolve_potchengpu" => {
                "势程普 effect: 督作 gives a draw-pile fire Sha after a gain; 避憾 discards one eligible role's hand down to HP and reduces the bound Sha damage. Requires Rule VM ABI 515."
            }
            "resolve_potjugu" => {
                "势臧洪拒孤 effect: draw two, then discard min(discardable hand/equipment cards, this skill's fires this turn). Requires Rule VM ABI 516."
            }
            "resolve_potsifeng" => {
                "势曹真伺锋 effect: distribute three top cards to other players' expansion piles, consume one after each card use with a color-mismatch discard, then gain all or discard all and damage at phase end. Requires Rule VM ABI 517."
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
            "choose_lose_hp_use_virtual_sha_by_basic_variants" => {
                "Phase-end trigger effect: choose legal targets for an ignore-distance virtual Sha, then lose 1 HP and use it without ordinary use-count limits. Its base damage gains 1 when the user used at least two distinct basic-card name/nature variants this turn. Requires `trigger phase_end owner_is_target` and Rule VM ABI 361.\n\n```\neffect choose_lose_hp_use_virtual_sha_by_basic_variants(self);\n```"
            }
            "resolve_twfupan" => {
                "Damage-trigger effect: draw `event_amount`, then if possible force the owner to give one hand/equipment card to an eligible other player. The first gift to each target draws two; a later gift may deal 1 damage and permanently makes that target ineligible. Requires Rule VM ABI 362.\n\n```\neffect resolve_twfupan(self, event_amount);\n```"
            }
            "resolve_twjiaohua" => {
                "Cards-drawn trigger effect: the owner or a tied-lowest-HP recipient gains the topmost draw-pile, then discard-pile card whose basic/trick/equipment category was neither in the triggering draw nor previously gained through TW Jiaohua by that recipient this turn. Requires Rule VM ABI 363.\n\n```\neffect resolve_twjiaohua(self, event_player);\n```"
            }
            "resolve_twkujian" => {
                "TW Kujian effect: action 0 gives and tracks one to three selected hand cards; action 1 removes matching tracked event cards and draws that many for both players; action 2 instead makes both players discard once per matching card. Requires Rule VM ABI 364.\n\n```\neffect resolve_twkujian(self, event_source, 0);\n```"
            }
            "resolve_twruilian" => {
                "TW Ruilian effect: action 0 optionally selects this round's tracked role; action 1 records that role's own-turn discarded count and categories; action 2 clears tracking and optionally gains up to two current discard-pile cards of one recorded category. Requires Rule VM ABI 365.\n\n```\neffect resolve_twruilian(self, event_player, 0);\n```"
            }
            "equip_weapon_from_draw_pile" => {
                "Effect: if `player` has no physical weapon equipped, deterministically selects a random weapon from the shared draw pile and equips it. Requires Rule VM ABI 366.\n\n```\neffect equip_weapon_from_draw_pile(self);\n```"
            }
            "resolve_twjianwei" => {
                "TW Jianwei effect: stage 0 lets the owner compare with an in-range role; stage 1 lets another prepare-phase player opt to compare with the owner. The shared comparison pipeline applies the owner's weapon attack range to their rank and settles region-card or weapon gains. Requires Rule VM ABI 367.\n\n```\neffect resolve_twjianwei(self, event_player, 0);\n```"
            }
            "resolve_twtanfeng" => {
                "TW Tanfeng effect: during the owner's Prepare phase, optionally discards one hand/equipment/judgment card from another player. That player may convert one hand/equipment card into a normal-distance Sha that must include the owner; otherwise they take 1 fire damage and, if alive, choose one future Judge/Draw/Play/Discard phase for the owner to skip. Requires Rule VM ABI 368.\n\n```\neffect resolve_twtanfeng(self);\n```"
            }
            "resolve_twzhibian" => {
                "TW Zhibian effect: at the start of the owner's Play phase, optionally compares with another player. A non-win loses 1 HP; a win may move one legal card into the corresponding owner zone, recover 1 HP, or discard a non-basic hand/equipment card to perform every currently available branch. Requires Rule VM ABI 369.\n\n```\neffect resolve_twzhibian(self);\n```"
            }
            "resolve_twyuyan" => {
                "TW Yuyan effect: in the current physical Sha target context, the higher-HP source may give the owner one hand/equipment card whose rank exceeds the Sha's rank (or one non-basic card when the Sha has no rank). Declining or having no eligible card removes only the owner from that Sha. Requires Rule VM ABI 370.\n\n```\neffect resolve_twyuyan(self, event_source, event_card);\n```"
            }
            "owner_target_physical_sha_source_hp_gt_owner" => {
                "Card-targeted filter: the living owner is targeted by another living player's physical Sha and that source currently has strictly more HP. Requires Rule VM ABI 370."
            }
            "resolve_twguoyi" => {
                "TW Guoyi effect: the current Sha/ordinary-trick target chooses a turn-long hand-card use/response lock or a forced discard of lost-HP plus one cards. When both trigger conditions hold, or both branches have been chosen this turn, the same logical card is replayed on every original target only after the source use fully settles. Requires Rule VM ABI 371.\n\n```\neffect resolve_twguoyi(self, event_player, event_card, helper_skill_def);\n```"
            }
            "owner_source_other_target_sha_or_ordinary_trick_twguoyi_condition" => {
                "Card-targeted filter: the owner used a Sha or ordinary trick on another living player, that target ties for maximum HP/hand or the owner's hand is at most lost HP plus one, and the card was not replayed by the same skill. Requires Rule VM ABI 371."
            }
            "resolve_twchuhai" => {
                "TW Chuhai effect: action 0 completes the mission by disabling the judge zone and collecting one hand/equipment card from each other living player; action 1 discards exactly one card from the current batch gained from another player's personal zone. Requires Rule VM ABI 372.\n\n```\neffect resolve_twchuhai(self, action);\n```"
            }
            "owner_alive_phase_end_caused_two_distinct_dying" => {
                "Phase-end filter: the living owner caused at least two distinct other players to enter dying during the current turn. Requires Rule VM ABI 372."
            }
            "owner_gained_event_cards_from_other_personal_zone" => {
                "Cards-moved filter: the owner gained at least one event card from another player's hand, equipment, or judge zone and at least one such card remains in hand. Requires Rule VM ABI 372."
            }
            "resolve_twyizhu" => {
                "TW Yizhu effect: action 0 draws and plants up to two hand/equipment cards near the draw-pile top, action 1 optionally adds or replaces the single target of another player's use of a tracked card then draws one, and action 2 clears tracked cards entering discard. Requires Rule VM ABI 373.\n\n```\neffect resolve_twyizhu(self, source, card, action);\n```"
            }
            "owner_alive_other_single_target_used_tracked_twyizhu" => {
                "Card-targeted filter: another player used a tracked TW Yizhu physical card with exactly one declared target. Requires Rule VM ABI 373."
            }
            "owner_tracked_twyizhu_entered_discard" => {
                "Cards-moved filter: at least one physical card tracked by TW Yizhu entered the discard pile. Requires Rule VM ABI 373."
            }
            "resolve_twluanchou" => {
                "TW Luanchou/Gonghuan effect: action 0 removes the shared helper from every player and grants it to the active invocation's two selected living targets; action 1 prevents the current damage and recreates it on the helper owner with the same source, amount, nature, card, kind, and flags. Requires Rule VM ABI 374.\n\n```\neffect resolve_twluanchou(self, helper_skill_def, action);\n```"
            }
            "owner_alive_other_twgonghuan_damage_before_hp_lte_owner_unused" => {
                "Damage-before filter: another living holder of the same Gonghuan helper has HP no greater than the owner, the owner has not redirected damage this turn, and the segment was not itself redirected. Requires Rule VM ABI 374."
            }
            "resolve_twxiawei" => {
                "TW Xiawei/Qiongji effect: initializes and clears a playable Wei skill pile, prompts the one-through-four Wangxing choice and stores top cards, increases empty-pile incoming damage, or settles the exact discard/lose-max-HP cost. Requires Rule VM ABI 375.\n\n```\neffect resolve_twxiawei(self, wangxing_skill_def, action);\n```"
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
            "resolve_twbianfuren" => {
                "TW Bianfuren effect: action 0 prevents minimum-HP damage to another player and/or schedules Wanwei's end-phase effect, action 1 gains the draw-pile top card and uses the revealed bottom card when legal, and action 2 partitions selected hand/equipment cards between draw-pile top and bottom before applying Yuejian's tiered rewards. Requires Rule VM ABI 376.\n\n```\neffect resolve_twbianfuren(self, event_player, helper_skill_def, action);\n```"
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
            "resolve_twfeiyi" => {
                "TW Feiyi effect: action 0 gains or creates a suit-distinct Tiaojiyanmei, action 1 chooses one canonical zhinang kind then gains it when present and draws, and action 2 transfers one triggering non-use discard card to another living player. Requires Rule VM ABI 377.\n\n```\neffect resolve_twfeiyi(self, action);\n```"
            }
            "owner_alive_phase_end_used_card_no_damage" => {
                "Owner End-phase filter: at least one card was used and the owner dealt no damage this turn. Requires Rule VM ABI 377."
            }
            "owner_cards_entered_discard_not_use_unused" => {
                "Cards-moved filter: the owner's personal cards entered discard for a cause other than use/response and the skill has not fired this turn. Requires Rule VM ABI 377."
            }
            "resolve_twchenzhen" => {
                "TW Chenzhen effect: action 0 declares a basic/trick kind, pays or consumes Muyue's free-use helper, grants the named shared-pile card and restores the helper on a matching discard; actions 1-3 set up Chayi, force its next-use discard, and switch the delayed branch when hand count changes. Requires Rule VM ABI 378.\n\n```\neffect resolve_twchenzhen(self, event_player, helper_skill_def, record_skill_def, action);\n```"
            }
            "resolve_twqianju" => {
                "TW Qianju effect: equip the first compatible equipment card found in the shared draw/discard piles. Requires Rule VM ABI 379.\n\n```\neffect resolve_twqianju(self, event_player);\n```"
            }
            "resolve_twqingxi" => {
                "TW Qingxi effect: the first Sha target chooses draw-and-unrespondable or mutual equipment discard with +1 target damage. Requires Rule VM ABI 379.\n\n```\neffect resolve_twqingxi(self, event_player, event_card);\n```"
            }
            "resolve_twzaoli" => {
                "TW Zaoli effect: discard all equipment cards plus chosen non-equipment hand cards, draw the same count, refill each discarded equipment slot from shared piles, and lose HP after more than two refills. Requires Rule VM ABI 380.\n\n```\neffect resolve_twzaoli(self);\n```"
            }
            "resolve_twzhengjian" => {
                "TW Zhengjian effect: choose the initial no-nonbasic/no-gain branch, collect tribute at failed Play phases, optionally switch, and awaken Zhongchi after distinct contributors reach half the player count. Requires Rule VM ABI 381.\n\n```\neffect resolve_twzhengjian(self, event_player, helper_skill_def, action);\n```"
            }
            "suppress_non_locked_skills_until_turn_end" => {
                "Effect: suppresses the specified living player's non-locked skills until the current turn ends. Requires Rule VM ABI 356.\n\n```\neffect suppress_non_locked_skills_until_turn_end(event_source);\n```"
            }
            "choose_recover_target" => {
                "Effect: lets `chooser` cancel or recover a specified living wounded `target` by `amount`. Requires Rule VM ABI 356.\n\n```\neffect choose_recover_target(self, event_source, 1);\n```"
            }
            "resolve_jiaohua" => {
                "Active-only effect: chooses one basic/trick/equipment category not yet recorded in the persistent three-category cycle, then lets any living player gain the topmost matching draw-pile card when one exists. The category is recorded even when no card exists, and completing all three clears the cycle. Requires `active play_phase uses_per_play_phase 2` and Rule VM ABI 328.\n\n```\neffect resolve_jiaohua(self);\n```"
            }
            "resolve_luanqun" => {
                "Active-only effect: every living player with hand cards privately selects one, then all selections are revealed together. The owner may gain matching-color selections (up to four in identity-standard, otherwise two); mismatching players receive the configured first-Sha target restriction and turn-long unrespondable helpers. Requires `active play_phase uses_per_turn 1 hand_nonempty` and Rule VM ABI 329.\n\n```\neffect resolve_luanqun(self, effect_skill_def, direct_hit_skill_def);\n```"
            }
            "choose_discard_basic_then_target_discard_equip_or_damage" => {
                "Effect: lets `player` cancel or discard exactly one effective basic hand card. After its movement triggers finish, `target` may discard one effective equipment card from hand/equipment; declining or having none causes 1 normal damage from `player`. Requires Rule VM ABI 178.\n\n```\neffect choose_discard_basic_then_target_discard_equip_or_damage(self, event_player);\n```"
            }
            "choose_current_sha_target_discard_redirect_to_owner" => {
                "Effect: lets the current Sha target cancel or discard one current hand/equipment card. After discard triggers finish, that exact target slot is replaced by the living skill owner; ordinary Sha-user distance is bypassed, while card target policies and the target-to-owner attack-range condition are revalidated. Requires Rule VM ABI 180.\n\n```\neffect choose_current_sha_target_discard_redirect_to_owner(self, event_source, event_card, event_player);\n```"
            }
            "choose_target_gain_bottom_card" => {
                "Active-only effect: moves the draw-pile bottom card into the public processing zone, waits for movement triggers, then forces `player` to choose any living recipient. The transfer emits an attributed `cards_given` event. Requires Rule VM ABI 181.\n\n```\neffect choose_target_gain_bottom_card(self);\n```"
            }
            "judge_black_gain_repeat" => {
                "Effect: performs a judgment for `player`. A red result stays in discard and ends the effect; a black result is gained if still in discard, then `player` may repeat after movement triggers finish. Requires Rule VM ABI 182.\n\n```\neffect judge_black_gain_repeat(self);\n```"
            }
            "judge_black_gain_repeat_protect_hand_limit" => {
                "Effect: performs the same black-result judgment loop, but each black card actually gained is excluded from this player's ordinary hand-limit count and discard-phase choices for the current turn. Requires Rule VM ABI 309.\n\n```\neffect judge_black_gain_repeat_protect_hand_limit(self);\n```"
            }
            "judge_same_color_gain_repeat" => {
                "Effect: performs a judgment for `player`, records its effective color after rewrites, and gains it if still in discard. After movement triggers finish, `player` may repeat; later matching-color results are gained and the first different color ends the effect. Requires Rule VM ABI 307.\n\n```\neffect judge_same_color_gain_repeat(self);\n```"
            }
            "use_virtual_juedou_second_on_first" => {
                "Active-only effect: materializes a suitless virtual Duel for the second selected target to use on the first. It emits ordinary card-use and Duel events but deliberately opens no Wuxie window. Requires `targets 2 2 other male second_can_use_juedou_on_first` and Rule VM ABI 183.\n\n```\neffect use_virtual_juedou_second_on_first();\n```"
            }
            "choose_draw_or_recover" => {
                "Effect: lets the specified player choose between drawing and recovering; at full HP, draws immediately. The final argument controls cancellation.\n\n```\neffect choose_draw_or_recover(event_source, 1, 1, 0);\n```"
            }
            "choose_use_virtual_sha" => {
                "Effect: materializes a suitless virtual Sha, lets the player choose legal targets, and resolves it through the ordinary use/response/damage pipeline. The policy is `normal_distance` or `ignore_distance`.\n\n```\neffect choose_use_virtual_sha(self, ignore_distance);\n```"
            }
            "adjust_hand_to" => {
                "Effect: adjusts a player's hand to an exact count. It draws when below the target and opens a forced exact hand-card discard choice when above it. Requires Rule VM ABI 9.\n\n```\neffect adjust_hand_to(self, 4);\n```"
            }
            "gain_card_by_kind" => {
                "Effect: gains the topmost card of one rule kind, searching the draw pile before the discard pile. Does nothing when neither pile contains one. Requires Rule VM ABI 10.\n\n```\neffect gain_card_by_kind(self, tiesuo);\n```"
            }
            "gain_discard_card_by_kind" => {
                "Effect: gains the topmost card of one rule kind from the discard pile only. Does nothing when the discard pile has no matching card. Requires Rule VM ABI 74.\n\n```\neffect gain_discard_card_by_kind(self, sha);\n```"
            }
            "reveal_uniform_hand_choose_damage" => {
                "Effect: publicly reveals the owner's current hand. If every card has one uniform effective color, the owner must choose any living player, including themselves, and deals the supplied normal damage to that target. Requires Rule VM ABI 76.\n\n```\neffect reveal_uniform_hand_choose_damage(self, 1);\n```"
            }
            "choose_gain_discard_card_by_kind" => {
                "Effect: forces the player to choose and gain one matching card from the discard pile. Does nothing when the discard pile has no matching card. Requires Rule VM ABI 77.\n\n```\neffect choose_gain_discard_card_by_kind(self, sha);\n```"
            }
            "draw_for_lifetime_card_action_divisors" => {
                "Effect: reads the player's match-lifetime total of cards used or played as responses and draws one card for each supplied positive divisor that divides the total. Requires Rule VM ABI 78 and the `owner_lifetime_card_action` trigger filter.\n\n```\neffect draw_for_lifetime_card_action_divisors(self, 3, 5, 8);\n```"
            }
            "gain_draw_cards_for_lifetime_card_action_divisors" => {
                "Effect: reads the player's match-lifetime card-action total. For each matching divisor, gains the topmost draw-pile card in the paired group: `sha_or_shan`, `tao_or_jiu`, or `juedou_or_wuzhong`. The discard pile is not searched. Requires Rule VM ABI 198.\n\n```\neffect gain_draw_cards_for_lifetime_card_action_divisors(self, 3, sha_or_shan, 5, tao_or_jiu, 8, juedou_or_wuzhong);\n```"
            }
            "lose_hp_gain_discard_card_for_opening_extra_play" => {
                "Effect: only during the turn-opening extra Play phase created by this skill, loses 1 HP and then gains the topmost matching discard-pile card. The action is forced while the supplied skill definition is still owned and optional after it has been removed. Requires Rule VM ABI 200.\n\n```\neffect lose_hp_gain_discard_card_for_opening_extra_play(self, sha, mandatory_skill_def);\n```"
            }
            "choose_gain_own_lost_discard_card_this_turn" => {
                "Effect: forces the player to choose and gain one unique physical card they lost from hand/equipment this turn that still remains in discard. Requires Rule VM ABI 201 and the matching loss-history trigger filter.\n\n```\neffect choose_gain_own_lost_discard_card_this_turn(self);\n```"
            }
            "choose_move_field_card_to" => {
                "Effect: forces `player` to move one equipment or delayed-trick card from another living player's field to `target`. Equipment requires an enabled empty matching slot; delayed tricks cannot duplicate a kind already in the destination judgment area. Requires Rule VM ABI 202.\n\n```\neffect choose_move_field_card_to(self, event_source);\n```"
            }
            "draw_then_optionally_distribute_hand_cards" => {
                "Effect: draws `draw_count` cards, waits for nested draw triggers, then lets `player` plan and give up to `give_cap` distinct current hand cards to one or more other living players. Cancelling gives the cards already planned, or none when cancelled immediately. Requires Rule VM ABI 203.\n\n```\neffect draw_then_optionally_distribute_hand_cards(self, 2, 2);\n```"
            }
            "give_selected_then_continue_distinct_recover_at_two" => {
                "Active-only effect: gives the invocation's selected hand cards to its first other target, then after each complete movement/recovery chain optionally chooses another nonempty hand-card batch and a living other recipient not already used by this activation. The giver recovers once when the activation total first reaches two. Requires `uses_per_turn 1 targets 1 1 other select_hand 1 255` and Rule VM ABI 204.\n\n```\neffect give_selected_then_continue_distinct_recover_at_two(self, event_source);\n```"
            }
            "give_selected_then_recover_target" => {
                "Active-only effect: gives selected hand cards in one batch, waits for all movement events and nested triggers, then recovers the still-living recipient. Requires `targets 1 1 other select_hand 1 ...` and Rule VM ABI 301.\n\n```\neffect give_selected_then_recover_target(self, event_source, 1);\n```"
            }
            "resolve_dingpan" => {
                "Active-only effect: lets the selected equipment-bearing target draw one card, then choose either to let the owner discard one currently discardable equipment card, or move all current equipment to that target's hand in one batch and take 1 normal cardless damage after nested movement triggers. Requires `uses_per_turn living_rebel_count targets 1 1 has_equip` and Rule VM ABI 302.\n\n```\neffect resolve_dingpan(self, event_source);\n```"
            }
            "resolve_tanlu" => {
                "Phase-begin-only effect: freezes the absolute signed HP difference between the owner and current-turn player. A zero gap or a gap above the target's original hand count forces 1 normal damage; otherwise the target must choose a batch gift or that damage. After the full damage chain, a surviving target discards one currently discardable owner hand card. Requires `trigger phase_begin owner_alive_not_target optional` and Rule VM ABI 303.\n\n```\neffect resolve_tanlu(self, event_player);\n```"
            }
            "resolve_spxizhan" => {
                "Phase-begin effect: the living owner other than the turn player must discard one hand/equipment card or lose 1 HP. Discarding disables the supplied skill definition until turn end, then resolves a suit-based virtual use: spade → turn player uses Jiu; heart → owner uses Wuzhong; club → owner uses Tiesuo on the turn player; diamond → owner uses fire Sha on the turn player ignoring distance. Requires `trigger phase_begin owner_alive_not_target` and Rule VM ABI 403.\n\n```\neffect resolve_spxizhan(self, event_player, 2304282956);\n```"
            }
            "choose_max_mark_other_judge_kill_unless_tao_or_taoyuan" => {
                "Effect: lets the skill owner choose one living other player tied for the highest positive count of `mark_def`. That target judges and is killed without a killer unless the physical result is Tao or Taoyuan. The chooser may be dead so a death trigger can finish. Requires Rule VM ABI 205.\n\n```\neffect choose_max_mark_other_judge_kill_unless_tao_or_taoyuan(self, mark_def);\n```"
            }
            "choose_zone_priority_gain_random_card_from_each_other_then_turn_over" => {
                "Effect: lets the owner choose hand/equipment/judgment zone priority, then in seat order gains one authoritative deterministic-random card from each other living player's first nonempty preferred zone and finally turns over. Each gain completes before the next target is inspected. Requires Rule VM ABI 206.\n\n```\neffect choose_zone_priority_gain_random_card_from_each_other_then_turn_over(self);\n```"
            }
            "choose_add_missing_skill" => {
                "Effect: forces the player to choose and gain one of three distinct skill definitions they do not already own. If exactly one remains it is granted directly; if none remain the effect does nothing. Requires Rule VM ABI 79.\n\n```\neffect choose_add_missing_skill(self, first_skill_def, second_skill_def, third_skill_def);\n```"
            }
            "add_first_missing_skill" => {
                "Effect: permanently grants the first of three ordered skill definitions the player does not already own. If all are owned, it does nothing. Requires Rule VM ABI 575.\n\n```\neffect add_first_missing_skill(self, first_skill_def, second_skill_def, third_skill_def);\n```"
            }
            "resolve_dccunsi" => {
                "Active-only effect: grants the selected target Yongjue, lets the owner draw two cards when targeting another player, waits for those effects and nested triggers to finish, then recovers the wounded owner if Guixiu is still enabled. Public content must pass stage 0. Requires Rule VM ABI 578.\n\n```\neffect resolve_dccunsi(self, event_source, guixiu_def, yongjue_def, 0);\n```"
            }
            "record_current_sha_all_targets" => {
                "Card-use effect: binds every declared target of the exact currently resolving Sha to the originating skill, so later damage filters can verify provenance. Requires Rule VM ABI 580.\n\n```\neffect record_current_sha_all_targets(self, event_card);\n```"
            }
            "deal_damage" => {
                "Effect: deals damage from one player to another. The optional fourth argument is `normal`, `fire`, or `thunder`; omitting it preserves normal damage.\n\n```\neffect deal_damage(self, event_source, 1, fire);\n```"
            }
            "discard_one_hand_or_equip" => {
                "Effect: forces the specified player to choose and discard exactly one card from hand or equipment.\n\n```\neffect discard_one_hand_or_equip(self);\n```"
            }
            "disable_skill_by_def_until_turn_end" => {
                "Effect: disables every owned instance of the configured skill definition until the next turn begins, preserving each instance and its state. Requires Rule VM ABI 191.\n\n```\neffect disable_skill_by_def_until_turn_end(self, skill_def);\n```"
            }
            "choose_discard_one_then_gain_target_equipment" => {
                "Effect: lets `player` cancel or discard exactly one hand/equipment card. After a valid cost, every card currently equipped by `target` in the requested slot is moved to the player's hand. The cost and matching equipment are revalidated atomically. Slots are `weapon`, `armor`, `horse_def`, `horse_off`, and `treasure`. Requires Rule VM ABI 40.\n\n```\neffect choose_discard_one_then_gain_target_equipment(self, event_source, weapon);\n```"
            }
            "choose_target_draw" => {
                "Effect: forces `player` to choose one living target from `other`, `male_other`, `female_other`, `any`, `min_hand`, `same_hp`, or `hand_empty`, then that target draws `count` cards. Sex scopes include double-sex characters; `min_hand` includes every living player tied for the fewest hand cards; `same_hp` compares current HP with the chooser; `hand_empty` requires no hand cards. The latter three scopes include the chooser and are revalidated at the choice barrier. Base scopes require Rule VM ABI 41; `min_hand` requires ABI 83; `same_hp` requires ABI 135; `hand_empty` requires ABI 162.\n\n```\neffect choose_target_draw(self, 1, min_hand);\n```"
            }
            "choose_targets_draw" => {
                "Effect: forces `player` to choose exactly `target_count` distinct living targets from `other`, `male_other`, `female_other`, `any`, `min_hand`, `same_hp`, or `hand_empty`, then each target draws `count` cards in seat order. If too few legal targets exist, the effect does nothing. Base scopes require Rule VM ABI 80; `min_hand` requires ABI 83; `same_hp` requires ABI 135; `hand_empty` requires ABI 162.\n\n```\neffect choose_targets_draw(self, 2, 2, any);\n```"
            }
            "choose_optional_targets_draw" => {
                "Effect: lets `player` cancel or choose between `min_count` and `max_count` distinct living targets from a canonical scope, then each selected target draws `count` cards in seat order. `wounded` includes every living player below maximum HP, including the chooser. Requires Rule VM ABI 195.\n\n```\neffect choose_optional_targets_draw(self, 1, 1, 255, wounded);\n```"
            }
            "reveal_target_hand_sha_or_discard_bind" => {
                "Active-only effect: implements SP Cao Ren's Weikui sequence. `player` first loses 1 HP and completes the full dying window. If both participants remain alive, absence of an effective Shan forces one target hand discard; otherwise the hand is revealed only to `player`, a distance-free virtual Sha is attempted with the ordinary use-count limit, and the living target is bound to the helper afterward. Requires Rule VM ABI 196.\n\n```\neffect reveal_target_hand_sha_or_discard_bind(self, event_source, helper_skill_def);\n```"
            }
            "draw_remembered_hand_cards" => {
                "Effect: draws `count` cards for `player` and records the exact drawn cards that remain continuously in that player's hand. A recorded card is forgotten when it leaves that hand. Requires Rule VM ABI 197.\n\n```\neffect draw_remembered_hand_cards(self, 2);\n```"
            }
            "discard_remembered_hand_cards" => {
                "Effect: clears the originating skill's remembered hand-card set and discards every still-discardable recorded card that has remained continuously in `player`'s hand. Requires Rule VM ABI 197.\n\n```\neffect discard_remembered_hand_cards(self);\n```"
            }
            "choose_targets_damage" => {
                "Effect: forces `player` to choose between `min_count` and `max_count` distinct living targets from a canonical target scope, then deals `amount` normal damage from `player` to each selected target in seat order. `same_hp` and `hand_empty` include the chooser; `hand_lte_chooser` contains only other living players whose hand count does not exceed the chooser's. Every scope revalidates current state when the choice resumes. Base scopes require Rule VM ABI 135; `hand_empty` requires ABI 162; `hand_lte_chooser` requires ABI 338.\n\n```\neffect choose_targets_damage(self, 1, 1, hand_lte_chooser, 1);\n```"
            }
            "choose_target_draw_to_max_hp_cap" => {
                "Effect: forces `player` to choose any living player, including themselves. The selected target draws until their hand reaches the lesser of their maximum HP and `cap`; an already-full target remains a legal no-op choice. The target and count are revalidated at the choice barrier. Requires Rule VM ABI 84.\n\n```\neffect choose_target_draw_to_max_hp_cap(self, 5);\n```"
            }
            "choose_target_draw_then_owner_if_below_max_hp" => {
                "Effect: forces `player` to choose any living player, including themselves. The selected target draws `count` cards. After that draw and its nested triggers finish, `player` draws one card if the selected target's hand remains below their maximum HP. Requires Rule VM ABI 120.\n\n```\neffect choose_target_draw_then_owner_if_below_max_hp(self, 2);\n```"
            }
            "choose_target_draw_max_hp_cap_then_discard_excess" => {
                "Effect: lets `player` choose any living player with positive maximum HP. The selected target draws the lesser of their current maximum HP and `cap`; after that draw and nested triggers finish, the limit is evaluated again and the target must choose excess hand cards to discard down to it. The chooser may be dead so a death trigger can finish, while the target must be living at the choice barrier. Requires Rule VM ABI 122.\n\n```\neffect choose_target_draw_max_hp_cap_then_discard_excess(self, 5);\n```"
            }
            "give_selected_then_target_choose_discard_same_suit_or_lose_hp" => {
                "Effect: gives the active invocation's single selected hand card to `target`. After movement triggers finish, the recipient must choose either to reveal their hand and discard every discardable hand/equipment card with the given card's physical suit, or lose 1 HP. Requires Rule VM ABI 123.\n\n```\neffect give_selected_then_target_choose_discard_same_suit_or_lose_hp(self, event_source);\n```"
            }
            "give_selected_then_target_may_use_draw_owner" => {
                "Active-only effect: gives the invocation's selected non-equipment hand card to the selected other target. The recipient may immediately use that exact card; after a successful use, the owner draws 1. Requires `active play_phase uses_per_turn 1 targets 1 1 other select_hand 1 1 card_non_equip` and Rule VM ABI 497.\n\n```\neffect give_selected_then_target_may_use_draw_owner(self, event_source);\n```"
            }
            "gain_equipment_from_draw_pile" => {
                "Effect: moves the topmost equipment card in the shared draw pile into `player`'s hand. It does not search the discard pile. Requires Rule VM ABI 124.\n\n```\neffect gain_equipment_from_draw_pile(self);\n```"
            }
            "give_all_hand_recover_to_one" => {
                "Effect: gives every current hand card from `player` to `target`. After the movement and all nested triggers finish, recovers the living target to exactly 1 HP. Requires Rule VM ABI 125.\n\n```\neffect give_all_hand_recover_to_one(self, event_player);\n```"
            }
            "give_all_hand" => {
                "Effect: gives every current hand card from `player` to `target`. Active use requires `hand_nonempty face_up targets 1 1 other male`. Requires Rule VM ABI 151.\n\n```\neffect give_all_hand(self, event_source);\n```"
            }
            "owner_alive_other_source_sha_resolved_not_target_gainable_once_per_turn" => {
                "Sha-resolved trigger filter: once per turn, a living non-target owner observes another player's fully resolved Sha while its physical carrier remains in discard. Requires Rule VM ABI 151."
            }
            "choose_target_self_discard_then_owner_lose_hp_if_hand_ne_hp" => {
                "Effect: forces `player` to choose any living target able to discard one of their own hand/equipment cards. That target chooses and discards exactly one. After the movement and all nested triggers finish, `player` loses 1 HP if the target's hand count differs from their HP. Requires Rule VM ABI 127.\n\n```\neffect choose_target_self_discard_then_owner_lose_hp_if_hand_ne_hp(self);\n```"
            }
            "choose_give_one_hand_equip_or_recover" => {
                "Effect: lets `giver` choose whether to give one current hand/equipment card to `target`. If the giver declines or has no eligible card, the target recovers 1 HP. `owner` remains the originating skill owner even though the giver observes the choice. Requires Rule VM ABI 129.\n\n```\neffect choose_give_one_hand_equip_or_recover(self, event_source, event_player);\n```"
            }
            "resolve_gzjinfa" => {
                "Active-only effect: discards the invocation's selected hand/equipment card, then lets the selected target give one equipment card or lets the owner gain one target hand/equipment card. A spade gift makes the target use a distance-free virtual Sha on the owner. Requires `uses_per_turn 1 targets 1 1 other hand_equip_nonempty select_he 1 1` and Rule VM ABI 495.\n\n```\neffect resolve_gzjinfa(self, event_source);\n```"
            }
            "resolve_wn_qifeng" => {
                "Phase-end effect: uses a distance- and count-free virtual Sha on `target`; its damage equals the number of physical card-loss occurrences for `player` during the current turn, including repeated losses of the same card. Requires `owner_alive_global_phase_end_lost_cards_target_other_alive` and Rule VM ABI 496.\n\n```\neffect resolve_wn_qifeng(self, event_player);\n```"
            }
            "resolve_dcjue" => {
                "Phase-end effect: chooses or confirms the Jue target, then repeats up to the smaller of that target's max HP and the number of cards discarded this turn that remain in discard. Each repetition authoritatively randomizes Sha, Guohe, and Wugu and uses the first kind legal for at least one of owner/target. Requires `owner_alive_global_phase_end_discarded_cards_available` and Rule VM ABI 583.\n\n```\neffect resolve_dcjue(self, event_player);\n```"
            }
            "resolve_ruanji" => {
                "XIANDING Ruanji effect: action 0 tracks the hand revealed by Zhaowen, action 1 draws for tracked red cards used, action 2 rewrites the current Jiu effect, and action 3 resolves Jiudun's black-card branch. Requires Rule VM ABI 584.\n\n```\neffect resolve_ruanji(self, event_source, event_card, 0);\n```"
            }
            "resolve_zhengbi" => {
                "XIANDING Cuimao effect: resolves Zhengbi target/branch selection, phase gain history, zone-distinct gains, and exact gift payments. Requires Rule VM ABI 585."
            }
            "resolve_fengying" => {
                "XIANDING Cuimao effect: discards all hand cards, queues the exact extra turn, and refills to max HP when tied for minimum HP. Requires Rule VM ABI 585."
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
            "resolve_qiaojian" => {
                "XIANDING Fugan effect: privately collects both discard selections, compares physical ranks, recovers or gains the surviving discarded cards, and arms the chosen next-turn policy. Requires Rule VM ABI 586."
            }
            "resolve_xicha" => {
                "XIANDING Fugan effect: privately marks up to two source hand cards, consumes the watch on that source's next card use, and cancels/gains a matching physical use. Requires Rule VM ABI 586."
            }
            "owner_alive_damage_target_self_or_in_range_other_source_numeric_card" => {
                "Zhantao damage filter: owner or a character in range took numeric card damage from another source. Requires Rule VM ABI 587."
            }
            "owner_source_wounded_exists_once_per_turn" => {
                "Anjing damage filter: owner dealt damage, a wounded living target exists, and the skill is unused this turn. Requires Rule VM ABI 587."
            }
            "resolve_huzun" => {
                "XIANDING Huzun effect: resolves Zhantao's rank judgment and virtual Sha or Anjing's growing target cap, ordered draws, and random minimum-HP recovery. Requires Rule VM ABI 587."
            }
            "resolve_zhugemengxue" => {
                "XIANDING Zhugemengxue effect: resolves Jichun's per-branch card-name-length payment and Hanying's draw-pile equipment use. Requires Rule VM ABI 588."
            }
            "resolve_bailingyun" => {
                "XIANDING Bailingyun effect: resolves Linghui's top-three use/random gain/ordered return and Xiace's two once-per-turn branches. Requires Rule VM ABI 593."
            }
            "resolve_dcshaowei" => {
                "XIANDING Wei Guanyinping effect: losing diamonds randomly gains up to three red draw-pile cards; losing hearts recovers 1 HP and restores one equipment slot, each once per turn. Requires Rule VM ABI 594."
            }
            "resolve_dcdichou" => {
                "XIANDING Wei Guanyinping effect: disables an equipment slot, discards a marked target card for name-length bonuses, and resolves selected marked targets at End phase. Requires Rule VM ABI 594."
            }
            "resolve_dcdufeng" => {
                "XIANDING Ling Cao effect: forces one or both of losing 1 HP and disabling an enabled equipment slot, then draws by disabled slots plus lost HP and grants turn-long attack-range/Sha-limit bonuses. Requires Rule VM ABI 595."
            }
            "choose_discard_hand_range" => {
                "Effect: forces `player` to choose and discard between `min` and `max` currently discardable hand cards. The choice cannot be cancelled and card ownership is revalidated on resume. Requires Rule VM ABI 596.\n\n```\neffect choose_discard_hand_range(self, 1, 2147483647);\n```"
            }
            "resolve_dcctjiuxian" => {
                "XIANDING Chen Tai effect: recasts exactly half the owner's hand rounded up, then uses a tracked virtual Duel through the ordinary card pipeline; exact original-target damage may optionally recover one wounded character in that target's attack range. Requires Rule VM ABI 597."
            }
            "resolve_bianyue" => {
                "XIANDING Bian Yue effect: Bizu draws every character tied with the owner's hand count and disables itself plus recovers on a repeated exact group; Wuxie swaps whole hands with a chosen other character, then moves every damage card in the owner's new hand to the draw-pile bottom. Requires Rule VM ABI 598."
            }
            "resolve_zhupeilan" => {
                "XIANDING Zhu Peilan effect: Cilv draws by remaining choices, permanently removes one selected option, binds nullification/damage prevention/material gain to the exact trick use, and Tongdao resets a chosen character to fresh starting skills before recovery. Requires Rule VM ABI 600."
            }
            "resolve_dc_sb_zhouyu" => {
                "XIANDING DC SB Zhou Yu effect: Ronghuo arms and consumes the next fire-card boost; Yingmou toggles conversion state and resolves the draw/Fire Attack or physical damage-card sequence. Requires Rule VM ABI 602."
            }
            "resolve_dc_sb_lusu" => {
                "XIANDING DC SB Lu Su effect: Mingshi draws two, reveals three selected hand cards, and lets another character gain one; Mengmou toggles its conversion state and resolves its bounded Sha-use or Sha-response sequence. Requires Rule VM ABI 603."
            }
            "owner_hand_transfer_other_dcsbmengmou_branch_available" => {
                "Mengmou transfer filter: the owner and another living character exchanged at least one hand card, and the current conversion branch remains unused this turn. Requires Rule VM ABI 603."
            }
            "dcsbmengmou_tracked_sha_damage" => {
                "Mengmou Yang filter: exact damage from the currently tracked prompted Sha, repeated once per damage point. Requires Rule VM ABI 603."
            }
            "dcsbmengmou_tracked_sha_resolved" => {
                "Mengmou Yang filter: the currently tracked prompted Sha finished resolving, allowing the bounded sequence to continue. Requires Rule VM ABI 603."
            }
            "resolve_zhangjian" => {
                "XIANDING Zhangjian effect: rewrites card damage to its physical rank or prevents rankless logical-card damage, and discards all discardable hand/equipment cards before granting a source-turn-bound skill. Requires Rule VM ABI 604."
            }
            "resolve_zhugeruoxue" => {
                "XIANDING Zhugeruoxue effect: Qiongying moves one legal field card then discards a matching-suit hand card or reveals the hand; Nuanhui repeatedly uses legal virtual basics up to the target's snapshotted equipment count and discards equipment after a repeated name. Requires Rule VM ABI 605."
            }
            "owner_is_target_card_damage_before" => {
                "Card-damage-before filter: the living owner is the pending damage target and the damage carries a physical or logical card context. Requires Rule VM ABI 604."
            }
            "choose_give_two_different_colors_or_recover" => {
                "Effect: lets `giver` give exactly two current hand/equipment cards with different effective colors to `target`. If the giver declines or has no valid pair, the target recovers 1 HP. `owner` remains the originating skill owner. Requires Rule VM ABI 330.\n\n```\neffect choose_give_two_different_colors_or_recover(self, event_source, event_player);\n```"
            }
            "resolve_mbzhuguo" => {
                "Active-only effect: adjusts the selected target's hand to min(max HP, 5). A non-draw branch recovers that target by 1 HP. After an actual draw, if the target ties for the largest living hand, the owner may select another character; the adjusted target may then use a current physical Sha including that character, ignoring ordinary distance and use count. Requires `active play_phase uses_per_turn 1 targets 1 1` and Rule VM ABI 331.\n\n```\neffect resolve_mbzhuguo(self, event_source);\n```"
            }
            "resolve_mbanxian" => {
                "Cards-moved-only effect: forces the owner to choose one physical Sha from the triggering first hand-discard event that remains in the shared discard pile, gains it, waits for movement triggers, then uses that exact Sha when a legal target exists without ordinary distance or use-count limits. Requires `trigger cards_moved owner_first_hand_discard_has_sha_in_discard` and Rule VM ABI 332.\n\n```\neffect resolve_mbanxian(self);\n```"
            }
            "resolve_mbjinzu" => {
                "Active-only effect: uses the invocation's selected owner hand card, lets the selected other player privately choose two hand cards when needed, reveals them together, resolves the extrema discard/refresh branches, and accumulates a turn-bound helper for the owner's next Sha. Requires `active play_phase uses_per_turn 1 targets 1 1 other select_hand 1 1` and Rule VM ABI 333.\n\n```\neffect resolve_mbjinzu(self, event_source, 4156343864);\n```"
            }
            "apply_mbjinzu_next_sha" => {
                "Card-used-only effect for the locked Jinsu helper: consumes its target map on the owner's next Sha, makes matching targets unable to respond, and increases only those targets' damage by their accumulated amounts. Requires `trigger card_used owner_source_sha` and Rule VM ABI 333.\n\n```\neffect apply_mbjinzu_next_sha(self, event_card);\n```"
            }
            "choose_other_draw_owner_lost_hp_turn_over" => {
                "Effect: forces `player` to choose another living player. On choice resolution, the target draws cards equal to the owner's current lost HP, then turns over. Owner/target state is revalidated at the continuation barrier. Requires Rule VM ABI 85.\n\n```\neffect choose_other_draw_owner_lost_hp_turn_over(self);\n```"
            }
            "draw_by_play_phase_card_category_count" => {
                "Effect: draws cards equal to the number of distinct logical basic, trick, and equipment categories the player used during the current play phase. Requires Rule VM ABI 42.\n\n```\neffect draw_by_play_phase_card_category_count(self);\n```"
            }
            "choose_discard_one_for_sha_refund_and_draw_if_red" => {
                "Effect: forces `player` to discard exactly one current hand/equipment card, then revalidates the suspended proactive Sha, refunds its ordinary play-phase use count at most once across all skills, and makes `player` draw one if the Sha's effective suit is red. If no cost card remains when the effect starts, nothing happens. Requires Rule VM ABI 43.\n\n```\neffect choose_discard_one_for_sha_refund_and_draw_if_red(self, event_source, event_card);\n```"
            }
            "choose_discard_one_for_sha_refund_and_draw_if_red_restore_if_equal_rank" => {
                "Effect: preserves Longyin's discard, at-most-once Sha-use refund, and effective-red draw, then restores `restore_skill_def` only when the discarded card and suspended Sha have equal effective rank. Requires Rule VM ABI 295.\n\n```\neffect choose_discard_one_for_sha_refund_and_draw_if_red_restore_if_equal_rank(self, event_source, event_card, restore_skill_def);\n```"
            }
            "resolve_beige_configured" => {
                "Effect: reuses Beige's discard, judgment, and rewrite pipeline while supplying the Heart recovery amount and Diamond draw count. Requires Rule VM ABI 296.\n\n```\neffect resolve_beige_configured(self, event_player, event_source, event_amount, 3);\n```"
            }
            "resolve_zhuiyi" => {
                "Effect: lets the dead skill owner cancel or choose one living player other than the owner and credited killer. The chosen player recovers 1 HP, then draws 3 cards even if already at full HP. Requires Rule VM ABI 297.\n\n```\neffect resolve_zhuiyi(self, event_source);\n```"
            }
            "queue_opening_extra_play_phase" => {
                "Effect: queues one independent Play phase at the current player's recoverable turn-begin boundary. Queued phases run before Prepare; each resets play-phase skill counters, logical-card category history, and Sha-use allowance while retaining turn-wide history. Available only from `phase_begin` and requires Rule VM ABI 44.\n\n```\neffect queue_opening_extra_play_phase(self);\n```"
            }
            "convert_current_sha_to_fire_lose_hp_if_damage" => {
                "Effect: revalidates the suspended proactive ordinary Sha, changes only that use's damage nature to fire, and records a post-use cost. After every target and nested dying window resolves, the living user loses 1 HP iff the use committed damage. Requires Rule VM ABI 45.\n\n```\neffect convert_current_sha_to_fire_lose_hp_if_damage(self, event_card);\n```"
            }
            "choose_add_current_sha_target" => {
                "Effect: lets the current Sha user cancel or add one other legal living target. The saved source/card, existing targets, range, and card-target policies are revalidated at the choice barrier. Requires Rule VM ABI 45.\n\n```\neffect choose_add_current_sha_target(self, event_card);\n```"
            }
            "increase_current_sha_base_damage" => {
                "Effect: revalidates the suspended proactive Sha source/card and increases only that use's saved base damage. Requires Rule VM ABI 99.\n\n```\neffect increase_current_sha_base_damage(self, event_card, 1);\n```"
            }
            "increase_current_card_base_damage" => {
                "Effect: revalidates the suspended proactive Sha, Juedou, Nanman, or Wanjian source/card and increases only that use's saved base damage. Juedou support requires Rule VM ABI 229; the original surface requires ABI 104.\n\n```\neffect increase_current_card_base_damage(self, event_card, 1);\n```"
            }
            "lose_hp_then_damage_other_lose_max_hp_if_dying" => {
                "Active-only effect: the owner loses `amount` HP; if that exact loss first puts them into dying, they lose one maximum HP. Equal ordinary damage to the selected other target resolves only after the complete dying/save window. Requires `targets 1 1 other`, `self_skill`, and Rule VM ABI 108.\n\n```\neffect lose_hp_then_damage_other_lose_max_hp_if_dying(self, event_source, 1, self_skill);\n```"
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
            "choose_store_hand_sha_in_skill_pile" => {
                "Effect: forces the owner to select one or more logical Sha from hand and move them into this skill definition's public expansion pile. Requires Rule VM ABI 46.\n\n```\neffect choose_store_hand_sha_in_skill_pile(self);\n```"
            }
            "choose_spend_skill_pile_for_dying_jiu" => {
                "Effect: forces the owner to spend one card from this skill definition's public expansion pile, then makes the current dying player use a suitless virtual Jiu on themselves. Requires Rule VM ABI 46.\n\n```\neffect choose_spend_skill_pile_for_dying_jiu(self, event_player);\n```"
            }
            "choose_compare_selected_hand_card" => {
                "Effect: after an active `select_hand 1 1`, forces the target to choose one hand card, publicly reveals and discards both cards, then grants a target-bound helper on strict victory or a normal helper on tie/loss. Requires `targets ... can_compare` and Rule VM ABI 47.\n\n```\neffect choose_compare_selected_hand_card(self, event_source, win_skill_def, lose_skill_def);\n```"
            }
            "choose_add_skill_bound_card_target" => {
                "Effect: lets the card user cancel or append this skill instance's bound living target to a suspended one-target Sha or ordinary trick. Source/card, target policies, and the bound state are revalidated at the choice barrier. Requires Rule VM ABI 47.\n\n```\neffect choose_add_skill_bound_card_target(self, event_card);\n```"
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
            "discard_all_hand_and_equip" => {
                "Effect: discards every card in the specified player's hand and equipment zones.\n\n```\neffect discard_all_hand_and_equip(event_source);\n```"
            }
            "remove_all_skills" => {
                "Effect: removes every skill currently owned by the specified player. Requires Rule VM ABI 159.\n\n```\neffect remove_all_skills(event_source);\n```"
            }
            "judge_gain_equipment_by_color" => {
                "Effect: performs a judgment; red gains the topmost equipment from the draw pile, black from the discard pile.\n\n```\neffect judge_gain_equipment_by_color(self);\n```"
            }
            "judge_recover_on_suit" => {
                "Effect: performs a judgment for the specified player and recovers HP when the effective result suit matches. The suit is `spade`, `heart`, `club`, or `diamond`. Requires Rule VM ABI 12.\n\n```\neffect judge_recover_on_suit(event_player, heart, 1);\n```"
            }
            "judge_recover_other_on_suit" => {
                "Effect: performs a judgment for `judge_player`, then recovers `recover_player` when the effective result suit matches. Requires Rule VM ABI 88.\n\n```\neffect judge_recover_other_on_suit(event_source, self, spade, 1);\n```"
            }
            "force_others_use_sha_on_nearest_or_lose_hp" => {
                "Effect: snapshots every other living player in clockwise seat order. Each participant may supply a physical or play-phase view-as Sha and choose only minimum-distance legal targets; cancelling or having no legal Sha loses 1 HP. Prompted Sha does not consume the ordinary Sha-use count, and the queue resumes after responses, triggers, and dying saves. Requires Rule VM ABI 90.\n\n```\neffect force_others_use_sha_on_nearest_or_lose_hp(self);\n```"
            }
            "judge_damage_other_on_suit" => {
                "Effect: performs a judgment for the specified player. When the effective suit matches, that player must choose another living player and deal the specified normal, fire, or thunder damage. Requires Rule VM ABI 15.\n\n```\neffect judge_damage_other_on_suit(self, spade, 1, thunder);\n```"
            }
            "recast_selected" => {
                "Effect: moves the active invocation's selected personal-zone cards to the discard pile with the semantic Recast reason, then draws the same number of replacements. Requires Rule VM ABI 16.\n\n```\neffect recast_selected(self);\n```"
            }
            "disable_all_equip_slots" => {
                "Effect: disables every currently enabled equipment slot and discards equipment occupying those slots. Requires Rule VM ABI 18.\n\n```\neffect disable_all_equip_slots(self);\n```"
            }
            "move_equipment_then_draw" => {
                "Effect: lets the skill owner choose one movable equipment card and a legal destination other than themselves and the current holder. Replaces matching equipment at the destination, then the original holder draws one. Requires Rule VM ABI 30.\n\n```\neffect move_equipment_then_draw(self);\n```"
            }
            "choose_use_virtual_sha_or_draw" => {
                "Effect: materializes a normal-distance virtual Sha without consuming the ordinary Sha-use count. The player chooses legal targets; cancelling or having no legal use draws one card instead. Requires Rule VM ABI 31.\n\n```\neffect choose_use_virtual_sha_or_draw(self);\n```"
            }
            "discard_equipment_pair_with_armor" => {
                "Effect: for the active invocation's two selected targets, choose and discard one equipment card from each while requiring at least one selected card to be armor. Requires `targets 2 2 equipment_pair_with_armor` and Rule VM ABI 32.\n\n```\neffect discard_equipment_pair_with_armor(self);\n```"
            }
            "discard_target_equipment_gain_mount" => {
                "Effect: forces the player to choose and discard one discardable equipment card from the target. If the card is an offensive or defensive mount and remains in the discard pile, it is then moved to the player's hand as a separate card movement. Requires Rule VM ABI 33.\n\n```\neffect discard_target_equipment_gain_mount(self, event_player);\n```"
            }
            "discard_target_equipment_draw_weapon_range" => {
                "Active-only effect: forces the skill owner to choose and discard one currently discardable equipment card from the selected target. After the discard movement and nested triggers finish, a discarded physical weapon makes that target draw its printed attack range (minimum one); other equipment draws nothing. Requires exact `targets 1 1 has_discardable_equip` and Rule VM ABI 136.\n\n```\neffect discard_target_equipment_draw_weapon_range(self, event_source);\n```"
            }
            "choose_target_reveal_hand_damage_if_sha" => {
                "Effect: forces `player` to choose any living player with a nonempty hand, publicly reveals that target's complete current hand, and deals `amount` normal damage from `player` when at least one revealed card is an effective Sha for that target. Skill, observer, participants, hand state, and logical card kinds are revalidated at the Choice barrier. Requires Rule VM ABI 137.\n\n```\neffect choose_target_reveal_hand_damage_if_sha(self, 1);\n```"
            }
            "choose_other_give_one_hand_equip" => {
                "Effect: forces `player` to choose another living player, then give that target exactly one current hand/equipment card through the shared gift pipeline. Skill ownership, participants, target legality, and card ownership are revalidated at the choice barriers. Requires Rule VM ABI 138.\n\n```\neffect choose_other_give_one_hand_equip(self);\n```"
            }
            "choose_reveal_hand_card_bind_category_target" => {
                "Effect: forces the skill owner to choose and publicly reveal one current hand card from the specified Play-phase player, then binds that player and the card's effective basic/trick/equipment category to this skill instance until the phase changes. Requires Rule VM ABI 140.\n\n```\neffect choose_reveal_hand_card_bind_category_target(self, event_player);\n```"
            }
            "recover_to_max_then_optional_exchange_skill" => {
                "Effect: consumes the originating limited skill, recovers its owner to max HP, waits for recovery and nested triggers to finish, then optionally sacrifices the specified owned skill definition to restore a fresh instance of the limited skill. Requires Rule VM ABI 142.\n\n```\neffect recover_to_max_then_optional_exchange_skill(self, 3956037825);\n```"
            }
            "reveal_hand_discard_kind" => {
                "Effect: publicly reveals `player`'s complete current hand, then discards every currently discardable hand card whose effective kind matches `card_kind`. Requires Rule VM ABI 144.\n\n```\neffect reveal_hand_discard_kind(event_player, shan);\n```"
            }
            "discard_target_card_gain_mount" => {
                "Effect: forces the player to choose and discard one target card from `equip`, `hand_equip`, or `hand_equip_judge`. If the card is a mount and remains in the discard pile, it is moved to the player's hand as a separate movement. The scope is revalidated at the continuation barrier. Requires Rule VM ABI 34.\n\n```\neffect discard_target_card_gain_mount(self, event_player, hand_equip_judge);\n```"
            }
            "choose_discard_target_card" => {
                "Effect: forces the player to choose and discard one target card from `hand`, `equip`, `hand_equip`, or `hand_equip_judge`. The scope, life state, card ownership, and discard protection are revalidated at the continuation barrier. `hand` requires Rule VM ABI 153; other scopes require ABI 36.\n\n```\neffect choose_discard_target_card(self, event_player, hand_equip);\n```"
            }
            "reveal_hand_to_player" => {
                "Effect: reveals one living player's complete current hand only to one living observer. Requires Rule VM ABI 153.\n\n```\neffect reveal_hand_to_player(self, event_source);\n```"
            }
            "reveal_hand" => {
                "Effect: publicly reveals one living player's complete current hand. Requires Rule VM ABI 336.\n\n```\neffect reveal_hand(self);\n```"
            }
            "choose_optional_discard_target_card" => {
                "Effect: lets the player cancel or choose and discard one target card from `hand`, `equip`, `hand_equip`, or `hand_equip_judge`. An empty scope is a no-op; submitted cards are revalidated at the continuation barrier. Requires Rule VM ABI 153.\n\n```\neffect choose_optional_discard_target_card(self, event_source, hand);\n```"
            }
            "reveal_hand_choose_discard_color" => {
                "Effect: publicly reveals `player`'s complete current hand. If both effective red and black are present, that player must choose one color; if only one is present it is selected automatically. Every currently discardable hand card of the selected color is then discarded. Requires Rule VM ABI 154.\n\n```\neffect reveal_hand_choose_discard_color(event_player);\n```"
            }
            "choose_discard_target_cards_recover_same_category" => {
                "Active-only effect: forces the owner to discard `min(count, available)` current hand/equipment cards from the selected target. If exactly `count` cards were discarded and their legacy type2 categories match, the owner recovers `amount` HP after movement triggers and nested choices finish. Requires exact `targets 1 1 other in_attack_range has_discardable_hand_equip` and Rule VM ABI 155.\n\n```\neffect choose_discard_target_cards_recover_same_category(self, event_source, 2, 1);\n```"
            }
            "choose_compare_selected_hand_card_second_failure_seat_damage" => {
                "Active-only effect: compares the selected hand card with one unused target. A strict win lets the owner draw or discard one target hand/equipment card. On the configured non-win threshold, the owner loses HP, chooses between equal seat paths or takes the shorter path, damages its living players in order, and the active skill is banned until turn end. Requires `targets 1 1 other can_compare unused_by_skill_this_turn select_hand 1 1` and Rule VM ABI 156.\n\n```\neffect choose_compare_selected_hand_card_second_failure_seat_damage(self, event_source, 1, 2, 1, 1);\n```"
            }
            "reveal_discard_suit_gain_or_use" => {
                "Effect: reveals base_count plus capped mark_count cards, then lets the player perform the discard-by-suit gain and use-one-revealed-card options in either order. Completing both adds one mark and recasts the player's hand.\n\n```\neffect reveal_discard_suit_gain_or_use(self, mark_def, 3, 2);\n```"
            }
            "choose_discard_or_fire_damage_skip_phases" => {
                "Effect: lets the current-turn player choose the discard branch, fire-damage branch, or both. The discard branch may skip draw; the damage branch may skip play.\n\n```\neffect choose_discard_or_fire_damage_skip_phases(self, 2, 1);\n```"
            }
            "remove_cards_by_kind" => {
                "Effect: moves every card of one rule kind from draw, discard, hand, equipment, and judgment zones into the shared removed-from-game zone.\n\n```\neffect remove_cards_by_kind(horse_off);\n```"
            }
            "kill_player" => {
                "Effect: immediately kills a player and attributes the death to the specified killer.\n\n```\neffect kill_player(event_player, event_source);\n```"
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
            "duel_resolved" => {
                "Trigger event emitted after a Duel resolves; `event_player` is the winner and `event_source` is the loser."
            }
            "character_state_changed" => {
                "Trigger event emitted when a character card turns over or actually changes between linked and unlinked; `event_player` is the affected player. Requires Rule VM ABI 25."
            }
            "use_virtual_card_on" => {
                "Effect: materialize a suitless virtual card and use it on one explicit target through the ordinary card pipeline."
            }
            "recover_hp_as" => {
                "Effect: recover HP using an explicit source player for event attribution. Requires Rule VM ABI 185.\n\n```\neffect recover_hp_as(event_source, self, 1);\n```"
            }
            "choose_discard_redirect_current_sha" => {
                "Effect: let the current Sha target select a discardable hand/equipment card and a legal replacement in the owner's attack range. The card is paid only after both choices, and the saved Sha target slot changes after discard triggers finish. Requires Rule VM ABI 186.\n\n```\neffect choose_discard_redirect_current_sha(self, event_source, event_card);\n```"
            }
            "judge_set_skill_mark_to_rank" => {
                "Effect: perform a judgment for `player`, then replace the originating skill's same-id mark with the final physical result rank. Requires Rule VM ABI 188.\n\n```\neffect judge_set_skill_mark_to_rank(self, self_skill);\n```"
            }
            "choose_discard_one_from_each_linked" => {
                "Effect: snapshot living linked players in seat order and let the skill owner choose one discardable hand/equipment card from each; each choice is revalidated. Requires Rule VM ABI 189.\n\n```\neffect choose_discard_one_from_each_linked(self, self_skill);\n```"
            }
            "target_discard_then_duel_if_non_sha_and_hp_gte_owner" => {
                "Active-only effect: force the selected target to discard one hand/equipment card, wait for its movement triggers, then use a virtual Duel on it when that card was not Sha, its HP is at least the owner's, and the Duel remains legal. Requires `targets 1 1 other in_attack_range hand_equip_nonempty` and Rule VM ABI 192."
            }
            "choose_target_lose_hp_discard_hand_sha_if_basic" => {
                "Effect: forces the owner to choose any living player able to discard a hand card, loses 1 HP, then forces that target to discard one hand card. After movement triggers, a basic discarded card causes an ordinary-count virtual Sha that ignores distance when the use remains legal. Requires Rule VM ABI 194."
            }
            "sha_distance_unlimited_if_rank_lt_skill_mark" => {
                "Modifier target: a logical Sha whose physical rank is below this skill's same-id mark ignores ordinary distance. Requires `override 1` and Rule VM ABI 188."
            }
            "sha_distance_unlimited_if_target_distance_lte_rank" => {
                "Modifier target: a logical physical Sha ignores its ordinary target-distance limit only when the final distance to that target is at most the card's physical rank. Requires `override 1` and Rule VM ABI 223."
            }
            "sha_no_use_count_if_rank_gt_skill_mark" => {
                "Modifier target: a logical Sha whose physical rank is above this skill's same-id mark bypasses and does not consume the ordinary use count. Requires `override 1` and Rule VM ABI 188."
            }
            "use_virtual_card_on_selected_targets" => {
                "Effect: materialize one suitless virtual Duel or Tiesuo and use it on the active skill's selected distinct targets through the ordinary card pipeline. Duel requires other targets and Rule VM ABI 95; Tiesuo requires 1–2 targets and Rule VM ABI 343."
            }
            "deal_damage_ban_active_if_dying" => {
                "Active-only effect: deal ordinary source-free normal damage through the standard damage-before and dying pipelines. If this exact damage first makes the target enter dying, ban `self_skill` from active use until the next turn begins. Requires Rule VM ABI 96.\n\n```\neffect deal_damage_ban_active_if_dying(self, event_source, 1, self_skill);\n```"
            }
            "choose_target_reclaim_equipment_draw_if_range_reduced" => {
                "Active-only effect: let the skill owner choose one card from the selected target's equipment zone, move it into that target's own hand, then draw one card for the owner only if the target's attack-range population decreased after nested card-movement triggers finish. Requires exact `targets 1 1 other has_equip` and Rule VM ABI 97.\n\n```\neffect choose_target_reclaim_equipment_draw_if_range_reduced(self, event_source, self_skill);\n```"
            }
            "choose_use_virtual_card" => {
                "Triggered effect: materialize a suitless source-free Wanjian or Juedou and pass it through ordinary use, target, Wuxie, response, damage, and discard resolution. Wanjian uses its complete legal target set; Juedou opens a mandatory target choice. Requires Rule VM ABI 98.\n\n```\neffect choose_use_virtual_card(self, wanjian, self_skill);\n```"
            }
            "choose_use_virtual_nanman" => {
                "Triggered effect: materialize a suitless source-free Nanman (南蛮入侵) and pass it through ordinary use, target, Wuxie, response, damage, and discard resolution. Nanman uses its complete legal target set and is phase-agnostic (no phase-end history suppression). Requires Rule VM ABI 604.\n\n```\neffect choose_use_virtual_nanman(self, self_skill);\n```"
            }
            "resolve_qiluan" => {
                "Triggered effect: draw Σ alive players' current-turn kill counts × (3 if the killer is the owner else 1). Implements 起乱 (qiluan). No-op when the total is zero. Requires Rule VM ABI 605.\n\n```\neffect resolve_qiluan(self, self_skill);\n```"
            }
            "discard_all_own_judge" => {
                "Triggered effect: discard every card currently in the player's judgment zone. Implements 飞杨 (olfeiyang). Requires Rule VM ABI 606.\n\n```\neffect discard_all_own_judge(self, self_skill);\n```"
            }
            "store_event_card_in_skill_pile" => {
                "Triggered effect: store the triggering event's used card onto the owner's skill pile (武将牌) when the card sits in the discard pile. Implements 灵杉 (huamu). Requires Rule VM ABI 607.\n\n```\neffect store_event_card_in_skill_pile(self, event_card, self_skill);\n```"
            }
            "discard_hand_expr" => {
                "Triggered effect: forced hand discard with an expression-driven count, clamped to the available hand (e.g. `7 - mark_count(self, def)`). Compiles to the same VM op as literal-count discard_hand.\n\n```\neffect discard_hand_expr(event_source, 7 - mark_count(self, 3913395307));\n```"
            }
            "discard_hand_equip_expr" => {
                "Triggered effect: forced hand-or-equip discard with an expression-driven count, clamped to the available cards. Compiles to the same VM op as literal-count discard_hand_equip.\n\n```\neffect discard_hand_equip_expr(self, hand_count(self) + 2 - hand_limit(self));\n```"
            }
            "gain_target_hand_card" => {
                "Triggered effect: the owner selects another living player, then gains one of that player's hand cards (the owner picks the card). Implements 肃州 (pxsizhou). Requires Rule VM ABI 647.\n\n```\neffect gain_target_hand_card(self, self_skill);\n```"
            }
            "choose_target_gain_max_hp" => {
                "Triggered effect: the owner selects any living player (self included); the selected player gains 1 max hp. Implements 寿春 (pxshouchun). Requires Rule VM ABI 646.\n\n```\neffect choose_target_gain_max_hp(self, self_skill);\n```"
            }
            "self" => "Refers to the skill owner (the player who has this skill).",
            "event_amount" => {
                "Amount bound to a damage, HP-loss, or HP-recovery trigger. It can be used in effect expressions, for example `draw_cards(self, event_amount * 3)`."
            }
            "event_target_count" => {
                "Number of declared targets bound to a `card_used` or `card_targeted` trigger."
            }
            "turn_damaged_player_count" => {
                "Number of distinct players damaged during the current turn, including players who later died; available in `phase_end` triggers."
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
            "mark_count" => {
                "Read-only query: `mark_count(player, mark_id)` returns the player's current stable mark count."
            }
            "alive_count" => {
                "Read-only query: `alive_count()` returns the number of living players."
            }
            "card_action_count" => {
                "Read-only query: `card_action_count(player)` returns cards used or played as responses by that player during the current turn."
            }
            "empty_equip_slots" => {
                "Read-only query: `empty_equip_slots(player)` returns the number of empty equipment slots."
            }
            _ => return super::owner_is_target_no_turn_damage::documentation(word),
    })
}
