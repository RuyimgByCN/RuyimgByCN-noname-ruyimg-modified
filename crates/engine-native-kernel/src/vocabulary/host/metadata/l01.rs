use super::super::*;

impl HostBehavior {
    /// L01 `AI优化` 宿主行为触发元数据。
    pub(super) fn trigger_meta_l01(&self) -> Option<TriggerMeta> {
        match self {
            HostBehavior::L01YiyiCard
            | HostBehavior::L01NeiKey
            | HostBehavior::L01AiRuleHooks => None,
            HostBehavior::L01FirstKs => Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 100,
                order_selectable: false,
                optional: false,
                extra_events: vec![EventKind::Death],
            }),
            HostBehavior::L01AttitudeTracker => Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: -50,
                order_selectable: false,
                optional: false,
                extra_events: vec![
                    EventKind::Damage,
                    EventKind::LoseHp,
                    EventKind::Recover,
                    EventKind::Death,
                ],
            }),
            _ => None,
        }
    }
}
