use super::*;

pub(super) fn has_unlimited_sha_public(
    core: &GameStateCore,
    catalog: &SharedCatalog,
    player_id: PlayerId,
    card: CardId,
) -> bool {
    let red = matches!(
        catalog.cards[card as usize].suit,
        crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
    );
    let fentian_red = red
        && catalog
            .skill_by_name(crate::content::standard::BOSS_FENTIAN)
            .is_some_and(|def| core.has_skill(player_id, def));
    let paoxiao = !catalog.is_rechanyuan_sealed_name(core, player_id, crate::content::standard::PAO_XIAO)
        && catalog
            .skill_by_name(crate::content::standard::PAO_XIAO)
            .is_some_and(|d| core.has_skill(player_id, d));
    let has_zhuge = !crate::kernel::mbcmfangzhu_baiban(core, player_id)
        && core
            .player(player_id)
            .equips
            .iter()
            .any(|c| catalog.card_kind(*c) == CardKind::Zhuge);
    let baolei_sentinel = catalog
        .skill_by_name(crate::content::host::WUDIAN_BAOLEI_KEY)
        .is_some_and(|def| core.has_skill(player_id, def) && core.player(player_id).baolei_mode == 1);
    let poyue_red_free = red
        && catalog
            .skill_by_name("poyue")
            .is_some_and(|def| core.has_skill(player_id, def));
    let sbzhaxiang_extra = catalog
        .skill_by_name(crate::content::host::L27_SBZHAXIANG_KEY)
        .is_some_and(|def| core.has_skill(player_id, def))
        && crate::content::host::sbzhaxiang_active(core, player_id);
    paoxiao || has_zhuge || baolei_sentinel || fentian_red || poyue_red_free || sbzhaxiang_extra
}

/// Upstream `lib.filter.targetInRange(card, from, target)` for `Sha` (including skill/modifier
/// unlimited range, distance-1 exemptions, `ow_feidan`, and weapon/horse attack range).
pub(crate) fn sha_target_in_range(
    core: &GameStateCore,
    catalog: &SharedCatalog,
    from: PlayerId,
    target: PlayerId,
    card: Option<CardId>,
    kind: CardKind,
    sha_nature: crate::event::DamageNature,
    va_ignore_distance: Option<bool>,
) -> bool {
    let baolei_range_free = core.skills.keys().any(|(o, def)| {
        *o == from
            && core.player(*o).baolei_mode == 1
            && catalog
                .skills
                .get(*def as usize)
                .map(|sd| sd.name == "bshaowei")
                .unwrap_or(false)
    });
    let fentian_red = card.is_some_and(|c| {
        matches!(
            catalog.resolved_card_suit(core, c),
            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
        )
    }) && catalog
        .skill_by_name(crate::content::standard::BOSS_FENTIAN)
        .is_some_and(|def| core.has_skill(from, def));
    let vm_va_ignore_dist = va_ignore_distance.unwrap_or_else(|| {
        card.is_some_and(|c| {
            (kind != catalog.resolved_card_kind(core, c) || !core.player(from).hand.contains(&c))
                && catalog
                    .find_player_vm_view_as(core, from, c, true, Some(CardKind::Sha))
                    .is_some_and(|s| s.ignore_distance)
        })
    });
    let sha_range_free = catalog
        .skill_by_name(crate::content::host::WUDIAN_FEIREN_KEY)
        .is_some_and(|def| core.has_skill(from, def))
        || catalog
            .skill_by_name(crate::content::host::WUDIAN_FEIDAN_KEY)
            .is_some_and(|def| {
                catalog.skills[def as usize].behavior.as_host_behavior()
                    != Some(crate::vocabulary::host::HostBehavior::U05OwBatchD)
                    && core.has_skill(from, def)
            })
        || catalog
            .skill_by_name(crate::content::host::WUDIAN_XJ_TIANXIAN_KEY)
            .is_some_and(|def| core.has_skill(from, def))
        || catalog
            .skill_by_name(crate::content::standard::BOSS_KUANGXIAO)
            .is_some_and(|def| core.has_skill(from, def))
        || fentian_red
        || catalog
            .skill_by_name("jiying")
            .is_some_and(|def| core.has_skill(from, def))
        || (catalog
            .skill_by_name("poyue")
            .is_some_and(|def| core.has_skill(from, def))
            && card.is_some_and(|c| {
                matches!(
                    catalog.resolved_card_suit(core, c),
                    crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                )
            }))
        || {
            let active = catalog
                .skill_by_name(crate::content::standard::BOSS_SHENEN)
                .is_some_and(|def| core.skills.keys().any(|(_, dd)| *dd == def));
            active && core.is_boss_side(from)
        }
        || catalog
            .skill_by_name(crate::content::standard::BOSS_SHOUYI)
            .is_some_and(|def| core.has_skill(from, def))
        || vm_va_ignore_dist;
    let sbzhaxiang_free = catalog
        .skill_by_name(crate::content::host::L27_SBZHAXIANG_KEY)
        .is_some_and(|d| core.has_skill(from, d))
        && crate::content::host::sbzhaxiang_active(core, from);
    // 新烈弓（hs_alleria · character/shenhua/skill.js:3766）：
    // 【杀】攻击范围判断中，目标距离 ≤ 此【杀】点数时视为在攻击范围内。
    let dist = core.computed_distance(catalog, from, target);
    let xinliegong_range_free = card.is_some_and(|c| {
        let rank = catalog.cards.get(c as usize).map_or(0, |cd| cd.rank);
        rank > 0
            && dist <= u32::from(rank)
            && !catalog.is_rechanyuan_sealed_name(core, from, "xinliegong")
            && !catalog.is_tieji_sealed(core, from)
            && catalog.skill_by_name("xinliegong").is_some_and(|def| {
                core.has_skill(from, def)
                    && catalog.skills[def as usize].behavior.as_host_behavior()
                        == Some(crate::vocabulary::host::HostBehavior::U05Hearth2BatchF)
            })
    });
    let mod_dist_free = card.is_some_and(|c| {
        crate::rvm_bridge::modifiers::eval_card_distance_unlimited_to(
            core,
            catalog,
            from,
            Some(target),
            c,
            kind,
            sha_nature,
        )
    });
    let zfs_free = core.turn.zhufangshenshi_target == Some(target)
        || core
            .vm_aux
            .flags
            .get(&(from, format!("zhufangshenshi_extra:{target}")))
            .copied()
            .unwrap_or(0)
            > 0;
    let zhuyue_free = core.turn.zhuyue_targets.contains(&target);
    let sliufeng_free = catalog
        .skill_by_name("sliufeng")
        .is_some_and(|def| core.has_skill(from, def) && core.player(target).hp <= core.player(from).hp);
    let tanbei_tgt_free =
        crate::rvm_bridge::modifiers::tanbei_unlimited_target(core, catalog, from, target);
    let ow_feidan = catalog.skill_by_name("feidan").is_some_and(|def| {
        catalog.skills[def as usize].behavior.as_host_behavior()
            == Some(crate::vocabulary::host::HostBehavior::U05OwBatchD)
            && core.has_skill(from, def)
    });
    if ow_feidan && dist <= 1 {
        return false;
    }
    let shanxian_free = core
        .vm_aux
        .flags
        .get(&(from, "ow_shanxian_target".to_string()))
        .copied()
        == Some(u32::from(target) + 1);
    zfs_free
        || baolei_range_free
        || sha_range_free
        || sbzhaxiang_free
        || xinliegong_range_free
        || mod_dist_free
        || ow_feidan
        || zhuyue_free
        || sliufeng_free
        || tanbei_tgt_free
        || shanxian_free
        || core.in_attack_range(catalog, from, target)
}

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
    let mut mark_vm_sha_extra = false;
    let mut mark_tanlin3 = false;
    let player_id = core.turn.current;
    let sha_nature = match kind {
        CardKind::FireSha => crate::event::DamageNature::Fire,
        CardKind::LeiSha => crate::event::DamageNature::Thunder,
        _ => crate::event::DamageNature::Normal,
    };
    let (vm_sha_limit, vm_sha_unlimited) =
        crate::rvm_bridge::modifiers::eval_sha_use_limit(
            core,
            catalog,
            player_id,
            Some(card),
            kind,
            sha_nature,
            1,
        );
    // 焚天（Boss 朱雀，规则 v236）：锁定——红色牌使用无
    // 次数限制且无距离限制（上游 mod cardUsable Infinity
    // + targetInRange=true 同口径）
    let fentian_red = {
        let red = matches!(
            catalog.resolved_card_suit(core, card),
            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
        );
        red && catalog
            .skill_by_name(crate::content::standard::BOSS_FENTIAN)
            .map(|def| core.has_skill(player_id, def))
            .unwrap_or(false)
    };
    let paoxiao = !catalog.is_rechanyuan_sealed_name(
        core,
        core.turn.current,
        crate::content::standard::PAO_XIAO,
    ) && catalog
        .skill_by_name(crate::content::standard::PAO_XIAO)
        .map(|d| core.has_skill(core.turn.current, d))
        .unwrap_or(false);
    // 碧日浓雾（gwent 天气）：持有者不能使用杀
    if [
        crate::content::host::GW_BIRIN_STATE_KEY,
        "gw_birinongwu",
    ]
    .into_iter()
    .any(|k| {
        catalog.skill_by_name(k).is_some_and(|def| {
            core.skills
                .get(&(core.turn.current, def))
                .is_some_and(|i| i.enabled)
        })
    })
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
        || catalog
            .skill_by_name("guangshu_club")
            .is_some_and(|def| {
                catalog.skills[def as usize].behavior.as_host_behavior()
                    == Some(crate::vocabulary::host::HostBehavior::U05OwBatchD)
                    && core.has_skill(core.turn.current, def)
            })
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
    let has_zhuge = !crate::kernel::mbcmfangzhu_baiban(core, player_id)
        && core.has_active_equip_kind(catalog, player_id, CardKind::Zhuge);
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
            catalog.effective_card_suit(core, player_id, card),
            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
        );
    let vm_va_sha = if kind != catalog.resolved_card_kind(core, card)
        || !core.player(player_id).hand.contains(&card)
    {
        catalog
            .find_player_vm_view_as(core, player_id, card, true, Some(CardKind::Sha))
            .cloned()
    } else {
        None
    };
    let eff_suit = catalog.effective_card_suit(core, player_id, card);
    let is_black = matches!(
        eff_suit,
        crate::catalog::Suit::Spade | crate::catalog::Suit::Club
    );
    let is_red = matches!(
        eff_suit,
        crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
    );
    let vm_va_no_count = vm_va_sha
        .as_ref()
        .is_some_and(|s| s.no_use_count || (s.black_no_use_count && is_black));
    let tanbei_free = !targets.is_empty()
        && targets
            .iter()
            .all(|&t| crate::rvm_bridge::modifiers::tanbei_unlimited_target(core, catalog, player_id, t));
    // 立牧（`xinfu_limu`，sp/skill.js:49250）：判定区有牌时，仅对攻击范围内目标出杀不限次数。
    let vm_sha_unlimited = vm_sha_unlimited
        || (!targets.is_empty()
            && targets
                .iter()
                .all(|&t| xinfu_limu_in_range(core, catalog, player_id, t)));
    let zhanlong2_unlimited = catalog.skill_by_name("zhanlong2").is_some_and(|d| {
        catalog.skills[d as usize].behavior.as_host_behavior()
            == Some(crate::vocabulary::host::HostBehavior::U05OwBatchC)
            && core.has_skill(player_id, d)
    });
    let sha_unlimited_ext = vm_sha_unlimited
        || zhanlong2_unlimited
        || tanbei_free
        || crate::rvm_bridge::modifiers::eval_sha_unlimited_for_targets(
            core, catalog, player_id, &targets,
        )
        || crate::rvm_bridge::modifiers::eval_sha_no_use_count(
            core,
            catalog,
            player_id,
            Some(card),
            kind,
        );
    if !paoxiao
        && !has_zhuge
        && !baolei_sentinel
        && !fentian_red
        && !poyue_red_free
        && !vm_va_no_count
        && !sha_unlimited_ext
        && (core.turn.sha_used_play_phase || vm_sha_limit == 0)
        && !land_sha_bonus
    {
        if vm_sha_limit == 0 {
            return Err(RejectReason::ShaLimit);
        }
        // 元宝肉（G1 古剑食物）：出牌阶段额外一张杀
        //（每回合一次；once_used 随回合开始重置）
        let extra = [crate::content::host::GJ_FOOD_YUANBAOROU_KEY, "yuanbaorou"]
            .into_iter()
            .any(|k| {
                catalog.skill_by_name(k).is_some_and(|def| {
                    core.skills
                        .get(&(core.turn.current, def))
                        .is_some_and(|i| i.enabled && !i.once_used)
                })
            });
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
                let vm_sha_extra_used = core
                    .vm_aux
                    .flags
                    .get(&(core.turn.current, "vm_sha_extra_used".to_string()))
                    .copied()
                    .unwrap_or(0);
                let vm_sha_extra = vm_sha_limit > 1 + vm_sha_extra_used;
                let tanlin3_extra = catalog
                    .skill_by_behavior(crate::content::host::HostBehavior::SwdTanlinOwner)
                    .map(|def| {
                        core.skills
                            .get(&(core.turn.current, def))
                            .map(|i| i.enabled && !i.once_used)
                            .unwrap_or(false)
                    })
                    .unwrap_or(false);
                if !lingshi_extra
                    && !sbzhaxiang_extra
                    && !dcdichou_eff_extra
                    && !vm_sha_extra
                    && !tanlin3_extra
                {
                    return Err(RejectReason::ShaLimit);
                }
                if lingshi_extra {
                    mark_lingshi = true;
                } else if !sbzhaxiang_extra && dcdichou_eff_extra {
                    mark_dcdichou_eff = true;
                } else if !sbzhaxiang_extra && vm_sha_extra {
                    mark_vm_sha_extra = true;
                } else if !sbzhaxiang_extra && tanlin3_extra {
                    mark_tanlin3 = true;
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
    let fangtian = !crate::kernel::mbcmfangzhu_baiban(core, player_id)
        && core.player(player_id).hand.len() == 1
        && core.has_active_equip_kind(catalog, player_id, CardKind::Fangtian);
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
            catalog.resolved_card_suit(core, card),
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
            .any(|c| catalog.equip_card_has_kind(core, *c, crate::catalog::CardKind::Weapon));
    // 疾风（swd 批 150）：单目标牌目标数无限（上游 mod
    // selectTarget range[1]=Infinity 同口径）
    let jifeng_inf = kind.is_sha_like()
        && catalog
            .skill_by_name(crate::content::host::SWD_JIFENG_KEY)
            .map(|def| core.has_skill(player_id, def))
            .unwrap_or(false);
    let vm_extra_targets = crate::rvm_bridge::modifiers::eval_sha_extra_targets(
        core,
        catalog,
        player_id,
        card,
        kind,
        sha_nature,
    );
    let max_targets = if tianxian_inf || kuangxiao_inf || jifeng_inf || vm_extra_targets >= 99 {
        usize::MAX
    } else {
        (if fangtian { 3 } else { 1 })
            + kangyin_extra
            + u32::from(feiren_extra) as usize
            + u32::from(juyuan_extra) as usize
            + u32::from(lianwu_extra) as usize
            + ljifeng_n as usize
            + u32::from(poxiao_extra) as usize
            + vm_extra_targets
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
        let has_sk = |p: PlayerId, name: &str| {
            catalog
                .skill_by_name(name)
                .is_some_and(|d| core.has_skill(p, d))
        };
        if !has_sk(core.turn.current, "hslingjian_chaofeng") {
            let seats = core.players.len() as u8;
            let cur = core.turn.current;
            let holder = (0..seats).find(|p| {
                *p != cur
                    && core.player(*p).alive
                    && (core.player(*p).chaofeng_active || has_sk(*p, "hslingjian_chaofeng"))
                    && {
                        let d =
                            (u16::from(*p) + u16::from(seats) - u16::from(cur)) % u16::from(seats);
                        d == 1 || d == u16::from(seats) - 1
                    }
            });
            if let Some(h) = holder {
                if targets.iter().any(|t| *t != h) {
                    return Err(RejectReason::IllegalTargets);
                }
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
    let va_ignore_dist = vm_va_sha.as_ref().is_some_and(|s| s.ignore_distance);
    if !sha_target_in_range(
        core,
        catalog,
        from,
        target,
        Some(card),
        kind,
        sha_nature,
        Some(va_ignore_dist),
    ) {
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
        if !sha_target_in_range(
            core,
            catalog,
            from,
            *t,
            Some(card),
            kind,
            sha_nature,
            Some(va_ignore_dist),
        ) {
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
        for k in [crate::content::host::GJ_FOOD_YUANBAOROU_KEY, "yuanbaorou"] {
            if let Some(def) = catalog.skill_by_name(k) {
                if let Some(inst) = state.core.skills.get_mut(&(player_id, def)) {
                    inst.once_used = true;
                }
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
        for k in [crate::content::host::GJ_MUTOU_SKILL_KEY, "mutoumianju_skill"] {
            if let Some(def) = catalog.skill_by_name(k) {
                if let Some(inst) = state.core.skills.get_mut(&(player_id, def)) {
                    inst.once_used = true;
                }
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
    if mark_tanlin3 {
        if let Some(def) =
            catalog.skill_by_behavior(crate::content::host::HostBehavior::SwdTanlinOwner)
        {
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
    if mark_vm_sha_extra {
        let key = (player_id, "vm_sha_extra_used".to_string());
        let next = state.core.vm_aux.flags.get(&key).copied().unwrap_or(0) + 1;
        state.core.vm_aux.flags.insert(key, next);
    }
    if vm_va_no_count {
        state
            .core
            .vm_aux
            .flags
            .insert((player_id, "vm_va_sha_no_count".to_string()), 1);
    }
    if let Some(spec) = &vm_va_sha {
        if spec.refund_use_count_if_no_damage {
            state
                .core
                .vm_aux
                .flags
                .insert((player_id, format!("sha_refund_if_no_damage:{card}")), 1);
        }
        let src_suit = catalog.resolved_card_suit(&state.core, card);
        for &t in &targets {
            let lacks_suit = spec.target_lacks_source_suit_unrespondable
                && !state
                    .core
                    .player(t)
                    .hand
                    .iter()
                    .any(|hc| catalog.resolved_card_suit(&state.core, *hc) == src_suit);
            if (spec.red_unrespondable && is_red) || lacks_suit {
                state
                    .core
                    .vm_aux
                    .flags
                    .insert((player_id, format!("sha_unrespondable:{card}:{t}")), 1);
            }
        }
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
