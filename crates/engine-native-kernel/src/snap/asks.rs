use super::*;
mod judge;
mod shapes;
pub use shapes::*;

// ---------------------------------------------------------------------------
// 开放选择承接派生（N04-e）
// ---------------------------------------------------------------------------

/// 从挂起机器派生当前应有的开放选择询问。
///
/// 返回 `Some` 当且仅当机器处于「已发出询问、等待应答」的静止状态；
/// 派生形状与机器创建该询问时的 `ChoiceSpec` 逐字段一致（选项含顺序）。
/// 恢复期以本函数核对 `open_choice` 的承接关系（N04-e）。
pub fn active_ask(
    machine: &MachineSnapshot,
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    match machine {
        MachineSnapshot::NativeInvocation {
            owner, progress, ..
        } => crate::kernel::NativeInvocation::expected_ask(*owner, progress, core),
        MachineSnapshot::TriggerOrder {
            owner,
            candidates,
            asked,
            ..
        } => {
            if !asked {
                return None;
            }
            Some(ask(
                *owner,
                "trigger_order",
                ChoiceKind::PickTag,
                candidates
                    .iter()
                    .copied()
                    .map(ChoiceOption::Skill)
                    .collect(),
                1,
                1,
                false,
            ))
        }
        // ---- 内核机器 ----
        MachineSnapshot::MonkeySteal { holders, asked, .. } => {
            if !*asked {
                return None;
            }
            let holder = *holders.first()?;
            Some(ask(
                holder,
                "huanle_monkey_confirm",
                ChoiceKind::RespondCard,
                vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                1,
                1,
                false,
            ))
        }
        MachineSnapshot::Sha {
            target,
            source,
            asked,
            tieji_red,
            liuli_stage,
            hujia_holder,
            ..
        } => {
            if *liuli_stage == 1 {
                // 流离弃牌自选（逐技对照批二 v76）：手牌+装备选一张，可取消
                let mut options: Vec<ChoiceOption> = core
                    .player(*target)
                    .hand
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect();
                options.extend(
                    core.player(*target)
                        .equips
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card),
                );
                return Some(ask(
                    *target,
                    "liuli_discard_choose",
                    ChoiceKind::SelectCards,
                    options,
                    0,
                    1,
                    true,
                ));
            }
            if *liuli_stage == 2 {
                // 流离转移目标询问：在（被转移者）攻击范围内、非来源非己、
                // 非空城（kernel liuli_candidate_ok 同口径）
                let candidates: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|p| {
                        *p != *target
                            && *p != *source
                            && {
                                let kc = catalog
                                    .skill_by_name(crate::vocabulary::names::KONG_CHENG)
                                    .map(|d| core.has_skill(*p, d))
                                    .unwrap_or(false);
                                !(kc && core.player(*p).hand.is_empty())
                            }
                            && core.computed_distance(catalog, *target, *p)
                                <= core.attack_range(catalog, *target)
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                return Some(ask(
                    *target,
                    "liuli_transfer",
                    ChoiceKind::SelectPlayers,
                    candidates,
                    1,
                    1,
                    false,
                ));
            }
            if let Some(h) = hujia_holder {
                // v204：持牌者自选一张闪（含倾国转化黑手牌）；空选=拒绝
                let qingguo_h = catalog
                    .skill_by_name(QING_GUO)
                    .map(|d| core.has_skill(*h, d))
                    .unwrap_or(false);
                let options: Vec<ChoiceOption> = core
                    .player(*h)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| {
                        let kind = catalog.card_kind(*c);
                        kind == CardKind::Shan
                            || (qingguo_h
                                && matches!(
                                    catalog.cards[*c as usize].suit,
                                    crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                                ))
                    })
                    .map(ChoiceOption::Card)
                    .collect();
                return Some(ask(
                    *h,
                    "hujia_respond",
                    ChoiceKind::SelectCards,
                    options,
                    0,
                    1,
                    false, // 空选=拒绝本位（非取消）
                ));
            }
            if *asked {
                // 出闪响应：Pass + 目标的闪（龙胆并入杀、倾国并入装备黑牌、
                // 铁骑红色判定封锁常规闪；诱敌深入独立并入一张）
                let tieji_blocks = *tieji_red == Some(true);
                let longdan = catalog
                    .skill_by_name(LONG_DAN)
                    .map(|d| core.has_skill(*target, d))
                    .unwrap_or(false);
                let qingguo = catalog
                    .skill_by_name(QING_GUO)
                    .map(|d| core.has_skill(*target, d))
                    .unwrap_or(false);
                let mut options: Vec<ChoiceOption> = vec![ChoiceOption::Pass];
                for c in core.player(*target).hand.iter().copied() {
                    let kind = catalog.card_kind(c);
                    if !tieji_blocks
                        && (kind == CardKind::Shan || (longdan && kind == CardKind::Sha))
                    {
                        options.push(ChoiceOption::Card(c));
                    }
                }
                for c in core.player(*target).hand.iter().copied() {
                    if catalog.card_kind(c) == CardKind::Youdishenru {
                        options.push(ChoiceOption::Card(c));
                        break;
                    }
                }
                if qingguo && !tieji_blocks {
                    // 倾国（v202 对齐上游 position "hs"）：黑色手牌并入
                    for c in core.player(*target).hand.iter().copied() {
                        let kind = catalog.card_kind(c);
                        let already = kind == CardKind::Shan || (longdan && kind == CardKind::Sha);
                        let black = matches!(
                            catalog.cards[c as usize].suit,
                            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                        );
                        if black && !already {
                            options.push(ChoiceOption::Card(c));
                        }
                    }
                }
                if !tieji_blocks {
                    append_vm_shan_options(core, catalog, *target, &mut options);
                }
                return Some(ask(
                    *target,
                    "respond_shan",
                    ChoiceKind::RespondCard,
                    options,
                    1,
                    1,
                    false,
                ));
            }
            None
        }
        MachineSnapshot::DyingSave {
            responders,
            asked,
            dying,
            ..
        } => {
            if !*asked {
                return None;
            }
            let responder = *responders.first()?;
            let options = dying_save_options(core, catalog, responder, *dying);
            Some(ask(
                responder,
                "dying_save",
                ChoiceKind::RespondCard,
                options,
                1,
                1,
                false,
            ))
        }
        MachineSnapshot::Duel {
            source,
            target,
            on_target,
            asked,
            ..
        } => {
            if !*asked {
                return None;
            }
            let responder = if *on_target { *target } else { *source };
            let options = response_kind_options(core, catalog, responder, CardKind::Sha);
            if options.len() == 1 {
                if let Some((helper, helper_shas)) =
                    crate::kernel::jijiang_shu_helper(core, catalog, responder)
                {
                    let mut opts = vec![ChoiceOption::Pass];
                    opts.extend(helper_shas.into_iter().map(ChoiceOption::Card));
                    return Some(ask(
                        helper,
                        "duel_sha",
                        ChoiceKind::RespondCard,
                        opts,
                        1,
                        1,
                        false,
                    ));
                }
            }
            Some(ask(
                responder,
                "duel_sha",
                ChoiceKind::RespondCard,
                options,
                1,
                1,
                false,
            ))
        }
        MachineSnapshot::NullifyWindow {
            queue,
            asked,
            window_open,
            effect,
            ..
        } => {
            if *window_open {
                if !*asked {
                    return None;
                }
                let responder = *queue.first()?;
                let options = response_kind_options(core, catalog, responder, CardKind::Nullify);
                Some(ask(
                    responder,
                    "nullify_window",
                    ChoiceKind::Confirm,
                    options,
                    1,
                    1,
                    false,
                ))
            } else {
                // 窗口已关闭：委托效果机器
                let e = effect.as_deref()?;
                active_ask(e, core, catalog)
            }
        }
        MachineSnapshot::JudgePhase { owner, current, .. }
        | MachineSnapshot::JudgeContinuation { owner, current, .. } => {
            judge::active_judge_ask(owner, current, core, catalog)
        }
        MachineSnapshot::PlaceDelayed { .. } => None,
        MachineSnapshot::Aoe { inner, .. } => {
            let e = inner.as_deref()?;
            active_ask(e, core, catalog)
        }
        MachineSnapshot::AoeTarget {
            target,
            need,
            asked,
            ..
        } => {
            if !*asked {
                return None;
            }
            let options = response_kind_options(core, catalog, *target, *need);
            Some(ask(
                *target,
                "aoe_response",
                ChoiceKind::RespondCard,
                options,
                1,
                1,
                false,
            ))
        }
        // 铁索逐目标无懈窗口（规则 v290）：当前目标窗口内的询问由窗口机器回答；
        // 派发层自身不产生询问。
        MachineSnapshot::Chain { inner, .. } => {
            let e = inner.as_deref()?;
            active_ask(e, core, catalog)
        }
        MachineSnapshot::DiscardToLimit {
            player,
            need,
            asked,
        } => {
            if !*asked {
                return None;
            }
            let options = hand_card_options(&core.player(*player).hand);
            Some(ask(
                *player,
                "discard_to_limit",
                ChoiceKind::SelectCards,
                options,
                *need,
                *need,
                false,
            ))
        }
        MachineSnapshot::GangLie {
            source,
            asked,
            ..
        } if *asked => {
            let options = hand_card_options(&core.player(*source).hand);
            Some(ask(
                *source,
                "discard_to_limit",
                ChoiceKind::SelectCards,
                options,
                2,
                2,
                true,
            ))
        }
        MachineSnapshot::Death { .. } => None,
        MachineSnapshot::OptionalGate {
            owner,
            def,
            event,
            asked,
            inner,
        } => {
            if let Some(e) = inner {
                let mut ask = active_ask(e, core, catalog)?;
                ask.source_skill = optional_source(e, catalog);
                return Some(ask);
            }
            if !*asked {
                return None;
            }
            let mut ask = confirm_ask(
                catalog.skills[*def as usize].behavior.confirmation_actor(*owner, event, event.kind()),
                "trigger_confirm",
            );
            ask.source_skill = optional_source(machine, catalog);
            Some(ask)
        }
        // ---- 内容机器 ----
        MachineSnapshot::SanCai { owner, stage, .. } => match stage {
            SanCaiStageSnap::GiveCard => Some(ask(
                *owner,
                "sancai_target",
                ChoiceKind::SelectPlayers,
                other_player_options(core, *owner),
                1,
                1,
                false,
            )),
            SanCaiStageSnap::Finish => Some(ask(
                *owner,
                "sancai_card",
                ChoiceKind::SelectCards,
                hand_card_options(&core.player(*owner).hand),
                1,
                1,
                false,
            )),
            _ => None,
        },
        MachineSnapshot::YiShang { owner, stage } => match stage {
            YiShangStageSnap::Apply => Some(ask(
                *owner,
                "yishang_target",
                ChoiceKind::SelectPlayers,
                other_player_options(core, *owner),
                1,
                1,
                false,
            )),
            _ => None,
        },
        MachineSnapshot::TuXi { owner, stage } => match stage {
            TuXiStageSnap::Read => {
                let candidates: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|p| *p != *owner && !core.player(*p).hand.is_empty())
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(
                    *owner,
                    "tuxi_target",
                    ChoiceKind::SelectPlayers,
                    candidates,
                    1,
                    2,
                    true,
                ))
            }
            _ => None,
        },
        MachineSnapshot::Rende { owner, stage, .. } => match stage {
            RendeStageSnap::TargetAsk => {
                let hand = &core.player(*owner).hand;
                let max = hand.len();
                Some(ask(
                    *owner,
                    "rende_cards",
                    ChoiceKind::SelectCards,
                    hand_card_options(hand),
                    0,
                    max,
                    true,
                ))
            }
            RendeStageSnap::TargetRead => Some(ask(
                *owner,
                "rende_target",
                ChoiceKind::SelectPlayers,
                other_player_options(core, *owner),
                1,
                1,
                true,
            )),
            _ => None,
        },
        MachineSnapshot::Jijiang {
            owner,
            stage,
            helper,
            sha,
            inner,
            ..
        } => {
            if let Some(e) = inner {
                return active_ask(e, core, catalog);
            }
            match stage {
                JijiangStageSnap::HelperAsk => {
                    let h = (*helper)?;
                    Some(ask(
                        h,
                        "jijiang_respond",
                        ChoiceKind::Confirm,
                        vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                        1,
                        1,
                        false,
                    ))
                }
                JijiangStageSnap::TargetAsk => {
                    // 目标询问仅在借杀成功后存在
                    if helper.is_none() || sha.is_none() {
                        return None;
                    }
                    let candidates: Vec<ChoiceOption> = core
                        .alive_players()
                        .filter(|p| {
                            *p != *owner
                                && core.computed_distance(catalog, *owner, *p)
                                    <= core.attack_range(catalog, *owner)
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    Some(ask(
                        *owner,
                        "jijiang_target",
                        ChoiceKind::SelectPlayers,
                        candidates,
                        1,
                        1,
                        true,
                    ))
                }
            }
        }
        MachineSnapshot::JieYin { owner, stage, .. } => match stage {
            JieYinStageSnap::TargetAsk => {
                let hand = &core.player(*owner).hand;
                Some(ask(
                    *owner,
                    "jieyin_cards",
                    ChoiceKind::SelectCards,
                    hand_card_options(hand),
                    2,
                    2,
                    true,
                ))
            }
            JieYinStageSnap::TargetRead => {
                let candidates: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|t| {
                        *t != *owner
                            && !core.player(*t).equips.is_empty()
                            && seat_is_male_core(core, *t)
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(
                    *owner,
                    "jieyin_target",
                    ChoiceKind::SelectPlayers,
                    candidates,
                    1,
                    1,
                    true,
                ))
            }
            _ => None,
        },
        MachineSnapshot::BossXiangde { .. }
        | MachineSnapshot::BossFutai { .. }
        | MachineSnapshot::BossLuanchang { .. }
        | MachineSnapshot::BossZhaohuo { .. }
        | MachineSnapshot::BossQizuo { .. }
        | MachineSnapshot::BossXiaoxiong { .. }
        | MachineSnapshot::BossLingbo { .. }
        | MachineSnapshot::BossTashui { .. }
        | MachineSnapshot::BossZjLeiji { .. }
        | MachineSnapshot::BossHeisheng { .. }
        | MachineSnapshot::BossLeizhu { .. }
        | MachineSnapshot::BossRenao { .. }
        | MachineSnapshot::BossWanghun { .. }
        | MachineSnapshot::BossZhuxin { .. }
        | MachineSnapshot::BossPingdeng { .. }
        | MachineSnapshot::BossHuoxing { .. }
        | MachineSnapshot::BossBingfeng { .. }
        | MachineSnapshot::BossWangxiang { .. } => None,
        MachineSnapshot::Rvm {
            commit_pending,
            commit_state,
            ..
        } if !commit_state.is_empty() => {
            let effects = crate::rvm_ops::dec_effects(commit_pending)?;
            (catalog.hooks.vm_ask)(effects.first()?, commit_state, core, catalog)
        }
        MachineSnapshot::Rvm {
            owner: _,
            commit_pick: Some((player, count)),
            ..
        } => {
            // J1（规则 v328）：提交期挂起——开窗座位的手牌+装备（手牌在前），
            // 恰好 min(count, 可用) 张；无牌空过由恢复后首步处理（不派询问）。
            let hand_equips: Vec<ChoiceOption> = core
                .player(*player)
                .hand
                .iter()
                .chain(core.player(*player).equips.iter())
                .map(|c| ChoiceOption::Card(*c))
                .collect();
            let exact = (*count as usize).min(hand_equips.len());
            if exact == 0 {
                None
            } else {
                Some(ask(
                    *player,
                    "rvm_discard_pick",
                    ChoiceKind::SelectCards,
                    hand_equips,
                    exact,
                    exact,
                    false,
                ))
            }
        }
        MachineSnapshot::Rvm {
            owner,
            paused: Some(p),
            ..
        } => {
            // T8 增量 2：VM 选择挂起——选项 tokens 派生（PickTag）
            let options: Vec<ChoiceOption> = p
                .options
                .iter()
                .map(|t| ChoiceOption::Tag(*t as u8))
                .collect();
            Some(ask(
                *owner,
                "rvm_choice",
                ChoiceKind::PickTag,
                options,
                1,
                1,
                false,
            ))
        }
        MachineSnapshot::Rvm { .. } => {
            // 非挂起态：无询问
            None
        }
        MachineSnapshot::BossBuchun { owner, stage, .. } if *stage == 3 => {
            // 不臣恢复线目标询问挂起：同阵营受伤角色逐字段派生
            let seats = core.players.len() as u8;
            let candidates: Vec<ChoiceOption> = (0..seats)
                .filter(|&t| {
                    let p = core.player(t);
                    p.alive
                        && p.hp < p.max_hp as i32
                        && core.is_boss_side(t) == core.is_boss_side(*owner)
                })
                .map(ChoiceOption::Player)
                .collect();
            Some(ask(
                *owner,
                "buchun_target",
                ChoiceKind::SelectPlayers,
                candidates,
                1,
                1,
                false,
            ))
        }
        MachineSnapshot::BossBuchun { .. } => {
            // 复活线/恢复线非挂起阶段：无询问
            None
        }
        MachineSnapshot::BossXingxia { owner, stage, .. } if *stage == 1 => {
            // 星夏敌方询问挂起：异侧存活角色派生
            let seats = core.players.len() as u8;
            let candidates: Vec<ChoiceOption> = (0..seats)
                .filter(|&t| {
                    core.player(t).alive && core.is_boss_side(t) != core.is_boss_side(*owner)
                })
                .map(ChoiceOption::Player)
                .collect();
            Some(ask(
                *owner,
                "xingxia_enemy",
                ChoiceKind::SelectPlayers,
                candidates,
                0,
                1,
                true,
            ))
        }
        MachineSnapshot::BossXingxia { actor, stage, .. } if *stage == 3 => {
            // 星夏弃红询问挂起：焦点敌方的手牌+装备红色牌派生
            let t = *actor as u8;
            let mut candidates: Vec<ChoiceOption> = core
                .player(t)
                .hand
                .iter()
                .copied()
                .filter(|c| {
                    matches!(
                        catalog.cards[*c as usize].suit,
                        crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                    )
                })
                .map(ChoiceOption::Card)
                .collect();
            for c in core.player(t).equips.iter().copied() {
                if matches!(
                    catalog.cards[c as usize].suit,
                    crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                ) {
                    candidates.push(ChoiceOption::Card(c));
                }
            }
            Some(ask(
                t,
                "xingxia_discard",
                ChoiceKind::SelectCards,
                candidates,
                0,
                1,
                true,
            ))
        }
        MachineSnapshot::BossXingxia { .. } => {
            // 非挂起阶段：无询问
            None
        }
        MachineSnapshot::Shedu { owner, .. } => {
            // 蛇毒：挂起态询问重挂（弃 X 或失 X）
            Some(ask(
                *owner,
                "shedu_discard_or_lose",
                ChoiceKind::SelectCards,
                hand_card_options(&core.player(*owner).hand),
                0,
                core.player(*owner).shedu as usize,
                true,
            ))
        }
        MachineSnapshot::FanJian {
            owner,
            stage,
            target,
            ..
        } => match stage {
            // stage 名 = 消费者：SuitAsk 消费 fanjian_target 应答、
            // PickAsk 消费 fanjian_suit、Consume 消费 fanjian_pick（v203）
            FanJianStageSnap::SuitAsk => Some(ask(
                *owner,
                "fanjian_target",
                ChoiceKind::SelectPlayers,
                other_player_options(core, *owner),
                1,
                1,
                true,
            )),
            FanJianStageSnap::PickAsk => {
                let t = (*target)?;
                Some(ask(
                    t,
                    "fanjian_suit",
                    ChoiceKind::PickTag,
                    (0..=3).map(crate::choice::ChoiceOption::Tag).collect(),
                    1,
                    1,
                    false,
                ))
            }
            FanJianStageSnap::Consume => {
                let t = (*target)?;
                Some(ask(
                    t,
                    "fanjian_pick",
                    ChoiceKind::SelectCards,
                    hand_card_options(&core.player(*owner).hand),
                    1,
                    1,
                    false,
                ))
            }
            FanJianStageSnap::TargetAsk => None,
        },
        MachineSnapshot::LiJian {
            owner,
            stage,
            inner,
            ..
        } => {
            if let Some(e) = inner {
                return active_ask(e, core, catalog);
            }
            match stage {
                LiJianStageSnap::TargetsAsk => {
                    let hand = &core.player(*owner).hand;
                    Some(ask(
                        *owner,
                        "lijian_discard",
                        ChoiceKind::SelectCards,
                        hand_card_options(hand),
                        1,
                        1,
                        true,
                    ))
                }
                LiJianStageSnap::Launch => {
                    let candidates: Vec<ChoiceOption> = core
                        .alive_players()
                        .filter(|p| *p != *owner && seat_is_male_core(core, *p))
                        .map(ChoiceOption::Player)
                        .collect();
                    Some(ask(
                        *owner,
                        "lijian_targets",
                        ChoiceKind::SelectPlayers,
                        candidates,
                        2,
                        2,
                        true,
                    ))
                }
                _ => None,
            }
        }
        MachineSnapshot::XunXun {
            owner,
            stage: XunXunStageSnap::Arrange,
            top,
        } => Some(ask(
            *owner,
            "xunxun_arrange",
            ChoiceKind::SelectCards,
            hand_card_options(top),
            2,
            2,
            false,
        )),
        MachineSnapshot::JiJie {
            owner,
            stage: JiJieStageSnap::Read,
            ..
        } => Some(ask(
            *owner,
            "jijie_give_target",
            ChoiceKind::SelectPlayers,
            core.alive_players().map(ChoiceOption::Player).collect(),
            1,
            1,
            false,
        )),
        MachineSnapshot::GuanXing { owner, stage, top } => match stage {
            GuanXingStageSnap::Place => {
                let take = top.len();
                Some(ask(
                    *owner,
                    "guanxing_arrange",
                    ChoiceKind::SelectCards,
                    hand_card_options(top),
                    0,
                    take,
                    false,
                ))
            }
            _ => None,
        },
        MachineSnapshot::QingNang { owner, stage, .. } => match stage {
            QingNangStageSnap::TargetRead => {
                let wounded: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|p| {
                        *p != *owner && {
                            let pl = core.player(*p);
                            pl.alive && (pl.hp as i64) < (pl.max_hp as i64)
                        }
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(
                    *owner,
                    "qingnang_target",
                    ChoiceKind::SelectPlayers,
                    wounded,
                    1,
                    1,
                    true,
                ))
            }
            QingNangStageSnap::CardRead => Some(ask(
                *owner,
                "qingnang_card",
                ChoiceKind::SelectCards,
                hand_card_options(&core.player(*owner).hand),
                1,
                1,
                true,
            )),
            _ => None,
        },
        MachineSnapshot::JiJi { owner, stage, .. } => match stage {
            JiJiStageSnap::GiveCard => Some(ask(
                *owner,
                "jiji_target",
                ChoiceKind::SelectPlayers,
                other_player_options(core, *owner),
                0,
                1,
                true,
            )),
            JiJiStageSnap::GiveChooseCard => Some(ask(
                *owner,
                "jiji_card",
                ChoiceKind::SelectCards,
                hand_card_options(&core.player(*owner).hand),
                1,
                1,
                true,
            )),
            _ => None,
        },
        MachineSnapshot::ZhiHeng { owner, stage } => match stage {
            ZhiHengStageSnap::Draw => {
                let hand = &core.player(*owner).hand;
                let max = hand.len();
                Some(ask(
                    *owner,
                    "zhiheng_select",
                    ChoiceKind::SelectCards,
                    hand_card_options(hand),
                    0,
                    max,
                    true,
                ))
            }
            _ => None,
        },
        // ---- G1 规则宿主机器 ----
        MachineSnapshot::Host {
            behavior,
            stage,
            owner,
            target,
            targets,
            card,
            cards,
            ..
        } => (catalog.hooks.host_ask)(
            u16::from(*behavior),
            *stage,
            *owner,
            *target,
            targets,
            *card,
            cards,
            core,
            catalog,
        ),
        // ---- 无询问机器 ----
        _ => None,
    }
}

// Only OptionalGate::step has the direct delegation contract used by this source.
fn optional_source(machine: &MachineSnapshot, catalog: &Catalog) -> Option<&'static str> {
    let MachineSnapshot::OptionalGate {
        def, asked, inner, ..
    } = machine
    else {
        return None;
    };
    if let Some(inner) = inner {
        return optional_source(inner, catalog);
    }
    if !asked {
        return None;
    }
    let declared = catalog.skills.get(*def as usize)?;
    if declared.name.is_empty()
        || declared.behavior.is_vm_skill()
        || !declared.trigger.as_ref().is_some_and(|meta| meta.optional)
        || catalog
            .skills
            .iter()
            .filter(|entry| entry.name == declared.name)
            .count()
            != 1
    {
        return None;
    }
    Some(declared.name)
}
