# NC08 触发全序复核 + NC09 选择窗超时：反例与方案（2026-10-28，仅取证）

- 实际执行：2026-10-28（[开始](start-time.txt)）；源码基准 [`ec672a1d`](source-head.txt) + 本会话 B7/NC03 批工作树（规则 v324）。**本批不改触发排序语义、不新增命令或协议字段、不升任何版本**——按领取顺序第 5 项限定，只做反例与方案。
- 登记来源：[规范审查证据](../2026-10-28-agent-spec-review-v1.2/README.md)（NC08「触发全序」定案复核、NC09 选择窗超时缺失）。

## NC08：触发全序定案复核——反例已取证

### 现行定案（复核对象）

规则基准「触发全序」（2026-10-16 定案）：同角色同优先级非 silent 技能按 `SkillDefId` 决胜的确定性全序（`resolution.rs` `plan_triggers`），不向玩家开放顺序选择。定案理由「上游交互并列不可无交互复现」。

### 上游对照（既有摘录，本批引用）

上游 `arrangeTrigger`（`content.ts:~4017-4058`，摘录已存 [2026-10-17 证据 sources.txt](../2026-10-17-cov-permanent-diffs/sources.txt)）：普通时机下，同优先级多个可用技能且无 silent/无描述技时，置 `event.choice` 交 `chooseControl` **由玩家选择下一个触发的技能**；firstDo/lastDo 时机不做优先级选择。

### 原生反例（新探针，[nc08_trigger_order_counterexample.rs](../../../../crates/engine-native/tests/nc08_trigger_order_counterexample.rs)，[日志](nc08-probe.log) 1/0）

两个可选 VM 技能（同一 `TriggerMeta`：PhaseEnd/Normal/priority 0/optional，Always 滤镜，效果分别为摸 1/摸 2）授予座位 1：

1. 注册序 [甲(摸1), 乙(摸2)] → 双 confirm 门依次应答后手牌增量 **[1, 2]**；
2. 翻转注册序 [乙, 甲] → 增量 **[2, 1]**——执行序严格跟随 def（注册）序，玩家无任何影响途径；
3. 两个对局的整个触发窗口内**只出现**逐技能 confirm（`trigger_confirm`）与常规阶段询问，**不存在任何「选择发动顺序」询问**——与上游 `chooseControl` 的交互并列即分歧点。

该测试同时补上 GT-002「现行定案的确定性全序回归」缺失的锁面（此前待查）。

### 复核方案（供裁决，未实施）

- **选项 A（维持现状）**：确定性最强（AI/回放/跨端无交互依赖），差异已在规则基准登记；代价是上游交互语义不还原。可先做**差分统计**（不改语义）：扫 37 来源技能对，统计「同角色同事件同优先级且均非 silent/均有描述」的真实命中面——命中面小则维持 A 有数据支撑。
- **选项 B（还原玩家可选顺序）**：同优先组 >1 非锁定技时先向持有者开 `OpenChoice(prompt="trigger_order", options=Tag(技能))`，选择作为普通命令进入命令流（可确定性回放）；AI 座位按策略默认（现 def 序）。影响面：触发全序键变更、规则版本升号与 golden 重录、新增询问投影须 native/WASM/服务端三处一致、OptionalGate 前置排序窗与恢复轨迹——**须另立 ADR 并经用户授权升规则版本后方可实施**。
- **建议**：先做选项 A 的差分统计（独立小批、零语义变化），以数据决定是否立项 B。

## NC09：选择窗超时——方案（未实施）

### 现状缺口

原生房间只有离线座位的惰性回收（`room_native_actor.rs`），**没有任何入口在选择超时后产生命令**；旧线已有 `engine_api::CommandKind::TimeoutAction { choice_id }` 形态，原生无对应动作。上游联机按 `choose_timeout` 倒计时、到时 `ui.click.auto()` 托管。

### 方案骨架（供裁决；实施须先授权）

1. **时限与到时动作**（须用户定）：按 `ChoiceKind` 分级——如 confirm/选牌/选目标各定默认（上游 `ui.click.auto()` 语义=按 auto 键的默认项）；房间级可配置上限与缺省值待定。
2. **架构落点**：计时器在**房间 actor（宿主侧）**，规则核心不读时钟（§0 约束不变）；到时由 actor 以**显式命令**提交进命令流，复用现有 `Answer`/去重/回执两层机制（`dedup_id` 高水位 + `(reconnect_token, idempotency_key)` 回执），不新建命令体系。
3. **命令形态**：对齐旧线 `TimeoutAction { choice_id }` 语义——**新增原生 `GameAction` 变体即命令/协议扩展，实施前按 §0.10 与台账授权门槛单独请示**；存档与 WAL 只记录该命令（命令流本已持久化，无格式变化预期，仍须验证）。
4. **确定性**：超时决策在宿主、效果经命令流——回放与跨端一致；同一 `choice_id` 的重复超时提交被去重拒绝（与人工应答同路径）。
5. **覆盖矩阵**（实施批验收）：断线中超时、重连后不重复执行、恢复后窗口继续、重复提交幂等、观战/裁判无超时权。

## 验证

- NC08 探针：`cargo test --offline -p engine-native --test nc08_trigger_order_counterexample` **1/0/0**（[nc08-probe.log](nc08-probe.log)）。
- 本批零源码语义改动（新增测试与文档除外）；未运行全量（探针定向绿即足，同日 B7/NC03 批已各跑影响面全量）。

## 收尾补充

含本批 NC08 探针的 engine-native 最终全量：`cargo test --offline -p engine-native --no-fail-fast`
**1618/0/73（332 测试二进制）**（[final-native-full.log](final-native-full.log)，2026-10-28 会话末）。
