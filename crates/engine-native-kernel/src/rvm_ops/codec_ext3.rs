//! PA12 Batch 15：`EffectRequest` 扩展编解码子表 3（tags 154..=178 + tag 255 u16 扩展头预留）。

use super::codec::{push_u32, Rd};
use super::codec_ext2::{push_card_vec, push_player_vec, rd_card_vec, rd_player_vec};
use super::*;

pub(super) fn enc_ext3_effect(w: &mut Vec<u8>, e: &EffectRequest) {
    match e {
        EffectRequest::ResolvePotweiyan {
            player,
            target,
            card,
            action,
            skill,
        } => {
            w.push(154);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, card.raw());
            w.push(*action);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolvePotcaoshuang {
            player,
            target,
            card,
            action,
            skill,
        } => {
            w.push(155);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, card.raw());
            w.push(*action);
            push_u32(w, skill.raw());
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
            w.push(156);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, card.raw());
            w.push(*action);
            push_card_vec(w, event_cards);
            push_player_vec(w, targets);
            push_u32(w, skill.raw());
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
            w.push(157);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, card.raw());
            w.push(*action);
            push_card_vec(w, cards);
            push_player_vec(w, targets);
            push_u32(w, skill.raw());
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
            w.push(158);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, card.raw());
            w.push(*action);
            w.push(*card_flags);
            push_player_vec(w, targets);
            push_u32(w, skill.raw());
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
            w.push(159);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            w.push(*action);
            push_u32(w, *amount);
            push_u32(w, mbsizi_def.raw());
            push_u32(w, mbkechang_def.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveDddLiuhong {
            player,
            actor,
            action,
            skill,
        } => {
            w.push(160);
            push_u32(w, player.raw());
            push_u32(w, actor.raw());
            w.push(*action);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveTwguohuai {
            player,
            target,
            source,
            card,
            mark_def,
            helper_skill_def,
            action,
            stage,
            skill,
        } => {
            w.push(161);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, source.raw());
            push_u32(w, card.raw());
            push_u32(w, *mark_def);
            push_u32(w, *helper_skill_def);
            w.push(*action);
            w.push(*stage);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveDddXuelingyun {
            player,
            actor,
            card,
            cards,
            action,
            skill,
        } => {
            w.push(162);
            push_u32(w, player.raw());
            push_u32(w, actor.raw());
            push_u32(w, card.raw());
            push_card_vec(w, cards);
            w.push(*action);
            push_u32(w, skill.raw());
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
            w.push(163);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            w.push(*action);
            push_card_vec(w, cards);
            push_u32(w, hanzhan_def.raw());
            push_u32(w, zhanlie_def.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveDddXujing {
            player,
            source,
            card,
            action,
            targets,
            skill,
        } => {
            w.push(164);
            push_u32(w, player.raw());
            push_u32(w, source.raw());
            push_u32(w, card.raw());
            w.push(*action);
            push_player_vec(w, targets);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveDddLiuba {
            player,
            actor,
            action,
            cards,
            targets,
            skill,
        } => {
            w.push(165);
            push_u32(w, player.raw());
            push_u32(w, actor.raw());
            w.push(*action);
            push_card_vec(w, cards);
            push_player_vec(w, targets);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveDddLiuye {
            player,
            actor,
            card,
            action,
            skill,
        } => {
            w.push(166);
            push_u32(w, player.raw());
            push_u32(w, actor.raw());
            push_u32(w, card.raw());
            w.push(*action);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveDddZhangkai {
            player,
            actor,
            card,
            cards,
            amount,
            action,
            skill,
        } => {
            w.push(167);
            push_u32(w, player.raw());
            push_u32(w, actor.raw());
            push_u32(w, card.raw());
            push_card_vec(w, cards);
            push_u32(w, *amount);
            w.push(*action);
            push_u32(w, skill.raw());
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
            w.push(168);
            push_u32(w, player.raw());
            push_u32(w, pile_skill_def.raw());
            w.push(*action);
            push_u32(w, card.raw());
            w.push(*stage);
            push_card_vec(w, hand_cards);
            push_card_vec(w, pile_cards);
            push_card_vec(w, event_cards);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveTwfuman {
            player,
            target,
            cards,
            action,
            stage,
            skill,
        } => {
            w.push(169);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_card_vec(w, cards);
            w.push(*action);
            w.push(*stage);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveTwkujian {
            player,
            target,
            cards,
            action,
            skill,
        } => {
            w.push(170);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_card_vec(w, cards);
            w.push(*action);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveTwxiawei {
            player,
            wangxing_skill_def,
            action,
            stage,
            skill,
        } => {
            w.push(171);
            push_u32(w, player.raw());
            push_u32(w, *wangxing_skill_def);
            w.push(*action);
            w.push(*stage);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveCaoyi {
            player,
            card,
            helper_skill_def,
            action,
            event_targets,
            skill,
        } => {
            w.push(172);
            push_u32(w, player.raw());
            push_u32(w, card.raw());
            push_u32(w, helper_skill_def.raw());
            w.push(*action);
            push_player_vec(w, event_targets);
            push_u32(w, skill.raw());
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
            w.push(173);
            push_u32(w, player.raw());
            push_u32(w, source.raw());
            push_u32(w, card.raw());
            w.push(*action);
            push_card_vec(w, cards);
            push_u32(w, *amount);
            push_u32(w, skill.raw());
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
            w.push(174);
            push_u32(w, player.raw());
            push_u32(w, source.raw());
            push_u32(w, card.raw());
            w.push(*action);
            push_player_vec(w, event_targets);
            push_player_vec(w, damaged_targets);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveMbxiongtu {
            player,
            target,
            stage,
            card,
            cards,
            skill,
        } => {
            w.push(175);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            w.push(*stage);
            push_u32(w, card.raw());
            push_card_vec(w, cards);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolvePothongyi {
            player,
            stage,
            skill,
        } => {
            w.push(176);
            push_u32(w, player.raw());
            w.push(*stage);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveSpyingjia {
            player,
            action,
            helper_def,
            card,
            card_kind,
            target,
            skill,
        } => {
            w.push(177);
            push_u32(w, player.raw());
            w.push(*action);
            push_u32(w, helper_def.raw());
            push_u32(w, card.raw());
            w.push(*card_kind);
            push_u32(w, target.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveDddCaoshuang {
            player,
            actor,
            card,
            action,
            skill,
        } => {
            w.push(178);
            push_u32(w, player.raw());
            push_u32(w, actor.raw());
            push_u32(w, card.raw());
            w.push(*action);
            push_u32(w, skill.raw());
        }
        _ => unreachable!("accept_effect 已拒绝未编码变体"),
    }
}

pub(super) fn dec_ext3_effect(tag: u8, rd: &mut Rd<'_>) -> Option<EffectRequest> {
    Some(match tag {
        154 => EffectRequest::ResolvePotweiyan {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            action: rd.u8()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        155 => EffectRequest::ResolvePotcaoshuang {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            action: rd.u8()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        156 => EffectRequest::ResolvePotlvyi {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            action: rd.u8()?,
            event_cards: rd_card_vec(rd)?,
            targets: rd_player_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        157 => EffectRequest::ResolvePotyuji {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            action: rd.u8()?,
            cards: rd_card_vec(rd)?,
            targets: rd_player_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        158 => EffectRequest::ResolvePotzhouyu {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            action: rd.u8()?,
            card_flags: rd.u8()?,
            targets: rd_player_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        159 => EffectRequest::ResolveMbzhonghui {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            action: rd.u8()?,
            amount: rd.u32()?,
            mbsizi_def: rule_api::SkillDefId::new(rd.u32()?),
            mbkechang_def: rule_api::SkillDefId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        160 => EffectRequest::ResolveDddLiuhong {
            player: rule_api::PlayerId::new(rd.u32()?),
            actor: rule_api::PlayerId::new(rd.u32()?),
            action: rd.u8()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        161 => EffectRequest::ResolveTwguohuai {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            source: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            mark_def: rd.u32()?,
            helper_skill_def: rd.u32()?,
            action: rd.u8()?,
            stage: rd.u8()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        162 => EffectRequest::ResolveDddXuelingyun {
            player: rule_api::PlayerId::new(rd.u32()?),
            actor: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            cards: rd_card_vec(rd)?,
            action: rd.u8()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        163 => EffectRequest::ResolvePottaishici {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            action: rd.u8()?,
            cards: rd_card_vec(rd)?,
            hanzhan_def: rule_api::SkillDefId::new(rd.u32()?),
            zhanlie_def: rule_api::SkillDefId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        164 => EffectRequest::ResolveDddXujing {
            player: rule_api::PlayerId::new(rd.u32()?),
            source: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            action: rd.u8()?,
            targets: rd_player_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        165 => EffectRequest::ResolveDddLiuba {
            player: rule_api::PlayerId::new(rd.u32()?),
            actor: rule_api::PlayerId::new(rd.u32()?),
            action: rd.u8()?,
            cards: rd_card_vec(rd)?,
            targets: rd_player_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        166 => EffectRequest::ResolveDddLiuye {
            player: rule_api::PlayerId::new(rd.u32()?),
            actor: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            action: rd.u8()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        167 => EffectRequest::ResolveDddZhangkai {
            player: rule_api::PlayerId::new(rd.u32()?),
            actor: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            cards: rd_card_vec(rd)?,
            amount: rd.u32()?,
            action: rd.u8()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        168 => EffectRequest::ResolveHiddenSkillPile {
            player: rule_api::PlayerId::new(rd.u32()?),
            pile_skill_def: rule_api::SkillDefId::new(rd.u32()?),
            action: rd.u8()?,
            card: rule_api::CardId::new(rd.u32()?),
            stage: rd.u8()?,
            hand_cards: rd_card_vec(rd)?,
            pile_cards: rd_card_vec(rd)?,
            event_cards: rd_card_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        169 => EffectRequest::ResolveTwfuman {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            cards: rd_card_vec(rd)?,
            action: rd.u8()?,
            stage: rd.u8()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        170 => EffectRequest::ResolveTwkujian {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            cards: rd_card_vec(rd)?,
            action: rd.u8()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        171 => EffectRequest::ResolveTwxiawei {
            player: rule_api::PlayerId::new(rd.u32()?),
            wangxing_skill_def: rd.u32()?,
            action: rd.u8()?,
            stage: rd.u8()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        172 => EffectRequest::ResolveCaoyi {
            player: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            helper_skill_def: rule_api::SkillDefId::new(rd.u32()?),
            action: rd.u8()?,
            event_targets: rd_player_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        173 => EffectRequest::ResolveDcSbLusu {
            player: rule_api::PlayerId::new(rd.u32()?),
            source: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            action: rd.u8()?,
            cards: rd_card_vec(rd)?,
            amount: rd.u32()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        174 => EffectRequest::ResolveDcSbZhouyu {
            player: rule_api::PlayerId::new(rd.u32()?),
            source: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            action: rd.u8()?,
            event_targets: rd_player_vec(rd)?,
            damaged_targets: rd_player_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        175 => EffectRequest::ResolveMbxiongtu {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            stage: rd.u8()?,
            card: rule_api::CardId::new(rd.u32()?),
            cards: rd_card_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        176 => EffectRequest::ResolvePothongyi {
            player: rule_api::PlayerId::new(rd.u32()?),
            stage: rd.u8()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        177 => EffectRequest::ResolveSpyingjia {
            player: rule_api::PlayerId::new(rd.u32()?),
            action: rd.u8()?,
            helper_def: rule_api::SkillDefId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            card_kind: rd.u8()?,
            target: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        178 => EffectRequest::ResolveDddCaoshuang {
            player: rule_api::PlayerId::new(rd.u32()?),
            actor: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            action: rd.u8()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        255 => {
            let ext_tag = u16::from_le_bytes([rd.u8()?, rd.u8()?]);
            decode_ext_u16(ext_tag, rd)?
        }
        _ => return None,
    })
}

fn decode_ext_u16(_ext_tag: u16, _rd: &mut Rd<'_>) -> Option<EffectRequest> {
    None
}
