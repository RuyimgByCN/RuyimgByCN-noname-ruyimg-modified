# U04 金币八项整包（2026-10-17）

核验日期：2026-10-17。规则 **v295**，存档 **v80**。未改规则版本，未重录 golden。

本目录证明来源 **U04 coin**（noname 仓）。与已计入的 U03、U06、L06、L07、L23 一起，整包计数为 **6/37**。其余 31 个来源未验收。L05 使用同一金币经济，本目录不把它计入。U01 末焱切片和 U02 仍未验收。G0 清单不单独构成这次上调。

## 代码

余额、商店、下注和终局公式在 `crates/content-native-loader/src/coin.rs`。它不创建画布，不播放音效，不弹提示，不注入 CSS，也不生成粒子。`coin_canvas_playpackconfig` 只被记录。`unit_random` 由调用方给出，范围是 0 到 1，不是上游 `Math.random`。`additional_reward` 与 `coin_coeff` 是请求字段，缺省为 0 和空。重载是返回一份配置再 `open_coin_session`，不是页面刷新。

门槛：录像、联机，以及战棋且为主公时，没有可持久的金币按钮。联机和战棋主公在公式前返回 `Inactive`。录像仍计算 payout，但不写入余额。下注只在非在线且阶段未开始时允许，固定扣 20，不检查余额，因此可以为负；第二次为 `AlreadyBet`。阶段开始或在线后为 `BetClosed`。

商店：烟花 50、下雪 20、星云 10、闪烁 10。余额不足为 `NotEnough`，已拥有为 `AlreadyOwned`，两者都不改状态。局内购买除余额、`snowFall` 和 `coinSnowSize` 外不写入可重启配置。`snowFall` 为真时，重载后下雪视为已购买且不再扣费。

终局：`coeff = unit_random * 0.4 + 0.8`。胜时若已下注，池先加 10 并稍后返还 20，再加 20 和 `additional_reward`。败时加成为 10。身份：主公、忠臣、明忠在不强化时为 20、强化时为 10，反贼为 8 或 16，内奸为 40，再乘 `(存活+死亡)/8`。国战：野心家为 8，否则 `5/势力人数`，人数 0 且非野心家为 `EmptyPopulation`，再乘座次数。对决：有友方人数时为 `5 * (存活+友方)`，不用死亡数。战棋和其他模式加成为 10。战棋且 `chess_combat_extra` 时加成改为 2，覆盖胜败。然后 `池 += 加成 * coeff`，可选再乘 `coin_coeff`，非有限数拒绝，`ceil` 为 i64。可持久时把 payout 与下注返还加进余额。

安装包是 `packages/extensions/coin-upstream-noname`（`upstream-noname/coin@1.0.0`）。`logic_hash` 前四字节为 `225,61,18,44`，`asset_hash` 前四字节为 `136,133,28,110`，配置 `source_hash` 前四字节为 `35,40,148,244`。目标平台字符串辅助是 `crates/engine-native-wasm/src/lib.rs` 的 `coin_apply`。成功时在结果后附加 `effect:refused`。

## 命令

工作目录 `noname-next`。

```
cargo test --offline -p engine-native --test u04_coin_eight -- --test-threads=1
cargo test --offline -p engine-native-wasm --lib -- --test-threads=1 coin_pack_json_enters_wasm_helper
```

结果：八项测试 **1 passed, 0 failed**（[host-test.log](host-test.log)，0.05s）。wasm 辅助 **1 passed, 0 failed**（[wasm-helper.log](wasm-helper.log)，6 filtered，0.00s）。两条命令退出码都是 0。未跑 engine-native 全量。未重跑 U03、U06、L06、L07、L23。新增目录是 `packages/extensions/coin-upstream-noname`；八项测试从该目录经 `import_extensions` 安装。

## 八项

`u04_eight_dimension_from_fresh_install` 从 `import_extensions` 安装真实包。配置读取的是商店副本 `packages/upstream-noname__coin/1.0.0/assets/config.json`。

| 项 | 正例 | 反例 |
|---|---|---|
| 功能 | 余额 0 时先加 50，买下雪后余额 30，拥有并运行；停止后再开始。常下雪重载后仍拥有下雪且余额仍是 30。下注从 20 扣到 0。其他模式胜、随机 0.5：payout 40、返还 20、余额 60。身份主公 8 人、随机 0：36。反贼同条件：27。其他模式败、随机 0.5：10。战棋战斗加成、败、随机 0：2 | 余额 0 买闪烁为 `NotEnough {0,10}`。重复买下雪为 `AlreadyOwned`。余额 30 买烟花为 `NotEnough {30,50}`。开始未拥有的星云为 `NotOwned`。第二次下注为 `AlreadyBet`，余额仍是 0。国战势力人数 0 为 `EmptyPopulation`。随机 1.5 为 `BadRandom`。阶段开始或在线为 `BetClosed` |
| 配置 | 商店副本为余额 0、显示 `icon`、画布关、常下雪关、雪尺寸大。`content_hash` 等于真实 npack 字节的哈希，版本 `1.0.0` | 显示 `wide` 为 `BadConfig`，信息含 `unknown`。`coin` 写成 1.5 拒绝。`manifest.enabled=false` 的 `coin-disabled` 被跳过且不安装 |
| 依赖 | npack `dependencies` 为空 | 激活 `9.9.9` 为 `NotFound`。包 id `../coin` 为 `Package`，信息含 `invalid package_id`。声明 `missing-base` 的 `1.0.1` 为 `Invalid`，信息含 `missing dependency` |
| 组合 | 同一次导入仍激活欢乐卡牌 `1.0.0`。结算不改变内容锁。空临时目录里没有画布文件 | 失败的缺依赖安装不撤掉欢乐卡牌。回滚后欢乐卡牌仍是 `1.0.0`。战棋主公购买和结算为 `Inactive`，余额保持 0。联机结算为 `Inactive` |
| 保存恢复 | 加 10 并买闪烁后的快照恢复为余额 0、运行闪烁。重开商店后仍是 `1.0.0`，配置余额 0、显示 `icon`。`persist_config` 再打开保留余额 30，并丢掉局内购买 | 快照里的 `blink` 换成 `nope` 拒绝。录像 `change_coin` 为 `Inactive`；其他模式胜的 payout 为 30，`persisted` 为假，余额保持 0 |
| 安装升级 | npack `2.0.0`（重算 logic 与 asset 哈希，从真实目录 `install_with_assets`）激活后 `rollback` 回到 `1.0.0` | 失败的缺依赖安装不改写激活集 |
| 目标平台 | `coin_pack_json_enters_wasm_helper` 读仓库 `config.json`。余额改成 30 后买闪烁含 `balance:20`、`owned:blink`、`running:blink`、`effect:refused`。身份 `settle:win:0` 含 `balance:36` | 余额 0 买闪烁以 `error:` 开头。`bet;bet` 以 `error:` 开头。战棋主公买星云以 `error:` 开头。空配置以 `error:` 开头。门槛 `browser` 以 `error:gate:` 开头 |
| 规则差异 | 显示只接受 `text` 和 `icon`。雪尺寸切换在已拥有下雪时于大、小之间翻转 | 画布、音效、提示、CSS 和粒子未迁入。`unit_random` 不是 `Math.random`。重载不是页面刷新。录像仍算出 payout。L05 不计入。不记为上游等价 |
