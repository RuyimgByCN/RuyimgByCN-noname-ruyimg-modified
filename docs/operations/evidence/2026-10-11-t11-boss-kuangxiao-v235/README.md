# 2026-10-11 T11 四象系末片首刀：狂笑（规则 v235）＋白虎条目

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（四象系末片第一刀；领取延续本会话批次链）。上游依据：`mode/boss.js` `boss_kuangxiao`（5520：mod targetInRange=true + mod selectTarget sha range[0..1]=-1/-1 + playerEnabled 友方禁指）；`boss_baihu`（1056：shen/hp4/skills `[boss_shenyi, boss_kuangxiao, boss_baimang2]`）；`boss_baimang2`（6633：dieBegin hide + 换位重生至 boss_jinshenrushou）。

## 建模

- **狂笑**（查询型 Noop，`BOSS_KUANGXIAO`）两面（engine PlayCard 校验段）：
  - **无距离**：sha_range_free 链追加（飞刃/飞弹/天弦同位；单目标与多目标 skip(1) 逐目标校验共用该链）。
  - **目标数无限**：`kuangxiao_inf` 臂（天弦 `tianxian_inf` 同构）→ max_targets=usize::MAX。
- **白虎条目**：`BOSS_GENERALS` 尾追加（def 索引 11）`[神裔, 狂笑]`（神裔共用 v233 面）。
- **白芒2**：dieBegin hide + 换位重生链=关卡流程面（同青木2），登记不落内核，白虎条目不含。

## 登记差异

- 势力：上游 shen → 群目录近似（Faction 枚举无神势力；烛阴空势力→群先例同族）。
- 友方禁指面（playerEnabled sha 不能指定 isFriendOf 目标）：boss 侧同队关系网未入原生 PlayCard 校验（fellows 关系既有边界），未建模。
- 上游 range[0]=-1（最小目标数 0=可空指）未建模——原生杀 targets 非空校验保持（出牌语义不变面）。

## 坑位实录

- 狂笑 SkillDef 首版**第三次**误插非表尾（雷击 def 前）——冻结表测试全量红拦截，移真表尾转绿。三连坑已固化教训：新增 SkillDef 锚点必须用数组闭合处（真表尾），禁用「最后一个已知 def」。
- 回归首版 ②多目标段被 ShaLimit 拒（同回合已用一杀）——推进新回合后重试转绿。
- 14 golden 对旧版本哈希的探针失败为预期行为（235 期望 vs 234 运行时），非 def 移尾回归——技能表序不入 14 场景哈希（v234 同证）。

## 验证

| 项 | 结果 |
|---|---|
| 新回归 `boss_baihu_kuangxiao_sha_range_and_targets` | 四段：①隔位（距离 2>攻击范围）杀 3 号可指定且命中 ②三目标齐中[新回合避开杀限+全清手] ③非持有者多目标杀被拒 ④存档往返+守恒 |
| 既有回归 | boss 全滤 31/31；机械审计：无多重注册/零属性清零 |
| 版本递增探针 | 234→235 全 14 位移；回退 234 复跑 **全绿**（位移唯一来源=版本字段） |
| 14 golden 重录 | 235 全量重录 2/2 绿（def 移尾后复跑仍绿） |
| 全量 | **581 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0（def 移尾后最终源码） |
| 版本 | 规则 234→**235**；存档不变 v53（查询型，无新字段/快照变体） |

## 遗留登记（不变）

- boss_stage 局 first=0 与虚失翻面臂自动推进交互；炎狱跨回合计数器精确口径。
- 下批：白虎续片/朱雀首片（boss_fentian 焚天+boss_chiyan 赤炎系）或玄武首片（boss_lingqu 灵渠/boss_xuanlin 玄麟系；boss_xingqiu 星囚）。
