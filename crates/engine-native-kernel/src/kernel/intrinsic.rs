use super::*;

/// 帧固有流程机器（`Intrinsic` 阶段启动）。
pub fn intrinsic_machine(
    ctx: &mut Ctx<'_>,
    frame_id: EventId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    let data = ctx.res.frame(frame_id).expect("frame").data.clone();
    match data {
        EventData::OrdinaryJudge { owner, invocation } => {
            if invocation == 0
                || !ctx
                    .res
                    .frame(invocation)
                    .is_some_and(|f| matches!(f.data, EventData::SkillInvocation(_)))
            {
                return None;
            }
            let mut m = JudgePhaseMachine::new(owner, Vec::new(), None);
            m.ordinary = Some(invocation);
            Some(Box::new(m))
        }
        EventData::SkillInvocation(data) if data.phase == crate::event::InvocationPhase::Body => {
            let window = match ctx.res.frame(data.window)?.data {
                EventData::DamageWindow { window, .. } => window,
                _ => return None,
            };
            Some(Box::new(NativeInvocation {
                owner: data.owner,
                def: data.def,
                window,
                progress: crate::event::NativeInvocationProgress::Body,
            }))
        }
        EventData::Damage(_) => Some(Box::new(DamageFlow { root: frame_id })),
        EventData::DrawPhase { .. } => Some(Box::new(crate::kernel::DrawPhaseMachine {
            root: frame_id,
        })),
        EventData::Draw { .. } => Some(Box::new(crate::kernel::DrawMachine {
            root: frame_id,
        })),
        EventData::DrawGain { .. } => Some(Box::new(crate::kernel::GainMachine {
            root: frame_id,
        })),
        EventData::LoseHp { .. } => None,
        EventData::DyingSaved { .. } => None,
        EventData::Dying { player, source } => Some(Box::new(DyingSaveMachine {
            dying: player,
            source,
            responders: ctx.core.seat_order_from(ctx.core.turn.current),
            asked: false,
            round_used: false,
            guihan_asked: false,
        })),
        EventData::CardUsed(u) => {
            // 筑城守卫（U07 批三，规则 v113）等 AfterTriggers 机器写回的
            // 取消：跳过结算（杀已由守卫归还手牌）
            if u.cancelled {
                return None;
            }
            let effective_kind = u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog));
            // 目标被结算窗全部移出（贞烈等「移出目标」类，swd 批 139）：
            // 需要「恰一名目标」的牌种在目标清空后不结算（避免机器对
            // 空目标越界）；无目标/群体牌种照常（其机器自行处理空列表）
            if u.targets.is_empty()
                && matches!(
                    effective_kind,
                    CardKind::Sha
                        | CardKind::FireSha
                        | CardKind::LeiSha
                        | CardKind::Duel
                        | CardKind::Dismantle
                        | CardKind::Snatch
                        | CardKind::Le
                        | CardKind::Bing
                        | CardKind::Jiedao
                        | CardKind::Chain
                )
            {
                return None;
            }
            if effective_kind.is_sha_like() {
                return sha_intrinsic(ctx, &u);
            }
            let Some(card) = u.card else {
                match effective_kind {
                    CardKind::Tao => {
                        for &t in &u.targets {
                            if ctx.core.player(t).alive {
                                ctx.recover(t, 1, None);
                            }
                        }
                    }
                    CardKind::Jiu => {
                        ctx.core.turn.jiu_active = true;
                    }
                    CardKind::Wuzhong => {
                        return Some(Box::new(WuzhongMachine {
                            owner: u.player,
                            card: None,
                            draw_done: false,
                        }));
                    }
                    CardKind::Taoyuan => {
                        return Some(Box::new(TaoyuanMachine {
                            owner: u.player,
                            card: None,
                        }));
                    }
                    CardKind::Chain => {
                        for &t in &u.targets {
                            if ctx.core.player(t).alive {
                                let next = !ctx.core.player(t).linked;
                                ctx.emit(EventData::Linked {
                                    player: t,
                                    linked: next,
                                });
                            }
                        }
                    }
                    CardKind::Dismantle => {
                        if let Some(&target) = u.targets.first() {
                            return Some(Box::new(DismantleMachine {
                                card: 0,
                                source: u.player,
                                target,
                                asked: false,
                            }));
                        }
                    }
                    CardKind::Snatch => {
                        if let Some(&target) = u.targets.first() {
                            return Some(Box::new(SnatchMachine {
                                card: 0,
                                source: u.player,
                                target,
                                asked: false,
                            }));
                        }
                    }
                    CardKind::Wugu => {
                        return Some(Box::new(WuguMachine {
                            owner: u.player,
                            cards: Vec::new(),
                            pickers: Vec::new(),
                            stage: WuguStage::Init,
                        }));
                    }
                    CardKind::Duel => {
                        if let Some(&target) = u.targets.first() {
                            return Some(Box::new(DuelMachine {
                                card: 0,
                                source: u.player,
                                target,
                                on_target: true,
                                asked: false,
                                second_pending: false,
                            }));
                        }
                    }
                    _ => {}
                }
                return None;
            };
            // 内容卡牌（G1 宿主牌等）的固有结算机器由内容层提供（目录回调）；
            // 内容牌种与下方内核牌种互斥，且均无 aoe_need，先问内容层不改变分派结果。
            if let Some(machine) = (ctx.catalog.hooks.card_intrinsic)(ctx, &u, effective_kind) {
                return machine;
            }
            match effective_kind {
                CardKind::Dismantle => {
                    let effect = DismantleMachine {
                        card: card,
                        source: u.player,
                        target: *u.targets.first().expect("dismantle has one target"),
                        asked: false,
                    };
                    Some(Box::new(wrap_nullify_window(
                        ctx,
                        card,
                        u.player,
                        Box::new(effect),
                    )))
                }
                // 无中生有（P2/N08 缺牌种批①）：使用后摸 2（无懈窗口包装）。
                CardKind::Wuzhong => Some(Box::new(wrap_nullify_window(
                    ctx,
                    card,
                    u.player,
                    Box::new(WuzhongMachine {
                        owner: u.player,
                        card: Some(card),
                        draw_done: false,
                    }),
                ))),
                // 桃园结义（P2/N08 缺牌种批①）：所有其他存活角色回复 1 点
                // 体力（无懈窗口包装）。
                CardKind::Taoyuan => Some(Box::new(wrap_nullify_window(
                    ctx,
                    card,
                    u.player,
                    Box::new(TaoyuanMachine {
                        owner: u.player,
                        card: Some(card),
                    }),
                ))),
                // 五谷丰登（批①-2）：亮存活角色数的牌，从使用者起按座位序
                // 逐人择一（无懈窗口包装）。
                CardKind::Wugu => Some(Box::new(wrap_nullify_window(
                    ctx,
                    card,
                    u.player,
                    Box::new(WuguMachine {
                        owner: u.player,
                        cards: Vec::new(),
                        pickers: Vec::new(),
                        stage: WuguStage::Init,
                    }),
                ))),
                // 借刀杀人（批①-2）：武器持有者出杀（登记差异：直接伤害
                // 结算，无出闪窗口）或交出武器（无懈窗口包装）。
                CardKind::Jiedao => {
                    let holder = *u.targets.first().expect("jiedao two targets");
                    let victim = *u.targets.get(1).expect("jiedao two targets");
                    Some(Box::new(wrap_nullify_window(
                        ctx,
                        card,
                        u.player,
                        Box::new(JiedaoMachine {
                            owner: u.player,
                            holder,
                            victim,
                            card: Some(card),
                            stage: JiedaoStage::Ask,
                        }),
                    )))
                }
                CardKind::Snatch => {
                    let effect = SnatchMachine {
                        card: card,
                        source: u.player,
                        target: *u.targets.first().expect("snatch has one target"),
                        asked: false,
                    };
                    Some(Box::new(wrap_nullify_window(
                        ctx,
                        card,
                        u.player,
                        Box::new(effect),
                    )))
                }

                CardKind::Dizaizhen => {
                    // 地载阵（U05 zhenfa 首切片，规则 v110）：无懈窗口包装
                    // + 所有非大势力存活角色各摸一张
                    let effect = Box::new(DizaizhenMachine { owner: u.player });
                    Some(Box::new(wrap_nullify_window(ctx, card, u.player, effect)))
                }
                kind if kind.aoe_need().is_some() => {
                    // AOE 的无懈窗口为**逐目标**（由 AoeResolveMachine 对每个目标
                    // 自行开窗），外层不再包整牌级窗口——旧包裹会吞掉应答并把
                    // 整张牌作废（本批修复，ADR 0705）。
                    Some(Box::new(AoeResolveMachine {
                        card: card,
                        source: u.player,
                        need: kind.aoe_need().expect("checked"),
                        targets: u.targets,
                        idx: 0,
                        inner: None,
                        nature: kind.aoe_nature(),
                    }))
                }
                CardKind::Chain => {
                    // 铁索的逐目标无懈窗口由 ChainMachine 自身对每个目标开启
                    // （对齐上游 useCardToBegin 逐目标窗口），外层不再包整牌级
                    // 窗口——旧包裹在任一无懈生效时令**全部**目标不横置。
                    Some(Box::new(ChainMachine::dispatcher(
                        card, u.player, u.targets,
                    )))
                }
                CardKind::Duel => {
                    let effect = DuelMachine {
                        card: card,
                        source: u.player,
                        target: *u.targets.first().expect("duel has one target"),
                        on_target: true,
                        asked: false,
                        second_pending: false,
                    };
                    Some(Box::new(wrap_nullify_window(
                        ctx,
                        card,
                        u.player,
                        Box::new(effect),
                    )))
                }
                CardKind::Zhufangshenshi => {
                    // 贴房绅视（U05 swd 第十批，规则 v94）：他人锦囊——本回合
                    // 无视距离指定该目标并摸一张牌。效果在 intrinsic 提交段
                    // 同步执行（无需机器）；此处仅占位以透过穷尽 match。
                    None
                }
                CardKind::Guiyoujie => {
                    // 归有界（U05 swd，规则 v91）：他人延时锦囊——判定为黑色
                    // 则失去 1 点体力 + 随机弃一张牌（上游 loseHp+randomDiscard）
                    let target = *u.targets.first().expect("guiyoujie has one target");
                    let effect = PlaceDelayedMachine {
                        card: card,
                        target,
                        placed: false,
                    };
                    Some(Box::new(wrap_nullify_window(
                        ctx,
                        card,
                        u.player,
                        Box::new(effect),
                    )))
                }
                CardKind::Le | CardKind::Bing => {
                    // 延时锦囊使用时即开无懈窗口（对齐旧线：牌先入桌面，
                    // 无懈后入判定区、被无懈则弃置），窗口未抵消由
                    // PlaceDelayedMachine 置入目标判定区。
                    let target = *u.targets.first().expect("delayed trick has one target");
                    let effect = PlaceDelayedMachine {
                        card: card,
                        target,
                        placed: false,
                    };
                    Some(Box::new(wrap_nullify_window(
                        ctx,
                        card,
                        u.player,
                        Box::new(effect),
                    )))
                }
                CardKind::ShanDian => {
                    let effect = PlaceDelayedMachine {
                        card: card,
                        target: u.player,
                        placed: false,
                    };
                    Some(Box::new(wrap_nullify_window(
                        ctx,
                        card,
                        u.player,
                        Box::new(effect),
                    )))
                }
                CardKind::HongShui => {
                    // 洪水：自-target 延时锦囊（判定：梅花 2–9 → 3 点雷伤）
                    let effect = PlaceDelayedMachine {
                        card: card,
                        target: u.player,
                        placed: false,
                    };
                    Some(Box::new(wrap_nullify_window(
                        ctx,
                        card,
                        u.player,
                        Box::new(effect),
                    )))
                }
                _ => None,
            }
        }
        EventData::Phase {
            player,
            phase: Phase::Judge,
            begin: true,
        } => {
            if ctx
                .core
                .vm_aux
                .flags
                .remove(&(player, "skip_judge".to_string()))
                .is_some()
            {
                ctx.record_phase_skipped(player);
                return None;
            }
            let judge = std::mem::take(&mut ctx.core.player_mut(player).judge);
            if judge.is_empty() {
                None
            } else {
                // 判定排序分组（N06-a，对齐旧线 pending_delayed_cards）：
                // 非闪电延时锦囊按放置序在前、闪电在后，队首先消费。
                let mut queue: Vec<crate::ids::CardId> = judge
                    .iter()
                    .copied()
                    .filter(|c| ctx.catalog.card_kind(*c) != CardKind::ShanDian)
                    .collect();
                queue.extend(
                    judge
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c) == CardKind::ShanDian),
                );
                Some(Box::new(JudgePhaseMachine {
                    owner: player,
                    queue,
                    current: None,
                    ordinary: None,
                    started: false,
                    payment: None,
                }))
            }
        }
        EventData::Phase {
            player,
            phase: Phase::Discard,
            begin: true,
        } => {
            // 贪欲（v215）/庆祝（v245）：跳弃牌族——弃牌机器不生成（上游
            // phaseDiscardBefore trigger.cancel 同口径；此前贪欲面被测试
            // drain 掩盖，本臂补真实闸门）
            let mut skip_discard = [
                crate::vocabulary::names::BOSS_TANYU,
                crate::vocabulary::names::BOSS_QINGZHU,
            ]
            .iter()
            .any(|name| {
                ctx.catalog
                    .skill_by_name(name)
                    .map(|d| ctx.core.has_skill(player, d))
                    .unwrap_or(false)
            });
            // 猛态（Boss 年兽，规则 v248）：本回合未进入出牌阶段 → 跳过
            // 弃牌阶段（上游 boss_mengtai_discard trigger.cancel 同口径）
            if ctx.core.player(player).skip_discard_phase {
                ctx.core.player_mut(player).skip_discard_phase = false;
                skip_discard = true;
            }
            if !skip_discard && ctx.core.turn.mengtai_use_pending {
                if let Some(mtdef) = ctx
                    .catalog
                    .skill_by_name(crate::vocabulary::names::BOSS_MENGTAI)
                {
                    if ctx.core.has_skill(player, mtdef) {
                        skip_discard = true;
                    }
                }
            }
            if skip_discard {
                ctx.core.turn.discard_phase_discarded = 0;
                None
            }
            // 克己（吕蒙）：本回合未使用杀则无需弃牌（手牌上限视为无限）
            else {
                let keji = ctx
                    .catalog
                    .skill_by_name(crate::vocabulary::names::KE_JI)
                    .map(|d| ctx.core.has_skill(player, d))
                    .unwrap_or(false);
                if keji && !ctx.core.turn.sha_used_play_phase {
                    None
                } else {
                    let mut limit = ctx.core.hand_limit(ctx.catalog, player);
                    // 机甲（U05 ow D.Va，规则 v141）：机甲血>0 时手牌上限 +4
                    //（上游 mod maxHandcard storage.jijia 同口径）
                    if ctx
                        .catalog
                        .skill_by_name(crate::vocabulary::host::WUDIAN_JIJIA_KEY)
                        .map(|def| ctx.core.has_skill(player, def))
                        .unwrap_or(false)
                        && ctx.core.player(player).jijia_hp > 0
                    {
                        limit += 4;
                    }
                    // 蛊舞·缚（U05 xianjian 阿奴，规则 v154）：蛊舞生效中手牌
                    // 上限 -1（上游 anwugu2 mod maxHandcard 同口径）
                    if ctx.core.player(player).anwugu_count > 0 {
                        limit = limit.saturating_sub(1);
                    }
                    // R05-F（§5 Limit）：最终上限在其他基数/修饰之后加全部
                    // 启用 Limit 槽的有符号值并 clamp 0（负数不 cast usize；
                    // disabled 的 Limit 不计入）。
                    for def in (ctx.catalog.hooks.r05_role_defs)(ctx.catalog, "limit") {
                        if let Some(inst) = ctx.core.skills.get(&(player, def)) {
                            if !inst.enabled {
                                continue;
                            }
                            let delta = ctx
                                .core
                                .vm_aux
                                .marks
                                .get(&(player, def as u32))
                                .copied()
                                .unwrap_or(0);
                            let next = (limit as i64) + delta as i64;
                            limit = next.clamp(0, i32::MAX as i64) as usize;
                        }
                    }
                    // 春饼（G1 古剑食物）：手牌上限 +1（持有状态即生效）
                    if let Some(def) = ctx
                        .catalog
                        .skill_by_name(crate::vocabulary::host::GJ_FOOD_CHUNBING_KEY)
                    {
                        if ctx
                            .core
                            .skills
                            .get(&(player, def))
                            .map(|i| i.enabled)
                            .unwrap_or(false)
                        {
                            limit += 1;
                        }
                    }
                    // 壶境（壶中仙，swd 卡牌面补切批③）：装备炼妖壶时手牌上限
                    // +2（上游 hujing mod maxHandcard getEquip("lianyaohu") 同口径）
                    if ctx
                        .core
                        .player(player)
                        .equips
                        .iter()
                        .any(|c| ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Lianyaohu)
                    {
                        limit += 2;
                    }
                    // 乾坤袋（swd 卡牌面补切批④）：装备时手牌上限 +1（上游
                    // qiankundai mod maxHandcard 同口径）
                    if ctx
                        .core
                        .player(player)
                        .equips
                        .iter()
                        .any(|c| ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Qiankundai)
                    {
                        limit += 1;
                    }
                    // 倾盆大雨（gwent 天气）：手牌上限 -1
                    if let Some(def) = ctx
                        .catalog
                        .skill_by_name(crate::vocabulary::host::GW_QINPEN_STATE_KEY)
                    {
                        if ctx
                            .core
                            .skills
                            .get(&(player, def))
                            .map(|i| i.enabled)
                            .unwrap_or(false)
                        {
                            limit = limit.saturating_sub(1);
                        }
                    }
                    // 妄尊印（袁术妄尊授予的回合级印记）：手牌上限 -1
                    if let Some(def) = ctx
                        .catalog
                        .skill_by_name(crate::vocabulary::names::WANG_ZUN_MARK)
                    {
                        if ctx
                            .core
                            .skills
                            .get(&(player, def))
                            .map(|i| i.enabled)
                            .unwrap_or(false)
                        {
                            limit = limit.saturating_sub(1);
                        }
                    }
                    // 水没墓穴地图效果（mtg 第三批，规则 v102）：锁定全局——
                    // 手牌上限 -1（current_land 状态直读，同倾盆大雨口径）
                    if ctx.core.current_land == Some(crate::catalog::CardKind::Shuimomuxue) {
                        limit = limit.saturating_sub(1);
                    }
                    // 沙漠绿洲地图效果（mtg 第六批，规则 v105）：基本牌不
                    // 计入手牌上限——需弃数按非基本手牌数计（上游
                    // ignoredHandcard basic 同口径）
                    let shamolvzhou =
                        ctx.core.current_land == Some(crate::catalog::CardKind::Shamolvzhou);
                    // 红莲（Boss 毕方，规则 v228）：红色牌不计入手牌上限
                    //（上游 ignoredHandcard red 同口径）
                    let honglianx = ctx
                        .catalog
                        .skill_by_name(crate::vocabulary::names::BOSS_HONGLIANX)
                        .map(|def| ctx.core.has_skill(player, def))
                        .unwrap_or(false);
                    let hand = if shamolvzhou {
                        ctx.core
                            .player(player)
                            .hand
                            .iter()
                            .filter(|c| {
                                crate::vocabulary::card_skills::yangpijuan_class(
                                    ctx.catalog.card_kind(**c),
                                ) != Some(0)
                            })
                            .count()
                    } else if honglianx {
                        ctx.core
                            .player(player)
                            .hand
                            .iter()
                            .filter(|c| {
                                !matches!(
                                    ctx.catalog.cards[**c as usize].suit,
                                    crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                                )
                            })
                            .count()
                    } else {
                        ctx.core
                            .player(player)
                            .hand
                            .len()
                            .saturating_sub(ctx.core.ignored_hand_card_count(ctx.catalog, player))
                    };
                    if hand > limit {
                        Some(Box::new(DiscardToLimitMachine {
                            player,
                            need: hand - limit,
                            asked: false,
                        }))
                    } else {
                        None
                    }
                }
            }
        }
        EventData::Death { player, killer } => Some(Box::new(DeathMachine { player, killer })),
        _ => None,
    }
}

fn sha_intrinsic(
    ctx: &mut Ctx<'_>,
    u: &crate::event::CardUseData,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    // 方天画戟（批③-b 第九把）：多目标队列（首目标即期结算，
    // 其余入队列逐目标推进）。
    let mut it = u.targets.iter().copied();
    let first = it.next().expect("sha has one target");
    let extra: Vec<PlayerId> = it.collect();
    let cid = u.card.unwrap_or(0);
    let shoulie_direct = ctx
        .core
        .vm_aux
        .flags
        .remove(&(u.player, format!("sha_unrespondable:{cid}:{first}")))
        .is_some()
        || ctx
            .core
            .vm_aux
            .flags
            .remove(&(u.player, format!("sha_unrespondable:{cid}:{}", u.player)))
            .is_some()
        || ctx.core.is_card_use_unrespondable(
            ctx.catalog,
            u.player,
            first,
            cid,
            crate::catalog::CardKind::Sha,
            u.nature,
        );
    Some(Box::new(ShaResolveMachine {
        card: u.card,
        source: u.player,
        target: first,
        nature: u.nature,
        asked: false,
        armor_judged: false,
        dodges_used: 0,
        tieji_red: None,
        liuli_done: false,
        liuli_stage: 0,
        liuli_card: None,
        asking_qilin: false,
        qilin_horse: None,
        asking_hanbing: false,
        asking_fengxueren: false,
        asking_guanshi: false,
        guanshi_cards: Vec::new(),
        asking_qinglong: false,
        asking_cixiong: false,
        cixiong_done: false,
        hanbing_pending: 0,
        asking_xiandan: false,
        xiandan_done: false,
        xiandan_bonus: false,
        asking_lueying: false,
        lueying_victim: None,
        lueying_declined: false,
        asking_fuchou: false,
        fuchou_card: None,
        asking_sadeng: false,
        asking_sadeng_shan: false,
        shoulie_direct,
        extra_targets: extra,
        hujia_queue: None,
        hujia_holder: None,
        // 酒效果在杀派发（=使用）时捕获并消耗——「下一张杀」
        jiu_bonus: {
            let jb = std::mem::take(&mut ctx.core.turn.jiu_active);
            if jb
                && ctx
                    .core
                    .vm_aux
                    .flags
                    .remove(&(u.player, "mbcmjiushi_jiu_active".to_string()))
                    .unwrap_or(0)
                    > 0
            {
                if let Some(cid) = u.card {
                    ctx.core
                        .vm_aux
                        .flags
                        .insert((u.player, format!("mbcmjiushi_boosted_card:{cid}")), 1);
                }
            }
            jb
        },
    }))
}
