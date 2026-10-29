//! 效果请求的规范哈希编码：tw、mb 族结算器。

use super::*;

pub(super) fn hash_effect(h: &mut blake3::Hasher, effect: &EffectRequest) {
    match effect {
        EffectRequest::ResolveTwluanchou {
            player,
            helper_skill_def,
            action,
            targets,
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
            h.update(&398u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *helper_skill_def);
            put_u8(h, *action);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u32(h, *amount);
            put_u8(h, *nature);
            put_u32(h, card.raw());
            put_u8(h, *card_kind);
            put_u8(h, *card_flags);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwxiawei {
            player,
            wangxing_skill_def,
            action,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&399u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, *wangxing_skill_def);
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwbianfuren {
            player,
            target,
            helper_skill_def,
            action,
            stage,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&400u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *helper_skill_def);
            put_u8(h, *action);
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwfeiyi {
            player,
            action,
            stage,
            card,
            event_cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&401u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, card.raw());
            put_ids(h, event_cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
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
            put_u8(h, 0);
            h.update(&402u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *helper_skill_def);
            put_u32(h, *record_skill_def);
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u8(h, *card_kind);
            put_u32(h, card.raw());
            put_u8(h, *branch);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwqianju {
            player,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&403u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwqingxi {
            player,
            target,
            card,
            stage,
            count,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&404u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_u8(h, *count);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwzaoli {
            player,
            stage,
            cards,
            slots,
            count,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&405u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, slots.len() as u32);
            h.update(slots);
            put_u8(h, *count);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwzhengjian {
            player,
            target,
            helper_skill_def,
            action,
            stage,
            card,
            branch,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&406u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *helper_skill_def);
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, card.raw());
            put_u8(h, *branch);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwtianyu {
            player,
            target,
            helper_skill_def,
            action,
            stage,
            card,
            branch,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&407u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, *helper_skill_def);
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, card.raw());
            put_u8(h, *branch);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwwujing {
            player,
            target_count,
            action,
            event_cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&408u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *target_count);
            put_u8(h, *action);
            put_ids(h, event_cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwshanghe {
            player,
            stage,
            remaining,
            got_jiu,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&409u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_ids(h, remaining.iter().map(|target| target.raw()));
            put_bool(h, *got_jiu);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwwangchang {
            player,
            target,
            card,
            selected_card,
            action,
            stage,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&410u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u32(h, selected_card.raw());
            put_u8(h, *action);
            put_u8(h, *stage);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
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
            put_u8(h, 0);
            h.update(&411u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, *selected_skill_def);
            put_u32(h, *twfuzuan_skill_def);
            put_u32(h, *twfeifu_skill_def);
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
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
            put_u8(h, 0);
            h.update(&412u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, *mark_def);
            put_u32(h, *helper_skill_def);
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
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
            put_u8(h, 0);
            h.update(&413u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, *chunlao_skill_def);
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, selected_card.raw());
            put_u32(h, skill.raw());
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
            put_u8(h, 0);
            h.update(&415u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_u32(h, *helper_skill_def);
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveTwniujin {
            player,
            limited_skill_def,
            action,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&417u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, limited_skill_def.raw());
            put_u8(h, *action);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbqingdao {
            player,
            source,
            card,
            damaged,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&477u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, card.raw());
            put_bool(h, *damaged);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbxiugeng {
            player,
            helper_def,
            hand_limit_def,
            stage,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&478u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, helper_def.raw());
            put_u32(h, hand_limit_def.raw());
            put_u8(h, *stage);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbchenshe {
            player,
            dying,
            source,
            stage,
            roles,
            suits,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&479u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, dying.raw());
            put_u32(h, source.raw());
            put_u8(h, *stage);
            put_ids(h, roles.iter().map(|target| target.raw()));
            put_u32(h, suits.len() as u32);
            for suit in suits {
                put_u8(h, *suit);
            }
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbquanchong {
            player,
            helper_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&480u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, helper_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbrenxing {
            player,
            stage,
            target,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&481u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *stage);
            put_u32(h, target.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbchizhang {
            player,
            card,
            stage,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&482u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
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
            put_u8(h, 0);
            h.update(&483u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *mode);
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbrunwei {
            player,
            target,
            stage,
            value,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&484u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, *value);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbshuanghuai {
            player,
            target,
            stage,
            choice,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&485u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u8(h, *choice);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbxiongtu {
            player,
            target,
            stage,
            card,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&486u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, card.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbganggeng {
            player,
            target,
            stage,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&487u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbsijian {
            player,
            target,
            stage,
            choice,
            helper_def,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&488u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u8(h, *choice);
            put_u32(h, helper_def.raw());
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbjianji {
            player,
            source,
            target,
            stage,
            card,
            cards,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&489u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, source.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, card.raw());
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbyuanmo {
            player,
            target,
            stage,
            card,
            value,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&490u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, card.raw());
            put_u32(h, *value);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbfeijing {
            player,
            target,
            card,
            stage,
            targets,
            cards,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&491u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_ids(h, cards.iter().map(|card| card.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbxiaoge {
            player,
            target,
            card,
            stage,
            targets,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&492u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u32(h, card.raw());
            put_u8(h, *stage);
            put_ids(h, targets.iter().map(|target| target.raw()));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbxuye {
            player,
            target,
            stage,
            was_max,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&496u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u8(h, u8::from(*was_max));
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbkuangxiang {
            player,
            target,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&497u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u32(h, target.raw());
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        EffectRequest::ResolveMbzhuji {
            player,
            choice,
            stage,
            skill,
        } => {
            put_u8(h, 0);
            h.update(&506u16.to_le_bytes());
            put_u32(h, player.raw());
            put_u8(h, *choice);
            put_u8(h, *stage);
            put_u32(h, skill.raw());
        }

        _ => unreachable!("effect routed to a different hash group"),
    }
}
