//! 效果请求的规范哈希编码：通用效果原语。

use super::*;

pub(super) fn hash_effect(h: &mut blake3::Hasher, effect: &EffectRequest) {
    match effect {
        EffectRequest::DiscardHandEquipCount {
            player,
            count,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&422u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *count);
            put_u32(h, skill.raw());
        }

        EffectRequest::SkipNextDiscardPhase { player } => {
            put_u8(h, 0);
            h.update(&441u16.to_le_bytes());
            put_u32(h, player.raw());
        }

        EffectRequest::RevealChooseCategoryDistributeSelfThresholdHpLoss {
            player,
            reveal_count,
            recipient_cap,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&450u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *reveal_count);
            put_u8(h, *recipient_cap);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseOtherRandomGainRegionThenDrawCompareColorLoseHp { player, skill } => {
            put_u8(h, 0);
            h.update(&451u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseGainTargetHandCeilHalfDifference {
            player,
            target,
            card,
            skill,
            ready_to_choose,
        } => {
            put_u8(h, 0);
            h.update(&461u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
            put_u8(h, u8::from(*ready_to_choose));
        }

        EffectRequest::GiveSelectedThenTargetMayUseDrawOwner {
            player,
            target,
            card,
            targets,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&468u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseCategoryThenTargetGiveMatchingOrDamagePlusOne {
            player,
            target,
            damage_id,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&472u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *damage_id);
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip {
            player,
            source,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&473u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::EmitCardResolved {
            source,
            card,
            kind,
            targets,
            damaged_targets,
            source_cards,
            all_sources_were_hand,
        } => {
            put_u8(h, 0);
            h.update(&476u16.to_le_bytes());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u8(h, *kind);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_ids(h, damaged_targets.iter().map(|target| target.raw()));
            put_ids(h, source_cards.iter().map(|card| card.raw()));
            put_bool(h, *all_sources_were_hand);
        }

        _ => unreachable!("effect routed to a different hash group"),
    }
}
