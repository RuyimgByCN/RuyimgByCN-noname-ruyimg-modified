//! DSL Language Server Protocol implementation (architecture §35).
//!
//! Provides the three core LSP methods for the skill DSL:
//! - `textDocument/completion` — context-aware keyword/identifier completion
//! - `textDocument/hover` — documentation for keywords and constructs
//! - `textDocument/diagnostics` — compile errors as LSP diagnostics
//!
//! This is a library, not a server binary. The actual LSP server (stdio JSON-RPC)
//! is a thin wrapper that calls these functions.

mod completions;
mod hover_docs;

use content_schema::Span;
use serde::{Deserialize, Serialize};

// ---- LSP types (subset of the Language Server Protocol) ----

/// LSP Position (0-based line and character).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LspPosition {
    /// Line (0-based).
    pub line: u32,
    /// Character (0-based, UTF-16 offset).
    pub character: u32,
}

/// LSP Range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LspRange {
    /// Start.
    pub start: LspPosition,
    /// End.
    pub end: LspPosition,
}

/// LSP Severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum LspSeverity {
    /// Error.
    Error = 1,
    /// Warning.
    Warning = 2,
    /// Information.
    Information = 3,
    /// Hint.
    Hint = 4,
}

/// LSP Diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LspDiagnostic {
    /// Range in the document.
    pub range: LspRange,
    /// Severity.
    pub severity: LspSeverity,
    /// Source (always "noname-dsl").
    pub source: String,
    /// Message.
    pub message: String,
}

/// LSP Completion Item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LspCompletionItem {
    /// Label (text to insert).
    pub label: String,
    /// Kind (1=Text, 14=Keyword, 15=Snippet).
    pub kind: u32,
    /// Detail (short description).
    pub detail: String,
    /// Insert text (may differ from label for snippets).
    pub insert_text: String,
}

/// LSP Hover result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LspHover {
    /// Contents (markdown).
    pub contents: String,
    /// Range of the hovered symbol.
    pub range: Option<LspRange>,
}

// ---- Byte offset ↔ LSP position conversion ----

/// Convert a byte offset to an LSP position (0-based line/char).
pub fn byte_offset_to_position(source: &str, offset: usize) -> LspPosition {
    let mut line = 0u32;
    let mut last_line_start = 0usize;
    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            last_line_start = i + 1;
        }
    }
    LspPosition {
        line,
        character: (offset.saturating_sub(last_line_start)) as u32,
    }
}

/// Convert a Span to an LSP range.
pub fn span_to_range(source: &str, span: &Span) -> LspRange {
    LspRange {
        start: byte_offset_to_position(source, span.start as usize),
        end: byte_offset_to_position(source, span.end as usize),
    }
}

// ---- textDocument/diagnostics ----

/// Produce LSP diagnostics for a DSL source file by running the compiler.
pub fn diagnostics(source: &str) -> Vec<LspDiagnostic> {
    match crate::compile_source(source) {
        Ok(_) => vec![],
        Err(e) => {
            let (pos, message) = match &e {
                crate::CompileError::Lex { pos, message } => (*pos, message.clone()),
                crate::CompileError::Parse { pos, message } => (*pos, message.clone()),
                crate::CompileError::Type { pos, message } => (*pos, message.clone()),
                crate::CompileError::Codegen(msg) => (0, msg.clone()),
            };
            let start = byte_offset_to_position(source, pos);
            let end = LspPosition {
                line: start.line,
                character: start.character + 1,
            };
            vec![LspDiagnostic {
                range: LspRange { start, end },
                severity: LspSeverity::Error,
                source: "noname-dsl".into(),
                message,
            }]
        }
    }
}

// ---- textDocument/completion ----

/// Context: what kind of DSL construct the cursor is inside.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompletionContext {
    /// Top-level (expecting "skill").
    TopLevel,
    /// Inside skill body (expecting trigger/modifier/active/view_as).
    SkillBody,
    /// Inside trigger body (expecting effect/control flow).
    TriggerBody,
    /// After "trigger" keyword (expecting event name).
    TriggerEvent,
    /// After "modifier" keyword (expecting target name).
    ModifierTarget,
    /// After modifier target (expecting mode).
    ModifierMode,
    /// Unknown / no useful completions.
    None,
}

/// Detect completion context from source + cursor byte offset.
fn detect_context(source: &str, offset: usize) -> CompletionContext {
    let before = &source[..offset.min(source.len())];
    let trimmed = before.trim_end();

    // Count brace depth from raw characters.
    let mut depth = 0i32;
    for ch in before.chars() {
        match ch {
            '{' => depth += 1,
            '}' => depth -= 1,
            _ => {}
        }
    }

    if depth <= 0 {
        return CompletionContext::TopLevel;
    }

    // Find the last word before the cursor (for keyword detection).
    let last_word: String = trimmed
        .chars()
        .rev()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>()
        .chars()
        .rev()
        .collect();

    if depth == 1 {
        // Inside skill body.
        match last_word.as_str() {
            "trigger" => return CompletionContext::TriggerEvent,
            "modifier" => return CompletionContext::ModifierTarget,
            "distance_from"
            | "global_distance_to"
            | "attack_range"
            | "global_attack_range"
            | "global_damage_taken_amount"
            | "global_damage_taken_final_amount"
            | "global_hand_limit"
            | "hand_limit"
            | "hand_limit_base"
            | "hand_limit_base_max_hp"
            | "card_suit"
            | "card_name"
            | "card_nature"
            | "sha_unrespondable"
            | "card_use_unrespondable"
            | "sha_unrespondable_within_distance"
            | "sha_use_unrespondable_within_distance"
            | "card_use_unrespondable_within_responder_to_owner_distance"
            | "card_use_unrespondable_within_owner_to_responder_distance"
            | "sha_target_unrespondable"
            | "damage_taken_source_range_threshold"
            | "card_distance_unlimited"
            | "sha_distance_unlimited_if_rank_lt_skill_mark"
            | "sha_distance_unlimited_if_target_distance_lte_rank"
            | "sha_no_use_count_if_rank_gt_skill_mark"
            | "card_discard_protected"
            | "card_gain_protected"
            | "card_targets_all"
            | "card_target_cancel"
            | "armor_card_target_cancel"
            | "armor_link_gain_cancel"
            | "link_loss_cancel"
            | "card_target_cancel_from_current"
            | "card_target_cancel_if_other_user_hp_gte_target"
            | "card_target_cancel_if_other_user_distance_gt_one"
            | "card_targets_self_only"
            | "card_use_cancel"
            | "card_response_cancel"
            | "card_save_cancel"
            | "sha_response_cancel"
            | "sha_ignored_hand_limit"
            | "card_ignored_hand_limit"
            | "card_use_unlimited"
            | "card_target_limit"
            | "observer_foreign_hand_faces_visible"
            | "observer_hidden_identities_visible"
            | "skip_judge_phase"
            | "skip_discard_phase"
            | "restore_skipped_phase"
            | "turn_face_down_cancel"
            | "turn_over_cancel"
            | "discard_phase_includes_equip"
            | "draw_phase_scry_four_split_two"
            | "initial_hand_count"
            | "sha_use_cancel"
            | "armor_damage_taken_amount"
            | "ignore_armor"
            | "armor_effect_disabled" => return CompletionContext::ModifierMode,
            _ => return CompletionContext::SkillBody,
        }
    }

    if depth >= 2 {
        // Inside trigger body or nested block.
        return CompletionContext::TriggerBody;
    }

    CompletionContext::None
}

/// Provide completion items for a cursor position.
pub fn completion(source: &str, offset: usize) -> Vec<LspCompletionItem> {
    let ctx = detect_context(source, offset);
    match ctx {
        CompletionContext::TopLevel => completions::top_level::items(),
        CompletionContext::SkillBody => completions::skill_body::items(),
        CompletionContext::TriggerEvent => completions::trigger_event::items(),
        CompletionContext::TriggerBody => completions::trigger_body::items(),
        CompletionContext::ModifierTarget => completions::modifier_target::items(),
        CompletionContext::ModifierMode => completions::modifier_mode::items(),
        CompletionContext::None => vec![],
    }
}

// ---- textDocument/hover ----

/// Provide hover documentation for a keyword at a given offset.
pub fn hover(source: &str, offset: usize) -> Option<LspHover> {
    // Extract the word at the cursor position.
    let before = &source[..offset.min(source.len())];
    let after = &source[offset.min(source.len())..];

    // Find word boundary going backwards.
    let start = before
        .char_indices()
        .rev()
        .take_while(|(_, c)| c.is_alphanumeric() || *c == '_')
        .last()
        .map(|(i, _)| i)
        .unwrap_or(before.len());

    // Find word boundary going forwards.
    let end_rel = after
        .char_indices()
        .take_while(|(_, c)| c.is_alphanumeric() || *c == '_')
        .last()
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(0);

    let word = &source[start..offset + end_rel];

    let doc = if word == "owner_targeted_by_nanman" {
        "Trigger filter: fires when the living owner becomes the target of a Nanman invasion (sp xinfu_zhennan)."
    } else if word == "owner_in_multitarget_sha_or_trick_targets" {
        "Trigger filter: fires when a sha or ordinary trick with more than one declared target is used and the living owner is among those targets (sp danlao)."
    } else if word == "owner_black_card_moved_equip_judge" {
        "Trigger filter: fires when a black card enters or leaves the living owner's equipment or judgment zone (sp2 liangjue)."
    } else if word
        .strip_prefix("owner_discard_suit_no_own_mark_")
        .is_some()
    {
        "Trigger filter: fires when a card of the suit encoded in the token moves to the discard pile from the living owner's zones while they lack the matching falu mark (sp xinfu_falu family)."
    } else if word == "owner_alive_damage_card_nanman" {
        "Trigger filter: fires when damage dealt by a Nanman invasion card is recorded, whoever the target, while the owner is alive (xianding mansi)."
    } else if word == "owner_alive_other_dying_has_equip_card" {
        "Trigger filter: fires when another living player enters dying while the living owner holds an equip-kind card in hand or equipment zone (onlyOL olrenxin)."
    } else if word == "owner_source_trick_resolved_own_play_phase_once_per_turn" {
        "Trigger filter: fires when an ordinary trick the living owner used finishes resolving during their own play phase, once per turn (sp2 xinfu_yisuan)."
    } else if word == "owner_targeted_source_hp_hand_differ_once_per_turn" {
        "Trigger filter: fires before damage when the living owner is the target, the source is alive, and the source's HP and hand count both differ from the owner's, once per turn (sp2 yinyi)."
    } else if word == "owner_gained_cards_during_play_phase_wounded" {
        "Trigger filter: fires on the living owner's own card gains (draw or move into hand) while it is their play phase and they are wounded (collab haoxue recover arm)."
    } else if word == "owner_discarded_outside_play_phase" {
        "Trigger filter: fires when the living owner discards hand or equip cards to the discard pile outside their own play phase (collab haoxue hp-loss arm)."
    } else if word == "owner_source_play_card_count_eq_hp" {
        "Trigger filter: fires when the living owner proactively uses the card whose Play-phase ordinal equals their current HP."
    } else if word
        .strip_prefix("owner_source_play_card_count_eq_")
        .is_some_and(|count| count.parse::<u32>().is_ok_and(|value| value > 0))
    {
        "Parameterized trigger filter: fires when the living owner proactively uses the suffix-numbered card in their current play phase. Responses do not count, and a new play phase resets the counter."
    } else if word
        .strip_prefix("owner_is_target_source_alive_mark_positive_")
        .is_some_and(|mark| mark.parse::<u32>().is_ok_and(|value| value > 0))
    {
        "Parameterized trigger filter: fires when the marked owner is the damage target and the source remains alive."
    } else if word
        .strip_prefix("owner_is_target_active_used_at_least_")
        .and_then(|value| value.strip_suffix("_this_play_phase"))
        .is_some_and(|threshold| threshold.parse::<u32>().is_ok_and(|value| value > 0))
    {
        "Parameterized trigger filter: fires when the owner is ending the play phase and this active skill was invoked at least the suffix number of times in that same play phase."
    } else if word
        .strip_prefix("owner_is_target_hp_below_")
        .is_some_and(|threshold| threshold.parse::<u32>().is_ok_and(|value| value > 0))
    {
        "Parameterized trigger filter: fires when the living skill owner is the event player and their current HP is below the positive threshold in the suffix."
    } else if word
        .strip_prefix("owner_is_target_mark_positive_")
        .is_some_and(|mark| mark.parse::<u32>().is_ok())
    {
        "Parameterized trigger filter: fires when the living skill owner is the event player and has at least one stack of the stable mark id in the suffix."
    } else if word
        .strip_prefix("owner_mark_positive_")
        .is_some_and(|mark| mark.parse::<u32>().is_ok())
    {
        "Parameterized trigger filter: fires while the living skill owner has at least one stack of the stable mark id in the suffix."
    } else if word
        .strip_prefix("owner_skill_added_self_")
        .is_some_and(|skill_def| skill_def.parse::<u32>().is_ok())
    {
        "Parameterized trigger filter: fires when the living skill owner just gained the skill with the stable def id in the suffix. Implements 州郡 skills (sp/px*)."
    } else if word
        .strip_prefix("owner_source_ordinary_trick_mark_positive_")
        .is_some_and(|mark| mark.parse::<u32>().is_ok())
    {
        "Parameterized trigger filter: fires when the skill owner used an ordinary trick and has at least one stack of the stable mark id in the suffix."
    } else if word
        .strip_prefix("owner_source_sha_or_black_trick_mark_positive_")
        .is_some_and(|mark| mark.parse::<u32>().is_ok())
    {
        "Parameterized trigger filter: fires when the skill owner used a Sha or a black ordinary trick and has at least one stack of the stable mark id in the suffix. Implements 乱战·增 (reluanzhan_add)."
    } else if word
        .strip_prefix("owner_source_damage_target_mark_positive_")
        .and_then(|value| value.strip_suffix("_once"))
        .is_some_and(|mark| mark.parse::<u32>().is_ok())
    {
        "Parameterized damage trigger filter: the living owner dealt damage to a living player carrying the given mark, once per turn. Implements 政略③ (jsrgzhenglve)."
    } else if word
        .strip_prefix("owner_alive_marked_alive_above_two_")
        .is_some_and(|mark| mark.parse::<u32>().is_ok())
    {
        "Parameterized prepare-phase trigger filter: own prepare phase with more than two marked living players. Implements 会猎 (jsrghuilie) awakening gate."
    } else if word
        .strip_prefix("owner_alive_someone_mark_positive_")
        .is_some_and(|mark| mark.parse::<u32>().is_ok())
    {
        "Parameterized phase-end trigger filter: some living player carries the given mark. Implements 平戎 gate (jsrgpingrong)."
    } else if word
        .strip_prefix("owner_alive_discard_matches_own_pile_card_")
        .is_some_and(|pile_def| pile_def.parse::<u32>().is_ok())
    {
        "Parameterized cards-moved trigger filter: a discard event where some card matches the suit and rank of the first card in the owner's own skill pile. Implements 表召·应 gate (biaozhao2)."
    } else if word
        .strip_prefix("owner_alive_damage_before_event_target_mark_positive_")
        .is_some_and(|mark| mark.parse::<u32>().is_ok())
    {
        "Parameterized damage-before trigger filter: the living owner observes damage about to land on a player carrying the given mark. Implements 宽释 protection (kuanshi)."
    } else if word
        .strip_prefix("owner_alive_phase_begin_event_target_mark_positive_")
        .is_some_and(|mark| mark.parse::<u32>().is_ok())
    {
        "Parameterized prepare-phase trigger filter: the living owner observes a turn starting for a player carrying the given mark. Implements 宽释 expiry (kuanshi)."
    } else if word
        .strip_prefix("owner_alive_sha_resolved_city_")
        .is_some_and(|skill_def| skill_def.parse::<u32>().is_ok())
    {
        "Parameterized Sha-resolved trigger filter: the living owner still holds City cards for the given skill and some other player lost cards this turn. Implements 沙城② (jsrgshacheng)."
    } else if word
        .strip_prefix("owner_alive_event_target_mark_positive_")
        .and_then(|value| value.strip_suffix("_no_damage"))
        .is_some_and(|mark| mark.parse::<u32>().is_ok())
    {
        "Parameterized turn-end trigger filter: the ending turn player carries the given mark and dealt no damage this turn; the living owner gains an extra-turn window. Implements 归离 (xianding guili) recovery branch."
    } else if word
        .strip_prefix("owner_source_hand_gaintag_absent_")
        .is_some_and(|tag| tag.parse::<u32>().is_ok())
    {
        "Parameterized card-use trigger filter: the living owner uses a card while holding no card carrying the given gaintag. Implements 兴门 (collab natuxing)."
    } else if word
        .strip_prefix("judgment_rewrite_hand_equip_cost_mark_")
        .is_some_and(|mark| mark.parse::<u32>().is_ok_and(|mark| mark > 0))
    {
        "Parameterized modifier target: permit a judgment rewrite from hand or equipment and consume one stack of the stable mark id in the suffix after a successful rewrite. Requires `override 1` and Rule VM ABI 218."
    } else if word
        .strip_prefix("alive_wounded_count_at_least_")
        .is_some_and(|threshold| threshold.parse::<u32>().is_ok_and(|value| value > 0))
    {
        "Parameterized query-time modifier value: one while at least the suffix number of living players are wounded, otherwise zero."
    } else if word
        .strip_prefix("owner_equip_count_at_least_")
        .or_else(|| source.strip_prefix("owner_equip_count_floor_"))
        .is_some_and(|threshold| threshold.parse::<u32>().is_ok_and(|value| value > 0))
    {
        "Parameterized query-time modifier value: the owner's equipment-card count when it is at least the suffix number, otherwise zero."
    } else if word
        .strip_prefix("owner_dying_other_tao_responder_group_")
        .is_some_and(|group| !group.is_empty())
    {
        "Parameterized CardResponded filter: the living owner is dying, another living responder of the suffix group answered that owner with Tao, and the trigger resolves before the base dying-save recovery. Requires Rule VM ABI 185."
    } else if word
        .strip_prefix("owner_alive_other_shan_outside_turn_group_")
        .and_then(|value| value.strip_suffix("_once_per_turn"))
        .is_some_and(|group| !group.is_empty())
    {
        "Parameterized CardUsed/CardResponded filter: a living owner observes another living player of the embedded group use or respond with Shan outside that player's turn; fires at most once in the current turn. Requires Rule VM ABI 199."
    } else if word
        .strip_prefix("group_")
        .is_some_and(|group| !group.is_empty())
    {
        "Parameterized delegated card source: ask other living characters of the suffix group in seat order to provide the declared card. `shan response` requires Rule VM ABI 184; `sha any` requires ABI 187."
    } else {
        hover_docs::documentation(word)?
    };

    Some(LspHover {
        contents: format!("```noname-dsl\n{word}\n```\n\n{doc}"),
        range: Some(LspRange {
            start: byte_offset_to_position(source, start),
            end: byte_offset_to_position(source, offset + end_rel),
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostics_for_valid_source() {
        let src =
            "skill test(1) { trigger damage owner_is_target { effect draw_cards(self, 1); } }";
        let diags = diagnostics(src);
        assert!(diags.is_empty(), "valid source should have no diagnostics");
    }

    #[test]
    fn diagnostics_for_invalid_source() {
        let src = "skill { }";
        let diags = diagnostics(src);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].severity, LspSeverity::Error);
        assert_eq!(diags[0].source, "noname-dsl");
    }

    #[test]
    fn completion_at_top_level() {
        let src = "";
        let items = completion(src, 0);
        assert!(items.iter().any(|i| i.label == "skill"));
    }

    #[test]
    fn completion_inside_skill_body() {
        let src = "skill test(1) {\n\t";
        let items = completion(src, src.len());
        assert!(items.iter().any(|i| i.label == "trigger"));
        assert!(items.iter().any(|i| i.label == "modifier"));
    }

    #[test]
    fn completion_after_trigger_keyword() {
        let src = "skill test(1) {\n\ttrigger ";
        let items = completion(src, src.len());
        assert!(items.iter().any(|i| i.label == "damage"));
        assert!(items.iter().any(|i| i.label == "phase_begin"));
    }

    #[test]
    fn completion_after_modifier_keyword() {
        let src = "skill test(1) {\n\tmodifier ";
        let items = completion(src, src.len());
        assert!(items.iter().any(|i| i.label == "distance_from"));
        assert!(items.iter().any(|i| i.label == "attack_range"));
    }

    #[test]
    fn completion_for_modifier_mode() {
        let src = "skill test(1) {\n\tmodifier distance_from ";
        let items = completion(src, src.len());
        assert!(items.iter().any(|i| i.label == "add"));
        assert!(items.iter().any(|i| i.label == "override"));
    }

    #[test]
    fn completion_in_trigger_body() {
        let src = "skill test(1) {\n\ttrigger damage owner_is_target {\n\t\t";
        let items = completion(src, src.len());
        assert!(items.iter().any(|i| i.label == "effect draw_cards"));
        assert!(items
            .iter()
            .any(|i| i.label == "effect discard_one_hand_or_equip"));
        assert!(items
            .iter()
            .any(|i| i.label == "effect discard_all_hand_and_equip"));
        assert!(items.iter().any(|i| i.label == "effect remove_all_skills"));
        assert!(items
            .iter()
            .any(|i| i.label == "effect choose_other_recover"));
        assert!(items.iter().any(|i| i.label == "effect enable_equip_slot"));
        assert!(items.iter().any(|i| i.label == "effect gain_card_by_kind"));
    }

    #[test]
    fn hover_for_skill_keyword() {
        let src = "skill test(1) {}";
        let h = hover(src, 2).unwrap(); // cursor on "skill"
        assert!(h.contents.contains("Define a skill"));
    }

    #[test]
    fn hover_for_trigger_keyword() {
        let src = "skill test(1) { trigger damage x {} }";
        // Find "trigger" position
        let pos = src.find("trigger").unwrap();
        let h = hover(src, pos + 2).unwrap();
        assert!(h.contents.contains("Define a triggered ability"));
    }

    #[test]
    fn hover_for_damage_event() {
        let src = "skill test(1) { trigger damage x {} }";
        let pos = src.find("damage").unwrap();
        let h = hover(src, pos + 2).unwrap();
        assert!(h.contents.contains("fires after damage"));
    }

    #[test]
    fn hover_for_draw_cards() {
        let src = "skill test(1) { trigger damage x { effect draw_cards(self, 1); } }";
        let pos = src.find("draw_cards").unwrap();
        let h = hover(src, pos + 3).unwrap();
        assert!(h.contents.contains("draws N cards"));
    }

    #[test]
    fn hover_for_choose_other_recover() {
        let src = "skill test(1) { trigger death owner_is_source { effect choose_other_recover(self, 2); } }";
        let pos = src.find("choose_other_recover").unwrap();
        let h = hover(src, pos + 3).unwrap();
        assert!(h.contents.contains("forces `player`"));
        assert!(h.contents.contains("ABI 160"));
    }

    #[test]
    fn hover_for_enable_equip_slot() {
        let src = "skill test(1) { trigger damage owner_is_target_source_alive_disabled_weapon { effect enable_equip_slot(event_source, 1, self); } }";
        let pos = src.find("enable_equip_slot").unwrap();
        let h = hover(src, pos + 3).unwrap();
        assert!(h.contents.contains("disabled copy"));
        assert!(h.contents.contains("ABI 161"));
    }

    #[test]
    fn hover_for_parameterized_owner_equip_threshold() {
        let src = "skill test(1) { modifier draw_count add owner_equip_count_at_least_4; }";
        let pos = src.find("owner_equip_count_at_least_4").unwrap();
        let h = hover(src, pos + 5).unwrap();
        assert!(h.contents.contains("equipment-card count"));
        assert!(h.contents.contains("otherwise zero"));
    }

    #[test]
    fn hover_returns_none_for_unknown_word() {
        let src = "skill test(1) {}";
        // Hover on "test" (a user-defined name, not a keyword)
        let pos = src.find("test").unwrap();
        assert!(hover(src, pos + 1).is_none());
    }

    #[test]
    fn byte_offset_to_position_multiline() {
        let src = "line1\nline2\nline3";
        let pos = byte_offset_to_position(src, 12); // start of "line3"
        assert_eq!(pos.line, 2);
        assert_eq!(pos.character, 0);
    }

    #[test]
    fn byte_offset_to_position_same_line() {
        let src = "hello world";
        let pos = byte_offset_to_position(src, 6); // start of "world"
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 6);
    }
}
