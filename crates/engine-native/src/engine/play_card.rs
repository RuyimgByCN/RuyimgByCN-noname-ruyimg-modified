use super::*;

pub(super) fn play_card(
    state: &mut GameState,
    catalog: &SharedCatalog,
    actor: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
) -> Result<(), RejectReason> {
    common_play_validation(&state.core, catalog, actor, card, &targets)?;
    let core = &state.core;
    let (kind, mark_mutoumianju) = play_kind(core, catalog, card, &targets);
    check_play_target_restrictions(core, catalog, card, &targets)?;
    if kind.is_equip() {
        return play_equip(state, catalog, card, &targets);
    }
    check_play_use_restrictions(core, catalog, card, kind, &targets)?;
    let actor = core.turn.current;
    if let Some(res) = super::play_card_u05_zhenfa::try_play_u05_b1_special(
        state,
        catalog,
        actor,
        card,
        kind,
        targets.clone(),
    ) {
        return res;
    }
    let core = &state.core;
    match kind {
                CardKind::Xuanyuanjian => {
                    // 轩辕剑（swd 批 146）：无目标；装备者须持有「轩辕」或
                    // 体力 >2（上游 enable 同口径）
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let has_xuanyuan = catalog
                        .skill_by_name(crate::content::host::SWD_XUANYUAN_KEY)
                        .map(|d| core.has_skill(core.turn.current, d))
                        .unwrap_or(false);
                    if !has_xuanyuan && core.player(core.turn.current).hp <= 2 {
                        return Err(RejectReason::CardLimit);
                    }
                }
                CardKind::Xuejibingbao => {
                    // 雪霁冰雹（swd 卡牌面补切批②）：恰 1 名存活目标且未
                    // 处于标记中（上游 filterTarget 无同标记者同口径）
                    if targets.len() != 1
                        || !core.player(targets[0]).alive
                        || core.player(targets[0]).xuejibingbao > 0
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Shenmiguo => {
                    // 神秘果不可直接使用（经用牌后窗口消耗——窗口机登记
                    // 结构性暂缓，见补切批②证据）
                    return Err(RejectReason::IllegalTargets);
                }
                CardKind::Lianyaohu
                | CardKind::Qiankundai
                | CardKind::Nvwashi
                | CardKind::Shennongding
                | CardKind::Kunlunjingc
                | CardKind::Donghuangzhong
                | CardKind::Kongdongyin
                | CardKind::Guilingzhitao
                | CardKind::Sifeizhenmian
                | CardKind::Shuchui
                | CardKind::Sadengjinhuan
                | CardKind::Haotianta
                | CardKind::Yuruyi
                | CardKind::Yuchanqian
                | CardKind::Yuchankun
                | CardKind::Yuchanzhen
                | CardKind::Yuchanxun
                | CardKind::Yuchankan
                | CardKind::Yuchanli
                | CardKind::Yuchangen
                | CardKind::Yuchandui
                | CardKind::Shencaojie
                // 器族五件：手持牌（非装备路径；效果经手持触发技）
                | CardKind::Qinglongzhigui
                | CardKind::Zhuquezhizhang
                | CardKind::Baishouzhihu
                | CardKind::Xuanwuzhihuang
                | CardKind::Huanglinzhicong
                | CardKind::Cangchizhibi => {
                    // 炼妖壶/乾坤袋/女娲石/神农鼎：装备牌走 equip 校验
                    //（同栏替换由 commit_equip 处理；神农鼎/女娲石效果经
                    // 装备技实例）
                }

                CardKind::Fengyinzhidan => {
                    // 封印之丹：对自身使用（无目标或目标=自己）
                    if targets.len() > 1 || targets.first().map(|t| *t != core.turn.current).unwrap_or(false) {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Sha | CardKind::FireSha | CardKind::LeiSha => { return play_sha(state, catalog, card, targets, kind, mark_mutoumianju); }
                CardKind::Chain => {
                    if targets.is_empty() || targets.len() > 2 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    if targets.iter().any(|t| !core.player(*t).alive) {
                        return Err(RejectReason::IllegalTargets);
                    }
                    // 铁索为结算型锦囊：走 CardUsed → 无懈窗口 → ChainMachine
                    // 翻转（旧实现只推 Linked 帧，牌从未离手，ADR 0705 修复）。
                    let use_as = (kind != catalog.resolved_card_kind(core, card)
                        || !core.player(core.turn.current).hand.contains(&card))
                    .then_some(CardKind::Chain);
                    crate::resolution::push_frame(
                        &mut state.resolution,
                        EventData::CardUsed(crate::machine::card_use_as(
                            card,
                            core.turn.current,
                            targets,
                            use_as,
                        )),
                    );
                    return Ok(());
                }
                CardKind::Duel => {
                    if kind == catalog.card_kind(card)
                        && catalog
                            .cards
                            .get(card as usize)
                            .is_some_and(|cd| cd.name == "txhj_jiedao")
                    {
                        if targets.len() != 2 {
                            return Err(RejectReason::IllegalTargets);
                        }
                        let (holder, victim) = (targets[0], targets[1]);
                        if holder == core.turn.current
                            || !core.player(holder).alive
                            || victim == holder
                            || !core.player(victim).alive
                            || core.computed_distance(catalog, holder, victim)
                                > core.attack_range(catalog, holder)
                        {
                            return Err(RejectReason::IllegalTargets);
                        }
                        crate::resolution::push_frame(
                            &mut state.resolution,
                            EventData::CardUsed(crate::machine::card_use(
                                card,
                                core.turn.current,
                                targets,
                            )),
                        );
                        return Ok(());
                    }
                    // 决斗：恰好 1 名其他存活目标，无距离/射程限制。
                    if targets.len() != 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let target = targets[0];
                    if target == core.turn.current || !core.player(target).alive {
                        return Err(RejectReason::IllegalTargets);
                    }
                    // 空城（诸葛亮）：同杀
                    let kongcheng = catalog
                        .skill_by_name(crate::content::standard::KONG_CHENG)
                        .map(|d| core.has_skill(target, d))
                        .unwrap_or(false);
                    if kongcheng && core.player(target).hand.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let use_as = (kind != catalog.resolved_card_kind(core, card)
                        || !core.player(core.turn.current).hand.contains(&card))
                    .then_some(CardKind::Duel);
                    crate::resolution::push_frame(
                        &mut state.resolution,
                        EventData::CardUsed(crate::machine::card_use_as(
                            card,
                            core.turn.current,
                            targets,
                            use_as,
                        )),
                    );
                    return Ok(());
                }
                CardKind::Dismantle => {
                    // 过河拆桥 / 草木皆兵：弃置一名其他存活玩家的一张牌
                    if targets.len() != 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let target = targets[0];
                    if target == core.turn.current || !core.player(target).alive {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let use_as = (kind != catalog.card_kind(card)
                        || !core.player(core.turn.current).hand.contains(&card))
                    .then_some(CardKind::Dismantle);
                    crate::resolution::push_frame(
                        &mut state.resolution,
                        EventData::CardUsed(crate::machine::card_use_as(
                            card,
                            core.turn.current,
                            targets,
                            use_as,
                        )),
                    );
                    return Ok(());
                }
                CardKind::Snatch => {
                    // 顺手牵羊：获得距离 1 的一名其他存活角色的一张牌
                    if targets.len() != 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let target = targets[0];
                    if target == core.turn.current || !core.player(target).alive {
                        return Err(RejectReason::IllegalTargets);
                    }
                    // 谦逊（陆逊）：不能成为顺手牵羊目标（锁定门控）
                    let qianxun = catalog
                        .skill_by_name(crate::content::standard::QIAN_XUN)
                        .map(|d| core.has_skill(target, d))
                        .unwrap_or(false);
                    if qianxun {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let from = core.turn.current;
                    // 奇才（黄月英，锁定）：使用锦囊无距离限制
                    let qicai = catalog
                        .skill_by_name(crate::content::standard::QI_CAI)
                        .map(|d| core.has_skill(from, d))
                        .unwrap_or(false);
                    // 神恩（Boss 太昊/炎帝 光环，规则 v238）：boss 方用
                    // 牌无视距离
                    let shenen_boss_free = {
                        let active = catalog
                            .skill_by_name(crate::content::standard::BOSS_SHENEN)
                            .map(|def| core.skills.keys().any(|(_, dd)| *dd == def))
                            .unwrap_or(false);
                        active && core.is_boss_side(from)
                    };
                    // 守翼（年兽守翼形态，规则 v251）：所有牌无视距离
                    let shouyi_free = catalog
                        .skill_by_name(crate::content::standard::BOSS_SHOUYI)
                        .map(|def| core.has_skill(from, def))
                        .unwrap_or(false);
                    // 焚天（Boss 朱雀，规则 v236）：红色牌无视距离
                    let fentian_red_free = {
                        let red = matches!(
                            catalog.resolved_card_suit(core, card),
                            crate::catalog::Suit::Heart
                                | crate::catalog::Suit::Diamond
                        );
                        red
                            && catalog
                                .skill_by_name(crate::content::standard::BOSS_FENTIAN)
                                .map(|def| core.has_skill(from, def))
                                .unwrap_or(false)
                    };
                    // 探囊（U05 xianjian 韩菱纱，规则 v148）：视为的顺手
                    // 牵羊无距离限制（上游 mod targetInRange 同口径；仅
                    // 声明转化命中——实体牌非顺牵即视为路径）
                    let tannang_free = catalog.resolved_card_kind(core, card) != CardKind::Snatch
                        && (catalog
                            .skill_by_name(crate::content::host::XJ_TANNANG_KEY)
                            .map(|def| core.has_skill(from, def))
                            .unwrap_or(false)
                            || catalog
                                .find_player_vm_view_as(core, from, card, true, Some(CardKind::Snatch))
                                .is_some_and(|s| s.ignore_distance));
                    let limu_in_range_free = xinfu_limu_in_range(core, catalog, from, target);
                    let shanxian_free = core
                        .vm_aux
                        .flags
                        .get(&(from, "ow_shanxian_target".to_string()))
                        .copied()
                        == Some(u32::from(target) + 1);
                    if !qicai
                        && !tannang_free
                        && !fentian_red_free
                        && !shenen_boss_free
                        && !shouyi_free
                        && !limu_in_range_free
                        && !shanxian_free
                        && !crate::rvm_bridge::modifiers::tanbei_unlimited_target(
                            core, catalog, from, target,
                        )
                        && !crate::rvm_bridge::modifiers::eval_card_distance_unlimited_to(
                            core,
                            catalog,
                            from,
                            Some(target),
                            card,
                            CardKind::Snatch,
                            crate::event::DamageNature::Normal,
                        )
                        && core.computed_distance(catalog, from, target) > 1
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let t = core.player(target);
                    if t.hand.is_empty() && t.equips.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                    crate::resolution::push_frame(
                        &mut state.resolution,
                        EventData::CardUsed(crate::machine::card_use_as(
                            card,
                            core.turn.current,
                            targets,
                            Some(CardKind::Snatch),
                        )),
                    );
                    return Ok(());
                }
                CardKind::Le | CardKind::Bing => {
                    // 延时锦囊：置于 1 名其他存活玩家的判定区（不立即生效；`xinfu_limu` 方块牌可置自己）；
                    // 判定区不能有同名牌（同时检查 `declared_kinds`）。
                    if targets.len() != 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let target = targets[0];
                    let actor = core.turn.current;
                    let limu_self_le = kind == CardKind::Le
                        && target == actor
                        && catalog
                            .skill_by_name("xinfu_limu")
                            .is_some_and(|d| core.has_skill(actor, d))
                        && !catalog.is_tieji_sealed(core, actor)
                        && catalog.effective_card_suit(core, actor, card)
                            == crate::catalog::Suit::Diamond;
                    if (target == actor && !limu_self_le) || !core.player(target).alive {
                        return Err(RejectReason::IllegalTargets);
                    }
                    // 无心（Boss 张春华，规则 v260）：锁定——你不能使用延时
                    // 锦囊指定其他角色为目标（上游 mod targetEnabled delay 同
                    // 口径；原生乐/兵路由本就只能置他人→等价持有者不可用乐/
                    // 兵；闪电自置不受限）
                    if !limu_self_le {
                        if let Some(wxdef) = catalog
                            .skill_by_name(crate::content::standard::BOSS_WUXIN)
                        {
                            if core.has_skill(core.turn.current, wxdef) {
                                return Err(RejectReason::IllegalTargets);
                            }
                        }
                    }
                    // 谦逊（陆逊）：不能成为乐不思蜀目标（锁定门控；
                    // 上游无顺手牵羊，见覆盖清单）。
                    // G1 墨守宿主原语：延时锦囊目标免疫（乐/兵均免疫）
                    for t in &targets {
                        if catalog.blocks_delayed_trick(core, *t)
                            || crate::rvm_bridge::modifiers::eval_card_target_canceled(
                                core,
                                catalog,
                                core.turn.current,
                                *t,
                                card,
                                kind,
                                crate::event::DamageNature::Normal,
                            )
                        {
                            return Err(RejectReason::IllegalTargets);
                        }
                    }
                    if kind == CardKind::Le {
                        let qianxun = catalog
                            .skill_by_name(crate::content::standard::QIAN_XUN)
                            .map(|d| core.has_skill(target, d))
                            .unwrap_or(false);
                        if qianxun {
                            return Err(RejectReason::IllegalTargets);
                        }
                    }
                    if core
                        .player(target)
                        .judge
                        .iter()
                        .any(|c| catalog.resolved_card_kind(core, *c) == kind)
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let use_as = (kind != catalog.card_kind(card) || limu_self_le).then_some(kind);
                    crate::resolution::push_frame(
                        &mut state.resolution,
                        EventData::CardUsed(crate::machine::card_use_as(
                            card,
                            actor,
                            targets,
                            use_as,
                        )),
                    );
                    return Ok(());
                }
                CardKind::Longlushanfeng
                | CardKind::Haidao
                | CardKind::Shuimomuxue
                | CardKind::Feixu
                | CardKind::Yixialan
                | CardKind::Shamolvzhou
                | CardKind::Linzhongjianta
                | CardKind::Bingheyaosai
                | CardKind::Youlin
                | CardKind::Lindixiliu => {
                    // 地（mtg 首切片 v100 龙颅山峰/第二批 v101 海岛/第三批
                    // v102 水没墓穴）：无目标（notarget），每回合限一张
                    //（上游 land_used 临时技同口径）；结算在 CardUsed 帧
                    // 内核机器
                    if !targets.is_empty() || core.turn.land_used {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let player = core.turn.current;
                    crate::resolution::push_frame(
                        &mut state.resolution,
                        EventData::CardUsed(crate::machine::card_use(
                            card, player, targets,
                        )),
                    );
                    return Ok(());
                }
                // 无中生有/桃园结义/五谷丰登（P2/N08 缺牌种批①）：无
                // 目标锦囊，结算在 CardUsed 帧内核机器；羊皮卷（swd 第
                // 十三批 v97）对自己使用，同路；地载阵（zhenfa 首切片
                // v110）自动展开非大势力目标，同路。
                | CardKind::Wuzhong
                | CardKind::Taoyuan
                | CardKind::Wugu
                | CardKind::Yangpijuan
                | CardKind::Dizaizhen
                | CardKind::Changshezhen
                | CardKind::Pozhenjue
                | CardKind::ShanDian
                | CardKind::HongShui => {
                    // 长蛇阵：须处于队列或存在友方（上游 enable 同口径）；
                    // 破阵诀：恒可用（enable true）
                    if kind == CardKind::Changshezhen
                        && !core.inline(catalog, core.turn.current)
                        && !core
                            .alive_players()
                            .any(|p| {
                                p != core.turn.current
                                    && core.is_friend_of(catalog, core.turn.current, p)
                            })
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                    // 地载阵：须存在非大势力存活角色（上游 enable 同口径）
                    if kind == CardKind::Dizaizhen
                        && !core
                            .alive_players()
                            .any(|p| !core.is_major_faction(catalog, p))
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                    validate_wuzhong_targets(core, catalog, card, kind, &targets)?;
                    if play_card_targets::try_recast_l11_trick(state, catalog, card, kind, &targets)? {
                        return Ok(());
                    }
                    let core = &state.core;
                    let mut targets = targets;
                    if kind == CardKind::Wuzhong
                        && kind == catalog.card_kind(card)
                        && catalog
                            .cards
                            .get(card as usize)
                            .is_some_and(|cd| cd.name == "txhj_yiyi")
                        && targets.is_empty()
                    {
                        let actor = core.turn.current;
                        targets = core
                            .seat_order_from(actor)
                            .into_iter()
                            .filter(|&p| {
                                core.player(p).alive
                                    && core.is_friend_of(catalog, actor, p)
                                    && !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                                        core,
                                        catalog,
                                        actor,
                                        p,
                                        card,
                                        kind,
                                        crate::event::DamageNature::Normal,
                                    )
                            })
                            .collect();
                        if targets.is_empty() {
                            return Err(RejectReason::IllegalTargets);
                        }
                    }
                    let player = core.turn.current;
                    if matches!(kind, CardKind::ShanDian | CardKind::HongShui)
                        && (catalog.blocks_delayed_trick(core, player)
                            || crate::rvm_bridge::modifiers::eval_card_target_canceled(
                                core,
                                catalog,
                                player,
                                player,
                                card,
                                kind,
                                crate::event::DamageNature::Normal,
                            ))
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                    if core
                        .player(player)
                        .judge
                        .iter()
                        .any(|c| catalog.resolved_card_kind(core, *c) == kind)
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                    // 良策转化（玩点论杀批 15）：基本牌当五谷——use_as 声明
                    // 必须随帧（否则机器工厂回退目录种）。
                    let use_as = (kind != catalog.resolved_card_kind(core, card)
                        || !core.player(player).hand.contains(&card))
                    .then_some(kind);
                    crate::resolution::push_frame(
                        &mut state.resolution,
                        EventData::CardUsed(crate::machine::card_use_as(
                            card, player, targets, use_as,
                        )),
                    );
                    return Ok(());
                }
                CardKind::Tianfuzhen | CardKind::Qixingzhen | CardKind::Niaoxiangzhen => {
                    // 阵法三张（U05 zhenfa，规则 v200/T19a；上游
                    // guozhanZhenfaEnabled=存活≥4 同口径；派生查询见
                    // state.rs）。天覆阵=AOE 全部大势力且有牌者；七星阵
                    // =自目标（须围攻/被围攻）；鸟翔阵=AOE 全部合法目
                    // 标（异势力、已知、与邻座同势力）。
                    if core.players.len() < 4 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    match kind {
                        CardKind::Tianfuzhen => {
                            if !core.alive_players().any(|p| core.is_major_faction(catalog, p)) {
                                return Err(RejectReason::IllegalTargets);
                            }
                            let expected: Vec<u8> = core
                                .alive_players()
                                .filter(|p| {
                                    core.is_major_faction(catalog, *p)
                                        && (!core.player(*p).hand.is_empty()
                                            || !core.player(*p).equips.is_empty())
                                })
                                .collect();
                            if targets != expected {
                                return Err(RejectReason::IllegalTargets);
                            }
                        }
                        CardKind::Qixingzhen => {
                            let cur = core.turn.current;
                            let under_siege = core.alive_players().any(|q| {
                                core.sieges(catalog, q, cur)
                            });
                            let besieging = core.alive_neighbors(cur).0.is_some_and(|n| {
                                core.sieged(catalog, n)
                            }) || core.alive_neighbors(cur).1.is_some_and(|pv| {
                                core.sieged(catalog, pv)
                            });
                            if !(under_siege || besieging) || targets != [cur] {
                                return Err(RejectReason::IllegalTargets);
                            }
                        }
                        _ => {
                            // 鸟翔阵：targets=全部合法目标（异势力、已知、
                            // 与自身邻座同势力）
                            let expected: Vec<u8> = core
                                .alive_players()
                                .filter(|t| {
                                    *t != core.turn.current
                                        && !core.is_friend_of(catalog, core.turn.current, *t)
                                        && {
                                            let (n, pv) = core.alive_neighbors(*t);
                                            n.is_some_and(|q| {
                                                core.effective_faction(catalog, *t)
                                                    == core.effective_faction(catalog, q)
                                            }) || pv.is_some_and(|q| {
                                                core.effective_faction(catalog, *t)
                                                    == core.effective_faction(catalog, q)
                                            })
                                        }
                                })
                                .collect();
                            if targets != expected {
                                return Err(RejectReason::IllegalTargets);
                            }
                        }
                    }
                }
                CardKind::Jiedao => {
                    // 持有者装备区须有武器，另一角色在其攻击范围内。
                    if targets.len() != 2 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let (holder, victim) = (targets[0], targets[1]);
                    if holder == core.turn.current
                        || victim == core.turn.current
                        || holder == victim
                        || !core.player(holder).alive
                        || !core.player(victim).alive
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let has_weapon = core
                        .player(holder)
                        .equips
                        .iter()
                        .any(|c| {
                            catalog.card_slot_of(core, *c)
                                == Some(crate::catalog::EquipSlot::Weapon)
                        });
                    let in_range = core.in_attack_range(catalog, holder, victim);
                    if !has_weapon || !in_range {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::SavageAttack
                | CardKind::ArcheryAttack
                | CardKind::Jingleishan
                | CardKind::Chiyuxi => {
                    let cname = catalog.cards.get(card as usize).map(|c| c.name).unwrap_or("");
                    if kind == catalog.card_kind(card) && cname == "txhj_huoshaolianying" {
                        let player = core.turn.current;
                        let next = core.next_alive(player);
                        if next == player || !core.player(next).alive {
                            return Err(RejectReason::IllegalTargets);
                        }
                        if !(targets.is_empty() || (targets.len() == 1 && targets[0] == next)) {
                            return Err(RejectReason::IllegalTargets);
                        }
                        if crate::rvm_bridge::modifiers::eval_card_target_canceled(
                            core,
                            catalog,
                            player,
                            next,
                            card,
                            kind,
                            crate::event::DamageNature::Fire,
                        ) {
                            return Err(RejectReason::IllegalTargets);
                        }
                        crate::resolution::push_frame(
                            &mut state.resolution,
                            EventData::CardUsed(crate::machine::card_use(card, player, vec![next])),
                        );
                        return Ok(());
                    }
                    // AOE：目标 = 出牌时刻的全部其他存活玩家（自下家起按座位
                    // 顺序冻结），命令显式 targets 必须为空。惊雷山/炽羽袭/酒池肉林
                    // reverseOrder：结算序反转为从自己上家起逆座位序。
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let player = core.turn.current;
                    let aoe_nature = match kind {
                        CardKind::Chiyuxi => crate::event::DamageNature::Fire,
                        CardKind::Jingleishan => crate::event::DamageNature::Thunder,
                        _ => crate::event::DamageNature::Normal,
                    };
                    let mut aoe_targets: Vec<PlayerId> = core
                        .seat_order_from(core.next_alive(player))
                        .into_iter()
                        .filter(|p| {
                            *p != player
                                && !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                                    core, catalog, player, *p, card, kind, aoe_nature,
                                )
                        })
                        .collect();
                    if aoe_targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                    if matches!(kind, CardKind::Jingleishan | CardKind::Chiyuxi)
                        || (kind == catalog.card_kind(card) && cname == "txhj_jiuchiroulin")
                    {
                        aoe_targets.reverse();
                    }
                    // viewAs 转化（万钧装备牌当南蛮等）须随帧声明——工厂按
                    // 声明种路由 AOE 机，实体种会把机路由丢空
                    let declared =
                        if kind != catalog.resolved_card_kind(core, card) {
                            Some(kind)
                        } else {
                            None
                        };
                    crate::resolution::push_frame(
                        &mut state.resolution,
                        EventData::CardUsed(crate::machine::card_use_as(
                            card, player, aoe_targets, declared,
                        )),
                    );
                    return Ok(());
                }
                CardKind::Guisheqi => {
                    // 鬼蛇旗：恰 1 名任意存活目标（可含自己）
                    if targets.len() != 1 || !core.player(targets[0]).alive {
                        return Err(RejectReason::IllegalTargets);
                    }
                    crate::resolution::push_frame(
                        &mut state.resolution,
                        EventData::CardUsed(crate::machine::card_use(
                            card,
                            core.turn.current,
                            targets,
                        )),
                    );
                    return Ok(());
                }
                CardKind::Xianluhui => {
                    // 仙露回：AOE 全体受伤角色（含自己，reverseOrder 逆序近似
                    // 为逆座位序），命令显式 targets 必须为空。
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let player = core.turn.current;
                    let mut aoe_targets: Vec<PlayerId> = core
                        .seat_order_from(core.next_alive(player))
                        .into_iter()
                        .filter(|p| {
                            *p != player && core.player(*p).alive && core.player(*p).hp < core.player(*p).max_hp as i32
                        })
                        .collect();
                    if core.player(player).hp < core.player(player).max_hp as i32 {
                        aoe_targets.push(player);
                    }
                    aoe_targets.reverse();
                    crate::resolution::push_frame(
                        &mut state.resolution,
                        EventData::CardUsed(crate::machine::card_use(
                            card, player, aoe_targets,
                        )),
                    );
                    return Ok(());
                }
                CardKind::HorsePlus
                | CardKind::HorseMinus
                | CardKind::Weapon
                | CardKind::Armor
                | CardKind::Suolianjia
                | CardKind::WuxingPan
                | CardKind::Sanjian
                // 猴子/写轮眼/酒尾（E03）：equip5 宝物，equip 路径按 is_equip
                // 提前处理。
                | CardKind::Monkey
                | CardKind::Xuelunyang
                | CardKind::Jiuwei
                | CardKind::Mianju
                | CardKind::Renwang
                | CardKind::Qinggang
                | CardKind::Qilin
                | CardKind::Hanbing
                | CardKind::Zhuge
                | CardKind::Guanshi
                | CardKind::Qinglong
                | CardKind::Zhangba
                | CardKind::Cixiong
                | CardKind::Fangtian
                | CardKind::Chilongya
                | CardKind::Xixueguizhihuan
                | CardKind::Shentoumianju
                | CardKind::Xingjunyan
                | CardKind::Qinglonglingzhu => {
                    // 占栏装备：equip 路径在上方按 is_equip 提前处理；
                    // 此臂仅为穷尽性——正常不会到达
                    return Err(RejectReason::IllegalTargets);
                }
                CardKind::Guiyoujie => {
                    // 归有界（U05 swd，规则 v91）：他人延时锦囊——恰 1 名
                    // 其他存活目标，且未免疫延时锦囊、判定区无同名延时
                    if targets.len() != 1
                        || !core.player(targets[0]).alive
                        || targets[0] == core.turn.current
                        || catalog.blocks_delayed_trick(core, targets[0])
                        || crate::rvm_bridge::modifiers::eval_card_target_canceled(
                            core,
                            catalog,
                            core.turn.current,
                            targets[0],
                            card,
                            kind,
                            crate::event::DamageNature::Normal,
                        )
                        || core.player(targets[0])
                            .judge
                            .iter()
                            .any(|c| catalog.resolved_card_kind(core, *c) == CardKind::Guiyoujie)
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Zhufangshenshi => {
                    // 贴房绅视（U05 swd 第十批，规则 v94）：恰 1 名其他
                    // 存活目标（上游 filterTarget target !== player）
                    if targets.len() != 1
                        || !core.player(targets[0]).alive
                        || targets[0] == core.turn.current
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Fengxueren
                | CardKind::Yufulu
                | CardKind::Daihuofenglun
                | CardKind::Xiayuncailing
                | CardKind::Langeguaiyi => {
                    // 装备牌（风雪刃 equip1/御夫录·蓝格怪衣 equip5/带火
                    // 风轮 equip4/霞云彩绫 equip3）：走装配路径（is_equip
                    // 提前处理），出牌区使用为穷尽性占位拒绝
                    return Err(RejectReason::IllegalTargets);
                }
                CardKind::Caoyao => {
                    // 草药（U05 swd 第三批，规则 v87）：为距离 1 以内的
                    // 受伤其他目标回复 1 点体力（上游 range.global 1 +
                    // filterTarget hp<maxHp + recover）。
                    if targets.len() != 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let t = targets[0];
                    let tp = core.player(t);
                    if !tp.alive || tp.hp >= tp.max_hp as i32 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let user = core.turn.current;
                    let shanxian_free = core
                        .vm_aux
                        .flags
                        .get(&(user, "ow_shanxian_target".to_string()))
                        .copied()
                        == Some(u32::from(t) + 1);
                    if !shanxian_free
                        && !xinfu_limu_in_range(core, catalog, user, t)
                        && !crate::rvm_bridge::modifiers::tanbei_unlimited_target(
                            core, catalog, user, t,
                        )
                        && !crate::rvm_bridge::modifiers::eval_card_distance_unlimited_to(
                            core,
                            catalog,
                            user,
                            Some(t),
                            card,
                            CardKind::Caoyao,
                            crate::event::DamageNature::Normal,
                        )
                        && core.computed_distance(catalog, user, t) > 1
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                CardKind::Tao | CardKind::Hufu => {
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let user = core.turn.current;
                    if super::play_card_targets::is_shihuawuqi_card(core, catalog, user, card) {
                        // 石化武器（`card/hearth/card.js:342`）：满体力亦可对自己使用，每回合限一次
                        if core
                            .vm_aux
                            .flags
                            .get(&(user, "hs_shihuawuqi_used_turn".to_string()))
                            .copied()
                            .unwrap_or(0)
                            > 0
                        {
                            return Err(RejectReason::CardLimit);
                        }
                    } else {
                        // 覆泰2（Boss 穷奇，规则 v214）：全场禁桃时非穷奇玩
                        // 家非穷奇回合不能用桃（上游 cardEnabled 同口径；护
                        // 符濒死专用不受此 mod）
                        if kind == CardKind::Tao
                            && crate::kernel::tao_futai2_banned(core, catalog, user)
                        {
                            return Err(RejectReason::IllegalTargets);
                        }
                        let p = core.player(user);
                        if kind == CardKind::Tao
                            && p.dying
                            && crate::rvm_bridge::modifiers::eval_dying_peach_target_canceled(
                                core, catalog, user, user, card,
                            )
                        {
                            return Err(RejectReason::IllegalTargets);
                        }
                        // 护符（U05 swd 首切片，规则 v85）：仅濒死（自己进入
                        // 濒死流程）时可用；非濒死拒绝（上游 savable 守卫）
                        if kind == CardKind::Hufu && !p.dying {
                            return Err(RejectReason::IllegalTargets);
                        }
                        if p.hp >= p.max_hp as i32 {
                            return Err(RejectReason::HpFull);
                        }
                    }
                }
                CardKind::Jiu | CardKind::Tianxianjiu => {
                    // 酒/天仙酒（swd 变体，规则 v89；登记差异：与酒共享
                    // 每回合一次计数）：出牌阶段对自身使用，每回合限一次
                    // ——本回合使用的下一张杀伤害 +1；濒死使用走濒死救援
                    // 通道（不经此命令）。
                    if !targets.is_empty() {
                        return Err(RejectReason::IllegalTargets);
                    }
                    // 焚天（Boss 朱雀，规则 v236）：红色酒不受每回合一次
                    // 限制（上游 mod cardUsable Infinity 同口径）
                    let fentian_red_jiu = {
                        let red = matches!(
                            catalog.resolved_card_suit(core, card),
                            crate::catalog::Suit::Heart
                                | crate::catalog::Suit::Diamond
                        );
                        red
                            && catalog
                                .skill_by_name(crate::content::standard::BOSS_FENTIAN)
                                .map(|def| core.has_skill(core.turn.current, def))
                                .unwrap_or(false)
                    };
                    let bushi_unlimited = crate::rvm_bridge::modifiers::d34_l28_mods::bushi_unlimited_card(
                        core,
                        catalog,
                        core.turn.current,
                        Some(card),
                    );
                    if core.turn.jiu_used && !fentian_red_jiu && !bushi_unlimited {
                        return Err(RejectReason::CardLimit);
                    }
                }
                CardKind::ShouLiJian => {
                    // 手里剑（U06 欢乐卡牌 card.js:39-40）：恰 1 名距离 > 1 的其他存活目标（outrange: { global: 2 }）
                    if targets.len() != 1 {
                        return Err(RejectReason::IllegalTargets);
                    }
                    let target = targets[0];
                    if target == core.turn.current
                        || !core.player(target).alive
                        || core.computed_distance(catalog, core.turn.current, target) <= 1
                    {
                        return Err(RejectReason::IllegalTargets);
                    }
                }
                // 闪/无懈可击/帝女玄霜：仅在响应或濒死窗口使用，不可主动使用
                CardKind::Shan | CardKind::Nullify | CardKind::Dinvxuanshuang => {
                    return Err(RejectReason::WrongPhase);
                }
                CardKind::Shuigong
                | CardKind::Fudichouxin
                | CardKind::Chenhuodajie
                | CardKind::Linghunzhihuo
                | CardKind::Zhaomingdan
                | CardKind::Jihuocard
                | CardKind::Youdishenru
                | CardKind::Caochuanjiejian
                | CardKind::Shezhanqunru
                | CardKind::Geanguanhuo
                | CardKind::Toulianghuanzhu
                | CardKind::WangmeiZhike
                | CardKind::Shujinsan
                | CardKind::Mianlijinzhen
                | CardKind::Feibiao
                | CardKind::Jinlianzhu
                | CardKind::Ziyangdan
                | CardKind::Shihuifen
                | CardKind::Liufengsan
                | CardKind::Shatang
                | CardKind::Liutouge
                | CardKind::Heilonglinpian
                | CardKind::Mutoumianju
                | CardKind::Gjyuheng
                | CardKind::Dunpaigedang
                | CardKind::Xingjiegoutong
                | CardKind::Chuansongmen
                | CardKind::Tanshezhiren
                | CardKind::Zhiliaobo
                | CardKind::Yuansuhuimie
                | CardKind::Shijieshu
                | CardKind::Kunenfayin
                | CardKind::Poxiao
                | CardKind::Zumoshoukao
                | CardKind::Shanbengshu
                | CardKind::Yanziyaoshui
                | CardKind::Aozuzhilei
                | CardKind::Wenyi
                | CardKind::Zhihuanjun
                | CardKind::Baoxueyaoshui
                | CardKind::Liuxinghuoyu
                | CardKind::Jiguan
                | CardKind::Hslingjian
                | CardKind::Qinpendayu
                | CardKind::Birinongwu
                | CardKind::Ciguhanshuang
                | CardKind::Bingpotong
                | CardKind::Qiankunbiao
                | CardKind::Shenhuofeiya
                | CardKind::Longxugou
                | CardKind::Tanhuadong
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
                | CardKind::Yunvyuanshen => {
                    play_card_targets::validate_tail_ext_targets(core, catalog, card, kind, &targets)?;
                }
            }
    play_card_targets::push_play_card_frame(state, catalog, card, kind, targets)
}
