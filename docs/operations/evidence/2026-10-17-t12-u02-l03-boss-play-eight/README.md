# U02 / L03 boss「play 包」八项整包（2026-10-17）

核验日期：2026-10-17。规则 **v295**，存档 **v80**。未改规则版本，未重录 golden。未改其它来源的 npack 字节。

本目录证明来源 **U02（noname boss）** 与 **L03（noname-liuli boss）**——两来源是同一 `game.import("play", …)` 的挑战/剑阁武将包装配扩展，语义一致，仅包键/标签/别名/允许旗标不同。与已计入的 16 个来源一起，整包计数为 **18/37**。其余 19 个来源未验收。

## 代码

挑战/剑阁武将包装配、体力归一（Infinity）、禁 AI、翻译默认与 intro 别名、`isBossAllowed` 过滤与 `versus_ladder` 排除在 `crates/content-native-loader/src/boss_playpack.rs`。不读写 localStorage、不加载模式、不画界面、不播音。

安装包 `packages/extensions/boss-play-upstream-noname`（`upstream-noname/boss-play@1.0.0`）与 `packages/extensions/boss-play-upstream-liuli`（`upstream-liuli/boss-play@1.0.0`），由 `crates/content-runtime/examples/gen_boss_packages.rs` 生成。目标平台字符串辅助是 `crates/engine-native-wasm/src/lib.rs` 的 `boss_play_apply`。

| 来源 | namespace | 上游提交 | logic_hash 前四 | asset_hash 前四 | source_hash 前四 |
|---|---|---|---|---|---|
| U02 | `upstream-noname` | `df2682e66af9930fe02861a25cb6f2cd2ef6ae36` | 29,209,96,172 | 92,185,113,227 | 93,14,246,28 |
| L03 | `upstream-liuli` | `8485dbb29ff16f9b258a766b1a17ef75f883aade` | 142,215,163,173 | 134,69,173,45 | 143,53,233,202 |

## 命令

```
cargo test --offline -p engine-native --test boss_play_eight -- --test-threads=8
cargo test --offline -p engine-native-wasm --lib boss_play_pack_json_enters_wasm_helper -- --test-threads=8
```

结果：八项测试 **2 passed, 0 failed**（[host-test.log](host-test.log)，0.12s）。wasm 辅助 **1 passed, 0 failed**（[wasm-helper.log](wasm-helper.log)，17 filtered，0.01s）。两条命令退出码都是 0。未跑 engine-native 全量（收口门禁另记）。

## 八项（两来源同套断言）

| 项 | 正例 | 反例 |
|---|---|---|
| 功能 | storage 解析挑战 4/剑阁 3。体力归一：`4` 保留；`"3"`（数字字符串）→ Infinity；`"3/4"` 挑战保留分数、剑阁归一 Infinity。装配 `characters = boss + jiange`。模式门控：`tafang` 早退；`boss` 不装挑战包；`versus+jiange` 装 boss 块但剑阁走 else 分支。禁 AI 时 7 名入 forbidai。翻译默认键被默认值覆盖、storage 键按 `||=` 合并；intro 别名 15（noname）/8（liuli）。`isBossAllowed` 与 `bossallowed` 标签都解析为允许，过滤后剑阁剩 1 | 非法 storage（非 JSON/数组）视为空。`versus+jiange+status=jiange` 无覆盖。`versus_ladder` 从技能表排除。liuli 追加 `mode:boss`/`mode:versus` 标签，noname 不加 |
| 配置 | 商店 `content_hash` 等于本包 npack 字节哈希，版本 `1.0.0`，依赖为空，武将/卡牌索引为空。两来源包键/翻译键/别名数不同 | 商店副本与仓库源逐字节一致；文本不含 `localStorage`、`eval(`、`fs.` |
| 依赖 | npack `dependencies` 为空 | 激活 `9.9.9` 为 `NotFound` |
| 组合 | 同一次导入两来源 boss-play 都在且内容哈希各自独立 | 未安装版本拒绝 |
| 保存恢复 | 重开商店后仍是 `1.0.0`；纯函数不改内容锁。资产目录只有 `config.json` | 装配前后内容锁哈希不变 |
| 安装升级 | npack `2.0.0`（重算 logic 与 asset 哈希，从本包目录 `install_with_assets`）激活后 `rollback` 回到 `1.0.0` | 未安装版本 `NotFound` |
| 目标平台 | `boss_play_pack_json_enters_wasm_helper` 读两仓真实 `config.json`。装配计数、`versus_ladder` 排除、`tafang` 早退都走同一字符串辅助 | 非 JSON 以 `error:BadConfig` 开头。不调用 `wasm_bindgen` |
| 规则差异 | 模式门控顺序（tafang → boss 块 → 剑阁/else）、体力归一分支、禁 AI、翻译默认覆盖与 `\|\|=` 合并、intro 别名表、允许旗标，与两仓 `extension.js` 里仍会执行的路径一致 | 不读写 localStorage、不加载模式、不画界面、不播音、不请求网络。`arenaReady` 的异步模式加载只取确定性面（允许过滤 + `versus_ladder` 排除），不记为上游等价 |

## 登记差异

1. `arenaReady` 的 `game.loadModeAsync('boss'/'versus')` 与技能表实际合并未迁入——本模块只覆盖确定性面（`isBossAllowed` 过滤 + `versus_ladder` 排除）。
2. 挑战/剑阁武将的实际可玩性依赖 Boss 模式（`mode_boss`/`jiangeboss` 数据面，属核心模式而非本来源）；本来源是 play-pack 配置装配。
3. 图片路径（`image/mode/boss/character/*.jpg`）等表现面未迁入。
