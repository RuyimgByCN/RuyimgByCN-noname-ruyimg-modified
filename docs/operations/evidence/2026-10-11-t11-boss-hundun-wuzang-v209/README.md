# 2026-10-11 T11 boss 内容第二批首片：混沌无脏（规则 v209）

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `a28b6caf`；boss_hundun 四凶批次按指令做最小切片——仅无脏单技，相德/隐贼(+switch) 语义已对照登记下批）。上游依据：`mode/boss.js` `boss_wuzang`（5666：phaseDrawBegin `trigger.num += max(5, floor(hp/2)) - 2`；mod maxHandcard `num - player.hp`）与 `boss_hundun`（863：hp25 五技）。

## 建模

- `BOSS_GENERALS` 增**混沌**（群/hp25/`[凶兽, 无脏]`）——登记：相德/隐贼(+switch) 三技未落随下批补入 skills。
- **无脏** SkillDef（权限型 Noop+两面查询）：
  1. 摸牌阶段基数 `max(5, floor(hp/2))`——固定摸 2 后补额外 `max(5,hp/2)-2`（kernel 摸牌阶段臂）；
  2. 手牌上限基数 0——`hand_limit` 签名增 `catalog` 参数（`computed_distance` 同款先例），体力贡献清零、其余加成修饰保留（机甲/蛊舞/春饼链不变）。
- FROZEN 技能名序尾加「无脏」（63 名）。

## 登记差异

- 混沌 `skills` 暂只含凶兽+无脏（相德：他人有武器对你伤害+1；隐贼(+switch)：hp 首次≤半激活+你无手牌时他人伤害后随机弃一——随机改确定性先例族）——下批补。
- 摸牌基数语义=固定摸 2 后补额外量（原生无「基数改写」机制，总量等价）。

## 验证

| 项 | 结果 |
|---|---|
| 回归 `boss_hundun_wuzang_draw_and_hand_limit` | 混沌 25 血入场+无脏实例；摸牌阶段总摸 12（hp25→max(5,12)）；hp 降档后基数随体力变化（8→5）；手牌上限基数 0（25-25=0）且他人 hp4 上限不受影响 |
| 既有回归 | 烛阴/关卡/重整/addFellow 不变；`hand_limit` 签名变更连带 swd 测试调用点同步 |
| 版本回退探针（209→208 复跑 golden） | 2/2 全绿（位移唯一来源=版本字段；Boss 分支不在标准场景可达路径） |
| 14 golden 重录 | 全部更新后 2/2 绿（重录依据注释入文件头） |
| 全量 | **556 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 208→**209**（存档不变 v48） |
