//! 效果请求的规范哈希编码：choose、draw 族效果。

use super::*;

pub(super) fn hash_effect(h: &mut blake3::Hasher, effect: &EffectRequest) {
    match effect {
        EffectRequest::DrawCards { player, count } => {
            put_u8(h, 1);
            put_u32(h, player.raw());
            put_u32(h, *count);
        }

        EffectRequest::ChooseDiscardOneHandOrEquip { player } => {
            put_u8(h, 23);
            put_u32(h, player.raw());
        }

        EffectRequest::ChooseDiscardOrFireDamageSkipPhases {
            player,
            max_discard,
            damage,
            selected,
            step,
            target,
        } => {
            put_u8(h, 26);
            put_u32(h, player.raw());
            put_u8(h, *max_discard);
            put_u8(h, *damage);
            put_u8(h, *selected);
            put_u8(h, *step);
            put_u32(h, target.raw());
        }

        EffectRequest::DrawCardsFromSkill {
            player,
            count,
            skill,
        } => {
            put_u8(h, 27);
            put_u32(h, player.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
        }

        EffectRequest::DrawEachBySeat {
            first,
            second,
            count,
            skill,
        } => {
            put_u8(h, 30);
            put_u32(h, first.raw());
            put_u32(h, second.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
        }

        EffectRequest::DrawThenGiveOne {
            player,
            target,
            count,
            drawn_only,
            use_given_equipment,
            skill,
            drawn,
            ready_to_give,
        } => {
            put_u8(h, 31);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *count);
            put_bool(h, *drawn_only);
            put_bool(h, *use_given_equipment);
            put_u32(h, skill.raw());
            put_ids(h, drawn.iter().map(|card| card.raw()));
            put_bool(h, *ready_to_give);
        }

        EffectRequest::DrawThenTakeOne {
            player,
            taker,
            count,
            skill,
            ready_to_take,
        } => {
            put_u8(h, 32);
            put_u32(h, player.raw());
            put_u32(h, taker.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_take);
        }

        EffectRequest::ChooseDiscardHandAddSkillByColor {
            player,
            red_skill_def,
            black_skill_def,
        } => {
            put_u8(h, 36);
            put_u32(h, player.raw());
            put_u32(h, red_skill_def.raw());
            put_u32(h, black_skill_def.raw());
        }

        EffectRequest::ChooseDrawOrRecover {
            player,
            draw_count,
            recover_count,
            allow_cancel,
        } => {
            put_u8(h, 41);
            put_u32(h, player.raw());
            put_u32(h, *draw_count);
            put_u32(h, *recover_count);
            put_bool(h, *allow_cancel);
        }

        EffectRequest::ChooseUseVirtualSha {
            player,
            ignore_distance,
        } => {
            put_u8(h, 43);
            put_u32(h, player.raw());
            put_bool(h, *ignore_distance);
        }

        EffectRequest::ChooseUseVirtualShaWithBonus {
            player,
            ignore_distance,
            damage_bonus,
        } => {
            put_u8(h, 120);
            put_u32(h, player.raw());
            put_bool(h, *ignore_distance);
            put_u32(h, *damage_bonus);
        }

        EffectRequest::ChooseUseVirtualShaOrDraw { player, skill } => {
            put_u8(h, 54);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseDiscardTargetCard {
            player,
            target,
            scope,
        } => {
            put_u8(h, 61);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, scope.tag());
        }

        EffectRequest::ChooseGainTopCardsBySuit {
            player,
            count,
            suit,
        } => {
            put_u8(h, 63);
            put_u32(h, player.raw());
            put_u8(h, *count);
            put_u8(h, suit_tag(*suit));
        }

        EffectRequest::ChooseOtherDrawByHandCount {
            player,
            normal_count,
            empty_count,
            times,
            skill,
        } => {
            put_u8(h, 64);
            put_u32(h, player.raw());
            put_u32(h, *normal_count);
            put_u32(h, *empty_count);
            put_u32(h, *times);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseDiscardOneThenGainTargetEquipment {
            player,
            target,
            slot,
        } => {
            put_u8(h, 65);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *slot);
        }

        EffectRequest::ChooseTargetDraw {
            player,
            count,
            scope,
            skill,
        } => {
            put_u8(h, 66);
            put_u32(h, player.raw());
            put_u32(h, *count);
            put_u8(h, scope.tag());
            put_u32(h, skill.raw());
        }

        EffectRequest::DrawByPlayPhaseCardCategoryCount { player, skill } => {
            put_u8(h, 67);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRed {
            player,
            sha_user,
            card,
            skill,
        } => {
            put_u8(h, 69);
            put_u32(h, player.raw());
            put_u32(h, sha_user.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseAddCurrentShaTarget {
            player,
            card,
            skill,
        } => {
            put_u8(h, 75);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseStoreHandShaInSkillPile { player, skill } => {
            put_u8(h, 77);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseSpendSkillPileForDyingJiu {
            player,
            dying,
            skill,
        } => {
            put_u8(h, 78);
            put_u32(h, player.raw());
            put_u32(h, dying.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseOtherGiveThenRecover { player, skill } => {
            put_u8(h, 133);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseHandCompareBetween {
            player,
            first,
            skill,
        } => {
            put_u8(h, 131);
            put_u32(h, player.raw());
            put_u32(h, first.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseHandCompareGrantTargetOnWin {
            player,
            win_grant_def,
            lose_skill_def,
            skill,
        } => {
            put_u8(h, 129);
            put_u32(h, player.raw());
            put_u32(h, win_grant_def.raw());
            put_u32(h, lose_skill_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseCompareSelectedHandCard {
            player,
            target,
            card,
            win_skill_def,
            lose_skill_def,
            skill,
        } => {
            put_u8(h, 81);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, win_skill_def.raw());
            put_u32(h, lose_skill_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseAddSkillBoundCardTarget {
            player,
            card,
            skill,
        } => {
            put_u8(h, 83);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseDiscardMatchingEquipColorAddSkill {
            player,
            target,
            skill_def,
            skill,
        } => {
            put_u8(h, 85);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseCurrentCardUseEnhancements {
            player,
            card,
            helper_skill_def,
            skill,
        } => {
            put_u8(h, 88);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, helper_skill_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseDrawOrDiscardAddSkill {
            player,
            draw_skill_def,
            discard_skill_def,
            skill,
            three_way,
        } => {
            put_u8(h, 90);
            put_u32(h, player.raw());
            put_u32(h, draw_skill_def.raw());
            put_u32(h, discard_skill_def.raw());
            put_u32(h, skill.raw());
            if *three_way {
                put_u8(h, 1);
            }
        }

        EffectRequest::ChooseDiscardedBlackBasicAsBingliang {
            player,
            cards,
            skill,
        } => {
            put_u8(h, 91);
            put_u32(h, player.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseDiscardJudgeUseVirtualShaDrawIfNoDamage {
            player,
            target,
            skill,
        } => {
            put_u8(h, 92);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded {
            player,
            target,
            toggle_link,
            skill,
        } => {
            put_u8(h, 95);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
            if *toggle_link {
                put_u8(h, 1);
            }
        }

        EffectRequest::ChooseRevealOtherHandCardRememberCategory { player, skill } => {
            put_u8(h, 97);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::DrawThenGiveExactRecordPhaseTarget {
            player,
            target,
            count,
            skill,
            ready_to_give,
        } => {
            put_u8(h, 98);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_give);
        }

        EffectRequest::ChooseGainResolvedShaOrShanThenOptionalUseSha {
            player,
            sha_card,
            shan_cards,
            excluded_recipients,
            skill,
        } => {
            put_u8(h, 101);
            put_u32(h, player.raw());
            put_u32(h, sha_card.raw());
            put_ids(h, shan_cards.iter().map(|card| card.raw()));
            put_ids(h, excluded_recipients.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseUseShaAfterResolvedGift {
            owner,
            player,
            skill,
        } => {
            put_u8(h, 102);
            put_u32(h, owner.raw());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamage {
            player,
            amount,
            skill,
        } => {
            put_u8(h, 104);
            put_u32(h, player.raw());
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }
        EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamageDraw {
            player,
            amount,
            skill,
        } => {
            put_u8(h, 106);
            put_u32(h, player.raw());
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseCardUserOrAddTarget {
            player,
            card,
            skill,
        } => {
            put_u8(h, 106);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseDiscardEquipmentReduceCurrentDamage {
            player,
            damage_id,
            skill,
        } => {
            put_u8(h, 108);
            put_u32(h, player.raw());
            put_u32(h, *damage_id);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseDiscardEquipmentPreventCurrentDamage {
            player,
            damage_id,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&434u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *damage_id);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseMoveTargetEquipmentOrDraw {
            player,
            target,
            skill,
        } => {
            put_u8(h, 109);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseRecoverOwnerOrDamageByGivenEquipment {
            owner,
            recipient,
            count,
            skill,
        } => {
            put_u8(h, 111);
            put_u32(h, owner.raw());
            put_u32(h, recipient.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseTargetDiscardWeaponRangeHandOrIncreaseDamage {
            player,
            target,
            damage_id,
            skill,
        } => {
            put_u8(h, 112);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *damage_id);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseHandCountDifferentTargetDrawOrRandomDiscard { player, skill } => {
            put_u8(h, 113);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw { player, skill } => {
            put_u8(h, 115);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseSelfDiscardOrTargetDrawAfterTargetDiscard {
            player,
            target,
            count,
            skill,
        } => {
            put_u8(h, 116);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
        }

        EffectRequest::DrawThenChooseGainOneFromDamageTargetAndNeighbors {
            player,
            target,
            skill,
            ready_to_choose,
        } => {
            put_u8(h, 117);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_choose);
        }

        EffectRequest::ChoosePreventDamageOrLoseMaxHpIncreaseDamage {
            player,
            damage_id,
            amount,
            skill,
        } => {
            put_u8(h, 119);
            put_u32(h, player.raw());
            put_u32(h, *damage_id);
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseAddOrRemoveCurrentCardTarget {
            player,
            card,
            skill,
        } => {
            put_u8(h, 126);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseHandCompare {
            player,
            win_skill_def,
            lose_skill_def,
            skill,
        } => {
            put_u8(h, 127);
            put_u32(h, player.raw());
            put_u32(h, win_skill_def.raw());
            put_u32(h, lose_skill_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseOtherDamage {
            player,
            amount,
            skill,
        } => {
            put_u8(h, 128);
            put_u32(h, player.raw());
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseOtherDiscardTargetCard {
            player,
            scope,
            skill,
        } => {
            put_u8(h, 129);
            put_u32(h, player.raw());
            put_u8(h, scope.tag());
            put_u32(h, skill.raw());
        }

        _ => unreachable!("effect routed to a different hash group"),
    }
}
