//! 扩展牌（机关、灵剑、古剑食物）出牌目标校验——`play_card` 牌种分派的只读分支。

use super::*;

#[path = "play_card_targets_hearth.rs"]
mod hearth;
#[path = "play_card_targets_gwent.rs"]
mod gwent;

pub(super) use gwent::clean_gwent_name;

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
    } else if catalog.cards[card as usize].name.contains("yanjiadan")
        || catalog.cards[card as usize].name.contains("lingjiandai")
    {
        if !(targets.is_empty()
            || (targets.len() == 1 && core.player(targets[0]).alive))
        {
            return Err(RejectReason::IllegalTargets);
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

fn strip_hearth_base(raw: &str) -> &str {
    let s = raw
        .strip_prefix("hs_")
        .or_else(|| raw.strip_prefix("ht_"))
        .unwrap_or(raw);
    if let Some((stem, tail)) = s.rsplit_once('_') {
        if !tail.is_empty() && tail.bytes().all(|b| b.is_ascii_digit()) {
            return stem;
        }
    }
    s
}

pub(super) fn is_hearth_formal(catalog: &SharedCatalog) -> bool {
    catalog.skill_by_name("shihuawuqi").is_some_and(|d| {
        catalog.skills[d as usize]
            .behavior
            .as_host_behavior()
            .is_some()
    }) || catalog.skill_by_name("hshuanyu").is_some_and(|d| {
        catalog.skills[d as usize]
            .behavior
            .as_host_behavior()
            .is_some()
    })
}

pub(super) fn is_shihuawuqi_card(
    core: &GameStateCore,
    catalog: &SharedCatalog,
    user: PlayerId,
    card: CardId,
) -> bool {
    let eff_card = if catalog
        .cards
        .get(card as usize)
        .is_some_and(|c| strip_hearth_base(c.name) == "hsfashu_anyingjingxiang")
        && core.declared_kinds.get(&card).copied() == Some(CardKind::Tao)
    {
        core.vm_aux
            .flags
            .get(&(user, format!("hs_wxuying_src_card:{card}")))
            .and_then(|&v| (v > 0).then(|| (v - 1) as CardId))
            .unwrap_or(card)
    } else {
        card
    };
    catalog.cards.get(eff_card as usize).is_some_and(|c| strip_hearth_base(c.name) == "shihuawuqi")
        && catalog.skill_by_name("shihuawuqi").is_some_and(|d| {
            catalog.skills[d as usize].behavior.as_host_behavior().is_some()
        })
}

pub(super) fn validate_wuzhong_targets(
    core: &GameStateCore,
    catalog: &SharedCatalog,
    card: CardId,
    kind: CardKind,
    targets: &[PlayerId],
) -> Result<(), RejectReason> {
    if kind == CardKind::Wuzhong {
        let cname = catalog.cards.get(card as usize).map(|c| c.name).unwrap_or("");
        if kind == catalog.resolved_card_kind(core, card) {
            let actor = core.turn.current;
            let unseen_in_kingdoms = |p: PlayerId| -> bool {
                matches!(core.mode, crate::state::GameMode::Kingdoms)
                    && core.player(p).revealed_generals.iter().any(|&r| !r)
            };
            match cname {
                "txhj_zhijizhibi" => {
                    if targets.is_empty() {
                        return Ok(());
                    }
                    if targets.len() == 1 {
                        let t = targets[0];
                        if t != actor
                            && core.player(t).alive
                            && (!core.player(t).hand.is_empty() || unseen_in_kingdoms(t))
                        {
                            return Ok(());
                        }
                    }
                    return Err(RejectReason::IllegalTargets);
                }
                "txhj_yuanjiao" | "txhj_diaohu" => {
                    if targets.len() == 1 && targets[0] != actor && core.player(targets[0]).alive {
                        return Ok(());
                    }
                    return Err(RejectReason::IllegalTargets);
                }
                "txhj_yiyi" => {
                    if targets.is_empty() {
                        return Ok(());
                    }
                    let mut seen = std::collections::BTreeSet::new();
                    if targets.iter().all(|&t| {
                        seen.insert(t) && core.player(t).alive && core.is_friend_of(catalog, actor, t)
                    }) {
                        return Ok(());
                    }
                    return Err(RejectReason::IllegalTargets);
                }
                "txhj_shengdong" => {
                    if targets.is_empty() {
                        return Ok(());
                    }
                    if targets.len() == 2
                        && targets[0] != actor
                        && core.player(targets[0]).alive
                        && targets[1] != targets[0]
                        && core.player(targets[1]).alive
                    {
                        return Ok(());
                    }
                    return Err(RejectReason::IllegalTargets);
                }
                "huogong" => {
                    if targets.len() == 1
                        && core.player(targets[0]).alive
                        && !core.player(targets[0]).hand.is_empty()
                    {
                        return Ok(());
                    }
                    return Err(RejectReason::IllegalTargets);
                }
                "txzhaoshu_skill" => {
                    if targets.is_empty() || (targets.len() == 1 && targets[0] == actor) {
                        return Ok(());
                    }
                    return Err(RejectReason::IllegalTargets);
                }
                "txzhaoshu_global" => {
                    if targets.is_empty() || (targets.len() == 1 && core.player(targets[0]).alive) {
                        return Ok(());
                    }
                    return Err(RejectReason::IllegalTargets);
                }
                _ => {}
            }
        }
        if cname != "无中生有" && cname != "wuzhong" && !cname.starts_with("wuzhong_") {
            if cname == "yiyi" || cname.ends_with("__ref_yiyi") {
                let (auto_allies, candidates) =
                    core.yiyi_target_candidates(catalog, core.turn.current);
                let mut given = std::collections::BTreeSet::new();
                for &t in targets {
                    if !given.insert(t) || !candidates.contains(&t) {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                if auto_allies {
                    let expected: std::collections::BTreeSet<PlayerId> =
                        candidates.into_iter().collect();
                    if targets.is_empty() || given == expected {
                        return Ok(());
                    }
                    return Err(RejectReason::IllegalTargets);
                }
                if (1..=3).contains(&targets.len())
                    || (targets.is_empty()
                        && catalog
                            .skill_by_name(crate::content::host::L01_FIRST_KS_KEY)
                            .is_none())
                {
                    return Ok(());
                }
                return Err(RejectReason::IllegalTargets);
            }
            if let Some(res) =
                hearth::validate_hearth_wuzhong_targets(core, catalog, card, cname, targets)
            {
                return res;
            }
            if cname.contains("diaobingqianjiang") {
                let actor = core.turn.current;
                let expected: std::collections::BTreeSet<PlayerId> = (0..core.players.len()
                    as PlayerId)
                    .filter(|&p| {
                        core.player(p).alive && (p == actor || !core.player(p).hand.is_empty())
                    })
                    .collect();
                let given: std::collections::BTreeSet<PlayerId> = targets.iter().copied().collect();
                if targets.is_empty() || (given.len() == targets.len() && given == expected) {
                    return Ok(());
                }
                return Err(RejectReason::IllegalTargets);
            }
            let clean = gwent::clean_gwent_name(cname);
            if clean == "gwjinli_jiu"
                || clean.starts_with("gw_")
                || clean.starts_with("gwmaoxian_")
            {
                return gwent::validate_u05_gwent_targets(core, catalog, clean, targets);
            }
            if targets.is_empty() || (targets.len() == 1 && core.player(targets[0]).alive) {
                return Ok(());
            }
            return Err(RejectReason::IllegalTargets);
        }
    }
    if !targets.is_empty() {
        return Err(RejectReason::IllegalTargets);
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

/// 扩展牌（古剑、炉石、昆特、苍梧/神兵族）出牌目标校验——自 `play_card.rs` 拆分。
pub(super) fn validate_tail_ext_targets(
    core: &GameStateCore,
    catalog: &SharedCatalog,
    card: CardId,
    kind: CardKind,
    targets: &[PlayerId],
) -> Result<(), RejectReason> {
    match kind {
        CardKind::Feibiao => {
            if targets.len() != 1
                || targets[0] == core.turn.current
                || !core.player(targets[0]).alive
                || (!crate::rvm_bridge::modifiers::eval_card_distance_unlimited(
                    core,
                    catalog,
                    core.turn.current,
                    card,
                    CardKind::Feibiao,
                    crate::event::DamageNature::Normal,
                ) && core.computed_distance(catalog, core.turn.current, targets[0]) < 2)
            {
                return Err(RejectReason::IllegalTargets);
            }
        }
        CardKind::Jinlianzhu => {
            return Err(RejectReason::IllegalTargets);
        }
        CardKind::Ziyangdan => {
            if targets.len() != 1 || !core.player(targets[0]).alive {
                return Err(RejectReason::IllegalTargets);
            }
        }
        CardKind::Shihuifen => {
            if targets.len() != 1
                || targets[0] == core.turn.current
                || !core.player(targets[0]).alive
            {
                return Err(RejectReason::IllegalTargets);
            }
        }
        CardKind::Liufengsan => {
            if targets.len() != 1 || !core.player(targets[0]).alive {
                return Err(RejectReason::IllegalTargets);
            }
        }
        CardKind::Shatang | CardKind::Liutouge => {
            if targets.len() != 1 || !core.player(targets[0]).alive {
                return Err(RejectReason::IllegalTargets);
            }
        }
        CardKind::Heilonglinpian => {
            if !targets.is_empty() {
                return Err(RejectReason::IllegalTargets);
            }
        }
        CardKind::Mutoumianju => {
            if !targets.is_empty() {
                return Err(RejectReason::IllegalTargets);
            }
        }
        CardKind::Dinvxuanshuang => {
            return Err(RejectReason::WrongPhase);
        }
        CardKind::Gjyuheng => {
            if !targets.is_empty() {
                return Err(RejectReason::IllegalTargets);
            }
        }
        CardKind::Dunpaigedang
        | CardKind::Xingjiegoutong
        | CardKind::Chuansongmen
        | CardKind::Tanshezhiren => {
            if !targets.is_empty() {
                return Err(RejectReason::IllegalTargets);
            }
        }
        CardKind::Zhiliaobo => {
            if targets.len() != 1
                || !core.player(targets[0]).alive
                || core.player(targets[0]).hp >= core.player(targets[0]).max_hp as i32
            {
                return Err(RejectReason::IllegalTargets);
            }
        }
        CardKind::Yuansuhuimie => {
            if !targets.is_empty() {
                return Err(RejectReason::IllegalTargets);
            }
        }
        CardKind::Shijieshu => {
            let formal = catalog.skill_by_name("shihuawuqi").is_some_and(|d| {
                catalog.skills[d as usize].behavior.as_host_behavior().is_some()
            });
            if formal {
                if targets.len() != 1 || !core.player(targets[0]).alive {
                    return Err(RejectReason::IllegalTargets);
                }
            } else if !targets.is_empty() {
                return Err(RejectReason::IllegalTargets);
            }
        }
        CardKind::Kunenfayin
        | CardKind::Poxiao
        | CardKind::Zumoshoukao
        | CardKind::Shanbengshu
        | CardKind::Yanziyaoshui
        | CardKind::Aozuzhilei
        | CardKind::Wenyi
        | CardKind::Zhihuanjun
        | CardKind::Baoxueyaoshui
        | CardKind::Liuxinghuoyu
        | CardKind::Qinpendayu
        | CardKind::Birinongwu
        | CardKind::Ciguhanshuang => {
            gwent::validate_u05_gwent_bronze_targets(core, catalog, kind, targets)?;
        }
        CardKind::Jiguan => validate_jiguan_targets(core, catalog, card, targets)?,
        CardKind::Hslingjian => validate_lingjian_targets(core, catalog, card, targets)?,
        CardKind::Bingpotong => {
            if targets.is_empty() || targets.len() > 3 {
                return Err(RejectReason::IllegalTargets);
            }
            for t in targets {
                if !core.player(*t).alive || core.player(*t).hand.is_empty() {
                    return Err(RejectReason::IllegalTargets);
                }
            }
        }
        CardKind::Qiankunbiao | CardKind::Shenhuofeiya => {
            if targets.len() != 1
                || targets[0] == core.turn.current
                || !core.player(targets[0]).alive
                || (kind == CardKind::Qiankunbiao
                    && core.player(targets[0]).hand.is_empty()
                    && core.player(targets[0]).equips.is_empty())
            {
                return Err(RejectReason::IllegalTargets);
            }
        }
        CardKind::Longxugou => {
            if targets.len() != 1
                || targets[0] == core.turn.current
                || !core.player(targets[0]).alive
                || core.player(targets[0]).equips.is_empty()
            {
                return Err(RejectReason::IllegalTargets);
            }
        }
        CardKind::Shuigong => {
            let is_tx_shuiyan = kind == catalog.resolved_card_kind(core, card)
                && catalog
                    .cards
                    .get(card as usize)
                    .is_some_and(|cd| cd.name == "txhj_shuiyan");
            if is_tx_shuiyan {
                if targets.len() != 1
                    || targets[0] == core.turn.current
                    || !core.player(targets[0]).alive
                {
                    return Err(RejectReason::IllegalTargets);
                }
            } else {
                // 水攻：所有有装备的存活角色（selectTarget: -1 全选）
                let expected: Vec<PlayerId> = core
                    .alive_players()
                    .filter(|&p| !core.player(p).equips.is_empty())
                    .collect();
                if expected.is_empty()
                    || targets.len() != expected.len()
                    || !expected.iter().all(|t| targets.contains(t))
                {
                    return Err(RejectReason::IllegalTargets);
                }
            }
        }
        CardKind::Fudichouxin => {
            if targets.len() != 1 {
                return Err(RejectReason::IllegalTargets);
            }
            let target = targets[0];
            if target == core.turn.current
                || !core.player(target).alive
                || core.player(target).hand.is_empty()
                || core.player(core.turn.current).hand.is_empty()
            {
                return Err(RejectReason::IllegalTargets);
            }
        }
        CardKind::Chenhuodajie => {
            if targets.len() != 1 {
                return Err(RejectReason::IllegalTargets);
            }
            let target = targets[0];
            if target == core.turn.current
                || !core.player(target).alive
                || (core.player(target).hand.is_empty()
                    && core.player(target).equips.is_empty())
            {
                return Err(RejectReason::IllegalTargets);
            }
        }
        CardKind::Zhaomingdan => {
            if targets.len() != 1 {
                return Err(RejectReason::IllegalTargets);
            }
            let target = targets[0];
            let tp = core.player(target);
            if target == core.turn.current
                || !tp.alive
                || (tp.hand.is_empty() && tp.equips.is_empty() && tp.judge.is_empty())
            {
                return Err(RejectReason::IllegalTargets);
            }
        }
        CardKind::Caochuanjiejian => {
            if targets.len() != 1 {
                return Err(RejectReason::IllegalTargets);
            }
            let target = targets[0];
            if target == core.turn.current
                || !core.player(target).alive
                || core.player(target).hand.is_empty()
            {
                return Err(RejectReason::IllegalTargets);
            }
        }
        CardKind::Linghunzhihuo | CardKind::WangmeiZhike => {
            if targets.len() != 1 || !core.player(targets[0]).alive {
                return Err(RejectReason::IllegalTargets);
            }
        }
        CardKind::Jihuocard | CardKind::Shezhanqunru => {
            if !targets.is_empty() {
                return Err(RejectReason::IllegalTargets);
            }
        }
        CardKind::Youdishenru => return Err(RejectReason::WrongPhase),
        CardKind::Geanguanhuo => {
            if targets.len() != 2 || targets[0] == targets[1] {
                return Err(RejectReason::IllegalTargets);
            }
            for t in targets {
                if *t == core.turn.current
                    || !core.player(*t).alive
                    || core.player(*t).hand.is_empty()
                {
                    return Err(RejectReason::IllegalTargets);
                }
            }
        }
        CardKind::Toulianghuanzhu => {
            if targets.len() != 1 {
                return Err(RejectReason::IllegalTargets);
            }
            let target = targets[0];
            if target == core.turn.current
                || !core.player(target).alive
                || core.player(target).hand.is_empty()
            {
                return Err(RejectReason::IllegalTargets);
            }
        }
        CardKind::Shujinsan => {
            if targets.len() != 1
                || !core.player(targets[0]).alive
                || (core.player(targets[0]).hand.is_empty()
                    && core.player(targets[0]).equips.is_empty())
            {
                return Err(RejectReason::IllegalTargets);
            }
        }
        CardKind::Mianlijinzhen => {
            if targets.len() != 1
                || !core.player(targets[0]).alive
                || core.player(targets[0]).hp < core.player(core.turn.current).hp
            {
                return Err(RejectReason::IllegalTargets);
            }
        }
        CardKind::Tanhuadong
        | CardKind::Yougeng
        | CardKind::Qingtuan
        | CardKind::Yuanbaorou
        | CardKind::Gudonggeng
        | CardKind::Mapodoufu
        | CardKind::Chunbing
        | CardKind::Liyutang
        | CardKind::Molicha
        | CardKind::Xiajiao
        | CardKind::Mizhilianou
        | CardKind::Luyugeng
        | CardKind::Yunvyuanshen => validate_gujian_food_targets(core, catalog, kind, targets)?,
        _ => {}
    }
    Ok(())
}

/// Recast (`chongzhu: true`) `txhj_zhijizhibi` / `txhj_shengdong` when played with 0 targets.
pub(super) fn try_recast_l11_trick(
    state: &mut GameState,
    catalog: &SharedCatalog,
    card: CardId,
    kind: CardKind,
    targets: &[PlayerId],
) -> Result<bool, RejectReason> {
    let recastable = kind == CardKind::Wuzhong
        && kind == catalog.resolved_card_kind(&state.core, card)
        && targets.is_empty()
        && catalog
            .cards
            .get(card as usize)
            .is_some_and(|cd| matches!(cd.name, "txhj_zhijizhibi" | "txhj_shengdong"));
    if !recastable {
        return Ok(false);
    }
    let player = state.core.turn.current;
    if !state.core.player(player).hand.contains(&card) {
        return Err(RejectReason::IllegalTargets);
    }
    state.core.player_mut(player).hand.retain(|&c| c != card);
    if !state.core.discard.contains(&card) {
        state.core.discard.push(card);
    }
    // LIFO stack order: Draw pushed first, CardsLost pushed second so loss settles before draw.
    crate::resolution::push_frame(
        &mut state.resolution,
        EventData::Draw {
            player,
            requested: 1,
            held_cards: Vec::new(),
            terminal: crate::event::DrawTerminal::Running,
        },
    );
    crate::resolution::push_frame(
        &mut state.resolution,
        EventData::CardsLost {
            player,
            cards: vec![card],
            origins: vec![crate::event::LostOrigin::Hand],
        },
    );
    Ok(true)
}

/// 生成常规出牌 `CardUsed` 结算帧（含 view-as 转化与还侠跟踪）。
pub(super) fn push_play_card_frame(
    state: &mut GameState,
    catalog: &SharedCatalog,
    card: CardId,
    kind: CardKind,
    targets: Vec<PlayerId>,
) -> Result<(), RejectReason> {
    let player = state.core.turn.current;
    // view-as 声明（蜜链藕红桃当桃等）：转化后按声明牌种结算
    let use_as = if kind != catalog.resolved_card_kind(&state.core, card)
        || !state.core.player(player).hand.contains(&card)
    {
        Some(kind)
    } else {
        None
    };
    // 还侠跟踪（swd 屈娴，批 27）：还侠转化的杀入跟踪列（是否造
    // 成伤害由 commit_damage 置位；阶段末回收未造成伤害者）
    if use_as == Some(CardKind::Sha)
        && catalog
            .skill_by_name("huanxia")
            .map(|d| state.core.has_skill(state.core.turn.current, d))
            .unwrap_or(false)
    {
        state.core.turn.huanxia_track.push((card, false));
    }
    crate::resolution::push_frame(
        &mut state.resolution,
        EventData::CardUsed(crate::machine::card_use_as(card, player, targets, use_as)),
    );
    Ok(())
}
