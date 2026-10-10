//! Generate Next DSL skeletons from skill profiles.
//!
//! Skeletons are migration aids — not proven equivalent to legacy JS.

use crate::extract::SkillProfile;
use crate::grade::{grade_skill, CompatGrade};

/// Generate a `.skill` DSL skeleton for one profile.
pub fn generate_dsl_skeleton(p: &SkillProfile, def_id: u32) -> String {
    let grade = grade_skill(p);
    let mut out = String::new();
    out.push_str(&format!(
        "// Migrated skeleton from legacy skill `{}` (grade {})\n",
        p.name,
        grade.grade.as_str()
    ));
    out.push_str("// WARNING: Not proven semantically equivalent to original JavaScript.\n");
    for r in &grade.reasons {
        out.push_str(&format!("// note: {r}\n"));
    }
    if !p.capability_hits.is_empty() {
        out.push_str(&format!(
            "// capabilities: {}\n",
            p.capability_hits.join(", ")
        ));
    }
    out.push_str(&format!(
        "skill {}({}) {{\n",
        sanitize_ident(&p.name),
        def_id
    ));

    if p.has_mod {
        if p.mod_hints.is_empty() {
            out.push_str("  // TODO: map mod {{ ... }} to modifier lines\n");
            out.push_str("  // modifier distance_from add 0;\n");
        } else {
            for h in &p.mod_hints {
                out.push_str(&format!(
                    "  modifier {} {} {};\n",
                    h.target, h.mode, h.value
                ));
            }
        }
    }

    if !p.triggers.is_empty() {
        for t in &p.triggers {
            for ev in &t.events {
                let mapped = crate::extract::map_event(ev);
                let filter = match t.role.as_str() {
                    "player" => "owner_is_target",
                    "source" => "owner_is_source",
                    _ => "always",
                };
                let event = if mapped.is_empty() {
                    "damage".to_string()
                } else {
                    mapped
                };
                out.push_str(&format!("  // legacy trigger: {} / {}\n", t.role, ev));
                out.push_str(&format!("  trigger {event} {filter} {{\n"));
                if p.mapped_effects.is_empty() {
                    out.push_str("    // TODO: implement content\n");
                    out.push_str("    effect emit_note(0);\n");
                } else {
                    for ef in &p.mapped_effects {
                        match ef.as_str() {
                            "draw_cards" => {
                                out.push_str("    effect draw_cards(self, 1);\n");
                            }
                            "recover_hp" => {
                                out.push_str("    effect recover_hp(self, 1);\n");
                            }
                            "deal_damage" => {
                                out.push_str(
                                    "    // TODO: effect deal_damage needs host extension\n",
                                );
                                out.push_str("    effect emit_note(1);\n");
                            }
                            other => {
                                out.push_str(&format!("    // TODO: map effect `{other}`\n"));
                                out.push_str("    effect emit_note(2);\n");
                            }
                        }
                    }
                }
                if grade.grade != CompatGrade::A {
                    out.push_str("    // TODO: port filter/cost/check logic\n");
                }
                out.push_str("  }\n");
            }
        }
    } else if !p.enable.is_empty() {
        out.push_str("  // TODO: active skill enable=");
        out.push_str(&p.enable.join(","));
        out.push_str("\n");
        out.push_str("  // trigger phase_begin always {\n");
        out.push_str("  //   effect emit_note(0);\n");
        out.push_str("  // }\n");
    } else {
        out.push_str("  // TODO: unrecognized skill shape\n");
    }

    out.push_str("}\n");
    out
}

fn sanitize_ident(s: &str) -> String {
    let mut o = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_alphanumeric() || c == '_' {
            o.push(c);
        } else if i == 0 {
            o.push('s');
        } else {
            o.push('_');
        }
    }
    if o.is_empty() {
        "skill".into()
    } else {
        o
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extract::analyze_source;

    #[test]
    fn skeleton_has_skill_block() {
        let src = r#"
jianxiong: {
  trigger: { player: "damageEnd" },
  forced: true,
  content(event, trigger, player) { player.draw(); },
},
"#;
        let p = &analyze_source(src).unwrap()[0];
        let sk = generate_dsl_skeleton(p, 1);
        assert!(sk.contains("skill jianxiong(1)"));
        assert!(sk.contains("trigger damage"));
        assert!(sk.contains("draw_cards"));
    }
}
