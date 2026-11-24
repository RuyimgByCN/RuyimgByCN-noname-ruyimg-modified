use super::*;

/// 帧固有流程机器（`Intrinsic` 阶段启动）。
pub fn intrinsic_machine(
    ctx: &mut Ctx<'_>,
    frame_id: EventId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    let data = ctx.res.frame(frame_id).expect("frame").data.clone();
    match data {
        EventData::OrdinaryJudge { owner, invocation } => {
            if ctx
                .core
                .vm_aux
                .flags
                .remove(&(owner, format!("l11_resolved_ordinary_judge:{invocation}")))
                .is_some()
            {
                return None;
            }
            let is_host_judge = invocation == u32::MAX;
            if !is_host_judge
                && (invocation == 0
                    || ctx.res.frame(frame_id).and_then(|f| f.parent) != Some(invocation)
                    || !ctx
                        .res
                        .frame(invocation)
                        .is_some_and(|f| matches!(f.data, EventData::SkillInvocation(_))))
            {
                return None;
            }
            let mut m = JudgePhaseMachine::new(owner, Vec::new(), None);
            m.ordinary = Some(if is_host_judge { frame_id } else { invocation });
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
        EventData::CardUsed(u) => super::intrinsic_card_use::card_use_intrinsic(ctx, frame_id, u),
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
                "yuling1",
                "yuling",
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
                        ctx.record_phase_skipped(player);
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
                    // 机甲（U05 ow D.Va，规则 v141）：WudianJijia 保持历史 +4；U05OwBatchC 已在 core.hand_limit 累加 jijia_hp
                    if ctx
                        .catalog
                        .skill_by_name(crate::vocabulary::host::WUDIAN_JIJIA_KEY)
                        .map(|def| {
                            ctx.catalog.skills[def as usize].behavior.as_host_behavior()
                                != Some(crate::vocabulary::host::HostBehavior::U05OwBatchC)
                                && ctx.core.has_skill(player, def)
                        })
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
                    let has_chunbing = [
                        crate::vocabulary::host::GJ_FOOD_CHUNBING_KEY,
                        "chunbing",
                    ]
                    .into_iter()
                    .any(|k| {
                        ctx.catalog.skill_by_name(k).is_some_and(|def| {
                            ctx.core
                                .skills
                                .get(&(player, def))
                                .is_some_and(|i| i.enabled)
                        })
                    });
                    if has_chunbing {
                        limit += 1;
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
                    // qiankundai mod maxHandcard 同口径，含技能门控）。
                    let qkd_equip = ctx
                        .core
                        .player(player)
                        .equips
                        .iter()
                        .any(|c| ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Qiankundai);
                    let qkd_skill = ctx
                        .catalog
                        .skill_by_name("qiankundai")
                        .is_some_and(|d| ctx.core.has_skill(player, d));
                    if qkd_equip || qkd_skill {
                        limit += 1;
                    }
                    // 倾盆大雨（gwent 天气）：手牌上限 -1
                    let gw_qinpen = [
                        crate::vocabulary::host::GW_QINPEN_STATE_KEY,
                        "gw_qinpendayu",
                    ]
                    .into_iter()
                    .any(|k| {
                        ctx.catalog.skill_by_name(k).is_some_and(|def| {
                            ctx.core
                                .skills
                                .get(&(player, def))
                                .is_some_and(|i| i.enabled)
                        })
                    });
                    if gw_qinpen {
                        limit = limit.saturating_sub(1);
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
                    } else if crate::rvm_bridge::modifiers::d34_flags::has_ddddanggu_negative(
                        ctx.core,
                        ctx.catalog,
                        player,
                    ) {
                        let p = ctx.core.player(player);
                        (p.hand.len() + p.equips.len())
                            .saturating_sub(ctx.core.ignored_hand_card_count(ctx.catalog, player))
                    } else {
                        ctx.core
                            .player(player)
                            .hand
                            .len()
                            .saturating_sub(ctx.core.ignored_hand_card_count(ctx.catalog, player))
                    };
                    if hand > limit {
                        let discardable = ctx
                            .core
                            .player(player)
                            .hand
                            .iter()
                            .filter(|c| {
                                !crate::kernel::combat_rules::is_gw_dudayuanshuai2_locked(
                                    ctx.core,
                                    ctx.catalog,
                                    player,
                                    **c,
                                )
                            })
                            .count();
                        let need = (hand - limit).min(discardable);
                        if need > 0 {
                            Some(Box::new(DiscardToLimitMachine {
                                player,
                                need,
                                asked: false,
                            }))
                        } else {
                            None
                        }
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
