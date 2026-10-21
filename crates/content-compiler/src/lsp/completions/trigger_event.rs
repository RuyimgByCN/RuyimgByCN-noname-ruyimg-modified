use super::*;

pub(in super::super) fn items() -> Vec<LspCompletionItem> {
    vec![
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
    ]
}
