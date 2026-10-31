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
    Before,
    HpCommitted,
    TargetDamage,
    DyingPending,
    DyingDone,
    Source,
    End,
    After,
    Chain {
        remaining: Vec<PlayerId>,
    },
    Done,
    ZeroPending,
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
    Source,
    End,
    After,
    Zero,
    Cancelled,
}
impl DamageWindowKind {
    pub fn event_kind(self) -> EventKind {
        match self {
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
        }
    }
    pub fn from_tag(tag: u8) -> Option<Self> {
        Some(match tag {
            0 => Self::Source,
            1 => Self::End,
            2 => Self::After,
            3 => Self::Zero,
            4 => Self::Cancelled,
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
        let (tag, remaining) = match &self.progress {
            DamageProgress::Before => (0, None),
            DamageProgress::HpCommitted => (1, None),
            DamageProgress::TargetDamage => (2, None),
            DamageProgress::DyingPending => (3, None),
            DamageProgress::DyingDone => (4, None),
            DamageProgress::Source => (5, None),
            DamageProgress::End => (6, None),
            DamageProgress::After => (7, None),
            DamageProgress::Chain { remaining } => (8, Some(remaining)),
            DamageProgress::Done => (9, None),
            DamageProgress::ZeroPending => (10, None),
            DamageProgress::Stopped => (11, None),
        };
        out.push(tag);
        if let Some(xs) = remaining {
            out.extend_from_slice(&(xs.len() as u32).to_le_bytes());
            out.extend_from_slice(xs);
        }
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
