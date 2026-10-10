# ADR 0729：N08 八卦阵语义与武将表扩面、N09 关闭计数器对照

日期：2026-09-16
状态：已实施（engine-native）
关联：[0724](0724-native-n08-content-expansion.md)、[0728](0728-native-n05-n02-n09-small-batch.md)、进度文档 N08/N09

## 背景与决定

进度文档 N08 剩余「armor 等无效果装备语义」与「武将表扩面」；N09 剩余「关闭计数器
对照」。本批逐项交付。

### 1. 八卦阵（armor 防具语义落地，规则版本 7 → 8）

`armor_occupies_slot_without_protection` 所锁定的「防具仅占栏」升级为真实语义：
**受击者装备防具时，杀结算的出闪询问前先做防具判定——红桃视为已出闪（伤害免除），
否则照常进入出闪询问；每杀至多判定一次。**

- `ShaResolveMachine` 增加 `armor_judged` 标志（五处同步：结构体/构造/快照/
  restore/`push_machine`，标志入状态哈希）；判定牌判定后即弃置（无获得路径）。
- 新日志事件 `ArmorJudged { player, card, success }`（宿主日志，不入摘要）。
- 语义变化 → `STANDARD_RULESET_VERSION` 7 → 8，14 golden 重录并通过。
- 回归 `armor_bagua_judgement_semantics` 三段：守方防具+红桃判定（伤害免除、
  无出闪询问、杀照常弃置、守恒）/ 判定未命中（照常出闪询问、出闪后无伤害）/
  无防具对照（不产生 ArmorJudged）。

### 2. 武将表扩面（机制样例组合，诚实标注）

`STANDARD_GENERALS` 扩为三名：张角（官方技能迁移）、样例·策援（移伤+授艺）、
样例·守财（碎盾+散财+肃清）。样例条目为 ADR 0712 机制样例组合，**不计官方技能
迁移**——712 的五个样例技能无官方武将对应，不虚构命名。
回归 `general_table_grants_all_bundles`：表完整性（武将名唯一、技能均在目录）+
逐武将授予后全部技能 enabled。

### 3. N09 关闭计数器对照（`N09_PLAIN=1`）

计数分配器增加 plain 模式（计数钩子短路，近似关闭计数器——分支开销仍在）。
20 局 seed 9000 对照（release）：counting=on p50=1917ns vs plain p50=1958ns（FFA）、
2083 vs 2167（身份）——**计数器开销在噪声范围内（~2%）**，既有纳秒分位数据
无需修正口径。plain 模式下 `alloc_delta/peak` 输出为 0/0（符合预期）。

## 影响

- 测试 148 项（native_kernel 84、boundary_snapshot 59、场景 2、源码 2、调试 1），净增 1
  （armor 测试为原位重写；武将表测试 +1）。
- 规则版本 7 → 8；14 golden 重录；runner 复跑退出 0
  （evidence/2026-09-16-native-n09/runner-v8.log，26,552 行原始样本）。
- N08 剩余：真正官方武将/技能迁移（需新机器语义，如过河拆桥类锦囊外技能）；
  N09 剩余：旧线等价负载跨目标对照（N12，需旧线可运行）。
