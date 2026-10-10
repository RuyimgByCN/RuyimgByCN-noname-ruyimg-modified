# ADR 0297：SHIJI SP 陈董「分明」与 `hp_lte_owner`（ABI 402）

ABI 402 增加主动目标约束 `hp_lte_owner`：每个被选目标的当前体力值不得大于技能拥有者。

「分明」（`spfenming`）以该约束锁定合法目标，再按目标是否拥有 `MARK_CHAIN`（横置）分支：
- 未横置：`add_mark(..., 2, 1)` 横置目标；
- 已横置：对自己 `take_one_card(..., equip)`，对其他角色 `take_one_card(..., hand_equip)`。

已横置但无可获得牌时，`take_one_card` 空操作；旧 JS 的目标过滤会在选目标阶段排除此类角色，
当前以权威体力约束 + 获牌原语近似，不新增专用目标 OR 谓词。

SP 陈董（`sp_chendong`）由「毅烈」+「分明」组成完整 4 体力吴势力武将闭包。
旧 JavaScript 不进入 npack 或运行时。
