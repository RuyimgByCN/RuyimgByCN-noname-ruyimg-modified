# 2026-10-17 T13 武将包产品路径接入（T13-NATIVE-GENERALS-PRODUCT）

**核验日期**：2026-10-17。**待做 ID**：`T13-NATIVE-GENERALS-PRODUCT`（自 T6③「装载面暂限卡牌包」）。
**性质**：产品面接线 + 测试（无规则语义/版本/golden 变化；规则 v295 / 存档 v80）。

## 结论：武将包经 `/native` 产品路径端到端接入

`crates/content-native-loader` 的 `bridge_pack` 早已把同一 `assets/cards.json` 里的 `generals` 数组一起桥接（`bridge_generals_inner(.., false)`），因此**服务端装载路径本身已能处理武将包**；缺的是产品面证据与目录计数观测。本批：

1. `NativeGameSummary` 新增 `catalog_generals` / `base_catalog_generals`（建局后目录武将数与纯标准基线），与既有 `catalog_cards` 对称——使「武将包确实并入目录」可在产品面观测。
2. 新增 `t13_generals_package_loads_via_native_path`（`apps/game-server/tests/native_product_path.rs`）：`NativeContent::open → import → installed` 发现 `wudian`（玩点论杀，namespace `upstream-noname`）→ `load` 得到 9 武将/14 实体牌 → `create_native_game` 建局，断言 `catalog_generals == base + 9` 与 `catalog_cards == base + 14`。

## 覆盖的武将包

`packages/extensions/wudian-upstream-noname`（玩点论杀；`package_id=wudian`，9 名武将：费诗/韩龙/刘馥/刘焉/留赞/田豫/西真夕红/造之仁君/杨仪 + 14 张实体牌）。同一路径同样适用于 `shaai-ow`/`shaai-xianjian`（武将包）与其余含 `generals` 段的转换包——本轮以 wudian 作为产品面样本取证。

## 命令与结果

```
cargo test --offline -p game-server --test native_product_path -- --test-threads=1
```

结果：**6 passed / 0 failed**（[game-server-native-path.log](game-server-native-path.log)），含新增 `t13_generals_package_loads_via_native_path`。`cargo test --offline -p game-server --no-fail-fast` 全绿（49 passed / 0 failed）。

## 生态与边界

- 产品路径仍以「安装包 → C0 装载器 → 原生建局 → BoundedPlanner 驱动」为主；本批不引进新通道。
- `generals` 采用「与卡牌同文件声明」的既有装载口径；纯武将包（无 cards 段）由 `bridge_pack` 的缺省空 cards 分支承载。
- 未改任何规则语义：包内容锁、目录计数在装载时确定，无版本/golden 变化。
