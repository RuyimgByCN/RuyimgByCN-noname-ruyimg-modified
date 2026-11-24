//! Trigger metadata for `HostBehavior::L11VmFixC` (`P3-E3`).

use crate::catalog::TriggerMeta;
use crate::event::{EventKind, Timing};

fn meta(
    event: EventKind,
    timing: Timing,
    priority: i32,
    optional: bool,
    extra_events: Vec<EventKind>,
) -> TriggerMeta {
    TriggerMeta {
        event,
        timing,
        priority,
        order_selectable: optional,
        optional,
        extra_events,
    }
}

pub(super) fn trigger_meta_l11_vm_c(name: &str, kind: EventKind) -> Option<TriggerMeta> {
    match (name, kind) {
        ("baonu", EventKind::DamageSource | EventKind::DamageEnd | EventKind::TurnBegin) => {
            let mut extra = vec![
                EventKind::DamageSource,
                EventKind::DamageEnd,
                EventKind::TurnBegin,
            ];
            extra.retain(|&e| e != kind);
            Some(meta(kind, Timing::Normal, 0, false, extra))
        }
        ("chongzhen", EventKind::CardUsed | EventKind::CardResponded) => {
            let other = if kind == EventKind::CardUsed {
                EventKind::CardResponded
            } else {
                EventKind::CardUsed
            };
            Some(meta(kind, Timing::Normal, 0, false, vec![other]))
        }
        ("dujin", EventKind::DrawPhaseBegin2) => {
            Some(meta(EventKind::DrawPhaseBegin2, Timing::Normal, -2, false, Vec::new()))
        }
        ("gnjinfan", EventKind::PhaseBegin) => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()))
        }
        ("juece", EventKind::CardsLost | EventKind::CardsDiscarded) => {
            let other = if kind == EventKind::CardsLost {
                EventKind::CardsDiscarded
            } else {
                EventKind::CardsLost
            };
            Some(meta(kind, Timing::Normal, 0, false, vec![other]))
        }
        ("juejing", EventKind::DrawPhaseBegin2) => {
            Some(meta(EventKind::DrawPhaseBegin2, Timing::Normal, -5, false, Vec::new()))
        }
        ("mieji", EventKind::CardUsed) => {
            Some(meta(EventKind::CardUsed, Timing::Last, 0, false, Vec::new()))
        }
        ("nsfeixiong", EventKind::PhaseBegin) => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()))
        }
        ("nsyice", EventKind::CardsDiscarded) => {
            Some(meta(EventKind::CardsDiscarded, Timing::Normal, 0, false, Vec::new()))
        }
        ("nzry_jieying", EventKind::PhaseBegin | EventKind::TurnBegin) => {
            let other = if kind == EventKind::PhaseBegin {
                EventKind::TurnBegin
            } else {
                EventKind::PhaseBegin
            };
            Some(meta(kind, Timing::Normal, 0, false, vec![other]))
        }
        ("qiaoshui", EventKind::PhaseBegin) => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()))
        }
        ("qiaoshui3", EventKind::CardUsed) => {
            Some(meta(EventKind::CardUsed, Timing::Last, 0, false, Vec::new()))
        }
        ("ranshang", EventKind::DamageEnd) => {
            Some(meta(EventKind::DamageEnd, Timing::Normal, 0, false, Vec::new()))
        }
        ("ranshang2", EventKind::PhaseBegin) => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()))
        }
        ("redangxian", EventKind::TurnBegin) => {
            Some(meta(EventKind::TurnBegin, Timing::Normal, 0, false, Vec::new()))
        }
        ("rejizhi", EventKind::CardUsed) => {
            Some(meta(EventKind::CardUsed, Timing::Normal, 1, false, Vec::new()))
        }
        ("rekuangcai", EventKind::PhaseBegin) => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()))
        }
        ("reshanxi", EventKind::PhaseBegin) => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()))
        }
        ("reshanxi2", EventKind::PhaseEnd) => {
            Some(meta(EventKind::PhaseEnd, Timing::Normal, 0, false, Vec::new()))
        }
        ("rezhenxing", EventKind::DamageEnd | EventKind::PhaseBegin) => {
            let other = if kind == EventKind::DamageEnd {
                EventKind::PhaseBegin
            } else {
                EventKind::DamageEnd
            };
            Some(meta(kind, Timing::Normal, 0, false, vec![other]))
        }
        ("shelie", EventKind::DrawPhaseBegin1) => {
            Some(meta(EventKind::DrawPhaseBegin1, Timing::Normal, 5, true, Vec::new()))
        }
        ("shibei", EventKind::DamageEnd) => {
            Some(meta(EventKind::DamageEnd, Timing::Normal, 0, false, Vec::new()))
        }
        ("anjian", EventKind::DamageBegin1) => {
            Some(meta(EventKind::DamageBegin1, Timing::Normal, 0, false, Vec::new()))
        }
        ("xinkuanggu", EventKind::DamageSource) => {
            Some(meta(EventKind::DamageSource, Timing::Normal, 0, false, Vec::new()))
        }
        ("xinniluan", EventKind::PhaseBegin) => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()))
        }
        ("zhuikong", EventKind::PhaseBegin) => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()))
        }
        _ => None,
    }
}
