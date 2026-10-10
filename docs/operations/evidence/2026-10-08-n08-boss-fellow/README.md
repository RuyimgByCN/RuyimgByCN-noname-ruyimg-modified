# P2/N08 模式入口批三-b：Boss addFellow 座位扩张（规则 v84，2026-10-08）

主稿 P2/N08「模式入口扩充」批三-b：Boss 模式助战中途入队——座位向量
扩张面。前置：[Boss 批三-a](../2026-10-08-n08-boss-mode/README.md)。

## 上游语义核验（源码为准，mode/boss.js）

`game.addFellow(position, name, "zoominanim")` → `fellow.directgain(
get.cards(4))` → `fellow.side = true`、`identity = "zhong"`——助战为
**boss 方**（checkResult 语义下不算挑战者）；直授 4 张手牌。

## 改动（分开陈述）

- **代码已实现**：`GameMode::Boss` 增 `fellows: Vec<PlayerId>` 助战
  座位表（存档 tag5 内 len+pid 序列、哈希 push_players、快照座位界
  检查）；`debug_add_fellow(hp)` = players 向量队尾 push 新玩家 +
  牌堆直授 4 张 + fellows 登记；DeathMachine 胜负臂挑战者判定排除
  fellow（fellow 存活时挑战者全灭即 boss 胜）。
- **登记差异**：入队位置固定**队尾**——原生玩家 id=下标、按位置插入
  会移动既有座位号（不可接受）；上游可按 position 插入。

## 验收口径

- **回归已加入**：`boss_mode_add_fellow_seat_expansion`——入队座位=
  队尾、直授 4 张手牌、座位表扩张（players.len()=4）、fellows 登记、
  存档往返（座位表/fellows/助歌手牌一致）、fellow 不算挑战者（挑战者
  全灭且 fellow 存活 → `Boss { boss_won: true }`；濒死桃强制应答
  守卫）。
- **全量通过**：engine-native **387 passed / 0 failed / 1 ignored**
  （唯一 ignore=写轮眼规模诊断探针），单线程
  `cargo test --offline -p engine-native -- --test-threads=1`；
  依赖包 content-runtime + engine-ai + content-native-loader
  **62 passed / 0 failed**，DEPS_EXIT=0。明细见 [test-logs.txt](test-logs.txt)。
- **golden**：版本 83→84 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（标准场景无 fellow，纯版本位移）。
- **上游等价边界**：入队/直授/阵营归属等价；入队位置固定队尾（登记
  差异）；boss 技能包/loop 变体仍登记。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（11 文件，相对 noname-next 根）。
