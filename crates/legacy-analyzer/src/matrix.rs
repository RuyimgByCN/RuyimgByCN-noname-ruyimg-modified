//! Source-backed skill capability matrix for M0 planning.

use crate::{grade_skill, SkillProfile};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Machine-checkable capability matrix built from legacy sources.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityMatrix {
    /// Artifact schema.
    pub schema_version: u32,
    /// Number of analyzed skills.
    pub total_skills: usize,
    /// Source label → skill count.
    pub source_counts: BTreeMap<String, usize>,
    /// One source-backed row per extracted legacy skill.
    pub skills: Vec<SkillCapabilityRow>,
    /// Interpretation warning.
    pub disclaimer: String,
}

/// One skill's extracted capability portrait and current surface gaps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillCapabilityRow {
    /// Source pack label.
    pub source: String,
    /// Legacy skill key.
    pub name: String,
    /// Analyzer compatibility grade.
    pub grade: String,
    /// Raw trigger bindings.
    pub triggers: Vec<String>,
    /// Active enable windows.
    pub active_windows: Vec<String>,
    /// Forced/locked marker in the legacy object.
    pub locked: bool,
    /// Frequent/direct triggers need owner policy/choice review.
    pub optional_review: bool,
    /// View-as conversion marker.
    pub view_as: bool,
    /// Modifier marker.
    pub modifier: bool,
    /// Statically mapped primitive effects.
    pub mapped_effects: Vec<String>,
    /// Required DSL/host/trigger surfaces inferred from the object.
    pub required_surfaces: Vec<String>,
    /// Explicit gaps to resolve before claiming semantic parity.
    pub gaps: Vec<String>,
    /// Forbidden or special host references detected by the analyzer.
    pub capability_hits: Vec<String>,
}

/// Build a deterministic matrix from labeled analyzer outputs.
pub fn build_capability_matrix(
    sources: impl IntoIterator<Item = (String, Vec<SkillProfile>)>,
) -> CapabilityMatrix {
    let mut rows = Vec::new();
    let mut source_counts = BTreeMap::new();
    for (source, profiles) in sources {
        source_counts.insert(source.clone(), profiles.len());
        for profile in profiles {
            rows.push(row(&source, &profile));
        }
    }
    rows.sort_by(|a, b| (&a.source, &a.name).cmp(&(&b.source, &b.name)));
    CapabilityMatrix {
        schema_version: 1,
        total_skills: rows.len(),
        source_counts,
        skills: rows,
        disclaimer: "Heuristic source inventory only; each row still requires semantic Golden tests before migration is complete.".into(),
    }
}

impl CapabilityMatrix {
    /// Pretty JSON artifact.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Human summary; the JSON file remains the complete row-level matrix.
    pub fn to_markdown(&self) -> String {
        let mut out = String::from("# M0 技能能力画像与原语矩阵\n\n");
        out.push_str(&format!("> {}\n\n", self.disclaimer));
        out.push_str(&format!(
            "共 **{}** 个源码技能画像：\n\n",
            self.total_skills
        ));
        for (source, count) in &self.source_counts {
            out.push_str(&format!("- `{source}`：{count}\n"));
        }
        let mut grades: BTreeMap<&str, usize> = BTreeMap::new();
        let mut gaps: BTreeMap<&str, usize> = BTreeMap::new();
        for row in &self.skills {
            *grades.entry(&row.grade).or_default() += 1;
            for gap in &row.gaps {
                *gaps.entry(gap).or_default() += 1;
            }
        }
        out.push_str("\n## 覆盖等级\n\n| 等级 | 数量 |\n|---|---:|\n");
        for (grade, count) in grades {
            out.push_str(&format!("| {grade} | {count} |\n"));
        }
        out.push_str("\n## 缺口聚合\n\n| 缺口 | 技能数 | 当前处理 |\n|---|---:|---|\n");
        for (gap, count) in gaps {
            out.push_str(&format!("| `{gap}` | {count} | {} |\n", gap_status(gap)));
        }
        out.push_str("\n完整逐技能行见 `skill-capability-matrix.json`。该 JSON 保存触发点、主动/视为/锁定/可选画像、已映射 effect、所需 surface 与缺口，可由 `legacy-analyze matrix` 重建。\n");
        out
    }
}

fn row(source: &str, profile: &SkillProfile) -> SkillCapabilityRow {
    let grade = grade_skill(profile).grade.as_str().to_string();
    let triggers: Vec<_> = profile
        .triggers
        .iter()
        .map(|trigger| format!("{}:{}", trigger.role, trigger.events.join(",")))
        .collect();
    let mut required = profile.mapped_effects.clone();
    required.extend(
        profile
            .mapped_events
            .iter()
            .map(|event| format!("trigger:{event}")),
    );
    if !profile.enable.is_empty() {
        required.push("active_skill".into());
    }
    if profile.has_view_as {
        required.push("view_as".into());
    }
    if profile.has_mod {
        required.push("intent_modifier".into());
    }
    if profile.has_cost || profile.has_check || profile.frequent || profile.direct {
        required.push("choice_or_cost".into());
    }
    if profile.has_group {
        required.push("subskill_state".into());
    }
    required.sort();
    required.dedup();

    let mut gaps = Vec::new();
    let raw_event_count: usize = profile
        .triggers
        .iter()
        .map(|trigger| trigger.events.len())
        .sum();
    if profile.mapped_events.len() < raw_event_count {
        gaps.push("trigger_point_review".into());
    }
    if profile.has_content && profile.mapped_effects.is_empty() {
        gaps.push("effect_or_intent_review".into());
    }
    if profile.has_cost || profile.has_check || profile.frequent || profile.direct {
        gaps.push("choice_cost_policy".into());
    }
    if profile.has_group {
        gaps.push("subskill_state".into());
    }
    if !profile.capability_hits.is_empty() {
        gaps.push("forbidden_host_rewrite".into());
    }
    if grade != "A" && gaps.is_empty() {
        gaps.push("semantic_manual_review".into());
    }
    gaps.sort();
    gaps.dedup();

    SkillCapabilityRow {
        source: source.into(),
        name: profile.name.clone(),
        grade,
        triggers,
        active_windows: profile.enable.clone(),
        locked: profile.forced,
        optional_review: profile.frequent || profile.direct,
        view_as: profile.has_view_as,
        modifier: profile.has_mod,
        mapped_effects: profile.mapped_effects.clone(),
        required_surfaces: required,
        gaps,
        capability_hits: profile.capability_hits.clone(),
    }
}

fn gap_status(gap: &str) -> &'static str {
    match gap {
        "choice_cost_policy" => "已有 Choice/optional；逐技能确认成本语义",
        "effect_or_intent_review" => "已有通用 Effect/Intent；缺口按 M0 工艺补原语",
        "forbidden_host_rewrite" => "禁止透传 UI/状态全局，改写为投影或通用 Host",
        "subskill_state" => "需技能状态/组合技能语义",
        "trigger_point_review" => "与 rule-semantics 对齐后新增通用 Trigger Point",
        _ => "逐技能 Golden 人工确认",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyze_source;

    #[test]
    fn matrix_marks_choice_and_host_gaps() {
        let profiles = analyze_source(
            r#"x: { trigger: { player: "damageEnd" }, frequent: true, content() { ui.foo(); } },"#,
        )
        .unwrap();
        let matrix = build_capability_matrix([("sample".into(), profiles)]);
        assert_eq!(matrix.total_skills, 1);
        assert!(matrix.skills[0].gaps.contains(&"choice_cost_policy".into()));
        assert!(matrix.skills[0]
            .gaps
            .contains(&"forbidden_host_rewrite".into()));
    }
}
