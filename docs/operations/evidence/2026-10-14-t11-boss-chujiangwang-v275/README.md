# T11 楚江王片（规则 v275，2026-10-14）

## 任务来源与上游依据

[进度主稿 T11](../../../architecture/native-progress.md)（地藏王批 6a025438 后续片）。上游依据：`boss_chujiangwang`（楚江王·厉温/male/qun/**hp4**/skills `[weimu, refankui, boss_bingfeng]`，apps/core/mode/boss.js:1306）；`boss_bingfeng`@3576（die→source 未翻面则 turnOver）；weimu/refankui=标准技能引用（mode 内 `boss_chujiangwang_weimu/fankui` 仅 audio 壳）。**勘正**：此前批次证据（v269 起）把楚江王误计入「已完成」——目录实无楚江王条目（rg 零命中），本批实际落包。

## 三技语义与原生落法

- **帷幕**（查询型单面，双消费点，**新落**——原生此前无帷幕）：黑色锦囊对持有者无效——①显式目标锦囊（顺手/拆桥/决斗/乐）目标不可指定（PlayCard 拒绝位 `IllegalTargets`，造异/热闷同族；上游 mod targetEnabled→目标不可指定近似登记）；②黑色 AOE 窗口前跳过（`weimu_blocks_aoe`，按 AOE 牌实体花色判定——万箭为红天然不触发）；无指向锦囊（五谷/桃园/无中/闪电/连环）无指向面天然不适用=登记。
- **反馈**（**标准 `FAN_KUI` def 复用**，魔道/马面反馈先例）：原生反馈语义=任意伤害后获得来源一张牌（手牌优先其次装备）——与界反馈一致，零新增代码。
- **冰封**（Death 触发遗言机，`MachineSnapshot::BossBingfeng` 新变体 tag80/机哈希 80/a0 变体 78，忘魂同位直写一步机）：你死亡时，killer 存活且未翻面→翻面（`face_down` 写+`TurnOvered`，风雪刃确认臂同款写法；已翻面不重复翻同上游 filter）。

条目 `("楚江王", Qun, &[帷幕, 反馈, 冰封], 4, Male)`（BOSS_GENERALS[53]）；`STANDARD_RULESET_VERSION` 274→**275**（存档不变 v63）。FROZEN 尾追加「帷幕」「冰封」（175–176 名）。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向回归 | `boss_chujiangwang_weimu_black_trick_block`（黑顺手拒/红拆可指定/黑南蛮 1 号零询问零伤害+2/3 Pass 各-1）、`boss_chujiangwang_fankui_reuse_gain`（trigger_confirm 确认→获得来源首张手牌）、`boss_chujiangwang_bingfeng_death_flip_source`（真死→来源 0 号翻面+往返保留）——3/3 绿（[日志](chujiangwang-targeted.log)） |
| 版本探针 | 275→274 回退复跑 golden **全绿**（2/2）=位移唯一来源是版本字段 |
| 14 golden | 程序化行级重录 14/14（[actuals](golden-v275-actuals.txt)），重录依据注释链追加 v275 条目（含勘正说明）；`replay_scenarios` 联动守护 2/2 |
| 全量 | **650/0/0**（647→650，新增 3 测）——[日志](engine-native-tests.log) |
| Gate 1b | 独立 target 目录 `RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` **exit 0**——[日志](gate1b-all-targets.log) |

## 已知边界与登记

- 帷幕目标不可指定近似（上游为使用后生效取消——仅剩其一时整卡取消与牌去向差异登记）；无指向黑锦囊面天然不适用登记。
- **坑位**：Death 事件无 source 字段——遗言机的伤害来源从 `EventData::Death { killer }` 在 behavior.start 捕获入机器。
- **十殿阎罗 11/13（勘正后）**：五官王/宋帝王/卞城王/泰山王/转轮王/孟婆/阎罗王/平等王/都市王/地藏王/楚江王✓。余：秦广王[判官/拘魂/望乡/还人——4 技含还人关卡面]/Shikieiki[映姬]（初判完，需逐片勘验落包）+庞统/诸葛亮专项+族级机制 3 项（需 D 级决策）。
