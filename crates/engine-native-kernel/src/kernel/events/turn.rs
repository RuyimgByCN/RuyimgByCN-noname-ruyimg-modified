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
        ctx.emit(EventData::TurnEndWindow {
            player,
            after: true,
        });
    }
}

pub(in crate::kernel) fn commit_turn_begin(ctx: &mut Ctx<'_>, player: PlayerId) {
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
                    let kind = ctx.catalog.card_kind(c);
                    ctx.core.player_mut(v).hand.retain(|x| *x != c);
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
                    let judged = ctx.draw_judge_card();
                    let red = matches!(
                        ctx.catalog.cards[judged as usize].suit,
                        crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                    );
                    ctx.to_discard(judged);
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
    ctx.core.vm_aux.clear_turn();
    ctx.log(LogEvent::TurnEnded { player });
    // R05（K21）：先发整回合结束窗（phaseEnd 段 → phaseAfter 段 → 交接），
    // 期间 turn.current 仍是刚结束回合的角色；换人移至 phaseAfter 段的
    // Commit（turn_handoff）。
    ctx.emit(EventData::TurnEndWindow {
        player,
        after: false,
    });
    ctx.expire_temporary_skills(player);
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

/// R05（K21／K25）：回合交接——在整回合结束窗 phaseAfter 段的 Commit 中
/// 执行：轮次推进、换人、回合级存量清零、**新回合玩家的回合重置**
/// （`begin_judge` 原 `!turn_opened` 门整体前移至此，使回合开始窗与准备
/// 阶段读到本回合的新值），最后发 TurnBegin（自动接准备阶段→判定）。
pub(in crate::kernel) fn turn_handoff(ctx: &mut Ctx<'_>, prev: PlayerId, next: PlayerId) {
    // R05-F（§6 兜底位）：任何已死亡角色的 Limit/Combs/Current 语义槽与
    // helper 实例在回合交接时清理（真实死亡经 DeathMachine 前置清理；此
    // 处覆盖无死亡帧的跳过路径——Next 保留）。
    for role in ["limit", "combs", "current"] {
        for def in (ctx.catalog.hooks.r05_role_defs)(ctx.catalog, role) {
            for seat in 0..ctx.core.players.len() as u8 {
                if !ctx.core.player(seat).alive {
                    ctx.core.vm_aux.marks.remove(&(seat, def as u32));
                    ctx.core.skills.remove(&(seat, def));
                }
            }
        }
    }
    // K18：灵武在 phaseAfter 段插入的额外出牌阶段——回合不交接，本人
    // 直接再入出牌阶段（上游 insertPhase；extra_phase_lingwu 防递归，
    // 收束后队列空→正常交接）。
    if !ctx.core.turn.extra_phases.is_empty() && ctx.core.turn.current == prev {
        let code = ctx.core.turn.extra_phases.remove(0);
        let phase = match code {
            0 => Phase::Judge,
            1 => Phase::Draw,
            2 => Phase::Play,
            3 => Phase::Discard,
            _ => Phase::End,
        };
        ctx.core.turn.phase = phase;
        ctx.emit(EventData::Phase {
            player: prev,
            phase,
            begin: true,
        });
        return;
    }
    // R05（K10）：翻面整回合跳过在回合开始窗之前（上游 phaseBefore 检测）
    // ——翻面者不发 TurnBegin／不进任何阶段（含准备阶段）；翻回序列与
    // TurnEnded 同 E03/U05 原口径，TurnEnded 的窗→handoff 链继续交接。
    if ctx.core.player(next).face_down {
        ctx.core.player_mut(next).jileng_active = false;
        ctx.emit(EventData::TurnOverBefore {
            player: next,
            cancelled: false,
        });
        ctx.core.player_mut(next).boss_mad = false;
        ctx.emit(EventData::TurnEnded { player: next });
        return;
    }
    if next <= prev {
        ctx.core.turn.round += 1;
        ctx.core.vm_aux.clear_round();
    }
    ctx.core.turn.current = next;
    ctx.core.turn.phase = Phase::Judge;
    ctx.core.turn.sha_used_play_phase = false;
    ctx.core.turn.land_used = false;
    ctx.core.turn.land_sha_used = false;
    ctx.core.turn.turn_opened = false;
    // 回合级存量清零（v128 修复 v126 遗留：TurnState 换回合为原地
    // 改字段而非重建——乐动三角/链钩必须显式复位）
    ctx.core.turn.yuedong_num = 0;
    ctx.core.turn.yuedong_eff = 0;
    ctx.core.turn.yuedong_recover = false;
    ctx.core.turn.liangou_hook = None;
    ctx.core.turn.liangou_bonus_used = false;
    ctx.core.turn.feitiao_target = None;
    ctx.core.turn.fanling_used = false;
    ctx.core.turn.xjyufeng_used = 0;
    ctx.core.turn.diewu_draw_used = false;
    ctx.core.turn.lueying_used = false;
    // 冥顽锁定集（Boss 梼杌，规则 v219）：回合更替清零（上游 phaseAfter
    // clear 同窗）
    ctx.core.turn.minwan_targets.clear();
    ctx.core.turn.yanyu_count = 0;
    ctx.core.turn.txianqu_used = false;
    ctx.core.turn.jtjubao_used = false;
    reset_new_turn_state(ctx, next);
    ctx.emit(EventData::TurnBegin { player: next });
}

/// 新回合玩家的回合重置（K25：原 `begin_judge` 的 `!turn_opened` 门整体
/// 前移；`turn_opened` 置真保留字段语义）。含红莲回合开始效果（原判定
/// 开始执行——时点前移属 A2 审计 K25 项，行为面等价：仍先于回合开始窗
/// 与准备阶段）。
pub(in crate::kernel) fn reset_new_turn_state(ctx: &mut Ctx<'_>, player: PlayerId) {
    if ctx.core.turn.turn_opened {
        return;
    }
    ctx.core.turn.turn_opened = true;
    ctx.reset_turn_limits(player);
    // 本回合已用牌数（swd 灵武批 120，规则 v307）：回合开始清零
    ctx.core.turn.cards_used_turn = 0;
    // 回合伤害统计清零（U05 xianjian 明绣，规则 v160）
    ctx.core.player_mut(player).turn_damage = 0;
    // 忘忧戳清零（U05 xianjian 紫萱，规则 v162；登记差异：保留期延长）
    ctx.core.player_mut(player).wangyou3_round = 0;
    // 摄灵回合次数清零（U05 xianjian 南宫煌，规则 v168）
    ctx.core.turn.sheling_used = 0;
    // 射影回合清零（U05 xianjian 苏媚，规则 v175）
    ctx.core.turn.sheying_used = false;
    // 张目回合清零（U05 xianjian 南宫煌，规则 v181）
    ctx.core.turn.zhangmu_used = false;
    // 红莲（Boss 毕方，规则 v228）：回合开始——顺序获牌堆前缀红牌至多
    // 3 张，并对座位序前至多 3 名其他角色各造成 1 点火伤
    if let Some(hdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_HONGLIANX)
    {
        if ctx.core.has_skill(player, hdef) {
            let mut gained = 0usize;
            while gained < 3 && !ctx.core.deck.is_empty() {
                let c = ctx.core.deck[ctx.core.deck.len() - 1];
                if matches!(
                    ctx.catalog.cards[c as usize].suit,
                    crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                ) {
                    ctx.core.deck.pop();
                    ctx.core.player_mut(player).hand.push(c);
                    gained += 1;
                } else {
                    break;
                }
            }
            if gained > 0 {
                ctx.log(LogEvent::CardsDrawn {
                    player,
                    cards: ctx
                        .core
                        .player(player)
                        .hand
                        .iter()
                        .rev()
                        .take(gained)
                        .copied()
                        .collect::<Vec<_>>(),
                });
            }
            let targets: Vec<PlayerId> = ctx
                .core
                .seat_order_from(player)
                .into_iter()
                .filter(|t| *t != player && ctx.core.player(*t).alive)
                .take(3)
                .collect();
            for t in targets {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(player),
                    t,
                    1,
                    crate::event::DamageNature::Fire,
                    None,
                )));
            }
        }
    }
    // K02/K03/K04：三个「至 phaseBegin」到期解除（先于回合开始窗）。
    if ctx.core.player(player).qianggu_active {
        ctx.core.player_mut(player).qianggu_active = false;
    }
    if ctx.core.player(player).yinshen_active {
        ctx.core.player_mut(player).yinshen_active = false;
    }
    if ctx.core.player(player).chaofeng_active {
        ctx.core.player_mut(player).chaofeng_active = false;
    }
    // K09：屏障标志转计数（上游 pingzhang_count phaseBegin silent）。
    {
        let used = u8::from(ctx.core.player(player).pingzhang_self_used)
            + u8::from(ctx.core.player(player).pingzhang_other_used);
        let p = ctx.core.player_mut(player);
        p.pingzhang_count = used;
        p.pingzhang_self_used = false;
        p.pingzhang_other_used = false;
    }
    // K30：形态轮换（Boss 年兽系，规则 v249）——上游 phaseBefore 换将，
    // 须先于 TurnBegin 使新形态的回合开始技生效。
        // 形态轮换（Boss 年兽系，规则 v249）：回合开始——
        // 交替旗标首次置位不换将（初始形态即条目形态），此后
        // 每回合轮换至另一「形态轮换」携带将（assembly 重挂+
        // hp 保留——versus 替补换将先例同构；随机序列→确定性
        // 互斥轮转登记）
        if let Some(nbdef) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_NBIANSHEN)
        {
            if ctx.core.has_skill(player, nbdef) {
                let armed = ctx.core.player(player).nbianshen_toggle;
                if armed {
                    // 启动后每回合轮换（上游 storage 存在后每
                    // 次 phaseBefore 均换同口径）
                    // 轮换：找另一携带「形态轮换」的 BOSS 段将
                    let base = (ctx.catalog.general_count()
                        - crate::vocabulary::generals::BOSS_GENERALS.len())
                        as u16;
                    let cur = ctx.core.player(player).general_slots[0];
                    let other = (base..ctx.catalog.general_count() as u16).find(|&d| {
                        Some(d) != cur
                            && ctx
                                .catalog
                                .general_meta(d)
                                .map(|m| m.2.contains(&crate::vocabulary::names::BOSS_NBIANSHEN))
                                .unwrap_or(false)
                    });
                    if let (Some(other), Some(cur)) = (other, cur) {
                        let _ = cur;
                        // 技能实例清空+重挂（区域保留）
                        let removed: Vec<crate::ids::SkillDefId> = ctx
                            .core
                            .skills
                            .keys()
                            .filter(|(o, _)| *o == player)
                            .map(|(_, d)| *d)
                            .collect();
                        for d in removed {
                            ctx.core.skills.remove(&(player, d));
                        }
                        ctx.core.player_mut(player).general_slots[0] = Some(other);
                        crate::state::apply_general_assembly(ctx.core, ctx.catalog, player, 0);
                        // hp/手牌/装备/判定区保留（上游 init 保
                        // 留 hp/maxHp/hujia 同口径）
                        ctx.log(LogEvent::SkillToggled {
                            player,
                            skill: "形态轮换",
                            enabled: true,
                        });
                    }
                }
                ctx.core.player_mut(player).nbianshen_toggle = true;
            }
        }

}
