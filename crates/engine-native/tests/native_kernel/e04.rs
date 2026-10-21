use super::*;

/// E04（ADR 0776 修订）：冲突返回**可处理错误**——与基础目录同名、
/// 以及扩展包内重复名，均不再 panic。
#[test]
fn e04_extension_conflict_detection() {
    use engine_native::content::extension::{
        example_extension, merge_extension_cards, ExtensionError,
    };
    let ext = example_extension();
    let e0 = Engine::new(1, 2);
    let base_cards: Vec<engine_native::catalog::CardDef> = e0
        .catalog()
        .cards
        .iter()
        .map(|c| {
            engine_native::catalog::CardDef::basic(c.name, c.kind).with_suit_rank(c.suit, c.rank)
        })
        .collect();
    // 正常合并（无冲突）
    let merged = merge_extension_cards(base_cards.clone(), &ext.cards).expect("合并应成功");
    assert_eq!(merged.len(), base_cards.len() + ext.card_count());
    // 与基础目录同名 → 可处理错误
    let conflict_card = vec![engine_native::catalog::CardDef::basic("sha", CardKind::Sha)];
    match merge_extension_cards(base_cards.clone(), &conflict_card) {
        Err(ExtensionError::DuplicateCard { name }) => assert_eq!(name, "sha"),
        other => panic!("与基础目录同名应返回冲突错误: {other:?}"),
    }
    // 包内重复名 → 可处理错误（此前放行）
    let dup_in_pack = vec![
        engine_native::catalog::CardDef::basic("ext_x", CardKind::Sha),
        engine_native::catalog::CardDef::basic("ext_x", CardKind::Shan),
    ];
    match merge_extension_cards(base_cards.clone(), &dup_in_pack) {
        Err(ExtensionError::DuplicateCard { name }) => assert_eq!(name, "ext_x"),
        other => panic!("包内重复名应返回冲突错误: {other:?}"),
    }
}
