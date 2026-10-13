# 2026-10-11 T11 群雄乱入张角片：天道/雷击·张角/极点（规则 v264，存档 v61）＋张角条目

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁延续）。上游依据：`boss_zhangjiao`（张|角/shen/hp8/skills `[boss_leiji, tiandao, jidian]`，boss.js:1575）；`tiandao`@8126 / `boss_leiji`@9278 / `jidian`@9615。

## 建模

- **天道**（查询型 Noop，`BOSS_TIANDAO`）：**改判机 rewriters 三处扩展**（JudgePhase 机器内联）——①队列构造 filter 加天道持有者（与鬼道/鬼才并列，座位序同队）；②skip 步可改判条件加「天道+任意 hes 牌」；③选项步天道持有者全部手牌+装备牌入选项。改判语义与鬼道同款：所打牌成为新判定牌、原判定牌由改判者获得（机器既有行为，无新增日志）。上游 `global judge` cost/content：respond hes + gain judging + 替换同口径。
- **雷击·张角**（触发 `CardResponded`+extra `CardUsed`+一步机 `ZjLeijiMachine`，凌波/踏水同款触发形）：使用/打出【闪】后，座位序首个其他存活角色**内联确定性判定**（铁骑先例，无改判窗=登记）：黑→其受 1 点雷伤+张角摸 1（chooseTarget AI 态度面→首个确定性近似登记）。
- **极点**（查询型 Noop，`BOSS_JIDIAN`）：Damage commit 尾臂（章武臂后）——张角造成伤害后，**距离受伤者 ≤1**（`computed_distance`，马术/马匹全修饰）的首个其他存活角色判定：黑→其受 1 点雷伤。**链挡旗标**（`TurnState.jidian_chain`）：emit 前置位、嵌套 Damage 帧的极点臂见旗标清除并跳过（挡恰一层）。
- **张角条目**：`BOSS_GENERALS` 尾追加（def 索引 42，hp8，male）`[雷击·张角, 天道, 极点]`；势力 shen→群目录近似。

## 同名改名映射（迅影批先例，第二例）

上游 `boss_leiji`（张角）与青龙 `boss_releiji` 显示名**同为「雷击」**。青龙雷击已以 `BOSS_RELEIJI="雷击"` 落目录（FROZEN 冻结）——张角雷击按迅影批改名映射先例以 **「雷击·张角」**（`BOSS_ZJLEIJI`）入目录。映射登记：原生「雷击·张角」↔上游 `boss_leiji`；原生「雷击」↔上游 `boss_releiji`。

## 字段接线（存档 v61）

`TurnState.jidian_chain: bool`（极点链挡旗标）：state.rs 字段+3 处构造、canonical_hash（回合段）、archive **v60→v61**（TurnState 尾部 encode/decode）。

## 验证

| 项 | 结果 |
|---|---|
| 新回归 `boss_zhangjiao_tiandao_rewrite` | 闪电判定询问改判：张角以天道资格被问（非黑手牌选项存在=扩展直接证据）→ 打非黑牌改判 → `JudgementRewritten` 日志+手牌数不变（打一张+获得原判定牌）+闪电未造成伤害+往返 |
| 新回归 `boss_zhangjiao_leiji_shan_thunder` | 打出闪 → 首个其他存活（0 号）判定黑（预置黑桃）→ 1 雷伤+张角摸 1+往返 |
| 新回归 `boss_zhangjiao_jidian_chain_guard` | 张角对 3 号伤害 → 距 3 号 ≤1 首个（0 号）判定黑→1 雷伤；链挡：极点雷伤后旗标已清、hp 稳定；再触发：对 2 号伤害 → 距 2 号 ≤1 首个=张角自身（上游 filter 仅排除受伤者，自身合法候选；AI 态度面不选己=登记差异）判定黑自雷伤 8→7 |
| 版本递增探针（字段批口径） | 263（无字段）/**263+字段**/264+字段 三值互异（版本+字段双源、无行为面扰动） |
| 14 golden 重录 | 行级程序化重录 2/2 绿 |
| 全量 | **621 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 263→**264**；存档 v60→**v61** |

## 坑位实录

- **TurnState 构造器在 engine.rs 不在 state.rs**（接线脚本两次锚错文件）。
- **next_phase_settled 假设无前置挂起**：循环内已有挂起（discard_to_limit）时它发 NP 被拒→询问永不消除假死——循环内须直接 `answer_sel` 应答当前挂起。
- **天道应答牌被测试自身清除**：break 后 clear 手牌再应答旧选项中的牌→`take_owned_card` panic——应答须用当前选项快照中的牌。
- **改判者获得原判定牌无独立日志**（`JudgementGained` 仅天妒阶段）：以 `JudgementRewritten`+手牌数收支断言替代。
- **链挡旗标时序**：emit 后立即清除=无挡（嵌套帧处理时旗标已复位）——须嵌套臂见旗标清除+跳过（挡恰一层）。
- **判定牌自然序不可控**：负例须预置黑桃（自然判定恰红假失败，仙音批同款）。

## 遗留登记（本批新增）

- 天道：与鬼道/鬼才同持时改判选项并集（无此条目组合）；改判询问 prompt 沿用 `gui_dao_rewrite`（全改判者共用）。
- 雷击·张角：判定为内联确定性口径（无鬼道/天妒改判窗=铁骑先例登记）；chooseTarget AI→首个其他存活。
- 极点：极点雷伤不自我递归（上游 damageAfter 真递归=链挡登记差异）；判定内联口径同上；AI 不选己方→确定性首个可含张角自身（登记差异，测试覆盖自雷伤路径）。
- 下批候选：五官王/宋帝王/卞城王/泰山王/转轮王（勘验完备，三技全温和）、庞统（涅槃重活）、诸葛亮（hp=Infinity 专项）、十殿阎罗余量。
