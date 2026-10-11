# 2026-10-11 T19b：swapSeat 座位环原语 + 长蛇阵（规则 v201，存档 v46）+ 破阵诀登记

任务来源：[进度主稿 T19](../../../architecture/native-progress.md)（T19b）；开工 HEAD `a57cabfa`（领取锁）。上游：`mode/guozhan/src/patch/player.js`（swapSeat/座位环）、`zhenfa/card.js`（长蛇阵/破阵诀）、`zhenfa/list.js`（张数权威）。

## 原语（ADR 0781）

`GameStateCore.seat_ring: Vec<PlayerId>`（显式座位环，建局恒等 0..n）：
- `swap_seat(a, b)` 交换环位置（PlayerId 身份不变，区域/装配全随身份）。
- 座位语义全经 `ring_pos`/`ring_next`/`ring_prev` 访问器（环长度不符退恒等）；`+1 % n` 直算禁止。
- 下游适配：`next_alive`/`seat_order_from`/`alive_neighbors`/`inline(_with)`/kernel Boss 重整巡席。
- 存档 v46：GameStateCore 尾部 `seat_ring`（len+pid，恒等亦显式编码，自校验）；v45 字节被版本门禁拒绝。

## 长蛇阵（♣1/♦1 各 1 张，规则 v201）

队列中（`inline`）→ 全队列（含自己）各摸 1；否则与座位距离最近的友方 `swap_seat` 成队列（上游 absolute 距离排序同口径，`seat_distance` 即环上最短）。本体入弃堆。

## 破阵诀——登记「上游未上架」

上游 list.js 中破阵诀 4 条目**全部注释**（无活跃实体）——无实体可转，保持 deferred 并登记该事实（非原生缺口；若上游上架随批转换，机器臂已就绪）。

## 验证

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p engine-native --test g1_shahai_zhenfa` | **6/6**（新增：换座成队列[环变化断言]+入队列判定；入队列后全队列摸 1[施放者净 0]；环不变量=每 id 恰一次） |
| 版本回退探针（201/46→200/45 复跑 `replay_scenarios`） | 2/2 全绿（恢复后重录 14 哈希） |
| `cargo test --offline -p engine-native --no-fail-fast` | **549 通过 / 0 失败 / 1 ignore**（548+1，exit 0）——[完整日志](engine-native-tests.log) |
| `RUSTFLAGS="-D warnings" cargo check --offline -p engine-native --lib`（独立 target 目录） | exit 0（零警告） |

## 边界

- swapSeat 第 4 参（保留相对方位等 UI 语义）不建模（登记差异）。
- 死亡座位在环中保留、阵法判定经存活过滤——与上游 getNext 的差异 T19a 已登记。
- 嘲讽/冷冻/潜行等状态判定沿用既有钩（座位邻接经环访问器自动跟随换座）。
