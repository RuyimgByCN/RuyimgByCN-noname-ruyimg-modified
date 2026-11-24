//! Phase 3 P3-C1a `L11` 跨包 NPC Part A（80 + 3 技）触发元数据与次数上限声明。

use crate::catalog::{SkillLimits, TriggerMeta};
use crate::event::{EventKind, Timing};

pub(in crate::content::host) fn trigger_meta_l11_npc_a(name: &str) -> Option<TriggerMeta> {
    let m = |event: EventKind, timing: Timing, priority: i32, optional: bool, extra_events| {
        Some(TriggerMeta {
            event,
            timing,
            priority,
            order_selectable: false,
            optional,
            extra_events,
        })
    };
    match name {
        // Dying / Death / CardsLost / Recover / TurnOvered
        "buqu" => m(EventKind::Dying, Timing::Normal, 6, false, Vec::new()),
        "chenqing" => m(EventKind::Dying, Timing::Normal, 0, true, Vec::new()),
        "olniepan" => m(EventKind::Dying, Timing::Normal, 0, false, Vec::new()),
        "rexingshang" => m(EventKind::Death, Timing::Normal, 0, true, Vec::new()),
        "dualside" => m(EventKind::TurnOvered, Timing::Normal, 0, false, Vec::new()),
        "fenji" | "retuntian" => m(
            EventKind::CardsLost,
            Timing::Normal,
            0,
            false,
            vec![EventKind::CardsDiscarded],
        ),
        "lirang" => m(
            EventKind::CardsLost,
            Timing::Normal,
            0,
            true,
            vec![EventKind::CardsDiscarded],
        ),
        "relianying" => m(
            EventKind::CardsLost,
            Timing::Normal,
            0,
            true,
            vec![
                EventKind::CardUsed,
                EventKind::CardsMoved,
                EventKind::CardsDiscarded,
            ],
        ),
        "liangzhu" => m(EventKind::Recover, Timing::Normal, 0, true, Vec::new()),
        "rexuanfeng" => m(
            EventKind::PhaseEnd,
            Timing::Normal,
            0,
            true,
            vec![EventKind::CardsLost],
        ),
        "reqizhou" => m(
            EventKind::Equipped,
            Timing::Normal,
            0,
            false,
            vec![EventKind::CardsLost],
        ),

        // DamageBefore (`DamageBegin1..4`)
        "jieyuan" => m(EventKind::DamageBefore, Timing::Normal, 30, true, Vec::new()),
        "rejinjiu" => m(EventKind::DamageBefore, Timing::Normal, 10, false, Vec::new()),
        "mjmouzhi" => m(
            EventKind::DamageBefore,
            Timing::Normal,
            15,
            false,
            vec![EventKind::DamageEnd],
        ),
        "retianxiang" => m(EventKind::DamageBefore, Timing::Normal, -10, true, Vec::new()),
        "repojun" => m(
            EventKind::CardUsed,
            Timing::Normal,
            0,
            true,
            vec![EventKind::DamageBefore],
        ),

        // DamageEnd / DamageSource / DyingSaved
        "benyu" | "renshe" => m(EventKind::DamageEnd, Timing::Normal, 0, true, Vec::new()),
        "chouce" => m(EventKind::DamageEnd, Timing::Normal, 0, false, Vec::new()),
        "guju" => m(EventKind::DamageEnd, Timing::Normal, 0, false, Vec::new()),
        "requanji" => m(
            EventKind::DamageEnd,
            Timing::Normal,
            0,
            false,
            vec![EventKind::PhaseEnd],
        ),
        "qingxian" => m(
            EventKind::DamageEnd,
            Timing::Normal,
            0,
            true,
            vec![EventKind::Recover, EventKind::DyingSaved],
        ),
        "liyu" => m(EventKind::DamageSource, Timing::Normal, 0, false, Vec::new()),
        "nsxianhai" => m(EventKind::DamageSource, Timing::Normal, 0, false, Vec::new()),
        "duwu" => m(EventKind::DyingSaved, Timing::Normal, 0, false, Vec::new()),

        // CardUsed / CardResponded
        "dcmoukui" | "gaoyuan" | "reduodao" => {
            m(EventKind::CardUsed, Timing::Normal, 0, true, Vec::new())
        }
        "bmcanshi" | "jsrgshichong" => {
            m(EventKind::CardUsed, Timing::Normal, 0, false, Vec::new())
        }
        "jili" | "jinglve" | "jsrgzhenqiao" | "kyou_zhidian" | "olzhuiji" => {
            m(EventKind::CardUsed, Timing::Normal, 0, false, Vec::new())
        }
        "dcnutao" => m(
            EventKind::CardUsed,
            Timing::Normal,
            0,
            false,
            vec![EventKind::DamageSource],
        ),
        "rezhijian" => m(EventKind::CardUsed, Timing::Normal, 0, false, Vec::new()),
        "leiji" => m(
            EventKind::CardUsed,
            Timing::Normal,
            0,
            true,
            vec![EventKind::CardResponded],
        ),
        "reyajiao" => m(
            EventKind::CardUsed,
            Timing::Normal,
            0,
            false,
            vec![EventKind::CardResponded],
        ),
        "reluanji" => m(EventKind::CardResponded, Timing::Normal, 0, false, Vec::new()),

        // PhaseBegin / PhaseEnd / DrawPhase
        "gnsheque" | "mozhi" | "rejuece" | "rezaiqi" => {
            m(EventKind::PhaseBegin, Timing::Normal, 0, true, Vec::new())
        }
        "jigong" | "piaoling" | "reguanxing" => {
            m(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new())
        }
        "lixia" => m(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()),
        "refenxun" => m(EventKind::PhaseBegin, Timing::Normal, 0, false, Vec::new()),
        "qingzhongx" => m(
            EventKind::PhaseBegin,
            Timing::Normal,
            0,
            false,
            vec![EventKind::PhaseEnd],
        ),
        "lihun" => m(EventKind::PhaseEnd, Timing::Normal, 0, false, Vec::new()),
        "nzry_kuizhu" => m(EventKind::PhaseEnd, Timing::Normal, 0, true, Vec::new()),
        "diancai" => m(EventKind::PhaseEnd, Timing::Normal, 0, false, Vec::new()),
        "reluoyi" => m(EventKind::DrawPhaseBegin1, Timing::Normal, 0, false, Vec::new()),
        "reshuangxiong" => m(
            EventKind::DrawPhaseBegin1,
            Timing::Normal,
            0,
            false,
            vec![EventKind::DamageEnd, EventKind::CardResponded],
        ),
        "retuxi" => m(EventKind::DrawPhaseBegin2, Timing::Normal, 0, true, Vec::new()),

        // Active-only or pure passive/viewAs
        _ => None,
    }
}

pub(in crate::content::host) fn window_trigger_meta_l11_npc_a(
    ctx: &crate::catalog::QueryCtx<'_>,
    name: &str,
    kind: EventKind,
) -> Option<TriggerMeta> {
    let mut meta = trigger_meta_l11_npc_a(name)?;
    match (name, kind) {
        ("repojun", EventKind::DamageBefore) => meta.optional = false,
        ("qingxian", EventKind::DyingSaved) => meta.optional = false,
        ("qingxian", EventKind::Recover)
            if ctx.state.alive_players().any(|p| ctx.state.player(p).dying) =>
        {
            meta.optional = false;
        }
        ("qingzhongx", EventKind::PhaseEnd) => meta.optional = false,
        ("reshuangxiong", EventKind::CardResponded) => meta.optional = false,
        ("requanji", EventKind::PhaseEnd) => meta.optional = false,
        _ => {}
    }
    Some(meta)
}

pub(in crate::content::host) fn limits_l11_npc_a(name: &str) -> SkillLimits {
    let once = matches!(
        name,
        "fenxun"
            | "guanzong"
            | "gushe"
            | "jinglve"
            | "jujian"
            | "lianzhu"
            | "mingjian"
            | "mizhao"
            | "mouzhu"
            | "new_yijue"
            | "nzry_chenglve"
            | "nzry_feijun"
            | "quji"
            | "reganlu"
            | "reguose"
            | "rejunxing"
            | "rewurong"
            | "rexingluan"
    );
    SkillLimits {
        once_per_turn: once,
        active_outside_play: false,
    }
}

pub(in crate::content::host) fn has_phase_use_l11_npc_a(name: &str) -> bool {
    matches!(
        name,
        "baiyi"
            | "duwu"
            | "fenxun"
            | "guanzong"
            | "gushe"
            | "jinglve"
            | "jujian"
            | "lianzhu"
            | "lihun"
            | "mingjian"
            | "mizhao"
            | "mouzhu"
            | "new_reqingnang"
            | "new_yijue"
            | "nzry_chenglve"
            | "nzry_feijun"
            | "quji"
            | "refenxun"
            | "reganlu"
            | "reguose"
            | "rejunxing"
            | "reluanji"
            | "rewurong"
            | "rezhijian"
            | "rongbei"
            | "kyou_zhidian"
            | "dclbjiuxian"
            | "olhuoji"
            | "reshuangxiong"
    )
}
