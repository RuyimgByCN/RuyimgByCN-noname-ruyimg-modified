# ADR 0419：SHIJI 宗预闭环（直辩 / 预言，复用 ABI 369 / 370）

本批发布 SHIJI 宗预（`sp_zongyu`，4036677422，男/蜀/3 血）。技能「直辩」
（`zhibian`）与「预言」（`yuyan`）完成闭环；旧 JavaScript 仅作语义来源。
**零新增 ABI**——两个技能均复用 TW 扩展既有管线。

## 决策

- **zhibian（直辩）**：准备阶段开始（可选）与一名其他角色拼点；胜后三选一
  （移动目标装备区/判定区牌、回复 1 点体力、背水跳过摸牌阶段并依次执行）；
  输则失去 1 点体力。**复用 ABI 369 `resolve_twzhibian`**——其 stage 0 的
  阶段检查由 `Phase::Play` 放宽为 `Prepare | Play`（TW 直辩的 DSL filter 已限定
  出牌阶段，放宽不影响 TW 语义）。
- **yuyan（预言）**：锁定技。成为高体力来源的实体【杀】目标后，来源须交一张点数
  大于该杀的牌（非闪/桃/酒），否则此杀对你无效。**复用 ABI 370
  `resolve_twyuyan`**（过滤器 `owner_target_physical_sha_source_hp_gt_owner` 与
  TW 御严一致，语义等价）。

## 验证

- 新增 `original_shiji_zhibian_yuyan.rs` 2 个行为测试：直辩准备阶段打开拼点目标
  choice、预言触发交牌 choice（直接调用原语验证——手动 dispatch 无技能上下文，
  原语行为以真实 skill 实例验证）。
- 打包：59 技能 / 16 武将，`rule_vm_abi_version` 502。
- 顺手补齐并行会话（omp，ABI 537/538 姜维）缺失的 ABI 常量。
