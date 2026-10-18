# NC05/NC09 正式选将的 WASM 导出面（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC05**（[方案 §NC05](../../../architecture/native-convergence-plan.md#nc05)）+ **NC09** 客户端面
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**选将导出面已落地 + 2 项回归通过**；前端适配层接线由并行子 agent 承接（进行中）

## 1 动机

NC05 要求扩展经「**正式选将/开局**」进入对局（不得依赖 `debug_grant`）。内核侧已有
`general_candidates`/`assign_general`（2026-10-18 早前切片），但 **WASM 导出面没有**——
客户端（Web/桌面 worker）无法选将，即「正式选将」在产品入口不可达。

## 2 实现（`crates/engine-native-wasm/src/lib.rs`）

| 面 | 接口 | 语义 |
|---|---|---|
| 宿主侧（非 wasm-bindgen，供集成测试与内部复用） | `general_candidates_json_for(&Engine)` | `{"count":N,"candidates":[{"def","name","skills"}]}` |
| 宿主侧 | `assign_general_for(&mut Engine, seat, name)` | 成功 → `{"seat","name","revision","state_hash"}`；座位越界/未知武将 → **明确报错** |
| wasm-bindgen 导出 | `NativeEngine::general_candidates_json()` | 客户端选将 UI 数据源 |
| wasm-bindgen 导出 | `NativeEngine::assign_general(seat, name)` | 正式选将（失败返回 `JsValue` 错误） |

两者**共用同一实现路径**（导出只是薄封装），避免导出面与内核行为分叉。

## 3 回归（新增 `crates/engine-native-wasm/tests/nc05_general_select_export.rs`，2 项）

| 验收项 | 判据 | 结果 |
|---|---|---|
| 候选导出 | 候选非空且含扩展包武将（`nc05_hero`） | 通过 |
| 选将导出 | 成功后该武将技能集**在册**（来自武将表）；未知武将/越界座位**明确报错**；技能缺失时选将**报错而非静默** | 通过 |

> 过程中修正了测试包 JSON 的 schema（`generals[].faction`/`gender` 必填、`skills[].mapping` 必填）——
> 首次尝试因字段缺失被装载器明确拒绝（`MissingField`），说明扩展包校验按预期工作。

## 4 验证结果

```
cargo test --offline -p engine-native-wasm --test nc05_general_select_export
cargo test --offline -p engine-native-wasm --no-fail-fast
python3 -B scripts/check-source-lines.py
```

| 检查 | 结果 | 证据 |
|---|---|---|
| 选将导出定向 | **2 passed / 0 failed** | [targeted.log](targeted.log) |
| `engine-native-wasm` 全量 | **31 passed / 0 failed**（29 → +2） | [wasm-tests.log](wasm-tests.log) |
| 行数门禁 | 8255 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 5 未完成 / 待办

- [ ] **前端适配层接线**（子 agent 进行中）：`generalCandidates()`/`chooseGeneral(seat,name)` + 畸形返回显式报错 + 旧产物缺导出时显式报错（不静默降级）
- [ ] 选将的**信息隔离**：候选是否应限制为「本座位可选池」（当前导出全目录候选）
- [ ] 赛前大厅阶段（原生建局即开局；选将发生在开局后）
- [ ] 选将 UI 的真实客户端渲染与平台档位
