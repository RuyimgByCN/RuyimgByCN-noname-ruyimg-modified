# ADR 0756：国战 M3 势力胜负判定

日期：2026-09-16（第五十七批）
状态：已采纳
上游：[0753](0753-native-n11-guozhan-kof-mode-design.md) / [0754](0754-native-kof-m1-guozhan-data-model.md) / [0755](0755-native-n11-faction-kof-m2.md)

## 背景

ADR 0753 立项的 N11 国战里程碑推进至 M3：`GameMode::Kingdoms` 已有势力模型
（`Faction` 四势力 + `STANDARD_GENERALS` 武将→势力映射，ADR 0755）与双将槽/
明置数据模型（ADR 0754），但死亡机器的胜负判定 match 仅覆盖 Prototype 与
Identity 两臂，国战对局无法终局。

## 决策

1. **胜负判定臂**：死亡机器胜负 match 新增 `GameMode::Kingdoms` 臂——
   遍历存活角色的主将槽 0，经 `STANDARD_GENERALS` 查势力；全场存活角色
   均已声明且势力唯一 → `Winner::Faction(faction)` 终局。主将槽为空
   （未明置）或目录越界 → 势力未定，不判负，对局继续。副将槽不参与
   势力判定（与「暗置武将无势力」的判定门一致，明置门控另批）。
2. **终局可持久化**：`Winner::Faction` 入状态哈希（tag 4，`Traitor` 顺延
   为 5 避免撞号）并存档往返；`identity_roles` 对 Kingdoms 返回 None。
3. **附带修复（暴露的既有缺陷）**：`GameStateCore::seat_order_from` 原以
   `next_alive` 环游并以 `cur == start` 退出——起点为已死亡座位时
   （濒死询问链以 `turn.current` 为锚，回合玩家可能先死于段内）游标只在
   存活座位间循环、永不回到死锚，无限循环。改为固定扫描一圈座位数，
   存活者按座位序入队；对存活起点输出与原实现逐字节一致（191 项既有
   回归与 14 golden 复放零变化），仅消除病态输入下的挂死。
4. **验收**：`guozhan_m3_faction_victory`——魏+蜀混杂不判负；仅剩蜀 →
   `Winner::Faction(Shu)`；主将未明置独存不终局；胜负结果入存档往返。
   全量 192 passed / 0 failed。

## 后果

- 国战「同势力独存即胜」闭环；明置门控（暗置武将的势力隐藏与半明置）
  与多点回归矩阵仍属后续批次。
- 无语义/编码变化对既有对局的影响：Kingdoms 臂仅在国战模式触发，
  Prototype/Identity 路径不变，规则版本维持 20，无需重录 golden。
- `seat_order_from` 死锚修复属于防御纵深：正常对局流程（问询链锚点
  恒为存活者）行为不变，回放确定性不受影响。
