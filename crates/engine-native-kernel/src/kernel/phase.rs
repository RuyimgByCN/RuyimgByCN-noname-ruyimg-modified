use super::*;

mod judge;
use judge::{begin_judge, begin_prepare};
mod draw;
use draw::begin_draw;
mod play;
use play::begin_play;
mod discard;
use discard::begin_discard;
mod end;
use end::begin_end;

/// 阶段帧提交：PhaseEnd 推进阶段或转入回合收束；PhaseBegin 执行入场效果。
pub(super) fn commit_phase(ctx: &mut Ctx<'_>, player: PlayerId, phase: Phase, begin: bool) {
    // R05（K10）：翻面整回合跳过已前移 `turn_handoff`（回合开始窗之前，
    // 上游 phaseBefore 检测时点）；本函数不再承担翻面检测。
    // 强固解除（U05 ow 奥丽莎，规则 v136）：自己回合判定阶段开始——
    // 上游 addTempSkill("qianggu2") 至 phaseBegin 同口径
    // 强固解除（K02）：已迁 reset_new_turn_state（TurnBegin 前）。
    // 潜行解除（U05 ow 黑影，规则 v139）：自己回合判定阶段开始——上游
    // tempHide 至 phaseBeginStart 同口径
    // 潜行解除（K03）：已迁 reset_new_turn_state（上游 tempHide 至
    // phaseBeginStart，先于 phaseBegin）。
    // 嘲讽解除（U05 swd 灵剑圣休号角，规则 v198）：自己回合判定阶段开始
    // ——上游 addTempSkill {player:'phaseBegin'} 同口径
    // 嘲讽解除（K04）：已迁 reset_new_turn_state（上游 {player:'phaseBegin'}）。
    // 星体改造重置（K05，A2）：已迁整回合结束窗 phaseEnd 段（上游
    // phaseEnd 移除+清零）——原判定开始重置的「差一夜」登记差异随迁对齐。
    // 蛊舞递减（U05 xianjian 阿奴，规则 v154）：已迁整回合结束窗
    // phaseAfter 段（kernel/events.rs commit_turn_end_window）——上游
    // anwugu2 trigger {player:'phaseAfter'} 每回合递减一次，原「每阶段
    // 结束」臂在准备阶段加入后每回合递减 6 次（A2 审计 K01）。
    // 月行（U05 xianjian 龙幽，规则 v159）：出牌阶段结束解除距离基数交
    // 换（双向互指一并清除；上游 yuexing2 临时技 phaseUseAfter 同口径）
    if !begin && phase == Phase::Play {
        if ctx.core.player(player).yuexing_pair.is_some() {
            if let Some(pair) = ctx.core.player_mut(player).yuexing_pair.take() {
                if ctx.core.player(pair).yuexing_pair == Some(player) {
                    ctx.core.player_mut(pair).yuexing_pair = None;
                }
            }
        }
    }
    // K56：虚失·自翻面（Boss 毕方系；上游 player ['phaseUseEnd',
    // 'turnOverEnd']）——出牌阶段结束时经 TurnOverBefore 提交翻面。
    if !begin && phase == Phase::Play {
        if let Some(xdef) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_XUSHI)
        {
            if ctx.core.has_skill(player, xdef) && ctx.core.player(player).alive {
                ctx.emit(EventData::TurnOverBefore {
                    player,
                    cancelled: false,
                });
            }
        }
    }
    // 逐月（U05 xianjian 云天河，规则 v161）：出牌阶段结束清白名单
    //（上游 zhuyue2 phaseUseEnd silent 同口径）
    if !begin && phase == Phase::Play {
        ctx.core.turn.zhuyue_targets.clear();
    }
    // 诚信免伤清除（U05 xianjian 明绣，规则 v164）：本人 End 阶段结束
    //（登记差异：上游 chengxin2 存活至最近阶段结束）
    if !begin && phase == Phase::End && ctx.core.player(player).chengxin_active {
        ctx.core.player_mut(player).chengxin_active = false;
    }
    // 屏障标志转计数（U05 ow 查莉娅，规则 v138）：自己回合判定阶段开
    // 始——上回合两标志各计一次并清除（上游 pingzhang_count phaseBegin
    // silent 同口径；应用技在随后摸牌阶段读计数）
    // 屏障转计数（K09）：已迁 reset_new_turn_state（phaseBegin silent）。
    if begin && phase == Phase::Play && ctx.core.turn.skip_play_phase {
        ctx.core.turn.skip_play_phase = false;
        ctx.record_phase_skipped(player);
        ctx.core.turn.phase = Phase::Discard;
        ctx.emit(EventData::Phase {
            player,
            phase: Phase::Discard,
            begin: true,
        });
        return;
    }
    ctx.log(LogEvent::PhaseChanged {
        player,
        phase,
        begin,
    });
    // 全局阶段计数（U07 ow 自愈，规则 v120）：每阶段开始 +1（上游
    // game.phaseNumber 同口径）
    // 准备阶段（R05）不计入：保持既有「每回合五阶段」计数口径不漂移。
    if begin && phase != Phase::Prepare {
        ctx.core.turn.phase_number += 1;
    }
    if begin {
        match phase {
            // 准备阶段无固有入场行为；开始/结束触发窗由���生命周期提供。
            // R05（A2）：准备阶段入场＝原判定开始 `!turn_opened` 门内的
            // 准备阶段 Boss 技段（上游 phaseZhunbeiBegin）。
            Phase::Prepare => begin_prepare(ctx, player),
            Phase::Judge => begin_judge(ctx, player),
            Phase::Draw => begin_draw(ctx, player),
            Phase::Play => begin_play(ctx, player),
            Phase::Discard => begin_discard(ctx, player),
            Phase::End => begin_end(ctx, player),
        }
        return;
    }
    // PhaseEnd
    // 筑城开关（K14，A2；规则 v113→333）：上游 player:"phaseEnd"＝本人
    // 整回合结束（每回合一次）——原「每阶段结束」在准备阶段加入后每回合
    // 6 次询问；已迁 kernel/events.rs commit_turn_end_window phaseEnd 段。
    // 还侠回收（swd 屈娴，批 27）：持有者每阶段结束收回未造成伤害的还
    // 侠转化杀（上游 gain 子技 player phaseEnd；登记差异上游经特殊区暂
    // 存，原生从弃牌堆取回——牌若已离开弃牌堆则不收回）
    if !ctx.core.turn.huanxia_track.is_empty() && phase == Phase::End && !begin {
        let pending = std::mem::take(&mut ctx.core.turn.huanxia_track);
        for (card, caused) in pending {
            if caused {
                continue;
            }
            if let Some(pos) = ctx.core.discard.iter().position(|x| *x == card) {
                let c = ctx.core.discard.remove(pos);
                ctx.core.player_mut(player).hand.push(c);
                ctx.log(LogEvent::CardsGivenPublic {
                    from: player,
                    to: player,
                    cards: vec![c],
                });
            }
        }
    }
    // 水没墓穴地图效果（mtg 第三批，规则 v102）：锁定——本弃牌阶段弃
    // 置过牌则阶段结束摸一（上游 phaseDiscardEnd forced 同口径）
    if phase == Phase::Discard {
        if ctx.core.current_land == Some(crate::catalog::CardKind::Shuimomuxue)
            && ctx.core.turn.discard_phase_discarded > 0
        {
            ctx.start_draw(player, 1);
        }
        ctx.core.turn.discard_phase_discarded = 0;
    }
    // 光塔（K13，A2；规则 v119→333）：♠子技持有者于**下一次整回合结束**
    // 受 1 无源雷伤（上游 trigger phaseEnd；intro 写「下个结束阶段」与
    // trigger 矛盾——按 trigger 记，规则基准登记取舍）。已迁 kernel/
    // events.rs commit_turn_end_window phaseEnd 段。
    // 魂殉志·殉（swd 批 132）：出牌阶段**收束**时持有「殉」者死亡（上游
    // hxunzhi2 phaseUseEnd forced player.die() 同口径——直接死亡，非伤害）
    if !begin && phase == Phase::Play {
        if let Some(def) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::host::SWD_HXUNZHI2_KEY)
        {
            if ctx.core.skills.remove(&(player, def)).is_some() {
                ctx.emit(EventData::Death {
                    player,
                    killer: None,
                });
            }
        }
    }
    // 护天（swd 批 131，规则 v314）：寄存只跨一个阶段边界（上游 `hutian4`
    // 临时技在下一阶段边界移除 → `hutian3` 于下一次阶段收束归还）。原生
    // 阶段收束的 commit 先于该阶段的触发窗，故「寄存当次的收束」看不到
    // 寄存（just_given 尚未置位），**下一次阶段收束**即归还 ✓
    // 护天寄存取回（K17，A2）：已迁整回合结束窗 phaseEnd 段（kernel/
    // events.rs commit_turn_end_window——上游 hutian4 临时技下一次阶段
    // 边界归还；原「任意阶段结束」在准备阶段加入后归还时点×6/回合）。
    // 颂词组技 songci_draw（swd 批 129，规则 v313）：弃牌阶段**收束**时若
    // 全部角色均已记录（无未记录者）→ 摸 1（上游 phaseDiscardEnd forced）
    if !begin && phase == Phase::Discard {
        if let Some(def) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::host::SONGCI_KEY)
        {
            if ctx.core.has_skill(player, def) {
                // 上游 `game.hasPlayer(!storage.songci.contains(current))`：
                // 计**全部角色**（含已阵亡者）——阵亡未记录即不触发
                let all_recorded = {
                    let p = ctx.core.player(player);
                    ctx.core
                        .players
                        .iter()
                        .enumerate()
                        .all(|(idx, _)| p.songci_targets.contains(&(idx as PlayerId)))
                };
                if all_recorded {
                    ctx.start_draw(player, 1);
                }
            }
        }
    }
    // 乐不思蜀生效：摸牌阶段结束时若跳过出牌旗标在位，
    // 直接进入弃牌阶段（跳过 Play 的 End/Begin 事件对——挂账见清单）。
    if phase == Phase::Draw && ctx.core.turn.skip_play_phase {
        ctx.core.turn.skip_play_phase = false;
        ctx.record_phase_skipped(player);
        ctx.core.turn.phase = Phase::Discard;
        ctx.emit(EventData::Phase {
            player,
            phase: Phase::Discard,
            begin: true,
        });
        return;
    }
    // 灵武（swd 妮可，批 120，规则 v307）：频繁技——阶段结束时若本回合
    // 已用牌数 ≥ 体力则插入一个额外出牌阶段（上游 `insertPhase()` 同口径；
    // 插入阶段的收束不再触发=上游 `event.skill != 'lingwu'` 防递归）。
    {
        let from_lingwu = ctx.core.turn.extra_phase_lingwu;
        ctx.core.turn.extra_phase_lingwu = false;
        if !from_lingwu {
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
        }
    }
    // 额外阶段队列（insertPhase 原语）：非空则弹出为下一阶段
    if !ctx.core.turn.extra_phases.is_empty() {
        let code = ctx.core.turn.extra_phases.remove(0);
        let next = match code {
            0 => Phase::Judge,
            1 => Phase::Draw,
            2 => Phase::Play,
            3 => Phase::Discard,
            _ => Phase::End,
        };
        ctx.core.turn.phase = next;
        ctx.emit(EventData::Phase {
            player,
            phase: next,
            begin: true,
        });
        return;
    }
    match phase.next() {
        Some(next) => {
            ctx.core.turn.phase = next;
            ctx.emit(EventData::Phase {
                player,
                phase: next,
                begin: true,
            });
        }
        None => {
            // 混乱解除（Boss 郭嘉鬼谋，规则 v259）：混乱者回合收束解除
            //（上游 addTempSkill "mad" {player:"phaseAfter"} 同口径）；
            // 奇佐副本载体槽跨回合兜底清
            ctx.core.player_mut(player).boss_mad = false;
            ctx.core.turn.qizuo_copy = None;
            ctx.emit(EventData::TurnEnded { player });
        }
    }
}
