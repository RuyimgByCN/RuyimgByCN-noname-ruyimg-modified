# 2026-10-11 T11 明星片：庆祝/夹击/急叮三技（规则 v245）＋明星条目

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁延续批次链；绝红后下一切片）。上游依据：`boss_qingzhu`（5382：phaseDiscardBefore cancel+mod cardEnabled 杀禁用[!hasSkill boss_jiding]）；`boss_jiazu`（5396：准备阶段相邻敌方坐骑[equip3/4]全弃）；`boss_jiding`（5434：global damageEnd——友方受敌方来源伤害→对来源虚拟雷杀+蓐收 damageSource 加成尾）；携带者 boss_mingxingzhu 明星（1075：female/shen/hp3/skills `[boss_qingzhu, boss_jiazu, boss_jiding]`）。

## 建模

- **庆祝**（查询型 Noop）两面：①跳弃牌——**弃牌机器生成口真实闸门**（`Phase{Discard,begin}` dispatch：tanyu/qingzhu → None+计数清零；见坑位）②杀禁用（engine PlayCard 校验臂：qz && !jd → 拒）。
- **夹击**（查询型 Noop）：Judge begin 臂（准备阶段锚）——环上/下家异侧存活者的坐骑（`card_kind.equip_slot()` ∈ HorsePlus/HorseMinus）全部撤技能+入弃堆。
- **急叮**（查询型 Noop）：commit_damage damageEnd 段臂（灵渠同位）——holder（同侧≠受伤者）在场 && 来源为异侧存活 → `use_virtual_card(holder, CardKind::LeiSha, [source])`（**LeiSha kind 载体**——nature=Thunder 免改 use_virtual_card 签名；机器直发不经 PlayCard 校验）。
- **明星条目**：def 索引 22/hp3/female/`[庆祝, 夹击]`——**急叮不随条目装配**（星囚授予技：上游蓐收 boss_xingqiu addSkills 授予；庆祝禁杀窗口依赖初始缺席——条目全装配会让禁杀永无窗口，首版踩中后修正）。
- **登记差异**：急叮蓐收加成尾面（damageSource→jinshenrushou）未建模；势力 shen→群目录近似。

## 坑位实录

- **贪欲「跳弃牌」的历史真相**：v215 起贪欲臂的 `return` 只跳过阶段收尾计数，弃牌机器实际仍生成——贪欲回归一直靠 `drain_choices` 空应答掩盖（min=1 时空应答被拒……回查：贪欲测试手牌未超限或以 min 合法路径通过）。本批补真实闸门（生成口 None）后贪欲双测复跑仍绿。
- 测试误把**技能 def 当武将 def** 装配 fellow（`debug_set_general_def(fellow, jz)`）——fellow 拿到任意垃圾武将，后续回合态错乱（WrongPhase 假象）；移除装配（无将 fellow 即纯伤害载体）。
- **min=0 可放弃询问不 drain**（星夏批三踩的老坑）本批再犯两次——激活后/选敌后的两处过早 drain 把追击/弃红询问吞成放弃分支；已全部摘除并注释。
- ④段把杀打在弃牌阶段（庆祝跳过机器后阶段仍推进到 Discard）——断言链按「验跳弃牌→回出牌阶段→验杀解除」重排。

## 验证

| 项 | 结果 |
|---|---|
| 新回归 `boss_mingxingzhu_qingzhu_jiazu_jiding` | 五段：①庆祝杀禁用（无急叮拒）②急叮授予面（不随条目）+虚拟雷杀（fellow 受伤→对 0 号 1 雷）③夹击（相邻敌方坐骑全弃、对座保留）④庆祝跳弃牌+获急叮后杀解除 ⑤庆祝实例存档往返+守恒 |
| 既有回归 | boss 全滤 41/41；贪欲双测复跑绿（真实闸门无回归）；机械审计：无多重注册/零属性清零 |
| 版本递增探针 | 244→245 全 14 位移；回退 244 复跑 **全绿**（位移唯一来源=版本字段） |
| 14 golden 重录 | 245 全量重录 2/2 绿 |
| 全量 | **591 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | cargo 真实 exit 0 |
| 版本 | 规则 244→**245**；存档不变 v57（无新字段/快照变体——急叮为既有实例的运行时授予） |

## 遗留登记（不变）

- boss_stage 局 first=0 与虚失翻面臂自动推进交互；炎狱跨回合计数器精确口径。
- 下批候选：boss_zaoyi 造异（玄冥/共工死亡触发系+回合开始最小体力失去全部体力）；星宿阶段 2 系明星/蓐收/勾芒/炎帝/少昊/祝融/共工条目已齐（8 将）。
