use super::*;

pub(in super::super) fn items() -> Vec<LspCompletionItem> {
    vec![LspCompletionItem {
        label: "skill".into(),
        kind: 14, // Keyword
        detail: "Define a skill".into(),
        insert_text: "skill ${1:name}(${2:def_id}) {\n\t$0\n}".into(),
    }]
}
