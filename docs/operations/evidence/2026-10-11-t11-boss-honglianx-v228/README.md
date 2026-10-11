# 2026-10-11 T11 毕方批续片：红莲（规则 v228）

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `eecfdc90`；毕方批续片按指令最小切片——仅红莲单技，炎狱(+switch) 登记下批）。上游依据：`mode/boss.js` `boss_honglianx`（4974：ignoredHandcard red + cardDiscardable phaseDiscard red=false + phaseZhunbeiBegin 随机获 0-3 红牌+随机对 3-0 名各 1 火伤）。

## 建模

- **红莲**两面（权限型 Noop+查询）：
  1. **红色牌不计入手牌上限**——弃牌机器双位排除（沙漠绿洲 shamolvzhou 先例同构）：统计口径 `hand` 只数非红手牌；SelectCards 选项排除红牌（红牌不会被要求弃）；
  2. **回合开始**——Judge begin 段内联：顺序获牌堆**前缀红牌至多 3**（`deck` 顶连续红牌，非红即停），并对座位序前至多 3 名其他角色各 1 火伤（招祸使毕方伤害恒为火——两技自然联动）。
- FROZEN 尾加「红莲」（81 名）。

## 登记差异

- 上游随机（获牌数 0-3、目标 3-0 名）→ **确定性近似**（前缀红牌连续取、座位序前 3 名）——随机近似先例族。
- 获牌未逐张 push log（以 CardsDrawn 汇总）——观测面近似。

## 验证

| 项 | 结果 |
|---|---|
| 回归 `boss_bifang_honglianx_red_excluded_and_judge_gain` | 置顶红黑交替后回合开始：0/2/3 号各-1（招祸火伤+座位序前 3 名）；毕方手牌含获红牌 |
| 既有回归 | 招祸/虚失/巨源/兵先/相柳/梼杌/书精/四凶/关卡全部不变；native_kernel 230/0 |
| 版本回退探针（228→227 复跑 golden） | 2/2 全绿（位移唯一来源=版本字段） |
| 14 golden 重录 | 全部更新后 2/2 绿（重录依据注释入文件头） |
| 全量 | **575 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 227→**228**（存档不变 v51） |
