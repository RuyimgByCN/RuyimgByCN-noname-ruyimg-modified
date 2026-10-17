# L05 金币八项整包（2026-10-17）

核验日期：2026-10-17。规则 **v295**，存档 **v80**。未改规则版本，未重录 golden。noname 金币 npack 字节未改，未重算哈希。

本目录证明来源 **L05 coin**（noname-liuli 仓）。与已计入的 U03、U06、L06、L07、L23、U04、L04 一起，整包计数为 **8/37**。其余 29 个来源未验收。U04 使用同一金币服务，本目录不把它再计一次。U01 末焱切片和 U02 仍未验收。G0 清单不单独构成这次上调。

## 代码

余额、商店、下注和终局公式在 `crates/content-native-loader/src/coin.rs`。价格和针点与 noname 仓相同，所以同一门槛得到同一数额。商店副本的 `source` 是 `noname-liuli`。noname 包的配置没有 `source` 键，解析缺省为 `noname`。画布、音效、提示、CSS 和粒子没有迁入。`unit_random` 由调用方给出，不是上游 `Math.random`。

安装包是 `packages/extensions/coin-upstream-liuli`（`upstream-liuli/coin@1.0.0`）。`logic_hash` 前四字节为 `125,154,116,150`，`asset_hash` 前四字节为 `25,5,51,38`，配置 `source_hash` 前四字节为 `124,8,91,25`。目标平台字符串辅助是 `crates/engine-native-wasm/src/lib.rs` 的 `coin_apply`，测试读的是 liuli 仓 `config.json`。成功时在结果后附加 `effect:refused`。

## 命令

工作目录 `noname-next`。

```
cargo test --offline -p engine-native --test l05_coin_eight -- --test-threads=1
cargo test --offline -p engine-native-wasm --lib -- --test-threads=1 liuli_coin_pack_json_enters_wasm_helper
```

结果：八项测试 **1 passed, 0 failed**（[host-test.log](host-test.log)，0.05s）。wasm 辅助 **1 passed, 0 failed**（[wasm-helper.log](wasm-helper.log)，8 filtered，0.00s）。两条命令退出码都是 0。未跑 engine-native 全量。未把 U04 证据日志重写为这次结果。同一次编译里 `u04_eight_dimension_from_fresh_install` 也通过，日志不并入本目录。

## 八项

`l05_eight_dimension_from_fresh_install` 从 `import_extensions` 安装真实包。配置读取的是商店副本 `packages/upstream-liuli__coin/1.0.0/assets/config.json`。

| 项 | 正例 | 反例 |
|---|---|---|
| 功能 | 来源字段 `noname-liuli`。烟花 50、下雪 20、闪烁 10。余额 50 买下雪后为 30，可停再开。身份主公 8 人、随机 0、不下注为 36；反贼同座次为 27；其他模式胜、随机 0.5、不下注为 30；败为 10；下注后其他模式胜为 payout 40、返还 20、余额 60；战棋战斗加成且败、随机 0 为 2 | 余额 0 买闪烁为 `NotEnough`。重复购买为 `AlreadyOwned`。未购买不能开始。国战势力人数 0 为 `EmptyPopulation`。随机 1.5 为 `BadRandom` |
| 配置 | 商店 `content_hash` 等于 liuli npack 字节的哈希，版本 `1.0.0`，依赖为空。文本含上游路径 `noname/extension/coin/extension.js` 与提交 `8485dbb29ff16f9b258a766b1a17ef75f883aade` | 空 `source`、带空格的 `source`、未知显示值拒绝。`manifest.enabled=false` 的 `coin-liuli-disabled` 被跳过 |
| 依赖 | npack `dependencies` 为空 | 激活 `9.9.9` 为 `NotFound`。包 id `../coin` 为 `Package`。声明 `missing-base` 的 `1.0.1` 为 `Invalid` |
| 组合 | 同一次导入仍安装 `upstream-noname/coin@1.0.0`，其 `content_hash` 仍等于未改的 noname npack。noname 配置不含 `noname-liuli`，解析来源为 `noname`。两份配置字节不同。结算不改内容锁 | 失败的缺依赖安装不撤掉 noname。升级和回滚 liuli 后 noname 仍是 `1.0.0` |
| 保存恢复 | 快照恢复后来源仍是 `noname-liuli`，局内闪烁仍在跑，余额按持久规则回到购买前的 0。常下雪重载后视为已购且不再扣费，来源字段保留。重开商店后仍是 `1.0.0`，来源仍是 `noname-liuli` | 商品名 `nope` 拒绝。来源改成 `bad token` 拒绝。局内购买除余额、`snowFall`、`coinSnowSize` 外不保留 |
| 安装升级 | npack `2.0.0`（重算 logic 与 asset 哈希，从 liuli 目录 `install_with_assets`）激活后 `rollback` 回到 `1.0.0` | 失败的缺依赖安装不改写激活集 |
| 目标平台 | `liuli_coin_pack_json_enters_wasm_helper` 读仓库 liuli `config.json`。余额改成 30 后买闪烁含 `balance:20`、`source:noname-liuli`、`effect:refused`。身份 `settle:win:0` 含 `balance:36` 与 `source:noname-liuli` | 余额 0 买闪烁以 `error:` 开头。`bet;bet` 以 `error:` 开头。战棋主公买星云以 `error:` 开头。空来源以 `error:` 开头 |
| 规则差异 | 数额与 U04 的针点相同，包身份和配置字节不同 | 画布、音效、提示、CSS 和粒子没有迁入。`unit_random` 不是 `Math.random`。同一经济不因此把两行合成一次验收。不记为上游等价 |
