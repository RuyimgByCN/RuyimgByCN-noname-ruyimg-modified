//! DSL Language Server Protocol implementation (architecture §35).
//!
//! Provides the three core LSP methods for the skill DSL:
//! - `textDocument/completion` — context-aware keyword/identifier completion
//! - `textDocument/hover` — documentation for keywords and constructs
//! - `textDocument/diagnostics` — compile errors as LSP diagnostics
//!
//! This is a library, not a server binary. The actual LSP server (stdio JSON-RPC)
//! is a thin wrapper that calls these functions.

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
        CompletionContext::TopLevel => vec![LspCompletionItem {
            label: "skill".into(),
            kind: 14, // Keyword
            detail: "Define a skill".into(),
            insert_text: "skill ${1:name}(${2:def_id}) {\n\t$0\n}".into(),
        }],
        CompletionContext::SkillBody => vec![
            LspCompletionItem {
                label: "trigger".into(),
                kind: 14,
                detail: "Define a triggered skill".into(),
                insert_text: "trigger ${1:event} ${2:filter} {\n\t$0\n}".into(),
            },
            LspCompletionItem {
                label: "modifier".into(),
                kind: 14,
                detail: "Define a passive modifier".into(),
                insert_text: "modifier ${1:target} ${2:mode} ${3:0};".into(),
            },
        ],
        CompletionContext::TriggerEvent => vec![
            LspCompletionItem {
                label: "damage".into(),
                kind: 14,
                detail: "After damage is dealt".into(),
                insert_text: "damage".into(),
            },
            LspCompletionItem {
                label: "hp_lost".into(),
                kind: 14,
                detail: "After HP is lost without damage".into(),
                insert_text: "hp_lost".into(),
            },
            LspCompletionItem {
                label: "phase_prepare".into(),
                kind: 14,
                detail: "At the start of the preparation phase".into(),
                insert_text: "phase_prepare".into(),
            },
            LspCompletionItem {
                label: "phase_begin".into(),
                kind: 14,
                detail: "At the start of a phase".into(),
                insert_text: "phase_begin".into(),
            },
            LspCompletionItem {
                label: "phase_end".into(),
                kind: 14,
                detail: "At the end of a phase".into(),
                insert_text: "phase_end".into(),
            },
            LspCompletionItem {
                label: "phase_draw_end".into(),
                kind: 14,
                detail: "When the draw phase ends".into(),
                insert_text: "phase_draw_end".into(),
            },
            LspCompletionItem {
                label: "phase_play_end".into(),
                kind: 14,
                detail: "When the play phase ends".into(),
                insert_text: "phase_play_end".into(),
            },
            LspCompletionItem {
                label: "phase_any_end".into(),
                kind: 14,
                detail: "When any concrete turn phase ends".into(),
                insert_text: "phase_any_end".into(),
            },
            LspCompletionItem {
                label: "phase_draw_skipped".into(),
                kind: 14,
                detail: "When a draw phase is skipped or cancelled".into(),
                insert_text: "phase_draw_skipped".into(),
            },
            LspCompletionItem {
                label: "death".into(),
                kind: 14,
                detail: "When a player dies".into(),
                insert_text: "death".into(),
            },
            LspCompletionItem {
                label: "card_targeted".into(),
                kind: 14,
                detail: "For each declared target of a used card".into(),
                insert_text: "card_targeted".into(),
            },
            LspCompletionItem {
                label: "card_responded".into(),
                kind: 14,
                detail: "When a card is played as a response".into(),
                insert_text: "card_responded".into(),
            },
            LspCompletionItem {
                label: "card_resolved".into(),
                kind: 14,
                detail: "After one logical card and all nested choices fully resolve".into(),
                insert_text: "card_resolved".into(),
            },
            LspCompletionItem {
                label: "sha_dodged".into(),
                kind: 14,
                detail: "After a Sha target supplies every required Shan".into(),
                insert_text: "sha_dodged".into(),
            },
            LspCompletionItem {
                label: "sha_resolved".into(),
                kind: 14,
                detail: "After every target and Shan response of one Sha fully resolves".into(),
                insert_text: "sha_resolved".into(),
            },
            LspCompletionItem {
                label: "dying".into(),
                kind: 14,
                detail: "When a player enters dying state".into(),
                insert_text: "dying".into(),
            },
            LspCompletionItem {
                label: "cards_drawn".into(),
                kind: 14,
                detail: "When cards are drawn".into(),
                insert_text: "cards_drawn".into(),
            },
            LspCompletionItem {
                label: "skill_added".into(),
                kind: 14,
                detail: "When a player gains one skill".into(),
                insert_text: "skill_added".into(),
            },
            LspCompletionItem {
                label: "turn_ended".into(),
                kind: 14,
                detail: "After a player's whole turn ends".into(),
                insert_text: "turn_ended".into(),
            },
            LspCompletionItem {
                label: "damage_before".into(),
                kind: 14,
                detail: "Before prepared damage changes HP".into(),
                insert_text: "damage_before".into(),
            },
            LspCompletionItem {
                label: "damage_prevented".into(),
                kind: 14,
                detail: "After positive damage is cancelled or reduced to zero".into(),
                insert_text: "damage_prevented".into(),
            },
            LspCompletionItem {
                label: "hp_recovered".into(),
                kind: 14,
                detail: "After a player recovers HP".into(),
                insert_text: "hp_recovered".into(),
            },
            LspCompletionItem {
                label: "character_state_changed".into(),
                kind: 14,
                detail: "When a character card turns over or changes linked state".into(),
                insert_text: "character_state_changed".into(),
            },
            LspCompletionItem {
                label: "hand_compare_started".into(),
                kind: 14,
                detail: "After the initiator selects a comparison card".into(),
                insert_text: "hand_compare_started".into(),
            },
            LspCompletionItem {
                label: "hand_compare_finished".into(),
                kind: 14,
                detail: "After both comparison cards enter the discard pile".into(),
                insert_text: "hand_compare_finished".into(),
            },
            LspCompletionItem {
                label: "judgment_resolved".into(),
                kind: 14,
                detail: "After a judgment and every rewrite complete".into(),
                insert_text: "judgment_resolved".into(),
            },
        ],
        CompletionContext::TriggerBody => vec![
            LspCompletionItem {
                label: "effect choose_discard_one_from_each_linked".into(),
                kind: 15,
                detail: "Discard one hand/equipment card from each linked player in seat order"
                    .into(),
                insert_text:
                    "effect choose_discard_one_from_each_linked(${1:self}, ${2:self_skill});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect judge_set_skill_mark_to_rank".into(),
                kind: 15,
                detail: "Judge and replace this skill's same-id mark with the result rank".into(),
                insert_text:
                    "effect judge_set_skill_mark_to_rank(${1:self}, ${2:self_skill});".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_redirect_current_sha".into(),
                kind: 15,
                detail: "Discard one card, then redirect the saved Sha target slot".into(),
                insert_text:
                    "effect choose_discard_redirect_current_sha(${1:self}, ${2:event_source}, ${3:event_card});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect recover_hp_as".into(),
                kind: 15,
                detail: "Recover HP while preserving an explicit recovery source".into(),
                insert_text:
                    "effect recover_hp_as(${1:event_source}, ${2:self}, ${3:1});".into(),
            },
            LspCompletionItem {
                label: "effect prevent_current_damage".into(),
                kind: 15,
                detail: "Prevent the damage bound to a damage_before trigger".into(),
                insert_text: "effect prevent_current_damage();".into(),
            },
            LspCompletionItem {
                label: "effect choose_prevent_damage_or_lose_max_hp_increase_damage".into(),
                kind: 15,
                detail: "Forced choice: prevent bound damage, or lose 1 max HP and increase it"
                    .into(),
                insert_text:
                    "effect choose_prevent_damage_or_lose_max_hp_increase_damage(self, 2);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_draw_or_increase_current_card_damage_or_both_mark".into(),
                kind: 15,
                detail: "Choose draw, current-card damage +1, or both plus a mark".into(),
                insert_text:
                    "effect choose_draw_or_increase_current_card_damage_or_both_mark(self, event_card, 2, 1, ${1:mark_def}, 1);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect randomize_current_hand_compare_opponent".into(),
                kind: 15,
                detail: "Replace the other comparison participant's choice with deterministic RNG"
                    .into(),
                insert_text: "effect randomize_current_hand_compare_opponent(self);".into(),
            },
            LspCompletionItem {
                label: "effect gain_highest_rank_sha_from_event_cards".into(),
                kind: 15,
                detail: "Gain every tied highest-rank Sha from the completed comparison".into(),
                insert_text: "effect gain_highest_rank_sha_from_event_cards(self);".into(),
            },
            LspCompletionItem {
                label: "effect gain_hand_compare_result_card".into(),
                kind: 15,
                detail: "Gain the comparison result card selected by legacy zongshi semantics"
                    .into(),
                insert_text: "effect gain_hand_compare_result_card(self);".into(),
            },
            LspCompletionItem {
                label: "effect choose_add_or_remove_current_card_target".into(),
                kind: 15,
                detail: "Optionally add or remove one target from the current card use".into(),
                insert_text:
                    "effect choose_add_or_remove_current_card_target(self, event_card);".into(),
            },
            LspCompletionItem {
                label: "effect judge_prevent_damage_lose_max_hp_on_nonheart".into(),
                kind: 15,
                detail: "Judge; on non-heart prevent current damage and reduce target max HP"
                    .into(),
                insert_text:
                    "effect judge_prevent_damage_lose_max_hp_on_nonheart(self, event_player);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_gain_top_cards_by_suit".into(),
                kind: 15,
                detail: "Privately view top cards and optionally gain matching-suit cards"
                    .into(),
                insert_text:
                    "effect choose_gain_top_cards_by_suit(${1:self}, ${2:3}, ${3:heart});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_other_draw_by_hand_count".into(),
                kind: 15,
                detail: "Repeatedly choose another player; empty hands use a different draw count"
                    .into(),
                insert_text:
                    "effect choose_other_draw_by_hand_count(${1:self}, ${2:1}, ${3:2}, ${4:event_amount});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_target_draw".into(),
                kind: 15,
                detail: "Choose a living target from a typed scope and make them draw".into(),
                insert_text:
                    "effect choose_target_draw(${1:self}, ${2:1}, ${3:min_hand});".into(),
            },
            LspCompletionItem {
                label: "effect choose_targets_draw".into(),
                kind: 15,
                detail: "Choose distinct living targets from a typed scope and make each draw"
                    .into(),
                insert_text:
                    "effect choose_targets_draw(${1:self}, ${2:2}, ${3:2}, ${4:any});".into(),
            },
            LspCompletionItem {
                label: "effect choose_optional_targets_draw".into(),
                kind: 15,
                detail: "Optionally choose a bounded number of targets and make each draw"
                    .into(),
                insert_text:
                    "effect choose_optional_targets_draw(${1:self}, ${2:1}, ${3:1}, ${4:255}, ${5:wounded});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect reveal_target_hand_sha_or_discard_bind".into(),
                kind: 15,
                detail: "Resolve Weikui's Shan branch and bind its temporary helper".into(),
                insert_text:
                    "effect reveal_target_hand_sha_or_discard_bind(${1:self}, ${2:event_source}, ${3:helper_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect draw_remembered_hand_cards".into(),
                kind: 15,
                detail: "Draw cards and remember those that remain continuously in hand".into(),
                insert_text:
                    "effect draw_remembered_hand_cards(${1:self}, ${2:2});".into(),
            },
            LspCompletionItem {
                label: "effect discard_remembered_hand_cards".into(),
                kind: 15,
                detail: "Discard remembered cards still continuously in hand".into(),
                insert_text:
                    "effect discard_remembered_hand_cards(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect choose_targets_damage".into(),
                kind: 15,
                detail: "Choose a bounded number of targets and damage each in seat order".into(),
                insert_text:
                    "effect choose_targets_damage(${1:self}, ${2:1}, ${3:1}, ${4:hand_empty}, ${5:1});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_target_draw_to_max_hp_cap".into(),
                kind: 15,
                detail: "Choose any living target and draw them up to capped maximum HP".into(),
                insert_text:
                    "effect choose_target_draw_to_max_hp_cap(${1:self}, ${2:5});".into(),
            },
            LspCompletionItem {
                label: "effect choose_target_draw_then_owner_if_below_max_hp".into(),
                kind: 15,
                detail:
                    "Choose a living target to draw, then draw one for the owner if that hand remains below max HP"
                        .into(),
                insert_text:
                    "effect choose_target_draw_then_owner_if_below_max_hp(${1:self}, ${2:2});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_target_draw_max_hp_cap_then_discard_excess".into(),
                kind: 15,
                detail:
                    "Choose a target to draw by capped max HP, then make that target discard excess hand cards"
                        .into(),
                insert_text:
                    "effect choose_target_draw_max_hp_cap_then_discard_excess(${1:self}, ${2:5});"
                        .into(),
            },
            LspCompletionItem {
                label:
                    "effect give_selected_then_target_choose_discard_same_suit_or_lose_hp".into(),
                kind: 15,
                detail:
                    "Give the selected hand card, then let its recipient discard matching-suit cards or lose HP"
                        .into(),
                insert_text:
                    "effect give_selected_then_target_choose_discard_same_suit_or_lose_hp(${1:self}, ${2:event_source});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect gain_equipment_from_draw_pile".into(),
                kind: 15,
                detail: "Gain the topmost equipment card from the draw pile".into(),
                insert_text: "effect gain_equipment_from_draw_pile(${1:self});".into(),
            },
            LspCompletionItem {
                label:
                    "effect target_choose_suit_gain_owner_hand_damage_if_mismatch".into(),
                kind: 15,
                detail:
                    "Target guesses a suit, gains one owner hand card, and takes damage on mismatch"
                        .into(),
                insert_text:
                    "effect target_choose_suit_gain_owner_hand_damage_if_mismatch(${1:self}, ${2:event_source});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect judge_source_discard_two_hand_or_damage".into(),
                kind: 15,
                detail:
                    "Judge; on non-heart the damage source discards two hand cards or takes 1 damage"
                        .into(),
                insert_text:
                    "effect judge_source_discard_two_hand_or_damage(${1:self}, ${2:event_source});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect give_selected_recover_after_phase_total_two".into(),
                kind: 15,
                detail:
                    "Give selected hand cards; recover once when the Play-phase total reaches two"
                        .into(),
                insert_text:
                    "effect give_selected_recover_after_phase_total_two(${1:self}, ${2:event_source});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_other_hand_targets_gain_one_replace_draw".into(),
                kind: 15,
                detail:
                    "Choose 1–2 other hand-card targets, gain one from each, and replace the draw"
                        .into(),
                insert_text:
                    "effect choose_other_hand_targets_gain_one_replace_draw(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect choose_other_hand_targets_gain_one_reduce_draw".into(),
                kind: 15,
                detail:
                    "Gain one hand card from selected players and reduce the draw by that count"
                        .into(),
                insert_text:
                    "effect choose_other_hand_targets_gain_one_reduce_draw(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect distribute_top_two_cards".into(),
                kind: 15,
                detail:
                    "Move the top two cards to processing and distribute all of them".into(),
                insert_text: "effect distribute_top_two_cards(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect give_all_hand_recover_to_one".into(),
                kind: 15,
                detail: "Give every hand card, then recover the dying recipient to one HP".into(),
                insert_text:
                    "effect give_all_hand_recover_to_one(${1:self}, ${2:event_player});".into(),
            },
            LspCompletionItem {
                label: "effect give_all_hand".into(),
                kind: 15,
                detail: "Give every current hand card to one recipient".into(),
                insert_text: "effect give_all_hand(${1:self}, ${2:event_source});".into(),
            },
            LspCompletionItem {
                label:
                    "effect choose_target_self_discard_then_owner_lose_hp_if_hand_ne_hp".into(),
                kind: 15,
                detail:
                    "Choose a target to self-discard, then lose HP if their hand differs from HP"
                        .into(),
                insert_text:
                    "effect choose_target_self_discard_then_owner_lose_hp_if_hand_ne_hp(${1:self});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_give_one_hand_equip_or_recover".into(),
                kind: 15,
                detail: "Let another player give one hand/equipment card or recover the target"
                    .into(),
                insert_text:
                    "effect choose_give_one_hand_equip_or_recover(${1:self}, ${2:event_source}, ${3:event_player});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_gzjinfa".into(),
                kind: 15,
                detail: "Discard the selected card, then resolve equipment gift or target-card gain"
                    .into(),
                insert_text: "effect resolve_gzjinfa(${1:self}, ${2:event_source});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_wn_qifeng".into(),
                kind: 15,
                detail: "Use a virtual Sha whose damage equals cards lost this turn".into(),
                insert_text: "effect resolve_wn_qifeng(${1:self}, ${2:event_player});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_dcjue".into(),
                kind: 15,
                detail: "Resolve Jue's discarded-card-capped random virtual-card loop".into(),
                insert_text: "effect resolve_dcjue(${1:self}, ${2:event_player});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_ruanji".into(),
                kind: 15,
                detail: "Resolve Ruanji's Zhaowen or Jiudun branch".into(),
                insert_text:
                    "effect resolve_ruanji(${1:self}, ${2:event_source}, ${3:event_card}, ${4:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_zhengbi".into(),
                kind: 15,
                detail: "Resolve Cuimao's Zhengbi branch".into(),
                insert_text:
                    "effect resolve_zhengbi(${1:self}, ${2:event_source}, ${3:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_fengying".into(),
                kind: 15,
                detail: "Resolve Cuimao's Fengying extra turn".into(),
                insert_text:
                    "effect resolve_fengying(${1:self}, ${2:597375372}, ${3:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_qiaojian".into(),
                kind: 15,
                detail: "Resolve Fugan's Qiaojian simultaneous discard and next-turn branch".into(),
                insert_text:
                    "effect resolve_qiaojian(${1:self}, ${2:event_source}, ${3:0}, ${4:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_xicha".into(),
                kind: 15,
                detail: "Resolve Fugan's Xicha hidden card watch".into(),
                insert_text:
                    "effect resolve_xicha(${1:self}, ${2:event_source}, ${3:event_card}, ${4:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_huzun".into(),
                kind: 15,
                detail: "Resolve Huzun's Zhantao or Anjing branch".into(),
                insert_text:
                    "effect resolve_huzun(${1:self}, ${2:event_source}, ${3:event_card}, ${4:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_zhugemengxue".into(),
                kind: 15,
                detail: "Resolve Zhugemengxue's Jichun or Hanying branch".into(),
                insert_text: "effect resolve_zhugemengxue(${1:self}, ${2:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_bailingyun".into(),
                kind: 15,
                detail: "Resolve Bailingyun's Linghui or Xiace branch".into(),
                insert_text: "effect resolve_bailingyun(${1:self}, ${2:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_dcshaowei".into(),
                kind: 15,
                detail: "Resolve Wei Guanyinping's Shaowei branches".into(),
                insert_text: "effect resolve_dcshaowei(${1:self}, ${2:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_dcdichou".into(),
                kind: 15,
                detail: "Resolve Wei Guanyinping's Dichou turn branches".into(),
                insert_text:
                    "effect resolve_dcdichou(${1:self}, ${2:0}, ${3:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_dcdufeng".into(),
                kind: 15,
                detail: "Resolve Ling Cao's Dufeng costs and turn modifiers".into(),
                insert_text:
                    "effect resolve_dcdufeng(${1:self}, ${2:0}, ${3:0});".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_hand_range".into(),
                kind: 15,
                detail: "Force a player to choose a range of hand cards to discard".into(),
                insert_text:
                    "effect choose_discard_hand_range(${1:self}, ${2:1}, ${3:2147483647});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_dcctjiuxian".into(),
                kind: 15,
                detail: "Resolve Chen Tai's Jiu Xian recast, Duel, and recovery".into(),
                insert_text:
                    "effect resolve_dcctjiuxian(${1:self}, ${2:self}, ${3:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_bianyue".into(),
                kind: 15,
                detail: "Resolve Bian Yue's Bizu or Wuxie branch".into(),
                insert_text: "effect resolve_bianyue(${1:self}, ${2:self}, ${3:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_zhupeilan".into(),
                kind: 15,
                detail: "Resolve Zhu Peilan's Cilv or Tongdao branch".into(),
                insert_text:
                    "effect resolve_zhupeilan(${1:self}, ${2:event_source}, ${3:event_card}, ${4:helper_skill_def}, ${5:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_dc_sb_zhangxiu".into(),
                kind: 15,
                detail: "Resolve DC SB Zhang Xiu's Fuxi or Haoyi branch".into(),
                insert_text:
                    "effect resolve_dc_sb_zhangxiu(${1:self}, ${2:event_player}, ${3:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_dc_sb_zhouyu".into(),
                kind: 15,
                detail: "Resolve DC SB Zhou Yu's Ronghuo or Yingmou branch".into(),
                insert_text:
                    "effect resolve_dc_sb_zhouyu(${1:self}, ${2:event_source}, ${3:event_card}, ${4:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_dc_sb_lusu".into(),
                kind: 15,
                detail: "Resolve DC SB Lu Su's Mingshi or Mengmou branch".into(),
                insert_text:
                    "effect resolve_dc_sb_lusu(${1:self}, ${2:event_source}, ${3:event_card}, ${4:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_zhangjian".into(),
                kind: 15,
                detail: "Resolve Zhangjian's damage rewrite or temporary skill grant".into(),
                insert_text:
                    "effect resolve_zhangjian(${1:self}, ${2:event_card}, ${3:grant_skill_def}, ${4:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_zhugeruoxue".into(),
                kind: 15,
                detail: "Resolve Zhugeruoxue's Qiongying or Nuanhui sequence".into(),
                insert_text:
                    "effect resolve_zhugeruoxue(${1:self}, ${2:self}, ${3:event_card}, ${4:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_give_two_different_colors_or_recover".into(),
                kind: 15,
                detail:
                    "Let another player give two differently colored hand/equipment cards or recover the target"
                        .into(),
                insert_text:
                    "effect choose_give_two_different_colors_or_recover(${1:self}, ${2:event_source}, ${3:event_player});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_mbzhuguo".into(),
                kind: 15,
                detail: "Resolve capped hand adjustment and the optional follow-up Sha".into(),
                insert_text: "effect resolve_mbzhuguo(${1:self}, ${2:event_source});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_mbanxian".into(),
                kind: 15,
                detail: "Gain and use a Sha from the first hand-discard event".into(),
                insert_text: "effect resolve_mbanxian(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_mbjinzu".into(),
                kind: 15,
                detail: "Reveal selected hand cards and install the next-Sha helper".into(),
                insert_text:
                    "effect resolve_mbjinzu(${1:self}, ${2:event_source}, ${3:helper_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect apply_mbjinzu_next_sha".into(),
                kind: 15,
                detail: "Consume the Jinzu helper on its owner's next Sha".into(),
                insert_text:
                    "effect apply_mbjinzu_next_sha(${1:self}, ${2:event_card});".into(),
            },
            LspCompletionItem {
                label: "effect add_bound_skill".into(),
                kind: 15,
                detail: "Grant a skill instance bound to one living target".into(),
                insert_text:
                    "effect add_bound_skill(${1:self}, ${2:skill_def}, ${3:event_source});"
                        .into(),
            },
            LspCompletionItem {
                label:
                    "effect replace_skill_with_bound_after_resolution_if_target_alive".into(),
                kind: 15,
                detail:
                    "After the resolution barrier, replace a skill with a target-bound helper when the target survived"
                        .into(),
                insert_text:
                    "effect replace_skill_with_bound_after_resolution_if_target_alive(${1:self}, ${2:self_skill}, ${3:skill_def}, ${4:event_source});"
                        .into(),
            },
            LspCompletionItem {
                label:
                    "effect choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp".into(),
                kind: 15,
                detail:
                    "Choose a nonempty card zone, discard it, draw by discarded cards plus lost HP, then lose 1 HP"
                        .into(),
                insert_text:
                    "effect choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp(${1:self});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_other_draw_owner_lost_hp_turn_over".into(),
                kind: 15,
                detail: "Choose another player, draw by owner lost HP, then turn them over".into(),
                insert_text:
                    "effect choose_other_draw_owner_lost_hp_turn_over(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect draw_by_play_phase_card_category_count".into(),
                kind: 15,
                detail: "Draw by distinct logical card categories used this play phase".into(),
                insert_text:
                    "effect draw_by_play_phase_card_category_count(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_one_for_sha_refund_and_draw_if_red".into(),
                kind: 15,
                detail: "Discard one card, refund the current Sha use once, and draw if red"
                    .into(),
                insert_text:
                    "effect choose_discard_one_for_sha_refund_and_draw_if_red(${1:self}, ${2:event_source}, ${3:event_card});"
                        .into(),
            },
            LspCompletionItem {
                label:
                    "effect choose_discard_one_for_sha_refund_and_draw_if_red_restore_if_equal_rank"
                        .into(),
                kind: 15,
                detail:
                    "Discard one card, resolve Longyin, and restore a skill on equal effective rank"
                        .into(),
                insert_text:
                    "effect choose_discard_one_for_sha_refund_and_draw_if_red_restore_if_equal_rank(${1:self}, ${2:event_source}, ${3:event_card}, ${4:skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_beige_configured".into(),
                kind: 15,
                detail:
                    "Resolve Beige with configurable Heart recovery and Diamond draw counts"
                        .into(),
                insert_text:
                    "effect resolve_beige_configured(${1:self}, ${2:event_player}, ${3:event_source}, ${4:event_amount}, ${5:3});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect queue_opening_extra_play_phase".into(),
                kind: 15,
                detail: "Queue an extra Play phase before the current turn's Prepare phase"
                    .into(),
                insert_text: "effect queue_opening_extra_play_phase(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect convert_current_sha_to_fire_lose_hp_if_damage".into(),
                kind: 15,
                detail: "Convert the current ordinary Sha to fire with its post-use HP cost"
                    .into(),
                insert_text:
                    "effect convert_current_sha_to_fire_lose_hp_if_damage(${1:self}, ${2:event_card});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_add_current_sha_target".into(),
                kind: 15,
                detail: "Optionally add one legal target to the current Sha".into(),
                insert_text:
                    "effect choose_add_current_sha_target(${1:self}, ${2:event_card});".into(),
            },
            LspCompletionItem {
                label: "effect increase_current_sha_base_damage".into(),
                kind: 15,
                detail: "Increase the suspended Sha use's base damage".into(),
                insert_text:
                    "effect increase_current_sha_base_damage(${1:self}, ${2:event_card}, ${3:1});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect increase_current_card_base_damage".into(),
                kind: 15,
                detail: "Increase the suspended Sha/Nanman/Wanjian use's base damage".into(),
                insert_text:
                    "effect increase_current_card_base_damage(${1:self}, ${2:event_card}, ${3:1});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect lose_hp_then_damage_other_lose_max_hp_if_dying".into(),
                kind: 15,
                detail:
                    "Lose HP, conditionally lose max HP on exact dying, then damage another"
                        .into(),
                insert_text:
                    "effect lose_hp_then_damage_other_lose_max_hp_if_dying(${1:self}, ${2:event_source}, ${3:1}, self_skill);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_store_hand_sha_in_skill_pile".into(),
                kind: 15,
                detail: "Store selected hand Sha in this skill's public pile".into(),
                insert_text: "effect choose_store_hand_sha_in_skill_pile(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect choose_spend_skill_pile_for_dying_jiu".into(),
                kind: 15,
                detail: "Spend one skill-pile card so a dying player uses virtual Jiu".into(),
                insert_text:
                    "effect choose_spend_skill_pile_for_dying_jiu(${1:self}, ${2:event_player});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_compare_selected_hand_card".into(),
                kind: 15,
                detail: "Compare the selected hand card and grant a win/lose helper skill".into(),
                insert_text:
                    "effect choose_compare_selected_hand_card(${1:self}, ${2:event_source}, ${3:win_skill_def}, ${4:lose_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_hand_compare".into(),
                kind: 15,
                detail: "Choose a hand-comparison target and both cards".into(),
                insert_text:
                    "effect choose_hand_compare(${1:self}, ${2:win_skill_def}, ${3:lose_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_other_damage".into(),
                kind: 15,
                detail: "Choose another living player and deal normal damage".into(),
                insert_text: "effect choose_other_damage(${1:self}, ${2:1});".into(),
            },
            LspCompletionItem {
                label: "effect choose_bound_source_attack_range_damage".into(),
                kind: 15,
                detail: "Choose a target in a bound source's attack range and deal damage from that source".into(),
                insert_text:
                    "effect choose_bound_source_attack_range_damage(${1:self}, ${2:event_source}, ${3:1});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect reveal_uniform_hand_choose_damage".into(),
                kind: 15,
                detail: "Reveal a uniform-color hand, then choose any living player to damage"
                    .into(),
                insert_text:
                    "effect reveal_uniform_hand_choose_damage(${1:self}, ${2:1});".into(),
            },
            LspCompletionItem {
                label: "effect choose_gain_discard_card_by_kind".into(),
                kind: 15,
                detail: "Choose and gain one matching card from the discard pile".into(),
                insert_text:
                    "effect choose_gain_discard_card_by_kind(${1:self}, ${2:sha});".into(),
            },
            LspCompletionItem {
                label: "effect draw_for_lifetime_card_action_divisors".into(),
                kind: 15,
                detail: "Draw once for each divisor of the player's lifetime card-action count"
                    .into(),
                insert_text:
                    "effect draw_for_lifetime_card_action_divisors(${1:self}, ${2:3}, ${3:5}, ${4:8});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect gain_draw_cards_for_lifetime_card_action_divisors".into(),
                kind: 15,
                detail: "Gain one topmost matching draw-pile card for each matching divisor"
                    .into(),
                insert_text:
                    "effect gain_draw_cards_for_lifetime_card_action_divisors(${1:self}, ${2:3}, ${3:sha_or_shan}, ${4:5}, ${5:tao_or_jiu}, ${6:8}, ${7:juedou_or_wuzhong});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect lose_hp_gain_discard_card_for_opening_extra_play".into(),
                kind: 15,
                detail: "Lose HP and gain a discard card in this skill's opening extra Play phase"
                    .into(),
                insert_text:
                    "effect lose_hp_gain_discard_card_for_opening_extra_play(${1:self}, ${2:sha}, ${3:mandatory_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_gain_own_lost_discard_card_this_turn".into(),
                kind: 15,
                detail: "Choose one own card lost this turn that remains in discard".into(),
                insert_text:
                    "effect choose_gain_own_lost_discard_card_this_turn(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect choose_move_field_card_to".into(),
                kind: 15,
                detail: "Move one compatible field card to a fixed player".into(),
                insert_text:
                    "effect choose_move_field_card_to(${1:self}, ${2:event_source});".into(),
            },
            LspCompletionItem {
                label: "effect draw_then_optionally_distribute_hand_cards".into(),
                kind: 15,
                detail: "Draw, then optionally distribute current hand cards".into(),
                insert_text:
                    "effect draw_then_optionally_distribute_hand_cards(${1:self}, ${2:2}, ${3:2});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect give_selected_then_continue_distinct_recover_at_two".into(),
                kind: 15,
                detail: "Give selected cards, then optionally repeat for distinct recipients"
                    .into(),
                insert_text:
                    "effect give_selected_then_continue_distinct_recover_at_two(${1:self}, ${2:event_source});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect give_selected_then_recover_target".into(),
                kind: 15,
                detail: "Give selected cards, then recover the target after movement triggers"
                    .into(),
                insert_text:
                    "effect give_selected_then_recover_target(${1:self}, ${2:event_source}, ${3:1});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect give_selected_then_target_may_use_draw_owner".into(),
                kind: 15,
                detail: "Give one non-equipment card; recipient may use it; owner draws"
                    .into(),
                insert_text:
                    "effect give_selected_then_target_may_use_draw_owner(${1:self}, ${2:event_source});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_dingpan".into(),
                kind: 15,
                detail: "Draw, then resolve Dingpan's equipment branch and delayed damage".into(),
                insert_text: "effect resolve_dingpan(${1:self}, ${2:event_source});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_tanlu".into(),
                kind: 15,
                detail: "Resolve Tanlu's frozen HP-gap gift or damage branch".into(),
                insert_text: "effect resolve_tanlu(${1:self}, ${2:event_player});".into(),
            },
            LspCompletionItem {
                label: "effect choose_max_mark_other_judge_kill_unless_tao_or_taoyuan".into(),
                kind: 15,
                detail: "Choose a highest-marked other player, judge, and conditionally kill"
                    .into(),
                insert_text:
                    "effect choose_max_mark_other_judge_kill_unless_tao_or_taoyuan(${1:self}, ${2:mark_def});"
                        .into(),
            },
            LspCompletionItem {
                label:
                    "effect choose_zone_priority_gain_random_card_from_each_other_then_turn_over"
                        .into(),
                kind: 15,
                detail: "Choose a zone priority, gain one random card from each other player, then turn over"
                    .into(),
                insert_text:
                    "effect choose_zone_priority_gain_random_card_from_each_other_then_turn_over(${1:self});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_add_missing_skill".into(),
                kind: 15,
                detail: "Choose and gain one candidate skill not already owned".into(),
                insert_text:
                    "effect choose_add_missing_skill(${1:self}, ${2:first_skill_def}, ${3:second_skill_def}, ${4:third_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect add_first_missing_skill".into(),
                kind: 15,
                detail: "Gain the first candidate skill not already owned".into(),
                insert_text:
                    "effect add_first_missing_skill(${1:self}, ${2:first_skill_def}, ${3:second_skill_def}, ${4:third_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_dccunsi".into(),
                kind: 15,
                detail: "Grant Yongjue, draw for another target, then resolve Guixiu recovery"
                    .into(),
                insert_text:
                    "effect resolve_dccunsi(${1:self}, ${2:event_source}, ${3:guixiu_def}, ${4:yongjue_def}, 0);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect record_current_sha_all_targets".into(),
                kind: 15,
                detail: "Bind every current Sha target to the originating skill".into(),
                insert_text:
                    "effect record_current_sha_all_targets(${1:self}, ${2:event_card});".into(),
            },
            LspCompletionItem {
                label: "effect choose_other_discard_target_card".into(),
                kind: 15,
                detail: "Choose another player, then discard one of their scoped cards".into(),
                insert_text:
                    "effect choose_other_discard_target_card(${1:self}, ${2:hand_equip});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_add_skill_bound_card_target".into(),
                kind: 15,
                detail: "Optionally append this skill instance's bound target".into(),
                insert_text:
                    "effect choose_add_skill_bound_card_target(${1:self}, ${2:event_card});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect judge_cancel_current_sha_target_on_red".into(),
                kind: 15,
                detail: "Judge and exclude the current Sha target on red".into(),
                insert_text: "effect judge_cancel_current_sha_target_on_red(self, event_card, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect make_current_sha_target_unrespondable".into(),
                kind: 15,
                detail: "Disable only the current Sha target's response window".into(),
                insert_text: "effect make_current_sha_target_unrespondable(self, event_card, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect judge_make_current_sha_target_unrespondable_on_red".into(),
                kind: 15,
                detail: "Judge and disable the current Sha target's response window on red".into(),
                insert_text: "effect judge_make_current_sha_target_unrespondable_on_red(self, event_card, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_retieji".into(),
                kind: 15,
                detail: "Resolve 界铁骑's judgment, skill suppression, and gift branch".into(),
                insert_text: "effect resolve_retieji(self, event_card, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_new_reluoyi".into(),
                kind: 15,
                detail: "Resolve 界裸衣's top-three optional draw replacement".into(),
                insert_text: "effect resolve_new_reluoyi(self, ${1:buff_skill_def});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_remoshi".into(),
                kind: 15,
                detail: "Place or retrieve 没矢 cards".into(),
                insert_text:
                    "effect resolve_remoshi(self, event_player, event_card, ${1:helper_skill_def}, ${2:action});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_xinyaoming".into(),
                kind: 15,
                detail: "Resolve 界邀名's per-turn target and action choices".into(),
                insert_text: "effect resolve_xinyaoming(self);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_rehuaiyi".into(),
                kind: 15,
                detail: "Resolve 界怀异's reveal, color discard, and target gains".into(),
                insert_text: "effect resolve_rehuaiyi(self);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_reanguo".into(),
                kind: 15,
                detail: "Resolve 界安国's minimum-metric branches and optional recast".into(),
                insert_text: "effect resolve_reanguo(self);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_beizhu".into(),
                kind: 15,
                detail: "Resolve 备诛's hand reveal, discard-or-Sha sequence".into(),
                insert_text: "effect resolve_beizhu(self, event_source);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_mbshanjia".into(),
                kind: 15,
                detail: "Resolve mobile 缮甲's draw, dynamic discard, and category rewards"
                    .into(),
                insert_text:
                    "effect resolve_mbshanjia(self, mark_count(self, ${1:main_skill_def}), ${2:distance_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_daoji".into(),
                kind: 15,
                detail: "Resolve mobile 盗戟's equipment gain, forced use, and weapon damage"
                    .into(),
                insert_text: "effect resolve_daoji(self, event_source);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_tw_daoji".into(),
                kind: 15,
                detail: "Resolve TW 盗戟's hand/equipment gain and category reward".into(),
                insert_text: "effect resolve_tw_daoji(self, event_source);".into(),
            },
            LspCompletionItem {
                label: "effect give_all_hand_then_choose_use_one_or_damage".into(),
                kind: 15,
                detail: "Give all hand cards; recipient uses one or takes damage".into(),
                insert_text:
                    "effect give_all_hand_then_choose_use_one_or_damage(self, event_source);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_category_then_target_give_matching_or_sha_unrespondable"
                    .into(),
                kind: 15,
                detail: "Choose a card category; Sha target gives one or cannot respond".into(),
                insert_text: "effect choose_category_then_target_give_matching_or_sha_unrespondable(self, event_card, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect choose_category_then_target_give_matching_or_damage_plus_one"
                    .into(),
                kind: 15,
                detail: "Choose a category; damage target gives one or damage increases".into(),
                insert_text: "effect choose_category_then_target_give_matching_or_damage_plus_one(self, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect choose_other_excluding_source_random_gain_hand_equip_then_use_if_equip"
                    .into(),
                kind: 15,
                detail: "Choose a non-source role, randomly gain hand/equipment, use equipment"
                    .into(),
                insert_text: "effect choose_other_excluding_source_random_gain_hand_equip_then_use_if_equip(self, event_source);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_mobiledanshou".into(),
                kind: 15,
                detail: "Draw at zero targetings, otherwise discard that many to damage".into(),
                insert_text: "effect resolve_mobiledanshou(self, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_jsrgchiying".into(),
                kind: 15,
                detail: "Force in-range roles to discard; target conditionally gains basics"
                    .into(),
                insert_text: "effect resolve_jsrgchiying(self, event_source);".into(),
            },
            LspCompletionItem {
                label: "effect choose_lose_hp_use_virtual_sha_by_basic_variants".into(),
                kind: 15,
                detail: "Choose an ignore-distance virtual Sha after losing HP; distinct basic variants add damage".into(),
                insert_text:
                    "effect choose_lose_hp_use_virtual_sha_by_basic_variants(self);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twfupan".into(),
                kind: 15,
                detail: "Draw by damage, give a card, then apply per-target Fupan state".into(),
                insert_text: "effect resolve_twfupan(self, event_amount);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twjiaohua".into(),
                kind: 15,
                detail: "Give the draw recipient an unused category from draw/discard".into(),
                insert_text: "effect resolve_twjiaohua(self, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twkujian".into(),
                kind: 15,
                detail: "Gift or settle tracked TW Kujian physical cards".into(),
                insert_text: "effect resolve_twkujian(self, event_source, ${1:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twruilian".into(),
                kind: 15,
                detail: "Track a role's round discard categories and gain matching cards".into(),
                insert_text: "effect resolve_twruilian(self, event_player, ${1:0});".into(),
            },
            LspCompletionItem {
                label: "effect equip_weapon_from_draw_pile".into(),
                kind: 15,
                detail: "Randomly equip one draw-pile weapon if the slot is empty".into(),
                insert_text: "effect equip_weapon_from_draw_pile(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twjianwei".into(),
                kind: 15,
                detail: "Start either direction of TW Jianwei's prepare-phase comparison".into(),
                insert_text: "effect resolve_twjianwei(self, event_player, ${1:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twtanfeng".into(),
                kind: 15,
                detail: "Resolve TW Tanfeng's discard, Sha, damage, and phase-skip branches"
                    .into(),
                insert_text: "effect resolve_twtanfeng(self);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twzhibian".into(),
                kind: 15,
                detail: "Resolve TW Zhibian's comparison and win/loss branches".into(),
                insert_text: "effect resolve_twzhibian(self);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twyuyan".into(),
                kind: 15,
                detail: "Resolve TW Yuyan's gift-or-cancel current Sha target".into(),
                insert_text: "effect resolve_twyuyan(self, event_source, event_card);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twguoyi".into(),
                kind: 15,
                detail: "Resolve TW Guoyi's target branch and post-use replay".into(),
                insert_text:
                    "effect resolve_twguoyi(self, event_player, event_card, ${1:helper_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_twchuhai".into(),
                kind: 15,
                detail: "Resolve TW Chuhai's mission success or gained-card discard".into(),
                insert_text: "effect resolve_twchuhai(self, ${1:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twyizhu".into(),
                kind: 15,
                detail: "Resolve TW Yizhu planting, use, or discard tracking".into(),
                insert_text:
                    "effect resolve_twyizhu(self, ${1:event_source}, event_card, ${2:1});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_twluanchou".into(),
                kind: 15,
                detail: "Reset Gonghuan holders or redirect current damage".into(),
                insert_text:
                    "effect resolve_twluanchou(self, ${1:helper_skill_def}, ${2:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twxiawei".into(),
                kind: 15,
                detail: "Maintain Wei pile, Wangxing cost, or Qiongji damage".into(),
                insert_text:
                    "effect resolve_twxiawei(self, ${1:wangxing_skill_def}, ${2:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twbianfuren".into(),
                kind: 15,
                detail: "Resolve TW Wanwei damage/end phase or Yuejian pile placement".into(),
                insert_text: "effect resolve_twbianfuren(self, ${1:event_player}, ${2:helper_skill_def}, ${3:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twfeiyi".into(),
                kind: 15,
                detail: "Resolve TW Shengxi prepare/end or Kuanji transfer".into(),
                insert_text: "effect resolve_twfeiyi(self, ${1:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twchenzhen".into(),
                kind: 15,
                detail: "Resolve TW Muyue or Chayi state machines".into(),
                insert_text: "effect resolve_twchenzhen(self, ${1:event_player}, ${2:helper_skill_def}, ${3:record_skill_def}, ${4:0});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twqianju".into(),
                kind: 15,
                detail: "Equip a compatible shared-pile equipment card".into(),
                insert_text: "effect resolve_twqianju(self, ${1:event_player});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twqingxi".into(),
                kind: 15,
                detail: "Resolve TW Qingxi's first-Sha target choice".into(),
                insert_text: "effect resolve_twqingxi(self, ${1:event_player}, ${2:event_card});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twzaoli".into(),
                kind: 15,
                detail: "Resolve TW Zaoli discard, draw, and equipment refill".into(),
                insert_text: "effect resolve_twzaoli(self);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_twzhengjian".into(),
                kind: 15,
                detail: "Resolve TW Zhengjian setup, tribute, switching, and Zhongchi".into(),
                insert_text: "effect resolve_twzhengjian(self, ${1:event_player}, ${2:helper_skill_def}, ${3:0});".into(),
            },
            LspCompletionItem {
                label: "effect suppress_non_locked_skills_until_turn_end".into(),
                kind: 15,
                detail: "Suppress one target's non-locked skills for this turn".into(),
                insert_text:
                    "effect suppress_non_locked_skills_until_turn_end(${1:event_source});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_jiaohua".into(),
                kind: 15,
                detail: "Resolve mobile 教化's persistent category cycle and recipient gain"
                    .into(),
                insert_text: "effect resolve_jiaohua(self);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_luanqun".into(),
                kind: 15,
                detail: "Resolve mobile 乱群's private reveal and temporary Sha restrictions"
                    .into(),
                insert_text:
                    "effect resolve_luanqun(self, ${1:effect_skill_def}, ${2:direct_hit_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_basic_then_target_discard_equip_or_damage".into(),
                kind: 15,
                detail: "Pay a basic hand card, then make the target discard equipment or take damage".into(),
                insert_text: "effect choose_discard_basic_then_target_discard_equip_or_damage(self, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect choose_current_sha_target_discard_redirect_to_owner".into(),
                kind: 15,
                detail: "Let the current target discard, then redirect the Sha to the skill owner".into(),
                insert_text: "effect choose_current_sha_target_discard_redirect_to_owner(self, event_source, event_card, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect choose_target_gain_bottom_card".into(),
                kind: 15,
                detail: "Reveal the bottom card, then give it to a chosen living player".into(),
                insert_text: "effect choose_target_gain_bottom_card(self);".into(),
            },
            LspCompletionItem {
                label: "effect judge_black_gain_repeat".into(),
                kind: 15,
                detail: "Judge, gain a black result, and optionally repeat".into(),
                insert_text: "effect judge_black_gain_repeat(self);".into(),
            },
            LspCompletionItem {
                label: "effect judge_black_gain_repeat_protect_hand_limit".into(),
                kind: 15,
                detail: "Judge, gain black results, and protect those cards from this turn's hand limit".into(),
                insert_text: "effect judge_black_gain_repeat_protect_hand_limit(self);".into(),
            },
            LspCompletionItem {
                label: "effect judge_same_color_gain_repeat".into(),
                kind: 15,
                detail: "Judge, gain results matching the first color, and optionally repeat".into(),
                insert_text: "effect judge_same_color_gain_repeat(self);".into(),
            },
            LspCompletionItem {
                label: "effect use_virtual_juedou_second_on_first".into(),
                kind: 15,
                detail: "Make the second active target use Duel on the first".into(),
                insert_text: "effect use_virtual_juedou_second_on_first();".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_target_card_resolve_current_sha".into(),
                kind: 15,
                detail: "Discard a target card, then resolve the equipment/non-equipment Sha branch".into(),
                insert_text: "effect choose_discard_target_card_resolve_current_sha(self, event_card, event_player);".into(),
            },
            LspCompletionItem {
                label:
                    "effect choose_discard_target_card_resolve_current_sha_nonbasic_bonus".into(),
                kind: 15,
                detail:
                    "Discard a target card, then resolve Rejianchu's basic/non-basic Sha branch"
                        .into(),
                insert_text: "effect choose_discard_target_card_resolve_current_sha_nonbasic_bonus(self, event_card, event_player, ${1:bonus_skill_def});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_dcqiaomeng".into(),
                kind: 15,
                detail:
                    "Choose a target card, discard it, then resolve 趫猛's equipment/trick branch"
                        .into(),
                insert_text:
                    "effect resolve_dcqiaomeng(self, event_card, ${1:helper_skill_def});".into(),
            },
            LspCompletionItem {
                label: "effect choose_draw_or_recover".into(),
                kind: 15,
                detail: "Let a specified player draw or recover".into(),
                insert_text:
                    "effect choose_draw_or_recover(${1:event_source}, ${2:1}, ${3:1}, ${4:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_recover_target".into(),
                kind: 15,
                detail: "Let one player optionally recover a specified target".into(),
                insert_text:
                    "effect choose_recover_target(${1:self}, ${2:event_source}, ${3:1});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_use_virtual_sha".into(),
                kind: 15,
                detail: "Choose legal targets and use a suitless virtual Sha".into(),
                insert_text:
                    "effect choose_use_virtual_sha(${1:self}, ${2:ignore_distance});".into(),
            },
            LspCompletionItem {
                label: "effect adjust_hand_to".into(),
                kind: 15,
                detail: "Draw or force-discard to an exact hand count".into(),
                insert_text: "effect adjust_hand_to(${1:self}, ${2:4});".into(),
            },
            LspCompletionItem {
                label: "effect gain_card_by_kind".into(),
                kind: 15,
                detail: "Gain a card kind from draw pile, then discard pile".into(),
                insert_text: "effect gain_card_by_kind(${1:self}, ${2:tiesuo});".into(),
            },
            LspCompletionItem {
                label: "effect gain_discard_card_by_kind".into(),
                kind: 15,
                detail: "Gain a card kind from the discard pile only".into(),
                insert_text: "effect gain_discard_card_by_kind(${1:self}, ${2:sha});".into(),
            },
            LspCompletionItem {
                label: "effect disable_all_equip_slots".into(),
                kind: 15,
                detail: "Disable every currently enabled equipment slot".into(),
                insert_text: "effect disable_all_equip_slots(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect enable_equip_slot".into(),
                kind: 15,
                detail: "Restore one disabled copy of an equipment slot".into(),
                insert_text:
                    "effect enable_equip_slot(${1:event_source}, ${2:1}, ${3:self});".into(),
            },
            LspCompletionItem {
                label: "effect move_equipment_then_draw".into(),
                kind: 15,
                detail: "Move equipment to another legal player, then draw for its owner".into(),
                insert_text: "effect move_equipment_then_draw(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect choose_use_virtual_sha_or_draw".into(),
                kind: 15,
                detail: "Use a normal-distance virtual Sha without counting it, or draw one"
                    .into(),
                insert_text: "effect choose_use_virtual_sha_or_draw(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect use_virtual_card_on_selected_targets".into(),
                kind: 15,
                detail: "Use one virtual Duel or Tiesuo on active-skill targets".into(),
                insert_text:
                    "effect use_virtual_card_on_selected_targets(${1:self}, juedou);".into(),
            },
            LspCompletionItem {
                label: "effect target_discard_then_duel_if_non_sha_and_hp_gte_owner".into(),
                kind: 15,
                detail: "Target discards, then receives Duel when its non-Sha discard and HP allow"
                    .into(),
                insert_text:
                    "effect target_discard_then_duel_if_non_sha_and_hp_gte_owner(${1:self}, ${2:event_source});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_target_lose_hp_discard_hand_sha_if_basic".into(),
                kind: 15,
                detail: "Choose a target, lose HP, then conditionally use a virtual Sha".into(),
                insert_text:
                    "effect choose_target_lose_hp_discard_hand_sha_if_basic(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect deal_damage_ban_active_if_dying".into(),
                kind: 15,
                detail: "Deal damage and ban this active skill if it creates dying".into(),
                insert_text:
                    "effect deal_damage_ban_active_if_dying(${1:self}, ${2:event_source}, ${3:1}, self_skill);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect record_selected_play_phase_category".into(),
                kind: 15,
                detail: "Record the selected card category for this play phase".into(),
                insert_text: "effect record_selected_play_phase_category(self_skill);".into(),
            },
            LspCompletionItem {
                label: "effect choose_target_reclaim_equipment_draw_if_range_reduced".into(),
                kind: 15,
                detail: "Reclaim target equipment and draw if its attack range shrinks".into(),
                insert_text:
                    "effect choose_target_reclaim_equipment_draw_if_range_reduced(${1:self}, ${2:event_source}, self_skill);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_use_virtual_card".into(),
                kind: 15,
                detail: "Use a source-free Wanjian or choose a Duel target".into(),
                insert_text:
                    "effect choose_use_virtual_card(${1:self}, ${2:wanjian}, self_skill);".into(),
            },
            LspCompletionItem {
                label: "effect discard_equipment_pair_with_armor".into(),
                kind: 15,
                detail: "Discard equipment from two selected targets, including armor".into(),
                insert_text: "effect discard_equipment_pair_with_armor(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect discard_target_equipment_gain_mount".into(),
                kind: 15,
                detail: "Discard target equipment, then gain it when it is a mount".into(),
                insert_text:
                    "effect discard_target_equipment_gain_mount(${1:self}, ${2:event_player});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect discard_target_equipment_draw_weapon_range".into(),
                kind: 15,
                detail: "Discard target equipment; a weapon makes its former owner draw its range"
                    .into(),
                insert_text:
                    "effect discard_target_equipment_draw_weapon_range(${1:self}, ${2:event_source});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_target_reveal_hand_damage_if_sha".into(),
                kind: 15,
                detail: "Choose and reveal a nonempty hand, then damage it when it contains Sha"
                    .into(),
                insert_text:
                    "effect choose_target_reveal_hand_damage_if_sha(${1:self}, ${2:1});".into(),
            },
            LspCompletionItem {
                label: "effect choose_other_give_one_hand_equip".into(),
                kind: 15,
                detail: "Choose another living player, then give them one hand/equipment card"
                    .into(),
                insert_text: "effect choose_other_give_one_hand_equip(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect choose_reveal_hand_card_bind_category_target".into(),
                kind: 15,
                detail: "Reveal a Play-phase player's hand card and bind its category".into(),
                insert_text:
                    "effect choose_reveal_hand_card_bind_category_target(${1:self}, ${2:event_player});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect recover_to_max_then_optional_exchange_skill".into(),
                kind: 15,
                detail:
                    "Recover to max, then optionally exchange another skill to restore this one"
                        .into(),
                insert_text:
                    "effect recover_to_max_then_optional_exchange_skill(${1:self}, ${2:skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect reveal_hand_discard_kind".into(),
                kind: 15,
                detail: "Publicly reveal a hand, then discard every matching effective card kind"
                    .into(),
                insert_text:
                    "effect reveal_hand_discard_kind(${1:event_player}, ${2:shan});".into(),
            },
            LspCompletionItem {
                label: "effect discard_target_card_gain_mount".into(),
                kind: 15,
                detail: "Discard a scoped target card, then gain it when it is a mount".into(),
                insert_text: "effect discard_target_card_gain_mount(${1:self}, ${2:event_player}, ${3:hand_equip_judge});".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_target_card".into(),
                kind: 15,
                detail: "Choose and discard one scoped card from a target".into(),
                insert_text: "effect choose_discard_target_card(${1:self}, ${2:event_player}, ${3:hand_equip});".into(),
            },
            LspCompletionItem {
                label: "effect reveal_hand_to_player".into(),
                kind: 15,
                detail: "Reveal a complete hand only to one observer".into(),
                insert_text:
                    "effect reveal_hand_to_player(${1:self}, ${2:event_source});".into(),
            },
            LspCompletionItem {
                label: "effect reveal_hand".into(),
                kind: 15,
                detail: "Publicly reveal a complete hand".into(),
                insert_text: "effect reveal_hand(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect choose_optional_discard_target_card".into(),
                kind: 15,
                detail: "Optionally choose and discard one scoped target card".into(),
                insert_text: "effect choose_optional_discard_target_card(${1:self}, ${2:event_source}, ${3:hand});".into(),
            },
            LspCompletionItem {
                label: "effect reveal_hand_choose_discard_color".into(),
                kind: 15,
                detail: "Reveal a hand, then let its owner choose and discard one color".into(),
                insert_text:
                    "effect reveal_hand_choose_discard_color(${1:event_player});".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_target_cards_recover_same_category".into(),
                kind: 15,
                detail: "Discard target cards and recover when their categories match".into(),
                insert_text: "effect choose_discard_target_cards_recover_same_category(${1:self}, ${2:event_source}, ${3:2}, ${4:1});".into(),
            },
            LspCompletionItem {
                label: "effect choose_scskuiji_discard".into(),
                kind: 15,
                detail: "Discard up to count distinct-suit cards from owner+target hands".into(),
                insert_text: "effect choose_scskuiji_discard(${1:self}, ${2:event_source}, ${3:4});".into(),
            },
            LspCompletionItem {
                label: "effect choose_pskuiji_discard".into(),
                kind: 15,
                detail: "Like scskuiji; under-discarder loses 1 HP on asymmetric counts".into(),
                insert_text: "effect choose_pskuiji_discard(${1:self}, ${2:event_source}, ${3:4});".into(),
            },
            LspCompletionItem {
                label:
                    "effect choose_compare_selected_hand_card_second_failure_seat_damage".into(),
                kind: 15,
                detail: "Compare, branch on a strict win, and resolve a repeated-failure seat path"
                    .into(),
                insert_text: "effect choose_compare_selected_hand_card_second_failure_seat_damage(${1:self}, ${2:event_source}, ${3:1}, ${4:2}, ${5:1}, ${6:1});".into(),
            },
            LspCompletionItem {
                label: "effect deal_damage".into(),
                kind: 15,
                detail: "Deal normal, fire, or thunder damage".into(),
                insert_text:
                    "effect deal_damage(self, ${1:event_source}, ${2:1}, ${3:fire});".into(),
            },
            LspCompletionItem {
                label: "effect draw_cards".into(),
                kind: 15, // Snippet
                detail: "Draw N cards for self".into(),
                insert_text: "effect draw_cards(self, ${1:1});".into(),
            },
            LspCompletionItem {
                label: "effect draw_each_by_seat".into(),
                kind: 15,
                detail: "Draw equally for two players in seat order".into(),
                insert_text:
                    "effect draw_each_by_seat(${1:event_player}, ${2:event_source}, ${3:1});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect draw_then_give_one".into(),
                kind: 15,
                detail: "Draw, then force one hand/equipment gift".into(),
                insert_text: "effect draw_then_give_one(self, ${1:event_source}, ${2:2}, ${3:any_hand_equip});".into(),
            },
            LspCompletionItem {
                label: "effect draw_then_take_one".into(),
                kind: 15,
                detail: "Draw, then let another player take one hand/equipment card".into(),
                insert_text:
                    "effect draw_then_take_one(self, ${1:event_source}, ${2:3});".into(),
            },
            LspCompletionItem {
                label: "effect draw_then_choose_give_or_discard_hand".into(),
                kind: 15,
                detail: "Draw, then give an exact number of hand cards or discard them".into(),
                insert_text:
                    "effect draw_then_choose_give_or_discard_hand(self, ${1:2}, ${2:2});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_give_or_discard_hand".into(),
                kind: 15,
                detail: "Give hand cards to a hand-empty target or discard them".into(),
                insert_text:
                    "effect choose_give_or_discard_hand(self, ${1:2}, hand_empty);".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_hand_equip_then_recover_target".into(),
                kind: 15,
                detail: "Optionally discard an exact cost, then recover a specified target"
                    .into(),
                insert_text: "effect choose_discard_hand_equip_then_recover_target(self, ${1:event_player}, ${2:2}, ${3:1});".into(),
            },
            LspCompletionItem {
                label: "effect take_one_card".into(),
                kind: 15,
                detail:
                    "Let a player take one card from hand/equipment/judgment zones".into(),
                insert_text:
                    "effect take_one_card(${1:event_player}, self, ${2:hand_equip_judge});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect take_one_event_target_hand".into(),
                kind: 15,
                detail: "Take one hand card from the event counterpart".into(),
                insert_text: "effect take_one_event_target_hand(self);".into(),
            },
            LspCompletionItem {
                label: "effect choose_end_extra_draw_or_play_by_used_suits".into(),
                kind: 15,
                detail: "Choose extra Draw/Play phases after End from used-card suits".into(),
                insert_text:
                    "effect choose_end_extra_draw_or_play_by_used_suits(self);".into(),
            },
            LspCompletionItem {
                label: "effect lose_hp_double_current_damage_then_replace".into(),
                kind: 15,
                detail: "Lose HP, double the bound damage, then replace this skill".into(),
                insert_text:
                    "effect lose_hp_double_current_damage_then_replace(self, ${1:replacement_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect take_target_hand_then_repeat_until_max".into(),
                kind: 15,
                detail: "Take from a larger hand, then optionally repeat until tied for maximum"
                    .into(),
                insert_text:
                    "effect take_target_hand_then_repeat_until_max(self, ${1:event_source}, self_skill);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect gain_event_cards".into(),
                kind: 15,
                detail: "Gain all captured event cards still in discard".into(),
                insert_text: "effect gain_event_cards(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect gain_card_if_discard".into(),
                kind: 15,
                detail: "Gain one physical card if it remains in discard".into(),
                insert_text:
                    "effect gain_card_if_discard(${1:self}, ${2:event_card});".into(),
            },
            LspCompletionItem {
                label: "effect choose_draw_or_discard_target_card_record_sha".into(),
                kind: 15,
                detail: "Choose draw/discard and bind the current Sha target".into(),
                insert_text:
                    "effect choose_draw_or_discard_target_card_record_sha(self, event_card, event_player);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect gain_target_equipment".into(),
                kind: 15,
                detail: "Gain every target equipment card in one slot".into(),
                insert_text:
                    "effect gain_target_equipment(${1:self}, ${2:event_source}, ${3:weapon});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect discard_all_target_equipment".into(),
                kind: 15,
                detail: "Discard all currently discardable target equipment".into(),
                insert_text:
                    "effect discard_all_target_equipment(${1:self}, ${2:event_player});".into(),
            },
            LspCompletionItem {
                label: "effect increase_current_sha_target_damage".into(),
                kind: 15,
                detail: "Increase current Sha damage for exactly one target".into(),
                insert_text: "effect increase_current_sha_target_damage(${1:self}, ${2:event_card}, ${3:event_player}, ${4:1});".into(),
            },
            LspCompletionItem {
                label: "effect turn_over".into(),
                kind: 15,
                detail: "Toggle a player's face-up/face-down state".into(),
                insert_text: "effect turn_over(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect restore_character_state".into(),
                kind: 15,
                detail: "Restore a player to unlinked and face-up".into(),
                insert_text: "effect restore_character_state(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect discard_all_hand_equip_judge".into(),
                kind: 15,
                detail: "Discard every hand, equipment, and judgment card".into(),
                insert_text: "effect discard_all_hand_equip_judge(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect damage_then_discard_all_judge".into(),
                kind: 15,
                detail: "Self-damage, then discard the current judgment zone".into(),
                insert_text: "effect damage_then_discard_all_judge(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect replace_draw_gain_top_two_add_skills_if_different_colors".into(),
                kind: 15,
                detail: "Replace draw, gain two revealed cards, grant skills when colors differ"
                    .into(),
                insert_text: "effect replace_draw_gain_top_two_add_skills_if_different_colors(${1:self}, ${2:first_skill_def}, ${3:second_skill_def});".into(),
            },
            LspCompletionItem {
                label: "effect choose_gain_target_card_alternate_zone_then_target_draw".into(),
                kind: 15,
                detail: "Gain one target card from each of hand/equipment at most once per Play phase"
                    .into(),
                insert_text: "effect choose_gain_target_card_alternate_zone_then_target_draw(${1:self}, ${2:event_source});".into(),
            },
            LspCompletionItem {
                label: "effect remove_skill_by_def".into(),
                kind: 15,
                detail: "Remove all matching skill instances from a player".into(),
                insert_text: "effect remove_skill_by_def(self, ${1:skill_def});".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_hand_add_skill_by_color".into(),
                kind: 15,
                detail: "Optionally discard a hand card and grant a skill by color".into(),
                insert_text: "effect choose_discard_hand_add_skill_by_color(self, ${1:red_skill_def}, ${2:black_skill_def});".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_matching_equip_color_add_skill".into(),
                kind: 15,
                detail: "Discard a matching-color non-basic card and grant a color lock".into(),
                insert_text: "effect choose_discard_matching_equip_color_add_skill(self, ${1:event_player}, ${2:skill_def});".into(),
            },
            LspCompletionItem {
                label: "effect use_virtual_sha_from_card_color_lock_source_if_no_sha".into(),
                kind: 15,
                detail: "Let the recorded source retaliate when no Sha was used".into(),
                insert_text:
                    "effect use_virtual_sha_from_card_color_lock_source_if_no_sha(self);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_current_card_use_enhancements".into(),
                kind: 15,
                detail: "Choose one or two exact-card enhancements for the suspended use".into(),
                insert_text:
                    "effect choose_current_card_use_enhancements(self, ${1:event_card}, ${2:helper_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect make_current_card_unrespondable".into(),
                kind: 15,
                detail: "Make only the suspended physical card use unrespondable".into(),
                insert_text:
                    "effect make_current_card_unrespondable(self, ${1:event_card}, ${2:helper_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_draw_or_discard_add_skill".into(),
                kind: 15,
                detail: "Choose draw or discard and grant the matching temporary skill".into(),
                insert_text:
                    "effect choose_draw_or_discard_add_skill(self, ${1:draw_skill_def}, ${2:discard_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_three_way_draw_or_discard_add_skill".into(),
                kind: 15,
                detail:
                    "Cancel or choose draw one, draw two with a helper, or discard with a helper"
                        .into(),
                insert_text:
                    "effect choose_three_way_draw_or_discard_add_skill(self, ${1:draw_skill_def}, ${2:discard_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_discarded_black_basic_as_bingliang".into(),
                kind: 15,
                detail: "Repeatedly use eligible triggering discard cards as Bingliang".into(),
                insert_text: "effect choose_discarded_black_basic_as_bingliang(self);".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_judge_use_virtual_sha_draw_if_no_damage".into(),
                kind: 15,
                detail: "Discard a judgment card, use virtual Sha, and draw if it deals no damage"
                    .into(),
                insert_text:
                    "effect choose_discard_judge_use_virtual_sha_draw_if_no_damage(self, ${1:event_player});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect reveal_top_gain_non_basic_or_swap_basic".into(),
                kind: 15,
                detail: "Reveal the draw-pile top card, then gain or swap it by category".into(),
                insert_text: "effect reveal_top_gain_non_basic_or_swap_basic(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect discard_one_hand_or_equip".into(),
                kind: 15,
                detail: "Force one hand/equipment discard".into(),
                insert_text: "effect discard_one_hand_or_equip(self);".into(),
            },
            LspCompletionItem {
                label: "effect disable_skill_by_def_until_turn_end".into(),
                kind: 15,
                detail: "Temporarily disable one owned skill definition".into(),
                insert_text:
                    "effect disable_skill_by_def_until_turn_end(self, ${1:skill_def});".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_one_then_gain_target_equipment".into(),
                kind: 15,
                detail: "Optionally discard one card, then gain a target equipment slot".into(),
                insert_text: "effect choose_discard_one_then_gain_target_equipment(self, ${1:event_source}, ${2:weapon});".into(),
            },
            LspCompletionItem {
                label: "effect discard_all_hand_and_equip".into(),
                kind: 15,
                detail: "Discard all hand and equipment cards".into(),
                insert_text: "effect discard_all_hand_and_equip(self);".into(),
            },
            LspCompletionItem {
                label: "effect remove_all_skills".into(),
                kind: 15,
                detail: "Remove all current skills from a player".into(),
                insert_text: "effect remove_all_skills(${1:event_source});".into(),
            },
            LspCompletionItem {
                label: "effect judge_gain_equipment_by_color".into(),
                kind: 15,
                detail: "Judge, then gain equipment by result color".into(),
                insert_text: "effect judge_gain_equipment_by_color(self);".into(),
            },
            LspCompletionItem {
                label: "effect judge_recover_on_suit".into(),
                kind: 15,
                detail: "Judge, then recover when the effective suit matches".into(),
                insert_text:
                    "effect judge_recover_on_suit(${1:event_player}, ${2:heart}, ${3:1});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect judge_recover_other_on_suit".into(),
                kind: 15,
                detail: "Judge for one player, then recover another on a suit match".into(),
                insert_text: "effect judge_recover_other_on_suit(${1:event_source}, ${2:self}, ${3:spade}, ${4:1});"
                    .into(),
            },
            LspCompletionItem {
                label: "effect force_others_use_sha_on_nearest_or_lose_hp".into(),
                kind: 15,
                detail: "In seat order, require every other player to Sha a nearest player or lose HP".into(),
                insert_text:
                    "effect force_others_use_sha_on_nearest_or_lose_hp(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect judge_damage_other_on_suit".into(),
                kind: 15,
                detail: "Judge, then force damage to another player on a suit match".into(),
                insert_text: "effect judge_damage_other_on_suit(${1:self}, ${2:spade}, ${3:1}, ${4:thunder});"
                    .into(),
            },
            LspCompletionItem {
                label: "effect discard_selected_record_unique_category_target".into(),
                kind: 15,
                detail: "Record a turn- or play-phase-unique logical category and target, then discard the selected card".into(),
                insert_text:
                    "effect discard_selected_record_unique_category_target(self, ${1:event_source});"
                        .into(),
            },
            LspCompletionItem {
                label:
                    "effect choose_target_draw_or_discard_by_skill_use_count_link_if_wounded"
                        .into(),
                kind: 15,
                detail: "Choose target draw/discard by use count, then conditionally link owner"
                    .into(),
                insert_text:
                    "effect choose_target_draw_or_discard_by_skill_use_count_link_if_wounded(self, ${1:event_source});"
                        .into(),
            },
            LspCompletionItem {
                label:
                    "effect choose_target_draw_or_discard_by_skill_use_count_toggle_link_if_wounded"
                        .into(),
                kind: 15,
                detail:
                    "Choose target draw/discard by use count, then toggle owner link state if wounded"
                        .into(),
                insert_text:
                    "effect choose_target_draw_or_discard_by_skill_use_count_toggle_link_if_wounded(self, ${1:event_source});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_reveal_other_hand_card_remember_category".into(),
                kind: 3,
                detail: "Reveal another player's hand card and remember its category".into(),
                insert_text:
                    "effect choose_reveal_other_hand_card_remember_category(self);".into(),
            },
            LspCompletionItem {
                label: "effect draw_then_give_exact_record_phase_target".into(),
                kind: 3,
                detail: "Draw, give an exact count, and record the phase target".into(),
                insert_text:
                    "effect draw_then_give_exact_record_phase_target(self, ${1:event_player}, ${2:2});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect clear_phase_gift_target".into(),
                kind: 3,
                detail: "Clear the current skill's delayed Play-phase gift target".into(),
                insert_text: "effect clear_phase_gift_target(self);".into(),
            },
            LspCompletionItem {
                label: "effect choose_gain_resolved_sha_or_shan_then_optional_use_sha".into(),
                kind: 3,
                detail: "Give resolved Sha/Shan cards, then optionally use a physical Sha".into(),
                insert_text:
                    "effect choose_gain_resolved_sha_or_shan_then_optional_use_sha(self, event_card);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_gain_all_resolved_sha_shan_then_color_rewards".into(),
                kind: 3,
                detail: "Give all resolved Sha/Shan cards, then apply red/black rewards".into(),
                insert_text:
                    "effect choose_gain_all_resolved_sha_shan_then_color_rewards(self, event_card);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect skip_current_phase".into(),
                kind: 3,
                detail: "Skip the current non-idle phase".into(),
                insert_text: "effect skip_current_phase(self);".into(),
            },
            LspCompletionItem {
                label: "effect finish_current_phase_abort_resolution".into(),
                kind: 3,
                detail: "Abort the current resolution and finish the phase".into(),
                insert_text: "effect finish_current_phase_abort_resolution(self);".into(),
            },
            LspCompletionItem {
                label: "effect gain_top_card_to_skill_pile".into(),
                kind: 3,
                detail: "Move the draw-pile top card into this skill's public pile".into(),
                insert_text: "effect gain_top_card_to_skill_pile(self);".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_skill_pile_card_add_marked_skill".into(),
                kind: 3,
                detail: "Discard one card from this skill's pile and grant a marked helper"
                    .into(),
                insert_text:
                    "effect choose_discard_skill_pile_card_add_marked_skill(self, ${1:event_player}, ${2:helper_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect gain_one_hand_from_selected_targets".into(),
                kind: 3,
                detail: "Gain one hand card from each active target in submitted order".into(),
                insert_text: "effect gain_one_hand_from_selected_targets(self);".into(),
            },
            LspCompletionItem {
                label: "effect choose_repolu_targets_draw".into(),
                kind: 3,
                detail: "Choose living Repolu recipients and draw by activation count".into(),
                insert_text: "effect choose_repolu_targets_draw(self);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_xinfu_guolun".into(),
                kind: 3,
                detail: "Reveal and optionally exchange Guolun cards, then draw by rank".into(),
                insert_text: "effect resolve_xinfu_guolun(self, event_source);".into(),
            },
            LspCompletionItem {
                label: "effect choose_other_targets_by_skipped_phase_count_damage".into(),
                kind: 3,
                detail: "Choose other targets up to the skipped-phase count and damage each"
                    .into(),
                insert_text:
                    "effect choose_other_targets_by_skipped_phase_count_damage(self, ${1:1});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect reveal_uniform_hand_draw_self_choose_others".into(),
                kind: 3,
                detail: "Reveal a uniform-color hand, then draw for self and chosen others"
                    .into(),
                insert_text: "effect reveal_uniform_hand_draw_self_choose_others(self);".into(),
            },
            LspCompletionItem {
                label: "effect choose_card_user_or_add_target".into(),
                kind: 3,
                detail: "Choose a card-user transfer or add the chooser as a target".into(),
                insert_text: "effect choose_card_user_or_add_target(self, event_card);".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_equipment_reduce_current_damage".into(),
                kind: 3,
                detail: "Discard an equipment card to reduce current damage by one".into(),
                insert_text:
                    "effect choose_discard_equipment_reduce_current_damage(self);".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_equipment_prevent_current_damage".into(),
                kind: 3,
                detail: "Discard an equipment card to prevent the current damage".into(),
                insert_text:
                    "effect choose_discard_equipment_prevent_current_damage(self);".into(),
            },
            LspCompletionItem {
                label: "effect choose_move_target_equipment_or_draw".into(),
                kind: 3,
                detail: "Move one compatible target equipment, draw one, or cancel".into(),
                insert_text:
                    "effect choose_move_target_equipment_or_draw(self, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect choose_move_target_equipment_and_or_draw".into(),
                kind: 3,
                detail: "Move compatible equipment, draw one, do both, or cancel".into(),
                insert_text:
                    "effect choose_move_target_equipment_and_or_draw(self, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect choose_target_discard_attack_range_cards_or_increase_damage_weapon_forces_both".into(),
                kind: 3,
                detail: "Target discards attack-range cards or takes +1 damage; a weapon forces both".into(),
                insert_text: "effect choose_target_discard_attack_range_cards_or_increase_damage_weapon_forces_both(self, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect choose_draw_or_recast_add_skill".into(),
                kind: 3,
                detail: "Cancel, draw and gain one helper, or recast and gain another".into(),
                insert_text:
                    "effect choose_draw_or_recast_add_skill(self, ${1:draw_skill_def}, ${2:recast_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_gain_sha_for_opening_extra_play".into(),
                kind: 3,
                detail: "Optionally gain a distance-unlimited Sha in this skill's opening Play phase"
                    .into(),
                insert_text: "effect choose_gain_sha_for_opening_extra_play(self);".into(),
            },
            LspCompletionItem {
                label: "effect finish_opening_extra_play_sha_gain".into(),
                kind: 3,
                detail: "Finish the opening-phase Sha gain and apply its no-damage penalty".into(),
                insert_text: "effect finish_opening_extra_play_sha_gain(self);".into(),
            },
            LspCompletionItem {
                label: "effect reveal_target_hand_gain_shan_armor_or_draw_two".into(),
                kind: 3,
                detail: "Reveal a target hand; gain Shan and armor, or draw two".into(),
                insert_text:
                    "effect reveal_target_hand_gain_shan_armor_or_draw_two(self, event_source);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect give_all_equipment_choose_recover_or_damage".into(),
                kind: 3,
                detail: "Give all equipment, then choose count-bound recovery or damage".into(),
                insert_text:
                    "effect give_all_equipment_choose_recover_or_damage(self, event_source);"
                        .into(),
            },
            LspCompletionItem {
                label:
                    "effect choose_target_discard_weapon_range_hand_or_increase_damage".into(),
                kind: 3,
                detail: "Target discards weapon-range hand cards or bound Sha damage +1"
                    .into(),
                insert_text:
                    "effect choose_target_discard_weapon_range_hand_or_increase_damage(self, event_player);"
                        .into(),
            },
            LspCompletionItem {
                label:
                    "effect choose_target_discard_distance_hand_or_increase_damage".into(),
                kind: 3,
                detail: "Target discards max(1, 4-distance) hand cards or bound damage +1"
                    .into(),
                insert_text:
                    "effect choose_target_discard_distance_hand_or_increase_damage(self, event_player);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_draw_view_two_gain_or_move_equipment".into(),
                kind: 3,
                detail: "Draw, privately view two and gain one, or move equipment".into(),
                insert_text:
                    "effect choose_draw_view_two_gain_or_move_equipment(self, event_player);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_hand_count_different_target_draw_or_random_discard".into(),
                kind: 3,
                detail: "Choose a different-hand-count target to draw or randomly discard"
                    .into(),
                insert_text:
                    "effect choose_hand_count_different_target_draw_or_random_discard(self);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect increase_current_draw_count".into(),
                kind: 3,
                detail: "Increase and atomically consume the pending draw-phase count".into(),
                insert_text: "effect increase_current_draw_count(self, ${1:1});".into(),
            },
            LspCompletionItem {
                label: "effect skip_next_play_phase".into(),
                kind: 3,
                detail: "Skip the current player's upcoming Play phase".into(),
                insert_text: "effect skip_next_play_phase(self);".into(),
            },
            LspCompletionItem {
                label: "effect skip_next_judge_phase".into(),
                kind: 3,
                detail: "Skip the player's next judgment phase".into(),
                insert_text: "effect skip_next_judge_phase(self);".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_damage_endpoint_card_increase_if_suit".into(),
                kind: 3,
                detail: "Discard from either damage endpoint and conditionally increase damage"
                    .into(),
                insert_text: "effect choose_discard_damage_endpoint_card_increase_if_suit(self, event_player, ${1:diamond}, ${2:1});".into(),
            },
            LspCompletionItem {
                label: "effect choose_other_draw_or_recover".into(),
                kind: 3,
                detail: "Choose another player, then that target draws or recovers".into(),
                insert_text:
                    "effect choose_other_draw_or_recover(self, ${1:2}, ${2:1}, ${3:event_amount});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_other_recover".into(),
                kind: 3,
                detail: "Force choosing another player, then recover that target".into(),
                insert_text: "effect choose_other_recover(self, ${1:2});".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_hand_equip_then_remove_self_skill".into(),
                kind: 3,
                detail: "Forced dynamic discard followed by one-shot skill cleanup".into(),
                insert_text:
                    "effect choose_discard_hand_equip_then_remove_self_skill(self, ${1:1});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_hand_equip_then_recover".into(),
                kind: 3,
                detail: "Discard an exact hand/equipment count, then recover".into(),
                insert_text:
                    "effect choose_discard_hand_equip_then_recover(self, ${1:2}, ${2:1});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect draw_selected_targets".into(),
                kind: 3,
                detail: "Draw cards for every active-skill target in seat order".into(),
                insert_text: "effect draw_selected_targets(${1:3});".into(),
            },
            LspCompletionItem {
                label: "effect draw_linked_players".into(),
                kind: 3,
                detail: "Draw cards for every linked living player in seat order".into(),
                insert_text: "effect draw_linked_players(${1:1});".into(),
            },
            LspCompletionItem {
                label: "effect reveal_top_gain_if_color".into(),
                kind: 3,
                detail: "Reveal the draw-pile top card and gain it if its printed color matches"
                    .into(),
                insert_text: "effect reveal_top_gain_if_color(self, ${1:red});".into(),
            },
            LspCompletionItem {
                label: "effect recover_selected_targets".into(),
                kind: 3,
                detail: "Recover HP for every active-skill target in seat order".into(),
                insert_text: "effect recover_selected_targets(${1:1});".into(),
            },
            LspCompletionItem {
                label: "effect damage_then_draw_selected_targets".into(),
                kind: 3,
                detail: "Damage then draw for every active-skill target in seat order".into(),
                insert_text:
                    "effect damage_then_draw_selected_targets(self, ${1:1}, ${2:1});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_youdi".into(),
                kind: 3,
                detail: "Resolve Youdi's target-owned discard and conditional counter-gain".into(),
                insert_text: "effect resolve_youdi(self);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_kuangfu".into(),
                kind: 3,
                detail: "Resolve Kuangfu's equipment discard or move choice".into(),
                insert_text: "effect resolve_kuangfu(self, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_shanjia".into(),
                kind: 3,
                detail: "Resolve Shanjia's draw, discard, and optional virtual Sha".into(),
                insert_text: "effect resolve_shanjia(self, ${1:count});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_new_yajiao".into(),
                kind: 3,
                detail: "Resolve New Yajiao's reveal, recipient, and mismatch discard".into(),
                insert_text: "effect resolve_new_yajiao(self, event_card);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_dingpin".into(),
                kind: 3,
                detail: "Resolve Dingpin's judgment and color-dependent result".into(),
                insert_text: "effect resolve_dingpin(self, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect recover_if_min_current_hp".into(),
                kind: 3,
                detail: "Recover only while tied for the lowest current HP".into(),
                insert_text: "effect recover_if_min_current_hp(self, ${1:1});".into(),
            },
            LspCompletionItem {
                label:
                    "effect choose_target_discard_excess_then_self_discard_or_target_draw".into(),
                kind: 3,
                detail: "Choose a target, discard excess cards, then self-discard or target draw"
                    .into(),
                insert_text:
                    "effect choose_target_discard_excess_then_self_discard_or_target_draw(self);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_decadezhenjun".into(),
                kind: 3,
                detail:
                    "Resolve Decade Zhenjun in Prepare or End without changing legacy Rezhenjun"
                        .into(),
                insert_text: "effect resolve_decadezhenjun(self);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_sp_xiaoguo".into(),
                kind: 3,
                detail: "Resolve SP Xiaoguo without changing Standard Xiaoguo".into(),
                insert_text: "effect resolve_sp_xiaoguo(self, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect choose_gain_card_from_each_other_replace_draw".into(),
                kind: 3,
                detail: "Replace the normal draw by gaining one card from each other player"
                    .into(),
                insert_text: "effect choose_gain_card_from_each_other_replace_draw(self);".into(),
            },
            LspCompletionItem {
                label:
                    "effect draw_then_choose_gain_one_from_damage_target_and_neighbors".into(),
                kind: 3,
                detail: "Draw, then gain from the damaged player and both living neighbors"
                    .into(),
                insert_text:
                    "effect draw_then_choose_gain_one_from_damage_target_and_neighbors(self, event_player);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect reveal_discard_suit_gain_or_use".into(),
                kind: 15,
                detail: "Reveal cards, then discard-by-suit gain and/or use one".into(),
                insert_text:
                    "effect reveal_discard_suit_gain_or_use(self, ${1:mark_def}, ${2:3}, ${3:2});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_or_fire_damage_skip_phases".into(),
                kind: 15,
                detail: "Choose discard and/or fire damage, with conditional phase skips".into(),
                insert_text:
                    "effect choose_discard_or_fire_damage_skip_phases(self, ${1:2}, ${2:1});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect remove_cards_by_kind".into(),
                kind: 15,
                detail: "Move every card of one kind out of the game".into(),
                insert_text: "effect remove_cards_by_kind(${1:horse_off});".into(),
            },
        ],
        CompletionContext::ModifierTarget => vec![
            LspCompletionItem {
                label: "distance_from".into(),
                kind: 14,
                detail: "Distance from owner to others".into(),
                insert_text: "distance_from".into(),
            },
            LspCompletionItem {
                label: "distance_from_if_hp_gte_target".into(),
                kind: 14,
                detail: "Set final distance to one when owner HP is at least target HP".into(),
                insert_text: "distance_from_if_hp_gte_target".into(),
            },
            LspCompletionItem {
                label: "global_distance_to".into(),
                kind: 14,
                detail: "Distance to queried targets while the owner is alive".into(),
                insert_text: "global_distance_to".into(),
            },
            LspCompletionItem {
                label: "attack_range".into(),
                kind: 14,
                detail: "Owner's attack range".into(),
                insert_text: "attack_range".into(),
            },
            LspCompletionItem {
                label: "global_attack_range".into(),
                kind: 14,
                detail: "Every living player's attack range while the owner is alive".into(),
                insert_text: "global_attack_range".into(),
            },
            LspCompletionItem {
                label: "global_hand_limit".into(),
                kind: 14,
                detail: "Every queried player's hand limit while the owner is alive".into(),
                insert_text: "global_hand_limit".into(),
            },
            LspCompletionItem {
                label: "global_damage_taken_amount(nature)".into(),
                kind: 15,
                detail: "Adjust every queried player's ordinary incoming damage while the owner is alive".into(),
                insert_text: "global_damage_taken_amount(${1:any})".into(),
            },
            LspCompletionItem {
                label: "global_damage_taken_final_amount(nature)".into(),
                kind: 15,
                detail: "Adjust every queried player's final incoming damage while the owner is alive".into(),
                insert_text: "global_damage_taken_final_amount(${1:any})".into(),
            },
            LspCompletionItem {
                label: "hand_limit".into(),
                kind: 14,
                detail: "Owner's hand size limit".into(),
                insert_text: "hand_limit".into(),
            },
            LspCompletionItem {
                label: "hand_limit_base".into(),
                kind: 14,
                detail: "Owner's base hand size limit".into(),
                insert_text: "hand_limit_base".into(),
            },
            LspCompletionItem {
                label: "hand_limit_base_max_hp".into(),
                kind: 14,
                detail: "Use maximum HP as the owner's hand-limit base".into(),
                insert_text: "hand_limit_base_max_hp".into(),
            },
            LspCompletionItem {
                label: "skip_judge_phase".into(),
                kind: 14,
                detail: "Skip the owner's judgment phase".into(),
                insert_text: "skip_judge_phase".into(),
            },
            LspCompletionItem {
                label: "skip_discard_phase".into(),
                kind: 14,
                detail: "Skip the owner's discard phase".into(),
                insert_text: "skip_discard_phase".into(),
            },
            LspCompletionItem {
                label: "restore_skipped_phase".into(),
                kind: 14,
                detail: "Restore the owner's phase after a skipped/cancelled audit event".into(),
                insert_text: "restore_skipped_phase".into(),
            },
            LspCompletionItem {
                label: "turn_face_down_cancel".into(),
                kind: 14,
                detail: "Prevent the owner from being turned face-down".into(),
                insert_text: "turn_face_down_cancel".into(),
            },
            LspCompletionItem {
                label: "turn_over_cancel".into(),
                kind: 14,
                detail: "Prevent either face-up/face-down transition".into(),
                insert_text: "turn_over_cancel".into(),
            },
            LspCompletionItem {
                label: "discard_phase_includes_equip".into(),
                kind: 14,
                detail: "Count equipment in the owner's discard-to-limit phase".into(),
                insert_text: "discard_phase_includes_equip".into(),
            },
            LspCompletionItem {
                label: "draw_phase_scry_four_split_two".into(),
                kind: 14,
                detail: "Optionally reorder four draw-pile cards as two top and two bottom".into(),
                insert_text: "draw_phase_scry_four_split_two".into(),
            },
            LspCompletionItem {
                label: "initial_hand_count".into(),
                kind: 14,
                detail: "Adjust the owner's initial hand size".into(),
                insert_text: "initial_hand_count".into(),
            },
            LspCompletionItem {
                label: "card_suit(selector)".into(),
                kind: 15,
                detail: "Override one physical suit for the skill owner".into(),
                insert_text: "card_suit(${1:spade})".into(),
            },
            LspCompletionItem {
                label: "card_name(selector)".into(),
                kind: 15,
                detail: "Override the rule card name for matching owner cards".into(),
                insert_text: "card_name(${1:red})".into(),
            },
            LspCompletionItem {
                label: "card_nature(selector)".into(),
                kind: 15,
                detail: "Override the damage nature of matching owner cards".into(),
                insert_text: "card_nature(${1:sha})".into(),
            },
            LspCompletionItem {
                label: "sha_unrespondable(selector)".into(),
                kind: 15,
                detail: "Prevent targets from answering matching cards resolved as Sha".into(),
                insert_text: "sha_unrespondable(${1:heart})".into(),
            },
            LspCompletionItem {
                label: "card_use_unrespondable(selector)".into(),
                kind: 15,
                detail: "Prevent other players from answering matching cards used by the owner"
                    .into(),
                insert_text: "card_use_unrespondable(${1:ordinary_trick})".into(),
            },
            LspCompletionItem {
                label: "sha_target_unrespondable".into(),
                kind: 14,
                detail: "Prevent Sha from being answered when its target owns this skill".into(),
                insert_text: "sha_target_unrespondable".into(),
            },
            LspCompletionItem {
                label: "damage_taken_source_range_threshold".into(),
                kind: 14,
                detail: "Adjust incoming damage around the source attack-range threshold".into(),
                insert_text: "damage_taken_source_range_threshold".into(),
            },
            LspCompletionItem {
                label: "sha_damage_source_range_threshold".into(),
                kind: 14,
                detail: "Adjust the owner's logical Sha damage around an attack-range threshold"
                    .into(),
                insert_text: "sha_damage_source_range_threshold".into(),
            },
            LspCompletionItem {
                label: "card_distance_unlimited".into(),
                kind: 14,
                detail: "Ignore target-distance limits for the owner's cards".into(),
                insert_text: "card_distance_unlimited".into(),
            },
            LspCompletionItem {
                label: "card_discard_protected(selector)".into(),
                kind: 15,
                detail: "Protect matching equipment from other players' discard effects".into(),
                insert_text: "card_discard_protected(${1:equip_non_mount})".into(),
            },
            LspCompletionItem {
                label: "card_gain_protected(selector)".into(),
                kind: 15,
                detail: "Protect matching cards from other players' gain effects".into(),
                insert_text: "card_gain_protected(${1:any})".into(),
            },
            LspCompletionItem {
                label: "card_distance_unlimited_if_target_hand_gte_owner(selector)".into(),
                kind: 15,
                detail:
                    "Ignore distance when the target has at least as many hand cards as the owner"
                        .into(),
                insert_text: "card_distance_unlimited_if_target_hand_gte_owner(${1:bingliang})"
                    .into(),
            },
            LspCompletionItem {
                label: "card_distance_limit(selector)".into(),
                kind: 15,
                detail: "Override the target-distance limit for matching cards".into(),
                insert_text: "card_distance_limit(${1:bingliang})".into(),
            },
            LspCompletionItem {
                label: "virtual_bagua_if_armor_slot_empty".into(),
                kind: 14,
                detail: "Provide a virtual Bagua response while the armor slot is enabled and empty"
                    .into(),
                insert_text: "virtual_bagua_if_armor_slot_empty".into(),
            },
            LspCompletionItem {
                label: "sha_response_count_if_other_female".into(),
                kind: 14,
                detail: "Increase Sha Shan count when the other participant has female sex"
                    .into(),
                insert_text: "sha_response_count_if_other_female".into(),
            },
            LspCompletionItem {
                label: "sha_response_count_if_source_owner_target_distance_le_one".into(),
                kind: 14,
                detail: "Increase Shan count for the owner's Sha targets within distance 1"
                    .into(),
                insert_text: "sha_response_count_if_source_owner_target_distance_le_one".into(),
            },
            LspCompletionItem {
                label: "duel_opponent_response_count".into(),
                kind: 14,
                detail: "Increase the Sha count required from the owner's Duel opponent".into(),
                insert_text: "duel_opponent_response_count".into(),
            },
            LspCompletionItem {
                label: "tao_use_save_cancel_for_non_dying_others_during_owner_turn".into(),
                kind: 14,
                detail: "During the owner's turn, block Tao for other non-dying players".into(),
                insert_text:
                    "tao_use_save_cancel_for_non_dying_others_during_owner_turn".into(),
            },
            LspCompletionItem {
                label: "hand_card_use_response_cancel".into(),
                kind: 14,
                detail: "Prevent the owner from using or responding with hand cards".into(),
                insert_text: "hand_card_use_response_cancel".into(),
            },
            LspCompletionItem {
                label: "hand_card_gained_this_turn_only".into(),
                kind: 12,
                detail: "Only hand cards gained this turn are playable in Play phase".into(),
                insert_text: "hand_card_gained_this_turn_only".into(),
            },
            LspCompletionItem {
                label: "card_targets_all(selector)".into(),
                kind: 15,
                detail: "Require every otherwise-legal matching-card target".into(),
                insert_text: "card_targets_all(${1:sha})".into(),
            },
            LspCompletionItem {
                label: "card_target_cancel(selector)".into(),
                kind: 15,
                detail: "Prevent matching cards from selecting the skill owner".into(),
                insert_text: "card_target_cancel(${1:tao})".into(),
            },
            LspCompletionItem {
                label: "armor_card_target_cancel(selector)".into(),
                kind: 15,
                detail: "Prevent matching cards while the owner's armor effect is active".into(),
                insert_text: "armor_card_target_cancel(${1:ordinary_trick})".into(),
            },
            LspCompletionItem {
                label: "armor_damage_taken_amount(nature)".into(),
                kind: 15,
                detail: "Adjust incoming damage while the owner's armor effect is active".into(),
                insert_text: "armor_damage_taken_amount(${1:fire})".into(),
            },
            LspCompletionItem {
                label: "armor_link_gain_cancel".into(),
                kind: 14,
                detail: "Prevent an unlinked owner from becoming linked while armor is active"
                    .into(),
                insert_text: "armor_link_gain_cancel".into(),
            },
            LspCompletionItem {
                label: "link_loss_cancel".into(),
                kind: 14,
                detail: "Prevent a linked owner from becoming unlinked".into(),
                insert_text: "link_loss_cancel".into(),
            },
            LspCompletionItem {
                label: "ignore_armor".into(),
                kind: 14,
                detail: "Make source-side effects ignore armor".into(),
                insert_text: "ignore_armor".into(),
            },
            LspCompletionItem {
                label: "armor_effect_disabled".into(),
                kind: 14,
                detail: "Disable armor effects owned by this player".into(),
                insert_text: "armor_effect_disabled".into(),
            },
            LspCompletionItem {
                label: "card_target_cancel_from_current(any)".into(),
                kind: 15,
                detail: "Prevent the current turn owner from targeting the skill owner".into(),
                insert_text: "card_target_cancel_from_current(any)".into(),
            },
            LspCompletionItem {
                label: "card_target_cancel_if_target_lacks_matching_suit".into(),
                kind: 14,
                detail: "Prevent targeting the owner when their hand lacks the card's suit".into(),
                insert_text: "card_target_cancel_if_target_lacks_matching_suit".into(),
            },
            LspCompletionItem {
                label: "card_target_cancel_if_other_user_hp_gte_target".into(),
                kind: 14,
                detail:
                    "Prevent another player with at least the owner's HP from targeting the owner"
                        .into(),
                insert_text: "card_target_cancel_if_other_user_hp_gte_target".into(),
            },
            LspCompletionItem {
                label: "card_target_cancel_if_other_user_distance_gt_one(selector)".into(),
                kind: 15,
                detail:
                    "Prevent matching cards from a player more than one final-distance away"
                        .into(),
                insert_text:
                    "card_target_cancel_if_other_user_distance_gt_one(${1:ordinary_trick})"
                        .into(),
            },
            LspCompletionItem {
                label:
                    "card_target_cancel_if_other_user_hand_gte_target_and_armor_empty(selector)"
                        .into(),
                kind: 15,
                detail: "Prevent matching cards from a hand-rich other player targeting an armorless owner"
                    .into(),
                insert_text:
                    "card_target_cancel_if_other_user_hand_gte_target_and_armor_empty(${1:sha})"
                        .into(),
            },
            LspCompletionItem {
                label: "card_targets_self_only".into(),
                kind: 14,
                detail: "Prevent the owner's cards from targeting other players".into(),
                insert_text: "card_targets_self_only".into(),
            },
            LspCompletionItem {
                label: "card_use_cancel(selector?)".into(),
                kind: 14,
                detail: "Prevent the owner from proactively using all or matching cards".into(),
                insert_text: "card_use_cancel${1:(trick)}".into(),
            },
            LspCompletionItem {
                label: "card_action_cancel_for_skill_color_mask".into(),
                kind: 14,
                detail: "Prevent use, response, and saves matching the skill-state color mask"
                    .into(),
                insert_text: "card_action_cancel_for_skill_color_mask".into(),
            },
            LspCompletionItem {
                label: "ignore_armor_for_skill_card_policy".into(),
                kind: 14,
                detail: "Ignore armor for exact cards recorded in this skill's policy state"
                    .into(),
                insert_text: "ignore_armor_for_skill_card_policy".into(),
            },
            LspCompletionItem {
                label: "card_use_unrespondable_for_skill_card_policy".into(),
                kind: 14,
                detail: "Prevent responses to exact cards recorded in this skill's policy state"
                    .into(),
                insert_text: "card_use_unrespondable_for_skill_card_policy".into(),
            },
            LspCompletionItem {
                label: "card_distance_unlimited_for_skill_card_policy".into(),
                kind: 14,
                detail: "Ignore distance for exact cards recorded in this skill's policy state"
                    .into(),
                insert_text: "card_distance_unlimited_for_skill_card_policy".into(),
            },
            LspCompletionItem {
                label: "card_use_unrespondable_within_responder_to_owner_distance".into(),
                kind: 14,
                detail: "Prevent responses when responder-to-owner distance is within the threshold"
                    .into(),
                insert_text:
                    "card_use_unrespondable_within_responder_to_owner_distance".into(),
            },
            LspCompletionItem {
                label: "card_use_unrespondable_within_owner_to_responder_distance".into(),
                kind: 14,
                detail: "Prevent responses when owner-to-responder distance is within the threshold"
                    .into(),
                insert_text:
                    "card_use_unrespondable_within_owner_to_responder_distance".into(),
            },
            LspCompletionItem {
                label: "card_response_cancel".into(),
                kind: 14,
                detail: "Prevent the owner from responding with any card".into(),
                insert_text: "card_response_cancel".into(),
            },
            LspCompletionItem {
                label: "card_save_cancel(tao?)".into(),
                kind: 14,
                detail: "Prevent all saves, or specifically prevent 桃 saves".into(),
                insert_text: "card_save_cancel${1:(tao)}".into(),
            },
            LspCompletionItem {
                label: "sha_response_cancel".into(),
                kind: 14,
                detail: "Prevent the owner from responding with 杀".into(),
                insert_text: "sha_response_cancel".into(),
            },
            LspCompletionItem {
                label: "sha_ignored_hand_limit".into(),
                kind: 14,
                detail: "Compatibility alias: exclude 杀 from hand-limit count and phase discard"
                    .into(),
                insert_text: "sha_ignored_hand_limit".into(),
            },
            LspCompletionItem {
                label: "card_ignored_hand_limit(selector)".into(),
                kind: 15,
                detail: "Exclude cards selected by name/category/suit/color from phase discard"
                    .into(),
                insert_text: "card_ignored_hand_limit(${1:sha})".into(),
            },
            LspCompletionItem {
                label: "sha_use_cancel".into(),
                kind: 14,
                detail: "Prevent the owner from proactively using 杀".into(),
                insert_text: "sha_use_cancel".into(),
            },
            LspCompletionItem {
                label: "card_use_unlimited(selector?)".into(),
                kind: 14,
                detail: "Remove ordinary use limits for all cards or a selected card family".into(),
                insert_text: "card_use_unlimited".into(),
            },
            LspCompletionItem {
                label: "sha_distance_unlimited_if_rank_lt_skill_mark".into(),
                kind: 14,
                detail: "Lower-ranked Sha ignores distance while this skill has a rank mark".into(),
                insert_text: "sha_distance_unlimited_if_rank_lt_skill_mark".into(),
            },
            LspCompletionItem {
                label: "sha_distance_unlimited_if_target_distance_lte_rank".into(),
                kind: 14,
                detail: "Sha ignores distance to targets within its physical rank".into(),
                insert_text: "sha_distance_unlimited_if_target_distance_lte_rank".into(),
            },
            LspCompletionItem {
                label: "sha_no_use_count_if_rank_gt_skill_mark".into(),
                kind: 14,
                detail: "Higher-ranked Sha bypasses and does not consume its ordinary use count"
                    .into(),
                insert_text: "sha_no_use_count_if_rank_gt_skill_mark".into(),
            },
            LspCompletionItem {
                label: "card_target_limit(selector)".into(),
                kind: 15,
                detail: "Adjust the maximum target count for matching independently-targeted cards"
                    .into(),
                insert_text: "card_target_limit(${1:ordinary_trick})".into(),
            },
            LspCompletionItem {
                label: "observer_foreign_hand_faces_visible".into(),
                kind: 14,
                detail: "Reveal other players' hand-card faces to the owning observer".into(),
                insert_text: "observer_foreign_hand_faces_visible".into(),
            },
            LspCompletionItem {
                label: "observer_hidden_identities_visible".into(),
                kind: 14,
                detail: "Reveal normally hidden identities to the owning observer".into(),
                insert_text: "observer_hidden_identities_visible".into(),
            },
        ],
        CompletionContext::ModifierMode => vec![
            LspCompletionItem {
                label: "add".into(),
                kind: 14,
                detail: "Add to base value".into(),
                insert_text: "add".into(),
            },
            LspCompletionItem {
                label: "override".into(),
                kind: 14,
                detail: "Override base value".into(),
                insert_text: "override".into(),
            },
            LspCompletionItem {
                label: "multiply".into(),
                kind: 14,
                detail: "Multiply base value".into(),
                insert_text: "multiply".into(),
            },
            LspCompletionItem {
                label: "min".into(),
                kind: 14,
                detail: "Minimum value".into(),
                insert_text: "min".into(),
            },
            LspCompletionItem {
                label: "max".into(),
                kind: 14,
                detail: "Maximum value".into(),
                insert_text: "max".into(),
            },
        ],
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
        match word {
            "skill" => {
                "Define a skill.\n\n```\nskill name(def_id) [locked] [lord_only] [non_gainable] [conversion] {\n  trigger ... { effect ...; }\n  active play_phase [uses_per_turn N|owner_hp|owner_lost_hp_min_one] [once_per_game] [owner_missing_skill def_id] [hand_gt_hp|hand_gt_max_hp|hand_limit_positive|equip_nonempty|hand_nonempty|face_up] [targets min max [other] [lowest_hp|highest_hp] [unlinked] [wounded] [male] [max_by_owner_lost_hp|max_by_owner_hp] [hp_at_least_owner|hp_gt_owner|hp_lte_owner] [hp_gt N] [hand_gt_owner|hand_lte_owner] [hand_lt N] [hand_nonempty] [can_compare] [can_use_on_owner card_kind] [second_can_use_juedou_on_first] [equipment_pair_with_armor] [has_equip] [has_discardable_equip] [has_gainable_equip] [in_attack_range] [hand_equip_nonempty] [has_discardable_hand_equip] [unused_by_skill_this_turn|unused_by_skill_this_play_phase] [match_selected_card_count] [selected_cards_match_target_equip_count]] [select_hand|select_he min max [allow_cancel] [same_suit|same_color|distinct_suits] [distinct_categories] [card_equip|card_armor|card_sha|card_basic|card_nonbasic|card_non_equip|card_red] [unused_category_by_skill_this_turn|unused_category_by_skill_this_play_phase|unused_category_by_owner_history_this_turn]] { effect ...; }\n  modifier target mode value;\n}\n```\n`lord_only` keeps every trigger, active, view-as, and modifier inactive unless the owner has the Lord identity. `conversion` exposes the skill's branch to generic conversion rules."
            }
            "trigger" => {
                "Define a triggered ability.\n\nEvents include game_started, damage_before, damage_prevented, damage, hp_lost, card_used, phase_prepare, phase_play_end, phase_draw_skipped, phase_end, death, dying, dying_after, max_hp_changed, cards_drawn.\nFilters include owner_is_target, owner_is_source, owner_is_source_or_target, owner_alive, owner_alive_card_targets_gt_one, and the documented source/rate filters."
            }
            "active" => {
                "Define a player-activated skill body. Optional `uses_per_turn N|owner_hp|owner_lost_hp_min_one|living_rebel_count`, `uses_per_play_phase N`, `once_per_game`, `owner_missing_skill def_id`, `hand_gt_hp`, `hand_gt_max_hp`, `hand_limit_positive`, `equip_nonempty`, `hand_nonempty`, `face_up`, `targets min max`, and `select_hand`/`select_he` clauses are authoritative contracts enforced by the engine."
            }
            "owner_missing_skill" => {
                "Active-skill clause: activation is legal only while the owner does not have the given stable skill definition. Requires Rule VM ABI 234."
            }
            "view_as" => {
                "Treat matching physical source cards as another card kind. Optional `from hand_equip`, `from hand_expansion`, `up_to 2` or `exactly 2`, `same_suit`, `nature fire|thunder`, `multi_card_bonus damage_1|recover_1|discard_current_hand_equip`, `uses_per_turn N`, `ignore_distance`, `red_unrespondable`, `black_no_use_count`, `no_use_count`, `target_lacks_source_suit_unrespondable`, and `refund_use_count_if_no_damage` clauses are authoritative. Source-free dynamic families use `basic|ordinary_trick any any from none branch N once_per_round next_branch N cost ...`; `cost swap_lower_hand uses_per_turn 1` swaps whole hands with a lower-hand target before continuing; delegated `shan response` and `sha any` rules use `from group_<tag>` and ask matching other players in seat order."
            }
            "exactly" => {
                "View-as clause: require exactly the following number of physical source cards."
            }
            "choose_discard_weapon_or_lose_hp" => {
                "Effect: pay one of two active-skill costs: discard one current physical weapon from hand/equipment, or lose 1 HP. If only one cost is legal it is forced. Requires Rule VM ABI 222.\n\n```\neffect choose_discard_weapon_or_lose_hp(self);\n```"
            }
            "uses_per_turn" => {
                "Active/view-as clause: positive fixed per-turn activation limit; active skills may use `owner_hp`, `owner_lost_hp_min_one`, or `living_rebel_count` to read a dynamic limit at each activation. Checked by the engine and reset at turn start."
            }
            "uses_per_play_phase" => {
                "Active-only clause: positive activation limit reset on every entry to a Play phase, including extra Play phases. Requires Rule VM ABI 301."
            }
            "choose_range" => {
                "Expression: `choose_range(min, max)` offers every non-negative integer in the inclusive dynamic range and evaluates to the selected token. At most 255 options; requires Rule VM ABI 92."
            }
            "branch" => {
                "Source-free view-as clause: require the skill instance's current conversion branch (0..31)."
            }
            "once_per_round" => {
                "Source-free view-as clause: the declared branch may resolve only once in the current round."
            }
            "next_branch" => {
                "Source-free view-as clause: select this conversion branch after a successful cost payment."
            }
            "cost" => {
                "Source-free view-as clause: authoritative cost `gain_max_hp_1`, `disable_equip_slot`, deterministic `guhuo`, or a tagged `face_down_hand` card."
            }
            "ignore_distance" => {
                "View-as clause: skip only the converted card's ordinary distance check; all other target, use-count, response, and effect rules remain active. Requires Rule VM ABI 4."
            }
            "red_unrespondable" => {
                "View-as Sha clause: when the selected source card is effectively red, only this converted Sha becomes unrespondable. Requires Rule VM ABI 14."
            }
            "black_no_use_count" => {
                "View-as Sha clause: when the selected source card is effectively black, only this converted Sha skips the ordinary use-limit check and count increment. Requires Rule VM ABI 14."
            }
            "no_use_count" => {
                "View-as Sha clause: this converted Sha skips the ordinary use-limit check and count increment. Requires Rule VM ABI 521."
            }
            "target_lacks_source_suit_unrespondable" => {
                "View-as Sha clause: each target without a hand card matching the physical source card's effective suit cannot respond to this Sha. Requires Rule VM ABI 521."
            }
            "refund_use_count_if_no_damage" => {
                "View-as Sha clause: after this exact converted Sha finishes, refund its ordinary use count if it dealt no damage. Requires Rule VM ABI 354."
            }
            "non_basic" => "Card filter: match cards whose effective kind is not a basic card.",
            "hand_gt_hp" => {
                "Active-skill clause: activation is legal only while the owner's hand count exceeds current HP."
            }
            "hand_gt_max_hp" => {
                "Active-skill clause: activation is legal only while the owner's hand count exceeds maximum HP."
            }
            "hand_limit_positive" => {
                "Active-skill clause: activation is legal only while the owner's current hand limit is positive. Requires Rule VM ABI 143."
            }
            "equip_nonempty" => {
                "Active-skill clause: activation is legal only while the owner's equipment zone is nonempty. Requires Rule VM ABI 58."
            }
            "face_up" => {
                "Active-skill clause: activation is legal only while the owner is face up. Requires Rule VM ABI 151."
            }
            "once_per_game" => {
                "Active-skill clause: the skill may resolve at most once during the match. The lifetime counter is not reset at turn boundaries. Requires Rule VM ABI 58."
            }
            "targets" => {
                "Active-skill clause: inclusive minimum and maximum number of distinct living player targets. Append `other` to exclude the owner, `lowest_hp`/`highest_hp` to require a global HP extremum, `unlinked` to require an unlinked target, `wounded` to require current HP below maximum HP, `male` to require male/double-sex targets, `hp_at_least_owner`/`hp_gt_owner`/`hp_lte_owner` to compare target HP with the owner's current HP, `hp_gt N` to require current HP above a literal threshold, `hand_gt_owner` to require more hand cards than the owner, `hand_lt N` to require fewer than N hand cards, `hand_nonempty` to require at least one hand card, `can_compare` to require both participants to have hand cards, `can_use_on_owner card_kind` to require a legal source-free use against the owner, `second_can_use_juedou_on_first` to require the second of exactly two targets to be able to use Duel on the first, `equipment_pair_with_armor` for an exact two-target equipment pair containing discardable armor, `has_equip` to require a nonempty equipment zone, `has_discardable_equip` to require equipment the skill owner may discard, `has_gainable_equip` to require equipment the skill owner may gain, `in_attack_range` to require the owner's current attack range, `hand_equip_nonempty` to require a nonempty hand/equipment region, `has_discardable_hand_equip` to require a discardable hand/equipment card, `unused_by_skill_this_turn` or `unused_by_skill_this_play_phase` to reject targets this skill already recorded in that scope, `match_selected_card_count` to bind target count to card count, or `selected_cards_match_target_equip_count` to bind the selected-card count to the sole target's equipment count."
            }
            "other" => "Active-target constraint: the skill owner cannot be selected as a target.",
            "lowest_hp" => {
                "Active-target constraint: every selected target must tie for the lowest current HP among living players."
            }
            "highest_hp" => {
                "Active-target constraint: every selected target must tie for the highest current HP among living players."
            }
            "unlinked" => {
                "Active-target constraint: every selected target must currently be unlinked."
            }
            "wounded" => {
                "Active-target constraint: every selected target must currently be alive and below maximum HP. Requires Rule VM ABI 109."
            }
            "male" => {
                "Active-target constraint: every selected target must be male or double-sex. Requires Rule VM ABI 151."
            }
            "hp_at_least_owner" => {
                "Active-target constraint: every selected target's current HP must be at least the skill owner's current HP."
            }
            "hp_gt_owner" => {
                "Active-target constraint: every selected target's current HP must be greater than the skill owner's current HP. Requires Rule VM ABI 121."
            }
            "hp_lte_owner" => {
                "Active-target constraint: every selected target's current HP must be less than or equal to the skill owner's current HP. Requires Rule VM ABI 402."
            }
            "hp_gt" => {
                "Active-target constraint: `hp_gt N` requires every selected target to have more than N current HP. Revalidated by the engine and requires Rule VM ABI 132."
            }
            "hand_gt_owner" => {
                "Active-target constraint: every selected target must currently have more hand cards than the skill owner. Revalidated by the engine and requires Rule VM ABI 113."
            }
            "hand_lte_owner" => {
                "Active-target constraint: every selected target must currently have no more hand cards than the skill owner. Revalidated by the engine and requires Rule VM ABI 514."
            }
            "pair_hand_difference_lte_three" => {
                "Active-target constraint: exactly two selected targets must have hand-card counts differing by at most three. Revalidated by the engine and requires Rule VM ABI 535."
            }
            "resolve_pothaoshi" => {
                "Effect: action 0 optionally authorizes another living player to use/respond with the owner's current hand until action 1 clears all grants at the owner's next turn start. The first two borrowed uses that empty the owner's hand draw it to three. Requires Rule VM ABI 536.\n\n```\neffect resolve_pothaoshi(self, 0);\n```"
            }
            "resolve_spmiaolve" => {
                "Effect: action 0 creates up to two globally suit-distinct Mantianguohai cards in the owner's hand; action 1 lets the owner cancel, draw two, or gain one canonical zhinang. Requires Rule VM ABI 539.\n\n```\neffect resolve_spmiaolve(self, 0);\n```"
            }
            "resolve_pottuntian" => {
                "势屯田 effect: gain charge after a qualifying loss, grow the maximum and draw when full at turn start, or spend charge to distribute heart cards and trigger 势凿险 threshold rewards. Requires Rule VM ABI 543."
            }
            "resolve_potjixi" => {
                "势急袭 effect: reset/record current-turn card targets, then discard one card from the turn player and use an ignore-distance virtual Shunshou on selected recorded targets. Requires Rule VM ABI 543."
            }
            "resolve_potceduan" => {
                "势策断 effect: collect one hidden hand-card choice from every player in the selected target's attack range, reveal them together, and use all owner hand cards of every tied majority color as an unlimited physical Sha. Requires Rule VM ABI 544."
            }
            "resolve_pottaishici" => {
                "势太史慈 effect: reset/collect 战烈 marks, resolve its enhanced virtual Sha, draw-to-X then duel for 酣战, and choose the limited 振锋 recovery or X-source rewrite. Requires Rule VM ABI 548."
            }
            "resolve_mbzhonghui" => {
                "势钟会 effect: spend/settle 俟资 charge, grow 挟志 limits, awaken 狱难, and upgrade 克昌. Requires Rule VM ABI 553."
            }
            "resolve_potjiejie" => {
                "势辛宪英诫节 effect: privately reveal the current actor hand, choose a suit, apply the matching discard/unlimited-use or draw-pile gain branch, and track the twice-per-round unique-maximum 清识 bonus. Requires Rule VM ABI 554."
            }
            "resolve_potqingshi" => {
                "势辛宪英清识 effect: choose any living player; allied participants each draw one, otherwise both discard one hand/equipment card. Requires Rule VM ABI 554."
            }
            "resolve_potweiyan" => {
                "势魏延 effect: resolve 忠傲 mission state, 壮誓 first-N-card helpers, 饮战 Sha follow-up, upgraded 狂骨, or 困奋. Requires Rule VM ABI 558."
            }
            "resolve_potzhouyu" => {
                "势周瑜 effect: resolve 炽沄 gifts, 焰洄 reveal history, 焚涛 linked fire-damage choices, or 雄姿 branch locking. Requires Rule VM ABI 561."
            }
            "resolve_potlvyi" => {
                "势吕壹 effect: resolve 隳律 tracked-card settlement, 颂谳 Wuzhong response conversion, or 势施 exact-card response locks. Requires Rule VM ABI 564."
            }
            "resolve_potcaoshuang" => {
                "势曹爽 effect: resolve 典易/奢权 timing assignment, persistent card tags, 渐专 awakening, and 溺尊 pile use. Requires Rule VM ABI 568."
            }
            "resolve_potyuji" => {
                "势于吉 effect: resolve 道转 basic-card conversion costs and 符济 tagged-card rewards. Requires Rule VM ABI 570."
            }
            "resolve_rehengjiang" => {
                "SP 横江 effect: stack the current turn player's temporary hand-limit penalty and settle the owner's draw at turn end. Requires Rule VM ABI 572."
            }
            "owner_alive_turn_ended_rehengjiang_pending" => {
                "横江 turn-end filter: the living helper owner has pending state for the ended turn. Requires Rule VM ABI 572."
            }
            "owner_gained_cards_potzhouyu" => {
                "炽沄 gain filter: the living owner gained cards and still has hand cards. Requires Rule VM ABI 561."
            }
            "owner_source_first_target_has_hand_potyanhui" => {
                "焰洄 target filter: the owner used a card, this is its first target event, and some target has hand cards. Requires Rule VM ABI 561."
            }
            "owner_alive_fire_damage_potfentao" => {
                "焚涛 fire-damage filter; target and chain eligibility are revalidated by the resolver. Requires Rule VM ABI 561."
            }
            "owner_alive_potfentao_pending_damage" => {
                "焚涛 post-damage filter for its pending relink target. Requires Rule VM ABI 561."
            }
            "owner_source_sha_damage_before_potyinzhan_condition" => {
                "饮战 Damage-before filter: the owner dealt Sha damage and has no more HP or hand cards than the target. Requires Rule VM ABI 558."
            }
            "owner_source_damage_within_one" => {
                "狂骨 Damage filter: the owner dealt positive damage to a living target at distance 1 or less. Requires Rule VM ABI 558."
            }
            "owner_alive_play_phase_actor_hand_nonempty_potjiejie_unused" => {
                "诫节 Play-phase filter: owner and current actor are alive, the actor has hand cards, and no 诫节 has been invoked by that actor this phase. Requires Rule VM ABI 554."
            }
            "potqingyan" => {
                "View-as cost: reveal exactly min(round uses + 1, 5) hand cards without consuming them; unavailable while any card shown by the previous activation remains in hand. Requires Rule VM ABI 544."
            }
            "owner_lost_non_damage_card_pottuntian_charge_not_full" => {
                "Cards-moved filter: the owner lost at least one non-damage card from a personal zone, excluding an equipment card consumed by using it, while 势屯田 charge is below its stored maximum. Requires Rule VM ABI 543."
            }
            "resolve_spyingjia" => {
                "Effect: actions 0/1 reset and record same-name trick uses; action 2 may discard one hand card and queue an extra turn for any living target, granting the supplied draw helper. Requires Rule VM ABI 539.\n\n```\neffect resolve_spyingjia(self, 2, 3612952432);\n```"
            }
            "hand_lt" => {
                "Active-target constraint: `hand_lt N` requires every selected target to have fewer than N hand cards. Revalidated by the engine and requires Rule VM ABI 128."
            }
            "hand_nonempty" => {
                "Active-skill/target constraint: outside `targets`, the owner must have at least one hand card (ABI 151); inside `targets`, every selected target must have at least one hand card (ABI 150)."
            }
            "can_compare" => {
                "Active-target constraint: the skill owner and every selected target must each currently have at least one hand card. Revalidated before opening the owner's card selection; requires Rule VM ABI 47."
            }
            "can_use_on_owner" => {
                "Active-target constraint: every selected target must currently be able to use the following source-free card kind on the skill owner. The engine checks user state, use locks, target locks, self-target rules, and target-count modifiers. Currently supports `juedou`; requires Rule VM ABI 23."
            }
            "second_can_use_juedou_on_first" => {
                "Active-target constraint: requires exactly two other targets and checks that the second can currently use a source-free Duel on the first. Target order is authoritative and legality is checked before active-skill card costs. Requires Rule VM ABI 183."
            }
            "equipment_pair_with_armor" => {
                "Active-target constraint: requires exactly two distinct living targets, each with discardable equipment, and at least one with discardable armor. Requires Rule VM ABI 32."
            }
            "has_equip" => {
                "Active-target constraint: every selected target must currently have at least one card in its equipment zone. Requires Rule VM ABI 97."
            }
            "has_discardable_equip" => {
                "Active-target constraint: every selected target must currently have at least one equipment card the skill owner may discard. Equipment-zone ownership and discard protection are authoritatively revalidated. Requires Rule VM ABI 136."
            }
            "has_gainable_equip" => {
                "Active-target constraint: every selected target must currently have at least one equipment card the skill owner may gain. Equipment-zone ownership and gain protection are authoritatively revalidated. Requires Rule VM ABI 327."
            }
            "has_gainable_hand_equip" => {
                "Active-target constraint: every selected target must currently have at least one hand/equipment card the skill owner may gain. Zone ownership and gain protection are authoritatively revalidated. Requires Rule VM ABI 355."
            }
            "in_attack_range" => {
                "Active-target constraint: every selected target must currently be inside the skill owner's attack range. Distance and attack-range modifiers are authoritatively revalidated. Requires Rule VM ABI 155."
            }
            "owner_in_target_attack_range" => {
                "Active-target constraint: the skill owner must currently be inside every selected target's attack range. Requires Rule VM ABI 544."
            }
            "hand_equip_nonempty" => {
                "Active-target constraint: every selected target must currently have at least one card in hand or equipment. Requires Rule VM ABI 192."
            }
            "has_discardable_hand_equip" => {
                "Active-target constraint: every selected target must currently have at least one hand/equipment card the skill owner may discard. Zone ownership and discard protection are authoritatively revalidated. Requires Rule VM ABI 155."
            }
            "unused_by_skill_this_turn" => {
                "Active-target constraint: rejects any target already recorded in this skill instance's canonical turn-scoped uniqueness state. Requires Rule VM ABI 52."
            }
            "unused_by_skill_this_play_phase" => {
                "Active-target constraint: rejects any target already recorded by this skill in the current Play phase. The target history resets on every entry to a Play phase, including extra Play phases. Requires Rule VM ABI 292."
            }
            "match_selected_card_count" => {
                "Active-target constraint: requires selected-player and selected-card counts to be equal; both declarations must use the same min/max range. Requires Rule VM ABI 109."
            }
            "selected_cards_match_target_equip_count" => {
                "Active-target constraint: requires exactly one equipped target and binds the selected-card count to that target's current equipment-card count. The count is revalidated before the activation resolves. Requires Rule VM ABI 157."
            }
            "select_hand" => {
                "Active-skill clause: inclusive minimum and maximum number of distinct cards selected from the owner's hand. Append `allow_cancel` to allow cancellation at the choice barrier, `same_suit` to require one non-virtual effective suit, `same_color` to require one effective color, `distinct_categories` to require unique basic/trick/equipment categories, `card_sha` to offer only logical Sha cards, `card_basic` to offer only effective basic cards, `card_nonbasic` to exclude effective basic cards, `card_non_equip` to exclude effective equipment cards, or `card_heart` to require an effective heart suit."
            }
            "select_he" => {
                "Active-skill clause: inclusive minimum and maximum number of distinct cards selected from the owner's hand or equipment zone. Append `allow_cancel` to allow cancellation at the choice barrier, `same_suit` to require one non-virtual effective suit, `distinct_categories` to require unique basic/trick/equipment categories, `card_equip` to offer only effective equipment cards, `card_armor` to offer only physical armor cards, `card_sha` to offer only logical Sha cards, `card_basic` to offer only effective basic cards, or `card_nonbasic` to exclude effective basic cards."
            }
            "card_nonbasic" => {
                "Active-card selection filter: offer only cards whose effective kind is not basic. Requires Rule VM ABI 327."
            }
            "card_non_equip" => {
                "Active-card selection filter: offer only cards whose effective kind is not equipment. Requires Rule VM ABI 497."
            }
            "card_heart" => {
                "Active-card selection filter: offer only cards whose effective suit is heart. Requires Rule VM ABI 494."
            }
            "allow_cancel" => {
                "Active-card selection constraint: the selection barrier may be cancelled without paying cards or consuming the active-skill use. Cancellation remains invalid for non-cancellable choices."
            }
            "same_suit" => {
                "Active-card constraint: all selected physical cards must have the same non-virtual effective suit. Requires Rule VM ABI 304."
            }
            "same_color" => {
                "Active-card constraint: all selected cards must have the same effective red/black color."
            }
            "distinct_categories" => {
                "Active-card constraint: selected cards must have pairwise-distinct effective basic/trick/equipment categories. Requires Rule VM ABI 109."
            }
            "sum_effective_number_exact" => {
                "Active hand-card constraint: selected distinct cards must have effective numbers totaling exactly the positive configured value. Requires Rule VM ABI 301."
            }
            "unused_category_by_skill_this_turn" => {
                "Active-card constraint: offers and accepts only cards whose effective basic/trick/equipment category is not yet recorded by this skill in the current turn. Requires Rule VM ABI 52."
            }
            "unused_category_by_skill_this_play_phase" => {
                "Active-card constraint: offers and accepts only cards whose effective basic/trick/equipment category is not yet recorded by this skill in the current play phase. The history resets on every entry to a Play phase. Requires Rule VM ABI 110."
            }
            "unused_category_by_owner_history_this_turn" => {
                "Active-card constraint: offers and accepts only cards whose effective basic/trick/equipment category has not been used or discarded by the owner in the current turn. Requires Rule VM ABI 285."
            }
            "modifier" => {
                "Define a passive modifier.\n\n```\nmodifier target mode value;\n```\nA value may be an integer or a documented query-time source such as owner_hp, owner_lost_hp, owner_skill_mark, owner_skill_use_count, alive_group_count, alive_female_count, alive_other_count_capped_four, alive_wounded_count_zero_or_at_least_four, alive_wounded_count_at_least_N, alive_other_qun_count, alive_field_rank_thirteen_player_count, field_mount_card_count_ceil_half, owner_unique_max_hand, owner_hand_gt_other, owner_hand_lt_other, owner_qun_other_shu, owner_turned_over_diff_other, owner_linked_diff_other, owner_hand_equip_gte_other, owner_sha_damage_dealt_zero_this_turn, queried_player_hand_gt_hp_signed, queried_player_judge_nonempty_other_than_owner, queried_player_linked_other_than_owner, queried_player_other_than_owner, queried_source_enemy_owner_ally_target_other_than_owner, owner_max_hand_before_skill_fires_this_turn, owner_no_mount, owner_armor_empty, owner_healthy, owner_has_weapon, owner_has_treasure, owner_has_heart_equip, alive_mounted_player_count_min_one, owner_equip_count, owner_equip_count_at_least_N, owner_equip_count_odd, owner_equip_count_even, owner_equip_count_half_floor, owner_equip_count_exactly_one, owner_hp_gt_two, owner_hp_le_two, owner_lost_hp_at_least_two, owner_all_others_in_attack_range, owner_sha_used_zero, owner_sha_used_nonzero, owner_sha_used_exactly_one, owner_sha_used_exactly_two, owner_non_damage_hand_type_count, owner_sha_hand_count, owner_distinct_hand_card_suits_used_this_turn, owner_current_turn_card_use_count, owner_play_phase_card_use_count_zero, or owner_turn_damage_dealt; prefix a dynamic value with `-` to subtract it.\nTargets include distance_from, distance_to, global_distance_to, attack_range, global_attack_range, global_hand_limit, global_damage_taken_amount(nature), global_damage_taken_final_amount(nature), hand_limit_base, hand_limit, draw_count, optional_draw_count, sha_use_limit, sha_target_limit, judgment_rewrite, judgment_rewrite_hand_equip, judgment_rewrite_red_hand_equip_expansion, judgment_best_from_draw_pile, card_target_limit(selector), observer_foreign_hand_faces_visible, observer_hidden_identities_visible, damage_amount(selector), card_suit(selector), card_name(selector), card_nature(selector), card_targets_all(selector), card_targets_self_only.\nModes: add, override, multiply, min, max."
            }
            "optional_draw_count" => {
                "Modifier target: offers one draw-phase confirmation per owning skill, then adds that skill's grouped positive contribution."
            }
            "effect" => {
                "Request an effect from the trigger body.\n\n```\neffect draw_cards(self, N);\n```"
            }
            "damage" => "Trigger event: fires after damage is dealt to the owner.",
            "hp_lost" => "Trigger event: fires after HP is lost without damage.",
            "phase_prepare" => "Trigger event: fires at the start of the preparation phase.",
            "phase_begin" => {
                "Trigger event: fires once at the start of a turn, immediately before phase_prepare triggers."
            }
            "phase_judge" => "Trigger event: fires at the start of the judgment phase.",
            "phase_draw" => "Trigger event: fires at the start of the draw phase.",
            "phase_play" => "Trigger event: fires at the start of the play phase.",
            "phase_discard" => "Trigger event: fires at the start of the discard phase.",
            "phase_end" => "Trigger event: fires at the end of a phase.",
            "phase_draw_end" => {
                "Trigger event: fires when the draw phase ends. Requires Rule VM ABI 50."
            }
            "phase_play_end" => "Trigger event: fires when the play phase ends.",
            "phase_any_end" => {
                "Trigger event: fires after any concrete Prepare, Judge, Draw, Play, Discard, or End phase. It carries bounded target and actual-HP-change histories for that one phase. Requires Rule VM ABI 98."
            }
            "phase_draw_skipped" => {
                "Trigger event: fires when a player's draw phase is skipped or cancelled."
            }
            "death" => "Trigger event: fires when a player dies.",
            "card_targeted" => {
                "Trigger event: fires once for each declared target, in target order."
            }
            "card_responded" => "Trigger event: fires when a logical response card is committed.",
            "card_resolved" => {
                "Trigger event: fires once after one logical card finishes all targets, responses, damage, dying, and nested choices. `event_source` is the user, `event_card` is the physical carrier, and `event_targets` preserves declared target order. Requires Rule VM ABI 503."
            }
            "sha_dodged" => {
                "Trigger event: fires once after a target supplies every Shan required to fully dodge one logical Sha. `event_source` is the Sha user, `event_player` is the dodging target, and `event_card` is the physical Sha."
            }
            "sha_resolved" => {
                "Trigger event: fires once after one logical Sha finishes every target, response, damage, dying, and nested choice. `event_source` and `event_player` are the Sha user; `event_card` is its physical carrier. Requires Rule VM ABI 54."
            }
            "hand_compare_started" => {
                "Trigger event: fires after the initiator has selected a valid hand card and before the other participant chooses. `event_player` is the initiator, `event_source` is the target, and the initiator's selected card is captured privately for follow-up effects. Requires Rule VM ABI 65 for comparison effects."
            }
            "hand_compare_finished" => {
                "Trigger event: fires after both comparison cards are revealed and moved to the discard pile. `event_player` is the initiator, `event_source` is the target, and both physical cards are captured in participant order. Requires Rule VM ABI 65 for comparison effects."
            }
            "judgment_resolved" => {
                "Trigger event: fires once after a judgment and every rewrite opportunity complete. `event_player` is the judged player, `event_card` is the delayed trick/equipment cause when present, and the final physical result card is captured as the first event card. Requires Rule VM ABI 86."
            }
            "cards_given" => {
                "Trigger event: fires after an attributed player-to-player gift. `event_source` is the giver, `event_player` is the recipient, and `event_cards` contains the transferred physical cards. Requires Rule VM ABI 181."
            }
            "dying" => "Trigger event: fires when a player enters dying state.",
            "dying_after" => "Trigger event: fires when a saved player leaves dying state.",
            "max_hp_changed" => "Trigger event: fires after a player's maximum HP changes.",
            "distance_from" => "Modifier target: adjusts distance from owner to other players.",
            "distance_from_if_hp_gte_target" => {
                "Modifier target: sets the owner's final distance to a target to one when the owner's HP is at least that target's HP. Requires `override 1`."
            }
            "distance_from_skill_bound_target" => {
                "Modifier target: sets the owner's final distance to this skill instance's bound target to one. Requires `override 1` and Rule VM ABI 196."
            }
            "attack_range" => "Modifier target: adjusts owner's attack range.",
            "hand_limit_base" => {
                "Modifier target: adjusts the base hand limit before ordinary hand-limit bonuses."
            }
            "hand_limit_base_max_hp" => {
                "Modifier target: uses maximum HP instead of current HP as the hand-limit base when non-zero."
            }
            "skip_judge_phase" => {
                "Modifier target: skips the owner's judgment phase while non-zero."
            }
            "skip_discard_phase" => {
                "Modifier target: skips the owner's discard phase while non-zero."
            }
            "restore_skipped_phase" => {
                "Modifier target: after the engine records a skipped/cancelled phase audit event, restore that exact owner phase instead of advancing past it. Covers Judge, Draw, Play, and Discard phase skip paths. Requires `override 1` and Rule VM ABI 111."
            }
            "turn_face_down_cancel" => {
                "Modifier target: prevents a face-up owner from being turned face-down. An owner who is already face-down may still turn face-up. Requires `override 1` and Rule VM ABI 111."
            }
            "turn_over_cancel" => {
                "Modifier target: prevents the owner from changing either face-up/face-down state. Requires `override 1` and Rule VM ABI 164."
            }
            "discard_phase_includes_equip" => {
                "Modifier target: counts the owner's equipment together with hand cards during the ordinary discard-to-limit phase and permits discarding from either zone. Requires `override 1`."
            }
            "draw_phase_scry_four_split_two" => {
                "Modifier target: offers an optional draw-phase reorder of four cards with exactly two left on top and two sent to the bottom."
            }
            "initial_hand_count" => {
                "Modifier target: adjusts the owner's initial hand size before the first turn."
            }
            "card_suit" => {
                "Card-suit modifier: `card_suit(spade) override heart` makes the owner's physical spade cards count as hearts in rule checks."
            }
            "card_name" => {
                "Card-name modifier: `card_name(red) override sha` makes the owner's red cards behave as Sha without mutating the physical cards."
            }
            "card_nature" => {
                "Card-nature modifier: `card_nature(sha) override kami` changes the damage nature of matching cards without mutating their physical identity."
            }
            "sha_unrespondable" => {
                "Modifier target: prevents targets from answering matching cards when they resolve as Sha."
            }
            "card_use_unrespondable" => {
                "Modifier target: prevents other players from answering matching cards used by the skill owner."
            }
            "sha_unrespondable_within_distance" => {
                "Modifier target: prevents a target from answering the owner's Sha when their final distance is at most the positive override value."
            }
            "sha_use_unrespondable_within_distance" => {
                "Modifier target: prevents every target from answering a Sha use when its source or one target owns the modifier and the other participant's final distance to that owner is at most the positive override value."
            }
            "sha_target_unrespondable" => {
                "Target-side modifier: prevents an incoming Sha from being answered when the target owns this skill."
            }
            "damage_taken_source_range_threshold" => {
                "Target-side damage policy: source attack range below the positive threshold caps damage at one; above it adds one."
            }
            "sha_damage_source_range_threshold" => {
                "Source-side logical Sha damage policy: owner attack range below the positive threshold caps final damage at one; above it adds one before later damage modifiers."
            }
            "sha_damage" => {
                "Modifier target: adjusts damage dealt by the owner's Sha/Juedou, or only matching Sha when given a card selector."
            }
            "sha_damage_taken" => {
                "Target-side modifier: adjusts damage received by the owner specifically from a logical Sha."
            }
            "sha_damage_if_outside_target_range" => {
                "Source-side modifier: adjusts logical Sha damage only when the source is outside the target's attack range."
            }
            "card_distance_unlimited" => {
                "Modifier target: ignores ordinary target-distance limits for all cards used by the owner, or only matching cards when given a selector."
            }
            "card_discard_protected" => {
                "Target-side modifier: matching physical equipment in the owner's equipment zone cannot be discarded by another player. Requires `override 1`."
            }
            "card_gain_protected" => {
                "Target-side modifier: matching physical cards in the owner's hand/equipment/judgment zones cannot be gained by another player. Requires `override 1` and Rule VM ABI 300."
            }
            "card_distance_unlimited_if_target_hand_gte_owner" => {
                "Target-aware modifier: matching cards ignore ordinary distance only when the target has at least as many hand cards as the owner."
            }
            "card_distance_limit" => {
                "Card-aware modifier target: overrides the ordinary target-distance limit for matching cards. Requires `override` with a non-negative integer."
            }
            "virtual_bagua_if_armor_slot_empty" => {
                "Provide the owner with a virtual Bagua response only while their armor slot is enabled and empty. Requires `override 1` and Rule VM ABI 87."
            }
            "sha_response_count_if_other_female" => {
                "Sha participant modifier: when the owner is the source or current target and the other participant has female or double sex, add to that target's required Shan count. Requires `add` and Rule VM ABI 89."
            }
            "sha_response_count_if_source_owner_target_distance_le_one" => {
                "Sha source modifier: when the owner uses Sha against a target within final distance one, add to that target's required Shan count. Requires `add` and Rule VM ABI 353."
            }
            "duel_opponent_response_count" => {
                "Duel participant modifier: add to the Sha count required from the modifier owner's opponent in each exchange. Requires `add` and Rule VM ABI 167."
            }
            "tao_use_save_cancel_for_non_dying_others_during_owner_turn" => {
                "Global Tao restriction: during the modifier owner's current turn, other players who are not themselves dying cannot proactively use Tao or use Tao to save a dying player. The turn owner and a dying player remain allowed. Use `override 1`, or `override owner_skill_mark` for a turn-scoped activation mark (ABI 218)."
            }
            "card_target_limit" => {
                "Card-aware modifier target: adjusts the maximum number of distinct targets for matching independently-targeted cards. It requires a selector and Rule VM ABI 19."
            }
            "judgment_rewrite_hand_equip" => {
                "Permit one optional judgment rewrite from the owner's hand or equipment zone. Every eligible rewriter is offered in seat order; requires Rule VM ABI 24."
            }
            "judgment_rewrite_red_hand_equip_expansion" => {
                "Permit one optional judgment rewrite using an effective-red card from the owner's hand, equipment, or expansion zone. Every eligible rewriter is offered in seat order; requires Rule VM ABI 342."
            }
            "judgment_best_from_draw_pile" => {
                "Use the highest-scoring card in the draw pile for each judgment owned by this player; requires Rule VM ABI 72."
            }
            "observer_foreign_hand_faces_visible" => {
                "Observer-projection modifier: reveals every other player's hand-card faces only to the owning seated player or that player's private replay export. Requires `override 1` and Rule VM ABI 20."
            }
            "observer_hidden_identities_visible" => {
                "Observer-projection modifier: reveals identities that would otherwise be hidden only to the owning seated player or that player's private replay export. Requires `override 1` and Rule VM ABI 20."
            }
            "card_targets_all" => {
                "Card-aware modifier target: matching cards must target every otherwise-legal player. Selectors: sha, basic, non_basic, trick, equip, suits, red, black."
            }
            "card_target_cancel" => {
                "Target-side modifier: matching cards cannot select the skill owner. Selectors include names, categories, suits/colors, ordinary_trick, black_trick, and damage_trick; a query-time value such as owner_hand_empty may make the restriction conditional."
            }
            "armor_card_target_cancel" => {
                "Armor modifier: matching cards cannot select the skill owner unless the source ignores armor or the owner's armor effects are disabled."
            }
            "armor_damage_taken_amount" => {
                "Armor modifier: adjusts incoming normal, fire, or thunder damage unless the source ignores armor or the owner's armor effects are disabled."
            }
            "armor_link_gain_cancel" => {
                "Armor modifier: prevents the unlinked owner from gaining linked state while armor is active; removing existing linked state remains allowed."
            }
            "link_loss_cancel" => {
                "Modifier: prevents a living linked owner from losing the chain mark. Requires `override 1` and Rule VM ABI 213."
            }
            "ignore_armor" => {
                "Source-side modifier: suppresses armor-owned targeting and damage modifiers."
            }
            "armor_effect_disabled" => {
                "Target-side modifier: suppresses armor-owned targeting and damage modifiers for the owner."
            }
            "damage_amount" => {
                "Source-side modifier: adjusts non-lose-HP damage dealt by the owner. An optional (normal), (fire), or (thunder) selector limits the original nature."
            }
            "damage_as_hp_loss" => {
                "Source-side replacement: damage dealt by the owner becomes equal HP loss. Add `(sha)` to limit the replacement to logical Sha damage. Requires `override 1`."
            }
            "damage_between_owner_and_other_as_hp_loss" => {
                "Relationship replacement: damage dealt by the owner to another player, or received by the owner from another living player, becomes equal HP loss. Requires `override 1`."
            }
            "damage_taken_cancel" => {
                "Target-side modifier: cancels damage received by the owner. With (normal), (fire), or (thunder), it only matches that resolved nature."
            }
            "damage_taken_amount" => {
                "Target-side modifier: adjusts non-lose-HP damage received by the owner. An optional (normal), (fire), or (thunder) selector limits the resolved nature."
            }
            "global_damage_taken_amount" => {
                "Living-owner aura: adjusts ordinary incoming damage for every queried player. An optional nature selector limits the resolved nature."
            }
            "global_damage_taken_final_amount" => {
                "Living-owner aura: adjusts final incoming damage after ordinary and armor modifiers for every queried player. An optional nature selector limits the resolved nature."
            }
            "global_draw_count" => {
                "Living-owner aura: adjusts every queried player's draw-phase count. Requires Rule VM ABI 215."
            }
            "global_hand_limit" => {
                "Living-owner aura: adjusts every queried player's hand limit while the skill owner is alive."
            }
            "global_sha_use_limit" => {
                "Living-owner aura: adjusts every queried player's ordinary Sha use limit. Requires Rule VM ABI 215."
            }
            "global_distance_to" => {
                "Living-owner aura: adjusts distance from a queried source to a queried target."
            }
            "damage_taken_final_amount" => {
                "Late target-side modifier: adjusts final non-lose-HP damage after equipment and ordinary amount modifiers. An optional (normal), (fire), or (thunder) selector limits the resolved nature."
            }
            "damage_taken_nature" => {
                "Target-side modifier: overrides the resolved nature of incoming non-lose-HP damage. Use 0 for normal, 1 for fire, or 2 for thunder."
            }
            "card_target_cancel_from_current" => {
                "Target-side modifier: cards from the current turn owner cannot select the skill owner. The currently supported selector is any."
            }
            "card_target_cancel_if_target_lacks_matching_suit" => {
                "Target-side modifier: a card cannot select the skill owner unless their hand contains that card's suit."
            }
            "card_target_cancel_if_other_user_hp_gte_target" => {
                "Target-side modifier: cards from another player whose current HP is at least the skill owner's current HP cannot select the owner. Requires Rule VM ABI 107."
            }
            "card_target_cancel_if_other_user_distance_gt_one" => {
                "Target-side modifier: matching cards from another player whose final distance to the skill owner is greater than one cannot select the owner. Requires Rule VM ABI 166."
            }
            "card_target_cancel_if_other_user_hand_gte_target_and_armor_empty" => {
                "Target-side modifier: matching cards from another player with at least as many hand cards cannot select an armorless skill owner."
            }
            "card_targets_self_only" => {
                "Modifier target: cards used by the owner may not target another player."
            }
            "card_use_cancel" => {
                "Modifier target: prevents the owner from proactively using any card, or only cards matching an optional selector."
            }
            "card_action_cancel_for_skill_color_mask" => {
                "Modifier target: prevents the owner from using, responding with, or saving with a physical card whose effective red/black color appears in this skill instance's color-lock state. Requires `override 1` and Rule VM ABI 48."
            }
            "card_use_unrespondable_within_responder_to_owner_distance" => {
                "Source-side modifier: cards used by the owner cannot be answered by another player whose final distance to the owner is within the configured threshold."
            }
            "card_use_unrespondable_within_owner_to_responder_distance" => {
                "Source-side modifier: cards used by the owner cannot be answered by another player within the configured final distance from the owner."
            }
            "card_response_cancel" => {
                "Modifier target: prevents the owner from responding with any card, or only cards matching an optional selector."
            }
            "hand_card_use_response_cancel" => {
                "Modifier target: prevents the owner from using or responding with physical cards in hand while leaving non-hand responses available."
            }
            "hand_card_gained_this_turn_only" => {
                "Modifier target: during the owner's Play phase, only physical hand cards gained this turn may be used or played as responses. Requires Rule VM ABI 380."
            }
            "card_save_cancel" => {
                "Modifier target: prevents the owner from using any card to save a dying player, or specifically 桃/酒 with the tao or jiu selector."
            }
            "sha_response_cancel" => {
                "Modifier target: prevents the owner from responding with 杀 when non-zero."
            }
            "sha_ignored_hand_limit" => {
                "Modifier target: excludes the owner's 杀 from hand-limit count and discard-phase choices when non-zero."
            }
            "card_ignored_hand_limit" => {
                "Card-aware modifier target: excludes matching cards from hand-limit count and discard-phase choices. Selectors include card names, sha/sha_fire/sha_red/sha_diamond, categories, suits, and colors."
            }
            "queried_player_other_than_owner" => {
                "Query-time modifier value: one when the queried living player is not the skill owner, otherwise zero."
            }
            "queried_player_wounded_in_owner_attack_range_other" => {
                "Query-time modifier value: one when the queried living player is wounded, differs from the owner, and is in the owner's attack range; otherwise zero."
            }
            "queried_source_enemy_owner_ally_target_other_than_owner" => {
                "Global distance aura value: one when the target is the owner's living ally other than the owner and the source is the target's enemy."
            }
            "owner_hp" => {
                "Query-time modifier value: the skill owner's current HP, clamped to zero."
            }
            "owner_max_hp" => "Query-time modifier value: the skill owner's maximum HP.",
            "owner_hp_minus_one" => {
                "Query-time modifier value: the skill owner's current HP minus one, clamped to zero."
            }
            "owner_lost_hp" => {
                "Query-time modifier value: the skill owner's current lost HP, clamped to zero."
            }
            "owner_lost_hp_minus_one" => {
                "Query-time modifier value: the skill owner's current lost HP minus one, clamped to zero."
            }
            "owner_lost_hp_min_one" => {
                "Query-time modifier value: the skill owner's current lost HP, with a minimum of one."
            }
            "owner_skill_mark" => {
                "Query-time modifier value: the owner's mark count whose id equals this skill definition id."
            }
            "owner_skill_state_u32" => {
                "Query-time modifier value: the first little-endian u32 in this skill instance's state blob. ABI 522."
            }
            "owner_skill_state_u32_negative" => {
                "Query-time modifier value: the negative first little-endian u32 in this skill instance's state blob. ABI 612."
            }
            "queried_player_qun_owner_lord_alive" => {
                "Query-time modifier value: one for a living Qun card user while the living modifier owner is the Lord. ABI 553."
            }
            "owner_mark_2058376721" => {
                "Parameterized query-time modifier value: the owner's mark count whose stable definition id is the numeric suffix."
            }
            "alive_group_count" => {
                "Read-only `alive_group_count()` query and query-time modifier value: the number of distinct non-empty groups among living players' characters. The expression query requires Rule VM ABI 6."
            }
            "alive_wounded_or_guiming_wu_count" => {
                "Read-only `alive_wounded_or_guiming_wu_count(player)` query: living wounded-player count, plus other living Wu players while that player is the Lord and has enabled Guiming. Requires Rule VM ABI 190."
            }
            "unique_max_current_hp" => {
                "Read-only `unique_max_current_hp(player)` query: one only when the living player has strictly more current HP than every other living player. Requires Rule VM ABI 7."
            }
            "unique_max_current_hp_at" => {
                "Read-only `unique_max_current_hp_at(player, value)` query: one only when the living player would be the unique current-HP leader at the supplied value, leaving every other player unchanged. Requires Rule VM ABI 7."
            }
            "lifetime_damage_dealt" => {
                "Read-only `lifetime_damage_dealt(player)` query: total non-HP-loss damage points dealt by that player during the current match. Requires Rule VM ABI 8."
            }
            "public_card_kind_count" => {
                "Read-only `public_card_kind_count(card_kind)` query: number of physical matching cards in the shared draw and discard piles. Requires Rule VM ABI 11."
            }
            "field_equipment_kind_count" => {
                "Read-only `field_equipment_kind_count(card_kind)` query: number of physical matching cards in living players' equipment zones. Requires Rule VM ABI 102."
            }
            "hand_equip_suit_count" => {
                "Read-only `hand_equip_suit_count(player, suit)` query: number of cards in that player's hand/equipment region with the requested effective suit. Requires Rule VM ABI 229."
            }
            "round_number_capped" => {
                "Read-only `round_number_capped(cap)` query: the current round number limited to the supplied non-negative cap. Requires Rule VM ABI 75."
            }
            "alive_female_count" => {
                "Query-time modifier value: the number of alive players whose character sex is female."
            }
            "alive_wounded_count_zero_or_at_least_four" => {
                "Query-time modifier value: one while no alive player is wounded or at least four alive players are wounded, otherwise zero."
            }
            "alive_other_count_capped_four" => {
                "Query-time modifier value: the number of other alive players, capped at four."
            }
            "alive_other_qun_count" => {
                "Query-time modifier value: the number of other alive players whose character group is qun."
            }
            "owner_has_weapon" => {
                "Query-time modifier value: one while the skill owner's equipment zone contains a weapon, otherwise zero."
            }
            "owner_no_weapon" => {
                "Query-time modifier value: one while the skill owner's equipment zone lacks a weapon, otherwise zero."
            }
            "owner_has_armor_equip" => {
                "Query-time modifier value: one while the skill owner's equipment zone contains an armor, otherwise zero."
            }
            "round_number_capped_five" => {
                "Query-time modifier value: the current round number, capped at five."
            }
            "round_at_least_alive_qun_count" => {
                "Query-time modifier value: one while the round number is at least the number of alive qun characters, otherwise zero."
            }
            "round_below_alive_qun_count" => {
                "Query-time modifier value: one while the round number is below the number of alive qun characters, otherwise zero."
            }
            "alive_field_rank_thirteen_player_count" => {
                "Query-time modifier value: number of alive players with at least one rank-K card in their equip or judge zone."
            }
            "field_mount_card_count_ceil_half" => {
                "Query-time modifier value: half the mount cards on living players' fields, rounded up."
            }
            "owner_unique_max_hand" => {
                "Query-time modifier value: one while the skill owner uniquely has the most hand cards among living players, otherwise zero."
            }
            "owner_hand_gt_other" => {
                "Contextual damage modifier value: one while the skill owner's hand count is greater than the other damage participant's, otherwise zero."
            }
            "owner_hand_lt_other" => {
                "Contextual damage modifier value: one while the skill owner's hand count is less than the other damage participant's, otherwise zero."
            }
            "owner_qun_other_shu" => {
                "Contextual damage modifier value: one while the skill owner is a living qun character and the other damage participant is a living shu character, otherwise zero. Requires Rule VM ABI 81."
            }
            "owner_turned_over_diff_other" => {
                "Contextual damage modifier value: one while the skill owner's face-up/face-down state differs from the other damage participant's, otherwise zero."
            }
            "owner_linked_diff_other" => {
                "Contextual damage modifier value: one while the skill owner and the other damage participant have different linked states, otherwise zero."
            }
            "owner_hand_equip_gte_other" => {
                "Contextual damage modifier value: one while the skill owner's hand and equipment counts are each no smaller than the other damage participant's corresponding count, otherwise zero."
            }
            "owner_sha_damage_dealt_zero_this_turn" => {
                "Turn-scoped damage modifier value: one before the skill owner has dealt any logical-Sha damage event in the current turn, otherwise zero."
            }
            "queried_player_hand_gt_hp_signed" => {
                "Global attack-range modifier value: positive one when the queried player's hand count exceeds HP, otherwise negative one."
            }
            "queried_player_judge_nonempty_other_than_owner" => {
                "Global incoming-damage aura value: one when the queried living player differs from the owner and has a judge-zone card, otherwise zero."
            }
            "queried_player_linked_other_than_owner" => {
                "Global incoming-damage aura value: one when the queried living player differs from the owner and is linked, otherwise zero."
            }
            "queried_player_linked_first_living_skill_owner" => {
                "Global aura value: one when the queried living player is linked and this modifier belongs to the first living enabled owner of its skill definition; otherwise zero. Requires Rule VM ABI 213."
            }
            "queried_player_mark_positive_2172383355" => {
                "Parameterized global-aura value: one while the queried living player has the stable mark id in the numeric suffix; otherwise zero. Requires Rule VM ABI 215."
            }
            "resolve_nzry_longnu" => {
                "Effect: toggles Longnu, pays the matching HP/max-HP cost, draws one card, and grants the matching helper. Requires Rule VM ABI 213."
            }
            "choose_link_other" => {
                "Effect: forces the owner to choose another living unlinked player and link them. Requires Rule VM ABI 213."
            }
            "resolve_drlt_poxi" => {
                "Effect: privately exposes the selected target hand, chooses exactly four distinct-suit cards across both hands, discards them, and resolves Poxi by the owner's selected-card count. Requires Rule VM ABI 214."
            }
            "resolve_drlt_jieying" => {
                "Effect: resolves Jieying's ensure-mark, optional transfer, or other-player turn-end hand collection mode. Requires Rule VM ABI 215."
            }
            "resolve_shen_simayi" => {
                "Effect: resolves Shen Simayi's discard-phase Renjie count, Lianpo extra-turn queue, or a Jilue Wansha/Zhiheng/Jizhi branch selected by mode. Requires Rule VM ABI 218."
            }
            "lord_only" => {
                "Skill declaration marker: the skill is active only while its owner has the Lord identity."
            }
            "non_gainable" => {
                "Skill declaration marker: the skill cannot be copied or gained by another skill."
            }
            "conversion" => {
                "Skill declaration marker: the skill has a generic togglable conversion branch."
            }
            "owner_no_mount" => {
                "Query-time modifier value: one while the skill owner's equipment zone contains no mount, otherwise zero."
            }
            "owner_armor_empty" => {
                "Query-time modifier value: one while the skill owner's equipment zone contains no armor, otherwise zero."
            }
            "owner_healthy" => {
                "Query-time modifier value: one while the skill owner is at full health, otherwise zero."
            }
            "owner_hand_empty" => {
                "Trigger filter or query-time modifier value: matches while the skill owner's hand is empty."
            }
            "owner_has_treasure" => {
                "Query-time modifier value: one while the skill owner's equipment zone contains a treasure, otherwise zero."
            }
            "alive_mounted_player_count_min_one" => {
                "Query-time modifier value: the number of alive players with at least one offensive or defensive mount equipped, with a minimum of one."
            }
            "owner_equip_count_odd" => {
                "Query-time modifier value: one while the skill owner's equipment-card count is odd, otherwise zero."
            }
            "owner_equip_count_even" => {
                "Query-time modifier value: one while the skill owner's equipment-card count is even (including zero), otherwise zero."
            }
            "owner_equip_count_half_floor" => {
                "Query-time modifier value: half the skill owner's equipment-card count, rounded down."
            }
            "owner_equip_count_exactly_one" => {
                "Query-time modifier value: one while the skill owner has exactly one equipment card, otherwise zero."
            }
            "owner_equip_count" => {
                "Query-time modifier value: the number of cards in the skill owner's equipment zone."
            }
            "owner_hp_gt_two" => {
                "Query-time modifier value: one while the skill owner's current HP is greater than two, otherwise zero."
            }
            "owner_hp_le_two" => {
                "Query-time modifier value: one while the skill owner's current HP is at most two, otherwise zero."
            }
            "owner_lost_hp_at_least_two" => {
                "Query-time modifier value: one while the skill owner's lost HP is at least two, otherwise zero."
            }
            "owner_all_others_in_attack_range" => {
                "Query-time modifier value: one while every other alive player is within the skill owner's attack range, otherwise zero."
            }
            "owner_current_turn_all_others_distance_at_most_one" => {
                "Query-time modifier value: one during the owner's current turn while every other living player's final distance from the owner is at most one; otherwise zero. Requires Rule VM ABI 281."
            }
            "owner_sha_used_zero" => {
                "Query-time modifier value: one before the first Sha is used in the current turn, otherwise zero."
            }
            "owner_sha_used_nonzero" => {
                "Query-time modifier value: one after the skill owner uses at least one Sha in the current turn, otherwise zero."
            }
            "owner_sha_used_exactly_one" => {
                "Query-time modifier value: one after the skill owner uses exactly one Sha in their current turn, otherwise zero."
            }
            "owner_sha_used_exactly_two" => {
                "Query-time modifier value: one after the skill owner uses exactly two Sha in their current turn, otherwise zero."
            }
            "owner_max_hand_before_skill_fires_this_turn" => {
                "Query-time modifier value: one while the skill owner ties for maximum hand size and this skill has not fired in the current turn, otherwise zero."
            }
            "owner_non_damage_hand_type_count" => {
                "Query-time modifier value: number of distinct basic, trick, and equipment categories among the owner's non-damage hand cards."
            }
            "owner_sha_hand_count" => {
                "Query-time modifier value: number of Sha cards in the skill owner's hand (任城 pxrencheng)."
            }
            "owner_turn_damage_dealt" => {
                "Query-time modifier value: total damage dealt by the skill owner during their current turn."
            }
            "owner_current_turn_card_use_count" => {
                "Query-time modifier value: cards proactively used by the skill owner during their current turn; zero outside that owner's turn. Requires Rule VM ABI 49."
            }
            "owner_play_phase_card_use_count_zero" => {
                "Query-time modifier value: one before the skill owner proactively uses a card in their current Play phase; zero otherwise. Requires Rule VM ABI 115."
            }
            "owner_skill_play_phase_fire_count_below_2" => {
                "Parameterized query-time modifier value: one during the owner's Play phase while this skill instance has fired fewer than the positive numeric suffix; zero otherwise. Requires Rule VM ABI 220."
            }
            "owner_skill_use_count" => {
                "Query-time modifier value: originating active skill invocations in the current turn. Requires Rule VM ABI 143."
            }
            "alive_wounded_or_guiming_wu_count_min_one" => {
                "Query-time modifier value: living wounded-player count, plus other living Wu players while the owner is the Lord and has enabled Guiming; minimum one. Requires Rule VM ABI 145."
            }
            "owner_has_heart_equip" => {
                "Query-time modifier value: one while the owner's equipment zone contains a card whose effective suit is heart."
            }
            "sha_use_limit" => {
                "Modifier target: adjusts proactive 杀 uses allowed during the owner's turn."
            }
            "sha_use_cancel" => {
                "Modifier target: prevents the owner from proactively using 杀 when non-zero."
            }
            "card_use_unlimited" => {
                "Modifier target: removes ordinary use limits for all cards, or for cards matching an optional name/category/suit/color selector."
            }
            "first_hand_card_suit_use_unlimited" => {
                "Modifier target: during the owner's turn, the first physical hand card of each effective suit ignores and does not consume ordinary use limits. Requires Rule VM ABI 507."
            }
            "add" => {
                "Modifier mode: adds to the base value.\n\n```\nmodifier distance_from add -1;\n```"
            }
            "override" => "Modifier mode: overrides the base value entirely.",
            "draw_cards" => {
                "Effect: draws N cards for the specified player.\n\n```\neffect draw_cards(self, 1);\n```"
            }
            "draw_each_by_seat" => {
                "Effect: both distinct living players draw the same count in seat order.\n\n```\neffect draw_each_by_seat(event_player, event_source, 1);\n```"
            }
            "draw_then_give_one" => {
                "Effect: draws first, resolves draw triggers, then forces one card from hand/equipment to the target. Use `drawn_only` to restrict the gift to cards drawn by this activation, `any_hand_equip` for any current card, or `any_hand_equip_use_equipment` to let the recipient optionally use gifted equipment after movement triggers. The last mode requires Rule VM ABI 227.\n\n```\neffect draw_then_give_one(self, event_source, 2, drawn_only);\n```"
            }
            "draw_then_take_one" => {
                "Effect: draws first, resolves draw triggers, then lets a living taker choose one of the drawer's hand/equipment cards to gain.\n\n```\neffect draw_then_take_one(self, event_source, 3);\n```"
            }
            "draw_then_choose_give_or_discard_hand" => {
                "Active-only staged effect: draws `draw_count` cards, waits for every nested draw trigger, then forces the owner to give exactly `payment_count` hand cards to another living player or discard that many hand cards. If fewer cards remain, every currently discardable hand card is discarded. Skill ownership, living players, target legality, and hand-card ownership are revalidated at every choice barrier. Requires Rule VM ABI 116.\n\n```\neffect draw_then_choose_give_or_discard_hand(self, 2, 2);\n```"
            }
            "choose_give_or_discard_hand" => {
                "Effect: if the owner has at least `payment_count` hand cards, forces a choice to give exactly that many to one currently hand-empty living player or discard that many. Recipient scope and card ownership are revalidated at every choice barrier. Requires Rule VM ABI 581.\n\n```\neffect choose_give_or_discard_hand(self, 2, hand_empty);\n```"
            }
            "choose_discard_hand_equip_then_recover_target" => {
                "Effect: lets the chooser cancel or discard exactly `count` current hand/equipment cards, then recovers the specified living wounded target by `amount` after movement triggers finish. Skill ownership, target state, and selected cards are revalidated when the choice resolves. Requires Rule VM ABI 582.\n\n```\neffect choose_discard_hand_equip_then_recover_target(self, event_player, 2, 1);\n```"
            }
            "take_one_card" => {
                "Effect: lets the recipient choose and gain one card from the source. Scope is `hand_equip`, `equip`, or `hand_equip_judge`; the same scope and gain protection are revalidated when resolving the choice. `hand_equip_judge` requires Rule VM ABI 334.\n\n```\neffect take_one_card(event_player, self, hand_equip_judge);\n```"
            }
            "take_one_event_target_hand" => {
                "Effect: lets the player choose and gain one hand card from the first counterpart carried by the triggering event. Requires Rule VM ABI 231.\n\n```\neffect take_one_event_target_hand(self);\n```"
            }
            "choose_end_extra_draw_or_play_by_used_suits" => {
                "Effect: after the current End phase, choose a complete extra Draw or Play phase; when distinct effective suits among cards used this turn reach current HP, execute both in Draw-then-Play order. Requires Rule VM ABI 232.\n\n```\neffect choose_end_extra_draw_or_play_by_used_suits(self);\n```"
            }
            "lose_hp_double_current_damage_then_replace" => {
                "Damage-before effect: lose HP equal to the bound current damage, wait for that HP-loss resolution, double the bound damage, then permanently replace the originating skill. Requires Rule VM ABI 233.\n\n```\neffect lose_hp_double_current_damage_then_replace(self, 3481582878);\n```"
            }
            "take_target_hand_then_repeat_until_max" => {
                "Active-only effect: take one hand card from the mandatory selected target, then after each complete movement-trigger chain optionally choose another player whose hand is still larger. Stops once the owner ties for the largest living hand or cancels. Requires `targets 1 1 other hand_gt_owner` and Rule VM ABI 113.\n\n```\neffect take_target_hand_then_repeat_until_max(self, event_source, self_skill);\n```"
            }
            "gain_event_cards" => {
                "Effect: gains every physical card captured by the triggering event that is still in the discard pile. Requires Rule VM ABI 82.\n\n```\neffect gain_event_cards(self);\n```"
            }
            "gain_card_if_discard" => {
                "Effect: gains one physical card only if it is still in the shared discard pile. Requires Rule VM ABI 151.\n\n```\neffect gain_card_if_discard(self, event_card);\n```"
            }
            "choose_draw_or_discard_target_card_record_sha" => {
                "Effect: lets the current Sha user cancel, draw one card, or discard one current target hand/equipment card. Accepting either branch binds this skill and target to the exact resolving Sha. Requires Rule VM ABI 152.\n\n```\neffect choose_draw_or_discard_target_card_record_sha(self, event_card, event_player);\n```"
            }
            "gain_target_equipment" => {
                "Effect: gains every card currently equipped by `target` in the requested slot. The target zone and slot membership are revalidated at effect resolution. Slots are `weapon`, `armor`, `horse_def`, `horse_off`, and `treasure`. Requires Rule VM ABI 117.\n\n```\neffect gain_target_equipment(self, event_source, weapon);\n```"
            }
            "discard_all_target_equipment" => {
                "Effect: discards every equipment card that `actor` may currently discard from `target`. Target living state, equipment-zone ownership, and discard protection are revalidated at resolution; no eligible equipment is a deterministic no-op. Requires Rule VM ABI 118.\n\n```\neffect discard_all_target_equipment(self, event_player);\n```"
            }
            "increase_current_sha_target_damage" => {
                "Effect: revalidates the suspended proactive Sha source/card/current-target tuple and increases only that target's saved damage. Other targets of the same Sha are unchanged. Available only in `card_targeted` triggers and requires Rule VM ABI 119.\n\n```\neffect increase_current_sha_target_damage(self, event_card, event_player, 1);\n```"
            }
            "turn_over" => {
                "Effect: toggles a living player's character card. A face-down player skips their next whole turn and returns face-up.\n\n```\neffect turn_over(self);\n```"
            }
            "restore_character_state" => {
                "Effect: restores a living character card to its normal state without toggling an already-normal state: remove linked state and turn face-up if needed. Requires Rule VM ABI 93.\n\n```\neffect restore_character_state(self);\n```"
            }
            "discard_all_hand_equip_judge" => {
                "Effect: forcibly discards every card in a living player's hand, equipment, and judgment zones. Requires Rule VM ABI 93.\n\n```\neffect discard_all_hand_equip_judge(self);\n```"
            }
            "damage_then_discard_all_judge" => {
                "Effect: deals one ordinary self-damage, waits for the complete damage, dying, and save chain, then discards every card currently in the player's judgment zone. Requires Rule VM ABI 224.\n\n```\neffect damage_then_discard_all_judge(self);\n```"
            }
            "replace_draw_gain_top_two_add_skills_if_different_colors" => {
                "Effect: replaces the pending normal draw, publicly reveals and gains the top two cards, waits for gain movement triggers, then grants both configured skills until turn end only when the two effective colors differ. Requires Rule VM ABI 225.\n\n```\neffect replace_draw_gain_top_two_add_skills_if_different_colors(self, first_skill_def, second_skill_def);\n```"
            }
            "choose_gain_target_card_alternate_zone_then_target_draw" => {
                "Active-only effect: choose and gain one current hand/equipment card from the selected other target, excluding the zone already used by this skill in the current Play phase, then let that target draw one card after movement triggers finish. Requires `targets 1 1 other hand_equip_nonempty` and Rule VM ABI 226.\n\n```\neffect choose_gain_target_card_alternate_zone_then_target_draw(self, event_source);\n```"
            }
            "choose_discard_target_card_resolve_current_sha" => {
                "Effect: forces the current Sha user to discard one hand/equipment card from the current target. Equipment makes only that target unable to respond; a non-equipment card makes the target gain the physical Sha if it remains in the discard pile. Available only in `card_targeted` triggers and requires Rule VM ABI 94.\n\n```\neffect choose_discard_target_card_resolve_current_sha(self, event_card, event_player);\n```"
            }
            "choose_discard_target_card_resolve_current_sha_nonbasic_bonus" => {
                "Effect: forces the current Sha user to discard one hand/equipment card from the current target. A non-basic card makes only that target unable to respond, grants the supplied modifier skill, and adds one matching mark; a basic card makes the target gain the physical Sha if it remains in the discard pile. Available only in `card_targeted` triggers and requires Rule VM ABI 312.\n\n```\neffect choose_discard_target_card_resolve_current_sha_nonbasic_bonus(self, event_card, event_player, bonus_skill_def);\n```"
            }
            "resolve_dcqiaomeng" => {
                "Effect: choose one other target of the current black card, then discard one of that target's hand/equipment cards. Gain discarded equipment; discarded trick cards make the current card use unrespondable to all players. Available only in the matching `card_targeted` trigger and requires Rule VM ABI 313.\n\n```\neffect resolve_dcqiaomeng(self, event_card, helper_skill_def);\n```"
            }
            "remove_skill_by_def" => {
                "Effect: removes every instance of one stable skill definition from the specified player.\n\n```\neffect remove_skill_by_def(self, skill_def);\n```"
            }
            "add_bound_skill" => {
                "Effect: grants the specified skill definition to a living player and binds that skill instance to one living target. Repeated grants update the same instance's bound target. Requires Rule VM ABI 130.\n\n```\neffect add_bound_skill(self, skill_def, event_source);\n```"
            }
            "replace_skill_with_bound_after_resolution_if_target_alive" => {
                "Effect: after the current effect, trigger, and dying/save resolution finishes, replaces one skill instance with a helper bound to `target` only if that target remains alive. Requires Rule VM ABI 132.\n\n```\neffect replace_skill_with_bound_after_resolution_if_target_alive(self, self_skill, helper_skill_def, event_source);\n```"
            }
            "choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp" => {
                "Effect: forces the owner to choose a nonempty hand or equipment zone, discards every discardable card in that zone, waits for movement triggers, draws the discarded count plus current lost HP, waits for draw triggers, then loses 1 HP. Requires Rule VM ABI 133.\n\n```\neffect choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp(self);\n```"
            }
            "choose_discard_hand_add_skill_by_color" => {
                "Effect: optionally discards one hand card, then grants the configured red- or black-discard skill using the card's effective color.\n\n```\neffect choose_discard_hand_add_skill_by_color(self, red_skill_def, black_skill_def);\n```"
            }
            "choose_discard_matching_equip_color_add_skill" => {
                "Effect: optionally discards one non-basic hand/equipment card whose effective color matches at least one of the player's equipped cards, then grants or updates a color-lock skill on the target. Repeated grants merge colors and replace the recorded source. Requires Rule VM ABI 48.\n\n```\neffect choose_discard_matching_equip_color_add_skill(self, event_player, skill_def);\n```"
            }
            "use_virtual_sha_from_card_color_lock_source_if_no_sha" => {
                "Effect: if the owner used no Sha in the current play phase, the source recorded in this skill instance uses a suitless virtual Sha on the owner while ignoring distance and ordinary Sha-use count. Requires Rule VM ABI 48.\n\n```\neffect use_virtual_sha_from_card_color_lock_source_if_no_sha(self);\n```"
            }
            "choose_current_card_use_enhancements" => {
                "Effect: for the suspended one-target Sha or ordinary trick, forces the user to choose one or two of: add one legal target, ignore armor, become unrespondable, or draw after each damage event. Exact-card policies are stored in the supplied temporary helper and revalidated at each choice barrier. Requires Rule VM ABI 49.\n\n```\neffect choose_current_card_use_enhancements(self, event_card, helper_skill_def);\n```"
            }
            "make_current_card_unrespondable" => {
                "Effect: records the suspended physical card in the supplied temporary helper so only that exact use skips Shan and Wuxie response windows. The helper is created or merged without affecting later cards. Requires Rule VM ABI 220.\n\n```\neffect make_current_card_unrespondable(self, event_card, helper_skill_def);\n```"
            }
            "choose_draw_or_discard_add_skill" => {
                "Effect: forces the skill owner to choose between gaining the draw helper then drawing one card, or gaining the discard helper then discarding exactly one hand/equipment card. The discard branch is omitted when it has no legal cost, and both stages are revalidated. Requires Rule VM ABI 50.\n\n```\neffect choose_draw_or_discard_add_skill(self, draw_skill_def, discard_skill_def);\n```"
            }
            "choose_three_way_draw_or_discard_add_skill" => {
                "Effect: lets the skill owner cancel or choose between drawing one card, gaining the draw helper then drawing two cards, or discarding exactly one hand/equipment card then gaining the discard helper. The discard branch is omitted when it has no legal cost, and both stages are revalidated. Requires Rule VM ABI 315.\n\n```\neffect choose_three_way_draw_or_discard_add_skill(self, draw_skill_def, discard_skill_def);\n```"
            }
            "choose_discarded_black_basic_as_bingliang" => {
                "Effect: captures the full triggering discard batch, repeatedly offers its still-discarded effective black basic cards, and uses each selected physical card as a distance-unlimited Bingliang through ordinary target, CardUsed, Wuxie, and delayed-trick settlement. Each iteration is cancellable and revalidated. Requires Rule VM ABI 51.\n\n```\neffect choose_discarded_black_basic_as_bingliang(self);\n```"
            }
            "choose_discard_judge_use_virtual_sha_draw_if_no_damage" => {
                "Effect: optionally discards one judgment card from the target, then uses a fixed suitless virtual Sha without consuming the ordinary use count. If that exact Sha is illegal or commits no damage, the owner draws one card with skill provenance. Requires Rule VM ABI 51.\n\n```\neffect choose_discard_judge_use_virtual_sha_draw_if_no_damage(self, event_player);\n```"
            }
            "discard_selected_record_unique_category_target" => {
                "Effect: atomically records the selected card's effective basic/trick/equipment category and the selected target in the originating skill's turn- or play-phase-scoped state, then discards that selected hand/equipment card. Requires the matching uniqueness clauses.\n\n```\neffect discard_selected_record_unique_category_target(self, event_source);\n```"
            }
            "choose_target_draw_or_discard_by_skill_use_count_link_if_wounded" => {
                "Effect: lets the owner choose whether the target draws or discards the originating active skill's current turn use count. The target chooses forced discard cards; after nested effects finish, the owner becomes linked if the target is wounded. Requires Rule VM ABI 52.\n\n```\neffect choose_target_draw_or_discard_by_skill_use_count_link_if_wounded(self, event_source);\n```"
            }
            "choose_target_draw_or_discard_by_skill_use_count_toggle_link_if_wounded" => {
                "Effect: lets the owner choose whether the target draws or discards the originating active skill's current turn use count. The target chooses forced discard cards; after nested effects finish, the owner's linked state toggles if the target is wounded. Requires the exact play-phase uniqueness contract and Rule VM ABI 314.\n\n```\neffect choose_target_draw_or_discard_by_skill_use_count_toggle_link_if_wounded(self, event_source);\n```"
            }
            "choose_reveal_other_hand_card_remember_category" => {
                "Effect: forces the owner to choose another living player with hand cards, choose and publicly reveal one of those cards, and remember its effective basic/trick/equipment category for the current Play phase. Requires Rule VM ABI 53.\n\n```\neffect choose_reveal_other_hand_card_remember_category(self);\n```"
            }
            "draw_then_give_exact_record_phase_target" => {
                "Effect: draws `count` cards, then forces the owner to give exactly that many current hand/equipment cards to the other current Play-phase player and records that target for delayed phase logic. Requires Rule VM ABI 53.\n\n```\neffect draw_then_give_exact_record_phase_target(self, event_player, 2);\n```"
            }
            "clear_phase_gift_target" => {
                "Effect: clears the originating skill instance's delayed Play-phase gift target. Requires Rule VM ABI 53.\n\n```\neffect clear_phase_gift_target(self);\n```"
            }
            "choose_gain_resolved_sha_or_shan_then_optional_use_sha" => {
                "Effect: gives a legal non-owner/non-original-target recipient either the resolved physical Sha or all captured physical Shan cards still in discard. A red gift lets the recipient optionally use a physical or physical view-as Sha without ordinary use count; targets must be in the original owner's attack range and cannot include that owner. Requires Rule VM ABI 54.\n\n```\neffect choose_gain_resolved_sha_or_shan_then_optional_use_sha(self, event_card);\n```"
            }
            "choose_gain_all_resolved_sha_shan_then_color_rewards" => {
                "Effect: gives one legal non-owner/non-original-target recipient every resolved physical Sha and captured physical Shan source still in discard. A red gift lets the recipient optionally use Sha without ordinary use count; a black gift makes that recipient draw one card. Follow-up Sha targets use the original owner's attack range and cannot include that owner. Requires Rule VM ABI 310.\n\n```\neffect choose_gain_all_resolved_sha_shan_then_color_rewards(self, event_card);\n```"
            }
            "skip_current_phase" => {
                "Effect: skips the current player's entered non-idle phase, records the skipped-phase audit event/count, and advances through the ordinary phase lifecycle. Prepare/Judge/End support requires Rule VM ABI 388.\n\n```\neffect skip_current_phase(self);\n```"
            }
            "finish_current_phase_abort_resolution" => {
                "Effect: discards the shared processing area, aborts the remaining current resolution, and finishes the current phase without recording it as skipped. Requires Rule VM ABI 286.\n\n```\neffect finish_current_phase_abort_resolution(self);\n```"
            }
            "gain_top_card_to_skill_pile" => {
                "Effect: moves the draw-pile top card into the originating skill's public skill pile. Requires Rule VM ABI 287.\n\n```\neffect gain_top_card_to_skill_pile(self);\n```"
            }
            "choose_discard_skill_pile_card_add_marked_skill" => {
                "Effect: chooses and discards one card from the originating skill's public pile, then grants the target a helper skill and adds one same-id mark after movement triggers. Requires Rule VM ABI 287.\n\n```\neffect choose_discard_skill_pile_card_add_marked_skill(self, event_player, helper_skill_def);\n```"
            }
            "gain_one_hand_from_selected_targets" => {
                "Effect: gains one current hand card from each active-skill target in submitted order. Requires `targets ... other max_by_owner_hp hand_nonempty` and Rule VM ABI 288.\n\n```\neffect gain_one_hand_from_selected_targets(self);\n```"
            }
            "owner_outside_turn" => {
                "View-as filter: matches any legal source card only while the skill owner is not the current turn player. Requires Rule VM ABI 289."
            }
            "owner_alive_is_source" => {
                "Trigger filter: the skill owner is the event source and remains alive after the event. Requires Rule VM ABI 291."
            }
            "choose_repolu_targets_draw" => {
                "Death-trigger effect: the owner, including a dead owner, may cancel or choose one or more living recipients. A successful choice increments the originating skill's same-id mark once, then each selected recipient draws the previous mark plus one card in seat order. Requires Rule VM ABI 291.\n\n```\neffect choose_repolu_targets_draw(self);\n```"
            }
            "resolve_xinfu_guolun" => {
                "Active effect: the owner privately selects and publicly reveals one target hand card, then may cancel or publicly reveal one own hand/equipment card. A successful choice atomically exchanges both cards into the opposite hands; the original owner of the lower physical rank draws one card, while a tie draws none. Requires Rule VM ABI 294.\n\n```\neffect resolve_xinfu_guolun(self, event_source);\n```"
            }
            "owner_alive_shan_used_by_owner_or_during_owner_turn" => {
                "Card-used trigger filter: the owner is alive, the logical card is Shan, and either the owner used it or it was used during the owner's current turn. Requires Rule VM ABI 287."
            }
            "owner_alive_other_phase_play_skill_pile_nonempty" => {
                "Play-phase trigger filter: the owner and entering player are distinct and alive, and the originating skill's public pile is nonempty. Requires Rule VM ABI 287."
            }
            "choose_other_targets_by_skipped_phase_count_damage" => {
                "Effect: forces the owner to choose one to X distinct living other players, where X is this turn's current skipped-phase count, then deals the supplied amount to each in submitted order. Requires Rule VM ABI 55.\n\n```\neffect choose_other_targets_by_skipped_phase_count_damage(self, 1);\n```"
            }
            "owner_is_target_other_hand_nonempty" => {
                "Phase trigger filter: the owner is the entering Play- or Draw-phase player and at least one other living player has a hand card. Requires Rule VM ABI 53."
            }
            "owner_is_target_positive_draw_other_hand_nonempty" => {
                "Draw-phase trigger filter: the living owner is entering an unfixed Draw phase with a positive pending draw count and at least one other living player has a hand card. Requires Rule VM ABI 311."
            }
            "owner_source_first_target_black_card_any_target_other_discardable_hand_equip" => {
                "Card-target trigger filter: the living owner used a black card, this is its first target event, and at least one other target has a discardable hand/equipment card. Requires Rule VM ABI 313."
            }
            "owner_source_play_card_matches_remembered_category" => {
                "Card-used trigger filter: during the owner's Play phase, the used card's effective basic/trick/equipment category matches the category remembered by this skill instance. Requires Rule VM ABI 53."
            }
            "owner_phase_gift_target_is_source" => {
                "Death trigger filter: the killer matches this skill instance's delayed Play-phase gift target. Requires Rule VM ABI 53."
            }
            "owner_phase_gift_target_is_target" => {
                "Play-phase-end trigger filter: the ending phase player matches this skill instance's delayed gift target. Requires Rule VM ABI 53."
            }
            "owner_source_sha_resolved_has_gainable_cards_and_recipient" => {
                "Sha-resolved trigger filter: the living owner used this Sha, at least one physical Sha/Shan group remains in discard, and at least one living recipient is neither the owner nor an original target. Requires Rule VM ABI 54."
            }
            "owner_is_target_tied_max_hand" => {
                "Phase trigger filter: the living owner is the event player and ties for the largest hand among living players. Requires Rule VM ABI 55."
            }
            "owner_is_target_tied_max_hp" => {
                "Phase trigger filter: the living owner is the event player and ties for the highest current HP among living players. Requires Rule VM ABI 55."
            }
            "owner_is_target_tied_max_equip_nonempty" => {
                "Phase trigger filter: the living owner is the event player, has at least one equipment card, and ties for the largest equipment count among living players. Requires Rule VM ABI 55."
            }
            "owner_is_target_skipped_phase_count_positive" => {
                "Phase trigger filter: the living owner is the event player and at least one phase has been skipped this turn. Requires Rule VM ABI 55."
            }
            "owner_discarded_has_hand_once_per_phase" => {
                "Cards-moved trigger filter: the living owner discarded at least one hand/equipment card, still has hand cards, and has not accepted this trigger in the current phase. Requires Rule VM ABI 56."
            }
            "owner_alive_lost_hand_this_turn" => {
                "Phase-end trigger filter: the living owner lost at least one hand/equipment card during the current turn. Requires Rule VM ABI 405."
            }
            "owner_source_first_sha_this_play_phase" => {
                "Card-used trigger filter: the living owner used the first logical Sha of their current Play phase. Requires Rule VM ABI 407."
            }
            "owner_source_damage_target_distance_at_most_one_empty_equip_slot_unused" => {
                "Damage trigger filter: the owner dealt damage to a target within distance 1, has an enabled empty equipment slot, and has not fired this turn. Requires Rule VM ABI 379."
            }
            "owner_source_first_sha_target_this_turn" => {
                "Card-targeted trigger filter: this target belongs to the owner's first logical Sha of the current turn. Requires Rule VM ABI 379."
            }
            "owner_alive_other_phase_play_end_twzhengjian_condition" => {
                "Play-phase-end filter: another living player failed the current Zhengjian branch (no non-basic use or no gain), and can pay unless Zhengjian has awakened. Requires Rule VM ABI 381."
            }
            "owner_source_sha_target_twzhenxi_available_once_per_turn" => {
                "Card-targeted filter: once per turn, the owner's Sha target has discardable hand cards or a field card that can move to another player. Requires Rule VM ABI 382."
            }
            "owner_source_first_target_damage_card_once_per_turn" => {
                "Card-targeted filter: the first declared target of the owner's Sha or damage trick, once per turn. Requires Rule VM ABI 384."
            }
            "owner_outside_turn_discarded_red_cards_in_discard" => {
                "Cards-moved filter: outside the owner's turn, at least one red card discarded by the owner remains in the shared discard pile. Requires Rule VM ABI 384."
            }
            "resolve_twtianyu" => {
                "TW Tianyu resolver: Zhenxi discards up to distance hand cards and/or moves one target field card; Yangshi adds a permanent attack-range mark unless all others are already in range, then gains a Sha. Requires Rule VM ABI 382."
            }
            "resolve_twwujing" => {
                "TW Wujing resolver: Fenghan chooses up to the triggering card's target count to draw one each; Congji chooses red event cards still in discard and gives them to one other player. Requires Rule VM ABI 384."
            }
            "reveal_uniform_hand_draw_self_choose_others" => {
                "Effect: publicly reveals the owner's current hand. If every card has one uniform effective color, the owner chooses zero to the current hand count of other living players; the chosen players and owner each draw one in canonical seat order. Requires Rule VM ABI 56.\n\n```\neffect reveal_uniform_hand_draw_self_choose_others(self);\n```"
            }
            "owner_source_single_target_sha_or_black_ordinary_trick_current_play_phase_unused" => {
                "Card-used trigger filter: during the owner's Play phase, the owner used a single-target Sha or black ordinary trick and this skill has not completed in the current phase. Requires Rule VM ABI 57."
            }
            "owner_is_target_male_source_has_discardable_equipment" => {
                "Damage-before trigger filter: the living owner is the damage target, the living source is male/double-sex, and the owner can discard an effective equipment card from hand/equipment. Requires Rule VM ABI 57."
            }
            "choose_card_user_or_add_target" => {
                "Effect: chooses another legal target of the suspended single-target Sha/ordinary trick. That player may give one hand card and become the user, or become an extra target. Requires Rule VM ABI 57.\n\n```\neffect choose_card_user_or_add_target(self, event_card);\n```"
            }
            "choose_discard_equipment_reduce_current_damage" => {
                "Effect: forces one effective equipment card to be discarded from hand/equipment, then reduces the bound pre-resolution damage by one. Requires a damage_before trigger and Rule VM ABI 57.\n\n```\neffect choose_discard_equipment_reduce_current_damage(self);\n```"
            }
            "choose_discard_equipment_prevent_current_damage" => {
                "Effect: forces one effective equipment card to be discarded from hand/equipment, then prevents the entire bound pre-resolution damage segment. Requires a damage_before trigger and Rule VM ABI 398.\n\n```\neffect choose_discard_equipment_prevent_current_damage(self);\n```"
            }
            "owner_alive_other_turn_ended_no_card_targeted_other" => {
                "Turn-ended trigger filter: the living owner is not the living ending player, and that player used no card targeting another player during the ended turn. Requires Rule VM ABI 58."
            }
            "owner_alive_other_turn_ended_card_targeted_other_no_damage_other" => {
                "Turn-ended trigger filter: the living owner is not the living ending player, and that player targeted another player with a card but dealt no damage to another player during the ended turn. Requires Rule VM ABI 146."
            }
            "owner_alive_phase_targeted_hp_changed_can_use_wanjian" => {
                "Phase-any-end filter: the living owner was named as a card target and their actual HP changed during the ending phase, and a source-free Wanjian is currently legal. Requires Rule VM ABI 98."
            }
            "owner_alive_phase_targeted_hp_unchanged_can_use_juedou" => {
                "Phase-any-end filter: the living owner was named as a card target and their actual HP did not change during the ending phase, and a source-free Juedou is currently legal. Requires Rule VM ABI 98."
            }
            "choose_move_target_equipment_or_draw" => {
                "Effect: lets the owner cancel, draw one, or move one compatible equipment card from the other player into the owner's matching slot, replacing an occupied slot normally. Requires Rule VM ABI 58.\n\n```\neffect choose_move_target_equipment_or_draw(self, event_player);\n```"
            }
            "choose_move_target_equipment_and_or_draw" => {
                "Effect: lets the owner cancel, draw one, move one compatible equipment card from the other player into the owner's matching slot, or choose both available actions in move-then-draw order. Requires Rule VM ABI 146.\n\n```\neffect choose_move_target_equipment_and_or_draw(self, event_player);\n```"
            }
            "owner_source_sha_direct_damage_before" => {
                "Damage-before trigger filter: the living owner is the source of direct logical-Sha damage and the event player is its living target. Chain-propagated damage is excluded. Requires Rule VM ABI 147."
            }
            "choose_target_discard_attack_range_cards_or_increase_damage_weapon_forces_both" => {
                "Effect: the bound direct-Sha damage target discards up to the source's current attack range from hand/equipment or increases the damage and chain base by one. A physical source weapon forces discard followed by the increase, then is discarded. Requires Rule VM ABI 147.\n\n```\neffect choose_target_discard_attack_range_cards_or_increase_damage_weapon_forces_both(self, event_player);\n```"
            }
            "choose_draw_or_recast_add_skill" => {
                "Effect: lets the owner cancel, draw one card and gain `draw_skill_def`, or choose one current hand/equipment card to recast and gain `recast_skill_def`. The branch and card are revalidated. Requires Rule VM ABI 148.\n\n```\neffect choose_draw_or_recast_add_skill(self, draw_skill_def, recast_skill_def);\n```"
            }
            "choose_gain_sha_for_opening_extra_play" => {
                "Effect: only during the turn-opening extra Play phase created by this skill, lets the owner cancel or gain the first physical Sha from the draw pile, then discard pile. The gained card alone ignores distance and accepting arms the phase-end consequence. Requires Rule VM ABI 149.\n\n```\neffect choose_gain_sha_for_opening_extra_play(self);\n```"
            }
            "finish_opening_extra_play_sha_gain" => {
                "Effect: clears this skill's armed exact-card policy and deals 1 self-damage if the owner dealt no damage after accepting the opening-phase Sha. Requires Rule VM ABI 149.\n\n```\neffect finish_opening_extra_play_sha_gain(self);\n```"
            }
            "reveal_target_hand_gain_shan_armor_or_draw_two" => {
                "Effect: publicly reveals one nonempty target hand, gives every effective Shan and physical armor to the skill owner in hand order, or draws two when none match. Requires `targets 1 1 other hand_nonempty` and Rule VM ABI 150.\n\n```\neffect reveal_target_hand_gain_shan_armor_or_draw_two(self, event_source);\n```"
            }
            "give_all_equipment_choose_recover_or_damage" => {
                "Effect: gives all current equipment to one recipient. After movement triggers, that recipient may cancel target selection to recover the owner by the captured count, or damage one to that many distinct players in current attack range. Requires Rule VM ABI 58.\n\n```\neffect give_all_equipment_choose_recover_or_damage(self, event_source);\n```"
            }
            "owner_source_sha_has_weapon" => {
                "Damage-before trigger filter: the living owner is the source of Sha damage, the event player is its living target, and the owner has a physical weapon equipped. Requires Rule VM ABI 59."
            }
            "choose_target_discard_weapon_range_hand_or_increase_damage" => {
                "Effect: the bound Sha damage target may discard exactly the source's physical weapon range in hand cards. If the target cannot or cancels, the damage and its chain base increase by one; if paid, the selected hand cards and the source's current physical weapon are discarded in that order. Requires Rule VM ABI 59.\n\n```\neffect choose_target_discard_weapon_range_hand_or_increase_damage(self, event_player);\n```"
            }
            "owner_source_other_damage_before_once_per_turn" => {
                "Damage-before trigger filter: once per current turn, the living owner is the damage source and the living event player is another player. Requires Rule VM ABI 322."
            }
            "choose_target_discard_distance_hand_or_increase_damage" => {
                "Effect: the bound damage target may discard exactly `max(1, 4-distance from source)` current hand cards. If the target has fewer cards or cancels, the current damage and chain base increase by one. Requires Rule VM ABI 322.\n\n```\neffect choose_target_discard_distance_hand_or_increase_damage(self, event_player);\n```"
            }
            "owner_alive_other_turn_ended_no_damage_other" => {
                "Turn-ended trigger filter: the living owner is not the living ending player, and that player dealt no damage to any other player during the ended turn. Requires Rule VM ABI 323."
            }
            "owner_alive_gained_two_this_turn_can_compare_other" => {
                "End-phase trigger filter: the living owner gained at least two physical cards during the current turn, has a hand card, and another living player has a hand card for comparison. Card gains are tracked only when loaded content needs this filter. Requires Rule VM ABI 325."
            }
            "choose_draw_view_two_gain_or_move_equipment" => {
                "Effect: lets the owner cancel, draw one card, privately inspect up to two deterministically sampled current hand cards from the other player and gain one, or move one compatible equipment card into the owner's matching slot. Requires Rule VM ABI 323.\n\n```\neffect choose_draw_view_two_gain_or_move_equipment(self, event_player);\n```"
            }
            "owner_source_or_target_hand_count_differs_once_per_turn" => {
                "Damage trigger filter: the living owner is the damage source or target, some living player's hand count differs from the owner's, and this skill has not fired in the current turn. Requires Rule VM ABI 60."
            }
            "owner_hand_compare_participant" => {
                "Hand-comparison trigger filter: the living owner is either the initiator or target of the current comparison."
            }
            "owner_skill_pile_nonempty_<def>" => {
                "Trigger filter: the owner's named skill pile (keyed by the skill definition id literal) holds at least one card. Matches upstream getExpansions(...).length > 0 gates."
            }
            "gain_from_skill_pile" => {
                "取用（expansion P1，ADR 0641）：从自己的技能牌堆（武将牌上）选一张牌加入手牌。参数 (player, self_skill)。"
            }
            "gain_whole_skill_pile" => {
                "Effect: moves every card from your own skill pile (the cards placed on your general card) into your hand, creating the pile if needed. No-op when the pile is empty.\n\n```\neffect gain_whole_skill_pile(self, self_skill, 1234567890);\n```"
            }
            "place_hand_cards_to_deck_top" => {
                "Effect: select cards from your hand and place them on top of the draw pile.\n\n```\neffect place_hand_cards_to_deck_top(self, self_skill, 2);\n```"
            }
            "rearrange_pile_to_deck_top" => {
                "Effect: select count cards from the specified skill pile and place them on top of the draw pile.\n\n```\neffect rearrange_pile_to_deck_top(self, self_skill, 1234567890, 3);\n```"
            }
            "resolve_qianlong" => {
                "Effect: authoritative machine for 潜龙 — after damage, view the top three deck cards and gain up to lost-HP count, the rest go to deck bottom.\n\n```\neffect resolve_qianlong(self, self_skill);\n```"
            }
            "resolve_jsrgzuozhan" => {
                "Effect: authoritative machine for 坐瞻 — at game start choose up to two other players; gain an attack-range mark equal to min(their highest HP, 5) and mark them as 坐瞻 allies.\n\n```\neffect resolve_jsrgzuozhan(self, self_skill);\n```"
            }
            "resolve_jsrgzuozhan_gain" => {
                "Effect: authoritative machine for the 坐瞻 group face — after a marked ally dies, have one surviving 坐瞻 ally gain up to X basic cards of distinct names from the discard pile.\n\n```\neffect resolve_jsrgzuozhan_gain(self, self_skill, event_player);\n```"
            }
            "resolve_twmouzhi" => {
                "Effect: authoritative machine for 谋制 — mode 0 (damage_before): if the damage card's color matches the recorded last damage color, prevent the damage; mode 1 (damage): record the event card's color (red/black, cleared when colorless).\n\n```\neffect resolve_twmouzhi(self, self_skill, mode, event_card);\n```"
            }
            "resolve_twdaigui" => {
                "Effect: authoritative machine for 待归 (slice 1) — at play-phase end with a uniform-colored hand, choose up to hand-count players and reveal that many cards from the deck bottom (per-target picks follow in a later slice).\n\n```\neffect resolve_twdaigui(self, self_skill, stage, count);\n```"
            }
            "resolve_twmitu" => {
                "Effect: authoritative machine for 迷途 (slice 1) — at your prepare phase choose up to three players; each draws two cards tagged 迷途 and shows them publicly (compare rounds follow in a later slice).\n\n```\neffect resolve_twmitu(self, self_skill, stage, count);\n```"
            }
            "resolve_qiantun" => {
                "Effect: authoritative machine for 谦吞 — once per play phase, have another player show any number of hand cards (their only compare cards), then compare: if you win, take the shown cards; otherwise take their other hand cards.\n\n```\neffect resolve_qiantun(self, self_skill);\n```"
            }
            "resolve_yangming" => {
                "Effect: authoritative machine for 养名 — once per play phase, compare with one player; if the target wins, they draw once per prior loss and you recover 1; otherwise count a loss and you may repeat.\n\n```\neffect resolve_yangming(self, self_skill);\n```"
            }
            "resolve_canyun" => {
                "Effect: authoritative machine for 残殒 — discard one card and pick 1-2 other players; compare equipment counts: if you have more they recover 1, if equal they draw 1, if they have more they lose 1 HP; you draw 1 when a target is at 1 HP.\n\n```\neffect resolve_canyun(self, self_skill);\n```"
            }
            "resolve_hengwu" => {
                "Effect: authoritative machine for 横骛 — after using or playing a card whose suit is absent from your hand, draw one card per equipment of that suit on the field.\n\n```\neffect resolve_hengwu(self, self_skill, event_card);\n```"
            }
            "resolve_biaozhao3" => {
                "Effect: authoritative machine for 表召·归 — at your prepare phase with a 表 card present: discard it, then optionally have one player draw up to the field's largest hand count (max 5) and recover 1.\n\n```\neffect resolve_biaozhao3(self, self_skill);\n```"
            }
            "resolve_chuanyun" => {
                "Effect: authoritative machine for 穿云 — after your Sha targets a player with equipment, randomly discard one of their equipped cards.\n\n```\neffect resolve_chuanyun(self, self_skill, event_player, event_card);\n```"
            }
            "resolve_biaozhao2" => {
                "Effect: authoritative machine for 表召·应 — when a discard matches the 表 card's suit and rank, give the 表 card to the discarder; you lose 1 HP.\n\n```\neffect resolve_biaozhao2(self, self_skill, event_player);\n```"
            }
            "resolve_dcniji" => {
                "Effect: authoritative machine for 逆击 — mode 0 (targeted by a non-equipment card): draw a card and tag it 逆击; mode 1 (your end phase): discard all tagged cards from your hand.\n\n```\neffect resolve_dcniji(self, self_skill, mode, event_card);\n```"
            }
            "resolve_dcmoukui" => {
                "Effect: authoritative machine for 谋溃 — after your Sha targets a player, choose one or both: draw a card, or discard one of the target's hand cards.\n\n```\neffect resolve_dcmoukui(self, self_skill, stage, target);\n```"
            }
            "resolve_zhongyan" => {
                "Effect: authoritative machine for 忠言 — once per play phase, reveal the top three deck cards; a chosen player with hand cards exchanges one of them for one revealed card.\n\n```\neffect resolve_zhongyan(self, self_skill, stage);\n```"
            }
            "resolve_chumei" => {
                "Effect: authoritative machine for 狐魅 — during your play phase, choose a player whose HP is at most the damage you dealt this turn; they draw a card, give you a card, or recover 1 HP.\n\n```\neffect resolve_chumei(self, self_skill, stage);\n```"
            }
            "owner_is_target_damage_amount_gt_one" => {
                "Trigger filter: the owner is the damage target and the incoming damage amount is greater than one (照月狮子盔 gate)."
            }
            "owner_phase_end_lost_cards" => {
                "Trigger filter: at the owner\'s own end phase, if they have lost at least one card this turn (镇抚 gate)."
            }
            "owner_alive_second_global_dying" => {
                "Trigger filter: after the second dying window of the whole match resolves (觉醒 gate; fires exactly once)."
            }
            "owner_is_target_source_hp_gt_owner_or_source_damaged_owner" => {
                "Trigger filter: you are targeted by another player\'s card, and their hp is higher than yours, or they have damaged you this match (粗遐 gate)."
            }
            "owner_is_target_owner_hp_gt_source_or_source_damaged_owner" => {
                "Trigger filter: you are targeted by another player\'s card, and your hp is higher than theirs, or they have damaged you this match (未命 gate)."
            }
            "owner_is_target_sanshou_type_tracked" => {
                "Trigger filter: the owner is the damage target (carries the used-type bitmap track for 三首)."
            }
            "owner_source_in_turn_target_unharmed_card_actions_ge_target_hp" => {
                "Trigger filter: the owner is the damage source during their own turn, the target has never taken damage this match, and the owner\'s lifetime card actions reach the target\'s hp (威临 gate)."
            }
            "owner_used_and_was_targeted_this_turn_in_range" => {
                "Trigger filter: at any player\'s end phase, if this turn you have used or played a card, been named a card target, and someone is within your attack range (直谏 gate)."
            }
            "resolve_qiangyong" => {
                "Effect: authoritative machine for 羌勇 — when your Sha deals damage, discard X cards from the wounded player (X = Sha uses this turn); if they then have no hand cards this damage is increased by one.\n\n```\neffect resolve_qiangyong(self, event_player);\n```"
            }
            "resolve_twxiongjun" => {
                "Effect: authoritative machine for 雄军 — after the owner deals damage, every living player owning 雄军 draws one card.\n\n```\neffect resolve_twxiongjun(self);\n```"
            }
            "owner_is_target_source_hp_gt_owner_per_role_turn" => {
                "Trigger filter: once per role turn, when the owner is targeted by a card and the source has more HP than the owner."
            }
            "resolve_dcqiaolve" => {
                "Effect: authoritative machine for 巧略 — discard one of your cards first, then take a random hand card from the event target (cost-added completion).\n\n```\neffect resolve_dcqiaolve(self);\n```"
            }
            "resolve_yixiang" => {
                "Effect: authoritative machine for 义襄 — randomly gain a basic card from the deck that is not already in your hand (once per role turn when targeted).\n\n```\neffect resolve_yixiang(self);\n```"
            }
            "resolve_xinfu_qiai" => {
                "Effect: authoritative machine for 求援 — once per game, while dying, every other player in seat order hands you one hand/equip card (players with none are skipped).\n\n```\neffect resolve_xinfu_qiai(self);\n```"
            }
            "resolve_olwenren" => {
                "Effect: authoritative machine for 温仁 — once per play phase, pick any number of targets; each draws 1 if they have no hand cards and 1 more if their hand count is not greater than yours.\n\n```\neffect resolve_olwenren(self);\n```"
            }
            "resolve_spolaige" => {
                "Effect: authoritative machine for 霸业 — after the second dying window of the match, exchange 西向 for 朔北, draw up to your max HP, and recover up to your max HP.\n\n```\neffect resolve_spolaige(self);\n```"
            }
            "resolve_olzhijie" => {
                "Effect: authoritative machine for 直谏 — at any player\'s end phase, choose a player within your attack range and deal 1 damage to them.\n\n```\neffect resolve_olzhijie(self);\n```"
            }
            "resolve_olsuyi" => {
                "Effect: authoritative machine for 俗义 — once per play phase, search the deck for equipment the target can use, have them equip it, then use a virtual Sha against them.\n\n```\neffect resolve_olsuyi(self, event_source);\n```"
            }
            "resolve_olqiaozhi" => {
                "Effect: authoritative machine for 巧织 — discard one card, reveal the top two deck cards, and gain one of them (tagged 巧织; while you hold one you cannot activate again).\n\n```\neffect resolve_olqiaozhi(self, stage);\n```"
            }
            "resolve_dczhee" => {
                "Effect: authoritative machine for 拓跋? — on entry/game start search the draw pile, and at every end phase search the discard pile, for a Lightning to hang as your judge card.\n\n```\neffect resolve_dczhee(self, mode);\n```"
            }
            "resolve_dchaoxian" => {
                "Effect: authoritative machine for 朝献 — once per game, shuffle every rank-3 card from the discard pile back into the deck bottom and collect all rank-3 hand cards from other players.\n\n```\neffect resolve_dchaoxian(self);\n```"
            }
            "resolve_xiangshu" => {
                "Effect: authoritative machine for 襄戍 — once per game, a wounded target recovers X hp and draws X cards (X = min(5, damage you dealt this turn)).\n\n```\neffect resolve_xiangshu(self, event_target);\n```"
            }
            "resolve_nhlundao" => {
                "Effect: authoritative machine for 论道 — after taking damage, if the source is alive with a different hand count: draw one when your hand is larger, otherwise discard one of the source\'s cards.\n\n```\neffect resolve_nhlundao(self, event_source);\n```"
            }
            "resolve_sanshou" => {
                "Effect: authoritative machine for 三首 — reveal the top three deck cards; if any revealed card is of a type never used this match, the incoming damage is cancelled.\n\n```\neffect resolve_sanshou(self);\n```"
            }
            "resolve_starcuxia_weiming" => {
                "Effect: authoritative machine for 粗遐/未命 — the card user randomly discards a hand card; the owner draws one if the dominance condition still holds.\n\n```\neffect resolve_starcuxia_weiming(self, event_source, variant);\n```"
            }
            "resolve_zhongjie" => {
                "Effect: authoritative machine for 张介 — when you die, choose another player: they gain 1 max hp, recover 1 hp, and draw a card.\n\n```\neffect resolve_zhongjie(self);\n```"
            }
            "resolve_dingce" => {
                "Effect: authoritative machine for 定策 — after taking damage, discard one of your hand cards, then the damage source discards one (you choose it); same-color bonus is deferred (card absent from the pool).\n\n```\neffect resolve_dingce(self, event_source);\n```"
            }
            "resolve_meiyong" => {
                "Effect: authoritative machine for 姝勇 — when you use or play a Sha, take one card from another player\'s hand/equip/judge area, then they draw one.\n\n```\neffect resolve_meiyong(self);\n```"
            }
            "resolve_mbzhenann" => {
                "Effect: authoritative machine for 镇南 — when targeted by a trick whose target count exceeds max(1, the user\'s hp), discard one card and deal 1 damage to any character.\n\n```\neffect resolve_mbzhenann(self);\n```"
            }
            "resolve_friendbiwei" => {
                "Effect: authoritative machine for 鄙位 — discard the highest-rank hand card, then the target discards every hand card of rank >= it; if they discarded nothing the skill\'s use is refunded.\n\n```\neffect resolve_friendbiwei(self, event_source);\n```"
            }
            "resolve_chulao" => {
                "Effect: authoritative machine for 黜老 — discard one card, then for each selected target of a distinct group the owner picks one of their cards to discard; spade discards draw.\n\n```\neffect resolve_chulao(self);\n```"
            }
            "resolve_dcanxu" => {
                "Effect: authoritative machine for 暗蓄 — the player with fewer hand cards takes one card from the other; non-spade gain draws owner one, equalized hands recover owner one.\n\n```\neffect resolve_dcanxu(self);\n```"
            }
            "resolve_duoduan" => {
                "Effect: authoritative machine for 度断 — once per turn when targeted by a Sha, recast one card, then choose: the Sha user draws two and the Sha has no effect on you, or the user discards a card and you cannot respond.\n\n```\neffect resolve_duoduan(self, event_source, event_card);\n```"
            }
            "resolve_gaoyuan" => {
                "Effect: authoritative machine for 告援 — when targeted by a Sha, discard one card to redirect the Sha to a legal target holding a 诤 mark.\n\n```\neffect resolve_gaoyuan(self, event_source, event_card);\n```"
            }
            "reset_linked_selected_targets" => {
                "Effect: resets every selected linked target, then deals 1 fire damage to each (duansuo).\n\n```\neffect reset_linked_selected_targets(self);\n```"
            }
            "resolve_mbbeiming" => {
                "Effect: authoritative machine for 孛明 — at game start choose up to two players; each gains a weapon from the deck whose attack range equals the number of suits in their hand.\n\n```\neffect resolve_mbbeiming(self, self_skill, stage);\n```"
            }
            "resolve_shuaiyan" => {
                "Effect: authoritative machine for 率言 — at your discard phase with more than one hand card, choose another player; they give you one card from their hand or equipment.\n\n```\neffect resolve_shuaiyan(self, self_skill, stage);\n```"
            }
            "resolve_zhengjian" => {
                "Effect: authoritative machine for 诤荐 — mode 0: at your end phase, grant a 诤 mark to one player; mode 1/2: a marked player's card use or play increments their mark; mode 3: at your turn begin, each marked player draws min(5, max HP, marks) and marks clear.\n\n```\neffect resolve_zhengjian(self, self_skill, stage, event_player);\n```"
            }
            "resolve_qinmian" => {
                "Effect: authoritative machine for 勤勉 — mode 0: at your prepare phase or after damage, draw that many cards tagged 勤勉; mode 1: at an end phase, optionally discard all tagged cards to damage the current turn player equal to their count (or 1 if your whole hand is tagged).\n\n```\neffect resolve_qinmian(self, self_skill, mode, amount);\n```"
            }
            "resolve_jsrgdaoren" => {
                "Effect: authoritative machine for 蹈仁 — give one hand card to another player, then deal 1 damage to every player inside both attack ranges.\n\n```\neffect resolve_jsrgdaoren(self, self_skill);\n```"
            }
            "resolve_duwang" => {
                "Effect: authoritative machine for 度望 — at game start, place the first five non-Sha cards from the deck top onto the own skill pile.\n\n```\neffect resolve_duwang(self, self_skill);\n```"
            }
            "resolve_quanjia" => {
                "Effect: authoritative machine for 拳甲 — randomly view (2+marks) cards of the damage source's hand; use a virtual Sha on them if any, else gain a mark.\n\n```\neffect resolve_quanjia(self, self_skill, event_source, event_player);\n```"
            }
            "resolve_pxqixian" => {
                "Effect: authoritative machine for 祁县 — recast one equip card as a virtual Sha or Guohe used on another player.\n\n```\neffect resolve_pxqixian(self, self_skill, 0, 0);\n```"
            }
            "resolve_dcbingji" => {
                "Effect: authoritative machine for 秉纪 — validates hand suit uniformity and per-turn suit history, then uses a virtual Sha or Tao.\n\n```\neffect resolve_dcbingji(self, self_skill, 0, 0);\n```"
            }
            "choose_hand_cards_to_skill_pile" => {
                "Effect: select between min and max cards from your hand and place them into your own skill pile.\n\n```\neffect choose_hand_cards_to_skill_pile(self, self_skill, 1, hand_count(self));\n```"
            }
            "choose_target_hand_card_to_pile" => {
                "Effect: pick another player with hand cards, then one of their hand cards, and move it into your own skill pile (the cards on your general card).\n\n```\neffect choose_target_hand_card_to_pile(self, self_skill, 1234567890);\n```"
            }
            "choose_one_from_skill_pile_discard" => {
                "Effect: lets you pick one card from your own skill pile and discard it. No-op when the pile is empty.\n\n```\neffect choose_one_from_skill_pile_discard(self, self_skill, 1234567890);\n```"
            }
            "place_top_deck_to_expansion" => {
                "Effect: places the top N cards of the draw pile into your own skill pile (the cards placed on your general card), creating the pile if needed.\n\n```\neffect place_top_deck_to_expansion(self, self_skill, 1234567890, 2);\n```"
            }
            "discard_skill_pile" => {
                "Effect: discards every card from your own skill pile (the cards placed on your general card by this skill) into the discard pile. No-op when the pile is empty.\n\n```\neffect discard_skill_pile(self, self);\n```"
            }
            "discard_basic_draw_equal" => {
                "玉蝉引擎（ABI 657）：出牌阶段限一次，弃置任意张（≥1）基本手牌并摸等量的牌。参数 (player, self_skill)。"
            }
            "start_debate_all_others" => {
                "发起议事·全员（ADR 0644 P2，ABI 655）：所有其他有手牌角色亮首张手牌分组，只发 DebateResolved 事件。参数 (player, self_skill)。"
            }
            "start_debate_targets" => {
                "发起议事·指定目标（ADR 0644 P2，ABI 656）：选 1..N 名其他角色议事并发 DebateResolved 事件。参数 (player, self_skill)。"
            }
            "resolve_jsrgchaozheng" => {
                "朝争议事（ADR 0644 P1，ABI 653）：所有其他角色亮首张手牌为意见，红黑分组多数决：红=红意见者各回复1；黑=红意见者各失去1；全同意见发起者摸参与者数张。参数 (player, self_skill)。"
            }
            "resolve_jsrgsheju" => {
                "设居双人议事（ADR 0644 P1，ABI 654）：与对方亮牌议事，黑多数则双方各减1点体力上限。参数 (player, counterpart, self_skill)。"
            }
            "choose_other_add_mark" => {
                "选他人授标记（ABI 652）：选一名其他存活角色，对其施加标记增减（护甲=3 / 横置=2 等）。参数 (player, mark_def, delta)。"
            }
            "place_hand_card_to_expansion" => {
                "置入（expansion P2，ADR 0641）：选择一张手牌置于自己的技能牌堆（武将牌上）。参数 (player, self_skill)。"
            }
            "extra_resolve_current_trick(player, card, self_skill)" => {
                "额外结算家族（clanfenshi 等，上游 effectCount++）：为当前挂起中的锦囊（WuxieWindow）追加一次效果结算。必须在 card_used 触发体内调用，card 传 event_card。"
            }
            "choose_clan_not_max_hand_limit_gain" => {
                "穆荫 (clanmuyin)：选择一名自己或与自己同族（共享任一族名）且手牌上限非全场最高的存活角色，其手牌上限永久 +1。参数 (player, self_skill)。"
            }
                        "debate_owner_is_initiator" => {
                "议事结算后触发：拥有者是发起者（ADR 0644 P2）。"
            }
            "debate_owner_participated" => {
                "议事结算后触发：拥有者亮牌参与（红或黑意见）。"
            }
            "owner_debate_participant" => {
                "Trigger filter: the owner is among the participants of a debate that just opened (ADR 0644 P3, debateShowOpinion window)."
            }
            "owner_cards_moved_contain_ying" => {
                "Trigger filter: cards moved into the discard pile and at least one is a shadow card (jsrg funi③)."
            }
            "owner_is_lord" => {
                "Trigger filter: the skill owner has the Lord identity (identity mode)."
            }
            "cards_moved_discard_at_least_three" => {
                "Card-move trigger filter: at least three cards moved into the discard pile in a single discard move. Implements 集计 (diy nsjihui)."
            }
            "owner_alive_play_end_used_categories_positive" => {
                "Play-phase-end trigger filter: the living owner's own play phase ends with at least one card used. Implements 连计 (offline zylianji)."
            }
            "owner_alive_other_targeted_by_sha" => {
                "Card-target trigger filter: a living owner observes a logical Sha targeting another player (daimou)."
            }
            "owner_alive_targeted_by_other_sha_judge_xumou" => {
                "Card-target trigger filter: a living owner is targeted by another player's Sha and their judge zone holds at least one xumou card (daimou)."
            }
            "owner_alive_event_player_lord" => {
                "Phase-end trigger filter: the living owner observes the ending turn of the Lord player (jsrgzhenglve)."
            }
            "owner_alive_event_player_lord_no_damage" => {
                "Phase-end trigger filter: the living owner observes the Lord's ending turn and the Lord dealt no damage this turn (jsrgzhenglve two-grant branch)."
            }
            "owner_alive_event_player_lord_damaged" => {
                "Phase-end trigger filter: the living owner observes the Lord's ending turn and the Lord dealt damage this turn (jsrgzhenglve single-grant branch)."
            }
            "owner_alive_turn_used_all_suits" => {
                "Turn-end trigger filter: the living owner's own turn ends with all four suits used during the turn. Implements 博图 (refresh botu)."
            }
            "owner_alive_other_source_damage" => {
                "伤害事件且来源为非拥有者的其他角色（旁观者视角）时触发。"
            }
            "debate_owner_is_initiator_result_red" => {
                "议事结算为红色且拥有者为发起者时触发。"
            }
            "debate_owner_is_initiator_result_black" => {
                "议事结算为黑色且拥有者为发起者时触发。"
            }
            "debate_owner_opinion_differs_initiator" => {
                "议事结算后触发：拥有者与发起者均参与且意见颜色不同。"
            }
"owner_is_respond_to" => {
                "矢仄 (clanshize) 过滤器：CardResponded 事件中被响应的牌属于技能拥有者（ctx.respond_to == owner）。注意响应事件中 ctx.source 是响应者。"
            }
            "owner_alive_event_player_self_or_clan_<clan>" => {
                "保族 (clanbaozu) 家族过滤器：owner 存活、事件角色存活，且事件角色是 owner 本人或 <clan>（CJK 族名，如 颍川钟氏）成员。配合 dying 事件使用；上游 hp<=0 由濒死事件本身保证，未横置门槛由 DSL body 的 mark 守卫表达。"
            }
            "owner_hand_compare_finished_participant" => {
                "The living skill owner is either participant (initiator or target) in a completed hand comparison (hand_compare_finished)."
            }
            "owner_hand_compare_participant_has_sha" => {
                "Completed hand-comparison trigger filter: the living owner participated and at least one compared physical card is an effective Sha in the discard pile."
            }
            "owner_hand_compare_participant_result_card_in_discard" => {
                "Completed hand-comparison trigger filter: the living owner participated and the exact result card selected by legacy zongshi semantics remains in the discard pile. Requires Rule VM ABI 66."
            }
            "choose_hand_count_different_target_draw_or_random_discard" => {
                "Effect: forces the owner to select one snapshotted living player with a currently different hand count. A lower-hand target draws one; a higher-hand target discards one hand card selected by authoritative deterministic RNG. Requires Rule VM ABI 60.\n\n```\neffect choose_hand_count_different_target_draw_or_random_discard(self);\n```"
            }
            "increase_current_draw_count" => {
                "Effect: adds a nonnegative amount to the current draw phase's pending draw count. The authoritative draw entry consumes the accumulated bonus exactly once, preserving one draw batch. Only valid in `phase_draw` triggers. Requires Rule VM ABI 61.\n\n```\neffect increase_current_draw_count(self, alive_group_count());\n```"
            }
            "skip_next_play_phase" => {
                "Effect: marks the current living player's upcoming Play phase to be skipped through the authoritative turn lifecycle. Valid only before Play in that player's current turn. Requires Rule VM ABI 100.\n\n```\neffect skip_next_play_phase(self);\n```"
            }
            "skip_next_judge_phase" => {
                "Effect: persistently marks a living player's next judgment phase to be skipped, then consumes the mark when that phase is skipped. Requires Rule VM ABI 228.\n\n```\neffect skip_next_judge_phase(self);\n```"
            }
            "choose_discard_damage_endpoint_card_increase_if_suit" => {
                "Damage-before effect: the source chooses and discards one currently discardable hand/equipment card from themselves or the damage target. After the discard and nested triggers resolve, the bound damage increases by `amount` when the card's owner-effective suit matches. Requires Rule VM ABI 230.\n\n```\neffect choose_discard_damage_endpoint_card_increase_if_suit(self, event_player, diamond, 1);\n```"
            }
            "choose_other_draw_or_recover" => {
                "Effect: opens `times` independently cancellable choices for `player` to select another living player. Each selected target must then choose to draw `draw_count` cards or recover `recover_count` HP; when recovery is unavailable, drawing resolves immediately. Requires Rule VM ABI 101.\n\n```\neffect choose_other_draw_or_recover(self, 2, 1, event_amount);\n```"
            }
            "choose_other_recover" => {
                "Effect: forces `player` to select another living player, then recovers that target by `amount` HP. Requires Rule VM ABI 160.\n\n```\neffect choose_other_recover(self, 2);\n```"
            }
            "enable_equip_slot" => {
                "Effect: restores one disabled copy of equipment `slot` (`1` weapon through `5` treasure) for `player`, attributed to `source`. Requires Rule VM ABI 161.\n\n```\neffect enable_equip_slot(event_source, 1, self);\n```"
            }
            "target_choose_suit_gain_owner_hand_damage_if_mismatch" => {
                "Effect: `target` chooses one suit, then one current hand card from `player` to gain. If the card's effective suit for `player` differs, `player` deals 1 normal damage to `target`. Active use requires `hand_nonempty targets 1 1 other`. Requires Rule VM ABI 168.\n\n```\neffect target_choose_suit_gain_owner_hand_damage_if_mismatch(self, event_source);\n```"
            }
            "judge_source_discard_two_hand_or_damage" => {
                "Effect: `player` judges. A heart ends the effect. Otherwise the living damage `source` may discard exactly two discardable hand cards; cancelling or having fewer than two makes `player` deal 1 normal damage to `source`. Requires Rule VM ABI 171.\n\n```\neffect judge_source_discard_two_hand_or_damage(self, event_source);\n```"
            }
            "give_selected_recover_after_phase_total_two" => {
                "Active-only effect: gives the invocation's selected hand cards from `player` to `target`. The skill tracks the number given in the current Play phase and recovers `player` by 1 after movement triggers only when the total first crosses two. Requires `targets 1 1 other select_hand 1 255` and Rule VM ABI 172.\n\n```\neffect give_selected_recover_after_phase_total_two(self, event_source);\n```"
            }
            "choose_other_hand_targets_gain_one_replace_draw" => {
                "Draw-phase effect: `player` chooses one or two other living players with hand cards, then gains one current hand card from each in selection order and replaces the normal draw. Requires Rule VM ABI 173.\n\n```\neffect choose_other_hand_targets_gain_one_replace_draw(self);\n```"
            }
            "choose_other_hand_targets_gain_one_reduce_draw" => {
                "Draw-phase effect: `player` chooses between one and the current pending draw count of other living players with hand cards, gains one current hand card from each in seat order, and reduces the pending draw by the selected-player count. Requires Rule VM ABI 311.\n\n```\neffect choose_other_hand_targets_gain_one_reduce_draw(self);\n```"
            }
            "distribute_top_two_cards" => {
                "Damage-trigger effect: move the draw-pile top two cards to public processing, then repeatedly choose a nonempty remaining subset and any living recipient until all cards are distributed. Pair with `owner_is_target_per_damage_point optional`. Requires Rule VM ABI 174.\n\n```\neffect distribute_top_two_cards(self);\n```"
            }
            "choose_discard_hand_equip_then_remove_self_skill" => {
                "Effect: forces the owner to discard `min(count, available hand/equipment cards)` cards, then removes the originating one-shot skill after movement triggers and nested choices finish. Requires Rule VM ABI 102.\n\n```\neffect choose_discard_hand_equip_then_remove_self_skill(self, field_equipment_kind_count(weapon));\n```"
            }
            "choose_discard_hand_equip_then_recover" => {
                "Effect: forces the owner to discard exactly `count` current hand/equipment cards, then recovers `amount` HP after movement triggers and nested choices finish. If the cost is no longer payable, neither discard nor recovery occurs. Requires Rule VM ABI 134.\n\n```\neffect choose_discard_hand_equip_then_recover(self, 2, 1);\n```"
            }
            "draw_selected_targets" => {
                "Active-only effect: snapshots the authoritative active-skill target list and draws `count` cards for each still-living distinct target in seat order. Requires at least one declared active target and Rule VM ABI 103.\n\n```\neffect draw_selected_targets(3);\n```"
            }
            "draw_linked_players" => {
                "Effect: draw `count` cards for every currently linked living player in seat order. Requires Rule VM ABI 344.\n\n```\neffect draw_linked_players(1);\n```"
            }
            "reveal_top_gain_if_color" => {
                "Effect: publicly reveal the draw-pile top card and gain it when its printed color is `red` or `black`; otherwise put it in the discard pile. Requires Rule VM ABI 352.\n\n```\neffect reveal_top_gain_if_color(self, red);\n```"
            }
            "recover_selected_targets" => {
                "Active-only effect: snapshots the authoritative active-skill target list and recovers `amount` HP for each still-living distinct target in seat order. Requires at least one declared active target and Rule VM ABI 109.\n\n```\neffect recover_selected_targets(1);\n```"
            }
            "damage_then_draw_selected_targets" => {
                "Active-only effect: snapshots distinct living active targets and, in seat order, deals damage from `source` then lets that target draw before continuing. Requires Rule VM ABI 279.\n\n```\neffect damage_then_draw_selected_targets(self, 1, 1);\n```"
            }
            "resolve_youdi" => {
                "End-phase effect: choose another player; that player discards one of the owner's hand/equipment cards; if it was not logical Sha, the owner gains one of that player's hand/equipment cards. Requires Rule VM ABI 280.\n\n```\neffect resolve_youdi(self);\n```"
            }
            "resolve_kuangfu" => {
                "Damage-source effect: choose one discardable equipment card from the damaged target, then discard it or move it into the owner's compatible equipment slot. Requires Rule VM ABI 282.\n\n```\neffect resolve_kuangfu(self, event_player);\n```"
            }
            "owner_source_sha_damage_target_alive_has_discardable_equip" => {
                "Damage trigger filter: the living owner caused logical Sha damage to a living target that currently has equipment the owner may discard. Requires Rule VM ABI 282."
            }
            "resolve_shanjia" => {
                "Play-phase effect: draw `min(count, 7)` cards, then discard that many current hand/equipment cards. Discarding equipment from the owner's equipment zone offers a normal-use-count, distance-unlimited virtual Sha after movement triggers finish. Requires Rule VM ABI 283.\n\n```\neffect resolve_shanjia(self, mark_count(self, self_skill));\n```"
            }
            "owner_source_equip_used_skill_mark_below_seven" => {
                "Card-used trigger filter: the living owner used a logical equipment card and this skill definition's stable player mark is below seven. Requires Rule VM ABI 283."
            }
            "resolve_new_yajiao" => {
                "Outside-turn card effect: reveal the draw-pile top card, force the owner to choose any living recipient, and give it after reveal triggers. If its logical basic/trick/equipment category differs from the triggering card, the owner then discards one hand/equipment card. Requires Rule VM ABI 284.\n\n```\neffect resolve_new_yajiao(self, event_card);\n```"
            }
            "resolve_dingpin" => {
                "Active effect: the selected target judges. Black makes that target draw their lost HP and become unavailable to this skill for the turn; red turns the owner over. Requires Rule VM ABI 285.\n\n```\neffect resolve_dingpin(self, event_player);\n```"
            }
            "owner_source_outside_turn_card_used_or_responded" => {
                "Card-used/responded filter: the living owner is the event source and used or responded with a logical card outside their own turn. Requires Rule VM ABI 284."
            }
            "record_selected_play_phase_category" => {
                "Active-only effect: record the exact selected card's effective basic/trick/equipment category in `self_skill` for the current Play phase. Requires exact one-card selection with `unused_category_by_skill_this_play_phase` and Rule VM ABI 110.\n\n```\neffect record_selected_play_phase_category(self_skill);\n```"
            }
            "recover_if_min_current_hp" => {
                "Effect: at resolution time, recovers `amount` HP for `player` only when that living player is tied for the lowest current HP. Requires Rule VM ABI 103.\n\n```\neffect recover_if_min_current_hp(self, 1);\n```"
            }
            "owner_is_target_any_alive_hand_nonempty" => {
                "Prepare-phase trigger filter: the living owner is the event player and at least one living player, including the owner, has a nonempty hand. Requires Rule VM ABI 62."
            }
            "owner_is_target_any_alive_hand_empty" => {
                "End-phase trigger filter: the living owner is the event player and at least one living player, including the owner, has no hand cards. Requires Rule VM ABI 162."
            }
            "owner_is_target_draw_reshuffle_zero_wounded" => {
                "Prepare-phase trigger filter: the living owner is the event player, is wounded, and the discard pile has never been reshuffled into the draw pile. Requires Rule VM ABI 163."
            }
            "owner_is_target_draw_reshuffle_positive" => {
                "Prepare-phase trigger filter: the living owner is the event player after at least one discard-pile reshuffle into the draw pile. Requires Rule VM ABI 163."
            }
            "choose_target_discard_excess_then_self_discard_or_target_draw" => {
                "Effect: during the owner's Prepare phase, choose any living player with discardable hand/equipment cards. Discard max(target hand minus HP, 1) cards, capped to what is available. If any discarded card is not equipment, the owner may discard that many cards; cancellation or inability makes the target draw that many instead. Requires Rule VM ABI 62.\n\n```\neffect choose_target_discard_excess_then_self_discard_or_target_draw(self);\n```"
            }
            "resolve_decadezhenjun" => {
                "Effect: during the owner's Prepare or End phase, choose any living player whose hand/equipment cards the owner may discard, including the owner. Discard max(target hand minus HP, 1), capped to currently available cards. Any equipment among the discarded cards ends the effect. If all are non-equipment, the owner may discard exactly one hand/equipment card; cancellation or inability makes the target draw the total number discarded. Uses independent payload stages so legacy Rezhenjun remains stable. Requires Rule VM ABI 298.\n\n```\neffect resolve_decadezhenjun(self);\n```"
            }
            "resolve_sp_xiaoguo" => {
                "Effect: during another living player's End phase, the owner may discard any one hand card. The phase player then may discard an effective equipment card from hand or equipment; success makes the owner draw one after movement triggers, while cancellation or inability deals one normal damage from the owner. Uses an independent state machine so Standard Xiaoguo remains stable. Requires Rule VM ABI 299.\n\n```\neffect resolve_sp_xiaoguo(self, event_player);\n```"
            }
            "choose_gain_card_from_each_other_replace_draw" => {
                "Effect: immediately fixes the pending normal Draw-phase count to zero, then snapshots every other living player in owner-relative clockwise order. For each target still alive, choose and gain exactly one currently gainable hand/equipment/judgment card, waiting for its movement triggers before continuing. Requires Rule VM ABI 300.\n\n```\neffect choose_gain_card_from_each_other_replace_draw(self);\n```"
            }
            "draw_then_choose_gain_one_from_damage_target_and_neighbors" => {
                "Effect: after damage caused by the owner, draw one card and wait for nested draw triggers. Then process the damaged player, their previous living seat, and their next living seat in that order, choosing and gaining one hand/equipment card from each. If the owner appears in the sequence, only equipment is eligible; duplicate previous/next seats are preserved. Requires Rule VM ABI 63.\n\n```\neffect draw_then_choose_gain_one_from_damage_target_and_neighbors(self, event_player);\n```"
            }
            "owner_discarded_black_basic_to_discard" => {
                "Cards-moved trigger filter: the owner discarded at least one effective black basic card from hand/equipment into the discard pile. The full moved-card batch is preserved for the effect. Requires Rule VM ABI 51."
            }
            "owner_alive_other_judge_phase_in_range_has_judge_card" => {
                "Judgment-phase trigger filter: another living player is within the living owner's attack range and has a discardable judgment card. Requires Rule VM ABI 51."
            }
            "ignore_armor_for_skill_card_policy" => {
                "Modifier target: ignores armor only while resolving a card carrying the ignore-armor bit in this skill instance's exact-card policy state. Requires `override 1` and Rule VM ABI 49."
            }
            "card_use_unrespondable_for_skill_card_policy" => {
                "Modifier target: prevents Shan and Wuxie responses only for a card carrying the unrespondable bit in this skill instance's exact-card policy state. Requires `override 1` and Rule VM ABI 49."
            }
            "card_distance_unlimited_for_skill_card_policy" => {
                "Modifier target: ignores ordinary target distance only for a card carrying the distance-unlimited bit in this skill instance's exact-card policy state. Requires `override 1` and Rule VM ABI 149."
            }
            "owner_source_single_target_sha_or_ordinary_trick_current_turn" => {
                "Card-used trigger filter: during the owner's turn, the owner used a logical Sha or ordinary trick with exactly one declared target. Requires Rule VM ABI 49."
            }
            "owner_source_event_card_policy_draw" => {
                "Damage trigger filter: the owner caused this damage with the exact card carrying the draw-on-damage bit in the current skill instance. Requires Rule VM ABI 49."
            }
            "owner_source_card_resolved_phase_policy" => {
                "Card-resolved filter: the owner finished the exact physical card armed in this skill instance's phase policy. Requires Rule VM ABI 519."
            }
            "owner_phase_play_end_potguansha_available_once_per_game" => {
                "Play-phase-end filter: the owner has hand/equipment cards and has not used 灌沙 this match. Requires Rule VM ABI 520."
            }
            "owner_discarded_sha_to_discard" => {
                "Cards-moved filter: the owner discarded at least one physical Sha that remains in the shared discard pile. Requires Rule VM ABI 522."
            }
            "owner_alive_other_turn_discarded_recorded_potfuan_suit" => {
                "End-phase filter: another player's turn ended after a suit recorded by 伏暗 entered the discard pile this turn. Requires Rule VM ABI 525."
            }
            "owner_source_sha_resolved_potfuan_dealt_damage" => {
                "Sha-resolved filter: the owner's first pending 伏暗 virtual Sha dealt damage. Requires Rule VM ABI 525."
            }
            "reveal_top_gain_non_basic_or_swap_basic" => {
                "Effect: publicly reveals the draw-pile top card. A non-basic card is gained immediately; a basic card may be exchanged by placing one hand card on top of the draw pile, otherwise it is discarded.\n\n```\neffect reveal_top_gain_non_basic_or_swap_basic(self);\n```"
            }
            "prevent_current_damage" => {
                "Effect: prevents the damage event bound to the current `damage_before` trigger.\n\n```\neffect prevent_current_damage();\n```"
            }
            "choose_prevent_damage_or_lose_max_hp_increase_damage" => {
                "Effect: forces the damage source to choose between preventing the damage bound to the current `damage_before` trigger, or losing one maximum HP and increasing that same damage by the requested amount. Requires Rule VM ABI 64.\n\n```\neffect choose_prevent_damage_or_lose_max_hp_increase_damage(self, 2);\n```"
            }
            "choose_draw_or_increase_current_card_damage_or_both_mark" => {
                "Effect: during `card_used`, lets the owner choose to draw cards, increase the suspended damaging card's base damage, or perform both and then add a mark. A non-damaging multi-target card keeps the damage branch as a no-op. Requires Rule VM ABI 114.\n\n```\neffect choose_draw_or_increase_current_card_damage_or_both_mark(self, event_card, 2, 1, mark_def, 1);\n```"
            }
            "randomize_current_hand_compare_opponent" => {
                "Effect: during `hand_compare_started`, marks the other comparison participant's card selection for authoritative deterministic RNG. The comparison context and skill ownership are revalidated before selection. Requires Rule VM ABI 65.\n\n```\neffect randomize_current_hand_compare_opponent(self);\n```"
            }
            "gain_highest_rank_sha_from_event_cards" => {
                "Effect: during `hand_compare_finished`, gains every tied highest-rank effective Sha among the captured comparison cards that remains in the discard pile. Requires Rule VM ABI 65.\n\n```\neffect gain_highest_rank_sha_from_event_cards(self);\n```"
            }
            "gain_hand_compare_result_card" => {
                "Effect: during `hand_compare_finished`, gains the opponent's comparison card on a strict win, otherwise regains the owner's comparison card, if it remains in the discard pile. Requires Rule VM ABI 66.\n\n```\neffect gain_hand_compare_result_card(self);\n```"
            }
            "choose_add_or_remove_current_card_target" => {
                "Effect: during `card_used`, consumes the originating one-shot helper after optionally adding one legal target to or removing one target from the current basic/trick card. Requires Rule VM ABI 67.\n\n```\neffect choose_add_or_remove_current_card_target(self, event_card);\n```"
            }
            "choose_hand_compare" => {
                "Effect: chooses another living player with hand cards, then lets both participants choose one hand card for a comparison. Strict victory grants one helper; tie/loss grants the other. Requires Rule VM ABI 68.\n\n```\neffect choose_hand_compare(self, win_skill_def, lose_skill_def);\n```"
            }
            "resolve_lieren" => {
                "Effect: opens the generic hand-comparison card choices with a fixed compare target (the damaged player), then grants the win/lose helper skills. ABI 531.\n\n```\neffect resolve_lieren(self, event_player, win_skill_def, lose_skill_def);\n```"
            }
            "resolve_tuntian" => {
                "Effect: judges; if the judgment card is not a heart, moves it into the originating skill's public pile (田). ABI 533.\n\n```\neffect resolve_tuntian(self);\n```"
            }
            "resolve_zaoxian" => {
                "Effect: awakens when the 屯田 pile (田) has at least three cards — the owner loses 1 max HP and gains the supplied helper skill (急袭). ABI 534.\n\n```\neffect resolve_zaoxian(self, jixi_skill_def, tuntian_skill_def);\n```"
            }
            "resolve_tiaoxin" => {
                "Effect: once per play phase, choose another player whose attack range contains the owner and who has hand/equipment cards; that player either uses a Sha on the owner or the owner discards one of their cards. ABI 537.\n\n```\neffect resolve_tiaoxin(self);\n```"
            }
            "resolve_zhiji" => {
                "Effect: awakens at the owner's prepare phase with no hand cards — draws two, recovers 1 HP, loses 1 max HP and gains the supplied 观星 skill. ABI 538.\n\n```\neffect resolve_zhiji(self, guanxing_skill_def);\n```"
            }
            "choose_other_damage" => {
                "Effect: forces the player to choose one other living player, then deals the supplied amount of normal damage from the player to that target. Requires Rule VM ABI 69.\n\n```\neffect choose_other_damage(self, 1);\n```"
            }
            "choose_bound_source_attack_range_damage" => {
                "Effect: forces the skill owner to choose one living player other than the skill-bound source and inside that source's current attack range, then deals normal damage from the bound source. The skill binding, participants, range, and submitted target are revalidated at the choice barrier. Requires Rule VM ABI 121.\n\n```\neffect choose_bound_source_attack_range_damage(self, event_source, 1);\n```"
            }
            "choose_other_discard_target_card" => {
                "Effect: forces the player to choose another living player with at least one discardable card in the requested scope, then reuses the authoritative target-card discard choice. Scopes are `equip`, `hand_equip`, and `hand_equip_judge`. Requires Rule VM ABI 70.\n\n```\neffect choose_other_discard_target_card(self, hand_equip);\n```"
            }
            "judge_prevent_damage_lose_max_hp_on_nonheart" => {
                "Effect: performs a judgment for the skill owner. On a non-heart result, prevents the damage bound to the current `damage_before` trigger and reduces the target's maximum HP by one. Requires Rule VM ABI 37.\n\n```\neffect judge_prevent_damage_lose_max_hp_on_nonheart(self, event_player);\n```"
            }
            "choose_gain_top_cards_by_suit" => {
                "Effect: privately shows up to `count` draw-pile top cards to `player`, lets that player cancel or reveal and gain any nonempty subset of cards whose effective suit matches, and leaves every unselected card on top in original relative order. Requires Rule VM ABI 38.\n\n```\neffect choose_gain_top_cards_by_suit(self, 3, heart);\n```"
            }
            "choose_other_draw_by_hand_count" => {
                "Effect: opens `times` independently cancellable choices for `player`. Each time, that player may choose a different living player; the target draws `empty_count` cards while empty-handed, otherwise `normal_count`. The target and hand state are revalidated at every barrier, and draw-trigger provenance stays bound to the originating skill. Requires Rule VM ABI 39.\n\n```\neffect choose_other_draw_by_hand_count(self, 1, 2, event_amount);\n```"
            }
            "judge_cancel_current_sha_target_on_red" => {
                "Effect: performs a judgment for the specified player and, on a red result, excludes the current target from the currently resolving Sha. Available only in `card_targeted` triggers.\n\n```\neffect judge_cancel_current_sha_target_on_red(self, event_card, event_player);\n```"
            }
            "make_current_sha_target_unrespondable" => {
                "Effect: disables the response window for exactly one target of the currently resolving physical Sha. The source/card/target tuple is revalidated against the saved continuation; no persistent modifier is installed. Available only in `card_targeted` triggers and requires Rule VM ABI 35.\n\n```\neffect make_current_sha_target_unrespondable(self, event_card, event_player);\n```"
            }
            "judge_make_current_sha_target_unrespondable_on_red" => {
                "Effect: performs a judgment for the Sha user and, on a red result, disables the response window for exactly the current target of the currently resolving physical Sha. Available only in `card_targeted` triggers and requires Rule VM ABI 177.\n\n```\neffect judge_make_current_sha_target_unrespondable_on_red(self, event_card, event_player);\n```"
            }
            "resolve_retieji" => {
                "Effect: performs 界铁骑's neutral judgment, suppresses the current Sha target's non-locked skills until turn end, then lets that target give one hand/equipment card of the judgment suit to the skill owner. Declining or having no eligible card makes only this target unable to respond and increases only this target's Sha damage by one. Requires Rule VM ABI 316.\n\n```\neffect resolve_retieji(self, event_card, event_player);\n```"
            }
            "resolve_new_reluoyi" => {
                "Effect: reveals the draw-pile top three during an unfixed Draw phase, then asks the player whether to replace the normal draw. Accepting gains every basic card, Duel, and weapon, discards the rest, and grants the configured helper; declining discards all three and keeps the normal draw. Requires Rule VM ABI 317.\n\n```\neffect resolve_new_reluoyi(self, buff_skill_def);\n```"
            }
            "resolve_remoshi" => {
                "Effect: action 0 moves the current gainable physical Sha into the damaged player's public 没矢 pile and grants the configured presentation helper; action 1 gives that event player's complete pile to the skill owner. Requires Rule VM ABI 318.\n\n```\neffect resolve_remoshi(self, event_player, event_card, helper_skill_def, action);\n```"
            }
            "resolve_xinyaoming" => {
                "Effect: lets the owner choose a legal target and one unused 邀名 branch this turn: another player draws one; discard one hand card from another player; or make any target discard one or two hand/equipment cards and draw the same count. Requires Rule VM ABI 319.\n\n```\neffect resolve_xinyaoming(self);\n```"
            }
            "resolve_rehuaiyi" => {
                "Active-only effect: reveals the owner's hand. A uniform color draws one and permits a second activation this Play phase; mixed colors force one fully discardable color to be discarded, then optionally gain one hand/equipment card from up to that many other players. Gaining at least two cards loses 1 HP. Requires `active play_phase uses_per_play_phase 2 hand_nonempty` and Rule VM ABI 320.\n\n```\neffect resolve_rehuaiyi(self);\n```"
            }
            "resolve_reanguo" => {
                "Active-only effect: runs the selected other player's minimum-hand draw, wounded minimum-HP recovery, and minimum-equipment random-use branches in order; the owner then runs only branches not actually completed. If all three complete, the owner may recast any nonempty hand/equipment subset. Requires `active play_phase uses_per_turn 1 targets 1 1 other` and Rule VM ABI 321.\n\n```\neffect resolve_reanguo(self);\n```"
            }
            "resolve_beizhu" => {
                "Active-only effect: privately reveals the selected other player's hand. With no effective Sha, discards one of that player's hand/equipment cards and may give them the first physical Sha in the draw pile. Otherwise that player sequentially uses the snapshotted Sha on the owner, ignoring distance and ordinary Sha count; each point of direct resulting damage makes the owner draw one. Requires `active play_phase uses_per_turn 1 targets 1 1 other hand_nonempty` and Rule VM ABI 324.\n\n```\neffect resolve_beizhu(self, event_source);\n```"
            }
            "resolve_mbshanjia" => {
                "Active-only effect: draws three, then discards `max(0, 3-lost_equipment_count)` current hand/equipment cards. Discarding no basic card offers an optional virtual Sha that ignores ordinary use count; discarding no trick card grants the configured all-card distance helper until Play-phase end. Requires `active play_phase uses_per_turn 1` and Rule VM ABI 326.\n\n```\neffect resolve_mbshanjia(self, mark_count(self, main_skill_def), distance_skill_def);\n```"
            }
            "resolve_daoji" => {
                "Active-only effect: lets the owner gain one currently gainable equipment card from the selected other player, forcibly uses it when still legal, then deals 1 normal damage after the use attempt when the physical card is a weapon. Requires `active play_phase uses_per_turn 1 targets 1 1 other has_gainable_equip select_he 1 1 card_nonbasic` and Rule VM ABI 327.\n\n```\neffect resolve_daoji(self, event_source);\n```"
            }
            "resolve_tw_daoji" => {
                "Active-only effect: gains one hand/equipment card from the selected in-range player. A basic card still in hand draws one; an equipment card still in hand is used when legal, then the original owner takes 1 normal damage. Requires `active play_phase uses_per_turn 1 targets 1 1 other has_gainable_hand_equip in_attack_range select_he 1 1 card_nonbasic` and Rule VM ABI 355.\n\n```\neffect resolve_tw_daoji(self, event_source);\n```"
            }
            "give_all_hand_then_choose_use_one_or_damage" => {
                "Active-only effect: gives every hand card to the selected player. They may use one gifted card; the other gifted cards return to the owner. If they decline or cannot use one, they take 1 normal damage from the owner. Requires `active play_phase uses_per_turn 1 hand_nonempty targets 1 1 other` without card selection and Rule VM ABI 359.\n\n```\neffect give_all_hand_then_choose_use_one_or_damage(self, event_source);\n```"
            }
            "choose_category_then_target_give_matching_or_sha_unrespondable" => {
                "Trigger-only current-Sha effect: the skill owner chooses basic, trick, or equipment. The current Sha target may give one matching hand/equipment card; declining or having none makes that Sha unrespondable for the target. Requires a current Sha target context and Rule VM ABI 360.\n\n```\neffect choose_category_then_target_give_matching_or_sha_unrespondable(self, event_card, event_player);\n```"
            }
            "choose_category_then_target_give_matching_or_damage_plus_one" => {
                "Damage-before effect: the damage source chooses basic, trick, or equipment. The damage target may give one matching hand/equipment card; declining or having none increases the bound damage by 1. Requires Rule VM ABI 498.\n\n```\neffect choose_category_then_target_give_matching_or_damage_plus_one(self, event_player);\n```"
            }
            "choose_other_excluding_source_random_gain_hand_equip_then_use_if_equip" => {
                "Damage-end effect: the owner chooses a living role other than self and the damage source, deterministically gains one currently gainable hand/equipment card at random, and uses it when it is equipment and remains legal. Requires Rule VM ABI 499.\n\n```\neffect choose_other_excluding_source_random_gain_hand_equip_then_use_if_equip(self, event_source);\n```"
            }
            "resolve_mobiledanshou" => {
                "End-phase effect: count cards the phase player used targeting the owner this turn. At zero the owner draws one; otherwise the owner may discard exactly that many hand/equipment cards, then deals 1 damage to the phase player. Requires Rule VM ABI 500.\n\n```\neffect resolve_mobiledanshou(self, event_player);\n```"
            }
            "resolve_jsrgchiying" => {
                "Active effect: snapshot the living roles in the selected target's attack range except the owner, then make each discard exactly one hand/equipment card in seat order when possible. If the discarded basic-card count does not exceed the target's current HP, the target gains those basic cards still in the discard pile. Requires Rule VM ABI 501.\n\n```\neffect resolve_jsrgchiying(self, event_source);\n```"
            }
            "resolve_mbqingdao" => {
                "Post-card effect for 清蹈: after a foreign damage card targeting the owner fully resolves, choose the damaged or undamaged branch and reuse the normal gain, discard, and physical-card play pipelines. Requires Rule VM ABI 503.\n\n```\neffect resolve_mbqingdao(self, event_source, event_card, 1);\n```"
            }
            "resolve_mbxiugeng" => {
                "修耕 effect: record one or two players' current hand counts, then at each recorded player's next Draw phase apply the lower/equal draw and higher/equal permanent hand-limit branches. Requires Rule VM ABI 503."
            }
            "resolve_mbchenshe" => {
                "陈赦 effect: sequentially discard from owner, dying player, and damage source when possible; three cards of one suit recover the dying player to maximum HP and remove this skill. Requires Rule VM ABI 503."
            }
            "resolve_mbquanchong" => {
                "权宠 effect: discard all owner hand/equipment cards, queue an extra turn, and arm the supplied phase-begin HP-loss helper unless the owner is the unique current-HP leader. Requires Rule VM ABI 504."
            }
            "resolve_mbrenxing" => {
                "任行 effect: consume one of two round uses, then choose to draw for the owner/current turn player or discard one card from a role that has not used or responded with Sha this turn. Requires Rule VM ABI 504."
            }
            "resolve_mbchizhang" => {
                "鸱张 effect: after a hand damage card selects its first target, optionally discard hand cards and lock matching response colors for that exact card use. Requires Rule VM ABI 505."
            }
            "resolve_mbduanyang" => {
                "断鞅 effect: store a random ownerless physical Sha lost from hand, use stored Sha at phase ends ignoring distance, and resolve its optional target recast plus owner draw. Requires Rule VM ABI 505."
            }
            "owner_source_first_target_hand_damage_card_discardable" => {
                "Card-targeted filter for 鸱张: the owner used a hand damage card, this is its first target, and the owner can discard a hand card. Requires Rule VM ABI 505."
            }
            "owner_lost_hand_ownerless_sha_not_use_once_per_turn" => {
                "Cards-moved filter for 断鞅: the owner lost an ownerless physical Sha from hand outside use/response, once per turn. Requires Rule VM ABI 505."
            }
            "owner_alive_phase_any_end_originating_skill_pile_nonempty" => {
                "Phase-any-end filter: the living owner's pile for the originating skill is nonempty. Requires Rule VM ABI 505."
            }
            "owner_source_damage_tracked_mbduanyang_target_recastable" => {
                "Damage filter for 断鞅: its tracked Sha dealt damage and the target has a hand/equipment/judgment card. Requires Rule VM ABI 505."
            }
            "resolve_mbrunwei" => {
                "润微 effect: choose one to five draw-pile cards to reveal, choose a color and recipient, arm the qualifying-loss threshold, enable the second activation, and clean tracked self-gains at Play-phase end. Requires Rule VM ABI 506."
            }
            "resolve_mbshuanghuai" => {
                "霜怀 effect: choose to prevent the bound damage or let its target gain a Tao from discard, then apply the repeated/different previous-target result. Requires Rule VM ABI 506."
            }
            "owner_mbrunwei_loss_threshold" => {
                "Cards-moved filter for an armed 润微 loss threshold, excluding pure equipment-card use. Requires Rule VM ABI 506."
            }
            "owner_phase_play_end_mbrunwei_tracked_hand" => {
                "Play-phase-end filter for tracked 润微 self-gains still in hand. Requires Rule VM ABI 506."
            }
            "owner_alive_damage_before_target_distance_le_one_once_per_turn" => {
                "Damage-before filter: the target is within final distance one of the living owner, once per turn. Requires Rule VM ABI 506."
            }
            "choose_hand_compare_grant_target_on_win" => {
                "Effect: choose one other player with hand cards for a hand comparison (both pick a card); on the chooser's strict victory the **target** is granted win_grant_def, on tie or loss nothing happens. Requires Rule VM ABI 678."
            }
            "choose_other_give_then_recover" => {
                "Effect: choose one other player with hand cards — they give one hand card to the chooser, then recover 1 HP themselves. Requires Rule VM ABI 682."
            }
            "guanxing_top_by_falu_marks" => {
                "Effect: guanxing with N = number of non-zero falu marks among the four declared defs. Requires Rule VM ABI 683."
            }
            "guanxing_top" => {
                "Effect: watch the top `count` cards of the draw pile and re-order them one by one — each round pick top-sequence or bottom-sequence for the current card. Approximation: free ordering within each sequence is not modeled. Requires Rule VM ABI 681."
            }
            "choose_hand_compare_between" => {
                "Effect: third-party hand comparison — `first` picks an opponent (excluding the skill owner), both stash one hand card, and on a non-tie the winner uses a distance-free virtual Sha against the loser (fixed to the loser). Requires Rule VM ABI 680."
            }
            "give_selected_then_recover_self" => {
                "Effect: give the active-selected hand/equip cards to the target, then the chooser recovers `amount` HP. Requires Rule VM ABI 679."
            }
            "resolve_mbxiongtu" => {
                "凶图 effect: reveal a selected target hand card, discard the missing-suit count or that revealed card, then arm the next external-damage draw and second activation. Requires Rule VM ABI 507."
            }
            "owner_source_damage_mbxiongtu_armed" => {
                "Damage filter for an armed 凶图 reward, excluding damage caused by 凶图 itself. Requires Rule VM ABI 507."
            }
            "owner_phase_play_end_mbxiongtu_state" => {
                "Play-phase-end filter for clearing 凶图's transient activation state. Requires Rule VM ABI 507."
            }
            "resolve_mbganggeng" => {
                "刚鲠 effect: give the selected hand cards to another player, then at turn end draw if that player ties for maximum hand size or discard one of their area cards otherwise. Requires Rule VM ABI 508."
            }
            "resolve_mbsijian" => {
                "死谏 effect: choose its discard, current-player draw, or combined branch; combined uses accumulate the later HP-loss cost. Requires Rule VM ABI 508."
            }
            "owner_phase_end_mbganggeng_target_alive" => {
                "Phase-end filter for resolving the living target remembered by 刚鲠. Requires Rule VM ABI 508."
            }
            "owner_from_hand_empty_or_dying_mbsijian_twice_per_turn" => {
                "Cards-moved or dying filter for 死谏, shared across both event sources and limited to twice per turn. Requires Rule VM ABI 508."
            }
            "resolve_mbjianji" => {
                "间计 effect: let two participants secretly compare using either their own hand card or the owner's shared selected card, then use a no-limit Sha from winner to loser and damage shared-Sha choosers. Requires Rule VM ABI 509."
            }
            "resolve_mbyuanmo" => {
                "远谟 effect: move one legal equipment or judgment card, compare its former owner's attack-range population before and after, then optionally draw the capped loss. Requires Rule VM ABI 509."
            }
            "owner_is_target_can_move_field_card_mbyuanmo_twice_per_turn" => {
                "Prepare or post-damage filter for 远谟 when any legal field-card move exists, shared limit twice per turn. Requires Rule VM ABI 509."
            }
            "resolve_mbfeijing" => {
                "飞径 effect: choose a seat path between a single-target Sha's endpoints, collect one secret hand discard per eligible role, then add legal targets matching the chosen color. Requires Rule VM ABI 510."
            }
            "resolve_mbxiaoge" => {
                "骁戈 effect: prevent damage to a 飞径-added target, recover and reclaim its discard, or follow the originally single-target Sha with a legal Duel. Requires Rule VM ABI 510."
            }
            "owner_source_first_single_target_sha_mbfeijing_twice_per_turn" => {
                "Card-targeted filter for the owner's first and sole Sha target, limited to twice per turn. Requires Rule VM ABI 510."
            }
            "owner_source_damage_before_mbfeijing_extra" => {
                "Damage-before filter for a target added to this exact Sha by 飞径. Requires Rule VM ABI 510."
            }
            "owner_source_sha_resolved_original_single_mbxiaoge" => {
                "Card-resolved filter for the owner's originally single-target Sha, including targets later added by 飞径. Requires Rule VM ABI 510."
            }
            "resolve_mbxuye" => {
                "蓄业 effect: draw two for the selected role, then if they newly tie for maximum hand size put one of their HEJ cards on top of the draw pile. Requires Rule VM ABI 514."
            }
            "resolve_mbkuangxiang" => {
                "狂想 effect: swap all hands, track received cards until the owner's next Play phase, and invoke 蓄业 when either tracked set is exhausted. Requires Rule VM ABI 514."
            }
            "resolve_potchengpu" => {
                "势程普 effect: 督作 gives a draw-pile fire Sha after a gain; 避憾 discards one eligible role's hand down to HP and reduces the bound Sha damage. Requires Rule VM ABI 515."
            }
            "resolve_potjugu" => {
                "势臧洪拒孤 effect: draw two, then discard min(discardable hand/equipment cards, this skill's fires this turn). Requires Rule VM ABI 516."
            }
            "resolve_potsifeng" => {
                "势曹真伺锋 effect: distribute three top cards to other players' expansion piles, consume one after each card use with a color-mismatch discard, then gain all or discard all and damage at phase end. Requires Rule VM ABI 517."
            }
            "owner_alive_other_dying_mbchenshe_available" => {
                "Dying filter: another living player is dying and at least one of the owner, dying player, or living damage source has a card the owner can discard. Requires Rule VM ABI 503."
            }
            "owner_alive_other_phase_end_mobiledanshou" => {
                "Phase-end filter: the owner is alive, the phase player is another player, and Mobile Danshou can either draw for zero targetings or pay the exact current-turn targeting count. Requires Rule VM ABI 500."
            }
            "owner_alive_other_phase_end_not_damaged_owner_this_turn" => {
                "Phase-end filter: the owner is alive, the phase player is another player, and that player dealt no damage to the owner this turn."
            }
            "owner_alive_other_phase_end_used_card_on_owner_undamaged_this_turn" => {
                "Phase-end filter: the owner is alive, the phase player is another player who used at least one card targeting the owner this turn, and the owner took no damage this turn."
            }
            "choose_lose_hp_use_virtual_sha_by_basic_variants" => {
                "Phase-end trigger effect: choose legal targets for an ignore-distance virtual Sha, then lose 1 HP and use it without ordinary use-count limits. Its base damage gains 1 when the user used at least two distinct basic-card name/nature variants this turn. Requires `trigger phase_end owner_is_target` and Rule VM ABI 361.\n\n```\neffect choose_lose_hp_use_virtual_sha_by_basic_variants(self);\n```"
            }
            "resolve_twfupan" => {
                "Damage-trigger effect: draw `event_amount`, then if possible force the owner to give one hand/equipment card to an eligible other player. The first gift to each target draws two; a later gift may deal 1 damage and permanently makes that target ineligible. Requires Rule VM ABI 362.\n\n```\neffect resolve_twfupan(self, event_amount);\n```"
            }
            "resolve_twjiaohua" => {
                "Cards-drawn trigger effect: the owner or a tied-lowest-HP recipient gains the topmost draw-pile, then discard-pile card whose basic/trick/equipment category was neither in the triggering draw nor previously gained through TW Jiaohua by that recipient this turn. Requires Rule VM ABI 363.\n\n```\neffect resolve_twjiaohua(self, event_player);\n```"
            }
            "resolve_twkujian" => {
                "TW Kujian effect: action 0 gives and tracks one to three selected hand cards; action 1 removes matching tracked event cards and draws that many for both players; action 2 instead makes both players discard once per matching card. Requires Rule VM ABI 364.\n\n```\neffect resolve_twkujian(self, event_source, 0);\n```"
            }
            "resolve_twruilian" => {
                "TW Ruilian effect: action 0 optionally selects this round's tracked role; action 1 records that role's own-turn discarded count and categories; action 2 clears tracking and optionally gains up to two current discard-pile cards of one recorded category. Requires Rule VM ABI 365.\n\n```\neffect resolve_twruilian(self, event_player, 0);\n```"
            }
            "equip_weapon_from_draw_pile" => {
                "Effect: if `player` has no physical weapon equipped, deterministically selects a random weapon from the shared draw pile and equips it. Requires Rule VM ABI 366.\n\n```\neffect equip_weapon_from_draw_pile(self);\n```"
            }
            "resolve_twjianwei" => {
                "TW Jianwei effect: stage 0 lets the owner compare with an in-range role; stage 1 lets another prepare-phase player opt to compare with the owner. The shared comparison pipeline applies the owner's weapon attack range to their rank and settles region-card or weapon gains. Requires Rule VM ABI 367.\n\n```\neffect resolve_twjianwei(self, event_player, 0);\n```"
            }
            "resolve_twtanfeng" => {
                "TW Tanfeng effect: during the owner's Prepare phase, optionally discards one hand/equipment/judgment card from another player. That player may convert one hand/equipment card into a normal-distance Sha that must include the owner; otherwise they take 1 fire damage and, if alive, choose one future Judge/Draw/Play/Discard phase for the owner to skip. Requires Rule VM ABI 368.\n\n```\neffect resolve_twtanfeng(self);\n```"
            }
            "resolve_twzhibian" => {
                "TW Zhibian effect: at the start of the owner's Play phase, optionally compares with another player. A non-win loses 1 HP; a win may move one legal card into the corresponding owner zone, recover 1 HP, or discard a non-basic hand/equipment card to perform every currently available branch. Requires Rule VM ABI 369.\n\n```\neffect resolve_twzhibian(self);\n```"
            }
            "resolve_twyuyan" => {
                "TW Yuyan effect: in the current physical Sha target context, the higher-HP source may give the owner one hand/equipment card whose rank exceeds the Sha's rank (or one non-basic card when the Sha has no rank). Declining or having no eligible card removes only the owner from that Sha. Requires Rule VM ABI 370.\n\n```\neffect resolve_twyuyan(self, event_source, event_card);\n```"
            }
            "owner_target_physical_sha_source_hp_gt_owner" => {
                "Card-targeted filter: the living owner is targeted by another living player's physical Sha and that source currently has strictly more HP. Requires Rule VM ABI 370."
            }
            "resolve_twguoyi" => {
                "TW Guoyi effect: the current Sha/ordinary-trick target chooses a turn-long hand-card use/response lock or a forced discard of lost-HP plus one cards. When both trigger conditions hold, or both branches have been chosen this turn, the same logical card is replayed on every original target only after the source use fully settles. Requires Rule VM ABI 371.\n\n```\neffect resolve_twguoyi(self, event_player, event_card, helper_skill_def);\n```"
            }
            "owner_source_other_target_sha_or_ordinary_trick_twguoyi_condition" => {
                "Card-targeted filter: the owner used a Sha or ordinary trick on another living player, that target ties for maximum HP/hand or the owner's hand is at most lost HP plus one, and the card was not replayed by the same skill. Requires Rule VM ABI 371."
            }
            "resolve_twchuhai" => {
                "TW Chuhai effect: action 0 completes the mission by disabling the judge zone and collecting one hand/equipment card from each other living player; action 1 discards exactly one card from the current batch gained from another player's personal zone. Requires Rule VM ABI 372.\n\n```\neffect resolve_twchuhai(self, action);\n```"
            }
            "owner_alive_phase_end_caused_two_distinct_dying" => {
                "Phase-end filter: the living owner caused at least two distinct other players to enter dying during the current turn. Requires Rule VM ABI 372."
            }
            "owner_gained_event_cards_from_other_personal_zone" => {
                "Cards-moved filter: the owner gained at least one event card from another player's hand, equipment, or judge zone and at least one such card remains in hand. Requires Rule VM ABI 372."
            }
            "resolve_twyizhu" => {
                "TW Yizhu effect: action 0 draws and plants up to two hand/equipment cards near the draw-pile top, action 1 optionally adds or replaces the single target of another player's use of a tracked card then draws one, and action 2 clears tracked cards entering discard. Requires Rule VM ABI 373.\n\n```\neffect resolve_twyizhu(self, source, card, action);\n```"
            }
            "owner_alive_other_single_target_used_tracked_twyizhu" => {
                "Card-targeted filter: another player used a tracked TW Yizhu physical card with exactly one declared target. Requires Rule VM ABI 373."
            }
            "owner_tracked_twyizhu_entered_discard" => {
                "Cards-moved filter: at least one physical card tracked by TW Yizhu entered the discard pile. Requires Rule VM ABI 373."
            }
            "resolve_twluanchou" => {
                "TW Luanchou/Gonghuan effect: action 0 removes the shared helper from every player and grants it to the active invocation's two selected living targets; action 1 prevents the current damage and recreates it on the helper owner with the same source, amount, nature, card, kind, and flags. Requires Rule VM ABI 374.\n\n```\neffect resolve_twluanchou(self, helper_skill_def, action);\n```"
            }
            "owner_alive_other_twgonghuan_damage_before_hp_lte_owner_unused" => {
                "Damage-before filter: another living holder of the same Gonghuan helper has HP no greater than the owner, the owner has not redirected damage this turn, and the segment was not itself redirected. Requires Rule VM ABI 374."
            }
            "resolve_twxiawei" => {
                "TW Xiawei/Qiongji effect: initializes and clears a playable Wei skill pile, prompts the one-through-four Wangxing choice and stores top cards, increases empty-pile incoming damage, or settles the exact discard/lose-max-HP cost. Requires Rule VM ABI 375.\n\n```\neffect resolve_twxiawei(self, wangxing_skill_def, action);\n```"
            }
            "owner_phase_begin_playable_skill_pile_nonempty" => {
                "Phase-begin filter: the owner has at least one nonempty skill pile exposed at hand-card use/response boundaries. Requires Rule VM ABI 375."
            }
            "owner_used_playable_skill_pile_card_unused" => {
                "Card-used/responded filter: the owner consumed a physical card from a playable skill pile and this trigger has not fired this turn. Requires Rule VM ABI 375."
            }
            "owner_target_damage_before_playable_skill_piles_empty" => {
                "Damage-before filter: the owner is the target and all playable skill piles they own are empty. Requires Rule VM ABI 375."
            }
            "resolve_twbianfuren" => {
                "TW Bianfuren effect: action 0 prevents minimum-HP damage to another player and/or schedules Wanwei's end-phase effect, action 1 gains the draw-pile top card and uses the revealed bottom card when legal, and action 2 partitions selected hand/equipment cards between draw-pile top and bottom before applying Yuejian's tiered rewards. Requires Rule VM ABI 376.\n\n```\neffect resolve_twbianfuren(self, event_player, helper_skill_def, action);\n```"
            }
            "owner_alive_damage_before_target_min_hp_unused" => {
                "Damage-before filter: a living player tied for minimum HP is the target and this skill has not fired this turn. Requires Rule VM ABI 376."
            }
            "owner_alive_global_phase_end" => {
                "Phase-end filter: the owner is alive at the beginning of any player's End phase. Requires Rule VM ABI 376."
            }
            "owner_alive_global_phase_end_lost_cards_target_other_alive" => {
                "Phase-end filter: the owner is alive, has lost at least one physical card this turn, and another living player's turn is ending. Requires Rule VM ABI 496."
            }
            "owner_alive_global_phase_end_discarded_cards_available" => {
                "Phase-end filter: the owner and phase player are alive, at least one card discarded this turn remains in the discard pile, and the owner has a fixed or selectable other target. Requires Rule VM ABI 583."
            }
            "resolve_twfeiyi" => {
                "TW Feiyi effect: action 0 gains or creates a suit-distinct Tiaojiyanmei, action 1 chooses one canonical zhinang kind then gains it when present and draws, and action 2 transfers one triggering non-use discard card to another living player. Requires Rule VM ABI 377.\n\n```\neffect resolve_twfeiyi(self, action);\n```"
            }
            "owner_alive_phase_end_used_card_no_damage" => {
                "Owner End-phase filter: at least one card was used and the owner dealt no damage this turn. Requires Rule VM ABI 377."
            }
            "owner_cards_entered_discard_not_use_unused" => {
                "Cards-moved filter: the owner's personal cards entered discard for a cause other than use/response and the skill has not fired this turn. Requires Rule VM ABI 377."
            }
            "resolve_twchenzhen" => {
                "TW Chenzhen effect: action 0 declares a basic/trick kind, pays or consumes Muyue's free-use helper, grants the named shared-pile card and restores the helper on a matching discard; actions 1-3 set up Chayi, force its next-use discard, and switch the delayed branch when hand count changes. Requires Rule VM ABI 378.\n\n```\neffect resolve_twchenzhen(self, event_player, helper_skill_def, record_skill_def, action);\n```"
            }
            "resolve_twqianju" => {
                "TW Qianju effect: equip the first compatible equipment card found in the shared draw/discard piles. Requires Rule VM ABI 379.\n\n```\neffect resolve_twqianju(self, event_player);\n```"
            }
            "resolve_twqingxi" => {
                "TW Qingxi effect: the first Sha target chooses draw-and-unrespondable or mutual equipment discard with +1 target damage. Requires Rule VM ABI 379.\n\n```\neffect resolve_twqingxi(self, event_player, event_card);\n```"
            }
            "resolve_twzaoli" => {
                "TW Zaoli effect: discard all equipment cards plus chosen non-equipment hand cards, draw the same count, refill each discarded equipment slot from shared piles, and lose HP after more than two refills. Requires Rule VM ABI 380.\n\n```\neffect resolve_twzaoli(self);\n```"
            }
            "resolve_twzhengjian" => {
                "TW Zhengjian effect: choose the initial no-nonbasic/no-gain branch, collect tribute at failed Play phases, optionally switch, and awaken Zhongchi after distinct contributors reach half the player count. Requires Rule VM ABI 381.\n\n```\neffect resolve_twzhengjian(self, event_player, helper_skill_def, action);\n```"
            }
            "suppress_non_locked_skills_until_turn_end" => {
                "Effect: suppresses the specified living player's non-locked skills until the current turn ends. Requires Rule VM ABI 356.\n\n```\neffect suppress_non_locked_skills_until_turn_end(event_source);\n```"
            }
            "choose_recover_target" => {
                "Effect: lets `chooser` cancel or recover a specified living wounded `target` by `amount`. Requires Rule VM ABI 356.\n\n```\neffect choose_recover_target(self, event_source, 1);\n```"
            }
            "resolve_jiaohua" => {
                "Active-only effect: chooses one basic/trick/equipment category not yet recorded in the persistent three-category cycle, then lets any living player gain the topmost matching draw-pile card when one exists. The category is recorded even when no card exists, and completing all three clears the cycle. Requires `active play_phase uses_per_play_phase 2` and Rule VM ABI 328.\n\n```\neffect resolve_jiaohua(self);\n```"
            }
            "resolve_luanqun" => {
                "Active-only effect: every living player with hand cards privately selects one, then all selections are revealed together. The owner may gain matching-color selections (up to four in identity-standard, otherwise two); mismatching players receive the configured first-Sha target restriction and turn-long unrespondable helpers. Requires `active play_phase uses_per_turn 1 hand_nonempty` and Rule VM ABI 329.\n\n```\neffect resolve_luanqun(self, effect_skill_def, direct_hit_skill_def);\n```"
            }
            "choose_discard_basic_then_target_discard_equip_or_damage" => {
                "Effect: lets `player` cancel or discard exactly one effective basic hand card. After its movement triggers finish, `target` may discard one effective equipment card from hand/equipment; declining or having none causes 1 normal damage from `player`. Requires Rule VM ABI 178.\n\n```\neffect choose_discard_basic_then_target_discard_equip_or_damage(self, event_player);\n```"
            }
            "choose_current_sha_target_discard_redirect_to_owner" => {
                "Effect: lets the current Sha target cancel or discard one current hand/equipment card. After discard triggers finish, that exact target slot is replaced by the living skill owner; ordinary Sha-user distance is bypassed, while card target policies and the target-to-owner attack-range condition are revalidated. Requires Rule VM ABI 180.\n\n```\neffect choose_current_sha_target_discard_redirect_to_owner(self, event_source, event_card, event_player);\n```"
            }
            "choose_target_gain_bottom_card" => {
                "Active-only effect: moves the draw-pile bottom card into the public processing zone, waits for movement triggers, then forces `player` to choose any living recipient. The transfer emits an attributed `cards_given` event. Requires Rule VM ABI 181.\n\n```\neffect choose_target_gain_bottom_card(self);\n```"
            }
            "judge_black_gain_repeat" => {
                "Effect: performs a judgment for `player`. A red result stays in discard and ends the effect; a black result is gained if still in discard, then `player` may repeat after movement triggers finish. Requires Rule VM ABI 182.\n\n```\neffect judge_black_gain_repeat(self);\n```"
            }
            "judge_black_gain_repeat_protect_hand_limit" => {
                "Effect: performs the same black-result judgment loop, but each black card actually gained is excluded from this player's ordinary hand-limit count and discard-phase choices for the current turn. Requires Rule VM ABI 309.\n\n```\neffect judge_black_gain_repeat_protect_hand_limit(self);\n```"
            }
            "judge_same_color_gain_repeat" => {
                "Effect: performs a judgment for `player`, records its effective color after rewrites, and gains it if still in discard. After movement triggers finish, `player` may repeat; later matching-color results are gained and the first different color ends the effect. Requires Rule VM ABI 307.\n\n```\neffect judge_same_color_gain_repeat(self);\n```"
            }
            "use_virtual_juedou_second_on_first" => {
                "Active-only effect: materializes a suitless virtual Duel for the second selected target to use on the first. It emits ordinary card-use and Duel events but deliberately opens no Wuxie window. Requires `targets 2 2 other male second_can_use_juedou_on_first` and Rule VM ABI 183.\n\n```\neffect use_virtual_juedou_second_on_first();\n```"
            }
            "choose_draw_or_recover" => {
                "Effect: lets the specified player choose between drawing and recovering; at full HP, draws immediately. The final argument controls cancellation.\n\n```\neffect choose_draw_or_recover(event_source, 1, 1, 0);\n```"
            }
            "choose_use_virtual_sha" => {
                "Effect: materializes a suitless virtual Sha, lets the player choose legal targets, and resolves it through the ordinary use/response/damage pipeline. The policy is `normal_distance` or `ignore_distance`.\n\n```\neffect choose_use_virtual_sha(self, ignore_distance);\n```"
            }
            "adjust_hand_to" => {
                "Effect: adjusts a player's hand to an exact count. It draws when below the target and opens a forced exact hand-card discard choice when above it. Requires Rule VM ABI 9.\n\n```\neffect adjust_hand_to(self, 4);\n```"
            }
            "gain_card_by_kind" => {
                "Effect: gains the topmost card of one rule kind, searching the draw pile before the discard pile. Does nothing when neither pile contains one. Requires Rule VM ABI 10.\n\n```\neffect gain_card_by_kind(self, tiesuo);\n```"
            }
            "gain_discard_card_by_kind" => {
                "Effect: gains the topmost card of one rule kind from the discard pile only. Does nothing when the discard pile has no matching card. Requires Rule VM ABI 74.\n\n```\neffect gain_discard_card_by_kind(self, sha);\n```"
            }
            "reveal_uniform_hand_choose_damage" => {
                "Effect: publicly reveals the owner's current hand. If every card has one uniform effective color, the owner must choose any living player, including themselves, and deals the supplied normal damage to that target. Requires Rule VM ABI 76.\n\n```\neffect reveal_uniform_hand_choose_damage(self, 1);\n```"
            }
            "choose_gain_discard_card_by_kind" => {
                "Effect: forces the player to choose and gain one matching card from the discard pile. Does nothing when the discard pile has no matching card. Requires Rule VM ABI 77.\n\n```\neffect choose_gain_discard_card_by_kind(self, sha);\n```"
            }
            "draw_for_lifetime_card_action_divisors" => {
                "Effect: reads the player's match-lifetime total of cards used or played as responses and draws one card for each supplied positive divisor that divides the total. Requires Rule VM ABI 78 and the `owner_lifetime_card_action` trigger filter.\n\n```\neffect draw_for_lifetime_card_action_divisors(self, 3, 5, 8);\n```"
            }
            "gain_draw_cards_for_lifetime_card_action_divisors" => {
                "Effect: reads the player's match-lifetime card-action total. For each matching divisor, gains the topmost draw-pile card in the paired group: `sha_or_shan`, `tao_or_jiu`, or `juedou_or_wuzhong`. The discard pile is not searched. Requires Rule VM ABI 198.\n\n```\neffect gain_draw_cards_for_lifetime_card_action_divisors(self, 3, sha_or_shan, 5, tao_or_jiu, 8, juedou_or_wuzhong);\n```"
            }
            "lose_hp_gain_discard_card_for_opening_extra_play" => {
                "Effect: only during the turn-opening extra Play phase created by this skill, loses 1 HP and then gains the topmost matching discard-pile card. The action is forced while the supplied skill definition is still owned and optional after it has been removed. Requires Rule VM ABI 200.\n\n```\neffect lose_hp_gain_discard_card_for_opening_extra_play(self, sha, mandatory_skill_def);\n```"
            }
            "choose_gain_own_lost_discard_card_this_turn" => {
                "Effect: forces the player to choose and gain one unique physical card they lost from hand/equipment this turn that still remains in discard. Requires Rule VM ABI 201 and the matching loss-history trigger filter.\n\n```\neffect choose_gain_own_lost_discard_card_this_turn(self);\n```"
            }
            "choose_move_field_card_to" => {
                "Effect: forces `player` to move one equipment or delayed-trick card from another living player's field to `target`. Equipment requires an enabled empty matching slot; delayed tricks cannot duplicate a kind already in the destination judgment area. Requires Rule VM ABI 202.\n\n```\neffect choose_move_field_card_to(self, event_source);\n```"
            }
            "draw_then_optionally_distribute_hand_cards" => {
                "Effect: draws `draw_count` cards, waits for nested draw triggers, then lets `player` plan and give up to `give_cap` distinct current hand cards to one or more other living players. Cancelling gives the cards already planned, or none when cancelled immediately. Requires Rule VM ABI 203.\n\n```\neffect draw_then_optionally_distribute_hand_cards(self, 2, 2);\n```"
            }
            "give_selected_then_continue_distinct_recover_at_two" => {
                "Active-only effect: gives the invocation's selected hand cards to its first other target, then after each complete movement/recovery chain optionally chooses another nonempty hand-card batch and a living other recipient not already used by this activation. The giver recovers once when the activation total first reaches two. Requires `uses_per_turn 1 targets 1 1 other select_hand 1 255` and Rule VM ABI 204.\n\n```\neffect give_selected_then_continue_distinct_recover_at_two(self, event_source);\n```"
            }
            "give_selected_then_recover_target" => {
                "Active-only effect: gives selected hand cards in one batch, waits for all movement events and nested triggers, then recovers the still-living recipient. Requires `targets 1 1 other select_hand 1 ...` and Rule VM ABI 301.\n\n```\neffect give_selected_then_recover_target(self, event_source, 1);\n```"
            }
            "resolve_dingpan" => {
                "Active-only effect: lets the selected equipment-bearing target draw one card, then choose either to let the owner discard one currently discardable equipment card, or move all current equipment to that target's hand in one batch and take 1 normal cardless damage after nested movement triggers. Requires `uses_per_turn living_rebel_count targets 1 1 has_equip` and Rule VM ABI 302.\n\n```\neffect resolve_dingpan(self, event_source);\n```"
            }
            "resolve_tanlu" => {
                "Phase-begin-only effect: freezes the absolute signed HP difference between the owner and current-turn player. A zero gap or a gap above the target's original hand count forces 1 normal damage; otherwise the target must choose a batch gift or that damage. After the full damage chain, a surviving target discards one currently discardable owner hand card. Requires `trigger phase_begin owner_alive_not_target optional` and Rule VM ABI 303.\n\n```\neffect resolve_tanlu(self, event_player);\n```"
            }
            "resolve_spxizhan" => {
                "Phase-begin effect: the living owner other than the turn player must discard one hand/equipment card or lose 1 HP. Discarding disables the supplied skill definition until turn end, then resolves a suit-based virtual use: spade → turn player uses Jiu; heart → owner uses Wuzhong; club → owner uses Tiesuo on the turn player; diamond → owner uses fire Sha on the turn player ignoring distance. Requires `trigger phase_begin owner_alive_not_target` and Rule VM ABI 403.\n\n```\neffect resolve_spxizhan(self, event_player, 2304282956);\n```"
            }
            "choose_max_mark_other_judge_kill_unless_tao_or_taoyuan" => {
                "Effect: lets the skill owner choose one living other player tied for the highest positive count of `mark_def`. That target judges and is killed without a killer unless the physical result is Tao or Taoyuan. The chooser may be dead so a death trigger can finish. Requires Rule VM ABI 205.\n\n```\neffect choose_max_mark_other_judge_kill_unless_tao_or_taoyuan(self, mark_def);\n```"
            }
            "choose_zone_priority_gain_random_card_from_each_other_then_turn_over" => {
                "Effect: lets the owner choose hand/equipment/judgment zone priority, then in seat order gains one authoritative deterministic-random card from each other living player's first nonempty preferred zone and finally turns over. Each gain completes before the next target is inspected. Requires Rule VM ABI 206.\n\n```\neffect choose_zone_priority_gain_random_card_from_each_other_then_turn_over(self);\n```"
            }
            "choose_add_missing_skill" => {
                "Effect: forces the player to choose and gain one of three distinct skill definitions they do not already own. If exactly one remains it is granted directly; if none remain the effect does nothing. Requires Rule VM ABI 79.\n\n```\neffect choose_add_missing_skill(self, first_skill_def, second_skill_def, third_skill_def);\n```"
            }
            "add_first_missing_skill" => {
                "Effect: permanently grants the first of three ordered skill definitions the player does not already own. If all are owned, it does nothing. Requires Rule VM ABI 575.\n\n```\neffect add_first_missing_skill(self, first_skill_def, second_skill_def, third_skill_def);\n```"
            }
            "resolve_dccunsi" => {
                "Active-only effect: grants the selected target Yongjue, lets the owner draw two cards when targeting another player, waits for those effects and nested triggers to finish, then recovers the wounded owner if Guixiu is still enabled. Public content must pass stage 0. Requires Rule VM ABI 578.\n\n```\neffect resolve_dccunsi(self, event_source, guixiu_def, yongjue_def, 0);\n```"
            }
            "record_current_sha_all_targets" => {
                "Card-use effect: binds every declared target of the exact currently resolving Sha to the originating skill, so later damage filters can verify provenance. Requires Rule VM ABI 580.\n\n```\neffect record_current_sha_all_targets(self, event_card);\n```"
            }
            "deal_damage" => {
                "Effect: deals damage from one player to another. The optional fourth argument is `normal`, `fire`, or `thunder`; omitting it preserves normal damage.\n\n```\neffect deal_damage(self, event_source, 1, fire);\n```"
            }
            "discard_one_hand_or_equip" => {
                "Effect: forces the specified player to choose and discard exactly one card from hand or equipment.\n\n```\neffect discard_one_hand_or_equip(self);\n```"
            }
            "disable_skill_by_def_until_turn_end" => {
                "Effect: disables every owned instance of the configured skill definition until the next turn begins, preserving each instance and its state. Requires Rule VM ABI 191.\n\n```\neffect disable_skill_by_def_until_turn_end(self, skill_def);\n```"
            }
            "choose_discard_one_then_gain_target_equipment" => {
                "Effect: lets `player` cancel or discard exactly one hand/equipment card. After a valid cost, every card currently equipped by `target` in the requested slot is moved to the player's hand. The cost and matching equipment are revalidated atomically. Slots are `weapon`, `armor`, `horse_def`, `horse_off`, and `treasure`. Requires Rule VM ABI 40.\n\n```\neffect choose_discard_one_then_gain_target_equipment(self, event_source, weapon);\n```"
            }
            "choose_target_draw" => {
                "Effect: forces `player` to choose one living target from `other`, `male_other`, `female_other`, `any`, `min_hand`, `same_hp`, or `hand_empty`, then that target draws `count` cards. Sex scopes include double-sex characters; `min_hand` includes every living player tied for the fewest hand cards; `same_hp` compares current HP with the chooser; `hand_empty` requires no hand cards. The latter three scopes include the chooser and are revalidated at the choice barrier. Base scopes require Rule VM ABI 41; `min_hand` requires ABI 83; `same_hp` requires ABI 135; `hand_empty` requires ABI 162.\n\n```\neffect choose_target_draw(self, 1, min_hand);\n```"
            }
            "choose_targets_draw" => {
                "Effect: forces `player` to choose exactly `target_count` distinct living targets from `other`, `male_other`, `female_other`, `any`, `min_hand`, `same_hp`, or `hand_empty`, then each target draws `count` cards in seat order. If too few legal targets exist, the effect does nothing. Base scopes require Rule VM ABI 80; `min_hand` requires ABI 83; `same_hp` requires ABI 135; `hand_empty` requires ABI 162.\n\n```\neffect choose_targets_draw(self, 2, 2, any);\n```"
            }
            "choose_optional_targets_draw" => {
                "Effect: lets `player` cancel or choose between `min_count` and `max_count` distinct living targets from a canonical scope, then each selected target draws `count` cards in seat order. `wounded` includes every living player below maximum HP, including the chooser. Requires Rule VM ABI 195.\n\n```\neffect choose_optional_targets_draw(self, 1, 1, 255, wounded);\n```"
            }
            "reveal_target_hand_sha_or_discard_bind" => {
                "Active-only effect: implements SP Cao Ren's Weikui sequence. `player` first loses 1 HP and completes the full dying window. If both participants remain alive, absence of an effective Shan forces one target hand discard; otherwise the hand is revealed only to `player`, a distance-free virtual Sha is attempted with the ordinary use-count limit, and the living target is bound to the helper afterward. Requires Rule VM ABI 196.\n\n```\neffect reveal_target_hand_sha_or_discard_bind(self, event_source, helper_skill_def);\n```"
            }
            "draw_remembered_hand_cards" => {
                "Effect: draws `count` cards for `player` and records the exact drawn cards that remain continuously in that player's hand. A recorded card is forgotten when it leaves that hand. Requires Rule VM ABI 197.\n\n```\neffect draw_remembered_hand_cards(self, 2);\n```"
            }
            "discard_remembered_hand_cards" => {
                "Effect: clears the originating skill's remembered hand-card set and discards every still-discardable recorded card that has remained continuously in `player`'s hand. Requires Rule VM ABI 197.\n\n```\neffect discard_remembered_hand_cards(self);\n```"
            }
            "choose_targets_damage" => {
                "Effect: forces `player` to choose between `min_count` and `max_count` distinct living targets from a canonical target scope, then deals `amount` normal damage from `player` to each selected target in seat order. `same_hp` and `hand_empty` include the chooser; `hand_lte_chooser` contains only other living players whose hand count does not exceed the chooser's. Every scope revalidates current state when the choice resumes. Base scopes require Rule VM ABI 135; `hand_empty` requires ABI 162; `hand_lte_chooser` requires ABI 338.\n\n```\neffect choose_targets_damage(self, 1, 1, hand_lte_chooser, 1);\n```"
            }
            "choose_target_draw_to_max_hp_cap" => {
                "Effect: forces `player` to choose any living player, including themselves. The selected target draws until their hand reaches the lesser of their maximum HP and `cap`; an already-full target remains a legal no-op choice. The target and count are revalidated at the choice barrier. Requires Rule VM ABI 84.\n\n```\neffect choose_target_draw_to_max_hp_cap(self, 5);\n```"
            }
            "choose_target_draw_then_owner_if_below_max_hp" => {
                "Effect: forces `player` to choose any living player, including themselves. The selected target draws `count` cards. After that draw and its nested triggers finish, `player` draws one card if the selected target's hand remains below their maximum HP. Requires Rule VM ABI 120.\n\n```\neffect choose_target_draw_then_owner_if_below_max_hp(self, 2);\n```"
            }
            "choose_target_draw_max_hp_cap_then_discard_excess" => {
                "Effect: lets `player` choose any living player with positive maximum HP. The selected target draws the lesser of their current maximum HP and `cap`; after that draw and nested triggers finish, the limit is evaluated again and the target must choose excess hand cards to discard down to it. The chooser may be dead so a death trigger can finish, while the target must be living at the choice barrier. Requires Rule VM ABI 122.\n\n```\neffect choose_target_draw_max_hp_cap_then_discard_excess(self, 5);\n```"
            }
            "give_selected_then_target_choose_discard_same_suit_or_lose_hp" => {
                "Effect: gives the active invocation's single selected hand card to `target`. After movement triggers finish, the recipient must choose either to reveal their hand and discard every discardable hand/equipment card with the given card's physical suit, or lose 1 HP. Requires Rule VM ABI 123.\n\n```\neffect give_selected_then_target_choose_discard_same_suit_or_lose_hp(self, event_source);\n```"
            }
            "give_selected_then_target_may_use_draw_owner" => {
                "Active-only effect: gives the invocation's selected non-equipment hand card to the selected other target. The recipient may immediately use that exact card; after a successful use, the owner draws 1. Requires `active play_phase uses_per_turn 1 targets 1 1 other select_hand 1 1 card_non_equip` and Rule VM ABI 497.\n\n```\neffect give_selected_then_target_may_use_draw_owner(self, event_source);\n```"
            }
            "gain_equipment_from_draw_pile" => {
                "Effect: moves the topmost equipment card in the shared draw pile into `player`'s hand. It does not search the discard pile. Requires Rule VM ABI 124.\n\n```\neffect gain_equipment_from_draw_pile(self);\n```"
            }
            "give_all_hand_recover_to_one" => {
                "Effect: gives every current hand card from `player` to `target`. After the movement and all nested triggers finish, recovers the living target to exactly 1 HP. Requires Rule VM ABI 125.\n\n```\neffect give_all_hand_recover_to_one(self, event_player);\n```"
            }
            "give_all_hand" => {
                "Effect: gives every current hand card from `player` to `target`. Active use requires `hand_nonempty face_up targets 1 1 other male`. Requires Rule VM ABI 151.\n\n```\neffect give_all_hand(self, event_source);\n```"
            }
            "owner_alive_other_source_sha_resolved_not_target_gainable_once_per_turn" => {
                "Sha-resolved trigger filter: once per turn, a living non-target owner observes another player's fully resolved Sha while its physical carrier remains in discard. Requires Rule VM ABI 151."
            }
            "choose_target_self_discard_then_owner_lose_hp_if_hand_ne_hp" => {
                "Effect: forces `player` to choose any living target able to discard one of their own hand/equipment cards. That target chooses and discards exactly one. After the movement and all nested triggers finish, `player` loses 1 HP if the target's hand count differs from their HP. Requires Rule VM ABI 127.\n\n```\neffect choose_target_self_discard_then_owner_lose_hp_if_hand_ne_hp(self);\n```"
            }
            "choose_give_one_hand_equip_or_recover" => {
                "Effect: lets `giver` choose whether to give one current hand/equipment card to `target`. If the giver declines or has no eligible card, the target recovers 1 HP. `owner` remains the originating skill owner even though the giver observes the choice. Requires Rule VM ABI 129.\n\n```\neffect choose_give_one_hand_equip_or_recover(self, event_source, event_player);\n```"
            }
            "resolve_gzjinfa" => {
                "Active-only effect: discards the invocation's selected hand/equipment card, then lets the selected target give one equipment card or lets the owner gain one target hand/equipment card. A spade gift makes the target use a distance-free virtual Sha on the owner. Requires `uses_per_turn 1 targets 1 1 other hand_equip_nonempty select_he 1 1` and Rule VM ABI 495.\n\n```\neffect resolve_gzjinfa(self, event_source);\n```"
            }
            "resolve_wn_qifeng" => {
                "Phase-end effect: uses a distance- and count-free virtual Sha on `target`; its damage equals the number of physical card-loss occurrences for `player` during the current turn, including repeated losses of the same card. Requires `owner_alive_global_phase_end_lost_cards_target_other_alive` and Rule VM ABI 496.\n\n```\neffect resolve_wn_qifeng(self, event_player);\n```"
            }
            "resolve_dcjue" => {
                "Phase-end effect: chooses or confirms the Jue target, then repeats up to the smaller of that target's max HP and the number of cards discarded this turn that remain in discard. Each repetition authoritatively randomizes Sha, Guohe, and Wugu and uses the first kind legal for at least one of owner/target. Requires `owner_alive_global_phase_end_discarded_cards_available` and Rule VM ABI 583.\n\n```\neffect resolve_dcjue(self, event_player);\n```"
            }
            "resolve_ruanji" => {
                "XIANDING Ruanji effect: action 0 tracks the hand revealed by Zhaowen, action 1 draws for tracked red cards used, action 2 rewrites the current Jiu effect, and action 3 resolves Jiudun's black-card branch. Requires Rule VM ABI 584.\n\n```\neffect resolve_ruanji(self, event_source, event_card, 0);\n```"
            }
            "resolve_zhengbi" => {
                "XIANDING Cuimao effect: resolves Zhengbi target/branch selection, phase gain history, zone-distinct gains, and exact gift payments. Requires Rule VM ABI 585."
            }
            "resolve_fengying" => {
                "XIANDING Cuimao effect: discards all hand cards, queues the exact extra turn, and refills to max HP when tied for minimum HP. Requires Rule VM ABI 585."
            }
            "owner_alive_phase_play_end_zhengbi_pending" => {
                "Zhengbi Play-phase-end filter: the living owner has a pending target; the resolver consumes it and checks that phase's gain history. Requires Rule VM ABI 585."
            }
            "owner_is_target_source_hand_nonempty_per_damage_point" => {
                "Xicha damage filter: owner is the damaged player and the source currently has hand cards; repeats once per damage point. Requires Rule VM ABI 586."
            }
            "owner_source_qiaojian_extra_available" => {
                "Qiaojian helper filter: owner used a Sha or ordinary trick whose category has not yet been consumed this turn. Requires Rule VM ABI 586."
            }
            "owner_xicha_watched_source_card_used" => {
                "Xicha watch filter: a player recorded in this skill's private state used their next card. Requires Rule VM ABI 586."
            }
            "owner_qiaojian_tracked_card_left_hand" => {
                "Qiaojian tag cleanup filter: a tracked exempt card left the owner's hand. Requires Rule VM ABI 586."
            }
            "resolve_qiaojian" => {
                "XIANDING Fugan effect: privately collects both discard selections, compares physical ranks, recovers or gains the surviving discarded cards, and arms the chosen next-turn policy. Requires Rule VM ABI 586."
            }
            "resolve_xicha" => {
                "XIANDING Fugan effect: privately marks up to two source hand cards, consumes the watch on that source's next card use, and cancels/gains a matching physical use. Requires Rule VM ABI 586."
            }
            "owner_alive_damage_target_self_or_in_range_other_source_numeric_card" => {
                "Zhantao damage filter: owner or a character in range took numeric card damage from another source. Requires Rule VM ABI 587."
            }
            "owner_source_wounded_exists_once_per_turn" => {
                "Anjing damage filter: owner dealt damage, a wounded living target exists, and the skill is unused this turn. Requires Rule VM ABI 587."
            }
            "resolve_huzun" => {
                "XIANDING Huzun effect: resolves Zhantao's rank judgment and virtual Sha or Anjing's growing target cap, ordered draws, and random minimum-HP recovery. Requires Rule VM ABI 587."
            }
            "resolve_zhugemengxue" => {
                "XIANDING Zhugemengxue effect: resolves Jichun's per-branch card-name-length payment and Hanying's draw-pile equipment use. Requires Rule VM ABI 588."
            }
            "resolve_bailingyun" => {
                "XIANDING Bailingyun effect: resolves Linghui's top-three use/random gain/ordered return and Xiace's two once-per-turn branches. Requires Rule VM ABI 593."
            }
            "resolve_dcshaowei" => {
                "XIANDING Wei Guanyinping effect: losing diamonds randomly gains up to three red draw-pile cards; losing hearts recovers 1 HP and restores one equipment slot, each once per turn. Requires Rule VM ABI 594."
            }
            "resolve_dcdichou" => {
                "XIANDING Wei Guanyinping effect: disables an equipment slot, discards a marked target card for name-length bonuses, and resolves selected marked targets at End phase. Requires Rule VM ABI 594."
            }
            "resolve_dcdufeng" => {
                "XIANDING Ling Cao effect: forces one or both of losing 1 HP and disabling an enabled equipment slot, then draws by disabled slots plus lost HP and grants turn-long attack-range/Sha-limit bonuses. Requires Rule VM ABI 595."
            }
            "choose_discard_hand_range" => {
                "Effect: forces `player` to choose and discard between `min` and `max` currently discardable hand cards. The choice cannot be cancelled and card ownership is revalidated on resume. Requires Rule VM ABI 596.\n\n```\neffect choose_discard_hand_range(self, 1, 2147483647);\n```"
            }
            "resolve_dcctjiuxian" => {
                "XIANDING Chen Tai effect: recasts exactly half the owner's hand rounded up, then uses a tracked virtual Duel through the ordinary card pipeline; exact original-target damage may optionally recover one wounded character in that target's attack range. Requires Rule VM ABI 597."
            }
            "resolve_bianyue" => {
                "XIANDING Bian Yue effect: Bizu draws every character tied with the owner's hand count and disables itself plus recovers on a repeated exact group; Wuxie swaps whole hands with a chosen other character, then moves every damage card in the owner's new hand to the draw-pile bottom. Requires Rule VM ABI 598."
            }
            "resolve_zhupeilan" => {
                "XIANDING Zhu Peilan effect: Cilv draws by remaining choices, permanently removes one selected option, binds nullification/damage prevention/material gain to the exact trick use, and Tongdao resets a chosen character to fresh starting skills before recovery. Requires Rule VM ABI 600."
            }
            "resolve_dc_sb_zhouyu" => {
                "XIANDING DC SB Zhou Yu effect: Ronghuo arms and consumes the next fire-card boost; Yingmou toggles conversion state and resolves the draw/Fire Attack or physical damage-card sequence. Requires Rule VM ABI 602."
            }
            "resolve_dc_sb_lusu" => {
                "XIANDING DC SB Lu Su effect: Mingshi draws two, reveals three selected hand cards, and lets another character gain one; Mengmou toggles its conversion state and resolves its bounded Sha-use or Sha-response sequence. Requires Rule VM ABI 603."
            }
            "owner_hand_transfer_other_dcsbmengmou_branch_available" => {
                "Mengmou transfer filter: the owner and another living character exchanged at least one hand card, and the current conversion branch remains unused this turn. Requires Rule VM ABI 603."
            }
            "dcsbmengmou_tracked_sha_damage" => {
                "Mengmou Yang filter: exact damage from the currently tracked prompted Sha, repeated once per damage point. Requires Rule VM ABI 603."
            }
            "dcsbmengmou_tracked_sha_resolved" => {
                "Mengmou Yang filter: the currently tracked prompted Sha finished resolving, allowing the bounded sequence to continue. Requires Rule VM ABI 603."
            }
            "resolve_zhangjian" => {
                "XIANDING Zhangjian effect: rewrites card damage to its physical rank or prevents rankless logical-card damage, and discards all discardable hand/equipment cards before granting a source-turn-bound skill. Requires Rule VM ABI 604."
            }
            "resolve_zhugeruoxue" => {
                "XIANDING Zhugeruoxue effect: Qiongying moves one legal field card then discards a matching-suit hand card or reveals the hand; Nuanhui repeatedly uses legal virtual basics up to the target's snapshotted equipment count and discards equipment after a repeated name. Requires Rule VM ABI 605."
            }
            "owner_is_target_card_damage_before" => {
                "Card-damage-before filter: the living owner is the pending damage target and the damage carries a physical or logical card context. Requires Rule VM ABI 604."
            }
            "choose_give_two_different_colors_or_recover" => {
                "Effect: lets `giver` give exactly two current hand/equipment cards with different effective colors to `target`. If the giver declines or has no valid pair, the target recovers 1 HP. `owner` remains the originating skill owner. Requires Rule VM ABI 330.\n\n```\neffect choose_give_two_different_colors_or_recover(self, event_source, event_player);\n```"
            }
            "resolve_mbzhuguo" => {
                "Active-only effect: adjusts the selected target's hand to min(max HP, 5). A non-draw branch recovers that target by 1 HP. After an actual draw, if the target ties for the largest living hand, the owner may select another character; the adjusted target may then use a current physical Sha including that character, ignoring ordinary distance and use count. Requires `active play_phase uses_per_turn 1 targets 1 1` and Rule VM ABI 331.\n\n```\neffect resolve_mbzhuguo(self, event_source);\n```"
            }
            "resolve_mbanxian" => {
                "Cards-moved-only effect: forces the owner to choose one physical Sha from the triggering first hand-discard event that remains in the shared discard pile, gains it, waits for movement triggers, then uses that exact Sha when a legal target exists without ordinary distance or use-count limits. Requires `trigger cards_moved owner_first_hand_discard_has_sha_in_discard` and Rule VM ABI 332.\n\n```\neffect resolve_mbanxian(self);\n```"
            }
            "resolve_mbjinzu" => {
                "Active-only effect: uses the invocation's selected owner hand card, lets the selected other player privately choose two hand cards when needed, reveals them together, resolves the extrema discard/refresh branches, and accumulates a turn-bound helper for the owner's next Sha. Requires `active play_phase uses_per_turn 1 targets 1 1 other select_hand 1 1` and Rule VM ABI 333.\n\n```\neffect resolve_mbjinzu(self, event_source, 4156343864);\n```"
            }
            "apply_mbjinzu_next_sha" => {
                "Card-used-only effect for the locked Jinsu helper: consumes its target map on the owner's next Sha, makes matching targets unable to respond, and increases only those targets' damage by their accumulated amounts. Requires `trigger card_used owner_source_sha` and Rule VM ABI 333.\n\n```\neffect apply_mbjinzu_next_sha(self, event_card);\n```"
            }
            "choose_other_draw_owner_lost_hp_turn_over" => {
                "Effect: forces `player` to choose another living player. On choice resolution, the target draws cards equal to the owner's current lost HP, then turns over. Owner/target state is revalidated at the continuation barrier. Requires Rule VM ABI 85.\n\n```\neffect choose_other_draw_owner_lost_hp_turn_over(self);\n```"
            }
            "draw_by_play_phase_card_category_count" => {
                "Effect: draws cards equal to the number of distinct logical basic, trick, and equipment categories the player used during the current play phase. Requires Rule VM ABI 42.\n\n```\neffect draw_by_play_phase_card_category_count(self);\n```"
            }
            "choose_discard_one_for_sha_refund_and_draw_if_red" => {
                "Effect: forces `player` to discard exactly one current hand/equipment card, then revalidates the suspended proactive Sha, refunds its ordinary play-phase use count at most once across all skills, and makes `player` draw one if the Sha's effective suit is red. If no cost card remains when the effect starts, nothing happens. Requires Rule VM ABI 43.\n\n```\neffect choose_discard_one_for_sha_refund_and_draw_if_red(self, event_source, event_card);\n```"
            }
            "choose_discard_one_for_sha_refund_and_draw_if_red_restore_if_equal_rank" => {
                "Effect: preserves Longyin's discard, at-most-once Sha-use refund, and effective-red draw, then restores `restore_skill_def` only when the discarded card and suspended Sha have equal effective rank. Requires Rule VM ABI 295.\n\n```\neffect choose_discard_one_for_sha_refund_and_draw_if_red_restore_if_equal_rank(self, event_source, event_card, restore_skill_def);\n```"
            }
            "resolve_beige_configured" => {
                "Effect: reuses Beige's discard, judgment, and rewrite pipeline while supplying the Heart recovery amount and Diamond draw count. Requires Rule VM ABI 296.\n\n```\neffect resolve_beige_configured(self, event_player, event_source, event_amount, 3);\n```"
            }
            "resolve_zhuiyi" => {
                "Effect: lets the dead skill owner cancel or choose one living player other than the owner and credited killer. The chosen player recovers 1 HP, then draws 3 cards even if already at full HP. Requires Rule VM ABI 297.\n\n```\neffect resolve_zhuiyi(self, event_source);\n```"
            }
            "queue_opening_extra_play_phase" => {
                "Effect: queues one independent Play phase at the current player's recoverable turn-begin boundary. Queued phases run before Prepare; each resets play-phase skill counters, logical-card category history, and Sha-use allowance while retaining turn-wide history. Available only from `phase_begin` and requires Rule VM ABI 44.\n\n```\neffect queue_opening_extra_play_phase(self);\n```"
            }
            "convert_current_sha_to_fire_lose_hp_if_damage" => {
                "Effect: revalidates the suspended proactive ordinary Sha, changes only that use's damage nature to fire, and records a post-use cost. After every target and nested dying window resolves, the living user loses 1 HP iff the use committed damage. Requires Rule VM ABI 45.\n\n```\neffect convert_current_sha_to_fire_lose_hp_if_damage(self, event_card);\n```"
            }
            "choose_add_current_sha_target" => {
                "Effect: lets the current Sha user cancel or add one other legal living target. The saved source/card, existing targets, range, and card-target policies are revalidated at the choice barrier. Requires Rule VM ABI 45.\n\n```\neffect choose_add_current_sha_target(self, event_card);\n```"
            }
            "increase_current_sha_base_damage" => {
                "Effect: revalidates the suspended proactive Sha source/card and increases only that use's saved base damage. Requires Rule VM ABI 99.\n\n```\neffect increase_current_sha_base_damage(self, event_card, 1);\n```"
            }
            "increase_current_card_base_damage" => {
                "Effect: revalidates the suspended proactive Sha, Juedou, Nanman, or Wanjian source/card and increases only that use's saved base damage. Juedou support requires Rule VM ABI 229; the original surface requires ABI 104.\n\n```\neffect increase_current_card_base_damage(self, event_card, 1);\n```"
            }
            "lose_hp_then_damage_other_lose_max_hp_if_dying" => {
                "Active-only effect: the owner loses `amount` HP; if that exact loss first puts them into dying, they lose one maximum HP. Equal ordinary damage to the selected other target resolves only after the complete dying/save window. Requires `targets 1 1 other`, `self_skill`, and Rule VM ABI 108.\n\n```\neffect lose_hp_then_damage_other_lose_max_hp_if_dying(self, event_source, 1, self_skill);\n```"
            }
            "owner_source_basic" => {
                "Trigger filter: the owner used a logical basic card. Requires Rule VM ABI 99."
            }
            "owner_source_non_basic" => {
                "Trigger filter: the owner used a logical non-basic card. Requires Rule VM ABI 99."
            }
            "owner_source_wounded_nanman_wanjian_or_normal_spade_sha" => {
                "Card-use trigger filter: the wounded owner used logical Nanman, Wanjian, or an ordinary-nature spade Sha. Requires Rule VM ABI 104."
            }
            "owner_alive_other_play_phase_duplicate_card_category_has_hand_equip" => {
                "Play-phase-end trigger filter: the living owner is not the ending player, can discard from hand/equipment, and that player used at least two logical cards of the same basic/trick/equipment category in the exact Play phase. Requires Rule VM ABI 105."
            }
            "owner_alive_other_phase_end_damage_gt_one_has_hand_equip" => {
                "End-phase trigger filter: the living owner is not the phase player, can discard from hand/equipment, and that player dealt more than one total damage during the current turn. Requires Rule VM ABI 105."
            }
            "choose_store_hand_sha_in_skill_pile" => {
                "Effect: forces the owner to select one or more logical Sha from hand and move them into this skill definition's public expansion pile. Requires Rule VM ABI 46.\n\n```\neffect choose_store_hand_sha_in_skill_pile(self);\n```"
            }
            "choose_spend_skill_pile_for_dying_jiu" => {
                "Effect: forces the owner to spend one card from this skill definition's public expansion pile, then makes the current dying player use a suitless virtual Jiu on themselves. Requires Rule VM ABI 46.\n\n```\neffect choose_spend_skill_pile_for_dying_jiu(self, event_player);\n```"
            }
            "choose_compare_selected_hand_card" => {
                "Effect: after an active `select_hand 1 1`, forces the target to choose one hand card, publicly reveals and discards both cards, then grants a target-bound helper on strict victory or a normal helper on tie/loss. Requires `targets ... can_compare` and Rule VM ABI 47.\n\n```\neffect choose_compare_selected_hand_card(self, event_source, win_skill_def, lose_skill_def);\n```"
            }
            "choose_add_skill_bound_card_target" => {
                "Effect: lets the card user cancel or append this skill instance's bound living target to a suspended one-target Sha or ordinary trick. Source/card, target policies, and the bound state are revalidated at the choice barrier. Requires Rule VM ABI 47.\n\n```\neffect choose_add_skill_bound_card_target(self, event_card);\n```"
            }
            "owner_alive_play_phase_sha_user_has_discardable_hand_equip" => {
                "Trigger filter: the living owner can discard a hand/equipment card and a living player is currently using a logical Sha in that player's play phase. Requires Rule VM ABI 43."
            }
            "owner_is_target_play_phase_card_category_count_positive" => {
                "Trigger filter: the owner is ending their play phase after using at least one logical basic, trick, or equipment card in that phase. Requires Rule VM ABI 42."
            }
            "owner_alive_play_phase_card_category_count_positive" => {
                "Trigger filter: a living owner used at least one logical basic, trick, or equipment card in the exact Play phase that is ending. Requires Rule VM ABI 293."
            }
            "owner_is_target_sha_source_has_discardable_hand_equip" => {
                "Trigger filter: the owner is the logical-Sha damage target, the damage has an identified source, and the owner can currently discard at least one hand/equipment card."
            }
            "discard_all_hand_and_equip" => {
                "Effect: discards every card in the specified player's hand and equipment zones.\n\n```\neffect discard_all_hand_and_equip(event_source);\n```"
            }
            "remove_all_skills" => {
                "Effect: removes every skill currently owned by the specified player. Requires Rule VM ABI 159.\n\n```\neffect remove_all_skills(event_source);\n```"
            }
            "judge_gain_equipment_by_color" => {
                "Effect: performs a judgment; red gains the topmost equipment from the draw pile, black from the discard pile.\n\n```\neffect judge_gain_equipment_by_color(self);\n```"
            }
            "judge_recover_on_suit" => {
                "Effect: performs a judgment for the specified player and recovers HP when the effective result suit matches. The suit is `spade`, `heart`, `club`, or `diamond`. Requires Rule VM ABI 12.\n\n```\neffect judge_recover_on_suit(event_player, heart, 1);\n```"
            }
            "judge_recover_other_on_suit" => {
                "Effect: performs a judgment for `judge_player`, then recovers `recover_player` when the effective result suit matches. Requires Rule VM ABI 88.\n\n```\neffect judge_recover_other_on_suit(event_source, self, spade, 1);\n```"
            }
            "force_others_use_sha_on_nearest_or_lose_hp" => {
                "Effect: snapshots every other living player in clockwise seat order. Each participant may supply a physical or play-phase view-as Sha and choose only minimum-distance legal targets; cancelling or having no legal Sha loses 1 HP. Prompted Sha does not consume the ordinary Sha-use count, and the queue resumes after responses, triggers, and dying saves. Requires Rule VM ABI 90.\n\n```\neffect force_others_use_sha_on_nearest_or_lose_hp(self);\n```"
            }
            "judge_damage_other_on_suit" => {
                "Effect: performs a judgment for the specified player. When the effective suit matches, that player must choose another living player and deal the specified normal, fire, or thunder damage. Requires Rule VM ABI 15.\n\n```\neffect judge_damage_other_on_suit(self, spade, 1, thunder);\n```"
            }
            "recast_selected" => {
                "Effect: moves the active invocation's selected personal-zone cards to the discard pile with the semantic Recast reason, then draws the same number of replacements. Requires Rule VM ABI 16.\n\n```\neffect recast_selected(self);\n```"
            }
            "disable_all_equip_slots" => {
                "Effect: disables every currently enabled equipment slot and discards equipment occupying those slots. Requires Rule VM ABI 18.\n\n```\neffect disable_all_equip_slots(self);\n```"
            }
            "move_equipment_then_draw" => {
                "Effect: lets the skill owner choose one movable equipment card and a legal destination other than themselves and the current holder. Replaces matching equipment at the destination, then the original holder draws one. Requires Rule VM ABI 30.\n\n```\neffect move_equipment_then_draw(self);\n```"
            }
            "choose_use_virtual_sha_or_draw" => {
                "Effect: materializes a normal-distance virtual Sha without consuming the ordinary Sha-use count. The player chooses legal targets; cancelling or having no legal use draws one card instead. Requires Rule VM ABI 31.\n\n```\neffect choose_use_virtual_sha_or_draw(self);\n```"
            }
            "discard_equipment_pair_with_armor" => {
                "Effect: for the active invocation's two selected targets, choose and discard one equipment card from each while requiring at least one selected card to be armor. Requires `targets 2 2 equipment_pair_with_armor` and Rule VM ABI 32.\n\n```\neffect discard_equipment_pair_with_armor(self);\n```"
            }
            "discard_target_equipment_gain_mount" => {
                "Effect: forces the player to choose and discard one discardable equipment card from the target. If the card is an offensive or defensive mount and remains in the discard pile, it is then moved to the player's hand as a separate card movement. Requires Rule VM ABI 33.\n\n```\neffect discard_target_equipment_gain_mount(self, event_player);\n```"
            }
            "discard_target_equipment_draw_weapon_range" => {
                "Active-only effect: forces the skill owner to choose and discard one currently discardable equipment card from the selected target. After the discard movement and nested triggers finish, a discarded physical weapon makes that target draw its printed attack range (minimum one); other equipment draws nothing. Requires exact `targets 1 1 has_discardable_equip` and Rule VM ABI 136.\n\n```\neffect discard_target_equipment_draw_weapon_range(self, event_source);\n```"
            }
            "choose_target_reveal_hand_damage_if_sha" => {
                "Effect: forces `player` to choose any living player with a nonempty hand, publicly reveals that target's complete current hand, and deals `amount` normal damage from `player` when at least one revealed card is an effective Sha for that target. Skill, observer, participants, hand state, and logical card kinds are revalidated at the Choice barrier. Requires Rule VM ABI 137.\n\n```\neffect choose_target_reveal_hand_damage_if_sha(self, 1);\n```"
            }
            "choose_other_give_one_hand_equip" => {
                "Effect: forces `player` to choose another living player, then give that target exactly one current hand/equipment card through the shared gift pipeline. Skill ownership, participants, target legality, and card ownership are revalidated at the choice barriers. Requires Rule VM ABI 138.\n\n```\neffect choose_other_give_one_hand_equip(self);\n```"
            }
            "choose_reveal_hand_card_bind_category_target" => {
                "Effect: forces the skill owner to choose and publicly reveal one current hand card from the specified Play-phase player, then binds that player and the card's effective basic/trick/equipment category to this skill instance until the phase changes. Requires Rule VM ABI 140.\n\n```\neffect choose_reveal_hand_card_bind_category_target(self, event_player);\n```"
            }
            "recover_to_max_then_optional_exchange_skill" => {
                "Effect: consumes the originating limited skill, recovers its owner to max HP, waits for recovery and nested triggers to finish, then optionally sacrifices the specified owned skill definition to restore a fresh instance of the limited skill. Requires Rule VM ABI 142.\n\n```\neffect recover_to_max_then_optional_exchange_skill(self, 3956037825);\n```"
            }
            "reveal_hand_discard_kind" => {
                "Effect: publicly reveals `player`'s complete current hand, then discards every currently discardable hand card whose effective kind matches `card_kind`. Requires Rule VM ABI 144.\n\n```\neffect reveal_hand_discard_kind(event_player, shan);\n```"
            }
            "discard_target_card_gain_mount" => {
                "Effect: forces the player to choose and discard one target card from `equip`, `hand_equip`, or `hand_equip_judge`. If the card is a mount and remains in the discard pile, it is moved to the player's hand as a separate movement. The scope is revalidated at the continuation barrier. Requires Rule VM ABI 34.\n\n```\neffect discard_target_card_gain_mount(self, event_player, hand_equip_judge);\n```"
            }
            "choose_discard_target_card" => {
                "Effect: forces the player to choose and discard one target card from `hand`, `equip`, `hand_equip`, or `hand_equip_judge`. The scope, life state, card ownership, and discard protection are revalidated at the continuation barrier. `hand` requires Rule VM ABI 153; other scopes require ABI 36.\n\n```\neffect choose_discard_target_card(self, event_player, hand_equip);\n```"
            }
            "reveal_hand_to_player" => {
                "Effect: reveals one living player's complete current hand only to one living observer. Requires Rule VM ABI 153.\n\n```\neffect reveal_hand_to_player(self, event_source);\n```"
            }
            "reveal_hand" => {
                "Effect: publicly reveals one living player's complete current hand. Requires Rule VM ABI 336.\n\n```\neffect reveal_hand(self);\n```"
            }
            "choose_optional_discard_target_card" => {
                "Effect: lets the player cancel or choose and discard one target card from `hand`, `equip`, `hand_equip`, or `hand_equip_judge`. An empty scope is a no-op; submitted cards are revalidated at the continuation barrier. Requires Rule VM ABI 153.\n\n```\neffect choose_optional_discard_target_card(self, event_source, hand);\n```"
            }
            "reveal_hand_choose_discard_color" => {
                "Effect: publicly reveals `player`'s complete current hand. If both effective red and black are present, that player must choose one color; if only one is present it is selected automatically. Every currently discardable hand card of the selected color is then discarded. Requires Rule VM ABI 154.\n\n```\neffect reveal_hand_choose_discard_color(event_player);\n```"
            }
            "choose_discard_target_cards_recover_same_category" => {
                "Active-only effect: forces the owner to discard `min(count, available)` current hand/equipment cards from the selected target. If exactly `count` cards were discarded and their legacy type2 categories match, the owner recovers `amount` HP after movement triggers and nested choices finish. Requires exact `targets 1 1 other in_attack_range has_discardable_hand_equip` and Rule VM ABI 155.\n\n```\neffect choose_discard_target_cards_recover_same_category(self, event_source, 2, 1);\n```"
            }
            "choose_compare_selected_hand_card_second_failure_seat_damage" => {
                "Active-only effect: compares the selected hand card with one unused target. A strict win lets the owner draw or discard one target hand/equipment card. On the configured non-win threshold, the owner loses HP, chooses between equal seat paths or takes the shorter path, damages its living players in order, and the active skill is banned until turn end. Requires `targets 1 1 other can_compare unused_by_skill_this_turn select_hand 1 1` and Rule VM ABI 156.\n\n```\neffect choose_compare_selected_hand_card_second_failure_seat_damage(self, event_source, 1, 2, 1, 1);\n```"
            }
            "reveal_discard_suit_gain_or_use" => {
                "Effect: reveals base_count plus capped mark_count cards, then lets the player perform the discard-by-suit gain and use-one-revealed-card options in either order. Completing both adds one mark and recasts the player's hand.\n\n```\neffect reveal_discard_suit_gain_or_use(self, mark_def, 3, 2);\n```"
            }
            "choose_discard_or_fire_damage_skip_phases" => {
                "Effect: lets the current-turn player choose the discard branch, fire-damage branch, or both. The discard branch may skip draw; the damage branch may skip play.\n\n```\neffect choose_discard_or_fire_damage_skip_phases(self, 2, 1);\n```"
            }
            "remove_cards_by_kind" => {
                "Effect: moves every card of one rule kind from draw, discard, hand, equipment, and judgment zones into the shared removed-from-game zone.\n\n```\neffect remove_cards_by_kind(horse_off);\n```"
            }
            "kill_player" => {
                "Effect: immediately kills a player and attributes the death to the specified killer.\n\n```\neffect kill_player(event_player, event_source);\n```"
            }
            "owner_alive_current_turn_other_event_player" => {
                "Trigger filter: the living skill owner is the current turn player and differs from the event player."
            }
            "owner_alive_current_turn_other_event_player_from_hand_empty" => {
                "Trigger filter: the living current-turn skill owner observes another living player lose hand cards and become empty-handed."
            }
            "owner_is_target_current_turn" => {
                "Trigger filter: the living skill owner is both the event player and the current turn player. Requires Rule VM ABI 351."
            }
            "owner_gains_multiple_outside_own_draw" => {
                "Trigger filter: the living skill owner gains more than one card in one draw or move event outside their own draw phase."
            }
            "owner_gained_or_lost_at_least_two" => {
                "Trigger filter: the living skill owner gains or loses at least two cards in one draw or move event."
            }
            "owner_gains_outside_turn_mark_below_alive_once_2058376721" => {
                "Parameterized trigger filter: once per turn, the living owner gains cards outside their turn while the mark named by the numeric suffix is below the living-player count."
            }
            "owner_card_responded_to_by_player_in_range" => {
                "Trigger filter: a living player whose attack range includes the skill owner responded to a card used by that owner."
            }
            "owner_alive_first_cards_moved_to_hand" => {
                "Trigger filter: the living skill owner observes the first non-draw card move into any player's hand this turn."
            }
            "owner_draws_during_own_play_not_self" => {
                "Trigger filter: the living owner draws during their own play phase, excluding draws emitted by this skill instance."
            }
            "owner_source_non_equip_no_basic_hand_has_targets" => {
                "Trigger filter: the living owner used a non-equipment card with at least one target while holding no basic hand cards."
            }
            "owner_equip_zone_changed_alive_wounded_at_least_two" => {
                "Trigger filter: an equipment card entered or left the living owner's equipment zone while at least two living players were wounded."
            }
            "owner_alive_once_per_round" => {
                "Trigger filter: the skill owner is alive and this skill instance has not fired during the current round."
            }
            "owner_is_event_player_no_enabled_equip_slot_once_per_game" => {
                "Lifetime trigger filter: the living skill owner is the event player, has no enabled equipment slot, and this skill instance has never fired before. Requires Rule VM ABI 19."
            }
            "owner_alive_other_dying_hand_nonempty_once_per_game" => {
                "Lifetime dying trigger filter: the living owner observes another living player at non-positive HP, has at least one hand card, and this skill instance has never fired before. Requires Rule VM ABI 125."
            }
            "owner_alive_sha_or_duel_damage_source_target_alive_hand_equip_nonempty" => {
                "Damage trigger filter: the living owner observes damage from a logical Sha or Duel; the living source and target must collectively have at least one hand/equipment card. Requires Rule VM ABI 126."
            }
            "owner_alive_sha_damage_source_target_alive_owner_hand_equip_nonempty" => {
                "Damage trigger filter: the living owner observes sourced logical-Sha damage to a living player and can discard one of their own hand/equipment cards. The source may already be dead when damage-end triggers settle. Requires Rule VM ABI 219."
            }
            "owner_is_target_any_alive_self_discardable_hand_equip" => {
                "End-phase trigger filter: the living owner is the phase player and at least one living player can discard one of their own hand/equipment cards. Requires Rule VM ABI 127."
            }
            "owner_is_target_any_alive_discardable_by_owner_hand_equip" => {
                "Prepare/End-phase trigger filter: the living owner is the phase player and may discard at least one hand/equipment card from some living player, including themselves. Requires Rule VM ABI 298."
            }
            "owner_is_target_any_alive_self_discardable_hand" => {
                "Phase trigger filter: the living owner is the phase player and at least one living player can discard one of their own hand cards. Requires Rule VM ABI 194."
            }
            "owner_is_target_any_alive_wounded" => {
                "Phase trigger filter: the living owner is the phase player and at least one living player is below maximum HP. Requires Rule VM ABI 195."
            }
            "owner_current_turn_other_equipment_lost" => {
                "Trigger filter: during the living skill owner's turn, another player loses one or more cards from their equipment zone."
            }
            "owner_alive_other_discarded_basic_outside_turn" => {
                "Trigger filter: outside the living owner's turn, another player discards at least one basic card from hand or equipment."
            }
            "owner_alive_other_discarded_basic_outside_turn_once" => {
                "Trigger filter: once during each character's turn, outside the living owner's turn, another player discards at least one basic card from hand or equipment."
            }
            "owner_alive_other_discarded_black_to_discard" => {
                "Cards-moved trigger filter: another player discarded at least one raw black physical card into the discard pile. Only matching black cards are exposed through `event_cards`; requires Rule VM ABI 112."
            }
            "owner_from_equip_per_card" => {
                "Cards-moved trigger filter: the skill owner lost one or more equipment cards; offers the trigger independently once per moved equipment card. Requires Rule VM ABI 175."
            }
            "owner_outside_turn_red" => {
                "View-as filter: the source card is red and the skill owner is not the current turn player. Requires Rule VM ABI 176."
            }
            "owner_alive_dealt_or_received_damage_this_turn" => {
                "Turn-end trigger filter: the living owner dealt or received damage during the turn that just ended."
            }
            "owner_received_gift_from_other" => {
                "Cards-moved trigger filter: the living skill owner received cards voluntarily given by another player."
            }
            "owner_initiated_duel_target_alive_loser_has_hand_or_equip" => {
                "Duel-resolved trigger filter: the living owner initiated the Duel, its original target survived, and the loser can give a hand/equipment card to the winner."
            }
            "duel_resolved" => {
                "Trigger event emitted after a Duel resolves; `event_player` is the winner and `event_source` is the loser."
            }
            "character_state_changed" => {
                "Trigger event emitted when a character card turns over or actually changes between linked and unlinked; `event_player` is the affected player. Requires Rule VM ABI 25."
            }
            "use_virtual_card_on" => {
                "Effect: materialize a suitless virtual card and use it on one explicit target through the ordinary card pipeline."
            }
            "recover_hp_as" => {
                "Effect: recover HP using an explicit source player for event attribution. Requires Rule VM ABI 185.\n\n```\neffect recover_hp_as(event_source, self, 1);\n```"
            }
            "choose_discard_redirect_current_sha" => {
                "Effect: let the current Sha target select a discardable hand/equipment card and a legal replacement in the owner's attack range. The card is paid only after both choices, and the saved Sha target slot changes after discard triggers finish. Requires Rule VM ABI 186.\n\n```\neffect choose_discard_redirect_current_sha(self, event_source, event_card);\n```"
            }
            "judge_set_skill_mark_to_rank" => {
                "Effect: perform a judgment for `player`, then replace the originating skill's same-id mark with the final physical result rank. Requires Rule VM ABI 188.\n\n```\neffect judge_set_skill_mark_to_rank(self, self_skill);\n```"
            }
            "choose_discard_one_from_each_linked" => {
                "Effect: snapshot living linked players in seat order and let the skill owner choose one discardable hand/equipment card from each; each choice is revalidated. Requires Rule VM ABI 189.\n\n```\neffect choose_discard_one_from_each_linked(self, self_skill);\n```"
            }
            "target_discard_then_duel_if_non_sha_and_hp_gte_owner" => {
                "Active-only effect: force the selected target to discard one hand/equipment card, wait for its movement triggers, then use a virtual Duel on it when that card was not Sha, its HP is at least the owner's, and the Duel remains legal. Requires `targets 1 1 other in_attack_range hand_equip_nonempty` and Rule VM ABI 192."
            }
            "choose_target_lose_hp_discard_hand_sha_if_basic" => {
                "Effect: forces the owner to choose any living player able to discard a hand card, loses 1 HP, then forces that target to discard one hand card. After movement triggers, a basic discarded card causes an ordinary-count virtual Sha that ignores distance when the use remains legal. Requires Rule VM ABI 194."
            }
            "sha_distance_unlimited_if_rank_lt_skill_mark" => {
                "Modifier target: a logical Sha whose physical rank is below this skill's same-id mark ignores ordinary distance. Requires `override 1` and Rule VM ABI 188."
            }
            "sha_distance_unlimited_if_target_distance_lte_rank" => {
                "Modifier target: a logical physical Sha ignores its ordinary target-distance limit only when the final distance to that target is at most the card's physical rank. Requires `override 1` and Rule VM ABI 223."
            }
            "sha_no_use_count_if_rank_gt_skill_mark" => {
                "Modifier target: a logical Sha whose physical rank is above this skill's same-id mark bypasses and does not consume the ordinary use count. Requires `override 1` and Rule VM ABI 188."
            }
            "use_virtual_card_on_selected_targets" => {
                "Effect: materialize one suitless virtual Duel or Tiesuo and use it on the active skill's selected distinct targets through the ordinary card pipeline. Duel requires other targets and Rule VM ABI 95; Tiesuo requires 1–2 targets and Rule VM ABI 343."
            }
            "deal_damage_ban_active_if_dying" => {
                "Active-only effect: deal ordinary source-free normal damage through the standard damage-before and dying pipelines. If this exact damage first makes the target enter dying, ban `self_skill` from active use until the next turn begins. Requires Rule VM ABI 96.\n\n```\neffect deal_damage_ban_active_if_dying(self, event_source, 1, self_skill);\n```"
            }
            "choose_target_reclaim_equipment_draw_if_range_reduced" => {
                "Active-only effect: let the skill owner choose one card from the selected target's equipment zone, move it into that target's own hand, then draw one card for the owner only if the target's attack-range population decreased after nested card-movement triggers finish. Requires exact `targets 1 1 other has_equip` and Rule VM ABI 97.\n\n```\neffect choose_target_reclaim_equipment_draw_if_range_reduced(self, event_source, self_skill);\n```"
            }
            "choose_use_virtual_card" => {
                "Triggered effect: materialize a suitless source-free Wanjian or Juedou and pass it through ordinary use, target, Wuxie, response, damage, and discard resolution. Wanjian uses its complete legal target set; Juedou opens a mandatory target choice. Requires Rule VM ABI 98.\n\n```\neffect choose_use_virtual_card(self, wanjian, self_skill);\n```"
            }
            "choose_use_virtual_nanman" => {
                "Triggered effect: materialize a suitless source-free Nanman (南蛮入侵) and pass it through ordinary use, target, Wuxie, response, damage, and discard resolution. Nanman uses its complete legal target set and is phase-agnostic (no phase-end history suppression). Requires Rule VM ABI 604.\n\n```\neffect choose_use_virtual_nanman(self, self_skill);\n```"
            }
            "resolve_qiluan" => {
                "Triggered effect: draw Σ alive players' current-turn kill counts × (3 if the killer is the owner else 1). Implements 起乱 (qiluan). No-op when the total is zero. Requires Rule VM ABI 605.\n\n```\neffect resolve_qiluan(self, self_skill);\n```"
            }
            "discard_all_own_judge" => {
                "Triggered effect: discard every card currently in the player's judgment zone. Implements 飞杨 (olfeiyang). Requires Rule VM ABI 606.\n\n```\neffect discard_all_own_judge(self, self_skill);\n```"
            }
            "store_event_card_in_skill_pile" => {
                "Triggered effect: store the triggering event's used card onto the owner's skill pile (武将牌) when the card sits in the discard pile. Implements 灵杉 (huamu). Requires Rule VM ABI 607.\n\n```\neffect store_event_card_in_skill_pile(self, event_card, self_skill);\n```"
            }
            "discard_hand_expr" => {
                "Triggered effect: forced hand discard with an expression-driven count, clamped to the available hand (e.g. `7 - mark_count(self, def)`). Compiles to the same VM op as literal-count discard_hand.\n\n```\neffect discard_hand_expr(event_source, 7 - mark_count(self, 3913395307));\n```"
            }
            "discard_hand_equip_expr" => {
                "Triggered effect: forced hand-or-equip discard with an expression-driven count, clamped to the available cards. Compiles to the same VM op as literal-count discard_hand_equip.\n\n```\neffect discard_hand_equip_expr(self, hand_count(self) + 2 - hand_limit(self));\n```"
            }
            "gain_target_hand_card" => {
                "Triggered effect: the owner selects another living player, then gains one of that player's hand cards (the owner picks the card). Implements 肃州 (pxsizhou). Requires Rule VM ABI 647.\n\n```\neffect gain_target_hand_card(self, self_skill);\n```"
            }
            "choose_target_gain_max_hp" => {
                "Triggered effect: the owner selects any living player (self included); the selected player gains 1 max hp. Implements 寿春 (pxshouchun). Requires Rule VM ABI 646.\n\n```\neffect choose_target_gain_max_hp(self, self_skill);\n```"
            }
            "self" => "Refers to the skill owner (the player who has this skill).",
            "event_amount" => {
                "Amount bound to a damage, HP-loss, or HP-recovery trigger. It can be used in effect expressions, for example `draw_cards(self, event_amount * 3)`."
            }
            "event_target_count" => {
                "Number of declared targets bound to a `card_used` or `card_targeted` trigger."
            }
            "turn_damaged_player_count" => {
                "Number of distinct players damaged during the current turn, including players who later died; available in `phase_end` triggers."
            }
            "owner_is_target" => {
                "Trigger filter: fires only when the owner is the target of the event."
            }
            "owner_alive_is_target_current_play_phase" => {
                "Trigger filter: the living owner is the event player, the owner is the current turn player, and the current phase is Play. Requires Rule VM ABI 106."
            }
            "owner_source" => "Trigger filter: fires only when the owner is the event source.",
            "owner_is_target_not_unique_max_current_hp" => {
                "Trigger filter: fires only when the owner is the event player and does not have strictly more current HP than every other living player."
            }
            "owner_is_target_once" => {
                "Trigger filter: fires only when the owner is the event player and this skill has not fired during the current turn."
            }
            "owner_source_target_alive_has_equip" => {
                "Damage trigger filter: the living owner is the source and the living damaged player has at least one equipment card."
            }
            "owner_source_black_sha_target_alive_has_discardable_equip" => {
                "Damage trigger filter: the living owner caused damage with a physical black logical Sha, and the living damaged player has at least one equipment card the owner may discard. Requires Rule VM ABI 33."
            }
            "owner_source_sha_target_alive_has_discardable_region_card" => {
                "Damage trigger filter: the living owner caused damage with a logical Sha, and the living damaged player has at least one discardable card in hand, equipment, or judgment zones. Requires Rule VM ABI 34."
            }
            "owner_source_sha_target_alive_has_discardable_hand_equip" => {
                "Card-target trigger filter: the living owner used a logical Sha, and the current living target has at least one hand/equipment card the owner may discard. Requires Rule VM ABI 94."
            }
            "owner_source_tiesuo_target_alive_unlinked_hand_equip_nonempty" => {
                "Card-target trigger filter: the living owner used a logical Tiesuo, and the current living target is unlinked with a nonempty hand/equipment region. Requires Rule VM ABI 305."
            }
            "owner_alive_source_target_was_linked" => {
                "Damage trigger filter: the living owner was the source and the target was linked immediately before that damage segment committed. Requires Rule VM ABI 306."
            }
            "owner_source_sha_target_alive_hand_nonempty" => {
                "Card-target trigger filter: the living owner used a logical Sha, and the current living target has at least one hand card. Requires Rule VM ABI 154."
            }
            "owner_source_sha_target_hand_gte_hp_or_lte_attack_range" => {
                "Card-target trigger filter: the living owner used a logical Sha, and the current living target's hand count is at least the owner's HP or at most the owner's current attack range. Requires Rule VM ABI 35."
            }
            "owner_source_sha_target_hand_lte_owner_or_hp_gte_owner" => {
                "Card-target trigger filter: the living owner used a logical Sha, and the current living target has no more hand cards than the owner or at least as much HP as the owner. Requires Rule VM ABI 223."
            }
            "owner_source_direct_sha_damage_target_armor_or_mount_card_gainable" => {
                "Damage trigger filter: the living owner dealt direct logical Sha damage to a living target with armor or a mount, and the physical Sha remains in processing or discard. Requires Rule VM ABI 318."
            }
            "owner_is_source_or_target_xinyaoming_available" => {
                "Damage trigger filter: the living owner is the damage source or target and at least one legal target/action pair remains for 界邀名 this turn. Requires Rule VM ABI 319."
            }
            "owner_alive_event_player_lost_armor_or_mount_has_originating_skill_pile" => {
                "Cards-moved trigger filter: a living event player lost armor or a mount from equipment while holding cards in the public pile keyed by the observer's originating skill. Requires Rule VM ABI 318."
            }
            "owner_is_target_judge_nonempty" => {
                "Prepare-phase trigger filter: the living owner is the event player and currently has at least one judgment-zone card. Requires Rule VM ABI 224."
            }
            "owner_is_target_draw_not_replaced" => {
                "Draw-phase trigger filter: the living owner is the event player and the pending normal draw has not already been replaced. Requires Rule VM ABI 225."
            }
            "owner_is_target_draw_not_fixed_hp_one_or_hand_empty" => {
                "Draw-phase trigger filter: the living owner is the event player, the pending normal draw count is not fixed, and the owner has exactly 1 HP or no hand cards. Requires Rule VM ABI 300."
            }
            "owner_source_sha_target_alive_owner_outside_target_attack_range" => {
                "Card-target trigger filter: the living owner used a logical Sha on a living target whose current attack range does not include the owner. Requires Rule VM ABI 119."
            }
            "owner_source_sha_dodged_target_alive_has_discardable_hand_equip" => {
                "Sha-dodged trigger filter: the living owner used the fully dodged Sha, and the living target has at least one discardable hand or equipment card. Requires Rule VM ABI 36."
            }
            "owner_source_sha_dodged_target_recorded_by_skill_has_discardable_hand_equip" => {
                "Sha-dodged trigger filter: the living owner armed this skill for the exact dodged Sha target, and that target can discard one owner hand/equipment card. Requires Rule VM ABI 152."
            }
            "owner_source_sha_target_alive_distance_le_one" => {
                "Damage trigger filter: the living owner is the source of logical Sha damage, the living target remains alive, and the final distance from owner to target is at most one. Requires Rule VM ABI 37."
            }
            "owner_alive_sha_target_distance_le_one" => {
                "Per-target trigger filter: a living owner observes a living logical Sha target whose final distance from the owner is at most one. Requires Rule VM ABI 227."
            }
            "owner_alive_sha_target_distance_le_one_has_equip_card" => {
                "Per-target trigger filter: a living owner observes a living logical Sha target within distance one and can discard an equipment card from hand/equipment. Requires Rule VM ABI 613."
            }
            "owner_source_target_alive_has_hand_or_equip" => {
                "Damage trigger filter: the living owner is the source and the living damaged player has at least one hand or equipment card."
            }
            "owner_source_other_target_alive_has_discardable_hand_equip" => {
                "Damage trigger filter: the living owner damaged another living player and may currently discard at least one card from that target's hand or equipment zone. Requires Rule VM ABI 118."
            }
            "owner_targeted_by_other_sha_once" => {
                "Card-target trigger filter: the living owner is targeted by another player's Sha and this skill has not fired during the current turn."
            }
            "owner_targeted_by_other_sha" => {
                "Card-target trigger filter: the living owner is targeted by another player's Sha."
            }
            "owner_is_target_wounded" => {
                "Trigger filter: the skill owner is the event player and is wounded."
            }
            "owner_is_target_linked" => {
                "Trigger filter: the living skill owner is the event player and is currently linked. Requires Rule VM ABI 189."
            }
            "owner_is_target_chain_damage_linked" => {
                "Damage-before trigger filter: the skill owner is the damage target and the damage is iron-chain-propagated. Implements 红棉百花袍. Requires Rule VM ABI 575."
            }
            "owner_is_target_wounded_sha" => {
                "Damage-before trigger filter: the skill owner is wounded and the damage comes from a Sha card. Implements 得势 (dcdeshi). Requires Rule VM ABI 575."
            }
            "owner_is_target_healthy_draw_not_fixed" => {
                "Draw-phase trigger filter: the skill owner is at full health and the draw phase amount is not fixed. Implements 螭纹玉佩 (chiwenyupei) draw branch. Appended for ABI-stable serialization."
            }
            "owner_is_target_alive_wounded_or_guiming_wu_exists" => {
                "Draw-phase trigger filter: the living owner is the event player and at least one living wounded player, or eligible other living Wu player under enabled Lord skill Guiming, exists. Requires Rule VM ABI 190."
            }
            "owner_is_target_hand_nonempty" => {
                "Trigger filter: the skill owner is the event player and has at least one hand card."
            }
            "owner_is_target_hand_below_max_hp" => {
                "Trigger filter: the skill owner is the event player and has fewer hand cards than maximum HP."
            }
            "owner_is_target_hand_above_hp" => {
                "Trigger filter: the living skill owner is the event player and has more hand cards than current HP."
            }
            "owner_first_transformed_sha_used_this_turn" => {
                "Card-used trigger filter: the living owner used a logical Sha converted by a skill from at least one physical card, limited to the first matching use each turn. Requires Rule VM ABI 357."
            }
            "owner_target_hand_empty_elemental_or_trick_damage_before" => {
                "Damage-before trigger filter: the living owner has no hand cards and is about to take elemental or ordinary-trick damage. Requires Rule VM ABI 358."
            }
            "owner_is_target_hand_at_least_hp_other_alive" => {
                "Trigger filter: the living skill owner is the event player, has at least as many hand cards as current HP, and another living player exists. Requires Rule VM ABI 69."
            }
            "owner_is_target_lost_cards_at_most_hp_gainable" => {
                "Trigger filter: the living skill owner is the event player, lost between one and current-HP cards from hand/equipment this turn, and at least one unique lost physical card remains in discard. Loss history is tracked only when loaded content needs this filter. Requires Rule VM ABI 201."
            }
            "owner_is_target_source_movable_field_card" => {
                "Damage trigger filter: the living owner is the damaged event player, the source is living, and at least one equipment or delayed-trick card can be moved from another living player's field to the source. Requires Rule VM ABI 202."
            }
            "owner_from_hand_empty_other_discardable_hand_equip" => {
                "Cards-moved trigger filter: the living skill owner lost at least one card from hand in this movement batch, is now empty-handed, and another living player has at least one hand/equipment card the owner may discard. Requires Rule VM ABI 70."
            }
            "owner_is_source_or_target" => {
                "Trigger filter: the skill owner is either the damage source or the damaged player."
            }
            "owner_is_source_or_target_other_alive_per_damage_point" => {
                "Damage trigger filter: the owner is one endpoint of sourced damage between two distinct living players; the trigger is offered independently once per damage point."
            }
            "owner_target_fire" => "Trigger filter: the skill owner is receiving fire damage.",
            "owner_target_thunder" => {
                "Trigger filter: the skill owner is receiving thunder damage."
            }
            "mark_count" => {
                "Read-only query: `mark_count(player, mark_id)` returns the player's current stable mark count."
            }
            "alive_count" => {
                "Read-only query: `alive_count()` returns the number of living players."
            }
            "card_action_count" => {
                "Read-only query: `card_action_count(player)` returns cards used or played as responses by that player during the current turn."
            }
            "empty_equip_slots" => {
                "Read-only query: `empty_equip_slots(player)` returns the number of empty equipment slots."
            }
            "owner_is_target_no_turn_damage" => {
                "Trigger filter: the skill owner is the event player and has dealt no damage during the current turn."
            }
            "owner_is_target_turn_damage" => {
                "Trigger filter: the skill owner is the event player and has dealt damage during the current turn."
            }
            "owner_is_target_turn_damage_at_least_three" => {
                "Trigger filter: the skill owner is the event player and has dealt at least three damage during the current turn."
            }
            "owner_is_target_card_used_at_least_hp" => {
                "Trigger filter: the skill owner is the event player and has proactively used at least as many cards as their current HP during this turn."
            }
            "owner_is_target_use_response_discarded_at_least_hp" => {
                "Trigger filter: the skill owner is the current turn's event player and at least current-HP physical cards used or played as responses by any player entered the discard pile this turn."
            }
            "owner_is_target_damaged_other_qun" => {
                "Trigger filter: the skill owner is the event player and damaged another Qun character during the current turn."
            }
            "owner_is_target_source_alive" => {
                "Trigger filter: owner is the event target and the event source is still alive."
            }
            "owner_is_target_source_alive_once" => {
                "Trigger filter: owner is the event target, the event source is still alive, and this skill has not fired during the current turn. Requires Rule VM ABI 165."
            }
            "owner_is_target_source_has_weapon" => {
                "Damage trigger filter: owner is the damaged player and an identified damage source currently has at least one weapon equipped. Requires Rule VM ABI 117."
            }
            "owner_is_target_source_alive_disabled_weapon" => {
                "Damage trigger filter: owner is the damaged player and the living damage source has at least one disabled weapon slot. Requires Rule VM ABI 161."
            }
            "owner_is_target_other_source_alive" => {
                "Trigger filter: owner is the event target and a different event source is still alive."
            }
            "owner_is_target_other_source_alive_per_damage_point" => {
                "Damage trigger filter: owner is the damaged player and a different source remains alive; the trigger is emitted once per damage point."
            }
            "owner_is_target_per_damage_point" => {
                "Damage trigger filter: owner is the damaged player; the trigger is emitted once per damage point, including self-sourced or source-free damage. Requires Rule VM ABI 84."
            }
            "owner_is_target_per_damage_point_other_hand_equip_judge_nonempty" => {
                "Damage trigger filter: the living owner is the damaged player and at least one other living player has a hand, equipment, or judgment card; the trigger is emitted once per damage point. Requires Rule VM ABI 206."
            }
            "owner_is_target_other_alive" => {
                "Trigger filter: owner is the event player and at least one other player remains alive. Requires Rule VM ABI 85."
            }
            "owner_is_target_hand_equip_nonempty_other_alive" => {
                "Damage trigger filter: the living owner is the damaged player of a positive damage event, owns at least one hand/equipment card, and at least one other player remains alive. Requires Rule VM ABI 138."
            }
            "owner_is_target_phase_end_hand_equip_nonempty_other_alive" => {
                "End-phase trigger filter: the living owner is the event player, owns at least one hand/equipment card, and at least one other player remains alive. Requires Rule VM ABI 280."
            }
            "owner_alive_gained_three_this_turn_other_alive" => {
                "Turn-end trigger filter: the living owner gained at least three physical cards during the current turn and another living player exists. Card gains are tracked only when loaded content needs this filter. Requires Rule VM ABI 139."
            }
            "owner_alive_event_player_hand_nonempty_once_per_round" => {
                "Play-phase-start trigger filter: the owner and phase player are alive, the phase player has at least one hand card, and this skill instance has not fired during the current round. Requires Rule VM ABI 140."
            }
            "owner_source_bound_category_first_three_play_phase" => {
                "Card-used trigger filter: the current Play-phase player matches this skill instance's bound target and the used card's effective basic/trick/equipment category matches the bound category; fires at most three times in that Play phase. Requires Rule VM ABI 140."
            }
            "owner_source_damage_card_once_per_turn" => {
                "Card-used trigger filter: the living owner used a logical Sha or damage trick and this skill instance has not fired during the current turn. Requires Rule VM ABI 141."
            }
            value if value.starts_with("owner_alive_other_judgment_black_event_player_group_") => {
                "Trigger filter: owner and another judged player are alive, the judged player has the requested group tag, and the final judgment result is black. Requires Rule VM ABI 86."
            }
            value if value.starts_with("owner_alive_other_damage_source_group_") => {
                "Trigger filter: a living owner observes damage dealt by another living player whose character group equals the suffix. Requires Rule VM ABI 88."
            }
            "owner_is_target_source_alive_gainable_card_per_damage_point" => {
                "Damage trigger filter: owner is the damaged player and the living source has a card the owner may take; emits once per damage point. Self-source exposes equipment only, another source exposes hand and equipment. Requires Rule VM ABI 22."
            }
            "owner_is_target_source_alive_gainable_card" => {
                "Damage trigger filter: owner is the damaged player and the living source has a card the owner may take; emits once for the damage event. Self-source exposes equipment only, another source exposes hand and equipment. Requires Rule VM ABI 169."
            }
            "owner_is_target_card_in_processing" => {
                "Damage trigger filter: owner is the damaged player and the current physical damage-card carrier is still in the processing zone. Requires Rule VM ABI 170."
            }
            "owner_is_target_red_sha_source_alive" => {
                "Trigger filter: owner is receiving card-linked damage from a red logical Sha whose source is alive."
            }
            "owner_is_target_nonred_sha" => {
                "Trigger filter: owner is receiving card-linked damage from a non-red logical Sha."
            }
            "owner_is_target_red_or_jiu_sha" => {
                "Trigger filter: owner received card-linked damage from a red logical Sha or a logical Sha boosted by Jiu."
            }
            "owner_is_target_red_card_source_alive" => {
                "Trigger filter: owner is receiving card-linked damage from a red card whose source is alive."
            }
            "owner_is_target_nonred_card" => {
                "Trigger filter: owner is receiving card-linked damage from a non-red card."
            }
            "owner_is_target_first_damage" => {
                "Trigger filter: owner is the damaged player and this skill has not handled damage earlier this turn."
            }
            "owner_is_target_second_damage" => {
                "Trigger filter: owner is the damaged player and this skill has handled exactly one damage event earlier this turn."
            }
            "owner_is_target_after_first_damage" => {
                "Trigger filter: owner is the damaged player and this skill already handled damage earlier this turn."
            }
            "owner_alive_source_target_same_sex" => {
                "Trigger filter: the skill owner is alive and the event source and target share a sex tag; `double` matches either known sex."
            }
            "owner_alive_event_player_same_group" => {
                "Trigger filter: the living skill owner and event player have the same non-empty character group tag."
            }
            "owner_alive_event_player_same_group_hand_nonempty" => {
                "Trigger filter: the living skill owner and event player have the same non-empty group tag, and the owner has at least one discardable hand card. Requires Rule VM ABI 596."
            }
            "owner_tracked_juedou_original_target_damage_recoverable_in_range" => {
                "Damage trigger filter: the owner dealt damage to the original target of this skill instance's exact tracked virtual Duel, and that target's attack range contains a wounded living character other than the owner. Requires Rule VM ABI 597."
            }
            "owner_is_target_phase_end_hand_nonempty_other_hand_nonempty" => {
                "End-phase trigger filter: the living owner is the phase owner, has hand cards, and another living character also has hand cards. Requires Rule VM ABI 598."
            }
            "owner_is_target_ordinary_trick_cilv_available" => {
                "Card-targeted filter: the owner is targeted by an ordinary trick and still has at least one Cilv option. Requires Rule VM ABI 600."
            }
            "cilv_tracked_card_damage_before" => {
                "Damage-before filter: the exact source card is tracked by this Cilv instance's damage-prevention option. Requires Rule VM ABI 600."
            }
            "cilv_tracked_card_resolved" => {
                "Card-resolved filter: the exact card has pending Cilv damage-prevention cleanup or material gain. Requires Rule VM ABI 600."
            }
            "owner_dying_zhupeilan_once_per_game" => {
                "Dying filter: the owner is dying and this Tongdao instance has not fired during the match. Requires Rule VM ABI 600."
            }
            "owner_alive_other_play_phase_tied_max_hand_dcsbfuxi_available" => {
                "Play-phase filter: another tied-most-hand player began their Play phase and Fuxi has a legal branch. Requires Rule VM ABI 601."
            }
            "owner_phase_end_dcsbhaoyi_available" => {
                "End-phase filter: the owner can gain a damage card that entered discard this turn without dealing damage. Requires Rule VM ABI 601."
            }
            "owner_alive_global_fire_damage_dcsbronghuo" => {
                "Damage filter: a living Ronghuo owner observes completed fire damage and arms one non-stacking permission. Requires Rule VM ABI 602."
            }
            "owner_source_fire_sha_or_huogong_dcsbronghuo" => {
                "Card-used filter: the owner uses a fire Sha or Fire Attack while Ronghuo's one-shot permission is armed. Requires Rule VM ABI 602."
            }
            "owner_source_card_resolved_has_other_target_dcsbyingmou_once_per_turn" => {
                "Card-resolved filter: once per turn, the living owner finished a card with at least one other original target and the current Yingmou branch has a legal selection. Requires Rule VM ABI 602."
            }
            "dcsbyingmou_tracked_card_resolved" => {
                "Card-resolved filter: the exact physical damage card currently sequenced by Yingmou finished resolving. Requires Rule VM ABI 602."
            }
            "owner_wounded_not_event_source" => {
                "Trigger filter: the living skill owner is wounded and is not the event source."
            }
            "owner_alive_event_player_group_wei" => {
                "Parameterized trigger filter: both players are alive and the event player's character group equals the suffix (`wei` here). Other canonical group tags may replace the suffix."
            }
            "owner_wounded_event_player_group_qun" => {
                "Parameterized trigger filter: the living skill owner is wounded and the event player's character group equals the suffix (`qun` here)."
            }
            "owner_wounded_event_player_qun" => {
                "Legacy spelling of `owner_wounded_event_player_group_qun`."
            }
            "owner_source_amount_gt_one" => {
                "Trigger filter: owner dealt damage whose amount is greater than one."
            }
            "owner_source_wounded_distance_le_one" => {
                "Trigger filter: owner is the wounded damage source and is within distance 1 of the target."
            }
            "owner_source_distance_le_one_per_damage_point" => {
                "Damage trigger filter: owner is the source and within distance 1 of the target; the trigger is offered once per damage point."
            }
            "owner_source_target_distance_gt_one" => {
                "Per-target card trigger filter: owner is the card source and the current declared target's final distance from the owner is greater than one."
            }
            "owner_source_target_not_group_wei" => {
                "Parameterized damage trigger filter: owner is the source and the damage target's character group differs from the suffix (`wei` here). Other canonical group tags may replace the suffix. Requires Rule VM ABI 498."
            }
            "owner_source_target_distance_le_one_first_damage_to_target_this_turn" => {
                "Damage trigger filter: the living owner is the source, the damaged player is within final distance one, and this is the owner's first applied damage event to that player this turn. Requires Rule VM ABI 353."
            }
            "owner_alive" => {
                "Trigger filter: fires only while the skill owner remains alive after the event."
            }
            "owner_alive_event_cards_nonempty" => {
                "Trigger filter: the skill owner is alive and the triggering event carries at least one captured physical card. Death events capture the victim's hand and equipment before cleanup. Requires Rule VM ABI 82."
            }
            "owner_alive_neighbor_gained_cards_outside_own_draw_once_per_turn" => {
                "Cards-moved filter: once per turn, a living previous/next-seat neighbor gained cards into hand outside their Draw phase and has a hand/equipment card the owner may gain. Requires Rule VM ABI 487."
            }
            "owner_first_hand_discard_has_sha_in_discard" => {
                "Cards-moved trigger filter: the living owner is discarding hand cards for the first time this turn, and at least one physical Sha from this event still remains in the shared discard pile. Requires Rule VM ABI 332."
            }
            "owner_alive_hand_below_alive_count_capped_seven" => {
                "Trigger filter: the living owner has fewer hand cards than the living-player count capped at seven."
            }
            "owner_alive_hand_below_alive_count_capped_eight" => {
                "Trigger filter: the living owner has fewer hand cards than the living-player count capped at eight. Requires Rule VM ABI 228."
            }
            "owner_is_target_first_hp_change_turn_unique_trick_majority" => {
                "HP-change trigger filter: the living owner is the event player on their first actual HP change this turn, and trick cards are the sole most numerous basic/trick/equipment category in their current hand. Supports damage, hp_lost, hp_recovered, and max_hp_changed. Requires Rule VM ABI 336."
            }
            "owner_alive_delayed_trick_target_without_pending_judge_skip" => {
                "Per-target trigger filter: the living owner is targeted by a delayed trick and has no pending judgment-phase skip. Requires Rule VM ABI 228."
            }
            "owner_source_sha_or_juedou_single_target_first_same_name_this_turn" => {
                "Per-target trigger filter: the living owner used their first logical Sha or first Duel of the current turn and specified exactly one target. Requires Rule VM ABI 229."
            }
            "owner_source_sha_damage_before_endpoint_discardable_hand_equip" => {
                "Damage-before trigger filter: the living owner is the source of logical-Sha damage to a living target and may currently discard a hand/equipment card from either endpoint. Chain-propagated Sha damage is included. Requires Rule VM ABI 230."
            }
            "owner_longdan_view_as_first_counterpart_hand_nonempty" => {
                "Card-targeted/CardResponded trigger filter: the living owner used or responded with a Sha/Shan converted by a Longdan/Fanghun-named skill, the current player is the first counterpart, and that counterpart has hand cards. Requires Rule VM ABI 231."
            }
            "owner_is_source_hand_below_alive_count_capped_seven" => {
                "Trigger filter: the owner is the event source and has fewer hand cards than the living-player count capped at seven."
            }
            "owner_alive_event_player_alive" => {
                "Trigger filter: the skill owner and event player are both alive."
            }
            "owner_alive_event_player_equip_empty" => {
                "Trigger filter: the skill owner and event player are both alive, and the event player's equipment zone is empty."
            }
            "owner_alive_event_player_from_equip_empty" => {
                "Cards-moved trigger filter: while the owner remains alive, a living event player actually lost one or more cards from their equipment zone and now has no equipment. Requires Rule VM ABI 28."
            }
            "owner_alive_sha_target_no_other_red_card_this_turn" => {
                "Per-target Sha trigger filter: while the owner and event target remain alive, no distinct red physical card other than the current card targeted this player earlier in the current turn. Requires Rule VM ABI 29."
            }
            "owner_alive_event_player_hand_empty" => {
                "Trigger filter: the skill owner and event player are both alive, and the event player's hand is empty."
            }
            "owner_alive_event_player_outside_turn_from_hand_empty" => {
                "Trigger filter: a living non-current event player lost hand cards and now has an empty hand while the skill owner remains alive."
            }
            "owner_alive_event_player_outside_turn_from_hand_empty_once" => {
                "Trigger filter: a living non-current event player lost hand cards and now has an empty hand while the skill owner remains alive; fires at most once during each current character's turn. Requires Rule VM ABI 26."
            }
            "owner_alive_event_player_turn_changed_or_became_linked" => {
                "Character-state trigger filter: while the owner is alive, fires after either turn-over direction or after the event player becomes linked; becoming unlinked does not fire. Requires Rule VM ABI 27."
            }
            "owner_alive_event_player_became_face_up_or_linked" => {
                "Character-state trigger filter: while the owner is alive, fires after the event player becomes face-up or linked; becoming face-down or unlinked does not fire. Requires Rule VM ABI 27."
            }
            "owner_turned_over" => {
                "Character-state trigger filter: the owner (event player) just completed a turn-over in either direction (face-up or face-down). Matches upstream `player: \"turnOverEnd\"`."
            }
            "owner_alive_other_wei_recovery_own_phase_once_per_turn" => {
                "Recovery trigger filter: while the owner is alive, another living wei-group player recovers HP during their own turn's phases (once per turn). Matches upstream zhushi's wei-recovery watch."
            }
            "owner_alive_global_juedou_or_red_sha_used" => {
                "Card-used trigger filter: while the owner is alive, any living player uses a duel or a red sha (owner involvement not required). Matches upstream global juedou/red-sha watch."
            }
            "owner_alive_other_global_juedou_or_red_sha_used" => {
                "Card-used trigger filter: while the owner is alive, another player (user != owner) uses a duel or a red sha."
            }
            "owner_alive_event_player_distance_le_owner_hp" => {
                "Trigger filter: the living event player is within the living owner's current HP distance from the owner."
            }
            "owner_alive_other_normal_damage_target_unlinked" => {
                "Damage-before trigger filter: a living owner observes another living, currently unlinked player about to receive normal damage. Requires Rule VM ABI 73."
            }
            "owner_alive_not_target" => {
                "Trigger filter: fires for each living owner other than the event player."
            }
            "owner_alive_not_target_hand_equip_nonempty" => {
                "Trigger filter: fires for a living owner other than the living event player while the owner has at least one hand/equipment card. Requires Rule VM ABI 144."
            }
            "owner_source_sha_or_ordinary_trick_target_wounded_has_discardable_hand_equip" => {
                "Per-target trigger filter: the living owner used a logical Sha or ordinary trick on a wounded living target and can discard one hand/equipment card. Requires Rule VM ABI 145."
            }
            "owner_source_sha_or_ordinary_trick_has_discardable_hand_equip" => {
                "Card-used trigger filter: the living owner used a logical Sha or ordinary trick and can discard one hand/equipment card. Requires Rule VM ABI 190."
            }
            "owner_source_jiu_boosted_sha_damage_skill_3365741831_enabled" => {
                "Parameterized damage trigger filter: the living owner dealt logical-Sha damage boosted by Jiu and still has the numeric-suffix skill definition effectively enabled. Requires Rule VM ABI 191."
            }
            "owner_alive_not_target_neither_unique_max_hand" => {
                "Trigger filter: the living owner and another living event player are both not the unique maximum-hand player."
            }
            "owner_alive_not_target_source_same_group" => {
                "Trigger filter: the living owner is not the event player and has the same non-empty group tag as the event source."
            }
            "owner_alive_not_target_equal_hand" => {
                "Trigger filter: fires when the living owner and another living event player have equal hand-card counts."
            }
            "owner_alive_not_target_hand_below_hp" => {
                "Trigger filter: the living owner is not the event player and has fewer hand cards than current HP."
            }
            "owner_alive_not_target_once" => {
                "Trigger filter: fires at most once per turn for each living owner other than the event player."
            }
            "owner_alive_current_or_turn_fires_below_hp" => {
                "Trigger filter: the living owner may fire up to their current HP times per turn, or without limit during their own turn."
            }
            "owner_alive_other_source_sha_twice" => {
                "Damage trigger filter: a living owner observes another living player deal logical Sha damage, at most twice per turn."
            }
            "owner_alive_card_is_sha" => {
                "Trigger filter: fires for any logical 杀 response while the skill owner remains alive."
            }
            "owner_alive_other_responded_sha_gainable" => {
                "Card-responded trigger filter: a living owner observes another player respond with logical 杀 whose physical cards are in the discard pile and can be gained. Requires Rule VM ABI 276."
            }
            "owner_targeted_by_trick_equip_empty" => {
                "Trigger filter: the skill owner was a declared trick target and their equipment zone is empty."
            }
            "owner_targeted_by_sha_hand_below_hp" => {
                "Trigger filter: the skill owner was a declared Sha target and their hand count is below their current HP."
            }
            "owner_targeted_by_sha_or_juedou_damaged_hp_le_source_once" => {
                "Trigger filter: once per turn, the damaged skill owner was a declared Sha or Juedou target and their HP is no greater than the card user's HP."
            }
            "owner_targeted_outside_turn_by_sha_or_trick_hand_empty" => {
                "Trigger filter: outside the skill owner's turn, they were a declared Sha or trick target while holding no hand cards."
            }
            "owner_targeted_hand_below_max_hp" => {
                "Trigger filter: the skill owner was a declared target and their hand count is below maximum HP."
            }
            "owner_targeted_by_other_single_target_trick" => {
                "Trigger filter: another player used a single-target trick on the skill owner."
            }
            "owner_source_or_targeted_by_juedou" => {
                "Trigger filter: the skill owner used or was a declared target of Juedou."
            }
            "owner_source_or_targeted_by_juedou_or_red_sha" => {
                "Trigger filter: the skill owner used or was a declared target of Juedou or a red Sha."
            }
            "owner_source_or_targeted_by_sha_equal_hp" => {
                "Trigger filter: in a per-target event, the skill owner is the Sha source or target and both players have equal HP."
            }
            "owner_source_or_targeted_by_other_diamond" => {
                "Trigger filter: the skill owner used/responded with a diamond card, or another player targeted them with one."
            }
            "owner_source_red" => {
                "Trigger filter: the skill owner used a red card, using the owner's effective suit."
            }
            "owner_source_alternates_color" => {
                "Trigger filter: during the owner's turn, the used card has a different color from the owner's previous used card this turn."
            }
            "owner_source_repeats_color_has_discardable_hand_equip" => {
                "Card-used trigger filter: during the owner's current turn, the used red/black card matches the immediately previous used card's effective color and the owner can discard a hand/equipment card. Requires Rule VM ABI 131."
            }
            "owner_alive_skill_bound_target_is_event_player" => {
                "Trigger filter: the living owner observes an event whose player equals this skill instance's bound target. Requires Rule VM ABI 132."
            }
            "owner_source_matches_previous_suit_or_rank" => {
                "Trigger filter: during the owner's play phase, the used card matches the previous card from that play phase by effective suit or rank."
            }
            "owner_source_ordinary_trick" => {
                "Trigger filter: the skill owner used an ordinary (non-delayed) trick card."
            }
            "owner_source_basic_or_trick" => {
                "Trigger filter: the skill owner used a basic or trick card. Requires Rule VM ABI 67."
            }
            "owner_is_target_can_compare_other" => {
                "Play-phase trigger filter: the owner is entering their Play phase and both they and at least one other living player have a hand card. Requires Rule VM ABI 68."
            }
            "owner_source_physical_ordinary_trick" => {
                "Trigger filter: the skill owner used a physical ordinary trick, excluding cards converted by view-as rules."
            }
            "owner_hand_below_lost_hp_after_relevant_change" => {
                "Trigger filter: the owner's hand is below lost HP after their HP/max HP changes or a move touches one of their hand, equipment, or judgment zones."
            }
            "owner_alive_source_first_card_ordinary_trick" => {
                "Trigger filter: a living skill owner observes a player's first used card this turn, and that card is an ordinary trick."
            }
            "owner_card_action_count_eq_attack_range" => {
                "Trigger filter: the skill owner's current-turn used-or-responded card count reaches their attack range on this event."
            }
            "owner_source_matches_last_used_suit_or_rank" => {
                "Trigger filter: the owner used a card matching their previous used card across turn boundaries by effective suit or rank."
            }
            "owner_alive_equip_source_max_equip" => {
                "Trigger filter: a living owner observes an equipment user tied for the most cards in the equipment zone after use."
            }
            "owner_is_target_event_card_sha" => {
                "Trigger filter: the skill owner is `event_player` and `event_card` is an effective Sha. For `hand_compare_finished`, this identifies the initiator's comparison card. Requires Rule VM ABI 339."
            }
            "owner_source_target_earlier_seat_hand_below_target_max_hp_once_per_turn" => {
                "Damage-before trigger filter: once per turn, the living owner is the damage source, the living target has an earlier seat, and the owner's hand is below the target's max HP. Requires Rule VM ABI 340."
            }
            "owner_alive_equip_used" => {
                "Card-used trigger filter: a living owner observes any logical equipment-card use. Requires Rule VM ABI 158."
            }
            "owner_alive_equip_used_skill_mark_below_three" => {
                "Trigger filter: a living owner observes an equipment-card use while the stable mark matching this skill definition is below three."
            }
            "owner_alive_source_alive_once_per_round" => {
                "Trigger filter: a living owner observes a sourced event at most once in the current round."
            }
            "owner_alive_source_alive_event_player_dying_once_per_round" => {
                "Trigger filter: a living owner observes a living sourced dying event while the event player remains alive and at 0 HP or below, at most once in the current round. Requires Rule VM ABI 129."
            }
            "owner_is_target_source_group_qun_discardable_hand_equip_gte_two" => {
                "Damage trigger filter: the living owner is the damaged event player, the damage source's character group is Qun, and the owner can currently discard at least two hand/equipment cards. Requires Rule VM ABI 134."
            }
            "owner_alive_other_current_source_repeats_card_name" => {
                "Trigger filter: a living owner observes another current-turn player use the same logical card name as their immediately previous card."
            }
            "owner_source_hand_basic_equals_non_basic" => {
                "Trigger filter: the skill owner used a card while their remaining hand contains equal numbers of basic and non-basic cards."
            }
            "owner_source_sha" => "Trigger filter: the skill owner used a logical Sha.",
            "owner_target_sha" => "Trigger filter: the owner took Sha-card damage (mirror of owner_source_sha).",
            "owner_source_heart_sha" => {
                "Card-use trigger filter: the living skill owner used a logical Sha whose effective suit is heart. Requires Rule VM ABI 580."
            }
            "owner_source_normal_sha" => {
                "Card-use trigger filter: the skill owner used an ordinary-nature logical Sha. Requires Rule VM ABI 45."
            }
            "owner_source_fire_sha" => {
                "Card-use trigger filter: the skill owner used a fire-nature logical Sha. Requires Rule VM ABI 45."
            }
            "owner_is_target_hand_sha_skill_pile_empty" => {
                "End-phase trigger filter: the living owner has at least one logical hand Sha and this skill definition's public expansion pile is empty. Requires Rule VM ABI 46."
            }
            "owner_alive_dying_has_skill_pile_cards" => {
                "Dying trigger filter: the living owner observes a living player at non-positive HP while this skill definition's public expansion pile is nonempty. Requires Rule VM ABI 46."
            }
            "owner_source_play_sha_other_alive_once_per_play_phase" => {
                "Damage trigger filter: during the owner's play phase, their logical Sha damaged another living player; fires at most once in that play phase."
            }
            "owner_source_jiu" => "Trigger filter: the skill owner used a logical Jiu.",
            "owner_is_target_hp_odd" => {
                "Trigger filter: owner is the living event player and has odd current HP."
            }
            "owner_is_target_hp_even_wounded" => {
                "Trigger filter: owner is the living event player, has even current HP, and is wounded."
            }
            "owner_alive_card_targets_gt_one" => {
                "Trigger filter: owner is alive and the used card declared more than one target."
            }
            "owner_alive_non_multitarget_trick_targets_gt_one" => {
                "Card-used trigger filter: a living owner observes a non-fixed-multitarget ordinary trick with at least two declared targets. Requires Rule VM ABI 308."
            }
            "owner_alive_wounded_card_targets_gt_one" => {
                "Trigger filter: owner is alive, wounded, and the used card declared more than one target. Requires Rule VM ABI 114."
            }
            "owner_source_sha_or_trick_current_play_phase" => {
                "Card-used trigger filter: the owner used a logical Sha or trick during their current Play phase. Requires Rule VM ABI 115."
            }
            "owner_source_card_has_other_target_current_play_phase_first_two" => {
                "Card-used trigger filter: fires for the first two cards the owner uses during their Play phase that declare at least one other player as a target. Requires Rule VM ABI 220."
            }
            "owner_alive_other_phase_end_has_discardable_basic_hand" => {
                "End-phase trigger filter: a living owner observes another living player's End phase and has at least one effective basic card in hand for the optional cost. Requires Rule VM ABI 178."
            }
            "owner_alive_other_phase_end_hand_nonempty" => {
                "End-phase trigger filter: a living owner observes another living player's End phase and has at least one hand card. Requires Rule VM ABI 299."
            }
            "owner_alive_other_phase_begin_both_hand_nonempty" => {
                "Turn-begin filter: owner and another living current-turn player both have hand cards. Requires Rule VM ABI 625."
            }
            "owner_alive_ddd_dongcha_pile_nonempty" => {
                "End-phase filter: owner is alive and the 洞察 skill pile is nonempty. Requires Rule VM ABI 625."
            }
            "owner_alive_prepare_player_hp_gt_owner" => {
                "Prepare-phase trigger filter: a living owner observes a living player's Prepare phase and that player's current HP is greater than the owner's. Requires Rule VM ABI 179."
            }
            "owner_alive_prepare_lord_hp_gt_owner" => {
                "Prepare-phase trigger filter: the HP-greater comparison also requires the event player to hold the Lord identity. Requires Rule VM ABI 179."
            }
            "owner_alive_other_sha_target_can_redirect_to_owner" => {
                "Per-target Sha filter: the living owner is not already targeted; another current target can discard a hand/equipment card and has the owner in that target's attack range. Requires Rule VM ABI 180."
            }
            "owner_sha_target_can_discard_redirect_to_other_in_attack_range" => {
                "Per-target Sha filter: the living owner is the current target, has a discardable hand/equipment card, and has another legal untargeted player other than the Sha user in the owner's attack range. Requires Rule VM ABI 186."
            }
            "owner_alive_gave_cards_to_other_alive" => {
                "Cards-given filter: the living owner is the attributed giver and the living recipient is another player. Requires Rule VM ABI 181."
            }
            "owner_alive_first_target_virtual_or_converted_card" => {
                "Trigger filter: a living owner observes only the first declared target of a virtual card or a card whose logical kind differs from its physical kind."
            }
            "owner_alive_targeted_by_other_single_card_discardable" => {
                "Trigger filter: a living owner is the sole target of another player's card and has at least one hand or equipment card to discard."
            }
            "owner_alive_targeted_by_trick_single_or_discardable" => {
                "Trigger filter: a living owner is targeted by a trick; multi-target tricks require at least one hand or equipment card the owner can discard."
            }
            "owner_alive_targeted_by_other_trick" => {
                "Trigger filter: a living owner is targeted by another player's logical trick card. Requires Rule VM ABI 111."
            }
            "owner_alive_targeted_by_other_matching_equip_suit" => {
                "Trigger filter: a living owner is targeted by another player's card whose raw suit matches an equipped card's effective suit."
            }
            "owner_alive_targeted_by_other" => {
                "Trigger filter: a living owner is targeted by another player's card."
            }
            "owner_alive_targeted_by_other_black_card" => {
                "Trigger filter: a living owner is targeted by another player's black card. Implements 协穆·指定 (xiemu2)."
            }
            "owner_alive_targeted_by_other_red_card" => {
                "Trigger filter: a living owner is targeted by another player's red card (mirror of the black variant). Implements the per-color targeted family (郊夏/芳华)."
            }
            "owner_alive_other_source_death" => {
                "Trigger filter: a living owner observes another player being killed and is not the killer. Implements 起乱 draw-on-death (qiluan)."
            }
            "owner_is_target_play_phase_hand_no_sha" => {
                "Trigger filter: the owner is the event player during their own play phase and has no Sha in hand. Implements 摧怀/咄咄 (cihuai/oldcihuai)."
            }
            "owner_alive_other_phase_end_player_outside_owner_range" => {
                "Trigger filter: a living owner observes another player's end phase and that event player is not in the owner's attack range. Implements 礼下·OL (ollixia)."
            }
            "owner_source_sha_target_alive_has_discardable_equip" => {
                "Trigger filter: the living owner used a Sha and the event player (a living target) has a discardable equipment card. Implements 风石 (zfengshi)."
            }
            "owner_source_card_used_targets_self" => {
                "Trigger filter: the living owner used a card and is among its targets. Implements 成都 (pxchengdu)."
            }
            "owner_source_current_phase_card_targeted_other_healthy" => {
                "Trigger filter: the living owner (current phase) targets a healthy other player with a card. Implements 廷机·OL (oltingji)."
            }
            "owner_source_first_card_suit_this_turn" => {
                "Trigger filter: the living owner used a card whose effective suit is the first of that suit this turn. Implements 文喜 (pxwenxi)."
            }
            "owner_lost_hand_now_empty" => {
                "Trigger filter: the living owner lost hand cards and their hand is now empty. Implements 平原 (pxpingyuan)."
            }
            "owner_lost_spade_two_to_nine" => {
                "Trigger filter: the living owner lost a physical spade 2-9 card from their hand. Implements 巨鹿 (pxjulu)."
            }
            "owner_min_hp" => {
                "Trigger filter: skill owner is tied for the lowest HP among living players."
            }
            "owner_is_target_not_min_hp" => {
                "Trigger filter: the living owner is the event player and is not tied for the lowest current HP. Requires Rule VM ABI 89."
            }
            "owner_is_target_has_movable_equipment" => {
                "Trigger filter: the owner is the event player and at least one equipment card can move to a living recipient other than its holder and the owner. Requires Rule VM ABI 30."
            }
            "owner_alive_used_sha_this_turn" => {
                "Trigger filter: the owner is alive and has actually used at least one logical Sha during the current character's turn. Requires Rule VM ABI 31."
            }
            "owner_target_min_hand" => {
                "Trigger filter: owner is the event player and tied for the fewest hand cards."
            }
            "owner_target_not_min_hand" => {
                "Trigger filter: owner is the event player but not tied for the fewest hand cards."
            }
            _ => return None,
        }
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
