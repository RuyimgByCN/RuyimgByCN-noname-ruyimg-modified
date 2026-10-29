//! VM 效果请求解码链：带包前缀（POT/TW/OL/RE/MB）的具名技能结算器（链尾）。

use super::*;

pub(in super::super) fn decode_request(
    host: &mut dyn VmHost,
    frame: &mut Frame,
    tag: u16,
) -> Result<EffectRequest, VmError> {
    Ok(match tag {
        effect_tag::RESOLVE_POTJIXI => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotjixi {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                action,
                targets: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTCEDUAN => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotceduan {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTTAISHICI => {
            let zhanlie_def = pop(frame)?.0 as u32;
            let hanzhan_def = pop(frame)?.0 as u32;
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePottaishici {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                action,
                cards: host
                    .event_cards()
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                hanzhan_def: engine_api::SkillDefId::new(hanzhan_def),
                zhanlie_def: engine_api::SkillDefId::new(zhanlie_def),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_RELEIJI => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveReleiji {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_MBZHONGHUI => {
            let mbkechang_def = pop(frame)?.0 as u32;
            let mbsizi_def = pop(frame)?.0 as u32;
            let amount = pop(frame)?.0.max(0) as u32;
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMbzhonghui {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                action,
                amount,
                mbsizi_def: engine_api::SkillDefId::new(mbsizi_def),
                mbkechang_def: engine_api::SkillDefId::new(mbkechang_def),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTJIEJIE => {
            let qingshi_def = pop(frame)?.0 as u32;
            let diamond_def = pop(frame)?.0 as u32;
            let club_def = pop(frame)?.0 as u32;
            let heart_def = pop(frame)?.0 as u32;
            let spade_def = pop(frame)?.0 as u32;
            let used_def = pop(frame)?.0 as u32;
            let actor = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotjiejie {
                player: PlayerId::new(player),
                actor: PlayerId::new(actor),
                used_def: engine_api::SkillDefId::new(used_def),
                suit_defs: [spade_def, heart_def, club_def, diamond_def]
                    .map(engine_api::SkillDefId::new),
                qingshi_def: engine_api::SkillDefId::new(qingshi_def),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTQINGSHI => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotqingshi {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTWEIYAN => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotweiyan {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTZHOUYU => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotzhouyu {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                action,
                card_flags: 0,
                targets: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTLVYI => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotlvyi {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                action,
                event_cards: host
                    .event_cards()
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                targets: host
                    .event_targets()
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTCAOSHUANG => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotcaoshuang {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_POTYUJI => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolvePotyuji {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                action,
                cards: host
                    .event_cards()
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                targets: host
                    .event_targets()
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_REHENGJIANG => {
            let draw_def = pop(frame)?.0 as u32;
            let hand_limit_def = pop(frame)?.0 as u32;
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let actor = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveRehengjiang {
                player: PlayerId::new(player),
                actor: PlayerId::new(actor),
                action,
                hand_limit_def: engine_api::SkillDefId::new(hand_limit_def),
                draw_def: engine_api::SkillDefId::new(draw_def),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWMOUKUI => {
            let card = pop(frame)?.0 as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwmoukui {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWFUMAN => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            let context = host.context();
            let cards = match action {
                0 => host.selected_cards().to_vec(),
                1 => (context.event_card != 0)
                    .then_some(context.event_card)
                    .into_iter()
                    .collect(),
                _ => host.event_cards().to_vec(),
            };
            EffectRequest::ResolveTwfuman {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                cards: cards.iter().copied().map(engine_api::CardId::new).collect(),
                action,
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWQINGKOU => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0 as u32;
            let winner = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwqingkou {
                player: PlayerId::new(player),
                winner: PlayerId::new(winner),
                card: engine_api::CardId::new(card),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWCHENXUN => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0 as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwchenxun {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWFENXIAN => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0 as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwfenxian {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_OLDANXU => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveOldAnxu {
                player: PlayerId::new(player),
                stage: 0,
                giver: PlayerId::NONE,
                receiver: PlayerId::NONE,
                card: engine_api::CardId::NONE,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_OLDQINQING => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveOldQinqing {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                lord: PlayerId::NONE,
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_OLDMINGJIAN => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveOldMingjian {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_OLDXUANFENG => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveOldXuanfeng {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                card: engine_api::CardId::NONE,
                stage: 0,
                branch: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_OLDHUISHENG => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveOldHuisheng {
                player: PlayerId::new(player),
                source: PlayerId::NONE,
                damage_id: 0,
                stage: 0,
                cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_OLDJIEFAN => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveOldJiefan {
                player: PlayerId::new(player),
                dying: PlayerId::NONE,
                current: PlayerId::NONE,
                card: engine_api::CardId::NONE,
                damage_id: 0,
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_OL_JIE_DANSHOU => {
            let skill = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveOlJieDanshou {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::new(skill),
            }
        }
        _ => {
            return Err(VmError::Host(format!("unknown effect tag {tag}")));
        }
    })
}
