# 2026-10-11 T11 星宿阶段2系续片：星囚（规则 v239/存档 v55）＋蓐收条目

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁延续批次链；主稿登记的下一切片二选一取星囚）。上游依据：`boss_xingqiu`（5485：init storage=false；trigger `player phaseUseBegin` locked——cost 取反 storage、content 把未横置的全部敌方 `link()`；另对 boss_mingxingzhu 授 boss_jiding）；携带者 boss_jinshenrushou 蓐收（1065：嬴|该/shen/hp5/skills `[boss_shenyi, boss_xingqiu, boss_baimang3]`）。

## 建模

- **星囚**（查询型 Noop，`BOSS_XINGQIU`）单臂（kernel Phase::Play begin 段）：持有者**本人**出牌阶段开始——`xingqiu_toggled` 取反；true 回合把未横置的敌方全部 `linked=true`+`LogEvent::Linked`（direct 直写，索链甲同款；只链不解——已横置保持）。敌方判定=`is_boss_side(t) != is_boss_side(holder)`。
- **蓐收条目**：`BOSS_GENERALS` 尾追加（def 索引 15，hp5）`[神裔, 星囚]`；boss_baimang3 白芒3=关卡流程面（已整族处置），条目不含。
- **xingqiu_toggled 新字段**（`PlayerState`，**存档 54→55**；archive 编/解码双表；canonical_hash 入参——字段新增批）。

## 登记差异

- 对 boss_mingxingzhu 授 boss_jiding 面：两武将均未入库（明星/急叮），登记不落。
- 上游 link() 走 turnOver 类事件链（含视觉）——native 直写 linked+日志（索链甲先例同口径）。
- 势力 shen→群目录近似（先例族）。

## 坑位实录

- **首版 Play 臂漏「持有者本人阶段」过滤**——`trigger: { player: phaseUseBegin }` 的 player 限定被忽略，每个玩家的 Play begin 都在翻转开关（0/2/3 的回合各翻一次→校验时开关已回 false 而横置面"恰好"成立）。回归断言抓出，补 `holder != player` 过滤。教训：**phase 触发技的 player 域必须过滤到持有者本人**，横置面绿不代表开关面绿。
- `debug_add_fellow(hp)` 是**追加新座位**（返回新 pid）而非改编现有座位——⑤段首版误用改编语义。
- `drain_choices` 的空应答会被 min>0 的 SelectCards 校验拒绝（返回值被忽略、选择静默挂起→后续 next_phase 全部 ChoicePending）——②③收回合前先清空手牌使弃牌阶段无询问；helper 临时插桩后已还原。
- 14 golden 重录脚本首跑 old 表误填（沿用 v237 旧值）被 assert 拦截，修正 v238 现值后转绿（防呆第三次生效）。

## 验证

| 项 | 结果 |
|---|---|
| 新回归 `boss_jinshenrushou_xingqiu_link_toggle` | 五段：①首回合三挑战者全横置/boss 本体不横置/toggled=true ②次回合取反不解除（只链不解）③第三回合再开横置保持 ④toggled+横置态存档 v55 往返 ⑤fellow（追加座位 4）属 boss 方不横置 |
| 既有回归 | boss 全滤 35/35；机械审计：无多重注册/零属性清零 |
| 版本/字段探针 | 字段批口径：238+新代码 14 全位移（字段源佐证）+237→239 版本源；重录后 2/2 绿 |
| 全量 | **585 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 238→**239**；存档 54→**55** |

## 遗留登记（不变）

- boss_stage 局 first=0 与虚失翻面臂自动推进交互；炎狱跨回合计数器精确口径。
- 下批：boss_buchun 不臣（复活书精，需死亡区复活路径）或星宿阶段 2 其余内容技（赤翼/星夏/白翼等）；相关将（炎帝/共工/句芒/祝融/明星）逐将入库。
