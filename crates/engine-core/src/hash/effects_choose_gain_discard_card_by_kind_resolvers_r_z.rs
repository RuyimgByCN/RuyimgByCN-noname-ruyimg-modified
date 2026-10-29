//! 效果请求的规范哈希编码：r_z 族结算器。

use super::*;

pub(super) fn hash_effect(h: &mut blake3::Hasher, effect: &EffectRequest) {
    match effect {
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
        } => {
            put_u8(h, 0);
            h.update(&409u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *mode);
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u32(h, *amount);
            put_u8(h, *nature);
            put_u32(h, card.raw());
            put_u8(h, *card_kind);
            put_u8(h, *card_flags);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTingwei {
            player,
            mode,
            card,
            targets,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&410u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *mode);
            put_u32(h, card.raw());
            put_ids(h, targets.iter().map(|id| id.raw()));
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveXunyi {
            player,
            mode,
            source,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&412u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *mode);
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveYiyong {
            player,
            card,
            source,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&415u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, source.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveShanxie { player, skill } => {
            put_u8(h, 0);
            h.update(&416u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSheyi {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&417u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTianyin { player, skill } => {
            put_u8(h, 0);
            h.update(&418u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveZhengsu {
            player,
            mode,
            stage,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&432u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, u32::from(*mode));
            put_u32(h, u32::from(*stage));
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveRechuhai {
            player,
            target,
            card,
            target_card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&435u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, target_card.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveRechuhai { .. } => {
            // 其他 agent 在建批次：落盘前暂以桩哈希。
            put_u8(h, 0);
            h.update(&999u16.to_le_bytes());
        }

        EffectRequest::ResolveXiongshu {
            player,
            target,
            card,
            action,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&436u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, u32::from(*action));
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveXiongshu { .. } => {
            put_u8(h, 0);
            h.update(&997u16.to_le_bytes());
        }

        EffectRequest::ResolveTongduo {
            player,
            target,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&442u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveYuanqing { player, skill } => {
            put_u8(h, 0);
            h.update(&443u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveShuchen {
            player,
            dying,
            renku_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&444u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, dying.raw());
            put_u32(h, renku_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveZaoli {
            player,
            mark,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&446u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *mark);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveRehengjiang {
            player,
            actor,
            action,
            hand_limit_def,
            draw_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&551u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u8(h, *action);
            put_u32(h, hand_limit_def.raw());
            put_u32(h, draw_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTuntian { player, skill } => {
            put_u8(h, 0);
            h.update(&503u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveZaoxian {
            player,
            jixi_def,
            tuntian_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&504u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, jixi_def.raw());
            put_u32(h, tuntian_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTiaoxin {
            player,
            target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&505u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveZhiji {
            player,
            guanxing_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&506u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, guanxing_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveZaiqi {
            player,
            extra,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&507u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *extra);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveZhengrong {
            player,
            candidates,
            target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&509u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, candidates.iter().map(|candidate| candidate.raw()));
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveReleiji {
            player,
            target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&512u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveXiangle {
            player,
            source,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&518u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveRuoyu {
            player,
            jijiang_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&521u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, jijiang_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveZuilun {
            player,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&522u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveShenshi {
            player,
            source,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&529u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveZhenliang {
            player,
            source,
            card,
            mingren_def,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&531u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, mingren_def.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveZhizheng {
            player,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&533u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        _ => unreachable!("effect routed to a different hash group"),
    }
}
