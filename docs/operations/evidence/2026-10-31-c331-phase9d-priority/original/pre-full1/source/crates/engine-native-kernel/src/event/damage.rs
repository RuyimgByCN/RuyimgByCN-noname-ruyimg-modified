//! Canonical damage quantity, submission receipt and sole continuation progress.
use super::*;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DamageOutcome {
    #[default]
    Running,
    Cancelled,
    ReplacedByLoseHp,
    Zero,
    Completed,
}
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DamageProgress {
    #[default]
    Start,
    Before,
    Begin,
    Begin1,
    Begin2,
    Begin3,
    Begin4,
    SubmitHp,
    HpCommitted {
        chain_eligible: bool,
    },
    TargetDamage {
        chain_eligible: bool,
    },
    DyingPending {
        chain_eligible: bool,
    },
    DyingDone {
        chain_eligible: bool,
    },
    Source {
        chain_eligible: bool,
    },
    End {
        chain_eligible: bool,
    },
    After {
        chain_eligible: bool,
    },
    Chain {
        remaining: Vec<PlayerId>,
    },
    Done,
    ZeroPending,
    CancelledPending,
    Stopped,
}
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DamageSettlement {
    pub committed_damage: Option<u32>,
    pub hp_loss: u32,
    pub outcome: DamageOutcome,
    pub notrigger: bool,
    pub progress: DamageProgress,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageWindowKind {
    Before,
    Begin,
    Begin1,
    Begin2,
    Begin3,
    Begin4,
    Target,
    Source,
    End,
    After,
    Zero,
    Cancelled,
}
impl DamageWindowKind {
    pub fn event_kind(self) -> EventKind {
        match self {
            Self::Before => EventKind::DamageBefore,
            Self::Begin => EventKind::DamageBegin,
            Self::Begin1 => EventKind::DamageBegin1,
            Self::Begin2 => EventKind::DamageBegin2,
            Self::Begin3 => EventKind::DamageBegin3,
            Self::Begin4 => EventKind::DamageBegin4,
            Self::Target => EventKind::Damage,
            Self::Source => EventKind::DamageSource,
            Self::End => EventKind::DamageEnd,
            Self::After => EventKind::DamageAfter,
            Self::Zero => EventKind::DamageZero,
            Self::Cancelled => EventKind::DamageCancelled,
        }
    }
    pub fn tag(self) -> u8 {
        match self {
            Self::Source => 0,
            Self::End => 1,
            Self::After => 2,
            Self::Zero => 3,
            Self::Cancelled => 4,
            Self::Before => 5,
            Self::Begin => 6,
            Self::Begin1 => 7,
            Self::Begin2 => 8,
            Self::Begin3 => 9,
            Self::Begin4 => 10,
            Self::Target => 11,
        }
    }
    pub fn from_tag(tag: u8) -> Option<Self> {
        Some(match tag {
            0 => Self::Source,
            1 => Self::End,
            2 => Self::After,
            3 => Self::Zero,
            4 => Self::Cancelled,
            5 => Self::Before,
            6 => Self::Begin,
            7 => Self::Begin1,
            8 => Self::Begin2,
            9 => Self::Begin3,
            10 => Self::Begin4,
            11 => Self::Target,
            _ => return None,
        })
    }
}
impl DamageSettlement {
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut out = b"native-damage-settlement/1".to_vec();
        out.push(u8::from(self.committed_damage.is_some()));
        out.extend_from_slice(&self.committed_damage.unwrap_or(0).to_le_bytes());
        out.extend_from_slice(&self.hp_loss.to_le_bytes());
        out.push(match self.outcome {
            DamageOutcome::Running => 0,
            DamageOutcome::Cancelled => 1,
            DamageOutcome::ReplacedByLoseHp => 2,
            DamageOutcome::Zero => 3,
            DamageOutcome::Completed => 4,
        });
        out.push(u8::from(self.notrigger));
        out.extend(self.progress.canonical_bytes());
        out
    }
}

/// A window carries only its checked ancestor identity; filters read the live canonical root.
pub fn damage_event<'a>(frames: &'a [Frame], frame: &'a Frame) -> &'a EventData {
    if let EventData::DamageWindow { root, .. } = frame.data {
        &frames
            .iter()
            .find(|f| f.id == root)
            .expect("checked damage ancestor")
            .data
    } else {
        &frame.data
    }
}

impl DamageProgress {
    pub fn tag(&self) -> u8 {
        match self {
            Self::Before => 0,
            Self::HpCommitted { .. } => 1,
            Self::TargetDamage { .. } => 2,
            Self::DyingPending { .. } => 3,
            Self::DyingDone { .. } => 4,
            Self::Source { .. } => 5,
            Self::End { .. } => 6,
            Self::After { .. } => 7,
            Self::Chain { .. } => 8,
            Self::Done => 9,
            Self::ZeroPending => 10,
            Self::Stopped => 11,
            Self::Start => 12,
            Self::Begin => 13,
            Self::Begin1 => 14,
            Self::Begin2 => 15,
            Self::Begin3 => 16,
            Self::Begin4 => 17,
            Self::SubmitHp => 18,
            Self::CancelledPending => 19,
        }
    }
    pub fn chain_eligible(&self) -> Option<bool> {
        match self {
            Self::HpCommitted { chain_eligible }
            | Self::TargetDamage { chain_eligible }
            | Self::DyingPending { chain_eligible }
            | Self::DyingDone { chain_eligible }
            | Self::Source { chain_eligible }
            | Self::End { chain_eligible }
            | Self::After { chain_eligible } => Some(*chain_eligible),
            _ => None,
        }
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut out = vec![self.tag()];
        if let Some(eligible) = self.chain_eligible() {
            out.push(u8::from(eligible));
        }
        if let Self::Chain { remaining } = self {
            out.extend_from_slice(&(remaining.len() as u32).to_le_bytes());
            out.extend_from_slice(remaining);
        }
        out
    }
    pub fn is_pre_hp(&self) -> bool {
        matches!(
            self,
            Self::Start
                | Self::Before
                | Self::Begin
                | Self::Begin1
                | Self::Begin2
                | Self::Begin3
                | Self::Begin4
                | Self::SubmitHp
        )
    }
    pub fn window(&self) -> Option<DamageWindowKind> {
        use DamageWindowKind as W;
        Some(match self {
            Self::Before => W::Before,
            Self::Begin => W::Begin,
            Self::Begin1 => W::Begin1,
            Self::Begin2 => W::Begin2,
            Self::Begin3 => W::Begin3,
            Self::Begin4 => W::Begin4,
            Self::TargetDamage { .. } => W::Target,
            Self::Source { .. } => W::Source,
            Self::End { .. } => W::End,
            Self::After { .. } => W::After,
            _ => return None,
        })
    }
}
