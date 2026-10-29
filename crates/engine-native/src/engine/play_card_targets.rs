//! 扩展牌（机关、灵剑、古剑食物）出牌目标校验——`play_card` 牌种分派的只读分支。

use super::*;

/// 机关牌（swd 第十六~十八批）逐卡目标规则。
pub(super) fn validate_jiguan_targets(
    core: &GameStateCore,
    catalog: &SharedCatalog,
    card: CardId,
    targets: &[PlayerId],
) -> Result<(), RejectReason> {
    // 机关牌（swd 第十六~十八批，规则 v189/v190/v191）：
    // 指路小虎/木甲人=自目标（上游 selectTarget -1+
    // filterTarget target===player 同口径）；木甲人
    // usable:1 每回合一次；勾魂锣=恰 1 名无挂附目标
    // （上游 filterTarget !hasSkill("gouhunluo") 同口径）
    if catalog.cards[card as usize].name.contains("jiguantong") {
        // 机关桶：AOE=全部其他存活角色（上游 selectTarget
        // -1 target!==player 同口径；结算序=座位序登记差异）
        let expected: Vec<u8> = core
            .seat_order_from(core.turn.current)
            .into_iter()
            .filter(|p| *p != core.turn.current && core.player(*p).alive)
            .collect();
        if targets != expected {
            return Err(RejectReason::IllegalTargets);
        }
    } else if catalog.cards[card as usize].name.contains("gouhunluo") {
        if targets.len() != 1 || !core.player(targets[0]).alive {
            return Err(RejectReason::IllegalTargets);
        }
        if core.player(targets[0]).gouhunluo_rounds > 0 {
            return Err(RejectReason::IllegalTargets);
        }
    } else if catalog.cards[card as usize].name.contains("jiguanfeng") {
        // 机关凤（swd 第二十批，规则 v195）：恰 1 名其他
        // 角色（上游 filterTarget target!==player 同口径）
        if targets.len() != 1
            || targets[0] == core.turn.current
            || !core.player(targets[0]).alive
        {
            return Err(RejectReason::IllegalTargets);
        }
    } else if catalog.cards[card as usize].name.contains("jiguanyaoshu") {
        // 机关妖术（swd 第二十二批，规则 v197）：恰 1 名其
        // 他存活角色（range.global 1 无距离约束）；目标有空
        // 装备栏（上游 filterTarget 空 subtype 槽同口径）、
        // 无防重标记（上游 !hasSkill 同口径）、非 isMin
        //（手牌数>体力，上游 !isMin 同口径）
        if targets.len() != 1 || !core.player(targets[0]).alive {
            return Err(RejectReason::IllegalTargets);
        }
        let t = targets[0];
        let marker = catalog
            .skill_by_name(crate::content::host::SWD_JIGUANYAOSHU_STATE_KEY);
        if marker.is_some_and(|d| core.skills.contains_key(&(t, d))) {
            return Err(RejectReason::IllegalTargets);
        }
        let hand = core.player(t).hand.len() as i32;
        if hand <= core.player(t).hp {
            return Err(RejectReason::IllegalTargets);
        }
        let occupied: Vec<Option<crate::catalog::EquipSlot>> = core
            .player(t)
            .equips
            .iter()
            .map(|c| catalog.card_slot_of(core, *c))
            .collect();
        if occupied.len() >= 5 {
            return Err(RejectReason::IllegalTargets);
        }
    } else if catalog.cards[card as usize].name.contains("jiguanshu") {
        // 机关术（swd 机关术批，规则 v199/ADR 0780）：自目
        // 标（上游 filterTarget target===player 同口径）；
        // 至少一张有效装备
        if targets.len() != 1 || targets[0] != core.turn.current {
            return Err(RejectReason::IllegalTargets);
        }
        if core.player(core.turn.current).equips.is_empty() {
            return Err(RejectReason::IllegalTargets);
        }
    } else if catalog.cards[card as usize].name.contains("jiguanyuan") {
        // 机关怨（swd 第二十一批，规则 v196）：恰 1 名其他
        // 角色（上游 filterTarget target!==player+
        // !hasSkill("jiguanyuan") 同口径）；enable 要求除本
        // 体至少还有一张手牌（上游 enable 同口径）
        if targets.len() != 1
            || targets[0] == core.turn.current
            || !core.player(targets[0]).alive
            || !core.player(targets[0]).jiguanyuan_cards.is_empty()
        {
            return Err(RejectReason::IllegalTargets);
        }
        let hand = core.player(core.turn.current).hand.clone();
        if hand.is_empty()
            || (hand.len() == 1 && hand[0] == card)
        {
            return Err(RejectReason::CardLimit);
        }
    } else {
        if targets != [core.turn.current] {
            return Err(RejectReason::IllegalTargets);
        }
        if catalog.cards[card as usize].name.contains("mujiaren")
            && core.turn.mujiaren_used
        {
            return Err(RejectReason::CardLimit);
        }
    }
    Ok(())
}

/// 灵剑（swd 机关术批）逐卡目标规则。
pub(super) fn validate_lingjian_targets(
    core: &GameStateCore,
    catalog: &SharedCatalog,
    card: CardId,
    targets: &[PlayerId],
) -> Result<(), RejectReason> {
    // 灵剑（swd 机关术批，规则 v198）：逐卡目标规则（上游
    // 各卡 filterTarget 同口径）
    let cname = &catalog.cards[card as usize].name;
    let t_ok = if cname.contains("xingtigaizao") {
        targets.len() == 1 && targets[0] == core.turn.current
    } else if cname.contains("shijianhuisu") {
        targets.len() == 1
            && targets[0] != core.turn.current
            && core.player(targets[0]).alive
            && !core.player(targets[0]).equips.is_empty()
    } else if cname.contains("jinjilengdong") {
        targets.len() == 1
            && targets[0] != core.turn.current
            && core.player(targets[0]).alive
            && !core.player(targets[0]).face_down
    } else if cname.contains("yinmilichang") {
        targets.len() == 1
            && targets[0] != core.turn.current
            && core.player(targets[0]).alive
            && !core.player(targets[0]).yinmi_active
            && !core.player(targets[0]).yinshen_active
    } else if cname.contains("shengxiuhaojiao") {
        targets.len() == 1
            && core.player(targets[0]).alive
            && !core.player(targets[0]).chaofeng_active
    } else if cname.contains("zhongxinghujia") {
        targets.len() == 1
            && core.player(targets[0]).alive
            && core.player(targets[0]).hand.len() as i32
                > core.player(targets[0]).hp
    } else {
        targets.len() == 1
            && core.player(targets[0]).alive
            && (!core.player(targets[0]).hand.is_empty()
                || !core.player(targets[0]).equips.is_empty())
    };
    if !t_ok {
        return Err(RejectReason::IllegalTargets);
    }
    Ok(())
}

/// 古剑食物：恰 1 名存活目标（可含自己），且目标尚未持有同名食物效果。
pub(super) fn validate_gujian_food_targets(
    core: &GameStateCore,
    catalog: &SharedCatalog,
    kind: CardKind,
    targets: &[PlayerId],
) -> Result<(), RejectReason> {
    // 古剑食物：恰 1 名存活目标（可含自己），且目标尚
    // 未持有同名状态（登记差异：上游按技能名过滤）
    if targets.len() != 1 || !core.player(targets[0]).alive {
        return Err(RejectReason::IllegalTargets);
    }
    let listener = match kind {
        CardKind::Tanhuadong => crate::content::host::GJ_FOOD_TANHUADONG_KEY,
        CardKind::Qingtuan => crate::content::host::GJ_FOOD_QINGTUAN_KEY,
        CardKind::Yuanbaorou => crate::content::host::GJ_FOOD_YUANBAOROU_KEY,
        CardKind::Gudonggeng => crate::content::host::GJ_FOOD_GUDONGGENG_KEY,
        CardKind::Mapodoufu => crate::content::host::GJ_FOOD_MAPODOUFU_KEY,
        CardKind::Chunbing => crate::content::host::GJ_FOOD_CHUNBING_KEY,
        CardKind::Liyutang => crate::content::host::GJ_FOOD_LIYUTANG_KEY,
        CardKind::Molicha => crate::content::host::GJ_FOOD_MOLICHA_KEY,
        CardKind::Xiajiao => crate::content::host::GJ_FOOD_XIAJIAO_KEY,
        CardKind::Mizhilianou => crate::content::host::GJ_FOOD_MIZHLIANOU_KEY,
        CardKind::Luyugeng => crate::content::host::GJ_FOOD_LUYUGENG_KEY,
        CardKind::Yunvyuanshen => crate::content::host::GJ_YUNVYUANSHEN_LISTENER_KEY,
        _ => crate::content::host::GJ_FOOD_YOUGENG_KEY,
    };
    if let Some(def) = catalog.skill_by_name(listener) {
        if core.skills.contains_key(&(targets[0], def)) {
            return Err(RejectReason::IllegalTargets);
        }
    }
    Ok(())
}
