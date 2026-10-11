# 2026-10-11 T11 星宿阶段2系末片：不臣（规则 v240/存档 v56）＋勾芒条目

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁延续批次链；星囚后下一切片）。上游依据：`boss_buchun`（7157：group 两主动子技——revive 复活场上已死亡书精[revive(1)+draw(2)]、recover 令一名己方受伤角色回复 2；两线 filter 互斥[有死亡书精=复活线，无死亡书精=恢复线]；storage=roundNumber 间隔 ≥2 轮）；携带者 boss_mushengoumang 木神勾芒（972：姬|重/shen/hp5/skills `[boss_shenyi, boss_buchun, boss_qingmu3]`）。

## 建模

- **不臣**（`BOSS_BUCHUN`；`BuchunBehavior`+`BuchunMachine`，本批首个 **boss 主动技**——`ActivateSkill` 直调 `activate`，不经触发总线）：
  - **复活线**（场上有死亡书精）：失 1 体（`EventData::LoseHp`）→ 逐个复活（`alive/dying/boss_chongzheng=0` 重整同款 + hp=1 + `BossRevived` 日志 + 摸 2）。死亡书精判定=座位 `!alive && general_slots[0]==书精目录索引`（BOSS_GENERALS 段按名查目录，表序重排稳健）。
  - **恢复线**（无死亡书精且存在同阵营受伤者）：SelectPlayers 目标询问（`is_boss_side` 同侧 + hp<max）→ `ctx.recover(t,2)`。
  - **间隔门控**：`PlayerState.buchun_used_round`（0=未发动；`round-used<2` → activate 返回 None → `InvalidChoice` 拒）。
  - **快照变体 `BossBuchun`**（archive tag 66/机哈希 67/a0 variant_index 64；stage 显式编码 0/1=复活线、2/3=恢复线；active_ask 挂起态按 stage==3 派生受伤同侧候选）。
- **勾芒条目**：`BOSS_GENERALS` 尾追加（def 索引 16，hp5）`[神裔, 不臣]`；青木3 已处置（关卡面），条目不含。
- **buchun_used_round 新字段**（`PlayerState`，**存档 55→56**；编解码双表；canonical_hash 入参——字段新增批）。

## 登记差异

- 上游复活仅复「与携带者同 parentNode 的书精」并按座次先后（previousSeat 邻位优先）——native 为全场死亡书精座位序逐个（ parentNode=关卡分组概念，裸骨架无对应，登记）。
- 上游 revive 线 targets 排序含「上家优先」细节未建模（座位序近似）。
- 势力 shen→群目录近似（先例族）。

## 坑位实录

- `BOSS_SHUJING` 常量不存在（书精的技能常量是催毒 `BOSS_CUIDUI`，将名无常量）——改为 BOSS_GENERALS 段内按名查目录索引。
- `SkillBehavior` 的 `start` 为必需方法（主动技也须实现——`unreachable!("不经触发总线")` 占位）。
- `ctx.answer_first()` 不存在——kernel 惯用 `ctx.answer().first().copied()`。
- `general_count()` 返回 usize（测试先例强转 u16 在测试侧；行为内按 usize 运算后再转）。

## 验证

| 项 | 结果 |
|---|---|
| 新回归 `boss_mushengoumang_buchun_revive_and_recover` | 五段：①击杀书精→发动失 1 体、书精复活（hp1+摸 2+轮次登记）②同轮再发动被拒 ③恢复线（fellow 受伤）目标询问→回 2+轮次登记 ④buchun_used_round 存档 v56 往返 ⑤全程守恒 |
| 既有回归 | boss 全滤 36/36；a0 样例 2/2（新变体登记）；机械审计：无多重注册/零属性清零 |
| 版本/字段探针 | 字段批口径：239+新代码 14 全位移（字段源）+版本源；重录后 2/2 绿 |
| 全量 | **586 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 239→**240**；存档 55→**56** |

## 遗留登记（不变）

- boss_stage 局 first=0 与虚失翻面臂自动推进交互；炎狱跨回合计数器精确口径。
- 下批：星宿阶段 2 其余内容技（赤翼[炎帝]/星夏[祝融]/白翼[句芒]等）或 boss_huihuo/furan（炎灵陪将）；相关将逐将入库。
