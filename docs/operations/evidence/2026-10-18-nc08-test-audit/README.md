# NC08 测试面审计：重复测试属性 / 固定目录数守护 / 重复执行（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）——「修复重复测试属性、固定目录数等过期守护」
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**两项审计为 0 缺陷**；**重复执行已量化并给出成因**（非缺陷，属 `#[path]` 包含模式的固有代价）

## 1 审计一：重复 `#[test]` 属性 / 同名用例

扫描 `crates/*/{src,tests}` 与 `apps/*/{src,tests}` 全部 `.rs`：按「同一文件内同名 `#[test]` 函数」判定。

| 结果 | 值 |
|---|---|
| 扫描文件数 | 见 [dup-attrs.log](dup-attrs.log) |
| **同名 `#[test]` 的文件** | **0** |

> 结论：审查快照中「native-wasm 20 次执行含重复 `#[test]` 导致的同一案例重跑」在当前源码中**不成立**
> （同名属性为 0）；见下节对「同一案例在多个二进制中重跑」的量化与成因。

## 2 审计二：固定目录数等过期守护

检索测试/源码中硬编码的目录规模断言（`catalog().cards.len() == <3 位数>`、`general_count() == <2 位数>` 等模式）：
**命中 0 处**。既有守护均为「基线 + 增量」形式（如 `base_catalog_cards + N`），不随内容增长过期。

## 3 重复执行：量化与成因（非缺陷）

命令：`cargo test --offline -p game-server --no-fail-fast` 与 `-p engine-native --no-fail-fast`，
按用例名统计执行次数（[dup-executions.log](dup-executions.log)）：

| 用例 | 执行次数 | 成因 |
|---|---|---|
| `persist::tests::wal_manifest_mismatch_is_rejected` | **8** | `persist.rs` 被多个测试二进制以 `#[path]` 包含（`room.rs` 依赖它，而 `room.rs` 又被多个集成测试包含）→ 其内部 `#[cfg(test)] mod tests` 在每个二进制中各跑一次 |
| `persist::tests::wal_and_snapshot_recover_preserves_manifest` | **8** | 同上 |

- 这不是「重复属性」，而是 **`#[path]` 包含模块的固有代价**：无法从外部屏蔽被包含模块内的 `#[cfg(test)]`；
- 影响：测试**计数**虚高（同一断言多次执行），但不掩盖失败——任一次失败都会让该二进制失败；
- 处置：**量化并登记**（不改测试语义）。若要消除，需要把 `persist`/`room` 拆成独立 crate 或改用
  `#[cfg(not(test))]` 门控（属结构性改动，登记为待办，不在本批强行改）。

## 4 验证与证据

```
cargo test --offline -p game-server --no-fail-fast        # 84/0（本批未改产品源码）
python3 -B scripts/check-source-lines.py                  # 8255 文件 ≤2000 行
```

| 检查 | 结果 | 证据 |
|---|---|---|
| 同名 `#[test]` | 0 | [dup-attrs.log](dup-attrs.log) |
| 固定目录数守护 | 0 | 本节 §2 |
| 重复执行量化 | `persist::tests` 8×（成因已说明） | [dup-executions.log](dup-executions.log) |
| 行数门禁 | 8255 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 5 未完成 / 待办（NC08）

- [ ] 消除重复执行（拆 crate 或 `#[cfg(not(test))]` 门控）——结构性改动，登记待办
- [ ] 「在测试中重新实现生产逻辑的假闭环」逐项排查（本批未做；NC03 前端切片已将适配层改为调用生产实现路径）
- [ ] 各层边界清单化（单元/内核集成/生产适配器/真实客户端/服务端/native-WASM/上游/异常输入）
- [ ] 上游行为逐命令对照（NC08 主体，未开始）
