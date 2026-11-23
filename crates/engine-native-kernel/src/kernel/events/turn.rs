//! 回合/阶段帧族的提交与交接（自 events.rs 按帧族拆出，R05-F2 行数门
//! 禁）：TurnBegin/TurnEndWindow/TurnEnded 的 Commit 步、回合交接
//! turn_handoff 与新回合重置 reset_new_turn_state。行为与可见性零变化。
use super::super::*;

/// 回合开始窗（R05，规则 333）的 Commit 步：压入准备阶段的结束与开始帧。
///
/// 帧栈按后进先出结算，因此先压「准备阶段结束」再压「准备阶段开始」——
/// 开始窗（含其 BeforeTriggers 触发与入场）先完整结算，随后结束帧把
/// `turn.phase` 推进到判定阶段并进入 Judge 入场；整个回合开始流程对
/// 调用方是单次 emit 的子树，不额外消耗一次 NextPhase 命令。
/// R05（K21）：整回合结束窗 phaseEnd 段的 Commit——记日志并压 after 段。
pub(in crate::kernel) fn commit_turn_end_window(ctx: &mut Ctx<'_>, player: PlayerId, after: bool) {
    if after {
        ctx.log(LogEvent::TurnPhaseAfter { player });
        // R05-F（§6 全局 phaseAfter 到期，收其他 trigger 前）：清全部角色的
        // Limit/Current 语义槽与 helper 实例（含 disabled）；Next/Combs 保留；
        // 边界中新授保留至下一边界，不二扫。
        for role in ["limit", "current"] {
            for def in (ctx.catalog.hooks.r05_role_defs)(ctx.catalog, role) {
                let seats = ctx.core.players.len() as u8;
                for seat in 0..seats {
                    ctx.core.vm_aux.marks.remove(&(seat, def as u32));
                    ctx.core.skills.remove(&(seat, def));
                }
            }
        }
        // K50：封冻解除（Boss 应昭）——回合结束全清封印。
        for p in ctx.core.players.iter_mut() {
            p.fengyin_active = false;
        }
        // 仙剑 debuff 封印（qiongguang/tianwu `fengyin`）：仅至被封印者本人 phaseAfter 解除。
        ctx.core
            .vm_aux
            .flags
            .remove(&(player, "xj_fengyin_sealed".to_string()));
        let xj_disabled: Vec<u16> = ctx
            .core
            .vm_aux
            .flags
            .keys()
            .filter_map(|(seat, k)| {
                (*seat == player)
                    .then(|| k.strip_prefix("xj_fengyin_disabled:"))
                    .flatten()
                    .and_then(|d| d.parse::<u16>().ok())
            })
            .collect();
        for def in xj_disabled {
            ctx.core
                .vm_aux
                .flags
                .remove(&(player, format!("xj_fengyin_disabled:{def}")));
            if !ctx
                .core
                .vm_aux
                .flags
                .contains_key(&(player, format!("mbcmfangzhu_disabled:{def}")))
            {
                if let Some(inst) = ctx.core.skills.get_mut(&(player, def)) {
                    inst.enabled = true;
                }
            }
        }
        for seat in 0..ctx.core.players.len() as PlayerId {
            ctx.core
                .vm_aux
                .flags
                .remove(&(seat, "fz_new_tieji_target_sealed".to_string()));
            ctx.core
                .vm_aux
                .flags
                .remove(&(seat, "sbtieji_target_sealed".to_string()));
            ctx.core
                .vm_aux
                .flags
                .remove(&(seat, "tspowei_in_range".to_string()));
            ctx.core
                .vm_aux
                .flags
                .remove(&(seat, "rekuanshi_turn_dmg".to_string()));
            ctx.core
                .vm_aux
                .flags
                .remove(&(seat, "u05_dujian2_turn".to_string()));
            ctx.core
                .vm_aux
                .flags
                .remove(&(seat, "l11_noda_axe2_turn".to_string()));
            ctx.core
                .vm_aux
                .flags
                .remove(&(seat, "l11_phase_sha_count".to_string()));
            ctx.core
                .vm_aux
                .flags
                .remove(&(seat, "l11_turn_used_suits_mask".to_string()));
            ctx.core
                .vm_aux
                .flags
                .remove(&(seat, "l11_turn_used_ranks_mask".to_string()));
            ctx.core
                .vm_aux
                .flags
                .remove(&(seat, "l11_turn_used_types_mask".to_string()));
            ctx.core
                .vm_aux
                .flags
                .remove(&(seat, "l11_jiuwei1_holder".to_string()));
            ctx.core
                .vm_aux
                .flags
                .retain(|(s, k), _| !(*s == seat && k.starts_with("l11_jiuwei2_ts:")));
            for tmp_sk in [
                "txhj_noda_axe2",
                "txhj_daiyuanA",
                "txhj_daiyuanB",
                "txshixin_red",
                "txhj_jiuweiSkill1_disable",
            ] {
                if let Some(def) = ctx.catalog.skill_by_name(tmp_sk) {
                    ctx.core.skills.remove(&(seat, def));
                }
            }
            ctx.core
                .vm_aux
                .flags
                .remove(&(seat, "dddlianer_rank_cap".to_string()));
            ctx.core
                .vm_aux
                .flags
                .remove(&(seat, "dddyulv_turn_cnt".to_string()));
            ctx.core
                .vm_aux
                .flags
                .remove(&(seat, "dddxuanlun_choose".to_string()));
            ctx.core
                .vm_aux
                .flags
                .remove(&(seat, "hs_shihuawuqi_used_turn".to_string()));
            ctx.core
                .vm_aux
                .flags
                .remove(&(seat, "u05_tianxianjiu_used_turn".to_string()));
            ctx.core
                .vm_aux
                .flags
                .remove(&(seat, "u05_tianxianjiu_active_fallback".to_string()));
            let had_after = ctx
                .core
                .vm_aux
                .flags
                .remove(&(seat, "u05_yinmi_until_phase_after".to_string()))
                .is_some();
            if had_after
                && ctx
                    .core
                    .vm_aux
                    .flags
                    .get(&(seat, "u05_yinmi_until_phase_begin".to_string()))
                    .copied()
                    .unwrap_or(0)
                    == 0
            {
                ctx.core.player_mut(seat).yinmi_active = false;
            }
        }
        if let Some(tdef) = ctx.catalog.skill_by_name("tianxianjiu") {
            let seats = ctx.core.players.len() as u8;
            for seat in 0..seats {
                ctx.core.skills.remove(&(seat, tdef));
            }
        }
        ctx.core
            .vm_aux
            .flags
            .remove(&(player, "hs_jinzhou_sealed".to_string()));
        if ctx
            .core
            .vm_aux
            .flags
            .remove(&(player, "hs_xuetu_clear_phase_after".to_string()))
            .is_some()
        {
            ctx.core
                .vm_aux
                .flags
                .remove(&(player, "hs_fengyin_xuetu".to_string()));
            if let Some(def) = ctx.catalog.skill_by_name("zhoujiang") {
                ctx.core.skills.remove(&(player, def));
            }
        }
        ctx.core.vm_aux.flags.retain(|(_, k), _| {
            !k.starts_with("u01_zhuanshe_used:")
                && !k.starts_with("u01_cj_len:")
                && !k.starts_with("u01_cj_card:")
                && !k.starts_with("dddzhuanshe_given:")
                && k != "u01_turn_trick_used_stamp"
        });
        // K52：往事封禁清零（十殿阎罗孟婆）。
        for p in ctx.core.players.iter_mut() {
            p.wangshi_ban = false;
        }
        // K53：巨源记体力（Boss 朱厌；上游 phaseAfter storage=hp）。
        if let Some(jdef) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_JUYUAN)
        {
            if ctx.core.has_skill(player, jdef) {
                let hp = ctx.core.player(player).hp;
                ctx.core.player_mut(player).juyuan_storage = hp;
            }
        }
        // K18：灵武（swd 妮可）——整回合结束后若本回合已用牌数 ≥ 体力且
        // 本回合非灵武插入的额外阶段，插入一个额外出牌阶段（上游
        // insertPhase；cards_used_turn 尚未被新回合重置，reset 在 handoff；
        // 额外出牌阶段 vs 上游额外回合＝既有登记差异保留）。
        if !ctx.core.turn.extra_phase_lingwu {
            if let Some(def) = ctx
                .catalog
                .skill_by_name(crate::vocabulary::host::SWD_LINGWU_KEY)
            {
                let hp = ctx.core.player(player).hp.max(0) as u32;
                if ctx.core.has_skill(player, def)
                    && ctx.core.player(player).alive
                    && ctx.core.turn.cards_used_turn >= hp
                {
                    ctx.core.turn.extra_phases.push(2); // Phase::Play
                    ctx.core.turn.extra_phase_lingwu = true;
                }
            }
        } else {
            ctx.core.turn.extra_phase_lingwu = false;
        }
        // 蛊舞递减（A2 审计 K01；上游 anwugu2 player:'phaseAfter'）：
        // 本人整回合结束后 -1，归 0 移除（出牌受限/手牌上限随之解除）。
        if ctx.core.player(player).anwugu_count > 0 {
            ctx.core.player_mut(player).anwugu_count -= 1;
        }
        // U05-5b `gwqinwu`: `target.addTempSkill(["gwqinwu", "gwqinwu2"], { player: "phaseAfter" })`
        if ctx
            .core
            .vm_aux
            .flags
            .remove(&(player, "gw_qinwu_temp".to_string()))
            .is_some()
        {
            for sk in ["gwqinwu", "gwqinwu2"] {
                if let Some(def) = ctx.catalog.skill_by_name(sk) {
                    ctx.core.skills.remove(&(player, def));
                }
            }
        }
        // U05-5b `gwgouhun`: `goMad({ player: "phaseAfter" })`
        if ctx
            .core
            .vm_aux
            .flags
            .remove(&(player, "gw_gouhun_mad".to_string()))
            .is_some()
        {
            ctx.core.player_mut(player).boss_mad = false;
        }
        // U05-5b `g_gw_yewu` (`character/gwent/skill.js:1124`): phaseAfter 移去 gw_wuyao / gw_lang
        let yewu_vanish: Vec<crate::ids::CardId> = ctx
            .core
            .player(player)
            .hand
            .iter()
            .copied()
            .filter(|&c| {
                matches!(
                    ctx.catalog.cards[c as usize].name,
                    "gw_wuyao" | "gw_lang"
                )
            })
            .collect();
        if !yewu_vanish.is_empty() {
            for &c in &yewu_vanish {
                ctx.take_hand_card(player, c);
            }
            ctx.note_cards_lost(player, &yewu_vanish);
            let lost_n = yewu_vanish.len();
            ctx.core
                .vm_aux
                .piles
                .entry((0, "u05_vanished".to_string()))
                .or_default()
                .extend(yewu_vanish.iter().copied());
            ctx.emit(EventData::CardsLost {
                player,
                cards: yewu_vanish,
                origins: vec![crate::event::LostOrigin::Hand; lost_n],
            });
        }
        if ctx.core.turn.extra_phases.is_empty()
            && ctx.core.player(player).alive
            && (ctx.catalog.hooks.on_shanxian_turnover)(ctx, player)
        {
            return;
        }
        // phaseAfter 段 Commit＝回合交接（K25：回合重置已在 handoff 内、
        // 先于 TurnBegin）。boss 重整 tick 在窗前已由 commit_turn_ended
        // 处理（与存活者选取耦合），此处不再重复。
        turn_handoff(ctx, player, ctx.core.next_alive(player));
    } else {
        ctx.log(LogEvent::TurnPhaseEnded { player });
        // K17：护天寄存取回——寄存主面（H54）在本窗 BeforeTriggers 刚授
        // 予（just_given=true），须跨一整轮：本窗不取回，仅清 just_given；
        // 下一整回合结束窗取回（上游 hutian4 临时技下一阶段边界语义按
        // 审计对齐为整回合结束）。
        if ctx.core.player(player).hutian_just_given {
            ctx.core.player_mut(player).hutian_just_given = false;
        } else if ctx.core.player(player).hutian_deposit.is_some() {
            hutian_return_deposit(ctx, player);
        }
        // 星体改造重置（K05；上游 phaseEnd 移除+清零）。
        if ctx.core.player(player).xingtigaizao_count > 0 {
            ctx.core.player_mut(player).xingtigaizao_count = 0;
        }
        // 光塔（K13）：♠子技持有者受 1 无源雷伤并移除（一次性）。
        if let Some(def) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::host::WUDIAN_GUANGSHU_SPADE_KEY)
        {
            if ctx.core.skills.remove(&(player, def)).is_some() {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    None,
                    player,
                    1,
                    crate::event::DamageNature::Thunder,
                    None,
                )));
            }
        }
        // 筑城确认门（K14）：本人整回合结束确认翻转（上游 phaseEnd
        // optional）——直推 OptionalGate，同款公园机制。
        if let Some(def) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::host::WUDIAN_ZHUCHENG_KEY)
        {
            if ctx.core.has_skill(player, def) {
                let event = EventData::TurnEndWindow {
                    player,
                    after: false,
                };
                ctx.res.parks.push(crate::resolution::Park {
                    resume_depth: ctx.res.stack.len(),
                    slot: crate::resolution::Slot::Trigger {
                        frame: crate::resolution::NO_FRAME,
                    },
                    machine: Box::new(crate::resolution::OptionalGate::new(
                        player, def, event, false, None,
                    )),
                    awaiting_answer: false,
                });
            }
        }
        // 暗织覆盖 phaseEnd，须在 phaseAfter 的 BeforeTriggers 收集前到期。
        for seat in 0..ctx.core.players.len() as PlayerId {
            ctx.core
                .vm_aux
                .flags
                .remove(&(seat, "dddanzhi_immune".to_string()));
            ctx.core
                .vm_aux
                .flags
                .remove(&(seat, "dddanzhi_stamp".to_string()));
        }
        (ctx.catalog.hooks.on_turn_phase_after)(ctx, player);
        // U05-3 ow 侵入封印：持续至目标自身回合结束 ({ player: "phaseAfter" })
        ctx.core
            .vm_aux
            .flags
            .remove(&(player, "ow_qinru_fengyin".to_string()));
        ctx.emit(EventData::TurnEndWindow {
            player,
            after: true,
        });
    }
}

pub(in crate::kernel) fn commit_turn_begin(ctx: &mut Ctx<'_>, player: PlayerId) {
    let fid = ctx.current_frame_id();
    if let Some(owner_plus1) = ctx
        .core
        .vm_aux
        .flags
        .remove(&(0, format!("ow_shanxian_hijack_frame:{fid}")))
    {
        if let Some(owner) = owner_plus1.checked_sub(1).map(|v| v as PlayerId) {
            ctx.core.turn.current = owner;
            ctx.core.turn.turn_opened = false;
            reset_new_turn_state(ctx, owner);
            ctx.emit(EventData::TurnBegin { player: owner });
            return;
        }
    }
    if ctx.catalog.skill_by_name("ziyu").is_some_and(|d| {
        ctx.catalog.skills[d as usize].behavior.as_host_behavior()
            == Some(crate::vocabulary::host::HostBehavior::U05OwBatchD)
    }) {
        let key = (0, "ow_phase_number".to_string());
        let cur = ctx.core.vm_aux.flags.get(&key).copied().unwrap_or(0);
        ctx.core.vm_aux.flags.insert(key, cur + 1);
        for s in 0..ctx.core.players.len() as PlayerId {
            if !ctx.core.player(s).alive {
                continue;
            }
            if ctx.catalog.skill_by_name("jijia").is_some_and(|d| {
                ctx.catalog.skills[d as usize].behavior.as_host_behavior()
                    == Some(crate::vocabulary::host::HostBehavior::U05OwBatchC)
                    && ctx.core.has_skill(s, d)
            }) && ctx
                .core
                .vm_aux
                .flags
                .get(&(s, "ow_jijia_inited".to_string()))
                .copied()
                .unwrap_or(0)
                == 0
            {
                ctx.core
                    .vm_aux
                    .flags
                    .insert((s, "ow_jijia_inited".to_string()), 1);
                ctx.core.player_mut(s).jijia_hp = 4;
            }
            if ctx.catalog.skill_by_name("zhongdun").is_some_and(|d| {
                ctx.catalog.skills[d as usize].behavior.as_host_behavior()
                    == Some(crate::vocabulary::host::HostBehavior::U05OwBatchB)
                    && ctx.core.has_skill(s, d)
            }) && ctx
                .core
                .vm_aux
                .flags
                .get(&(s, "ow_zhongdun_inited".to_string()))
                .copied()
                .unwrap_or(0)
                == 0
            {
                ctx.core
                    .vm_aux
                    .flags
                    .insert((s, "ow_zhongdun_inited".to_string()), 1);
                let cur_h = ctx.core.player(s).hujia;
                ctx.core.player_mut(s).hujia = cur_h.saturating_add(8);
            }
        }
    }
    crate::kernel::elo_on_phase_before(ctx.core, ctx.catalog, player);
    if let Some(raw_def) = ctx
        .core
        .vm_aux
        .flags
        .remove(&(player, "gw_guaiwuchaoxue_temp_def".to_string()))
    {
        if let Some(def) = raw_def.checked_sub(1).and_then(|v| u16::try_from(v).ok()) {
            ctx.core.skills.remove(&(player, def));
        }
    }
    (ctx.catalog.hooks.on_turn_begin)(ctx, player);
    // K46/K47/K48（A2）：封冻／巡游／炎狱＝上游 player/global phaseBegin
    // 内联效果——置于 TurnBegin 窗 Commit 步（BeforeTriggers 已跑、准备
    // 阶段帧未压），严格落在 phaseBegin 窗内（原锚判定开始→prepare 入场）。
        // 封冻（Boss 应昭，规则 v228）：回合开始——所有其他
        // 存活角色获「封印」（本回合无法使用手牌；回合结束解
        // 除。上游 boss_fengdong phaseBegin addTempSkill fengyin
        // 同口径）
        if let Some(fddef) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_FENGDONG)
        {
            if ctx.core.has_skill(player, fddef) {
                for t in ctx.core.seat_order_from(player) {
                    if t != player && ctx.core.player(t).alive {
                        ctx.core.player_mut(t).fengyin_active = true;
                    }
                }
            }
        }

        // 巡游（Boss 应昭，规则 v231）：其他角色回合开始——
        // 获得座位序首名有手牌的其他角色的一张手牌（确定性近
        // 似：hej 全区域/随机以座位序+手牌首张近似登记）；装
        // 备牌自动使用（直接装上，v180 双刃系 grant 先例）
        if let Some(xdef) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_XUNYOU)
        {
            // global phaseBegin：任意 enabled 持有者（≠当前回
            // 合玩家）在场即触发（炎狱 holders 先例同构）
            let holders: Vec<PlayerId> = ctx
                .core
                .skills
                .iter()
                .filter(|((_, dd), i)| *dd == xdef && i.enabled)
                .map(|((o, _), _)| *o)
                .filter(|o| *o != player && ctx.core.player(*o).alive)
                .collect();
            if !holders.is_empty() {
                let holder = holders[0];
                let mut victim: Option<PlayerId> = None;
                for t in ctx.core.seat_order_from(holder) {
                    if t != holder
                        && ctx.core.player(t).alive
                        && !ctx.core.player(t).hand.is_empty()
                    {
                        victim = Some(t);
                        break;
                    }
                }
                if let Some(v) = victim {
                    let c = ctx.core.player(v).hand[0];
                    let kind = ctx.catalog.resolved_card_kind(ctx.core, c);
                    ctx.take_hand_card(v, c);
                    if let Some(slot) = kind.equip_slot() {
                        // 装备牌自动使用：同栏替换（旧装备入弃
                        // 堆并撤销技能——931 区 play 装备先例）
                        if let Some(old) = ctx
                            .core
                            .player(player)
                            .equips
                            .iter()
                            .copied()
                            .find(|e| ctx.catalog.card_slot_of(ctx.core, *e) == Some(slot))
                        {
                            ctx.core.player_mut(holder).equips.retain(|x| *x != old);
                            revoke_equip_skills(ctx.core, ctx.catalog, holder, old);
                            ctx.to_discard(old);
                        }
                        ctx.core.player_mut(holder).equips.push(c);
                        grant_equip_skills(ctx, holder, c);
                    } else {
                        ctx.core.player_mut(holder).hand.push(c);
                    }
                    ctx.log(LogEvent::CardsGiven {
                        from: v,
                        to: player,
                        cards: vec![c],
                    });
                }
            }
        }

        // 炎狱（Boss 毕方，规则 v229）：global phaseBegin——
        // 其他角色的回合开始判定（确定性判定铁骑先例），红色 →
        // 该角色受 1 点来自毕方的火伤；每回合至多 3 次（TurnState
        // 计数器，回合更替清零；上游 getIndex 3 同口径）
        if let Some(ydef) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_YANYU)
        {
            let holders: Vec<PlayerId> = ctx
                .core
                .skills
                .iter()
                .filter(|((_, dd), i)| *dd == ydef && i.enabled)
                .map(|((o, _), _)| *o)
                .filter(|o| *o != player && ctx.core.player(*o).alive)
                .collect();
            if ctx.core.turn.yanyu_count < 3 {
                for holder in holders {
                    if ctx.core.deck.is_empty() && ctx.core.discard.is_empty() {
                        break;
                    }
                    let judged = ctx.draw_judge_card_with_elo(
                        player,
                        crate::kernel::EloJudgePref::BlackPositive,
                    );
                    let (suit, _) = ctx.inline_judge_suit_rank(holder, judged);
                    let red = matches!(
                        suit,
                        crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                    );
                    let _ = ctx.settle_inline_judge_card(holder, suit, judged);
                    if red {
                        ctx.core.turn.yanyu_count += 1;
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(holder),
                            player,
                            1,
                            crate::event::DamageNature::Fire,
                            None,
                        )));
                    }
                    break; // 每次判定至多一持技者结算（单 Boss 场景）
                }
            }
        }


    // K23：废墟地图（mtg v103）——回合开始随机弃堆一张置牌堆顶（上游
    // phaseBegin silent；RNG 真消除口径）。
    if ctx.core.current_land == Some(crate::catalog::CardKind::Feixu)
        && !ctx.core.discard.is_empty()
    {
        let n = ctx.core.discard.len() as u32;
        let i = ctx.core.rng.gen_range(n) as usize;
        let c = ctx.core.discard.remove(i);
        ctx.core.deck.push(c);
        crate::kernel::elo_mark_deck_insert(ctx.core, c, false);
    }
    ctx.core.turn.phase = Phase::Prepare;
    ctx.emit(EventData::Phase {
        player,
        phase: Phase::Prepare,
        begin: false,
    });
    ctx.emit(EventData::Phase {
        player,
        phase: Phase::Prepare,
        begin: true,
    });
}

pub(in crate::kernel) fn commit_turn_ended(ctx: &mut Ctx<'_>, player: PlayerId) {
    if ctx.core.turn.extra_phases.is_empty() {
        let disabled_turn: Vec<(u8, u16)> = ctx
            .core
            .vm_aux
            .flags
            .keys()
            .filter_map(|(p, k)| {
                k.strip_prefix("skill_disabled_turn:")
                    .and_then(|d| d.parse::<u16>().ok())
                    .map(|def| (*p, def))
            })
            .collect();
        for (p, def) in disabled_turn {
            if let Some(inst) = ctx.core.skills.get_mut(&(p, def)) {
                inst.enabled = true;
            }
        }
        ctx.core
            .vm_aux
            .flags
            .remove(&(player, "mbcmfangzhu_ban".to_string()));
        ctx.core
            .vm_aux
            .flags
            .remove(&(player, "mbcmfangzhu_baiban".to_string()));
        let fangzhu_disabled: Vec<u16> = ctx
            .core
            .vm_aux
            .flags
            .keys()
            .filter_map(|(p, k)| {
                (*p == player)
                    .then(|| k.strip_prefix("mbcmfangzhu_disabled:"))
                    .flatten()
                    .and_then(|d| d.parse::<u16>().ok())
            })
            .collect();
        for def in fangzhu_disabled {
            ctx.core
                .vm_aux
                .flags
                .remove(&(player, format!("mbcmfangzhu_disabled:{def}")));
            if !ctx
                .core
                .vm_aux
                .flags
                .contains_key(&(player, format!("xj_fengyin_disabled:{def}")))
            {
                if let Some(inst) = ctx.core.skills.get_mut(&(player, def)) {
                    inst.enabled = true;
                }
            }
        }
        ctx.core
            .vm_aux
            .flags
            .retain(|(p, k), _| !(*p == player && k.starts_with("ssr_lieren_temp:")));
        ctx.core.vm_aux.clear_turn();
    }
    for p in 0..ctx.core.players.len() as PlayerId {
        ctx.core
            .vm_aux
            .flags
            .remove(&(p, "l25_wumou_trick_in_play".to_string()));
        ctx.core
            .vm_aux
            .flags
            .remove(&(p, "l25_gongxin_temp".to_string()));
    }
    ctx.log(LogEvent::TurnEnded { player });
    // R05（K21）：先发整回合结束窗（phaseEnd 段 → phaseAfter 段 → 交接），
    // 期间 turn.current 仍是刚结束回合的角色；换人移至 phaseAfter 段的
    // Commit（turn_handoff）。
    ctx.emit(EventData::TurnEndWindow {
        player,
        after: false,
    });
    for state in &mut ctx.core.players {
        state.zlfanshi_terra = false;
    }
    // 清残留跳阶段旗标（正常流程已被消费）
    ctx.core.turn.skip_draw_phase = false;
    ctx.core.turn.skip_play_phase = false;
    // 还侠跟踪清残留（正常流程已被阶段末回收消费）
    ctx.core.turn.huanxia_track.clear();
    // 神秘果/鼠槌窗口旗清残留（正常流程已被窗口机消费；防御换回合残留）
    ctx.core.turn.shenmiguo_used = false;
    ctx.core.turn.shuchui_armed = false;
    ctx.core.turn.shuchui_hit = false;
    ctx.core.turn.lianda_used = false;
    // Boss 模式（P2/N08 模式入口批三-a，规则 v82）：死亡座位不跳过——
    // 轮到其回合时执行重整 tick（上游 bossPhaseLoop）：计数 +1、hp<上限
    // 则 +1（否则手牌 <4 摸 1），计数达阈值复活；连续死亡座位逐个 tick
    // 后落在下一存活座位。
    if let Some((_, threshold)) = ctx.core.boss_mode() {
        let seats = ctx.core.players.len() as u8;
        let mut next = (player + 1) % seats;
        loop {
            if ctx.core.player(next).alive {
                break;
            }
            if threshold == 0 {
                // 上游 game.bossinfo.chongzheng 未设：无复活模式，跳过 tick
                next = (next + 1) % seats;
                if next == (player + 1) % seats {
                    break;
                }
                continue;
            }
            // 重整 tick
            let cnt = ctx.core.player(next).boss_chongzheng + 1;
            ctx.core.player_mut(next).boss_chongzheng = cnt;
            if (ctx.core.player(next).hp as u32) < ctx.core.player(next).max_hp {
                ctx.core.player_mut(next).hp += 1;
            } else if ctx.core.player(next).hand.len() < 4 {
                ctx.start_draw(next, 1);
            }
            ctx.log(LogEvent::BossChongzhengTick {
                player: next,
                count: cnt,
            });
            if cnt >= threshold {
                let p = ctx.core.player_mut(next);
                p.alive = true;
                p.dying = false;
                p.boss_chongzheng = 0;
                ctx.log(LogEvent::BossRevived { player: next });
            }
            next = (next + 1) % seats;
            if next == (player + 1) % seats {
                break; // 全场死亡（终局判负已在死亡结算处理）
            }
        }
        // R05（K21）：boss 重整 tick 已运行（复活者已被 next_alive 视为
        // 存活）；交接统一由 phaseAfter 段的 Commit 执行。
        let _ = next;
    }
    // 交接由整回合结束窗 phaseAfter 段的 Commit（commit_turn_end_window）
    // 统一执行——本函数不再直接换人。
}

pub(crate) use super::turn_handoff::turn_handoff;
pub(crate) use super::turn_reset::reset_new_turn_state;

/// Commits `Jiu` or `Tianxianjiu` card use independently (`card/swd/card.js:1751-1772`).
pub(in crate::kernel) fn commit_jiu_or_tianxianjiu_use(
    ctx: &mut Ctx<'_>,
    u: &crate::event::CardUseData,
    is_dying_save: bool,
    nullified: bool,
) {
    let jiu_no_count = ctx
        .core
        .vm_aux
        .flags
        .remove(&(u.player, "vm_va_jiu_no_count".to_string()))
        .is_some();
    if is_dying_save {
        ctx.core.table.extend(u.card);
        return;
    }
    ctx.to_discard(u.card);
    if u.effective_kind(ctx.catalog) == crate::catalog::CardKind::Tianxianjiu {
        if !jiu_no_count {
            ctx.core
                .vm_aux
                .flags
                .insert((u.player, "u05_tianxianjiu_used_turn".to_string()), 1);
        }
        if !nullified {
            if let Some(tdef) = ctx.catalog.skill_by_name("tianxianjiu") {
                ctx.ensure_skill(
                    (u.player, tdef),
                    crate::state::SkillInstance {
                        owner: u.player,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: true,
                        rounds: 0,
                    },
                );
            } else {
                ctx.core
                    .vm_aux
                    .flags
                    .insert((u.player, "u05_tianxianjiu_active_fallback".to_string()), 1);
            }
        }
        return;
    }
    // 酒：命令边界已校验每回合一次（addCount: false 时置 vm_va_jiu_no_count 不占位）——
    // 上游 target.storage.jiu += baseDamage 可跨多次使用叠加；回合外印记同理按次数累加。
    if u.player == ctx.core.turn.current {
        if !jiu_no_count {
            ctx.core.turn.jiu_used = true;
        }
        if !nullified {
            if !ctx.core.turn.jiu_active {
                ctx.core.turn.jiu_active = true;
            } else {
                let key = (u.player, "jiu_extra_stacks".to_string());
                let prev = ctx.core.vm_aux.flags.get(&key).copied().unwrap_or(0);
                ctx.core.vm_aux.flags.insert(key, prev + 1);
            }
        }
    } else if !nullified {
        let key = (u.player, "jiu_active_offturn".to_string());
        let prev = ctx.core.vm_aux.flags.get(&key).copied().unwrap_or(0);
        ctx.core.vm_aux.flags.insert(key, prev + 1);
    }
}
