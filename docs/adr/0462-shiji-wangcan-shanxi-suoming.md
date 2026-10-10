# ADR 0462：SHIJI 王粲闭环（擅袭 / 索命，ABI 440 / 441）

本批发布王粲（`sp_wangcan`，3846453081，男/魏/3 血）完成闭环：继「弃哀」
（ABI 439）后补齐「擅袭」（`spshanxi`，ABI 440）与「索命」（`spshanxi_suoming`，
ABI 441），作为完整武将进入选将池。旧 JavaScript 仅作语义来源。

## 决策

- **ABI 440 `resolve_spshanxi(player, target, stage, skill)`**：出牌阶段开始
  选一名无“檄”标记的其他角色；授予“檄”标记时**全局唯一**——先清除所有角色
  的檄标记再授予目标（`RemoveMark` amount 0 = 移除整项，注意 u32::MAX 转 i32
  为 -1 会变成加法）。檄标记键 = 擅袭技能 def id。
- **ABI 441 `resolve_spshanxi_suoming(player, actor, stage, skill)`**：其他角色
  回复体力后（新过滤器 `owner_alive_other_marked_xi`：HpRecovered 事件、他人、
  有檄标记且体力 > 0）——可交 2 张 he 则开交牌 choice（可取消），取消或不足
  2 张直接失去 1 点体力；交牌以 blob "SM1" 记录所选 2 张后 `MoveCards` 赠予。

## 验证

- `original_shiji_zhengsu.rs` 新增第 10 个测试：擅袭选人授予檄标记 → 再授他人
  时旧标记被清除 → 索命开交牌 choice → 两张牌交到拥有者手中。
- 打包：87 技能 / 27 武将，`rule_vm_abi_version` 502（max 语义）。
- conformance 3/3 通过；我的 5 套件 25 测试全绿。
- 顺带协作补位：Codex 势曹爽（ABI 568）迁移中间态（重复 stub 清理、
  `resolve_potcaoshuang_choice` 可见性、借用冲突、测试借用修复、Hermes
  `take_one_event_target_hand` IR/codegen 缺臂、ABI 567 常量）。
