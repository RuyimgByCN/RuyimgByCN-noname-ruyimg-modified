# 2026-10-11 T11 十殿阎罗宋帝王片：黑绳/胜负（规则 v266）＋宋帝王条目（恩怨登记未落）

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁延续）。上游依据：`boss_songdiwang`（余|懃/qun/hp4/skills `[boss_heisheng, boss_shengfu, enyuan]`，boss.js:1316）；`boss_heisheng`@3591 / `boss_shengfu`@3605；`boss_songdiwang_enyuan` 仅 audio 桩——`enyuan` 本体=外部包组技（refresh renyuan 群继承 xinenyuan1/2 双分支：受伤后来源失 1 体或交手牌非♥摸 1），按绝策先例**登记未落随外部包批**。

## 建模

- **黑绳**（Death 触发 SkillDef，`BOSS_HEISHENG`，**遗言面首例**）：死亡前置窗口（Death 帧 BeforeTriggers，changan 氏同款；该窗口 AfterTriggers 不重排=Death 帧 plan_triggers 早退，遗言恰触发一次）——全部其他存活横置（索命直写同款）。上游 `die forceDie` 逐个 `link()` 同口径。**登记差异**：窗口先于取消判定——若死亡被取消仍已横置。
- **胜负**（查询型 Noop，`BOSS_SHENGFU`）：End begin 臂——首个装备区有坐骑牌（HorsePlus/HorseMinus）的其他存活弃置其中一张（随机目标/随机牌→首个有牌者/首张确定性近似登记；上游 logSkill 误标 "boss_heisheng" 为上游原样不复刻；equip6 宝物面随宝物栏机制登记）。
- **宋帝王条目**：`BOSS_GENERALS` 尾追加（def 索引 44，hp4，male）`[黑绳, 胜负]`——**条目只列已转换技**（守翼年兽先例；恩怨未落）。

## 遗言面首例说明

死亡触发技能此前无 catalog 先例（changan 氏为 host 内联）。Death 帧 stage 流=BeforeTriggers（遗言面 plan_triggers 查 `EventKind::Death`）→ AfterTriggers（Death 帧 plan_triggers 早退不重排）→ Intrinsic（DeathMachine）。黑绳为首个 Death 触发 SkillDef，机器=无挂起一步机（全体横置直写）。

## 验证

| 项 | 结果 |
|---|---|
| 新回归 `boss_songdiwang_heisheng_death_link_shengfu` | ①黑绳：宋帝王（fellow 位+chongzheng=0 真死）死亡 → 全部其他存活横置 ②胜负：0 号装 +1 马 → 1 号 End 弃其坐骑 ③往返 |
| 版本递增探针 | 普通批口径：266→265 回退复跑 golden **全绿**（位移唯一来源=版本字段） |
| 14 golden 重录 | 行级程序化重录 2/2 绿 |
| 全量 | **623 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 265→**266**；存档不变 v61 |

## 坑位实录

- **Boss 模式全座位重整复活拦真死**：宋帝王在 boss 位/fellow 位被 10 伤打至 -6 仍 alive（chongzheng 计数复活）——真死测试须 `new_boss_stage(chongzheng=0)`。
- **dying_save 询问 min=1 空应答被拒**：濒死救援询问须显式 Pass（drain_choices 空应答假死——同枳枉批 min≥1 反例族）。
- **debug_place 置入手牌非装备区**：装备须当前回合玩家 play 出（0 号回合装马，随后穿回合到 1 号 End 验胜负）。

## 遗留登记（本批新增）

- 黑绳：死亡被取消仍已横置（前置窗口语义）；黑绳机死亡者自身存活期触发（forceDie 死后面=时序不敏感）。
- 胜负：equip6 宝物面随宝物栏机制；上游 logSkill 误标不复刻。
- 恩怨：外部包组技（reenyuan 继承 xinenyuan1/2）未落随外部包批。
- 下批候选：卞城王（雷舟/雷斧/雷诛）、泰山王（复读/苦久/热恼）、转轮王（轮回/往生/反噬+魔道共用）——勘验均已完备。
