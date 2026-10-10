//! 事件模型：显式事件栈的帧。
//!
//! 与旧实现的差别：事件嵌套是一棵真实的栈（父/子/返回值都在帧上），
//! 不再是「单槽恢复 + 三个延迟队列」；帧载荷是类型化的 `EventData`，
//! 技能可以在结算窗口内改写帧载荷（如伤害转移）。

use crate::ids::{CardId, EventId, PlayerId, SkillDefId};

/// 触发唤醒用的事件类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EventKind {
    DamageBefore,
    Damage,
    Recover,
    CardsDrawn,
    CardsMoved,
    CardUsed,
    CardResponded,
    Dying,
    Death,
    PhaseBegin,
    PhaseEnd,
    TurnEnded,
    SkillToggled,
    Equipped,
    Linked,
    CardsDiscarded,
    TurnOverBefore,
    TurnOvered,
    LoseHp,
    /// 濒死救回（U05 xianjian 焚逝挂点，规则 v157）：固有救援机在
    /// 体力>0 且脱离濒死时发射（上游 dyingAfter 时序等价）。
    DyingSaved,
}

/// 伤害属性：普通 / 火 / 雷。属性伤害触发铁索连环传导。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageNature {
    Normal,
    Fire,
    Thunder,
}

/// 伤害事件载荷：结算窗口内可被技能改写（转移/防止）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DamageData {
    pub source: Option<PlayerId>,
    /// 当前承受者——转移技能改写此字段。
    pub target: PlayerId,
    pub amount: u32,
    pub nature: DamageNature,
    /// 防止类技能改写此字段；提交阶段读到 true 则不扣血。
    pub prevented: bool,
    /// 转移链记录：原始承受者 -> …… -> 最终承受者。
    pub redirect_chain: Vec<PlayerId>,
    /// 造成伤害的牌（奸雄/遗计类技能依赖；非牌类伤害为 None）。
    pub card: Option<CardId>,
    /// 铁索连环传导伤害（ow 源氏飞刃 notLink 过滤依赖；主伤害为
    /// false——上游 event.notLink() 同口径，规则 v129）。
    pub from_chain: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardUseData {
    pub card: CardId,
    pub player: PlayerId,
    pub targets: Vec<PlayerId>,
    /// 杀的属性（仅 Sha/FireSha 有意义）。
    pub nature: DamageNature,
    /// view-as 转化声明（武圣/龙胆）：以此牌种结算而非目录牌种。
    pub use_as: Option<crate::catalog::CardKind>,
    /// E03 使用前取消（猴子偷桃）：确认后由窗口机器执行夺桃并把本位
    /// 置真；提交阶段读到真值即跳过结算。不入 canonical_hash（取消的
    /// 最终状态——牌的位置与体力——已可观测）。
    pub cancelled: bool,
}

/// 类型化事件载荷。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventData {
    /// E03 翻面前置窗口（漩涡面具等锁定技可取消）。
    TurnOverBefore {
        player: PlayerId,
        cancelled: bool,
    },
    /// E03 翻面完成。
    TurnOvered {
        player: PlayerId,
        face_down: bool,
    },
    Damage(DamageData),
    Recover {
        player: PlayerId,
        amount: u32,
        by_card: Option<CardId>,
    },
    CardsDrawn {
        player: PlayerId,
        cards: Vec<CardId>,
    },
    CardsMoved {
        cards: Vec<CardId>,
        from: PlayerId,
        to: PlayerId,
    },
    CardUsed(CardUseData),
    CardResponded {
        card: CardId,
        player: PlayerId,
    },
    Dying {
        player: PlayerId,
        source: Option<PlayerId>,
    },
    Death {
        player: PlayerId,
        killer: Option<PlayerId>,
    },
    Phase {
        player: PlayerId,
        phase: Phase,
        begin: bool,
    },
    TurnEnded {
        player: PlayerId,
    },
    SkillToggled {
        player: PlayerId,
        skill: SkillDefId,
        enabled: bool,
        /// 是否临时获得、回合结束移除。
        temporary: bool,
    },
    /// 装备一张装备牌；同栏旧牌被替换入弃牌堆。
    Equipped {
        player: PlayerId,
        card: CardId,
        /// 同栏被替换下的旧牌（若有）。
        replaced: Option<CardId>,
    },
    /// 连环状态切换（铁索连环横置/重置）。
    Linked {
        player: PlayerId,
        linked: bool,
    },
    /// 弃牌（弃牌阶段批量弃置等；供连营类「失去手牌」触发）。
    CardsDiscarded {
        player: PlayerId,
        cards: Vec<CardId>,
    },
    /// 失去体力（逐技对照批三 v77）：直接扣减体力——不经护甲、不触发
    /// 伤害类窗口/铁索/伤害来源技；体力 ≤0 进入濒死（上游 loseHp）。
    LoseHp {
        player: PlayerId,
        amount: u32,
    },
    /// 濒死救回（dyingAfter 等价，见 EventKind::DyingSaved）。
    DyingSaved {
        player: PlayerId,
        source: Option<PlayerId>,
    },
}

impl EventData {
    pub fn kind(&self) -> EventKind {
        match self {
            EventData::Damage(_) => EventKind::Damage,
            EventData::Recover { .. } => EventKind::Recover,
            EventData::CardsDrawn { .. } => EventKind::CardsDrawn,
            EventData::CardsMoved { .. } => EventKind::CardsMoved,
            EventData::CardUsed(_) => EventKind::CardUsed,
            EventData::CardResponded { .. } => EventKind::CardResponded,
            EventData::Dying { .. } => EventKind::Dying,
            EventData::Death { .. } => EventKind::Death,
            EventData::Phase { begin: true, .. } => EventKind::PhaseBegin,
            EventData::Phase { begin: false, .. } => EventKind::PhaseEnd,
            EventData::TurnEnded { .. } => EventKind::TurnEnded,
            EventData::SkillToggled { .. } => EventKind::SkillToggled,
            EventData::Equipped { .. } => EventKind::Equipped,
            EventData::Linked { .. } => EventKind::Linked,
            EventData::CardsDiscarded { .. } => EventKind::CardsDiscarded,
            EventData::TurnOverBefore { .. } => EventKind::TurnOverBefore,
            EventData::TurnOvered { .. } => EventKind::TurnOvered,
            EventData::LoseHp { .. } => EventKind::LoseHp,
            EventData::DyingSaved { .. } => EventKind::DyingSaved,
        }
    }

    /// DamageBefore 与 Damage 共用 Damage 载荷。
    pub fn trigger_kind(&self, before: bool) -> EventKind {
        match self {
            EventData::Damage(_) if before => EventKind::DamageBefore,
            other => other.kind(),
        }
    }
}

/// 回合阶段。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Phase {
    Judge,
    Draw,
    Play,
    Discard,
    End,
}

impl Phase {
    pub const ALL: [Phase; 5] = [
        Phase::Judge,
        Phase::Draw,
        Phase::Play,
        Phase::Discard,
        Phase::End,
    ];

    pub fn next(self) -> Option<Phase> {
        Some(match self {
            Phase::Judge => Phase::Draw,
            Phase::Draw => Phase::Play,
            Phase::Play => Phase::Discard,
            Phase::Discard => Phase::End,
            Phase::End => return None,
        })
    }

    pub fn name(&self) -> &'static str {
        match self {
            Phase::Judge => "judge",
            Phase::Draw => "draw",
            Phase::Play => "play",
            Phase::Discard => "discard",
            Phase::End => "end",
        }
    }
}

/// 触发时机带（镜像非官方 first/normal/last 三段）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Timing {
    First,
    Normal,
    Last,
}

/// 已入队待运行的触发项。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueuedTrigger {
    pub def: SkillDefId,
    pub owner: PlayerId,
    /// 完全序键：Timing -> Priority 降 -> 可选后置 -> 座位相对 -> 定义号。
    pub order: (u8, i32, u8, u8, u16),
}

/// 帧所处的管线阶段。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    /// 伤害前置触发窗口。
    BeforeTriggers,
    /// 内核提交动作（扣血/摸牌/移牌等）。
    Commit,
    /// 提交后触发窗口。
    AfterTriggers,
    /// 内核固有流程机器（濒死救援/出牌结算/弃牌选择等）。
    Intrinsic,
    /// 已完成，待弹栈。
    Done,
}

/// 事件栈上的一帧。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub id: EventId,
    pub parent: Option<EventId>,
    pub kind: EventKind,
    pub data: EventData,
    pub stage: Stage,
    pub queue: Vec<QueuedTrigger>,
    /// 当前阶段的触发队列是否已收集。
    pub planned: bool,
    pub intrinsic_done: bool,
}

impl Frame {
    pub fn damage(&self) -> &DamageData {
        match &self.data {
            EventData::Damage(d) => d,
            other => panic!("frame {:?} is not damage: {other:?}", self.kind),
        }
    }

    pub fn damage_mut(&mut self) -> &mut DamageData {
        match &mut self.data {
            EventData::Damage(d) => d,
            other => panic!("frame {other:?} is not damage"),
        }
    }

    /// TurnOverBefore 帧载荷可变访问（漩涡面具取消写回）。
    pub fn turn_over_before_mut(&mut self) -> (&mut PlayerId, &mut bool) {
        match &mut self.data {
            EventData::TurnOverBefore { player, cancelled } => (player, cancelled),
            other => panic!("frame {other:?} is not turn_over_before"),
        }
    }

    /// CardUsed 帧载荷可变访问（猴子偷桃取消写回）。
    pub fn card_use_mut(&mut self) -> &mut CardUseData {
        match &mut self.data {
            EventData::CardUsed(u) => u,
            other => panic!("frame {other:?} is not card_use"),
        }
    }

    /// CardUsed 帧载荷只读访问（同疾读目标；非 CardUsed 帧返回 None）。
    pub fn card_use(&self) -> Option<&CardUseData> {
        match &self.data {
            EventData::CardUsed(u) => Some(u),
            _ => None,
        }
    }
}
