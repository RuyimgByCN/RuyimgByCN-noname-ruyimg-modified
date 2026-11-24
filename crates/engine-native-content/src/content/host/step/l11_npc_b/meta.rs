//! Trigger metadata and per-skill limits for `L11` Part B (`l11_npc_b`).

use crate::catalog::{SkillLimits, TriggerMeta};
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

pub(in crate::content::host) fn limits_l11_npc_b(_name: &str) -> SkillLimits {
    // Note: `usable: 1` / `usable: X` / `limited: true` skills track exact phase/turn/game
    // state flags internally so multi-trigger skills (e.g., `twyuanhu`, `xinjiewei`,
    // `twjiefan`/`twjiefan2`) never suppress sibling windows via `SkillInstance::once_used`.
    SkillLimits::default()
}

pub(in crate::content::host) fn trigger_meta_l11_npc_b(
    name: &str,
    kind: EventKind,
) -> Option<TriggerMeta> {
    match (name, kind) {
        ("satomi_luodao", EventKind::CardUsed) => {
            Some(meta(EventKind::CardUsed, Timing::Normal, 0, true, Vec::new()))
        }
        ("sbliegong", EventKind::CardUsed) => Some(meta(
            EventKind::CardUsed,
            Timing::Normal,
            1,
            false,
            vec![EventKind::DamageBegin1],
        )),
        ("sbliegong", EventKind::DamageBegin1) => Some(meta(
            EventKind::DamageBegin1,
            Timing::Normal,
            0,
            false,
            vec![EventKind::CardUsed],
        )),
        ("shefu", EventKind::PhaseBegin) => Some(meta(
            EventKind::PhaseBegin,
            Timing::Normal,
            0,
            false,
            vec![EventKind::CardUsed],
        )),
        ("shefu", EventKind::CardUsed) => Some(meta(
            EventKind::CardUsed,
            Timing::Normal,
            2,
            false,
            vec![EventKind::PhaseBegin],
        )),
        ("shensu", EventKind::PhaseBegin) => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()))
        }
        ("shunshi", EventKind::DamageEnd) => Some(meta(
            EventKind::DamageEnd,
            Timing::Normal,
            0,
            false,
            vec![EventKind::PhaseBegin],
        )),
        ("shunshi", EventKind::PhaseBegin) => Some(meta(
            EventKind::PhaseBegin,
            Timing::Normal,
            0,
            false,
            vec![EventKind::DamageEnd],
        )),
        ("spshiji", EventKind::DamageBegin2) => {
            Some(meta(EventKind::DamageBegin2, Timing::Normal, 0, false, Vec::new()))
        }
        ("sptunjiang", EventKind::PhaseBegin) => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, true, Vec::new()))
        }
        ("spwenji", EventKind::PhaseBegin) => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()))
        }
        ("spxiangzhen", EventKind::CardUsed) => {
            Some(meta(EventKind::CardUsed, Timing::First, 10, false, Vec::new()))
        }
        ("spyishi", EventKind::DamageBegin2) => {
            Some(meta(EventKind::DamageBegin2, Timing::Normal, 0, true, Vec::new()))
        }
        ("tianming", EventKind::CardUsed) => {
            Some(meta(EventKind::CardUsed, Timing::Normal, 0, true, Vec::new()))
        }
        ("tianshu", EventKind::PhaseBegin) => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()))
        }
        ("tuifeng", EventKind::DamageEnd) => Some(meta(
            EventKind::DamageEnd,
            Timing::Normal,
            0,
            false,
            vec![EventKind::PhaseBegin],
        )),
        ("tuifeng", EventKind::PhaseBegin) => Some(meta(
            EventKind::PhaseBegin,
            Timing::Normal,
            0,
            false,
            vec![EventKind::DamageEnd],
        )),
        ("twfenming", EventKind::PhaseBegin) => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()))
        }
        ("twhanyu", EventKind::TurnBegin) => {
            Some(meta(EventKind::TurnBegin, Timing::First, 20, false, Vec::new()))
        }
        ("twhengjiang", EventKind::CardUsed) => {
            Some(meta(EventKind::CardUsed, Timing::Normal, 1, false, Vec::new()))
        }
        ("twhongju", EventKind::PhaseBegin) => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()))
        }
        ("twhuangjin", EventKind::CardUsed) => {
            Some(meta(EventKind::CardUsed, Timing::Normal, 3, false, Vec::new()))
        }
        ("twjianming", EventKind::CardUsed) => Some(meta(
            EventKind::CardUsed,
            Timing::Normal,
            0,
            false,
            vec![EventKind::CardResponded],
        )),
        ("twjianming", EventKind::CardResponded) => Some(meta(
            EventKind::CardResponded,
            Timing::Normal,
            0,
            false,
            vec![EventKind::CardUsed],
        )),
        ("twjiefan", EventKind::Dying) => {
            Some(meta(EventKind::Dying, Timing::Normal, 0, false, Vec::new()))
        }
        ("twjuntun", EventKind::TurnBegin) => Some(meta(
            EventKind::TurnBegin,
            Timing::First,
            15,
            false,
            vec![EventKind::Death, EventKind::DamageSource],
        )),
        ("twjuntun", EventKind::Death) => Some(meta(
            EventKind::Death,
            Timing::Normal,
            0,
            false,
            vec![EventKind::TurnBegin, EventKind::DamageSource],
        )),
        ("twjuntun", EventKind::DamageSource) => Some(meta(
            EventKind::DamageSource,
            Timing::Normal,
            0,
            false,
            vec![EventKind::TurnBegin, EventKind::Death],
        )),
        ("twxiongjun", EventKind::DamageSource) => {
            Some(meta(EventKind::DamageSource, Timing::Normal, -1, false, Vec::new()))
        }
        ("baonvezhi", EventKind::DamageEnd) => Some(meta(
            EventKind::DamageEnd,
            Timing::Normal,
            0,
            false,
            vec![EventKind::DamageSource],
        )),
        ("baonvezhi", EventKind::DamageSource) => Some(meta(
            EventKind::DamageSource,
            Timing::Normal,
            0,
            false,
            vec![EventKind::DamageEnd],
        )),
        ("twliancai", EventKind::PhaseBegin) => Some(meta(
            EventKind::PhaseBegin,
            Timing::Normal,
            0,
            true,
            vec![EventKind::TurnOvered],
        )),
        ("twliancai", EventKind::TurnOvered) => Some(meta(
            EventKind::TurnOvered,
            Timing::Normal,
            0,
            true,
            vec![EventKind::PhaseBegin],
        )),
        ("twlingfa", EventKind::CardUsed) => {
            Some(meta(EventKind::CardUsed, Timing::Normal, 0, false, Vec::new()))
        }
        ("twlinglu", EventKind::PhaseBegin) => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()))
        }
        ("twmibei", EventKind::CardUsed) => {
            Some(meta(EventKind::CardUsed, Timing::Normal, 5, false, Vec::new()))
        }
        ("twqianxi", EventKind::PhaseBegin) => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()))
        }
        ("twqirang", EventKind::Equipped) => Some(meta(
            EventKind::Equipped,
            Timing::Normal,
            0,
            true,
            vec![EventKind::CardUsed],
        )),
        ("twqirang", EventKind::CardUsed) => Some(meta(
            EventKind::CardUsed,
            Timing::Normal,
            1,
            false,
            vec![EventKind::Equipped],
        )),
        ("twqueshi", EventKind::TurnBegin) => {
            Some(meta(EventKind::TurnBegin, Timing::First, 18, false, Vec::new()))
        }
        ("twwuhun", EventKind::DamageEnd) => Some(meta(
            EventKind::DamageEnd,
            Timing::Normal,
            0,
            false,
            vec![EventKind::DamageSource, EventKind::Death],
        )),
        ("twwuhun", EventKind::DamageSource) => Some(meta(
            EventKind::DamageSource,
            Timing::Normal,
            0,
            false,
            vec![EventKind::DamageEnd, EventKind::Death],
        )),
        ("twwuhun", EventKind::Death) => Some(meta(
            EventKind::Death,
            Timing::Normal,
            0,
            false,
            vec![EventKind::DamageEnd, EventKind::DamageSource],
        )),
        ("twwushen", EventKind::CardUsed) => {
            Some(meta(EventKind::CardUsed, Timing::Normal, 2, false, Vec::new()))
        }
        ("twxiangyu", EventKind::CardUsed) => Some(meta(
            EventKind::CardUsed,
            Timing::Normal,
            1,
            false,
            vec![EventKind::CardsLost, EventKind::CardsMoved],
        )),
        ("twxiangyu", EventKind::CardsLost | EventKind::CardsMoved) => Some(meta(
            kind,
            Timing::Normal,
            1,
            false,
            vec![EventKind::CardUsed, EventKind::CardsLost, EventKind::CardsMoved],
        )),
        ("twyuhua", EventKind::CardsLost | EventKind::CardsMoved) => Some(meta(
            kind,
            Timing::Normal,
            0,
            true,
            vec![EventKind::CardsLost, EventKind::CardsMoved],
        )),
        ("meiyingqiang", EventKind::CardsLost | EventKind::CardsMoved) => Some(meta(
            kind,
            Timing::Normal,
            0,
            false,
            vec![EventKind::CardsLost, EventKind::CardsMoved],
        )),
        ("xiehui", EventKind::CardsMoved) => {
            Some(meta(EventKind::CardsMoved, Timing::Normal, 0, false, Vec::new()))
        }
        ("twxingqi", EventKind::PhaseBegin) => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()))
        }
        ("twxuanhuo", EventKind::PhaseEnd) => {
            Some(meta(EventKind::PhaseEnd, Timing::Normal, 0, false, Vec::new()))
        }
        ("twxuhe", EventKind::ShaDodged) => Some(meta(
            EventKind::ShaDodged,
            Timing::Normal,
            0,
            true,
            vec![EventKind::DamageBegin1],
        )),
        ("twxuhe", EventKind::DamageBegin1) => Some(meta(
            EventKind::DamageBegin1,
            Timing::Normal,
            0,
            false,
            vec![EventKind::ShaDodged],
        )),
        ("twyilie", EventKind::PhaseBegin) => Some(meta(
            EventKind::PhaseBegin,
            Timing::Normal,
            0,
            false,
            vec![EventKind::CardUsed, EventKind::ShaDodged],
        )),
        ("twyilie", EventKind::CardUsed) => Some(meta(
            EventKind::CardUsed,
            Timing::Normal,
            0,
            false,
            vec![EventKind::PhaseBegin, EventKind::ShaDodged],
        )),
        ("twyilie", EventKind::ShaDodged) => Some(meta(
            EventKind::ShaDodged,
            Timing::Normal,
            0,
            false,
            vec![EventKind::PhaseBegin, EventKind::CardUsed],
        )),
        ("twyingjia", EventKind::PhaseEnd) => {
            Some(meta(EventKind::PhaseEnd, Timing::Normal, 0, false, Vec::new()))
        }
        ("twyuanhu", EventKind::PhaseBegin) => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()))
        }
        ("twzhengrong", EventKind::DamageSource) => {
            Some(meta(EventKind::DamageSource, Timing::Normal, 0, false, Vec::new()))
        }
        ("wangzun", EventKind::PhaseBegin) => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, true, Vec::new()))
        }
        ("weifeng", EventKind::DamageBegin3) => {
            Some(meta(EventKind::DamageBegin3, Timing::Normal, 0, false, Vec::new()))
        }
        ("wulie", EventKind::PhaseBegin) => Some(meta(
            EventKind::PhaseBegin,
            Timing::Normal,
            0,
            false,
            vec![EventKind::DamageBegin3],
        )),
        ("wulie", EventKind::DamageBegin3) => Some(meta(
            EventKind::DamageBegin3,
            Timing::Normal,
            0,
            false,
            vec![EventKind::PhaseBegin],
        )),
        ("xianfu", EventKind::TurnBegin) => {
            Some(meta(EventKind::TurnBegin, Timing::First, 16, false, Vec::new()))
        }
        ("xinfu_daigong", EventKind::DamageBegin4) => {
            Some(meta(EventKind::DamageBegin4, Timing::Normal, 0, true, Vec::new()))
        }
        ("xinfu_fuyin", EventKind::CardUsed) => {
            Some(meta(EventKind::CardUsed, Timing::Normal, 4, false, Vec::new()))
        }
        ("xinfu_shajue", EventKind::Dying) => {
            Some(meta(EventKind::Dying, Timing::Normal, 7, false, Vec::new()))
        }
        ("xinfu_zengdao" | "xinfu_zengdao2", EventKind::DamageBegin1) => {
            Some(meta(EventKind::DamageBegin1, Timing::Normal, 0, false, Vec::new()))
        }
        ("xinfu_zhaoxin", EventKind::PhaseEnd) => {
            Some(meta(EventKind::PhaseEnd, Timing::Normal, 0, false, Vec::new()))
        }
        ("xinjiewei", EventKind::TurnOvered) => {
            Some(meta(EventKind::TurnOvered, Timing::Normal, 0, false, Vec::new()))
        }
        ("xinjushou", EventKind::PhaseBegin) => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, true, Vec::new()))
        }
        ("xinsheng", EventKind::DamageEnd) => {
            Some(meta(EventKind::DamageEnd, Timing::Normal, 0, true, Vec::new()))
        }
        ("yechou", EventKind::Death) => {
            Some(meta(EventKind::Death, Timing::Normal, 0, false, Vec::new()))
        }
        ("yinbing", EventKind::PhaseBegin) => Some(meta(
            EventKind::PhaseBegin,
            Timing::Normal,
            0,
            false,
            vec![EventKind::DamageEnd],
        )),
        ("yinbing", EventKind::DamageEnd) => Some(meta(
            EventKind::DamageEnd,
            Timing::Normal,
            0,
            false,
            vec![EventKind::PhaseBegin],
        )),
        ("juedi", EventKind::PhaseBegin) => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()))
        }
        ("yise", EventKind::DamageBegin1) => Some(meta(
            EventKind::DamageBegin1,
            Timing::Normal,
            0,
            false,
            vec![EventKind::CardsMoved],
        )),
        ("yise", EventKind::CardsMoved) => Some(meta(
            EventKind::CardsMoved,
            Timing::Normal,
            0,
            false,
            vec![EventKind::DamageBegin1],
        )),
        ("yongsi", EventKind::PhaseBegin) => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()))
        }
        ("yuce", EventKind::DamageEnd) => {
            Some(meta(EventKind::DamageEnd, Timing::Normal, 0, false, Vec::new()))
        }
        ("zhanyi", EventKind::ShaBegin) => {
            Some(meta(EventKind::ShaBegin, Timing::Normal, 0, false, Vec::new()))
        }
        ("zhendu", EventKind::PhaseBegin) => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()))
        }
        ("zhichi", EventKind::DamageEnd) => Some(meta(
            EventKind::DamageEnd,
            Timing::Normal,
            0,
            false,
            vec![EventKind::CardUsed],
        )),
        ("zhichi", EventKind::CardUsed) => Some(meta(
            EventKind::CardUsed,
            Timing::First,
            10,
            false,
            vec![EventKind::DamageEnd],
        )),
        ("zhidao", EventKind::DamageSource) => {
            Some(meta(EventKind::DamageSource, Timing::Normal, 0, false, Vec::new()))
        }
        ("zhiman", EventKind::DamageBegin2) => {
            Some(meta(EventKind::DamageBegin2, Timing::Normal, 0, true, Vec::new()))
        }
        ("zhiyan", EventKind::PhaseBegin) => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()))
        }
        ("zhongyong", EventKind::ShaDodged) => {
            Some(meta(EventKind::ShaDodged, Timing::Normal, 0, false, Vec::new()))
        }
        ("zhuide", EventKind::Death) => {
            Some(meta(EventKind::Death, Timing::Normal, 0, false, Vec::new()))
        }
        ("zili", EventKind::PhaseBegin) => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()))
        }
        ("zongkui", EventKind::TurnBegin) => {
            Some(meta(EventKind::TurnBegin, Timing::First, 14, false, Vec::new()))
        }
        ("xinfu_kannan", EventKind::DamageBegin1) => {
            Some(meta(EventKind::DamageBegin1, Timing::Normal, 0, false, Vec::new()))
        }
        ("twxuechang", EventKind::DamageBegin1) => {
            Some(meta(EventKind::DamageBegin1, Timing::Normal, 0, false, Vec::new()))
        }
        ("taipingyaoshu", EventKind::DamageBegin4) => {
            Some(meta(EventKind::DamageBegin4, Timing::Normal, 5, false, Vec::new()))
        }
        _ => None,
    }
}
