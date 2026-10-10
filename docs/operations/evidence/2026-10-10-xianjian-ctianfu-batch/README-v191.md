# U05 swd 第十八批切片（规则 v191，2026-10-10）

勾魂锣（gouhunluo）机关牌第三切片——倒计时标记类首例（天符挂附模
式复用）+kind 级 def 注册首例（card_kind_host_skills）。**机关牌池
勘误**：上游 list.js 实为 6 种 13 张（机关术×2/机关妖术×2/机关桶
×2/勾魂锣×2/指路小虎×2/木甲人×3），此前「2/4」登记有误——已切
3 种 5 张。前置：[木甲人批](README-v190.md)。

## 上游语义核验（源码为准，…/card/swd/card.js gouhunluo + skill.js gouhunluo/gouhunluo2）

- 牌：type:"jiguan"、wuxieable、filterTarget=!hasSkill("gouhunluo")
  （任意无标记角色）；content=目标 storage.gouhunluo=3、
  gouhunluo2=使用者、addSkill。
- `gouhunluo`：global phaseBegin forced——来源回合开始计数-1；归
  0=失去 1 体力+弃全部手牌+清挂附。intro 文案称「3 轮后」但实现按
  阶段计（每阶段 -1）——**原生跟随实现口径**。
- `gouhunluo2`：global dieBegin forced——来源死亡清标记。

## 改动（分开陈述）

- **宿主行为 1**：`SwdGouhunluoCard`（tag 218，无懈窗包装）——目
  标挂 3 阶段倒计时+授倒计时实例+本体弃置。PlayCard 校验：恰 1 名
  存活且无挂附目标（rounds==0）。
- **宿主行为 2**：`SwdGouhunluoSub`（tag 219，PhaseBegin forced
  global 定向 filter=event.player==source）——逐阶段 -1；归 0=
  LoseHp 1+弃全部手牌（真实弃置事件=械磐可触发）+清挂附清实例。
  **kind 级 def 注册首例**：card_kind_host_skills(Jiguan)=
  [gouhunluo_sub decl]。
- **状态**：`PlayerState.gouhunluo_rounds: u32`+
  `gouhunluo_source: Option<u8>`（入 canonical_hash+存档）。存档
  v40→**v41**。
- **cards.json**：swd_gouhunluo1 ♠9/swd_gouhunluo2 ♣7（上游原文花
  色）+rehash 两步；**版本** 190→191；a0 追加 218/219。
- **swd 存量**：计数 27→29（三处断言）+青龙令珠注入补 discard
  retain——16/16 全绿；木甲人获牌断言随池扩充修正（弃 1 获 2）。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`swd_gouhunluo_countdown_lose_hp_
  discard`（挂附三断言[rounds=3+来源+实例]→重复挂同目标拒→循环推
  进至归 0→失 1 体力+手牌清+挂附全清）。
- **全量通过**：engine-native **524 passed / 0 failed / 1 ignored**
  （v190 基线 523 +1；g1_shahai_swd 16/16）；依赖包 **63/0**。
- **golden**：版本 190→191+挂附字段入 canonical_hash——14 场景全
  mismatch，重录后 replay_scenarios 2/2。

## 剩余

- 机关牌 3 种 8 张未切（机关术×2/机关妖术×2/机关桶×2——机关术/
  妖术效果待核，机关桶=?）；御甲/偃师解锁需机关牌池扩充+计数基建；
  dujiang 毒牌种独立评估；长期项不变。

## 改动文件

见 [source-hashes-v191.txt](source-hashes-v191.txt)（15 文件）+
[test-logs-v191.txt](test-logs-v191.txt)。
