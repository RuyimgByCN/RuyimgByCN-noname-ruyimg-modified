//! 效果请求的规范哈希编码：具名技能结算器（Resolve*）。

use super::*;

pub(super) fn hash_effect(h: &mut blake3::Hasher, effect: &EffectRequest) {
    match effect {
        EffectRequest::ResolveSpxizhan {
            player,
            phase_player,
            fangzong_skill_def,
        } => {
            put_u8(h, 0);
            h.update(&372u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, phase_player.raw());
            put_u32(h, fangzong_skill_def.raw());
        }

        EffectRequest::ResolveSpsongshu {
            player,
            phase_player,
            renku_skill_def,
            block_skill_def,
        } => {
            put_u8(h, 0);
            h.update(&384u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, phase_player.raw());
            put_u32(h, renku_skill_def.raw());
            put_u32(h, block_skill_def.raw());
        }

        EffectRequest::ResolveSpcunsi { player, target } => {
            put_u8(h, 0);
            h.update(&390u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
        }

        EffectRequest::ResolveXinmingshi {
            player,
            source,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&411u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveBoming {
            player,
            target,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&413u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveEjian {
            player,
            target,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&414u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDbzhuifeng { player, skill } => {
            put_u8(h, 0);
            h.update(&419u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDbquedi {
            player,
            target,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&420u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSpyinju {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&421u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSpchijie {
            player,
            source,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&422u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSpshangyi {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&423u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSpjianyi { player, skill } => {
            put_u8(h, 0);
            h.update(&424u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveFyjianyu {
            player,
            source,
            target,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&425u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_ids(h, targets.iter().map(|id| id.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveLiubing {
            player,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&426u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveHeji {
            player,
            target,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&427u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSpyajun {
            player,
            less_def,
            card,
            target,
            target_card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&428u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, less_def.raw());
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u32(h, target_card.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSpzundi {
            player,
            target,
            card,
            destination,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&429u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, destination.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveXinyangjie {
            player,
            target,
            card,
            target_card,
            source,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&430u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, target_card.raw());
            put_u32(h, source.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveQingyu {
            player,
            action,
            xuancun_def,
            mbyongjue_def,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&431u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, u32::from(*action));
            put_u32(h, xuancun_def.raw());
            put_u32(h, mbyongjue_def.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveHoufeng {
            player,
            target,
            mode,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&433u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, u32::from(*mode));
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbdiancai {
            player,
            actor,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&434u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveJianhui {
            player,
            source,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&437u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSpqiai {
            player,
            target,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&439u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSpshanxi {
            player,
            target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&440u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSpshanxi { .. } => {
            put_u8(h, 0);
            h.update(&996u16.to_le_bytes());
        }

        EffectRequest::ResolveSpshanxiSuoming {
            player,
            actor,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&441u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSpshanxiSuoming { .. } => {
            put_u8(h, 0);
            h.update(&995u16.to_le_bytes());
        }

        EffectRequest::ResolveDuanbi {
            player,
            targets,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&445u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
            for target in targets {
                put_u32(h, target.raw());
            }
        }

        EffectRequest::ResolveGuying {
            player,
            actor,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&447u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u32(h, card.raw());
            put_u32(h, u32::from(*stage));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbzhonghui {
            player,
            target,
            action,
            amount,
            mbsizi_def,
            mbkechang_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&545u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *action);
            put_u32(h, *amount);
            put_u32(h, mbsizi_def.raw());
            put_u32(h, mbkechang_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotjiejie {
            player,
            actor,
            used_def,
            suit_defs,
            qingshi_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&516u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u32(h, used_def.raw());
            for def in suit_defs {
                put_u32(h, def.raw());
            }
            put_u32(h, qingshi_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotqingshi { player, skill } => {
            put_u8(h, 0);
            h.update(&517u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotweiyan {
            player,
            target,
            card,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&546u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotzhouyu {
            player,
            target,
            card,
            action,
            card_flags,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&547u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u8(h, *card_flags);
            put_u32(h, targets.len() as u32);
            for target in targets {
                put_u32(h, target.raw());
            }
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotlvyi {
            player,
            target,
            card,
            action,
            event_cards,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&548u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, event_cards.len() as u32);
            for card in event_cards {
                put_u32(h, card.raw());
            }
            put_u32(h, targets.len() as u32);
            for target in targets {
                put_u32(h, target.raw());
            }
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotcaoshuang {
            player,
            target,
            card,
            action,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&549u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotyuji {
            player,
            target,
            card,
            action,
            cards,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&550u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *action);
            put_u32(h, cards.len() as u32);
            for card in cards {
                put_u32(h, card.raw());
            }
            put_u32(h, targets.len() as u32);
            for target in targets {
                put_u32(h, target.raw());
            }
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwqiushou { player, skill } => {
            put_u8(h, 0);
            h.update(&508u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveHongju {
            player,
            qingjian_def,
            rong_def,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&510u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, qingjian_def.raw());
            put_u32(h, rong_def.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveQingjian {
            player,
            rong_def,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&511u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, rong_def.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveHuangtian {
            player,
            actor,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&513u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveHaoshi {
            player,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&514u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDimeng {
            player,
            targets,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&515u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveQiaobian {
            player,
            phase,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&516u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *phase);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveFangquanSkip { player, skill } => {
            put_u8(h, 0);
            h.update(&519u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveFangquanGrant {
            player,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&520u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveFuyin {
            player,
            source,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&523u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveJuzhan {
            player,
            source,
            card,
            targets,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&525u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveQizhi {
            player,
            targets,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&526u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveJinqu {
            player,
            qizhi_def,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&527u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, qizhi_def.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveJianxiang {
            player,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&528u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMingren {
            player,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&530u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveKuizhu {
            player,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&532u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveLijun {
            player,
            source,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&534u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveLiangyin {
            player,
            kongsheng_def,
            stage,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&535u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, kongsheng_def.raw());
            put_u8(h, *stage);
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveKongsheng {
            player,
            stage,
            target,
            card,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&536u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDccunsi {
            player,
            target,
            guixiu_def,
            yongjue_def,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&537u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, guixiu_def.raw());
            put_u32(h, yongjue_def.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        _ => unreachable!("effect routed to a different hash group"),
    }
}
