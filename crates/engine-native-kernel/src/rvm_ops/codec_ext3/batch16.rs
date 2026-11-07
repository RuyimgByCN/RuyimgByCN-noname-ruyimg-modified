//! PA12 Batch 16：`EffectRequest` 扩展编解码子表 4（tags 179..=208）。

use super::super::codec::{push_u32, Rd};
use super::super::codec_ext2::{push_card_vec, push_player_vec, rd_card_vec, rd_player_vec};
use super::super::*;

fn push_u8_vec(w: &mut Vec<u8>, bytes: &[u8]) {
    push_u32(w, bytes.len() as u32);
    for b in bytes {
        w.push(*b);
    }
}

fn rd_u8_vec(rd: &mut Rd<'_>) -> Option<Vec<u8>> {
    let n = rd.u32()? as usize;
    let mut out = Vec::with_capacity(n.min(256));
    for _ in 0..n {
        out.push(rd.u8()?);
    }
    Some(out)
}

pub(super) fn enc_batch16_effect(w: &mut Vec<u8>, e: &EffectRequest) {
    match e {
        EffectRequest::ResolveStddechong {
            player,
            target,
            cards,
            skill,
        } => {
            w.push(179);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_card_vec(w, cards);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveDddYujin {
            player,
            actor,
            target,
            move_mask,
            action,
            candidates,
            skill,
        } => {
            w.push(180);
            push_u32(w, player.raw());
            push_u32(w, actor.raw());
            push_u32(w, target.raw());
            w.push(*move_mask);
            w.push(*action);
            push_player_vec(w, candidates);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveDddZhenji {
            player,
            target,
            action,
            stage,
            piles,
            cards,
            skill,
        } => {
            w.push(181);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            w.push(*action);
            w.push(*stage);
            push_u8_vec(w, piles);
            push_card_vec(w, cards);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveZhengjian {
            player,
            skill,
            stage,
            event_player,
        } => {
            w.push(182);
            push_u32(w, player.raw());
            push_u32(w, skill.raw());
            push_u32(w, *stage);
            push_u32(w, event_player.raw());
        }
        EffectRequest::ResolveQiaobian {
            player,
            phase,
            stage,
            skill,
        } => {
            w.push(183);
            push_u32(w, player.raw());
            w.push(*phase);
            w.push(*stage);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveTwfeiyi {
            player,
            action,
            stage,
            card,
            event_cards,
            skill,
        } => {
            w.push(184);
            push_u32(w, player.raw());
            w.push(*action);
            w.push(*stage);
            push_u32(w, card.raw());
            push_card_vec(w, event_cards);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveZhengsu {
            player,
            mode,
            stage,
            card,
            skill,
        } => {
            w.push(185);
            push_u32(w, player.raw());
            w.push(*mode);
            w.push(*stage);
            push_u32(w, card.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveTwcaoang {
            player,
            target,
            source,
            card,
            helper_skill_def,
            action,
            stage,
            skill,
        } => {
            w.push(186);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, source.raw());
            push_u32(w, card.raw());
            push_u32(w, *helper_skill_def);
            w.push(*action);
            w.push(*stage);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveTwcaozhao {
            player,
            target,
            source,
            card,
            selected_skill_def,
            twfuzuan_skill_def,
            twfeifu_skill_def,
            action,
            stage,
            skill,
        } => {
            w.push(187);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, source.raw());
            push_u32(w, card.raw());
            push_u32(w, *selected_skill_def);
            push_u32(w, *twfuzuan_skill_def);
            push_u32(w, *twfeifu_skill_def);
            w.push(*action);
            w.push(*stage);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveTwchengpu {
            player,
            target,
            source,
            card,
            chunlao_skill_def,
            action,
            stage,
            selected_card,
            skill,
        } => {
            w.push(188);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, source.raw());
            push_u32(w, card.raw());
            push_u32(w, *chunlao_skill_def);
            w.push(*action);
            w.push(*stage);
            push_u32(w, selected_card.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveTwchenxun {
            player,
            target,
            card,
            action,
            skill,
        } => {
            w.push(189);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, card.raw());
            w.push(*action);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveTwchenzhen {
            player,
            target,
            helper_skill_def,
            record_skill_def,
            action,
            stage,
            card_kind,
            card,
            branch,
            skill,
        } => {
            w.push(190);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, *helper_skill_def);
            push_u32(w, *record_skill_def);
            w.push(*action);
            w.push(*stage);
            w.push(*card_kind);
            push_u32(w, card.raw());
            w.push(*branch);
            push_u32(w, skill.raw());
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
            w.push(191);
            push_u32(w, player.raw());
            push_u32(w, source.raw());
            push_u32(w, card.raw());
            w.push(*action);
            w.push(*stage);
            push_card_vec(w, event_cards);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveRuanji {
            player,
            source,
            card,
            action,
            cards,
            skill,
        } => {
            w.push(192);
            push_u32(w, player.raw());
            push_u32(w, source.raw());
            push_u32(w, card.raw());
            w.push(*action);
            push_card_vec(w, cards);
            push_u32(w, skill.raw());
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
            w.push(193);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, helper_def.raw());
            w.push(*action);
            push_card_vec(w, cards);
            push_u32(w, *split);
            push_u32(w, skill.raw());
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
            w.push(194);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, card.raw());
            push_u32(w, helper_skill_def.raw());
            w.push(*action);
            push_card_vec(w, event_cards);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveMbduanyang {
            player,
            target,
            card,
            mode,
            stage,
            cards,
            skill,
        } => {
            w.push(195);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, card.raw());
            w.push(*mode);
            w.push(*stage);
            push_card_vec(w, cards);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveMbrunwei {
            player,
            target,
            stage,
            value,
            cards,
            skill,
        } => {
            w.push(196);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            w.push(*stage);
            push_u32(w, *value);
            push_card_vec(w, cards);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveMbkuangxiang {
            player,
            target,
            stage,
            skill,
        } => {
            w.push(197);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            w.push(*stage);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolvePotsifeng {
            player,
            target,
            card,
            stage,
            skill,
        } => {
            w.push(198);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, card.raw());
            w.push(*stage);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolvePotwanglie {
            player,
            card,
            helper_skill_def,
            stage,
            skill,
        } => {
            w.push(199);
            push_u32(w, player.raw());
            push_u32(w, card.raw());
            push_u32(w, helper_skill_def.raw());
            w.push(*stage);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolvePotchengpu {
            player,
            target,
            stage,
            damage_id,
            skill,
        } => {
            w.push(200);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            w.push(*stage);
            push_u32(w, *damage_id);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolvePottuntian {
            player,
            action,
            amount,
            targets,
            zaoxian_def,
            skill,
        } => {
            w.push(201);
            push_u32(w, player.raw());
            w.push(*action);
            w.push(*amount);
            push_player_vec(w, targets);
            push_u32(w, zaoxian_def.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolvePotjixi {
            player,
            target,
            action,
            targets,
            skill,
        } => {
            w.push(202);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            w.push(*action);
            push_player_vec(w, targets);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolvePotjiyu {
            player,
            card,
            stage,
            skill,
        } => {
            w.push(203);
            push_u32(w, player.raw());
            push_u32(w, card.raw());
            w.push(*stage);
            push_u32(w, skill.raw());
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
            w.push(204);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, card.raw());
            push_player_vec(w, targets);
            w.push(*index);
            w.push(*declined);
            w.push(*stage);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolvePotfuan {
            player,
            card,
            stage,
            range_helper_def,
            yinxian_def,
            skill,
        } => {
            w.push(205);
            push_u32(w, player.raw());
            push_u32(w, card.raw());
            w.push(*stage);
            push_u32(w, range_helper_def.raw());
            push_u32(w, yinxian_def.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveDddGuanning {
            player,
            actor,
            action,
            card,
            skill,
        } => {
            w.push(206);
            push_u32(w, player.raw());
            push_u32(w, actor.raw());
            w.push(*action);
            push_u32(w, card.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveDddJianshuo {
            player,
            target,
            action,
            cards,
            targets,
            skill,
        } => {
            w.push(207);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            w.push(*action);
            push_card_vec(w, cards);
            push_player_vec(w, targets);
            push_u32(w, skill.raw());
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
            w.push(208);
            push_u32(w, player.raw());
            push_u32(w, event_player.raw());
            push_u32(w, *event_amount);
            push_u32(w, mark.raw());
            w.push(*mode);
            push_card_vec(w, cards);
            push_player_vec(w, targets);
            push_u32(w, skill.raw());
        }
        _ => unreachable!("accept_effect 已拒绝未编码变体"),
    }
}

pub(super) fn dec_batch16_effect(tag: u8, rd: &mut Rd<'_>) -> Option<EffectRequest> {
    Some(match tag {
        179 => EffectRequest::ResolveStddechong {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            cards: rd_card_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        180 => EffectRequest::ResolveDddYujin {
            player: rule_api::PlayerId::new(rd.u32()?),
            actor: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            move_mask: rd.u8()?,
            action: rd.u8()?,
            candidates: rd_player_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        181 => EffectRequest::ResolveDddZhenji {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            action: rd.u8()?,
            stage: rd.u8()?,
            piles: rd_u8_vec(rd)?,
            cards: rd_card_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        182 => EffectRequest::ResolveZhengjian {
            player: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
            stage: rd.u32()?,
            event_player: rule_api::PlayerId::new(rd.u32()?),
        },
        183 => EffectRequest::ResolveQiaobian {
            player: rule_api::PlayerId::new(rd.u32()?),
            phase: rd.u8()?,
            stage: rd.u8()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        184 => EffectRequest::ResolveTwfeiyi {
            player: rule_api::PlayerId::new(rd.u32()?),
            action: rd.u8()?,
            stage: rd.u8()?,
            card: rule_api::CardId::new(rd.u32()?),
            event_cards: rd_card_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        185 => EffectRequest::ResolveZhengsu {
            player: rule_api::PlayerId::new(rd.u32()?),
            mode: rd.u8()?,
            stage: rd.u8()?,
            card: rule_api::CardId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        186 => EffectRequest::ResolveTwcaoang {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            source: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            helper_skill_def: rd.u32()?,
            action: rd.u8()?,
            stage: rd.u8()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        187 => EffectRequest::ResolveTwcaozhao {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            source: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            selected_skill_def: rd.u32()?,
            twfuzuan_skill_def: rd.u32()?,
            twfeifu_skill_def: rd.u32()?,
            action: rd.u8()?,
            stage: rd.u8()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        188 => EffectRequest::ResolveTwchengpu {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            source: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            chunlao_skill_def: rd.u32()?,
            action: rd.u8()?,
            stage: rd.u8()?,
            selected_card: rule_api::CardId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        189 => EffectRequest::ResolveTwchenxun {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            action: rd.u8()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        190 => EffectRequest::ResolveTwchenzhen {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            helper_skill_def: rd.u32()?,
            record_skill_def: rd.u32()?,
            action: rd.u8()?,
            stage: rd.u8()?,
            card_kind: rd.u8()?,
            card: rule_api::CardId::new(rd.u32()?),
            branch: rd.u8()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        191 => EffectRequest::ResolveTwyizhu {
            player: rule_api::PlayerId::new(rd.u32()?),
            source: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            action: rd.u8()?,
            stage: rd.u8()?,
            event_cards: rd_card_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        192 => EffectRequest::ResolveRuanji {
            player: rule_api::PlayerId::new(rd.u32()?),
            source: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            action: rd.u8()?,
            cards: rd_card_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        193 => EffectRequest::ResolveQiaojian {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            helper_def: rule_api::SkillDefId::new(rd.u32()?),
            action: rd.u8()?,
            cards: rd_card_vec(rd)?,
            split: rd.u32()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        194 => EffectRequest::ResolveZhupeilan {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            helper_skill_def: rule_api::SkillDefId::new(rd.u32()?),
            action: rd.u8()?,
            event_cards: rd_card_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        195 => EffectRequest::ResolveMbduanyang {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            mode: rd.u8()?,
            stage: rd.u8()?,
            cards: rd_card_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        196 => EffectRequest::ResolveMbrunwei {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            stage: rd.u8()?,
            value: rd.u32()?,
            cards: rd_card_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        197 => EffectRequest::ResolveMbkuangxiang {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            stage: rd.u8()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        198 => EffectRequest::ResolvePotsifeng {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            stage: rd.u8()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        199 => EffectRequest::ResolvePotwanglie {
            player: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            helper_skill_def: rule_api::SkillDefId::new(rd.u32()?),
            stage: rd.u8()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        200 => EffectRequest::ResolvePotchengpu {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            stage: rd.u8()?,
            damage_id: rd.u32()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        201 => EffectRequest::ResolvePottuntian {
            player: rule_api::PlayerId::new(rd.u32()?),
            action: rd.u8()?,
            amount: rd.u8()?,
            targets: rd_player_vec(rd)?,
            zaoxian_def: rule_api::SkillDefId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        202 => EffectRequest::ResolvePotjixi {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            action: rd.u8()?,
            targets: rd_player_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        203 => EffectRequest::ResolvePotjiyu {
            player: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            stage: rd.u8()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        204 => EffectRequest::ResolvePotnigu {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            targets: rd_player_vec(rd)?,
            index: rd.u8()?,
            declined: rd.u8()?,
            stage: rd.u8()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        205 => EffectRequest::ResolvePotfuan {
            player: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            stage: rd.u8()?,
            range_helper_def: rule_api::SkillDefId::new(rd.u32()?),
            yinxian_def: rule_api::SkillDefId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        206 => EffectRequest::ResolveDddGuanning {
            player: rule_api::PlayerId::new(rd.u32()?),
            actor: rule_api::PlayerId::new(rd.u32()?),
            action: rd.u8()?,
            card: rule_api::CardId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        207 => EffectRequest::ResolveDddJianshuo {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            action: rd.u8()?,
            cards: rd_card_vec(rd)?,
            targets: rd_player_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        208 => EffectRequest::ResolveShenSimayi {
            player: rule_api::PlayerId::new(rd.u32()?),
            event_player: rule_api::PlayerId::new(rd.u32()?),
            event_amount: rd.u32()?,
            mark: rule_api::MarkDefId::new(rd.u32()?),
            mode: rd.u8()?,
            cards: rd_card_vec(rd)?,
            targets: rd_player_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        _ => return None,
    })
}
