//! 效果请求的规范哈希编码：具名技能结算器（EffectResolve*）。

use super::*;

pub(super) fn hash_effect(h: &mut blake3::Hasher, effect: &EffectRequest) {
    match effect {
        EffectRequest::ResolveDcjue {
            player,
            target,
            stage,
            remaining,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&572u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, *remaining);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveRuanji {
            player,
            source,
            card,
            action,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&573u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveZhengbi {
            player,
            target,
            card,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&574u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveFengying {
            player,
            helper_def,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&575u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, helper_def.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveQiaojian {
            player,
            target,
            helper_def,
            action,
            cards,
            split,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&576u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, helper_def.raw());
            put_u8(h, *action);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, *split);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveHuzun {
            player,
            source,
            card,
            action,
            cards,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&582u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveZhugemengxue {
            player,
            target,
            card,
            action,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&583u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveChenglve {
            player,
            stage,
            suits,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&585u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u8(h, *suits);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveShicai {
            player,
            card,
            placed,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&586u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_ids(h, placed.iter().map(|card| card.raw()));
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveBailingyun {
            player,
            target,
            card,
            action,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&588u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDcshaowei {
            player,
            action,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&591u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *action);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDcdichou {
            player,
            target,
            card,
            action,
            helper_skill_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&592u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, helper_skill_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDcdufeng {
            player,
            action,
            helper_skill_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&593u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *action);
            put_u32(h, helper_skill_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDcctjiuxian {
            player,
            event_player,
            target,
            card,
            action,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&595u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, event_player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveBianyue {
            player,
            target,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&598u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveZhupeilan {
            player,
            target,
            card,
            helper_skill_def,
            action,
            event_cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&600u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, helper_skill_def.raw());
            put_u8(h, *action);
            put_ids(h, event_cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDcSbZhangxiu {
            player,
            target,
            action,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&601u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *action);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDcSbZhouyu {
            player,
            source,
            card,
            action,
            event_targets,
            damaged_targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&602u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_ids(h, event_targets.iter().map(|target| target.raw()));
            put_ids(h, damaged_targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDcSbLusu {
            player,
            source,
            card,
            action,
            cards,
            amount,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&603u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, *amount);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveZhangjian {
            player,
            card,
            grant_skill_def,
            action,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&604u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, grant_skill_def.raw());
            put_u8(h, *action);
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveZhugeruoxue {
            player,
            target,
            card,
            action,
            remaining,
            used_mask,
            repeated,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&605u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, *remaining);
            put_u32(h, *used_mask);
            put_u8(h, u8::from(*repeated));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveCaoyi {
            player,
            card,
            helper_skill_def,
            action,
            event_targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&606u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, helper_skill_def.raw());
            put_u8(h, *action);
            put_ids(h, event_targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

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
        } => {
            put_u8(h, 0);
            h.update(&607u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, helper_skill_def.raw());
            put_u8(h, *action);
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u8(h, *nature);
            put_u32(h, *damage_id);
            put_ids(h, event_cards.iter().map(|card| card.raw()));
            put_ids(h, event_targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveNsshuaiyan {
            player,
            recovered_player,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&608u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, recovered_player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveShoulijian {
            player,
            target,
            skill,
        } => {
            // ABI 642: 欢乐卡牌手里剑 custom-basic-card settlement.
            put_u8(h, 0);
            h.update(&642u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveGwyewuChain {
            player,
            skill,
            cost_cards,
            wuyao_def,
            lang_def,
        } => {
            // ABI 645: 叶舞 (gwent) random-discard chain + 雾妖/狼 skill gains.
            put_u8(h, 0);
            h.update(&645u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
            put_u32(h, cost_cards.len() as u32);
            for card in cost_cards {
                put_u32(h, card.raw());
            }
            put_u32(h, wuyao_def.raw());
            put_u32(h, lang_def.raw());
        }

        EffectRequest::ResolveXiongluan {
            player,
            target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&589u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveGzbuqu {
            player,
            delta,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&591u16.to_le_bytes());
            put_u32(h, player.raw());
            put_i32(h, *delta);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveGzbuquRecover {
            player,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&592u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSbbiyue { player, skill } => {
            put_u8(h, 0);
            h.update(&599u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSxrmqiewang {
            player,
            event_player,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&596u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, event_player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSxrmqianliu {
            player,
            source,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&597u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwjuxiang {
            player,
            event_player,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&590u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, event_player.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveXicha {
            player,
            source,
            card,
            action,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&577u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePottuntian {
            player,
            action,
            amount,
            targets,
            zaoxian_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&509u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *action);
            put_u8(h, *amount);
            put_u32(h, targets.len() as u32);
            for target in targets {
                put_u32(h, target.raw());
            }
            put_u32(h, zaoxian_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotjixi {
            player,
            target,
            action,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&510u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *action);
            put_u32(h, targets.len() as u32);
            for target in targets {
                put_u32(h, target.raw());
            }
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotceduan {
            player,
            target,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&512u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePottaishici {
            player,
            target,
            action,
            cards,
            hanzhan_def,
            zhanlie_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&519u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *action);
            put_u32(h, cards.len() as u32);
            for card in cards {
                put_u32(h, card.raw());
            }
            put_u32(h, hanzhan_def.raw());
            put_u32(h, zhanlie_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwfupan {
            player,
            damage_amount,
            card,
            target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&373u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *damage_amount);
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwjiaohua {
            player,
            target,
            event_cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&375u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_ids(h, event_cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwkujian {
            player,
            target,
            cards,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&386u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwruilian {
            player,
            target,
            event_cards,
            action,
            category,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&387u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_ids(h, event_cards.iter().map(|card| card.raw()));
            put_u8(h, *action);
            put_u8(h, *category);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwjianwei {
            player,
            actor,
            target,
            card,
            winner,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&389u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, winner.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwtanfeng {
            player,
            target,
            card,
            stage,
            phase,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&391u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u8(h, *phase);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwzhibian {
            player,
            target,
            winner,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&392u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, winner.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwyuyan {
            player,
            source,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&393u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwguoyi {
            player,
            target,
            card,
            hand_lock_skill_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&394u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, *hand_lock_skill_def);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwchuhai {
            player,
            action,
            stage,
            index,
            event_cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&396u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, *index);
            put_ids(h, event_cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwyizhu {
            player,
            source,
            card,
            action,
            stage,
            event_cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&397u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u8(h, *stage);
            put_ids(h, event_cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }
        _ => unreachable!("effect routed to a different hash group"),
    }
}
