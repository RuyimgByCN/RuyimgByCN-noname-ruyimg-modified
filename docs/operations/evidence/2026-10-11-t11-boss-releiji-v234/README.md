# 2026-10-11 T11 四象系续片：雷击（规则 v234）＋青木2 处置登记

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `d558e08e`；四象系续片——青龙 releiji 雷击+boss_qingmu2 青木处置）。上游依据：`character/shenhua/skill.js` `releiji`（界雷击，audioname boss_qinglong：trigger `useCard/respond` 且 `card.name=="shan"` → chooseTarget 一名其他角色 → 判定：黑桃 -4/梅花 -2 → 梅花=持有者回复 1+目标 1 雷伤、黑桃=目标 2 雷伤、其他无事）；`mode/boss.js` `boss_qingmu2`（6253：dieBegin hide + global dieAfter 换位重生）。

## 建模

- **雷击**（查询型 Noop，`BOSS_RELEIJI`；青龙条目扩为 `[神裔, 雷击]`）：杀/万箭两个闪消费位内联臂（共享 `releiji_on_shan` helper）——打出闪后座位序下一名其他存活角色确定性判定（铁骑内联口径，无鬼道/天妒改判窗）：黑桃→2 点雷伤；梅花→`ctx.recover` 回 1+1 点雷伤；其他无事。无双第二闪重入本臂=每张闪各触发一次（上游同口径）。
- **青木2**：死亡隐藏（hide）+换位重生（changeSeat/init）=关卡流程面，属 Boss 模式裸骨架缺口（T12 族），登记不落内核——青龙条目不含此技。

## 登记差异

- 上游 chooseTarget 自由选目标 → 座位序首名确定性近似（随机/选择近似先例族）。
- 触发半面：上游 useCard+respond 两路——原生闪无主动使用面，useCard 半面由模型承载；挂钩面=杀+万箭两闪消费窗（护驾代出等其余打出窗未挂钩，登记）。
- 判定为铁骑内联口径：无鬼道/天妒改判窗（先例同族）。

## 坑位实录

- 雷击 SkillDef 首版再次误插在神裔 def **之前**（同 v233 坑：尾巴中段插入顶后既有 def 索引）——冻结表测试全量红当场拦截，移至技能表真表尾后转绿。教训固化：新增 SkillDef 锚点必须用「真表尾（数组闭合）」，禁用「最后一个已知 def」。
- 回复效果误用裸 `emit(EventData::Recover)`（不落 hp）——kernel 回复 API=`ctx.recover(...)`（内部应用并 emit），改后转绿。

## 验证

| 项 | 结果 |
|---|---|
| 新回归 `boss_qinglong_releiji_thunder_judge` | 六段：①梅花=持有者回 1+攻击者（座位序下一名）1 雷伤 ②黑桃=2 雷伤、不回 ③红桃=无事 ④非持有者响应闪对照不触发 ⑤万箭闪响应同触发（2 号=雷击 1+万箭 1；3 号=万箭 1）⑥存档往返+守恒 |
| 既有回归 | boss 全滤 30/30；机械审计：无多重注册/零属性清零/无同名重复 |
| 版本递增探针 | 233→234 全 14 位移；回退 233 复跑 **全绿**（位移唯一来源=版本字段） |
| 14 golden 重录 | 234 全量重录 2/2 绿（def 移尾复跑仍绿——技能表序不入 14 场景哈希） |
| 全量 | **580 通过 / 0 失败 / 1 ignore**（2026-10-11 本机，`cargo test --offline -p engine-native`；冻结表测试经 def 移尾后含于内） |
| Gate 1b | exit 0（def 移尾后最终源码复跑） |
| 版本 | 规则 233→**234**；存档不变 v53（查询型，无新字段/快照变体） |

## 遗留登记（不变）

- boss_stage 局 first=0 与虚失翻面臂自动推进交互；炎狱跨回合计数器精确口径。
- 下批：四象系末片（青龙 boss_qingmu3 阶段变体/白虎首片 boss_kuangxiao 狂笑+boss_baimang 白芒；或朱雀/玄武首片）。
