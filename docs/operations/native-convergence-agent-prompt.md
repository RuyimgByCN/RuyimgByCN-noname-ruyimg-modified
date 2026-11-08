# 原生架构收敛完整执行提示词

将下面整段交给执行 agent。方案正文保存需求与验收条件，实施状态以进度主稿顶部扫描和剩余台账为准；本提示词只列工作项、先后与待决事项，不复制一套当前数字。2026-11-08 晚间核对后重写，替换同日早间版（该版的独立复核、NC08 22/23、37 来源四端产品路径与前后性能、发布与 exact-head CI 已由 Batch 23 完成并入库，核对与源码一致）。

```text
请在 /Users/ruyimg/Documents/Claude/sanguosha/noname-next 继续完成《原生架构收敛执行方案》剩余工作：实现、集成并验收，清空剩余台账、扩展台账和规则基准里仍未达成的完成条件；不是再写建议、改口径或只跑代表样本；不以时间成本压缩范围。

一、先读（按顺序），并遵守 /Users/ruyimg/Documents/Claude/sanguosha/AGENTS.md：
1. docs/architecture/native-progress.md 顶部「状态扫描（2026-11-08 19:57–20:30）」与其下 Batch 23 记录
2. docs/operations/evidence/2026-10-28-user-decisions/README.md 的裁决 33、34（2026-11-08 新增）及裁决 19、23、25
3. docs/operations/evidence/2026-11-08-batch23-nc08-guards-and-review/README.md（§3.3 苍螭之璧边界、§4 八项核对表、§4.3 PA08–PA11 与安装包、§4.4 第六阶段）
4. docs/architecture/native-remaining-ledger.md（NC08、NC12、PA08–PA11、37 来源各行）
5. docs/migration/upstream-extension-reconstruction.md（顶部 2026-11-08 各条、签署表、八项定义）
6. docs/architecture/native-convergence-plan.md（六阶段门槛）、docs/adr/0784-desktop-native-engine-prototype.md、docs/architecture/native-rules-baseline.md、docs/native-coverage-manifest.md、docs/architecture/native-extensions.md、noname-performance-architecture.md
7. /Users/ruyimg/Documents/Claude/sanguosha/NONAME_NEXT_AGENT_SPEC.md（实施约束；位于 Git 仓库外）

二、开工前核对（2026-11-08 晚间快照，开工时以实际为准）：
- main HEAD `dd05ec203`（纯文档，未推送）；最后代码提交 `81ff9abd1` 已推送 native-convergence，exact-head run 42470608970 补跑后 10/10。规则 333／NGSA 107／content schema 5／ABI 759／WS8。
- 实跑：三 crate 2204/0/7、workspace 7596/0/9。三 crate 剩余 ignore：`upstream_cangchizhibi_target_discards`（裁决 33 待实施）与 6 个手工延迟／AI 质量／状态增长／悬挂引用探针。
- 37 来源已有四端（Web/WASM、Worker、Server、Desktop）产品路径与同设备前后性能数据，但整包八项签署仍为 0/37。
- 真机：无 Android 设备，物理 iPhone `unavailable`。签名证书与公证凭据未配置。
- 8 个临时工作树（1 个 locked）原样保留，处置理由见进度主稿「临时工作树处置」；不删别人的工作树。若发现其他 agent 正在写，先停下协调。

三、执行顺序（各项完成条件以台账对应行为准，完成后在台账行内更新）：
1. 裁决 33 苍螭之璧：把 `cangchizhibi` 补入 `shahai-swd-upstream-noname`（90→91 张），按上游 `extension/杀海拾遗/card/swd/{card,skill}.js` 的 `type: "jiqi"` 与 `g_cangchizhibi` 实现准备阶段持牌选 1–3 名角色横置；先写按上游语义会红的测试，再实现；同步 `package.npack.json` 哈希与锁、`u05_shahai_eight.rs`、`nc08_conversion_gap_regression.rs`、`scripts/report_card_coverage.py`；`nc08_upstream_swd_cangchizhibi.rs` 移除旧转换缺口断言、改写误生成的探针并解除 #[ignore]。核对包锁变化对旧档／内容锁恢复的影响；需要升号时先报边界。
2. 裁决 34 deferred 范围：逐来源清点 Batch 23 §4 表第①维列出的 deferred 渲染面（U01、L03、L07、L08、L09、L12–L15、L17–L18、L20–L23、L26–L29 等）与单机战役脚本（U02、U04、U08、L11 等）及大体量来源的剩余子技／子将，先在扩展台账为每项写出上游功能清单与原生落点（原生前端组件、rule-vm 包或原生规则），再逐项实现；每项须有上游对照的规则差异测试、保存恢复、浏览器真实渲染与帧率数据，不得以数据桥接契约或 deferred 列表收口。按来源分批提交，每批更新扩展台账该来源行。
3. PA09 桌面原生引擎（可在本机 macOS 推进）：按 ADR 0784 与剩余台账 PA09 行六条件逐条补齐同负载对照数据（i2p 与 IPC 往返、AI 时限与线程池收益、同 seed 逐步哈希一致与存档互恢复、扩展包经 IPC 全流程、只在已实测平台切默认、WASM 回退测试）；全部满足才在 macOS 切默认并写 ADR，否则维持 WASM 并回报数据。
4. 37 来源整包八项签署：第 1、2 项完成后，按扩展台账原八项（功能、配置、依赖、组合、保存恢复、目标平台、性能、规则差异）逐来源复核并更新签署表；移动端目标平台待第 5 项。
5. 需要用户提供的外部条件（未就绪时写明所需操作，继续其他项）：PA08 真机（中低端 Android 开启 USB 调试、iPhone 解锁并信任，裁决 19 已同意提供设备）→ 整局与 ≥30 分钟功耗／温度／降频，据此判定 PA11 是否触发，并补 NC12 真机温控与 37 来源移动端目标平台；三平台安装包签名／公证（Apple Developer ID 证书与公证凭据、Windows Authenticode 证书）→ 真实安装验证。
6. 发布：每个代码批完成第五节门禁后按裁决 25 正常发布并独立登记 exact-head CI；首次失败与补跑分开记录。
7. 第六阶段：上述门槛全部成立后完成原生产品切换、安全退出旧执行入口（engine-core／engine-ai／engine-wasm），保留上游来源、正式 ADR、原始证据与仍需的迁移／恢复能力；退出前先报精确范围取得用户确认。

四、工程约束：
- 保留单写 Rust 内核、不可变目录、显式结算/续行、内置 Rust＋第三方 rule-vm 路线；复用 PackageStore 与现有协议/登记工具，不并建第二套包管理、registry 或状态台账；不自动立项 ECS、QuickJS、全面 VM 化、强化学习等历史候选。
- 行数门禁：受维护源码/测试/DSL ≤2,000 行；Rust 非测试源码有效行 ≤1,000。按功能、来源包或项边界拆分并起有意义的名字，不压缩、不机械切块；抽函数时核对 return／?／Ok 控制流。拆文件后同步文档里的 file:line 引用（check_doc_line_refs.py 会扫描历史证据）。
- 分层：engine-native-kernel ← engine-native-content ← engine-native（门面）；check-native-layering.py 棘轮只降不升，内核需要内容行为时加 Catalog 回调。
- 集成测试按分组编译：新增 tests/*.rs 必须登记到 tests/groups/<组>.rs；单跑写 cargo test -p <crate> --test <组> <文件名>::。
- 每项先建立会红的回归，再修共享根因，同步调用方、导出、资源、测试与文档。规则/存档变化核对版本门禁、完整状态、事件、RNG 与续行；golden 变化先查原因再决定重录。
- 不改验收口径来宣告完成：完成条件以台账与扩展台账原文为准；不删改、跳过既有测试来转绿（裁决 33 明确授权的改写除外）。
- 版本号：同一号码只被一批占用；超出已登记授权的规则、存档、ABI、WS、schema 变化或新 ADR，先提交精确边界并取得用户明确答复。
- Windows 检出可能是 CRLF：新测试读包资产时按 LF 归一化（Batch 23 曾因此 CI 失败）；新增前端代码不依赖 Node 专有类型（如 Buffer）。
- 编译后检查磁盘并清理本批不再需要的私有缓存（裁决 23）；不清共享 target，不删别人的工作树。

五、验证（按影响面）：
- 必跑：python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'；python3 -B scripts/check-source-lines.py；cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native；独立 CARGO_TARGET_DIR 下 RUSTFLAGS="-D warnings" cargo check -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets；cargo check --target wasm32-unknown-unknown -p engine-native-wasm。
- 产品面：cargo test --offline --workspace --no-fail-fast；cargo run -p conformance；apps/web-client 的 pnpm test／typecheck／build:native／smoke:native；python3 -B scripts/check-native-wasm-cross-target.py；bash scripts/cross-target-hash.sh；cargo test -p desktop-client；涉及 UI／Worker／渲染的改动跑真实 headless Chrome 产品路径并记录帧率。
- 新增测试须在 CI 同版 rustc（1.99.x）下无 lint 失败。
- 收口：python3 -B scripts/preflight.py 全部通过。首次失败与补跑分开记录，不以补跑抹去失败。

六、协作：可用子 agent 并行处理互不冲突的实现或验证（裁决 34 可按来源分给不同 agent），先划清文件所有权和接口，主 agent 负责整合与最终验收。公共文档写入前重读；不要同时抢写核心文件。在授权范围内自主推进，不反复询问是否继续；缺少设备、凭据、权限或产品语义时，写明所需输入与未验收门槛，继续做不受阻的部分。

七、记录：当前状态只更新进度主稿顶部与剩余台账对应行；来源状态只更新扩展台账；规则差异只更新规则基准；原始命令与输出写入有日期的 evidence 目录。文档统一用简体中文，每条记录写明日期、HEAD 与是否 dirty、规则/存档/ABI/WS 版本、命令、通过/失败/忽略和证据链接；区分已实现、已修反例、已加回归、定向通过、全量通过、上游等价。不得在提交前写“已合入／已提交”；「仓内闭环」不等于整项完成，未达完成条件的行保持「部分」。

最终逐项报告：裁决 33、裁决 34（逐来源）、PA09、37 来源八项签署、PA08／PA11／NC12 真机、安装包签名、发布与 CI、第六阶段的结果与证据，列出仍未完成/未验证项及所需外部条件。只有全部门槛真实通过才可声明“全部完成”；否则明确整体尚未完成。现在先核对工作区，然后从第三节第 1 项开始，并行启动第 2 项的逐来源清点与第 3 项。
```
