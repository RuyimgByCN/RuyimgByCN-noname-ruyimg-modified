//! 效果请求的规范哈希编码：choose 族效果。

use super::*;

pub(super) fn hash_effect(h: &mut blake3::Hasher, effect: &EffectRequest) {
    match effect {
        EffectRequest::ChooseDrawOrIncreaseCurrentCardDamageOrBothMark {
            player,
            card,
            draw_count,
            damage_amount,
            mark_def,
            mark_count,
            skill,
        } => {
            put_u8(h, 167);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, *draw_count);
            put_u32(h, *damage_amount);
            put_u32(h, mark_def.raw());
            put_u32(h, *mark_count);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseTargetDrawThenOwnerIfBelowMaxHp {
            player,
            target,
            count,
            skill,
            ready_to_check,
        } => {
            put_u8(h, 172);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_check);
        }

        EffectRequest::ChooseBoundSourceAttackRangeDamage {
            player,
            source,
            amount,
            skill,
        } => {
            put_u8(h, 173);
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseTargetDrawMaxHpCapThenDiscardExcess {
            player,
            target,
            cap,
            skill,
            ready_to_discard,
        } => {
            put_u8(h, 174);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *cap);
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_discard);
        }

        EffectRequest::ChooseTargetDrawMaxHpCapThenAddSkill {
            player,
            target,
            cap,
            skill,
            add_def,
        } => {
            put_u8(h, 0);
            h.update(&414u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *cap);
            put_u32(h, skill.raw());
            put_u32(h, add_def.raw());
        }

        EffectRequest::ChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp {
            player,
            target,
            skill,
            ready_to_check,
        } => {
            put_u8(h, 178);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_check);
        }

        EffectRequest::ChooseGiveOneHandEquipOrRecover {
            owner,
            giver,
            target,
            skill,
        } => {
            put_u8(h, 179);
            put_u32(h, owner.raw());
            put_u32(h, giver.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp {
            player,
            skill,
            zone,
            discarded_count,
            ready_to_draw,
        } => {
            put_u8(h, 181);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u8(h, *zone);
            put_u32(h, *discarded_count);
            put_bool(h, *ready_to_draw);
        }

        EffectRequest::ChooseDiscardHandEquipThenRecover {
            player,
            count,
            amount,
            skill,
        } => {
            put_u8(h, 182);
            put_u32(h, player.raw());
            put_u32(h, *count);
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseTargetsDamage {
            player,
            min_count,
            max_count,
            scope,
            amount,
            skill,
        } => {
            put_u8(h, 183);
            put_u32(h, player.raw());
            put_u8(h, *min_count);
            put_u8(h, *max_count);
            put_u8(h, scope.tag());
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseTargetRevealHandDamageIfSha {
            player,
            amount,
            skill,
        } => {
            put_u8(h, 185);
            put_u32(h, player.raw());
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseOtherGiveOneHandEquip { player, skill } => {
            put_u8(h, 186);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseGiveOrRecastOneHand {
            player,
            skill,
            require_skill_def,
            grant_skill_def,
            grant_mark_def,
        } => {
            put_u8(h, 187);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, *require_skill_def);
            put_u32(h, *grant_skill_def);
            put_u32(h, *grant_mark_def);
        }

        EffectRequest::ChooseRevealHandCardBindCategoryTarget {
            player,
            target,
            skill,
        } => {
            put_u8(h, 187);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseMoveTargetEquipmentAndOrDraw {
            player,
            target,
            skill,
        } => {
            put_u8(h, 190);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth {
            player,
            target,
            damage_id,
            skill,
        } => {
            put_u8(h, 191);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *damage_id);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseDrawOrRecastAddSkill {
            player,
            draw_skill_def,
            recast_skill_def,
            skill,
        } => {
            put_u8(h, 192);
            put_u32(h, player.raw());
            put_u32(h, draw_skill_def.raw());
            put_u32(h, recast_skill_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseGainShaForOpeningExtraPlay { player, skill } => {
            put_u8(h, 193);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseDrawOrDiscardTargetCardRecordSha {
            player,
            card,
            target,
            skill,
        } => {
            put_u8(h, 198);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseOptionalDiscardTargetCard {
            player,
            target,
            scope,
        } => {
            put_u8(h, 200);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, scope.tag());
        }

        EffectRequest::ChooseDiscardTargetCardsRecoverSameCategory {
            player,
            target,
            count,
            amount,
            skill,
        } => {
            put_u8(h, 202);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *count);
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseScskuijiDiscard {
            player,
            target,
            count,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&360u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChoosePskuijiDiscard {
            player,
            target,
            count,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&361u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseCompareSelectedHandCardSecondFailureSeatDamage {
            player,
            target,
            card,
            draw_count,
            failure_count,
            lose_hp,
            damage,
            skill,
        } => {
            put_u8(h, 203);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, *draw_count);
            put_u32(h, *failure_count);
            put_u32(h, *lose_hp);
            put_u32(h, *damage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseDrawOrDiscardTargetCard {
            player,
            target,
            draw_count,
            skill,
        } => {
            put_u8(h, 205);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *draw_count);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseSeatPathDamage {
            player,
            first,
            second,
            amount,
            skill,
        } => {
            put_u8(h, 207);
            put_u32(h, player.raw());
            put_u32(h, first.raw());
            put_u32(h, second.raw());
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseOtherRecover { player, amount } => {
            put_u8(h, 210);
            put_u32(h, player.raw());
            put_u32(h, *amount);
        }

        EffectRequest::ChooseOtherHandTargetsGainOneReplaceDraw {
            player,
            targets,
            index,
            selected,
            skill,
        } => {
            put_u8(h, 215);
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, *index);
            put_u32(h, selected.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseDiscardBasicThenTargetDiscardEquipOrDamage {
            player,
            target,
            skill,
        } => {
            put_u8(h, 218);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseCurrentShaTargetDiscardRedirectToOwner {
            owner,
            source,
            card,
            target,
            skill,
            ready_to_redirect,
        } => {
            put_u8(h, 219);
            put_u32(h, owner.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_redirect);
        }

        EffectRequest::ChooseTargetGainBottomCard {
            player,
            card,
            target,
            skill,
        } => {
            put_u8(h, 220);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseDiscardRedirectCurrentSha {
            player,
            source,
            card,
            target,
            skill,
        } => {
            put_u8(h, 223);
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseDiscardOneFromEachLinked { player, skill } => {
            put_u8(h, 225);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseTargetLoseHpDiscardHandShaIfBasic {
            player,
            target,
            skill,
            stage,
            discarded_basic,
        } => {
            put_u8(h, 228);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
            put_u8(h, *stage);
            put_u8(h, u8::from(*discarded_basic));
        }

        EffectRequest::ChooseOptionalTargetsDraw {
            player,
            count,
            min_count,
            max_count,
            scope,
            skill,
        } => {
            put_u8(h, 229);
            put_u32(h, player.raw());
            put_u32(h, *count);
            put_u8(h, *min_count);
            put_u8(h, *max_count);
            put_u8(h, scope.tag());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseGainOwnLostDiscardCardThisTurn { player, skill } => {
            put_u8(h, 235);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseMoveFieldCardTo {
            player,
            target,
            skill,
        } => {
            put_u8(h, 236);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan {
            player,
            mark,
            skill,
        } => {
            put_u8(h, 239);
            put_u32(h, player.raw());
            put_u32(h, mark.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver {
            player,
            priority,
            remaining_targets,
            skill,
        } => {
            put_u8(h, 240);
            put_u32(h, player.raw());
            put_u8(h, *priority);
            put_ids(h, remaining_targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseGainCardFromEachOtherThenTurnOver {
            player,
            initialized,
            remaining_targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&271u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, u8::from(*initialized));
            put_ids(h, remaining_targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseAllLoseHpOrRecover { player, skill } => {
            put_u8(h, 249);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseLinkOther { player, skill } => {
            put_u8(h, 253);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseDiscardWeaponOrLoseHp { player, skill } => {
            put_u8(h, 0);
            h.update(&261u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseDiscardOneHeOrLoseHp {
            player,
            lose_amount,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&601u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *lose_amount);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseSourceConfirmOwnerLoseHp {
            owner,
            source,
            amount,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&603u16.to_le_bytes());
            put_u32(h, owner.raw());
            put_u32(h, source.raw());
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseGainTargetCardAlternateZoneThenTargetDraw {
            player,
            target,
            selected,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&264u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, selected.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseDiscardDamageEndpointCardIncreaseIfSuit {
            player,
            target,
            damage_id,
            suit,
            amount,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&266u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *damage_id);
            put_u8(h, *suit);
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseOtherHandStoreSkillPile {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&567u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseEndExtraDrawOrPlayByUsedSuits { player, skill } => {
            put_u8(h, 0);
            h.update(&268u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        _ => unreachable!("effect routed to a different hash group"),
    }
}
