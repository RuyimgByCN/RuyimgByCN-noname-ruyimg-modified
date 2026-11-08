# 原生架构收敛完整执行提示词

将下面整段交给执行 agent。方案正文保存需求与验收条件，实施状态以进度主稿顶部扫描和剩余台账为准；本提示词只列工作项、先后与待决事项，不复制一套当前数字。2026-11-08 核对后重写，替换 2026-11-03 版（该版的 R05、C331 余项、I02、UP37 R/N/I 各行、PA12 桥接及 NC 系列仓内项已由 2026-11-05/06 各批入库；核对发现的未达项见下文第二节）。

```text
请在 /Users/ruyimg/Documents/Claude/sanguosha/noname-next 继续完成《原生架构收敛执行方案》剩余工作：实现、集成并验收，清空剩余台账、扩展台账和规则基准里仍未达成的完成条件；不是再写建议、改口径或只跑代表样本；不以时间成本压缩范围。

一、先读（按顺序），并遵守 /Users/ruyimg/Documents/Claude/sanguosha/AGENTS.md：
1. docs/architecture/native-progress.md 顶部「状态扫描（2026-11-08）」及其后 Batch 18–22 记录
2. docs/architecture/native-remaining-ledger.md 顶部「2026-11-08导航」与 NC08、NC12、PA08–PA11、37 来源各行
3. docs/migration/upstream-extension-reconstruction.md 顶部「2026-11-08 核对勘正」、签署表与八项定义
4. docs/operations/evidence/2026-10-28-user-decisions/README.md（裁决 1–32；授权≠实现≠验收）
5. docs/architecture/native-convergence-plan.md（六阶段与验收门槛）、docs/architecture/native-rules-baseline.md、docs/native-coverage-manifest.md、docs/architecture/native-extensions.md、noname-performance-architecture.md
6. /Users/ruyimg/Documents/Claude/sanguosha/NONAME_NEXT_AGENT_SPEC.md（实施约束；位于 Git 仓库外）

二、开工前核对（2026-11-08 快照，开工时以实际为准）：
- main HEAD `4b0ba831a`（Batch 22），规则 333／NGSA 107／content schema 5／ABI 759／WS8。自 `29712b2b` 起 85 个提交，主要由 Antigravity 于 2026-11-05/06 写入（R05、I02、PA12 批⑥–⑰、UP37 批⑱、37 来源批⑲、NC/PA 批⑳、nc08 守卫批㉑㉒）。
- 2026-11-08 实跑：三 crate 2178/0/29；前端 pnpm test 461/0；行数门禁 9187 文件通过；workspace 结果见进度主稿扫描。build:native、Worker smoke、跨目标、desktop、strict Gate 1b／wasm32、preflight、headless Chrome 未由该扫描重跑。
- 与「全部完成」不符、仍为待做的项：
  a. NC08：nc08_registered_defect_guards 19 项与上游探针 4 项仍 #[ignore]（baiyi、cap_card_targeted／death_before／game_start／judge_before／sha_after／sha_before／sha_begin、g19_rexue／zhuhai、cigu、fenglue、fuchou2、gaiming_judge_before、kuwu_and_shengke、mb_phase_before_enter_game、xiangxing、jiguanyuan、lexue；shenenshu、jijiang、cangchizhibi、chenhuodajie_window）。
  b. 37 来源：批⑲「八维」是自定义口径，不含目标平台实机、性能、规则差异三项，第⑧维把 DOM/CSS/WebGL/Spine 渲染面与单机战役脚本保留为 deferred；扩展台账八项整包验收仍未达成。
  c. PA08 真机未开始（2026-11-08 探测：物理 iPhone unavailable，无 Android 设备）；PA09 仍为设计草案、默认 WASM；PA11 条件暂缓；NC12 真机温控与旧线退出未做；三平台安装包签名／公证未做。
  d. 2026-11-03 之后未见按裁决 25 发布与 exact-head CI 登记。
  e. 第六阶段（原生产品切换、旧执行入口退出）未开始。
- 用 git status／git log／git worktree list 复核；仍有 8 个临时工作树（1 个 locked）和 1 个 prunable 登记，处置理由见进度主稿「临时工作树处置」，不删别人的工作树。若发现其他 agent 正在写，先停下协调。

三、执行顺序（各项完成条件以台账对应行为准，完成后在台账行内更新）：
1. 独立复核 2026-11-05/06 各批：抽查语义改动（如批㉒移除 SwdTiebi `t != owner`、Qiankunbiao he 区门、Jiguan 装备入栏、mizhilianou 红桃转桃）与上游源码逐行对照；核对「14 场 golden 哈希零变化」与「3,641/3,641 非占位 .skill 桥接」由可重跑脚本得出；在 HEAD 上重跑并入库产品面全套（见第五节），发现回归先建红测再修。
2. 发布：第 1 项通过后按裁决 25 正常发布并独立登记 exact-head CI；本地通过与 CI 实跑分开记，失败如实登记。
3. NC08 剩余差异：逐项修复第二节 a 的 23 个守卫／探针并解除 #[ignore]；确属上游无法对齐或需产品语义选择的，先报精确边界取得用户答复，不得改记为登记差异来关闭。
4. 37 来源整包八项：按扩展台账原八项（功能、配置、依赖、组合、保存恢复、目标平台、性能、规则差异）逐来源验收；目标平台须在 Web/WASM/Server/Desktop 实际产品路径（可行时含移动端）跑通，性能须有同设备前后数据，规则差异须逐项对照上游；deferred 的渲染面与战役脚本须原生实现，或报用户裁决范围后再登记。
5. 性能与平台：PA08 真机（中低端 Android＋iOS 整局、≥30 分钟功耗/温度/降频；裁决 19 已同意提供设备，未接入时写明需要用户做什么并继续其他项），据此判定 PA11 是否触发；PA09 六条件与默认引擎选择；三平台安装包真实安装、签名／公证。
6. 第六阶段：替代路径与全矩阵验收成立后完成原生产品切换、安全退出旧执行入口（NC12 旧线退出前提），保留上游来源、正式 ADR、原始证据与仍需的迁移/恢复能力。
7. 收尾：清理已无用的临时工作树（先备份 diff 与未跟踪清单并与 main 比对）与私有 target（裁决 23）。

四、工程约束：
- 保留单写 Rust 内核、不可变目录、显式结算/续行、内置 Rust＋第三方 rule-vm 路线；复用 PackageStore 与现有协议/登记工具，不并建第二套包管理、registry 或状态台账；不自动立项 ECS、QuickJS、全面 VM 化、强化学习等历史候选。
- 行数门禁：受维护源码/测试/DSL ≤2,000 行；Rust 非测试源码有效行 ≤1,000。按功能、来源包或项边界拆分并起有意义的名字，不压缩、不机械切块；抽函数时核对 return／?／Ok 控制流。拆文件后同步文档里的 file:line 引用（check_doc_line_refs.py 会扫描历史证据）。
- 分层：engine-native-kernel ← engine-native-content ← engine-native（门面）；check-native-layering.py 棘轮只降不升，内核需要内容行为时加 Catalog 回调。
- 集成测试按分组编译：新增 tests/*.rs 必须登记到 tests/groups/<组>.rs；单跑写 cargo test -p <crate> --test <组> <文件名>::。
- 每项先建立会红的回归，再修共享根因，同步调用方、导出、资源、测试与文档。规则/存档变化核对版本门禁、完整状态、事件、RNG 与续行；golden 变化先查原因再决定重录。
- 不改验收口径来宣告完成：完成条件以台账与扩展台账原文为准，自定义的替代维度只能作为补充证据。
- 版本号：同一号码只被一批占用；超出已登记授权的规则、存档、ABI、WS、schema 变化或新 ADR，先提交精确边界并取得用户明确答复。
- 编译后检查磁盘并清理本批不再需要的私有缓存（裁决 23）；不清共享 target，不删别人的工作树。

五、验证（按影响面）：
- 必跑：python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'；python3 -B scripts/check-source-lines.py；cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native；独立 CARGO_TARGET_DIR 下 RUSTFLAGS="-D warnings" cargo check -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets；cargo check --target wasm32-unknown-unknown -p engine-native-wasm。
- 产品面：cargo test --offline --workspace --no-fail-fast；cargo run -p conformance；apps/web-client 的 pnpm test／typecheck／build:native／smoke:native；python3 -B scripts/check-native-wasm-cross-target.py；bash scripts/cross-target-hash.sh；cargo test -p desktop-client；涉及 UI／Worker 的改动跑真实 headless Chrome 产品路径。
- 新增测试须在 CI 同版 rustc（1.99.x）下无 lint 失败。
- 收口：python3 -B scripts/preflight.py 全部通过。首次失败与补跑分开记录，不以补跑抹去失败。

六、协作：可用子 agent 并行处理互不冲突的实现或验证，先划清文件所有权和接口，主 agent 负责整合与最终验收；第三节第 1 项的复核最好由未参与 2026-11-05/06 实现的 agent 执行。公共文档写入前重读；不要同时抢写核心文件。在授权范围内自主推进，不反复询问是否继续；缺少设备、凭据、权限或产品语义时，写明所需输入与未验收门槛，继续做不受阻的部分。

七、记录：当前状态只更新进度主稿顶部与剩余台账对应行；来源状态只更新扩展台账；规则差异只更新规则基准；原始命令与输出写入有日期的 evidence 目录。文档统一用简体中文，每条记录写明日期、HEAD 与是否 dirty、规则/存档/ABI/WS 版本、命令、通过/失败/忽略和证据链接；区分已实现、已修反例、已加回归、定向通过、全量通过、上游等价。不得在提交前写“已合入／已提交”；「仓内闭环」不等于整项完成，未达完成条件的行保持「部分」。

最终逐项报告：复核结论、发布与 CI、NC08 剩余差异、37 来源八项、PA08–PA11、安装包、第六阶段的结果与证据，列出仍未完成/未验证项。只有全部门槛真实通过才可声明“全部完成”；否则明确整体尚未完成。现在先核对工作区，然后从第三节第 1 项开始执行，并行推进第 3 项。
```
