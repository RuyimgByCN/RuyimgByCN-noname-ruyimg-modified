use super::*;

/// E05：扩展卡牌在真实对局中生效——扩展闪电牌作为判定牌使用。
#[test]
fn e05_extension_card_in_game() {
    use engine_native::content::extension::{example_extension, merge_extension_cards};
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
    let merged = merge_extension_cards(base_cards, &ext.cards).expect("合并应成功");
    // 验证扩展卡牌在合并目录中存在且类别正确
    let found = merged.iter().find(|c| c.name == "shandian_red");
    assert!(found.is_some(), "扩展卡牌应在合并目录中");
    assert_eq!(
        found.unwrap().kind,
        engine_native::catalog::CardKind::ShanDian
    );
    assert_eq!(found.unwrap().suit, engine_native::catalog::Suit::Heart);
}
