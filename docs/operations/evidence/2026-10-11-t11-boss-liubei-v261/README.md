# 2026-10-11 T11 群雄乱入刘备片：枭雄/章武（规则 v261）＋刘备条目

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁延续）。上游依据：`boss_liubei`（刘|备/shen/hp8/skills `[xiaoxiong, boss_zhangwu]`，boss.js:1507）；`xiaoxiong`+子技 `xiaoxiong_damage`（boss.js:7674）；`boss_zhangwu`（boss.js:7701）。

## 建模

- **枭雄**（触发 `EventKind::CardUsed` AfterTriggers+一步机 `XiaoxiongMachine`，奇佐同款）：filter=非 cancelled、**他人**使用、牌种白名单（基本 7 种+锦囊 12 种含延时——上游 `get.type(card,"trick")` 延时算 trick；装备排除）；step=牌堆底实体入手（上游 createCard 凭空造牌→载体实体入手登记差异：花色点数随载体+消耗库顶实体；牌堆空静默）。副本入手不 emit CardUsed=无递归。
  - **子技 xiaoxiong_damage**：End begin 臂（shangshix2 同槽）——回合角色≠持有者且 `cards_used_play_phase==0` → 对其造成 1 伤害（上游 `countUsed()==0`→出牌阶段用牌计数==0 登记差异：响应/装备不计；刘备自己回合不触发）。
- **章武**（查询型 Noop，`BOSS_ZHANGWU`）：Damage commit 尾臂（damageEnd 族，伤逝臂后）——target 持有、来源存活且非自己 → 刘备弃手牌首张（若有）→ 来源弃手牌首张，无手牌则受 1 点反伤（`emit Damage{Some(target), src, 1}`）。
- **刘备条目**：`BOSS_GENERALS` 尾追加（def 索引 39，hp8，male）`[枭雄, 章武]`；势力 shen→群目录近似。

## 验证

| 项 | 结果 |
|---|---|
| 新回归 `boss_liubei_xiaoxiong_gain_and_damage` | ①入手：他人出杀→刘备获得副本（手牌+1）②子技负例：已用牌回合 End 不受伤 ③子技正例：2 号未用牌 End 受 1+刘备自己 End 不自伤 ④独立局：刘备自己用牌不入手 ⑤往返 |
| 新回归 `boss_liubei_zhangwu_discard_or_damage` | 场景 A：双方有手牌→各弃 1（章武弃先于枭雄入手相抵）无反伤；场景 B：来源无手牌→受 1 反伤；往返 |
| 版本递增探针 | 普通批口径：261→260 回退复跑 golden **全绿**（位移唯一来源=版本字段） |
| 14 golden 重录 | 行级程序化重录 2/2 绿 |
| 全量 | **613 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 260→**261**；存档不变 v60 |

## 坑位实录

- **测试流程错位**：`new_boss_stage(first=0)` 下 0 号先手——`advance_to_play(0)` 不跨任何回合，子技无从触发（首版误记跨回合全员-1）；未用牌正例须显式推到 2 号 End。
- **目标持闪开响应询问**：章武场景刘备弃牌源用「闪」→杀被 Pending 响应（非拒绝）——弃牌源改「桃」。
- **章武弃牌与枭雄入手相抵**：章武弃在 Damage commit 内、枭雄入手在 CardUsed AfterTriggers——同事件内先弃后拿，刘备手牌数不变（断言按相抵写）。

## 遗留登记（本批新增）

- 枭雄入手=牌堆底实体载体（花色点数随载体+消耗库顶）vs 上游 createCard 凭空造牌；白名单外牌种（G1 扩展锦囊/装备）不触发（装备上游亦排除 ✓，G1 锦囊 boss 场不入）。
- xiaoxiong_damage 未用牌口径=出牌阶段用牌计数（响应打出的杀/装备不计）vs 上游 countUsed() 全量。
- 章武：刘备 [1,∞] 自选弃牌数→恒 1 张；双方 chooseToDiscard AI 态度面→恒弃手牌首张（无装备面）；来源无手牌但装备有牌时不弃装备直受反伤（上游 he 含装备）。
- 下批候选：群雄乱入甄姬（踏水/凌波/娇孝/芳华四技）、诸葛亮（hp=Infinity 需 maxHp 建模先行勘验）、十殿阎罗族首勘、吕布变身系。
