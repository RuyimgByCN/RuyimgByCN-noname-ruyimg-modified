//! L12 如真似幻装载器：罗马数字与随机文案表。

use content_native_loader::{roman, sanguo_text, tip, SANGUO_TEXTS, TIPS};

#[test]
fn roman_covers_one_to_six() {
    assert_eq!(roman(1), Some("Ⅰ"));
    assert_eq!(roman(2), Some("Ⅱ"));
    assert_eq!(roman(3), Some("Ⅲ"));
    assert_eq!(roman(4), Some("Ⅳ"));
    assert_eq!(roman(5), Some("Ⅴ"));
    assert_eq!(roman(6), Some("Ⅵ"));
    assert_eq!(roman(0), None);
    assert_eq!(roman(7), None);
    assert_eq!(roman(999), None);
}

#[test]
fn text_tables_match_upstream_counts() {
    assert_eq!(SANGUO_TEXTS.len(), 11, "上游 sanguoTexts 11 条");
    assert_eq!(TIPS.len(), 4, "上游 tips 4 条");
    assert!(SANGUO_TEXTS[0].contains("三国杀"));
    assert!(TIPS.iter().any(|t| t.contains("万箭齐发")));
}

#[test]
fn picks_are_deterministic_and_wrap() {
    assert_eq!(sanguo_text(0), Some(SANGUO_TEXTS[0]));
    assert_eq!(sanguo_text(11), Some(SANGUO_TEXTS[0]), "取模回绕");
    assert_eq!(sanguo_text(10), Some(SANGUO_TEXTS[10]));
    assert_eq!(tip(0), Some(TIPS[0]));
    assert_eq!(tip(4), Some(TIPS[0]), "取模回绕");
    // 确定性：同 roll 同结果
    assert_eq!(sanguo_text(3), sanguo_text(3));
}
