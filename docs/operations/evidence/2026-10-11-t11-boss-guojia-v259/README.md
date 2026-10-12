# 2026-10-11 T11 群雄乱入郭嘉片：鬼谋/远策/奇佐（规则 v259）＋郭嘉条目（天妒共用）

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 a20d03e3）。上游依据：`boss_guojia`（郭|嘉/shen/hp4/skills `[tiandu, boss_guimou, boss_yuance, boss_qizuo]`，isBoss，boss.js:1473）；`boss_qizuo`@7405 / `boss_guimou`@7447 / `boss_yuance`@7461（boss.js）。

## 勘验结论（三技语义）

- **鬼谋**：`phaseJieshuBegin` frequent——随机一名其他未混乱存活 `goMad({player:"phaseAfter"})`。上游 `goMad` = `addTempSkill("mad", end)`（player.js:9514），`mad` 技=纯标记（mark/locked/intro，skill.js:1793 **无行为面**）；mad 全部消费面：`get.attitude` 修正（get/index.js:6557/6560）、chooseControl 自动决策（player.js:13302）、UI 自动牌开关（player.js:13347）——**全为 AI/界面层，原生无对应决策层**，规则面消费仅鬼谋自身去重过滤。
- **远策**：`global damageEnd` direct——att1/att2 态度驱动 chooseControl（选项一/选项二/cancel2）+敌方 rejudge 技 stop 检查；选项一：判定黑→受伤者失 1 体力/否则来源失 1；选项二：判定红→受伤者回 1/否则来源回 1。judge 修正回调为 AI 选牌面（effect 读 `result.color`=翻牌实色）。
- **奇佐**：`player useCardAfter`——非奇佐递归、非无懈、type=trick、原目标全体存活且 canUse → `createCard` 副本对原目标再使用；AI check 排除铁索。

## 建模（三技均可落包——机制依赖面已逐项消除）

- **鬼谋**（查询型 Noop，`BOSS_GUIMOU`）：End begin 臂（炼狱/索命同槽）——座位序首个其他未混乱存活 `boss_mad=true`（随机→首个确定性近似登记，强征同款）；解除=本人回合收束（`addTempSkill "mad" {player:"phaseAfter"}` 同口径，翻面跳过路径同解）；mad 挂 `PlayerState.boss_mad`（上游 AI 态度面不落，登记）。
- **远策**（查询型 Noop，`BOSS_YUANCE`）：Damage commit 尾臂（逐厄/灵渠 damageEnd 族同槽，扣血后）——来源≠受伤者、双方存活且 hp>0 → 翻判定：黑→受伤者失 1 / 红→来源失 1（`emit LoseHp`）。chooseControl AI 态度选面+rejudge stop 检查→恒选选项一确定性近似登记；判定修正回调不落；濒死未决（hp≤0）跳过=登记差异。
- **奇佐**（`BOSS_QIZUO`，触发 `EventKind::CardUsed` AfterTriggers+一步机 `QizuoMachine`，乱常机同款）：filter=非 cancelled/本人使用/载体槽守卫（上游 event.parent name 递归守卫同口径）/可复制锦囊种（标准 12 种；无懈、铁索排除——铁索为上游 AI check 同口径）/延时锦囊判定区同名去重（canUse 同口径）/原目标全体存活；step=牌堆底实体为载体按原牌种 `use_virtual_card`（v169 原语，上游凭空造牌→载体弃置登记）+载体入 `TurnState.qizuo_copy` 槽。载体槽清：任一非载体卡牌提交即清（commit_card_used 顶）+回合收束兜底清。
- **郭嘉条目**：`BOSS_GENERALS` 尾追加（def 索引 37，hp4）`[天妒, 鬼谋, 远策, 奇佐]`——天妒=标准技共用（实例随条目授入）；势力 shen→群目录近似。

## 字段接线（存档 v60）

`PlayerState.boss_mad`+`TurnState.qizuo_copy` 双字段：state.rs 结构/初值、engine.rs 三处 TurnState 构造+canonical_hash（玩家段/回合段）、archive.rs v59→**v60**（编解码对称+`MachineSnapshot::BossQizuo` 变体 68）、snap.rs 三触点（变体/check_pid/恢复派生 None）、engine.rs push_machine 哈希臂、a0_tag_registry 变体登记 66+样例。

## 验证

| 项 | 结果 |
|---|---|
| 新回归 `boss_guojia_guimou_mad_lifecycle` | ①郭嘉四技实例（含天妒共用）②1 号 End→座位序首个（0 号）mad ③0 号回合收束解除（6 手牌弃 2 询问 settled 应答）④过滤负例（预置 0 mad→跳过取 2 号）⑤mad 存档往返+守恒 |
| 新回归 `boss_guojia_yuance_judge_branches` | ①黑桃顶：0→2 伤害后受伤者 2 号 4→2（伤害 1+远策 1）②红桃顶：来源 0 号失 1（4→3）、受伤者仅伤害 ③无郭嘉对照（仅伤害 1） |
| 新回归 `boss_guojia_qizuo_trick_reuse` | ①无中生有：原牌摸 2+副本摸 2=4（链挡：副本自身不再触发）+无懈双窗 Pass 放行 ②载体槽保持至下一用牌提交、杀（非锦囊）不复制且清槽+远策联动（1→0 黑判定→0 号 4→2）③守恒+存档往返 |
| 版本递增探针（字段批口径） | 259 全 14 位移；**版本回退 258 复跑 14 场景三值互异**（旧 258 无字段 / 258+字段 / 259+字段）——位移=版本字段+新增字段双源，无行为面扰动 |
| 14 golden 重录 | 行级程序化重录（按条目名定位+正则配新实际值，第三次使用）2/2 绿；重录依据注释入档 |
| 全量 | **608 通过 / 0 失败 / 1 ignore**（2026-10-11 本机，cargo test --offline -p engine-native） |
| Gate 1b | `RUSTFLAGS="-D warnings" cargo check -p engine-native --lib`（独立 target）exit 0 |
| 版本 | 规则 258→**259**；存档 v59→**v60** |

## 坑位实录

- **min≥1 询问空应答被拒（两处复踩）**：0 号 6 手牌 `discard_to_limit` min=2 空应答=ChoicePending 后续 NP 全拒（mad 未解除假象）；无懈窗（Confirm min=1）空应答同样被拒——新增 `next_phase_settled`（选牌取 min 个牌选项）与 `pass_nullify_windows`（无懈显式 Pass）两助手。
- **原生无 begin:false 帧**：阶段帧=begin 语义帧，阶段结束走 NextPhase 推 `Phase{begin:false}`——回合收束需 Play 起 3 次 NP（n08 口径复核）。
- **远策过滤勿排除持有者**：上游 filter 仅要求来源≠受伤者且双方存活，郭嘉自身可为来源/受伤者（首版误加排除，勘正）。
- `debug_move_named_card_to_hand` 用卡牌 name（"wuzhong"）非 kind key。

## 遗留登记（本批新增）

- 奇佐触发时机近似：`useCardAfter`→CardUsed AfterTriggers 窗口（副本效果先于原牌效果；原目标存活判定在过滤期=上游 isIn 同期）；载体牌花色点数随载体（上游 createCard 保留原牌花色点数）；载体实体入弃堆（v169 既有登记）。
- 远策：chooseControl 态度选面+rejudge stop+judge 修正回调不落（恒选项一）；濒死未决（hp≤0）跳过。
- 鬼谋：mad 的 AI 态度/决策消费面不落（原生无决策层）；随机→座位序首个。
- 下批候选：十殿阎罗族首勘（13 将/40 技）、魑魅魍魉系（guimei/minbao）、吕布变身系（暴怒）、群雄乱入余量（张春华/甄姬/华佗/貂蝉等）。
