//! 卡牌使用帧（`EventData::CardUsed`）的固有结算分派（自 `kernel/intrinsic.rs` 职责拆出）。

use super::*;

fn strip_digit_suffix(raw: &str) -> &str {
    match raw.rsplit_once('_') {
        Some((stem, tail)) if !tail.is_empty() && tail.bytes().all(|b| b.is_ascii_digit()) => {
            stem
        }
        _ => raw,
    }
}

fn split_jifeng_more_if_multi(
    ctx: &mut Ctx<'_>,
    frame_id: EventId,
    u: &mut crate::event::CardUseData,
) {
    if u.targets.len() > 1 {
        crate::resolution::pop_cleanup::setup_jifeng_more_targets(
            ctx.core, ctx.res, frame_id, u,
        );
    }
}

/// 卡牌使用帧（`EventData::CardUsed`）在 `Intrinsic` 阶段的固有机器分派。
pub(super) fn card_use_intrinsic(
    ctx: &mut Ctx<'_>,
    frame_id: EventId,
    mut u: crate::event::CardUseData,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    let dying_save = ctx
        .core
        .vm_aux
        .flags
        .remove(&(u.player, format!("dying_save_frame:{frame_id}")))
        .and_then(|packed| {
            let dying = ((packed & 0xff).checked_sub(1)?) as PlayerId;
            let raw_tao = packed >> 8;
            Some((dying, raw_tao))
        });
    let extra_carriers = crate::kernel::take_tao_extra_carriers(ctx.core, u.player, frame_id);
    let used_card = u.card;
    let discard_remaining_carriers = |ctx: &mut Ctx<'_>| {
        for c in used_card.into_iter().chain(extra_carriers.iter().copied()) {
            if ctx.core.table.contains(&c) {
                ctx.core.table.retain(|&x| x != c);
                ctx.to_discard(c);
            }
        }
    };
    // 筑城守卫（U07 批三，规则 v113）与猴子偷桃（U06 E03）等 AfterTriggers 机器写回的
    // 取消：跳过结算（清除待消费恢复加成并弃置处理区中残留载体）
    if u.cancelled {
        ctx.core
            .vm_aux
            .flags
            .remove(&(u.player, "vm_va_recover_bonus".to_string()));
        discard_remaining_carriers(ctx);
        return None;
    }
    // 追加/移除目标的 after 窗口已完成，按最终目标消费一次免疫。
    if dddanzhi_remove_targets(ctx, frame_id, &mut u) {
        ctx.core
            .vm_aux
            .flags
            .remove(&(u.player, "vm_va_recover_bonus".to_string()));
        discard_remaining_carriers(ctx);
        return None;
    }
    if dying_save.is_none() {
        let excl_kind = u.effective_kind_in(ctx.core, ctx.catalog);
        let excl_targets = match excl_kind {
            CardKind::Wugu if u.targets.is_empty() => ctx.core.alive_players().collect(),
            CardKind::Taoyuan if u.targets.is_empty() => {
                ctx.core.alive_players().filter(|&p| p != u.player).collect()
            }
            _ => u.targets.clone(),
        };
        let total = excl_targets.len();
        if total > 0 {
            let filtered: Vec<PlayerId> = excl_targets
                .into_iter()
                .filter(|&t| {
                    !crate::rvm_bridge::modifiers::d34_l28_mods::shengong_excludes_target(
                        ctx.core,
                        ctx.catalog,
                        u.player,
                        t,
                        u.card,
                        excl_kind,
                        u.nature,
                        total,
                    )
                })
                .collect();
            if filtered.len() != total {
                u.targets = filtered;
                ctx.res
                    .frame_mut(frame_id)
                    .expect("card use frame")
                    .card_use_mut()
                    .targets = u.targets.clone();
                if !u.targets.is_empty()
                    && matches!(excl_kind, CardKind::Wugu | CardKind::Taoyuan)
                {
                    ctx.core
                        .vm_aux
                        .flags
                        .insert((u.player, format!("dddanzhi_targets:{frame_id}")), 1);
                }
                if u.targets.is_empty()
                    || (excl_kind == CardKind::Jiedao && u.targets.len() < 2)
                {
                    ctx.core
                        .vm_aux
                        .flags
                        .remove(&(u.player, "vm_va_recover_bonus".to_string()));
                    discard_remaining_carriers(ctx);
                    return None;
                }
            }
        }
    }
    if let Some((dying, raw_tao)) = dying_save {
        discard_remaining_carriers(ctx);
        if u.card.is_some() {
            crate::kernel::dying::DyingSaveMachine::new(
                dying,
                None,
                Vec::new(),
                false,
                false,
            )
            .apply_tao_save(ctx, u.player, raw_tao);
        } else if ctx.core.player(dying).alive {
            ctx.recover(dying, 1, None);
        }
        for &extra in &u.targets {
            if extra != dying && ctx.core.player(extra).alive {
                ctx.recover(extra, 1, u.card);
            }
        }
        let eff = u.effective_kind_in(ctx.core, ctx.catalog);
        if eff == CardKind::Dinvxuanshuang && ctx.core.player(dying).alive {
            ctx.core
                .vm_aux
                .flags
                .insert((u.player, "dinv_already_recovered".to_string()), 1);
            let mut saved_u = u.clone();
            saved_u.targets = vec![dying];
            if let Some(machine) =
                (ctx.catalog.hooks.card_intrinsic)(ctx, &saved_u, CardKind::Dinvxuanshuang)
            {
                return machine;
            }
            ctx.core
                .vm_aux
                .flags
                .remove(&(u.player, "dinv_already_recovered".to_string()));
        }
        return None;
    }
    let effective_kind = u.effective_kind_in(ctx.core, ctx.catalog);
    if effective_kind == CardKind::Tao
        && (u.card.is_some()
            || ctx
                .core
                .vm_aux
                .flags
                .contains_key(&(u.player, "hs_virtual_card_src".to_string())))
    {
        if let Some(machine) = (ctx.catalog.hooks.card_intrinsic)(ctx, &u, effective_kind) {
            return machine;
        }
    }
    if effective_kind == CardKind::Tao {
        let bonus = ctx
            .core
            .vm_aux
            .flags
            .remove(&(u.player, "vm_va_recover_bonus".to_string()))
            .unwrap_or(0);
        discard_remaining_carriers(ctx);
        let amount = 1u32.saturating_add(bonus);
        if u.targets.is_empty() {
            if ctx.core.player(u.player).alive {
                ctx.recover(u.player, amount, u.card);
            }
        } else {
            for &t in &u.targets {
                if ctx.core.player(t).alive {
                    ctx.recover(t, amount, u.card);
                }
            }
        }
        return None;
    }
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
        discard_remaining_carriers(ctx);
        return None;
    }
    if effective_kind.is_sha_like() {
        return sha_intrinsic(ctx, &u);
    }
    let is_txhj_jiedao = effective_kind == CardKind::Duel
        && u.card.is_some_and(|c| {
            ctx.catalog
                .cards
                .get(c as usize)
                .is_some_and(|cd| cd.name == "txhj_jiedao")
        });
    let is_l11_single_target_trick = u.use_as.is_none()
        && u.card.is_some_and(|c| {
            ctx.catalog.cards.get(c as usize).is_some_and(|cd| {
                matches!(
                    strip_digit_suffix(cd.name),
                    "txhj_zhijizhibi"
                        | "txhj_yuanjiao"
                        | "txhj_diaohu"
                        | "txhj_caomu"
                        | "txhj_wangmeizhike"
                        | "txhj_shuiyan"
                        | "huogong"
                )
            })
        });
    let youji_dup = ctx
        .core
        .vm_aux
        .flags
        .remove(&(u.player, format!("txqyyouji_dup:{frame_id}")))
        .is_some();
    if is_l11_single_target_trick
        || youji_dup
        || (effective_kind.is_jifeng_single_target_trick()
            && effective_kind != CardKind::Zhufangshenshi
            && !is_txhj_jiedao)
    {
        split_jifeng_more_if_multi(ctx, frame_id, &mut u);
    }
    let Some(card) = u.card else {
        if matches!(
            effective_kind,
            CardKind::Youdishenru
                | CardKind::Feibiao
                | CardKind::Xingjiegoutong
                | CardKind::Jihuocard
                | CardKind::Qinpendayu
                | CardKind::Birinongwu
                | CardKind::Ciguhanshuang
                | CardKind::Zumoshoukao
                | CardKind::Kunenfayin
                | CardKind::Shanbengshu
                | CardKind::Poxiao
                | CardKind::Yanziyaoshui
                | CardKind::Aozuzhilei
                | CardKind::Wenyi
                | CardKind::Zhihuanjun
                | CardKind::Baoxueyaoshui
        ) || ctx
            .core
            .vm_aux
            .flags
            .get(&(u.player, "l11_virtual_huogong".to_string()))
            .copied()
            .unwrap_or(0)
            > 0
            || ctx
                .core
                .vm_aux
                .flags
                .get(&(u.player, "l11_virtual_trick_cid".to_string()))
                .copied()
                .unwrap_or(0)
                > 0
            || (effective_kind == CardKind::Wuzhong
                && ctx
                    .core
                    .vm_aux
                    .flags
                    .contains_key(&(u.player, "virtual_diaobing_use".to_string())))
            || ctx
                .core
                .vm_aux
                .flags
                .contains_key(&(u.player, "hs_virtual_card_src".to_string()))
        {
            if let Some(machine) = (ctx.catalog.hooks.card_intrinsic)(ctx, &u, effective_kind) {
                return machine;
            }
        }
        match effective_kind {
            CardKind::Jiu => {
                ctx.core.turn.jiu_active = true;
            }
            CardKind::Wuzhong => {
                split_jifeng_more_if_multi(ctx, frame_id, &mut u);
                let target = u.targets.first().copied().unwrap_or(u.player);
                return Some(Box::new(WuzhongMachine {
                    owner: target,
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
            CardKind::Jiedao => {
                if let (Some(&holder), Some(&victim)) = (u.targets.first(), u.targets.get(1)) {
                    return Some(Box::new(JiedaoMachine {
                        owner: u.player,
                        holder,
                        victim,
                        card: None,
                        stage: JiedaoStage::Ask,
                    }));
                }
            }
            kind if kind.aoe_need().is_some() => {
                return Some(Box::new(AoeResolveMachine {
                    card: 0,
                    source: u.player,
                    need: kind.aoe_need().expect("checked"),
                    targets: u.targets,
                    idx: 0,
                    inner: None,
                    nature: kind.aoe_nature(),
                }));
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
        CardKind::Wuzhong => {
            split_jifeng_more_if_multi(ctx, frame_id, &mut u);
            let target = u.targets.first().copied().unwrap_or(u.player);
            Some(Box::new(wrap_nullify_window(
                ctx,
                card,
                u.player,
                Box::new(WuzhongMachine {
                    owner: target,
                    card: Some(card),
                    draw_done: false,
                }),
            )))
        }
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
            // 利木（`xinfu_limu`，L01/L11）：对己使用方块牌作【乐】时先触发回 1 血（onuse）。
            if effective_kind == CardKind::Le
                && u.targets.as_slice() == [u.player]
                && ctx
                    .catalog
                    .skill_by_name("xinfu_limu")
                    .is_some_and(|d| ctx.core.has_skill(u.player, d))
            {
                let _ = ctx.recover(u.player, 1, Some(card));
            }
            let target = *u.targets.first().unwrap_or(&u.player);
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
        CardKind::Jiu | CardKind::Tianxianjiu => {
            // 幻爵等 AfterTriggers 将使用牌转为【酒】并置于处理区时，补完成结算与弃置。
            if ctx.core.table.contains(&card) {
                ctx.core.table.retain(|&x| x != card);
                ctx.to_discard(card);
                ctx.core.turn.jiu_used = true;
                ctx.core.turn.jiu_active = true;
            }
            None
        }
        kind if kind.equip_slot().is_some()
            && u.use_as.is_none()
            && ctx.core.table.contains(&card) =>
        {
            // 幻爵等 AfterTriggers 将使用牌变为装备并置于处理区时，自处理区直装目标装备栏。
            ctx.core.table.retain(|&x| x != card);
            crate::kernel::equipment::install_equip_card(ctx, None, u.player, card);
            None
        }
        _ => {
            if u.use_as.is_some() {
                discard_remaining_carriers(ctx);
            }
            None
        }
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
    let fid = ctx.current_frame_id();
    let jiu_stacks = if u.player == ctx.core.turn.current {
        u32::from(std::mem::take(&mut ctx.core.turn.jiu_active))
            + ctx
                .core
                .vm_aux
                .flags
                .remove(&(u.player, "jiu_extra_stacks".to_string()))
                .unwrap_or(0)
    } else {
        ctx.core
            .vm_aux
            .flags
            .remove(&(u.player, "jiu_active_offturn".to_string()))
            .unwrap_or(0)
    };
    if jiu_stacks > 1 {
        ctx.core.vm_aux.flags.insert(
            (u.player, format!("sha_jiu_extra:{fid}")),
            jiu_stacks - 1,
        );
    }
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
        jiu_bonus: jiu_stacks > 0,
    }))
}
