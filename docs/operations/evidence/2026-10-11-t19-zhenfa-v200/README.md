# 2026-10-11 T19a：国战派生判定原语 + 阵法三张（规则 v200）

任务来源：[进度主稿 T19](../../../architecture/native-progress.md)（领取锁 `e8ff0d8c`；下拆：T19a=派生判定原语+纯派生 3 阵；T19b=swapSeat+长蛇/破阵）。上游：`zhenfa/card.js`、`mode/guozhan/src/patch/player.js`（isFriendOf/inline/siege/sieged 完整语义）。

## 派生判定原语（state.rs，登记差异：构逆/野心家结盟/undist 不建模；邻居取存活者）

- `is_friend_of`：双方明置势力已知且相同（effective_faction 推导）。
- `alive_neighbors`：存活左右邻居（跳过死亡座位）。
- `inline`/`inline_with`：队列判定（连续同势力友方段）。
- `sieged`/`sieges`：被围攻=己方已知、存活左右邻互异非己方且互为友方。
- **zhenfa 启用门**：存活角色 ≥4（上游 guozhanZhenfaEnabled；九变 ≥3 未建模）。

## 阵法三张（zhenfa 包 3→6 张，CardKind 各自独立 134–136 追加）

| 阵 | 行为 | 机器形态 |
|---|---|---|
| 天覆阵 | AOE 全部大势力且有牌者各强制弃 1 张 he（目标自选） | HostMachine targets 队列+SelectCard 逐目标询问 |
| 七星阵 | 被围攻者施放：围攻者逐个弃 1 + 视为对其用杀（响应闪或 1 伤） | targets=围攻者弃牌队列、cards=用杀队列（kernel 工厂预填） |
| 鸟翔阵 | AOE 全部合法目标（异施放者势力、已知、与自身邻座同势力=连续段端点）各响应闪或 1 伤 | targets 队列+TargetChoice 询问（无闪直接伤害，机关凤同款） |

**登记差异**：七星阵视为用杀以响应闪或 1 伤简化（上游真实 useCard——虚拟用牌原语缺口同族，无加成/无额外面）；zhenfa 启用门九变 ≥3 未建模；邻居含死亡座位语义不明保守取存活。

## 验证

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p engine-native --test g1_shahai_zhenfa` | **6/6**（地敌阵 2 既有+生命周期 1+新增 3） |
| 版本回退探针（200→199 复跑 `replay_scenarios`） | 2/2 全绿（恢复后重录 14 哈希） |
| `cargo test --offline -p engine-native --no-fail-fast` | **548 通过 / 0 失败 / 1 ignore**（545+3，exit 0）——[完整日志](engine-native-tests.log) |
| `RUSTFLAGS="-D warnings" cargo check --offline -p engine-native --lib`（独立 target 目录） | exit 0 |

## 边界

- T19b：swapSeat 座位重排原语（座位环可变——PlayerId=索引的结构性重构）+ 长蛇阵/破阵诀，独立后批。
- 阵法派生判定的构逆/野心家/undist 面、九变门=3 人、邻居死亡座位语义——如需对齐随 T19b 一并登记。
