# ADR 0703：原生内核第一批落地——只读规则目录 + 紧凑对局状态 + 原生技能暂停恢复

- 状态：已接受（2026-09-15）；本文保留首批决策。2026-09-16 当前进度见 [核验记录](../architecture/native-progress.md)，扩面见 [覆盖清单](../native-coverage-manifest.md)。
- 关联：`crates/engine-native/`（新增 crate，仅入 workspace members，不入 default-members）；
  背景评审结论见会话目标「六处必须修改」；保真度台账口径见
  [content-fidelity-ledger](../content-fidelity-ledger.md)（两轴独立，勿混用）。

## 背景

架构评审确认旧 `engine-core` 的三处结构性问题不适合继续叠加：

1. `GameState` 按值内嵌整个 `ContentRegistry`（字节码 + 内容目录），`Engine::submit`
   与 `Transaction::begin` 各克隆一份——每次命令至少两次全状态深拷贝，但 registry 本体不编码进 canonical hash（经 ruleset 逻辑哈希绑定；2026-09-16 更正）。
2. 事件"嵌套"不是栈：单槽 `resume_kind/resume_payload` + 三个延迟队列
   （`deferred_effects/deferred_triggers/deferred_tail_effects`）承载被打断的工作，
   `ContinuationKind` 约 340 个变体、载荷为 `Vec<u32>` 无类型数据。
3. 具体技能逻辑大量写死在 `effect/primitives/pack_*` 与 `resolve_choice` 通用内核中，
   仅删除 VM 无法解耦。

## 决策

新建 `crates/engine-native` 作为干净原生内核的落地验证场，第一批实现并验证
「只读规则目录 + 紧凑对局状态 + 原生技能暂停恢复」链路：

### 1. 三分结构（对应评审②）

| 部分 | 位置 | 内容 |
|---|---|---|
| 只读规则目录 | 引擎侧 `Arc<Catalog>` | 卡牌/技能定义、触发元数据、分发索引；命令前后指针不变 |
| 对局状态 | `GameState { core, resolution }` | 玩家/牌区/技能实例/RNG/回合旗标 + 事件栈/挂起机器/开放选择 |
| 宿主数据 | 引擎侧 `Vec<LogEvent>` | 展示/录像日志；不进状态、不进哈希；拒绝时按长度标记截断 |

工作副本回滚：`submit` 克隆一份紧凑状态（单工作副本），拒绝/预算耗尽整段丢弃，
状态逐字节不变。目录与宿主数据不参与克隆与哈希。

### 2. 显式事件栈 + 类型化续执行（对应评审③）

- `Resolution.stack: Vec<Frame>`：帧含 `id/parent/kind/EventData/stage/queue`。
  帧推进为均匀状态机 `BeforeTriggers → Commit → AfterTriggers [→ Intrinsic] → Done → 弹栈`；
  伤害帧以前置触发窗口为初始阶段，Death 帧以 Intrinsic（先结算后触发）。
- 载荷类型化（`EventData::Damage(DamageData)` 等）；技能在窗口内直接改写帧载荷
  实现转移/防止（`redirect_chain`/`prevented`），提交阶段读最终值。
- 挂起机器 `parks: Vec<Park>`：每台机器记录 `resume_depth`，栈深回到该值时恢复；
  机器 `fork()` 支持暂停态随工作副本克隆（这是 Clone 而非重放的必要条件）。
- 选择：机器返回 `Flow::WaitChoice { observer, spec }`；`ChoiceSpec` 选项类型化
  （`ChoiceOption::{Confirm,Pass,Card,Player}`）；内核校验应答为规格子集且数量合规，
  应答只消费一次。
- 可选触发经 `OptionalGate` 闸门（确认后才启动内层机器，内层后续选择全权委托）。
- 触发队列全序：`Timing → Priority(降) → 强制先于可选 → 与当前回合玩家的座位距离 → 定义号`；
  队列**运行前重查**实例启用/存活与 filter——肃清类技能得以在同一伤害窗口内
  使已入队的碎盾失效。
- 预算：`Resolution.budget` 每调度步 -1，耗尽返回 `BudgetExhausted` 并整段回滚。

### 3. 内核/内容分界（对应评审①）

- 内核（`kernel.rs`）：伤害提交与内联濒死检查、濒死救援队列、死亡结算与胜负、
  回合/阶段推进、杀的响应结算、弃牌上限——均为帧提交动作或固有 Intrinsic 机器。
- 内容（`content/standard.rs`）：五个代表技能，全部为 Rust 函数 + 类型化局部状态枚举：
  散财（可选触发→摸牌子事件→连续两选择）、移伤（前置窗口改载荷转移）、
  碎盾（每回合限一次防止）、肃清（窗口内禁用他人技能）、授艺（动态获得临时技能）。

## 首批历史验证（2026-09-15，13 项）

目录指针跨命令稳定；杀/闪/限一次/距离；嵌套选择链与放弃路径；伤害转移（含
`redirected_from` 日志）；濒死自救/他人救（放弃链式询问）/无人救死亡（手牌入弃堆、
技能移除）；结算中途失效；每回合限一次；动态获得至回合结束到期；非法应答/错误
choice_id/选择挂起期出牌全部拒绝且哈希不变；预算耗尽整段回滚；弃牌上限；
同种子同命令序列摘要一致、不同种子发散（blake3 静止点摘要，RNG 入哈希；不覆盖挂起机器内部状态）。

## 限制与后续（2026-09-16 整理）

本 ADR 的首批缺失列表与测试数已被后续六批机制扩面取代，当前唯一清单见
[原生覆盖](../native-coverage-manifest.md) 和 [重写进度](../architecture/native-progress.md)。
仍需完成挂起机器内部状态摘要、快照恢复、宿主时间片让出、正式规则语义及内容迁移；
不得将原型机制测试算作官方技能正确性或发布完成证明。
