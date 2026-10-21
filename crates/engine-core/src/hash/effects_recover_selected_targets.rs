//! Canonical encoding for `RecoverSelectedTargets` and related requests.

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

        EffectRequest::RevealHandToPlayer { player, observer } => {
            put_u8(h, 199);
            put_u32(h, player.raw());
            put_u32(h, observer.raw());
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

        EffectRequest::RevealHandChooseDiscardColor { player } => {
            put_u8(h, 201);
            put_u32(h, player.raw());
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

        EffectRequest::ChooseOtherRecover { player, amount } => {
            put_u8(h, 210);
            put_u32(h, player.raw());
            put_u32(h, *amount);
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

        EffectRequest::JudgeSetSkillMarkToRank { player, skill } => {
            put_u8(h, 224);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseDiscardOneFromEachLinked { player, skill } => {
            put_u8(h, 225);
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

        EffectRequest::AddSkillUntilTurnEnd { player, skill_def } => {
            put_u8(h, 241);
            put_u32(h, player.raw());
            put_u32(h, skill_def.raw());
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
        } => {
            put_u8(h, 242);
            put_u32(h, player.raw());
            put_u32(h, mark.raw());
            put_u32(h, *mark_count);
            put_u32(h, *damage_amount);
            put_u32(h, *hand_count);
            put_u8(h, *stage);
            put_u32(h, *index);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

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
        } => {
            put_u8(h, 0);
            h.update(&270u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, mark.raw());
            put_u32(h, *mark_count);
            put_u32(h, *damage_amount);
            put_u32(h, *discard_count);
            put_u8(h, *stage);
            put_u32(h, *index);
            put_ids(h, targets.iter().map(|target| target.raw()));
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
        } => {
            put_u8(h, 244);
            put_u32(h, player.raw());
            put_u32(h, mark.raw());
            put_u32(h, *mass_threshold);
            put_u32(h, *damage_amount);
            put_u8(h, *stage);
            put_u32(h, *index);
            put_u32(h, target.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSelectedEquipmentDiscardThenChooseDamage {
            player,
            damage_amount,
            nature,
            stage,
            index,
            targets,
            skill,
        } => {
            put_u8(h, 245);
            put_u32(h, player.raw());
            put_u32(h, *damage_amount);
            put_u8(h, *nature);
            put_u8(h, *stage);
            put_u32(h, *index);
            put_ids(h, targets.iter().map(|target| target.raw()));
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

        EffectRequest::ResolveYeyan {
            player,
            mode,
            stage,
            index,
            targets,
            data,
            skill,
        } => {
            put_u8(h, 248);
            put_u32(h, player.raw());
            put_u8(h, *mode);
            put_u8(h, *stage);
            put_u32(h, *index);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, *data);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseAllLoseHpOrRecover { player, skill } => {
            put_u8(h, 249);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveQixing {
            player,
            initialize,
            stage,
            hand_cards,
            pile_cards,
            skill,
        } => {
            put_u8(h, 250);
            put_u32(h, player.raw());
            put_u8(h, u8::from(*initialize));
            put_u8(h, *stage);
            put_ids(h, hand_cards.iter().map(|card| card.raw()));
            put_ids(h, pile_cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveQixingWeather {
            player,
            qixing_skill_def,
            helper_skill_def,
            max_targets,
            stage,
            cards,
            targets,
            skill,
        } => {
            put_u8(h, 251);
            put_u32(h, player.raw());
            put_u32(h, qixing_skill_def.raw());
            put_u32(h, helper_skill_def.raw());
            put_u8(h, *max_targets);
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveNzryLongnu {
            player,
            fire_skill_def,
            thunder_skill_def,
            stage,
            skill,
        } => {
            put_u8(h, 252);
            put_u32(h, player.raw());
            put_u32(h, fire_skill_def.raw());
            put_u32(h, thunder_skill_def.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseLinkOther { player, skill } => {
            put_u8(h, 253);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDrltPoxi {
            player,
            target,
            hand_limit_skill_def,
            stage,
            own_count,
            skill,
        } => {
            put_u8(h, 254);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, hand_limit_skill_def.raw());
            put_u8(h, *stage);
            put_u8(h, *own_count);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDrltJieying {
            player,
            event_player,
            mark,
            mode,
            skill,
        } => {
            put_u8(h, 255);
            put_u32(h, player.raw());
            put_u32(h, event_player.raw());
            put_u32(h, mark.raw());
            put_u8(h, *mode);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDrltDuorui {
            player,
            target,
            helper_skill_def,
            stage,
            selected_skill_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&256u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, helper_skill_def.raw());
            put_u8(h, *stage);
            put_u32(h, selected_skill_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDrltZhiti {
            player,
            event_player,
            event_source,
            event_amount,
            mode,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&257u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, event_player.raw());
            put_u32(h, event_source.raw());
            put_u32(h, *event_amount);
            put_u8(h, *mode);
            put_u32(h, skill.raw());
        }

        EffectRequest::AdvanceTurn => {
            put_u8(h, 0);
            h.update(&258u16.to_le_bytes());
        }

        EffectRequest::ResolveShenSimayi {
            player,
            event_player,
            event_amount,
            mark,
            mode,
            cards,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&259u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, event_player.raw());
            put_u32(h, *event_amount);
            put_u32(h, mark.raw());
            put_u8(h, *mode);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

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
        } => {
            put_u8(h, 0);
            h.update(&260u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, source.raw());
            put_u8(h, *stage);
            put_u32(h, result_card.raw());
            put_u8(h, crate::command::judge::suit_tag(*suit) as u8);
            put_u8(h, *rank);
            put_ids(h, rewriters.iter().map(|player| player.raw()));
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

        EffectRequest::TakeOneHandCard { from, to } => {
            put_u8(h, 0);
            h.update(&267u16.to_le_bytes());
            put_u32(h, from.raw());
            put_u32(h, to.raw());
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

        EffectRequest::ChooseEndExtraDrawOrPlayByUsedSuits { player, skill } => {
            put_u8(h, 0);
            h.update(&268u16.to_le_bytes());
            put_u32(h, player.raw());
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
        } => {
            put_u8(h, 0);
            h.update(&272u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, pile_skill_def.raw());
            put_u8(h, *action);
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_ids(h, hand_cards.iter().map(|card| card.raw()));
            put_ids(h, pile_cards.iter().map(|card| card.raw()));
            put_ids(h, event_cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
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
