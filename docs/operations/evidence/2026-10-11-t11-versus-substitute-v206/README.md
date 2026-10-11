# 2026-10-11 T11 versus 替补面：死亡窗口从队池换将复活（规则 v206，存档 v48）

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `2fcf6bdf`；上一轮登记「versus 替补（中批待做）」消除）。上游依据：`mode/versus.js:8286` 替补出场——死亡座位从本队未出场武将列表替补：`revive + uninit/init（换将）+ draw(4) + 濒死链取消`。

## 建模

- `GameMode::Versus { teams, substitutes }`：每队替补武将池（与队伍同队序，可为空池）——**存档格式 47→48**（模式段追加池编码；v47 字节被版本门禁拒绝）；canonical_hash 模式段同步入池；快照校验=池数与队数一致+武将 def 界。
- `Engine::new_versus_with_substitutes(seed, teams, substitutes)`（`new_versus` 传空池不变）。
- **DeathMachine 死亡执行前替补臂**（death_cancelled 检查后）：死亡座位所在队池非空 → 弹队首 → 清本座位技能与区域（手牌/装备/判定区入弃堆——上游 uninit/init 重建面）→ `general_slots[0]=替补` + `apply_general_assembly`（授新将技+体力）→ 新将满血复活 → 摸 4 → **死亡执行被替代**（奖惩/终局判定不发生）；池空照常死亡结算。

## 登记差异

- 替补替代死亡时击杀奖惩不结算（上游 dieAfter 链顺序细节不引入——死亡未发生的口径）。
- 上游「出场」日志/dieAfter 展示面不入原生日志（状态可观测）。
- 替补出场为新将满血开局（上游 init 同语义）；复活选择 UI（我方主公操控选替补）以确定性队首序近似。

## 验证

| 项 | 结果 |
|---|---|
| 回归 `versus_substitute_revive_on_death` | 首杀替补介入（不死亡/新将满血/摸 4/终局不触发/奖惩不结算）；第二替补继续；池空第三次击杀照常死亡（奖惩摸三结算）；存档往返一致 |
| 既有 versus 回归 | `versus_4v4_eight_seats_team_victory`、`versus_mode_team_victory_and_rewards` 不变通过 |
| 版本回退探针（206→205 复跑 golden） | 2/2 全绿（位移唯一来源=版本字段；versus 不在标准场景可达路径） |
| 14 golden 重录 | 全部更新后 2/2 绿（重录依据注释入文件头） |
| 全量 | **552 通过 / 0 失败 / 1 ignore**（551+1，2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 205→**206**；存档 47→**48** |
