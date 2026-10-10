# 2026-09-23 第 17 轮：P1 修复批次与规则版本 29

本轮为**实现批次**：按[进度主稿](../../../architecture/native-progress.md)的 P1 优先级修改引擎与工程测试，
第 16 轮登记的上游规则反例与内核开放反例多数已改为正确行为断言。
`STANDARD_RULESET_VERSION` **28 → 29**（护驾/激将机器快照字段、仁德回合累计、阶段门控、
字节存档 v2——ADR 0776）；`ARCHIVE_FORMAT_VERSION` **1 → 2**（OpenChoice 显式编码 ChoiceKind，
v1 字节经版本门禁拒绝）。14 个 golden 场景哈希因状态编码与语义变化重录，原因已登记。

| 验证 | 结果及边界 |
|---|---|
| [原生全量测试](native-tests.log) | `cargo test --offline -p engine-native`：**266 passed / 0 failed**（第 16 轮 238 + 本轮新增 28 项回归）。测试全绿不代表与上游规则等价。 |
| [独立探针](probes.rs) / [输出](probes.log) | 21 项 `FIXED_*` 复核原反例已关闭；1 项 `OPEN_*` 诚实保留（单选候选枚举缺合法闪，P2）。 |
| [源码摘要](source-hashes.json) | 本轮 9 个关键源码文件 SHA-256。 |

## 本轮关闭的反例（探针逐条复核，工程回归同步落库）

- **N11 字节存档 v2**：`nullify_window`（实为 Confirm）、`jiji_target`（实为 SelectPlayers）
  曾被静态映射为 RespondCard 导致 OpenChoiceMismatch；`tian_du_gain` 未注册。
  现在 ChoiceKind 显式编码进字节、注册表保留未知 prompt 拒绝、v1 字节被版本门禁拒绝。
  回归：`archive_choice_kind_*` 5 项 + `archive_rejects_unknown_prompt_and_v1_bytes`。
- **N11 恢复派生**：`active_ask` 缺 JiJi 分支（遗计给牌两阶段挂起恢复必失败）已补；
  `respond_shan` 派生补齐龙胆（手牌杀）/倾国（黑色装备）选项；护驾/激将/离间挂起态派生
  与运行时同源。回归：boundary_snapshot 65→68、native_kernel 各恢复用例。
- **N04-f 牌归属完整核算**：只查「0..最高已见连续」会放行删最高牌/清空全部牌区的快照；
  改为目录全集逐一断言（牌区 ∪ 机器暂离持牌）。回归：`restore_rejects_{trailing,all_cards,middle}_card_missing`。
- **N10 命令序号**：规划克隆模拟提交曾用固定 `1_000_000 + nodes`——高水位 2,000,000 时
  合法杀候选全部被当重复拒绝、规划退化为 NextPhase；统一为克隆会话高水位推进；
  `u64::MAX` 耗尽经 saturating 干净拒绝（不回绕/不 panic）。回归：tests/ai_planner.rs 3 项。
- **N10 AI 信息集（原第 16 轮 `OPEN_AI_SAMPLED_OWNERSHIP`）**：`redact_opponent_hands`
  曾从真实牌堆采样并**复制**牌 ID 进对手手牌——重复物理归属（恢复被拒）且牌堆空时保留
  真实暗牌。现在把「对手暗手牌 + 牌库」整体作为观察者的**未知牌池**，座位号播种的 LCG
  Fisher–Yates 洗牌后按原数量重新分配：物理牌集合守恒（只移动不复制，重分配后克隆可
  恢复）、牌库序改变（不读真实暗牌归属）、牌堆空时退化为对手暗牌重排（仍守恒）、
  同种子确定。合法性依据：未知牌在未知槽位间的任何分配都与观察者知识一致。
  回归：tests/ai_planner.rs 新增 4 项（归属守恒+多重集、确定性与空牌堆退化、
  未知池洗牌、对手持暗牌时规划→应用闭环）。
- **N08 主动技阶段门控**：`ActivateSkill` 曾只查当前回合玩家（仁德可在判定阶段发动）。
  现默认限出牌阶段（上游 `enable: "phaseUse"`），激将经目录 `active_outside_play` 声明
  例外（上游 `chooseToUse/chooseToRespond`）。回归：`activate_skill_rejected_outside_play_phase`
  （含状态不变与恢复件同样拒绝断言）、`jijiang_activate_not_phase_locked`。
- **N08 仁德累计**：曾每回合限一次、按单次给牌数回血；现在出牌阶段可多次发动，
  `TurnState.rende_given` 按阶段累计（入快照/哈希/字节存档），跨过 2 张回复一次。
  回归：`rende_cumulative_crosses_two_across_activations`（含字节往返后累计口径不变）。
- **N10/N08 护驾重构（ADR 0776）**：曾预筛暗手牌持闪者、只问一人、不限主公/魏势力。
  现按目标下家起座位序逐人询问**魏势力**角色（不以暗手牌预筛），拒绝继续、确认代出
  闪（计入无双需求）、身份局要求主公资格（`is_lord`）；队列/被问者入快照（内存+字节
  恢复）；holder 越界（254）拒绝。回归：`hujia_*` 4 项新用例 + 门控/资格/无双组合。
- **N10/N08 激将重构（ADR 0776）**：曾由刘备从「预筛暗杀的蜀将」中选一人、首人拒绝即终止。
  现按座位序逐人询问蜀势力角色、拒绝继续、借出的杀经同一 `ShaResolveMachine` 直结算；
  全部拒绝时取消发动并回滚 once_per_turn（对齐上游「无人响应则使用取消」）；主公资格同上。
  回归：`jijiang_pending_ask_restore_continues` 及 3 处旧断言更新为逐人询问语义。
- **N10 信息隔离**：互换两名魏将暗手牌中的响应牌，攻击者可见视图与公开询问序列完全
  相同（询问资格公开、应答私密）。回归：`hujia_public_asks_do_not_leak_hidden_shan`。
- **E02/E03 扩展接线**：注册表拒绝未安装包的 enable/disable/uninstall（旧反例
  `OPEN_EXTENSION_VALIDATION` 关闭）；重复安装返回错误；冲突（与基础目录同名、包内
  重复）从 panic 改为可处理 `ExtensionError`；`standard_catalog_with_extensions` +
  `Engine::new_with_extensions`/`restore_with_extensions` 将启用扩展卡牌合并入真实目录，
  目录版本绑定扩展集合指纹（内容锁）——带扩展存档被纯标准恢复拒绝、同名不同版本拒绝、
  同集合恢复同哈希。回归：`e02_registry_rejects_uninstalled_operations`、
  `e03_extension_catalog_content_lock_in_game_and_restore`。
- **离间恢复派生**：lijian_targets 恢复派生补性别过滤（与运行时 `seat_is_male` 同源）。

## 仍开放（探针如实保留）

- `OPEN_AI_CANDIDATES`：单选询问候选仍 `options.take(max)`，max=1 时漏合法闪（P2）。

（原 `OPEN_AI_SAMPLED_OWNERSHIP` 已在本轮内修复并转为 `FIXED_AI_SAMPLE_OWNERSHIP`；
`redact_opponent_hands` 现为未知池确定性重分配——物理牌集合守恒、恢复通过、
对手持暗牌时规划→应用闭环成立。）

## 边界

- 262 项工程测试全绿不等于与上游规则等价；标准包 33 将/108 牌/12 模式差距、
  KOF 会话一致性、E02–E06 的真实旧包转换与规则宿主仍按进度主稿待办。
- 本轮未改 content-runtime 旧包发现器；两仓旧扩展根直接导入仍为 0（以第 16 轮探针为准）。
