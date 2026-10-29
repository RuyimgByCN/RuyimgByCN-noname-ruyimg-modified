//! 效果请求的规范哈希编码：通用效果原语。

use super::*;

pub(super) fn hash_effect(h: &mut blake3::Hasher, effect: &EffectRequest) {
    match effect {
        EffectRequest::RecoverSelectedTargets {
            source,
            targets,
            amount,
            skill,
        } => {
            put_u8(h, 163);
            put_u32(h, source.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }

        EffectRequest::RecordSelectedPlayPhaseCategory { card, skill } => {
            put_u8(h, 164);
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::TakeTargetHandThenRepeatUntilMax {
            player,
            target,
            skill,
        } => {
            put_u8(h, 165);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ContinueTakeLargerHandUntilMax { player, skill } => {
            put_u8(h, 166);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::DrawThenChooseGiveOrDiscardHand {
            player,
            draw_count,
            payment_count,
            skill,
            ready_to_choose,
        } => {
            put_u8(h, 168);
            put_u32(h, player.raw());
            put_u32(h, *draw_count);
            put_u32(h, *payment_count);
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_choose);
        }

        EffectRequest::GainTargetEquipment {
            player,
            target,
            slot,
        } => {
            put_u8(h, 169);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *slot);
        }

        EffectRequest::DiscardAllTargetEquipment { actor, target } => {
            put_u8(h, 170);
            put_u32(h, actor.raw());
            put_u32(h, target.raw());
        }

        EffectRequest::IncreaseCurrentShaTargetDamage {
            source,
            card,
            target,
            amount,
        } => {
            put_u8(h, 171);
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u32(h, *amount);
        }

        EffectRequest::GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp {
            player,
            target,
            card,
            suit,
            skill,
            ready_to_choose,
        } => {
            put_u8(h, 175);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, suit_tag(*suit));
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_choose);
        }

        EffectRequest::GainEquipmentFromDrawPile { player } => {
            put_u8(h, 176);
            put_u32(h, player.raw());
        }

        EffectRequest::GiveAllHandRecoverToOne {
            player,
            target,
            ready_to_recover,
        } => {
            put_u8(h, 177);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_bool(h, *ready_to_recover);
        }

        EffectRequest::ReplaceSkillWithBoundAfterResolutionIfTargetAlive {
            player,
            skill,
            skill_def,
            bound_target,
            ready_to_replace,
        } => {
            put_u8(h, 180);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, skill_def.raw());
            put_u32(h, bound_target.raw());
            put_bool(h, *ready_to_replace);
        }

        EffectRequest::DiscardTargetEquipmentDrawWeaponRange {
            player,
            target,
            skill,
        } => {
            put_u8(h, 184);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::RecoverToMaxThenOptionalExchangeSkill {
            player,
            skill,
            removable_skill_def,
            restore_skill_def,
            ready_to_choose,
        } => {
            put_u8(h, 188);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, removable_skill_def.raw());
            put_u32(h, restore_skill_def.raw());
            put_bool(h, *ready_to_choose);
        }

        EffectRequest::RevealHandDiscardKind { player, kind } => {
            put_u8(h, 189);
            put_u32(h, player.raw());
            put_u8(h, *kind);
        }

        EffectRequest::FinishOpeningExtraPlayShaGain { player, skill } => {
            put_u8(h, 194);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::RevealTargetHandGainShanArmorOrDrawTwo {
            player,
            target,
            skill,
        } => {
            put_u8(h, 195);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::GiveAllHand { player, target } => {
            put_u8(h, 196);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
        }

        EffectRequest::GainCardIfDiscard { player, card } => {
            put_u8(h, 197);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
        }

        EffectRequest::RevealHandToPlayer { player, observer } => {
            put_u8(h, 199);
            put_u32(h, player.raw());
            put_u32(h, observer.raw());
        }

        EffectRequest::RevealHandChooseDiscardColor { player } => {
            put_u8(h, 201);
            put_u32(h, player.raw());
        }

        EffectRequest::ContinueChooseCompareTargetHandCardSecondFailureSeatDamage {
            player,
            target,
            card,
            draw_count,
            failure_count,
            lose_hp,
            damage,
            skill,
        } => {
            put_u8(h, 204);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, *draw_count);
            put_u32(h, *failure_count);
            put_u32(h, *lose_hp);
            put_u32(h, *damage);
            put_u32(h, skill.raw());
        }

        EffectRequest::RecordCompareFailureSeatDamage {
            player,
            target,
            failure_count,
            lose_hp,
            damage,
            skill,
        } => {
            put_u8(h, 206);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *failure_count);
            put_u32(h, *lose_hp);
            put_u32(h, *damage);
            put_u32(h, skill.raw());
        }

        EffectRequest::DealDamageEachInOrder {
            source,
            targets,
            amount,
            skill,
        } => {
            put_u8(h, 208);
            put_u32(h, source.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }

        EffectRequest::RemoveAllSkills { player } => {
            put_u8(h, 209);
            put_u32(h, player.raw());
        }

        EffectRequest::EnableEquipSlot {
            player,
            slot,
            source,
        } => {
            put_u8(h, 211);
            put_u32(h, player.raw());
            put_u8(h, *slot);
            put_u32(h, source.raw());
        }

        EffectRequest::TargetChooseSuitGainOwnerHandDamageIfMismatch {
            player,
            target,
            skill,
        } => {
            put_u8(h, 212);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::JudgeSourceDiscardTwoHandOrDamage { player, source } => {
            put_u8(h, 213);
            put_u32(h, player.raw());
            put_u32(h, source.raw());
        }

        EffectRequest::GiveSelectedRecoverAfterPhaseTotalTwo {
            player,
            target,
            cards,
            skill,
            ready_to_recover,
        } => {
            put_u8(h, 214);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
            put_u8(h, u8::from(*ready_to_recover));
        }

        EffectRequest::DistributeTopTwoCards {
            player,
            cards,
            selected,
            target,
            skill,
            ready_to_choose,
        } => {
            put_u8(h, 216);
            put_u32(h, player.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_ids(h, selected.iter().map(|card| card.raw()));
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_choose);
        }

        EffectRequest::JudgeMakeCurrentShaTargetUnrespondableOnRed {
            player,
            card,
            target,
            retieji,
            skill,
            ready_to_choose,
            judgment_suit,
        } => {
            put_u8(h, 217);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            if *retieji {
                put_bool(h, true);
                put_u32(h, skill.raw());
                put_bool(h, *ready_to_choose);
                put_u8(h, *judgment_suit);
            }
        }

        EffectRequest::JudgeBlackGainRepeat {
            player,
            card,
            skill,
            ready_to_choose,
        } => {
            put_u8(h, 221);
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_choose);
        }

        EffectRequest::UseVirtualJuedouSecondOnFirst { targets, card } => {
            put_u8(h, 222);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, card.raw());
        }

        EffectRequest::JudgeSetSkillMarkToRank { player, skill } => {
            put_u8(h, 224);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::DisableSkillByDefUntilTurnEnd { player, skill_def } => {
            put_u8(h, 226);
            put_u32(h, player.raw());
            put_u32(h, skill_def.raw());
        }

        EffectRequest::TargetDiscardThenDuelIfNonShaAndHpGteOwner {
            player,
            target,
            skill,
        } => {
            put_u8(h, 227);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::RevealTargetHandShaOrDiscardBind {
            player,
            target,
            helper_skill_def,
            skill,
            stage,
        } => {
            put_u8(h, 230);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, helper_skill_def.raw());
            put_u32(h, skill.raw());
            put_u8(h, *stage);
        }

        EffectRequest::DrawRememberedHandCards {
            player,
            count,
            skill,
        } => {
            put_u8(h, 231);
            put_u32(h, player.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
        }

        EffectRequest::DiscardRememberedHandCards { player, skill } => {
            put_u8(h, 232);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::GainDrawCardsForLifetimeCardActionDivisors {
            player,
            divisors,
            groups,
        } => {
            put_u8(h, 233);
            put_u32(h, player.raw());
            for divisor in divisors {
                put_u32(h, *divisor);
            }
            for group in groups {
                put_u8(h, *group);
            }
        }

        EffectRequest::LoseHpGainDiscardCardForOpeningExtraPlay {
            player,
            kind,
            mandatory_skill_def,
            skill,
        } => {
            put_u8(h, 234);
            put_u32(h, player.raw());
            put_u8(h, *kind);
            put_u32(h, mandatory_skill_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::DrawThenOptionallyDistributeHandCards {
            player,
            draw_count,
            give_cap,
            cards,
            targets,
            skill,
            ready_to_choose,
            finish,
        } => {
            put_u8(h, 237);
            put_u32(h, player.raw());
            put_u32(h, *draw_count);
            put_u32(h, *give_cap);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_choose);
            put_bool(h, *finish);
        }

        EffectRequest::GiveSelectedThenContinueDistinctRecoverAtTwo {
            player,
            target,
            cards,
            used_targets,
            total_given,
            skill,
            continue_only,
        } => {
            put_u8(h, 238);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_ids(h, used_targets.iter().map(|target| target.raw()));
            put_u32(h, *total_given);
            put_u32(h, skill.raw());
            put_bool(h, *continue_only);
        }

        EffectRequest::AddSkillUntilTurnEnd { player, skill_def } => {
            put_u8(h, 241);
            put_u32(h, player.raw());
            put_u32(h, skill_def.raw());
        }

        EffectRequest::RecordActiveTarget {
            player,
            target,
            skill,
        } => {
            put_u8(h, 243);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ReplaceDrawWithDistinctSuitSelection {
            player,
            count,
            stage,
            cards,
            skill,
        } => {
            put_u8(h, 246);
            put_u32(h, player.raw());
            put_u8(h, *count);
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::InspectTargetHandChooseHeartDiscardOrTop {
            player,
            target,
            stage,
            card,
            skill,
        } => {
            put_u8(h, 247);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::AdvanceTurn => {
            put_u8(h, 0);
            h.update(&258u16.to_le_bytes());
        }

        EffectRequest::JudgeSpadeTwoToNineDrawTwoOrThunderShaTwice {
            source,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&602u16.to_le_bytes());
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::DamageThenDiscardAllJudge {
            player,
            ready_to_discard,
        } => {
            put_u8(h, 0);
            h.update(&262u16.to_le_bytes());
            put_u32(h, player.raw());
            put_bool(h, *ready_to_discard);
        }

        EffectRequest::ReplaceDrawGainTopTwoAddSkillsIfDifferentColors {
            player,
            first_skill_def,
            second_skill_def,
            stage,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&263u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, first_skill_def.raw());
            put_u32(h, second_skill_def.raw());
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::SkipNextJudgePhase { player } => {
            put_u8(h, 0);
            h.update(&265u16.to_le_bytes());
            put_u32(h, player.raw());
        }

        EffectRequest::SkipJudgeAndDrawPhases { player, skill } => {
            put_u8(h, 0);
            h.update(&433u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::TakeOneHandCard { from, to } => {
            put_u8(h, 0);
            h.update(&267u16.to_le_bytes());
            put_u32(h, from.raw());
            put_u32(h, to.raw());
        }

        EffectRequest::GainAllSkillPileCards { player, skill } => {
            put_u8(h, 0);
            h.update(&569u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::GainOtherSkillPileCards {
            player,
            target,
            skill_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&600u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *skill_def);
            put_u32(h, skill.raw());
        }

        EffectRequest::LoseHpDoubleCurrentDamageThenReplace {
            player,
            damage_id,
            amount,
            skill,
            replacement_skill_def,
            ready_to_replace,
        } => {
            put_u8(h, 0);
            h.update(&269u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *damage_id);
            put_u32(h, *amount);
            put_u32(h, skill.raw());
            put_u32(h, replacement_skill_def.raw());
            put_u8(h, u8::from(*ready_to_replace));
        }

        EffectRequest::SetNonLockedSkillSuppression {
            player,
            enabled,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&273u16.to_le_bytes());
            put_u32(h, player.raw());
            put_bool(h, *enabled);
            put_u32(h, skill.raw());
        }
        _ => unreachable!("effect routed to a different hash group"),
    }
}
