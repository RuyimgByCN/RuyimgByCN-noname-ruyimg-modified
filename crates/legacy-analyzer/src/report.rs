//! Migration reports (JSON-serializable).

use crate::extract::SkillProfile;
use crate::grade::{grade_skill, CompatGrade, GradeDetail};
use crate::skeleton::generate_dsl_skeleton;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Per-skill report line.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillReport {
    /// Skill name.
    pub name: String,
    /// Grade.
    pub grade: CompatGrade,
    /// Reasons.
    pub reasons: Vec<String>,
    /// Capability hits.
    pub capability_hits: Vec<String>,
    /// Triggers summary.
    pub triggers: Vec<String>,
    /// Mapped effects.
    pub mapped_effects: Vec<String>,
    /// DSL skeleton text.
    pub dsl_skeleton: String,
    /// Skeleton useful?
    pub skeleton_useful: bool,
}

/// Full file report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationReport {
    /// Source path or label.
    pub source: String,
    /// Counts by grade.
    pub counts: BTreeMap<String, usize>,
    /// Skills.
    pub skills: Vec<SkillReport>,
    /// Disclaimer.
    pub disclaimer: String,
}

impl MigrationReport {
    /// Build from profiles.
    pub fn from_profiles(source: String, profiles: Vec<SkillProfile>) -> Self {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        let mut skills = Vec::new();
        for (i, p) in profiles.into_iter().enumerate() {
            let g: GradeDetail = grade_skill(&p);
            *counts.entry(g.grade.as_str().to_string()).or_insert(0) += 1;
            let triggers: Vec<String> = p
                .triggers
                .iter()
                .map(|t| format!("{}:{}", t.role, t.events.join(",")))
                .collect();
            let dsl = if g.skeleton_useful {
                generate_dsl_skeleton(&p, (i as u32) + 1)
            } else {
                format!(
                    "// grade D — do not auto-generate logic for `{}`\n// migrate data/assets only\n",
                    p.name
                )
            };
            skills.push(SkillReport {
                name: p.name.clone(),
                grade: g.grade,
                reasons: g.reasons,
                capability_hits: p.capability_hits.clone(),
                triggers,
                mapped_effects: p.mapped_effects.clone(),
                dsl_skeleton: dsl,
                skeleton_useful: g.skeleton_useful,
            });
        }
        Self {
            source,
            counts,
            skills,
            disclaimer: "This report does not claim semantic equivalence between legacy JavaScript and generated DSL. Grade A means statically mappable candidates only.".into(),
        }
    }

    /// JSON pretty.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Markdown summary.
    pub fn to_markdown(&self) -> String {
        let mut o = String::new();
        o.push_str(&format!("# Migration report: {}\n\n", self.source));
        o.push_str(&format!("> {}\n\n", self.disclaimer));
        o.push_str("## Counts\n\n");
        for (g, n) in &self.counts {
            o.push_str(&format!("- **{g}**: {n}\n"));
        }
        o.push_str("\n## Skills\n\n");
        for s in &self.skills {
            o.push_str(&format!(
                "### `{}` — grade {}\n\n",
                s.name,
                s.grade.as_str()
            ));
            for r in &s.reasons {
                o.push_str(&format!("- {r}\n"));
            }
            if !s.capability_hits.is_empty() {
                o.push_str(&format!(
                    "- capabilities: {}\n",
                    s.capability_hits.join(", ")
                ));
            }
            if !s.triggers.is_empty() {
                o.push_str(&format!("- triggers: {}\n", s.triggers.join("; ")));
            }
            o.push('\n');
        }
        o
    }
}
