//! PA01 回归：目录静态索引与调度成员关系的等价性守护（2026-10-26）。
//!
//! `plan_triggers`（resolution.rs）自 PA01 起从实例侧单遍出发、直接比对
//! `TriggerMeta.event/extra_events`，不再按 `Catalog::candidates` 逐定义
//! 遍历实例。本文件守护两条路径的成员关系严格等价，防止将来漂移；
//! 同批 `skill_by_name` 的 O(1) 名称索引守护「首个匹配」语义（与原
//! 线性 `position` 扫描一致）。

use engine_native::catalog::{Catalog, SkillBehavior, SkillDef, SkillMachine, TriggerMeta};
use engine_native::event::{EventData, EventKind, Timing};

/// 与触发规划热路径相同的成员判定（resolution.rs `plan_triggers` 内联）。
fn responds_to(def: &SkillDef, kind: EventKind) -> bool {
    def.trigger
        .as_ref()
        .is_some_and(|meta| meta.event == kind || meta.extra_events.contains(&kind))
}

#[test]
fn dispatch_candidates_match_trigger_meta_membership() {
    let catalog = engine_native::content::standard::standard_catalog();
    let kinds = [
        EventKind::DamageBefore,
        EventKind::Damage,
        EventKind::Recover,
        EventKind::CardsDrawn,
        EventKind::CardsMoved,
        EventKind::CardUsed,
        EventKind::CardResponded,
        EventKind::Dying,
        EventKind::Death,
        EventKind::PhaseBegin,
        EventKind::PhaseEnd,
        EventKind::TurnEnded,
        EventKind::SkillToggled,
        EventKind::Equipped,
        EventKind::Linked,
        EventKind::CardsDiscarded,
        EventKind::TurnOverBefore,
        EventKind::TurnOvered,
        EventKind::LoseHp,
        EventKind::DyingSaved,
        EventKind::CardsLost,
        EventKind::ShaDodged,
    ];
    for kind in kinds {
        let from_dispatch: std::collections::BTreeSet<u16> =
            catalog.candidates(kind).iter().copied().collect();
        let from_meta: std::collections::BTreeSet<u16> = catalog
            .skills
            .iter()
            .enumerate()
            .filter(|(_, def)| responds_to(def, kind))
            .map(|(i, _)| i as u16)
            .collect();
        assert_eq!(
            from_dispatch, from_meta,
            "kind={kind:?}: dispatch 列表与 TriggerMeta 成员判定不一致"
        );
    }
}

/// 合成行为：只做注册（filter 恒否），不进入运行期。
struct Noop;

impl SkillBehavior for Noop {
    fn filter(
        &self,
        _ctx: &engine_native::catalog::QueryCtx<'_>,
        _owner: engine_native::ids::PlayerId,
        _event: &EventData,
    ) -> bool {
        false
    }

    fn start(
        &self,
        _owner: engine_native::ids::PlayerId,
        _event: &EventData,
    ) -> Box<dyn SkillMachine> {
        unimplemented!("Noop 行为不会被触发（filter 恒否）")
    }
}

#[test]
fn synthetic_catalog_membership_and_first_match_name() {
    let mk = |name: &'static str, event: EventKind, extra: Vec<EventKind>| SkillDef {
        name,
        trigger: Some(TriggerMeta {
            event,
            timing: Timing::Normal,
            priority: 0,
            order_selectable: true,
            optional: false,
            extra_events: extra,
        }),
        limits: Default::default(),
        behavior: Box::new(Noop),
    };
    let skills = vec![
        mk("dup", EventKind::Damage, vec![EventKind::CardsDrawn]),
        mk("dup", EventKind::Recover, vec![]),
        mk("plain", EventKind::PhaseEnd, vec![]),
    ];
    let mut catalog = Catalog::build(Vec::new(), skills);
    // F7: a raw catalog keeps its default metadata, without content hooks.
    use engine_native::content::standard::{Gender, BOSS_GENERALS, STANDARD_GENERALS};
    assert_eq!(
        catalog.general_count(),
        STANDARD_GENERALS.len() + BOSS_GENERALS.len()
    );
    for (i, &(name, faction, skills, hp, gender)) in
        STANDARD_GENERALS.iter().chain(BOSS_GENERALS).enumerate()
    {
        assert_eq!(
            catalog.general_meta(i as u16),
            Some((name, faction, skills.to_vec(), hp, gender))
        );
    }
    assert_eq!(catalog.general_meta(catalog.general_count() as u16), None);
    catalog
        .extension_generals
        .push(engine_native::content::extension::ExtensionGeneral {
            name: "F7 metadata probe",
            faction: engine_native::catalog::Faction::Qun,
            skills: &["dup"],
            max_hp: 3,
            gender: Gender::Female,
        });
    let ext = STANDARD_GENERALS.len() as u16;
    assert_eq!(catalog.general_meta(ext).unwrap().0, "F7 metadata probe");
    assert_eq!(catalog.general_meta(ext).unwrap().4, Gender::Female);
    for (i, &(name, faction, skills, hp, gender)) in BOSS_GENERALS.iter().enumerate() {
        assert_eq!(
            catalog.general_meta(ext + 1 + i as u16),
            Some((name, faction, skills.to_vec(), hp, gender))
        );
    }
    assert_eq!(catalog.general_meta(catalog.general_count() as u16), None);
    // dispatch ⟺ 元数据成员（含 extra_events 路径；主事件不互串）
    assert!(catalog.candidates(EventKind::Damage).contains(&0u16));
    assert!(catalog.candidates(EventKind::CardsDrawn).contains(&0u16));
    assert!(catalog.candidates(EventKind::Recover).contains(&1u16));
    assert!(!catalog.candidates(EventKind::Recover).contains(&0u16));
    assert_eq!(catalog.candidates(EventKind::PhaseEnd), &[2u16]);
    // skill_by_name 首个匹配语义（重名取定义号小者）
    assert_eq!(catalog.skill_by_name("dup"), Some(0u16));
    assert_eq!(catalog.skill_by_name("plain"), Some(2u16));
    assert_eq!(catalog.skill_by_name("missing"), None);
}
