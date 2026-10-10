//! Compatibility grading A/B/C/D (architecture §17.3).

use crate::extract::SkillProfile;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// Compatibility grade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompatGrade {
    /// Static map + auto compile + golden candidate.
    A,
    /// DSL skeleton complete; needs local manual fill.
    B,
    /// Structure known; rule logic needs rewrite.
    C,
    /// DOM / globals / I/O / undefined timing — data & assets only.
    D,
}

impl CompatGrade {
    /// As str.
    pub fn as_str(self) -> &'static str {
        match self {
            CompatGrade::A => "A",
            CompatGrade::B => "B",
            CompatGrade::C => "C",
            CompatGrade::D => "D",
        }
    }
}

/// Detailed grade with reasons.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GradeDetail {
    /// Grade.
    pub grade: CompatGrade,
    /// Human reasons.
    pub reasons: Vec<String>,
    /// Whether auto DSL skeleton is useful.
    pub skeleton_useful: bool,
}

/// Grade one skill profile.
pub fn grade_skill(p: &SkillProfile) -> GradeDetail {
    let mut reasons = Vec::new();

    // D: hard host escapes
    let d_tags = [
        "dom",
        "network",
        "filesystem",
        "process",
        "eval",
        "dynamic_import",
        "require",
        "node",
        "storage",
        "clock",
        "nondeterministic_rng",
    ];
    let d_hits: Vec<_> = p
        .capability_hits
        .iter()
        .filter(|h| d_tags.contains(&h.as_str()))
        .cloned()
        .collect();
    if !d_hits.is_empty() {
        reasons.push(format!("unsafe host capabilities: {}", d_hits.join(", ")));
        return GradeDetail {
            grade: CompatGrade::D,
            reasons,
            skeleton_useful: false,
        };
    }

    // A: simple forced trigger + only mapped effects, no cost/filter complexity
    let raw_event_count = p
        .triggers
        .iter()
        .map(|trigger| trigger.events.len())
        .sum::<usize>();
    let simple_trigger = raw_event_count > 0
        && p.mapped_events.len() == raw_event_count
        && p.enable.is_empty()
        && !p.has_view_as
        && !p.has_mod
        && !p.has_cost
        && !p.has_group
        && p.forced
        && p.mapped_effects.len() == 1
        && content_is_exactly_mappable(p)
        && p.capability_hits.is_empty();

    if simple_trigger {
        reasons.push("forced trigger with mappable events/effects".into());
        if p.has_filter {
            reasons.push("has filter — may need manual condition".into());
            return GradeDetail {
                grade: CompatGrade::B,
                reasons,
                skeleton_useful: true,
            };
        }
        return GradeDetail {
            grade: CompatGrade::A,
            reasons,
            skeleton_useful: true,
        };
    }

    // B: structure clear (trigger or enable) but needs manual body
    let structured = !p.triggers.is_empty() || !p.enable.is_empty() || p.has_mod || p.has_view_as;
    if structured {
        if p.has_cost || p.has_check || p.has_group {
            reasons.push("has cost/check/group — partial skeleton".into());
        }
        if p.has_view_as {
            reasons.push("viewAs skill".into());
        }
        if p.has_mod {
            reasons.push("mod skill".into());
        }
        if !p.enable.is_empty() {
            reasons.push(format!("active enable={:?}", p.enable));
        }
        if p.mapped_effects.is_empty() && p.has_content {
            reasons.push("content effects not auto-mapped".into());
        }
        // ui_global / status without D tags → still B/C
        if p.capability_hits
            .iter()
            .any(|h| h == "ui_global" || h == "status_global")
        {
            reasons.push("uses ui/_status globals".into());
            return GradeDetail {
                grade: CompatGrade::C,
                reasons,
                skeleton_useful: true,
            };
        }
        if reasons.is_empty() {
            reasons.push("structured skill; needs manual content review".into());
        }
        return GradeDetail {
            grade: CompatGrade::B,
            reasons,
            skeleton_useful: true,
        };
    }

    // C: opaque
    reasons.push("skill structure incomplete or nonstandard".into());
    GradeDetail {
        grade: CompatGrade::C,
        reasons,
        skeleton_useful: p.has_content || p.has_filter,
    }
}

fn content_is_exactly_mappable(profile: &SkillProfile) -> bool {
    static SIMPLE_EFFECT: OnceLock<Regex> = OnceLock::new();
    SIMPLE_EFFECT
        .get_or_init(|| {
            Regex::new(r"^\s*(?:await\s+)?player\.(?:draw|recover)\(\s*(?:1\s*)?\)\s*;?\s*$")
                .unwrap()
        })
        .is_match(&profile.content_snippet)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extract::analyze_source;

    #[test]
    fn grades_sample() {
        let src = r#"
x: {
  trigger: { player: "damageEnd" },
  forced: true,
  async content(event, trigger, player) { await player.draw(); },
},
y: {
  enable: "phaseUse",
  content() { document.write(1); },
},
"#;
        let ps = analyze_source(src).unwrap();
        let gx = grade_skill(ps.iter().find(|p| p.name == "x").unwrap());
        assert_eq!(gx.grade, CompatGrade::A);
        let gy = grade_skill(ps.iter().find(|p| p.name == "y").unwrap());
        assert_eq!(gy.grade, CompatGrade::D);
    }

    #[test]
    fn dynamic_effect_amount_is_not_grade_a() {
        let src = r#"
x: {
  trigger: { player: "phaseJieshuBegin" },
  forced: true,
  async content(event, trigger, player) {
    await player.draw(player.countCards("h") == 0 ? 2 : 1);
  },
},
"#;
        let profile = &analyze_source(src).unwrap()[0];
        assert_eq!(grade_skill(profile).grade, CompatGrade::B);
    }

    #[test]
    fn unknown_trigger_is_not_grade_a() {
        let src = r#"
x: {
  trigger: { global: "turnOverEnd" },
  forced: true,
  async content(event, trigger, player) { await player.draw(); },
},
"#;
        let profile = &analyze_source(src).unwrap()[0];
        assert_eq!(grade_skill(profile).grade, CompatGrade::B);
    }
}
