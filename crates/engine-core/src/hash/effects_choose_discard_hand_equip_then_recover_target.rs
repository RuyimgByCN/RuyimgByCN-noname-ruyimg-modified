//! 效果请求的规范哈希编码：通用效果原语（自 `effects_choose_discard_hand_equip_then_recover_target` 分组拆出具名技能结算器之后）。

use super::*;

pub(super) fn hash_effect(h: &mut blake3::Hasher, effect: &EffectRequest) {
    match effect {
        EffectRequest::ChooseDiscardHandEquipThenRecoverTarget {
            chooser,
            target,
            count,
            amount,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&571u16.to_le_bytes());
            put_u32(h, chooser.raw());
            put_u32(h, target.raw());
            put_u32(h, *count);
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }

        EffectRequest::DrawCardsFromBottom { player, count } => {
            put_u8(h, 0);
            h.update(&584u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *count);
        }

        EffectRequest::SetDrawPhaseFromBottom { player } => {
            put_u8(h, 0);
            h.update(&587u16.to_le_bytes());
            put_u32(h, player.raw());
        }

        EffectRequest::ChooseDiscardHandRange {
            player,
            min,
            max,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&594u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *min);
            put_u32(h, *max);
            put_u32(h, skill.raw());
        }

        EffectRequest::ContinueYoudishenru {
            action,
            protected,
            source,
            forced_card,
            saved,
        } => {
            put_u8(h, 1);
            h.update(&613u16.to_le_bytes());
            put_u8(h, *action);
            put_u32(h, protected.raw());
            put_u32(h, source.raw());
            put_u32(h, forced_card.raw());
            put_ids(h, saved.iter().copied());
        }

        EffectRequest::ContinueWdbFuchou {
            action,
            player,
            target,
            remaining,
            skill,
        } => {
            put_u8(h, 1);
            h.update(&614u16.to_le_bytes());
            put_u8(h, *action);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_ids(h, remaining.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ContinueDiaobingqianjiang {
            action,
            actor,
            source_card,
            targets,
            revealed,
        } => {
            put_u8(h, 1);
            h.update(&615u16.to_le_bytes());
            put_u8(h, *action);
            put_u32(h, actor.raw());
            put_u32(h, source_card.raw());
            put_ids(h, targets.iter().map(|player| player.raw()));
            put_ids(h, revealed.iter().map(|card| card.raw()));
        }

        EffectRequest::PreventRecovery { recovery_id } => {
            put_u8(h, 1);
            h.update(&635u16.to_le_bytes());
            put_u32(h, *recovery_id);
        }

        EffectRequest::CommitRecovery {
            recovery_id,
            source,
            player,
            amount,
        } => {
            put_u8(h, 2);
            h.update(&635u16.to_le_bytes());
            put_u32(h, *recovery_id);
            put_u32(h, source.raw());
            put_u32(h, player.raw());
            put_u32(h, *amount);
        }

        EffectRequest::GuozhanUseZhulianBihe { player } => {
            // ABI 641: VM-driven guozhan 珠联璧合 spend-as-无中生有 primitive.
            put_u8(h, 0);
            h.update(&641u16.to_le_bytes());
            put_u32(h, player.raw());
        }

        EffectRequest::CancelIncomingCardUse {
            player,
            source,
            card,
            skill,
        } => {
            // ABI 643: 取消使用牌 (gwent 杜达元帅1 special-card cancel-use primitive).
            put_u8(h, 0);
            h.update(&643u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ChooseOtherGainOneOfTwoSkills {
            player,
            target,
            first_def,
            second_def,
            skill,
        } => {
            // ABI 644: 择一加技能 (gwent 呼朋 choose-one-of-two-skill grant primitive).
            put_u8(h, 0);
            h.update(&644u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, first_def.raw());
            put_u32(h, second_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ContinueWugu {
            action,
            actor,
            source_card,
            targets,
            excluded,
            revealed,
        } => {
            put_u8(h, 1);
            h.update(&616u16.to_le_bytes());
            put_u8(h, *action);
            put_u32(h, actor.raw());
            put_u32(h, source_card.raw());
            put_ids(h, targets.iter().map(|player| player.raw()));
            put_ids(h, excluded.iter().map(|player| player.raw()));
            put_ids(h, revealed.iter().map(|card| card.raw()));
        }

        EffectRequest::EquipWeaponFromDrawPile { player } => {
            put_u8(h, 0);
            h.update(&388u16.to_le_bytes());
            put_u32(h, player.raw());
        }

        EffectRequest::ReplayVirtualCardOnTargets {
            player,
            kind,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&395u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *kind);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        _ => unreachable!("effect routed to a different hash group"),
    }
}
