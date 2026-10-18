# NC05 第二类纵向案例：战役/跨局档案 + 升级恢复（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC05**（[方案 §NC05](../../../architecture/native-convergence-plan.md#nc05)）第二类真实案例
- 案例来源：`boss-play-upstream-liuli`（Boss 模式扩展包）+ 原生 Boss 关卡档案
  （`GameMode::Boss.stage`：关卡等级 / 孟婆旗标 / 禁将表 / 选将挂起）
- 规则版本 v321 / 存档 v105（无规则与格式变化）
- 结论：**战役档案闭环通过**（安装 → 装载 → 带扩展 Boss 关卡建局 → 档案写入 →
  归档往返 → 升级/回退 → 恢复）

## 1 新增产品入口

`new_boss_stage` 不带扩展目录、`new_with_extensions` 不带 Boss 模式——战役扩展需要两者组合，故新增：

| 接口 | 语义 |
|---|---|
| `Engine::new_boss_stage_with_extensions(seed, seats, boss, chongzheng, boss_cards, player_cards, first, packs)` | 带扩展目录的 Boss 关卡建局（目录经 `standard_catalog_with_extensions` 合并，内容锁语义；模式为 `GameMode::Boss`；开局按 `OpeningPlan::BossStage` 发牌） |
| `Engine::boss_stage_progress()` | 战役档案只读快照 `(关卡等级, 孟婆已出现, 孟婆已死亡)`（非 Boss 模式 `None`） |

## 2 验收回归（新增 `apps/game-server/tests/nc05_campaign_profile.rs`，2 项）

| 验收项 | 用例 | 结果 |
|---|---|---|
| 安装 → 装载（**显式命名空间**，NC02 行为）→ 带扩展 Boss 关卡建局 → 写进度 → **归档往返**（`encode_archive`/`decode_archive` + 带扩展恢复）→ 进度与目录规模一致 → 升级 9.9.9 后**固定内容锁版本不变** → 回退/重新激活后进度仍一致 | `nc05_campaign_profile_lifecycle` | 通过 |
| 对照：档案与内容不符（缺少扩展）时恢复**明确失败** | `nc05_campaign_profile_rejects_wrong_content` | 通过 |

> 过程中实际触发 NC02 行为：`boss-play` 同时存在于 `upstream-liuli` 与 `upstream-noname` 两个命名空间，`load("boss-play")` **明确报错要求指定命名空间**，改用 `load_scoped(Some("upstream-liuli"), …)` 后成功——这是 NC02「同 id 不同命名空间需显式选择」在生产路径上的实证。

## 3 验证结果

```
cargo test --offline -p game-server --test nc05_campaign_profile
cargo test --offline -p game-server --no-fail-fast
python3 -B scripts/check-source-lines.py
```

| 检查 | 结果 | 证据 |
|---|---|---|
| NC05 第二类案例定向 | **2 passed / 0 failed** | [targeted.log](targeted.log) |
| game-server 全量 | **59 passed / 0 failed** | [game-server-tests.log](game-server-tests.log) |
| 行数门禁 | 8235 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 4 未完成 / 待办（NC05）

- [ ] 第三类案例（皮肤/UI 组合）
- [ ] **跨局档案的产品持久化层**（game-server `persist.rs` 房间存档）与真实客户端；本批覆盖内核档案 + 内容锁语义
- [ ] 构造期扩展初始化钩子（wuxing 五行 / swd 机巧）在 Boss 关卡入口未预置（登记差异，与 `new_with_extensions` 的钩子处理口径待统一）
- [ ] 战役档案的**跨版本迁移**（旧档案 + 新规则版本）与缺失内容重新取得
- [ ] 全量 `cargo test -p engine-native`（含 examples）与 Gate 1b：被并行在飞 `examples/nc12_bench.rs` 阻断，待其提交后复跑
