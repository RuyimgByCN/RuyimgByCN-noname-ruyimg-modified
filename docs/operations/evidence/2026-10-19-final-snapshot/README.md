# 全量验证快照（2026-10-19 会话末）

- 规则版本 **v322**（工作树）/ 存档格式 **v105**
- HEAD（快照时）：`a55978d5`
- 原始输出：[snapshot.txt](snapshot.txt)
- 交接索引：[2026-10-19-session-handover](../2026-10-19-session-handover/README.md)

## 1 全量测试

| crate | passed | failed | ignored |
|---|---:|---:|---:|
| **engine-native** | **1532** | 0 | 21 |
| **game-server** | **115** | 0 | 0 |
| rule-vm | 15 | 0 | 0 |
| content-compiler | 533 | 0 | 0 |
| content-native-loader | 32 | 0 | 0 |
| content-schema | 0 | 0 | 0 |
| **合计** | **2227** | **0** | **21** |

> `ignored=21` 是**既有**的忽略用例（多为「第 57 项」类已登记缺陷 + 少量平台/环境相关），**未新增忽略**。

## 2 门禁

```
Source line limit passed: 8479 files <= 2000 lines
OK   （scripts 自检 test_check_source_lines.py）
```

## 3 未跟踪文件检查（**重要**）

本会话**未修改、未新增**任何 `??`（未跟踪）文件：所有新建的脚本/用例/证据均已提交。
`git status` 里仍有**大量** `??`（如 `crates/engine-core/**`、`crates/engine-native/src/{kernel,engine,content/host}/**`、
`content/original/ported/**`、`crates/content-compiler/src/abi.rs` 等）——**这些属于其它任务**，
本会话**未触碰**（它们在本会话开始前就已存在）。

## 4 边界（**如实**）

1. 本快照只证明「**上述 crates 的测试全绿**」，**不等于**产品全部验收：
   不覆盖真机/浏览器、TLS/跨机、同负载前后对比、上游 harness、NC06、NC10 质量基线；
2. `ignored=21` 中仍有**已登记缺陷**（如第 57 项 `baishouzhihu` 的 `PhaseEnd` vs `begin==true`）；
3. 八项验收**按来源**：安装/配置 **32/32**、升级/保存恢复 **13**、组合 **11**、规则差异 **5**、性能 **13**、
   目标平台**仅全局构建级** ⇒ **完成八项的来源数 = 0**（见[交接索引](../2026-10-19-session-handover/README.md) §4）。
