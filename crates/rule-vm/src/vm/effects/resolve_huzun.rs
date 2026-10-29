//! VM 效果请求解码链：具名技能结算器（无包前缀者）与效果原语。

use super::*;

pub(in super::super) fn decode_request(
    host: &mut dyn VmHost,
    frame: &mut Frame,
    tag: u16,
) -> Result<EffectRequest, VmError> {
    Ok(match tag {
        effect_tag::RESOLVE_HUZUN => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let source = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveHuzun {
                player: PlayerId::new(player),
                source: PlayerId::new(source),
                card: engine_api::CardId::new(card),
                action,
                cards: Vec::new(),
                targets: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_ZHUGEMENGXUE => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveZhugemengxue {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                card: engine_api::CardId::NONE,
                action,
                cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_BAILINGYUN => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveBailingyun {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                card: engine_api::CardId::NONE,
                action,
                cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_DISCARD_HAND_RANGE => {
            let max = pop(frame)?.0.max(0) as u32;
            let min = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseDiscardHandRange {
                player: PlayerId::new(player),
                min,
                max,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_BIANYUE => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveBianyue {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                action,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_ZHUPEILAN => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let helper_skill_def = pop(frame)?.0 as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveZhupeilan {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                action,
                event_cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_ZHANGJIAN => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let grant_skill_def = pop(frame)?.0 as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveZhangjian {
                player: PlayerId::new(player),
                card: engine_api::CardId::new(card),
                grant_skill_def: engine_api::SkillDefId::new(grant_skill_def),
                action,
                target: PlayerId::NONE,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_ZHUGERUOXUE => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveZhugeruoxue {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                card: engine_api::CardId::new(card),
                action,
                remaining: 0,
                used_mask: 0,
                repeated: false,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_CAOYI => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let helper_skill_def = pop(frame)?.0 as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveCaoyi {
                player: PlayerId::new(player),
                card: engine_api::CardId::new(card),
                helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                action,
                event_targets: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_MALINGLI => {
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let helper_skill_def = pop(frame)?.0 as u32;
            let card = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMalingli {
                player: PlayerId::new(player),
                card: engine_api::CardId::new(card),
                helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                action,
                source: PlayerId::NONE,
                target: PlayerId::NONE,
                nature: 0,
                damage_id: 0,
                event_cards: Vec::new(),
                event_targets: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_NSSHUAIYAN => {
            let recovered_player = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveNsshuaiyan {
                player: PlayerId::new(player),
                recovered_player: PlayerId::new(recovered_player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::GUOZHAN_USE_ZHULIAN_BIHE => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::GuozhanUseZhulianBihe {
                player: PlayerId::new(player),
            }
        }
        effect_tag::RESOLVE_SHOULIJIAN => {
            let skill = pop(frame)?.0.max(0) as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveShoulijian {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                skill: engine_api::SkillId::new(skill),
            }
        }
        effect_tag::CANCEL_INCOMING_CARD_USE => {
            let card = pop(frame)?.0.max(0) as u32;
            let source = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::CancelIncomingCardUse {
                player: PlayerId::new(player),
                source: PlayerId::new(source),
                card: engine_api::CardId::new(card),
                skill: engine_api::SkillId::new(host.context().skill_id),
            }
        }
        effect_tag::CHOOSE_OTHER_GAIN_ONE_OF_TWO_SKILLS => {
            let second_def = pop(frame)?.0 as u32;
            let first_def = pop(frame)?.0 as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseOtherGainOneOfTwoSkills {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                first_def: engine_api::SkillDefId::new(first_def),
                second_def: engine_api::SkillDefId::new(second_def),
                skill: engine_api::SkillId::new(host.context().skill_id),
            }
        }
        effect_tag::GWYEWU_RANDOM_DISCARD_CHAIN => {
            let lang_def = pop(frame)?.0 as u32;
            let wuyao_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveGwyewuChain {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::new(host.context().skill_id),
                cost_cards: host
                    .selected_cards()
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                wuyao_def: engine_api::SkillDefId::new(wuyao_def),
                lang_def: engine_api::SkillDefId::new(lang_def),
            }
        }
        effect_tag::RESOLVE_XIONGLUAN => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveXiongluan {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_SBBIYUE => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveSbbiyue {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_SXRMQIEWANG => {
            let event_player = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveSxrmqiewang {
                player: PlayerId::new(player),
                event_player: PlayerId::new(event_player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_SXRMQIANLIU => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let source = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveSxrmqianliu {
                player: PlayerId::new(player),
                source: PlayerId::new(source),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::SET_DRAW_PHASE_FROM_BOTTOM => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::SetDrawPhaseFromBottom {
                player: PlayerId::new(player),
            }
        }
        effect_tag::DRAW_CARDS_FROM_BOTTOM => {
            let count = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::DrawCardsFromBottom {
                player: PlayerId::new(player),
                count,
            }
        }
        effect_tag::RESOLVE_CHENGLVE => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveChenglve {
                player: PlayerId::new(player),
                stage,
                suits: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_SHICAI => {
            let stage = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let card = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveShicai {
                player: PlayerId::new(player),
                card: engine_api::CardId::new(card),
                placed: Vec::new(),
                stage,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_TARGETS_DRAW => {
            let scope = u8::try_from(pop(frame)?.0)
                .ok()
                .and_then(DrawTargetScope::from_tag)
                .ok_or_else(|| VmError::Host("invalid draw target scope".into()))?;
            let target_count = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let count = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseTargetsDraw {
                player: PlayerId::new(player),
                count,
                target_count,
                scope,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_TARGETS_DAMAGE => {
            let amount = pop(frame)?.0.max(0) as u32;
            let scope = u8::try_from(pop(frame)?.0)
                .ok()
                .and_then(DrawTargetScope::from_tag)
                .ok_or_else(|| VmError::Host("invalid damage target scope".into()))?;
            let max_count = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let min_count = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseTargetsDamage {
                player: PlayerId::new(player),
                min_count,
                max_count,
                scope,
                amount,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_OPTIONAL_TARGETS_DRAW => {
            let scope = u8::try_from(pop(frame)?.0)
                .ok()
                .and_then(DrawTargetScope::from_tag)
                .ok_or_else(|| VmError::Host("invalid draw target scope".into()))?;
            let max_count = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let min_count = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let count = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseOptionalTargetsDraw {
                player: PlayerId::new(player),
                count,
                min_count,
                max_count,
                scope,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::REVEAL_TARGET_HAND_SHA_OR_DISCARD_BIND => {
            let helper_skill_def = pop(frame)?.0 as u32;
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::RevealTargetHandShaOrDiscardBind {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                helper_skill_def: rule_api::SkillDefId::new(helper_skill_def),
                skill: engine_api::SkillId::NONE,
                stage: 0,
            }
        }
        effect_tag::DRAW_REMEMBERED_HAND_CARDS => {
            let count = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::DrawRememberedHandCards {
                player: PlayerId::new(player),
                count,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::DISCARD_REMEMBERED_HAND_CARDS => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::DiscardRememberedHandCards {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::GAIN_DRAW_CARDS_FOR_LIFETIME_CARD_ACTION_DIVISORS => {
            let third_group = pop(frame)?.0.max(0) as u8;
            let third_divisor = pop(frame)?.0.max(0) as u32;
            let second_group = pop(frame)?.0.max(0) as u8;
            let second_divisor = pop(frame)?.0.max(0) as u32;
            let first_group = pop(frame)?.0.max(0) as u8;
            let first_divisor = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::GainDrawCardsForLifetimeCardActionDivisors {
                player: PlayerId::new(player),
                divisors: [first_divisor, second_divisor, third_divisor],
                groups: [first_group, second_group, third_group],
            }
        }
        effect_tag::LOSE_HP_GAIN_DISCARD_CARD_FOR_OPENING_EXTRA_PLAY => {
            let mandatory_skill_def = pop(frame)?.0 as u32;
            let kind = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::LoseHpGainDiscardCardForOpeningExtraPlay {
                player: PlayerId::new(player),
                kind,
                mandatory_skill_def: engine_api::SkillDefId::new(mandatory_skill_def),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_GAIN_OWN_LOST_DISCARD_CARD_THIS_TURN => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseGainOwnLostDiscardCardThisTurn {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_MOVE_FIELD_CARD_TO => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseMoveFieldCardTo {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::DRAW_THEN_OPTIONALLY_DISTRIBUTE_HAND_CARDS => {
            let give_cap = pop(frame)?.0.max(0) as u32;
            let draw_count = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::DrawThenOptionallyDistributeHandCards {
                player: PlayerId::new(player),
                draw_count,
                give_cap,
                cards: Vec::new(),
                targets: Vec::new(),
                skill: engine_api::SkillId::NONE,
                ready_to_choose: false,
                finish: false,
            }
        }
        effect_tag::GIVE_SELECTED_THEN_CONTINUE_DISTINCT_RECOVER_AT_TWO => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::GiveSelectedThenContinueDistinctRecoverAtTwo {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                cards: host
                    .selected_cards()
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                used_targets: Vec::new(),
                total_given: 0,
                skill: engine_api::SkillId::NONE,
                continue_only: false,
            }
        }
        effect_tag::CHOOSE_MAX_MARK_OTHER_JUDGE_KILL_UNLESS_TAO_OR_TAOYUAN => {
            let mark = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan {
                player: PlayerId::new(player),
                mark: engine_api::MarkDefId::new(mark),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_ZONE_PRIORITY_GAIN_RANDOM_CARD_FROM_EACH_OTHER_THEN_TURN_OVER => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver {
                player: PlayerId::new(player),
                priority: u8::MAX,
                remaining_targets: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_GAIN_CARD_FROM_EACH_OTHER_THEN_TURN_OVER => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseGainCardFromEachOtherThenTurnOver {
                player: PlayerId::new(player),
                initialized: false,
                remaining_targets: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_HIDDEN_SKILL_PILE => {
            let card = pop(frame)?.0.max(0) as u32;
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let pile_skill_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveHiddenSkillPile {
                player: PlayerId::new(player),
                pile_skill_def: engine_api::SkillDefId::new(pile_skill_def),
                action,
                card: engine_api::CardId::new(card),
                stage: 0,
                hand_cards: Vec::new(),
                pile_cards: Vec::new(),
                event_cards: host
                    .event_cards()
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::SET_NON_LOCKED_SKILL_SUPPRESSION => {
            let enabled = pop(frame)?.0 != 0;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::SetNonLockedSkillSuppression {
                player: PlayerId::new(player),
                enabled,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DILI_YUHENG => {
            let card = pop(frame)?.0.max(0) as u32;
            let action = pop(frame)?.0.clamp(0, u8::MAX as i32) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDiliYuheng {
                player: PlayerId::new(player),
                action,
                card: engine_api::CardId::new(card),
                event_cards: host
                    .event_cards()
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::DISCARD_SELECTED_DRAW_ALL_HAND_BONUS => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::DiscardSelectedDrawAllHandBonus {
                player: PlayerId::new(player),
                cards: host
                    .selected_cards()
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_OTHER_YINGHUN => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseOtherYinghun {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                count: 0,
                branch: 0,
                ready_to_discard: false,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_OTHER_BIZHENG => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseOtherBizheng {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                stage: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_OTHER_XIASHU => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseOtherXiashu {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                stage: 0,
                cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_GUANCHAO_RANK => {
            let card = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveGuanchaoRank {
                player: PlayerId::new(player),
                card: engine_api::CardId::new(card),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_GUANWEI => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveGuanwei {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                stage: 0,
                card: engine_api::CardId::NONE,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_LANJIANG => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveLanjiang {
                player: PlayerId::new(player),
                stage: 0,
                index: 0,
                target: PlayerId::NONE,
                targets: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_OLDIMENG => {
            let action = pop(frame)?.0.clamp(0, i32::from(u8::MAX)) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveOldimeng {
                player: PlayerId::new(player),
                stage: if action == 0 { 0 } else { 2 },
                first: PlayerId::NONE,
                second: PlayerId::NONE,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_DRLT_JUEYAN => {
            let treasure_skill_def = pop(frame)?.0 as u32;
            let armor_skill_def = pop(frame)?.0 as u32;
            let mount_skill_def = pop(frame)?.0 as u32;
            let weapon_skill_def = pop(frame)?.0 as u32;
            let sha_skill_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveDrltJueyan {
                player: PlayerId::new(player),
                stage: 0,
                slot: 0,
                first: false,
                sha_skill_def: engine_api::SkillDefId::new(sha_skill_def),
                weapon_skill_def: engine_api::SkillDefId::new(weapon_skill_def),
                mount_skill_def: engine_api::SkillDefId::new(mount_skill_def),
                armor_skill_def: engine_api::SkillDefId::new(armor_skill_def),
                treasure_skill_def: engine_api::SkillDefId::new(treasure_skill_def),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_JIEXUN => {
            let funan_helper_skill_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveJiexun {
                player: PlayerId::new(player),
                stage: 0,
                target: PlayerId::NONE,
                draw_count: 0,
                discard_count: 0,
                funan_helper_skill_def: engine_api::SkillDefId::new(funan_helper_skill_def),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_WENGUA_SETUP => {
            let helper_skill_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveWenguaSetup {
                player: PlayerId::new(player),
                helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_WENGUA => {
            let marker_skill_def = pop(frame)?.0 as u32;
            let main_skill_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveWengua {
                player: PlayerId::new(player),
                stage: 0,
                target: PlayerId::NONE,
                card: host
                    .selected_cards()
                    .first()
                    .copied()
                    .map(engine_api::CardId::new)
                    .unwrap_or(engine_api::CardId::NONE),
                branch: 0,
                main_skill_def: engine_api::SkillDefId::new(main_skill_def),
                marker_skill_def: engine_api::SkillDefId::new(marker_skill_def),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_GANGLIE_THREE => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveGanglieThree {
                player: PlayerId::new(player),
                target: PlayerId::NONE,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_ZHONGYI => {
            let helper_skill_def = pop(frame)?.0 as u32;
            let action = pop(frame)?.0 as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveZhongyi {
                player: PlayerId::new(player),
                action,
                helper_skill_def: engine_api::SkillDefId::new(helper_skill_def),
                card: host
                    .selected_cards()
                    .first()
                    .copied()
                    .map(engine_api::CardId::new)
                    .unwrap_or(engine_api::CardId::NONE),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::DISCARD_EQUIPMENT_SLOT => {
            let slot = pop(frame)?.0 as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::DiscardEquipmentSlot {
                player: PlayerId::new(player),
                slot,
            }
        }
        effect_tag::ADD_SKILL_UNTIL_TURN_END => {
            let skill_def = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::AddSkillUntilTurnEnd {
                player: PlayerId::new(player),
                skill_def: engine_api::SkillDefId::new(skill_def),
            }
        }
        effect_tag::RESOLVE_MASS_DAMAGE_DISCARD_TURN_OVER => {
            let hand_count = pop(frame)?.0.max(0) as u32;
            let damage_amount = pop(frame)?.0.max(0) as u32;
            let mark_count = pop(frame)?.0.max(0) as u32;
            let mark = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMassDamageDiscardTurnOver {
                player: PlayerId::new(player),
                mark: engine_api::MarkDefId::new(mark),
                mark_count,
                damage_amount,
                hand_count,
                stage: 0,
                index: 0,
                targets: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_MASS_DAMAGE_DISCARD_HAND_EQUIP => {
            let discard_count = pop(frame)?.0.max(0) as u32;
            let damage_amount = pop(frame)?.0.max(0) as u32;
            let mark_count = pop(frame)?.0.max(0) as u32;
            let mark = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMassDamageDiscardHandEquip {
                player: PlayerId::new(player),
                mark: engine_api::MarkDefId::new(mark),
                mark_count,
                damage_amount,
                discard_count,
                stage: 0,
                index: 0,
                targets: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RECORD_ACTIVE_TARGET => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::RecordActiveTarget {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_DISCARD_WEAPON_OR_LOSE_HP => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseDiscardWeaponOrLoseHp {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_MARK_PARITY_CONTROL => {
            let damage_amount = pop(frame)?.0.max(0) as u32;
            let mass_threshold = pop(frame)?.0.max(0) as u32;
            let mark = pop(frame)?.0 as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveMarkParityControl {
                player: PlayerId::new(player),
                mark: engine_api::MarkDefId::new(mark),
                mass_threshold,
                damage_amount,
                stage: 0,
                index: 0,
                target: PlayerId::NONE,
                targets: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_SELECTED_EQUIPMENT_DISCARD_THEN_CHOOSE_DAMAGE => {
            let nature = pop(frame)?.0.max(0) as u8;
            let damage_amount = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveSelectedEquipmentDiscardThenChooseDamage {
                player: PlayerId::new(player),
                damage_amount,
                nature,
                stage: 0,
                index: 0,
                targets: host
                    .selected_targets()
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::REPLACE_DRAW_WITH_DISTINCT_SUIT_SELECTION => {
            let count = pop(frame)?.0.max(0) as u8;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ReplaceDrawWithDistinctSuitSelection {
                player: PlayerId::new(player),
                count,
                stage: 0,
                cards: Vec::new(),
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::INSPECT_TARGET_HAND_CHOOSE_HEART_DISCARD_OR_TOP => {
            let target = pop(frame)?.0.max(0) as u32;
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::InspectTargetHandChooseHeartDiscardOrTop {
                player: PlayerId::new(player),
                target: PlayerId::new(target),
                stage: 0,
                card: engine_api::CardId::NONE,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::RESOLVE_YEYAN => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ResolveYeyan {
                player: PlayerId::new(player),
                mode: u8::MAX,
                stage: 0,
                index: 0,
                targets: Vec::new(),
                data: 0,
                skill: engine_api::SkillId::NONE,
            }
        }
        effect_tag::CHOOSE_ALL_LOSE_HP_OR_RECOVER => {
            let player = pop(frame)?.0.max(0) as u32;
            EffectRequest::ChooseAllLoseHpOrRecover {
                player: PlayerId::new(player),
                skill: engine_api::SkillId::NONE,
            }
        }
        _ => return super::resolve_huzun_packs::decode_request(host, frame, tag),
    })
}
