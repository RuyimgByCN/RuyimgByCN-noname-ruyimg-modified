# FzLiegong 选序登记独立边界复核

- 日期：2026-11-13；复核人：`queue0_independent_review`；核验时间 `2026-11-13T20:52:01.828634+08:00`。
- 源码 HEAD：`3a6e398485db6ef0729abe3013c3c4b7705a5d6d`；复核开始时 tracked dirty：`False`，本报告及同批证据为未跟踪新增。规则 333／NGSA 107／content schema 5／ABI 759／WS8；本复核不修改生产代码、版本号或台账。
- 复核输入：仅上游源码、原生实际调用链与现有元数据；未读取实现者说明。原始读取命令与输出见 [fz-source-audit.txt](fz-source-audit.txt)。

## 可无轨迹变化完成的修复

在 `metadata/source_order.rs` 为 `HostBehavior::FzLiegong` 登记 `(Timing::Normal, 1, false)`，与目前 `metadata/common.rs:179` 的元数据完全一致。`metadata.rs:871` 只用该 tuple 覆写 timing、priority、order_selectable，因此直接查询行为不变。

真实目录注册为 `host.rs:409` → `host_skill_trigger_meta`；`host.rs:270` 对 `FzLiegong` 委托到 `l20_batch_b.rs:326` 的按名元数据，当前同为 `(Normal, 1, false)`，此路径不查询 `source_order`。`extension.rs:243` 对 host 技能的逻辑指纹也使用同一按名函数，因此仅补登记不会改此路径的目录、内容锁或在档续行数据。所需同步是守卫测试条目及触发行为总数；不能为修门禁更改既有元数据。

这是调用链和字段恒等证明。尚未在本复核内运行 14 场 golden 或旧档续行，须由主 agent 的完整门禁与相应恢复测试验证。

## 与上游的现存差异

上游 `标记补充/extension.js:5160` 的 `fz_liegong` 及所继承的 `character/shenhua.js:6456` 的 `liegong` 均无 priority／silent／firstDo／lastDo 字段。Liuli `game/game.js:60167` 的 `get.priority` 默认 0；同文件 `15281` 与 `15346` 对有译名、非 silent、同角色同优先级技能提供选序。因此上游选序语义为 `(Normal, 0, true)`，当前原生 `(Normal, 1, false)` 保留了已有差异。

仅补登记 `(Normal, 1, false)` 可作为第 0 项漏登记修复，不能作为 `fz_liegong` 上游选序等价证明。若改成 `(Normal, 0, true)` 并同步实际 `l20_batch_b` 委托元数据，会改变与同人并列技能的触发顺序及选择窗口，属于用户目标第三节第 7③要求先报告的轨迹边界；须取得相应答复后实施，不能把新增 tuple 当作隐性升语义许可。

## 清单处置

2026-11-13 依据上述独立源码复核，将 `L20:skills:fz_liegong@archive/upstream/noname-liuli/noname/extension/标记补充/extension.js:5160` 从「已复核等价」降为「简化近似」，复核人填本 agent；旧项原文保存在 [fz-inventory-before.json](fz-inventory-before.json)。同时纠正该项将新烈弓描述混入本技能的条件／伤害+1误述，不改实际实现；全仓清单为已复核等价 850／简化近似 3,574／缺失 0，L20 为91／4／0。此计数只适用于本次清单写入时，后续以清单实际值为准。
