# NC09/NC03 桥接：原生视图 → 产品协议视图（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC09**（[方案 §NC09](../../../architecture/native-convergence-plan.md#nc09)）+ **NC03** 投影面
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**唯一映射点已建立 + 3 项回归通过**；联机房间接入原生权威规则待续

## 1 动机

联机房间与客户端使用 `engine-api` 的 `ObserverSnapshot`/`LegalAction`（旧线 `engine-core`
产出），原生线使用 `engine-native::view::PlayerView` + 内核权威动作查询。二者此前**没有
映射层**——NC09 要求「本地、WASM Worker 和服务端使用同一原生会话契约」。

## 2 实现（`apps/game-server/src/native_view.rs`，唯一映射点）

| 接口 | 语义 |
|---|---|
| `observer_snapshot_for(engine, role, seat)` | 原生 `view_for` → 协议 `ObserverSnapshot`：修订号/`state_hash` 逐字节带上；`Player` 用座位投影、`Spectator`/`Referee` 用无座位投影；`SeatView.hand_count` **只给数量**；技能按名字回查目录 def id 映射为 `SkillView` |
| `legal_actions_for(engine, seat)` | 内核 `legal_actions`（同源校验）→ 协议 `LegalAction`：`NextPhase`→`EndPhase`、`PlayCard`、`Answer`→`ResolveChoice`（含取消）；`ActivateSkill`/`RevealGeneral` **不伪造映射**（候选未展开这些形态，登记待办） |

## 3 回归（新增 `apps/game-server/tests/nc09_native_view_bridge.rs`，3 项）

| 验收项 | 判据 | 结果 |
|---|---|---|
| 投影不泄漏手牌内容 | 座位投影与观战投影的 `hand_count` 与内核一致；协议视图不含手牌 id | 通过 |
| 摘要/修订一致 | `snapshot.state_hash == engine.state_hash()`、`revision` 一致 | 通过 |
| 权威动作同源 | `legal_actions_for` 的候选数与内核 `legal_actions` **一致**（不丢项不造项），含 `EndPhase` 候选 | 通过 |
| 越界座位 | 返回空候选、不 panic、不伪造 | 通过 |

## 4 验证结果

```
cargo test --offline -p game-server --test nc09_native_view_bridge
cargo test --offline -p game-server --no-fail-fast
python3 -B scripts/check-source-lines.py
```

| 检查 | 结果 | 证据 |
|---|---|---|
| 桥接定向 | **3 passed / 0 failed** | [targeted.log](targeted.log) |
| game-server 全量 | **64 passed / 0 failed**（61 → +3） | [game-server-tests.log](game-server-tests.log) |
| 行数门禁 | 8247 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 5 未完成 / 待办（NC09）

- [ ] **房间接入原生权威**：`room.rs` 仍用 `engine_core::Engine` + `project_player`/`engine_ai::legal_actions`；
      本桥接是替换前置，房间循环/重连/快照/录像的产品接线未做
- [ ] **`MatchOutcome` 映射**：原生 `outcome` 只有胜者座位，协议为身份局语义枚举；
      当前终局也返回 `InProgress`（**不伪造**），需按对局模式做真实映射
- [ ] `SkillView` 的 `enabled`/`use_count`/`active`/`view_as`/`locked` 目前为占位值，
      需从原生技能实例与目录定义真实取值
- [ ] `known_cards` 为空：协议侧的授权可见牌（自己手牌/公开牌）映射未做
- [ ] `ActivateSkill`/`RevealGeneral` 候选未展开（NC10 待办）
- [ ] 录像与跳转、真实客户端渲染、平台档位取证
