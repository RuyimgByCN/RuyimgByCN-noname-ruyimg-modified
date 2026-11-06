//! PA12 Batch 14：`EffectRequest` 扩展编解码子表 2（tags 129..=153 + tag 255 u16 扩展头预留）。

use super::codec::{push_u32, Rd};
use super::*;

fn push_card_vec(w: &mut Vec<u8>, cards: &[rule_api::CardId]) {
    push_u32(w, cards.len() as u32);
    for c in cards {
        push_u32(w, c.raw());
    }
}

fn rd_card_vec(rd: &mut Rd<'_>) -> Option<Vec<rule_api::CardId>> {
    let n = rd.u32()? as usize;
    let mut out = Vec::with_capacity(n.min(256));
    for _ in 0..n {
        out.push(rule_api::CardId::new(rd.u32()?));
    }
    Some(out)
}

pub(super) fn enc_ext2_effect(w: &mut Vec<u8>, e: &EffectRequest) {
    match e {
        EffectRequest::ChooseOtherDrawOrRecover {
            player,
            draw_count,
            recover_count,
            times,
            skill,
        } => {
            w.push(129);
            push_u32(w, player.raw());
            push_u32(w, *draw_count);
            push_u32(w, *recover_count);
            push_u32(w, *times);
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseAddOrRemoveCurrentCardTarget {
            player,
            card,
            skill,
        } => {
            w.push(130);
            push_u32(w, player.raw());
            push_u32(w, card.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseCompareSelectedHandCard {
            player,
            target,
            card,
            win_skill_def,
            lose_skill_def,
            skill,
        } => {
            w.push(131);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, card.raw());
            push_u32(w, win_skill_def.raw());
            push_u32(w, lose_skill_def.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseTargetsDamage {
            player,
            min_count,
            max_count,
            scope,
            amount,
            skill,
        } => {
            w.push(132);
            push_u32(w, player.raw());
            w.push(*min_count);
            w.push(*max_count);
            w.push(scope.tag());
            push_u32(w, *amount);
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseAddCurrentShaTarget {
            player,
            card,
            skill,
        } => {
            w.push(133);
            push_u32(w, player.raw());
            push_u32(w, card.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::DrawThenGiveOne {
            player,
            target,
            count,
            drawn_only,
            use_given_equipment,
            skill,
            drawn,
            ready_to_give,
        } => {
            w.push(134);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, *count);
            w.push(u8::from(*drawn_only));
            w.push(u8::from(*use_given_equipment));
            push_u32(w, skill.raw());
            push_card_vec(w, drawn);
            w.push(u8::from(*ready_to_give));
        }
        EffectRequest::ChooseHandCompare {
            player,
            win_skill_def,
            lose_skill_def,
            skill,
        } => {
            w.push(135);
            push_u32(w, player.raw());
            push_u32(w, win_skill_def.raw());
            push_u32(w, lose_skill_def.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseOptionalTargetsDraw {
            player,
            count,
            min_count,
            max_count,
            scope,
            skill,
        } => {
            w.push(136);
            push_u32(w, player.raw());
            push_u32(w, *count);
            w.push(*min_count);
            w.push(*max_count);
            w.push(scope.tag());
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseTargetsDraw {
            player,
            count,
            target_count,
            scope,
            skill,
        } => {
            w.push(137);
            push_u32(w, player.raw());
            push_u32(w, *count);
            w.push(*target_count);
            w.push(scope.tag());
            push_u32(w, skill.raw());
        }
        EffectRequest::GuanxingTop {
            player,
            count,
            round,
            flipped,
            top,
            bottom,
            skill,
        } => {
            w.push(138);
            push_u32(w, player.raw());
            push_u32(w, *count);
            push_u32(w, *round);
            push_card_vec(w, flipped);
            push_card_vec(w, top);
            push_card_vec(w, bottom);
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseLinkOther { player, skill } => {
            w.push(139);
            push_u32(w, player.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseOtherDrawOwnerLostHpTurnOver { player, skill } => {
            w.push(140);
            push_u32(w, player.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseOneFromSkillPileDiscard {
            player,
            skill,
            pile_def,
        } => {
            w.push(141);
            push_u32(w, player.raw());
            push_u32(w, skill.raw());
            push_u32(w, *pile_def);
        }
        EffectRequest::ChooseAddCurrentShaTargetWithinDistance {
            player,
            card,
            max_distance,
            skill,
        } => {
            w.push(142);
            push_u32(w, player.raw());
            push_u32(w, card.raw());
            push_u32(w, *max_distance);
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseRecoverTarget {
            chooser,
            target,
            amount,
        } => {
            w.push(143);
            push_u32(w, chooser.raw());
            push_u32(w, target.raw());
            push_u32(w, *amount);
        }
        EffectRequest::ChooseTargetDrawToMaxHpCap { player, cap, skill } => {
            w.push(144);
            push_u32(w, player.raw());
            push_u32(w, *cap);
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseDiscardHandRange {
            player,
            min,
            max,
            skill,
        } => {
            w.push(145);
            push_u32(w, player.raw());
            push_u32(w, *min);
            push_u32(w, *max);
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseDiscardHandEquipThenRemoveSelfSkill {
            player,
            count,
            skill,
        } => {
            w.push(146);
            push_u32(w, player.raw());
            push_u32(w, *count);
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseHandCardsToSkillPile {
            player,
            skill,
            min_count,
            max_count,
        } => {
            w.push(147);
            push_u32(w, player.raw());
            push_u32(w, skill.raw());
            push_u32(w, *min_count);
            push_u32(w, *max_count);
        }
        EffectRequest::ChooseDiscardEquipmentPreventCurrentDamage {
            player,
            damage_id,
            skill,
        } => {
            w.push(148);
            push_u32(w, player.raw());
            push_u32(w, *damage_id);
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseGainDiscardCardByKind {
            player,
            kind,
            skill,
        } => {
            w.push(149);
            push_u32(w, player.raw());
            w.push(*kind);
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseAddMissingSkill {
            player,
            skill_defs,
            skill,
        } => {
            w.push(150);
            push_u32(w, player.raw());
            for d in skill_defs {
                push_u32(w, d.raw());
            }
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseMoveFieldCardTo {
            player,
            target,
            skill,
        } => {
            w.push(151);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::JudgeMakeCurrentShaTargetUnrespondableOnRed {
            player,
            card,
            target,
            retieji,
            skill,
            ready_to_choose,
            judgment_suit,
        } => {
            w.push(152);
            push_u32(w, player.raw());
            push_u32(w, card.raw());
            push_u32(w, target.raw());
            w.push(u8::from(*retieji));
            push_u32(w, skill.raw());
            w.push(u8::from(*ready_to_choose));
            w.push(*judgment_suit);
        }
        EffectRequest::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan {
            player,
            mark,
            skill,
        } => {
            w.push(153);
            push_u32(w, player.raw());
            push_u32(w, mark.raw());
            push_u32(w, skill.raw());
        }
        _ => unreachable!("accept_effect 已拒绝未编码变体"),
    }
}

pub(super) fn dec_ext2_effect(tag: u8, rd: &mut Rd<'_>) -> Option<EffectRequest> {
    Some(match tag {
        129 => EffectRequest::ChooseOtherDrawOrRecover {
            player: rule_api::PlayerId::new(rd.u32()?),
            draw_count: rd.u32()?,
            recover_count: rd.u32()?,
            times: rd.u32()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        130 => EffectRequest::ChooseAddOrRemoveCurrentCardTarget {
            player: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        131 => EffectRequest::ChooseCompareSelectedHandCard {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            win_skill_def: rule_api::SkillDefId::new(rd.u32()?),
            lose_skill_def: rule_api::SkillDefId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        132 => EffectRequest::ChooseTargetsDamage {
            player: rule_api::PlayerId::new(rd.u32()?),
            min_count: rd.u8()?,
            max_count: rd.u8()?,
            scope: rule_api::DrawTargetScope::from_tag(rd.u8()?)?,
            amount: rd.u32()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        133 => EffectRequest::ChooseAddCurrentShaTarget {
            player: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        134 => EffectRequest::DrawThenGiveOne {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            count: rd.u32()?,
            drawn_only: rd.u8()? != 0,
            use_given_equipment: rd.u8()? != 0,
            skill: rule_api::SkillId::new(rd.u32()?),
            drawn: rd_card_vec(rd)?,
            ready_to_give: rd.u8()? != 0,
        },
        135 => EffectRequest::ChooseHandCompare {
            player: rule_api::PlayerId::new(rd.u32()?),
            win_skill_def: rule_api::SkillDefId::new(rd.u32()?),
            lose_skill_def: rule_api::SkillDefId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        136 => EffectRequest::ChooseOptionalTargetsDraw {
            player: rule_api::PlayerId::new(rd.u32()?),
            count: rd.u32()?,
            min_count: rd.u8()?,
            max_count: rd.u8()?,
            scope: rule_api::DrawTargetScope::from_tag(rd.u8()?)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        137 => EffectRequest::ChooseTargetsDraw {
            player: rule_api::PlayerId::new(rd.u32()?),
            count: rd.u32()?,
            target_count: rd.u8()?,
            scope: rule_api::DrawTargetScope::from_tag(rd.u8()?)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        138 => EffectRequest::GuanxingTop {
            player: rule_api::PlayerId::new(rd.u32()?),
            count: rd.u32()?,
            round: rd.u32()?,
            flipped: rd_card_vec(rd)?,
            top: rd_card_vec(rd)?,
            bottom: rd_card_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        139 => EffectRequest::ChooseLinkOther {
            player: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        140 => EffectRequest::ChooseOtherDrawOwnerLostHpTurnOver {
            player: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        141 => EffectRequest::ChooseOneFromSkillPileDiscard {
            player: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
            pile_def: rd.u32()?,
        },
        142 => EffectRequest::ChooseAddCurrentShaTargetWithinDistance {
            player: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            max_distance: rd.u32()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        143 => EffectRequest::ChooseRecoverTarget {
            chooser: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            amount: rd.u32()?,
        },
        144 => EffectRequest::ChooseTargetDrawToMaxHpCap {
            player: rule_api::PlayerId::new(rd.u32()?),
            cap: rd.u32()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        145 => EffectRequest::ChooseDiscardHandRange {
            player: rule_api::PlayerId::new(rd.u32()?),
            min: rd.u32()?,
            max: rd.u32()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        146 => EffectRequest::ChooseDiscardHandEquipThenRemoveSelfSkill {
            player: rule_api::PlayerId::new(rd.u32()?),
            count: rd.u32()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        147 => EffectRequest::ChooseHandCardsToSkillPile {
            player: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
            min_count: rd.u32()?,
            max_count: rd.u32()?,
        },
        148 => EffectRequest::ChooseDiscardEquipmentPreventCurrentDamage {
            player: rule_api::PlayerId::new(rd.u32()?),
            damage_id: rd.u32()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        149 => EffectRequest::ChooseGainDiscardCardByKind {
            player: rule_api::PlayerId::new(rd.u32()?),
            kind: rd.u8()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        150 => EffectRequest::ChooseAddMissingSkill {
            player: rule_api::PlayerId::new(rd.u32()?),
            skill_defs: [
                rule_api::SkillDefId::new(rd.u32()?),
                rule_api::SkillDefId::new(rd.u32()?),
                rule_api::SkillDefId::new(rd.u32()?),
            ],
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        151 => EffectRequest::ChooseMoveFieldCardTo {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        152 => EffectRequest::JudgeMakeCurrentShaTargetUnrespondableOnRed {
            player: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            retieji: rd.u8()? != 0,
            skill: rule_api::SkillId::new(rd.u32()?),
            ready_to_choose: rd.u8()? != 0,
            judgment_suit: rd.u8()?,
        },
        153 => EffectRequest::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan {
            player: rule_api::PlayerId::new(rd.u32()?),
            mark: rule_api::MarkDefId::new(rd.u32()?),
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
