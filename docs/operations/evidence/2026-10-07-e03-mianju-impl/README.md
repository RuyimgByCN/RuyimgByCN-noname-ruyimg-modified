# E03 mianju 翻面原语实现核验（规则 v61 / 格式 v4，golden 重录后）

mianju（漩涡面具）翻面原语实现按立项评估清单执行完成（评估见
[2026-10-07-e03-mianju-eval](../2026-10-07-e03-mianju-eval/README.md)）。
前置批次：[E03 猴子/写轮眼](../2026-10-07-e03-monkey/README.md)。

## 检查与结果

| 检查 | 结果 |
|---|---|
| 翻面状态 | `PlayerState.face_down`（哈希/存档编码追加，face_down 初始 false 行为无变化） |
| 翻面事件对 | `TurnOverBefore`（前置取消窗口）/`TurnOvered`（完成），EventData/EventKind 编码 16/17（各自表尾追加）、哈希臂、归一化臂同步 |
| 翻转动作 | `commit_turn_over_before`：取消位为真跳过翻转；未取消翻转 + emit TurnOvered |
| 漩涡面具 | `CardKind::Mianju`（89）+ 引擎伴随表（TurnOverBefore 强制取消：filter=装备区有面具，step=写回取消位）+ 欢乐包独立载荷 |
| plan_triggers 前置窗口 | TurnOverBefore 帧的 before 阶段查 TurnOverBefore（其余帧仍查 DamageBefore）；Commit/After/Done 阶段流转同 Damage |
| debug_flip | 测试驱动的翻面来源（TurnOverBefore 前置取消窗口经命令层挂起模式） |
| 测试 | `huanle_mianju_blocks_flip_while_equipped`：无面具基线翻面正常 → 装备面具翻面被取消（face_down 不变）→ 摘除（离场撤销）后翻面正常 |
| 版本递增 | 规则 v60→**v61**、格式 v3→**v4**（PlayerState 编码追加，旧 v3 字节版本门禁拒绝；取舍见规则基准） |
| golden 重录 | **14 个场景全部重录**（face_down 编码追加，行为无变化，重录依据：确认语义后更新 golden 流程，`replay_scenarios_match_golden` 通过） |
| 全量验收 | engine-native **353/0/1ignored**、loader 1/0、content-runtime 32/0，合计 **386 通过 / 0 失败**；`RUSTFLAGS=-D warnings` 通过 |

## 结论边界

- 已实现并验收：翻面原语全链路（状态/事件/动作/面具锁定技/正反例）。
- 翻面来源现状：`debug_flip` 测试原语（真实翻面来源如据守类武将技另批评估）。
- A0 同步：CardKind 89 / HostBehavior 77 清单与键双射（同批 a0_tag_registry 全绿）。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（19 文件，相对 noname-next 根）。
