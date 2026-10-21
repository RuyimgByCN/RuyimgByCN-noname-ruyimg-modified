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
        effect_tag::RESOLVE_DCTINGXIAN => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDctingxian {
                player: PlayerId::new(player),
                card: engine_api::CardId::NONE,
                targets: Vec::new(),
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
        effect_tag::RESOLVE_DCZHIHENG => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDczhiheng {
                player: PlayerId::new(player),
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
        effect_tag::RESOLVE_SPJIEDAO => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveSpjiedao {
                player: PlayerId::new(player),
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
        effect_tag::RESOLVE_XINMINGSHI => {
            let source = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveXinmingshi {
                player: PlayerId::new(player),
                source: PlayerId::new(source),
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
        effect_tag::RESOLVE_MBDIANCAI => {
            let actor = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMbdiancai {
                player: PlayerId::new(player),
                actor: PlayerId::new(actor),
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
        effect_tag::RESOLVE_TWQIUSHOU => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveTwqiushou {
                player: PlayerId::new(player),
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
