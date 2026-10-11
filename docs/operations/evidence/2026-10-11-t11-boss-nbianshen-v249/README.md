# 2026-10-11 T11 年兽系形态片：形态轮换+忍性（规则 v249/存档 v59）＋忍性年兽条目

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁延续批次链；bianshen 勘验批承载结论落地）。上游依据：`boss_nbianshen`（8580：phaseBefore priority 25——storage 轮转序列+`init("boss_nianshou_<next>")` 保留 hp/maxHp/hujia；首回合仅置序列不换将）；`boss_renxing`（8419：global damageEnd/recoverEnd `_status.currentPhase != player` → draw(1)）；忍性年兽条目（ Enumeration：`[boss_nianrui, boss_mengtai, boss_renxing, boss_nbianshen]` hp12）。

## 建模

- **形态轮换**（查询型 Noop，`BOSS_NBIANSHEN`）单臂（Judge begin 段**最前**——上游 priority 25 早于其他准备阶段技）：
  - 首回合：`nbianshen_toggle=false` → 仅置位（初始形态即条目形态，不换将）。
  - 启动后每回合：轮换至另一「形态轮换」携带将（BOSS_GENERALS 段按 skills 含形态轮换查目录）——技能实例清空+`general_slots` 重挂+`apply_general_assembly`（**versus 替补换将先例同构**）；hp/手牌/装备/判定区保留（上游 init 保留 hp/maxHp/hujia 同口径）。
  - **随机序列→确定性互斥轮转登记**（上游 random+refill；原生 toggle 交替）。
- **忍性**（查询型 Noop，`BOSS_RENXING`）双臂：damageEnd 段（commit_damage，急叮同位）+recoverEnd（commit_recover_skills 头）——持有者非当前回合角色时任意伤害/回复 → 摸 1。
- **忍性年兽条目**：def 索引 26/hp12/`[年瑞, 猛态, 忍性, 形态轮换]`；年兽合体条目补「形态轮换」（轮换载体）。
- **nbianshen_toggle 新字段**（`PlayerState`，**存档 58→59**；编解码双表；canonical_hash 入参——字段新增批）。

## 登记差异

- 随机形态序列→确定性互斥轮转（先例族）。
- 年兽五形态将余量：jingjue（继承 danshu——胆术判定红回血，随夜叉批）/ruizhi（强制修剪手+装备至 1+1）/baonu（changeHp 全局+吕布 init 变身——复杂面）/shouyi（无距离 mod，随暴怒形态批）——各随其依赖批落。
- **⚠️ 引擎待查（本批发现）**：轮换后多回合推进触发 **Suspended 风暴**（中局 apply_general_assembly 后的 Yield 循环——0 号弃牌阶段每 resume 即再 Yield、无进展；单回合内轮换+忍性面全部正常）。宿主 resume 语义（N07 分片续行）在该状态下疑似不收敛——多回合轮换回归与收尾清点待查后补。先行回归覆盖：忍性直装面+轮换首换+hp/手牌保留。

## 坑位实录

- `general_slots[0]` 为 `Option<u16>`——轮换比较须 `Some(d) != cur`。
- 回归循环推进必须 **open_choice 前置检查**（先应答挂起再 next_phase；否则 Rejected(ChoicePending)）——两处循环同体需按索引分别补丁；Suspended 分支也须覆盖（中局 assembly 触发时间片）。
- `skill_name` 返回 `&str`（勿 to_string）。

## 验证

| 项 | 结果 |
|---|---|
| 新回归 `boss_nianshou_nbianshen_rotation_renxing` | 两局五段：①忍性直装（非当前回合伤害摸 1/回复摸 1/当前回合不触发）②轮换首回合（旗标置位不换将）③次回合轮换（assembly 重挂：忍性入场+hp 保留）④形态往返 ⑤守恒 |
| 既有回归 | boss 全滤 45/45；机械审计：无多重注册/零属性清零 |
| 版本/字段探针 | 字段批口径：248+新代码 14 全位移（字段源）+版本源；重录后 2/2 绿 |
| 全量 | **595 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | cargo 真实 exit 0（未用绑定清理后） |
| 版本 | 规则 248→**249**；存档 58→**59**（nbianshen_toggle 入档） |

## 遗留登记（新增一项）

- **中局 assembly 的 Suspended 风暴**（本批发现，引擎待查）：见上。
- boss_stage 局 first=0 与虚失翻面臂自动推进交互；炎狱跨回合计数器精确口径。
- 下批候选：年兽余量形态技（ruizhi/jingjue[随夜叉批]/baonu[随暴怒批]/shouyi）或 bianshen4 阴间系勘验落包。
