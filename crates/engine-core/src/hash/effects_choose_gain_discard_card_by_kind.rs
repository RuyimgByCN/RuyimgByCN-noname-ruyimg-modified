//! 效果请求的规范哈希编码：通用效果原语。

use super::*;

pub(super) fn hash_effect(h: &mut blake3::Hasher, effect: &EffectRequest) {
    match effect {
        EffectRequest::ChooseGainDiscardCardByKind {
            player,
            kind,
            skill,
        } => {
            put_u8(h, 132);
            put_u32(h, player.raw());
            put_u8(h, *kind);
            put_u32(h, skill.raw());
        }

        EffectRequest::DrawForLifetimeCardActionDivisors { player, divisors } => {
            put_u8(h, 133);
            put_u32(h, player.raw());
            for divisor in divisors {
                put_u32(h, *divisor);
            }
        }

        EffectRequest::ChooseAddMissingSkill {
            player,
            skill_defs,
            skill,
        } => {
            put_u8(h, 134);
            put_u32(h, player.raw());
            for skill_def in skill_defs {
                put_u32(h, skill_def.raw());
            }
            put_u32(h, skill.raw());
        }

        EffectRequest::AddFirstMissingSkill {
            player,
            skill_defs,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&530u16.to_le_bytes());
            put_u32(h, player.raw());
            for skill_def in skill_defs {
                put_u32(h, skill_def.raw());
            }
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseTargetsDraw {
            player,
            count,
            target_count,
            scope,
            skill,
        } => {
            put_u8(h, 135);
            put_u32(h, player.raw());
            put_u32(h, *count);
            put_u8(h, *target_count);
            put_u8(h, scope.tag());
            put_u32(h, skill.raw());
        }

        EffectRequest::GainEventCards { player, cards } => {
            put_u8(h, 136);
            put_u32(h, player.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
        }

        EffectRequest::ChooseTargetDrawToMaxHpCap { player, cap, skill } => {
            put_u8(h, 137);
            put_u32(h, player.raw());
            put_u32(h, *cap);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseOtherDrawOwnerLostHpTurnOver { player, skill } => {
            put_u8(h, 138);
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::EmitJudgmentResolved {
            player,
            result_card,
            suit,
            rank,
            cause_card,
        } => {
            put_u8(h, 139);
            put_u32(h, player.raw());
            put_u32(h, result_card.raw());
            put_u8(h, suit_tag(*suit));
            put_u8(h, *rank);
            put_u32(h, cause_card.raw());
        }

        EffectRequest::JudgeRecoverOtherOnSuit {
            source,
            judge_player,
            recover_player,
            suit,
            amount,
        } => {
            put_u8(h, 140);
            put_u32(h, source.raw());
            put_u32(h, judge_player.raw());
            put_u32(h, recover_player.raw());
            put_u8(h, suit_tag(*suit));
            put_u32(h, *amount);
        }

        EffectRequest::ForceOthersUseShaOnNearestOrLoseHp {
            player,
            remaining,
            started,
            skill,
        } => {
            put_u8(h, 141);
            put_u32(h, player.raw());
            put_ids(h, remaining.iter().map(|player| player.raw()));
            put_bool(h, *started);
            put_u32(h, skill.raw());
        }

        EffectRequest::ContinueDyingSave {
            player,
            source,
            responders,
        } => {
            put_u8(h, 142);
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_ids(h, responders.iter().map(|player| player.raw()));
        }

        EffectRequest::RestoreCharacterState { player } => {
            put_u8(h, 143);
            put_u32(h, player.raw());
        }

        EffectRequest::DiscardAllHandEquipJudge { player } => {
            put_u8(h, 144);
            put_u32(h, player.raw());
        }

        EffectRequest::ChooseDiscardTargetCardResolveCurrentSha {
            source,
            card,
            target,
        } => {
            put_u8(h, 145);
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
        }

        EffectRequest::UseVirtualCardOnSelectedTargets {
            player,
            kind,
            targets,
        } => {
            put_u8(h, 146);
            put_u32(h, player.raw());
            put_u8(h, *kind);
            put_ids(h, targets.iter().map(|target| target.raw()));
        }

        EffectRequest::DealDamageBanActiveIfDying {
            source,
            target,
            amount,
            skill,
        } => {
            put_u8(h, 147);
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }

        EffectRequest::SpjungongDiscardOrLoseThenSha {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&363u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::BanActiveUntilTurnEnd { player, skill } => {
            put_u8(h, 0);
            h.update(&364u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::SpwanweiActive { player, target } => {
            put_u8(h, 0);
            h.update(&365u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
        }

        EffectRequest::SpwanweiDying { player, target } => {
            put_u8(h, 0);
            h.update(&366u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
        }

        EffectRequest::SuppressNonLockedSkillsUntilTurnEnd { player } => {
            put_u8(h, 0);
            h.update(&367u16.to_le_bytes());
            put_u32(h, player.raw());
        }

        EffectRequest::ChooseRecoverTarget {
            chooser,
            target,
            amount,
        } => {
            put_u8(h, 0);
            h.update(&368u16.to_le_bytes());
            put_u32(h, chooser.raw());
            put_u32(h, target.raw());
            put_u32(h, *amount);
        }

        EffectRequest::GiveAllHandThenChooseUseOneOrDamage {
            player,
            target,
            cards,
            targets,
            stage,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&369u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u8(h, *stage);
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable {
            player,
            card,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&370u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseLoseHpUseVirtualShaByBasicVariants {
            player,
            card,
            targets,
            stage,
            damage_bonus,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&371u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u8(h, *stage);
            put_u32(h, *damage_bonus);
            put_u32(h, skill.raw());
        }

        EffectRequest::ExcludeSavedShaTarget { card, target } => {
            put_u8(h, 0);
            h.update(&517u16.to_le_bytes());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
        }

        EffectRequest::BanUseAgainst { user, target } => {
            put_u8(h, 0);
            h.update(&524u16.to_le_bytes());
            put_u32(h, user.raw());
            put_u32(h, target.raw());
        }

        EffectRequest::RecordCurrentShaAllTargets {
            player,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&538u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseGiveOrDiscardHand {
            player,
            payment_count,
            scope,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&570u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *payment_count);
            put_u8(h, scope.tag());
            put_u32(h, skill.raw());
        }
        _ => unreachable!("effect routed to a different hash group"),
    }
}
