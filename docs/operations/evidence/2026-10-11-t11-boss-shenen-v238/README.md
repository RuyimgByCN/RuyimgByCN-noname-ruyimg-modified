# 2026-10-11 T11 星宿阶段2系首片：神恩（规则 v238）＋太昊条目

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `af77503f`；隐藏链八技处置批（`b04b2eee`）勘察后的最小真实内容片）。上游依据：`boss_shenen`（6872：unique+global→boss_shenen2）；`boss_shenen2`：mod `targetInRange side→true` / mod `maxHandcard !side→num+1` / trigger `phaseDrawBegin !side→num++`；携带者 boss_taihao 太昊（990：风|伏羲/shen/hp6/skills `[boss_shenyi, boss_shenen, boss_qingyi]`）与 boss_yandi 炎帝等三处阶段 2 武将。

## 建模

- **神恩**（全局光环查询型 Noop，`BOSS_SHENEN`）三面：
  - **boss 方用牌无距离**：engine sha_range_free 链追加 + 顺手牵羊距离校验 `shenen_boss_free` 臂（条件=光环激活 && `core.is_boss_side(turn.current)`）。
  - **挑战者侧手牌上限 +1**：state hand_limit 尾臂（灵渠之后追加，加法修饰互不干扰）。
  - **挑战者侧摸牌 +1**：kernel Draw 臂固定摸 2 后追加（无脏补量之前，二者独立加法）。
  - **光环激活=场上存在神恩持有者**（`skills.keys().any`）——上游 global 子技自动全员发放的等价域化；**阵营承载**：新 `PlayerState::is_boss_side`（Boss 模式=boss 座位+fellows，其余=挑战者方；非 Boss 模式恒 false——光环仅 Boss 场生效）。
- **太昊条目**：`BOSS_GENERALS` 尾追加（def 索引 14，hp6）`[神裔, 神恩]`；boss_qingyi 青翼未落随下批；炎帝（yandi）等其余携带者待各自切片入库。

## 登记差异

- 上游 `side`（boss 方含全部 fellow 与 boss 换将体）以 native `Boss{boss,fellows}` 承载——阶段换将体（changeBoss 产物）在裸骨架下不存在，登记于关卡系统缺口。
- 光环激活域：上游 global 子技随技能存在即全员生效（含持有者死亡后至游戏结束）——native 以「存在持有者」近似（boss 死亡即终局，域等价）。
- 势力 shen→群目录近似（先例族）。

## 坑位实录

- 神恩 SkillDef **第五次**非表尾插入（灵渠 def 前）——冻结表测试拦截后移真表尾。五连犯后固化认知：该锚点错误模式在本会话脚本习惯中反复复现，后续批次脚本一律用数组闭合锚。
- 14 golden 重录脚本 sha_dodge 的 old 哈希误填 judge 的 v237 值——assert count=0 拦截（机械防呆第二次生效），修正后转绿。

## 验证

| 项 | 结果 |
|---|---|
| 新回归 `boss_taihao_shenen_aura` | 五段：①boss 方隔位杀可指定且命中 ②挑战者隔位杀距离拒（对照）③挑战者摸 3/boss 侧摸 2 ④挑战者上限=hp+1、boss 侧=hp ⑤神恩实例往返+守恒 |
| 既有回归 | boss 全滤 34/34；机械审计：无多重注册/零属性清零；lib 冻结表 10/10（def 移尾后） |
| 版本递增探针 | 237→238 全 14 位移；回退 237 复跑 **全绿**（位移唯一来源=版本字段） |
| 14 golden 重录 | 238 全量重录 2/2 绿（def 移尾后复跑仍绿） |
| 全量 | **584 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0（def 移尾后最终源码） |
| 版本 | 规则 237→**238**；存档不变 v54（is_boss_side 为纯查询 helper，无新字段） |

## 遗留登记（不变）

- boss_stage 局 first=0 与虚失翻面臂自动推进交互；炎狱跨回合计数器精确口径。
- 下批：boss_qingyi 青翼（太昊尾技）或 boss_xingqiu 星囚/boss_buchun 不臣（较重）；星宿阶段 2 武将（炎帝/共工/蓐收/祝融/句芒等）逐将入库。
