# U05 swd 第十一批：风雪刃（规则 v95，2026-10-08）

swd 子包第十一批转换：风雪刃（equip1 武器射程 2）——【杀】命中后可选
择防止伤害，目标翻面并摸一张牌。前置：
[swd 第十批](../2026-10-08-u05-swd-batch10/README.md)。

## 上游语义核验（源码为准）

- `card/swd/card.js` `fengxueren`：equip1，`distance: { attackFrom: -1 }`
  （射程 2），`skills: ["fengxueren"]`。
- `skill.js` `fengxueren`：equipSkill、`trigger: { player: "shaHit" }`、
  filter `!event.target.isTurnedOver()`、content `trigger.unhurt = true` +
  `trigger.target.turnOver()` + `trigger.target.draw()`；check 为 AI 选择
  逻辑（不迁移，原生以 Confirm 询问代替）。

## 改动（分开陈述）

- **代码已实现**：`CardKind::Fengxueren`（编码 115：archive enc/dec、a0
  variant/ALL 同步；equip1 占栏 + 射程 2）+ ShaResolve
  `asking_fengxueren`（快照/存档/哈希接线）——shaHit 后 Confirm 询问：
  确认 → 防止伤害 + 目标 face_down 翻转（TurnOvered 事件）+ 摸 1；
  放弃 → 照常伤害；目标已翻面不再询问（filter 同口径）。
- **登记差异**：上游 check() 为 AI 决策，原生改为玩家 Confirm 询问
  （与寒冰剑同口径）。

## 验收口径

- **回归已加入**：`swd_fengxueren_flip_or_damage`——确认翻面（未受伤
  4→4、face_down、摸 1）、放弃照常伤害（4→3）、翻面目标不再询问。
- **全量通过**：engine-native **410 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**，DEPS_EXIT=0。
- **golden**：版本 94→95 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（纯版本位移）。
- **剩余**：swd 其余卡种（羊皮卷/伏符录/兰阁怪衣/岱火风轮/夏云彩绫等
  多步型）逐批转换。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（13 文件，相对 noname-next 根）。
