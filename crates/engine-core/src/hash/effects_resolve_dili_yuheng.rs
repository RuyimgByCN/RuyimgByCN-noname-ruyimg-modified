//! 效果请求的规范哈希编码：通用效果原语。

use super::*;

pub(super) fn hash_effect(h: &mut blake3::Hasher, effect: &EffectRequest) {
    match effect {
        EffectRequest::FinalizeTiaojiyanmei {
            actor,
            left,
            right,
            card,
        } => {
            put_u8(h, 0);
            h.update(&275u16.to_le_bytes());
            put_u32(h, actor.raw());
            put_u32(h, left.raw());
            put_u32(h, right.raw());
            put_u32(h, card.raw());
        }

        EffectRequest::BeginTiaojiyanmeiDiscard {
            actor,
            left,
            right,
            higher,
        } => {
            put_u8(h, 0);
            h.update(&276u16.to_le_bytes());
            put_u32(h, actor.raw());
            put_u32(h, left.raw());
            put_u32(h, right.raw());
            put_u32(h, higher.raw());
        }

        EffectRequest::DiscardSelectedDrawAllHandBonus {
            player,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&277u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseOtherYinghun {
            player,
            target,
            count,
            branch,
            ready_to_discard,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&278u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *count);
            put_u8(h, *branch);
            put_bool(h, *ready_to_discard);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseOtherBizheng {
            player,
            target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&279u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseOtherXiashu {
            player,
            target,
            stage,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&280u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::DiscardEquipmentSlot { player, slot } => {
            h.update(&297u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *slot);
        }

        EffectRequest::StoreSelectedInSkillPile {
            player,
            cards,
            skill,
        } => {
            h.update(&299u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::EndMatchForCampIfSkillPileAtLeast {
            player,
            pile_skill_def,
            count,
        } => {
            h.update(&300u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, pile_skill_def.raw());
            put_u32(h, *count);
        }

        EffectRequest::ChooseAddCurrentShaTargetWithinDistance {
            player,
            card,
            max_distance,
            skill,
        } => {
            h.update(&310u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, *max_distance);
            put_u32(h, skill.raw());
        }

        EffectRequest::RefundCurrentShaUse {
            player,
            card,
            skill,
        } => {
            h.update(&311u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResetLinkedSelectedTargets { source, targets } => {
            h.update(&374u16.to_le_bytes());
            put_u32(h, source.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
        }

        EffectRequest::DamageThenDrawSelectedTargets {
            source,
            targets,
            damage,
            draw,
            skill,
        } => {
            h.update(&312u16.to_le_bytes());
            put_u32(h, source.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, *damage);
            put_u32(h, *draw);
            put_u32(h, skill.raw());
        }

        EffectRequest::FinishCurrentPhaseAbortResolution { player, stage } => {
            h.update(&318u16.to_le_bytes());
            put_u32(h, player.raw());
            h.update(&[*stage]);
        }

        EffectRequest::GainTopCardToSkillPile { player, skill } => {
            h.update(&319u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseDiscardSkillPileCardAddMarkedSkill {
            player,
            target,
            helper_skill_def,
            selected,
            skill,
        } => {
            h.update(&320u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, helper_skill_def.raw());
            put_u32(h, selected.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::GainOneHandFromSelectedTargets {
            player,
            targets,
            index,
            selected,
            skill,
        } => {
            h.update(&321u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, *index);
            put_u32(h, selected.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseRepoluTargetsDraw { player, skill } => {
            h.update(&322u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRedRestoreIfEqualRank {
            player,
            sha_user,
            card,
            restore_skill_def,
            skill,
        } => {
            h.update(&324u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, sha_user.raw());
            put_u32(h, card.raw());
            put_u32(h, restore_skill_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseGainCardFromEachOtherReplaceDraw {
            player,
            initialized,
            remaining_targets,
            skill,
        } => {
            h.update(&329u16.to_le_bytes());
            put_u32(h, player.raw());
            put_bool(h, *initialized);
            put_ids(h, remaining_targets.iter().map(|player| player.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::GiveSelectedThenRecoverTarget {
            player,
            target,
            cards,
            amount,
            skill,
            ready_to_recover,
        } => {
            h.update(&330u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, *amount);
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_recover);
        }

        EffectRequest::JudgeSameColorGainRepeat {
            player,
            card,
            skill,
            ready_to_choose,
            color,
        } => {
            h.update(&333u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_choose);
            put_u8(h, *color);
        }

        EffectRequest::ChooseCurrentTrickTargetsToNullify {
            player,
            source,
            card,
            skill,
        } => {
            h.update(&334u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::JudgeBlackGainRepeatProtectHandLimit {
            player,
            card,
            skill,
            ready_to_choose,
        } => {
            h.update(&335u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
            put_bool(h, *ready_to_choose);
        }

        EffectRequest::ChooseGainAllResolvedShaShanThenColorRewards {
            player,
            sha_card,
            shan_cards,
            excluded_recipients,
            skill,
        } => {
            h.update(&336u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, sha_card.raw());
            put_ids(h, shan_cards.iter().map(|card| card.raw()));
            put_ids(h, excluded_recipients.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseOtherHandTargetsGainOneReduceDraw {
            player,
            targets,
            index,
            selected,
            draw_count,
            skill,
        } => {
            h.update(&337u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, *index);
            put_u32(h, selected.raw());
            put_u32(h, *draw_count);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseDiscardTargetCardResolveCurrentShaNonbasicBonus {
            source,
            card,
            target,
            bonus_skill_def,
        } => {
            h.update(&338u16.to_le_bytes());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u32(h, bonus_skill_def.raw());
        }

        EffectRequest::ChooseTargetDiscardDistanceHandOrIncreaseDamage {
            player,
            target,
            damage_id,
            skill,
        } => {
            h.update(&345u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *damage_id);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseDrawViewTwoGainOrMoveEquipment {
            player,
            target,
            skill,
        } => {
            h.update(&346u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseGiveTwoDifferentColorsOrRecover {
            owner,
            giver,
            target,
            skill,
        } => {
            h.update(&352u16.to_le_bytes());
            put_u32(h, owner.raw());
            put_u32(h, giver.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::TakeOneScopedCard { from, to, scope } => {
            h.update(&356u16.to_le_bytes());
            put_u32(h, from.raw());
            put_u32(h, to.raw());
            put_u8(h, scope.tag());
        }

        EffectRequest::RevealHand { player } => {
            h.update(&357u16.to_le_bytes());
            put_u32(h, player.raw());
        }

        EffectRequest::DrawLinkedPlayers { count } => {
            h.update(&358u16.to_le_bytes());
            put_u32(h, *count);
        }

        EffectRequest::RevealTopGainIfColor { player, color_mask } => {
            put_u8(h, 0);
            h.update(&362u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *color_mask);
        }

        EffectRequest::RefreshSkill { skill } => {
            put_u8(h, 0);
            h.update(&994u16.to_le_bytes());
            put_u32(h, skill.raw());
        }

        EffectRequest::TagCard { cards, tag } => {
            put_u8(h, 0);
            h.update(&995u16.to_le_bytes());
            put_ids(h, cards.iter().map(|id| id.raw()));
            put_u32(h, *tag);
        }

        EffectRequest::UntagCard { cards, tag } => {
            put_u8(h, 0);
            h.update(&996u16.to_le_bytes());
            put_ids(h, cards.iter().map(|id| id.raw()));
            put_u32(h, *tag);
        }

        EffectRequest::ClanshenjunReveal {
            player,
            event_card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&997u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, event_card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ClanshenjunUse { player, skill } => {
            put_u8(h, 0);
            h.update(&998u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::StorageSet { skill, key, value } => {
            put_u8(h, 0);
            h.update(&999u16.to_le_bytes());
            put_u32(h, skill.raw());
            put_u32(h, *key);
            put_u32(h, *value);
        }

        EffectRequest::DrawAllEventTargets { players, count } => {
            put_u8(h, 0);
            h.update(&1000u16.to_le_bytes());
            put_u32(h, players.len() as u32);
            for player in players {
                put_u32(h, player.raw());
            }
            put_u32(h, *count);
        }

        EffectRequest::RoundEnded { round, current } => {
            put_u8(h, 0);
            h.update(&1001u16.to_le_bytes());
            put_u32(h, *round);
            put_u32(h, current.raw());
        }

        EffectRequest::RoundStarted { round, current } => {
            put_u8(h, 0);
            h.update(&1002u16.to_le_bytes());
            put_u32(h, *round);
            put_u32(h, current.raw());
        }

        EffectRequest::ClanyuzhiReveal { player, skill } => {
            put_u8(h, 0);
            h.update(&1003u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ClanyuzhiRoundEnd { player, skill } => {
            put_u8(h, 0);
            h.update(&1004u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::QieyiPhaseStart { player, skill } => {
            put_u8(h, 0);
            h.update(&1005u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::QieyiUseAfter {
            player,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&1006u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::JianzhiPhaseEnd { player, skill } => {
            put_u8(h, 0);
            h.update(&1007u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveTiqi { player, skill } => {
            put_u8(h, 0);
            h.update(&1008u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveBaoshu { player, skill } => {
            put_u8(h, 0);
            h.update(&1009u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        EffectRequest::ResolveDcbihuo { player, skill } => {
            put_u8(h, 0);
            h.update(&1010u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }
        _ => unreachable!("effect routed to a different hash group"),
    }
}
