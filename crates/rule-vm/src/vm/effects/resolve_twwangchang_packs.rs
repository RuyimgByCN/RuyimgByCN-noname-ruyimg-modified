//! VM 效果请求解码链：带包前缀（SP/TW/DC/XIN/RE）的具名技能结算器。

use super::*;

pub(in super::super) fn decode_request(
    host: &mut dyn VmHost,
    frame: &mut Frame,
    tag: u16,
) -> Result<EffectRequest, VmError> {
    Ok(match tag {
        effect_tag::RESOLVE_TWWANGCHANG => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwwangchang {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                selected_card: engine_api::CardId::NONE,
                action,
                stage: 0,
                targets: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWCAOZHAO => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let twfeifu_skill_def = pop(frame)?.0 as u32;
            let twfuzuan_skill_def = pop(frame)?.0 as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let source = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwcaozhao {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                source: PlayerId::new(source),
                card: engine_api::CardId::new(card),
                selected_skill_def: 0,
                twfuzuan_skill_def,
                twfeifu_skill_def,
                action,
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWGUOHUAI => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let helper_skill_def = pop(frame)?.0 as u32;
            let mark_def = pop(frame)?.0 as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let source = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwguohuai {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                source: PlayerId::new(source),
                card: engine_api::CardId::new(card),
                mark_def,
                helper_skill_def,
                action,
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWCHENGPU => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let chunlao_skill_def = pop(frame)?.0 as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let source = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwchengpu {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                source: PlayerId::new(source),
                card: engine_api::CardId::new(card),
                chunlao_skill_def,
                action,
                stage: 0,
                selected_card: engine_api::CardId::NONE,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWCAOANG => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let helper_skill_def = pop(frame)?.0 as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let source = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwcaoang {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                source: PlayerId::new(source),
                card: engine_api::CardId::new(card),
                helper_skill_def,
                action,
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWNIUJIN => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let limited_skill_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwniujin {
                player: PlayerId::new(player),
                limited_skill_def: engine_api::SkillDefId::new(limited_skill_def),
                action,
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_RETIANXIANG => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveRetianxiang {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                discard_card: engine_api::CardId::NONE,
                branch: 0,
                source: PlayerId::NONE,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DCTINGXIAN => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDctingxian {
                player: PlayerId::new(player),
                card: engine_api::CardId::NONE,
                targets: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DCNUCHEN => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDcnuchen {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DCDANJI => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDcdanji {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::RESOLVE_REFUHAI => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveRefuhai {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DCZHIHENG => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDczhiheng {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_SPJIEDAO => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveSpjiedao {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_SPFENGYIN => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveSpfengyin {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_XINFU_LIANPIAN => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveXinfuLianpian {
                player: PlayerId::new(player),
                targets: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_XINFU_QINGUO => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveXinfuQinguo {
                player: PlayerId::new(player),
                card: engine_api::CardId::NONE,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DCMANZHI => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDcmanzhi {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_XINXINGLUAN => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveXinxingluan {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DCKUIZHEN => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDckuizhen {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DCKUIZHEN_RESOLVE => {
            let loser = pop(frame)?.0.max(0) as u32;
            let winner = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDckuizhenResolve {
                player: PlayerId::new(player),
                winner: PlayerId::new(winner),
                loser: PlayerId::new(loser),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_REKUANSHI => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveRekuanshi {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_REKUANSHI_RESOLVE => {
            let damaged = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveRekuanshiResolve {
                player: PlayerId::new(player),
                damaged: PlayerId::new(damaged),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DCMOYU => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDcmoyu {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DCCHIYING => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDcchiying {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                stage: 1,
                accum: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_XINKUANGFU => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveXinkuangfu {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                sha_target: PlayerId::NONE,
                stage: 1,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DCXIANG_SHU => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDcxiangshu {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DCXIANG_SHU_LOTTERY => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDcxiangshuLottery {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DCYINGTU => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDcyingtu {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                stage: 0,
                cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_SPMIAOLVE => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveSpmiaolve {
                player: PlayerId::new(player),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_SPYINGJIA => {
            let helper_def = pop(frame)?.0 as u32;
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveSpyingjia {
                player: PlayerId::new(player),
                action,
                helper_def: engine_api::SkillDefId::new(helper_def),
                card: engine_api::CardId::NONE,
                card_kind: 0,
                target: PlayerId::NONE,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_XINMINGSHI => {
            let source = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveXinmingshi {
                player: PlayerId::new(player),
                source: PlayerId::new(source),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_SPYINJU => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveSpyinju {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_SPCHIJIE => {
            let card = pop(frame)?.0.max(0) as u32;
            let source = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveSpchijie {
                player: PlayerId::new(player),
                source: PlayerId::new(source),
                card: engine_api::CardId::new(card),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_SPSHANGYI => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveSpshangyi {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_SPJIANYI => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveSpjianyi {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_SPYAJUN => {
            let stage = pop(frame)?.0.max(0) as u32;
            let less_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveSpyajun {
                player: PlayerId::new(player),
                less_def: engine_api::SkillDefId::new(less_def),
                card: engine_api::CardId::NONE,
                target: PlayerId::NONE,
                target_card: engine_api::CardId::NONE,
                stage: stage as u8,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_SPZUNDI => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveSpzundi {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::NONE,
                destination: PlayerId::NONE,
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_XINYANGJIE => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveXinyangjie {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::NONE,
                target_card: engine_api::CardId::NONE,
                source: PlayerId::NONE,
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_RECHUHAI => {
            let stage = pop(frame)?.0.max(0) as u32;
            let target_card = pop(frame)?.0.max(0) as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveRechuhai {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                target_card: engine_api::CardId::new(target_card),
                stage: stage as u8,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_SPQIAI => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveSpqiai {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::NONE,
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_SPSHANXI => {
            let stage = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveSpshanxi {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                stage: stage as u8,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_SPSHANXI_SUOMING => {
            let stage = pop(frame)?.0.max(0) as u32;
            let actor = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveSpshanxiSuoming {
                player: PlayerId::new(player),
                actor: PlayerId::new(actor),
                stage: stage as u8,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWQIUSHOU => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwqiushou {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        _ => return super::resolve_twwangchang_mobile::decode_request(host, frame, tag),
    })
}
