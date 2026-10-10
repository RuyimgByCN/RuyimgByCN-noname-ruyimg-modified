# U05 swd 第六批：吸血鬼指环（规则 v90，2026-10-08）

swd 子包第六批转换：吸血鬼指环（♠5，equip5 宝物，锁定技——你的【杀】
造成伤害后，若你已受伤，回复等量体力）。前置：
[swd 第五批](../2026-10-08-u05-swd-batch5/README.md)。

## 上游语义核验（源码为准）

- `card/swd/card.js` `xixueguizhihuan`：equip5 宝物，`skills: ["xixue"]`。
- `skill.js` `xixue`：equipSkill、`trigger: { source: "damageEnd" }`、
  forced、filter `event.card.name === "sha" && player.hp < player.maxHp`、
  content `player.recover(trigger.num)`。

## 改动（分开陈述）

- **代码已实现**：`CardKind::Xixueguizhihuan`（编码 109：archive enc/dec、
  a0 variant/ALL 同步；`EquipSlot::Treasure` 占栏）+ commit_damage 杀
  伤害后吸血——来源存活、已受伤、伤害牌为【杀】→ `recover(amount,
  None)`；非杀/满血不触发（filter 同口径）。engine 穷尽臂追加。
- **转换包**：cards.json 增 ♠5 吸血鬼指环 + 哈希重算（现 12 张）。

## 验收口径

- **回归已加入**：`swd_xixue_lifesteal_on_sha_damage`——杀 1 伤后吸血
  （2→3）、满血时无回复、目标扣血正常；两局间 sha_used 快照重置脚手架。
- **全量通过**：engine-native **405 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**，DEPS_EXIT=0。
- **golden**：版本 89→90 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（纯版本位移）。
- **剩余**：swd 其余 10+ 卡种逐批转换。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（13 文件，相对 noname-next 根）。
