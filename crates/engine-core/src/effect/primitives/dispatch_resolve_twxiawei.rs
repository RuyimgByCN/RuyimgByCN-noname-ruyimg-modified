//! Primitive dispatch for `ResolveTwxiawei` and related requests.

use super::*;

pub(super) fn apply_effect(
    tx: &mut Transaction,
    req: EffectRequest,
) -> Result<Vec<DomainEvent>, EffectError> {
    match req {
        EffectRequest::ResolveTwxiawei {
            player,
            wangxing_skill_def,
            action,
            stage,
            skill,
        } => resolve_twxiawei(tx, player, wangxing_skill_def, action, stage, skill),

        EffectRequest::ResolveTwbianfuren {
            player,
            target,
            helper_skill_def,
            action,
            stage,
            cards,
            skill,
        } => resolve_twbianfuren(
            tx,
            player,
            target,
            helper_skill_def,
            action,
            stage,
            cards,
            skill,
        ),

        EffectRequest::ResolveTwfeiyi {
            player,
            action,
            stage,
            card,
            event_cards,
            skill,
        } => resolve_twfeiyi(tx, player, action, stage, card, event_cards, skill),

        EffectRequest::ResolveTwchenzhen {
            player,
            target,
            helper_skill_def,
            record_skill_def,
            action,
            stage,
            card_kind,
            card,
            branch,
            skill,
        } => resolve_twchenzhen(
            tx,
            player,
            target,
            helper_skill_def,
            record_skill_def,
            action,
            stage,
            card_kind,
            card,
            branch,
            skill,
        ),

        EffectRequest::ResolveTwqianju {
            player,
            target,
            skill,
        } => resolve_twqianju(tx, player, target, skill),

        EffectRequest::ResolveTwqingxi {
            player,
            target,
            card,
            stage,
            count,
            cards,
            skill,
        } => resolve_twqingxi(tx, player, target, card, stage, count, cards, skill),

        EffectRequest::ResolveTwzaoli {
            player,
            stage,
            cards,
            slots,
            count,
            skill,
        } => resolve_twzaoli(tx, player, stage, cards, slots, count, skill),

        EffectRequest::ResolveTwzhengjian {
            player,
            target,
            helper_skill_def,
            action,
            stage,
            card,
            branch,
            skill,
        } => resolve_twzhengjian(
            tx,
            player,
            target,
            helper_skill_def,
            action,
            stage,
            card,
            branch,
            skill,
        ),

        EffectRequest::ResolveTwtianyu {
            player,
            target,
            helper_skill_def,
            action,
            stage,
            card,
            branch,
            skill,
        } => resolve_twtianyu(
            tx,
            player,
            target,
            helper_skill_def,
            action,
            stage,
            card,
            branch,
            skill,
        ),

        EffectRequest::ResolveTwwujing {
            player,
            target_count,
            action,
            event_cards,
            skill,
        } => resolve_twwujing(tx, player, target_count, action, event_cards, skill),

        EffectRequest::ResolveTwshanghe {
            player,
            stage,
            remaining,
            got_jiu,
            skill,
        } => resolve_twshanghe(tx, player, stage, remaining, got_jiu, skill),

        EffectRequest::ResolveTwwangchang {
            player,
            target,
            card,
            selected_card,
            action,
            stage,
            targets,
            skill,
        } => resolve_twwangchang(
            tx,
            player,
            target,
            card,
            selected_card,
            action,
            stage,
            targets,
            skill,
        ),

        EffectRequest::ResolveTwcaozhao {
            player,
            target,
            source,
            card,
            selected_skill_def,
            twfuzuan_skill_def,
            twfeifu_skill_def,
            action,
            stage,
            skill,
        } => resolve_twcaozhao(
            tx,
            player,
            target,
            source,
            card,
            selected_skill_def,
            twfuzuan_skill_def,
            twfeifu_skill_def,
            action,
            stage,
            skill,
        ),

        EffectRequest::ResolveTwguohuai {
            player,
            target,
            source,
            card,
            mark_def,
            helper_skill_def,
            action,
            stage,
            skill,
        } => resolve_twguohuai(
            tx,
            player,
            target,
            source,
            card,
            mark_def,
            helper_skill_def,
            action,
            stage,
            skill,
        ),

        EffectRequest::ResolveTwchengpu {
            player,
            target,
            source,
            card,
            chunlao_skill_def,
            action,
            stage,
            selected_card,
            skill,
        } => resolve_twchengpu(
            tx,
            player,
            target,
            source,
            card,
            chunlao_skill_def,
            action,
            stage,
            selected_card,
            skill,
        ),

        EffectRequest::ResolveTwcaoang {
            player,
            target,
            source,
            card,
            helper_skill_def,
            action,
            stage,
            skill,
        } => resolve_twcaoang(
            tx,
            player,
            target,
            source,
            card,
            helper_skill_def,
            action,
            stage,
            skill,
        ),

        EffectRequest::ResolveTwniujin {
            player,
            limited_skill_def,
            action,
            stage,
            skill,
        } => resolve_twniujin(tx, player, limited_skill_def, action, stage, skill),

        EffectRequest::ResolveTwmoukui {
            player,
            target,
            card,
            stage,
            skill,
        } => resolve_twmoukui(tx, player, target, card, stage, skill),

        EffectRequest::ResolveTwfuman {
            player,
            target,
            cards,
            action,
            stage,
            skill,
        } => resolve_twfuman(tx, player, target, cards, action, stage, skill),

        EffectRequest::ResolveCuijin {
            player,
            source,
            card,
            action,
            skill,
        } => resolve_cuijin(tx, player, source, card, action, skill),

        EffectRequest::ResolveTwqingkou {
            player,
            winner,
            card,
            action,
            skill,
        } => resolve_twqingkou(tx, player, winner, card, action, skill),

        EffectRequest::ResolveTwchenxun {
            player,
            target,
            card,
            action,
            skill,
        } => resolve_twchenxun(tx, player, target, card, action, skill),

        EffectRequest::ResolveTwfenxian {
            player,
            target,
            card,
            action,
            skill,
        } => resolve_twfenxian(tx, player, target, card, action, skill),

        EffectRequest::SetSkillContentFlag {
            player,
            bit,
            value,
            skill,
        } => set_skill_content_flag(tx, player, bit, value, skill),

        EffectRequest::ResolveSpsongshu {
            player,
            phase_player,
            renku_skill_def,
            block_skill_def,
        } => resolve_spsongshu(tx, player, phase_player, renku_skill_def, block_skill_def),

        EffectRequest::ResolveSpcunsi { player, target } => resolve_spcunsi(tx, player, target),

        EffectRequest::ResolveYuli {
            player,
            mode,
            source,
            target,
            amount,
            nature,
            card,
            card_kind,
            card_flags,
            skill,
        } => resolve_yuli(
            tx, player, mode, source, target, amount, nature, card, card_kind, card_flags, skill,
        ),

        EffectRequest::ResolveTingwei {
            player,
            mode,
            card,
            targets,
            target,
            skill,
        } => resolve_tingwei(tx, player, mode, card, targets, target, skill),

        EffectRequest::ResolveXinmingshi {
            player,
            source,
            skill,
        } => resolve_xinmingshi(tx, player, source, skill),

        EffectRequest::ResolveXunyi {
            player,
            mode,
            source,
            target,
            skill,
        } => resolve_xunyi(tx, player, mode, source, target, skill),

        EffectRequest::ResolveBoming {
            player,
            target,
            card,
            skill,
        } => resolve_boming(tx, player, target, card, skill),

        EffectRequest::ResolveEjian {
            player,
            target,
            card,
            skill,
        } => resolve_ejian(tx, player, target, card, skill),

        EffectRequest::ResolveYiyong {
            player,
            card,
            source,
            skill,
        } => resolve_yiyong(tx, player, card, source, skill),

        EffectRequest::ResolveShanxie { player, skill } => resolve_shanxie(tx, player, skill),

        EffectRequest::ResolveSheyi {
            player,
            target,
            skill,
        } => resolve_sheyi(tx, player, target, skill),

        EffectRequest::ResolveTianyin { player, skill } => resolve_tianyin(tx, player, skill),

        EffectRequest::ResolveJiaohua {
            player,
            category,
            target,
            skill,
        } => resolve_jiaohua(tx, player, category, target, skill),

        EffectRequest::ResolveLuanqun {
            player,
            effect_skill_def,
            direct_hit_skill_def,
            stage,
            participants,
            cards,
            skill,
        } => resolve_luanqun(
            tx,
            player,
            effect_skill_def,
            direct_hit_skill_def,
            stage,
            participants,
            cards,
            skill,
        ),

        EffectRequest::ResolveGanglieThree {
            player,
            target,
            skill,
        } => resolve_ganglie_three(tx, player, target, skill),

        EffectRequest::ResolveZhongyi {
            player,
            action,
            helper_skill_def,
            card,
            skill,
        } => resolve_zhongyi(tx, player, action, helper_skill_def, card, skill),

        EffectRequest::DiscardEquipmentSlot { player, slot } => {
            discard_equipment_slot(tx, player, slot)
        }

        EffectRequest::ResolveOldAnxu {
            player,
            stage,
            giver,
            receiver,
            card,
            skill,
        } => resolve_oldanxu(tx, player, stage, giver, receiver, card, skill),

        EffectRequest::StoreSelectedInSkillPile {
            player,
            cards,
            skill,
        } => store_selected_in_skill_pile(tx, player, cards, skill),

        EffectRequest::GainTopCardToSkillPile { player, skill } => {
            gain_top_card_to_skill_pile(tx, player, skill)
        }

        EffectRequest::ChooseDiscardSkillPileCardAddMarkedSkill {
            player,
            target,
            helper_skill_def,
            selected,
            skill,
        } => choose_discard_skill_pile_card_add_marked_skill(
            tx,
            player,
            target,
            helper_skill_def,
            selected,
            skill,
        ),

        EffectRequest::GainOneHandFromSelectedTargets {
            player,
            targets,
            index,
            selected,
            skill,
        } => gain_one_hand_from_selected_targets(tx, player, targets, index, selected, skill),

        EffectRequest::ChooseRepoluTargetsDraw { player, skill } => {
            choose_repolu_targets_draw(tx, player, skill)
        }

        EffectRequest::ResolveZhuiyi {
            player,
            killer,
            skill,
        } => resolve_zhuiyi(tx, player, killer, skill),

        EffectRequest::ResolveDecadezhenjun {
            player,
            target,
            stage,
            count,
            skill,
        } => resolve_decadezhenjun(tx, player, target, stage, count, skill),

        EffectRequest::ResolveSpXiaoguo {
            player,
            target,
            stage,
            skill,
        } => resolve_sp_xiaoguo(tx, player, target, stage, skill),

        EffectRequest::ResolveXinfuGuolun {
            player,
            target,
            player_card,
            target_card,
            skill,
        } => resolve_xinfu_guolun(tx, player, target, player_card, target_card, skill),

        EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRedRestoreIfEqualRank {
            player,
            sha_user,
            card,
            restore_skill_def,
            skill,
        } => choose_discard_one_for_sha_refund_and_draw_if_red_restore_if_equal_rank(
            tx,
            player,
            sha_user,
            card,
            restore_skill_def,
            skill,
        ),

        EffectRequest::EndMatchForCampIfSkillPileAtLeast {
            player,
            pile_skill_def,
            count,
        } => end_match_for_camp_if_skill_pile_at_least(tx, player, pile_skill_def, count),

        EffectRequest::ResolveOldQinqing {
            player,
            target,
            lord,
            stage,
            skill,
        } => resolve_oldqinqing(tx, player, target, lord, stage, skill),

        EffectRequest::ResolveOldMingjian {
            player,
            target,
            stage,
            skill,
        } => resolve_oldmingjian(tx, player, target, stage, skill),

        EffectRequest::ResolveOldXuanfeng {
            player,
            target,
            card,
            stage,
            branch,
            skill,
        } => resolve_oldxuanfeng(tx, player, target, card, stage, branch, skill),

        EffectRequest::ResolveJianrou {
            player,
            damage_id,
            skill,
        } => resolve_jianrou(tx, player, damage_id, skill),

        EffectRequest::ResolveOldHuisheng {
            player,
            source,
            damage_id,
            stage,
            cards,
            skill,
        } => resolve_oldhuisheng(tx, player, source, damage_id, stage, cards, skill),

        EffectRequest::ResolveOldJiefan {
            player,
            dying,
            current,
            card,
            damage_id,
            stage,
            skill,
        } => resolve_oldjiefan(tx, player, dying, current, card, damage_id, stage, skill),

        EffectRequest::ResolveChegu {
            player,
            card,
            targets,
            cards,
            branch,
            index,
            stage,
            skill,
        } => resolve_chegu(
            tx, player, card, targets, cards, branch, index, stage, skill,
        ),

        EffectRequest::ResolveJunkguixin {
            player,
            target,
            stage,
            group,
            skill,
        } => resolve_junkguixin(tx, player, target, stage, group, skill),

        EffectRequest::ResolveYaoChenwei {
            player,
            target,
            card,
            mode,
            stage,
            skill,
        } => resolve_yao_chenwei(tx, player, target, card, mode, stage, skill),

        EffectRequest::ResolveSelectedEquipmentDiscardThenChooseDamage {
            player,
            damage_amount,
            nature,
            stage,
            index,
            targets,
            skill,
        } => resolve_selected_equipment_discard_then_choose_damage(
            tx,
            player,
            damage_amount,
            nature,
            stage,
            index,
            targets,
            skill,
        ),

        EffectRequest::ReplaceDrawWithDistinctSuitSelection {
            player,
            count,
            stage,
            cards,
            skill,
        } => replace_draw_with_distinct_suit_selection(tx, player, count, stage, cards, skill),

        EffectRequest::InspectTargetHandChooseHeartDiscardOrTop {
            player,
            target,
            stage,
            card,
            skill,
        } => {
            inspect_target_hand_choose_heart_discard_or_top(tx, player, target, stage, card, skill)
        }

        EffectRequest::ResolveYeyan {
            player,
            mode,
            stage,
            index,
            targets,
            data,
            skill,
        } => resolve_yeyan(tx, player, mode, stage, index, targets, data, skill),

        EffectRequest::ChooseAllLoseHpOrRecover { player, skill } => {
            choose_all_lose_hp_or_recover(tx, player, skill)
        }

        EffectRequest::ResolveQixing {
            player,
            initialize,
            stage,
            hand_cards,
            pile_cards,
            skill,
        } => resolve_qixing(tx, player, initialize, stage, hand_cards, pile_cards, skill),

        EffectRequest::ResolveQixingWeather {
            player,
            qixing_skill_def,
            helper_skill_def,
            max_targets,
            stage,
            cards,
            targets,
            skill,
        } => resolve_qixing_weather(
            tx,
            player,
            qixing_skill_def,
            helper_skill_def,
            max_targets,
            stage,
            cards,
            targets,
            skill,
        ),

        EffectRequest::ResolveNzryLongnu {
            player,
            fire_skill_def,
            thunder_skill_def,
            stage,
            skill,
        } => resolve_nzry_longnu(tx, player, fire_skill_def, thunder_skill_def, stage, skill),

        EffectRequest::ChooseLinkOther { player, skill } => choose_link_other(tx, player, skill),

        EffectRequest::ResolveDrltPoxi {
            player,
            target,
            hand_limit_skill_def,
            stage,
            own_count,
            skill,
        } => resolve_drlt_poxi(
            tx,
            player,
            target,
            hand_limit_skill_def,
            stage,
            own_count,
            skill,
        ),

        EffectRequest::ResolveDrltJieying {
            player,
            event_player,
            mark,
            mode,
            skill,
        } => resolve_drlt_jieying(tx, player, event_player, mark, mode, skill),

        EffectRequest::ResolveDrltDuorui {
            player,
            target,
            helper_skill_def,
            stage,
            selected_skill_def,
            skill,
        } => resolve_drlt_duorui(
            tx,
            player,
            target,
            helper_skill_def,
            stage,
            selected_skill_def,
            skill,
        ),

        EffectRequest::ResolveDrltZhiti {
            player,
            event_player,
            event_source,
            event_amount,
            mode,
            skill,
        } => resolve_drlt_zhiti(
            tx,
            player,
            event_player,
            event_source,
            event_amount,
            mode,
            skill,
        ),

        EffectRequest::ResolveShenSimayi {
            player,
            event_player,
            event_amount,
            mark,
            mode,
            cards,
            skill,
            ..
        } => resolve_shen_simayi(
            tx,
            player,
            event_player,
            event_amount,
            mark,
            mode,
            cards,
            skill,
        ),

        EffectRequest::ResolveBeige {
            player,
            target,
            source,
            stage,
            result_card,
            suit,
            rank,
            rewriters,
            skill,
        } => resolve_beige(
            tx,
            player,
            target,
            source,
            stage,
            result_card,
            suit,
            rank,
            rewriters,
            skill,
        ),

        EffectRequest::ResolveBeigeConfigured {
            player,
            target,
            source,
            stage,
            result_card,
            suit,
            rank,
            rewriters,
            heart_recover_amount,
            diamond_draw_count,
            skill,
        } => resolve_beige_configured(
            tx,
            player,
            target,
            source,
            stage,
            result_card,
            suit,
            rank,
            rewriters,
            heart_recover_amount,
            diamond_draw_count,
            skill,
        ),

        EffectRequest::LoseHpGainDiscardCardForOpeningExtraPlay {
            player,
            kind,
            mandatory_skill_def,
            skill,
        } => lose_hp_gain_discard_card_for_opening_extra_play(
            tx,
            player,
            kind,
            mandatory_skill_def,
            skill,
        ),

        EffectRequest::DrawForLifetimeCardActionDivisors { player, divisors } => {
            draw_for_lifetime_card_action_divisors(tx, player, divisors)
        }

        EffectRequest::GainDrawCardsForLifetimeCardActionDivisors {
            player,
            divisors,
            groups,
        } => gain_draw_cards_for_lifetime_card_action_divisors(tx, player, divisors, groups),

        EffectRequest::ChooseAddMissingSkill {
            player,
            skill_defs,
            skill,
        } => choose_add_missing_skill(tx, player, skill_defs, skill),

        EffectRequest::AddFirstMissingSkill {
            player,
            skill_defs,
            skill,
        } => add_first_missing_skill(tx, player, skill_defs, skill),

        EffectRequest::ResolveDccunsi {
            player,
            target,
            guixiu_def,
            yongjue_def,
            stage,
            skill,
        } => resolve_dccunsi(tx, player, target, guixiu_def, yongjue_def, stage, skill),

        EffectRequest::RecordCurrentShaAllTargets {
            player,
            card,
            skill,
        } => record_current_sha_all_targets(tx, player, card, skill),

        EffectRequest::ChooseGiveOrDiscardHand {
            player,
            payment_count,
            scope,
            skill,
        } => {
            if hand_cards_sorted(tx, player).len() < payment_count as usize {
                Ok(Vec::new())
            } else {
                draw_then_choose_give_or_discard_hand(
                    tx,
                    player,
                    0,
                    payment_count,
                    skill,
                    true,
                    scope,
                )
            }
        }

        EffectRequest::ChooseDiscardHandEquipThenRecoverTarget {
            chooser,
            target,
            count,
            amount,
            skill,
        } => {
            choose_discard_hand_equip_then_recover_target(tx, chooser, target, count, amount, skill)
        }

        EffectRequest::JudgeRecoverOnSuit {
            source,
            player,
            suit,
            amount,
        } => judge_recover_on_suit(tx, source, player, suit, amount),

        EffectRequest::JudgeRecoverOtherOnSuit {
            source,
            judge_player,
            recover_player,
            suit,
            amount,
        } => judge_recover_other_on_suit(tx, source, judge_player, recover_player, suit, amount),

        EffectRequest::ForceOthersUseShaOnNearestOrLoseHp {
            player,
            remaining,
            started,
            skill,
        } => force_others_use_sha_on_nearest_or_lose_hp(tx, player, remaining, started, skill),

        EffectRequest::JudgeDamageOtherOnSuit {
            player,
            suit,
            amount,
            nature,
        } => judge_damage_other_on_suit(tx, player, suit, amount, nature),

        EffectRequest::JudgeSourceDiscardTwoHandOrDamage { player, source } => {
            judge_source_discard_two_hand_or_damage(tx, player, source)
        }

        EffectRequest::GiveSelectedRecoverAfterPhaseTotalTwo {
            player,
            target,
            cards,
            skill,
            ready_to_recover,
        } => give_selected_recover_after_phase_total_two(
            tx,
            player,
            target,
            cards,
            skill,
            ready_to_recover,
        ),

        EffectRequest::ChooseOtherHandTargetsGainOneReplaceDraw {
            player,
            targets,
            index,
            selected,
            skill,
        } => choose_other_hand_targets_gain_one_replace_draw(
            tx, player, targets, index, selected, skill, None,
        ),

        EffectRequest::ChooseOtherHandTargetsGainOneReduceDraw {
            player,
            targets,
            index,
            selected,
            draw_count,
            skill,
        } => choose_other_hand_targets_gain_one_replace_draw(
            tx,
            player,
            targets,
            index,
            selected,
            skill,
            Some(draw_count),
        ),

        EffectRequest::DistributeTopTwoCards {
            player,
            cards,
            selected,
            target,
            skill,
            ready_to_choose,
        } => distribute_top_two_cards(tx, player, cards, selected, target, skill, ready_to_choose),

        EffectRequest::RecastCards { player, cards } => recast_cards(tx, player, cards),

        EffectRequest::DisableEquipSlots { player, slots } => {
            disable_equip_slots(tx, player, slots)
        }

        EffectRequest::MoveEquipmentThenDraw { player, skill } => {
            move_equipment_then_draw(tx, player, skill)
        }

        EffectRequest::DiscardEquipmentPairWithArmor {
            player,
            targets,
            skill,
        } => discard_equipment_pair_with_armor(tx, player, targets, skill),

        EffectRequest::DiscardTargetEquipmentGainMount { player, target } => {
            discard_target_equipment_gain_mount(tx, player, target)
        }

        EffectRequest::DiscardTargetEquipmentDrawWeaponRange {
            player,
            target,
            skill,
        } => discard_target_equipment_draw_weapon_range(tx, player, target, skill),

        EffectRequest::ChooseTargetRevealHandDamageIfSha {
            player,
            amount,
            skill,
        } => choose_target_reveal_hand_damage_if_sha(tx, player, amount, skill),

        EffectRequest::ChooseOtherGiveOneHandEquip { player, skill } => {
            choose_other_give_one_hand_equip(tx, player, skill)
        }

        EffectRequest::ChooseRevealHandCardBindCategoryTarget {
            player,
            target,
            skill,
        } => choose_reveal_hand_card_bind_category_target(tx, player, target, skill),

        EffectRequest::RecoverToMaxThenOptionalExchangeSkill {
            player,
            skill,
            removable_skill_def,
            restore_skill_def,
            ready_to_choose,
        } => recover_to_max_then_optional_exchange_skill(
            tx,
            player,
            skill,
            removable_skill_def,
            restore_skill_def,
            ready_to_choose,
        ),

        EffectRequest::DiscardTargetCardGainMount {
            player,
            target,
            scope,
        } => discard_target_card_gain_mount(tx, player, target, scope),

        EffectRequest::ChooseDiscardTargetCard {
            player,
            target,
            scope,
        } => choose_discard_target_card(tx, player, target, scope),

        EffectRequest::JudgePreventDamageLoseMaxHpOnNonHeart {
            player,
            target,
            damage_id,
        } => judge_prevent_damage_lose_max_hp_on_nonheart(tx, player, target, damage_id),

        EffectRequest::ChooseGainTopCardsBySuit {
            player,
            count,
            suit,
        } => choose_gain_top_cards_by_suit(tx, player, count, suit),

        EffectRequest::ChooseOtherDrawByHandCount {
            player,
            normal_count,
            empty_count,
            times,
            skill,
        } => choose_other_draw_by_hand_count(tx, player, normal_count, empty_count, times, skill),

        EffectRequest::ChooseDiscardOneThenGainTargetEquipment {
            player,
            target,
            slot,
        } => choose_discard_one_then_gain_target_equipment(tx, player, target, slot),

        EffectRequest::ChooseTargetDraw {
            player,
            count,
            scope,
            skill,
        } => choose_target_draw(tx, player, count, scope, skill),

        EffectRequest::ChooseTargetsDraw {
            player,
            count,
            target_count,
            scope,
            skill,
        } => choose_targets_draw(tx, player, count, target_count, scope, skill),

        EffectRequest::ChooseTargetsDamage {
            player,
            min_count,
            max_count,
            scope,
            amount,
            skill,
        } => choose_targets_damage(tx, player, min_count, max_count, scope, amount, skill),

        EffectRequest::ChooseOptionalTargetsDraw {
            player,
            count,
            min_count,
            max_count,
            scope,
            skill,
        } => choose_optional_targets_draw(tx, player, count, min_count, max_count, scope, skill),

        EffectRequest::RevealTargetHandShaOrDiscardBind {
            player,
            target,
            helper_skill_def,
            skill,
            stage,
        } => reveal_target_hand_sha_or_discard_bind(
            tx,
            player,
            target,
            helper_skill_def,
            skill,
            stage,
        ),

        EffectRequest::ChooseTargetDrawToMaxHpCap { player, cap, skill } => {
            choose_target_draw_to_max_hp_cap(tx, player, cap, skill)
        }

        EffectRequest::ChooseOtherDrawOwnerLostHpTurnOver { player, skill } => {
            choose_other_draw_owner_lost_hp_turn_over(tx, player, skill)
        }

        EffectRequest::DrawByPlayPhaseCardCategoryCount { player, .. } => {
            let count = tx.state().turn.play_card_category_count(player);
            draw_cards(tx, player, count)
        }

        EffectRequest::RefundCurrentShaUseAndDrawIfRed {
            player,
            sha_user,
            card,
            ..
        } => refund_current_sha_use_and_draw_if_red(tx, player, sha_user, card),

        EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRed {
            player,
            sha_user,
            card,
            skill,
        } => choose_discard_one_for_sha_refund_and_draw_if_red(tx, player, sha_user, card, skill),

        EffectRequest::ResolveOllinjie {
            player,
            source,
            skill,
        } => resolve_ollinjie(tx, player, source, skill),

        EffectRequest::ChooseTargetGainMaxHp { player, skill } => {
            choose_target_gain_max_hp(tx, player, skill)
        }

        EffectRequest::GainTargetHandCard { player, skill } => {
            gain_target_hand_card(tx, player, skill)
        }

        EffectRequest::ChooseClanNotMaxHandLimitGain { player, skill } => {
            choose_clan_not_max_hand_limit_gain(tx, player, skill)
        }

        EffectRequest::ExtraResolveCurrentTrick {
            player,
            card,
            skill,
        } => extra_resolve_current_trick(tx, player, card, skill),

        EffectRequest::PlaceHandCardToExpansion { player, skill } => {
            place_hand_card_to_expansion(tx, player, skill)
        }

        EffectRequest::GainFromSkillPile { player, skill } => {
            gain_from_skill_pile(tx, player, skill)
        }

        EffectRequest::DiscardSkillPile {
            player,
            skill,
            pile_def,
        } => discard_skill_pile(tx, player, skill, pile_def),

        EffectRequest::PlaceTopDeckToExpansion {
            player,
            skill,
            pile_def,
            count,
        } => place_top_deck_to_expansion(tx, player, skill, pile_def, count),

        EffectRequest::ChooseOneFromSkillPileDiscard {
            player,
            skill,
            pile_def,
        } => choose_one_from_skill_pile_discard(tx, player, skill, pile_def),

        EffectRequest::GainWholeSkillPile {
            player,
            skill,
            pile_def,
        } => gain_whole_skill_pile(tx, player, skill, pile_def),

        EffectRequest::RearrangePileToDeckTop {
            player,
            skill,
            pile_def,
            count,
        } => rearrange_pile_to_deck_top(tx, player, skill, pile_def, count),

        EffectRequest::ChooseHandCardsToSkillPile {
            player,
            skill,
            min_count,
            max_count,
        } => choose_hand_cards_to_skill_pile(tx, player, skill, min_count, max_count),

        EffectRequest::ResolveDcbingji {
            player,
            skill,
            stage,
            kind,
        } => resolve_dcbingji(tx, player, skill, stage, kind),

        EffectRequest::ResolvePxqixian { player, skill, .. } => resolve_pxqixian(tx, player, skill),

        EffectRequest::ResolveQuanjia {
            player,
            skill,
            source,
            target,
        } => resolve_quanjia(tx, player, skill, source, target),

        EffectRequest::ResolveDuwang { player, skill } => resolve_duwang(tx, player, skill),

        EffectRequest::ResolveJsrgdaoren { player, skill } => resolve_jsrgdaoren(tx, player, skill),

        EffectRequest::ResolveQianlong { player, skill } => resolve_qianlong(tx, player, skill),

        EffectRequest::ResolveJsrgzuozhan { player, skill } => {
            resolve_jsrgzuozhan(tx, player, skill)
        }

        EffectRequest::ResolveJsrgzuozhanGain {
            player,
            skill,
            event_player,
        } => resolve_jsrgzuozhan_gain(tx, player, skill, event_player),

        EffectRequest::ResolveTwmouzhi {
            player,
            skill,
            mode,
            event_card,
        } => resolve_twmouzhi(tx, player, skill, mode, event_card),

        EffectRequest::ResolveTwdaigui {
            player,
            skill,
            stage,
            count,
        } => resolve_twdaigui(tx, player, skill, stage, count),

        EffectRequest::ResolveTwmitu {
            player,
            skill,
            stage,
            count,
        } => resolve_twmitu(tx, player, skill, stage, count),

        EffectRequest::ResolveQiantun { player, skill } => resolve_qiantun(tx, player, skill, 0),

        EffectRequest::ResolveYangming { player, skill } => resolve_yangming(tx, player, skill, 0),

        EffectRequest::ResolveCanyun { player, skill } => resolve_canyun(tx, player, skill, 0),

        EffectRequest::ResolveChuanyun {
            player,
            skill,
            event_player,
            event_card,
        } => resolve_chuanyun(tx, player, skill, event_player, event_card),

        EffectRequest::ResolveQinmian {
            player,
            skill,
            mode,
            amount,
        } => resolve_qinmian(tx, player, skill, mode, amount),

        EffectRequest::ResolveBiaozhao2 {
            player,
            skill,
            event_player,
        } => resolve_biaozhao2(tx, player, skill, event_player),

        EffectRequest::ResolveChumei {
            player,
            skill,
            stage,
        } => resolve_chumei(tx, player, skill, stage),

        EffectRequest::ResolveShuaiyan {
            player,
            skill,
            stage,
        } => resolve_shuaiyan(tx, player, skill, stage),

        EffectRequest::ResolveMbbeiming {
            player,
            skill,
            stage,
        } => resolve_mbbeiming(tx, player, skill, stage),

        EffectRequest::ResetLinkedSelectedTargets { source, targets } => {
            reset_linked_selected_targets(tx, source, &targets)
        }

        EffectRequest::ResolveGaoyuan {
            player,
            source,
            card,
        } => resolve_gaoyuan(tx, player, source, card),

        EffectRequest::ResolveDuoduan {
            player,
            source,
            card,
            skill,
        } => resolve_duoduan(tx, player, source, card, skill),

        EffectRequest::ResolveDcanxu {
            player,
            targets,
            skill,
        } => resolve_dcanxu(tx, player, &targets),

        EffectRequest::ResolveChulao { player, targets } => resolve_chulao(tx, player, &targets),

        EffectRequest::ResolveMeiyong { player } => resolve_meiyong(tx, player),

        EffectRequest::ResolveTwxiongjun { player } => resolve_twxiongjun(tx, player),

        EffectRequest::ResolveDcqiaolve { player, target } => resolve_dcqiaolve(tx, player, target),

        EffectRequest::ResolveDingce { player, source } => resolve_dingce(tx, player, source),

        EffectRequest::ResolveYixiang { player } => resolve_yixiang(tx, player),

        EffectRequest::ResolveXinfuQiai { player } => resolve_xinfu_qiai(tx, player),

        EffectRequest::ResolveOlwenren {
            player,
            targets,
            skill: _,
        } => resolve_olwenren(tx, player, &targets),

        EffectRequest::ResolveSpolaige { player } => resolve_spolaige(tx, player),

        EffectRequest::ResolveOlzhijie { player } => resolve_olzhijie(tx, player),

        EffectRequest::ResolveOlsuyi { player, target } => resolve_olsuyi(tx, player, target),

        EffectRequest::ResolveOlqiaozhi { player, stage } => resolve_olqiaozhi(tx, player, stage),

        EffectRequest::ResolveDczhee { player, mode } => resolve_dczhee(tx, player, mode),

        EffectRequest::ResolveDchaoxian { player } => resolve_dchaoxian(tx, player),

        EffectRequest::ResolveZhongjie { player } => resolve_zhongjie(tx, player),

        EffectRequest::ResolveStarcuxiaWeiming {
            player,
            source,
            variant,
        } => resolve_starcuxia_weiming(tx, player, source, variant),

        EffectRequest::ResolveSanshou { player } => resolve_sanshou(tx, player),

        EffectRequest::ResolveNhlundao { player, source } => resolve_nhlundao(tx, player, source),

        EffectRequest::ResolveXiangshu { player, targets } => {
            resolve_xiangshu(tx, player, &targets)
        }

        EffectRequest::ResolveQiangyong {
            player,
            target,
            skill,
        } => resolve_qiangyong(tx, player, target, skill),

        EffectRequest::ResolveMbzhenann { player } => resolve_mbzhenann(tx, player),

        EffectRequest::ResolveFriendbiwei {
            player,
            target,
            card,
            skill,
        } => resolve_friendbiwei(tx, player, target, card, skill),

        EffectRequest::CommitGaoyuanRedirect {
            player,
            source,
            card,
            target,
        } => {
            let candidates = gaoyuan_redirect_candidates(tx, player, source, card)?;
            if !candidates.contains(&target) {
                return Ok(Vec::new());
            }
            let targets =
                crate::command::play::redirect_saved_sha_target(tx, source, card, player, target)?;
            Ok(vec![DomainEvent::CardTargetAdded {
                player: source,
                card,
                target,
                targets,
            }])
        }

        EffectRequest::ResolveZhengjian {
            player,
            skill,
            stage,
            event_player,
        } => resolve_zhengjian(tx, player, skill, stage, event_player),

        EffectRequest::ResolveReliangyin {
            player,
            skill,
            mode,
        } => resolve_reliangyin(tx, player, skill, mode),

        EffectRequest::ResolveZhongyan {
            player,
            skill,
            stage,
        } => resolve_zhongyan(tx, player, skill, stage),

        EffectRequest::ResolveDcmoukui {
            player,
            skill,
            stage,
            target,
        } => resolve_dcmoukui(tx, player, skill, stage, target),

        EffectRequest::ResolveDcniji {
            player,
            skill,
            mode,
            event_card,
        } => resolve_dcniji(tx, player, skill, mode, event_card),

        EffectRequest::ResolveHengwu {
            player,
            skill,
            event_card,
        } => resolve_hengwu(tx, player, skill, event_card),

        EffectRequest::ResolveBiaozhao3 { player, skill } => {
            resolve_biaozhao3(tx, player, skill, 0)
        }

        EffectRequest::PlaceHandCardsToDeckTop {
            player,
            skill,
            count,
        } => place_hand_cards_to_deck_top(tx, player, skill, count),

        EffectRequest::ChooseTargetHandCardToPile {
            player,
            skill,
            pile_def,
        } => choose_target_hand_card_to_pile(tx, player, skill, pile_def),

        EffectRequest::ChooseOtherAddMark {
            player,
            mark,
            delta,
            skill,
        } => choose_other_add_mark(tx, player, mark, delta, skill),

        EffectRequest::ResolveJsrgchaozheng { player, skill } => {
            resolve_jsrgchaozheng(tx, player, skill)
        }

        EffectRequest::StartDebateAllOthers { player, skill } => {
            start_debate_all_others(tx, player, skill)
        }

        EffectRequest::DiscardBasicDrawEqual { player, skill } => {
            discard_basic_draw_equal(tx, player, skill)
        }

        EffectRequest::StartDebateTargets { player, skill } => {
            start_debate_targets(tx, player, skill)
        }
        _ => unreachable!("effect routed to a different primitive group"),
    }
}
