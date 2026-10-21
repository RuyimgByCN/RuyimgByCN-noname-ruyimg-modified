//! Primitive dispatch for `DrawCards` and related requests.

use super::*;

pub(super) fn apply_effect(
    tx: &mut Transaction,
    req: EffectRequest,
) -> Result<Vec<DomainEvent>, EffectError> {
    match req {
        EffectRequest::DrawCards { player, count }
        | EffectRequest::DrawCardsFromSkill {
            player,
            count,
            skill: _,
        } => draw_cards(tx, player, count),

        EffectRequest::DrawCardsFromBottom { player, count } => {
            draw_cards_from_bottom(tx, player, count)
        }

        EffectRequest::SetDrawPhaseFromBottom { player } => {
            tx.state_mut()
                .turn
                .set_flag(turn_flag::DRAW_PHASE_FROM_BOTTOM, 1);
            Ok(Vec::new())
        }

        EffectRequest::ResolveChenglve {
            player,
            stage,
            suits,
            skill,
        } => resolve_chenglve(tx, player, stage, suits, skill),

        EffectRequest::ResolveShicai {
            player,
            card,
            placed,
            stage,
            skill,
        } => resolve_shicai(tx, player, card, placed, stage, skill),

        EffectRequest::ResolveXiongluan {
            player,
            target,
            stage,
            skill,
        } => resolve_xiongluan(tx, player, target, stage, skill),

        EffectRequest::ResolveGzbuqu {
            player,
            delta,
            stage,
            skill,
        } => resolve_gzbuqu(tx, player, delta, stage, skill),

        EffectRequest::ResolveGzbuquRecover {
            player,
            stage,
            skill,
        } => resolve_gzbuqu_recover(tx, player, stage, skill),

        EffectRequest::ResolveSbbiyue { player, skill } => resolve_sbbiyue(tx, player, skill),

        EffectRequest::ResolveSxrmqiewang {
            player,
            event_player,
            skill,
        } => resolve_sxrmqiewang(tx, player, event_player, skill),

        EffectRequest::ResolveSxrmqianliu {
            player,
            source,
            stage,
            skill,
        } => resolve_sxrmqianliu(tx, player, source, stage, skill),

        EffectRequest::ResolveTwjuxiang {
            player,
            event_player,
            stage,
            skill,
        } => resolve_twjuxiang(tx, player, event_player, stage, skill),

        EffectRequest::DrawRememberedHandCards {
            player,
            count,
            skill,
        } => draw_remembered_hand_cards(tx, player, count, skill),

        EffectRequest::DiscardRememberedHandCards { player, skill } => {
            discard_remembered_hand_cards(tx, player, skill)
        }

        EffectRequest::DrawEachBySeat {
            first,
            second,
            count,
            skill: _,
        } => draw_each_by_seat(tx, first, second, count),

        EffectRequest::DrawThenGiveOne {
            player,
            target,
            count,
            drawn_only,
            use_given_equipment,
            skill,
            drawn,
            ready_to_give,
        } => draw_then_give_one(
            tx,
            player,
            target,
            count,
            drawn_only,
            use_given_equipment,
            skill,
            drawn,
            ready_to_give,
        ),

        EffectRequest::DrawThenTakeOne {
            player,
            taker,
            count,
            skill,
            ready_to_take,
        } => draw_then_take_one(tx, player, taker, count, skill, ready_to_take),

        EffectRequest::TakeOneCard {
            from,
            to,
            equip_only,
        } => take_one_card(tx, from, to, if equip_only { 2 } else { 1 }),

        EffectRequest::TakeOneHandCard { from, to } => take_one_card(tx, from, to, 3),

        EffectRequest::ChooseOtherHandStoreSkillPile {
            player,
            target,
            skill,
        } => choose_other_hand_store_skill_pile(tx, player, target, skill),

        EffectRequest::GainAllSkillPileCards { player, skill } => {
            gain_all_skill_pile_cards(tx, player, skill)
        }

        EffectRequest::GainOtherSkillPileCards {
            player,
            target,
            skill_def,
            ..
        } => gain_other_skill_pile_cards(tx, player, target, skill_def),

        EffectRequest::TakeOneScopedCard { from, to, scope } => {
            take_one_card(tx, from, to, 10 + u32::from(scope.tag()))
        }

        EffectRequest::TakeTargetHandThenRepeatUntilMax {
            player,
            target,
            skill,
        } => take_target_hand_then_repeat_until_max(tx, player, target, skill),

        EffectRequest::ContinueTakeLargerHandUntilMax { player, skill } => {
            continue_take_larger_hand_until_max(tx, player, skill)
        }

        EffectRequest::ChooseDrawOrIncreaseCurrentCardDamageOrBothMark {
            player,
            card,
            draw_count,
            damage_amount,
            mark_def,
            mark_count,
            skill,
        } => choose_draw_or_increase_current_card_damage_or_both_mark(
            tx,
            player,
            card,
            draw_count,
            damage_amount,
            mark_def,
            mark_count,
            skill,
        ),

        EffectRequest::DrawThenChooseGiveOrDiscardHand {
            player,
            draw_count,
            payment_count,
            skill,
            ready_to_choose,
        } => draw_then_choose_give_or_discard_hand(
            tx,
            player,
            draw_count,
            payment_count,
            skill,
            ready_to_choose,
            DrawTargetScope::Other,
        ),

        EffectRequest::GainTargetEquipment {
            player,
            target,
            slot,
        } => gain_target_equipment(tx, player, target, slot),

        EffectRequest::DiscardAllTargetEquipment { actor, target } => {
            discard_all_target_equipment(tx, actor, target)
        }

        EffectRequest::IncreaseCurrentShaTargetDamage {
            source,
            card,
            target,
            amount,
        } => increase_current_sha_target_damage(tx, source, card, target, amount),

        EffectRequest::ChooseTargetDrawThenOwnerIfBelowMaxHp {
            player,
            target,
            count,
            skill,
            ready_to_check,
        } => choose_target_draw_then_owner_if_below_max_hp(
            tx,
            player,
            target,
            count,
            skill,
            ready_to_check,
        ),

        EffectRequest::ChooseTargetDrawMaxHpCapThenDiscardExcess {
            player,
            target,
            cap,
            skill,
            ready_to_discard,
        } => choose_target_draw_max_hp_cap_then_discard_excess(
            tx,
            player,
            target,
            cap,
            skill,
            ready_to_discard,
        ),

        EffectRequest::ChooseTargetDrawMaxHpCapThenAddSkill {
            player,
            target,
            cap,
            skill,
            add_def,
        } => choose_target_draw_max_hp_cap_then_add_skill(tx, player, target, cap, skill, add_def),

        EffectRequest::GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp {
            player,
            target,
            card,
            suit,
            skill,
            ready_to_choose,
        } => give_selected_then_target_choose_discard_same_suit_or_lose_hp(
            tx,
            player,
            target,
            card,
            suit,
            skill,
            ready_to_choose,
        ),

        EffectRequest::TargetChooseSuitGainOwnerHandDamageIfMismatch {
            player,
            target,
            skill,
        } => target_choose_suit_gain_owner_hand_damage_if_mismatch(tx, player, target, skill),

        EffectRequest::GainEquipmentFromDrawPile { player } => {
            gain_equipment_from_draw_pile(tx, player)
        }

        EffectRequest::GiveAllHandRecoverToOne {
            player,
            target,
            ready_to_recover,
        } => give_all_hand_recover_to_one(tx, player, target, ready_to_recover),

        EffectRequest::GiveAllHand { player, target } => give_all_hand(tx, player, target),

        EffectRequest::GainCardIfDiscard { player, card } => gain_event_cards(tx, player, &[card]),

        EffectRequest::ChooseDrawOrDiscardTargetCardRecordSha {
            player,
            card,
            target,
            skill,
        } => choose_draw_or_discard_target_card_record_sha(tx, player, card, target, skill),

        EffectRequest::RevealHandToPlayer { player, observer } => {
            reveal_hand_to_player(tx, player, observer)
        }

        EffectRequest::ChooseOptionalDiscardTargetCard {
            player,
            target,
            scope,
        } => choose_optional_discard_target_card(tx, player, target, scope),

        EffectRequest::RevealHandChooseDiscardColor { player } => {
            reveal_hand_choose_discard_color(tx, player)
        }

        EffectRequest::RevealHand { player } => reveal_hand(tx, player),

        EffectRequest::DrawLinkedPlayers { count } => draw_linked_players(tx, count),

        EffectRequest::RevealTopGainIfColor { player, color_mask } => {
            reveal_top_gain_if_color(tx, player, color_mask)
        }

        EffectRequest::SuppressNonLockedSkillsUntilTurnEnd { player } => {
            suppress_non_locked_skills_until_turn_end(tx, player)
        }

        EffectRequest::ChooseRecoverTarget {
            chooser,
            target,
            amount,
        } => choose_recover_target(tx, chooser, target, amount),

        EffectRequest::ChooseDiscardTargetCardsRecoverSameCategory {
            player,
            target,
            count,
            amount,
            skill,
        } => choose_discard_target_cards_recover_same_category(
            tx, player, target, count, amount, skill,
        ),

        EffectRequest::ChooseScskuijiDiscard {
            player,
            target,
            count,
            skill,
        } => choose_scskuiji_discard(tx, player, target, count, skill),

        EffectRequest::ChoosePskuijiDiscard {
            player,
            target,
            count,
            skill,
        } => choose_pskuiji_discard(tx, player, target, count, skill),

        EffectRequest::ChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp {
            player,
            target,
            skill,
            ready_to_check,
        } => choose_target_self_discard_then_owner_lose_hp_if_hand_ne_hp(
            tx,
            player,
            target,
            skill,
            ready_to_check,
        ),

        EffectRequest::ChooseGiveOneHandEquipOrRecover {
            owner,
            giver,
            target,
            skill,
        } => choose_give_one_hand_equip_or_recover(tx, owner, giver, target, skill),

        EffectRequest::ChooseGiveTwoDifferentColorsOrRecover {
            owner,
            giver,
            target,
            skill,
        } => choose_give_two_different_colors_or_recover(tx, owner, giver, target, skill),

        EffectRequest::ResolveMbzhuguo {
            player,
            target,
            stage,
            skill,
        } => resolve_mbzhuguo(tx, player, target, stage, skill),

        EffectRequest::ResolveMbanxian {
            player,
            cards,
            card,
            stage,
            skill,
        } => resolve_mbanxian(tx, player, cards, card, stage, skill),

        EffectRequest::ResolveMbjinzu {
            player,
            target,
            cards,
            card,
            helper_skill_def,
            stage,
            skill,
        } => resolve_mbjinzu(
            tx,
            player,
            target,
            cards,
            card,
            helper_skill_def,
            stage,
            skill,
        ),

        EffectRequest::ReplaceSkillWithBoundAfterResolutionIfTargetAlive {
            player,
            skill,
            skill_def,
            bound_target,
            ready_to_replace,
        } => replace_skill_with_bound_after_resolution_if_target_alive(
            tx,
            player,
            skill,
            skill_def,
            bound_target,
            ready_to_replace,
        ),

        EffectRequest::ChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp {
            player,
            skill,
            zone,
            discarded_count,
            ready_to_draw,
        } => choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp(
            tx,
            player,
            skill,
            zone,
            discarded_count,
            ready_to_draw,
        ),

        EffectRequest::GainEventCards { player, cards } => gain_event_cards(tx, player, &cards),

        EffectRequest::TurnOver { player } => turn_over(tx, player),

        EffectRequest::RestoreCharacterState { player } => restore_character_state(tx, player),

        EffectRequest::DiscardAllHandEquipJudge { player } => {
            discard_all_hand_equip_judge(tx, player)
        }

        EffectRequest::DamageThenDiscardAllJudge {
            player,
            ready_to_discard,
        } => damage_then_discard_all_judge(tx, player, ready_to_discard),

        EffectRequest::ReplaceDrawGainTopTwoAddSkillsIfDifferentColors {
            player,
            first_skill_def,
            second_skill_def,
            stage,
            cards,
            skill,
        } => replace_draw_gain_top_two_add_skills_if_different_colors(
            tx,
            player,
            first_skill_def,
            second_skill_def,
            stage,
            cards,
            skill,
        ),

        EffectRequest::ChooseGainTargetCardAlternateZoneThenTargetDraw {
            player,
            target,
            selected,
            skill,
        } => choose_gain_target_card_alternate_zone_then_target_draw(
            tx, player, target, selected, skill,
        ),

        EffectRequest::SkipNextJudgePhase { player } => skip_next_judge_phase(tx, player),

        EffectRequest::GainCardByKind { player, kind } => gain_card_by_kind(tx, player, kind),

        EffectRequest::GainDiscardCardByKind { player, kind } => {
            gain_discard_card_by_kind(tx, player, kind)
        }

        EffectRequest::RevealUniformHandChooseDamage {
            player,
            amount,
            skill,
        } => reveal_uniform_hand_choose_damage(tx, player, amount, skill),

        EffectRequest::ChooseGainDiscardCardByKind {
            player,
            kind,
            skill,
        } => choose_gain_discard_card_by_kind(tx, player, kind, skill),

        EffectRequest::ChooseGainOwnLostDiscardCardThisTurn { player, skill } => {
            choose_gain_own_lost_discard_card_this_turn(tx, player, skill)
        }

        EffectRequest::ChooseMoveFieldCardTo {
            player,
            target,
            skill,
        } => choose_move_field_card_to(tx, player, target, skill),

        EffectRequest::DrawThenOptionallyDistributeHandCards {
            player,
            draw_count,
            give_cap,
            cards,
            targets,
            skill,
            ready_to_choose,
            finish,
        } => draw_then_optionally_distribute_hand_cards(
            tx,
            player,
            draw_count,
            give_cap,
            cards,
            targets,
            skill,
            ready_to_choose,
            finish,
        ),

        EffectRequest::GiveSelectedThenContinueDistinctRecoverAtTwo {
            player,
            target,
            cards,
            used_targets,
            total_given,
            skill,
            continue_only,
        } => give_selected_then_continue_distinct_recover_at_two(
            tx,
            player,
            target,
            cards,
            used_targets,
            total_given,
            skill,
            continue_only,
        ),

        EffectRequest::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan {
            player,
            mark,
            skill,
        } => choose_max_mark_other_judge_kill_unless_tao_or_taoyuan(tx, player, mark, skill),

        EffectRequest::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver {
            player,
            priority,
            remaining_targets,
            skill,
        } => choose_zone_priority_gain_random_card_from_each_other_then_turn_over(
            tx,
            player,
            priority,
            remaining_targets,
            skill,
        ),

        EffectRequest::ChooseGainCardFromEachOtherThenTurnOver {
            player,
            initialized,
            remaining_targets,
            skill,
        } => choose_gain_card_from_each_other_then_turn_over(
            tx,
            player,
            initialized,
            remaining_targets,
            skill,
        ),

        EffectRequest::ChooseGainCardFromEachOtherReplaceDraw {
            player,
            initialized,
            remaining_targets,
            skill,
        } => choose_gain_card_from_each_other_replace_draw(
            tx,
            player,
            initialized,
            remaining_targets,
            skill,
        ),

        EffectRequest::GiveSelectedThenRecoverTarget {
            player,
            target,
            cards,
            amount,
            skill,
            ready_to_recover,
        } => give_selected_then_recover_target(
            tx,
            player,
            target,
            cards,
            amount,
            skill,
            ready_to_recover,
        ),

        EffectRequest::ResolveDingpan {
            player,
            target,
            stage,
            skill,
        } => resolve_dingpan(tx, player, target, stage, skill),

        EffectRequest::ResolveTanlu {
            player,
            target,
            amount,
            stage,
            skill,
        } => resolve_tanlu(tx, player, target, amount, stage, skill),

        EffectRequest::ResolveHiddenSkillPile {
            player,
            pile_skill_def,
            action,
            card,
            stage,
            hand_cards,
            pile_cards,
            event_cards,
            skill,
        } => resolve_hidden_skill_pile(
            tx,
            player,
            pile_skill_def,
            action,
            card,
            stage,
            hand_cards,
            pile_cards,
            event_cards,
            skill,
        ),

        EffectRequest::SetNonLockedSkillSuppression {
            player,
            enabled,
            skill,
        } => set_non_locked_skill_suppression(tx, player, enabled, skill),

        EffectRequest::ResolveDiliYuheng {
            player,
            action,
            card,
            event_cards,
            skill,
        } => resolve_dili_yuheng(tx, player, action, card, event_cards, skill),

        EffectRequest::FinalizeTiaojiyanmei {
            actor,
            left,
            right,
            card,
        } => finalize_tiaojiyanmei(tx, actor, left, right, card),

        EffectRequest::BeginTiaojiyanmeiDiscard {
            actor,
            left,
            right,
            higher,
        } => begin_tiaojiyanmei_discard(tx, actor, left, right, higher),

        EffectRequest::DiscardSelectedDrawAllHandBonus {
            player,
            cards,
            skill,
        } => discard_selected_draw_all_hand_bonus(tx, player, cards, skill),

        EffectRequest::ChooseOtherYinghun {
            player,
            target,
            count,
            branch,
            ready_to_discard,
            skill,
        } => choose_other_yinghun(tx, player, target, count, branch, ready_to_discard, skill),

        EffectRequest::AddSkillUntilTurnEnd { player, skill_def } => {
            add_skill_until_turn_end(tx, player, skill_def)
        }

        EffectRequest::ResolveMassDamageDiscardTurnOver {
            player,
            mark,
            mark_count,
            damage_amount,
            hand_count,
            stage,
            index,
            targets,
            skill,
        } => resolve_mass_damage_discard_turn_over(
            tx,
            player,
            mark,
            mark_count,
            damage_amount,
            hand_count,
            stage,
            index,
            targets,
            skill,
            false,
            true,
        ),

        EffectRequest::ResolveMassDamageDiscardHandEquip {
            player,
            mark,
            mark_count,
            damage_amount,
            discard_count,
            stage,
            index,
            targets,
            skill,
        } => resolve_mass_damage_discard_turn_over(
            tx,
            player,
            mark,
            mark_count,
            damage_amount,
            discard_count,
            stage,
            index,
            targets,
            skill,
            true,
            false,
        ),

        EffectRequest::RecordActiveTarget {
            player,
            target,
            skill,
        } => record_active_target(tx, player, target, skill),

        EffectRequest::ChooseDiscardWeaponOrLoseHp { player, skill } => {
            choose_discard_weapon_or_lose_hp(tx, player, skill)
        }

        EffectRequest::ChooseDiscardOneHeOrLoseHp {
            player,
            lose_amount,
            skill,
        } => choose_discard_one_he_or_lose_hp(tx, player, lose_amount, skill),

        EffectRequest::JudgeSpadeTwoToNineDrawTwoOrThunderShaTwice { source, target, .. } => {
            judge_spade_two_to_nine_draw_two_or_thunder_sha_twice(tx, source, target)
        }

        EffectRequest::ChooseSourceConfirmOwnerLoseHp {
            owner,
            source,
            amount,
            skill,
        } => choose_source_confirm_owner_lose_hp(tx, owner, source, amount, skill),

        EffectRequest::ResolveMarkParityControl {
            player,
            mark,
            mass_threshold,
            damage_amount,
            stage,
            index,
            target,
            targets,
            skill,
        } => resolve_mark_parity_control(
            tx,
            player,
            mark,
            mass_threshold,
            damage_amount,
            stage,
            index,
            target,
            targets,
            skill,
        ),

        EffectRequest::ChooseOtherBizheng {
            player,
            target,
            stage,
            skill,
        } => choose_other_bizheng(tx, player, target, stage, skill),

        EffectRequest::ChooseOtherXiashu {
            player,
            target,
            stage,
            cards,
            skill,
        } => choose_other_xiashu(tx, player, target, stage, cards, skill),

        EffectRequest::ResolveGuanchaoRank {
            player,
            card,
            skill,
        } => resolve_guanchao_rank(tx, player, card, skill),

        EffectRequest::ResolveGuanwei {
            player,
            target,
            stage,
            card,
            skill,
        } => resolve_guanwei(tx, player, target, stage, card, skill),

        EffectRequest::ResolveLanjiang {
            player,
            stage,
            index,
            target,
            targets,
            skill,
        } => resolve_lanjiang(tx, player, stage, index, target, targets, skill),

        EffectRequest::ResolveRebingyi { player, skill } => resolve_rebingyi(tx, player, skill),

        EffectRequest::ResolveRebotu { player, skill } => resolve_rebotu(tx, player, skill),

        EffectRequest::ResolveOldimeng {
            player,
            stage,
            first,
            second,
            skill,
        } => resolve_oldimeng(tx, player, stage, first, second, skill),

        EffectRequest::ResolveRejieyin {
            player,
            target,
            stage,
            card,
            equip,
            skill,
        } => resolve_rejieyin(tx, player, target, stage, card, equip, skill),

        EffectRequest::ResolveReanxu {
            player,
            stage,
            first,
            second,
            card,
            from_hand,
            skill,
        } => resolve_reanxu(tx, player, stage, first, second, card, from_hand, skill),

        EffectRequest::ResolveDrltJueyan {
            player,
            stage,
            slot,
            first,
            sha_skill_def,
            weapon_skill_def,
            mount_skill_def,
            armor_skill_def,
            treasure_skill_def,
            skill,
        } => resolve_drlt_jueyan(
            tx,
            player,
            stage,
            slot,
            first,
            sha_skill_def,
            weapon_skill_def,
            mount_skill_def,
            armor_skill_def,
            treasure_skill_def,
            skill,
        ),

        EffectRequest::ResolveRejizhi {
            player,
            stage,
            card,
            skill,
        } => resolve_rejizhi(tx, player, stage, card, skill),

        EffectRequest::ResolveJiexun {
            player,
            stage,
            target,
            draw_count,
            discard_count,
            funan_helper_skill_def,
            skill,
        } => resolve_jiexun(
            tx,
            player,
            stage,
            target,
            draw_count,
            discard_count,
            funan_helper_skill_def,
            skill,
        ),

        EffectRequest::ResolveWenguaSetup {
            player,
            helper_skill_def,
            skill,
        } => resolve_wengua_setup(tx, player, helper_skill_def, skill),

        EffectRequest::ResolveWengua {
            player,
            stage,
            target,
            card,
            branch,
            main_skill_def,
            marker_skill_def,
            skill,
        } => resolve_wengua(
            tx,
            player,
            stage,
            target,
            card,
            branch,
            main_skill_def,
            marker_skill_def,
            skill,
        ),

        EffectRequest::ResolveXinanguo {
            player,
            target,
            stage,
            index,
            remaining,
            skill,
        } => resolve_xinanguo(tx, player, target, stage, index, remaining, skill),

        EffectRequest::ResolveReanguo {
            player,
            target,
            stage,
            index,
            remaining,
            skill,
        } => resolve_reanguo(tx, player, target, stage, index, remaining, skill),

        EffectRequest::ResolveBeizhu {
            player,
            target,
            stage,
            cards,
            skill,
        } => resolve_beizhu(tx, player, target, stage, cards, skill),

        EffectRequest::ResolveMbshanjia {
            player,
            lost_equipment_count,
            distance_skill_def,
            stage,
            reward_mask,
            skill,
        } => resolve_mbshanjia(
            tx,
            player,
            lost_equipment_count,
            distance_skill_def,
            stage,
            reward_mask,
            skill,
        ),

        EffectRequest::ResolveDaoji {
            player,
            target,
            card,
            stage,
            tw,
            skill,
        } => resolve_daoji(tx, player, target, card, stage, tw, skill),

        EffectRequest::GiveAllHandThenChooseUseOneOrDamage {
            player,
            target,
            cards,
            targets,
            stage,
            card,
            skill,
        } => give_all_hand_then_choose_use_one_or_damage(
            tx, player, target, cards, targets, stage, card, skill,
        ),

        EffectRequest::ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable {
            player,
            card,
            target,
            skill,
        } => choose_category_then_target_give_matching_or_sha_unrespondable(
            tx, player, card, target, skill,
        ),

        EffectRequest::ChooseLoseHpUseVirtualShaByBasicVariants {
            player,
            card,
            targets,
            stage,
            damage_bonus,
            skill,
        } => choose_lose_hp_use_virtual_sha_by_basic_variants(
            tx,
            player,
            card,
            targets,
            stage,
            damage_bonus,
            skill,
        ),

        EffectRequest::ResolveTwfupan {
            player,
            damage_amount,
            card,
            target,
            stage,
            skill,
        } => resolve_twfupan(tx, player, damage_amount, card, target, stage, skill),

        EffectRequest::ResolveTwjiaohua {
            player,
            target,
            event_cards,
            skill,
        } => resolve_twjiaohua(tx, player, target, &event_cards, skill),

        EffectRequest::ResolveTwkujian {
            player,
            target,
            cards,
            action,
            skill,
        } => resolve_twkujian(tx, player, target, &cards, action, skill),

        EffectRequest::ResolveTwruilian {
            player,
            target,
            event_cards,
            action,
            category,
            skill,
        } => resolve_twruilian(tx, player, target, &event_cards, action, category, skill),

        EffectRequest::EquipWeaponFromDrawPile { player } => {
            if physical_equipped_weapons(tx, player).is_empty() {
                equip_random_from_draw_pile(tx, player, Some(EquipSlot::Weapon))
                    .map(|result| result.0)
            } else {
                Ok(Vec::new())
            }
        }

        EffectRequest::ResolveTwjianwei {
            player,
            actor,
            target,
            card,
            winner,
            stage,
            skill,
        } => resolve_twjianwei(tx, player, actor, target, card, winner, stage, skill),

        EffectRequest::ResolveTwtanfeng {
            player,
            target,
            card,
            stage,
            phase,
            skill,
        } => resolve_twtanfeng(tx, player, target, card, stage, phase, skill),

        EffectRequest::ResolveTwzhibian {
            player,
            target,
            winner,
            stage,
            skill,
        } => resolve_twzhibian(tx, player, target, winner, stage, skill),

        EffectRequest::ResolveTwyuyan {
            player,
            source,
            card,
            skill,
        } => resolve_twyuyan(tx, player, source, card, skill),

        EffectRequest::ResolveTwguoyi {
            player,
            target,
            card,
            hand_lock_skill_def,
            skill,
        } => resolve_twguoyi(tx, player, target, card, hand_lock_skill_def, skill),

        EffectRequest::ReplayVirtualCardOnTargets {
            player,
            kind,
            targets,
            skill: _,
        } => replay_virtual_card_on_targets(tx, player, kind, targets),

        EffectRequest::ResolveTwchuhai {
            player,
            action,
            stage,
            index,
            event_cards,
            skill,
        } => resolve_twchuhai(tx, player, action, stage, index, event_cards, skill),

        EffectRequest::ResolveTwyizhu {
            player,
            source,
            card,
            action,
            stage,
            event_cards,
            skill,
        } => resolve_twyizhu(tx, player, source, card, action, stage, event_cards, skill),

        EffectRequest::ResolveTwluanchou {
            player,
            helper_skill_def,
            action,
            targets,
            source,
            target,
            amount,
            nature,
            card,
            card_kind,
            card_flags,
            skill,
        } => resolve_twluanchou(
            tx,
            player,
            helper_skill_def,
            action,
            targets,
            source,
            target,
            amount,
            nature,
            card,
            card_kind,
            card_flags,
            skill,
        ),

        EffectRequest::ResolveTianxiang {
            player,
            target,
            discard_card,
            source,
            amount,
            nature,
            card,
            card_kind,
            card_flags,
            draw_skill_def,
            skill,
        } => resolve_tianxiang(
            tx,
            player,
            target,
            discard_card,
            source,
            amount,
            nature,
            card,
            card_kind,
            card_flags,
            draw_skill_def,
            skill,
        ),

        EffectRequest::ResolveRetianxiang {
            player,
            target,
            discard_card,
            branch,
            source,
            skill,
        } => resolve_retianxiang(tx, player, target, discard_card, branch, source, skill),

        EffectRequest::ResolveLianzhou {
            player,
            targets,
            skill,
        } => resolve_lianzhou(tx, player, targets, skill),

        EffectRequest::ResolveStddechong {
            player,
            target,
            cards,
            skill,
        } => resolve_stddechong(tx, player, target, cards, skill),

        EffectRequest::DiscardHandEquipCount {
            player,
            count,
            skill,
        } => discard_hand_equip_count(tx, player, count, skill),

        EffectRequest::ResolveJieliangGain {
            player,
            mover,
            cards,
            skill,
        } => resolve_jieliang_gain(tx, player, mover, cards, skill),

        EffectRequest::ResolveStdjujian {
            player,
            target,
            cards,
            skill,
        } => resolve_stdjujian(tx, player, target, cards, skill),

        EffectRequest::ResolveLulve {
            player,
            target,
            branch,
            skill,
        } => resolve_lulve(tx, player, target, branch, skill),

        EffectRequest::ResolveDctingxian {
            player,
            card,
            targets,
            skill,
        } => resolve_dctingxian(tx, player, card, targets, skill),

        EffectRequest::ResolveHfjieying {
            player,
            target,
            grant_def,
            skill,
        } => resolve_hfjieying(tx, player, target, grant_def, skill),

        EffectRequest::ResolveDcnuchen {
            player,
            target,
            skill,
        } => resolve_dcnuchen(tx, player, target, skill),

        EffectRequest::ResolveDcdanji { player, skill } => resolve_dcdanji(tx, player, skill),

        EffectRequest::ResolveDczhiheng { player, skill } => resolve_dczhiheng(tx, player, skill),

        EffectRequest::ResolveStdgaobian {
            player,
            chooser,
            skill,
        } => resolve_stdgaobian(tx, player, chooser, skill),

        EffectRequest::ResolveSpjiedao { player, skill } => resolve_spjiedao(tx, player, skill),

        EffectRequest::ResolveSibian { player, skill } => resolve_sibian(tx, player, skill),

        EffectRequest::RevealChooseCategoryDistributeSelfThresholdHpLoss {
            player,
            reveal_count,
            recipient_cap,
            skill,
        } => reveal_choose_category_distribute_self_threshold_hp_loss(
            tx,
            player,
            reveal_count,
            recipient_cap,
            skill,
        ),

        EffectRequest::SkipNextDiscardPhase { player } => skip_next_discard_phase(tx, player),

        EffectRequest::ResolveSpfengyin {
            player,
            target,
            skill,
        } => resolve_spfengyin(tx, player, target, skill),

        EffectRequest::ResolveXinfuLianpian {
            player,
            targets,
            skill,
        } => resolve_xinfu_lianpian(tx, player, targets, skill),

        EffectRequest::ResolveGongjian {
            player,
            targets,
            skill,
        } => resolve_gongjian(tx, player, targets, skill),

        EffectRequest::ResolveXinfuQinguo {
            player,
            card,
            skill,
        } => resolve_xinfu_qinguo(tx, player, card, skill),

        EffectRequest::ChooseOtherRandomGainRegionThenDrawCompareColorLoseHp { player, skill } => {
            choose_other_random_gain_region_then_draw_compare_color_lose_hp(tx, player, skill)
        }

        EffectRequest::ResolveRefuhai { player, skill } => resolve_refuhai(tx, player, skill),

        EffectRequest::ChooseGainTargetHandCeilHalfDifference {
            player,
            target,
            card,
            skill,
            ready_to_choose,
        } => choose_gain_target_hand_ceil_half_difference(
            tx,
            player,
            target,
            card,
            skill,
            ready_to_choose,
        ),

        EffectRequest::ResolveGzjinfa {
            player,
            target,
            cost_card,
            given_card,
            stage,
            skill,
        } => resolve_gzjinfa(tx, player, target, cost_card, given_card, stage, skill),

        EffectRequest::ResolveWnQifeng {
            player,
            target,
            skill,
        } => resolve_wn_qifeng(tx, player, target, skill),

        EffectRequest::ResolveDcjue {
            player,
            target,
            stage,
            remaining,
            skill,
        } => resolve_dcjue(tx, player, target, stage, remaining, skill),

        EffectRequest::ResolveRuanji {
            player,
            source,
            card,
            action,
            cards,
            skill,
        } => resolve_ruanji(tx, player, source, card, action, cards, skill),

        EffectRequest::ResolveZhengbi {
            player,
            target,
            card,
            action,
            skill,
        } => resolve_zhengbi(tx, player, target, card, action, skill),

        EffectRequest::ResolveFengying {
            player,
            helper_def,
            action,
            skill,
        } => resolve_fengying(tx, player, helper_def, action, skill),

        EffectRequest::ResolveQiaojian {
            player,
            target,
            helper_def,
            action,
            cards,
            split,
            skill,
        } => resolve_qiaojian(tx, player, target, helper_def, action, cards, split, skill),

        EffectRequest::ResolveXicha {
            player,
            source,
            card,
            action,
            cards,
            skill,
        } => resolve_xicha(tx, player, source, card, action, cards, skill),

        EffectRequest::ResolveHuzun {
            player,
            source,
            card,
            action,
            cards,
            targets,
            skill,
        } => resolve_huzun(tx, player, source, card, action, cards, targets, skill),

        EffectRequest::ResolveZhugemengxue {
            player,
            target,
            card,
            action,
            cards,
            skill,
        } => resolve_zhugemengxue(tx, player, target, card, action, cards, skill),

        EffectRequest::ResolveBailingyun {
            player,
            target,
            card,
            action,
            cards,
            skill,
        } => resolve_bailingyun(tx, player, target, card, action, cards, skill),

        EffectRequest::ResolveDcshaowei {
            player,
            action,
            cards,
            skill,
        } => resolve_dcshaowei(tx, player, action, cards, skill),

        EffectRequest::ResolveDcdichou {
            player,
            target,
            card,
            action,
            helper_skill_def,
            skill,
        } => resolve_dcdichou(tx, player, target, card, action, helper_skill_def, skill),

        EffectRequest::ResolveDcdufeng {
            player,
            action,
            helper_skill_def,
            skill,
        } => resolve_dcdufeng(tx, player, action, helper_skill_def, skill),

        EffectRequest::ChooseDiscardHandRange {
            player,
            min,
            max,
            skill,
        } => choose_discard_hand_range(tx, player, min, max, skill),

        EffectRequest::ResolveDcctjiuxian {
            player,
            event_player,
            target,
            card,
            action,
            cards,
            skill,
        } => resolve_dcctjiuxian(tx, player, event_player, target, card, action, cards, skill),

        EffectRequest::ResolveBianyue {
            player,
            target,
            action,
            skill,
        } => resolve_bianyue(tx, player, target, action, skill),

        EffectRequest::ResolveZhupeilan {
            player,
            target,
            card,
            helper_skill_def,
            action,
            event_cards,
            skill,
        } => resolve_zhupeilan(
            tx,
            player,
            target,
            card,
            helper_skill_def,
            action,
            event_cards,
            skill,
        ),

        EffectRequest::ResolveDcSbZhangxiu {
            player,
            target,
            action,
            cards,
            skill,
        } => resolve_dc_sb_zhangxiu(tx, player, target, action, cards, skill),

        EffectRequest::ResolveDcSbZhouyu {
            player,
            source,
            card,
            action,
            event_targets,
            damaged_targets,
            skill,
        } => resolve_dc_sb_zhouyu(
            tx,
            player,
            source,
            card,
            action,
            event_targets,
            damaged_targets,
            skill,
        ),

        EffectRequest::ResolveDcSbLusu {
            player,
            source,
            card,
            action,
            cards,
            amount,
            skill,
        } => resolve_dc_sb_lusu(tx, player, source, card, action, cards, amount, skill),
        _ => unreachable!("effect routed to a different primitive group"),
    }
}
