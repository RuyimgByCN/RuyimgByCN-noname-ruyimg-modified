# 2026-10-11 T11 boss 内容第四批首片：饕餮贪欲（规则 v215）

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `465b6174`；饕餮批按指令最小切片——仅贪欲单技，藏目/集财(+switch) 语义已对照登记下批）。上游依据：`mode/boss.js` `boss_tanyu`（5871：phaseDiscardBefore cancel + group tanyu_hp phaseJieshuBegin isMaxHandcard → loseHp）与 `boss_taotie`（881：hp20 五技）。

## 建模

- `BOSS_GENERALS` 增**饕餮**（群/hp20/`[凶兽, 贪欲]`）——**登记**：藏目（摸牌基数=存活角色数）/集财(+switch)（hp≤半激活+回复后双摸）未落随下批。
- **贪欲**两面（权限型 Noop+查询）：
  1. **跳过弃牌阶段**（kernel `Phase::Discard` begin 臂直查持技者 → 提前 return，上游 `trigger.cancel()` 同口径）；
  2. **结束阶段全场最多手牌失 1 体**（`Phase::End` begin 臂）：`tanyu_is_max_hand`（抽函数，持有者手牌数≥任何其他存活者）→ emit `LoseHp 1`。
- FROZEN 尾加「贪欲」（69 名）。

## 登记差异

- is_max=「≥任何其他存活者」（上游 `isMaxHandcard` 默认口径一致）。

## 验证

| 项 | 结果 |
|---|---|
| 回归 `boss_taotie_tanyu_skip_discard_and_max_hand_lose_hp` 三局 | ①跳弃牌（5 张保留）；②最多失体（resolution 自动推进多轮累计，断言 hp<20——End 帧后自动连推，非精确单次）；③非最多面状态断言（0 号 3>饕餮 1 → is_max=false；②局对照=true）——**注意**：③用状态面直验因饕餮摸牌必然随轮反超成最多，e2e 停帧不可控 |
| 既有回归 | 覆泰2/延妒/覆泰/逐厄/混沌/烛阴/关卡全部不变；native_kernel 217/0 |
| 版本回退探针（215→214 复跑 golden） | 2/2 全绿（位移唯一来源=版本字段） |
| 14 golden 重录 | 全部更新后 2/2 绿（重录依据注释入文件头） |
| 全量 | **562 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 214→**215**（存档不变 v48） |
