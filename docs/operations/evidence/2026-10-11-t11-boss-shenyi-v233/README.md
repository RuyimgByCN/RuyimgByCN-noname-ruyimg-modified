# 2026-10-11 T11 四象系首片：神裔（规则 v233）＋青龙条目

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `d4025df1`；四象系首片最小切片——神裔两面+青龙条目，releiji 雷击/boss_qingmu2 青木未落随下批）。上游依据：`mode/boss.js` `boss_shenyi`（6815：mod judge `_status.event.type=="phase"` 判定反转 + trigger `turnOverBefore` filter `!isTurnedOver` 取消翻面，priority 20 forced）；`boss_qinglong`（962：群/hp4/skills `[boss_shenyi, releiji, boss_qingmu2]`）。上游 `type=="phase"` 考证：仅判定阶段延时锦囊判定以 `player.judge(card).set("type","phase")` 打标（library content.ts:4968 phaseJudge 段）——技能自判定（炎狱/中毒等自定义 type）不受影响。

## 建模

- **神裔**（查询型 Noop，`BOSS_SHENYI` 常量；BOSS_GENERALS 尾追加**青龙** 群/hp4/`[神裔]`，def 索引 10）：
  - **判定反转面**：apply_judgement 五臂（乐/兵粮/洪水/归有界/闪电）效果条件一并短路（`!shenyi_voids &&` 前缀）——持有者判定阶段延时锦囊恒不生效；牌去向不变（锦囊照常弃置、闪电照常传下家、判定牌照常弃置/天妒照常）。上游 mod 只改 result.bool，与逐臂短路消费面等价（bool 非 true 即不生效）。
  - **翻面取消面**：commit_turn_over_before 神裔臂（凶兽同位，enabled 读法一致）——仅**正→背**取消（上游 filter `!isTurnedOver`），背→正照常翻回；虚失翻回臂不受影响。
- SkillDef 追加纪律：首版误插在司圃 def **之前**（尾巴中段插入、司圃 def 索引被顶后一位，冻结表测试当场红）——移至技能表真表尾（司圃之后）后转绿；FROZEN 名单尾加「神裔」（86 名）。
- 洪荒之力禁用交互（honghuangzhili disable/enable boss_shenyi）：属未转换技，登记差异未落。

## 登记差异

- 洪荒之力对神裔的禁用交互未建模（honghuangzhili 未落包）。
- 上游 AI 面（noturn/effect 权重）不在引擎范围。

## 验证

| 项 | 结果 |
|---|---|
| 新回归 `boss_qinglong_shenyi_judge_void_and_never_flipped` | 五段：①乐（黑桃顶）不跳过 1 号出牌阶段[首回合直达 Discard 即失败守卫] ②闪电（黑桃 2–9 顶）恒不命中、传 2 号判定区 ③非持有者对照：乐照常跳过出牌（标准局）④正→背取消、背→正照常翻回 ⑤神裔实例/翻面态/牌区守恒往返 |
| 既有回归 | boss 全滤 29/29（含 v232 批复位测试）；机械审计：无多重注册、boss_*/doudizhu_* 零属性清零、`--list` 无同名重复（延续 v232 缺陷修复纪律） |
| 版本递增探针 | 232→233 全 14 位移；回退 232 复跑 **全绿**（位移唯一来源=版本字段） |
| 14 golden 重录 | 233 全量重录 2/2 绿 |
| 全量 | **579 通过 / 0 失败 / 1 ignore**（2026-10-11 本机，`cargo test --offline -p engine-native`；native_kernel 234） |
| Gate 1b | `RUSTFLAGS="-D warnings" cargo check -p engine-native --lib` exit 0（独立 CARGO_TARGET_DIR，用后即删） |
| 版本 | 规则 232→**233**；存档不变 v53（查询型两面，无新字段/快照变体） |

## 遗留登记（不变）

- boss_stage 局 first=0 与虚失翻面臂自动推进交互；炎狱跨回合计数器精确口径。
- 下批：四象系续片（青龙 releiji 雷击/boss_qingmu2 青木；或白虎首片 boss_kuangxiao 狂笑/boss_baimang 白芒）。
