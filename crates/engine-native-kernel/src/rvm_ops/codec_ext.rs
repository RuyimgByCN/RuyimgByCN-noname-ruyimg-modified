//! PA12 批⑪–⑫ 扩展效果请求的字节编解码（tags 71..=108，满足 1,000 有效行门禁）。

use super::codec::{push_u32, Rd};
use super::*;

pub(super) fn enc_ext_effect(w: &mut Vec<u8>, e: &EffectRequest) {
    match e {
        // PA12 批⑪：13 种手牌调整/底牌摸/目标排除/取牌/禁用抑制确定性效果编码（71..=83）。
        EffectRequest::AdjustHandTo { player, count } => {
            w.push(71);
            push_u32(w, player.raw());
            push_u32(w, *count);
        }
        EffectRequest::DrawByPlayPhaseCardCategoryCount { player, skill } => {
            w.push(72);
            push_u32(w, player.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::DrawEachBySeat {
            first,
            second,
            count,
            skill,
        } => {
            w.push(73);
            push_u32(w, first.raw());
            push_u32(w, second.raw());
            push_u32(w, *count);
            push_u32(w, skill.raw());
        }
        EffectRequest::SetDrawPhaseFromBottom { player } => {
            w.push(74);
            push_u32(w, player.raw());
        }
        EffectRequest::DrawCardsFromBottom { player, count } => {
            w.push(75);
            push_u32(w, player.raw());
            push_u32(w, *count);
        }
        EffectRequest::CancelIncomingCardUse {
            player,
            source,
            card,
            skill,
        } => {
            w.push(76);
            push_u32(w, player.raw());
            push_u32(w, source.raw());
            push_u32(w, card.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ExcludeSavedShaTarget { card, target } => {
            w.push(77);
            push_u32(w, card.raw());
            push_u32(w, target.raw());
        }
        EffectRequest::TakeOneCard {
            from,
            to,
            equip_only,
        } => {
            w.push(78);
            push_u32(w, from.raw());
            push_u32(w, to.raw());
            w.push(u8::from(*equip_only));
        }
        EffectRequest::TakeOneHandCard { from, to } => {
            w.push(79);
            push_u32(w, from.raw());
            push_u32(w, to.raw());
        }
        EffectRequest::DisableSkillByDefUntilTurnEnd { player, skill_def } => {
            w.push(80);
            push_u32(w, player.raw());
            push_u32(w, skill_def.raw());
        }
        EffectRequest::BanActiveUntilTurnEnd { player, skill } => {
            w.push(81);
            push_u32(w, player.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::SetNonLockedSkillSuppression {
            player,
            enabled,
            skill,
        } => {
            w.push(82);
            push_u32(w, player.raw());
            w.push(u8::from(*enabled));
            push_u32(w, skill.raw());
        }
        EffectRequest::SuppressNonLockedSkillsUntilTurnEnd { player } => {
            w.push(83);
            push_u32(w, player.raw());
        }
        // PA12 批⑫：25 种选定集/确定性高频效果编码（84..=108）。
        EffectRequest::DiscardCards { player, cards } => {
            w.push(84);
            push_u32(w, player.raw());
            push_u32(w, cards.len() as u32);
            for c in cards {
                push_u32(w, c.raw());
            }
        }
        EffectRequest::StoreSelectedInSkillPile {
            player,
            cards,
            skill,
        } => {
            w.push(85);
            push_u32(w, player.raw());
            push_u32(w, cards.len() as u32);
            for c in cards {
                push_u32(w, c.raw());
            }
            push_u32(w, skill.raw());
        }
        EffectRequest::UseVirtualCardOn {
            player,
            kind,
            target,
        } => {
            w.push(86);
            push_u32(w, player.raw());
            w.push(*kind);
            push_u32(w, target.raw());
        }
        EffectRequest::RecastCards { player, cards } => {
            w.push(87);
            push_u32(w, player.raw());
            push_u32(w, cards.len() as u32);
            for c in cards {
                push_u32(w, c.raw());
            }
        }
        EffectRequest::AddCardUsePolicySkill {
            player,
            card,
            policy_mask,
            skill_def,
        } => {
            w.push(88);
            push_u32(w, player.raw());
            push_u32(w, card.raw());
            w.push(*policy_mask);
            push_u32(w, skill_def.raw());
        }
        EffectRequest::DrawSelectedTargets {
            targets,
            count,
            skill,
        } => {
            w.push(89);
            push_u32(w, targets.len() as u32);
            for t in targets {
                push_u32(w, t.raw());
            }
            push_u32(w, *count);
            push_u32(w, skill.raw());
        }
        EffectRequest::QueueExtraTurn {
            player,
            check,
            skill,
        } => {
            w.push(90);
            push_u32(w, player.raw());
            push_u32(w, *check);
            push_u32(w, skill.raw());
        }
        EffectRequest::DisableEquipSlots { player, slots } => {
            w.push(91);
            push_u32(w, player.raw());
            push_u32(w, slots.len() as u32);
            for s in slots {
                w.push(*s);
            }
        }
        EffectRequest::EnableEquipSlot {
            player,
            slot,
            source,
        } => {
            w.push(92);
            push_u32(w, player.raw());
            w.push(*slot);
            push_u32(w, source.raw());
        }
        EffectRequest::GainShadowCards {
            player,
            count,
            skill,
        } => {
            w.push(93);
            push_u32(w, player.raw());
            push_u32(w, *count);
            push_u32(w, skill.raw());
        }
        EffectRequest::GainTargetHandCard { player, skill } => {
            w.push(94);
            push_u32(w, player.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::AddBoundSkill {
            player,
            skill_def,
            bound_target,
        } => {
            w.push(95);
            push_u32(w, player.raw());
            push_u32(w, skill_def.raw());
            push_u32(w, bound_target.raw());
        }
        EffectRequest::GiveSelectedThenRecoverTarget {
            player,
            target,
            cards,
            amount,
            skill,
            ready_to_recover,
        } => {
            w.push(96);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, cards.len() as u32);
            for c in cards {
                push_u32(w, c.raw());
            }
            push_u32(w, *amount);
            push_u32(w, skill.raw());
            w.push(u8::from(*ready_to_recover));
        }
        EffectRequest::DistributeShadowCards {
            player,
            count,
            skill,
        } => {
            w.push(97);
            push_u32(w, player.raw());
            push_u32(w, *count);
            push_u32(w, skill.raw());
        }
        EffectRequest::PlaceHandCardToExpansion { player, skill } => {
            w.push(98);
            push_u32(w, player.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::RecordActiveTarget {
            player,
            target,
            skill,
        } => {
            w.push(99);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ClearPhaseGiftTarget { player, skill } => {
            w.push(100);
            push_u32(w, player.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::DealDamageBanActiveIfDying {
            source,
            target,
            amount,
            skill,
        } => {
            w.push(101);
            push_u32(w, source.raw());
            push_u32(w, target.raw());
            push_u32(w, *amount);
            push_u32(w, skill.raw());
        }
        EffectRequest::DiscardSelectedDrawAllHandBonus {
            player,
            cards,
            skill,
        } => {
            w.push(102);
            push_u32(w, player.raw());
            push_u32(w, cards.len() as u32);
            for c in cards {
                push_u32(w, c.raw());
            }
            push_u32(w, skill.raw());
        }
        EffectRequest::DiscardSelectedRecordUniqueCategoryTarget {
            player,
            target,
            card,
            skill,
        } => {
            w.push(103);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, card.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::DiscardTargetCardGainMount {
            player,
            target,
            scope,
        } => {
            w.push(104);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            w.push(scope.tag());
        }
        EffectRequest::DrawForLifetimeCardActionDivisors { player, divisors } => {
            w.push(105);
            push_u32(w, player.raw());
            for d in divisors {
                push_u32(w, *d);
            }
        }
        EffectRequest::RecoverSelectedTargets {
            source,
            targets,
            amount,
            skill,
        } => {
            w.push(106);
            push_u32(w, source.raw());
            push_u32(w, targets.len() as u32);
            for t in targets {
                push_u32(w, t.raw());
            }
            push_u32(w, *amount);
            push_u32(w, skill.raw());
        }
        EffectRequest::RevealTopGainIfColor { player, color_mask } => {
            w.push(107);
            push_u32(w, player.raw());
            w.push(*color_mask);
        }
        EffectRequest::UseVirtualCardOnSelectedTargets {
            player,
            kind,
            targets,
        } => {
            w.push(108);
            push_u32(w, player.raw());
            w.push(*kind);
            push_u32(w, targets.len() as u32);
            for t in targets {
                push_u32(w, t.raw());
            }
        }
        _ => unreachable!("accept_effect 已拒绝未编码变体"),
    }
}

pub(super) fn dec_ext_effect(tag: u8, rd: &mut Rd<'_>) -> Option<EffectRequest> {
    Some(match tag {
        71 => EffectRequest::AdjustHandTo {
            player: rule_api::PlayerId::new(rd.u32()?),
            count: rd.u32()?,
        },
        72 => EffectRequest::DrawByPlayPhaseCardCategoryCount {
            player: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        73 => EffectRequest::DrawEachBySeat {
            first: rule_api::PlayerId::new(rd.u32()?),
            second: rule_api::PlayerId::new(rd.u32()?),
            count: rd.u32()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        74 => EffectRequest::SetDrawPhaseFromBottom {
            player: rule_api::PlayerId::new(rd.u32()?),
        },
        75 => EffectRequest::DrawCardsFromBottom {
            player: rule_api::PlayerId::new(rd.u32()?),
            count: rd.u32()?,
        },
        76 => EffectRequest::CancelIncomingCardUse {
            player: rule_api::PlayerId::new(rd.u32()?),
            source: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        77 => EffectRequest::ExcludeSavedShaTarget {
            card: rule_api::CardId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
        },
        78 => EffectRequest::TakeOneCard {
            from: rule_api::PlayerId::new(rd.u32()?),
            to: rule_api::PlayerId::new(rd.u32()?),
            equip_only: rd.u8()? != 0,
        },
        79 => EffectRequest::TakeOneHandCard {
            from: rule_api::PlayerId::new(rd.u32()?),
            to: rule_api::PlayerId::new(rd.u32()?),
        },
        80 => EffectRequest::DisableSkillByDefUntilTurnEnd {
            player: rule_api::PlayerId::new(rd.u32()?),
            skill_def: rule_api::SkillDefId::new(rd.u32()?),
        },
        81 => EffectRequest::BanActiveUntilTurnEnd {
            player: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        82 => EffectRequest::SetNonLockedSkillSuppression {
            player: rule_api::PlayerId::new(rd.u32()?),
            enabled: rd.u8()? != 0,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        83 => EffectRequest::SuppressNonLockedSkillsUntilTurnEnd {
            player: rule_api::PlayerId::new(rd.u32()?),
        },
        84 => {
            let player = rule_api::PlayerId::new(rd.u32()?);
            let m = rd.u32()? as usize;
            let mut cards = Vec::with_capacity(m);
            for _ in 0..m {
                cards.push(rule_api::CardId::new(rd.u32()?));
            }
            EffectRequest::DiscardCards { player, cards }
        }
        85 => {
            let player = rule_api::PlayerId::new(rd.u32()?);
            let m = rd.u32()? as usize;
            let mut cards = Vec::with_capacity(m);
            for _ in 0..m {
                cards.push(rule_api::CardId::new(rd.u32()?));
            }
            let skill = rule_api::SkillId::new(rd.u32()?);
            EffectRequest::StoreSelectedInSkillPile {
                player,
                cards,
                skill,
            }
        }
        86 => EffectRequest::UseVirtualCardOn {
            player: rule_api::PlayerId::new(rd.u32()?),
            kind: rd.u8()?,
            target: rule_api::PlayerId::new(rd.u32()?),
        },
        87 => {
            let player = rule_api::PlayerId::new(rd.u32()?);
            let m = rd.u32()? as usize;
            let mut cards = Vec::with_capacity(m);
            for _ in 0..m {
                cards.push(rule_api::CardId::new(rd.u32()?));
            }
            EffectRequest::RecastCards { player, cards }
        }
        88 => EffectRequest::AddCardUsePolicySkill {
            player: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            policy_mask: rd.u8()?,
            skill_def: rule_api::SkillDefId::new(rd.u32()?),
        },
        89 => {
            let m = rd.u32()? as usize;
            let mut targets = Vec::with_capacity(m);
            for _ in 0..m {
                targets.push(rule_api::PlayerId::new(rd.u32()?));
            }
            let count = rd.u32()?;
            let skill = rule_api::SkillId::new(rd.u32()?);
            EffectRequest::DrawSelectedTargets {
                targets,
                count,
                skill,
            }
        }
        90 => EffectRequest::QueueExtraTurn {
            player: rule_api::PlayerId::new(rd.u32()?),
            check: rd.u32()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        91 => {
            let player = rule_api::PlayerId::new(rd.u32()?);
            let m = rd.u32()? as usize;
            let mut slots = Vec::with_capacity(m);
            for _ in 0..m {
                slots.push(rd.u8()?);
            }
            EffectRequest::DisableEquipSlots { player, slots }
        }
        92 => EffectRequest::EnableEquipSlot {
            player: rule_api::PlayerId::new(rd.u32()?),
            slot: rd.u8()?,
            source: rule_api::PlayerId::new(rd.u32()?),
        },
        93 => EffectRequest::GainShadowCards {
            player: rule_api::PlayerId::new(rd.u32()?),
            count: rd.u32()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        94 => EffectRequest::GainTargetHandCard {
            player: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        95 => EffectRequest::AddBoundSkill {
            player: rule_api::PlayerId::new(rd.u32()?),
            skill_def: rule_api::SkillDefId::new(rd.u32()?),
            bound_target: rule_api::PlayerId::new(rd.u32()?),
        },
        96 => {
            let player = rule_api::PlayerId::new(rd.u32()?);
            let target = rule_api::PlayerId::new(rd.u32()?);
            let m = rd.u32()? as usize;
            let mut cards = Vec::with_capacity(m);
            for _ in 0..m {
                cards.push(rule_api::CardId::new(rd.u32()?));
            }
            let amount = rd.u32()?;
            let skill = rule_api::SkillId::new(rd.u32()?);
            let ready_to_recover = rd.u8()? != 0;
            EffectRequest::GiveSelectedThenRecoverTarget {
                player,
                target,
                cards,
                amount,
                skill,
                ready_to_recover,
            }
        }
        97 => EffectRequest::DistributeShadowCards {
            player: rule_api::PlayerId::new(rd.u32()?),
            count: rd.u32()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        98 => EffectRequest::PlaceHandCardToExpansion {
            player: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        99 => EffectRequest::RecordActiveTarget {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        100 => EffectRequest::ClearPhaseGiftTarget {
            player: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        101 => EffectRequest::DealDamageBanActiveIfDying {
            source: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            amount: rd.u32()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        102 => {
            let player = rule_api::PlayerId::new(rd.u32()?);
            let m = rd.u32()? as usize;
            let mut cards = Vec::with_capacity(m);
            for _ in 0..m {
                cards.push(rule_api::CardId::new(rd.u32()?));
            }
            let skill = rule_api::SkillId::new(rd.u32()?);
            EffectRequest::DiscardSelectedDrawAllHandBonus {
                player,
                cards,
                skill,
            }
        }
        103 => EffectRequest::DiscardSelectedRecordUniqueCategoryTarget {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        104 => EffectRequest::DiscardTargetCardGainMount {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            scope: rule_api::TargetCardScope::from_tag(rd.u8()?)?,
        },
        105 => EffectRequest::DrawForLifetimeCardActionDivisors {
            player: rule_api::PlayerId::new(rd.u32()?),
            divisors: [rd.u32()?, rd.u32()?, rd.u32()?],
        },
        106 => {
            let source = rule_api::PlayerId::new(rd.u32()?);
            let m = rd.u32()? as usize;
            let mut targets = Vec::with_capacity(m);
            for _ in 0..m {
                targets.push(rule_api::PlayerId::new(rd.u32()?));
            }
            let amount = rd.u32()?;
            let skill = rule_api::SkillId::new(rd.u32()?);
            EffectRequest::RecoverSelectedTargets {
                source,
                targets,
                amount,
                skill,
            }
        }
        107 => EffectRequest::RevealTopGainIfColor {
            player: rule_api::PlayerId::new(rd.u32()?),
            color_mask: rd.u8()?,
        },
        108 => {
            let player = rule_api::PlayerId::new(rd.u32()?);
            let kind = rd.u8()?;
            let m = rd.u32()? as usize;
            let mut targets = Vec::with_capacity(m);
            for _ in 0..m {
                targets.push(rule_api::PlayerId::new(rd.u32()?));
            }
            EffectRequest::UseVirtualCardOnSelectedTargets {
                player,
                kind,
                targets,
            }
        }
        _ => return None,
    })
}
