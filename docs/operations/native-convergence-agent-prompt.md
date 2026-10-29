# 原生架构收敛完整执行提示词

将下面整段交给执行 agent。方案正文保存需求与验收条件，实施状态以进度主稿和剩余台账为准；本提示词只列工作项编号与先后，不复制一套当前状态。2026-10-29 晚扫描后更新（新增：九批未提交工作的处理、B9、F7 第 3–4 步、Rust 有效行与分层棘轮门禁、测试分组约定）。

```text
请在 /Users/ruyimg/Documents/Claude/sanguosha/noname-next 把剩余全部未完成工作做完：完整执行已采纳的《原生架构收敛执行方案》，并清空原生收敛剩余台账里的全部未完成项。目标是全部实现、集成并验收，不是再做一份建议、只完成首批修复、只搭接口或只运行代表样本。不以时间成本压缩范围。

一、先读（按顺序），并遵守 /Users/ruyimg/Documents/Claude/sanguosha/AGENTS.md 及适用的更深层规范：
1. docs/architecture/native-progress.md 顶部「状态扫描（2026-10-29 晚）」与优先级表（当前基准）
2. docs/architecture/native-remaining-ledger.md（统一剩余台账：NC01–NC12、UP37-R/N/I、PA01–PA12、当前阻断 B1–B9、2026-10-28 与 2026-10-29 进展登记）
3. docs/architecture/native-convergence-plan.md（完整目标、NC01–NC12、六阶段与验收门槛）
4. docs/operations/evidence/2026-10-28-user-decisions/README.md（已授权的版本、协议与修复范围；授权不等于验收）
5. /Users/ruyimg/Documents/Claude/sanguosha/NONAME_NEXT_AGENT_SPEC.md（实施约束与验收检查表；位于 Git 仓库外）
6. docs/migration/upstream-extension-reconstruction.md（两仓全部 37 来源、E01–E06/G0–G5）
7. docs/architecture/architecture-evolution-plan.md 的 F7（engine-native 三层拆 crate 方案）与 docs/architecture/native-dsl-migration-plan.md（PA12 迁移计划）
8. docs/architecture/native-extensions.md、docs/architecture/native-rules-baseline.md、docs/native-coverage-manifest.md、noname-performance-architecture.md
9. docs/operations/self-check-guide.md（提交前门禁与测试约定）

二、开工前先核对工作区。提交基线 HEAD 之上有九批**未提交**工作（进度主稿顶部①–⑨），其中⑧拆 crate 第 1–2 步与⑨文件拆分改动面很大。先查 git status 与各批证据，确认它们仍完整可编译；然后**先问用户是否按批入库这九批**（每批一个提交、提交信息对应证据目录）。未获明确答复前不要提交别人的改动，也不要开始需要干净工作树的工作（F7 第 4 步）。保留所有既有修改和未跟踪文件，不通过 reset/clean/stash 丢弃工作；遵守本地 Git 政策，不 push、不新增 remote。结构查询优先 CodeGraph；不以旧文档代替源码。

三、剩余工作（按依赖先后；每项的当前状态与完成条件以台账对应行为准，完成后在台账行内更新）：
1. 当前阻断：B9（engine-core 对延迟队列中的 ResolveOlJieDanshou 求状态哈希落入 unreachable!）——先写会红的回归，再补编码分支，核对 golden 不变。
2. F7 拆 crate：第 3 步把内核用到的技能名/键常量、Gender、辅助函数下沉或改 Catalog 回调、两处 apply_general_assembly 上调去掉，使 scripts/check-native-layering.py 计数降到 0（每降一次同步下调 BASELINE）；第 4 步在九批入库后的空窗，用 git mv 机械拆为 engine-native-kernel / engine-native-content / engine-native（门面 re-export，下游与测试路径不变），复测冷构建与改动后重编时间并写证据。engine-core 不拆（F2）。
3. PA12 单一引擎收敛：按迁移计划继续各批，含批⑥主动技桥与 §2 J 其余部分；R01/R03/N01 的原生移植（涤仇选择状态机、冯河/英霸、胆守/平寇）。旧线冻结新增规则，只修缺陷并保持可编译与 NC08 差分基准。
4. UP37：R02、R04–R10、N02–N07 直接进原生（优先 DSL→rule-vm 包，原生缺的 ABI/事件先补原生）；I01–I05（分包差异、旧锁兼容矩阵、资源/语音/皮肤、依赖/配置、验收数据）。
5. NC01–NC12 按六阶段推进到台账完成条件：座位与触发语义全矩阵；包身份与 VM 生命周期（签名、资源字节、续行来源认证、跨包规则）；产品 API 与 UI 全流程；VM 故障矩阵；三类真实扩展案例（剩混合包、依赖、卸载、旧锁恢复）；内容私有状态迁出核心（含 PA04）与字段同源；跨目标对照扩到多模式/上游差分/37 来源；原生联机与目标平台；AI 非静止态信息边界与质量基线；文档收敛（NC11）；WAL/批提交与旧线退出（NC12）。
6. 37 来源：逐来源完成 G0–G5 与八项验收（功能、配置、依赖、组合、保存恢复、目标平台、性能、规则差异）；前三类样本不能替代其余来源，不得把缺失功能改叫登记差异或排除出整包验收。
7. 性能与平台：PA08 真机（Android/iOS 整局、≥30 分钟功耗/温度/降频）、PA09 桌面原生引擎原型（按已裁决验收条件）、PA11 条件触发后再定；GitHub Actions 实跑一次并登记（本地通过与 CI 实跑分别记）。
8. 第六阶段：替代路径与全矩阵验收成立后完成原生产品切换、安全退出旧执行入口，保留上游来源、正式 ADR、原始证据与仍需的迁移/恢复能力。

四、工程约束：
- 保留单写 Rust 内核、不可变目录、显式结算/续行、内置 Rust＋第三方 rule-vm 路线；复用 PackageStore 与现有协议/登记工具，不并建第二套包管理或状态台账；不自动立项历史提案中的 ECS、QuickJS、全面 VM 化、强化学习等候选。
- 单文件门禁：受维护源码/测试/DSL ≤2,000 行；Rust 非测试源码有效行（去注释、空行、#[cfg(test)]）≤1,000。按功能、来源包或项边界拆分并起有意义的名字，不压缩代码、不机械切块。抽函数时检查原代码里的 return / return Ok / ? 是否改变控制流。
- 分层棘轮只降不升：engine-native 内核层需要内容行为时加 Catalog 回调，不新增对 content/engine 的引用。
- 集成测试按分组编译：engine-native、engine-core 新增 tests/*.rs 必须登记到 tests/groups/<组>.rs；单跑写 cargo test -p <crate> --test <组> <文件名>::。
- 每项先建立能暴露问题的回归，再修共享根因，同步调用方、测试、配置、导出、资源和文档。规则/存档变化核对版本、完整状态、事件、RNG 和续行；golden 变化先解释原因，不能只改期望值。
- 新立 ADR、改协议（含 VM 提交与选择/命令协议）、升规则版本/存档格式/ABI 前先查裁决登记与台账：已授权的按登记范围实施，超出的先向用户提交方案并取得明确答复；同一版本号只能被一批占用。

五、验证（按影响面；命令与期望见 self-check-guide）：python3 -B scripts/preflight.py（21 项）；行数门禁及其单测；cargo test 对应 crate 全量（engine-native 改动另跑 Gate 1b：独立 CARGO_TARGET_DIR 下 RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets）；cargo test --workspace；cargo run -p conformance；前端 pnpm test 与 typecheck；从当前源码重建 WASM 后的 Worker smoke 与 headless Chrome 产品路径；scripts/check-native-wasm-cross-target.py 与 scripts/cross-target-hash.sh；真实客户端/服务端与跨进程恢复；上游对照。旧线与原生线分开统计。性能结论须同设备同工具链同负载的前后数据。编译通过、定向通过、全量通过、测试文件存在都不能代替产品/平台/规则等价验收。

六、记录：当前状态、失败项与下一步只更新 native-progress 与剩余台账；来源状态只更新扩展台账；规则差异只更新规则基准；原始命令与输出写入有日期的 evidence 目录。每轮记录日期、HEAD 及 dirty 指纹、规则/格式版本、工具链/设备、命令、通过/失败/忽略和证据；区分已实现、已修反例、已加回归、定向通过、全量通过、平台通过与上游等价。移动或拆分源文件后同步修正文档里的 file:line 引用（check_doc_line_refs --strict）。

七、协作：可用子 agent 并行处理互不冲突的实现或验证，先划清文件所有权和接口，主 agent 负责整合与最终验收；公共文档写入前重读；不要同时抢写核心文件或清理共享 target。在授权范围内自主推进，不反复询问是否继续；缺少设备、素材、凭据、权限或产品语义时，明确所需输入与未验收门槛，同时继续所有不依赖它的工作，不擅自补造答案、缩小范围或把阻碍写成完成。需要交接时保存精确的完成/剩余清单与下一步。

最终逐项报告 B9、F7、PA01–PA12、UP37 各项、NC01–NC12、六阶段和 37 来源的结果与证据，列出仍未完成/未验证项。只有全部门槛真实通过才可声明“全部完成”；否则明确整体尚未完成，说明已完成部分和真正剩余阻碍。现在先核对工作区并就九批入库征求用户意见，然后开始执行。
```
