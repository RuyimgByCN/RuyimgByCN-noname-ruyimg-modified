//! N10 权威状态与可见视图分离（ADR 0730）。
//!
//! `PlayerView` 是权威状态（`GameStateCore`）的**只读投影**：每个座位只能
//! 看到自己的手牌与所有公开信息（装备/判定区/弃牌堆/牌桌面/体力/回合）；
//! 对手手牌与牌库内容一律不可见（仅数量）。视图构造为纯投影，不修改状态、
//! 不进入快照与摘要。

use crate::event::Phase;
use crate::ids::{CardId, PlayerId};
use crate::state::Role;
use crate::GameAction;
use crate::state::{Winner};
use crate::machine::LogEvent;

/// 手牌可见性：仅本人可见具体牌，他人只见数量。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandView {
    /// 自己（或公开）的手牌：具体牌 id。
    Own(Vec<CardId>),
    /// 不可见（对手手牌）：仅 `hand_count` 有意义。
    Hidden,
}

/// 明置武将的可见单元（N11 国战门控）：名称与势力。
/// 暗置槽不产生本单元——他人视图不可见（信息隔离，ADR 0757）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneralView {
    pub name: &'static str,
    pub faction: crate::catalog::Faction,
}

/// 单个座位的可见单元。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerCellView {
    pub seat: PlayerId,
    pub alive: bool,
    pub hp: i32,
    pub max_hp: u32,
    /// 自己的手牌（具体牌 id）；他人为 `Hidden`（仅数量）。
    pub hand: HandView,
    pub hand_count: usize,
    /// 装备区：公开。
    pub equips: Vec<CardId>,
    /// 判定区：公开。
    pub judge: Vec<CardId>,
    /// 双将槽视图（N11 国战，非国战局两槽恒 None）：
    /// 本座位两槽可见；他人仅明置槽可见，暗置槽为 `None`（不泄将领与势力）。
    pub generals: [Option<GeneralView>; 2],
    /// 明置状态：公开信息（明置行为本身可见）。
    pub revealed_generals: [bool; 2],
    /// 翻面状态：公开信息（T13 客户端深字段，批 19 起投影）。
    pub turned_over: bool,
    /// 已授技能名（公开信息；T13 起投影，排序稳定）。
    pub skills: Vec<&'static str>,
    /// 身份（仅身份局有值）：本人视角、主公（公开）与已死亡（遗言公开，
    /// 上游 `dieAfter`）成员可见；其余存活非主公成员的他人身份为 `None`
    /// （信息隔离）。非身份局恒 `None`。
    pub role: Option<Role>,
}

/// 单个座位的完整可见视图。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerView {
    /// 视角座位（观战者为其视图内不含任何手牌 id）。
    pub seat: PlayerId,
    pub players: Vec<PlayerCellView>,
    /// 牌库仅可见数量，不可见内容与顺序。
    pub deck_count: usize,
    /// 弃牌堆：公开。
    pub discard: Vec<CardId>,
    /// 牌桌面（处理中的牌）：公开。
    pub table: Vec<CardId>,
    pub turn: PlayerId,
    pub phase: Phase,
    pub round: u32,
    pub outcome: Option<Winner>,
    pub ruleset_version: u64,
}

/// 开放选择的座位可见性（N10 第二批）：
/// 询问的存在与类型是公开信息（众人可见有人被询问什么）；
/// **选项内容仅观察者本人可见**——选项会暴露应答者手牌（桃/闪）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChoiceView {
    /// 观察者本人：完整选项。
    Own {
        id: u64,
        prompt: &'static str,
        options: Vec<crate::choice::ChoiceOption>,
        min: usize,
        max: usize,
        allow_cancel: bool,
    },
    /// 他人/观战者：仅知道询问的存在、观察者与类型，**不含任何选项**。
    Other {
        id: u64,
        observer: PlayerId,
        prompt: &'static str,
    },
}

/// 日志事件的座位可见性（N10 第二批）：
/// 摸牌为暗置信息——他人摸到的牌 id 不可见（仅数量）；其余事件公开。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventView {
    /// 公开事件原样。
    Public(crate::machine::LogEvent),
    /// 他人摸牌：牌 id 隐藏，仅数量。
    CardsDrawnHidden { player: PlayerId, count: usize },
    /// 他人的私有选择应答（N10-a）：仅保留询问 id，
    /// 选项内容（含牌/目标 id）对非观察者不透明。
    ChoiceAnsweredHidden { id: u64 },
}

impl PlayerView {
    /// 收集视图中出现的全部牌 id（用于信息隔离验收：与隐藏集合求交）。
    pub fn visible_card_ids(&self) -> Vec<CardId> {
        let mut ids = Vec::new();
        for p in &self.players {
            if let HandView::Own(hand) = &p.hand {
                ids.extend(hand.iter().copied());
            }
            ids.extend(p.equips.iter().copied());
            ids.extend(p.judge.iter().copied());
        }
        ids.extend(self.discard.iter().copied());
        ids.extend(self.table.iter().copied());
        ids
    }
}

/// 座位策略（P2/N10，ADR 0772）：类型上只能看到 [`PolicyInput`]——
/// 权威状态入口（`&Engine`）不可达。返回 `None` 表示无动作
/// （驱动器回退为推进阶段）。
pub trait SeatPolicy {
    fn act(&mut self, input: &PolicyInput) -> Option<GameAction>;
}

/// AI 只读视角会话（N10）：座位固定的可见性投影入口——
/// 仅暴露 PlayerView/ChoiceView/EventView 与配额余量，
/// 不提供权威状态入口（类型系统约束：字段私有，构造经 `Engine::ai_session`）。
pub struct AiSession<'a> {
    pub(crate) engine: &'a crate::engine::Engine,
    pub(crate) seat: PlayerId,
}

/// 座位策略输入（P2/N10，ADR 0772）：类型上不含权威引擎入口——
/// 仅视图、询问投影与公开协议数据（修订号/配额余量）。
#[derive(Debug, Clone)]
pub struct PolicyInput {
    pub view: PlayerView,
    pub choice: Option<ChoiceView>,
    /// 提交命令所需的当前修订号（公开协议数据）。
    pub revision: u64,
    /// 本座位本轮剩余命令配额（None = 不限制）。
    pub quota_remaining: Option<u32>,
    /// 本座位手牌的（牌 id, 类别）——仅自己可见的信息。
    pub own_hand_kinds: Vec<(CardId, crate::catalog::CardKind)>,
}

impl<'a> AiSession<'a> {
    pub fn seat(&self) -> PlayerId {
        self.seat
    }

    /// 组装策略输入（视图 + 询问 + 协议数据 + 自己手牌类别）。
    pub fn input(&self) -> PolicyInput {
        let view = self.view();
        let own_hand_kinds = match (&view.players[self.seat as usize].hand, self.engine.catalog()) {
            (HandView::Own(hand), cat) => hand
                .iter()
                .map(|c| (*c, cat.cards[*c as usize].kind))
                .collect(),
            _ => Vec::new(),
        };
        PolicyInput {
            view,
            choice: self.choice(),
            revision: self.engine.revision(),
            quota_remaining: self.quota_remaining(),
            own_hand_kinds,
        }
    }

    /// 本座位可见状态视图。
    pub fn view(&self) -> PlayerView {
        self.engine.view_for(self.seat)
    }

    /// 本座位可见的开放选择（Own 完整选项 / Other 仅询问存在）。
    pub fn choice(&self) -> Option<ChoiceView> {
        self.engine.choice_view_for(Some(self.seat))
    }

    /// 本座位可见的事件流（他人摸牌仅数量）。
    pub fn events(&self) -> Vec<EventView> {
        self.engine.event_view_for(self.seat)
    }

    /// 本轮剩余配额（未设置配额返回 None）。
    pub fn quota_remaining(&self) -> Option<u32> {
        self.engine.seat_quota_remaining(self.seat)
    }
}

/// 公开事件粗粒度码（与旧线 `public_codes` / 前端 `PublicEvent` 标签一致）。
///
/// 只按**事件种类**投影，不携带牌 id、选项内容或暗牌信息；因此可以直接
/// 提供给所有观察者。未知/内部事件返回 `None`，不伪造“对局开始/结束”。
pub fn public_event_code(event: &LogEvent) -> Option<u16> {
    match event {
        LogEvent::CardsDrawn { .. } => Some(2),
        LogEvent::CardPlayed { .. } | LogEvent::CardResponded { .. } => Some(3),
        LogEvent::DamageDealt { .. } | LogEvent::DamagePrevented { .. } => Some(4),
        LogEvent::HpRecovered { .. } => Some(5),
        LogEvent::DyingEntered { .. } | LogEvent::DyingSaved { .. } => Some(6),
        LogEvent::PlayerDied { .. } => Some(7),
        LogEvent::PhaseChanged { .. } | LogEvent::TurnEnded { .. } => Some(8),
        LogEvent::ChoiceOpened { .. } | LogEvent::ChoiceAnswered { .. } => Some(10),
        _ => None,
    }
}
