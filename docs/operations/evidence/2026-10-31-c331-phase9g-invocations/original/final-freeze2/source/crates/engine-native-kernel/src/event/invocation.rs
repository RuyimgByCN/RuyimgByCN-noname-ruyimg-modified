//! A named invocation owns its lifecycle; continuation stages own only operation receipts.
use super::*;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvocationPhase {
    Before,
    Begin,
    Body,
    End,
    After,
    Omitted,
}
impl InvocationPhase {
    pub fn tag(self) -> u8 {
        match self {
            Self::Before => 0,
            Self::Begin => 1,
            Self::Body => 2,
            Self::End => 3,
            Self::After => 4,
            Self::Omitted => 5,
        }
    }
    pub fn from_tag(t: u8) -> Option<Self> {
        Some(match t {
            0 => Self::Before,
            1 => Self::Begin,
            2 => Self::Body,
            3 => Self::End,
            4 => Self::After,
            5 => Self::Omitted,
            _ => return None,
        })
    }
    pub fn kind(self) -> EventKind {
        match self {
            Self::Before => EventKind::SkillBefore,
            Self::Begin => EventKind::SkillBegin,
            Self::Body => EventKind::SkillInvocation,
            Self::End => EventKind::SkillEnd,
            Self::After => EventKind::SkillAfter,
            Self::Omitted => EventKind::SkillOmitted,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvocationData {
    pub owner: PlayerId,
    pub def: SkillDefId,
    pub window: EventId,
    pub source: Option<PlayerId>,
    pub target: PlayerId,
    pub phase: InvocationPhase,
    pub cancelled: bool,
    pub jiding_card: Option<EventId>,
    pub judgement: Option<crate::catalog::Suit>,
    pub deferred_log: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeInvocationProgress {
    Ready,
    BeforeInvocation {
        context: InvocationData,
    },
    AwaitInvocation {
        child: EventId,
    },
    Finished,
    Body,
    HolderAsk {
        candidates: Vec<CardId>,
    },
    HolderPay {
        cards: Vec<CardId>,
    },
    HolderLoss {
        count: u32,
    },
    HolderAfterLog {
        count: u32,
    },
    SourceAsk {
        player: PlayerId,
        count: u32,
        candidates: Vec<CardId>,
    },
    SourceLoss,
    JidingCard {
        child: EventId,
    },
    YuanceAsk,
    JidianAsk {
        targets: Vec<PlayerId>,
    },
    AwaitJudge {
        child: EventId,
        target: PlayerId,
        purpose: JudgePurpose,
    },
    BodyDone,
}
pub fn uses_native_invocation(effect: NativeDamageEffect) -> bool {
    matches!(
        effect,
        NativeDamageEffect::Zhangwu
            | NativeDamageEffect::Jiding
            | NativeDamageEffect::Yuance
            | NativeDamageEffect::Jidian
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum YuanceOption {
    LoseHp,
    Recover,
}
impl YuanceOption {
    pub fn tag(self) -> u8 {
        match self {
            Self::LoseHp => 0,
            Self::Recover => 1,
        }
    }
    pub fn from_tag(t: u8) -> Option<Self> {
        match t {
            0 => Some(Self::LoseHp),
            1 => Some(Self::Recover),
            _ => None,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::LoseHp => "yuance_lose_hp",
            Self::Recover => "yuance_recover",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JudgePurpose {
    Yuance(YuanceOption),
    Jidian,
}
