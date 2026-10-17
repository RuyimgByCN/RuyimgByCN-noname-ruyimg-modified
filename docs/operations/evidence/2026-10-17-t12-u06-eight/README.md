# U06 欢乐卡牌八项整包（2026-10-17）

核验日期：2026-10-17。规则 `STANDARD_RULESET_VERSION` **v295**，存档 `ARCHIVE_FORMAT_VERSION` **v80**。未改规则版本，未重录 golden。

本目录只证明来源 **U06 欢乐卡牌** 从真实安装走过八项正反例。写入当时总账成为 **1/37**。同日 L06 另计后曾为 2/37。其后的总账以进度主稿为准。苦无仍是登记差异，不是上游等价。2026-10-08 生命周期句、G0–G5 通道和 T13 HTTP 不单独构成这次上调。

## 命令

工作目录 `noname-next`。

```
cargo test --offline -p engine-native --test g1_huanle u06_eight_dimension_from_fresh_install -- --nocapture
cargo test --offline -p engine-native-wasm --lib huanle_pack_json_enters_wasm_helper -- --nocapture
```

结果：宿主 **1 passed, 0 failed**（[host-test.log](host-test.log)）；wasm 字符串辅助 **1 passed, 0 failed**（[wasm-helper.log](wasm-helper.log)）。未跑 engine-native 全量，进度主稿「原生全量」仍是 2026-10-16 的 833/0/1。

## 八项

测试 `u06_eight_dimension_from_fresh_install` 调用 `import_extensions` 安装 `packages/extensions`，再读激活集、内容锁和安装根上的 `cards.json`。

| 项 | 正例 | 反例 |
|---|---|---|
| 功能 | 安装副本桥接后目录为标准牌数 + 7，牌名前缀 `hnl_`，装备苦无写入技能实例 | `PlayCard { card: u32::MAX }` 为 `Rejected` |
| 配置 | 激活槽 `upstream-noname/huanle-cards` 为 `1.0.0`，内容锁同版本 | `manifest.enabled=false` 的目录不安装，跳过清单含目录名 |
| 依赖 | npack `dependencies` 为空；`content_hash` 等于真实 npack 字节 | 激活 `9.9.9` 为 `NotFound`；包 id `../huanle` 为 `Package`；声明缺失依赖的 `1.0.1` 激活为 `Invalid`（`missing dependency`），激活集仍为 `1.0.0` |
| 组合 | 欢乐与五行 `wuxing-upstream-liuli` 同时建局，牌数多于单包，`wxl_` 1 张且 `hnl_` 仍为 7 | 欢乐单包存档用双包目录恢复为 `RuleVersionMismatch` |
| 保存恢复 | 存档字节经 `encode_archive`/`decode_archive` 后用原包恢复，扩展牌仍为 7 | 标准目录 `Engine::restore` 为 `RuleVersionMismatch` |
| 安装升级 | npack `2.0.0` 经 `install_with_assets` 激活，`rollback` 回到 `1.0.0`；升级前存档仍可用原包恢复 | 包 `version + 1` 恢复升级前存档为 `RuleVersionMismatch` |
| 目标平台 | 宿主存档往返见上；wasm 侧 `huanle_pack_json_enters_wasm_helper` 读取仓库 `cards.json`，经 `engine_from_packs` / `engine_from_archive` / `prefix_count` 得到前缀 7，同包可恢复 | 空包与 `"version": 1` 换成 `"version": 2` 的文本均含「恢复失败」。宿主测试不调用 `wasm_bindgen` 方法 |
| 规则差异 | 源 JSON 映射为 `host:sha_damage_force_discard`。雷杀命中后目标首张手牌被弃、后一张保留 | 这不是上游「目标自选 + he 区 + notLink」。不记为上游等价 |

仓库 `cards.json` 与 `package.npack.json` 未被改写。升级用的 `2.0.0` 只存在于测试临时商店。
