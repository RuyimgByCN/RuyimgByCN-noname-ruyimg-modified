//! R05 抽牌流程机（裁决 28／29；合同 v3 §3–§4）。
//!
//! - `DrawPhaseMachine`（D）：窗口序 Before→Begin（pending 修正）→Begin1→
//!   Begin2→主 draw（R 子树）→End→After。Skipped/Cancelled 在 Before 前终止。
//! - `DrawMachine`（R）：逐张从牌堆取牌入 `held`（唯一 owner），完成后发一
//!   个 G 子帧；牌堆弃堆双枯竭 → `Exhausted`，held 转 detached。
//! - `GainMachine`（G）：Filter→RecordHistory（D.actual_gain 记账）→Transfer
//!   （held→recipient 手，CardsDrawn 日志）→After→Done。
//!
//! 当前为主 draw 路径的骨架：pending 修正由 D 机携带的宿主调整表计算
//! （与既有 `begin_draw` 同口径逐项移植），E/F 批把修正面换成通用 compound。

use super::*;
use crate::event::{
    DetachedKind, DrawGainProgress as GP, DrawOutcome as O, DrawPhaseProgress as DP,
    DrawTerminal as T, DrawWindowKind as W, EventKind, Stage,
};

#[derive(Clone)]
pub struct DrawPhaseMachine {
    pub root: EventId,
}

impl crate::catalog::SkillMachine for DrawPhaseMachine {
    fn fork(&self) -> Box<dyn crate::catalog::SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::DrawPhase { root: self.root }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let Some(frame) = ctx.res.frame(self.root) else {
            return Flow::Done;
        };
        let EventData::DrawPhase {
            player,
            mut progress,
            mut pending,
            num_fixed,
            mut actual_gain,
            mut outcome,
        } = frame.data.clone()
        else {
            return Flow::Done;
        };
        let mut child = None;
        match progress {
            DP::Before => {
                progress = DP::Begin;
                child = Some(win(self.root, W::Before));
            }
            DP::Begin => {
                // pending 修正窗（§3）：宿主 phaseDrawBegin num± 表——与旧
                // begin_draw 同口径逐项相加（有符号），执行入口统一 max(0)。
                pending += host_pending_adjust(ctx, player);
                // 玉如意（swd b44）：摸牌前把牌堆末 20 张窗口内最大点数牌
                // 移至摸牌位（pop 位）——与旧 begin_draw 同口径。
                if ctx
                    .core
                    .player(player)
                    .equips
                    .iter()
                    .any(|c| ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Yuruyi)
                    && ctx.core.deck.len() > 1
                {
                    let len = ctx.core.deck.len();
                    let window = len.min(20);
                    let mut best = len - 1;
                    for i in (len - window)..len {
                        if ctx.catalog.cards[ctx.core.deck[i] as usize].rank
                            > ctx.catalog.cards[ctx.core.deck[best] as usize].rank
                        {
                            best = i;
                        }
                    }
                    if best != len - 1 {
                        let card = ctx.core.deck.remove(best);
                        ctx.core.deck.push(card);
                    }
                }
                // 雪霁冰雹：应用 +1 后计数递减（旧 begin_draw 同步同口径）。
                if ctx.core.player(player).xuejibingbao > 0 {
                    ctx.core.player_mut(player).xuejibingbao -= 1;
                }
                progress = DP::Begin1;
                child = Some(win(self.root, W::Begin));
            }
            DP::Begin1 => {
                progress = DP::Begin2;
                child = Some(win(self.root, W::Begin1));
            }
            DP::Begin2 => {
                progress = DP::MainWaiting;
                child = Some(win(self.root, W::Begin2));
            }
            DP::MainWaiting => {
                // R 子树弹出后回到本机；主 draw 只发一次。
                let still_running = ctx.res.stack.iter().any(|f| {
                    matches!(f.data, EventData::Draw { .. }) && f.parent == Some(self.root)
                });
                if !still_running {
                    progress = DP::End;
                    child = Some(win(self.root, W::End));
                }
            }
            DP::End => {
                progress = DP::After;
                child = Some(win(self.root, W::After));
            }
            DP::After => {
                progress = DP::Done;
                outcome = O::Ended;
            }
            DP::Done => {
                return Flow::Done;
            }
        }
        let _ = (&num_fixed, &actual_gain);
        actual_gain = actual_gain.max(0);
        ctx.res.frame_mut(self.root).unwrap().data = EventData::DrawPhase {
            player,
            progress,
            pending,
            num_fixed,
            actual_gain,
            outcome,
        };
        if let Some(child) = child {
            // 主 draw：R 子帧（fixed 路径与替代 draw 由后续批次接入）。
            if matches!(child, EventData::DrawWindow { window: W::End, .. }) {
                let count = pending.max(0) as u32;
                if count > 0 {
                    let id = ctx.res.next_event_id;
                    ctx.res.stack.push(crate::event::Frame {
                        id,
                        parent: Some(self.root),
                        kind: EventKind::Draw,
                        data: EventData::Draw {
                            player,
                            requested: count,
                            held_cards: Vec::new(),
                            terminal: T::Running,
                        },
                        stage: Stage::Intrinsic,
                        queue: Vec::new(),
                        planned: false,
                        intrinsic_done: false,
                    });
                    ctx.res.next_event_id += 1;
                    return Flow::Yield;
                }
            }
            ctx.emit(child);
        }
        Flow::Yield
    }
}

fn win(root: EventId, window: W) -> EventData {
    EventData::DrawWindow { root, window }
}

/// 宿主 phaseDrawBegin num± 修正表（与旧 `begin_draw` 同口径移植；后续
/// 批次迁往 compound pending 前窗）。白翼/雪霁/神恩/无脏/藏目/年瑞/太平。
fn host_pending_adjust(ctx: &Ctx<'_>, player: PlayerId) -> i32 {
    let mut delta = 0i32;
    let name = |k: &str| {
        ctx.catalog
            .skill_by_name(k)
            .map(|def| ctx.core.has_skill(player, def))
            .unwrap_or(false)
    };
    // 白翼（Boss 少昊）：第 3 轮前敌方 -1（持有者在场光环，非本人持技）
    if ctx.core.turn.round < 3
        && !ctx.core.is_boss_side(player)
        && ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_BAIYI)
            .map(|def| {
                ctx.core
                    .skills
                    .keys()
                    .any(|(holder, dd)| *dd == def && ctx.core.player(*holder).alive)
            })
            .unwrap_or(false)
    {
        delta -= 1;
    }
    // 雪霁冰雹：标记角色 +1（计数 2→1→0 两轮，应用即递减）
    if ctx.core.player(player).xuejibingbao > 0 {
        delta += 1;
    }
    // 神恩：挑战者侧 +1（光环＝场上存在持有者）
    if ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_SHENEN)
        .map(|def| ctx.core.skills.keys().any(|(_, dd)| *dd == def))
        .unwrap_or(false)
        && !ctx.core.is_boss_side(player)
    {
        delta += 1;
    }
    // 无脏：基数改 max(5, hp/2)——固定 2 基数下的差值
    if name(crate::vocabulary::names::BOSS_WUZANG) {
        let hp = ctx.core.player(player).hp.max(0) as i32;
        delta += (hp / 2).max(5) - 2;
    }
    // 藏目：基数改存活数
    if name(crate::vocabulary::names::BOSS_CANGMU) {
        delta += ctx.core.alive_count() as i32 - 2;
    }
    // 年瑞／太平：+2
    if name(crate::vocabulary::names::BOSS_NIANRUI) {
        delta += 2;
    }
    if name(crate::vocabulary::names::BOSS_TAIPING) {
        delta += 2;
    }
    // 灵矢（swd 计亭）：装备区有马 +1
    if name("lingshi")
        && ctx.core.player(player).equips.iter().any(|c| {
            matches!(
                ctx.catalog.card_slot_of(ctx.core, *c),
                Some(crate::catalog::EquipSlot::HorsePlus)
                    | Some(crate::catalog::EquipSlot::HorseMinus)
            )
        })
    {
        delta += 1;
    }
    // 神威：+min(3, 其他角色数)
    if name(crate::vocabulary::host::BOSS_SHENWEI_KEY) {
        let others = ctx.core.alive_players().filter(|p| *p != player).count();
        delta += others.min(3) as i32;
    }
    // 灭道／诈降：+已损失体力
    for key in [
        crate::vocabulary::host::SWD_MIEDAO_KEY,
        crate::vocabulary::host::L27_SBZHAXIANG_KEY,
    ] {
        if name(key) {
            let lost = (ctx.core.player(player).max_hp as i32
                - ctx.core.player(player).hp)
                .max(0);
            delta += lost;
        }
    }
    // 摸牌翻倍：+2
    if name(crate::vocabulary::host::L20_MOPAI_KEY) {
        delta += 2;
    }
    // 脉轮：5=−1、6=+1；刺骨寒霜：−1（num>0 才减，执行入口统一 max(0)）
    delta += match ctx.core.player(player).mailun_mode {
        5 => -1,
        6 => 1,
        _ => 0,
    };
    if name(crate::vocabulary::host::GW_CIGU_STATE_KEY) {
        delta -= 1;
    }
    delta
}

#[derive(Clone)]
pub struct DrawMachine {
    pub root: EventId,
}

impl crate::catalog::SkillMachine for DrawMachine {
    fn fork(&self) -> Box<dyn crate::catalog::SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::Draw { root: self.root }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let Some(frame) = ctx.res.frame(self.root) else {
            return Flow::Done;
        };
        let EventData::Draw {
            player,
            requested,
            mut held_cards,
            mut terminal,
        } = frame.data.clone()
        else {
            return Flow::Done;
        };
        if terminal == T::Running {
            if held_cards.len() < requested as usize {
                if ctx.core.deck.is_empty() {
                    if ctx.core.discard.is_empty() {
                        // 双枯竭：held 转 detached，本次无 gain（§4）。
                        for (c, _) in held_cards.iter().map(|c| (*c, DetachedKind::Unclaimed)) {
                            ctx.core.detached_cards.push((c, DetachedKind::Unclaimed));
                        }
                        terminal = T::Exhausted;
                        ctx.res.frame_mut(self.root).unwrap().data = EventData::Draw {
                            player,
                            requested,
                            held_cards,
                            terminal,
                        };
                        return Flow::Done;
                    }
                    let mut cards = std::mem::take(&mut ctx.core.discard);
                    ctx.core.rng.shuffle(&mut cards);
                    ctx.core.deck = cards;
                    ctx.log(LogEvent::DeckReshuffled);
                }
                let card = ctx.core.deck.pop().expect("deck non-empty");
                held_cards.push(card);
                ctx.res.frame_mut(self.root).unwrap().data = EventData::Draw {
                    player,
                    requested,
                    held_cards,
                    terminal,
                };
                return Flow::Yield;
            }
            terminal = T::Completed;
            ctx.res.frame_mut(self.root).unwrap().data = EventData::Draw {
                player,
                requested,
                held_cards: held_cards.clone(),
                terminal,
            };
            if !held_cards.is_empty() {
                ctx.emit(EventData::DrawGain {
                    draw: self.root,
                    recipient: player,
                    eligible: held_cards,
                    progress: GP::Filter,
                    history_recorded: false,
                    cancelled: false,
                });
            }
            return Flow::Yield;
        }
        // Completed：等 G 子树弹出。
        let gain_running = ctx.res.stack.iter().any(|f| {
            matches!(f.data, EventData::DrawGain { draw, .. } if draw == self.root)
        });
        if !gain_running {
            return Flow::Done;
        }
        Flow::Yield
    }
}

#[derive(Clone)]
pub struct GainMachine {
    pub root: EventId,
}

impl crate::catalog::SkillMachine for GainMachine {
    fn fork(&self) -> Box<dyn crate::catalog::SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::Gain { root: self.root }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let Some(frame) = ctx.res.frame(self.root) else {
            return Flow::Done;
        };
        let EventData::DrawGain {
            draw,
            recipient,
            eligible,
            mut progress,
            mut history_recorded,
            cancelled,
        } = frame.data.clone()
        else {
            return Flow::Done;
        };
        if cancelled || eligible.is_empty() {
            // 原子释放剩余 held（骨架：取消面由调用方批次接入）。
            ctx.res.frame_mut(self.root).unwrap().data = EventData::DrawGain {
                draw,
                recipient,
                eligible,
                progress: GP::Done,
                history_recorded,
                cancelled,
            };
            return Flow::Done;
        }
        match progress {
            GP::Filter => progress = GP::RecordHistory,
            GP::RecordHistory => {
                history_recorded = true;
                // D.actual_gain 折叠：recipient 是阶段角色、gain 直接父是 R、
                // 最近 phaseDraw 祖先即该 D（§3）。
                if let Some(did) = nearest_draw_phase(ctx, draw) {
                    if let EventData::DrawPhase { actual_gain, .. } =
                        &mut ctx.res.frame_mut(did).unwrap().data
                    {
                        *actual_gain += eligible.len() as u32;
                    }
                }
                progress = GP::Transfer;
            }
            GP::Transfer => {
                // 同步更新真正 owner：R.held 移除、recipient 手加入（§4 表：
                // 同一步完成转移并推进 After，不加 D 计数）。
                if let EventData::Draw { held_cards, .. } =
                    &mut ctx.res.frame_mut(draw).unwrap().data
                {
                    for c in &eligible {
                        held_cards.retain(|x| x != c);
                    }
                }
                for c in &eligible {
                    ctx.core.player_mut(recipient).hand.push(*c);
                }
                // 事件帧＋日志双发（与旧 draw_cards 同面：mbqianlong 等
                // CardsDrawn 触发面依赖事件帧而非日志）。
                ctx.emit(EventData::CardsDrawn {
                    player: recipient,
                    cards: eligible.clone(),
                });
                ctx.log(LogEvent::CardsDrawn {
                    player: recipient,
                    cards: eligible.clone(),
                });
                progress = GP::After;
            }
            GP::After => progress = GP::Done,
            GP::Done => return Flow::Done,
        }
        ctx.res.frame_mut(self.root).unwrap().data = EventData::DrawGain {
            draw,
            recipient,
            eligible,
            progress,
            history_recorded,
            cancelled,
        };
        Flow::Yield
    }
}

/// 从 R 帧沿 parent 链找最近的 D 祖先（§3 谓词：最近 phaseDraw 祖先）。
fn nearest_draw_phase(ctx: &Ctx<'_>, from: EventId) -> Option<EventId> {
    let mut cur = ctx.res.frame(from)?.parent;
    while let Some(id) = cur {
        let f = ctx.res.frame(id)?;
        if matches!(f.data, EventData::DrawPhase { .. }) {
            return Some(id);
        }
        cur = f.parent;
    }
    None
}
