//! PA12 Batch 17：`EffectRequest` 扩展编解码子表 5 续（tags 239..=254 + tag 255 ext_tag 256..=269）。

use super::super::super::codec::{push_u32, Rd};
use super::super::super::codec_ext2::{push_card_vec, push_player_vec, rd_card_vec, rd_player_vec};
use super::super::super::*;

fn push_ext_tag(w: &mut Vec<u8>, ext_tag: u16) {
    w.push(255);
    w.extend_from_slice(&ext_tag.to_le_bytes());
}

fn enc_suit(s: rule_api::Suit) -> u8 {
    match s {
        rule_api::Suit::Spade => 1,
        rule_api::Suit::Heart => 2,
        rule_api::Suit::Club => 3,
        rule_api::Suit::Diamond => 4,
        rule_api::Suit::None => 0,
    }
}

fn dec_suit(tag: u8) -> Option<rule_api::Suit> {
    Some(match tag {
        0 => rule_api::Suit::None,
        1 => rule_api::Suit::Spade,
        2 => rule_api::Suit::Heart,
        3 => rule_api::Suit::Club,
        4 => rule_api::Suit::Diamond,
        _ => return None,
    })
}

pub(super) fn enc_batch17_rest(w: &mut Vec<u8>, e: &EffectRequest) {
    match e {
        EffectRequest::ResolveHoufeng {
            player,
            target,
            mode,
            stage,
            skill,
        } => {
            w.push(239);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            w.push(*mode);
            w.push(*stage);
            push_u32(w, skill.raw());
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
            w.push(240);
            push_u32(w, player.raw());
            push_u32(w, source.raw());
            push_u32(w, card.raw());
            w.push(*action);
            push_card_vec(w, cards);
            push_player_vec(w, targets);
            push_u32(w, skill.raw());
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
            w.push(241);
            push_u32(w, player.raw());
            push_u32(w, card.raw());
            push_u32(w, helper_skill_def.raw());
            w.push(*action);
            push_u32(w, source.raw());
            push_u32(w, target.raw());
            w.push(*nature);
            push_u32(w, *damage_id);
            push_card_vec(w, event_cards);
            push_player_vec(w, event_targets);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveZhugemengxue {
            player,
            target,
            card,
            action,
            cards,
            skill,
        } => {
            w.push(242);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, card.raw());
            w.push(*action);
            push_card_vec(w, cards);
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveTwyuyan {
            player,
            source,
            card,
            skill,
        } => {
            w.push(243);
            push_u32(w, player.raw());
            push_u32(w, source.raw());
            push_u32(w, card.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ResolveTwzhibian {
            player,
            target,
            winner,
            stage,
            skill,
        } => {
            w.push(244);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, winner.raw());
            w.push(*stage);
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseAddSkillBoundCardTarget {
            player,
            card,
            skill,
        } => {
            w.push(245);
            push_u32(w, player.raw());
            push_u32(w, card.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseDiscardSkillPileCardAddMarkedSkill {
            player,
            target,
            helper_skill_def,
            selected,
            skill,
        } => {
            w.push(246);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, helper_skill_def.raw());
            push_u32(w, selected.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseDiscardTargetCardResolveCurrentSha {
            source,
            card,
            target,
        } => {
            w.push(247);
            push_u32(w, source.raw());
            push_u32(w, card.raw());
            push_u32(w, target.raw());
        }
        EffectRequest::ChooseDiscardTargetCardResolveCurrentShaNonbasicBonus {
            source,
            card,
            target,
            bonus_skill_def,
        } => {
            w.push(248);
            push_u32(w, source.raw());
            push_u32(w, card.raw());
            push_u32(w, target.raw());
            push_u32(w, bonus_skill_def.raw());
        }
        EffectRequest::ChooseDiscardTargetCardsRecoverSameCategory {
            player,
            target,
            count,
            amount,
            skill,
        } => {
            w.push(249);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, *count);
            push_u32(w, *amount);
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseGainCardFromEachOtherReplaceDraw {
            player,
            initialized,
            remaining_targets,
            skill,
        } => {
            w.push(250);
            push_u32(w, player.raw());
            w.push(u8::from(*initialized));
            push_player_vec(w, remaining_targets);
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseGainCardFromEachOtherThenTurnOver {
            player,
            initialized,
            remaining_targets,
            skill,
        } => {
            w.push(251);
            push_u32(w, player.raw());
            w.push(u8::from(*initialized));
            push_player_vec(w, remaining_targets);
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseGainOwnLostDiscardCardThisTurn { player, skill } => {
            w.push(252);
            push_u32(w, player.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseGainTargetCardAlternateZoneThenTargetDraw {
            player,
            target,
            selected,
            skill,
        } => {
            w.push(253);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, selected.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseGainTopCardsBySuit {
            player,
            count,
            suit,
        } => {
            w.push(254);
            push_u32(w, player.raw());
            w.push(*count);
            w.push(enc_suit(*suit));
        }
        EffectRequest::ChooseRevealHandCardBindCategoryTarget {
            player,
            target,
            skill,
        } => {
            push_ext_tag(w, 256);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseRevealOtherHandCardRememberCategory { player, skill } => {
            push_ext_tag(w, 257);
            push_u32(w, player.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ChooseTargetHandCardToPile {
            player,
            skill,
            pile_def,
        } => {
            push_ext_tag(w, 258);
            push_u32(w, player.raw());
            push_u32(w, skill.raw());
            push_u32(w, *pile_def);
        }
        EffectRequest::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver {
            player,
            priority,
            remaining_targets,
            skill,
        } => {
            push_ext_tag(w, 259);
            push_u32(w, player.raw());
            w.push(*priority);
            push_player_vec(w, remaining_targets);
            push_u32(w, skill.raw());
        }
        EffectRequest::CityConsume { player, skill } => {
            push_ext_tag(w, 260);
            push_u32(w, player.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::DiscardEquipmentPairWithArmor {
            player,
            targets,
            skill,
        } => {
            push_ext_tag(w, 261);
            push_u32(w, player.raw());
            push_player_vec(w, targets);
            push_u32(w, skill.raw());
        }
        EffectRequest::DiscardOneOwnXumou { player, skill } => {
            push_ext_tag(w, 262);
            push_u32(w, player.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::DiscardRememberedHandCards { player, skill } => {
            push_ext_tag(w, 263);
            push_u32(w, player.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::DiscardTargetEquipmentDrawWeaponRange {
            player,
            target,
            skill,
        } => {
            push_ext_tag(w, 264);
            push_u32(w, player.raw());
            push_u32(w, target.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::ExtraResolveCurrentTrick {
            player,
            card,
            skill,
        } => {
            push_ext_tag(w, 265);
            push_u32(w, player.raw());
            push_u32(w, card.raw());
            push_u32(w, skill.raw());
        }
        EffectRequest::GainDrawCardsForLifetimeCardActionDivisors {
            player,
            divisors,
            groups,
        } => {
            push_ext_tag(w, 266);
            push_u32(w, player.raw());
            for d in divisors {
                push_u32(w, *d);
            }
            for g in groups {
                w.push(*g);
            }
        }
        EffectRequest::GainHandCompareResultCard {
            player,
            initiator,
            target,
            winner,
            cards,
            skill,
        } => {
            push_ext_tag(w, 267);
            push_u32(w, player.raw());
            push_u32(w, initiator.raw());
            push_u32(w, target.raw());
            push_u32(w, winner.raw());
            push_card_vec(w, cards);
            push_u32(w, skill.raw());
        }
        EffectRequest::GainHighestRankShaFromEventCards {
            player,
            cards,
            skill,
        } => {
            push_ext_tag(w, 268);
            push_u32(w, player.raw());
            push_card_vec(w, cards);
            push_u32(w, skill.raw());
        }
        EffectRequest::MoveEquipmentThenDraw { player, skill } => {
            push_ext_tag(w, 269);
            push_u32(w, player.raw());
            push_u32(w, skill.raw());
        }
        _ => unreachable!("accept_effect 已拒绝未编码变体"),
    }
}

pub(super) fn dec_batch17_rest(tag: u8, rd: &mut Rd<'_>) -> Option<EffectRequest> {
    Some(match tag {
        239 => EffectRequest::ResolveHoufeng {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            mode: rd.u8()?,
            stage: rd.u8()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        240 => EffectRequest::ResolveHuzun {
            player: rule_api::PlayerId::new(rd.u32()?),
            source: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            action: rd.u8()?,
            cards: rd_card_vec(rd)?,
            targets: rd_player_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        241 => EffectRequest::ResolveMalingli {
            player: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            helper_skill_def: rule_api::SkillDefId::new(rd.u32()?),
            action: rd.u8()?,
            source: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            nature: rd.u8()?,
            damage_id: rd.u32()?,
            event_cards: rd_card_vec(rd)?,
            event_targets: rd_player_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        242 => EffectRequest::ResolveZhugemengxue {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            action: rd.u8()?,
            cards: rd_card_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        243 => EffectRequest::ResolveTwyuyan {
            player: rule_api::PlayerId::new(rd.u32()?),
            source: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        244 => EffectRequest::ResolveTwzhibian {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            winner: rule_api::PlayerId::new(rd.u32()?),
            stage: rd.u8()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        245 => EffectRequest::ChooseAddSkillBoundCardTarget {
            player: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        246 => EffectRequest::ChooseDiscardSkillPileCardAddMarkedSkill {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            helper_skill_def: rule_api::SkillDefId::new(rd.u32()?),
            selected: rule_api::CardId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        247 => EffectRequest::ChooseDiscardTargetCardResolveCurrentSha {
            source: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
        },
        248 => EffectRequest::ChooseDiscardTargetCardResolveCurrentShaNonbasicBonus {
            source: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            bonus_skill_def: rule_api::SkillDefId::new(rd.u32()?),
        },
        249 => EffectRequest::ChooseDiscardTargetCardsRecoverSameCategory {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            count: rd.u32()?,
            amount: rd.u32()?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        250 => EffectRequest::ChooseGainCardFromEachOtherReplaceDraw {
            player: rule_api::PlayerId::new(rd.u32()?),
            initialized: rd.u8()? != 0,
            remaining_targets: rd_player_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        251 => EffectRequest::ChooseGainCardFromEachOtherThenTurnOver {
            player: rule_api::PlayerId::new(rd.u32()?),
            initialized: rd.u8()? != 0,
            remaining_targets: rd_player_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        252 => EffectRequest::ChooseGainOwnLostDiscardCardThisTurn {
            player: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        253 => EffectRequest::ChooseGainTargetCardAlternateZoneThenTargetDraw {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            selected: rule_api::CardId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        254 => EffectRequest::ChooseGainTopCardsBySuit {
            player: rule_api::PlayerId::new(rd.u32()?),
            count: rd.u8()?,
            suit: dec_suit(rd.u8()?)?,
        },
        _ => return None,
    })
}

pub(super) fn dec_ext_u16(ext_tag: u16, rd: &mut Rd<'_>) -> Option<EffectRequest> {
    Some(match ext_tag {
        256 => EffectRequest::ChooseRevealHandCardBindCategoryTarget {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        257 => EffectRequest::ChooseRevealOtherHandCardRememberCategory {
            player: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        258 => EffectRequest::ChooseTargetHandCardToPile {
            player: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
            pile_def: rd.u32()?,
        },
        259 => EffectRequest::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver {
            player: rule_api::PlayerId::new(rd.u32()?),
            priority: rd.u8()?,
            remaining_targets: rd_player_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        260 => EffectRequest::CityConsume {
            player: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        261 => EffectRequest::DiscardEquipmentPairWithArmor {
            player: rule_api::PlayerId::new(rd.u32()?),
            targets: rd_player_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        262 => EffectRequest::DiscardOneOwnXumou {
            player: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        263 => EffectRequest::DiscardRememberedHandCards {
            player: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        264 => EffectRequest::DiscardTargetEquipmentDrawWeaponRange {
            player: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        265 => EffectRequest::ExtraResolveCurrentTrick {
            player: rule_api::PlayerId::new(rd.u32()?),
            card: rule_api::CardId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        266 => EffectRequest::GainDrawCardsForLifetimeCardActionDivisors {
            player: rule_api::PlayerId::new(rd.u32()?),
            divisors: [rd.u32()?, rd.u32()?, rd.u32()?],
            groups: [rd.u8()?, rd.u8()?, rd.u8()?],
        },
        267 => EffectRequest::GainHandCompareResultCard {
            player: rule_api::PlayerId::new(rd.u32()?),
            initiator: rule_api::PlayerId::new(rd.u32()?),
            target: rule_api::PlayerId::new(rd.u32()?),
            winner: rule_api::PlayerId::new(rd.u32()?),
            cards: rd_card_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        268 => EffectRequest::GainHighestRankShaFromEventCards {
            player: rule_api::PlayerId::new(rd.u32()?),
            cards: rd_card_vec(rd)?,
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        269 => EffectRequest::MoveEquipmentThenDraw {
            player: rule_api::PlayerId::new(rd.u32()?),
            skill: rule_api::SkillId::new(rd.u32()?),
        },
        _ => return None,
    })
}
