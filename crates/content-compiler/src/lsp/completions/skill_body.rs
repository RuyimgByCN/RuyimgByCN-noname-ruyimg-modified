use super::*;

pub(in super::super) fn items() -> Vec<LspCompletionItem> {
    vec![
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
    ]
}
