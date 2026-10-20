//! **探针（只读）**：`EventKind::PhaseEnd` 是否存在可用派发面（第 57 项的**系统级**核对）。
//!
//! 背景：第 57 项发现 `baishouzhihu` 的 `TriggerMeta` 注册 `EventKind::PhaseEnd`，
//! 而引擎的 `Phase` 事件只在**回合起点的判定阶段**以 `begin: true` 发射（`kernel/events.rs:753/789`）。
//! 若**没有任何** `PhaseEnd` 事件被发射，则**所有**注册 `PhaseEnd` 的技能都是**死技能**——
//! 影响面远大于单个法器。
//!
//! 本探针**只读**：统计「目录中注册 PhaseEnd 的技能数」与「`candidates(PhaseEnd)` 的条目数」。

use engine_native::event::EventKind;

#[test]
fn probe_phase_end_skills_and_candidates() {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/shahai-swd-upstream-noname/assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).expect("读 swd cards");
    let pack = content_native_loader::bridge_pack(&cards).expect("桥接").pack;
    let e = engine_native::Engine::new_with_extensions(71, 4, &[&pack]).expect("建局");

    let mut phase_end_skills: Vec<(engine_native::ids::SkillDefId, String)> = Vec::new();
    let mut phase_begin = 0usize;
    for (i, def) in e.catalog().skills.iter().enumerate() {
        if let Some(t) = def.trigger.as_ref() {
            match t.event {
                EventKind::PhaseEnd => {
                    phase_end_skills.push((i as engine_native::ids::SkillDefId, def.name.to_string()))
                }
                EventKind::PhaseBegin => phase_begin += 1,
                _ => {}
            }
        }
    }
    let cand_end = e.catalog().candidates(EventKind::PhaseEnd);
    let cand_begin = e.catalog().candidates(EventKind::PhaseBegin);
    let in_end = phase_end_skills
        .iter()
        .filter(|(id, _)| cand_end.contains(id))
        .count();
    eprintln!(
        "DBG PhaseEnd：目录注册 PhaseEnd 技能 {} 个（其中进入 candidates {} 个）；\
         candidates(PhaseEnd) 共 {}；PhaseBegin 技能 {} → candidates {}",
        phase_end_skills.len(),
        in_end,
        cand_end.len(),
        phase_begin,
        cand_begin.len()
    );
    for (id, key) in phase_end_skills.iter().take(8) {
        eprintln!(
            "DBG PhaseEnd 技能：#{id} `{key}` 在 candidates 中={}",
            cand_end.contains(id)
        );
    }
    assert!(
        !phase_end_skills.is_empty(),
        "本探针应有 PhaseEnd 技能样本（否则说明取错包）"
    );
}
