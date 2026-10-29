//! VM 效果请求解码链：带包前缀（DDD/WDB/DC/RE/GZ/XIN/TW）的具名技能结算器。

use super::*;

pub(in super::super) fn decode_request(
    host: &mut dyn VmHost,
    frame: &mut Frame,
    tag: u16,
) -> Result<EffectRequest, VmError> {
    Ok(match tag {
        effect_tag::RESOLVE_DCSHAOWEI => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDcshaowei {
                player: PlayerId::new(player),
                action,
                cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DCDICHOU => {
            let helper_skill_def = pop(frame)?.0 as u32;
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDcdichou {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                card: engine_api::CardId::NONE,
                action,
                helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DCDUFENG => {
            let helper_skill_def = pop(frame)?.0 as u32;
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDcdufeng {
                player: PlayerId::new(player),
                action,
                helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DCCTJIUXIAN => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let event_player = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDcctjiuxian {
                player: PlayerId::new(player),
                event_player: PlayerId::new(event_player),
                target: PlayerId::NONE,
                card: engine_api::CardId::NONE,
                action,
                cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DC_SB_ZHANGXIU => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDcSbZhangxiu {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                action,
                cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DC_SB_ZHOUYU => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let source = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDcSbZhouyu {
                player: PlayerId::new(player),
                source: PlayerId::new(source),
                card: engine_api::CardId::new(card),
                action,
                event_targets: Vec::new(),
                damaged_targets: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DC_SB_LUSU => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let source = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDcSbLusu {
                player: PlayerId::new(player),
                source: PlayerId::new(source),
                card: engine_api::CardId::new(card),
                action,
                cards: Vec::new(),
                amount: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_WDB_HANLONG => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let helper_skill_def = pop(frame)?.0 as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveWdbHanlong {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_WDB_LIUFU => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let zhucheng_skill_def = pop(frame)?.0 as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveWdbLiufu {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                zhucheng_skill_def: engine_api::SkillDefId::new(zhucheng_skill_def),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_WDB_LIUYAN => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveWdbLiuyan {
                player: PlayerId::new(player),
                card: engine_api::CardId::new(card),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_WDB_LIUZAN => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let helper_skill_def = pop(frame)?.0 as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveWdbLiuzan {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_WDB_TIANYU => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveWdbTianyu {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_WDB_XIZHENXIHONG => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveWdbXizhenxihong {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_WDB_YANGYI => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveWdbYangyi {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_WDB_ZAOZHIRENJUN => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveWdbZaozhiRenjun {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                cards: host
                    .event_cards()
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DDD_XIANXI => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDddXianxi {
                player: PlayerId::new(player),
                card: engine_api::CardId::new(card),
                action,
                event_targets: Vec::new(),
                damaged_targets: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DDD_XIAHENG => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDddXiaheng {
                player: PlayerId::new(player),
                discard_target: PlayerId::NONE,
                damage_target: PlayerId::NONE,
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DDD_LIUCHONG => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let actor = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDddLiuchong {
                player: PlayerId::new(player),
                actor: PlayerId::new(actor),
                card: engine_api::CardId::new(card),
                action,
                cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DDD_DUANBING => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDddDuanbing {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DDD_WUZHI => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDddWuzhi {
                player: PlayerId::new(player),
                card: engine_api::CardId::new(card),
                action,
                cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DDD_XUJING => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let source = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDddXujing {
                player: PlayerId::new(player),
                source: PlayerId::new(source),
                card: engine_api::CardId::new(card),
                action,
                targets: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DDD_CAOMAO => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDddCaomao {
                player: PlayerId::new(player),
                action,
                cards: host
                    .selected_cards()
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                targets: host
                    .selected_targets()
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DDD_XINXIANYING => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let actor = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDddXinxianying {
                player: PlayerId::new(player),
                actor: PlayerId::new(actor),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DDD_XIANGLANG => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let actor = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDddXianglang {
                player: PlayerId::new(player),
                actor: PlayerId::new(actor),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DDD_YUJIN => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let actor = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDddYujin {
                player: PlayerId::new(player),
                actor: PlayerId::new(actor),
                target: PlayerId::NONE,
                move_mask: 0,
                action,
                candidates: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DDD_LIUYE => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let actor = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDddLiuye {
                player: PlayerId::new(player),
                actor: PlayerId::new(actor),
                card: rule_api::CardId::new(card),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DDD_BAOSANNIANG => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDddBaosanniang {
                player: PlayerId::new(player),
                target: host
                    .selected_targets()
                    .first()
                    .copied()
                    .map(PlayerId::new)
                    .unwrap_or(PlayerId::NONE),
                card: host
                    .selected_cards()
                    .first()
                    .copied()
                    .map(rule_api::CardId::new)
                    .unwrap_or(rule_api::CardId::NONE),
                action,
                cards: host
                    .selected_cards()
                    .iter()
                    .copied()
                    .map(rule_api::CardId::new)
                    .collect(),
                targets: host
                    .selected_targets()
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect(),
                used_kind: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DDD_ZHENJI => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDddZhenji {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                action,
                stage: 0,
                piles: Vec::new(),
                cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DDD_ZHAOANG => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDddZhaoang {
                player: PlayerId::new(player),
                action,
                stage: 0,
                card: rule_api::CardId::NONE,
                kind: 0,
                count: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DDD_LIUBA => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let actor = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDddLiuba {
                player: PlayerId::new(player),
                actor: PlayerId::new(actor),
                action,
                cards: Vec::new(),
                targets: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DDD_JIANSHUO => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDddJianshuo {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                action,
                cards: Vec::new(),
                targets: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DDD_GUANNING => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let actor = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDddGuanning {
                player: PlayerId::new(player),
                actor: PlayerId::new(actor),
                action,
                card: engine_api::CardId::NONE,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DDD_CAOSHUANG => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let actor = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDddCaoshuang {
                player: PlayerId::new(player),
                actor: PlayerId::new(actor),
                card: engine_api::CardId::new(card),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DDD_XUELINGYUN => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let actor = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDddXuelingyun {
                player: PlayerId::new(player),
                actor: PlayerId::new(actor),
                card: engine_api::CardId::new(card),
                cards: Vec::new(),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DDD_LIUHONG => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let actor = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDddLiuhong {
                player: PlayerId::new(player),
                actor: PlayerId::new(actor),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DDD_XIAHOUXUAN => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDddXiahouxuan {
                player: PlayerId::new(player),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DDD_ZHANGKAI => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let actor = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDddZhangkai {
                player: PlayerId::new(player),
                actor: PlayerId::new(actor),
                card: engine_api::CardId::new(card),
                cards: Vec::new(),
                amount: 0,
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DDD_LIANGXI => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let actor = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDddLiangxi {
                player: PlayerId::new(player),
                actor: PlayerId::new(actor),
                card: engine_api::CardId::new(card),
                cards: Vec::new(),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TWJUXIANG => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let event_player = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwjuxiang {
                player: PlayerId::new(player),
                event_player: PlayerId::new(event_player),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_GZBUQU => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let delta = pop(frame)?.0;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveGzbuqu {
                player: PlayerId::new(player),
                delta,
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_GZBUQU_RECOVER => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveGzbuquRecover {
                player: PlayerId::new(player),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_REBINGYI => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveRebingyi {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_REBOTU => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveRebotu {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_REJIEYIN => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveRejieyin {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                stage: 0,
                card: host
                    .selected_cards()
                    .first()
                    .copied()
                    .map(engine_api::CardId::new)
                    .unwrap_or(engine_api::CardId::NONE),
                equip: false,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_REANXU => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveReanxu {
                player: PlayerId::new(player),
                stage: 0,
                first: PlayerId::NONE,
                second: PlayerId::NONE,
                card: engine_api::CardId::NONE,
                from_hand: false,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_REJIZHI => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveRejizhi {
                player: PlayerId::new(player),
                stage: 0,
                card: engine_api::CardId::NONE,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_XINANGUO => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveXinanguo {
                player: PlayerId::new(player),
                target: host
                    .selected_targets()
                    .first()
                    .copied()
                    .map(PlayerId::new)
                    .unwrap_or(PlayerId::NONE),
                stage: 0,
                index: 0,
                remaining: 0b111,
                skill: engine_api::SkillId::NONE,
            }
        }
        _ => return super::resolve_qixing::decode_request(host, frame, tag),
    })
}
