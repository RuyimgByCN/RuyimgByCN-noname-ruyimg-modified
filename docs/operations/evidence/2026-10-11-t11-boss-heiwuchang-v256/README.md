# 2026-10-11 T11 阴间系黑无常片：鬼计/太平/索命/吸行（规则 v256）＋黑无常条目

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁延续批次链；精觉 v252 后阴间系第二片）。上游依据：`boss_heiwuchang`（范|无救/male/shen/hp9/skills `[boss_guiji, boss_taiping, boss_suoming, boss_xixing, boss_bianshen4]`）；`boss_guiji`（9092：phaseJudgeBegin 弃己判定区随机一张）；`boss_taiping`（8975：phaseDrawBegin num+=2）；`boss_suoming`（8941：phaseJieshuBegin choose [1,num] 未链接目标 → 逐一 link）；`boss_xixing`（9020 附近：phaseZhunbeiBegin direct——choose 已链接其他目标 → 1 雷伤+回 1）。

## 建模

- **鬼计**（查询型 Noop，`BOSS_GUIJI`）：Judge begin 臂——弃自己判定区首张（随机→首张确定性近似登记）。
- **太平**（查询型 Noop，`BOSS_TAIPING`）：Draw 臂 +2（年瑞同位）。
- **索命**（查询型 Noop，`BOSS_SUOMING`）：End 臂——全部未链接异侧存活横置（linked 直写+Linked 日志，索链甲同款；[1,num] 自选→全部确定性近似登记）。
- **吸行**（查询型 Noop，`BOSS_XIXING`）：Judge begin 臂——座位序首个已链接异侧受 1 雷伤 + 持有者回 1（目标自选→首个已链接确定性近似登记）。
- **黑无常条目**：`BOSS_GENERALS` 尾追加（def 索引 33，hp9）`[鬼计, 太平, 索命, 吸行]`；bianshen4 死亡换将=已处置关卡面。
- **登记差异**：鬼计随机→首张；索命 [1,num] 自选→全部；吸行目标自选→首个已链接（确定性近似先例族）；势力 shen→群目录近似。

## 坑位实录

- **条目索引连锁位移**：黑无常插在索引 33——v255 测试的牛头 nt_def 偏移 +33 需顺移 +34（插入将后续全部 +1，既有测试须同步）。机械审计未覆盖此错（名字对、索引错）——按名断言反而绿、按 hp 断言假失败定位。
- 回归流程时序：索命在 End 臂——断言前须驱动至 1 号 End（Draw 后两相）；吸行回复可见性须前置掉血（条目 hp9=上限，clamp 吞回复）。

## 验证

| 项 | 结果 |
|---|---|
| 新回归 `boss_heiwuchang_guiji_taiping_suoming_xixing` | 五段：①鬼计（判定区乐被弃）②太平（摸牌 4=2+2）③索命（End 后 0/2/3 全横置、本体不横置）④吸行（次回合判定锚：0 号受 1 雷伤 4→3、持有者回 1 8→9）⑤四技实例存档往返+守恒 |
| 既有回归 | boss 全滤 52/52（牛头偏移顺移后）；机械审计：无多重注册/零属性清零 |
| 版本递增探针 | 256→255 全 14 位移；回退 255 复跑 **全绿**（位移唯一来源=版本字段） |
| 14 golden 重录 | 256 全量重录 2/2 绿（行级程序化第六次） |
| 全量 | **602 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | cargo 真实 exit 0 |
| 版本 | 规则 255→**256**；存档不变 v59（无新字段/快照变体） |

## 遗留登记（不变）

- boss_stage 局 first=0 与虚失翻面臂自动推进交互；炎狱跨回合计数器精确口径。
- 下批候选：马面批（鬼计共用/反馈[标准技核对]/炼狱/绝策[refresh 包]）或年兽收尾（baonu 暴怒复杂面批）。
