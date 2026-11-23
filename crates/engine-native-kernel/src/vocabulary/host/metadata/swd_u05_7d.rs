//! U05-7d 杀海拾遗 swd 补充宿主技能触发元数据（tag 1370..=1386）。
use super::super::*;

impl HostBehavior {
    pub(super) fn trigger_meta_swd_u05_7d(&self) -> Option<TriggerMeta> {
        match self {
            HostBehavior::SwdDuijue => None,
            HostBehavior::SwdDuijueOut => Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: false,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::SwdDuijueTrigger => Some(TriggerMeta {
                event: EventKind::TurnPhaseAfter,
                timing: Timing::Normal,
                priority: -50,
                order_selectable: false,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::SwdTanlin => None,
            HostBehavior::SwdTanlinTarget | HostBehavior::SwdTanlinOwner => Some(TriggerMeta {
                event: EventKind::TurnPhaseAfter,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: false,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::SwdTanlinRange => None,
            HostBehavior::SwdTanlinDefence => Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::SwdOldLiaoyuan | HostBehavior::SwdOldLiaoyuan2 => None,
            HostBehavior::SwdHuanling2 => Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: false,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::SwdLexue2 => Some(TriggerMeta {
                event: EventKind::TurnPhaseEnd,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: false,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::SwdTianshuRemove => Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 1,
                order_selectable: false,
                optional: false,
                extra_events: Vec::new(),
            }),
            HostBehavior::SwdYinguo2 | HostBehavior::SwdSusheng2 => None,
            HostBehavior::SwdMoyu => Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: true,
                extra_events: Vec::new(),
            }),
            HostBehavior::SwdU05_7dSub => None,
            _ => None,
        }
    }
}
