//! VM 效果请求解码链：带包前缀（TW/MB/DC/SP/OL/JSRG/STAR）的具名技能结算器。

use super::*;

pub(in super::super) fn decode_request(
    host: &mut dyn VmHost,
    frame: &mut Frame,
    tag: u16,
) -> Result<EffectRequest, VmError> {
    Ok(match tag {
        effect_tag::RESOLVE_STARCUXIA_WEIMING => {
            let variant = pop(frame)?.0.max(0) as u32;
            let source = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveStarcuxiaWeiming {
                player: PlayerId::new(player),
                source: PlayerId::new(source),
                variant,
            }
        }
        effect_tag::RESOLVE_DCHAOXIAN => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDchaoxian {
                player: PlayerId::new(player),
            }
        }
        effect_tag::RESOLVE_DCZHEE => {
            let mode = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDczhee {
                player: PlayerId::new(player),
                mode,
            }
        }
        effect_tag::RESOLVE_OLQIAOZHI => {
            let stage = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveOlqiaozhi {
                player: PlayerId::new(player),
                stage,
            }
        }
        effect_tag::RESOLVE_OLSUYI => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveOlsuyi {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
            }
        }
        effect_tag::RESOLVE_OLZHIJIE => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveOlzhijie {
                player: PlayerId::new(player),
            }
        }
        effect_tag::RESOLVE_SPOLAIGE => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveSpolaige {
                player: PlayerId::new(player),
            }
        }
        effect_tag::RESOLVE_OLWENREN => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveOlwenren {
                player: PlayerId::new(player),
                targets: host
                    .selected_targets()
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DCQIAOLVE => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDcqiaolve {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
            }
        }
        effect_tag::RESOLVE_MBZHENANN => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMbzhenann {
                player: PlayerId::new(player),
            }
        }
        effect_tag::RESOLVE_DCANXU => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDcanxu {
                player: PlayerId::new(player),
                targets: host
                    .selected_targets()
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect(),
                skill: engine_api::SkillId::new(host.context().skill_id),
            }
        }
        effect_tag::RESOLVE_MBBEIMING => {
            let stage = pop(frame)?.0.max(0) as u32;
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMbbeiming {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
                stage,
            }
        }
        effect_tag::RESOLVE_JSRGDAOREN => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveJsrgdaoren {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
            }
        }
        effect_tag::RESOLVE_DCBINGJI => {
            let kind = pop(frame)?.0.max(0) as u32;
            let stage = pop(frame)?.0.max(0) as u32;
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDcbingji {
                player: PlayerId::new(player),
                skill: rule_api::SkillId::new(skill),
                stage,
                kind,
            }
        }
        effect_tag::RESOLVE_TWJIEQIU => {
            let skill = pop(frame)?.0.max(0) as u32;
            let sub = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwjieqiu {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                subaction: sub,
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::RESOLVE_JSRGCHAOZHENG => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveJsrgchaozheng {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::RESOLVE_JSRGSHEJU => {
            let skill = pop(frame)?.0.max(0) as u32;
            let counterpart = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveJsrgsheju {
                player: PlayerId::new(player),
                counterpart: PlayerId::new(counterpart),
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::RESOLVE_MBZHUGUO => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMbzhuguo {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_MBANXIAN => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMbanxian {
                player: PlayerId::new(player),
                cards: host
                    .event_cards()
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                card: engine_api::CardId::NONE,
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_MBJINZU => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let helper_skill_def = pop(frame)?.0 as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMbjinzu {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                cards: host
                    .selected_cards()
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                card: engine_api::CardId::new(card),
                helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_SPXIZHAN => {
            let fangzong = pop(frame)?.0.max(0) as u32;
            let phase_player = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveSpxizhan {
                player: PlayerId::new(player),
                phase_player: PlayerId::new(phase_player),
                fangzong_skill_def: rule_api::SkillDefId::new(fangzong),
            }
        }
        effect_tag::RESOLVE_SPSONGSHU => {
            let block = pop(frame)?.0.max(0) as u32;
            let renku = pop(frame)?.0.max(0) as u32;
            let phase_player = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveSpsongshu {
                player: PlayerId::new(player),
                phase_player: PlayerId::new(phase_player),
                renku_skill_def: rule_api::SkillDefId::new(renku),
                block_skill_def: rule_api::SkillDefId::new(block),
            }
        }
        effect_tag::RESOLVE_SPCUNSI => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveSpcunsi {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
            }
        }
        effect_tag::RESOLVE_TWFUPAN => {
            let damage_amount = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwfupan {
                player: PlayerId::new(player),
                damage_amount,
                card: engine_api::CardId::NONE,
                target: PlayerId::NONE,
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWJIAOHUA => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwjiaohua {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                event_cards: host
                    .event_cards()
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWKUJIAN => {
            let action = pop(frame)?.0.max(0) as u8;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            let context = host.context();
            let mut cards = if action == 0 {
                host.selected_cards().to_vec()
            } else {
                host.event_cards().to_vec()
            };
            if cards.is_empty() && context.event_card != 0 {
                cards.push(context.event_card);
            }
            EffectRequest::ResolveTwkujian {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                cards: cards.into_iter().map(engine_api::CardId::new).collect(),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWRUILIAN => {
            let action = pop(frame)?.0.max(0) as u8;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwruilian {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                event_cards: host
                    .event_cards()
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                action,
                category: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWJIANWEI => {
            let stage = pop(frame)?.0.max(0) as u8;
            let actor = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwjianwei {
                player: PlayerId::new(player),
                actor: PlayerId::new(actor),
                target: PlayerId::NONE,
                card: engine_api::CardId::NONE,
                winner: PlayerId::NONE,
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWTANFENG => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwtanfeng {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                card: engine_api::CardId::NONE,
                stage: 0,
                phase: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWZHIBIAN => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwzhibian {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                winner: PlayerId::NONE,
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWYUYAN => {
            let card = pop(frame)?.0.max(0) as u32;
            let source = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwyuyan {
                player: PlayerId::new(player),
                source: PlayerId::new(source),
                card: engine_api::CardId::new(card),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWGUOYI => {
            let hand_lock_skill_def = pop(frame)?.0 as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwguoyi {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                hand_lock_skill_def,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWCHUHAI => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwchuhai {
                player: PlayerId::new(player),
                action,
                stage: 0,
                index: 0,
                event_cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWYIZHU => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let source = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwyizhu {
                player: PlayerId::new(player),
                source: PlayerId::new(source),
                card: engine_api::CardId::new(card),
                action,
                stage: 0,
                event_cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWLUANCHOU => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let helper_skill_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwluanchou {
                player: PlayerId::new(player),
                helper_skill_def,
                action,
                targets: host
                    .selected_targets()
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect(),
                source: PlayerId::NONE,
                target: PlayerId::NONE,
                amount: 0,
                nature: 0,
                card: engine_api::CardId::NONE,
                card_kind: 0,
                card_flags: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWXIAWEI => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let wangxing_skill_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwxiawei {
                player: PlayerId::new(player),
                wangxing_skill_def,
                action,
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWBIANFUREN => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let helper_skill_def = pop(frame)?.0 as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwbianfuren {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                helper_skill_def,
                action,
                stage: 0,
                cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWFEIYI => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwfeiyi {
                player: PlayerId::new(player),
                action,
                stage: 0,
                card: engine_api::CardId::NONE,
                event_cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWCHENZHEN => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let record_skill_def = pop(frame)?.0 as u32;
            let helper_skill_def = pop(frame)?.0 as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwchenzhen {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                helper_skill_def,
                record_skill_def,
                action,
                stage: 0,
                card_kind: 0,
                card: engine_api::CardId::NONE,
                branch: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWQIANJU => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwqianju {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWQINGXI => {
            let card = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwqingxi {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                stage: 0,
                count: 0,
                cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWZAOLI => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwzaoli {
                player: PlayerId::new(player),
                stage: 0,
                cards: Vec::new(),
                slots: Vec::new(),
                count: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWZHENGJIAN => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let helper_skill_def = pop(frame)?.0 as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwzhengjian {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                helper_skill_def,
                action,
                stage: 0,
                card: engine_api::CardId::NONE,
                branch: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWTIANYU => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let helper_skill_def = pop(frame)?.0 as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwtianyu {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                helper_skill_def,
                action,
                stage: 0,
                card: engine_api::CardId::NONE,
                branch: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWWUJING => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let target_count = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwwujing {
                player: PlayerId::new(player),
                target_count,
                action,
                event_cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWSHANGHE => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwshanghe {
                player: PlayerId::new(player),
                stage: 0,
                remaining: Vec::new(),
                got_jiu: false,
                skill: engine_api::SkillId::NONE,
            }
        }
        _ => return super::resolve_twwangchang::decode_request(host, frame, tag),
    })
}
