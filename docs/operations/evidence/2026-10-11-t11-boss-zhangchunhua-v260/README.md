# 2026-10-11 T11 群雄乱入张春华片：绝情/无心/伤逝（规则 v260）＋张春华条目

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 257da137）。上游依据：`boss_zhangchunhua`（张|春华/shen/hp4/female/skills `[jueqing, boss_wuxin, shangshix]`，boss.js:1484）；`jueqing`（一将包 yijiang/skill.js:13060 张春华旧绝情）；`boss_wuxin`（boss.js:10163）；`shangshix`（boss.js:10056）+子技 `shangshix2`（boss.js:10144）。

## 建模

- **绝情**（查询型 Noop，`JUE_QING`）：commit_damage **入口直读臂**（急冻前——上游 `source damageBefore` 层）——来源≠目标且来源存活持有 → `emit LoseHp{target, d.amount}` + return（不走伤害路径：受击/加成类不触发=上游 trigger.cancel 同口径）。num 取 `d.amount`=commit 入口未增幅值（上游 damageBefore 先于 damageBegin 增幅同口径）。
- **无心**（查询型双面，`BOSS_WUXIN`）：①伤害半 commit_damage 入口直读臂（绝情后=同层 priority 10 序）——target 持有 → `LoseHp{target, 1}` + return（上游 `loseHp()` 无参=1）；②延时半 engine PlayCard 校验（Le/Bing 路由 targets 检查后）——当前回合持有者用乐/兵 → `IllegalTargets`（上游 mod targetEnabled delay 禁他人目标；原生乐/兵路由本就只能置他人→等价持有者不可用；闪电自置不受限）。
- **伤逝**（查询型双面，`BOSS_SHANGSHIX`）：①补牌半 `shangshix_draw` 公共函数四 commit 臂挂接（CardsDiscarded 尾/Damage 尾/commit_lose_hp 尾/commit_recover_skills 尾——上游 loseEnd/changeHp 双触发面展开；手牌<4 补至 4）；②子技 shangshix2 End begin 臂（鬼谋同槽）——持有者 hp>1 → 全体存活（含自己）逐个 `LoseHp{1}`（上游 get.players 全体同口径）。
- **张春华条目**：`BOSS_GENERALS` 尾追加（def 索引 38，hp4，female）`[绝情, 无心, 伤逝]`；势力 shen→群目录近似。

## 验证

| 项 | 结果 |
|---|---|
| 新回归 `boss_zhangchunhua_jueqing_wuxin_damage_faces` | ①绝情：出杀打无闪目标→失去 1（无 DamageDealt/HpLost 面）②对照：普通来源杀→伤害路径（DamageDealt 存在）③往返 |
| 新回归 `boss_zhangchunhua_wuxin_damage_only` | 独立局：杀打张春华→失去 1（无伤害日志；局内无 shangshix2 跨回合累积） |
| 新回归 `boss_zhangchunhua_wuxin_delay_and_shangshix` | ①张春华用乐→拒；②他人用乐照常（无懈窗放行）；③伤逝补牌（受伤后 0→4）；④shangshix2 全体各-1（含自己）+伤逝自补 0→4；⑤往返 |
| 版本递增探针 | 普通批口径：260→259 回退复跑 golden **全绿**（位移唯一来源=版本字段） |
| 14 golden 重录 | 行级程序化重录 2/2 绿；重录依据注释入档 |
| 全量 | **611 通过 / 0 失败 / 1 ignore**（2026-10-11 本机，cargo test --offline -p engine-native） |
| Gate 1b | `RUSTFLAGS="-D warnings" cargo check -p engine-native --lib`（独立 target）exit 0 |
| 版本 | 规则 259→**260**；存档不变 v60 |

## 坑位实录

- **跨回合 shangshix2 累积干扰断言**：`advance_to_play` 跨过张春华 End 时 shangshix2 已先全体-1——多段绝对值 hp 断言全部偏 1。解法=无心伤害面拆独立局（81003）；同局断言改相对值+注释。
- **出牌阶段限一次杀复用**：同回合第二段出杀被拒（0 号 ② 用后 ③ 被拒）——分段拆局。
- **杀距离**：4 人环 0→2 不相邻（首版目标 2 被拒）——目标改相邻座位。

## 遗留登记（本批新增）

- 绝情/无心为 commit 入口直读（Damage 帧仍完整走 AfterTriggers 触发扫描）vs 上游 cancel 后 damageEnd 技能不入队——与既有 prevent 族（急冻/诚信/招祸）同口径登记；无心与绝情同伤害事件双命中（自伤场景）的触发序未建模（上游同层 priority 序，原生固定绝情先于无心——登记）。
- 伤逝 loseEnd 面→CardsDiscarded 面（胆术/魔炎同口径登记）；changeHp→Damage/LoseHp/Recover 三 commit 面展开；弃装备/判定牌移动等非 CardsDiscarded 失牌面不触发（登记差异）。
- 绝情 AI 面（ai.jueqing=true 估伤标签）不落（AI 决策层惯例）。
- 下批候选：群雄乱入续片（甄姬[踏水/凌波/娇孝/芳华]、刘备[枭雄/章武]、诸葛亮[hp∞]等）、十殿阎罗族首勘、吕布变身系。
