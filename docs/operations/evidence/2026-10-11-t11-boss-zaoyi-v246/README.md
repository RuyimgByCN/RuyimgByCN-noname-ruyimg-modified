# 2026-10-11 T11 星宿阶段2系造异片：造异（规则 v246）＋颛顼条目

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `a0bba55c`）。上游依据：`boss_zaoyi`（5221：global dieAfter——玄冥/共工死亡且场上无存活同名将 → holder 摸 4+addSkills boss_zaoyi_hp；mod targetEnabled——玄冥在场敌方不可为锦囊目标、共工在场敌方不可为基本牌目标）；`boss_zaoyi_hp`（子技：回合开始敌方中体力最低者全体 loseHp(min)）；携带者 boss_zhuanxu 颛顼（1133：姬|颛顼/shen/hp6/skills `[boss_shenyi, boss_shenen, boss_zaoyi]`）。**勘误**：玄冥（boss_shuishenxuanming 嬴|禺强/hp5）携带的是自润（zirun）+玄麟3——非造异；造异携带者=颛顼。

## 建模

- **造异**（查询型 Noop，`BOSS_ZAOYI`）三面：
  - **死亡触发**：DeathMachine 死亡结算（`PlayerDied` 后，回火臂同位）——死亡者武将名∈{玄冥,共工}（BOSS_GENERALS 段按名查；武将槽跨死亡持久）且场上无存活同名将 → 造异持有者各摸 4 + `skills.insert(造异异变)`（中毒/蛇毒运行时授技先例）+`SkillToggled` 日志。
  - **targetEnabled 双门控**（engine PlayCard 校验前置段）：玄冥在场 → 敌方不可为锦囊目标（非基本非装备类）；共工在场 → 敌方不可为基本牌目标（Sha/FireSha/LeiSha/Shan/Tao/Jiu）；装备不受限。
  - **造异异变**（`BOSS_ZAOYI_HP` 运行时授予技，FROZEN 登记）：Judge begin 臂——敌方（异侧存活）体力最低值 min>0 → 全体 hp==min 者 `LoseHp(min)`（失去全部体力→濒死链）。
- **颛顼条目**：`BOSS_GENERALS` 尾追加（def 索引 23，hp6）`[神裔, 神恩, 造异]`——纯条目+造异（神恩臂通用性复用 v243 数据点）。
- **登记差异**：玄冥半面（锦囊门控+死亡触发另一半）待玄冥入库（自润批）激活——门控按名查表天然支持；上游 getEnemies 顺序/line 展示面不落；势力 shen→群目录近似。

## 坑位实录

- **`answer_cards` 会把濒死救援询问当弃牌应答**（从选项挑牌→用桃救人）——循环推进里救援询问必须显式 Pass（按 `prompt=="dying_save"` 分流），否则「全体放弃救援」前提失效（本批 hp2=1 假失败根因，日志 dump 定位）。
- 造异异变为**永续授予技**——授技后的首个颛顼判定锚即触发（测试时序漏算首波）。
- random 挑战者武将不确定性：三家挑战者钉成共工 def 消除干扰（绝红对空装备无副作用）。
- `min()` 迭代器模式 `Some(&min)`/`Some(min)`、`Some(ref n)` 借用比较等小类型错三处（编译期拦截）。

## 验证

| 项 | 结果 |
|---|---|
| 新回归 `boss_zhuanxu_zaoyi_grant_and_exec` | 五段：①共工存活基本牌门控（sha→敌拒、tao 自目标不受限）②共工死亡→颛顼摸 4+造异异变授予 ③门控解除（sha→敌 Accepted）④异变波（压 0/2 至 2、3 保持 5——最低者 0/2 失去全部体力死亡、非最低者不触发、挑战者未全灭不终局）⑤造异异变实例存档往返+守恒 |
| 既有回归 | boss 全滤 42/42；机械审计：无多重注册/零属性清零 |
| 版本递增探针 | 245→246 全 14 位移；回退 245 复跑 **全绿**（位移唯一来源=版本字段） |
| 14 golden 重录 | 246 全量重录 2/2 绿 |
| 全量 | **592 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | cargo 真实 exit 0 |
| 版本 | 规则 245→**246**；存档不变 v57（运行时授技无新字段/快照变体） |

## 遗留登记（不变）

- boss_stage 局 first=0 与虚失翻面臂自动推进交互；炎狱跨回合计数器精确口径。
- 下批候选：boss_zirun 自润（玄冥依赖技，随玄冥入库）/boss_xuanming 玄冥条目（造异玄冥半面激活）；或星宿阶段 2 系收尾清点。
