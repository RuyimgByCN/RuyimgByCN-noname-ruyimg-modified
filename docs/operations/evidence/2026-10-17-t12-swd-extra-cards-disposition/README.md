# 2026-10-17 T12 swd 包内「多余 13 张」处置（T12-SWD-EXTRA-CARDS-DISPOSITION）

**核验日期**：2026-10-17。**待做 ID**：`T12-SWD-EXTRA-CARDS-DISPOSITION`（自 [2026-10-15 swd 卡牌对账](../2026-10-15-t12-swd-cards-audit/README.md)「包内多余 13 张处置待决」）。
**规则** v295 / **存档** v80（本批不改语义、不加版本）。

## 结论

2026-10-15 对账把包内相对上游 `list.js` 多出的 13 张列为「多余」。核对后分两类处置，**均不需要删除**：

1. **7 张为灵剑池（`hslingjian`），有意建模**：`swd_xuanfengzhiren1`、`swd_zhongxinghujia1`、`swd_xingtigaizao1`、`swd_shijianhuisu1`、`swd_jinjilengdong1`、`swd_shengxiuhaojiao1`、`swd_yinmilichang1`（[U05 机关术批，规则 v198](../2026-10-11-t12-jiguanshu-v198/README.md)）。这是 swd 自造改造牌类型 `CardKind::Hslingjian`（`catalog.rs:444`），作为机关术合成与机关妖术补偿的池承载，上游 `card.js` 有定义、只是不在 `list.js` 的活动行内。**保留**，不是多余。
2. **6 类错位旧值已随补切批勘误消除**：`hufu`（现 spade1+heart1）、`zhiluxiaohu`（heart3+diamond4）、`mujiaren`（club7+heart6+diamond11）、`shentoumianju`、`xingjunyan`、`zhufangshenshi`（spade4+spade8）、`yangpijuan`（spade9+club7）——对账后 89 张（`+89` 计数守卫见 `g1_shahai_swd.rs`）。

## 守护

新增 `swd_extra_cards_disposition_guard`（`crates/engine-native/tests/g1_shahai_swd.rs`）：从真实 `assets/cards.json` 桥接后断言包内 **89** 张、`CardKind::Hslingjian` **7** 张。与既有 `base_cards + 89` 计数断言共同锁定包数据。

命令：`cargo test --offline -p engine-native --test g1_shahai_swd swd_extra_cards_disposition_guard`
结果：**1 passed / 0 failed**。

## 影响

包数据/规则/存档/golden 不变；仅添加守护测试。台账 U05 swd 的卡牌面「多余 13 张待决」项关闭。
