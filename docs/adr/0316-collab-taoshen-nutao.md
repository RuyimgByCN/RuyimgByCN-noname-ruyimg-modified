# ADR 0316：COLLAB 桃神（怒涛）

本批为 COLLAB 包桃神（`taoshen`）的单技能完整移植，全部以既有 DSL 原语组合表达，
不新增 Rule VM ABI。

## 技能移植

「怒涛」（`dcnutao`，3716839681）：使用目标数大于一的锦囊牌指定目标后，对其中一名
其他角色造成 1 点雷属性伤害。DSL 以 `card_targeted owner_alive_non_multitarget_trick_targets_gt_one`
触发 + `choose_targets_damage(self, 1, 1, other, 1)` 表达。

语义说明：`card_targeted` 在指定目标时触发，与旧 JS 的 `useCardToPlayer` 时机一致；
过滤器校验目标数 > 1 且为普通锦囊，与旧 JS 的 `event.targets.some(i => i != player)`
条件吻合。

文档化近似：旧 JS 以 `randomGet` 随机选择一名非拥有者目标，DSL 的
`choose_targets_damage` 由拥有者选择目标（AI 引导随机）。

桃神（`taoshen`，2630348587，男/群/3 体力）由单一技能构成完整闭包，加入
`characters.json` 选将池；collab npack 重建为 31 技能 + 18 武将，ABI 450。
旧 JavaScript 不进入 npack 或运行时。
