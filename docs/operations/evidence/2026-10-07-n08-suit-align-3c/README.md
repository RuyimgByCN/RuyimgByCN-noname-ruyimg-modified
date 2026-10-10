# P2/N08 批③-c：存量牌堆花色点数对齐上游 standard list（规则 v74，2026-10-07）

主稿 P2/N08「牌表逐张对照」收尾批：对齐段 79 张由原生循环花色改为上游
`card/standard.js` `list` 逐张落位（花色+点数+数量三对齐）。前置：
[批③-b⑨ 方天画戟](../2026-10-07-n08-fangtian-3b9/README.md)。

## 对齐范围（分开陈述）

- **已实现**：对齐段 79 张（杀 30/闪 15/桃 8/南蛮 3/决斗 3/顺牵 5/过拆 6/
  无懈 4/乐 3/闪电 2）逐张上游落位。闪电 ♠1 与 ♥12 同名，后者命名
  `shandian_safe` 防目录重名。**上游口径下闪 15 张（♥2×2 ♥13 ♦2×2 ♦3–11）
  与桃 8 张（♥3–9 ♥12 ♦12）全红**——黑闪/黑桃自此不存在。
- **登记差异（不立项）**：原生扩充段 25 张保留原生取值——±马 6（无花色）、
  酒 ♠3/♣9、雷杀 ♠4/♣5、泛型武器 3（射程 2/2/3）、八卦防具 2、火杀 4、
  铁索 4、万箭 ♥1（与上游同值）、兵粮 ♣3；理由：G1 cardpile/规则测试前置
  （v30 起），命名/军争属性差异已在批② 定案登记。
- 批①新牌种 9 张（v62/v63）与命名装备 11 张（v64–v73）此前已按上游落位，
  本批未动。张数不变：**124 张**。

## 版本门禁与 golden

- `STANDARD_RULESET_VERSION` 73→**74**（版本入 canonical_hash）。
- 牌堆花色点数分布变化 → 同 seed 洗牌序不同 → **14 golden 全部重录**
  （[重录前 14 场景全 mismatch 证据](test-logs.txt)）；场景脚本中途断言
  全部通过、仅终态哈希比较失败，语义无变化——非「只改期望值消除失败」。

## 反例与级联修复

- **`ganning_qixi_black_card_as_dismantle` 死循环（已修）**：测试循环
  `debug_place(0,"shan")` 直到取到黑色闪；v74 闪全红后，
  `debug_place` 在 deck/discard 耗尽后从任意手牌回收——把刚放回玩家 0
  手中的同一张红闪再取出来，永续循环（进程 sample 顶栈
  `Vec::index`/`Iter::position` 100% CPU）。修复：改取黑雷杀 ♠4
  （`debug_move_named_card_to_hand`，唯一命名、确定性），保持
  「黑色非拆牌当拆桥」转化语义。同批排查其余 3 处花色敏感循环
  （鬼才红闪/武圣红桃桃/国色方块闪）在 v74 下均可终止，未改。
- 过程中另清理一个遗留自旋进程（修复前旧二进制残留，非当前代码）。

## 验收口径（分开陈述）

- **代码已实现**：对齐段逐张落位（`standard_catalog_parts`）。
- **回归已加入**：`standard_catalog_matches_declared_inventory` +
  `defective_kinds_hold_declared_counts`（A0 守护，DECLARED 表 124 行同步）。
- **定向通过**：`standard_catalog_inventory` 2/2、`replay_scenarios` 2/2。
- **全量通过**：engine-native **382 passed / 0 failed / 1 ignored**
  （唯一 ignore=写轮眼规模诊断探针），单线程
  `cargo test --offline -p engine-native -- --test-threads=1`，EXIT=0；
  明细见 [test-logs.txt](test-logs.txt)。
- **上游等价边界**：本批仅达成「对齐段花色点数与上游逐张一致」；
  原生扩充段 25 张花色为原生取值（登记差异）；不构成整包规则等价证明，
  逐技能语义对照仍按主稿待做。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（7 文件，相对 noname-next 根）。
