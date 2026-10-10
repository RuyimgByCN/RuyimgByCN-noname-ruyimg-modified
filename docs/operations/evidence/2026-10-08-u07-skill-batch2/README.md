# U07 技能批二：率言（费诗）（规则 v112，2026-10-08）

玩点论杀武将技能声明批二：**率言**（费诗 nsshuaiyan）映射
`host:wudian_shuaiyan`——他人回复体力后（回复者非你且你非当前回合玩
家），回复者二选一：令你摸一张 / 令你弃置其一张牌。未映射声明 8→7。
前置：[U07 技能批一](../2026-10-08-u07-skill-batch1/README.md)。

## 上游语义核验（源码为准，archive/qita/sanguosha/noname/extension/玩点论杀/character/skill.js:3）

- `nsshuaiyan`：trigger global recoverAfter、filter 回复者≠你 &&
  currentPhase≠你；content 回复者 chooseControl（选项一「令费诗摸一
  张」/选项二「令费诗弃置你一张牌」）→ 选项一 `player.draw()`、选项二
  `player.discardPlayerCard(trigger.player, true, "he")`（被弃者自选，
  forced）。AI bool（态度+牌数推导）为决策逻辑不迁移——玩家自选。

## 改动（分开陈述）

- **代码已实现**：`HostBehavior::WudianShuaiyan`（tag 98）——
  TriggerMeta `EventKind::Recover / Timing::Normal`（global 触发：
  plan_triggers 按 (owner, def) 实例遍历，filter 排除回复者=你/当前回
  合=你/回复者死亡）；`machine()` target=回复者（Recover 事件上下文经
  `start(owner,&event)` 携带）；三段机（回复者 PickTag 二选一 → Tag0
  费诗摸 1 / Tag1 问回复者弃一张 → TargetChoice 弃置）；snap 派生两段
  （observer=target）。包 cards.json：费诗 nsshuaiyan mapping 接线
  （skills 段登记），npack 三哈希重算。
- **登记差异**：上游 chooseControl 的 AI 默认项（attitude 推导）不迁
  移（玩家自选，各批同口径）；discardPlayerCard 的「forced」=被弃者必
  须选一张（原生 min1 不可取消同义）。
- **费诗目录技能更新**：mapping 接线后目录 skills=
  ["nsshuaiyan","moshou"]（既有三处断言同步）。

## 验收口径

- **回归已加入**：`feishi_shuaiyan_recover_trigger_two_choice`——1 号
  回合 1 号吃桃回血触发（observer=1）→ ①选项一费诗摸 1 → ②再次触发
  选项二回复者弃 1 → ③负例：0 号自己回合自己回血不触发。
- **全量通过**：engine-native **429 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**（content-runtime 33 + engine-ai 29 +
  content-native-loader 1）。
- **golden**：版本 111→112 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（率言不在标准场景，纯版本位移）。
- **剩余**：U07 未映射 7 条（筑城/夺启/绝道/割据/抗yin/车阵/诱战）。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（17 文件，相对 noname-next 根）。
