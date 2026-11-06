use super::*;

pub(super) fn play_sha(
    state: &mut GameState,
    catalog: &SharedCatalog,
    card: CardId,
    targets: Vec<PlayerId>,
    kind: CardKind,
    mark_mutoumianju: bool,
) -> Result<(), RejectReason> {
    let core = &state.core;

    // 咆哮（张飞）：出牌阶段使用杀无次数限制
    let mut mark_yuanbaorou = false;
    let mut mark_huihuo = false;
    let mut mark_lingshi = false;
    let mut mark_dcdichou_eff = false;
    let player_id = core.turn.current;
    // 焚天（Boss 朱雀，规则 v236）：锁定——红色牌使用无
    // 次数限制且无距离限制（上游 mod cardUsable Infinity
    // + targetInRange=true 同口径）
    let fentian_red = {
        let red = matches!(
            catalog.cards[card as usize].suit,
            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
        );
        red && catalog
            .skill_by_name(crate::content::standard::BOSS_FENTIAN)
            .map(|def| core.has_skill(player_id, def))
            .unwrap_or(false)
    };
    let paoxiao = catalog
        .skill_by_name(crate::content::standard::PAO_XIAO)
        .map(|d| core.has_skill(core.turn.current, d))
        .unwrap_or(false);
    // 碧日浓雾（gwent 天气）：持有者不能使用杀
    if catalog
        .skill_by_name(crate::content::host::GW_BIRIN_STATE_KEY)
        .map(|def| {
            core.skills
                .get(&(core.turn.current, def))
                .map(|i| i.enabled)
                .unwrap_or(false)
        })
        .unwrap_or(false)
    {
        return Err(RejectReason::IllegalTargets);
    }
    // 堡垒哨卫（U05 ow，规则 v125）：哨卫模式杀无次数
    // 限制（上游 bshaowei_sha cardUsable Infinity 同口径）
    let baolei_sentinel = {
        let bdef = catalog.skill_by_name(crate::content::host::WUDIAN_BAOLEI_KEY);
        bdef.map(|def| {
            core.skills
                .get(&(core.turn.current, def))
                .map(|i| i.enabled)
                .unwrap_or(false)
                && core.player(core.turn.current).baolei_mode == 1
        })
        .unwrap_or(false)
    };
    // 守护（U07 ow，规则 v121）：锁定禁杀——持有者不能
    // 使用杀（上游 shouhu mod cardEnabled 同口径）
    if catalog
        .skill_by_name(crate::content::host::WUDIAN_SHOUHU_KEY)
        .map(|def| {
            core.skills
                .get(&(core.turn.current, def))
                .map(|i| i.enabled)
                .unwrap_or(false)
        })
        .unwrap_or(false)
    {
        return Err(RejectReason::IllegalTargets);
    }
    // 庆祝（Boss 明星，规则 v245）：锁定——自己回合内出牌
    // 阶段不能用【杀】，直到获得急叮（上游 mod cardEnabled
    // `!hasSkill(boss_jiding)` 同口径）
    {
        let qz = catalog
            .skill_by_name(crate::content::standard::BOSS_QINGZHU)
            .map(|def| {
                core.skills
                    .get(&(core.turn.current, def))
                    .map(|i| i.enabled)
                    .unwrap_or(false)
            })
            .unwrap_or(false);
        let jd = catalog
            .skill_by_name(crate::content::standard::BOSS_JIDING)
            .map(|def| core.has_skill(core.turn.current, def))
            .unwrap_or(false);
        if qz && !jd {
            return Err(RejectReason::IllegalTargets);
        }
    }
    // 诸葛连弩（批③-b 第四把）：装备时出杀不限次数（与
    // 咆哮同口径的锁定豁免，按装备牌种判断）。
    let has_zhuge = core
        .player(player_id)
        .equips
        .iter()
        .any(|c| catalog.card_kind(*c) == CardKind::Zhuge);
    // 龙颅山峰（mtg 首切片，规则 v100）：地图效果全局
    // 出杀上限 +1（上游 addGlobalSkill cardUsable 同口径）
    let land_sha_bonus =
        core.current_land == Some(CardKind::Longlushanfeng) && !core.turn.land_sha_used;
    // 破月（拓跋玉儿，swd 武将面批 28）：红杀不计入出
    // 牌阶段限一次（上游 cardUsable Infinity+stat 回退同
    // 口径；原生双位豁免=此处不查+kernel 不置位）
    let poyue_red_free = catalog
        .skill_by_name("poyue")
        .map(|def| core.has_skill(core.turn.current, def))
        .unwrap_or(false)
        && matches!(
            catalog.cards[card as usize].suit,
            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
        );
    if !paoxiao
        && !has_zhuge
        && !baolei_sentinel
        && !fentian_red
        && !poyue_red_free
        && core.turn.sha_used_play_phase
        && !land_sha_bonus
    {
        // 元宝肉（G1 古剑食物）：出牌阶段额外一张杀
        //（每回合一次；once_used 随回合开始重置）
        let extra = catalog
            .skill_by_name(crate::content::host::GJ_FOOD_YUANBAOROU_KEY)
            .map(|def| {
                core.skills
                    .get(&(core.turn.current, def))
                    .map(|i| i.enabled && !i.once_used)
                    .unwrap_or(false)
            })
            .unwrap_or(false);
        if !extra {
            // 回火（Boss 焰灵，规则 v241）：出牌阶段额外
            // 一张杀（每回合一次额度，元宝肉同款）
            let huihuo_extra = catalog
                .skill_by_name(crate::content::standard::BOSS_HUIHUO)
                .map(|def| {
                    core.skills
                        .get(&(core.turn.current, def))
                        .map(|i| i.enabled && !i.once_used)
                        .unwrap_or(false)
                })
                .unwrap_or(false);
            if !huihuo_extra {
                // 灵矢（swd 计亭，swd deferred 消解批 113）：
                // 锁定——装备区有宝物时回合内可额外使用一
                // 张杀（上游 mod cardUsable num+1；每回合
                // 一次额度，元宝肉同款 once_used 语义）
                let lingshi_extra = catalog
                    .skill_by_name("lingshi")
                    .map(|def| {
                        core.skills
                            .get(&(core.turn.current, def))
                            .map(|i| i.enabled && !i.once_used)
                            .unwrap_or(false)
                    })
                    .unwrap_or(false)
                    && core.player(core.turn.current).equips.iter().any(|c| {
                        catalog.card_slot_of(core, *c) == Some(crate::catalog::EquipSlot::Treasure)
                    });
                // 诈降（L27 批 4）：已用牌数 < 已损失体力 →
                // 出杀不限次数（上游 mod cardUsable Infinity）
                let sbzhaxiang_extra = catalog
                    .skill_by_name(crate::content::host::L27_SBZHAXIANG_KEY)
                    .map(|def| core.has_skill(core.turn.current, def))
                    .unwrap_or(false)
                    && crate::content::host::sbzhaxiang_active(core, core.turn.current);
                // 敌仇效果（UP37 R01 dcdichou_eff，哈希 2754298115）：
                // 出杀上限 +X（X = 2754298115 印记值）
                let dcdichou_eff_extra = {
                    let has_eff = catalog
                        .skill_by_name("dcdichou_eff")
                        .or_else(|| {
                            crate::rvm_ops::resolve_skill_def(catalog, 2_754_298_115)
                        })
                        .map(|def| core.has_skill(core.turn.current, def))
                        .unwrap_or(false)
                        || core
                            .vm_aux
                            .flags
                            .get(&(core.turn.current, "skill:2754298115".to_string()))
                            .copied()
                            .unwrap_or(0)
                            != 0;
                    let bonus = core
                        .vm_aux
                        .marks
                        .get(&(core.turn.current, 2_754_298_115))
                        .copied()
                        .unwrap_or(0);
                    let used = core
                        .vm_aux
                        .flags
                        .get(&(core.turn.current, "dcdichou_eff_sha_used".to_string()))
                        .copied()
                        .unwrap_or(0) as i32;
                    has_eff && bonus > used
                };
                if !lingshi_extra && !sbzhaxiang_extra && !dcdichou_eff_extra {
                    return Err(RejectReason::ShaLimit);
                }
                if lingshi_extra {
                    mark_lingshi = true;
                } else if !sbzhaxiang_extra && dcdichou_eff_extra {
                    mark_dcdichou_eff = true;
                }
            } else {
                mark_huihuo = true;
            }
        } else {
            mark_yuanbaorou = true;
        }
    }
    // 方天画戟（批③-b 第九把）：该杀为最后手牌且装备方天
    // 时可指定至多 3 个目标（逐个独立合法性检查，去重）。
    let fangtian = core.player(player_id).hand.len() == 1
        && core
            .player(player_id)
            .equips
            .iter()
            .any(|c| catalog.card_kind(*c) == CardKind::Fangtian);
    // 抗yin2（U07 批四，规则 v114）：杀额外目标上限 +X
    let kangyin_extra = match core.turn.kangyin2 {
        Some((o, x)) if o == player_id => x as usize,
        _ => 0,
    };
    // 飞刃（U05 ow 源氏，规则 v129）：梅花杀额外目标上限
    // +1（上游 selectTarget range[1]++ 同口径；花色按实
    // 体牌——转化声明牌种不影响）
    let feiren_extra = {
        let club = matches!(
            catalog.cards[card as usize].suit,
            crate::catalog::Suit::Club
        );
        let sha_kind = kind.is_sha_like();
        club && sha_kind
            && catalog
                .skill_by_name(crate::content::host::WUDIAN_FEIREN_KEY)
                .map(|def| core.has_skill(player_id, def))
                .unwrap_or(false)
    };
    // 天弦（U05 xianjian 柳梦璃，规则 v184）：锁定——杀
    // 目标数上限无限（上游 mod selectTarget range[1]=∞
    // 同口径；sha 有限上限必满足 !==-1 条件）
    let tianxian_inf = kind.is_sha_like()
        && catalog
            .skill_by_name(crate::content::host::WUDIAN_XJ_TIANXIAN_KEY)
            .map(|def| core.has_skill(player_id, def))
            .unwrap_or(false);
    // 狂笑（Boss 白虎，规则 v235）：锁定——【杀】目标数上
    // 限无限（上游 mod selectTarget range[1]=-1 同口径；
    // 天弦同构）
    let kuangxiao_inf = kind.is_sha_like()
        && catalog
            .skill_by_name(crate::content::standard::BOSS_KUANGXIAO)
            .map(|def| core.has_skill(player_id, def))
            .unwrap_or(false);
    // 巨源（Boss 朱厌，规则 v226）：出牌阶段体力低于上次
    // 回合结束时体力 → 【杀】额外 1 目标（上游 selectTarget
    // range[1]++ 同口径；飞刃同款）
    let juyuan_extra = {
        let jdef = catalog.skill_by_name(crate::content::standard::BOSS_JUYUAN);
        match jdef {
            Some(def) => {
                core.has_skill(player_id, def)
                    && core.player(player_id).juyuan_storage > 0
                    && core.player(player_id).hp < core.player(player_id).juyuan_storage
            }
            None => false,
        }
    };
    // 连舞（尚章，swd 武将面批㉒）：锁定——杀可额外指定一
    // 个目标（上游 mod selectTarget range[1]++ 同口径）
    let lianwu_extra = catalog
        .skill_by_name("lianwu")
        .map(|def| core.has_skill(core.turn.current, def))
        .unwrap_or(false);
    // 疾风（卢承轩，swd 武将面批㉘）：锁定——杀目标上限 +X
    //（X=已损失体力，上游 range[1] += maxHp-hp 同口径）
    let ljifeng_extra = catalog
        .skill_by_name("ljifeng")
        .map(|def| core.has_skill(core.turn.current, def))
        .unwrap_or(false)
        && core.player(core.turn.current).max_hp > core.player(core.turn.current).hp as u32;
    let ljifeng_n = if ljifeng_extra {
        core.player(core.turn.current).max_hp - core.player(core.turn.current).hp as u32
    } else {
        0
    };
    // 破晓（张烈，swd 武将面批㊞）：无武器时杀目标上限 +1
    //（上游 selectTarget range[1]++ 同口径）
    let poxiao_extra = catalog
        .skill_by_name("poxiao")
        .map(|def| core.has_skill(core.turn.current, def))
        .unwrap_or(false)
        && !core
            .player(core.turn.current)
            .equips
            .iter()
            .any(|c| catalog.card_kind(*c) == crate::catalog::CardKind::Weapon);
    // 疾风（swd 批 150）：单目标牌目标数无限（上游 mod
    // selectTarget range[1]=Infinity 同口径）
    let jifeng_inf = kind.is_sha_like()
        && catalog
            .skill_by_name(crate::content::host::SWD_JIFENG_KEY)
            .map(|def| core.has_skill(player_id, def))
            .unwrap_or(false);
    let max_targets = if tianxian_inf || kuangxiao_inf || jifeng_inf {
        usize::MAX
    } else {
        (if fangtian { 3 } else { 1 })
            + kangyin_extra
            + u32::from(feiren_extra) as usize
            + u32::from(juyuan_extra) as usize
            + u32::from(lianwu_extra) as usize
            + ljifeng_n as usize
            + u32::from(poxiao_extra) as usize
    };
    if targets.is_empty() || targets.len() > max_targets {
        return Err(RejectReason::IllegalTargets);
    }
    {
        let mut seen = std::collections::BTreeSet::new();
        for t in &targets {
            if !seen.insert(*t) {
                return Err(RejectReason::IllegalTargets);
            }
        }
    }
    let target = targets[0];
    if target == core.turn.current || !core.player(target).alive {
        return Err(RejectReason::IllegalTargets);
    }
    // 空城（诸葛亮）：无手牌者不能成为杀/决斗目标（锁定门控）
    let kongcheng = catalog
        .skill_by_name(crate::content::standard::KONG_CHENG)
        .map(|d| core.has_skill(target, d))
        .unwrap_or(false);
    if kongcheng && core.player(target).hand.is_empty() {
        return Err(RejectReason::IllegalTargets);
    }
    // 圣休号角嘲讽（U05 swd 灵剑，规则 v198）：相邻有嘲讽
    // 者时只能以其为杀目标（上游 hslingjian_chaofeng mod
    // targetEnabled 同口径）
    if kind.is_sha_like() {
        let seats = core.players.len() as u8;
        let cur = core.turn.current;
        let holder = (0..seats).find(|p| {
            *p != cur && core.player(*p).alive && core.player(*p).chaofeng_active && {
                let d = (u16::from(*p) + u16::from(seats) - u16::from(cur)) % u16::from(seats);
                d == 1 || d == u16::from(seats) - 1
            }
        });
        if let Some(h) = holder {
            if targets.iter().any(|t| *t != h) {
                return Err(RejectReason::IllegalTargets);
            }
        }
    }
    // 逐月（U05 xianjian 云天河，规则 v161）：本出牌阶段
    // 使用杀须以逐月目标为目标（上游 mod playerEnabled 同
    // 口径；白名单空=不限制）
    if !core.turn.zhuyue_targets.is_empty()
        && !targets.iter().all(|t| core.turn.zhuyue_targets.contains(t))
    {
        return Err(RejectReason::IllegalTargets);
    }
    let from = core.turn.current;
    // 贴房绅视（U05 swd，规则 v94）：本回合无视距离指定的
    // 目标豁免距离校验（上游 targetInRange mod 同口径）
    // 堡垒哨卫（U05 ow，规则 v125）：杀无视距离
    let baolei_range_free = core.skills.keys().any(|(o, def)| {
        *o == core.turn.current
            && core.player(*o).baolei_mode == 1
            && catalog
                .skills
                .get(*def as usize)
                .map(|sd| sd.name == "bshaowei")
                .unwrap_or(false)
    });
    // 飞刃（U05 ow 源氏，规则 v129）/飞弹（U05 ow 法老
    // 之鹰，规则 v130）：锁定——你的杀无视距离（上游
    // mod targetInRange 同口径）
    let sha_range_free = catalog
                        .skill_by_name(crate::content::host::WUDIAN_FEIREN_KEY)
                        .map(|def| core.has_skill(core.turn.current, def))
                        .unwrap_or(false)
                        || catalog
                            .skill_by_name(crate::content::host::WUDIAN_FEIDAN_KEY)
                            .map(|def| core.has_skill(core.turn.current, def))
                            .unwrap_or(false)
                        // 天弦（U05 xianjian 柳梦璃，规则 v184）：锁定——
                        // 杀无视距离（上游 mod targetInRange 同口径）
                        || catalog
                            .skill_by_name(crate::content::host::WUDIAN_XJ_TIANXIAN_KEY)
                            .map(|def| core.has_skill(core.turn.current, def))
                            .unwrap_or(false)
                        // 狂笑（Boss 白虎，规则 v235）：锁定——【杀】无视
                        // 距离（上游 mod targetInRange=true 同口径）
                        || catalog
                            .skill_by_name(crate::content::standard::BOSS_KUANGXIAO)
                            .map(|def| core.has_skill(core.turn.current, def))
                            .unwrap_or(false)
                        // 焚天（Boss 朱雀，规则 v236）：红色牌无视距离
                        || fentian_red
                        // 疾影（赛特，swd 批㉑）：锁定 sha 无视距离
                        || catalog
                            .skill_by_name("jiying")
                            .map(|def| core.has_skill(core.turn.current, def))
                            .unwrap_or(false)
                        // 破月（拓跋玉儿，swd 武将面批 28）：黑色杀无视
                        // 距离（上游 mod targetInRange black 同口径）
                        || (catalog
                            .skill_by_name("poyue")
                            .map(|def| core.has_skill(core.turn.current, def))
                            .unwrap_or(false)
                            && matches!(
                                catalog.cards[card as usize].suit,
                                crate::catalog::Suit::Spade
                                    | crate::catalog::Suit::Club
                            ))
                        // 神恩（Boss 太昊/炎帝 光环，规则 v238）：boss 方
                        // 用牌无视距离（光环激活=场上存在持有者）
                        || ({
                            let active = catalog
                                .skill_by_name(crate::content::standard::BOSS_SHENEN)
                                .map(|def| core.skills.keys().any(|(_, dd)| *dd == def))
                                .unwrap_or(false);
                            active && core.is_boss_side(core.turn.current)
                        })
                        // 守翼（年兽守翼形态，规则 v251）：所有牌无视距离
                        || catalog
                            .skill_by_name(crate::content::standard::BOSS_SHOUYI)
                            .map(|def| core.has_skill(core.turn.current, def))
                            .unwrap_or(false);
    // 诈降（L27 批 4）：已用牌数 < 已损失体力 → 用牌无距离限制
    let sbzhaxiang_free = catalog
        .skill_by_name(crate::content::host::L27_SBZHAXIANG_KEY)
        .map(|d| core.has_skill(player_id, d))
        .unwrap_or(false)
        && crate::content::host::sbzhaxiang_active(core, player_id);
    let feiren_range_free = sha_range_free || sbzhaxiang_free;
    let zfs_free = core.turn.zhufangshenshi_target == Some(target);
    // 逐月（U05 xianjian 云天河，规则 v161）：杀向逐月目标
    // 无距离限制（上游 mod targetInRange 同口径）
    let zhuyue_free = core.turn.zhuyue_targets.contains(&target);
    // 疾风·司空宇面（swd 武将面批㊟）：锁定——体力值不大于
    // 自己的角色视为在攻击范围（上游 mod targetInRange 同口径）
    let sliufeng_free = catalog
        .skill_by_name("sliufeng")
        .map(|def| {
            core.has_skill(core.turn.current, def)
                && core.player(target).hp <= core.player(core.turn.current).hp
        })
        .unwrap_or(false);
    let dist = core.computed_distance(catalog, from, target);
    if !zfs_free
        && !baolei_range_free
        && !feiren_range_free
        && !zhuyue_free
        && !sliufeng_free
        && dist > core.attack_range(catalog, from)
    {
        return Err(RejectReason::IllegalTargets);
    }
    // 方天多目标：其余目标逐个查距离/存活/空城。
    for t in targets.iter().skip(1) {
        if !core.player(*t).alive {
            return Err(RejectReason::IllegalTargets);
        }
        let kc = catalog
            .skill_by_name(crate::content::standard::KONG_CHENG)
            .map(|d| core.has_skill(*t, d))
            .unwrap_or(false);
        if kc && core.player(*t).hand.is_empty() {
            return Err(RejectReason::IllegalTargets);
        }
        let d2 = core.computed_distance(catalog, from, *t);
        if !feiren_range_free && d2 > core.attack_range(catalog, from) {
            return Err(RejectReason::IllegalTargets);
        }
    }
    let nature = if kind == CardKind::FireSha {
        crate::event::DamageNature::Fire
    } else if kind == CardKind::LeiSha {
        crate::event::DamageNature::Thunder
    } else {
        crate::event::DamageNature::Normal
    };
    let player = core.turn.current;
    if mark_yuanbaorou {
        if let Some(def) = catalog.skill_by_name(crate::content::host::GJ_FOOD_YUANBAOROU_KEY) {
            if let Some(inst) = state.core.skills.get_mut(&(player_id, def)) {
                inst.once_used = true;
            }
        }
    }
    if mark_huihuo {
        if let Some(def) = catalog.skill_by_name(crate::content::standard::BOSS_HUIHUO) {
            if let Some(inst) = state.core.skills.get_mut(&(player_id, def)) {
                inst.once_used = true;
            }
        }
    }
    if mark_mutoumianju {
        if let Some(def) = catalog.skill_by_name(crate::content::host::GJ_MUTOU_SKILL_KEY) {
            if let Some(inst) = state.core.skills.get_mut(&(player_id, def)) {
                inst.once_used = true;
            }
        }
    }
    if mark_lingshi {
        if let Some(def) = catalog.skill_by_name("lingshi") {
            if let Some(inst) = state.core.skills.get_mut(&(player_id, def)) {
                inst.once_used = true;
            }
        }
    }
    if mark_dcdichou_eff {
        let key = (player_id, "dcdichou_eff_sha_used".to_string());
        let next = state.core.vm_aux.flags.get(&key).copied().unwrap_or(0) + 1;
        state.core.vm_aux.flags.insert(key, next);
    }
    // 良策转化（玩点论杀批 15）：基本牌当五谷时按五谷
    // 机器路由（use_as 声明）。
    if kind == CardKind::Wugu {
        crate::resolution::push_frame(
            &mut state.resolution,
            EventData::CardUsed(crate::machine::card_use_as(
                card,
                player,
                targets,
                Some(CardKind::Wugu),
            )),
        );
        return Ok(());
    }
    crate::resolution::push_frame(
        &mut state.resolution,
        EventData::CardUsed(crate::machine::card_use_with_nature(
            card,
            player,
            targets,
            nature,
            Some(kind),
        )),
    );
    return Ok(());
}
