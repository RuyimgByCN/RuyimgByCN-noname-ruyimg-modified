# 2026-10-11 T11 bianshen 变身机制勘验＋试炼态四将处置（无版本递增）

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `bc552b8c`；年兽/阴间系共同依赖的机制勘验批）。本批为**勘验处置批**：无引擎代码改动——规则 247/存档 v57/golden 不动。

## 变身族三形态勘验（boss.js 行号实测）

| 形态 | 实例 | 机制 | 结论 |
|---|---|---|---|
| **开局 init 壳** | 试炼态四将 boss_qingmu@6240/chiyan@6058/baimang@6620/xuanlin@6620→6422、boss_bianshen@9248（捉鬼祛邪→魑魅魍魉随机） | gameStart forced → `player.init(<目标将>)` 整体替换武将（smoothAvatar/reinit2=视觉） | **纯视觉壳**：init 后即为目标将（青龙/朱雀/白虎/玄武——均已入库 v233–v237）。原生承载=直接用目标将条目，试炼态壳无内容面（intro1-4=nobracket UI 字段）。处置：不落，条目以目标将代位 |
| **死亡 changeBoss** | boss_bianshen4@8779（黑白无常/牛头马面等）：dieBegin hide + dieAfter `game.changeBoss([boss_yecha,boss_luocha] 随机)` | 与隐藏链八技同构（dieBegin+dieAfter 换将链），差异仅在 changeBoss 换「boss 本体」而非座位补给 | **隐藏链同族**（已处置口径延伸）：关卡流程面不落 |
| **形态轮换（保留属性）** | boss_nbianshen@8580（年兽五形态）：phaseBefore → `init("boss_nianshou_<next>")` 且**保留 hp/maxHp/hujia**，形态序列 storage 轮转（jingjue/renxing/ruizhi/baonu 循环+随机） | 机制=武将 def 重挂+属性保留+技能组随 assembly 重挂 | **可承载（有先例）**：versus 替补换将（v206）即「general_slots 重挂+apply_general_assembly+hp 保留」同构；随机形态选择以固定序/种子近似（登记先例族）。**可行结论：技能组重挂经运行时 assembly 承载成立**，changeSkin 视觉面登记不落 |

## 承载结论

1. 试炼态四将 → 不落（视觉壳，四象将代位）。
2. bianshen4 死亡换 boss → 隐藏链口径处置（不落）。
3. 年兽形态轮换 → **可行**：`debug_set_general_def` 同款 assembly 管线（技能组重挂）+ hp/maxHp/护甲保留三条直写；落包时按「形态序列确定性轮转+属性保留」建模，随机序列登记差异。年兽系本技（jingjue/renxing/ruizhi/baonu/nianrui/mengtai 等）逐技勘验后即可逐批落包。

## 验证

| 项 | 结果 |
|---|---|
| 引擎改动 | 无（勘验处置批） |
| 版本/存档/golden | 规则 247/存档 v57/14 golden 均不动 |
| 全量 | 本批无代码改动，沿用 v247 实测 **593/0/1**（同源码） |
