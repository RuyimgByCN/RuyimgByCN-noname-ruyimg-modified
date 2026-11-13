# 第 0 项独立静态复核第二轮

- 日期：2026-11-13；复核人：`queue0_independent_review`；HEAD `3a6e398485db6ef0729abe3013c3c4b7705a5d6d`，tracked dirty；规则 333／NGSA 107／schema 5／ABI 759／WS8。
- 本文记录实现整合过程的实际源码发现；尚未运行 Cargo，不作为最终修复或通过证明。未读本轮实现者说明／证据 README。

## 补充发现

1. **普通装备被暗织无效后不能伪发 equipAfter。** 装备提交最初只弃牌并 return，调度器仍推进 Equipped 的 AfterTriggers。真实例子为 `dddjiexing` 本人分支：此前用过该技时只根据 Equipped 事件重置，不检查装备已入槽。随后源码已增加将该帧置 Done，调度器 Commit 分支也保留 Done；需要以实际测试证明红牌正常装备、黑牌被免疫、旧栏不替换、装备技不授予、劫行不伪触发。
2. **phaseAfter 的 Commit 也晚于到期入口。** `TurnPhaseAfter` 从 BeforeTriggers 起步，先执行该窗技能，再 Commit；仅把暗织旗从 clear_turn 移到 after=true Commit 仍偏晚。最新源码改为在 phaseEnd 窗 Commit 发出 phaseAfter 之前，只清暗织两旗；需要分别测试 phaseEnd 内仍免疫、phaseAfter 内已到期。
3. **最大帧号不是末次用牌顺序。** 为避免旧事件回写，整合稿新增 `dddanzhi_last_event > current_frame` 守卫。既有 `UseVirtualCardOnSelectedTargets` 的多目标决斗会逆序压入多个兄弟 CardUsed 帧，后创建的先执行。首场黑色打出后，低编号的末场决斗真正开始时被该守卫忽略，末牌颜色错误保留黑色。上游 `gameEvent.ts` 在 `start()` 中才将事件 push 到全局 everything 历史；并非 constructor。实际 `offline/wn_dutan.skill` 可走此路径，不能按最大创建编号替代执行顺序。
4. **黑牌转桃园同族清理与恢复仍缺。** `play_rules.rs` 的 `fengze` 能以黑牌转桃园；TaoyuanMachine 收尾仍按目录原牌种扫描 table，黑 Sha 载体会留在结算区。其既有 `MachineSnapshot::Taoyuan { owner }` 已有 tag 57 编解码和校验，但恢复分派仍缺失，故在无懈窗口挂起时恢复失败。需要像五谷一样准确清理当前 CardUse 载体、重建现有形态，并验证只移除暗织接收者、其他角色正常回血、无懈挂起可恢复。

以上均已逐项通知主 agent；本轮不参与生产实现。最终结论以冻结后的实际源码及独立执行结果为准。

## 清单即时下调

按主 agent 明确授权，已将唯一 `U01:skills:dddanzhi@archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:5796` 项下调「简化近似」，标明修复执行中及保留的 Prepare 触发／选序差异。其他 4,423 个项在本次写入前后逐对象校验未变，仅同步 updated_at 和重算 counts。旧项见 [ddd-inventory-before.json](ddd-inventory-before.json)，执行中快照见 [ddd-inventory-downgrade-executing.json](ddd-inventory-downgrade-executing.json)。

写入时全仓为已复核等价 849／简化近似 3,575／缺失 0，U01 为 94／1／0，L20 为 91／4／0；`git diff --check` 通过。这是修复过程状态，最终验证描述稍后更新，状态不回升。
