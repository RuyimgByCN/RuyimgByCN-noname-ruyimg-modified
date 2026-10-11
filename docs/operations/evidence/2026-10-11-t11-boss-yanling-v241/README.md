# 2026-10-11 T11 炎灵陪将片：回火+复燃（规则 v241）＋焰灵条目

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `8e8f94be`）。上游依据：`boss_huihuo`（6980：mod cardUsable sha num+1；global boss_huihuo2 dieAfter——死亡者敌方全体 `damage(nosource,fire,3)`）；`boss_furan`（7073：global boss_furan2 dying chooseToUse viewAs tao——焰灵濒死时其敌方可用红色牌当桃救）；携带者 boss_yanling 焰灵（1030：male/shen/hp4/skills `[boss_huihuo, boss_furan]`——上游无神裔，陪将不在神裔名单）。译名勘误：上游戏称「回火」（指令面写作「回祸」，以 10973 译名为准）。

## 建模

- **回火**（查询型 Noop，`BOSS_HUIHUO`）两面：
  - **杀上限 +1**：engine 杀限 reject 臂的元宝肉同款额度——`SkillInstance.once_used` 作每回合一次额外额度（`reset_turn_limits` 随本人回合开始清零），允许后 `mark_huihuo` 置位（与元宝肉标记并列消费）。
  - **死亡反噬**：DeathMachine 死亡结算——技能实例移除**前**捕获 `had_huihuo`，`PlayerDied` 日志后敌方全体（`is_boss_side` 异侧、存活）各受 3 点无来源火伤（`damage_data_with(None, t, 3, Fire)`）。
- **复燃**（查询型 Noop，`BOSS_FURAN`）：`is_tao_like` 追加臂——濒死者持有复燃 && 响应者与其异侧 && 红色牌 → 可当桃入濒死救援选项（急救块同位）。
- **焰灵条目**：`BOSS_GENERALS` 尾追加（def 索引 17，hp4）`[回火, 复燃]`。

## 登记差异

- 上游 boss_huihuo2 的 damage 带 `animate=false`/logv 展示面——纯 UI，不落。
- boss 死亡即挑战者胜终局（截停事件队列）——回火反噬在「焰灵为 boss 本体」时不可观测；引擎语义不受影响（队列按序处理，终局先行），**回归以 fellow 线（座位 4 追加+目录装配）验证反噬**，登记此测试结构边界。

## 坑位实录

- 测试首版三误：sha4 由非当前回合玩家提交（PlayCard 仅回合玩家可提交——返回 Rejected 后续算术全错）；boss 本体死亡即终局截停反噬队列（hp 断言假失败）；`drain_choices` 空应答被 min 校验拒的老坑未犯（濒死链逐个显式 Pass）。
- 指令面技能名「回祸」与上游译名「回火」不一致——以译名为准（10973）。

## 验证

| 项 | 结果 |
|---|---|
| 新回归 `boss_yanling_huihuo_furan` | 三段：①回火杀限+1（第二杀命中、第三杀拒）②复燃（敌方红牌入濒死救援选项；放弃后焰灵死亡→挑战者胜终局）③回火反噬（fellow 线：fellow 死亡后敌方 0/2/3 各 4→3 火、boss 方同侧不受、非 boss 死亡不终局）+存档往返+守恒 |
| 既有回归 | boss 全滤 37/37；机械审计：无多重注册/零属性清零 |
| 版本递增探针 | 240→241 全 14 位移；回退 240 复跑 **全绿**（位移唯一来源=版本字段） |
| 14 golden 重录 | 241 全量重录 2/2 绿 |
| 全量 | **587 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 240→**241**；存档不变 v56（无新字段/快照变体） |

## 遗留登记（不变）

- boss_stage 局 first=0 与虚失翻面臂自动推进交互；炎狱跨回合计数器精确口径。
- 下批：星宿阶段 2 内容技（boss_chiyi 赤翼[炎帝，回合 3/5/7 编排族——青翼同族待勘]/boss_xingxia 星夏[祝融，对炎灵专用面]/boss_baiyi 白翼[句芒，回合 5 编排]）；相关将逐将入库。
