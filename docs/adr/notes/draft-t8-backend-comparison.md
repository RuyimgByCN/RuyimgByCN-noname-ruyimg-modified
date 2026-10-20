# ADR 草稿：扩展执行后端选择（T8 / 演进提案 C1）

**状态：已由 [ADR 0782](../0782-extension-backend-and-legacy-js-scope.md) 定案（2026-10-12，用户决定 D2）**。本文保留为对照原型的输入证据，内容维持 2026-10-11 原文；下文“未证明”各项已转为 T8 集成验收条件。

日期：2026-10-11。原型：`crates/content-compiler/examples/t8_backend_prototype.rs`（可复跑）；原始输出见 [证据目录](../../operations/evidence/2026-10-11-t8-backend-prototype/README.md)。

## 样本口径

「同一个真实扩展」取 huanle-upstream-noname（G1 第二样本）的技能**酒尾**（`huanle_jiuwei`，包数据 `mapping: host:phase_end_recover_or_draw`），语义为「阶段结束：hp < max_hp 回 1 体力，否则摸 1」。

诚实声明：该技能在真实包数据中的形态是候选 A 的 `host:*` 键映射；原型的候选 B 形态（DSL 源码）是为对照原型按 kernel 臂语义逐句重写的（`JIUWEI_DSL` 常量内嵌于示例，语义断言与 kernel.rs 的 `PhaseEndRecoverOrDraw` 臂逐分支等价），**不是**把现有包数据自动翻译成 DSL——包数据到 DSL 的迁移工具不在本原型范围。

## 三候选对照（C1 轴）

| 轴 | A 现状：编译期 HostBehavior 枚举 | B rule-vm 字节码（本仓既有） | C WASM 组件（wasmtime/WIT） |
|---|---|---|---|
| 新技能无需重编译 | ✗：新行为必须改 host.rs/kernel.rs 重编译 | ✓ 实测：DSL/IR 为包数据，编译产物随包安装 | ✓（声称；未实测） |
| 选择挂起/恢复 | 编译臂内自管（各自实现） | ✓ 实测：`Paused{continuation}`→JSON 序列化→`resume` 落账所选分支 | 声称支持（未实测） |
| 随机受控 | 臂内直调引擎 RNG | ✓ 无随机指令；随机只能经宿主查询/效果（结构保证） | 宿主注入（未实测） |
| 内容锁与存档 | 臂状态走 snap 枚举 | ✓ 实测：continuation 绑定 module logic_hash，跨模块续跑被拒；JSON 往返一致 | 模块哈希/组件校验（未实测） |
| 超限 | 无界（受信编译代码） | ✓ 实测：fuel 截断确定性 trap（`OutOfFuel`） | 线性内存+燃料类（未实测） |
| 故障回滚 | 编译臂内自行保证 | ✓ 实测：trap 时排队效果整批丢弃（提交点协议：仅 `Halted` 落账） | 声称事务性（未实测） |
| 隔离边界 | 无（同进程受信代码） | 进程内沙箱：verify + caps 白名单 + fuel/栈/堆上限 | 最强（进程外/独立沙箱；未实测） |
| 桌面/移动/浏览器 | 三端同源（Rust→wasm-bindgen 出浏览器） | 三端同源（纯 Rust，无平台依赖） | 桌面/移动可行；浏览器为「JS wasm 沙箱内再跑 wasm 运行时」的嵌套问题，需专项验证（未实测） |
| 调用成本 | 编译直派，未单独测量（无解释器层） | **实测（本机初步值）**：酒尾单次触发含 verify ≈ 85 ns（release）/ ≈ 703 ns（debug），N=20000；解释器量级参考值，非验收口径 | 跨沙箱调用通常高于进程内 VM（未实测，不得引用具体数字） |
| 依赖 | 零新增 | 零新增（rule-vm/content-compiler 已在依赖图内） | 新增 wasmtime 级重依赖（离线依赖图现无） |

## 候选 B 实测明细（原型七场景）

1. **安装即数据**：3 技能 DSL 一次编译为 npack 形态 `ContentPackage`（trigger_index/modules/caps/logic_hash），复编译 logic_hash 逐字节一致。
2. **载体可执行**：模块 `to_bytes`/`from_bytes` 往返后 hash 不变、可执行（npack 即可携带该形态）。
3. **语义一致**：hp<max_hp 分支回 1 体力 / 满血分支摸 1，效果请求与 kernel 臂逐字段等价。
4. **挂起/恢复**：`choose(1,2)` → `Paused{continuation}` → JSON 序列化/反序列化 → `resume(token=2)` → `Halted` → 落账所选分支。
5. **内容锁**：continuation 绑定 module logic_hash；对改动版模块（同技能名、体改动、hash 漂移）续跑被 `InvalidBytecode("continuation module mismatch")` 拒绝。
6. **超限/回滚**：fuel=2 与死循环模块均确定性 `OutOfFuel`；trap 前已排队的效果在提交点协议下整批丢弃，不落半次账。
7. **成本**：单次触发（含每执行一次的 verify）release ≈ 85 ns、debug ≈ 703 ns（Apple Silicon、本机、N=20000；初步参考值）。

## 边界（本原型证明了什么、没证明什么）

已证明：rule-vm 后端的**内核能力**对 C1 必须项全部可实测满足，且能用真实扩展技能对齐语义。

未证明（定案前必须补）：
- **调度接线**：原型的触发调用是测试壳直调 `vm.execute`；engine-native 的触发调度（TriggerMeta/filter/once_per_turn/阶段推进）尚未接 rule-vm 模块——这是主要集成工作量所在。
- **效果提交管线**：HarnessHost 的排队/落账是协议演示；engine-native 的状态机效果路径（回收/摸牌/伤害机器、存档快照与 continuation 共存）未接线。
- **跨进程恢复**：continuation JSON 往返在单进程内验证；跨进程/跨端恢复未测。
- **恶意样本**：超限/回滚用了良性样本；verify/caps 对对抗性字节码的拒收矩阵未系统测量。
- **候选 C 实测**：wasmtime 不在离线依赖图，全部 C 行为桌评；任何 C 收益/成本结论均不得作为定案依据。
- **候选 A 成本**：未单独测量，不得写「A 比 B 快 X 倍」类未测量数字。

## 开放问题（待 D2 / 后续任务）

1. D2 决定旧 JS 兼容范围后，分层策略是否成立：存量包走候选 A（已验收的 240 行为），新发布技能走候选 B——若成立，B 的接线范围可先限定为「新技能增量路径」。
2. 候选 C 是否值得为浏览器隔离/第三方生态投入 wasmtime 依赖与嵌套沙箱验证，需 D2 与正式客户端需求（T6③）共同决定。
3. 包数据格式：`host:*` 键（A）与 DSL/IR/字节码（B）在 npack 内的共存与版本门禁（`rule_vm_abi_version` 已是既有轴）需在定案 ADR 中明确。
