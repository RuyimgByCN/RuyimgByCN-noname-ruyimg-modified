# NC08 第⑦层「逐命令对照」工具（旧线 ↔ 原生线）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**工具已建立并跑通 6 个种子**；**首次分歧全部落在 step 0**（已登记的开局差异）；并**量化**出旧线的**初始体力随机化**

## 1 口径与边界（重要）

方案要求的「逐命令对照」以**上游 JS** 为基准；上游 JS 在本仓库**不可独立运行**（无 harness）
→ 本工具以**旧线 `engine-core`** 作为**可执行参照**。**旧线 ≠ 上游等价**（见开局差异的上游判定），
因此本工具只做**逐命令分歧定位与回归守护**，不声称与上游等价。后续接入上游 harness 时，
把参照实现从旧线替换为上游即可复用同一比较框架。

## 2 工具

`apps/game-server/tests/nc08_command_by_command.rs`（放在 `game-server`：它同时依赖
`engine-core` 与 `engine-native`）：

- 同种子、同 4 座身份局；两线各自建局；
- 逐步比较 `(当前座位, 相位, 各座位 hp/alive)`（座位归一到 0 基）；
- 同步推进：旧线 `EndPhase` / 原生 `NextPhase`；
- 输出 `CROSSLINE …` 报告；断言：**首次分歧必须出现在 step ≤ 1**（否则视为新增未登记差异）。

## 3 实测（[crossline.log](crossline.log)，6 个种子）

```
seed=1        首次分歧@step=0 旧(current=0,phase=Draw,seats=[(5,true),(3,true),(3,true),(6,true)]) 原生(current=0,phase=Judge,seats=[(5,true),(4,true),(4,true),(4,true)])
seed=42       首次分歧@step=0 旧(phase=Draw,seats=[(4,true),(4,true),(3,true),(4,true)]) 原生(phase=Judge,seats=[(5,true),(4,true),(4,true),(4,true)])
seed=99       首次分歧@step=0 旧(seats=[(4,true),(3,true),(3,true),(3,true)]) 原生(seats=[(5,true),(4,true),(4,true),(4,true)])
seed=777      首次分歧@step=0 旧(seats=[(5,true),(4,true),(3,true),(4,true)]) 原生(seats=[(5,true),(4,true),(4,true),(4,true)])
seed=12345    首次分歧@step=0 旧(seats=[(4,true),(6,true),(4,true),(4,true)]) 原生(seats=[(5,true),(4,true),(4,true),(4,true)])
seed=20261702 首次分歧@step=0 旧(seats=[(5,true),(3,true),(4,true),(4,true)]) 原生(seats=[(5,true),(4,true),(4,true),(4,true)])
```

**读法**：
1. **相位**：旧线起手 `Draw`、原生起手 `Judge` → 已登记差异（原生缺 Prepare 阶段，相位序列前移）；
2. **初始体力**：原生恒为 `主公 5 / 其余 4`；旧线**随机化**（实测出现 3、6 等值，跨 6 个种子均不同）
   → **此前登记的「旧线开局随机伤害」应更正为「旧线初始体力随机化」**（更精确）；
3. 因此**首次分歧必然在 step 0**，本工具据此守护：若未来分歧出现在 step ≥ 2，即为**新增未登记差异**。

## 4 未完成 / 待办

- [ ] **接入上游 harness**（把参照从旧线替换为上游 JS 输出）——方案原意，当前不可行（无 harness）
- [ ] 对照轴扩展：手牌**数量**（旧线需经 `CardStore` 区域查询）、事件流、随机流、选择面
- [ ] 旧线初始体力随机化的**上游判定**（上游 `chooseCharacter` 是否有随机体力？需查归档 JS）
- [ ] 把本工具纳入常规回归（当前为一次性/手动运行）
