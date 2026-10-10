# U05 swd 第十批：贴房绅视（规则 v94，2026-10-08）

swd 子包第十批转换：贴房绅视（锦囊，他人）——本回合无视距离指定该
目标，并摸一张牌。前置：
[swd 第九批](../2026-10-08-u05-swd-batch9/README.md)。

## 上游语义核验（源码为准）

- `card/swd/card.js` `zhufangshenshi`：type trick、`filterTarget:
  target !== player`、content——`player.storage.zhufangshenshi = target`
  + `addTempSkill("zhufangshenshi")`（至回合结束）。
- `skill.js` `zhufangshenshi`：`mod.targetInRange`——storage 命中该
  目标时距离恒可达；`g_zhufangshenshi`：`useCardAfter` 且 card 为本牌
  → `player.draw()`。
- 无懈：卡带 `wuxie() { return 0 }`（AI 权重 0，实战从不戒）——原生
  简化为直接结算，登记差异。

## 改动（分开陈述）

- **代码已实现**：`CardKind::Zhufangshenshi`（编码 114：archive enc/dec、
  a0 variant/ALL 同步）+ `TurnState.zhufangshenshi_target: Option<
  PlayerId>`（存档 opt_u8、canonical_hash match、TurnState 字面量×3）；
  commit 同步结算=置目标 + 摸 1 + 弃牌；sha 距离校验豁免（zfs_free
  跳过 computed_distance 比较，`targetInRange` mod 同口径）；engine
  穷尽臂同步（装备臂移除 Zhufangshenshi 误加）。
- **登记差异**：无懈不可触发（上游 wuxie AI 权重 0，实战不戒）。

## 验收口径

- **回归已加入**：`swd_zhufangshenshi_distance_waiver`——绅视对 1 号
  使用后对标记目标出杀被接受且造成伤害；存档往返
  （zhufangshenshi_target 一致）。
- **全量通过**：engine-native **409 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**，DEPS_EXIT=0。
- **golden**：版本 93→94 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（纯版本位移）。
- **剩余**：swd 其余卡种（羊皮卷/伏符录/风雪刃/兰阁怪衣/岱火风轮/
  夏云彩绫/青龙令珠外的宝物等）逐批转换。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（13 文件，相对 noname-next 根）。
