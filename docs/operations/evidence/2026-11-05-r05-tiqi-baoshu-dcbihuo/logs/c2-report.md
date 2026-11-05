# R05-C2 批证据：续行位模式迁移 C 批保留的 42 处 `ctx.draw_cards` → `start_draw`

- 日期：2026-11-04
- 工作区：/private/tmp/noname-r05/worktree（分支 r05-private）
- 基线 HEAD：fbf6275d1（C 批收尾后的 docs 提交；C 批代码终点 e4465d281）
- 批末 HEAD：566016366（13 个提交：wave0 红守卫 … wave11 + closeout）
- 测试命令：`CARGO_TARGET_DIR=/private/tmp/noname-r05/target CARGO_INCREMENTAL=0 timeout 1500 cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native --no-fail-fast`
- 基线结果：1954 通过 / 1 失败（replay_scenarios golden，预存）/ 75 ignored
- 批末结果：**1958 通过 / 1 失败（同一 replay_scenarios golden，14 场景 actual 与 C 批末逐字节相同）/ 75 ignored**
  （+4＝本批 4 个新红守卫测试转绿；日志 c2-02-full-run.log）

## 0. 结论摘要

- C 批保留的 **42 处（11 文件）同步 `ctx.draw_cards` 全部迁移**到
  `ctx.start_draw(p, n as u32)`（真实 R 帧异步摸牌）；16 处带摸后即时
  依赖的调用点按 **WuzhongMachine 续行位模式**（kernel/tricks.rs:20-38）
  改写：首步 `start_draw` + `Flow::Yield`，新增 **`HostStage::DrawSettle`**
  通用续行阶段（host.rs:92-96，tag 14，入快照）承载各行为自己的摸后尾段。
- `machine.rs` 的 `draw_cards` 同步原语**已删除**（含 C 批防回流注释）；
  `rg "\.draw_cards\(" crates/`＝0 活调用（tests/ 仅 3 处注释文本，与 C 批
  报告 §6 口径一致）。
- 两项前置决策（详见 §1/§2）：boss 孟婆 level 0 乱入＝**上游不摸牌，删除
  「摸 4＋回滚」整段**（红测锁零日志零扰动）；wudian 扶仇·清算＝**逐来源
  摸 1 全部先行 R 化、杀帧统一在续行段发射＋空堆守卫**（同步路径同场景
  panic，异步跳过该次虚拟杀＝不 panic 不造牌）。
- 3 处源码文本锁测试（yunchou 狡黠/雅量/狂骨·变体）按 C 批 yiji 先例
  更新为 `start_draw` 字面并注明 R05-C2（§5 分流账 a 类 3 项）。

## 1. 前置决策一：boss 孟婆 level 0 乱入（上游对照结论）

**结论：上游不摸牌 → 正确迁移＝删除「摸 4＋调用方回滚」整段（非续行位）。**

上游依据（archive/upstream/noname）：

| 事实 | 上游行号 |
| --- | --- |
| `game.addFellow(position, character, animation)` 只建座位 `player.init(character)`，**无任何发牌** | apps/core/noname/game/index.js:9956-9968 |
| 孟婆开局乱入（`boss_newhuanren`）：`game.addFellow(...)` 后无 directgain（:3448 `level !== 0` 门在开局恒假，:3449 不执行） | apps/core/mode/boss.js:3439-3455 |
| 孟婆转关乱入（`boss_newhuanren2`，level 已 ++≥1）：`if (_status.shidianyanluo_level !== 0) fellow.directgain(get.cards(4))` | apps/core/mode/boss.js:3575-3577 |
| `game.changeBoss`（换将本体）＝删旧建新＋`boss.directgain(get.cards(4))` | apps/core/mode/boss.js:1697-1731（directgain 在 1731） |
| `game.addBossFellow`（四象链随从）＝addFellow＋`fellow.directgain(get.cards(4))` | apps/core/mode/boss.js:1689-1696（directgain 在 1691） |

原生改写（host/boss.rs）：

- `sixiang_add_fellow` **不再自发牌**（返回座位，发牌归调用方）；
- `huanren_spawn_mengpo(ctx, draw4, draws)`：`draw4=false`（开局）＝**零摸牌
  零日志**（旧实现摸 4 后逐张取回压栈＝牌序恢复但 CardsDrawn 日志已落＝
  日志面扰动，删除）；`draw4=true`（level≥1 转关）＝登记 (pid,4)。
- 红测 `r05c2_mengpo_level0_spawn_is_card_silent`（r05c_async_draw.rs:352-395）：
  断言乱入座位手牌空**且全程无该座位 CardsDrawn 日志**——修前红（seed 3
  日志断言失败），迁移后绿。
- 既有 d4 断言（native_kernel/d4.rs:726「level 0 孟婆不应保留摸到的 4 张牌」）
  迁移后仍绿（26/26）。

## 2. 前置决策二：wudian 扶仇·清算 Fuchou2（wudian_status.rs:74）

上游依据：玩点论杀 extension（inventory 路径 archive/upstream/noname/
apps/core/extension/玩点论杀/character/skill.js:548-578）——每标记来源
`player.draw()` 后 `chooseToUse`（有杀用杀、无杀失体力）；上游 draw 双枯竭
**不 panic**（摸 0 张继续）。

R 机双枯竭行为（kernel/draw.rs:329-344）：deck+discard 全空 → held 转
`detached_cards`（Unclaimed）、terminal=Exhausted、无 gain、**不 panic**。

原生改写（同步循环「摸 1→use_virtual_card」）：

- Act 步：取标记、开局存活过滤存 `self.targets`（快照承载），逐来源
  `start_draw(owner,1)`（**逆序压帧＝标记序执行/取牌**，多人手牌次序与同步
  一致）→ `Flow::Yield`；
- DrawSettle 步：R 全部交付后**按标记序统一发虚拟杀**——同步序＝杀帧按
  标记序压栈、LIFO 解算＝逆标记序，两路径解算序一致；杀帧在 R 帧之下、
  R 先解，与同步「摸牌交付先于杀解算」一致；
- **空堆守卫**：`use_virtual_card` 取 `deck.first()` 空堆 panic（machine.rs
  删除前 :629）——同步路径双枯竭同样 panic（无既定行为可保）；续行段改为
  `if !ctx.core.deck.is_empty()` 跳过该次虚拟杀（不 panic、不凭空造牌、
  标记照常消耗；上游对应面＝无杀失体力，原生为登记近似的虚拟杀，跳过
  仅作用于同步即 panic 的不可达面）。
- 红测 `r05c2_fuchou2_exhausted_no_panic_virtual_sha_skipped`：修前红＝
  panic "virtual use: deck empty"（c2-01.log），迁移后绿（标记消耗/手牌
  不变/无 CardsDrawn）。
- 已知不可观测差异（登记）：≤2 张残堆极端下摸牌（牌堆顶）与载体（牌堆
  底）的取用交错序不同、手牌内序不同——同步路径在等价场景直接 panic，
  无测试面依赖；上游按「先 draw 后 chooseToUse」逐源结算，原生本就以
  「虚拟杀强制指定」登记差异。

## 3. 42 → 0 逐文件账（续行阶段设计＋行为等价证明）

通用续行位＝`HostStage::DrawSettle`（快照 tag 14）：同一行为至多一张在飞
R 帧，行为身份＋阶段唯一确定尾段。恢复语义：R 子树弹栈后调度器按
resume_depth 唤醒本机（Wuzhong 同型）；档位快照含 resolution 栈与 R 帧，
恢复即续行。

| # | 文件 | 处 | 续行位（DrawSettle 尾段） | 直迁（无尾段） | 行为等价证明（定向组） |
| --- | --- | --- | --- | --- | --- |
| 1 | host/boss.rs | 8 | —— | 全部（多点摸牌按同步执行序收集、`push_draws_reversed` 逆序压 R 帧＝收集序执行/取牌：huanren_apply 换将4→地藏3→孟婆4；run_chain 换将→侧座→随从→挑战者；wing_qing R7 勾芒→书精；孟婆 L0 删摸） | d4 26/26（含 dizang 手牌 7、孟婆手牌空、书精 +3）；kernel boss 131/131；boss_play_eight 2/2；r05c2_mengpo 绿 |
| 2 | step/yunchou.rs | 7 | Beige 方块臂（摸 2 后收回奖励＝同步序「摸先于奖励读弃堆」，洗牌边界序敏感；judged/discarded 经 cards 槽带入）；Diyguhuo（摸 2 后「判定区→装备→手牌首张」退弃×2——空手时同步弃刚摸的牌） | 雄姿/雅量（座位序逆压）/野升/狡黠/狂骨·变体 | nc08 yunchou 36/36；g1_shahai yunchou 26/26；悲歌 club/heart 臂测试绿 |
| 3 | step/gujian.rs | 6 | 舒筋散（摸等量后本体入弃堆——洗牌边界序）；棉里（摸 1 后失体力帧+本体——帧序保持）；虾饺（摸 1 后建 min1max1 弃牌询问——选项含刚摸的牌） | 探花冻/青团（摸 1+food_tick 纯状态翻转，次序不可观测）；鲈鱼羹（弃后摸 1 步末） | g1_shahai gujian 14/14；nc08 gujian 31/31 |
| 4 | step/wudian_status.rs | 5 | **扶仇·清算**（§2：全摸先行 R 化＋续行段统一发杀＋空堆守卫） | 割据（factions.len()）/♦光书（摸 2+实例移除纯状态）/拾火/应用（pingzhang） | g1 wudian 122/122；r05c2_fuchou2 绿 |
| 5 | step/xianjian_card_use.rs | 3 | 照妖（拼点赢摸 2 后建 min2max2 置顶两选——选项含刚摸的牌） | 蝶舞（给牌后摸 1）；千方（虚拟 AOE 帧在 R 帧之下、R 先解＝同步「摸牌先落、AOE 后结」序） | g1 nangonghuang 3/3；r05c2_zhaoyao 绿 |
| 6 | step/swd_equipment.rs | 3 | 空洞印（摸 1 后弃全部宝物——to_discard 与摸牌洗牌边界序）；青龙之圭（摸 2 后 he min1 询问）；逆攻（奇数层摸 1 后直伤帧——lv 经 cards 槽带入续行段，偶数层即时结算） | —— | swd 134/134 |
| 7 | step/mtg.rs | 3 | 冰河要塞（换地摸 2 后 he min2max2 弃两选——注释明言「摸 2 保证手牌恒 ≥2」） | 水没墓穴（弃后摸 2 步末）；羊皮卷（无锦囊补偿摸 1 步末） | nc08 mtg 9/9＋bingheyaosai 1/1；g1_shahai mtg 11/11 |
| 8 | step/hearth.rs | 2 | 盾牌（+1 护甲摸 1 后「弃首张手牌」——空手时同步弃刚摸的牌）；传送（摸 1 后本体入弃堆——洗牌边界序） | —— | nc08 hearth 13/13＋chuansong/dunpai 各 1/1 |
| 9 | step/xianjian_cards.rs | 2 | 灵泉（觉醒摸 3 后授水云+内置置牌 min1max1 询问——「摸 3 的牌含刚入手牌末」） | 忘忧（逐目标摸 1——机器既有 `Flow::Yield` 即续行位，R 弹栈回本段取下一目标，凤略同型） | g1 xiaoman 2/2、wangyou 1/1；r05c2_lingquan 绿 |
| 10 | step/skills_cards.rs | 2 | 让节「获得牌」臂（摸 1 后开类型询问——同步序＝先摸走顶牌再扫牌堆；WaitChoice 挂起期间 R 帧不泵，若同步内开询问则恢复步扫到未摸走的顶牌） | 颂词（摸 2 步末；songci_record 纯状态翻转） | nc08 rangjie 2/2（1 ignored 既有）；swd songci 4/4 |
| 11 | step/swd_damage_reactions.rs | 1 | 黑火（摸「弃后手牌数」后读 `hand.len()>=8` 自伤 3 火——须读摸后手牌；draw_n==0 臂即时判定） | —— | g1_shahai swd heihuo 1/1 |

「直迁」判定纪律：仅当调用点后同 step 无手牌/牌堆/弃堆读、且尾段为纯状态
翻转（无日志无帧，如 food_tick/songci_record/技能实例移除）或步末无后续
（R 泵在步 Done 后立即发生、中间无其他结算）时直迁；凡尾段含 to_discard/
emit 帧/读手牌，一律续行位（洗牌与帧序边界下序敏感）。

## 4. 红测（r05c_async_draw.rs 续加 4 个，groups/misc.rs 已登记）

| 测试 | 决策面/族 | 红（同步） | 绿（迁移后）断言 |
| --- | --- | --- | --- |
| r05c2_mengpo_level0_spawn_is_card_silent | boss 孟婆（决策一） | 乱入座位存在 CardsDrawn 日志（seed 3 断言失败） | 1..80 扫描：乱入座位手牌空＋全程无该座位 CardsDrawn（≥3 个种子核验） |
| r05c2_zhaoyao_exhausted_draw_detaches_ask_opens | xianjian_card_use（照妖） | 同步摸 2 取走第 3 张入手（hand 3≠2 红；有 CardsDrawn{0}；detached 0） | R 帧 held 1 后枯竭转 detached+1、不入手、无 CardsDrawn；询问仍开（min2max2、选项=预置两张）——照妖原生触发在主摸 R 后（登记时序），布 3 张=主摸 2+照妖 1 |
| r05c2_lingquan_exhausted_detached_ask_opens | xianjian_cards（灵泉） | 同步摸 3 取走最后 1 张入手（hand 2≠1 红） | held 1 转 detached、手牌不变、无 CardsDrawn；置牌询问仍开（min1max1、选项含预置牌） |
| r05c2_fuchou2_exhausted_no_panic_virtual_sha_skipped | wudian（决策二） | panic "virtual use: deck empty"（machine.rs:629） | 不 panic；标记消耗；手牌不变（0 held 无 detached）；无 CardsDrawn |

红证据：c2-01.log（wave0 提交 0c8405bc0 前，4/4 红，另 3 个 C 批守卫绿）；
绿证据：c2-03-r05c-green.log（7/7 绿＝C 批 3＋本批 4）。

## 5. 失败分流账（全量 c2-02-full-run.log）

全量失败 1 项：

| 测试 | 类别 | 处置 |
| --- | --- | --- |
| replay_scenarios::replay_scenarios_match_golden | 预存失败（C 批末即红） | 不重录、不改期望值。与 C 批末（c-async-04-final-full.log）比对：失败场景集同名同 **14 个**、**14 个 actual 哈希逐字节相同**（diff 验证，本批零新增漂移；sancai_nested_chain/rollback_invariants 位移＝C 批既定预期，其余 12 个与 G 批基线相同）。 |

另有 3 项定向红（迁移过程中即时分流，非全量失败）：

| 测试 | 类别 | 处置 |
| --- | --- | --- |
| nc08_upstream_yunchou_jiaoxia（:147 源码锁） | a) 断言锁旧实现形态 | 核对：摸 1 步末纯摸牌、行为面不变（狡黠行为测试绿）→ 更新断言为 `start_draw(self.owner, 1)` 并注明 R05-C2 与理由。 |
| nc08_upstream_yunchou_yaliang（:115 源码锁） | a) 同上 | 核对：座位序逆压 R 帧＝同步升序取牌、`sort_unstable` 保留 → 更新为 `start_draw(*s, 1)` 并注明。 |
| nc08_upstream_yunchou_kuanggu（:135 源码锁） | a) 同上 | 核对：距离>1 摸 1 臂纯摸牌 → 更新为 `start_draw(self.owner, 1)` 并注明。 |

同步更新 1 个既有 ignored 守卫（非失败）：nc08_upstream_rangjie.rs 的
`upstream_rangjie_move_branch_must_draw_one`（登记缺陷源码事实固定）字面
`draw_cards(self.owner, 1)` → `start_draw(self.owner, 1)`——缺陷本体
（分支 1 未摸牌）未变，仍 ignored。

分流计数：a) 3（源码锁）；b) 0（无行为真回归）；c) 0（无拿不准项）。

## 6. draw_cards 删除与零调用证明

- crates/engine-native-kernel/src/machine.rs：`draw_cards` 方法连同 C 批
  防回流 doc 注释整段删除（closeout 提交 566016366，-35 行）；
  `draw_judge_card`（判定牌语义）未动。
- 批末 `rg -n "\.draw_cards\(" crates/ --type rust`＝**0 调用**；唯三命中为
  tests 注释文本（nc08_upstream_rangjie.rs:39、nc08_upstream_swd_qinglong.rs:46、
  nc08_upstream_jiyuan.rs:162——上游依据引文，非调用）。
- 防回流：方法已不存在，新增同步摸牌无法编译；新摸牌唯一入口 `start_draw`。

## 7. 门禁

- `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`：OK（2 tests）
- `python3 -B scripts/check-source-lines.py`：exit 0（9083 文件 ≤2000 行；Rust 非测试 ≤1000 有效行）

## 8. 波次提交

| 波次 | 提交 | 内容 |
| --- | --- | --- |
| wave0 | 0c8405bc0 | 红守卫 4 个（修前 4/4 红，c2-01.log） |
| wave1 | 4605601f5 | boss.rs 8 处＋HostStage::DrawSettle（mengpo L0 删摸牌/回滚整段） |
| wave2 | d721a71d0 | yunchou.rs 7 处（beige/diyguhuo 续行；3 源码锁＋ignored rangjie 更新） |
| wave3 | 9dde37da5 | gujian.rs 6 处 |
| wave4 | ffd112773 | wudian_status.rs 5 处（fuchou2 决策二落地） |
| wave5 | 6d291eb32 | xianjian_card_use.rs 3 处 |
| wave6 | bb0b6b61f | swd_equipment.rs 3 处 |
| wave7 | c67c0a18f | mtg.rs 3 处 |
| wave8 | 202b49cd2 | hearth.rs 2 处 |
| wave9 | 44ffe11ac | xianjian_cards.rs 2 处 |
| wave10 | bbb56320e | skills_cards.rs 2 处 |
| wave11 | 3df780c1d | swd_damage_reactions.rs 1 处 |
| closeout | 566016366 | 删除 machine.rs draw_cards（零调用证明＋全量＋门禁） |

## 9. 证据文件

- c2-01.log：红守卫 4/4 红（同步路径）
- c2-02-full-run.log：终态全量 1958/1/75（14 golden actual 与 C 批末逐字节同）
- c2-03-r05c-green.log：r05c 守卫 7/7 绿（C 批 3＋本批 4）
- c2-report.md：本报告
