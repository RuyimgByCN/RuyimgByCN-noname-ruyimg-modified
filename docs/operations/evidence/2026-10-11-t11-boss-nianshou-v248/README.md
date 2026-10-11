# 2026-10-11 T11 年兽系首片：年瑞+猛态（规则 v248/存档 v58）＋年兽条目

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `64e9f0a8`；bianshen 勘验批（`db2c1aa9`）后年兽系首片）。上游依据：`boss_nianrui`（8643：phaseDrawBegin trigger.num+=2）；`boss_mengtai`（8524：五子技状态机——begin 准备置双 storage、draw/use 清位、discard 未用→cancel、end 未摸→draw(3)）；载体 boss_nianshou_heti 年兽合体（1151：shen/hp12/skills `[boss_nianrui, boss_mengtai, boss_nbianshen, boss_nbianshenx]`）。

## 建模

- **年瑞**（查询型 Noop，`BOSS_NIANRUI`）：Draw 臂固定摸 2 后补摸 2（藏目同位）。
- **猛态**（查询型 Noop，`BOSS_MENGTAI`）五臂（`TurnState.mengtai_draw_pending/use_pending` 双旗标承载）：
  - **准备阶段置位**（Judge begin 臂）：双旗标=true。
  - **摸牌阶段清位**：固定摸 2 前置 draw_pending=false——**跳摸路径（skip_next_draw/兽九跳）不清位**（上游 phaseDrawBegin 未触发同口径）。
  - **出牌阶段清位**（Play 臂）：use_pending=false——skip_play 跳过路径不经过 Play 臂 ✓。
  - **弃牌闸门**：生成口（3341 dispatch）use_pending=true → 机器不生成（庆祝族闸门同位延伸）。
  - **结束阶段补摸**：End 臂 draw_pending=true → 摸 3 + **旗标消费**（首版漏消费被回归抓出）。
- **年兽条目**：`BOSS_GENERALS` 尾追加（def 索引 25，hp12）`[年瑞, 猛态]`；形态轮换 boss_nbianshen（bianshen 勘验批已给承载结论：替补换将 assembly 同构）未落随下批。

## 登记差异

- 形态轮换（nbianshen/nbianshenx）未落（下批）；猛态 storage 语义以 TurnState 双旗标承载（跨回合持久化字段、随存档 v58）。
- 势力 shen→群目录近似（先例族）。

## 坑位实录

- TurnState 新字段三处构造点（engine.rs 352/5204/5291）——分批补齐时产生**双重插入+漏第三处**（E0062+E0063，lib test target 的内嵌 ctor 也算）；逐处精确修复。
- End 臂漏旗标消费（draw_pending 不清→回归断言抓出）。
- 14 golden 重录脚本 sha/judge old 哈希错位再犯（v238 同型，assert 拦截第四次数效）。

## 验证

| 项 | 结果 |
|---|---|
| 新回归 `boss_nianshou_nianrui_mengtai` | 四段：①年瑞+2（首回合手牌 10=6 初始+2 年瑞+2 摸牌；双旗标消费确认）②跳摸+跳用回合（判定阶段内置 skip 旗标→结束阶段摸 3+弃牌闸门跳过+旗标消费）③全程守恒 ④年瑞实例存档 v58 往返 |
| 既有回归 | boss 全滤 44/44；机械审计：无多重注册/零属性清零 |
| 版本/字段探针 | 字段批口径：247+新代码 14 全位移（字段源）+版本源；重录后 2/2 绿 |
| 全量 | **594 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | cargo 真实 exit 0 |
| 版本 | 规则 247→**248**；存档 57→**58**（TurnState 双旗标入档） |

## 遗留登记（不变）

- boss_stage 局 first=0 与虚失翻面臂自动推进交互；炎狱跨回合计数器精确口径。
- 下批：形态轮换 boss_nbianshen/nbianshenx（bianshen 勘验承载结论落地——assembly 重挂+hp/护甲保留+形态序列确定性轮转）与年兽五形态将（jingjue[继承 danshu]/renxing/ruizhi/baonu[changeHp 全局系]+shouyi[无距离 mod]）。
