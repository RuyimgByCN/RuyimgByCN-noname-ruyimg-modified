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
    pub jiding_changes: Vec<JidingCardChange>,
    pub judgement: Option<crate::catalog::Suit>,
    pub deferred_log: bool,
    pub payment: Option<InvocationPayment>,
    pub judge_receipt: Option<InvocationJudgeReceipt>,
    pub effect_applied: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JidingCardChange {
    pub card: EventId,
    pub def: SkillDefId,
    pub actor: PlayerId,
    pub before_player: PlayerId,
    pub before_target: PlayerId,
    pub cost: JidingCardCost,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JidingCardCost {
    Zhenlie { lose_hp: EventId },
    TongJi { card: CardId, loss: Option<EventId> },
    Yihua { cards: Vec<CardId>, losses: Vec<EventId> },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeInvocationProgress {
    Ready,
    BeforeInvocation {
        context: InvocationData,
    },
    AwaitInvocation {
        child: EventId,
        admission: InvocationData,
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
    QinglongAsk {
        target: PlayerId,
        hand: Vec<CardId>,
        equip: Vec<CardId>,
    },
    QinglongLog {
        target: PlayerId,
        card: CardId,
        public: bool,
    },
    QinglongLoss {
        target: PlayerId,
        card: CardId,
        public: bool,
    },
    BodyDone,
    JidianAfterLog {
        target: PlayerId,
    },
}
pub fn uses_native_invocation(effect: NativeDamageEffect) -> bool {
    matches!(
        effect,
        NativeDamageEffect::Qinglong
            | NativeDamageEffect::Zhangwu
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvocationPayment {
    pub selected: Vec<CardId>,
    pub log_begin: EventId,
    pub committed: bool,
    pub loss_children: Vec<EventId>,
    pub after_log: Option<EventId>,
}

pub fn qinglong_options(
    owner: PlayerId,
    target: PlayerId,
    hand: &[CardId],
    equip: &[CardId],
) -> Vec<crate::choice::ChoiceOption> {
    use crate::choice::ChoiceOption as O;
    hand.iter()
        .enumerate()
        .map(|(i, c)| {
            if owner == target {
                O::Card(*c)
            } else {
                O::HiddenCard(i as u32)
            }
        })
        .chain(equip.iter().copied().map(O::Card))
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvocationJudgeReceipt {
    pub child: EventId,
    pub target: PlayerId,
    pub purpose: JudgePurpose,
}
