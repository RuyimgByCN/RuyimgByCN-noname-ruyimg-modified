# U05 swd 第九批：星君眼 + 青龙令珠（规则 v93，2026-10-08）

swd 子包第九批转换：两张 equip5 宝物——星君眼（杀伤 +1 锁定）与青龙
令珠（属性伤害后获得目标一张牌）。前置：
[swd 第八批](../2026-10-08-u05-swd-batch8/README.md)。

## 上游语义核验（源码为准，card/swd/skill.js）

- `xingjunyan`：equipSkill、`trigger: { source: "damageBegin",
  player: "damageBegin" }`、forced、filter `event.card.name === "sha"`、
  content `trigger.num++`。
- `qinglonglingzhu`：equipSkill、`trigger: { source: "damageEnd" }`、
  `direct: true`、filter `hasNature() && player.isAlive()`（对其他角色）、
  content `gainPlayerCard(…, trigger.player, …, "he")`。

## 改动（分开陈述）

- **代码已实现**：`CardKind::Xingjunyan`（编码 112）+ `CardKind::
  Qinglonglingzhu`（编码 113）——双 equip5 宝物占栏；commit_damage 两
  钩子：星君眼 hp 扣减前 `amount += 1`（card==sha，铁索传导独立事件
  照常）；青龙令珠 DamageDealt 日志后属性伤害获得目标一张 he 牌
  （RNG 代抽登记差异——上游为 UI 选牌）。
- **首跑缺陷修正**：星君眼钩子初版误置于 hp 扣减后（日志区），测试以
  6→4（期望）vs 实测 5 捕获；装备注入未全区域清除致
  card_in_multiple_zones——补全区域 purge。
- **登记差异**：青龙令珠的「获得哪张」由目标自选（上游 gainPlayerCard
  带 prompt 由双方交互），原生 RNG 代抽（批四口径）。

## 验收口径

- **回归已加入**：`swd_xingjunyan_amp_and_qinglonglingzhu_steal`——
  星君眼杀伤 +1（6→4）、青龙令珠属性伤害后目标 -1 牌/使用者 +1 牌。
- **全量通过**：engine-native **408 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**，DEPS_EXIT=0。
- **golden**：版本 92→93 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（纯版本位移）。
- **剩余**：swd 其余卡种（羊皮卷/贴房绅视/伏符录/风雪刃/兰阁怪衣/
  岱火风轮/夏云彩绫等多步或距离修饰型）逐批转换。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（13 文件，相对 noname-next 根）。
