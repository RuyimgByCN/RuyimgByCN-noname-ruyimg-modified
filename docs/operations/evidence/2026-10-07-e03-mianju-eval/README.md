# E03 mianju（漩涡面具）翻面原语立项评估

日期：2026-10-07　|　性质：立项评估（未实现）
关联：[主稿 E03 行](../../../architecture/native-progress.md)、[欢乐包纵向样本]()

## 上游语义（欢乐卡牌 `card/skill.js` mianju）

- 牌：**漩涡面具**（equip5 宝物，`card.js` mianju，skills: ["mianju"]）。
- 技能：**锁定技**——`trigger: { player: "turnOverBefore" }`、`forced: true`、
  `equipSkill: true`，content = `trigger.cancel()`。
- 语义：装备面具的角色，其武将牌的翻面被取消（**不能被翻面**）。
- 翻面来源（上游）：武将技（如据守类）与规则效果——欢乐包内无翻面来源，
  面具是纯防御性掩体。

## 引擎缺口（三项，均为首次引入）

| 缺口 | 说明 |
|---|---|
| 翻面状态 | `PlayerState` 无 `face_down: bool`（武将牌朝向） |
| 翻面事件 | 无 `TurnOverBefore` / `TurnOvered` 事件对（翻面的前置取消窗口与完成） |
| 翻转动作原语 | 无 `flip(player)` 状态转移（含事件 emit） |

装备栏（`EquipSlot::Treasure`）与伴随技能表机制（`card_kind_host_skills`）已就绪，可直接复用。

## 版本影响评估

- `PlayerState` 追加 `face_down` → `canonical_hash` 编码追加 → **规则版本 v60→v61**；
- 存档 `PlayerState` 编码追加字段 → **格式 v3→v4**（旧 v3 字节版本门禁拒绝，
  无生产迁移数据）；
- **14 个 golden 全部重录**（状态编码变化）——需按「确认语义后更新 golden」
  流程执行（face_down 初始 false 的编码追加本身不改行为，重录依据充分）。

## 实现清单（立项后的切片，预估一个任务）

1. `PlayerState.face_down` + 哈希/存档编码 + 格式 v4（旧字节拒绝说明入规则基准）；
2. 翻转动作原语（`flip(player)`：TurnOverBefore 帧（前置取消窗口）→ 翻转 → TurnOvered）；
3. `CardKind::Mianju`（89）+ 引擎伴随表（`TurnOverBefore` 强制取消，filter =
   装备区有面具）——纯数据声明，包无需 skills mapping；
4. 翻面来源样本：`debug_flip` 原语（测试驱动）先行；真实翻面来源（据守类
   武将技）另批评估；
5. 测试：装备面具 → 翻面来源触发 → 翻面被取消（无状态翻转）→ 摘除面具后
   翻面可发生；挂起/恢复、装备技离场撤销。

## 验收条件

- 装备面具的角色翻面被取消（正例）；未装备/摘除后翻面正常（反例）；
- 挂起态恢复一致；规则 v61、格式 v4 递增与 golden 重录依据登记规则基准；
- 全量通过。

## 决策建议

立项可行（原语边界清晰、伴随表机制现成、版本影响可控），建议作为 E03 的
下一个实现任务单独授权；实现即动规则/格式版本与 golden 重录，须按
「确认语义后更新 golden」流程执行。


## 真实翻面来源查证（2026-10-07 补）

- **当前上游归档（noname HEAD 5c02a745）无标准曹仁/据守**：`apps/core/character`
  仅含变体条目（`jx_shen_caoren`、`jy_caoren`），其技能（`jxjushou` 等）语义
  远超最小翻面（翻面摸 X 张 + 全场翻面 + 弃装备链）。
- **欢乐包自身无翻面来源**（面具是纯防御掩体）。
- 路径结论：mianju 的「真实翻面来源」**依赖曹仁武将装配**——归入
  P2/N08 规则对齐（曹仁条目装配时一并实现据守与翻面驱动）；`debug_flip`
  作为验收驱动继续保留（翻面取消逻辑已有正反例）。写轮眼/猴子原语
  验收不受影响。
