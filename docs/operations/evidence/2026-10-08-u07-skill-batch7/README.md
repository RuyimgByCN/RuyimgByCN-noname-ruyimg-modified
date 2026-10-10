# U07 技能批七：割据（刘焉）（规则 v117，2026-10-08）

玩点论杀武将技能声明批七：**割据**（geju）映射 `host:wudian_geju`——
回合开始强制摸「攻击范围外其他势力种数」张（势力=effective_faction，
无主将/unknown 排除；杀范围内他势力不计入）。未映射声明 2（duoqi/
youzhan 暂缓，另有 diaobingqianjiang 卡牌暂缓）。前置：
[U07 技能批六](../2026-10-08-u07-skill-batch6/README.md)。

## 上游语义核验（源码为准，archive/qita/sanguosha/noname/extension/玩点论杀/character/skill.js:302）

- `geju`：trigger player phaseBegin、frequent（强制）、filter/content
  同构——他人势力集（unknown 排除）中移除「杀范围内攻击者」的势力 →
  `draw(剩余种数)`。targetInRange(sha) = computed_distance ≤ 攻击者攻
  击范围（attack_range 原语齐备）。

## 改动（分开陈述）

- **代码已实现**：`WudianGeju`（tag 104）——TriggerMeta
  PhaseBegin/Normal（Judge begin 帧触发，plan_triggers 既有通路）+
  filter（owner==player）+ 单步机：收集他人 effective_faction 种数 →
  按**各攻击者自身攻距**逐个移除杀范围内他势力（q→owner 距离 ≤ q 的
  attack_range）→ `draw(剩余种数)`；空则不摸。mapping 接线（刘焉
  geju）。
- **登记差异**：上游 frequent 无确认（原生 optional:false 同口径）；
  势力判定用 effective_faction（Kingdoms=明置势力、其余=主将势力）——
  与地载阵同口径。
- **转换包**：cards.json 刘焉 geju mapping（skills 段登记），三哈希
  重算。

## 验收口径

- **回归已加入**：`liuyan_geju_out_of_range_faction_draw`（3 人局负
  例：相邻全在杀范围内→不摸，手牌=Draw 摸 2）+
  `liuyan_geju_out_of_range_counts`（4 人局正例：1 号魏距离 1 范围内
  不计入、2 号蜀距离 2 范围外计入 → Judge 摸 1 + Draw 2 = 3）。
- **全量通过**：engine-native **435 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**（content-runtime 33 + engine-ai 29 +
  content-native-loader 1）。
- **golden**：版本 116→117 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（割据不在标准场景，纯版本位移）。
- **剩余**：U07 未映射 2 条（duoqi/youzhan 暂缓：夺启依赖筑城标记+
  phaseUse 中止；诱战重流程）——11 项声明全量盘点点位见台账。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（17 文件，相对 noname-next 根）。
