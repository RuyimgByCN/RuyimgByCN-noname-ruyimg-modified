//! Primitive effect implementations.
//!
//! Split out of a single 78k-line file (ADR 0594). This module keeps the
//! `apply_effect` dispatch table — the mapping from `EffectRequest` to primitive
//! implementations — while the implementations live in the `pack_*` submodules,
//! grouped by the content package that drove them. The split is positional, so no
//! item moved relative to its neighbours.

use crate::rng::RuleRng;
use crate::state::{
    turn_flag, Card, CardKind, ContinuationKind, EquipSlot, Identity, Phase, Player, Zone,
    ZoneKind, DEFAULT_DRAW_COUNT,
};
use crate::transaction::{JournalEntry, Transaction};
use engine_api::{
    CardDefId, CardId, CardMoveReason, ChoiceId, ChoiceKind, ChoiceOption, ChoiceRequest,
    DomainEvent, MatchOutcome, PlayerId, Suit, TimeoutPolicy, TurnPhaseTag, ZoneId,
};
use rule_api::{
    DrawTargetScope, EffectRequest, QueryRequest, QueryResponse, RelationTag, TargetCardScope,
};

use super::{run_effects, EffectError};

mod guarded_choice;
mod pack_shenhua_1;
mod pack_shenhua_2;
mod pack_huicui;
mod pack_bingshi_1;
mod pack_bingshi_2;
mod pack_sp_1;
mod pack_old;
mod pack_extra_1;
mod pack_extra_2;
mod pack_refresh;
mod pack_sp_2;
mod pack_mobile;
mod pack_tw_1;
mod pack_bingshi_3;
mod pack_bingshi_4;
mod pack_xianding_1;
mod pack_xianding_2;
mod pack_wandianlunsha;
mod pack_ddd_1;
mod pack_ddd_2;
mod pack_ddd_3;
mod pack_tw_2;
mod pack_tw_3;
mod pack_shiji;
mod pack_jsrg;

pub(crate) use guarded_choice::*;
pub use pack_shenhua_1::*;
pub(crate) use pack_shenhua_2::*;
pub(crate) use pack_huicui::*;
pub(crate) use pack_bingshi_1::*;
pub(crate) use pack_bingshi_2::*;
pub(crate) use pack_sp_1::*;
pub(crate) use pack_old::*;
pub(crate) use pack_extra_1::*;
pub(crate) use pack_extra_2::*;
pub(crate) use pack_refresh::*;
pub(crate) use pack_sp_2::*;
pub use pack_mobile::*;
pub use pack_tw_1::*;
pub use pack_bingshi_3::*;
pub(crate) use pack_bingshi_4::*;
pub(crate) use pack_xianding_1::*;
pub(crate) use pack_xianding_2::*;
pub use pack_wandianlunsha::*;
pub(crate) use pack_ddd_1::*;
pub(crate) use pack_ddd_2::*;
pub(crate) use pack_ddd_3::*;
pub(crate) use pack_tw_2::*;
pub(crate) use pack_tw_3::*;
pub(crate) use pack_shiji::*;
pub(crate) use pack_jsrg::*;

/// 卡牌粗类别（0=基本 1=锦囊 2=装备；与 owner_card_category_mask 位一致）。
pub(crate) fn card_category_of(
    state: &crate::state::GameState,
    card: CardId,
) -> Option<u32> {
    let card = state.cards.get(card)?;
    if card.def.is_none() {
        return None;
    }
    let kind = card.kind;
    if kind.is_basic() {
        Some(0)
    } else if kind.is_trick() {
        Some(1)
    } else if kind.is_equip() {
        Some(2)
    } else {
        None
    }
}

/// Apply one primitive effect to the transaction working state.
pub fn apply_effect(
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
        EffectRequest::JudgeSpadeTwoToNineDrawTwoOrThunderShaTwice {
            source,
            target,
            ..
        } => judge_spade_two_to_nine_draw_two_or_thunder_sha_twice(tx, source, target),
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
        EffectRequest::ResolveZhangjian {
            player,
            card,
            grant_skill_def,
            action,
            target,
            skill,
        } => resolve_zhangjian(tx, player, card, grant_skill_def, action, target, skill),
        EffectRequest::ResolveZhugeruoxue {
            player,
            target,
            card,
            action,
            remaining,
            used_mask,
            repeated,
            skill,
        } => resolve_zhugeruoxue(
            tx, player, target, card, action, remaining, used_mask, repeated, skill,
        ),
        EffectRequest::GiveSelectedThenTargetMayUseDrawOwner {
            player,
            target,
            card,
            targets,
            stage,
            skill,
        } => give_selected_then_target_may_use_draw_owner(
            tx, player, target, card, targets, stage, skill,
        ),
        EffectRequest::ChooseCategoryThenTargetGiveMatchingOrDamagePlusOne {
            player,
            target,
            damage_id,
            skill,
        } => choose_category_then_target_give_matching_or_damage_plus_one(
            tx, player, target, damage_id, skill,
        ),
        EffectRequest::ChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip {
            player,
            source,
            card,
            stage,
            skill,
        } => choose_other_excluding_source_random_gain_hand_equip_then_use_if_equip(
            tx, player, source, card, stage, skill,
        ),
        EffectRequest::ResolveMobiledanshou {
            player,
            target,
            skill,
        } => resolve_mobiledanshou(tx, player, target, skill),
        EffectRequest::ResolveJsrgchiying {
            player,
            target,
            started,
            remaining,
            accum,
            skill,
        } => resolve_jsrgchiying(tx, player, target, started, remaining, accum, skill),
        EffectRequest::ResolveMbqingdao {
            player,
            source,
            card,
            damaged,
            skill,
        } => resolve_mbqingdao(tx, player, source, card, damaged, skill),
        EffectRequest::ResolveMbxiugeng {
            player,
            helper_def,
            hand_limit_def,
            stage,
            targets,
            skill,
        } => resolve_mbxiugeng(
            tx,
            player,
            helper_def,
            hand_limit_def,
            stage,
            targets,
            skill,
        ),
        EffectRequest::ResolveCaoyi {
            player,
            card,
            helper_skill_def,
            action,
            event_targets,
            skill,
        } => resolve_caoyi(
            tx,
            player,
            card,
            helper_skill_def,
            action,
            event_targets,
            skill,
        ),
        EffectRequest::ResolveMalingli {
            player,
            card,
            helper_skill_def,
            action,
            source,
            target,
            nature,
            damage_id,
            event_cards,
            event_targets,
            skill,
        } => resolve_malingli(
            tx,
            player,
            card,
            helper_skill_def,
            action,
            source,
            target,
            nature,
            damage_id,
            event_cards,
            event_targets,
            skill,
        ),
        EffectRequest::ResolveNsshuaiyan {
            player,
            recovered_player,
            skill,
        } => resolve_nsshuaiyan(tx, player, recovered_player, skill),
        EffectRequest::ResolveWdbHanlong {
            player,
            target,
            card,
            helper_skill_def,
            action,
            skill,
        } => resolve_wdb_hanlong(tx, player, target, card, helper_skill_def, action, skill),
        EffectRequest::ResolveWdbLiufu {
            player,
            target,
            card,
            zhucheng_skill_def,
            action,
            skill,
        } => resolve_wdb_liufu(tx, player, target, card, zhucheng_skill_def, action, skill),
        EffectRequest::ResolveWdbLiuyan {
            player,
            card,
            action,
            skill,
        } => resolve_wdb_liuyan(tx, player, card, action, skill),
        EffectRequest::ResolveWdbLiuzan {
            player,
            target,
            helper_skill_def,
            action,
            skill,
        } => resolve_wdb_liuzan(tx, player, target, helper_skill_def, action, skill),
        EffectRequest::ResolveWdbTianyu {
            player,
            target,
            card,
            action,
            skill,
        } => resolve_wdb_tianyu(tx, player, target, card, action, skill),
        EffectRequest::ContinueYoudishenru {
            action,
            protected,
            source,
            forced_card,
            saved,
        } => {
            crate::command::play::continue_youdishenru(
                tx,
                action,
                protected,
                source,
                forced_card,
                &saved,
            )?;
            Ok(Vec::new())
        }
        EffectRequest::ResolveWdbXizhenxihong {
            player,
            target,
            card,
            action,
            skill,
        } => resolve_wdb_xizhenxihong(tx, player, target, card, action, skill),
        EffectRequest::ContinueWdbFuchou {
            action,
            player,
            target,
            remaining,
            skill,
        } => continue_wdb_fuchou(tx, action, player, target, remaining, skill),
        EffectRequest::ResolveWdbYangyi {
            player,
            target,
            card,
            action,
            skill,
        } => resolve_wdb_yangyi(tx, player, target, card, action, skill),
        EffectRequest::ContinueDiaobingqianjiang {
            action,
            actor,
            source_card,
            targets,
            revealed,
        } => continue_diaobingqianjiang(tx, action, actor, source_card, targets, revealed),
        EffectRequest::ResolveWdbZaozhiRenjun {
            player,
            target,
            card,
            cards,
            action,
            skill,
        } => resolve_wdb_zaozhirenjun(tx, player, target, card, cards, action, skill),
        EffectRequest::ResolveDddXianxi {
            player,
            card,
            action,
            event_targets,
            damaged_targets,
            skill,
        } => resolve_ddd_xianxi(
            tx,
            player,
            card,
            action,
            event_targets,
            damaged_targets,
            skill,
        ),
        EffectRequest::ResolveDddXiaheng {
            player,
            discard_target,
            damage_target,
            action,
            skill,
        } => resolve_ddd_xiaheng(tx, player, discard_target, damage_target, action, skill),
        EffectRequest::ResolveDddLiuchong {
            player,
            actor,
            card,
            action,
            cards,
            skill,
        } => resolve_ddd_liuchong(tx, player, actor, card, action, cards, skill),
        EffectRequest::ResolveDddDuanbing {
            player,
            target,
            card,
            action,
            skill,
        } => resolve_ddd_duanbing(tx, player, target, card, action, skill),
        EffectRequest::ResolveDddWuzhi {
            player,
            card,
            action,
            cards,
            skill,
        } => resolve_ddd_wuzhi(tx, player, card, action, cards, skill),
        EffectRequest::ResolveDddXujing {
            player,
            source,
            card,
            action,
            targets,
            skill,
        } => resolve_ddd_xujing(tx, player, source, card, action, targets, skill),
        EffectRequest::ResolveDddCaomao {
            player,
            action,
            cards,
            targets,
            skill,
        } => resolve_ddd_caomao(tx, player, action, cards, targets, skill),
        EffectRequest::ResolveDddXinxianying {
            player,
            actor,
            action,
            skill,
        } => resolve_ddd_xinxianying(tx, player, actor, action, skill),
        EffectRequest::ResolveDddXianglang {
            player,
            actor,
            action,
            skill,
        } => resolve_ddd_xianglang(tx, player, actor, action, skill),
        EffectRequest::ResolveDddYujin {
            player,
            actor,
            target,
            move_mask,
            action,
            candidates,
            skill,
        } => resolve_ddd_yujin(
            tx, player, actor, target, move_mask, action, candidates, skill,
        ),
        EffectRequest::ResolveDddLiuye {
            player,
            actor,
            card,
            action,
            skill,
        } => resolve_ddd_liuye(tx, player, actor, card, action, skill),
        EffectRequest::ResolveDddBaosanniang {
            player,
            target,
            card,
            action,
            cards,
            targets,
            used_kind,
            skill,
        } => resolve_ddd_baosanniang(
            tx, player, target, card, action, cards, targets, used_kind, skill,
        ),
        EffectRequest::ResolveDddZhenji {
            player,
            target,
            action,
            stage,
            piles,
            cards,
            skill,
        } => resolve_ddd_zhenji(tx, player, target, action, stage, piles, cards, skill),
        EffectRequest::ResolveDddZhaoang {
            player,
            action,
            stage,
            card,
            kind,
            count,
            skill,
        } => resolve_ddd_zhaoang(tx, player, action, stage, card, kind, count, skill),
        EffectRequest::ResolveDddLiuba {
            player,
            actor,
            action,
            cards,
            targets,
            skill,
        } => resolve_ddd_liuba(tx, player, actor, action, cards, targets, skill),
        EffectRequest::ResolveDddJianshuo {
            player,
            target,
            action,
            cards,
            targets,
            skill,
        } => resolve_ddd_jianshuo(tx, player, target, action, cards, targets, skill),
        EffectRequest::ResolveDddGuanning {
            player,
            actor,
            action,
            card,
            skill,
        } => resolve_ddd_guanning(tx, player, actor, action, card, skill),
        EffectRequest::ResolveDddCaoshuang {
            player,
            actor,
            card,
            action,
            skill,
        } => resolve_ddd_caoshuang(tx, player, actor, card, action, skill),
        EffectRequest::ResolveDddXuelingyun {
            player,
            actor,
            card,
            cards,
            action,
            skill,
        } => resolve_ddd_xuelingyun(tx, player, actor, card, cards, action, skill),
        EffectRequest::ResolveDddLiuhong {
            player,
            actor,
            action,
            skill,
        } => resolve_ddd_liuhong(tx, player, actor, action, skill),
        EffectRequest::ResolveDddXiahouxuan {
            player,
            action,
            skill,
        } => resolve_ddd_xiahouxuan(tx, player, action, skill),
        EffectRequest::ResolveDddZhangkai {
            player,
            actor,
            card,
            cards,
            amount,
            action,
            skill,
        } => resolve_ddd_zhangkai(tx, player, actor, card, cards, amount, action, skill),
        EffectRequest::ResolveDddLiangxi {
            player,
            actor,
            card,
            cards,
            action,
            skill,
        } => resolve_ddd_liangxi(tx, player, actor, card, cards, action, skill),
        EffectRequest::GuozhanUseZhulianBihe { player } => {
            resolve_guozhan_use_zhulian_bihe(tx, player)
        }
        EffectRequest::ResolveShoulijian {
            player,
            target,
            skill,
        } => resolve_shoulijian(tx, player, target, skill),
        EffectRequest::CancelIncomingCardUse {
            player,
            source,
            card,
            skill,
        } => cancel_incoming_card_use(tx, player, source, card, skill),
        EffectRequest::ChooseOtherGainOneOfTwoSkills {
            player,
            target,
            first_def,
            second_def,
            skill,
        } => choose_other_gain_one_of_two_skills(tx, player, target, first_def, second_def, skill),
        EffectRequest::ResolveGwyewuChain {
            player,
            skill,
            cost_cards,
            wuyao_def,
            lang_def,
        } => resolve_gwyewu_chain(tx, player, skill, cost_cards, wuyao_def, lang_def),
        EffectRequest::ContinueWugu {
            action,
            actor,
            source_card,
            targets,
            excluded,
            revealed,
        } => continue_wugu(tx, action, actor, source_card, targets, excluded, revealed),
        EffectRequest::ResolveMbchenshe {
            player,
            dying,
            source,
            stage,
            roles,
            suits,
            skill,
        } => resolve_mbchenshe(tx, player, dying, source, stage, roles, suits, skill),
        EffectRequest::ResolveMbquanchong {
            player,
            helper_def,
            skill,
        } => resolve_mbquanchong(tx, player, helper_def, skill),
        EffectRequest::ResolveMbrenxing {
            player,
            stage,
            target,
            skill,
        } => resolve_mbrenxing(tx, player, stage, target, skill),
        EffectRequest::ResolveMbchizhang {
            player,
            card,
            stage,
            cards,
            skill,
        } => resolve_mbchizhang(tx, player, card, stage, cards, skill),
        EffectRequest::ResolveMbduanyang {
            player,
            target,
            card,
            mode,
            stage,
            cards,
            skill,
        } => resolve_mbduanyang(tx, player, target, card, mode, stage, cards, skill),
        EffectRequest::ResolveMbrunwei {
            player,
            target,
            stage,
            value,
            cards,
            skill,
        } => resolve_mbrunwei(tx, player, target, stage, value, cards, skill),
        EffectRequest::ResolveMbshuanghuai {
            player,
            target,
            stage,
            choice,
            skill,
        } => resolve_mbshuanghuai(tx, player, target, stage, choice, skill),
        EffectRequest::ResolveMbxiongtu {
            player,
            target,
            stage,
            card,
            cards,
            skill,
        } => resolve_mbxiongtu(tx, player, target, stage, card, cards, skill),
        EffectRequest::ResolveMbganggeng {
            player,
            target,
            stage,
            cards,
            skill,
        } => resolve_mbganggeng(tx, player, target, stage, cards, skill),
        EffectRequest::ResolveMbsijian {
            player,
            target,
            stage,
            choice,
            helper_def,
            skill,
        } => resolve_mbsijian(tx, player, target, stage, choice, helper_def, skill),
        EffectRequest::ResolveMbjianji {
            player,
            source,
            target,
            stage,
            card,
            cards,
            targets,
            skill,
        } => resolve_mbjianji(
            tx, player, source, target, stage, card, cards, targets, skill,
        ),
        EffectRequest::ResolveMbyuanmo {
            player,
            target,
            stage,
            card,
            value,
            skill,
        } => resolve_mbyuanmo(tx, player, target, stage, card, value, skill),
        EffectRequest::ResolveMbfeijing {
            player,
            target,
            card,
            stage,
            targets,
            cards,
            skill,
        } => resolve_mbfeijing(tx, player, target, card, stage, targets, cards, skill),
        EffectRequest::ResolveMbxiaoge {
            player,
            target,
            card,
            stage,
            targets,
            skill,
        } => resolve_mbxiaoge(tx, player, target, card, stage, targets, skill),
        EffectRequest::ResolveMbxuye {
            player,
            target,
            stage,
            was_max,
            skill,
        } => resolve_mbxuye(tx, player, target, stage, was_max, skill),
        EffectRequest::ResolveMbkuangxiang {
            player,
            target,
            stage,
            skill,
        } => resolve_mbkuangxiang(tx, player, target, stage, skill),
        EffectRequest::ResolvePotchengpu {
            player,
            target,
            stage,
            damage_id,
            skill,
        } => resolve_potchengpu(tx, player, target, stage, damage_id, skill),
        EffectRequest::ResolvePotjugu { player, skill } => resolve_potjugu(tx, player, skill),
        EffectRequest::ResolvePotsifeng {
            player,
            target,
            card,
            stage,
            skill,
        } => resolve_potsifeng(tx, player, target, card, stage, skill),
        EffectRequest::ResolvePotchenqun {
            player,
            target,
            stage,
            skill,
        } => resolve_potchenqun(tx, player, target, stage, skill),
        EffectRequest::ResolvePotwanglie {
            player,
            card,
            helper_skill_def,
            stage,
            skill,
        } => resolve_potwanglie(tx, player, card, helper_skill_def, stage, skill),
        EffectRequest::ResolvePothongyi {
            player,
            stage,
            skill,
        } => resolve_pothongyi(tx, player, stage, skill),
        EffectRequest::ResolvePotguansha {
            player,
            helper_skill_def,
            skill,
        } => resolve_potguansha(tx, player, helper_skill_def, skill),
        EffectRequest::ResolvePotjiyu {
            player,
            card,
            stage,
            skill,
        } => resolve_potjiyu(tx, player, card, stage, skill),
        EffectRequest::ResolveMbzhuji {
            player,
            choice,
            stage,
            skill,
        } => resolve_mbzhuji(tx, player, choice, stage, skill),
        EffectRequest::ResolvePotlibing {
            player,
            card,
            stage,
            skill,
        } => resolve_potlibing(tx, player, card, stage, skill),
        EffectRequest::ResolvePotpoxi {
            player,
            cards,
            skill,
        } => resolve_potpoxi(tx, player, cards, skill),
        EffectRequest::ResolvePotfuan {
            player,
            card,
            stage,
            range_helper_def,
            yinxian_def,
            skill,
        } => resolve_potfuan(
            tx,
            player,
            card,
            stage,
            range_helper_def,
            yinxian_def,
            skill,
        ),
        EffectRequest::ResolvePotyinxian {
            player,
            card,
            stage,
            skill,
        } => resolve_potyinxian(tx, player, card, stage, skill),
        EffectRequest::ResolvePothuanjie {
            player,
            target,
            stage,
            qice_def,
            kanpo_def,
            skill,
        } => resolve_pothuanjie(tx, player, target, stage, qice_def, kanpo_def, skill),
        EffectRequest::ResolvePotnigu {
            player,
            target,
            card,
            targets,
            index,
            declined,
            stage,
            skill,
        } => resolve_potnigu(
            tx, player, target, card, targets, index, declined, stage, skill,
        ),
        EffectRequest::ResolvePotlulian {
            player,
            card,
            target,
            stage,
            skill,
        } => resolve_potlulian(tx, player, card, target, stage, skill),
        EffectRequest::ResolvePotdimeng {
            player,
            targets,
            skill,
        } => resolve_potdimeng(tx, player, targets, skill),
        EffectRequest::ResolvePothaoshi {
            player,
            target,
            action,
            skill,
        } => resolve_pothaoshi(tx, player, target, action, skill),
        EffectRequest::ResolveSpmiaolve {
            player,
            action,
            skill,
        } => resolve_spmiaolve(tx, player, action, skill),
        EffectRequest::ResolveSpyingjia {
            player,
            action,
            helper_def,
            card,
            card_kind,
            target,
            skill,
        } => resolve_spyingjia(
            tx, player, action, helper_def, card, card_kind, target, skill,
        ),
        EffectRequest::ResolveMantianguohai {
            player,
            stage,
            payload,
        } => resolve_mantianguohai(tx, player, stage, payload),
        EffectRequest::ResolveDcmanzhi { player, skill } => resolve_dcmanzhi(tx, player, skill),
        EffectRequest::ResolveXinxingluan { player, skill } => {
            resolve_xinxingluan(tx, player, skill)
        }
        EffectRequest::ResolveDckuizhen { player, skill } => resolve_dckuizhen(tx, player, skill),
        EffectRequest::ResolveDckuizhenResolve {
            player,
            winner,
            loser,
            skill,
        } => resolve_dckuizhen_resolve(tx, player, winner, loser, skill),
        EffectRequest::ResolveNsdingzhou { player, skill } => resolve_nsdingzhou(tx, player, skill),
        EffectRequest::ResolveSxrmlanjiao {
            player,
            target,
            stage,
            view,
            skill,
        } => resolve_sxrmlanjiao(tx, player, target, stage, view, skill),
        EffectRequest::ResolveRekuanshi { player, skill } => resolve_rekuanshi(tx, player, skill),
        EffectRequest::ResolveRekuanshiResolve {
            player,
            damaged,
            skill,
        } => resolve_rekuanshi_resolve(tx, player, damaged, skill),
        EffectRequest::ResolveDcmoyu { player, skill } => resolve_dcmoyu(tx, player, skill),
        EffectRequest::ResolveXizhen { player, skill } => resolve_xizhen(tx, player, skill),
        EffectRequest::ResolveXuxie { player, skill } => resolve_xuxie(tx, player, skill),
        EffectRequest::ResolveXuxieAdd { player, skill } => resolve_xuxie_add(tx, player, skill),
        EffectRequest::ResolveDcchiying {
            player,
            target,
            stage,
            accum,
            skill,
        } => resolve_dcchiying(tx, player, target, stage, accum, skill),
        EffectRequest::ResolveXinkuangfu {
            player,
            target,
            sha_target,
            stage,
            skill,
        } => resolve_xinkuangfu(tx, player, target, sha_target, stage, skill),
        EffectRequest::ResolveDcxiangshu {
            player,
            target,
            skill,
        } => resolve_dcxiangshu(tx, player, target, skill),
        EffectRequest::ResolveShuangxiong { player, skill } => {
            resolve_shuangxiong(tx, player, skill)
        }
        EffectRequest::ResolveZhijian {
            player,
            card,
            stage,
            target,
            skill,
        } => resolve_zhijian(tx, player, card, stage, target, skill),
        EffectRequest::ResolveGuzheng {
            player,
            event_player,
            stage,
            cards,
            return_card,
            take_rest,
            skill,
        } => resolve_guzheng(
            tx,
            player,
            event_player,
            stage,
            cards,
            return_card,
            take_rest,
            skill,
        ),
        EffectRequest::ResolveZhiba {
            player,
            actor,
            skill,
        } => resolve_zhiba(tx, player, actor, skill),
        EffectRequest::ResolveZhibaSettle {
            player,
            actor,
            winner,
            player_card,
            target_card,
            stage,
            skill,
        } => resolve_zhiba_settle(
            tx,
            player,
            actor,
            winner,
            player_card,
            target_card,
            stage,
            skill,
        ),
        EffectRequest::ResolveDrltCongjian {
            player,
            targets,
            card,
            target,
            stage,
            skill,
        } => resolve_drlt_congjian(tx, player, targets, card, target, stage, skill),
        EffectRequest::ResolveNzryFeijun {
            player,
            target,
            card,
            branch,
            stage,
            skill,
        } => resolve_nzry_feijun(tx, player, target, card, branch, stage, skill),
        EffectRequest::ResolveDrltYongsi {
            player,
            stage,
            skill,
        } => resolve_drlt_yongsi(tx, player, stage, skill),
        EffectRequest::ResolveDrltWeidi {
            player,
            cards,
            targets,
            stage,
            skill,
        } => resolve_drlt_weidi(tx, player, cards, targets, stage, skill),
        EffectRequest::ResolveLieren {
            player,
            target,
            win_skill_def,
            lose_skill_def,
            skill,
        } => resolve_lieren(tx, player, target, win_skill_def, lose_skill_def, skill),
        EffectRequest::ResolveTuntian { player, skill } => resolve_tuntian(tx, player, skill),
        EffectRequest::ResolveZaoxian {
            player,
            jixi_def,
            tuntian_def,
            skill,
        } => resolve_zaoxian(tx, player, jixi_def, tuntian_def, skill),
        EffectRequest::ResolveTiaoxin {
            player,
            target,
            stage,
            skill,
        } => resolve_tiaoxin(tx, player, target, stage, skill),
        EffectRequest::ResolveZhiji {
            player,
            guanxing_def,
            skill,
        } => resolve_zhiji(tx, player, guanxing_def, skill),
        EffectRequest::ResolveZaiqi {
            player,
            extra,
            skill,
        } => resolve_zaiqi(tx, player, extra, skill),
        EffectRequest::ResolveTwqiushou { player, skill } => resolve_twqiushou(tx, player, skill),
        EffectRequest::ResolveZhengrong {
            player,
            candidates,
            target,
            stage,
            skill,
        } => resolve_zhengrong(tx, player, &candidates, target, stage, skill),
        EffectRequest::ResolveHongju {
            player,
            qingjian_def,
            rong_def,
            stage,
            skill,
        } => resolve_hongju(tx, player, qingjian_def, rong_def, stage, skill),
        EffectRequest::ResolveQingjian {
            player,
            rong_def,
            stage,
            skill,
        } => resolve_qingjian(tx, player, rong_def, stage, skill),
        EffectRequest::ResolveReleiji {
            player,
            target,
            stage,
            skill,
        } => resolve_releiji(tx, player, target, stage, skill),
        EffectRequest::ResolveHuangtian {
            player,
            actor,
            stage,
            skill,
        } => resolve_huangtian(tx, player, actor, stage, skill),
        EffectRequest::ResolveHaoshi {
            player,
            stage,
            skill,
        } => resolve_haoshi(tx, player, stage, skill),
        EffectRequest::ResolveDimeng {
            player,
            targets,
            stage,
            skill,
        } => resolve_dimeng(tx, player, &targets, stage, skill),
        EffectRequest::ResolveQiaobian {
            player,
            phase,
            stage,
            skill,
        } => resolve_qiaobian(tx, player, phase, stage, skill),
        EffectRequest::ExcludeSavedShaTarget { card, target } => {
            exclude_saved_sha_target(tx, card, target)
        }
        EffectRequest::ResolveXiangle {
            player,
            source,
            card,
            stage,
            skill,
        } => resolve_xiangle(tx, player, source, card, stage, skill),
        EffectRequest::ResolveFangquanSkip { player, skill } => {
            resolve_fangquan_skip(tx, player, skill)
        }
        EffectRequest::ResolveFangquanGrant {
            player,
            stage,
            skill,
        } => resolve_fangquan_grant(tx, player, stage, skill),
        EffectRequest::ResolveRuoyu {
            player,
            jijiang_def,
            skill,
        } => resolve_ruoyu(tx, player, jijiang_def, skill),
        EffectRequest::ResolveZuilun {
            player,
            stage,
            skill,
        } => resolve_zuilun(tx, player, stage, skill),
        EffectRequest::ResolveFuyin {
            player,
            source,
            card,
            skill,
        } => resolve_fuyin(tx, player, source, card, skill),
        EffectRequest::BanUseAgainst { user, target } => ban_use_against(tx, user, target),
        EffectRequest::ResolveJuzhan {
            player,
            source,
            card,
            targets,
            stage,
            skill,
        } => resolve_juzhan(tx, player, source, card, &targets, stage, skill),
        EffectRequest::ResolveQizhi {
            player,
            targets,
            stage,
            skill,
        } => resolve_qizhi(tx, player, &targets, stage, skill),
        EffectRequest::ResolveJinqu {
            player,
            qizhi_def,
            stage,
            skill,
        } => resolve_jinqu(tx, player, qizhi_def, stage, skill),
        EffectRequest::ResolveJianxiang {
            player,
            stage,
            skill,
        } => resolve_jianxiang(tx, player, stage, skill),
        EffectRequest::ResolveShenshi {
            player,
            source,
            stage,
            skill,
        } => resolve_shenshi(tx, player, source, stage, skill),
        EffectRequest::ResolveMingren {
            player,
            stage,
            skill,
        } => resolve_mingren(tx, player, stage, skill),
        EffectRequest::ResolveZhenliang {
            player,
            source,
            card,
            mingren_def,
            stage,
            skill,
        } => resolve_zhenliang(tx, player, source, card, mingren_def, stage, skill),
        EffectRequest::ResolveKuizhu {
            player,
            stage,
            skill,
        } => resolve_kuizhu(tx, player, stage, skill),
        EffectRequest::ResolveZhizheng {
            player,
            stage,
            skill,
        } => resolve_zhizheng(tx, player, stage, skill),
        EffectRequest::ResolveLijun {
            player,
            source,
            card,
            stage,
            skill,
        } => resolve_lijun(tx, player, source, card, stage, skill),
        EffectRequest::ResolveLiangyin {
            player,
            kongsheng_def,
            stage,
            target,
            skill,
        } => resolve_liangyin(tx, player, kongsheng_def, stage, target, skill),
        EffectRequest::ResolveKongsheng {
            player,
            stage,
            target,
            card,
            skill,
        } => resolve_kongsheng(tx, player, stage, target, card, skill),
        EffectRequest::ResolvePottuntian {
            player,
            action,
            amount,
            targets,
            zaoxian_def,
            skill,
            ..
        } => resolve_pottuntian(tx, player, action, amount, targets, zaoxian_def, skill),
        EffectRequest::ResolvePotjixi {
            player,
            target,
            action,
            targets,
            skill,
        } => resolve_potjixi(tx, player, target, action, targets, skill),
        EffectRequest::ResolvePotceduan {
            player,
            target,
            action,
            skill,
        } => resolve_potceduan(tx, player, target, action, skill),
        EffectRequest::ResolvePottaishici {
            player,
            target,
            action,
            cards,
            hanzhan_def,
            zhanlie_def,
            skill,
        } => resolve_pottaishici(
            tx,
            player,
            target,
            action,
            cards,
            hanzhan_def,
            zhanlie_def,
            skill,
        ),
        EffectRequest::ResolveMbzhonghui {
            player,
            target,
            action,
            amount,
            mbsizi_def,
            mbkechang_def,
            skill,
        } => resolve_mbzhonghui(
            tx,
            player,
            target,
            action,
            amount,
            mbsizi_def,
            mbkechang_def,
            skill,
        ),
        EffectRequest::ResolvePotjiejie {
            player,
            actor,
            used_def,
            suit_defs,
            qingshi_def,
            skill,
        } => resolve_potjiejie(tx, player, actor, used_def, suit_defs, qingshi_def, skill),
        EffectRequest::ResolvePotqingshi { player, skill } => resolve_potqingshi(tx, player, skill),
        // ponytail: 巧变（ABI 559）由 C 段（omp）实现中，stub 保持编译通过。
        EffectRequest::ResolveQiaobian { .. } => Ok(Vec::new()),
        // ponytail: 并行会话中间态（ban-use-pairs / 拒战）stub 保持编译通过。
        EffectRequest::BanUseAgainst { .. } | EffectRequest::ResolveJuzhan { .. } => Ok(Vec::new()),
        EffectRequest::ResolvePotcaoshuang {
            player,
            target,
            card,
            action,
            skill,
        } => resolve_potcaoshuang(tx, player, target, card, action, skill),
        EffectRequest::ResolvePotyuji {
            player,
            target,
            card,
            action,
            cards,
            targets,
            skill,
        } => resolve_potyuji(tx, player, target, card, action, &cards, &targets, skill),
        EffectRequest::ResolveRehengjiang {
            player,
            actor,
            action,
            hand_limit_def,
            draw_def,
            skill,
        } => resolve_rehengjiang(tx, player, actor, action, hand_limit_def, draw_def, skill),
        EffectRequest::ResolveMuzhen { .. } => Err(EffectError::ContentFault(
            "resolve_muzhen pending implementation".into(),
        )),
        EffectRequest::ResolvePotlvyi {
            player,
            target,
            card,
            action,
            event_cards,
            targets,
            skill,
        } => resolve_potlvyi(
            tx,
            player,
            target,
            card,
            action,
            &event_cards,
            &targets,
            skill,
        ),
        EffectRequest::ResolvePotweiyan {
            player,
            target,
            card,
            action,
            skill,
        } => resolve_potweiyan(tx, player, target, card, action, skill),
        EffectRequest::ResolvePotzhouyu {
            player,
            target,
            card,
            action,
            card_flags,
            targets,
            skill,
        } => resolve_potzhouyu(
            tx, player, target, card, action, card_flags, &targets, skill,
        ),
        EffectRequest::ResolveDbzhuifeng { player, skill } => resolve_dbzhuifeng(tx, player, skill),
        EffectRequest::ResolveDbquedi {
            player,
            target,
            card,
            skill,
        } => resolve_dbquedi(tx, player, target, card, skill),
        EffectRequest::ResolveSpyinju {
            player,
            target,
            skill,
        } => resolve_spyinju(tx, player, target, skill),
        EffectRequest::ResolveSpchijie {
            player,
            source,
            card,
            skill,
        } => resolve_spchijie(tx, player, source, card, skill),
        EffectRequest::ResolveSpshangyi {
            player,
            target,
            skill,
        } => resolve_spshangyi(tx, player, target, skill),
        EffectRequest::ResolveSpjianyi { player, skill } => resolve_spjianyi(tx, player, skill),
        EffectRequest::ResolveFyjianyu {
            player,
            source,
            target,
            targets,
            skill,
        } => resolve_fyjianyu(tx, player, source, target, targets, skill),
        EffectRequest::ResolveLiubing {
            player,
            card,
            skill,
        } => resolve_liubing(tx, player, card, skill),
        EffectRequest::ResolveHeji {
            player,
            target,
            card,
            skill,
        } => resolve_heji(tx, player, target, card, skill),
        EffectRequest::ResolveSpyajun {
            player,
            less_def,
            card,
            target,
            target_card,
            stage,
            skill,
        } => resolve_spyajun(
            tx,
            player,
            less_def,
            card,
            target,
            target_card,
            stage,
            skill,
        ),
        EffectRequest::ResolveSpzundi {
            player,
            target,
            card,
            destination,
            stage,
            skill,
        } => resolve_spzundi(tx, player, target, card, destination, stage, skill),
        EffectRequest::ResolveXinyangjie {
            player,
            target,
            card,
            target_card,
            source,
            stage,
            skill,
        } => resolve_xinyangjie(tx, player, target, card, target_card, source, stage, skill),
        EffectRequest::ResolveQingyu {
            player,
            action,
            xuancun_def,
            mbyongjue_def,
            target,
            skill,
        } => resolve_qingyu(
            tx,
            player,
            action,
            xuancun_def,
            mbyongjue_def,
            target,
            skill,
        ),
        EffectRequest::ResolveZhengsu {
            player,
            mode,
            stage,
            card,
            skill,
        } => resolve_zhengsu(tx, player, mode, stage, card, skill),
        EffectRequest::ResolveHoufeng {
            player,
            target,
            mode,
            stage,
            skill,
        } => resolve_houfeng(tx, player, target, mode, stage, skill),
        EffectRequest::ResolveMbdiancai {
            player,
            actor,
            skill,
        } => resolve_mbdiancai(tx, player, actor, skill),
        EffectRequest::ResolveRechuhai {
            player,
            target,
            card,
            target_card,
            stage,
            skill,
        } => resolve_rechuhai(tx, player, target, card, target_card, stage, skill),
        EffectRequest::ResolveXiongshu {
            player,
            target,
            card,
            action,
            stage,
            skill,
        } => resolve_xiongshu(tx, player, target, card, action, stage, skill),
        EffectRequest::ResolveJianhui {
            player,
            source,
            target,
            skill,
        } => resolve_jianhui(tx, player, source, target, skill),
        EffectRequest::ResolveSpqiai {
            player,
            target,
            card,
            stage,
            skill,
        } => resolve_spqiai(tx, player, target, card, stage, skill),
        EffectRequest::ResolveSpshanxi {
            player,
            target,
            stage,
            skill,
        } => resolve_spshanxi(tx, player, target, stage, skill),
        EffectRequest::ResolveSpshanxiSuoming {
            player,
            actor,
            stage,
            skill,
        } => resolve_spshanxi_suoming(tx, player, actor, stage, skill),
        EffectRequest::ResolveTongduo {
            player,
            target,
            card,
            stage,
            skill,
        } => resolve_tongduo(tx, player, target, card, stage, skill),
        EffectRequest::ResolveYuanqing { player, skill } => resolve_yuanqing(tx, player, skill),
        EffectRequest::ResolveShuchen {
            player,
            dying,
            renku_def,
            skill,
        } => resolve_shuchen(tx, player, dying, renku_def, skill),
        EffectRequest::ResolveDuanbi {
            player,
            targets,
            stage,
            skill,
        } => resolve_duanbi(tx, player, &targets, stage, skill),
        EffectRequest::ResolveZaoli {
            player,
            mark,
            skill,
        } => resolve_zaoli(tx, player, mark, skill),
        EffectRequest::ResolveGuying {
            player,
            actor,
            card,
            stage,
            skill,
        } => resolve_guying(tx, player, actor, card, stage, skill),
        EffectRequest::ResolveDcxiangshuLottery {
            player,
            target,
            skill,
        } => resolve_dcxiangshu_lottery(tx, player, target, skill),
        EffectRequest::ResolveDcyingtu {
            player,
            target,
            stage,
            cards,
            skill,
        } => resolve_dcyingtu(tx, player, target, stage, cards, skill),
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
        EffectRequest::ExtraResolveCurrentTrick { player, card, skill } => {
            extra_resolve_current_trick(tx, player, card, skill)
        }
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
        EffectRequest::ChooseOneFromSkillPileDiscard { player, skill, pile_def } => {
            choose_one_from_skill_pile_discard(tx, player, skill, pile_def)
        }
        EffectRequest::GainWholeSkillPile { player, skill, pile_def } => {
            gain_whole_skill_pile(tx, player, skill, pile_def)
        }
        EffectRequest::RearrangePileToDeckTop { player, skill, pile_def, count } => {
            rearrange_pile_to_deck_top(tx, player, skill, pile_def, count)
        }
        EffectRequest::ChooseHandCardsToSkillPile { player, skill, min_count, max_count } => {
            choose_hand_cards_to_skill_pile(tx, player, skill, min_count, max_count)
        }
        EffectRequest::ResolveDcbingji { player, skill, stage, kind } => {
            resolve_dcbingji(tx, player, skill, stage, kind)
        }
        EffectRequest::ResolvePxqixian { player, skill, .. } => {
            resolve_pxqixian(tx, player, skill)
        }
        EffectRequest::ResolveQuanjia {
            player,
            skill,
            source,
            target,
        } => resolve_quanjia(tx, player, skill, source, target),
        EffectRequest::ResolveDuwang { player, skill } => {
            resolve_duwang(tx, player, skill)
        }
        EffectRequest::ResolveJsrgdaoren { player, skill } => {
            resolve_jsrgdaoren(tx, player, skill)
        }
        EffectRequest::ResolveQianlong { player, skill } => {
            resolve_qianlong(tx, player, skill)
        }
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
        EffectRequest::ResolveQiantun { player, skill } => {
            resolve_qiantun(tx, player, skill, 0)
        }
        EffectRequest::ResolveYangming { player, skill } => {
            resolve_yangming(tx, player, skill, 0)
        }
        EffectRequest::ResolveCanyun { player, skill } => {
            resolve_canyun(tx, player, skill, 0)
        }
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
        EffectRequest::ResolveChumei { player, skill, stage } => {
            resolve_chumei(tx, player, skill, stage)
        }
        EffectRequest::ResolveShuaiyan { player, skill, stage } => {
            resolve_shuaiyan(tx, player, skill, stage)
        }
        EffectRequest::ResolveMbbeiming { player, skill, stage } => {
            resolve_mbbeiming(tx, player, skill, stage)
        }
        EffectRequest::ResetLinkedSelectedTargets { source, targets } => {
            reset_linked_selected_targets(tx, source, &targets)
        }
        EffectRequest::ResolveGaoyuan { player, source, card } => {
            resolve_gaoyuan(tx, player, source, card)
        }
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
        EffectRequest::ResolveChulao { player, targets } => {
            resolve_chulao(tx, player, &targets)
        }
        EffectRequest::ResolveMeiyong { player } => {
            resolve_meiyong(tx, player)
        }
        EffectRequest::ResolveTwxiongjun { player } => {
            resolve_twxiongjun(tx, player)
        }
        EffectRequest::ResolveDcqiaolve { player, target } => {
            resolve_dcqiaolve(tx, player, target)
        }
        EffectRequest::ResolveDingce { player, source } => {
            resolve_dingce(tx, player, source)
        }
        EffectRequest::ResolveYixiang { player } => {
            resolve_yixiang(tx, player)
        }
        EffectRequest::ResolveXinfuQiai { player } => {
            resolve_xinfu_qiai(tx, player)
        }
        EffectRequest::ResolveOlwenren {
            player,
            targets,
            skill: _,
        } => resolve_olwenren(tx, player, &targets),
        EffectRequest::ResolveSpolaige { player } => {
            resolve_spolaige(tx, player)
        }
        EffectRequest::ResolveOlzhijie { player } => {
            resolve_olzhijie(tx, player)
        }
        EffectRequest::ResolveOlsuyi { player, target } => {
            resolve_olsuyi(tx, player, target)
        }
        EffectRequest::ResolveOlqiaozhi { player, stage } => {
            resolve_olqiaozhi(tx, player, stage)
        }
        EffectRequest::ResolveDczhee { player, mode } => {
            resolve_dczhee(tx, player, mode)
        }
        EffectRequest::ResolveDchaoxian { player } => {
            resolve_dchaoxian(tx, player)
        }
        EffectRequest::ResolveZhongjie { player } => {
            resolve_zhongjie(tx, player)
        }
        EffectRequest::ResolveStarcuxiaWeiming {
            player,
            source,
            variant,
        } => resolve_starcuxia_weiming(tx, player, source, variant),
        EffectRequest::ResolveSanshou { player } => {
            resolve_sanshou(tx, player)
        }
        EffectRequest::ResolveNhlundao { player, source } => {
            resolve_nhlundao(tx, player, source)
        }
        EffectRequest::ResolveXiangshu { player, targets } => {
            resolve_xiangshu(tx, player, &targets)
        }
        EffectRequest::ResolveQiangyong {
            player,
            target,
            skill,
        } => resolve_qiangyong(tx, player, target, skill),
        EffectRequest::ResolveMbzhenann { player } => {
            resolve_mbzhenann(tx, player)
        }
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
            let targets = crate::command::play::redirect_saved_sha_target(
                tx,
                source,
                card,
                player,
                target,
            )?;
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
        EffectRequest::ResolveReliangyin { player, skill, mode } => {
            resolve_reliangyin(tx, player, skill, mode)
        }
        EffectRequest::ResolveZhongyan { player, skill, stage } => {
            resolve_zhongyan(tx, player, skill, stage)
        }
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
        EffectRequest::PlaceHandCardsToDeckTop { player, skill, count } => {
            place_hand_cards_to_deck_top(tx, player, skill, count)
        }
        EffectRequest::ChooseTargetHandCardToPile { player, skill, pile_def } => {
            choose_target_hand_card_to_pile(tx, player, skill, pile_def)
        }
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
        EffectRequest::PinDebateOpinions { player, skill } => {
            pin_debate_opinions(tx, player, skill)
        }
        EffectRequest::AlignDebateOpinions { player, skill } => {
            align_debate_opinions(tx, player, skill)
        }
        EffectRequest::GainShadowCards {
            player,
            count,
            skill,
        } => gain_shadow_cards(tx, player, count, skill),
        EffectRequest::ChangeGroup {
            player,
            group,
            skill,
        } => change_group(tx, player, group, skill),
        EffectRequest::DistributeShadowCards {
            player,
            count,
            skill,
        } => distribute_shadow_cards(tx, player, count, skill),
        EffectRequest::RegisterTurnEndDebate { player, skill } => {
            register_turn_end_debate(tx, player, skill)
        }
        EffectRequest::ResolveTurnEndDebates { player, skill } => {
            resolve_turn_end_debates(tx, player, skill)
        }
        EffectRequest::ChooseGroup { player, skill } => {
            choose_group(tx, player, skill)
        }
        EffectRequest::ChooseGainOtherSkill { player, skill } => {
            choose_gain_other_skill(tx, player, skill)
        }
        EffectRequest::LoseGainedSkill { player, skill } => {
            lose_gained_skill(tx, player, skill)
        }
        EffectRequest::ResolvePotheyun { player, skill } => {
            resolve_potheyun(tx, player, skill)
        }
        EffectRequest::PlaceXumou { player, card, skill } => {
            place_xumou(tx, player, card, skill)
        }
        EffectRequest::PlaceDeckTopXumou { player, skill } => {
            place_deck_top_xumou(tx, player, skill)
        }
        EffectRequest::ChoosePlaceHandXumou { player, skill } => {
            choose_place_hand_xumou(tx, player, skill)
        }
        EffectRequest::DiscardOneOwnXumou { player, skill } => {
            discard_one_own_xumou(tx, player, skill)
        }
        EffectRequest::PlaceDeckTopExpansion {
            player,
            count,
            skill,
        } => place_deck_top_expansion(tx, player, count, skill),
        EffectRequest::CityConsume { player, skill } => city_consume(tx, player, skill),
        EffectRequest::QueueExtraTurn { player, check, skill } => {
            queue_extra_turn(tx, player, check, skill)
        }
        EffectRequest::ChooseOthersInsertExtraTurn { player, check, skill } => {
            choose_others_insert_extra_turn(tx, player, check, skill)
        }
        EffectRequest::QueueExtraPhases {
            player,
            phases_mask,
            skill,
        } => queue_extra_phases(tx, player, phases_mask, skill),
        EffectRequest::ResolveTwjieqiu {
            player,
            target,
            subaction,
            skill,
        } => resolve_twjieqiu(tx, player, target, subaction, skill),
        EffectRequest::SkipRemainingPhases {
            player,
            phases_mask,
            skill,
        } => skip_remaining_phases(tx, player, phases_mask, skill),
        EffectRequest::ChooseOthersAddMark {
            player,
            mark,
            delta,
            max_count,
            skill,
        } => choose_others_add_mark(tx, player, mark, delta, max_count, skill),
        EffectRequest::ResolveJsrgsheju {
            player,
            counterpart,
            skill,
        } => resolve_jsrgsheju(tx, player, counterpart, skill),
        EffectRequest::QueueOpeningExtraPlayPhase { player, skill } => {
            queue_opening_extra_play_phase(tx, player, skill)
        }
        EffectRequest::ChooseEndExtraDrawOrPlayByUsedSuits { player, skill } => {
            choose_end_extra_draw_or_play_by_used_suits(tx, player, skill)
        }
        EffectRequest::LoseHpDoubleCurrentDamageThenReplace {
            player,
            damage_id,
            amount,
            skill,
            replacement_skill_def,
            ready_to_replace,
        } => lose_hp_double_current_damage_then_replace(
            tx,
            player,
            damage_id,
            amount,
            skill,
            replacement_skill_def,
            ready_to_replace,
        ),
        EffectRequest::BeginTurn { player } => begin_turn(tx, player),
        EffectRequest::FinishTurnBegin { player } => finish_turn_begin(tx, player),
        EffectRequest::AdvanceOpeningExtraPlay { player } => advance_opening_extra_play(tx, player),
        EffectRequest::ConvertCurrentShaToFireLoseHpIfDamage { player, card, .. } => {
            crate::command::play::convert_saved_sha_to_fire(tx, player, card)?;
            Ok(Vec::new())
        }
        EffectRequest::IncreaseCurrentShaBaseDamage {
            player,
            card,
            amount,
            ..
        } => {
            crate::command::play::increase_saved_sha_base_damage(tx, player, card, amount)?;
            Ok(Vec::new())
        }
        EffectRequest::IncreaseCurrentCardBaseDamage {
            player,
            card,
            amount,
            ..
        } => {
            crate::command::play::increase_saved_card_base_damage(tx, player, card, amount)?;
            Ok(Vec::new())
        }
        EffectRequest::ChooseAddCurrentShaTarget {
            player,
            card,
            skill,
        } => choose_add_current_sha_target(tx, player, card, skill),
        EffectRequest::ChooseAddCurrentShaTargetWithinDistance {
            player,
            card,
            max_distance,
            skill,
        } => choose_add_current_sha_target_within_distance(tx, player, card, max_distance, skill),
        EffectRequest::RefundCurrentShaUse {
            player,
            card,
            skill: _,
        } => {
            crate::command::play::refund_resolving_sha_use(tx, player, card)?;
            Ok(Vec::new())
        }
        EffectRequest::CommitAddCurrentShaTarget {
            player,
            card,
            target,
            skill: _,
        } => {
            let targets = crate::command::play::add_saved_sha_target(tx, player, card, target)?;
            Ok(vec![DomainEvent::CardTargetAdded {
                player,
                card,
                target,
                targets,
            }])
        }
        EffectRequest::ChooseStoreHandShaInSkillPile { player, skill } => {
            choose_store_hand_sha_in_skill_pile(tx, player, skill)
        }
        EffectRequest::ChooseSpendSkillPileForDyingJiu {
            player,
            dying,
            skill,
        } => choose_spend_skill_pile_for_dying_jiu(tx, player, dying, skill),
        EffectRequest::CommitStoreHandShaInSkillPile {
            player,
            cards,
            skill,
        } => commit_store_hand_sha_in_skill_pile(tx, player, cards, skill),
        EffectRequest::CommitSpendSkillPileForDyingJiu {
            player,
            dying,
            card,
            skill,
        } => commit_spend_skill_pile_for_dying_jiu(tx, player, dying, card, skill),
        EffectRequest::ChooseCompareSelectedHandCard {
            player,
            target,
            card,
            win_skill_def,
            lose_skill_def,
            skill,
        } => choose_compare_selected_hand_card(
            tx,
            player,
            target,
            card,
            win_skill_def,
            lose_skill_def,
            skill,
        ),
        EffectRequest::ChooseCompareSelectedHandCardSecondFailureSeatDamage {
            player,
            target,
            card,
            draw_count,
            failure_count,
            lose_hp,
            damage,
            skill,
        } => choose_compare_selected_hand_card_second_failure_seat_damage(
            tx,
            player,
            target,
            card,
            draw_count,
            failure_count,
            lose_hp,
            damage,
            skill,
        ),
        EffectRequest::ChooseHandCompare {
            player,
            win_skill_def,
            lose_skill_def,
            skill,
        } => choose_hand_compare(tx, player, win_skill_def, lose_skill_def, skill),
        EffectRequest::ChooseHandCompareGrantTargetOnWin {
            player,
            win_grant_def,
            lose_skill_def,
            skill,
        } => choose_hand_compare_grant_target_on_win(tx, player, win_grant_def, lose_skill_def, skill),
        EffectRequest::ChooseHandCompareBetween {
            player,
            first,
            skill,
        } => choose_hand_compare_between(tx, player, first, skill),
        EffectRequest::GuanxingTop {
            player,
            count,
            round: _,
            flipped: _,
            top: _,
            bottom: _,
            skill,
        } => guanxing_top(tx, player, count, skill),
        EffectRequest::ChooseOtherGiveThenRecover { player, skill } => {
            choose_other_give_then_recover(tx, player, skill)
        }
        EffectRequest::GuanxingTopByFaluMarks {
            player,
            marks,
            skill,
        } => guanxing_top_by_falu_marks(tx, player, marks, skill),
        EffectRequest::GiveSelectedThenRecoverSelf {
            player,
            target,
            cards,
            amount,
            skill,
        } => give_selected_then_recover_self(tx, player, target, cards, amount, skill),
        EffectRequest::ChooseOtherDamage {
            player,
            amount,
            skill,
        } => choose_other_damage(tx, player, amount, skill),
        EffectRequest::ChooseBoundSourceAttackRangeDamage {
            player,
            source,
            amount,
            skill,
        } => choose_bound_source_attack_range_damage(tx, player, source, amount, skill),
        EffectRequest::ChooseOtherDiscardTargetCard {
            player,
            scope,
            skill,
        } => choose_other_discard_target_card(tx, player, scope, skill),
        EffectRequest::ContinueChooseCompareTargetHandCard {
            player,
            target,
            card,
            win_skill_def,
            lose_skill_def,
            skill,
        } => continue_choose_compare_target_hand_card(
            tx,
            player,
            target,
            card,
            win_skill_def,
            lose_skill_def,
            skill,
        ),
        EffectRequest::ContinueChooseCompareTargetHandCardSecondFailureSeatDamage {
            player,
            target,
            card,
            draw_count,
            failure_count,
            lose_hp,
            damage,
            skill,
        } => continue_choose_compare_target_hand_card_second_failure_seat_damage(
            tx,
            player,
            target,
            card,
            draw_count,
            failure_count,
            lose_hp,
            damage,
            skill,
        ),
        EffectRequest::ChooseDrawOrDiscardTargetCard {
            player,
            target,
            draw_count,
            skill,
        } => choose_draw_or_discard_target_card(tx, player, target, draw_count, skill),
        EffectRequest::ChooseSeatPathDamage {
            player,
            first,
            second,
            amount,
            skill,
        } => choose_seat_path_damage(tx, player, first, second, amount, skill),
        EffectRequest::RecordCompareFailureSeatDamage {
            player,
            target,
            failure_count,
            lose_hp,
            damage,
            skill,
        } => {
            let effects = record_compare_failure_seat_damage(
                tx,
                player,
                target,
                failure_count,
                lose_hp,
                damage,
                skill,
            )?;
            super::run_effects(tx, effects).map(|_| Vec::new())
        }
        EffectRequest::DealDamageEachInOrder {
            source,
            targets,
            amount,
            skill,
        } => {
            let effects = continue_damage_each_in_order(tx, source, targets, amount, skill)?;
            super::run_effects(tx, effects).map(|_| Vec::new())
        }
        EffectRequest::RandomizeCurrentHandCompareOpponent { player, skill } => {
            randomize_current_hand_compare_opponent(tx, player, skill)
        }
        EffectRequest::EmitHandCompareFinished {
            player,
            target,
            player_card,
            target_card,
            winner,
        } => Ok(vec![DomainEvent::HandCompareFinished {
            player,
            target,
            player_card,
            target_card,
            winner,
        }]),
        EffectRequest::GainHighestRankShaFromEventCards {
            player,
            cards,
            skill,
        } => gain_highest_rank_sha_from_event_cards(tx, player, &cards, skill),
        EffectRequest::GainHandCompareResultCard {
            player,
            initiator,
            target,
            winner,
            cards,
            skill,
        } => gain_hand_compare_result_card(tx, player, initiator, target, winner, &cards, skill),
        EffectRequest::ChooseAddOrRemoveCurrentCardTarget {
            player,
            card,
            skill,
        } => choose_add_or_remove_current_card_target(tx, player, card, skill),
        EffectRequest::ChooseAddSkillBoundCardTarget {
            player,
            card,
            skill,
        } => choose_add_skill_bound_card_target(tx, player, card, skill),
        EffectRequest::CommitAddSkillBoundCardTarget {
            player,
            card,
            target,
            skill: _,
        } => {
            let targets = crate::command::play::add_saved_card_target(tx, player, card, target)?;
            Ok(vec![DomainEvent::CardTargetAdded {
                player,
                card,
                target,
                targets,
            }])
        }
        EffectRequest::ChooseDiscardMatchingEquipColorAddSkill {
            player,
            target,
            skill_def,
            skill,
        } => choose_discard_matching_equip_color_add_skill(tx, player, target, skill_def, skill),
        EffectRequest::AddCardColorLockSkill {
            player,
            source,
            color_mask,
            skill_def,
        } => add_card_color_lock_skill(tx, player, source, color_mask, skill_def),
        EffectRequest::UseVirtualShaFromCardColorLockSourceIfNoSha { player, skill } => {
            use_virtual_sha_from_card_color_lock_source_if_no_sha(tx, player, skill)
        }
        EffectRequest::ChooseCurrentCardUseEnhancements {
            player,
            card,
            helper_skill_def,
            skill,
        } => choose_current_card_use_enhancements(tx, player, card, helper_skill_def, skill),
        EffectRequest::ChooseDrawOrDiscardAddSkill {
            player,
            draw_skill_def,
            discard_skill_def,
            skill,
            three_way,
        } => choose_draw_or_discard_add_skill(
            tx,
            player,
            draw_skill_def,
            discard_skill_def,
            skill,
            three_way,
        ),
        EffectRequest::ChooseDiscardedBlackBasicAsBingliang {
            player,
            cards,
            skill,
        } => choose_discarded_black_basic_as_bingliang(tx, player, cards, skill),
        EffectRequest::ChooseDiscardJudgeUseVirtualShaDrawIfNoDamage {
            player,
            target,
            skill,
        } => choose_discard_judge_use_virtual_sha_draw_if_no_damage(tx, player, target, skill),
        EffectRequest::UseVirtualShaDrawIfNoDamage {
            player,
            target,
            skill,
        } => crate::command::play::use_virtual_sha_on_draw_if_no_damage(tx, player, target, skill)
            .map(|_| Vec::new()),
        EffectRequest::DiscardSelectedRecordUniqueCategoryTarget {
            player,
            target,
            card,
            skill,
        } => discard_selected_record_unique_category_target(tx, player, target, card, skill),
        EffectRequest::ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded {
            player,
            target,
            toggle_link,
            skill,
        } => choose_target_draw_or_discard_by_skill_use_count_link_if_wounded(
            tx,
            player,
            target,
            toggle_link,
            skill,
        ),
        EffectRequest::FinishLinkIfTargetWounded {
            player,
            target,
            toggle_link,
            skill: _,
        } => finish_link_if_target_wounded(tx, player, target, toggle_link),
        EffectRequest::ChooseRevealOtherHandCardRememberCategory { player, skill } => {
            choose_reveal_other_hand_card_remember_category(tx, player, skill)
        }
        EffectRequest::DrawThenGiveExactRecordPhaseTarget {
            player,
            target,
            count,
            skill,
            ready_to_give,
        } => draw_then_give_exact_record_phase_target(
            tx,
            player,
            target,
            count,
            skill,
            ready_to_give,
        ),
        EffectRequest::ClearPhaseGiftTarget { player, skill } => {
            clear_phase_gift_target(tx, player, skill)
        }
        EffectRequest::EmitShaResolved {
            source,
            card,
            targets,
            response_cards,
            card_flags,
        } => Ok(vec![DomainEvent::ShaResolved {
            source,
            card,
            targets,
            response_cards,
            card_flags,
        }]),
        EffectRequest::EmitCardResolved {
            source,
            card,
            kind,
            targets,
            damaged_targets,
            source_cards,
            all_sources_were_hand,
        } => Ok(vec![DomainEvent::CardResolved {
            source,
            card,
            kind,
            targets,
            damaged_targets,
            source_cards,
            all_sources_were_hand,
        }]),
        EffectRequest::ChooseGainResolvedShaOrShanThenOptionalUseSha {
            player,
            sha_card,
            shan_cards,
            excluded_recipients,
            skill,
        } => choose_gain_resolved_sha_or_shan_then_optional_use_sha(
            tx,
            player,
            sha_card,
            shan_cards,
            excluded_recipients,
            skill,
            false,
        ),
        EffectRequest::ChooseGainAllResolvedShaShanThenColorRewards {
            player,
            sha_card,
            shan_cards,
            excluded_recipients,
            skill,
        } => choose_gain_resolved_sha_or_shan_then_optional_use_sha(
            tx,
            player,
            sha_card,
            shan_cards,
            excluded_recipients,
            skill,
            true,
        ),
        EffectRequest::ChooseUseShaAfterResolvedGift {
            owner,
            player,
            skill,
        } => choose_use_sha_after_resolved_gift(tx, owner, player, skill),
        EffectRequest::SkipCurrentPhase { player } => skip_current_phase(tx, player),
        EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamage {
            player,
            amount,
            skill,
        } => choose_other_targets_by_skipped_phase_count_damage(tx, player, amount, skill),
        EffectRequest::RevealUniformHandDrawSelfChooseOthers { player, skill } => {
            reveal_uniform_hand_draw_self_choose_others(tx, player, skill)
        }
        EffectRequest::ChooseCardUserOrAddTarget {
            player,
            card,
            skill,
        } => choose_card_user_or_add_target(tx, player, card, skill),
        EffectRequest::CommitTransferSavedCardUser {
            previous,
            new_user,
            card,
            skill: _,
        } => {
            crate::command::play::transfer_saved_card_user(tx, previous, new_user, card)?;
            Ok(vec![DomainEvent::CardUserChanged {
                previous,
                player: new_user,
                card,
            }])
        }
        EffectRequest::ChooseDiscardEquipmentReduceCurrentDamage {
            player,
            damage_id,
            skill,
        } => choose_discard_equipment_current_damage(tx, player, damage_id, skill, false),
        EffectRequest::ModifyCurrentDamage {
            player,
            delta,
            skill: _,
        } => modify_current_damage(tx, player, delta),
        EffectRequest::ChooseDiscardEquipmentPreventCurrentDamage {
            player,
            damage_id,
            skill,
        } => choose_discard_equipment_current_damage(tx, player, damage_id, skill, true),
        EffectRequest::ChooseDiscardDamageEndpointCardIncreaseIfSuit {
            player,
            target,
            damage_id,
            suit,
            amount,
            skill,
        } => choose_discard_damage_endpoint_card_increase_if_suit(
            tx, player, target, damage_id, suit, amount, skill,
        ),
        EffectRequest::ChooseMoveTargetEquipmentOrDraw {
            player,
            target,
            skill,
        } => choose_move_target_equipment_or_draw(tx, player, target, skill),
        EffectRequest::ChooseMoveTargetEquipmentAndOrDraw {
            player,
            target,
            skill,
        } => choose_move_target_equipment_and_or_draw(tx, player, target, skill),
        EffectRequest::ChooseDrawViewTwoGainOrMoveEquipment {
            player,
            target,
            skill,
        } => choose_draw_view_two_gain_or_move_equipment(tx, player, target, skill),
        EffectRequest::GiveAllEquipmentChooseRecoverOrDamage {
            player,
            target,
            skill,
        } => give_all_equipment_choose_recover_or_damage(tx, player, target, skill),
        EffectRequest::ChooseRecoverOwnerOrDamageByGivenEquipment {
            owner,
            recipient,
            count,
            skill,
        } => choose_recover_owner_or_damage_by_given_equipment(tx, owner, recipient, count, skill),
        EffectRequest::ChooseTargetDiscardWeaponRangeHandOrIncreaseDamage {
            player,
            target,
            damage_id,
            skill,
        } => choose_target_discard_weapon_range_hand_or_increase_damage(
            tx, player, target, damage_id, skill,
        ),
        EffectRequest::ChooseTargetDiscardDistanceHandOrIncreaseDamage {
            player,
            target,
            damage_id,
            skill,
        } => choose_target_discard_distance_hand_or_increase_damage(
            tx, player, target, damage_id, skill,
        ),
        EffectRequest::ChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth {
            player,
            target,
            damage_id,
            skill,
        } => choose_target_discard_attack_range_cards_or_increase_damage_weapon_forces_both(
            tx, player, target, damage_id, skill,
        ),
        EffectRequest::ChooseDrawOrRecastAddSkill {
            player,
            draw_skill_def,
            recast_skill_def,
            skill,
        } => choose_draw_or_recast_add_skill(tx, player, draw_skill_def, recast_skill_def, skill),
        EffectRequest::ChooseGainShaForOpeningExtraPlay { player, skill } => {
            choose_gain_sha_for_opening_extra_play(tx, player, skill)
        }
        EffectRequest::FinishOpeningExtraPlayShaGain { player, skill } => {
            finish_opening_extra_play_sha_gain(tx, player, skill)
        }
        EffectRequest::RevealTargetHandGainShanArmorOrDrawTwo {
            player,
            target,
            skill,
        } => reveal_target_hand_gain_shan_armor_or_draw_two(tx, player, target, skill),
        EffectRequest::ChooseHandCountDifferentTargetDrawOrRandomDiscard { player, skill } => {
            choose_hand_count_different_target_draw_or_random_discard(tx, player, skill)
        }
        EffectRequest::IncreaseCurrentDrawCount { player, count } => {
            increase_current_draw_count(tx, player, count)
        }
        EffectRequest::SkipJudgeAndDrawPhases { player, skill } => {
            skip_judge_and_draw_phases(tx, player, skill)
        }
        EffectRequest::SkipNextPlayPhase { player } => skip_next_play_phase(tx, player),
        EffectRequest::ChooseOtherDrawOrRecover {
            player,
            draw_count,
            recover_count,
            times,
            skill,
        } => choose_other_draw_or_recover(tx, player, draw_count, recover_count, times, skill),
        EffectRequest::ChooseOtherRecover { player, amount } => {
            choose_other_recover(tx, player, amount)
        }
        EffectRequest::EnableEquipSlot {
            player,
            slot,
            source,
        } => enable_equip_slot(tx, player, slot, source),
        EffectRequest::ChooseDiscardHandEquipThenRemoveSelfSkill {
            player,
            count,
            skill,
        } => choose_discard_hand_equip_then_remove_self_skill(tx, player, count, skill),
        EffectRequest::ChooseDiscardHandEquipThenRecover {
            player,
            count,
            amount,
            skill,
        } => choose_discard_hand_equip_then_recover(tx, player, count, amount, skill),
        EffectRequest::DrawSelectedTargets {
            targets,
            count,
            skill: _,
        } => draw_selected_targets(tx, &targets, count),
        EffectRequest::DamageThenDrawSelectedTargets {
            source,
            targets,
            damage,
            draw,
            skill: _,
        } => damage_then_draw_selected_targets(tx, source, &targets, damage, draw),
        EffectRequest::ResolveYoudi {
            player,
            target,
            card,
            stage,
            skill,
        } => resolve_youdi(tx, player, target, card, stage, skill),
        EffectRequest::ResolveKuangfu {
            player,
            target,
            card,
            stage,
            skill,
        } => resolve_kuangfu(tx, player, target, card, stage, skill),
        EffectRequest::ResolveShanjia {
            player,
            count,
            stage,
            skill,
        } => resolve_shanjia(tx, player, count, stage, skill),
        EffectRequest::ResolveNewYajiao {
            player,
            source_card,
            revealed,
            target,
            source_category,
            stage,
            skill,
        } => resolve_new_yajiao(
            tx,
            player,
            source_card,
            revealed,
            target,
            source_category,
            stage,
            skill,
        ),
        EffectRequest::ResolveDingpin {
            player,
            target,
            skill,
        } => resolve_dingpin(tx, player, target, skill),
        EffectRequest::FinishCurrentPhaseAbortResolution { player, stage } => {
            finish_current_phase_abort_resolution(tx, player, stage)
        }
        EffectRequest::RecoverSelectedTargets {
            source,
            targets,
            amount,
            skill: _,
        } => recover_selected_targets(tx, source, &targets, amount),
        EffectRequest::RecordSelectedPlayPhaseCategory { card, skill } => {
            record_selected_play_phase_category(tx, card, skill)
        }
        EffectRequest::RecoverIfMinCurrentHp { player, amount } => {
            recover_if_min_current_hp(tx, player, amount)
        }
        EffectRequest::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw { player, skill } => {
            choose_target_discard_excess_then_self_discard_or_target_draw(tx, player, skill)
        }
        EffectRequest::ChooseSelfDiscardOrTargetDrawAfterTargetDiscard {
            player,
            target,
            count,
            skill,
        } => choose_self_discard_or_target_draw_after_target_discard(
            tx, player, target, count, skill,
        ),
        EffectRequest::DrawThenChooseGainOneFromDamageTargetAndNeighbors {
            player,
            target,
            skill,
            ready_to_choose,
        } => draw_then_choose_gain_one_from_damage_target_and_neighbors(
            tx,
            player,
            target,
            skill,
            ready_to_choose,
        ),
        EffectRequest::ContinueChooseGainOneFromDamageTargetAndNeighbors {
            player,
            targets,
            index,
            selected,
            skill,
        } => continue_choose_gain_one_from_damage_target_and_neighbors(
            tx, player, targets, index, selected, skill,
        ),
        EffectRequest::ChoosePreventDamageOrLoseMaxHpIncreaseDamage {
            player,
            damage_id,
            amount,
            skill,
        } => choose_prevent_damage_or_lose_max_hp_increase_damage(
            tx, player, damage_id, amount, skill,
        ),
        EffectRequest::ContinueIncreaseBoundDamage {
            player,
            damage_id,
            amount,
            ..
        } => {
            if !bound_damage_source_matches(tx, damage_id, player) {
                return Err(EffectError::Illegal(
                    "bound damage increase context changed".into(),
                ));
            }
            increase_bound_damage(tx, damage_id, amount)?;
            Ok(Vec::new())
        }
        EffectRequest::AddCardUsePolicySkill {
            player,
            card,
            policy_mask,
            skill_def,
        } => add_card_use_policy_skill(tx, player, card, policy_mask, skill_def),
        EffectRequest::DiscardTargetCards {
            actor,
            target,
            cards,
            scope,
        } => discard_target_cards(tx, actor, target, cards, scope),
        EffectRequest::RemoveSkillByDef { player, skill_def } => {
            remove_skill_by_def(tx, player, skill_def)
        }
        EffectRequest::ChooseDiscardHandAddSkillByColor {
            player,
            red_skill_def,
            black_skill_def,
        } => choose_discard_hand_add_skill_by_color(tx, player, red_skill_def, black_skill_def),
        EffectRequest::RevealTopGainNonBasicOrSwapBasic { player } => {
            reveal_top_gain_non_basic_or_swap_basic(tx, player)
        }
        EffectRequest::PreventDamage { damage_id } => {
            if damage_id == 0 {
                return Err(EffectError::ContentFault(
                    "prevent_current_damage used outside damage_before".into(),
                ));
            }
            if !tx
                .state()
                .resolution
                .prevented_damage_ids
                .contains(&damage_id)
            {
                tx.state_mut()
                    .resolution
                    .prevented_damage_ids
                    .push(damage_id);
            }
            Ok(Vec::new())
        }
        EffectRequest::CommitDamage {
            damage_id,
            source,
            target,
            amount,
            base_amount,
            is_lose_hp,
            nature,
            card,
            card_kind,
            card_flags,
        } => {
            let (mut events, followups) = commit_damage(
                tx,
                damage_id,
                source,
                target,
                amount,
                base_amount,
                is_lose_hp,
                nature,
                card,
                card_kind,
                card_flags,
            )?;
            for followup in followups {
                events.extend(apply_effect(tx, followup)?);
            }
            Ok(events)
        }
        EffectRequest::CommitDamageBanActiveIfDying {
            damage_id,
            source,
            target,
            amount,
            base_amount,
            nature,
            skill,
        } => {
            let (mut events, followups) = commit_damage_ban_active_if_dying(
                tx,
                damage_id,
                source,
                target,
                amount,
                base_amount,
                nature,
                skill,
            )?;
            for followup in followups {
                events.extend(apply_effect(tx, followup)?);
            }
            Ok(events)
        }
        EffectRequest::CommitLoseHpThenDamageOtherLoseMaxHpIfDying {
            player,
            target,
            amount,
            skill,
        } => {
            let (events, followups) = commit_lose_hp_then_damage_other_lose_max_hp_if_dying(
                tx, player, target, amount, skill,
            )?;
            debug_assert!(followups.is_empty());
            Ok(events)
        }
        EffectRequest::MoveCards { cards, to, from } => move_cards(tx, cards, to, from),
        EffectRequest::MoveCardsWithReason {
            cards,
            to,
            from,
            reason,
        } => move_cards_with_reason(tx, cards, to, from, reason),
        EffectRequest::GiveCards { cards, to, from } => give_cards(tx, cards, to, from),
        EffectRequest::UseCard {
            player,
            card,
            targets,
        } => use_card(tx, player, card, targets),
        EffectRequest::UseFieldCard {
            player,
            card,
            targets,
        } => use_field_card(tx, player, card, targets),
        EffectRequest::DealDamage {
            source,
            target,
            amount,
            is_lose_hp,
            nature,
        } => deal_damage(
            tx,
            source,
            target,
            amount,
            is_lose_hp,
            nature,
            false,
            CardId::NONE,
            0,
            0,
        ),
        EffectRequest::DealDamageBanActiveIfDying {
            source,
            target,
            amount,
            skill,
        } => {
            let Some(EffectRequest::CommitDamage {
                damage_id,
                source,
                target,
                amount,
                base_amount,
                ..
            }) = prepare_damage(
                tx.state(),
                source,
                target,
                amount,
                false,
                rule_api::damage_nature::NORMAL,
                false,
                CardId::NONE,
            )?
            else {
                return Ok(Vec::new());
            };
            let (mut events, followups) = commit_damage_ban_active_if_dying(
                tx,
                damage_id,
                source,
                target,
                amount,
                base_amount,
                rule_api::damage_nature::NORMAL,
                skill,
            )?;
            for followup in followups {
                events.extend(apply_effect(tx, followup)?);
            }
            Ok(events)
        }
        EffectRequest::SpjungongDiscardOrLoseThenSha {
            player,
            target,
            skill,
        } => spjungong_discard_or_lose_then_sha(tx, player, target, skill),
        EffectRequest::BanActiveUntilTurnEnd { player, skill } => {
            ban_active_until_turn_end(tx, player, skill)
        }
        EffectRequest::SpwanweiActive { player, target } => spwanwei_active(tx, player, target),
        EffectRequest::SpwanweiDying { player, target } => spwanwei_dying(tx, player, target),
        EffectRequest::ResolveSpxizhan {
            player,
            phase_player,
            fangzong_skill_def,
        } => resolve_spxizhan(tx, player, phase_player, fangzong_skill_def),
        EffectRequest::LoseHpThenDamageOtherLoseMaxHpIfDying {
            player,
            target,
            amount,
            skill,
        } => {
            let Some(EffectRequest::CommitDamage { amount, .. }) = prepare_damage(
                tx.state(),
                PlayerId::NONE,
                player,
                amount,
                true,
                rule_api::damage_nature::NORMAL,
                false,
                CardId::NONE,
            )?
            else {
                return Ok(Vec::new());
            };
            let (events, followups) = commit_lose_hp_then_damage_other_lose_max_hp_if_dying(
                tx, player, target, amount, skill,
            )?;
            debug_assert!(followups.is_empty());
            Ok(events)
        }
        EffectRequest::DealShaDamage {
            source,
            target,
            card,
            amount,
            nature,
            card_flags,
        } => deal_damage(
            tx,
            source,
            target,
            amount,
            false,
            nature,
            true,
            card,
            CardKind::Sha.tag(),
            card_flags,
        ),
        EffectRequest::DealCardDamage {
            source,
            target,
            amount,
            is_lose_hp,
            nature,
            card,
            card_kind,
            card_flags,
        } => deal_damage(
            tx, source, target, amount, is_lose_hp, nature, false, card, card_kind, card_flags,
        ),
        EffectRequest::RecoverHp {
            source,
            player,
            amount,
        } => recover_hp(tx, source, player, amount),
        EffectRequest::PreventRecovery { recovery_id } => {
            if recovery_id == 0 || tx.state().resolution.current_recovery_id != recovery_id {
                return Err(EffectError::ContentFault(
                    "prevent recovery requested outside recover_before".into(),
                ));
            }
            if !tx
                .state()
                .resolution
                .prevented_recovery_ids
                .contains(&recovery_id)
            {
                tx.state_mut()
                    .resolution
                    .prevented_recovery_ids
                    .push(recovery_id);
            }
            Ok(Vec::new())
        }
        EffectRequest::CommitRecovery {
            recovery_id,
            source,
            player,
            amount,
        } => {
            if recovery_id != 0 {
                if tx.state().resolution.current_recovery_id == recovery_id {
                    tx.state_mut().resolution.current_recovery_id = 0;
                }
                if let Some(index) = tx
                    .state()
                    .resolution
                    .prevented_recovery_ids
                    .iter()
                    .position(|id| *id == recovery_id)
                {
                    tx.state_mut()
                        .resolution
                        .prevented_recovery_ids
                        .remove(index);
                    return Ok(Vec::new());
                }
            }
            recover_hp(tx, source, player, amount)
        }
        EffectRequest::ChangeMaxHp { player, delta } => change_max_hp(tx, player, delta),
        EffectRequest::AddSkill { player, skill_def } => add_skill(tx, player, skill_def),
        EffectRequest::AddBoundSkill {
            player,
            skill_def,
            bound_target,
        } => add_bound_skill(tx, player, skill_def, bound_target),
        EffectRequest::RemoveSkill { skill } => remove_skill(tx, skill),
        EffectRequest::RefreshSkill { skill } => refresh_skill(tx, skill),
        EffectRequest::TagCard { cards, tag } => tag_cards(tx, cards, tag),
        EffectRequest::UntagCard { cards, tag } => untag_cards(tx, cards, tag),
        EffectRequest::ClanshenjunReveal {
            player,
            event_card,
            skill,
        } => clanshenjun_reveal(tx, player, event_card, skill),
        EffectRequest::ClanshenjunUse { player, skill } => {
            clanshenjun_use(tx, player, skill)
        }
        EffectRequest::StorageSet { skill, key, value } => storage_set(tx, skill, key, value),
        EffectRequest::DrawAllEventTargets { players, count } => {
            draw_all_event_targets(tx, players, count)
        }
        EffectRequest::RoundEnded { round, current } => {
            Ok(vec![DomainEvent::RoundEnded { round, current }])
        }
        EffectRequest::RoundStarted { round, current } => {
            Ok(vec![DomainEvent::RoundStarted { round, current }])
        }
        EffectRequest::ClanyuzhiReveal { player, skill } => {
            clanyuzhi_reveal(tx, player, skill)
        }
        EffectRequest::ClanyuzhiRoundEnd { player, skill } => {
            clanyuzhi_round_end(tx, player, skill)
        }
        EffectRequest::QieyiPhaseStart { player, skill } => {
            qieyi_phase_start(tx, player, skill)
        }
        EffectRequest::QieyiUseAfter { player, card, skill } => {
            qieyi_use_after(tx, player, card, skill)
        }
        EffectRequest::JianzhiPhaseEnd { player, skill } => {
            jianzhi_phase_end(tx, player, skill)
        }
        EffectRequest::AddMark {
            player,
            mark,
            delta,
        } => add_mark(tx, player, mark.raw(), delta),
        EffectRequest::RemoveMark {
            player,
            mark,
            amount,
        } => remove_mark(tx, player, mark.raw(), amount),
        EffectRequest::RevealCards { cards, to } => {
            Ok(vec![DomainEvent::CardsRevealed { cards, to }])
        }
        EffectRequest::RevealHandDiscardKind { player, kind } => {
            reveal_hand_discard_kind(tx, player, kind)
        }
        EffectRequest::RequestChoice {
            observer,
            min_count,
            max_count,
            allow_cancel,
            option_tokens,
        } => request_choice(
            tx,
            observer,
            min_count,
            max_count,
            allow_cancel,
            option_tokens,
            ChoiceKind::Custom,
        ),
        EffectRequest::RequestDyingSave { player, source } => {
            request_dying_save(tx, player, source)
        }
        EffectRequest::ContinueDyingSave {
            player,
            source,
            responders,
        } => request_dying_save_from_queue(tx, player, source, responders),
        EffectRequest::EnterDying { player } => {

            enter_dying(tx, player)
        }
        EffectRequest::KillPlayer { player, killer } => kill_player(tx, player, killer),
        EffectRequest::SetTurnState { current, phase } => set_turn(tx, current, phase),
        EffectRequest::AdvanceTurn => {
            crate::command::turn::finish_turn(tx)?;
            Ok(Vec::new())
        }
        EffectRequest::EmitRuleEvent { code, payload } => {
            Ok(vec![DomainEvent::RuleNote { code, payload }])
        }
        EffectRequest::EmitDebateResolved {
            initiator,
            red,
            black,
            result,
        } => Ok(vec![DomainEvent::DebateResolved {
            initiator,
            red,
            black,
            result,
        }]),
        EffectRequest::DiscardFromHand { player, count } => {

            discard_from_hand(tx, player, count)
        }
        EffectRequest::DiscardFromHandEquip { player, count } => {

            discard_from_hand_equip(tx, player, count)
        }
        EffectRequest::AdjustHandTo { player, count } => {

            adjust_hand_to(tx, player, count)
        }
        EffectRequest::DiscardCards { player, cards } => {

            discard_cards(tx, player, cards)
        }
        EffectRequest::ChooseDiscardOneHandOrEquip { player } => {

            choose_discard_one_hand_or_equip(tx, player)
        }
        EffectRequest::DiscardAllHandAndEquip { player } => {

            discard_all_hand_and_equip(tx, player)
        }
        EffectRequest::RemoveAllSkills { player } => remove_all_skills(tx, player),
        EffectRequest::JudgeGainEquipmentByColor { player } => {
            judge_gain_equipment_by_color(tx, player)
        }
        EffectRequest::JudgeCancelCurrentShaTargetOnRed {
            player,
            card,
            target,
        } => judge_cancel_current_sha_target_on_red(tx, player, card, target),
        EffectRequest::MakeCurrentShaTargetUnrespondable {
            source,
            card,
            target,
        } => make_current_sha_target_unrespondable(tx, source, card, target),
        EffectRequest::JudgeMakeCurrentShaTargetUnrespondableOnRed {
            player,
            card,
            target,
            retieji,
            skill,
            ready_to_choose,
            judgment_suit,
        } => judge_make_current_sha_target_unrespondable_on_red(
            tx,
            player,
            card,
            target,
            retieji,
            skill,
            ready_to_choose,
            judgment_suit,
        ),
        EffectRequest::ChooseDiscardBasicThenTargetDiscardEquipOrDamage {
            player,
            target,
            skill,
        } => choose_discard_basic_then_target_discard_equip_or_damage(tx, player, target, skill),
        EffectRequest::TargetDiscardThenDuelIfNonShaAndHpGteOwner {
            player,
            target,
            skill,
        } => target_discard_then_duel_if_non_sha_and_hp_gte_owner(tx, player, target, skill),
        EffectRequest::ChooseTargetLoseHpDiscardHandShaIfBasic {
            player,
            target,
            skill,
            stage,
            discarded_basic,
        } => choose_target_lose_hp_discard_hand_sha_if_basic(
            tx,
            player,
            target,
            skill,
            stage,
            discarded_basic,
        ),
        EffectRequest::ChooseCurrentShaTargetDiscardRedirectToOwner {
            owner,
            source,
            card,
            target,
            skill,
            ready_to_redirect,
        } => choose_current_sha_target_discard_redirect_to_owner(
            tx,
            owner,
            source,
            card,
            target,
            skill,
            ready_to_redirect,
        ),
        EffectRequest::ChooseDiscardRedirectCurrentSha {
            player,
            source,
            card,
            target,
            skill,
        } => choose_discard_redirect_current_sha(tx, player, source, card, target, skill),
        EffectRequest::ChooseTargetGainBottomCard {
            player,
            card,
            target,
            skill,
        } => choose_target_gain_bottom_card(tx, player, card, target, skill),
        EffectRequest::JudgeBlackGainRepeat {
            player,
            card,
            skill,
            ready_to_choose,
        } => judge_black_gain_repeat(tx, player, card, skill, ready_to_choose, false),
        EffectRequest::JudgeBlackGainRepeatProtectHandLimit {
            player,
            card,
            skill,
            ready_to_choose,
        } => judge_black_gain_repeat(tx, player, card, skill, ready_to_choose, true),
        EffectRequest::JudgeSameColorGainRepeat {
            player,
            card,
            skill,
            ready_to_choose,
            color,
        } => judge_same_color_gain_repeat(tx, player, card, skill, ready_to_choose, color),
        EffectRequest::ChooseCurrentTrickTargetsToNullify {
            player,
            source,
            card,
            skill,
        } => choose_current_trick_targets_to_nullify(tx, player, source, card, skill),
        EffectRequest::UseVirtualJuedouSecondOnFirst { targets, card } => {
            use_virtual_juedou_second_on_first(tx, targets, card)
        }
        EffectRequest::JudgeSetSkillMarkToRank { player, skill } => {
            judge_set_skill_mark_to_rank(tx, player, skill)
        }
        EffectRequest::ChooseDiscardOneFromEachLinked { player, skill } => {
            choose_discard_one_from_each_linked(tx, player, skill)
        }
        EffectRequest::DisableSkillByDefUntilTurnEnd { player, skill_def } => {
            disable_skill_by_def_until_turn_end(tx, player, skill_def)
        }
        EffectRequest::ChooseDiscardTargetCardResolveCurrentSha {
            source,
            card,
            target,
        } => choose_discard_target_card_resolve_current_sha(tx, source, card, target, None),
        EffectRequest::ChooseDiscardTargetCardResolveCurrentShaNonbasicBonus {
            source,
            card,
            target,
            bonus_skill_def,
        } => choose_discard_target_card_resolve_current_sha(
            tx,
            source,
            card,
            target,
            Some(bonus_skill_def),
        ),
        EffectRequest::ResolveDcqiaomeng {
            player,
            card,
            targets,
            target,
            selected,
            helper_skill_def,
            stage,
            skill,
        } => resolve_dcqiaomeng(
            tx,
            player,
            card,
            targets,
            target,
            selected,
            helper_skill_def,
            stage,
            skill,
        ),
        EffectRequest::ResolveNewReluoyi {
            player,
            buff_skill_def,
            stage,
            cards,
            accepted,
            skill,
        } => resolve_new_reluoyi(tx, player, buff_skill_def, stage, cards, accepted, skill),
        EffectRequest::ResolveRemoshi {
            player,
            target,
            card,
            helper_skill_def,
            action,
            skill,
        } => resolve_remoshi(tx, player, target, card, helper_skill_def, action, skill),
        EffectRequest::ResolveXinyaoming {
            player,
            target,
            action,
            skill,
        } => resolve_xinyaoming(tx, player, target, action, skill),
        EffectRequest::ResolveRehuaiyi {
            player,
            stage,
            color,
            discarded_count,
            targets,
            index,
            gained_count,
            skill,
        } => resolve_rehuaiyi(
            tx,
            player,
            stage,
            color,
            discarded_count,
            targets,
            index,
            gained_count,
            skill,
        ),
        EffectRequest::ChooseDrawOrRecover {
            player,
            draw_count,
            recover_count,
            allow_cancel,
        } => choose_draw_or_recover(tx, player, draw_count, recover_count, allow_cancel),
        EffectRequest::ChooseUseVirtualSha {
            player,
            ignore_distance,
        } => choose_use_virtual_sha(
            tx,
            player,
            ignore_distance,
            false,
            false,
            engine_api::SkillId::NONE,
            0,
            0,
        ),
        EffectRequest::ChooseUseVirtualShaWithBonus {
            player,
            ignore_distance,
            damage_bonus,
        } => choose_use_virtual_sha(
            tx,
            player,
            ignore_distance,
            false,
            false,
            engine_api::SkillId::NONE,
            0,
            damage_bonus,
        ),
        EffectRequest::ChooseUseVirtualShaOrDraw { player, skill } => {
            choose_use_virtual_sha(tx, player, false, true, true, skill, 0, 0)
        }
        EffectRequest::UseVirtualCardOn {
            player,
            kind,
            target,
        } => use_virtual_card_on(tx, player, kind, target),
        EffectRequest::UseVirtualCardOnSelectedTargets {
            player,
            kind,
            targets,
        } => use_virtual_card_on_selected_targets(tx, player, kind, targets),
        EffectRequest::ChooseTargetReclaimEquipmentDrawIfRangeReduced {
            player,
            target,
            skill,
        } => choose_target_reclaim_equipment_draw_if_range_reduced(tx, player, target, skill),
        EffectRequest::CommitTargetReclaimEquipmentDrawIfRangeReduced {
            player,
            target,
            card,
            before,
            skill,
        } => commit_target_reclaim_equipment_draw_if_range_reduced(
            tx, player, target, card, before, skill,
        ),
        EffectRequest::FinishTargetReclaimEquipmentDrawIfRangeReduced {
            player,
            target,
            before,
            skill: _,
        } => finish_target_reclaim_equipment_draw_if_range_reduced(tx, player, target, before),
        EffectRequest::ChooseUseVirtualCard {
            player,
            kind,
            skill,
        } => choose_use_virtual_card(tx, player, kind, skill),
        EffectRequest::ChooseUseVirtualNanman { player, skill } => {
            choose_use_virtual_card_impl(
                tx,
                player,
                CardKind::TrickNanman.tag(),
                skill,
                VirtualCardTracking::None,
                false,
            )
        }
        EffectRequest::ResolveQiluan { player, skill } => resolve_qiluan(tx, player, skill),
        EffectRequest::DiscardAllOwnJudge { player, skill } => {
            let _ = skill;
            discard_all_judge(tx, player)
        }
        EffectRequest::StoreEventCardInSkillPile {
            player,
            card,
            skill,
        } => store_event_card_in_skill_pile(tx, player, card, skill),
        EffectRequest::FinishPhaseEndVirtualCard => {
            tx.state_mut().turn.clear_phase_histories();
            tx.state_mut()
                .turn
                .set_flag(turn_flag::PHASE_HISTORY_SUPPRESSED, 0);
            Ok(Vec::new())
        }
        EffectRequest::EmitDuelResolved {
            card,
            initiator,
            target,
            winner,
            loser,
        } => {
            let card_flags = tx
                .state_mut()
                .resolution
                .take_duel_damage(card)
                .then_some(rule_api::damage_card_flag::CARD_USE_DEALT_DAMAGE)
                .unwrap_or(0);
            Ok(vec![DomainEvent::DuelResolved {
                card,
                card_flags,
                initiator,
                target,
                winner,
                loser,
            }])
        }
        EffectRequest::EmitJudgmentResolved {
            player,
            result_card,
            suit,
            rank,
            cause_card,
        } => Ok(vec![DomainEvent::JudgmentResolved {
            player,
            result_card,
            suit,
            rank,
            cause_card,
        }]),
        EffectRequest::RevealDiscardSuitGainOrUse {
            player,
            mark,
            base_count,
            max_mark,
            revealed,
            completed,
            step,
            data,
        } => crate::command::reveal_options::apply(
            tx, player, mark, base_count, max_mark, revealed, completed, step, data,
        ),
        EffectRequest::ChooseDiscardOrFireDamageSkipPhases {
            player,
            max_discard,
            damage,
            selected,
            step,
            target,
        } => crate::command::discard_damage_options::apply(
            tx,
            player,
            max_discard,
            damage,
            selected,
            step,
            target,
        ),
        EffectRequest::TakeFromHand { from, to } => take_from_hand(tx, from, to),
        EffectRequest::CardResponded {
            player,
            respond_to,
            card,
            response_kind,
        } => {
            if tx.state().players.get(player).is_none() {
                return Err(EffectError::Illegal("bad responding player".into()));
            }
            if card.is_some() && tx.state().cards.get(card).is_none() {
                return Err(EffectError::Illegal("bad response card".into()));
            }
            if respond_to.is_some() && tx.state().players.get(respond_to).is_none() {
                return Err(EffectError::Illegal("bad response origin player".into()));
            }
            Ok(vec![DomainEvent::CardResponded {
                player,
                respond_to,
                card,
                response_kind,
            }])
        }
        EffectRequest::ShaDodged {
            source,
            target,
            card,
        } => {
            if ![source, target]
                .into_iter()
                .all(|player| tx.state().players.get(player).is_some())
            {
                return Err(EffectError::Illegal(
                    "sha-dodged participants must exist".into(),
                ));
            }
            if card.is_some()
                && (!tx.state().cards.get(card).is_some()
                    || !tx
                        .state()
                        .resolution
                        .sha_target_resolving(source, card, target))
            {
                return Err(EffectError::Illegal(
                    "sha-dodged card is not the current logical Sha carrier".into(),
                ));
            }
            Ok(vec![DomainEvent::ShaDodged {
                source,
                target,
                card,
            }])
        }
        EffectRequest::RemoveCardsByKind { kind } => remove_cards_by_kind(tx, kind),
    }
}
