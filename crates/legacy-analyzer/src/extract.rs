//! Heuristic extraction of skill objects from noname skill.js sources.
//!
//! Not a full JS parser — reliable for declaration-style skill tables.

use crate::AnalyzeError;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// Extracted skill profile (capability portrait).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillProfile {
    /// Skill id / key in lib.skill.
    pub name: String,
    /// Trigger roles → event names (e.g. player → damageEnd).
    pub triggers: Vec<TriggerSpec>,
    /// Active skill enable string(s).
    pub enable: Vec<String>,
    /// Has forced: true.
    pub forced: bool,
    /// Has frequent.
    pub frequent: bool,
    /// Has direct.
    pub direct: bool,
    /// Has mod: { ... }.
    pub has_mod: bool,
    /// Has viewAs.
    pub has_view_as: bool,
    /// Has filter function.
    pub has_filter: bool,
    /// Has cost function.
    pub has_cost: bool,
    /// Has content function.
    pub has_content: bool,
    /// Has check function.
    pub has_check: bool,
    /// Has group / subSkill.
    pub has_group: bool,
    /// Has ai block.
    pub has_ai: bool,
    /// Audio field present.
    pub has_audio: bool,
    /// Zhu skill.
    pub zhu_skill: bool,
    /// Content/body source snippet (truncated).
    pub content_snippet: String,
    /// Full skill object text (truncated for report).
    pub raw_span: String,
    /// Detected dangerous / host capability hits.
    pub capability_hits: Vec<String>,
    /// Mapped DSL event names when known.
    pub mapped_events: Vec<String>,
    /// Mapped simple effects when recognized in content.
    pub mapped_effects: Vec<String>,
    /// Statically recognized `mod { ... }` contributions (distance / range).
    ///
    /// Heuristic only: covers the common `globalFrom/globalTo/attackRange`
    /// `return distance ± N` shape. Anything else leaves `has_mod` set with an
    /// empty hint list (skeleton emits a TODO instead of a concrete line).
    pub mod_hints: Vec<ModHint>,
}

/// A statically recognized modifier contribution, mappable to a DSL
/// `modifier <target> <mode> <value>;` line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModHint {
    /// DSL modifier target (e.g. `distance_from`, `attack_range`).
    pub target: String,
    /// DSL modifier mode (currently always `add`).
    pub mode: String,
    /// Signed contribution value.
    pub value: i32,
}

/// One trigger binding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggerSpec {
    /// Role: player / source / target / global.
    pub role: String,
    /// Event name(s).
    pub events: Vec<String>,
}

/// Analyze entire source text; returns one profile per top-level skill key.
pub fn analyze_source(source: &str) -> Result<Vec<SkillProfile>, AnalyzeError> {
    let blocks = split_skill_blocks(source);
    if blocks.is_empty() {
        return Err(AnalyzeError::Extract(
            "no skill objects found (expected name: { ... })".into(),
        ));
    }
    let mut out = Vec::new();
    for (name, body) in blocks {
        out.push(profile_skill(&name, &body));
    }
    Ok(out)
}

fn split_skill_blocks(source: &str) -> Vec<(String, String)> {
    // Find identifiers followed by `: {` at skill-table depth roughly.
    // Strategy: scan for `ident: {` then brace-match.
    let mut out = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        // skip strings roughly
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            let q = bytes[i];
            i += 1;
            while i < bytes.len() && bytes[i] != q {
                if bytes[i] == b'\\' {
                    i += 1;
                }
                i += 1;
            }
            i += 1;
            continue;
        }
        // match ident
        if is_ident_start(bytes[i]) {
            let start = i;
            i += 1;
            while i < bytes.len() && is_ident_cont(bytes[i]) {
                i += 1;
            }
            let name = &source[start..i];
            // skip ws
            let mut j = i;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b':' {
                j += 1;
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j < bytes.len() && bytes[j] == b'{' {
                    if let Some(end) = brace_end(source, j) {
                        let body = source[j + 1..end].to_string();
                        // Heuristic: skill objects usually contain trigger/enable/content/mod/audio
                        if looks_like_skill(&body) {
                            out.push((name.to_string(), body));
                        }
                        i = end + 1;
                        continue;
                    }
                }
            }
            continue;
        }
        i += 1;
    }
    out
}

fn looks_like_skill(body: &str) -> bool {
    body.contains("trigger")
        || body.contains("enable")
        || body.contains("content")
        || body.contains("mod")
        || body.contains("viewAs")
        || body.contains("audio")
        || body.contains("filter")
        || body.contains("forced")
}

fn brace_end(s: &str, open: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    if open >= bytes.len() || bytes[open] != b'{' {
        return None;
    }
    let mut depth = 0i32;
    let mut i = open;
    let mut in_str: Option<u8> = None;
    while i < bytes.len() {
        let c = bytes[i];
        if let Some(q) = in_str {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == q {
                in_str = None;
            }
            i += 1;
            continue;
        }
        match c {
            b'"' | b'\'' | b'`' => {
                in_str = Some(c);
                i += 1;
            }
            b'{' => {
                depth += 1;
                i += 1;
            }
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
                i += 1;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                i += 2;
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }
    None
}

fn is_ident_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_' || c == b'$'
}
fn is_ident_cont(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'$'
}

fn profile_skill(name: &str, body: &str) -> SkillProfile {
    let triggers = extract_triggers(body);
    let enable = extract_enable(body);
    let capability_hits = detect_capabilities(body);
    let mapped_events = triggers
        .iter()
        .flat_map(|t| t.events.iter().map(|e| map_event(e)))
        .filter(|e| !e.is_empty())
        .collect();
    let mapped_effects = detect_mapped_effects(body);
    let content_snippet = extract_function_body(body, "content")
        .or_else(|| extract_function_body(body, "async content"))
        .unwrap_or_default();
    let raw_span: String = body.chars().take(400).collect();

    SkillProfile {
        name: name.to_string(),
        triggers,
        enable,
        forced: field_bool(body, "forced"),
        frequent: field_bool(body, "frequent"),
        direct: field_bool(body, "direct"),
        has_mod: body.contains("mod:") || body.contains("mod :"),
        has_view_as: body.contains("viewAs") || body.contains("viewAsFilter"),
        has_filter: body.contains("filter(")
            || body.contains("filter (")
            || body.contains("filter:"),
        has_cost: body.contains("cost(") || body.contains("async cost") || body.contains("cost:"),
        has_content: body.contains("content(")
            || body.contains("async content")
            || body.contains("content:"),
        has_check: body.contains("check(") || body.contains("check:"),
        has_group: body.contains("group:") || body.contains("subSkill"),
        has_ai: body.contains("ai:") || body.contains("ai :"),
        has_audio: body.contains("audio"),
        zhu_skill: body.contains("zhuSkill"),
        content_snippet: content_snippet.chars().take(300).collect(),
        raw_span,
        capability_hits,
        mapped_events,
        mapped_effects,
        mod_hints: extract_mod_hints(body),
    }
}

/// Recognize common `mod { globalFrom/globalTo/attackRange ... return distance ± N }`
/// shapes into structured [`ModHint`]s. Heuristic; unknown shapes yield nothing.
fn extract_mod_hints(body: &str) -> Vec<ModHint> {
    if !(body.contains("mod:") || body.contains("mod :")) {
        return Vec::new();
    }
    let mut hints = Vec::new();
    // (legacy mod key, DSL target)
    const MAP: &[(&str, &str)] = &[
        ("globalFrom", "distance_from"),
        ("globalTo", "distance_to"),
        ("attackFrom", "distance_from"),
        ("attackRange", "attack_range"),
        ("maxHandcard", "hand_limit"),
    ];
    for (legacy, target) in MAP {
        // Find `<legacy> ... return <expr>` and read a trailing ± integer.
        let Some(key_pos) = body.find(legacy) else {
            continue;
        };
        let tail = &body[key_pos..];
        let Some(ret_pos) = tail.find("return") else {
            continue;
        };
        // Scan the return expression up to the next `;` or `}`.
        let expr_start = ret_pos + "return".len();
        let expr_end = tail[expr_start..]
            .find([';', '}'])
            .map(|i| expr_start + i)
            .unwrap_or(tail.len());
        let expr = &tail[expr_start..expr_end];
        if let Some(delta) = parse_distance_delta(expr) {
            if delta != 0 {
                hints.push(ModHint {
                    target: (*target).to_string(),
                    mode: "add".to_string(),
                    value: delta,
                });
            }
        }
    }
    hints
}

/// Parse a `distance - 1` / `num + 2` style expression into its signed delta.
fn parse_distance_delta(expr: &str) -> Option<i32> {
    // Look for the last `+`/`-` operator followed by an integer literal.
    let bytes = expr.as_bytes();
    let mut i = 0;
    let mut result: Option<i32> = None;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'+' || c == b'-' {
            let sign = if c == b'-' { -1 } else { 1 };
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] == b' ' {
                j += 1;
            }
            let num_start = j;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            if j > num_start {
                if let Ok(n) = expr[num_start..j].parse::<i32>() {
                    result = Some(sign * n);
                }
            }
        }
        i += 1;
    }
    result
}

fn field_bool(body: &str, field: &str) -> bool {
    let re = Regex::new(&format!(r"{field}\s*:\s*true")).unwrap();
    re.is_match(body)
}

fn extract_triggers(body: &str) -> Vec<TriggerSpec> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r#"trigger\s*:\s*\{([^}]*)\}"#).unwrap());
    let mut out = Vec::new();
    if let Some(cap) = re.captures(body) {
        let inner = cap.get(1).unwrap().as_str();
        // role: "event" or role: ["a","b"]
        let role_re = Regex::new(
            r#"(player|source|target|global)\s*:\s*(?:"([^"]+)"|'([^']+)'|\[([^\]]*)\])"#,
        )
        .unwrap();
        for m in role_re.captures_iter(inner) {
            let role = m.get(1).unwrap().as_str().to_string();
            let mut events = Vec::new();
            if let Some(s) = m.get(2).or(m.get(3)) {
                events.push(s.as_str().to_string());
            } else if let Some(arr) = m.get(4) {
                let str_re = Regex::new(r#""([^"]+)"|'([^']+)'"#).unwrap();
                for s in str_re.captures_iter(arr.as_str()) {
                    if let Some(e) = s.get(1).or(s.get(2)) {
                        events.push(e.as_str().to_string());
                    }
                }
            }
            if !events.is_empty() {
                out.push(TriggerSpec { role, events });
            }
        }
    }
    // multiline trigger: { player: "x", } already covered if no nested braces deeply
    // Also: trigger: { player: ["a", "b"], }
    out
}

fn extract_enable(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let re = Regex::new(r#"enable\s*:\s*(?:"([^"]+)"|'([^']+)'|\[([^\]]*)\])"#).unwrap();
    if let Some(c) = re.captures(body) {
        if let Some(s) = c.get(1).or(c.get(2)) {
            out.push(s.as_str().to_string());
        } else if let Some(arr) = c.get(3) {
            let str_re = Regex::new(r#""([^"]+)"|'([^']+)'"#).unwrap();
            for s in str_re.captures_iter(arr.as_str()) {
                if let Some(e) = s.get(1).or(s.get(2)) {
                    out.push(e.as_str().to_string());
                }
            }
        }
    }
    out
}

fn extract_function_body(body: &str, marker: &str) -> Option<String> {
    let idx = body.find(marker)?;
    let rest = &body[idx..];
    let brace = rest.find('{')?;
    let abs = idx + brace;
    let end = brace_end(body, abs)?;
    Some(body[abs + 1..end].to_string())
}

/// Patterns that indicate host / unsafe capabilities (→ grade D pressure).
fn detect_capabilities(body: &str) -> Vec<String> {
    let patterns: &[(&str, &str)] = &[
        ("document", "dom"),
        ("window.", "dom"),
        ("localStorage", "storage"),
        ("sessionStorage", "storage"),
        ("fetch(", "network"),
        ("XMLHttpRequest", "network"),
        ("WebSocket", "network"),
        ("eval(", "eval"),
        ("Function(", "eval"),
        ("import(", "dynamic_import"),
        ("require(", "require"),
        ("process.", "node"),
        ("fs.", "filesystem"),
        ("child_process", "process"),
        ("ui.", "ui_global"),
        ("game.download", "io"),
        ("game.writeFile", "filesystem"),
        ("game.readFile", "filesystem"),
        ("lib.element", "dom_element"),
        ("createElement", "dom"),
        ("innerHTML", "dom"),
        ("_status.", "status_global"),
        ("setTimeout", "async_timer"),
        ("setInterval", "async_timer"),
        ("Date.now", "clock"),
        ("Math.random", "nondeterministic_rng"),
        ("performance.now", "clock"),
    ];
    let mut hits = Vec::new();
    for (pat, tag) in patterns {
        if body.contains(pat) {
            hits.push((*tag).to_string());
        }
    }
    hits.sort();
    hits.dedup();
    hits
}

fn detect_mapped_effects(body: &str) -> Vec<String> {
    let mut v = Vec::new();
    if body.contains("player.draw") || body.contains(".draw(") {
        v.push("draw_cards".into());
    }
    if body.contains(".damage(") || body.contains("player.damage") {
        v.push("deal_damage".into());
    }
    if body.contains(".recover(") || body.contains("player.recover") {
        v.push("recover_hp".into());
    }
    if body.contains(".gain(") || body.contains("player.gain") {
        v.push("gain_cards".into());
    }
    if body.contains(".discard(") || body.contains("chooseToDiscard") {
        v.push("discard".into());
    }
    if body.contains(".die(") || body.contains(".kill(") {
        v.push("kill".into());
    }
    v
}

/// Map noname event name → next DSL event token (empty if unknown).
pub fn map_event(noname_event: &str) -> String {
    match noname_event {
        "damageEnd" | "damageAfter" | "damage" => "damage".into(),
        "damageBegin" | "damageBegin1" | "damageBegin2" | "damageBegin3" | "damageBegin4" => {
            "damage".into()
        }
        "phaseBegin" => "phase_begin".into(),
        "phaseZhunbeiBegin" => "phase_prepare".into(),
        "phaseJudgeBegin" => "phase_judge".into(),
        "phaseDrawBegin" | "phaseDrawBegin1" | "phaseDrawBegin2" => "phase_draw".into(),
        "phaseUseBegin" => "phase_play".into(),
        "phaseDiscardBegin" => "phase_discard".into(),
        "phaseJieshuBegin" | "phaseJieshu" | "phaseEnd" => "phase_end".into(),
        "useCard" | "useCardEnd" | "useCardAfter" => "card_used".into(),
        "dying" | "dyingBegin" => "dying".into(),
        "die" | "dieAfter" => "death".into(),
        "recoverEnd" | "recoverAfter" => "damage".into(), // rough
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
const skills = {
  jianxiong: {
    audio: 2,
    trigger: { player: "damageEnd" },
    filter(event, player) {
      return event.num > 0;
    },
    async content(event, trigger, player) {
      await player.draw();
    },
  },
  bad_dom: {
    enable: "phaseUse",
    content(event, player) {
      document.body.appendChild(x);
      fetch("/x");
    },
  },
};
"#;

    #[test]
    fn extracts_jianxiong() {
        let profiles = analyze_source(SAMPLE).unwrap();
        assert!(profiles.iter().any(|p| p.name == "jianxiong"));
        let j = profiles.iter().find(|p| p.name == "jianxiong").unwrap();
        assert!(j.forced == false);
        assert!(j.has_content);
        assert!(!j.triggers.is_empty());
        assert_eq!(j.triggers[0].role, "player");
        assert!(j.triggers[0].events.iter().any(|e| e == "damageEnd"));
        assert!(j.mapped_effects.contains(&"draw_cards".into()));
    }

    #[test]
    fn detects_dom_network() {
        let profiles = analyze_source(SAMPLE).unwrap();
        let b = profiles.iter().find(|p| p.name == "bad_dom").unwrap();
        assert!(b.capability_hits.iter().any(|h| h == "dom"));
        assert!(b.capability_hits.iter().any(|h| h == "network"));
    }

    #[test]
    fn unknown_legacy_event_is_not_invented() {
        assert_eq!(map_event("turnOverEnd"), "");
    }
}
