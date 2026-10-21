use super::*;

pub(in super::super) fn items() -> Vec<LspCompletionItem> {
    vec![
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
    ]
}
