# 主 agent 独立复核（2026-10-18）

| 复核项 | 命令 | 结果 |
|---|---|---|
| 样本 5 定向 | `cargo test --offline -p engine-native --test nc08_upstream_kuwu` | **5 passed / 0 failed**（[verify-targeted.log](verify-targeted.log)） |
| 样本 6 定向 | `cargo test --offline -p engine-native --test nc08_upstream_shengke` | **5 passed / 0 failed**（同上日志） |
| engine-native 全量（含 examples） | `cargo test --offline -p engine-native --no-fail-fast` | **1182 passed / 0 failed / 2 ignored**（[verify-full.log](verify-full.log)） |
| 行数门禁 | `python3 -B scripts/check-source-lines.py` | 8279 文件 ≤ 2000 行（[verify-lines.log](verify-lines.log)） |
| 提交范围 | `git show --stat c4cbf324` / `d1383a92` | 仅其自有文件（未改 `crates/engine-native/src/**`） |

## 复核确认的两处**原生规则缺陷**（子 agent 只登记未修，主 agent 认可该处置）

| # | 位置 | 现象 | 修法（已登记） |
|---|---|---|---|
| A | `crates/engine-native/src/content/host/step/skills_cards.rs`（`rangjie` 分支） | 选「移动牌」后使用者**不摸牌**（上游 `player.draw()` 在分支前无条件执行） | 在 `HostStage::SelectNature` 的 `1 =>` 分支开头补 `ctx.draw_cards(self.owner, 1);` |
| B | `crates/engine-native/src/content/host.rs`（`host_skill_def` limits） | 五行盘 `usable:1` 失效（同阶段可二次发动） | `once_per_turn` 分支加入 `HostBehavior::ChangeCardNature` |

**为何未修（复核认可）**：两处源码均属**他人未提交的 host.rs→host/ 模块拆分**（未跟踪整目录）；
在此提交修复会把他人 200+ 在途改动并入本切片，或使本切片断言依赖未提交源码 → 按「只 add 自己
的文件」登记并上报；**待并行重构落定后修复**（与 L20 批 3、NC01 版本递增、NC06 试点、NC07 修复
同批处理）。

## 勘误（重要，已由子 agent 落入规则基准）

旧登记「上游 `card.name==="sha"` 为精确杀名、原生 `is_sha_like()` 是超集」**不成立**：
归档证据（火杀 `card/extra.js:1481 ["heart",4,"sha","fire"]`、`library/element/card.js:315 this.name=card[2]`、
`card/standard.js:99-102 sha.nature=[...]`）表明**杀族牌 `name` 恒为 `"sha"`**，火/雷走 `nature`
→ 两者**同口径**（非超集）。主 agent 复核认可该勘误（未改他人 g1 测试文件注释）。
