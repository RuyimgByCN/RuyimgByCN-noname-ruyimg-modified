# NC05 第一类纵向案例：重规则/嵌套选择扩展的全生命周期闭环（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC05**（[方案 §NC05](../../../architecture/native-convergence-plan.md#nc05)）第一类真实案例
- 案例来源：`shahai-swd-upstream-noname`（**505 个宿主行为**，含嵌套选择与多步机器；`package_id = shahai-swd`）
- 规则版本 v321 / 存档 v105（无规则与格式变化）
- 结论：**产品入口全生命周期闭环通过**（安装 → 装载 → **正式选将** → 建局驱动 → 真实交互 → 存读档 → 升级/回退 → 恢复）

## 1 闭环路径（全部走产品入口，未用 `debug_grant`）

| 环节 | 产品入口 | 断言 |
|---|---|---|
| 作者包产物 → 独立安装 | `NativeContent::import()`（`import_extensions`） | swd 包在安装清单 |
| 装载（NC02：安装副本为权威） | `NativeContent::load(package_id)` | 扩展卡牌/武将并入目录（`catalog_cards`/`catalog_generals` 大于基线） |
| **正式选将** | `NativeGameRequest.generals` → `Engine::assign_general_named` | 扩展包自带且带技能的武将可装配（候选来自目录，非调试授技） |
| 建局与驱动 | `create_native_game`（`BoundedPlanner` 有界驱动 120 步） | 建局成功、包清单一致、驱动不早退 |
| 真实交互 | `Engine::legal_actions(seat)` 取候选 → `submit` | 命令被接受（Accepted/Pending），未用调试入口 |
| 存读档 | `save_fresh_match` / `restore_match`（归档字节） | 读档目录规模与存盘一致 |
| 升级/回退 | `PackageStore::install_with_assets` → `load_locked_packages(&pinned)` → `rollback` | **固定内容锁版本不变**（升级不改写已固定锁的局） |
| 恢复能力保留 | 重新激活原版本后 `restore_match` | 恢复结果与首次一致 |

## 2 产品接线改动（本次新增）

`NativeGameRequest` 增加 **`generals: Vec<String>`**（按座位顺序的武将名；空 = 保持建局随机装配），建局时经 `Engine::assign_general_named`（NC05 产品入口）装配，失败整体报错——**产品层不再需要 `debug_grant` 才能让扩展武将上场**。

## 3 验证结果

```
cargo test --offline -p game-server --test nc05_lifecycle_heavy
cargo test --offline -p game-server --no-fail-fast
python3 -B scripts/check-source-lines.py
```

| 检查 | 结果 | 证据 |
|---|---|---|
| NC05 闭环定向 | **1 passed / 0 failed** | [targeted.log](targeted.log) |
| game-server 全量 | **57 passed / 0 failed** | [game-server-tests.log](game-server-tests.log) |
| 行数门禁 | 8232 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |
| `-D warnings` 检查 | **被并行重构阻断**（`engine-core` 未使用导入），无一条告警位于 `apps/game-server` | [gate-game-server.log](gate-game-server.log) |

## 4 未完成 / 待办（NC05）

- [ ] 第二类案例（战役/跨局档案 + 升级恢复）与第三类案例（皮肤/UI 组合）——本批只完成第一类
- [ ] 真实**客户端/Worker/浏览器**入口（当前闭环在服务端同进程产品入口）
- [ ] 多选/取消/过期选择、错误可见、跨座位信息隔离在**产品会话**上的端到端
- [ ] 赛前大厅阶段（当前选将在建局请求内，上游为开局前独立阶段）
- [ ] 卸载（移除安装）后按内容锁**重新取得**与缺失拒绝策略
- [ ] 全量 `cargo test -p engine-native`（含 examples）与 Gate 1b：本轮被并行任务在飞文件 `examples/nc12_bench.rs` 阻断，待其提交后复跑
