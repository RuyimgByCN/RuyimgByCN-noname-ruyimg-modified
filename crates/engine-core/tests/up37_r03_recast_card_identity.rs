//! R03 重铸过滤依赖稳定卡牌 def 身份：tiesuo 可重铸，sha 不可。

#[test]
fn tiesuo_def_is_stable_and_recastable() {
    let tiesuo = engine_core::content::card_def_id_for_name("tiesuo");
    let sha = engine_core::content::card_def_id_for_name("sha");
    assert_ne!(tiesuo, sha);
    assert!(engine_core::content::is_recastable_card_def(tiesuo));
    assert!(!engine_core::content::is_recastable_card_def(sha));
}
