# 2026-10-07 C3 冲突依赖图场景核验

本批次完成主稿 P1/E02·C3 的「冲突与依赖图」场景切片：依赖激活强校验、
吊销级联剪枝、升级受阻路径、激活失败原子性、revoke 重激活拒绝。
上一批：[E03 猴子切片](../2026-10-07-e03-monkey/README.md)。

## 检查与结果

| 检查 | 结果 |
|---|---|
| 新增场景 | `c3_dependency_conflict`（content-runtime）：自构造依赖包 B@1.0.0 → A@1.0.0 |
| ① 激活强校验 | 激活选择使 B 的依赖（A@1.0.0）不满足（A 升 1.1.0）→ 整体 `Invalid("dependency mismatch …")`，活动集不变——`activate_packages` 是激活时强校验，不允许悬空依赖的活动集 |
| ② 吊销级联剪枝 | `revoke_content_hash(A@1.0.0, deactivate=true)` → A 解除激活且依赖它的 B 被级联剪除（`prune_broken_dependents` 到不动点）；此后 A@1.1.0 可正常激活 |
| ③ 激活失败原子性 | 选择含不存在版本（B@9.9.9）→ 整体失败，活动集与内容锁哈希均不变 |
| ④ revoke 重激活拒绝 | `revoke_slot` 后重新激活 → 拒绝 |
| 全量验收 | content-runtime **32/0**（31+1）、engine-native 349/0、loader 1/0、engine-core 3777/0，合计 **4160 通过 / 0 失败**（[test-logs.txt](test-logs.txt)） |
| 规则/格式 | 规则 v60、格式 v3 不变（包管理语义不触及状态编码） |

## 结论边界

- 已实现并验收：依赖激活强校验、移除级联剪枝、升级受阻、激活原子性、
  revoke 重激活拒绝五条正反例。
- C3 剩余：跨包组合的更多场景（卸载进行中对局、旧版保留多级历史）、
  签名/信任链与冲突声明的产品化策略——按主稿 C3 行继续登记。
- 过程中发现并修正：`activate_packages` 与 deactivate/revoke 路径的依赖
  语义不同（前者强校验拒绝、后者级联剪枝）——场景按实际语义设计并在
  测试注释中写明。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（shasum -a 256，4 文件，相对
noname-next 根）。
