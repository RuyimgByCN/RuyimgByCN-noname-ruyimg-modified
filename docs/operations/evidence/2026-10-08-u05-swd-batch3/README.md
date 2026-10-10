# U05 swd 第三批：草药（规则 v87，2026-10-08）

swd 子包第三批转换：草药（♣3/♣4/♣5 三张，基本牌，global 距离 1）——
为距离 1 以内的受伤目标回复 1 点体力。前置：
[swd 第二批](../2026-10-08-u05-swd-batch2/README.md)。

## 上游语义核验（源码为准，card/swd/card.js）

`caoyao: { type: "basic", range: { global: 1 }, filterTarget: hp <
maxHp, content: target.recover() }`——可对距离 1 以内（含自己）的受伤
角色使用，回复 1 点体力。

## 改动（分开陈述）

- **代码已实现**：`CardKind::Caoyao`（编码 106：archive enc/dec、a0
  variant/ALL 同步）+ PlayCard 门控（恰 1 目标、存活且 hp<maxHp、
  `computed_distance ≤ 1`）+ 内联回复结算（Recover 事件走 Tao 同路，
  无宿主行为——纯基本牌直接结算）。
- **转换包**：cards.json 增 ♣3/♣4/♣5 三张草药 + source_hash/哈希重算
  （转换包现 8 张）。

## 验收口径

- **回归已加入**：`swd_caoyao_heals_wounded_in_range`——距离 1 的受伤
  目标回复 1（2→3）、满血目标拒绝（IllegalTargets）。
- **全量通过**：engine-native **402 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**，DEPS_EXIT=0。
- **golden**：版本 86→87 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（标准场景无草药，纯版本位移）。
- **剩余**：swd 其余 14+ 卡种（赤龙牙/归有介/草木灰等多步 content 需
  宿主原语增补）逐批转换。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（13 文件，相对 noname-next 根）。
