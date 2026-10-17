# L06 五行八项整包（2026-10-17）

核验日期：2026-10-17。规则 **v295**，存档 **v80**。未改规则版本，未重录 golden。

本目录只证明来源 **L06 wuxing**。写入当时与同日 U06 一起为 **2/37**。其后的总账以进度主稿为准。

## 代码

`packages/extensions/wuxing-upstream-liuli/assets/cards.json` 的五行盘补上上游 `skills:['wuxingpan_skill']` 对应的 `equip_skills`。随后用 `rehash_cards_asset` 与 `rehash_npack` 重算该资产的 `source_hash` 和包 `asset_hash`。`logic_hash` 前四字节仍为 `132,94,166,250`。

## 命令

工作目录 `noname-next`。

```
cargo test --offline -p engine-native --test g1_wuxing l06_eight_dimension_from_fresh_install -- --nocapture
cargo test --offline -p engine-native --test g1_wuxing
cargo test --offline -p engine-native-wasm --lib wuxing_pack_json_enters_wasm_helper -- --nocapture
```

结果：八项测试 **1 passed, 0 failed**（[host-test.log](host-test.log)）。同文件其余 4 个既有测试一并 **5 passed, 0 failed**。wasm 辅助 **1 passed, 0 failed**（[wasm-helper.log](wasm-helper.log)）。未跑 engine-native 全量。

## 八项

`l06_eight_dimension_from_fresh_install` 从 `import_extensions` 安装真实包 `upstream-liuli/wuxing@1.0.0`。

| 项 | 正例 | 反例 |
|---|---|---|
| 功能 | 目录多 1 张 `wxl_` 牌；角色有属性；牌堆属性非空；装备五行盘后宝物栏有牌且 `wuxingpan_skill` 实例在位 | `PlayCard { card: u32::MAX }` 为 `Rejected` |
| 配置 | `assets/wuxing.json` 的 `denominator` 为 3，激活版本 `1.0.0` | `manifest.enabled=false` 不安装，跳过清单含 `wuxing-disabled` |
| 依赖 | npack `dependencies` 为空，`content_hash` 等于真实 npack | 激活 `9.9.9` 为 `NotFound`；包 id `../wuxing` 为 `Package`；缺依赖的 `1.0.1` 为 `Invalid`，激活集仍为 `1.0.0` |
| 组合 | 与欢乐卡牌同时建局，`wxl_` 1 张且 `hnl_` 7 张 | 五行单包存档用双包目录恢复为 `RuleVersionMismatch` |
| 保存恢复 | 存档字节往返后同包恢复，扩展牌仍为 1 | 标准目录恢复为 `RuleVersionMismatch` |
| 安装升级 | npack `2.0.0` 激活后 `rollback` 回到 `1.0.0`；升级前存档仍可用原包恢复，技能实例仍在 | 包 `version + 1` 拒绝升级前存档 |
| 目标平台 | 宿主往返见上；`wuxing_pack_json_enters_wasm_helper` 读取仓库 cards.json，前缀计数 1，同包可恢复 | 空包与错版文本均含「恢复失败」 |
| 规则差异 | 同种子两次建局的 0 号属性相同；生克映射为 `host:shengke_check` | 这不是上游 `Math.random` 逐局等价。被克弃牌仍是确定性首张。联机提前返回、录像和帮助 HTML 未迁入。不记为上游等价 |

2026-10-08 生命周期句不单独构成这次上调。
