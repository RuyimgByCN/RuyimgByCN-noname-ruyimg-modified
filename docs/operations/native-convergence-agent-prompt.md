# 原生架构收敛完整执行提示词

将下面整段交给执行 agent。方案正文保存需求与验收条件，实施状态以进度主稿顶部扫描和剩余台账为准；本提示词只列工作项、先后与待决事项，不复制一套当前数字。2026-11-02 扫描后重写，替换 2026-10-29 版（该版的九批入库、B9、F7 第 3–4 步均已完成并入库）。

```text
请在 /Users/ruyimg/Documents/Claude/sanguosha/noname-next 把剩余全部未完成工作做完：完整执行已采纳的《原生架构收敛执行方案》，清空剩余台账、扩展台账和规则基准里登记的全部未完成项。目标是实现、集成并验收，不是再写建议、只修首批反例或只跑代表样本；不以时间成本压缩范围。

一、先读（按顺序），并遵守 /Users/ruyimg/Documents/Claude/sanguosha/AGENTS.md：
1. docs/architecture/native-progress.md 顶部「状态扫描（2026-11-02）」及其后最近几条带日期记录
2. docs/architecture/native-remaining-ledger.md（NC01–NC12、UP37-R/N/I、PA01–PA12 逐项剩余与完成条件；B1–B9 已全部关闭）
3. docs/operations/evidence/2026-10-28-user-decisions/README.md（裁决 1–30；授权≠实现≠验收）以及其中链接的 24、27–30 号具体合同
4. docs/architecture/native-convergence-plan.md（NC01–NC12、六阶段与验收门槛）
5. docs/migration/upstream-extension-reconstruction.md（37 来源、E01–E06/G0–G5、八项验收）
6. docs/architecture/native-rules-baseline.md、docs/native-coverage-manifest.md、docs/architecture/native-extensions.md、noname-performance-architecture.md
7. /Users/ruyimg/Documents/Claude/sanguosha/NONAME_NEXT_AGENT_SPEC.md（实施约束；位于 Git 仓库外）

二、开工前核对工作区（2026-11-02 21:20 扫描时的状态，开工时以实际为准）：
- main HEAD `f5e560db`＝C331 phase7 合并提交（父 `a4483b64`＋`6d36f47a`）：规则 331／NGSA 106／content schema 5／ABI 758／WS7。本地无远端跟踪，未见发布与 exact-head CI 登记。
- 合并树已实跑：三 crate 1903/0/75、workspace 7154/0/77、行数门禁、preflight 全过；前端／WASM 重建／Worker smoke／跨目标／Tauri IPC 只有合并提交说明的自报数字，原始日志未入库；需 331／106 夹具的 7 个浏览器案例与 legacy WASM 重建未跑。
- `stash@{0}`：合并前搁置的 I02／NC10 未提交工作（35 文件），尚未恢复；`/private/tmp/noname-main-wip-backup-20261102T205333Z` 为同一份备份。
- 未跟踪 `target-strict/`（构建产物，不入库）；`git worktree list` 中大量 prunable 临时工作树。
先用 git status／git log／git stash show／git worktree list 复核，再动手。若发现其他 agent 正在同一工作树写入，先停下协调，不抢写。

三、第 0 步：收口 C331 入库
1. WS7 与裁决冲突：裁决 22／29 要求 C331 保持届时 WS6，若必须改协议须另报精确边界，且不得借用 R05 预约的 WS7。合并已把 ChoiceRequest 计数改为 u32 并升 WS7，未见授权。先写精确边界报告（改了哪些 wire 字段、旧 WS6 客户端／服务端如何被拒、能否在 WS6 内兼容），按用户答复执行：经批准由 C331 占 WS7、R05 顺延 WS8（同步更新裁决 28／29 的 R05 版本元组），或改回 WS6 兼容。答复前不发布（不 push）。
2. 在 `f5e560db` 上补跑并入库原始日志：pnpm test／typecheck／build:native、Worker smoke、check-native-wasm-cross-target.py、cross-target-hash.sh、cargo test -p desktop-client、需 331／106 夹具的 7 个浏览器案例；golden 或 conformance 变化先解释原因。
3. 第 1、2 项完成后按裁决 25 正常发布并独立登记 exact-head CI；本地通过与 CI 实跑分开记。
4. 在 331 之上恢复 stash@{0}（先在隔离工作树做冲突分析），逐文件处理与 331／106 的冲突，按其原证据重跑定向与全量门禁后再入库；不能整体丢弃。

四、剩余工作（按依赖先后；各项完成条件以台账对应行为准，完成后在台账行内更新）：
1. C331 闭环（裁决 22／26／29）：C9e 登记的 17 组反例与 chain 出生 P1 在 C9f–C9K 轮次的逐项关闭证据未入库——逐项补齐红绿证据或重新验证；实现共享 DeathFlow、PhaseBegin、死亡压窗、救援到期桥、indexed、forceDie/out/hidden/filterStop、rescue／operation LossEnd 等裁决 29 列明机制；注册 262/290 剩 28（16 译名上下文＋12 机制）全部完成，不得把缺失功能改记为登记差异。
2. 规则 332 改命取消（裁决 24），仅在实际 331 入库后。
3. R05 涕泣/宝梳/避祸：规则 333／NGSA 107／ABI 759／WS 按第三节结果（裁决 28／29），仅在实际 332 入库后。
4. NC10：19 项 NoCertifiedRoot、root／Confirm-prefix／Cancel-tail／VM 续行门禁、旧 park Q 先于 P 的前缀反例、AI 质量基线；不放宽裁决 27 的限定范围。
5. I02（裁决 30）：生产 presentation constructor 仍为 None，需接通真实公开投影、资源 binding、一次消费与自动语音；VM KillPlayer／DeathMachine 真实死亡语义需另报方案；其余皮肤／18 类资源／2487 缺 backing 继续。
6. UP37：R01 剩余（resolve_dcdichou 选择状态机、dcdichou_eff）、R02、R04、R06–R09、R10、N01 剩余、N02–N07、I01–I05，全部直接进原生（优先 DSL→rule-vm 包，缺 ABI／事件先补原生并按授权升号）。
7. NC01–NC12 按台账完成条件推进（含 PA04 内容私有状态迁出核心、NC07 字段同源、NC08 多模式与上游差分、NC09 Referee 全信息投影、NC12 WAL／批提交与旧线退出前提）。
8. PA12 单一引擎收敛按 native-dsl-migration-plan 继续；旧线冻结新增规则，只修缺陷并保持 NC08 差分基准。
9. 37 来源：逐来源完成 G0–G5 与八项验收（功能、配置、依赖、组合、保存恢复、目标平台、性能、规则差异）；样本不能替代其余来源。
10. 性能与平台：PA08 真机（中低端 Android＋iOS 整局、≥30 分钟功耗/温度/降频，裁决 19 已同意提供设备）；PA09 六条件（release 原型无稳定收益，保持 WASM 默认，除非新同设备数据证明收益）；PA11 按条件触发；三平台安装包的真实安装、签名／公证。
11. 第六阶段：替代路径与全矩阵验收成立后完成原生产品切换、安全退出旧执行入口，保留上游来源、正式 ADR、原始证据与仍需的迁移/恢复能力。

五、工程约束：
- 保留单写 Rust 内核、不可变目录、显式结算/续行、内置 Rust＋第三方 rule-vm 路线；复用 PackageStore 与现有协议/登记工具，不并建第二套包管理、registry 或状态台账；不自动立项 ECS、QuickJS、全面 VM 化、强化学习等历史候选。
- 行数门禁：受维护源码/测试/DSL ≤2,000 行；Rust 非测试源码有效行 ≤1,000。按功能、来源包或项边界拆分并起有意义的名字，不压缩、不机械切块；抽函数时核对 return／?／Ok 控制流。拆文件后同步文档里的 file:line 引用（check_doc_line_refs.py 会扫描历史证据）。
- 分层：engine-native-kernel ← engine-native-content ← engine-native（门面）；check-native-layering.py 棘轮只降不升，内核需要内容行为时加 Catalog 回调。
- 集成测试按分组编译：新增 tests/*.rs 必须登记到 tests/groups/<组>.rs；单跑写 cargo test -p <crate> --test <组> <文件名>::。
- 每项先建立会红的回归，再修共享根因，同步调用方、导出、资源、测试与文档。规则/存档变化核对版本门禁、完整状态、事件、RNG 与续行；golden 变化先查原因再决定重录。
- 版本号：同一号码只被一批占用；超出已登记授权的规则、存档、ABI、WS、schema 变化或新 ADR，先提交精确边界并取得用户明确答复。
- 编译后检查磁盘并清理本批不再需要的私有缓存（裁决 23）；不清共享 target，不删别人的工作树。

六、验证（按影响面）：
- 必跑：python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'；python3 -B scripts/check-source-lines.py；cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native；独立 CARGO_TARGET_DIR 下 RUSTFLAGS="-D warnings" cargo check -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets；cargo check --target wasm32-unknown-unknown -p engine-native-wasm。
- 产品面：cargo test --offline --workspace --no-fail-fast；cargo run -p conformance；apps/web-client 的 pnpm test／typecheck／build:native／smoke:native；python3 -B scripts/check-native-wasm-cross-target.py；bash scripts/cross-target-hash.sh；cargo test -p desktop-client；涉及 UI／Worker 的改动跑真实 headless Chrome 产品路径。
- 收口：python3 -B scripts/preflight.py 全部通过。首次失败与补跑分开记录，不以补跑抹去失败。

七、需要用户决定的事项（2026-11-02 扫描时的建议；以用户在对话中的明确答复为准，未答复前不执行对应的不可逆步骤）：
1. C331 的 WS7：建议先出边界报告；若 u32 计数确为真实宽选择所需且无法在 WS6 内兼容，请用户批准 C331 占 WS7、R05 顺延 WS8；在此之前不发布 `f5e560db`。
2. stash@{0}：建议在 331 之上恢复并单独验收入库，不丢弃。
3. 进度主稿瘦身：主稿约 830 KB、单行最长 5 万字，建议把 2026-10-31 以前的条目原样移入 native-progress-history.md（保留锚点），主稿只留顶部扫描、近期条目与待办；需用户同意后执行。
4. 临时工作树：建议只对 `git worktree list` 标为 prunable 的执行 git worktree prune，locked 与仍有未登记产物的保留；需用户同意。

七之二、协作：可用子 agent 并行处理互不冲突的实现或验证，先划清文件所有权和接口，主 agent 负责整合与最终验收；公共文档写入前重读；不要同时抢写核心文件。在授权范围内自主推进，不反复询问是否继续；缺少设备、凭据、权限或产品语义时，写明所需输入与未验收门槛，继续做不受阻的部分。

八、记录：当前状态只更新进度主稿顶部扫描与剩余台账对应行；来源状态只更新扩展台账；规则差异只更新规则基准；原始命令与输出写入有日期的 evidence 目录。文档统一用简体中文，每条记录写明日期、HEAD 与是否 dirty、规则/存档/ABI/WS 版本、命令、通过/失败/忽略和证据链接；区分已实现、已修反例、已加回归、定向通过、全量通过、上游等价。不得在提交前写“已合入／已提交”。

最终逐项报告：第 0 步、C331、332、R05、NC01–NC12、UP37 各项、PA01–PA12、I02、37 来源、六阶段的结果与证据，列出仍未完成/未验证项。只有全部门槛真实通过才可声明“全部完成”；否则明确整体尚未完成。现在先核对工作区，就第七节第 1 项提交边界报告并询问用户，同时开始不受该决定阻塞的工作（第三节第 2、4 项、C9e 证据补录）。
```
