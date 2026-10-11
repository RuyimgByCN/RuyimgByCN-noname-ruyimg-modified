# 2026-10-11 T11 四象系玄武首片：灵渠（规则 v237/存档 v54）＋玄武条目

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（四象系玄武首刀；延续本会话批次链）。上游依据：`mode/boss.js` `boss_lingqu`（5291：damageEnd draw+storage++；mod maxHandcard `num+storage`；subSkill cancel damageBegin4 `num>1 → num=0` priority -11）；`boss_xuanwu`（1107：shen/hp4/skills `[boss_shenyi, boss_lingqu, boss_xuanlin2]`）；`boss_xuanlin2`（6435：dieBegin hide + 换位重生链）。

## 建模

- **灵渠**（查询型 Noop，`BOSS_LINGQU`）三臂：
  - **受击计数+摸 1**：commit_damage damageEnd 段（逐厄臂同位，目标域）——每次受伤（含归零后 0 点）摸 1、`PlayerState.lingqu_count` +1。
  - **大额伤害归零**：commit_damage 增幅后/统计前臂——终值>1 → amount=0（上游 damageBegin4 低优先级=终值口径；0 点伤害仍完成事件→摸 1/计数照常、turn_damage 统计 0）。
  - **手牌上限 +X**：hand_limit 尾臂 `limit += lingqu_count`（无脏/兽九修饰保留后追加）。
- **玄武条目**：`BOSS_GENERALS` 尾追加（def 索引 13）`[神裔, 灵渠]`。
- **玄麟2（boss_xuanlin2）**：dieBegin hide+换位重生=关卡流程面（同青木2/白芒2），登记不落内核。

## 存档与哈希（字段新增批）

- `PlayerState.lingqu_count: u8` 新字段（**存档 53→54**；archive 编/解码双表；canonical_hash 入参——**字段新增批**：14 golden 位移=新字段入哈希+版本字段两源，v230 口径，探针「位移唯一来源=版本字段」不适用）。实测：236+新代码全 14 位移（字段源）、237+新代码再次全位移（版本+字段源），两组哈希互异佐证两源独立。

## 坑位实录

- 灵渠 SkillDef **第四次**非表尾插入（fentian def 前）——冻结表测试拦截，移真表尾。
- 14 golden 重录脚本 old 哈希表误用上一批（v236）时期的值——assert count=0 拦截，改用 v236 现值后转绿（机械 assert 防呆生效）。

## 验证

| 项 | 结果 |
|---|---|
| 新回归 `boss_xuanwu_lingqu_damage_counter` | 五段：①3 点直伤归零（hp 不变）+摸 1+计数 1 ②1 点伤害照常（hp-1+摸 1+计数 2）③2 点直伤归零（终值门槛）④手牌上限=hp3+计数3=6 ⑤存档 v54 往返（计数持久）+守恒 |
| 既有回归 | boss 全滤 33/33；机械审计：无多重注册/零属性清零；lib 冻结表 10/10 |
| 版本/字段探针 | 字段批口径：236+新代码全 14 位移、237 再次全位移（两源互异佐证） |
| 14 golden 重录 | 237 全量重录 2/2 绿 |
| 全量 | **583 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 236→**237**；存档 53→**54** |

## 遗留登记（不变）

- boss_stage 局 first=0 与虚失翻面臂自动推进交互；炎狱跨回合计数器精确口径。
- 下批：玄武续片（boss_xuanlin3/云屏阶段变体）或朱雀续片（boss_chiyan2 赤炎）；四象系至此四条目全部入目录（青龙/白虎/朱雀/玄武 def 索引 10–13）。
