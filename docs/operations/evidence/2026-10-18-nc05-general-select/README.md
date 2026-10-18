# NC05 产品入口：正式选将/装配（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC05**（[方案 §NC05](../../../architecture/native-convergence-plan.md#nc05)）第一阶段切片：**产品级选将入口**
- 规则版本 v321 / 存档 v105（无规则与格式变化）
- 结论：**入口已实现 + 4 项回归通过**；三类纵向案例的全生命周期闭环待续

## 1 动机（NC05 硬要求）

「样本经作者包产物 → 独立安装 → 启用 → **正式选将/开局** → 真实交互 → 存读档 → 升级/回退/卸载闭环……**无需**重编译客户端、修改核心枚举或调用 `debug_grant` 才能运行」。

此前原生只有建局随机装配与 `debug_*` 助手：扩展武将的技能只能靠 `debug_grant` 授予，**缺少产品级选将入口**。

## 2 实现（`crates/engine-native/src/engine/general_select.rs`）

| 接口 | 语义 |
|---|---|
| `Engine::general_candidates()` | 该目录全部可装配武将（标准 + Boss + 扩展包），顺序稳定，只读 |
| `Engine::assign_general(seat, def)` | 产品入口：校验（终局 `MatchOver` / 越界 `UnknownActor` / 未知武将 `NoSuchGeneral`）→ 清旧技 → 写主将槽 → 按武将技能集与体力上限重装（与建局/国战明置共用 `apply_general_assembly`，**授予语义不分叉**）→ 回满血 |
| `Engine::assign_general_named(seat, name)` | 按名字装配（名字不在目录 → `NoSuchGeneral`） |

## 3 回归（新增 `crates/engine-native/tests/nc05_general_select.rs`，4 项）

| 验收项 | 用例 | 结果 |
|---|---|---|
| 扩展包武将装配后**技能集来自武将表**（非 `debug_grant`）、体力上限按武将设定、装配后回满 | `assign_general_grants_pack_skills` | 通过 |
| 校验：未知武将 / 越界座位 / 未知名 → 明确拒绝且**状态不变** | `assign_general_validates_inputs` | 通过 |
| 候选列表含扩展包武将；按名装配结果与请求一致 | `general_candidates_include_pack_generals` | 通过 |
| 装配后对局可继续推进（不破坏结算栈） | `match_continues_after_selection` | 通过 |

## 4 验证结果

```
cargo test --offline -p engine-native --test nc05_general_select
cargo test --offline -p engine-native --tests --no-fail-fast
python3 -B scripts/check-source-lines.py
```

| 检查 | 结果 | 证据 |
|---|---|---|
| NC05 选将入口定向 | **4 passed / 0 failed** | [targeted.log](targeted.log) |
| engine-native 测试面（`--tests`） | **1116 passed / 0 failed / 2 ignored** | [engine-native-tests.log](engine-native-tests.log) |
| 行数门禁 | 8232 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |
| 全量含 examples / Gate 1b | **本轮被并行任务的在飞文件阻断**：`examples/nc12_bench.rs`（NC12 子 agent 正在编写）编译错误 `E0061`（8 参数/传 7）→ 待其提交后复跑 | [gate1b-all-targets.log](gate1b-all-targets.log) |

## 5 未完成 / 待办（NC05）

- [ ] 三类纵向案例的全生命周期闭环（重规则/嵌套选择、战役/跨局档案、皮肤/UI 组合）——本入口是其中「正式选将」一环
- [ ] 赛前大厅阶段未建模（当前装配发生在对局内；上游选将在开局前）
- [ ] 多武将明置选择（国战双将槽）与选将信息隔离（他人不可见候选）
- [ ] 前端接线：选将 UI 调用 `general_candidates`/`assign_general`（NC03 前端切片进行中）
- [ ] 独立安装 → 启用 → 交互 → 存读档 → 升级/回退/卸载的**产品入口**端到端（含 game-server/HTTP）
