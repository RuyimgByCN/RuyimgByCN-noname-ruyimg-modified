//! 效果请求的规范哈希编码：具名技能结算器（Resolve*）。

use super::*;

pub(super) fn hash_effect(h: &mut blake3::Hasher, effect: &EffectRequest) {
    match effect {
        EffectRequest::ResolveMassDamageDiscardTurnOver {
            player,
            mark,
            mark_count,
            damage_amount,
            hand_count,
            stage,
            index,
            targets,
            skill,
        } => {
            put_u8(h, 242);
            put_u32(h, player.raw());
            put_u32(h, mark.raw());
            put_u32(h, *mark_count);
            put_u32(h, *damage_amount);
            put_u32(h, *hand_count);
            put_u8(h, *stage);
            put_u32(h, *index);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMassDamageDiscardHandEquip {
            player,
            mark,
            mark_count,
            damage_amount,
            discard_count,
            stage,
            index,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&270u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, mark.raw());
            put_u32(h, *mark_count);
            put_u32(h, *damage_amount);
            put_u32(h, *discard_count);
            put_u8(h, *stage);
            put_u32(h, *index);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMarkParityControl {
            player,
            mark,
            mass_threshold,
            damage_amount,
            stage,
            index,
            target,
            targets,
            skill,
        } => {
            put_u8(h, 244);
            put_u32(h, player.raw());
            put_u32(h, mark.raw());
            put_u32(h, *mass_threshold);
            put_u32(h, *damage_amount);
            put_u8(h, *stage);
            put_u32(h, *index);
            put_u32(h, target.raw());
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveSelectedEquipmentDiscardThenChooseDamage {
            player,
            damage_amount,
            nature,
            stage,
            index,
            targets,
            skill,
        } => {
            put_u8(h, 245);
            put_u32(h, player.raw());
            put_u32(h, *damage_amount);
            put_u8(h, *nature);
            put_u8(h, *stage);
            put_u32(h, *index);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveYeyan {
            player,
            mode,
            stage,
            index,
            targets,
            data,
            skill,
        } => {
            put_u8(h, 248);
            put_u32(h, player.raw());
            put_u8(h, *mode);
            put_u8(h, *stage);
            put_u32(h, *index);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, *data);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveQixing {
            player,
            initialize,
            stage,
            hand_cards,
            pile_cards,
            skill,
        } => {
            put_u8(h, 250);
            put_u32(h, player.raw());
            put_u8(h, u8::from(*initialize));
            put_u8(h, *stage);
            put_ids(h, hand_cards.iter().map(|card| card.raw()));
            put_ids(h, pile_cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveQixingWeather {
            player,
            qixing_skill_def,
            helper_skill_def,
            max_targets,
            stage,
            cards,
            targets,
            skill,
        } => {
            put_u8(h, 251);
            put_u32(h, player.raw());
            put_u32(h, qixing_skill_def.raw());
            put_u32(h, helper_skill_def.raw());
            put_u8(h, *max_targets);
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveNzryLongnu {
            player,
            fire_skill_def,
            thunder_skill_def,
            stage,
            skill,
        } => {
            put_u8(h, 252);
            put_u32(h, player.raw());
            put_u32(h, fire_skill_def.raw());
            put_u32(h, thunder_skill_def.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDrltPoxi {
            player,
            target,
            hand_limit_skill_def,
            stage,
            own_count,
            skill,
        } => {
            put_u8(h, 254);
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, hand_limit_skill_def.raw());
            put_u8(h, *stage);
            put_u8(h, *own_count);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDrltJieying {
            player,
            event_player,
            mark,
            mode,
            skill,
        } => {
            put_u8(h, 255);
            put_u32(h, player.raw());
            put_u32(h, event_player.raw());
            put_u32(h, mark.raw());
            put_u8(h, *mode);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDrltDuorui {
            player,
            target,
            helper_skill_def,
            stage,
            selected_skill_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&256u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, helper_skill_def.raw());
            put_u8(h, *stage);
            put_u32(h, selected_skill_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveDrltZhiti {
            player,
            event_player,
            event_source,
            event_amount,
            mode,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&257u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, event_player.raw());
            put_u32(h, event_source.raw());
            put_u32(h, *event_amount);
            put_u8(h, *mode);
            put_u32(h, skill.raw());
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
            put_u8(h, 0);
            h.update(&259u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, event_player.raw());
            put_u32(h, *event_amount);
            put_u32(h, mark.raw());
            put_u8(h, *mode);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveBeige {
            player,
            target,
            source,
            stage,
            result_card,
            suit,
            rank,
            rewriters,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&260u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, source.raw());
            put_u8(h, *stage);
            put_u32(h, result_card.raw());
            put_u8(h, crate::command::judge::suit_tag(*suit) as u8);
            put_u8(h, *rank);
            put_ids(h, rewriters.iter().map(|player| player.raw()));
            put_u32(h, skill.raw());
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
            put_u8(h, 0);
            h.update(&272u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, pile_skill_def.raw());
            put_u8(h, *action);
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_ids(h, hand_cards.iter().map(|card| card.raw()));
            put_ids(h, pile_cards.iter().map(|card| card.raw()));
            put_ids(h, event_cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        _ => unreachable!("effect routed to a different hash group"),
    }
}
