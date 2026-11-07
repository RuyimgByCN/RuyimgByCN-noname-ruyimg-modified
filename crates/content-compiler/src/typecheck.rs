//! Name resolution, type check, capability / termination checks → Rule IR.

use crate::ast::*;
use crate::CompileError;
use content_schema::{
    ActiveIr, ActiveSelectionIr, ActiveTargetsIr, DrawTargetScopeIr, ExprIr, ModifierIr,
    RuleModuleIr, SkillIr, Span, StmtIr, TargetCardScopeIr, TriggerIr, ViewAsIr,
};

mod active;
mod effects;
mod expressions;
mod modifiers;
mod skills;
mod statements;
mod vocabulary;

use active::*;
use effects::check_effect_stmt;
use expressions::*;
use modifiers::*;
use skills::*;
use statements::*;
pub use vocabulary::filter_vocabulary;
use vocabulary::*;

/// Typecheck AST → IR.
pub fn typecheck(file: &SourceFile) -> Result<RuleModuleIr, CompileError> {
    // Phase 3 compile-time cross-skill name resolution: build name→def map from all skill
    // declarations in this compilation unit, then rewrite `add_skill_by_name(player, NAME)` to
    // `add_skill(player, DEF)` at the AST level. This solves chained-helper resolution
    // (addTempSkill("X_effect")) without VM string-transport (ADR 0640 limit).
    let mut name_to_def: std::collections::BTreeMap<String, u32> =
        std::collections::BTreeMap::new();
    for sk in &file.skills {
        name_to_def.entry(sk.name.clone()).or_insert(sk.def_id);
    }
    let mut file = file.clone();
    if !name_to_def.is_empty() {
        for sk in &mut file.skills {
            for item in &mut sk.items {
                rewrite_add_skill_by_name(item, &name_to_def);
            }
        }
    }
    let mut skills = Vec::new();
    let mut seen_defs = std::collections::BTreeSet::new();
    for sk in &file.skills {
        if !seen_defs.insert(sk.def_id) {
            return Err(CompileError::Type {
                pos: sk.span.start as usize,
                message: format!("duplicate skill def id {}", sk.def_id),
            });
        }
        skills.push(check_skill(sk)?);
    }
    Ok(RuleModuleIr {
        name: "main".into(),
        skills,
    })
}

/// Recursively rewrite `add_skill_by_name(player, NAME)` → `add_skill(player, DEF)` in a skill item.
fn rewrite_add_skill_by_name(
    item: &mut SkillItemAst,
    name_to_def: &std::collections::BTreeMap<String, u32>,
) {
    match item {
        SkillItemAst::Trigger(t) => rewrite_stmts(&mut t.body, name_to_def),
        SkillItemAst::Active(a) => rewrite_stmts(&mut a.body, name_to_def),
        SkillItemAst::ViewAs(_) => {} // view_as uses declarative cost, no stmt body
        SkillItemAst::Modifier(_) => {}
    }
}

fn rewrite_stmts(stmts: &mut Vec<StmtAst>, name_to_def: &std::collections::BTreeMap<String, u32>) {
    for stmt in stmts.iter_mut() {
        rewrite_stmt(stmt, name_to_def);
    }
}

fn cross_file_skill_def_id(skill_name: &str) -> u32 {
    match skill_name {
        "chixueqingfeng2" => 3_863_274_410,
        "rebiluan2" => 3_680_299_345,
        "wangzun3" => 2_630_565_864,
        "decadexushen2" => 1_919_822_977,
        _ => {
            let mut h: u32 = 2_166_136_261;
            for b in skill_name.as_bytes() {
                h ^= u32::from(*b);
                h = h.wrapping_mul(16_777_619);
            }
            if h == 0 { 1 } else { h }
        }
    }
}

fn rewrite_stmt(stmt: &mut StmtAst, name_to_def: &std::collections::BTreeMap<String, u32>) {
    match stmt {
        StmtAst::Effect { name, args, .. } if name == "add_skill_by_name" => {
            if args.len() == 2 {
                if let ExprAst::Ident(skill_name) = &args[1] {
                    let def = name_to_def
                        .get(skill_name)
                        .copied()
                        .unwrap_or_else(|| cross_file_skill_def_id(skill_name));
                    *name = "add_skill".into();
                    args[1] = ExprAst::U32(def);
                }
            }
        }
        StmtAst::If {
            then_body,
            else_body,
            ..
        } => {
            rewrite_stmts(then_body, name_to_def);
            rewrite_stmts(else_body, name_to_def);
        }
        StmtAst::Loop { body, .. } => {
            rewrite_stmts(body, name_to_def);
        }
        StmtAst::Let { value: _, .. } => {
            // no nested stmts in let value
        }
        _ => {}
    }
}
