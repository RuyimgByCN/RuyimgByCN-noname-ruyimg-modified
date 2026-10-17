# L22 段位制裁八项整包（2026-10-17）

核验日期：2026-10-17。规则 **v295**，存档 **v80**。未改规则版本，未重录 golden。未改 U04、L05 的 npack 字节。

本目录证明来源 **L22 段位制裁**（noname-liuli 仓）。与已计入的 U03、U06、L06、L07、L23、U04、L04、L05 一起，整包计数为 **9/37**。其余 28 个来源未验收。L12 如真似幻没有安装，不计入。U01 末焱切片和 U02 仍未验收。G0 清单不单独构成这次上调。

## 代码

力度、已知牌、重排、透视和烧条秒数在 `crates/content-native-loader/src/elo.rs`。牌堆第 0 张是顶。分数由调用方给出，代替 `get.value` 和判定函数。`eloPlayers` 不筛将池。烧条只返回秒数，`chat_refused` 为真，`slept` 为假。重排不把牌标成已知。不挂标准局的摸牌，也不新增宿主行为标签。

安装包是 `packages/extensions/elo-sanction-upstream-liuli`（`upstream-liuli/elo-sanction@1.0.0`）。`logic_hash` 前四字节为 `136,39,197,88`，`asset_hash` 前四字节为 `150,246,150,120`，配置 `source_hash` 前四字节为 `93,10,76,101`。目标平台字符串辅助是 `crates/engine-native-wasm/src/lib.rs` 的 `elo_apply`。负数段位表示没有排位函数。上游提交 `8485dbb29ff16f9b258a766b1a17ef75f883aade`。

## 命令

工作目录 `noname-next`。

```
cargo test --offline -p engine-native --test l22_elo_eight -- --test-threads=1
cargo test --offline -p engine-native-wasm --lib -- --test-threads=1 elo_pack_json_enters_wasm_helper
```

结果：八项测试 **1 passed, 0 failed**（[host-test.log](host-test.log)，0.06s）。wasm 辅助 **1 passed, 0 failed**（[wasm-helper.log](wasm-helper.log)，9 filtered，0.00s）。两条命令退出码都是 0。未跑 engine-native 全量。

## 八项

`l22_eight_dimension_from_fresh_install` 从 `import_extensions` 安装真实包。配置读取的是商店副本 `packages/upstream-liuli__elo-sanction/1.0.0/assets/config.json`。

| 项 | 正例 | 反例 |
|---|---|---|
| 功能 | 来源 `noname-liuli`。仁济 4/1、爽局 3/1、简易 2/1、关闭 1/0、困难 2/-1、炼狱 3/-1、逆天 4/-1。未知档 4/0。排位在线、非太虚、无段位函数为 2/1。段位 0 为 4/1，1 为 3/1，3 为 1/0，5 为 3/-1，6 为 4/-1。样例牌堆己方偏向顶是 3，对方偏向顶是 2；偏向 -1 时相反。已知牌留在槽内。并列在偏向 -1 时反转。力度 3 抽样 1 才翻开：己方偏向只翻友方，对方偏向只翻敌方。重度在线、随机 0.5 为 2.25 秒 | 偏向 0、力度不大于 1、牌堆不足 2 张、摸牌张数小于 1 时不重排。自己、被控、没有 `game.me`、力度小于 3 不翻开。力度 3 抽样 0 不翻开，抽样不是 0 或 1 拒绝。离线烧条为 0。随机不在 0 到 1 拒绝。将池在 `eloPlayers` 真假下都是原名单 |
| 配置 | 商店 `content_hash` 等于本包 npack 字节的哈希，版本 `1.0.0`，依赖为空。`eloPlayers` 为真，`eloDifficulty` 为 `paiwei`，`timeKiller` 为 `none`。文本含上游路径 `noname/extension/段位制裁/extension.js` 与提交 `8485dbb29ff16f9b258a766b1a17ef75f883aade` | 空 `source`、带空格的 `source`、`eloPlayers` 不是布尔、`eloDifficulty` 不是字符串都拒绝。`manifest.enabled=false` 的 `elo-sanction-disabled` 被跳过 |
| 依赖 | npack `dependencies` 为空 | 激活 `9.9.9` 为 `NotFound`。包 id `../elo-sanction` 为 `Package`。声明 `missing-base` 的 `1.0.1` 为 `Invalid` |
| 组合 | 同一次导入仍安装 `upstream-liuli/coin@1.0.0` 与 `upstream-noname/coin@1.0.0`，内容哈希仍等于各自未改的 npack。公式不改内容锁。不安装如真似幻 | 失败的缺依赖安装不撤掉 liuli 金币。升级和回滚段位制裁后两个金币包仍是 `1.0.0` |
| 保存恢复 | 开局标记后的快照能恢复同一 id、已知和分数。洗牌清掉已知。塞入且 `no_mark` 为假才打已知；为真时不打，也不清掉已有标记。重开商店后仍是 `1.0.0`，来源仍是 `noname-liuli` | 已知标记改成数字拒绝。分数字符串拒绝。快照不是数组拒绝 |
| 安装升级 | npack `2.0.0`（重算 logic 与 asset 哈希，从本包目录 `install_with_assets`）激活后 `rollback` 回到 `1.0.0` | 失败的缺依赖安装不改写激活集 |
| 目标平台 | `elo_pack_json_enters_wasm_helper` 读仓库 `config.json`。无段位函数、在线、身份模式为 `diff:2 eh:1`，己方顶 3、对方顶 2，将池 `caocao,liubei`，烧条 `chat:refused`。段位 6 为 `diff:4 eh:-1`，对方顶 3 且敌方可见。逆天离线仍是 4/-1。重度在线 0.5 为 `delay:2.25` | 离线排位和太虚幻境为 `diff:1 eh:0`，顶仍是 2。空来源、随机 1.5、爽局抽样 2 都以 `error:` 开头。`eloPlayers` 改成假时将池不变 |
| 规则差异 | 难度表、排位公式、已知槽和透视条件与上游 `extension.js` 的读取一致 | 将池不筛。烧条不睡眠、不说台词。分数不是 `get.value` 或判定函数。偷看张数按整数相乘。重排函数本身不读 `game.me`，透视仍然拒绝没有 `game.me`。DOM、`isEloKnown` 节点和祖安台词没有迁入。不记为上游等价，也不把 L12 计入 |
