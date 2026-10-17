# U03 牌堆八项整包（2026-10-17）

核验日期：2026-10-17。规则 **v295**，存档 **v80**。未改规则版本，未重录 golden。牌堆 npack 字节未改，未重算哈希。

本目录证明来源 **U03 cardpile**（noname 仓）。与已计入的 U06、L06、L07、L23 一起，整包计数为 **5/37**。其余 32 个来源未验收。台账顺序上的 U01（3D精选：上游 34 将、64 个顶层技能，原生包只有刘冲，精彀已映射、末焱 `mapping` 为空）和 U02（boss：非数字体力被上游写成 Infinity，加载器拒绝；引擎里的 `BOSS_GENERALS` 名册不是该来源的模式规则）本次检查后仍未验收，不因本目录计入。L04 使用同一 `scale_plan`，本目录只把它当作组合对照，不单独计入。G0 清单、2026-10-08 生命周期和 G5 通道不单独构成这次上调。

## 代码

倍率在 `crates/engine-native/src/content/pile.rs` 的 `scale_plan`。`getn = round(count * factor)`，`num` 为各 `getn` 之和，`dn = num * (deck_len - total) / (total - num)`，正无穷或大于 1000 时截到 1000；`dn > 0` 时每个花色桶 `n = round(dn * getn / num)`，零张的桶不出现。缺任一倍率或倍率非有限数时为 `Unconfigured`，不追加。负倍率由 `parse_pile_factors` 拒绝。点数是调用方给出的 1..=13 序列，不是上游 `Math.random`。`extension_pack_from_pile` 的 113 张比例牌仍是 ADR 0776 的目录材料；缺配置时上游追加零张，二者不是同一件事。

倍率解析在 `crates/content-native-loader/src/lib.rs` 的 `parse_pile_factors`。允许的键是 `guohe`、`huosha`、`jiu`、`leisha`、`nanman`、`sha`、`shan`、`shunshou`、`tao`、`tiesuo`、`wanjian`、`wuxie`。缺键留给 `scale_plan` 判为未配置。

安装包是 `packages/extensions/cardpile-upstream-noname`（`upstream-noname/cardpile@1.0.0`）。`logic_hash` 前四字节为 `131,170,129,196`，`asset_hash` 前四字节为 `128,188,216,204`。目标平台字符串辅助是 `crates/engine-native-wasm/src/lib.rs` 的 `cardpile_scale`，不是设备外壳。

## 命令

工作目录 `noname-next`。

```
cargo test --offline -p engine-native --test u03_cardpile_eight -- --test-threads=1 u03_eight_dimension_from_fresh_install
cargo test --offline -p engine-native-wasm --lib -- --test-threads=1 cardpile_pack_json_enters_wasm_helper
cargo test --offline -p engine-native --lib scale_plan_follows_upstream_formula -- --test-threads=1
cargo test --offline -p engine-native --test g1_cardpile -- --test-threads=1
```

结果：八项测试 **1 passed, 0 failed**（[host-test.log](host-test.log)，0.05s）。wasm 辅助 **1 passed, 0 failed**（[wasm-helper.log](wasm-helper.log)，5 filtered，0.00s）。公式单测 **1 passed, 0 failed**（[unit-scale.log](unit-scale.log)，10 filtered，0.00s）。既有 `g1_cardpile` **5 passed, 0 failed**（[g1-cardpile.log](g1-cardpile.log)，0.11s；比例表仍是 113 张）。未跑 engine-native 全量。未新增 `packages/extensions` 目录，未重跑 U06、L06、L07、L23。

## 八项

`u03_eight_dimension_from_fresh_install` 从 `import_extensions` 安装真实包。比例表读取的是商店副本 `packages/upstream-noname__cardpile/1.0.0/assets/pile.json`，不是 `extension_root`。

| 项 | 正例 | 反例 |
|---|---|---|
| 功能 | 十二项倍率全为 1、牌堆 207：杀方片 6、闪方片 18、铁索梅花 4，追加合计 113。杀倍率 2：杀方片 33、火杀红桃 8。牌堆 600：杀方片 53、铁索梅花 35 | 牌堆 160 和 100 为 `NoAddition`。缺 `jiu` 为 `Unconfigured`。点数 14 拒绝 |
| 配置 | 十二个允许键解析为有限非负数。商店 `content_hash` 等于真实 npack 字节的哈希，版本 `1.0.0` | 未知键 `nope` 拒绝。负倍率拒绝。`manifest.enabled=false` 的 `cardpile-disabled` 被跳过 |
| 依赖 | npack `dependencies` 为空 | 激活 `9.9.9` 为 `NotFound`，两仓都保持 `1.0.0`。包 id `../cardpile` 为 `Package`。声明 `missing-base` 的 `1.0.1` 为 `Invalid` |
| 组合 | 同一次导入安装 `upstream-liuli/cardpile`。两仓同一比例表得到同一追加快照。计算倍率不改内容锁。比例包与追加包一起建局，目录长度为基准加 226 | 失败的缺依赖安装不撤掉 liuli。回滚 noname 后 liuli 仍是 `1.0.0` |
| 保存恢复 | `snapshot_scale` / `restore_scale` 往返。重开商店后仍是 `1.0.0`，同一追加快照 | `added` 后 `sha diamond 0` 拒绝 |
| 安装升级 | npack `2.0.0`（重算 logic 与 asset 哈希，从真实目录 `install_with_assets`）激活后 `rollback` 回到 `1.0.0` | 失败的缺依赖安装不改写激活集 |
| 目标平台 | `cardpile_pack_json_enters_wasm_helper` 读仓库 `pile.json`：牌堆 207 为 `added:113:sha:diamond:6`，牌堆 600 为 `added:1005:sha:diamond:53` | 牌堆 160 为 `none`。只有 `sha` 为 `unconfigured`。未知键为 `error:factors:`。坏的比例 JSON 为 `error:pile:` |
| 规则差异 | 追加牌名含 `_scale_huosha_`，`kind.key()` 为 `huosha`。花色按 club、diamond、heart、spade 排序，只改变生成顺序 | 点数不是 `Math.random`。未配置时不追加，113 张比例牌不是上游的默认追加。`playpackconfig` 未逐项对照。不记为上游等价 |
