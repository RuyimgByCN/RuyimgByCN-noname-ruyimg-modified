# L04 牌堆八项整包（2026-10-17）

核验日期：2026-10-17。规则 **v295**，存档 **v80**。未改规则版本，未重录 golden。liuli 牌堆 npack 字节未改，未重算哈希。

本目录证明来源 **L04 cardpile**（noname-liuli 仓）。与已计入的 U03、U06、L06、L07、L23、U04 一起，整包计数为 **7/37**。其余 30 个来源未验收。U03 使用同一 `scale_plan`，本目录不把它再计一次。L05 不计入。U01 末焱切片和 U02 仍未验收。G0 清单和 2026-10-08 生命周期不单独构成这次上调。

## 代码

倍率在 `crates/engine-native/src/content/pile.rs` 的 `scale_plan`。比例条目与 noname 仓相同，所以同一倍率、同一牌堆长度得到同一追加快照。商店副本的 `source` 是 `noname-liuli`，不是 `noname`。点数是调用方给出的 1..=13 序列，不是上游 `Math.random`。113 张比例牌仍是目录材料。

安装包是 `packages/extensions/cardpile-upstream-liuli`（`upstream-liuli/cardpile@1.0.0`）。`logic_hash` 前四字节为 `141,217,158,178`，`asset_hash` 前四字节为 `5,133,68,112`，比例表 `source_hash` 前四字节为 `223,149,103,52`。目标平台字符串辅助是 `crates/engine-native-wasm/src/lib.rs` 的 `cardpile_scale`，测试读的是 liuli 仓 `pile.json`。

## 命令

工作目录 `noname-next`。

```
cargo test --offline -p engine-native --test l04_cardpile_eight -- --test-threads=1
cargo test --offline -p engine-native-wasm --lib -- --test-threads=1 liuli_cardpile_pack_json_enters_wasm_helper
```

结果：八项测试 **1 passed, 0 failed**（[host-test.log](host-test.log)，0.06s）。wasm 辅助 **1 passed, 0 failed**（[wasm-helper.log](wasm-helper.log)，7 filtered，0.00s）。两条命令退出码都是 0。未跑 engine-native 全量。未重跑 U03、U04、U06、L06、L07、L23。

## 八项

`l04_eight_dimension_from_fresh_install` 从 `import_extensions` 安装真实包。比例表读取的是商店副本 `packages/upstream-liuli__cardpile/1.0.0/assets/pile.json`。

| 项 | 正例 | 反例 |
|---|---|---|
| 功能 | 来源字段 `noname-liuli`，`total` 160。十二项倍率全为 1、牌堆 207：杀方片 6、闪方片 18、铁索梅花 4，追加合计 113。杀倍率 2：杀方片 33、火杀红桃 8。牌堆 600：杀方片 53、铁索梅花 35。比例包牌名以 `clp_sha_` 开头，与倍率追加一起建局后目录长度为基准加 226 | 牌堆 160 和 100 为 `NoAddition`。缺 `jiu` 为 `Unconfigured`。点数 14 拒绝 |
| 配置 | 商店 `content_hash` 等于 liuli npack 字节的哈希，版本 `1.0.0`，依赖为空 | 未知键 `nope` 拒绝。负倍率拒绝。`manifest.enabled=false` 的 `cardpile-liuli-disabled` 被跳过 |
| 依赖 | npack `dependencies` 为空 | 激活 `9.9.9` 为 `NotFound`，两仓都保持 `1.0.0`。包 id `../cardpile` 为 `Package`。声明 `missing-base` 的 `1.0.1` 为 `Invalid` |
| 组合 | 同一次导入仍安装 `upstream-noname/cardpile`。两仓追加快照相同，但 noname 副本的来源字段是 `noname`，文本不含 `noname-liuli`。计算倍率不改内容锁 | 失败的缺依赖安装不撤掉 noname。升级和回滚 liuli 后 noname 仍是 `1.0.0` |
| 保存恢复 | `snapshot_scale` / `restore_scale` 往返。重开商店后仍是 `1.0.0`，来源字段仍是 `noname-liuli`，同一追加快照 | `added` 后 `sha diamond 0` 拒绝 |
| 安装升级 | npack `2.0.0`（重算 logic 与 asset 哈希，从 liuli 目录 `install_with_assets`）激活后 `rollback` 回到 `1.0.0` | 失败的缺依赖安装不改写激活集 |
| 目标平台 | `liuli_cardpile_pack_json_enters_wasm_helper` 读仓库 liuli `pile.json`：牌堆 207 为 `added:113:sha:diamond:6`，牌堆 600 为 `added:1005:sha:diamond:53` | 文本含 `source` 为 `noname-liuli`。牌堆 160 为 `none`。只有 `sha` 为 `unconfigured`。未知键为 `error:factors:` |
| 规则差异 | 追加牌名含 `_scale_huosha_`，`kind.key()` 为 `huosha` | 点数不是 `Math.random`。比例条目与 U03 相同，不因此把两行合成一次验收。`playpackconfig` 未逐项对照。不记为上游等价 |
