# 2026-10-11 T11 boss 内容第三批首片：穷奇逐厄（规则 v211）

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `1bcd3a8f`；boss_qiongqi 批按指令最小切片——仅逐厄单技）。上游依据：`mode/boss.js` `boss_zhue`（5699：global damageEnd → `asyncDraw([player, source])`）与 `boss_qiongqi`（870：hp20/maxHp25/五技）。

## 建模

- `BOSS_GENERALS` 增**穷奇**（群/hp20/`[凶兽, 逐厄]`）——**登记差异**：上游 maxHp25 暂以 hp=20 建模（general_meta 元组无 max 段，maxHp 面随覆泰批一并处理）；覆泰(+全场禁桃)/延妒(+switch) 三技未落随下批。
- **逐厄**：权限型 Noop+`commit_damage` 尾 global 臂——任何伤害结算后（来源存活且非穷奇），穷奇与来源各摸 1（穷奇为来源时不触发；单 boss 场景恰一持有者）。

## 验证

| 项 | 结果 |
|---|---|
| 回归 `boss_qiongqi_zhue_dual_draw` | 3 号杀 2 号 → 穷奇/来源各摸 1；穷奇自己为来源 → 不触发 |
| 既有回归 | 混沌四技/烛阴/关卡/重整/addFellow 不变 |
| 版本回退探针（211→210 复跑 golden） | 2/2 全绿（位移唯一来源=版本字段；Boss 分支不在标准场景可达路径） |
| 14 golden 重录 | 全部更新后 2/2 绿（重录依据注释入文件头） |
| 全量 | **558 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 210→**211**（存档不变 v48） |
