//! VM 效果请求解码链：具名技能结算器（无包前缀者）与效果原语。

use super::*;

pub(in super::super) fn decode_request(
    host: &mut dyn VmHost,
    frame: &mut Frame,
    tag: u16,
) -> Result<EffectRequest, VmError> {
    Ok(match tag {
        effect_tag::RESOLVE_LIANZHOU => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveLianzhou {
                player: PlayerId::new(player),
                targets: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_STDDECHONG => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveStddechong {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::DISCARD_HAND_EQUIP_COUNT => {
            let count = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::DiscardHandEquipCount {
                player: PlayerId::new(player),
                count,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_JIELIANG_GAIN => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveJieliangGain {
                player: PlayerId::new(player),
                mover: PlayerId::NONE,
                cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_STDJUJIAN => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveStdjujian {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_LULVE => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveLulve {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                branch: 2,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_HFJIEYING => {
            let grant_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveHfjieying {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                grant_def: engine_api::SkillDefId::new(grant_def),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_GAIN_TARGET_HAND_CEIL_HALF_DIFFERENCE => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            let card = host.selected_cards().first().copied().unwrap_or(0);
            EffectRequest::ChooseGainTargetHandCeilHalfDifference {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                skill: engine_api::SkillId::NONE,
                ready_to_choose: false,
            }
        }
        effect_tag::RESOLVE_GZJINFA => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            let cost_card = host.selected_cards().first().copied().unwrap_or(0);
            EffectRequest::ResolveGzjinfa {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                cost_card: engine_api::CardId::new(cost_card),
                given_card: engine_api::CardId::NONE,
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_WN_QIFENG => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveWnQifeng {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_STDGAOBIAN => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveStdgaobian {
                player: PlayerId::new(player),
                chooser: PlayerId::NONE,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_SIBIAN => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveSibian {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::SKIP_NEXT_DISCARD_PHASE => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::SkipNextDiscardPhase {
                player: PlayerId::new(player),
            }
        }
        effect_tag::RESOLVE_GONGJIAN => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveGongjian {
                player: PlayerId::new(player),
                targets: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::REVEAL_CHOOSE_CATEGORY_DISTRIBUTE_SELF_THRESHOLD_HP_LOSS => {
            let recipient_cap = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let reveal_count = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::RevealChooseCategoryDistributeSelfThresholdHpLoss {
                player: PlayerId::new(player),
                reveal_count,
                recipient_cap,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_OTHER_RANDOM_GAIN_REGION_THEN_DRAW_COMPARE_COLOR_LOSE_HP => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseOtherRandomGainRegionThenDrawCompareColorLoseHp {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_NSDINGZHOU => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveNsdingzhou {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_SXRMLANJIAO => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveSxrmlanjiao {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                stage: 0,
                view: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_XIZHEN => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveXizhen {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_XUXIE => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveXuxie {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_XUXIE_ADD => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveXuxieAdd {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::GIVE_SELECTED_THEN_TARGET_MAY_USE_DRAW_OWNER => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            let card = host.selected_cards().first().copied().unwrap_or(0);
            EffectRequest::GiveSelectedThenTargetMayUseDrawOwner {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                targets: Vec::new(),
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_CATEGORY_THEN_TARGET_GIVE_MATCHING_OR_DAMAGE_PLUS_ONE => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseCategoryThenTargetGiveMatchingOrDamagePlusOne {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                damage_id: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_OTHER_EXCLUDING_SOURCE_RANDOM_GAIN_HAND_EQUIP_THEN_USE_IF_EQUIP => {
            let source = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip {
                player: PlayerId::new(player),
                source: PlayerId::new(source),
                card: engine_api::CardId::NONE,
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_MOBILEDANSHOU => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMobiledanshou {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_JSRGCHIYING => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveJsrgchiying {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                started: false,
                remaining: Vec::new(),
                accum: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TIAOXIN => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTiaoxin {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_ZHIJI => {
            let guanxing_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveZhiji {
                player: PlayerId::new(player),
                guanxing_def: engine_api::SkillDefId::new(guanxing_def),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_YULI => {
            let mode = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveYuli {
                player: PlayerId::new(player),
                mode,
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
        effect_tag::RESOLVE_TINGWEI => {
            let mode = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTingwei {
                player: PlayerId::new(player),
                mode,
                card: engine_api::CardId::NONE,
                targets: Vec::new(),
                target: PlayerId::NONE,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_XUNYI => {
            let mode = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveXunyi {
                player: PlayerId::new(player),
                mode,
                source: PlayerId::NONE,
                target: PlayerId::NONE,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_BOMING => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveBoming {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::NONE,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_EJIAN => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveEjian {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::NONE,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_YIYONG => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveYiyong {
                player: PlayerId::new(player),
                card: engine_api::CardId::NONE,
                source: PlayerId::NONE,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_SHANXIE => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveShanxie {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_SHEYI => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveSheyi {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TIANYIN => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTianyin {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DBZHUIFENG => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDbzhuifeng {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DBQUEDI => {
            let card = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDbquedi {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_FYJIANYU => {
            let b = pop(frame)?.0.max(0) as u32;
            let a = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveFyjianyu {
                player: PlayerId::new(player),
                source: PlayerId::new(a),
                target: PlayerId::new(b),
                targets: host
                    .selected_targets()
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_LIUBING => {
            let card = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveLiubing {
                player: PlayerId::new(player),
                card: engine_api::CardId::new(card),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_HEJI => {
            let card = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveHeji {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_QINGYU => {
            let mbyongjue_def = pop(frame)?.0 as u32;
            let xuancun_def = pop(frame)?.0 as u32;
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveQingyu {
                player: PlayerId::new(player),
                action,
                xuancun_def: engine_api::SkillDefId::new(xuancun_def),
                mbyongjue_def: engine_api::SkillDefId::new(mbyongjue_def),
                target: PlayerId::NONE,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_ZHENGSU => {
            let card = pop(frame)?.0.max(0) as u32;
            let stage = pop(frame)?.0.max(0) as u32;
            let mode = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveZhengsu {
                player: PlayerId::new(player),
                mode,
                stage: stage as u8,
                card: engine_api::CardId::new(card),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_HOUFENG => {
            let stage = pop(frame)?.0.max(0) as u32;
            let mode = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveHoufeng {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                mode,
                stage: stage as u8,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_XIONGSHU => {
            let stage = pop(frame)?.0.max(0) as u32;
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveXiongshu {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                action,
                stage: stage as u8,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_JIANHUI => {
            let target = pop(frame)?.0.max(0) as u32;
            let source = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveJianhui {
                player: PlayerId::new(player),
                source: PlayerId::new(source),
                target: PlayerId::new(target),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_TONGDUO => {
            let stage = pop(frame)?.0.max(0) as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTongduo {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                stage: stage as u8,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_YUANQING => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveYuanqing {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_SHUCHEN => {
            let renku_def = pop(frame)?.0 as u32;
            let dying = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveShuchen {
                player: PlayerId::new(player),
                dying: PlayerId::new(dying),
                renku_def: engine_api::SkillDefId::new(renku_def),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DUANBI => {
            let stage = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDuanbi {
                player: PlayerId::new(player),
                targets: host
                    .selected_targets()
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect(),
                stage: stage as u8,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_ZAOLI => {
            let mark = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveZaoli {
                player: PlayerId::new(player),
                mark,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_GUYING => {
            let stage = pop(frame)?.0.max(0) as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let actor = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveGuying {
                player: PlayerId::new(player),
                actor: PlayerId::new(actor),
                card: engine_api::CardId::new(card),
                stage: stage as u8,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_ZAIQI => {
            let extra = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveZaiqi {
                player: PlayerId::new(player),
                extra,
                skill: engine_api::SkillId::NONE,
            }
        }
        _ => return super::resolve_twwangchang_packs::decode_request(host, frame, tag),
    })
}
