//! VM 效果请求解码链：手杀（MB）与谋攻（POT）具名技能结算器。

use super::*;

pub(in super::super) fn decode_request(
    host: &mut dyn VmHost,
    frame: &mut Frame,
    tag: u16,
) -> Result<EffectRequest, VmError> {
    Ok(match tag {
        effect_tag::RESOLVE_MBQINGDAO => {
            let damaged = pop(frame)?.0 != 0;
            let card = pop(frame)?.0 as u32;
            let source = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMbqingdao {
                player: PlayerId::new(player),
                source: PlayerId::new(source),
                card: engine_api::CardId::new(card),
                damaged,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_MBXIUGENG => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let hand_limit_def = pop(frame)?.0 as u32;
            let helper_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMbxiugeng {
                player: PlayerId::new(player),
                helper_def: engine_api::SkillDefId::new(helper_def),
                hand_limit_def: engine_api::SkillDefId::new(hand_limit_def),
                stage,
                targets: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_MBCHENSHE => {
            let source = pop(frame)?.0.max(0) as u32;
            let dying = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMbchenshe {
                player: PlayerId::new(player),
                dying: PlayerId::new(dying),
                source: PlayerId::new(source),
                stage: 0,
                roles: Vec::new(),
                suits: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_MBQUNCHONG => {
            let helper_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMbquanchong {
                player: PlayerId::new(player),
                helper_def: engine_api::SkillDefId::new(helper_def),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_MBRENXING => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMbrenxing {
                player: PlayerId::new(player),
                stage: 0,
                target: PlayerId::NONE,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_MBCHIZHANG => {
            let card = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMbchizhang {
                player: PlayerId::new(player),
                card: engine_api::CardId::new(card),
                stage: 0,
                cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_MBDUANYANG => {
            let mode = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0 as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMbduanyang {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                mode,
                stage: 0,
                cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_MBRUNWEI => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMbrunwei {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                stage,
                value: 0,
                cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_MBSHUANGHUAI => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMbshuanghuai {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                stage: 0,
                choice: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_MBXIONGTU => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMbxiongtu {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                stage,
                card: engine_api::CardId::NONE,
                cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_MBGANGGENG => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMbganggeng {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                stage,
                cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_MBSIJIAN => {
            let helper_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMbsijian {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                stage: 0,
                choice: 0,
                helper_def: engine_api::SkillDefId::new(helper_def),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_MBJIANJI => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMbjianji {
                player: PlayerId::new(player),
                source: PlayerId::NONE,
                target: PlayerId::NONE,
                stage: 0,
                card: engine_api::CardId::NONE,
                cards: Vec::new(),
                targets: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_MBYUANMO => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMbyuanmo {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                stage: 0,
                card: engine_api::CardId::NONE,
                value: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_MBFEIJING => {
            let card = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMbfeijing {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                stage: 0,
                targets: Vec::new(),
                cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_MBXIAOGE => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMbxiaoge {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                stage,
                targets: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_MBXUYE => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMbxuye {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                stage: 0,
                was_max: false,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_MBKUANGXIANG => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMbkuangxiang {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTCHENGPU => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotchengpu {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                stage,
                damage_id: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTJUGU => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotjugu {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTSIFENG => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotsifeng {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTCHENQUN => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotchenqun {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTWANGLIE => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let helper_skill_def = pop(frame)?.0 as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotwanglie {
                player: PlayerId::new(player),
                card: engine_api::CardId::new(card),
                helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTHONGYI => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePothongyi {
                player: PlayerId::new(player),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTGUANSHA => {
            let helper_skill_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotguansha {
                player: PlayerId::new(player),
                helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTJIYU => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotjiyu {
                player: PlayerId::new(player),
                card: engine_api::CardId::new(host.selected_cards().first().copied().unwrap_or(0)),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_MBZHUJI => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMbzhuji {
                player: PlayerId::new(player),
                choice: 0,
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTLIBING => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotlibing {
                player: PlayerId::new(player),
                card: engine_api::CardId::new(card),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTPOXI => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotpoxi {
                player: PlayerId::new(player),
                cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTFUAN => {
            let yinxian_def = pop(frame)?.0 as u32;
            let range_helper_def = pop(frame)?.0 as u32;
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotfuan {
                player: PlayerId::new(player),
                card: engine_api::CardId::new(card),
                stage,
                range_helper_def: engine_api::SkillDefId::new(range_helper_def),
                yinxian_def: engine_api::SkillDefId::new(yinxian_def),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTYINXIAN => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotyinxian {
                player: PlayerId::new(player),
                card: engine_api::CardId::new(card),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTHUANJIE => {
            let kanpo_def = pop(frame)?.0 as u32;
            let qice_def = pop(frame)?.0 as u32;
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePothuanjie {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                stage,
                qice_def: engine_api::SkillDefId::new(qice_def),
                kanpo_def: engine_api::SkillDefId::new(kanpo_def),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTNIGU => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotnigu {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                card: engine_api::CardId::NONE,
                targets: Vec::new(),
                index: 0,
                declined: 0,
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTLULIAN => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotlulian {
                player: PlayerId::new(player),
                card: engine_api::CardId::new(card),
                target: PlayerId::NONE,
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTDIMENG => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotdimeng {
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
        effect_tag::RESOLVE_POTHAOSHI => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePothaoshi {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_MBDIANCAI => {
            let actor = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMbdiancai {
                player: PlayerId::new(player),
                actor: PlayerId::new(actor),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTTUNTIAN => {
            let zaoxian_def = pop(frame)?.0 as u32;
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePottuntian {
                player: PlayerId::new(player),
                action,
                amount: 0,
                targets: host
                    .selected_targets()
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect(),
                zaoxian_def: engine_api::SkillDefId::new(zaoxian_def),
                skill: engine_api::SkillId::NONE,
            }
        }
        _ => return super::resolve_potjixi::decode_request(host, frame, tag),
    })
}
