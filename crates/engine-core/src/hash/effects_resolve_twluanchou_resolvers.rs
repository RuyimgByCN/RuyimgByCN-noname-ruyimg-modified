//! 效果请求的规范哈希编码：具名技能结算器（Resolve*）。

use super::*;

pub(super) fn hash_effect(h: &mut blake3::Hasher, effect: &EffectRequest) {
    match effect {
        EffectRequest::ResolveTianxiang {
            player,
            target,
            discard_card,
            source,
            amount,
            nature,
            card,
            card_kind,
            card_flags,
            draw_skill_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&416u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, discard_card.raw());
            put_u32(h, source.raw());
            put_u32(h, *amount);
            put_u8(h, *nature);
            put_u32(h, card.raw());
            put_u8(h, *card_kind);
            put_u8(h, *card_flags);
            put_u32(h, draw_skill_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveRetianxiang {
            player,
            target,
            discard_card,
            branch,
            source,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&418u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, discard_card.raw());
            put_u8(h, *branch);
            put_u32(h, source.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveLianzhou {
            player,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&419u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveStddechong {
            player,
            target,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&420u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveJieliangGain {
            player,
            mover,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&423u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, mover.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveStdjujian {
            player,
            target,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&424u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveLulve {
            player,
            target,
            branch,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&427u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *branch);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDctingxian {
            player,
            card,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&429u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveHfjieying {
            player,
            target,
            grant_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&430u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, grant_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDcnuchen {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&432u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDcdanji { player, skill } => {
            put_u8(h, 0);
            h.update(&435u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDczhiheng { player, skill } => {
            put_u8(h, 0);
            h.update(&436u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveStdgaobian {
            player,
            chooser,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&437u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, chooser.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSpjiedao { player, skill } => {
            put_u8(h, 0);
            h.update(&439u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSibian { player, skill } => {
            put_u8(h, 0);
            h.update(&440u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSpfengyin {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&442u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveXinfuLianpian {
            player,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&443u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, targets.len() as u32);
            for target in targets {
                put_u32(h, target.raw());
            }
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveGongjian {
            player,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&444u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, targets.len() as u32);
            for target in targets {
                put_u32(h, target.raw());
            }
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveRefuhai { player, skill } => {
            put_u8(h, 0);
            h.update(&456u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveGzjinfa {
            player,
            target,
            cost_card,
            given_card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&464u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, cost_card.raw());
            put_u32(h, given_card.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveWnQifeng {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&466u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveXinkuangfu {
            player,
            target,
            sha_target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&467u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, sha_target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDcxiangshu {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&469u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDcxiangshuLottery {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&470u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDcyingtu {
            player,
            target,
            stage,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&471u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMobiledanshou {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&474u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveJsrgchiying {
            player,
            target,
            started,
            remaining,
            accum,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&475u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_bool(h, *started);
            put_u32(h, remaining.len() as u32);
            for participant in remaining {
                put_u32(h, participant.raw());
            }
            put_u32(h, accum.len() as u32);
            for card in accum {
                put_u32(h, card.raw());
            }
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveShuangxiong { player, skill } => {
            put_u8(h, 0);
            h.update(&493u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveZhijian {
            player,
            card,
            stage,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&494u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveGuzheng {
            player,
            event_player,
            stage,
            cards,
            return_card,
            take_rest,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&495u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, event_player.raw());
            put_u8(h, *stage);
            put_ids(h, cards.iter().copied());
            put_u32(h, return_card.raw());
            put_u8(h, u8::from(*take_rest));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveZhiba {
            player,
            actor,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&496u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveZhibaSettle {
            player,
            actor,
            winner,
            player_card,
            target_card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&497u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, actor.raw());
            put_u32(h, winner.raw());
            put_u32(h, player_card.raw());
            put_u32(h, target_card.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDrltCongjian {
            player,
            targets,
            card,
            target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&498u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, card.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveNzryFeijun {
            player,
            target,
            card,
            branch,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&499u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *branch);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDrltYongsi {
            player,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&500u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDrltWeidi {
            player,
            cards,
            targets,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&501u16.to_le_bytes());
            put_u32(h, player.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveLieren {
            player,
            target,
            win_skill_def,
            lose_skill_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&502u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, win_skill_def.raw());
            put_u32(h, lose_skill_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotchengpu {
            player,
            target,
            stage,
            damage_id,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&498u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, *damage_id);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotjugu { player, skill } => {
            put_u8(h, 0);
            h.update(&499u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotsifeng {
            player,
            target,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&500u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotchenqun {
            player,
            target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&501u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotwanglie {
            player,
            card,
            helper_skill_def,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&502u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u32(h, helper_skill_def.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePothongyi {
            player,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&503u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotguansha {
            player,
            helper_skill_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&504u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, helper_skill_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotjiyu {
            player,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&505u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotlibing {
            player,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&507u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotpoxi {
            player,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&508u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, cards.len() as u32);
            for card in cards {
                put_u32(h, card.raw());
            }
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotfuan {
            player,
            card,
            stage,
            range_helper_def,
            yinxian_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&509u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u32(h, range_helper_def.raw());
            put_u32(h, yinxian_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotyinxian {
            player,
            card,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&510u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePothuanjie {
            player,
            target,
            stage,
            qice_def,
            kanpo_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&511u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, qice_def.raw());
            put_u32(h, kanpo_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolvePotnigu {
            player,
            target,
            card,
            targets,
            index,
            declined,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&512u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u8(h, *index);
            put_u8(h, *declined);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }
        _ => unreachable!("effect routed to a different hash group"),
    }
}
