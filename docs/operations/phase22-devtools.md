# 阶段 22：开发工具链（§35）

> 对应架构计划 v2.0 §35（工具链）。这些工具不是附属功能，而是大规模内容
> 迁移和长期维护的必要组成。

## 新增 crate

`crates/devtools` — 所有检查器/分析器/验证器实现为可测试的库函数。

## §35 工具清单 → 实现

| §35 工具 | 模块 | 状态 | 之前状态 |
|---|---|---|---|
| Headless Engine CLI | `apps/engine-cli` | ✅ 已有 | ✅ |
| **Rule Timeline Inspector** | `timeline` | ✅ 新增 | 未实现 |
| **Query Explanation Viewer** | `query_viewer` | ✅ 新增 | 未实现 |
| Trigger Window Inspector | `timeline` (内含) | ✅ 新增 | 未实现 |
| **State Diff Viewer** | `state_diff` | ✅ 新增 | 未实现 |
| **State Hash Divergence Bisector** | `bisector` | ✅ 新增 | 未实现 |
| **Replay Minimizer** | `minimizer` | ✅ 新增 | 未实现 |
| **Content Package Test Runner** | `package_runner` | ✅ 新增 | 未实现 |
| **VM Fuel Profiler** | `fuel_profiler` | ✅ 新增 | 未实现 |
| Skill / Package Performance Profiler | `benchmark-runner` | ✅ 已有 | ✅ |
| **Observer Leak Test Harness** | `leak_harness` | ✅ 新增 | 未实现 |
| Snapshot / WAL Recovery Simulator | `fault_injection` (阶段 17) | ✅ 已有 | ✅ |
| Legacy Analyzer | `crates/legacy-analyzer` | ✅ 已有 | ✅ |
| DSL Language Server | `content-compiler` lsp 模块 | ✅ 已实现（阶段 29，见 `phase29-dsl-lsp.md`；本行为阶段 22 时点快照） | 📋 骨架 |
| Extension Studio | `apps/extension-studio` | ✅ 已有 | ✅ |

## 各工具详情

### Rule Timeline Inspector（`timeline`）
从 replay log 构建可视化时间线：每个 ResolutionRecord → TimelineEntry（seq / revision / kind / description）。
- `build_timeline(log)` → `Vec<TimelineEntry>`
- `render_timeline(entries)` → 文本表格

### Query Explanation Viewer（`query_viewer`）
渲染 `query_explain` 的 trace 为人类可读的 base + modifiers → final 树。
- `render_explanation(expl, label)` → 文本树

### State Diff Viewer（`state_diff`）
逐字段比较两个 GameState：revision / turn / players.hp / hand_count / alive / dying / cards / outcome。
- `diff_states(old, new)` → `Vec<FieldDiff>`
- `render_diffs(diffs)` → 文本表格

### State Hash Divergence Bisector（`bisector`）
找到两个 replay log 的首个分歧 revision（跨目标/跨构建诊断）。
- `find_divergence(a, b)` → `DivergenceResult`
- `render_divergence(result)` → 文本

### Replay Minimizer（`minimizer`）
将命令日志缩减为仍能复现目标 state hash 的最小子集（delta-debugging）。
- `minimize(commands, target_hash)` → `MinimizeResult`

### VM Fuel Profiler（`fuel_profiler`）
报告单个 bytecode 模块的 fuel 消耗（识别烧 fuel 的技能）。
- `profile_module(module, fuel_budget)` → `FuelProfile`

### Observer Leak Test Harness（`leak_harness`）
系统扫描 observer views 的隐藏信息泄露：外人手牌面/私有身份/旁观者卡牌面。
- `scan_view(state, view)` → `Vec<Leak>`
- `scan_all_observers(state)` → 所有玩家 + 旁观者全面扫描

### Content Package Test Runner（`package_runner`）
验证编译后的内容包：编译/逻辑哈希可复现/引擎加载/StartGame/manifest 钉死/不变量。
- `run_package_tests(source)` → `Vec<PackageTestResult>`

## 测试

```bash
cargo test -p devtools    # 13 tests
```

| 模块 | 测试 |
|---|---|
| timeline | 命令序列时间线、表格渲染 |
| query_viewer | 距离查询解释渲染 |
| state_diff | revision 变更检测 |
| bisector | 相同日志无分歧、不同 seed 检出分歧 |
| minimizer | 无关命令移除、错误 hash 未找到 |
| fuel_profiler | HALT 消耗报告、无限循环 OutOfFuel |
| leak_harness | 干净对局无泄露 |
| package_runner | 奸雄包通过测试、无效源编译失败 |
